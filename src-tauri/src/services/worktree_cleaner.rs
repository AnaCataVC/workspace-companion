use crate::services::config::{AppConfig, ConfigService};
use crate::services::git::{GitService, WorktreeEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const IGNORED_DIRECTORIES: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    ".terraform",
    ".venv",
    "venv",
    "bin",
    "obj",
    ".git",
    ".svelte-kit",
    "coverage",
    ".cache",
    ".next",
    ".nuxt",
    ".turbo",
    "vendor",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWorktrees {
    pub repo_path: String,
    pub repo_name: String,
    pub associated_account: Option<String>,
    pub watch_folder_path: Option<String>,
    pub worktrees: Vec<WorktreeEntry>,
}

pub struct DiscoveredRepo {
    pub path: PathBuf,
    pub associated_account: Option<String>,
    pub watch_folder_path: Option<String>,
}

pub struct WorktreeCleanerService;

impl WorktreeCleanerService {
    fn should_skip_dir(entry: &walkdir::DirEntry) -> bool {
        if entry.depth() == 0 {
            return false;
        }
        let file_name = entry.file_name().to_string_lossy();
        if file_name.starts_with('.') && file_name != ".git" {
            return true;
        }
        IGNORED_DIRECTORIES
            .iter()
            .any(|&ignored| ignored.eq_ignore_ascii_case(&file_name))
    }

    /// Discovers git repositories dynamically using the loaded AppConfig.
    pub fn discover_repositories(config: &AppConfig) -> Vec<DiscoveredRepo> {
        let mut results = Vec::new();
        let mut seen_paths = HashSet::new();

        // 1. If watch folders exist, scan each enabled path recursively up to max_depth (1..5)
        for watch in &config.watch_folders {
            if !watch.enabled {
                continue;
            }

            let base_path = ConfigService::expand_path(&watch.path);
            if !base_path.exists() {
                continue;
            }

            // Direct check if watch folder itself is a Git repository
            if Self::is_git_repo(&base_path) {
                let canonical = base_path
                    .canonicalize()
                    .unwrap_or_else(|_| base_path.clone());
                if seen_paths.insert(canonical) {
                    results.push(DiscoveredRepo {
                        path: base_path.clone(),
                        associated_account: watch.account_username.clone(),
                        watch_folder_path: Some(watch.path.clone()),
                    });
                }
                continue;
            }

            if !base_path.is_dir() {
                continue;
            }

            let max_depth = (watch.max_depth.clamp(1, 5)) as usize;
            let mut it = WalkDir::new(&base_path)
                .max_depth(max_depth)
                .into_iter()
                .filter_entry(|e| !Self::should_skip_dir(e));

            while let Some(Ok(entry)) = it.next() {
                let path = entry.path();
                if entry.file_type().is_dir() && Self::is_git_repo(path) {
                    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                    if seen_paths.insert(canonical) {
                        results.push(DiscoveredRepo {
                            path: path.to_path_buf(),
                            associated_account: watch.account_username.clone(),
                            watch_folder_path: Some(watch.path.clone()),
                        });
                    }
                    // Stop deeper descent inside this Git repository
                    it.skip_current_dir();
                }
            }
        }

        // 2. Fallback: Always include current workspace directory if it is a Git repo
        if let Ok(current_dir) = std::env::current_dir() {
            if Self::is_git_repo(&current_dir) {
                let canonical = current_dir
                    .canonicalize()
                    .unwrap_or_else(|_| current_dir.clone());
                if seen_paths.insert(canonical) {
                    results.push(DiscoveredRepo {
                        path: current_dir,
                        associated_account: None,
                        watch_folder_path: None,
                    });
                }
            }
        }

        results
    }

    /// Checks if a directory is a Git repository root.
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        let p = path.as_ref();
        p.join(".git").exists()
    }

    /// Scans a repository and extracts its worktrees with rich status metadata and account tagging.
    pub fn scan_repository<P: AsRef<Path>>(
        repo_path: P,
        associated_account: Option<String>,
        watch_folder_path: Option<String>,
    ) -> Option<RepositoryWorktrees> {
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
            wt.uncommitted_files_count = if is_dirty {
                Some(uncommitted_count)
            } else {
                None
            };

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
            associated_account,
            watch_folder_path,
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
