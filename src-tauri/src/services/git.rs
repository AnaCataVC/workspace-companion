use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Flag to prevent command prompt window from popping up on Windows
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeEntry {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub bare: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
    #[serde(rename = "isOrphaned")]
    pub is_orphaned: bool,
    #[serde(rename = "orphanReason")]
    pub orphan_reason: Option<String>,
    #[serde(rename = "isDirty")]
    pub is_dirty: bool,
    #[serde(rename = "uncommittedFilesCount")]
    pub uncommitted_files_count: Option<usize>,
    #[serde(rename = "lastCommitMessage")]
    pub last_commit_message: Option<String>,
    #[serde(rename = "lastCommitAuthor")]
    pub last_commit_author: Option<String>,
    #[serde(rename = "lastCommitDate")]
    pub last_commit_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub id: String,
    pub name: String,
    pub is_available: bool,
    pub icon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchEntry {
    pub name: String,
    pub short_name: String,
    pub is_remote: bool,
    pub is_current: bool,
    pub is_locked_by_other: bool,
    pub locked_worktree_path: Option<String>,
    pub last_commit_sha: Option<String>,
    pub last_commit_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeBranchesResponse {
    pub repo_path: String,
    pub worktree_path: String,
    pub current_branch: Option<String>,
    pub branches: Vec<BranchEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutBranchResult {
    pub success: bool,
    pub new_branch: String,
    pub head_sha: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestWorktreePathResult {
    pub suggested_path: String,
    pub already_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorktreeResult {
    pub success: bool,
    pub worktree_path: String,
    pub branch_name: String,
    pub message: String,
}

pub struct GitService;

impl GitService {
    /// Builds a Command configured to run silently without spawning a terminal window on Windows.
    pub fn build_command<P: AsRef<Path>>(working_dir: P, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(working_dir);
        cmd.args(args);

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }

    /// Runs a git command and returns its standard output as a trimmed string.
    pub fn run_git<P: AsRef<Path>>(working_dir: P, args: &[&str]) -> Result<String, String> {
        let mut cmd = Self::build_command(working_dir, args);
        let output = cmd.output().map_err(|e| format!("Failed to execute git: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("Git exited with status: {}", output.status)
            } else {
                stderr
            })
        }
    }

    /// Parses `git worktree list --porcelain` output.
    pub fn parse_worktree_porcelain(raw_output: &str) -> Vec<WorktreeEntry> {
        let mut entries = Vec::new();
        let mut current_path: Option<String> = None;
        let mut current_head: Option<String> = None;
        let mut current_branch: Option<String> = None;
        let mut current_bare = false;
        let mut current_locked: Option<String> = None;
        let mut current_prunable: Option<String> = None;

        for line in raw_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                if let Some(path) = current_path.take() {
                    entries.push(WorktreeEntry {
                        path,
                        head: current_head.take().unwrap_or_default(),
                        branch: current_branch.take(),
                        bare: current_bare,
                        locked: current_locked.take(),
                        prunable: current_prunable.take(),
                        is_orphaned: false,
                        orphan_reason: None,
                        is_dirty: false,
                        uncommitted_files_count: None,
                        last_commit_message: None,
                        last_commit_author: None,
                        last_commit_date: None,
                    });
                }
                current_bare = false;
                continue;
            }

            if let Some(path) = line.strip_prefix("worktree ") {
                current_path = Some(path.to_string());
            } else if let Some(head) = line.strip_prefix("HEAD ") {
                current_head = Some(head.to_string());
            } else if let Some(branch) = line.strip_prefix("branch ") {
                current_branch = Some(branch.to_string());
            } else if line == "bare" {
                current_bare = true;
            } else if let Some(locked) = line.strip_prefix("locked ") {
                current_locked = Some(locked.to_string());
            } else if line == "locked" {
                current_locked = Some("Locked".to_string());
            } else if let Some(prunable) = line.strip_prefix("prunable ") {
                current_prunable = Some(prunable.to_string());
            }
        }

        // Catch last entry if output didn't end with a newline
        if let Some(path) = current_path.take() {
            entries.push(WorktreeEntry {
                path,
                head: current_head.take().unwrap_or_default(),
                branch: current_branch.take(),
                bare: current_bare,
                locked: current_locked.take(),
                prunable: current_prunable.take(),
                is_orphaned: false,
                orphan_reason: None,
                is_dirty: false,
                uncommitted_files_count: None,
                last_commit_message: None,
                last_commit_author: None,
                last_commit_date: None,
            });
        }

        entries
    }

    /// Checks if a worktree directory has uncommitted or untracked changes (`git status --porcelain`).
    pub fn check_dirty_status<P: AsRef<Path>>(worktree_path: P) -> (bool, usize) {
        if !worktree_path.as_ref().exists() {
            return (false, 0);
        }

        match Self::run_git(worktree_path, &["status", "--porcelain=v1"]) {
            Ok(output) => {
                let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
                (!lines.is_empty(), lines.len())
            }
            Err(_) => (false, 0),
        }
    }

    /// Fetches the last commit message, author, and date for a worktree.
    pub fn get_last_commit_info<P: AsRef<Path>>(
        worktree_path: P,
    ) -> (Option<String>, Option<String>, Option<String>) {
        if !worktree_path.as_ref().exists() {
            return (None, None, None);
        }

        // Format: %s|||%an|||%cr
        match Self::run_git(worktree_path, &["log", "-1", "--format=%s|||%an|||%cr"]) {
            Ok(log_line) => {
                let parts: Vec<&str> = log_line.split("|||").collect();
                let msg = parts.get(0).map(|s| s.to_string());
                let author = parts.get(1).map(|s| s.to_string());
                let date = parts.get(2).map(|s| s.to_string());
                (msg, author, date)
            }
            Err(_) => (None, None, None),
        }
    }

    /// Checks if a branch is merged into the main/default branch or marked as `[gone]` in upstream.
    pub fn check_orphan_status<P: AsRef<Path>>(
        repo_root: P,
        branch_ref: &str,
    ) -> (bool, Option<String>) {
        let short_branch = branch_ref.replace("refs/heads/", "");

        // Check if upstream branch is gone (git branch -vv)
        if let Ok(branch_vv) = Self::run_git(&repo_root, &["branch", "-vv"]) {
            for line in branch_vv.lines() {
                if line.contains(&short_branch) && line.contains(": gone]") {
                    return (true, Some("Upstream remote branch was deleted".to_string()));
                }
            }
        }

        // Check if merged into main or master
        let default_branch = Self::get_default_branch(&repo_root).unwrap_or_else(|| "main".to_string());
        if short_branch != default_branch && short_branch != "master" {
            if let Ok(merged_branches) = Self::run_git(&repo_root, &["branch", "--merged", &default_branch]) {
                for line in merged_branches.lines() {
                    let cleaned = line.trim().trim_start_matches('*').trim();
                    if cleaned == short_branch {
                        return (true, Some(format!("Merged into {}", default_branch)));
                    }
                }
            }
        }

        (false, None)
    }

    /// Identifies the default branch (main/master).
    pub fn get_default_branch<P: AsRef<Path>>(repo_root: P) -> Option<String> {
        if let Ok(symbolic) = Self::run_git(&repo_root, &["symbolic-ref", "refs/remotes/origin/HEAD"]) {
            return Some(symbolic.replace("refs/remotes/origin/", ""));
        }
        if let Ok(branches) = Self::run_git(&repo_root, &["branch", "--list", "main"]) {
            if !branches.trim().is_empty() {
                return Some("main".to_string());
            }
        }
        if let Ok(branches) = Self::run_git(&repo_root, &["branch", "--list", "master"]) {
            if !branches.trim().is_empty() {
                return Some("master".to_string());
            }
        }
        None
    }

    /// Launches the selected IDE, editor, or terminal targeting the given path.
    pub fn open_in_editor(editor: &str, path: &str) -> Result<(), String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Target path does not exist: {}", path));
        }

        match editor {
            "explorer" => {
                open::that(path).map_err(|e| format!("Failed to open Explorer: {}", e))?;
                Ok(())
            }
            "wt" => {
                #[cfg(target_os = "windows")]
                {
                    let mut cmd = Command::new("wt");
                    cmd.args(&["-d", path]);
                    cmd.creation_flags(CREATE_NO_WINDOW);
                    cmd.spawn().map_err(|e| format!("Failed to launch Windows Terminal: {}. Ensure wt is available.", e))?;
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Command::new("wt").args(&["-d", path]).spawn().map_err(|e| e.to_string())?;
                }
                Ok(())
            }
            "antigravity" | "agy" => {
                if Self::is_bin_available("antigravity") {
                    Self::launch_detached_editor("antigravity", path)
                } else if Self::is_bin_available("agy") {
                    Self::launch_detached_editor("agy", path)
                } else if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                    let candidate1 = Path::new(&local_app_data)
                        .join("Programs")
                        .join("Antigravity")
                        .join("Antigravity.exe");
                    let candidate2 = Path::new(&local_app_data)
                        .join("Programs")
                        .join("Antigravity IDE")
                        .join("Antigravity.exe");
                    if candidate1.exists() {
                        Self::launch_detached_editor(&candidate1.to_string_lossy(), path)
                    } else if candidate2.exists() {
                        Self::launch_detached_editor(&candidate2.to_string_lossy(), path)
                    } else {
                        Err("Antigravity executable not found. Ensure Antigravity IDE is installed.".into())
                    }
                } else {
                    Err("Antigravity executable not found.".into())
                }
            }
            "vscode" | "code" => Self::launch_detached_editor("code", path),
            "cursor" => Self::launch_detached_editor("cursor", path),
            "windsurf" => Self::launch_detached_editor("windsurf", path),
            unknown => Err(format!("Unsupported editor identifier: {}", unknown)),
        }
    }

    fn launch_detached_editor(bin: &str, path: &str) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", "start", "", bin, path]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.spawn().map_err(|e| format!("Failed to launch {}: {}", bin, e))?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new(bin).arg(path).spawn().map_err(|e| format!("Failed to launch {}: {}", bin, e))?;
        }
        Ok(())
    }

    /// Checks if a binary command exists in the system PATH.
    pub fn is_bin_available(bin: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("where");
            cmd.arg(bin);
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.output().map(|o| o.status.success()).unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("which")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Detects installed editors by probing executables and standard installation paths silently.
    pub fn detect_installed_editors() -> Vec<EditorInfo> {
        let editors = vec![
            ("antigravity", "Antigravity", "antigravity", "sparkles"),
            ("vscode", "VS Code", "code", "code"),
            ("cursor", "Cursor", "cursor", "sparkles"),
            ("windsurf", "Windsurf", "windsurf", "wind"),
            ("wt", "Windows Terminal", "wt", "terminal"),
            ("explorer", "File Explorer", "explorer", "folder"),
        ];

        editors
            .into_iter()
            .map(|(id, name, bin, icon)| {
                let available = if id == "explorer" {
                    true
                } else if id == "antigravity" {
                    Self::is_bin_available("antigravity")
                        || Self::is_bin_available("agy")
                        || std::env::var("LOCALAPPDATA")
                            .map(|la| {
                                Path::new(&la)
                                    .join("Programs")
                                    .join("Antigravity")
                                    .join("Antigravity.exe")
                                    .exists()
                                    || Path::new(&la)
                                        .join("Programs")
                                        .join("Antigravity IDE")
                                        .join("Antigravity.exe")
                                        .exists()
                            })
                            .unwrap_or(false)
                } else if id == "cursor" {
                    Self::is_bin_available("cursor")
                        || std::env::var("LOCALAPPDATA")
                            .map(|la| {
                                Path::new(&la)
                                    .join("Programs")
                                    .join("cursor")
                                    .join("Cursor.exe")
                                    .exists()
                            })
                            .unwrap_or(false)
                } else if id == "windsurf" {
                    Self::is_bin_available("windsurf")
                        || std::env::var("LOCALAPPDATA")
                            .map(|la| {
                                Path::new(&la)
                                    .join("Programs")
                                    .join("windsurf")
                                    .join("Windsurf.exe")
                                    .exists()
                            })
                            .unwrap_or(false)
                } else {
                    Self::is_bin_available(bin)
                };

                EditorInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    is_available: available,
                    icon_name: icon.to_string(),
                }
            })
            .collect()
    }

    /// Lists all local and remote branches for a repository, marking locks from sibling worktrees.
    pub fn list_branches_for_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_path: &str,
    ) -> Result<WorktreeBranchesResponse, String> {
        let repo_root = repo_path.as_ref();

        // 1. Map all checked out branches across worktrees
        let raw_wt = Self::run_git(repo_root, &["worktree", "list", "--porcelain"])?;
        let worktrees = Self::parse_worktree_porcelain(&raw_wt);

        let mut branch_to_worktree: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut current_worktree_branch: Option<String> = None;

        for wt in &worktrees {
            if let Some(b) = &wt.branch {
                let short_b = b.replace("refs/heads/", "");
                branch_to_worktree.insert(short_b.clone(), wt.path.clone());
                if wt.path == worktree_path {
                    current_worktree_branch = Some(short_b);
                }
            }
        }

        // 2. Query local branches
        let mut branches = Vec::new();
        let local_raw = Self::run_git(
            repo_root,
            &["branch", "--list", "--format=%(refname:short)|||%(objectname:short)|||%(subject)"],
        )?;

        for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
            let parts: Vec<&str> = line.split("|||").collect();
            let short_name = parts.get(0).unwrap_or(&"").to_string();
            let sha = parts.get(1).map(|s| s.to_string());
            let msg = parts.get(2).map(|s| s.to_string());

            let is_current = current_worktree_branch.as_deref() == Some(&short_name);
            let locked_by = branch_to_worktree.get(&short_name).cloned();
            let is_locked = locked_by.is_some() && !is_current;

            branches.push(BranchEntry {
                name: short_name.clone(),
                short_name,
                is_remote: false,
                is_current,
                is_locked_by_other: is_locked,
                locked_worktree_path: if is_locked { locked_by } else { None },
                last_commit_sha: sha,
                last_commit_message: msg,
            });
        }

        // 3. Query remote branches (excluding origin/HEAD)
        if let Ok(remote_raw) = Self::run_git(
            repo_root,
            &["branch", "-r", "--format=%(refname:short)|||%(objectname:short)|||%(subject)"],
        ) {
            for line in remote_raw.lines().filter(|l| !l.trim().is_empty()) {
                let parts: Vec<&str> = line.split("|||").collect();
                let full_name = parts.get(0).unwrap_or(&"").to_string();
                if full_name.contains("/HEAD") {
                    continue;
                }
                let clean_name = full_name
                    .splitn(2, '/')
                    .nth(1)
                    .unwrap_or(&full_name)
                    .to_string();

                // Only add if not already present in local branches
                if !branches.iter().any(|b| b.short_name == clean_name) {
                    let sha = parts.get(1).map(|s| s.to_string());
                    let msg = parts.get(2).map(|s| s.to_string());

                    branches.push(BranchEntry {
                        name: full_name,
                        short_name: clean_name,
                        is_remote: true,
                        is_current: false,
                        is_locked_by_other: false,
                        locked_worktree_path: None,
                        last_commit_sha: sha,
                        last_commit_message: msg,
                    });
                }
            }
        }

        Ok(WorktreeBranchesResponse {
            repo_path: repo_root.to_string_lossy().to_string(),
            worktree_path: worktree_path.to_string(),
            current_branch: current_worktree_branch,
            branches,
        })
    }

    /// Safely checks out a branch on an existing worktree after pre-flight dirty verification.
    pub fn checkout_worktree_branch(
        worktree_path: &str,
        target_branch: &str,
    ) -> Result<CheckoutBranchResult, String> {
        let wt_path = Path::new(worktree_path);
        if !wt_path.exists() {
            return Err(format!("Worktree path does not exist: {}", worktree_path));
        }

        // Pre-flight dirty check
        let (is_dirty, count) = Self::check_dirty_status(wt_path);
        if is_dirty {
            return Err(format!(
                "Cannot switch branch: {} uncommitted or modified files detected. Please stash or commit changes first.",
                count
            ));
        }

        // Run checkout
        let checkout_args = if target_branch.starts_with("origin/") {
            let local_name = target_branch.trim_start_matches("origin/");
            vec!["checkout", "-B", local_name, "--track", target_branch]
        } else {
            vec!["checkout", target_branch]
        };

        let output = Self::run_git(wt_path, &checkout_args)?;

        // Retrieve new HEAD SHA
        let head_sha = Self::run_git(wt_path, &["rev-parse", "--short", "HEAD"]).unwrap_or_default();

        Ok(CheckoutBranchResult {
            success: true,
            new_branch: target_branch.to_string(),
            head_sha,
            message: output,
        })
    }

    /// Generates a standardized sibling directory path for a new worktree.
    pub fn suggest_worktree_path<P: AsRef<Path>>(
        repo_path: P,
        branch_name: &str,
    ) -> Result<SuggestWorktreePathResult, String> {
        let repo_dir = repo_path.as_ref();
        let parent_dir = repo_dir.parent().ok_or("Cannot determine parent directory")?;
        let repo_name = repo_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo");

        // Sanitize branch name: replace invalid path characters
        let sanitized_branch = branch_name
            .trim()
            .replace("refs/heads/", "")
            .replace("origin/", "")
            .replace('/', "-")
            .replace('\\', "-")
            .replace(':', "-")
            .replace(' ', "-");

        let base_name = if sanitized_branch.is_empty() {
            format!("{}-worktree", repo_name)
        } else {
            format!("{}-{}", repo_name, sanitized_branch)
        };

        let mut candidate = parent_dir.join(&base_name);
        let mut index = 1;

        while candidate.exists() {
            candidate = parent_dir.join(format!("{}-{}", base_name, index));
            index += 1;
        }

        Ok(SuggestWorktreePathResult {
            suggested_path: candidate.to_string_lossy().to_string(),
            already_exists: candidate.exists(),
        })
    }

    /// Creates a new worktree with either a new branch or an existing branch.
    pub fn create_worktree<P: AsRef<Path>>(
        repo_path: P,
        target_path: &str,
        base_branch: &str,
        new_branch_name: Option<&str>,
    ) -> Result<CreateWorktreeResult, String> {
        let repo_root = repo_path.as_ref();
        let target_dir = Path::new(target_path);

        if target_dir.exists() {
            if let Ok(mut entries) = std::fs::read_dir(target_dir) {
                if entries.next().is_some() {
                    return Err(format!(
                        "Target path '{}' already exists and is not empty.",
                        target_path
                    ));
                }
            }
        }

        let branch_created = if let Some(new_branch) = new_branch_name {
            let clean_new_branch = new_branch.trim();
            if clean_new_branch.is_empty() {
                return Err("New branch name cannot be empty".to_string());
            }

            // Validate ref format
            Self::run_git(repo_root, &["check-ref-format", "--branch", clean_new_branch])
                .map_err(|_| format!("Invalid branch name format: '{}'", clean_new_branch))?;

            // Execute: git worktree add <target_path> -b <new_branch> <base_branch>
            Self::run_git(
                repo_root,
                &["worktree", "add", target_path, "-b", clean_new_branch, base_branch],
            )?;
            clean_new_branch.to_string()
        } else {
            // Checkout existing branch
            // Execute: git worktree add <target_path> <base_branch>
            Self::run_git(repo_root, &["worktree", "add", target_path, base_branch])?;
            base_branch.to_string()
        };

        Ok(CreateWorktreeResult {
            success: true,
            worktree_path: target_path.to_string(),
            branch_name: branch_created,
            message: format!("Successfully created worktree at '{}'", target_path),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_porcelain() {
        let sample_output = r#"worktree C:/Repos/my-project
HEAD 53843e57f26f296317769533f81e3a105f234327
branch refs/heads/main

worktree C:/Repos/my-project-feature
HEAD 90fc88349216172671239533f81e3a105f239082
branch refs/heads/feat/new-ui
locked In progress

worktree C:/Repos/my-project-bare
bare
"#;

        let entries = GitService::parse_worktree_porcelain(sample_output);
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].path, "C:/Repos/my-project");
        assert_eq!(entries[0].head, "53843e57f26f296317769533f81e3a105f234327");
        assert_eq!(entries[0].branch.as_deref(), Some("refs/heads/main"));
        assert!(!entries[0].bare);
        assert_eq!(entries[0].locked, None);

        assert_eq!(entries[1].path, "C:/Repos/my-project-feature");
        assert_eq!(entries[1].branch.as_deref(), Some("refs/heads/feat/new-ui"));
        assert_eq!(entries[1].locked.as_deref(), Some("In progress"));

        assert_eq!(entries[2].path, "C:/Repos/my-project-bare");
        assert!(entries[2].bare);
    }

    #[test]
    fn test_suggest_worktree_path_sanitization() {
        let res = GitService::suggest_worktree_path("C:/Repos/app", "feat/cool-feature").unwrap();
        assert!(res.suggested_path.contains("app-feat-cool-feature"));
    }
}

