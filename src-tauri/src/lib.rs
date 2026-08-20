pub mod commands;
pub mod services;
pub mod tray;

use commands::gh_auth::{get_gh_accounts, switch_gh_account};
use commands::worktrees::{open_path, prune_worktrees, remove_worktree, scan_worktrees};

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
            scan_worktrees,
            remove_worktree,
            prune_worktrees,
            open_path,
            get_gh_accounts,
            switch_gh_account
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
