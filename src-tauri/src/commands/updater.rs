use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// Payload emitted when an update is available.
#[derive(serde::Serialize, Clone)]
pub struct UpdateAvailablePayload {
    pub version: String,
    pub body: Option<String>,
}

/// Check for an update and emit the result to the frontend.
///
/// - `is_manual` — if true, also emits `update-not-available` when already up-to-date
///   so the user gets feedback after clicking "Check for Update".
pub async fn perform_update_check(app: &AppHandle, is_manual: bool) {
    let updater = match app.updater_builder().build() {
        Ok(u) => u,
        Err(e) => {
            if is_manual {
                let _ = app.emit("update-error", e.to_string());
            }
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let _ = app.emit(
                "update-available",
                UpdateAvailablePayload {
                    version: update.version.clone(),
                    body: update.body.clone(),
                },
            );
        }
        Ok(None) => {
            if is_manual {
                let _ = app.emit("update-not-available", ());
            }
        }
        Err(e) => {
            if is_manual {
                let _ = app.emit("update-error", e.to_string());
            }
        }
    }
}

/// Tauri command: check for an update manually (called from the tray menu).
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<(), String> {
    perform_update_check(&app, true).await;
    Ok(())
}

/// Tauri command: download and install the latest update, then restart.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app
        .updater_builder()
        .build()
        .map_err(|e| e.to_string())?;

    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Err("No update available".into());
    };

    let app_clone = app.clone();
    update
        .download_and_install(
            move |downloaded, total| {
                if let Some(total) = total {
                    let pct = (downloaded as f64 / total as f64 * 100.0) as u8;
                    let _ = app_clone.emit("update-progress", pct);
                }
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}
