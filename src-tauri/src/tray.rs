use crate::state::{AppState, lock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

const MAIN_WINDOW: &str = "main";
const HIDDEN_ARGUMENT: &str = "--hidden";

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Ouvrir Konsol", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut builder = TrayIconBuilder::with_id("konsol")
        .tooltip("Konsol")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;

    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        if std::env::args().any(|argument| argument == HIDDEN_ARGUMENT) {
            let _ = window.hide();
        }
        let handle = app.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = handle.state::<AppState>();
                let keep_running = lock(&state.config)
                    .load()
                    .is_ok_and(|config| config.preferences.keep_running_in_background);
                if keep_running && let Some(window) = handle.get_webview_window(MAIN_WINDOW) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        });
    }
    Ok(())
}

pub fn autostart_arguments() -> &'static str {
    HIDDEN_ARGUMENT
}
