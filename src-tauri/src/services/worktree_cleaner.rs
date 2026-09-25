use crate::services::config::{AppConfig, ConfigService, WatchFolder};
use crate::services::git::{GitService, RepoOrphanContext, WorktreeEntry};
use crate::services::BatchItemErrorKind;
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
    pub kind: BatchItemErrorKind,
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

/// How git's worktree registry sees a removal target.
#[derive(Debug, PartialEq, Eq)]
enum TargetState {
    /// Listed, not prunable, and its `.git` link is intact: `git worktree remove` works.
    Registered,
    /// Listed but prunable (e.g. its `.git` file is gone): `git worktree remove` rejects it.
    StaleRegistration,
    /// Not listed at all: `git worktree remove` answers "is not a working tree".
    Unregistered,
}

/// Canonicalizes a path (resolving case and short names) without the verbatim prefix Windows
/// adds, falling back to the input when the path does not exist.
fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => {
            let text = p.to_string_lossy();
            match text.strip_prefix(r"\\?\") {
                Some(stripped) => PathBuf::from(stripped),
                None => p,
            }
        }
        Err(_) => path.to_path_buf(),
    }
}

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

        // 1. If watch folders exist, scan each enabled path recursively up to max_depth (1..5).
        // Each watch folder is walked independently, so the folder-level walk is parallelized
        // with rayon; cross-folder de-duplication still happens sequentially afterwards.
        let enabled_watches: Vec<&WatchFolder> =
            config.watch_folders.iter().filter(|w| w.enabled).collect();

        let per_watch_results: Vec<Vec<DiscoveredRepo>> = enabled_watches
            .par_iter()
            .map(|watch| Self::discover_repos_in_watch_folder(watch))
            .collect();

        for repo in per_watch_results.into_iter().flatten() {
            let canonical = repo.path.canonicalize().unwrap_or_else(|_| repo.path.clone());
            if seen_paths.insert(canonical) {
                results.push(repo);
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

    /// Walks a single watch folder for git repositories. Extracted out of `discover_repositories`
    /// so each watch folder can be walked on its own rayon thread.
    fn discover_repos_in_watch_folder(watch: &WatchFolder) -> Vec<DiscoveredRepo> {
        let mut local_results = Vec::new();

        let base_path = ConfigService::expand_path(&watch.path);
        if !base_path.exists() {
            return local_results;
        }

        // Direct check if watch folder itself is a Git repository
        if Self::is_git_repo(&base_path) {
            local_results.push(DiscoveredRepo {
                path: base_path.clone(),
                associated_account: watch.account_username.clone(),
                watch_folder_path: Some(watch.path.clone()),
            });
            return local_results;
        }

        if !base_path.is_dir() {
            return local_results;
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
                    local_results.push(DiscoveredRepo {
                        path: path.to_path_buf(),
                        associated_account: watch.account_username.clone(),
                        watch_folder_path: Some(watch.path.clone()),
                    });
                    // Stop deeper descent inside this Git repository
                    it.skip_current_dir();
                } else if path.join(".git").is_file() {
                    // Skip descending into linked worktrees or submodules
                    it.skip_current_dir();
                }
            }
        }

        local_results
    }

    /// Checks if a directory is a Git repository root.
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        let p = path.as_ref();
        p.join(".git").is_dir()
    }

    /// Marks the first entry of a `git worktree list --porcelain` parse as the main worktree.
    /// Git always lists the main/root worktree first, before any linked worktrees. Shared by
    /// `scan_repository` and `build_single_worktree_info` so both agree on how it's identified.
    fn mark_main_worktree(worktrees: &mut [WorktreeEntry]) {
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }
    }

    /// Scans a repository and extracts its worktrees with rich status metadata and account tagging.
    pub fn scan_repository<P: AsRef<Path>>(
        repo_path: P,
        associated_account: Option<String>,
        watch_folder_path: Option<String>,
    ) -> Result<Option<RepositoryWorktrees>, String> {
        let path = repo_path.as_ref();
        if !Self::is_git_repo(path) {
            return Ok(None);
        }

        let repo_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        let raw_worktrees = GitService::run_git(path, &["worktree", "list", "--porcelain"])?;

        let mut worktrees = GitService::parse_worktree_porcelain(&raw_worktrees);
        Self::mark_main_worktree(&mut worktrees);

        // The default branch, `branch -vv`, and `branch --merged` are identical no matter which
        // worktree in this repo is being checked, so this is computed once per repo instead of
        // once per worktree (see `RepoOrphanContext`).
        let orphan_context = GitService::build_repo_orphan_context(path);

        // Enrich each worktree with dirty status, orphan status, and commit info. With the
        // repo-wide git calls hoisted out above, the remaining per-worktree work is cheap enough
        // to parallelize across worktrees.
        worktrees
            .par_iter_mut()
            .for_each(|wt| Self::enrich_worktree_info(wt, &orphan_context));

        Ok(Some(RepositoryWorktrees {
            repo_path: path.to_string_lossy().to_string(),
            repo_name,
            associated_account,
            watch_folder_path,
            worktrees,
        }))
    }

    /// Enriches a single worktree entry with last-commit info, dirty status, and orphan status.
    /// Shared by the per-repo scan loop in `scan_repository` and by `build_single_worktree_info`,
    /// so both code paths compute a worktree's status the same way.
    fn enrich_worktree_info(worktree: &mut WorktreeEntry, orphan_context: &RepoOrphanContext) {
        let wt_path = Path::new(&worktree.path);

        // Commit info
        let (msg, author, date) = GitService::get_last_commit_info(wt_path);
        worktree.last_commit_message = msg;
        worktree.last_commit_author = author;
        worktree.last_commit_date = date;

        // Dirty status
        let (is_dirty, uncommitted_count) = GitService::check_dirty_status(wt_path);
        worktree.is_dirty = is_dirty;
        worktree.uncommitted_files_count = if is_dirty {
            Some(uncommitted_count)
        } else {
            None
        };

        // Branch-level status, reported for every worktree including the main one — it describes
        // the branch, not the worktree, so the "never orphaned" rule below does not apply to it.
        if let Some(branch_ref) = &worktree.branch {
            let short_branch = branch_ref.replace("refs/heads/", "");
            let (is_merged, is_remote_gone) =
                GitService::branch_status_flags(&short_branch, orphan_context);
            worktree.is_branch_merged = is_merged;
            worktree.is_branch_remote_gone = is_remote_gone;
        }

        // Orphan status (Root/main worktrees are never orphaned worktrees)
        if !worktree.is_main {
            if let Some(branch_ref) = &worktree.branch {
                let (is_orphan, reason) = GitService::check_orphan_status(branch_ref, orphan_context);
                worktree.is_orphaned = is_orphan;
                worktree.orphan_reason = reason;
            }
        } else {
            worktree.is_orphaned = false;
            worktree.orphan_reason = None;
        }
    }

    /// Builds a fresh `WorktreeEntry` for a single worktree path without rescanning the whole
    /// repository. Used after `create_worktree`/`checkout_worktree_branch` so the frontend can
    /// patch just the affected worktree into its in-memory list instead of triggering a full
    /// repo rescan.
    pub fn build_single_worktree_info(
        repo_path: &str,
        worktree_path: &str,
    ) -> Result<WorktreeEntry, String> {
        let repo_root = Path::new(repo_path);
        let raw_worktrees = GitService::run_git(repo_root, &["worktree", "list", "--porcelain"])?;
        let mut worktrees = GitService::parse_worktree_porcelain(&raw_worktrees);
        Self::mark_main_worktree(&mut worktrees);

        let target_canonical = Path::new(worktree_path)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(worktree_path));

        let mut worktree = worktrees
            .into_iter()
            .find(|wt| {
                let entry_canonical = Path::new(&wt.path)
                    .canonicalize()
                    .unwrap_or_else(|_| PathBuf::from(&wt.path));
                entry_canonical == target_canonical
            })
            .ok_or_else(|| format!("Worktree not found at path: {}", worktree_path))?;

        // Cheap here: unlike the repo-wide scan loop, only one worktree needs this context.
        let orphan_context = GitService::build_repo_orphan_context(repo_root);
        Self::enrich_worktree_info(&mut worktree, &orphan_context);

        Ok(worktree)
    }

    /// Removes a worktree safely.
    pub fn remove_worktree<P: AsRef<Path>>(
        repo_path: P,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, String> {
        let result = Self::remove_single_target(repo_path.as_ref(), worktree_path, force)
            .map_err(|(message, _)| message);
        let _ = GitService::run_git(repo_path.as_ref(), &["worktree", "prune"]);
        result
    }

    /// Normalizes a path for comparison against `git worktree list` output, which uses forward
    /// slashes and may differ in case or short-name form from what the frontend sends.
    fn comparable_path(path: &Path) -> String {
        let resolved = strip_verbatim_prefix(path);
        let text = resolved.to_string_lossy().replace('\\', "/");
        let text = text.trim_end_matches('/').to_string();
        if cfg!(windows) {
            text.to_lowercase()
        } else {
            text
        }
    }

    /// Classifies a removal target against git's own worktree registry.
    fn classify_target(repo_path: &Path, wt_path: &Path) -> Result<TargetState, String> {
        let wanted = Self::comparable_path(wt_path);
        let raw = GitService::run_git(repo_path, &["worktree", "list", "--porcelain"])?;
        let entry = GitService::parse_worktree_porcelain(&raw)
            .into_iter()
            .skip(1) // The first entry is always the main worktree.
            .find(|wt| Self::comparable_path(Path::new(&wt.path)) == wanted);

        Ok(match entry {
            Some(wt) if wt.prunable.is_none() && wt_path.join(".git").is_file() => {
                TargetState::Registered
            }
            Some(_) => TargetState::StaleRegistration,
            None => TargetState::Unregistered,
        })
    }

    /// Removes one target. Registered worktrees go through `git worktree remove`; a directory git
    /// cannot validate (its `.git` link is gone, or its registration was already pruned) makes
    /// that command fail, so it is deleted from disk only after git confirms it is not a
    /// registered worktree. Such a directory has no git status to prove it clean, so a non-empty
    /// one requires `force`.
    fn remove_single_target(
        repo_path: &Path,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, (String, BatchItemErrorKind)> {
        let wt_path = Path::new(worktree_path);
        let skipped = |message: String| (message, BatchItemErrorKind::Skipped);
        let failed = |message: String| (message, BatchItemErrorKind::Failed);

        // Protect main repository working tree
        if wt_path.join(".git").is_dir()
            || Self::comparable_path(repo_path) == Self::comparable_path(wt_path)
        {
            return Err(skipped(
                "Cannot remove the main working tree of a repository.".to_string(),
            ));
        }

        let mut state = Self::classify_target(repo_path, wt_path).map_err(failed)?;
        let had_stale_registration = state == TargetState::StaleRegistration;
        if had_stale_registration {
            GitService::run_git(repo_path, &["worktree", "prune"]).map_err(failed)?;
            state = Self::classify_target(repo_path, wt_path).map_err(failed)?;
            if state != TargetState::Unregistered {
                return Err(failed(format!(
                    "Worktree registration for {} could not be pruned; run `git worktree prune` manually.",
                    worktree_path
                )));
            }
        }

        if state == TargetState::Unregistered && !wt_path.exists() {
            return if had_stale_registration {
                Ok(format!("Stale worktree entry pruned: {}", worktree_path))
            } else {
                Err(failed(format!(
                    "{} is not a git worktree of this repository and does not exist.",
                    worktree_path
                )))
            };
        }

        if state == TargetState::Unregistered {
            return Self::remove_unregistered_directory(wt_path, worktree_path, force);
        }

        if !force {
            let (is_dirty, count) = GitService::check_dirty_status(wt_path);
            if is_dirty {
                return Err(skipped(format!(
                    "Cannot remove dirty worktree: {} uncommitted files detected. Please commit, stash, or enable force delete.",
                    count
                )));
            }
        }

        let mut args = vec!["worktree", "remove"];
        if force {
            // Passing --force twice allows removing both dirty and locked worktrees in Git
            args.push("--force");
            args.push("--force");
        }
        args.push(worktree_path);
        GitService::run_git(repo_path, &args).map_err(failed)
    }

    /// Deletes a directory git has confirmed is not a registered worktree. `git status` cannot
    /// run inside it (it would report the enclosing repository instead), so a non-empty
    /// directory is only deleted with `force`.
    fn remove_unregistered_directory(
        wt_path: &Path,
        worktree_path: &str,
        force: bool,
    ) -> Result<String, (String, BatchItemErrorKind)> {
        let is_empty = std::fs::read_dir(wt_path)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty && !force {
            return Err((
                format!(
                    "{} is not a registered git worktree (its .git link is missing), so its contents cannot be checked for uncommitted work. Enable force delete to remove the directory.",
                    worktree_path
                ),
                BatchItemErrorKind::Skipped,
            ));
        }
        std::fs::remove_dir_all(wt_path)
            .map(|_| format!("Removed unregistered worktree directory: {}", worktree_path))
            .map_err(|e| {
                (
                    format!("Failed to delete directory {}: {}", worktree_path, e),
                    BatchItemErrorKind::Failed,
                )
            })
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
                    match Self::remove_single_target(
                        Path::new(&repo_path),
                        &item.worktree_path,
                        item.force,
                    ) {
                        Ok(_) => local_deleted_paths.push(item.worktree_path),
                        Err((error, kind)) => {
                            if kind == BatchItemErrorKind::Skipped {
                                local_skipped += 1;
                            }
                            local_errors.push(BatchItemError {
                                worktree_path: item.worktree_path,
                                error,
                                kind,
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
