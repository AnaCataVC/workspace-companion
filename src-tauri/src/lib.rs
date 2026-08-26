pub mod commands;
pub mod services;
pub mod tray;

use commands::gh_auth::{get_gh_accounts, switch_gh_account};
use commands::worktrees::{
    checkout_worktree_branch, create_worktree, detect_installed_editors, get_app_config,
    get_worktree_diff_summary, list_branches, open_in_editor, open_path, prune_worktrees,
    remove_worktree, save_app_config, scan_worktrees, suggest_worktree_path,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            // Setup Tray Icon
            tray::create_tray(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            save_app_config,
            scan_worktrees,
            get_worktree_diff_summary,
            remove_worktree,
            prune_worktrees,
            open_path,
            open_in_editor,
            detect_installed_editors,
            list_branches,
            checkout_worktree_branch,
            suggest_worktree_path,
            create_worktree,
            get_gh_accounts,
            switch_gh_account
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
