use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Flag to prevent command prompt window from popping up on Windows
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorktreeEntry {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub bare: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
    #[serde(rename = "isMain")]
    pub is_main: bool,
    #[serde(rename = "isOrphaned")]
    pub is_orphaned: bool,
    #[serde(rename = "orphanReason")]
    pub orphan_reason: Option<String>,
    /// Status of the branch this worktree has checked out, so the worktree view can say *why* a
    /// worktree is disposable instead of only that it is. Same source as the branch cleaner's own
    /// flags (`branch_status_flags`), so the two views can never disagree.
    #[serde(rename = "isBranchMerged")]
    pub is_branch_merged: bool,
    #[serde(rename = "isBranchRemoteGone")]
    pub is_branch_remote_gone: bool,
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
pub struct WorktreeDiffSummary {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub summary_text: String,
    pub modified_files: Vec<String>,
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

/// A local branch with the status the branch cleaner needs, independent of whether it has a
/// worktree at all — unlike `BranchEntry`, which is scoped to one worktree's checkout context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStatusEntry {
    pub repo_path: String,
    pub name: String,
    pub is_current: bool,
    pub is_default: bool,
    pub is_merged: bool,
    pub is_remote_gone: bool,
    pub is_checked_out: bool,
    pub checked_out_worktree_path: Option<String>,
    pub last_commit_sha: Option<String>,
    pub last_commit_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutBranchResult {
    pub success: bool,
    pub new_branch: String,
    pub head_sha: String,
    pub message: String,
    pub worktree_info: WorktreeEntry,
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
    pub worktree_info: WorktreeEntry,
}

/// Repo-wide git state needed to evaluate whether a branch is orphaned. `check_orphan_status`
/// used to recompute all of this (the default branch, `branch -vv`, `branch --merged`) from
/// scratch for every worktree, even though the answer is identical for every worktree in the
/// same repo. Computing it once per repo via `build_repo_orphan_context` and passing it in here
/// cuts a repo with N worktrees from ~5N git subprocesses down to ~5.
#[derive(Debug, Clone)]
pub struct RepoOrphanContext {
    pub default_branch: String,
    pub branch_vv_output: String,
    pub merged_branches_output: String,
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
                        is_main: false,
                        is_orphaned: false,
                        orphan_reason: None,
                        is_branch_merged: false,
                        is_branch_remote_gone: false,
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
                is_main: false,
                is_orphaned: false,
                orphan_reason: None,
                is_branch_merged: false,
                is_branch_remote_gone: false,
                is_dirty: false,
                uncommitted_files_count: None,
                last_commit_message: None,
                last_commit_author: None,
                last_commit_date: None,
            });
        }

        entries
    }

    /// Fetches on-demand git diff summary and modified file list for popover previews without overhead.
    pub fn get_diff_summary<P: AsRef<Path>>(worktree_path: P) -> Result<WorktreeDiffSummary, String> {
        let wt = worktree_path.as_ref();
        if !wt.exists() {
            return Err(format!("Worktree path does not exist: {}", wt.display()));
        }

        let mut modified_files = Vec::new();
        if let Ok(status_out) = Self::run_git(wt, &["status", "--porcelain=v1"]) {
            for line in status_out.lines().take(20) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    modified_files.push(trimmed.to_string());
                }
            }
        }

        let shortstat = Self::run_git(wt, &["diff", "--shortstat"]).unwrap_or_default();
        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        for part in shortstat.split(',') {
            let p = part.trim();
            if p.contains("file changed") || p.contains("files changed") {
                if let Some(num_str) = p.split_whitespace().next() {
                    files_changed = num_str.parse().unwrap_or(0);
                }
            } else if p.contains("insertion") {
                if let Some(num_str) = p.split_whitespace().next() {
                    insertions = num_str.parse().unwrap_or(0);
                }
            } else if p.contains("deletion") {
                if let Some(num_str) = p.split_whitespace().next() {
                    deletions = num_str.parse().unwrap_or(0);
                }
            }
        }

        if files_changed == 0 && !modified_files.is_empty() {
            files_changed = modified_files.len();
        }

        let summary_text = if shortstat.is_empty() {
            if modified_files.is_empty() {
                "Clean working tree".to_string()
            } else {
                format!("{} modified/untracked files", modified_files.len())
            }
        } else {
            shortstat
        };

        Ok(WorktreeDiffSummary {
            files_changed,
            insertions,
            deletions,
            summary_text,
            modified_files,
        })
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

    /// Computes the repo-wide state needed for orphan-status checks exactly once per repository.
    /// See `RepoOrphanContext` for why this must not be recomputed per worktree.
    pub fn build_repo_orphan_context<P: AsRef<Path>>(repo_root: P) -> RepoOrphanContext {
        let default_branch =
            Self::get_default_branch(&repo_root).unwrap_or_else(|| "main".to_string());
        let branch_vv_output = Self::run_git(&repo_root, &["branch", "-vv"]).unwrap_or_default();
        let merged_branches_output =
            Self::run_git(&repo_root, &["branch", "--merged", &default_branch]).unwrap_or_default();

        RepoOrphanContext {
            default_branch,
            branch_vv_output,
            merged_branches_output,
        }
    }

    /// Splits the orphan check into its two independent components (merged into the default
    /// branch vs. upstream deleted). `check_orphan_status` collapses both into one "orphaned"
    /// bucket for the worktree cleaner; the branch cleaner needs them separately so it can offer
    /// "Merged" and "Remote gone" as distinct filters.
    pub(crate) fn branch_status_flags(
        short_branch: &str,
        context: &RepoOrphanContext,
    ) -> (bool, bool) {
        let mut is_remote_gone = false;
        for line in context.branch_vv_output.lines() {
            let trimmed = line.trim().trim_start_matches(['*', '+']).trim();
            let branch_token = trimmed.split_whitespace().next().unwrap_or("");
            if branch_token == short_branch && line.contains(": gone]") {
                is_remote_gone = true;
                break;
            }
        }

        let mut is_merged = false;
        if short_branch != context.default_branch && short_branch != "master" {
            for line in context.merged_branches_output.lines() {
                let cleaned = line.trim().trim_start_matches(['*', '+']).trim();
                if cleaned == short_branch {
                    is_merged = true;
                    break;
                }
            }
        }

        (is_merged, is_remote_gone)
    }

    /// Checks if a branch is merged into the default branch or marked as `[gone]` in upstream,
    /// using a `RepoOrphanContext` precomputed once per repository instead of shelling out to
    /// git again for every worktree.
    pub fn check_orphan_status(
        branch_ref: &str,
        context: &RepoOrphanContext,
    ) -> (bool, Option<String>) {
        let short_branch = branch_ref.replace("refs/heads/", "");
        let (is_merged, is_remote_gone) = Self::branch_status_flags(&short_branch, context);

        if is_remote_gone {
            return (true, Some("Upstream remote branch was deleted".to_string()));
        }
        if is_merged {
            return (true, Some(format!("Merged into {}", context.default_branch)));
        }

        (false, None)
    }

    /// Lists every local branch in a repository — regardless of whether it has a worktree — with
    /// merged/gone/current/default/checked-out status, for the branch cleaner. Reuses the same
    /// `RepoOrphanContext` and status-classification logic as `check_orphan_status` instead of
    /// recomputing it, and cross-references `worktree list --porcelain` (same as
    /// `list_branches_for_worktree`) so a branch checked out anywhere — including the main
    /// worktree — is flagged rather than left deletable.
    pub fn list_local_branches_with_status<P: AsRef<Path>>(
        repo_path: P,
    ) -> Result<Vec<BranchStatusEntry>, String> {
        let repo_root = repo_path.as_ref();
        let repo_path_str = repo_root.to_string_lossy().to_string();

        let context = Self::build_repo_orphan_context(repo_root);

        let raw_wt = Self::run_git(repo_root, &["worktree", "list", "--porcelain"])?;
        let worktrees = Self::parse_worktree_porcelain(&raw_wt);
        let mut branch_to_worktree: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for wt in &worktrees {
            if let Some(b) = &wt.branch {
                branch_to_worktree.insert(b.replace("refs/heads/", ""), wt.path.clone());
            }
        }

        let current_branch =
            Self::run_git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"]).ok();

        let local_raw = Self::run_git(
            repo_root,
            &["branch", "--list", "--format=%(refname:short)|||%(objectname:short)|||%(subject)"],
        )?;

        let mut entries = Vec::new();
        for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
            let parts: Vec<&str> = line.split("|||").collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let sha = parts.get(1).map(|s| s.to_string());
            let msg = parts.get(2).map(|s| s.to_string());

            let (is_merged, is_remote_gone) = Self::branch_status_flags(&name, &context);
            let checked_out_worktree_path = branch_to_worktree.get(&name).cloned();

            entries.push(BranchStatusEntry {
                repo_path: repo_path_str.clone(),
                is_current: current_branch.as_deref() == Some(name.as_str()),
                is_default: name == context.default_branch || name == "master",
                is_merged,
                is_remote_gone,
                is_checked_out: checked_out_worktree_path.is_some(),
                checked_out_worktree_path,
                last_commit_sha: sha,
                last_commit_message: msg,
                name,
            });
        }

        Ok(entries)
    }

    /// Deletes a local branch. Uses `-d` (git's own safety net: refuses if the branch is not
    /// merged into the current branch) unless `force` is set, which uses `-D`.
    pub fn delete_branch<P: AsRef<Path>>(
        repo_path: P,
        branch_name: &str,
        force: bool,
    ) -> Result<(), String> {
        let flag = if force { "-D" } else { "-d" };
        Self::run_git(repo_path, &["branch", flag, branch_name])?;
        Ok(())
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

    /// Resolves the direct executable path for standard IDEs on Windows.
    pub fn resolve_gui_binary(id: &str) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").ok();
            let program_files = std::env::var("ProgramFiles").ok();
            let program_files_x86 = std::env::var("ProgramFiles(x86)").ok();

            match id {
                "vscode" | "code" => {
                    // 1. User setup
                    if let Some(ref la) = local_app_data {
                        let p = Path::new(la).join("Programs").join("Microsoft VS Code").join("Code.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    // 2. System setup (64-bit)
                    if let Some(ref pf) = program_files {
                        let p = Path::new(pf).join("Microsoft VS Code").join("Code.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    // 3. System setup (32-bit)
                    if let Some(ref pf86) = program_files_x86 {
                        let p = Path::new(pf86).join("Microsoft VS Code").join("Code.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    // 4. VS Code Insiders
                    if let Some(ref la) = local_app_data {
                        let p = Path::new(la).join("Programs").join("Microsoft VS Code Insiders").join("Code - Insiders.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    if let Some(ref pf) = program_files {
                        let p = Path::new(pf).join("Microsoft VS Code Insiders").join("Code - Insiders.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    // 5. Inspect PATH via where.exe code / code.cmd and look in parent folder for Code.exe
                    if let Some(p) = Self::find_exe_from_path_bin("code", "Code.exe") {
                        return Some(p);
                    }
                }
                "antigravity" => {
                    if let Some(ref la) = local_app_data {
                        let p1 = Path::new(la).join("Programs").join("Antigravity").join("Antigravity.exe");
                        if p1.exists() {
                            return Some(p1);
                        }
                        let p2 = Path::new(la).join("Programs").join("Antigravity IDE").join("Antigravity.exe");
                        if p2.exists() {
                            return Some(p2);
                        }
                    }
                    if let Some(ref pf) = program_files {
                        let p1 = Path::new(pf).join("Antigravity").join("Antigravity.exe");
                        if p1.exists() {
                            return Some(p1);
                        }
                        let p2 = Path::new(pf).join("Antigravity IDE").join("Antigravity.exe");
                        if p2.exists() {
                            return Some(p2);
                        }
                    }
                    let p_direct = Path::new(r"C:\Program Files\Antigravity\Antigravity.exe");
                    if p_direct.exists() {
                        return Some(p_direct.to_path_buf());
                    }
                    if let Some(p) = Self::find_exe_from_path_bin("antigravity", "Antigravity.exe") {
                        return Some(p);
                    }
                }
                "cursor" => {
                    if let Some(ref la) = local_app_data {
                        let p = Path::new(la).join("Programs").join("cursor").join("Cursor.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    if let Some(ref pf) = program_files {
                        let p = Path::new(pf).join("Cursor").join("Cursor.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    if let Some(p) = Self::find_exe_from_path_bin("cursor", "Cursor.exe") {
                        return Some(p);
                    }
                }
                "windsurf" => {
                    if let Some(ref la) = local_app_data {
                        let p1 = Path::new(la).join("Programs").join("windsurf").join("Windsurf.exe");
                        if p1.exists() {
                            return Some(p1);
                        }
                        let p2 = Path::new(la).join("Programs").join("Windsurf").join("Windsurf.exe");
                        if p2.exists() {
                            return Some(p2);
                        }
                    }
                    if let Some(ref pf) = program_files {
                        let p = Path::new(pf).join("Windsurf").join("Windsurf.exe");
                        if p.exists() {
                            return Some(p);
                        }
                    }
                    if let Some(p) = Self::find_exe_from_path_bin("windsurf", "Windsurf.exe") {
                        return Some(p);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Helper to find executable from where.exe resolution (handles bin/ scripts pointing to parent .exe)
    #[cfg(target_os = "windows")]
    fn find_exe_from_path_bin(bin_name: &str, target_exe_name: &str) -> Option<PathBuf> {
        let mut cmd = Command::new("where");
        cmd.arg(bin_name);
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let path_buf = PathBuf::from(line.trim());
                    if path_buf.exists() {
                        if path_buf.file_name().and_then(|n| n.to_str()).map(|n| n.eq_ignore_ascii_case(target_exe_name)).unwrap_or(false) {
                            return Some(path_buf);
                        }
                        // Check if located inside a bin/ subdirectory
                        if let Some(parent) = path_buf.parent() {
                            if parent.file_name().and_then(|n| n.to_str()).map(|n| n.eq_ignore_ascii_case("bin")).unwrap_or(false) {
                                if let Some(grandparent) = parent.parent() {
                                    let candidate = grandparent.join(target_exe_name);
                                    if candidate.exists() {
                                        return Some(candidate);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Launches the selected IDE, editor, or explorer targeting the given path.
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
            "vscode" | "code" => {
                if let Some(exe) = Self::resolve_gui_binary("vscode") {
                    Self::launch_detached_editor(&exe.to_string_lossy(), path)
                } else if Self::is_bin_available("code") {
                    Self::launch_detached_editor("code", path)
                } else {
                    Err("Visual Studio Code executable not found. Ensure VS Code is installed.".into())
                }
            }
            "antigravity" => {
                if let Some(exe) = Self::resolve_gui_binary("antigravity") {
                    Self::launch_detached_editor(&exe.to_string_lossy(), path)
                } else if Self::is_bin_available("antigravity") {
                    Self::launch_detached_editor("antigravity", path)
                } else {
                    Err("Antigravity IDE executable not found. Ensure Antigravity IDE is installed.".into())
                }
            }
            "cursor" => {
                if let Some(exe) = Self::resolve_gui_binary("cursor") {
                    Self::launch_detached_editor(&exe.to_string_lossy(), path)
                } else if Self::is_bin_available("cursor") {
                    Self::launch_detached_editor("cursor", path)
                } else {
                    Err("Cursor executable not found. Ensure Cursor is installed.".into())
                }
            }
            "windsurf" => {
                if let Some(exe) = Self::resolve_gui_binary("windsurf") {
                    Self::launch_detached_editor(&exe.to_string_lossy(), path)
                } else if Self::is_bin_available("windsurf") {
                    Self::launch_detached_editor("windsurf", path)
                } else {
                    Err("Windsurf executable not found. Ensure Windsurf is installed.".into())
                }
            }
            "wt" => Self::open_in_terminal("wt", path),
            unknown => Err(format!("Unsupported editor identifier: {}", unknown)),
        }
    }

    /// Launches the selected terminal or CLI targeting the given path.
    pub fn open_in_terminal(terminal: &str, path: &str) -> Result<(), String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Target path does not exist: {}", path));
        }

        #[cfg(target_os = "windows")]
        {
            match terminal {
                "wt" => {
                    if Self::is_bin_available("wt") {
                        let mut cmd = Command::new("wt");
                        cmd.args(&["-d", path]);
                        cmd.creation_flags(CREATE_NO_WINDOW);
                        cmd.spawn().map_err(|e| format!("Failed to launch Windows Terminal: {}", e))?;
                        Ok(())
                    } else {
                        Self::open_in_terminal("powershell", path)
                    }
                }
                "powershell" => {
                    let escaped_path = path.replace('\'', "''");
                    let mut cmd = Command::new("cmd");
                    cmd.args(&["/C", "start", "powershell", "-NoExit", "-Command", &format!("Set-Location -LiteralPath '{}'", escaped_path)]);
                    cmd.creation_flags(CREATE_NO_WINDOW);
                    cmd.spawn().map_err(|e| format!("Failed to launch PowerShell: {}", e))?;
                    Ok(())
                }
                "cmd" => {
                    let mut cmd = Command::new("cmd");
                    cmd.args(&["/C", "start", "cmd", "/K", &format!("cd /D \"{}\"", path)]);
                    cmd.creation_flags(CREATE_NO_WINDOW);
                    cmd.spawn().map_err(|e| format!("Failed to launch CMD: {}", e))?;
                    Ok(())
                }
                "git-bash" => {
                    let standard_path = Path::new(r"C:\Program Files\Git\git-bash.exe");
                    let user_path = std::env::var("LOCALAPPDATA")
                        .map(|la| Path::new(&la).join("Programs").join("Git").join("git-bash.exe"))
                        .unwrap_or_else(|_| PathBuf::from(""));

                    let bash_exe = if standard_path.exists() {
                        Some(standard_path.to_string_lossy().to_string())
                    } else if user_path.exists() {
                        Some(user_path.to_string_lossy().to_string())
                    } else if Self::is_bin_available("git-bash") {
                        Some("git-bash".to_string())
                    } else {
                        None
                    };

                    if let Some(exe) = bash_exe {
                        let mut cmd = Command::new("cmd");
                        cmd.args(&["/C", "start", "", &exe, &format!("--cd={}", path)]);
                        cmd.creation_flags(CREATE_NO_WINDOW);
                        cmd.spawn().map_err(|e| format!("Failed to launch Git Bash: {}", e))?;
                        Ok(())
                    } else {
                        Self::open_in_terminal("powershell", path)
                    }
                }
                "agy" => {
                    if Self::is_bin_available("agy") {
                        let mut cmd = Command::new("cmd");
                        cmd.args(&["/C", "start", "agy", path]);
                        cmd.creation_flags(CREATE_NO_WINDOW);
                        cmd.spawn().map_err(|e| format!("Failed to launch AGY CLI: {}", e))?;
                        Ok(())
                    } else {
                        Err("AGY CLI is not available in PATH.".into())
                    }
                }
                "none" => Ok(()),
                unknown => Err(format!("Unsupported terminal identifier: {}", unknown)),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("x-terminal-emulator").arg("-e").arg(path).spawn().map_err(|e| e.to_string())?;
            Ok(())
        }
    }

    fn launch_detached_editor(bin: &str, path: &str) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            // 1. Try spawning directly with CREATE_NO_WINDOW and detached Stdio (instantaneous for .exe GUI apps)
            let mut cmd = Command::new(bin);
            cmd.arg(path);
            cmd.stdin(Stdio::null());
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
            cmd.creation_flags(CREATE_NO_WINDOW);
            if cmd.spawn().is_ok() {
                return Ok(());
            }

            // 2. Fallback: spawn via cmd /C start if bin is a batch/shell script
            let mut fallback = Command::new("cmd");
            fallback.args(&["/C", "start", "", bin, path]);
            fallback.stdin(Stdio::null());
            fallback.stdout(Stdio::null());
            fallback.stderr(Stdio::null());
            fallback.creation_flags(CREATE_NO_WINDOW);
            fallback.spawn().map_err(|e| format!("Failed to launch {}: {}", bin, e))?;
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new(bin).arg(path).spawn().map_err(|e| format!("Failed to launch {}: {}", bin, e))?;
            Ok(())
        }
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
            ("vscode", "VS Code", "code", "code"),
            ("antigravity", "Antigravity IDE", "antigravity", "bot"),
            ("cursor", "Cursor", "cursor", "sparkles"),
            ("windsurf", "Windsurf", "windsurf", "wind"),
            ("explorer", "File Explorer", "explorer", "folder"),
        ];

        editors
            .into_iter()
            .map(|(id, name, bin, icon)| {
                let available = if id == "explorer" {
                    true
                } else {
                    Self::resolve_gui_binary(id).is_some() || Self::is_bin_available(bin)
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
            // Filled in by the command layer via WorktreeCleanerService::build_single_worktree_info;
            // GitService cannot call it directly without a circular dependency on that module.
            worktree_info: WorktreeEntry::default(),
        })
    }

    /// Detaches HEAD in the given worktree (freeing its branch) after pre-flight dirty verification.
    pub fn detach_worktree_head(worktree_path: &str) -> Result<(), String> {
        let wt_path = Path::new(worktree_path);
        if !wt_path.exists() {
            return Err(format!("Worktree path does not exist: {}", worktree_path));
        }

        let (is_dirty, count) = Self::check_dirty_status(wt_path);
        if is_dirty {
            return Err(format!(
                "Cannot detach HEAD: {} uncommitted or modified files detected. Please stash or commit changes first.",
                count
            ));
        }

        Self::run_git(wt_path, &["checkout", "--detach"]).map(|_| ())
    }

    /// Stashes uncommitted and untracked changes in the given worktree.
    pub fn stash_worktree<P: AsRef<Path>>(
        worktree_path: P,
        message: Option<&str>,
    ) -> Result<String, String> {
        let wt = worktree_path.as_ref();
        if !wt.exists() {
            return Err(format!("Worktree path does not exist: {}", wt.display()));
        }

        let custom_msg = message.unwrap_or("Stash before branch switch - Workspace Companion");
        Self::run_git(wt, &["stash", "push", "-u", "-m", custom_msg])
    }

    /// Discards all tracked modifications and untracked files in the given worktree.
    pub fn discard_worktree_changes<P: AsRef<Path>>(
        worktree_path: P,
    ) -> Result<String, String> {
        let wt = worktree_path.as_ref();
        if !wt.exists() {
            return Err(format!("Worktree path does not exist: {}", wt.display()));
        }

        let reset_out = Self::run_git(wt, &["reset", "--hard", "HEAD"])?;
        let clean_out = Self::run_git(wt, &["clean", "-fd"])?;

        Ok(format!("{}\n{}", reset_out, clean_out).trim().to_string())
    }

    /// Unlocks a locked worktree in the repository.
    pub fn unlock_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_path: &str,
    ) -> Result<String, String> {
        let repo = repo_path.as_ref();
        Self::run_git(repo, &["worktree", "unlock", worktree_path])
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

    /// Whether `git worktree add` would refuse this target: an existing file, or a directory
    /// that is not empty. A missing or empty directory is a valid target.
    pub fn is_occupied_target(target: &Path) -> bool {
        if target.is_file() {
            return true;
        }
        std::fs::read_dir(target)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
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

        if Self::is_occupied_target(target_dir) {
            return Err(format!(
                "Target path '{}' already exists and is not empty.",
                target_path
            ));
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
            // Filled in by the command layer via WorktreeCleanerService::build_single_worktree_info;
            // GitService cannot call it directly without a circular dependency on that module.
            worktree_info: WorktreeEntry::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_occupied_target() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        assert!(!GitService::is_occupied_target(&missing));
        assert!(!GitService::is_occupied_target(dir.path()));

        let file = dir.path().join("file.txt");
        std::fs::write(&file, "x").unwrap();
        assert!(GitService::is_occupied_target(&file));
        assert!(GitService::is_occupied_target(dir.path()));
    }

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

    #[test]
    fn test_detect_installed_editors() {
        let editors = GitService::detect_installed_editors();
        assert!(!editors.is_empty());
        // Explorer should always be available
        let explorer = editors.iter().find(|e| e.id == "explorer");
        assert!(explorer.is_some());
        assert!(explorer.unwrap().is_available);
    }

    #[test]
    fn test_resolve_gui_binary_explorer_or_invalid() {
        assert!(GitService::resolve_gui_binary("non_existent_editor_123").is_none());
    }
}

