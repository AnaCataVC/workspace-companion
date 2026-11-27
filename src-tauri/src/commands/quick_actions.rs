use crate::services::config::ConfigService;
use crate::services::quick_actions::{
    QuickActionPreset, QuickActionPreview, QuickActionResult, QuickActionScope, QuickActionsService,
};

#[tauri::command]
pub async fn get_quick_action_preview(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = ConfigService::load_config();
        QuickActionsService::compute_preview(preset, &scope, &config.watch_folders)
    })
    .await
    .map_err(|e| format!("Quick action preview task failed: {}", e))?
}

#[tauri::command]
pub async fn execute_quick_action(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = ConfigService::load_config();
        QuickActionsService::execute_action_with_config(preset, &scope, &config)
    })
    .await
    .map_err(|e| format!("Quick action execution task failed: {}", e))?
}
