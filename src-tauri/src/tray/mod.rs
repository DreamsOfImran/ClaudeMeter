/// Tray icon setup for ClaudeMeter.
///
/// Three colored icon variants are embedded at compile time so no resource
/// path look-up is needed at runtime:
///   green  → session < 70 %
///   orange → session 70–90 %
///   red    → session > 90 %
///
/// Platform notes:
///   macOS   — NSStatusItem. `icon_as_template` is NOT set so the OS renders
///             the real colors instead of a monochrome mask.
///   Windows — system-tray notification-area; same API.
///   Linux   — requires libappindicator3-dev on the build host.
use crate::state::{AppState, PollingCommand};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewWindow,
};
use tokio::sync::Mutex;

// Embed PNG bytes at compile time — no runtime resource path needed.
const ICON_GREEN_PNG: &[u8] = include_bytes!("../../icons/tray-green.png");
const ICON_ORANGE_PNG: &[u8] = include_bytes!("../../icons/tray-orange.png");
const ICON_RED_PNG: &[u8] = include_bytes!("../../icons/tray-red.png");

/// Decode a PNG byte slice into a Tauri `Image` (RGBA, owned).
fn decode_icon(png_bytes: &[u8]) -> Option<Image<'static>> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => info.buffer_size().checked_div(3).map(|n| {
            buf[..info.buffer_size()]
                .chunks(3)
                .fold(Vec::with_capacity(n * 4), |mut v, c| {
                    v.extend_from_slice(c);
                    v.push(255);
                    v
                })
        })?,
        _ => return None,
    };
    Some(Image::new_owned(rgba, info.width, info.height))
}

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let auth_i = MenuItem::with_id(app, "auth", "Sign In", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?;
    let update_i = MenuItem::with_id(app, "update", "Check for Update", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit ClaudeMeter", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&auth_i, &refresh_i, &update_i, &sep, &quit_i])?;

    // Start with the green icon; it will be updated after the first fetch.
    let initial_icon = decode_icon(ICON_GREEN_PNG)
        .unwrap_or_else(|| app.default_window_icon().unwrap().clone());

    TrayIconBuilder::with_id("main-tray")
        .icon(initial_icon)
        // Do NOT call icon_as_template(true) — we want the real colors to show.
        // Set an initial title so macOS initialises imagePosition = NSImageLeft
        // from the start, which keeps icon and text vertically centred together.
        .title("––")
        .tooltip("ClaudeMeter")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                let (pos_x, pos_y) = match rect.position {
                    tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                    tauri::Position::Logical(p) => (p.x, p.y),
                };
                let (width, height) = match rect.size {
                    tauri::Size::Physical(s) => (s.width as f64, s.height as f64),
                    tauri::Size::Logical(s) => (s.width, s.height),
                };
                show_window_at_tray(app, pos_x, pos_y, width, height);
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "auth" => {
                let state: Arc<Mutex<AppState>> =
                    app.state::<Arc<Mutex<AppState>>>().inner().clone();
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let is_logged_in = {
                        let guard = state.lock().await;
                        guard.usage.as_ref().map(|u| u.is_logged_in).unwrap_or(false)
                    };
                    if is_logged_in {
                        // Sign out: navigate to logout URL then reset state.
                        if let Some(wv) = app_clone.get_webview_window(
                            crate::commands::usage::CLAUDE_WEBVIEW_LABEL,
                        ) {
                            let _ = wv
                                .eval("window.location.href = 'https://claude.ai/logout';");
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            let _ = wv.hide();
                        }
                        let logout_data = crate::state::UsageData {
                            is_logged_in: false,
                            ..Default::default()
                        };
                        {
                            let mut guard = state.lock().await;
                            guard.usage = Some(logout_data.clone());
                        }
                        let _ = app_clone.emit("usage-updated", &logout_data);
                        if let Some(tray) = app_clone.tray_by_id("main-tray") {
                            let _ = tray.set_title(Some("––"));
                        }
                        update_auth_menu_item(&app_clone, false);
                        update_icon_for_percent(&app_clone, None);
                    } else {
                        let _ =
                            crate::commands::usage::open_login_window(app_clone).await;
                    }
                });
            }
            "refresh" => {
                let state: Arc<Mutex<AppState>> =
                    app.state::<Arc<Mutex<AppState>>>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    let guard = state.lock().await;
                    if let Some(tx) = &guard.polling_tx {
                        let _ = tx.send(PollingCommand::RefreshNow).await;
                    }
                });
            }
            "update" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    crate::commands::updater::perform_update_check(&app_clone, true).await;
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Update the "Sign In" / "Sign Out" menu item by rebuilding the tray menu.
/// (Tauri v2 `TrayIcon` has no `menu()` getter, only `set_menu()`.)
pub fn update_auth_menu_item(app: &AppHandle, is_logged_in: bool) {
    let auth_text = if is_logged_in { "Sign Out" } else { "Sign In" };
    let Ok(auth_i) = MenuItem::with_id(app, "auth", auth_text, true, None::<&str>) else {
        return;
    };
    let Ok(refresh_i) = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)
    else {
        return;
    };
    let Ok(update_i) =
        MenuItem::with_id(app, "update", "Check for Update", true, None::<&str>)
    else {
        return;
    };
    let Ok(sep) = PredefinedMenuItem::separator(app) else {
        return;
    };
    let Ok(quit_i) = MenuItem::with_id(app, "quit", "Quit ClaudeMeter", true, None::<&str>)
    else {
        return;
    };
    if let Ok(menu) = Menu::with_items(app, &[&auth_i, &refresh_i, &update_i, &sep, &quit_i]) {
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/// Update the tray icon color based on the current session usage percentage.
///
///   < 70 % → green
///   70–90 % → orange
///   > 90 % → red
pub fn update_icon_for_percent(app: &AppHandle, session_percent: Option<i32>) {
    let pct = session_percent.unwrap_or(0);
    let png: &[u8] = if pct >= 90 {
        ICON_RED_PNG
    } else if pct >= 70 {
        ICON_ORANGE_PNG
    } else {
        ICON_GREEN_PNG
    };

    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Some(icon) = decode_icon(png) {
            let _ = tray.set_icon(Some(icon));
        }
    }
}

// ---------------------------------------------------------------------------
// Window positioning
// ---------------------------------------------------------------------------

pub fn show_window_at_tray(
    app: &AppHandle,
    tray_x: f64,
    tray_y: f64,
    tray_width: f64,
    tray_height: f64,
) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    if tray_width > 0.0 {
        position_below_tray(&window, tray_x, tray_y, tray_width, tray_height);
    } else {
        position_fallback(&window);
    }

    let _ = window.show();
    let _ = window.set_focus();
}

fn position_below_tray(
    window: &WebviewWindow,
    tray_x: f64,
    tray_y: f64,
    tray_width: f64,
    tray_height: f64,
) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let tray_x_l = tray_x / scale;
    let tray_y_l = tray_y / scale;
    let tray_w_l = tray_width / scale;
    let tray_h_l = tray_height / scale;
    let window_width = 300.0_f64;

    let pos_x = tray_x_l + (tray_w_l / 2.0) - (window_width / 2.0);
    let pos_y = tray_y_l + tray_h_l + 5.0;

    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
        pos_x.max(0.0),
        pos_y,
    )));
}

fn position_fallback(window: &WebviewWindow) {
    let scale = window.scale_factor().unwrap_or(1.0);
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen = monitor.size();
        let win = window.outer_size().unwrap_or_default();
        let x = (screen.width as f64 - win.width as f64) / scale - 16.0;
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            x.max(0.0),
            32.0,
        )));
    }
}
