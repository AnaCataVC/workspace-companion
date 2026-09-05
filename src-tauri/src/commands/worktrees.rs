use crate::services::config::{AppConfig, ConfigService};
use crate::services::git::{
    CheckoutBranchResult, CreateWorktreeResult, EditorInfo, GitService, SuggestWorktreePathResult,
    WorktreeBranchesResponse, WorktreeDiffSummary,
};
use crate::services::worktree_cleaner::{
    BatchDeleteSummary, BatchDeleteTarget, WorktreeCleanerService,
};
use rayon::prelude::*;
use tauri::{AppHandle, Emitter};

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
pub async fn scan_worktrees(app: AppHandle) -> Result<(), String> {
    let config = ConfigService::load_config();
    let discovered = WorktreeCleanerService::discover_repositories(&config);

    tauri::async_runtime::spawn_blocking(move || {
        discovered
            .par_iter()
            .filter_map(|repo| {
                WorktreeCleanerService::scan_repository(
                    &repo.path,
                    repo.associated_account.clone(),
                    repo.watch_folder_path.clone(),
                )
            })
            .for_each(|repo_info| {
                let _ = app.emit("repo-scanned", &repo_info);
            });

        let _ = app.emit("scan-complete", ());
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
    tauri::async_runtime::spawn_blocking(move || WorktreeCleanerService::prune_worktrees(&repo_path))
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
