use crate::services::config::{AppConfig, ConfigService};
use crate::services::git::{GitService, WorktreeEntry};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredRepo {
    pub path: PathBuf,
    pub associated_account: Option<String>,
    pub watch_folder_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteTarget {
    pub repo_path: String,
    pub worktree_path: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItemError {
    pub worktree_path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteSummary {
    pub total_requested: usize,
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub deleted_paths: Vec<String>,
    pub errors: Vec<BatchItemError>,
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
                if entry.file_type().is_dir() {
                    if Self::is_git_repo(path) {
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
                    } else if path.join(".git").is_file() {
                        // Skip descending into linked worktrees or submodules
                        it.skip_current_dir();
                    }
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
        p.join(".git").is_dir()
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

        // Mark the root/main worktree (index 0 of git worktree list --porcelain)
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }

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

            // Orphan status (Root/main worktrees are never orphaned worktrees)
            if !wt.is_main {
                if let Some(branch_ref) = &wt.branch {
                    let (is_orphan, reason) = GitService::check_orphan_status(path, branch_ref);
                    wt.is_orphaned = is_orphan;
                    wt.orphan_reason = reason;
                }
            } else {
                wt.is_orphaned = false;
                wt.orphan_reason = None;
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
        let repo_p = repo_path.as_ref();
        let wt_path = Path::new(worktree_path);

        // Protect main repository working tree
        if wt_path.join(".git").is_dir() || repo_p == wt_path {
            return Err("Cannot remove the main working tree of a repository.".to_string());
        }

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

    /// Removes multiple worktrees safely and efficiently across repositories.
    /// Uses inter-repository parallelism while ensuring sequential execution per repo
    /// to avoid .git/worktrees lock collisions, with deferred single-prune per affected repo.
    pub fn remove_worktrees_batch(targets: Vec<BatchDeleteTarget>) -> BatchDeleteSummary {
        let total_requested = targets.len();
        if targets.is_empty() {
            return BatchDeleteSummary {
                total_requested: 0,
                deleted_count: 0,
                skipped_count: 0,
                deleted_paths: Vec::new(),
                errors: Vec::new(),
            };
        }

        // 1. Group targets by repository path to prevent Git lock collisions
        let mut grouped: HashMap<String, Vec<BatchDeleteTarget>> = HashMap::new();
        for target in targets {
            grouped
                .entry(target.repo_path.clone())
                .or_default()
                .push(target);
        }

        // 2. Parallelize inter-repo with Rayon; sequential intra-repo
        let repo_results: Vec<(Vec<String>, usize, Vec<BatchItemError>)> = grouped
            .into_par_iter()
            .map(|(repo_path, repo_targets)| {
                let mut local_deleted_paths = Vec::new();
                let mut local_skipped = 0;
                let mut local_errors = Vec::new();

                for item in repo_targets {
                    let wt_path = Path::new(&item.worktree_path);

                    // Protect main repository working tree
                    if wt_path.join(".git").is_dir() || Path::new(&repo_path) == wt_path {
                        local_skipped += 1;
                        local_errors.push(BatchItemError {
                            worktree_path: item.worktree_path.clone(),
                            error: "Cannot remove the main working tree of a repository.".to_string(),
                        });
                        continue;
                    }

                    // Pre-flight dirty check if force is false
                    if !item.force {
                        let (is_dirty, count) = GitService::check_dirty_status(wt_path);
                        if is_dirty {
                            local_skipped += 1;
                            local_errors.push(BatchItemError {
                                worktree_path: item.worktree_path.clone(),
                                error: format!(
                                    "Cannot remove dirty worktree: {} uncommitted files detected. Enable force delete to proceed.",
                                    count
                                ),
                            });
                            continue;
                        }
                    }

                    // git worktree remove [--force] <path>
                    let mut args = vec!["worktree", "remove"];
                    if item.force {
                        args.push("--force");
                    }
                    args.push(&item.worktree_path);

                    match GitService::run_git(Path::new(&repo_path), &args) {
                        Ok(_) => {
                            local_deleted_paths.push(item.worktree_path);
                        }
                        Err(err) => {
                            local_errors.push(BatchItemError {
                                worktree_path: item.worktree_path,
                                error: err,
                            });
                        }
                    }
                }

                // 3. Deferred single prune per affected repository
                let _ = GitService::run_git(Path::new(&repo_path), &["worktree", "prune"]);

                (local_deleted_paths, local_skipped, local_errors)
            })
            .collect();

        // 4. Consolidate results
        let mut deleted_paths = Vec::new();
        let mut skipped_count = 0;
        let mut errors = Vec::new();

        for (paths, skipped, mut errs) in repo_results {
            deleted_paths.extend(paths);
            skipped_count += skipped;
            errors.append(&mut errs);
        }

        let deleted_count = deleted_paths.len();

        BatchDeleteSummary {
            total_requested,
            deleted_count,
            skipped_count,
            deleted_paths,
            errors,
        }
    }

    /// Prunes stale worktree administrative files.
    pub fn prune_worktrees<P: AsRef<Path>>(repo_path: P) -> Result<String, String> {
        GitService::run_git(repo_path.as_ref(), &["worktree", "prune"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ignored_directories_contains_common_folders() {
        assert!(IGNORED_DIRECTORIES.contains(&"node_modules"));
        assert!(IGNORED_DIRECTORIES.contains(&"target"));
        assert!(IGNORED_DIRECTORIES.contains(&".venv"));
        assert!(IGNORED_DIRECTORIES.contains(&"dist"));
    }

    #[test]
    fn test_is_git_repo_detection() {
        let temp_dir = std::env::temp_dir().join("wt_cleaner_test_repo");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert!(!WorktreeCleanerService::is_git_repo(&temp_dir));

        // Create .git file (as in linked worktrees) -> must return false
        let git_file = temp_dir.join(".git");
        fs::write(&git_file, "gitdir: /path/to/main/.git/worktrees/wt1").unwrap();
        assert!(!WorktreeCleanerService::is_git_repo(&temp_dir));

        // Remove .git file and create .git directory (as in root repositories) -> must return true
        fs::remove_file(&git_file).unwrap();
        fs::create_dir_all(&git_file).unwrap();
        assert!(WorktreeCleanerService::is_git_repo(&temp_dir));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_batch_remove_empty_targets() {
        let summary = WorktreeCleanerService::remove_worktrees_batch(Vec::new());
        assert_eq!(summary.total_requested, 0);
        assert_eq!(summary.deleted_count, 0);
        assert_eq!(summary.skipped_count, 0);
        assert!(summary.deleted_paths.is_empty());
        assert!(summary.errors.is_empty());
    }

    #[test]
    fn test_remove_worktree_protects_main_repo() {
        let temp_dir = std::env::temp_dir().join("wt_cleaner_test_main_repo");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(temp_dir.join(".git")).unwrap();

        let path_str = temp_dir.to_string_lossy().to_string();
        let res = WorktreeCleanerService::remove_worktree(&temp_dir, &path_str, false);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("main"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
