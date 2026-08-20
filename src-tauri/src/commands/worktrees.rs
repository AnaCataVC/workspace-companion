use crate::services::worktree_cleaner::{RepositoryWorktrees, WorktreeCleanerService};

#[tauri::command]
pub async fn scan_worktrees() -> Result<Vec<RepositoryWorktrees>, String> {
    let repo_paths = WorktreeCleanerService::discover_repositories();
    let mut results = Vec::new();

    for path in repo_paths {
        if let Some(repo_info) = WorktreeCleanerService::scan_repository(path) {
            results.push(repo_info);
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn remove_worktree(
    repo_path: String,
    worktree_path: String,
    force: bool,
) -> Result<String, String> {
    WorktreeCleanerService::remove_worktree(&repo_path, &worktree_path, force)
}

#[tauri::command]
pub async fn prune_worktrees(repo_path: String) -> Result<String, String> {
    WorktreeCleanerService::prune_worktrees(&repo_path)
}

#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open path: {}", e))
}
