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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::usage::get_usage,
            commands::usage::refresh_now,
            commands::usage::set_refresh_interval,
            commands::usage::get_refresh_interval,
            commands::usage::open_login_window,
            commands::usage::sign_out,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
