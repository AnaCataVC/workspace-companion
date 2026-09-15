use crate::services::branch_cleaner::{
    BranchBatchDeleteSummary, BranchCleanerService, BranchDeleteTarget,
};
use crate::services::git::BranchStatusEntry;

#[tauri::command]
pub async fn scan_branches_for_cleanup(
    repo_paths: Vec<String>,
) -> Result<Vec<BranchStatusEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || BranchCleanerService::scan_branches(&repo_paths))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_branches_batch(
    targets: Vec<BranchDeleteTarget>,
) -> Result<BranchBatchDeleteSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        BranchCleanerService::remove_branches_batch(targets)
    })
    .await
    .map_err(|e| e.to_string())
}
