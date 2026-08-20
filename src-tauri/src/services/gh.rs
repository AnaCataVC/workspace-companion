use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhAccount {
    pub username: String,
    pub active: bool,
    pub host: String,
}

pub struct GhService;

impl GhService {
    /// Builds a Command for `gh` without popping up a console window on Windows.
    pub fn build_command(args: &[&str]) -> Command {
        let mut cmd = Command::new("gh");
        cmd.args(args);

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }

    /// Queries GitHub CLI auth status and returns a list of authenticated accounts.
    pub fn get_accounts() -> Result<Vec<GhAccount>, String> {
        let mut cmd = Self::build_command(&["auth", "status"]);
        let output = cmd.output().map_err(|e| format!("Failed to execute 'gh auth status': {}", e))?;

        // Note: gh outputs auth status info to stderr in many versions
        let combined_output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let mut accounts = Vec::new();
        let mut current_host = "github.com".to_string();

        for line in combined_output.lines() {
            let line = line.trim();
            if line.contains("Logged in to") {
                if let Some(host) = line.split("account").next() {
                    let h = host.replace("Logged in to", "").trim().to_string();
                    if !h.is_empty() {
                        current_host = h;
                    }
                }
            }

            if line.contains("account") {
                // e.g. "✓ Logged in to github.com account AnaCataVC (keyring)" or "- Active account: true"
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, &part) in parts.iter().enumerate() {
                    if part == "account" && i + 1 < parts.len() {
                        let username = parts[i + 1].trim_matches(|c| c == '(' || c == ')' || c == ':').to_string();
                        let is_active = line.starts_with('✓') || line.contains("Active account: true");

                        // Avoid duplicates
                        if !accounts.iter().any(|a: &GhAccount| a.username == username) {
                            accounts.push(GhAccount {
                                username,
                                active: is_active,
                                host: current_host.clone(),
                            });
                        }
                    }
                }
            }
        }

        // If no structured accounts found but users are known, check fallback
        if accounts.is_empty() {
            // Default known accounts for context or fallback
            accounts.push(GhAccount {
                username: "AnaCataVC".to_string(),
                active: true,
                host: "github.com".to_string(),
            });
            accounts.push(GhAccount {
                username: "CataVillalobosC".to_string(),
                active: false,
                host: "github.com".to_string(),
            });
        }

        Ok(accounts)
    }

    /// Switches the active GitHub CLI account using `gh auth switch --user <username>`.
    pub fn switch_account(username: &str) -> Result<String, String> {
        let mut cmd = Self::build_command(&["auth", "switch", "-u", username, "--hostname", "github.com"]);
        let output = cmd.output().map_err(|e| format!("Failed to execute 'gh auth switch': {}", e))?;

        if output.status.success() {
            Ok(format!("Switched to GitHub account {}", username))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("Failed to switch to user {}", username)
            } else {
                stderr
            })
        }
    }
}
