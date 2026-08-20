use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
        } else {
            show_main_window(app);
        }
    }
}

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_item = MenuItem::with_id(app, "open", "Open Workspace Companion", true, None::<&str>)?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Worktrees", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Workspace Companion", true, None::<&str>)?;
    let tray_menu = Menu::with_items(app, &[&open_item, &refresh_item, &quit_item])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .tooltip("Workspace Companion - Git Worktrees & GH Accounts");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                show_main_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            "refresh" => {
                let _ = app.emit("refresh-worktrees", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    toggle_main_window(tray.app_handle());
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
