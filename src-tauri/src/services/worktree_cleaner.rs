use crate::services::git::{GitService, WorktreeEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryWorktrees {
    #[serde(rename = "repoPath")]
    pub repo_path: String,
    #[serde(rename = "repoName")]
    pub repo_name: String,
    pub worktrees: Vec<WorktreeEntry>,
}

pub struct WorktreeCleanerService;

impl WorktreeCleanerService {
    /// Discovers git repositories in common developer directories (e.g., Repos / user workspace).
    pub fn discover_repositories() -> Vec<PathBuf> {
        let mut repos = Vec::new();

        // 1. Current workspace directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if Self::is_git_repo(&current_dir) {
            repos.push(current_dir);
        }

        // 2. Repos directory in UserProfile (standard Windows dev path: ~/Repos)
        if let Ok(home) = std::env::var("USERPROFILE") {
            let repos_dir = PathBuf::from(&home).join("Repos");
            if repos_dir.exists() && repos_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&repos_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && Self::is_git_repo(&path) {
                            if !repos.contains(&path) {
                                repos.push(path);
                            }
                        }
                    }
                }
            }

            // Also check Archivos Trabajo (work folder)
            let work_dir = PathBuf::from(&home).join("Archivos Trabajo");
            if work_dir.exists() && work_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&work_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && Self::is_git_repo(&path) {
                            if !repos.contains(&path) {
                                repos.push(path);
                            }
                        }
                    }
                }
            }
        }

        repos
    }

    /// Checks if a directory is a Git repository root.
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        let p = path.as_ref();
        p.join(".git").exists()
    }

    /// Scans a repository and extracts its worktrees with rich status metadata.
    pub fn scan_repository<P: AsRef<Path>>(repo_path: P) -> Option<RepositoryWorktrees> {
        let path = repo_path.as_ref();
        if !Self::is_git_repo(path) {
            return None;
        }

        let repo_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        let raw_worktrees = match GitService::run_git(path, &["worktree", "list", "--porcelain"]) {
            Ok(output) => output,
            Err(_) => return None,
        };

        let mut worktrees = GitService::parse_worktree_porcelain(&raw_worktrees);

        // Enrich each worktree with dirty status, orphan status, and commit info
        for wt in &mut worktrees {
            let wt_path = Path::new(&wt.path);

            // Commit info
            let (msg, author, date) = GitService::get_last_commit_info(wt_path);
            wt.last_commit_message = msg;
            wt.last_commit_author = author;
            wt.last_commit_date = date;

            // Dirty status
            let (is_dirty, uncommitted_count) = GitService::check_dirty_status(wt_path);
            wt.is_dirty = is_dirty;
            wt.uncommitted_files_count = if is_dirty { Some(uncommitted_count) } else { None };

            // Orphan status
            if let Some(branch_ref) = &wt.branch {
                let (is_orphan, reason) = GitService::check_orphan_status(path, branch_ref);
                wt.is_orphaned = is_orphan;
                wt.orphan_reason = reason;
            }
        }

        Some(RepositoryWorktrees {
            repo_path: path.to_string_lossy().to_string(),
            repo_name,
            worktrees,
        })
    }

    /// Removes a worktree safely.
    pub fn remove_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, String> {
        let wt_path = Path::new(worktree_path);

        if !force {
            let (is_dirty, count) = GitService::check_dirty_status(wt_path);
            if is_dirty {
                return Err(format!(
                    "Cannot remove dirty worktree: {} uncommitted files detected. Please commit, stash, or enable force delete.",
                    count
                ));
            }
        }

        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(worktree_path);

        let result = GitService::run_git(repo_path.as_ref(), &args)?;

        // Run git worktree prune afterwards
        let _ = GitService::run_git(repo_path.as_ref(), &["worktree", "prune"]);

        Ok(result)
    }

    /// Prunes stale worktree administrative files.
    pub fn prune_worktrees<P: AsRef<Path>>(repo_path: P) -> Result<String, String> {
        GitService::run_git(repo_path.as_ref(), &["worktree", "prune"])
    }
}
