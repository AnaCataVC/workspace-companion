use crate::services::config::ConfigService;
use crate::services::quick_actions::{
    QuickActionPreset, QuickActionPreview, QuickActionResult, QuickActionScope, QuickActionsService,
};

#[tauri::command]
pub async fn get_quick_action_preview(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionPreview, String> {
    let config = ConfigService::load_config();
    QuickActionsService::compute_preview(preset, &scope, &config.watch_folders)
}

#[tauri::command]
pub async fn execute_quick_action(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionResult, String> {
    let config = ConfigService::load_config();
    QuickActionsService::execute_action(preset, &scope, &config.watch_folders)
}
