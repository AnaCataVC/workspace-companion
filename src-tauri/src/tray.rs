use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewWindow,
};
use tauri_plugin_positioner::{Position, WindowExt};

/// A blur-hide this recent is treated as the same gesture as the tray click that caused it.
/// Clicking the tray icon steals focus from the panel before the click event arrives, so without
/// this grace window a "hide" click would hide on blur and immediately re-show on click.
const BLUR_TOGGLE_GRACE: Duration = Duration::from_millis(300);

/// Panel visibility policy shared between the window event handler, the tray and the shortcut.
/// `pinned` and `busy` are pushed by the frontend through `set_panel_state`.
#[derive(Default)]
pub struct PanelState {
    pinned: AtomicBool,
    busy: AtomicBool,
    last_blur_hide: Mutex<Option<Instant>>,
}

impl PanelState {
    pub fn set(&self, pinned: bool, busy: bool) {
        self.pinned.store(pinned, Ordering::SeqCst);
        self.busy.store(busy, Ordering::SeqCst);
    }

    fn should_hide_on_blur(&self) -> bool {
        should_hide_on_blur(
            self.pinned.load(Ordering::SeqCst),
            self.busy.load(Ordering::SeqCst),
        )
    }

    fn mark_blur_hide(&self, at: Instant) {
        if let Ok(mut last) = self.last_blur_hide.lock() {
            *last = Some(at);
        }
    }

    fn take_recent_blur_hide(&self, now: Instant) -> bool {
        self.last_blur_hide
            .lock()
            .map(|mut last| is_within_grace(last.take(), now))
            .unwrap_or(false)
    }
}

/// The panel auto-hides on focus loss only when the user has not pinned it and no modal or
/// long-running action is in progress (a blur mid-action would hide its progress and result).
pub fn should_hide_on_blur(pinned: bool, busy: bool) -> bool {
    !pinned && !busy
}

fn is_within_grace(last_hide: Option<Instant>, now: Instant) -> bool {
    last_hide.is_some_and(|at| now.saturating_duration_since(at) < BLUR_TOGGLE_GRACE)
}

/// Hides the panel on focus loss when the panel policy allows it.
pub fn handle_focus_lost<R: Runtime>(window: &tauri::Window<R>) {
    let state = window.state::<PanelState>();
    if state.should_hide_on_blur() {
        state.mark_blur_hide(Instant::now());
        let _ = window.hide();
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
    if app
        .state::<PanelState>()
        .take_recent_blur_hide(Instant::now())
    {
        return;
    }
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
    fn hides_on_blur_only_when_unpinned_and_idle() {
        assert!(should_hide_on_blur(false, false));
        assert!(!should_hide_on_blur(true, false));
        assert!(!should_hide_on_blur(false, true));
        assert!(!should_hide_on_blur(true, true));
    }

    #[test]
    fn blur_hide_inside_grace_swallows_the_toggle() {
        let hidden_at = Instant::now();
        assert!(is_within_grace(
            Some(hidden_at),
            hidden_at + Duration::from_millis(100)
        ));
        assert!(!is_within_grace(
            Some(hidden_at),
            hidden_at + BLUR_TOGGLE_GRACE
        ));
        assert!(!is_within_grace(None, hidden_at));
    }

    #[test]
    fn recent_blur_hide_is_consumed_once() {
        let state = PanelState::default();
        let now = Instant::now();
        state.mark_blur_hide(now);
        assert!(state.take_recent_blur_hide(now));
        assert!(!state.take_recent_blur_hide(now));
    }
}
