use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchFolder {
    pub id: String,
    pub path: String,
    pub account_username: Option<String>,
    pub enabled: bool,
    pub max_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub watch_folders: Vec<WatchFolder>,
    pub auto_switch_account: bool,
    pub default_editor: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            watch_folders: Vec::new(),
            auto_switch_account: true,
            default_editor: "vscode".to_string(),
        }
    }
}

pub struct ConfigService;

impl ConfigService {
    /// Resolves the OS AppData config directory for Workspace Companion.
    pub fn get_config_dir() -> Result<PathBuf, String> {
        let base_dir = if let Ok(app_data) = std::env::var("APPDATA") {
            PathBuf::from(app_data)
        } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            PathBuf::from(home).join(".config")
        } else {
            PathBuf::from(".")
        };

        let app_dir = base_dir.join("com.workspacecompanion.desktop");
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(app_dir)
    }

    /// Gets the path to the app_config.json file.
    pub fn get_config_path() -> Result<PathBuf, String> {
        let dir = Self::get_config_dir()?;
        Ok(dir.join("app_config.json"))
    }

    /// Loads the stored AppConfig or returns default if not found.
    pub fn load_config() -> AppConfig {
        if let Ok(config_path) = Self::get_config_path() {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        AppConfig::default()
    }

    /// Saves the AppConfig atomically to disk.
    pub fn save_config(config: &AppConfig) -> Result<(), String> {
        let config_path = Self::get_config_path()?;
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Expands home directory and environment variables in a path string.
    pub fn expand_path(raw_path: &str) -> PathBuf {
        let trimmed = raw_path.trim();
        if trimmed.starts_with("~/") || trimmed.starts_with("~\\") {
            if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                return PathBuf::from(home).join(&trimmed[2..]);
            }
        }
        PathBuf::from(trimmed)
    }
}
