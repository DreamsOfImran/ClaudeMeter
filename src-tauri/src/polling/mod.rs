use crate::commands::usage::fetch_usage_from_session;
use crate::state::{AppState, PollingCommand};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Start the background polling loop.
///
/// Spawned once at startup. Receives `PollingCommand` messages via an mpsc
/// channel to change the interval, trigger an immediate refresh, or stop.
pub async fn start_polling(app: AppHandle) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PollingCommand>(32);
    {
        let state = app.state::<Arc<Mutex<AppState>>>();
        let mut guard = state.lock().await;
        guard.polling_tx = Some(tx);
    }

    let mut interval_secs: u64 = {
        let state = app.state::<Arc<Mutex<AppState>>>();
        let secs = state.lock().await.refresh_interval_secs;
        secs
    };

    // Kick off an initial refresh immediately.
    perform_refresh(&app).await;

    let mut next_tick = Instant::now() + Duration::from_secs(interval_secs);

    loop {
        let remaining = next_tick.saturating_duration_since(Instant::now());

        tokio::select! {
            () = tokio::time::sleep(remaining) => {
                perform_refresh(&app).await;
                next_tick = Instant::now() + Duration::from_secs(interval_secs);
            }

            Some(cmd) = rx.recv() => {
                match cmd {
                    PollingCommand::SetInterval(secs) => {
                        interval_secs = secs;
                        next_tick = Instant::now() + Duration::from_secs(secs);
                        log::info!("Polling interval changed to {}s", secs);
                    }
                    PollingCommand::RefreshNow => {
                        perform_refresh(&app).await;
                        next_tick = Instant::now() + Duration::from_secs(interval_secs);
                    }
                    PollingCommand::Stop => {
                        log::info!("Polling task stopping");
                        break;
                    }
                }
            }
        }
    }
}

async fn perform_refresh(app: &AppHandle) {
    let state = app.state::<Arc<Mutex<AppState>>>();

    match fetch_usage_from_session(app).await {
        Ok(usage) => {
            // Update tray label (session %) and icon color.
            let label = tray_label(&usage);
            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_title(Some(&label));
            }
            crate::tray::update_icon_for_percent(app, usage.session_percent);
            crate::tray::update_auth_menu_item(app, usage.is_logged_in);

            {
                let mut guard = state.lock().await;
                guard.usage = Some(usage.clone());
            }
            let _ = app.emit("usage-updated", &usage);
            log::debug!("Usage refreshed: {}", label);
        }
        Err(err) => {
            {
                let mut guard = state.lock().await;
                if let Some(ref mut u) = guard.usage {
                    u.error = Some(err.clone());
                }
            }
            let _ = app.emit("usage-error", &err);
            log::warn!("Usage refresh failed: {}", err);
        }
    }
}

/// Pick the most representative usage % for the tray label.
fn tray_label(u: &crate::state::UsageData) -> String {
    if !u.is_logged_in {
        return "––".to_string();
    }
    // Show the current session percentage; fall back to weekly all-models.
    let pct = u
        .session_percent
        .or(u.weekly_all_models_percent)
        .unwrap_or(0);
    format!("{}%", pct)
}
