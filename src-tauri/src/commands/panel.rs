use crate::tray::{hide_main_window, PanelState};
use tauri::{AppHandle, Manager, State};

/// Syncs the frontend's pin toggle and "a modal or action is in progress" flag into the
/// auto-hide policy, and keeps the pinned panel above other windows.
#[tauri::command]
pub fn set_panel_state(
    app: AppHandle,
    state: State<'_, PanelState>,
    pinned: bool,
    busy: bool,
) -> Result<(), String> {
    state.set(pinned, busy);
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(pinned)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) {
    hide_main_window(&app);
}
