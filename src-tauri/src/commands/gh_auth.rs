use crate::services::gh::{GhAccount, GhService};

#[tauri::command]
pub async fn get_gh_accounts() -> Result<Vec<GhAccount>, String> {
    GhService::get_accounts()
}

#[tauri::command]
pub async fn switch_gh_account(username: String) -> Result<String, String> {
    GhService::switch_account(&username)
}
