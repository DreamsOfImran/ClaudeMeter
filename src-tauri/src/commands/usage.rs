use crate::state::{AppState, PollingCommand, UsageData};
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri::webview::WebviewWindowBuilder;
use tokio::sync::Mutex;

type SharedState = Arc<Mutex<AppState>>;

/// Label for the hidden persistent Claude session WebView.
pub const CLAUDE_WEBVIEW_LABEL: &str = "claude-session";

// ---------------------------------------------------------------------------
// Public response type
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct UsageResponse {
    pub data: Option<UsageData>,
    pub last_refresh: Option<String>,
}

// ---------------------------------------------------------------------------
// WebView helpers
// ---------------------------------------------------------------------------

/// Return the existing Claude session WebView or create a new hidden one.
fn get_or_create_claude_webview<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
) -> Result<tauri::WebviewWindow<R>, String> {
    if let Some(existing) = app.get_webview_window(CLAUDE_WEBVIEW_LABEL) {
        existing
            .eval(&format!("window.location.href = '{}';", url))
            .map_err(|e| format!("Failed to navigate: {}", e))?;
        Ok(existing)
    } else {
        WebviewWindowBuilder::new(
            app,
            CLAUDE_WEBVIEW_LABEL,
            tauri::WebviewUrl::External(url.parse().unwrap()),
        )
        .title("Claude")
        .inner_size(520.0, 740.0)
        .center()
        .visible(false)
        .skip_taskbar(true)
        .build()
        .map_err(|e| format!("Failed to create webview: {}", e))
    }
}

// ---------------------------------------------------------------------------
// Core fetch logic (shared by the command and the polling task)
// ---------------------------------------------------------------------------

