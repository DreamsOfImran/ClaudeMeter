mod commands;
mod error;
mod polling;
mod state;
mod tray;

use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(Mutex::new(AppState::new())))
        .setup(|app| {
            // Tray-only app: no dock icon on macOS.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Build the tray icon and context menu.
            tray::setup_tray(app)?;

            // Spawn the background polling task.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                polling::start_polling(handle).await;
            });

            // Silent background update check — 15 s after startup.
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                commands::updater::perform_update_check(&update_handle, false).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::usage::get_usage,
            commands::usage::refresh_now,
            commands::usage::set_refresh_interval,
            commands::usage::get_refresh_interval,
            commands::usage::open_login_window,
            commands::usage::sign_out,
            commands::updater::check_update,
            commands::updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
