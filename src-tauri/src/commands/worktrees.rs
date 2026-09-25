use crate::services::config::{AppConfig, ConfigService};
use crate::services::git::{
    CheckoutBranchResult, CreateWorktreeResult, EditorInfo, GitService, SuggestWorktreePathResult,
    WorktreeBranchesResponse, WorktreeDiffSummary, WorktreeEntry,
};
use crate::services::worktree_cleaner::{
    BatchDeleteSummary, BatchDeleteTarget, RepositoryWorktrees, WorktreeCleanerService,
};
use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Every scan event carries the id the frontend passed to `scan_worktrees`, so events from a
/// scan that a newer refresh superseded can be told apart and ignored.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RepoScannedEvent<'a> {
    scan_id: u64,
    repo: &'a RepositoryWorktrees,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RepoScanFailedEvent {
    scan_id: u64,
    repo_path: String,
    error: String,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct ScanCompleteEvent {
    scan_id: u64,
}

#[tauri::command]
pub async fn get_worktree_diff_summary(
    worktree_path: String,
) -> Result<WorktreeDiffSummary, String> {
    tauri::async_runtime::spawn_blocking(move || GitService::get_diff_summary(&worktree_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_app_config() -> Result<AppConfig, String> {
    Ok(ConfigService::load_config())
}

#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<AppConfig, String> {
    ConfigService::save_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub async fn scan_worktrees(app: AppHandle, scan_id: u64) -> Result<(), String> {
    let config = ConfigService::load_config();
    let discovered = WorktreeCleanerService::discover_repositories(&config);

    tauri::async_runtime::spawn_blocking(move || {
        discovered.par_iter().for_each(|repo| {
            match WorktreeCleanerService::scan_repository(
                &repo.path,
                repo.associated_account.clone(),
                repo.watch_folder_path.clone(),
            ) {
                Ok(Some(repo_info)) => {
                    let _ = app.emit(
                        "repo-scanned",
                        RepoScannedEvent {
                            scan_id,
                            repo: &repo_info,
                        },
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    let _ = app.emit(
                        "repo-scan-failed",
                        RepoScanFailedEvent {
                            scan_id,
                            repo_path: repo.path.to_string_lossy().to_string(),
                            error,
                        },
                    );
                }
            }
        });

        let _ = app.emit("scan-complete", ScanCompleteEvent { scan_id });
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_worktree(
    repo_path: String,
    worktree_path: String,
    force: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        WorktreeCleanerService::remove_worktree(&repo_path, &worktree_path, force)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn remove_worktrees_batch(
    targets: Vec<BatchDeleteTarget>,
) -> Result<BatchDeleteSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        WorktreeCleanerService::remove_worktrees_batch(targets)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prune_worktrees(repo_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        WorktreeCleanerService::prune_worktrees(&repo_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_path(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("Failed to open path: {}", e))
}

#[tauri::command]
pub async fn open_in_editor(editor: String, path: String) -> Result<(), String> {
    GitService::open_in_editor(&editor, &path)
}

#[tauri::command]
pub async fn open_in_terminal(terminal: String, path: String) -> Result<(), String> {
    GitService::open_in_terminal(&terminal, &path)
}

#[tauri::command]
pub async fn detect_installed_editors() -> Result<Vec<EditorInfo>, String> {
    Ok(GitService::detect_installed_editors())
}

#[tauri::command]
pub async fn list_branches(
    repo_path: String,
    worktree_path: String,
) -> Result<WorktreeBranchesResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::list_branches_for_worktree(&repo_path, &worktree_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn checkout_worktree_branch(
    worktree_path: String,
    target_branch: String,
) -> Result<CheckoutBranchResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = GitService::checkout_worktree_branch(&worktree_path, &target_branch)?;
        // No separate repo root is available here, but any worktree path works equally well as
        // the working directory for the repo-wide `worktree list`/orphan-context git calls.
        result.worktree_info =
            WorktreeCleanerService::build_single_worktree_info(&worktree_path, &worktree_path)?;
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn detach_worktree_head(worktree_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || GitService::detach_worktree_head(&worktree_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn suggest_worktree_path(
    repo_path: String,
    branch_name: String,
) -> Result<SuggestWorktreePathResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::suggest_worktree_path(&repo_path, &branch_name)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn is_worktree_target_occupied(target_path: String) -> bool {
    GitService::is_occupied_target(std::path::Path::new(target_path.trim()))
}

#[tauri::command]
pub async fn create_worktree(
    repo_path: String,
    target_path: String,
    base_branch: String,
    new_branch_name: Option<String>,
) -> Result<CreateWorktreeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = GitService::create_worktree(
            &repo_path,
            &target_path,
            &base_branch,
            new_branch_name.as_deref(),
        )?;
        result.worktree_info =
            WorktreeCleanerService::build_single_worktree_info(&repo_path, &target_path)?;
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_stash_worktree(
    worktree_path: String,
    message: Option<String>,
) -> Result<WorktreeEntry, String> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::stash_worktree(&worktree_path, message.as_deref())?;
        WorktreeCleanerService::build_single_worktree_info(&worktree_path, &worktree_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_discard_worktree_changes(worktree_path: String) -> Result<WorktreeEntry, String> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::discard_worktree_changes(&worktree_path)?;
        WorktreeCleanerService::build_single_worktree_info(&worktree_path, &worktree_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_unlock_worktree(
    repo_path: String,
    worktree_path: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        GitService::unlock_worktree(&repo_path, &worktree_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_events_serialize_scan_id_in_camel_case() {
        let complete = serde_json::to_value(ScanCompleteEvent { scan_id: 7 }).unwrap();
        assert_eq!(complete, serde_json::json!({ "scanId": 7 }));

        let failed = serde_json::to_value(RepoScanFailedEvent {
            scan_id: 7,
            repo_path: "repo".into(),
            error: "boom".into(),
        })
        .unwrap();
        assert_eq!(
            failed,
            serde_json::json!({ "scanId": 7, "repoPath": "repo", "error": "boom" })
        );
    }
}
