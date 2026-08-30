use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchFolder {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub account_username: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
}

fn default_true() -> bool {
    true
}

fn default_max_depth() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub watch_folders: Vec<WatchFolder>,
    #[serde(default = "default_true")]
    pub auto_switch_account: bool,
    #[serde(default = "default_editor")]
    pub default_editor: String,
    #[serde(default = "default_terminal")]
    pub default_terminal: String,
    #[serde(default = "default_true")]
    pub show_terminal_button: bool,
}

fn default_version() -> u32 {
    1
}

fn default_editor() -> String {
    "vscode".to_string()
}

fn default_terminal() -> String {
    "wt".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            watch_folders: Vec::new(),
            auto_switch_account: true,
            default_editor: "vscode".to_string(),
            default_terminal: "wt".to_string(),
            show_terminal_button: true,
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

    /// Saves the AppConfig atomically to disk using a temp file.
    pub fn save_config(config: &AppConfig) -> Result<(), String> {
        let config_path = Self::get_config_path()?;
        let temp_path = config_path.with_extension("tmp");
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&temp_path, json)
            .map_err(|e| format!("Failed to write temporary config file: {}", e))?;

        fs::rename(&temp_path, &config_path)
            .map_err(|e| format!("Failed to atomically commit config file: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.watch_folders.is_empty());
        assert!(config.auto_switch_account);
        assert_eq!(config.default_editor, "vscode");
        assert_eq!(config.default_terminal, "wt");
        assert!(config.show_terminal_button);
    }

    #[test]
    fn test_expand_path_standard() {
        let path = "C:/Projects/Repo";
        let expanded = ConfigService::expand_path(path);
        assert_eq!(expanded, PathBuf::from("C:/Projects/Repo"));
    }

    #[test]
    fn test_expand_path_with_tilde() {
        let path = "~/Projects/Repo";
        let expanded = ConfigService::expand_path(path);
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let expected = PathBuf::from(home).join("Projects/Repo");
            assert_eq!(expanded, expected);
        }
    }

    #[test]
    fn test_partial_json_deserialization() {
        let json = r#"{"watchFolders": [{"id": "1", "path": "C:/Repos"}]}"#;
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.default_editor, "vscode");
        assert_eq!(config.default_terminal, "wt");
        assert!(config.show_terminal_button);
        assert!(config.auto_switch_account);
        assert_eq!(config.watch_folders.len(), 1);
        assert!(config.watch_folders[0].enabled);
        assert_eq!(config.watch_folders[0].max_depth, 1);
    }
}