/// Navigate the hidden WebView to claude.ai/settings/usage, wait for it to
/// load, inject the extraction script, and return structured UsageData.
///
/// Takes ~4–10 s on first call (page load). Subsequent calls reuse the same
/// WebView so they are faster.
pub async fn fetch_usage_from_session<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<UsageData, String> {
    let webview = get_or_create_claude_webview(app, "https://claude.ai/settings/usage")?;
    let _ = webview.hide(); // keep it invisible while scraping

    // Wait for the page to load fully.
    tokio::time::sleep(Duration::from_secs(4)).await;

    // If redirected to login, the user is not authenticated.
    let url = webview
        .url()
        .map_err(|e| format!("Failed to get URL: {}", e))?;
    let url_str = url.as_str();

    if url_str.contains("/login") || url_str.contains("accounts.google") {
        return Ok(UsageData {
            is_logged_in: false,
            error: Some("Not signed in to Claude.".to_string()),
            last_updated: Some(Utc::now()),
            ..Default::default()
        });
    }

    // Inject the extraction script. Because window.__TAURI__ is not available
    // on external pages, we encode the result in the URL hash and poll it.
    let extraction_script = r#"
        (function() {
            function extractUsageData() {
                const data = {
                    sessionPercent: null,
                    sessionResetTime: null,
                    weeklyAllModelsPercent: null,
                    weeklyAllModelsReset: null,
                    weeklySonnetPercent: null,
                    weeklySonnetReset: null,
                    extraSpent: null,
                    extraLimit: null,
                    extraBalance: null,
                    extraPercent: null,
                    extraReset: null,
                    isLoggedIn: true,
                    error: null
                };

                function findSectionByText(text) {
                    const allElements = document.querySelectorAll('*');
                    for (const el of allElements) {
                        if (el.childNodes.length === 1 &&
                            el.textContent &&
                            el.textContent.trim() === text) {
                            let parent = el.parentElement;
                            for (let i = 0; i < 5 && parent; i++) {
                                if (parent.textContent && parent.textContent.includes('%')) {
                                    return parent;
                                }
                                parent = parent.parentElement;
                            }
                            return el.parentElement?.parentElement;
                        }
                    }
                    return null;
                }

                function extractPercent(container) {
                    if (!container) return null;
                    const text = container.textContent || '';
                    const match = text.match(/(\d+)%\s*used/);
                    return match ? parseInt(match[1], 10) : null;
                }

                function extractResetTime(container) {
                    if (!container) return null;
                    const text = container.textContent || '';
                    const match = text.match(/Resets?\s+(?:in\s+)?([^\n]+)/i);
                    if (match) {
                        let time = match[1].trim();
                        time = time.replace(/\s*(\d+%|used|Learn more).*$/i, '').trim();
                        return time;
                    }
                    return null;
                }

                const sessionSection = findSectionByText('Current session');
                if (sessionSection) {
                    data.sessionPercent = extractPercent(sessionSection);
                    data.sessionResetTime = extractResetTime(sessionSection);
                }

                const allModelsSection = findSectionByText('All models');
                if (allModelsSection) {
                    data.weeklyAllModelsPercent = extractPercent(allModelsSection);
                    data.weeklyAllModelsReset = extractResetTime(allModelsSection);
                }

                const sonnetSection = findSectionByText('Sonnet only');
                if (sonnetSection) {
                    data.weeklySonnetPercent = extractPercent(sonnetSection);
                    data.weeklySonnetReset = extractResetTime(sonnetSection);
                }

                const extraSection = findSectionByText('Extra usage');
                if (extraSection) {
                    const extraText = extraSection.textContent || '';
                    data.extraPercent = extractPercent(extraSection);
                    data.extraReset = extractResetTime(extraSection);
                    const spentMatch = extraText.match(/[€$£]([\d.]+)\s*spent/);
                    data.extraSpent = spentMatch ? parseFloat(spentMatch[1]) : null;
                    const limitMatch = extraText.match(/[€$£](\d+).*Monthly spending limit/);
                    data.extraLimit = limitMatch ? parseFloat(limitMatch[1]) : null;
                    const balanceMatch = extraText.match(/[€$£]([\d.]+).*Current balance/);
                    data.extraBalance = balanceMatch ? parseFloat(balanceMatch[1]) : null;
                }

                return data;
            }

            const extractedData = extractUsageData();
            window.location.hash = 'TAURI_RESULT:' + encodeURIComponent(JSON.stringify(extractedData));
        })();
    "#;

    webview
        .eval(extraction_script)
        .map_err(|e| format!("Failed to execute JS: {}", e))?;

    // Poll the URL hash for the result (up to 20 × 500 ms = 10 s).
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(500)).await;

        if let Ok(url) = webview.url() {
            let url_str = url.as_str();
            if let Some(hash_pos) = url_str.find("#TAURI_RESULT:") {
                let encoded = &url_str[hash_pos + 14..];
                if let Ok(decoded) = urlencoding::decode(encoded) {
                    let _ = webview.eval("window.location.hash = '';");
                    return match serde_json::from_str::<UsageData>(&decoded) {
                        Ok(mut data) => {
                            data.last_updated = Some(Utc::now());
                            Ok(data)
                        }
                        Err(e) => Ok(UsageData {
                            error: Some(format!("Parse error: {}", e)),
                            is_logged_in: true,
                            last_updated: Some(Utc::now()),
                            ..Default::default()
                        }),
                    };
                }
            }
        }
    }

    Err("Timeout: usage data not extracted within 10 s".to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Return the current cached usage snapshot. Does not hit the web.
#[tauri::command]
pub async fn get_usage(state: State<'_, SharedState>) -> Result<UsageResponse, String> {
    let guard = state.lock().await;
    Ok(UsageResponse {
        last_refresh: guard
            .usage
            .as_ref()
            .and_then(|u| u.last_updated)
            .map(|t| t.to_rfc3339()),
        data: guard.usage.clone(),
    })
}

/// Trigger an immediate refresh (asks the polling task to fetch now).
#[tauri::command]
pub async fn refresh_now(state: State<'_, SharedState>) -> Result<(), String> {
    let guard = state.lock().await;
    if let Some(tx) = &guard.polling_tx {
        tx.send(PollingCommand::RefreshNow)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Change the background polling interval and restart the timer.
#[tauri::command]
pub async fn set_refresh_interval(
    seconds: u64,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    if seconds < 30 {
        return Err("Minimum refresh interval is 30 seconds".into());
    }
    let mut guard = state.lock().await;
    guard.refresh_interval_secs = seconds;
    if let Some(tx) = &guard.polling_tx {
        let _ = tx.send(PollingCommand::SetInterval(seconds)).await;
    }
    Ok(())
}

/// Return the current polling interval in seconds.
#[tauri::command]
pub async fn get_refresh_interval(state: State<'_, SharedState>) -> Result<u64, String> {
    Ok(state.lock().await.refresh_interval_secs)
}

/// Sign the current user out by navigating to the logout page and resetting state.
#[tauri::command]
pub async fn sign_out(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    // Navigate the hidden session WebView to the logout URL.
    if let Some(wv) = app.get_webview_window(CLAUDE_WEBVIEW_LABEL) {
        let _ = wv.eval("window.location.href = 'https://claude.ai/logout';");
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = wv.hide();
    }

    let logout_data = UsageData {
        is_logged_in: false,
        last_updated: Some(Utc::now()),
        ..Default::default()
    };
    {
        let mut guard = state.lock().await;
        guard.usage = Some(logout_data.clone());
    }
    let _ = app.emit("usage-updated", &logout_data);

    Ok(())
}

/// Open the claude.ai login page in the hidden WebView and watch for
/// successful authentication (URL leaves the /login path).
/// Emits `login-success` to the main window when done.
#[tauri::command]
pub async fn open_login_window(app: AppHandle) -> Result<(), String> {
    let webview = get_or_create_claude_webview(&app, "https://claude.ai/login")?;
    webview
        .show()
        .map_err(|e| format!("Failed to show login window: {}", e))?;
    webview
        .set_focus()
        .map_err(|e| format!("Failed to focus login window: {}", e))?;

    // Spawn a watcher that polls the URL every 2 s until login is detected.
    let app_clone = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let Some(wv) = app_clone.get_webview_window(CLAUDE_WEBVIEW_LABEL) else {
                break;
            };

            // User may have closed the login window manually.
            if !wv.is_visible().unwrap_or(false) {
                break;
            }

            if let Ok(url) = wv.url() {
                let s = url.as_str();
                if s.contains("claude.ai")
                    && !s.contains("/login")
                    && !s.contains("accounts.google")
                {
                    let _ = wv.hide();
                    if let Some(main) = app_clone.get_webview_window("main") {
                        let _ = main.emit("login-success", ());
                    }
                    break;
                }
            }
        }
    });

    Ok(())
}
