pub mod commands;
pub mod services;
pub mod tray;

use commands::branches::{remove_branches_batch, scan_branches_for_cleanup};
use commands::gh_auth::{get_gh_accounts, switch_gh_account};
use commands::panel::{hide_panel, set_panel_state};
use commands::worktrees::{
    checkout_worktree_branch, create_worktree, detach_worktree_head, detect_installed_editors,
    get_app_config, get_worktree_diff_summary, git_discard_worktree_changes, git_stash_worktree,
    git_unlock_worktree, is_worktree_target_occupied, list_branches, open_in_editor,
    open_in_terminal, open_path, prune_worktrees, remove_worktree, remove_worktrees_batch,
    save_app_config, scan_worktrees, suggest_worktree_path,
};
use tauri::WindowEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Global shortcut that toggles the panel from anywhere, like the tray icon click.
const TOGGLE_SHORTCUT: &str = "Alt+Space";

fn register_toggle_shortcut(app: &tauri::AppHandle) {
    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                tray::toggle_main_window(app);
            }
        })
        .build();
    // Another app may already own the chord; the tray still works, so this is not fatal.
    if let Err(e) = app.plugin(plugin) {
        eprintln!("Failed to initialize global shortcut plugin: {e}");
        return;
    }
    if let Err(e) = app.global_shortcut().register(TOGGLE_SHORTCUT) {
        eprintln!("Failed to register {TOGGLE_SHORTCUT}: {e}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .manage(tray::PanelState::default())
        .setup(|app| {
            tray::create_tray(app.handle())?;
            register_toggle_shortcut(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // The panel lives in the tray: the window's X hides it, only the tray menu quits.
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Focused(false) => tray::handle_focus_lost(window),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            save_app_config,
            scan_worktrees,
            get_worktree_diff_summary,
            remove_worktree,
            remove_worktrees_batch,
            prune_worktrees,
            open_path,
            open_in_editor,
            open_in_terminal,
            detect_installed_editors,
            list_branches,
            checkout_worktree_branch,
            detach_worktree_head,
            suggest_worktree_path,
            create_worktree,
            is_worktree_target_occupied,
            git_stash_worktree,
            git_discard_worktree_changes,
            git_unlock_worktree,
            scan_branches_for_cleanup,
            remove_branches_batch,
            get_gh_accounts,
            switch_gh_account,
            set_panel_state,
            hide_panel
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
