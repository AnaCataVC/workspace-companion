use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewWindow,
};
use tauri_plugin_positioner::{Position, WindowExt};

/// Panel state shared between the commands, the tray and the shortcut.
/// `pinned` keeps the window always on top.
#[derive(Default)]
pub struct PanelState {
    pinned: AtomicBool,
    busy: AtomicBool,
}

impl PanelState {
    pub fn set(&self, pinned: bool, busy: bool) {
        self.pinned.store(pinned, Ordering::SeqCst);
        self.busy.store(busy, Ordering::SeqCst);
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }
}

fn position_near_tray<R: Runtime>(window: &WebviewWindow<R>) {
    // The tray position is only known after the first tray event; before that, center it.
    if window
        .move_window_constrained(Position::TrayCenter)
        .is_err()
    {
        let _ = window.move_window(Position::Center);
    }
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        position_near_tray(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_main_window(app);
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
        .tooltip("Workspace Companion - Git Worktrees & GH Accounts (Alt+Space)");

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

            // Only the single-click Up toggles: a double-click also delivers two Click events,
            // so reacting to DoubleClick too would toggle three times and leave the panel hidden.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_state_tracks_pinned_and_busy() {
        let state = PanelState::default();
        assert!(!state.is_pinned());
        assert!(!state.is_busy());

        state.set(true, false);
        assert!(state.is_pinned());
        assert!(!state.is_busy());

        state.set(false, true);
        assert!(!state.is_pinned());
        assert!(state.is_busy());
    }
}
