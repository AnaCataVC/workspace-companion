use crate::services::config::{AppConfig, WatchFolder};
use crate::services::git::GitService;
use crate::services::worktree_cleaner::WorktreeCleanerService;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionPreset {
    DiscardUncommitted,
    NukeWorktrees,
    CleanBranches,
    TotalFreshStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionScope {
    SingleRepo { repo_path: String },
    AllWatchedRepos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionPreview {
    pub preset: QuickActionPreset,
    pub repos_count: usize,
    pub worktrees_to_remove: Vec<String>,
    pub worktrees_to_discard: Vec<String>,
    pub dirty_files_count: usize,
    pub branches_to_delete: Vec<String>,
    pub unpushed_branches_count: usize,
    pub total_unpushed_commits: usize,
    pub protected_default_branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionResult {
    pub preset: QuickActionPreset,
    pub success: bool,
    pub worktrees_removed: usize,
    pub worktrees_discarded: usize,
    pub branches_deleted: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub struct QuickActionsService;

impl QuickActionsService {
    /// Resolves target repository root directories based on scope and watch folders.
    pub fn resolve_repos(
        scope: &QuickActionScope,
        watch_folders: &[WatchFolder],
    ) -> Result<Vec<PathBuf>, String> {
        match scope {
            QuickActionScope::SingleRepo { repo_path } => {
                let path = PathBuf::from(repo_path);
                if !path.exists() {
                    return Err(format!("Repository path does not exist: {}", repo_path));
                }
                if path.join(".git").is_dir() {
                    Ok(vec![path])
                } else if let Ok(top) = GitService::run_git(&path, &["rev-parse", "--show-toplevel"]) {
                    Ok(vec![PathBuf::from(top)])
                } else if path.join(".git").is_file() {
                    // Linked worktree; resolve common git dir / top-level
                    if let Ok(top) = GitService::run_git(&path, &["rev-parse", "--show-toplevel"]) {
                        Ok(vec![PathBuf::from(top)])
                    } else {
                        Ok(vec![path])
                    }
                } else {
                    Err(format!("Path is not a git repository: {}", repo_path))
                }
            }
            QuickActionScope::AllWatchedRepos => {
                let config = AppConfig {
                    watch_folders: watch_folders.to_vec(),
                    ..Default::default()
                };
                let discovered = WorktreeCleanerService::discover_repositories(&config);
                let paths: Vec<PathBuf> = discovered.into_iter().map(|r| r.path).collect();
                Ok(paths)
            }
        }
    }

    /// Computes a dry-run pre-flight audit for the selected preset and scope.
    pub fn compute_preview(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[WatchFolder],
    ) -> Result<QuickActionPreview, String> {
        let repos = Self::resolve_repos(scope, watch_folders)?;
        let repos_count = repos.len();

        if repos.is_empty() {
            return Ok(QuickActionPreview {
                preset,
                repos_count: 0,
                worktrees_to_remove: Vec::new(),
                worktrees_to_discard: Vec::new(),
                dirty_files_count: 0,
                branches_to_delete: Vec::new(),
                unpushed_branches_count: 0,
                total_unpushed_commits: 0,
                protected_default_branches: Vec::new(),
            });
        }

        // Rayon inter-repo parallel execution; intra-repo sequential
        let per_repo_previews: Vec<QuickActionPreview> = repos
            .par_iter()
            .map(|repo_root| Self::preview_single_repo(preset, repo_root))
            .collect();

        let mut worktrees_to_remove = Vec::new();
        let mut worktrees_to_discard = Vec::new();
        let mut dirty_files_count = 0;
        let mut branches_to_delete = Vec::new();
        let mut unpushed_branches_count = 0;
        let mut total_unpushed_commits = 0;
        let mut protected_default_branches = Vec::new();

        for prev in per_repo_previews {
            worktrees_to_remove.extend(prev.worktrees_to_remove);
            worktrees_to_discard.extend(prev.worktrees_to_discard);
            dirty_files_count += prev.dirty_files_count;
            branches_to_delete.extend(prev.branches_to_delete);
            unpushed_branches_count += prev.unpushed_branches_count;
            total_unpushed_commits += prev.total_unpushed_commits;
            protected_default_branches.extend(prev.protected_default_branches);
        }

        // Deduplicate protected default branch names for clean UI display
        let mut unique_defaults: Vec<String> = protected_default_branches
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();
        unique_defaults.sort();

        Ok(QuickActionPreview {
            preset,
            repos_count,
            worktrees_to_remove,
            worktrees_to_discard,
            dirty_files_count,
            branches_to_delete,
            unpushed_branches_count,
            total_unpushed_commits,
            protected_default_branches: unique_defaults,
        })
    }

    /// Previews a single repository.
    fn preview_single_repo(preset: QuickActionPreset, repo_root: &Path) -> QuickActionPreview {
        let default_branch =
            GitService::get_default_branch(repo_root).unwrap_or_else(|| "main".to_string());
        let protected_default_branches = vec![default_branch.clone()];

        let raw_wt = GitService::run_git(repo_root, &["worktree", "list", "--porcelain"])
            .unwrap_or_default();
        let mut worktrees = GitService::parse_worktree_porcelain(&raw_wt);
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }

        let mut worktrees_to_remove = Vec::new();
        let mut worktrees_to_discard = Vec::new();
        let mut dirty_files_count = 0;
        let mut branches_to_delete = Vec::new();
        let mut unpushed_branches_count = 0;
        let mut total_unpushed_commits = 0;

        match preset {
            QuickActionPreset::DiscardUncommitted => {
                for wt in &worktrees {
                    let (is_dirty, count) = GitService::check_dirty_status(&wt.path);
                    if is_dirty {
                        worktrees_to_discard.push(wt.path.clone());
                        dirty_files_count += count;
                    }
                }
            }
            QuickActionPreset::NukeWorktrees => {
                for wt in &worktrees {
                    if !wt.is_main {
                        worktrees_to_remove.push(wt.path.clone());
                        let (is_dirty, count) = GitService::check_dirty_status(&wt.path);
                        if is_dirty {
                            worktrees_to_discard.push(wt.path.clone());
                            dirty_files_count += count;
                        }
                    }
                }
            }
            QuickActionPreset::CleanBranches => {
                let checked_out_branches: HashSet<String> = worktrees
                    .iter()
                    .filter_map(|wt| wt.branch.as_ref().map(|b| b.replace("refs/heads/", "")))
                    .collect();

                let local_raw = GitService::run_git(
                    repo_root,
                    &["branch", "--list", "--format=%(refname:short)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    if checked_out_branches.contains(branch) {
                        continue;
                    }

                    branches_to_delete.push(branch.to_string());
                    let (is_unpushed, commits) = Self::calculate_unpushed_commits(repo_root, branch);
                    if is_unpushed {
                        unpushed_branches_count += 1;
                        total_unpushed_commits += commits;
                    }
                }
            }
            QuickActionPreset::TotalFreshStart => {
                for wt in &worktrees {
                    if !wt.is_main {
                        worktrees_to_remove.push(wt.path.clone());
                    }
                    let (is_dirty, count) = GitService::check_dirty_status(&wt.path);
                    if is_dirty {
                        worktrees_to_discard.push(wt.path.clone());
                        dirty_files_count += count;
                    }
                }

                let local_raw = GitService::run_git(
                    repo_root,
                    &["branch", "--list", "--format=%(refname:short)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    branches_to_delete.push(branch.to_string());
                    let (is_unpushed, commits) = Self::calculate_unpushed_commits(repo_root, branch);
                    if is_unpushed {
                        unpushed_branches_count += 1;
                        total_unpushed_commits += commits;
                    }
                }
            }
        }

        QuickActionPreview {
            preset,
            repos_count: 1,
            worktrees_to_remove,
            worktrees_to_discard,
            dirty_files_count,
            branches_to_delete,
            unpushed_branches_count,
            total_unpushed_commits,
            protected_default_branches,
        }
    }

    /// Calculates unpushed commits for a branch.
    /// Uses `git rev-list --count @{u}..HEAD` / `<branch>@{u}..<branch>` when upstream exists,
    /// or counts all commits on the branch if upstream is lacking.
    fn calculate_unpushed_commits(repo_root: &Path, branch: &str) -> (bool, usize) {
        let rev_spec = format!("{}@{{u}}..{}", branch, branch);
        match GitService::run_git(repo_root, &["rev-list", "--count", &rev_spec]) {
            Ok(output) => {
                let count = output.parse::<usize>().unwrap_or(0);
                (count > 0, count)
            }
            Err(_) => {
                // Branch lacks upstream: count all commits on this branch
                match GitService::run_git(repo_root, &["rev-list", "--count", branch]) {
                    Ok(output) => {
                        let count = output.parse::<usize>().unwrap_or(0);
                        (count > 0, count)
                    }
                    Err(_) => (false, 0),
                }
            }
        }
    }

    /// Executes the atomic cleanup pipeline across repositories.
    pub fn execute_action(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[WatchFolder],
    ) -> Result<QuickActionResult, String> {
        let repos = Self::resolve_repos(scope, watch_folders)?;

        if repos.is_empty() {
            return Ok(QuickActionResult {
                preset,
                success: true,
                worktrees_removed: 0,
                worktrees_discarded: 0,
                branches_deleted: 0,
                warnings: Vec::new(),
                errors: Vec::new(),
            });
        }

        // Rayon inter-repo parallel execution; intra-repo sequential
        let per_repo_results: Vec<QuickActionResult> = repos
            .par_iter()
            .map(|repo_root| Self::execute_single_repo(preset, repo_root))
            .collect();

        let mut worktrees_removed = 0;
        let mut worktrees_discarded = 0;
        let mut branches_deleted = 0;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        for res in per_repo_results {
            worktrees_removed += res.worktrees_removed;
            worktrees_discarded += res.worktrees_discarded;
            branches_deleted += res.branches_deleted;
            warnings.extend(res.warnings);
            errors.extend(res.errors);
        }

        let success = errors.is_empty();

        Ok(QuickActionResult {
            preset,
            success,
            worktrees_removed,
            worktrees_discarded,
            branches_deleted,
            warnings,
            errors,
        })
    }

    /// Executes the cleanup on a single repository.
    fn execute_single_repo(preset: QuickActionPreset, repo_root: &Path) -> QuickActionResult {
        let mut worktrees_removed = 0;
        let mut worktrees_discarded = 0;
        let mut branches_deleted = 0;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let default_branch =
            GitService::get_default_branch(repo_root).unwrap_or_else(|| "main".to_string());

        let raw_wt = GitService::run_git(repo_root, &["worktree", "list", "--porcelain"])
            .unwrap_or_default();
        let mut worktrees = GitService::parse_worktree_porcelain(&raw_wt);
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }

        match preset {
            QuickActionPreset::DiscardUncommitted => {
                for wt in &worktrees {
                    let (is_dirty, _) = GitService::check_dirty_status(&wt.path);
                    if is_dirty {
                        if let Err(e) = Self::discard_worktree_changes(&wt.path, &mut warnings) {
                            errors.push(e);
                        } else {
                            worktrees_discarded += 1;
                        }
                    }
                }
            }
            QuickActionPreset::NukeWorktrees => {
                for wt in &worktrees {
                    if wt.is_main {
                        // Main worktree immunity (Invariant 1)
                        continue;
                    }
                    let (is_dirty, _) = GitService::check_dirty_status(&wt.path);
                    if is_dirty {
                        let _ = Self::discard_worktree_changes(&wt.path, &mut warnings);
                    }

                    match GitService::run_git(
                        repo_root,
                        &["worktree", "remove", "--force", "--force", &wt.path],
                    ) {
                        Ok(_) => {
                            worktrees_removed += 1;
                        }
                        Err(e) => {
                            let lower = e.to_lowercase();
                            if lower.contains("permission denied")
                                || lower.contains("being used by another process")
                                || lower.contains("unlink")
                            {
                                warnings.push(format!("Lock warning removing {}: {}", wt.path, e));
                            } else {
                                errors.push(format!("Failed to remove {}: {}", wt.path, e));
                            }
                        }
                    }
                }
                let _ = GitService::run_git(repo_root, &["worktree", "prune"]);
            }
            QuickActionPreset::CleanBranches => {
                let checked_out_branches: HashSet<String> = worktrees
                    .iter()
                    .filter_map(|wt| wt.branch.as_ref().map(|b| b.replace("refs/heads/", "")))
                    .collect();

                let local_raw = GitService::run_git(
                    repo_root,
                    &["branch", "--list", "--format=%(refname:short)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    // Default branch immunity (Invariant 2)
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    if checked_out_branches.contains(branch) {
                        continue;
                    }

                    match GitService::run_git(repo_root, &["branch", "-D", branch]) {
                        Ok(_) => {
                            branches_deleted += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Failed to delete branch {}: {}", branch, e));
                        }
                    }
                }
            }
            QuickActionPreset::TotalFreshStart => {
                // 1. Discard dirty changes across all linked worktrees
                for wt in &worktrees {
                    if !wt.is_main {
                        let (is_dirty, _) = GitService::check_dirty_status(&wt.path);
                        if is_dirty {
                            if let Err(e) = Self::discard_worktree_changes(&wt.path, &mut warnings) {
                                errors.push(e);
                            } else {
                                worktrees_discarded += 1;
                            }
                        }
                    }
                }

                // 2. Remove all linked worktrees (--force --force) and prune
                for wt in &worktrees {
                    if !wt.is_main {
                        match GitService::run_git(
                            repo_root,
                            &["worktree", "remove", "--force", "--force", &wt.path],
                        ) {
                            Ok(_) => {
                                worktrees_removed += 1;
                            }
                            Err(e) => {
                                let lower = e.to_lowercase();
                                if lower.contains("permission denied")
                                    || lower.contains("being used by another process")
                                    || lower.contains("unlink")
                                {
                                    warnings.push(format!("Lock warning removing {}: {}", wt.path, e));
                                } else {
                                    errors.push(format!("Failed to remove {}: {}", wt.path, e));
                                }
                            }
                        }
                    }
                }
                let _ = GitService::run_git(repo_root, &["worktree", "prune"]);

                // 3. Switch main repository worktree to its default branch
                if let Err(_e) = GitService::run_git(repo_root, &["checkout", &default_branch]) {
                    // If checkout failed due to dirty working tree, discard first then retry
                    let _ = Self::discard_worktree_changes(&repo_root.to_string_lossy(), &mut warnings);
                    if let Err(e2) = GitService::run_git(repo_root, &["checkout", &default_branch]) {
                        errors.push(format!("Failed to checkout default branch {}: {}", default_branch, e2));
                    }
                }

                // 4. Discard uncommitted changes on the main repository worktree
                let (main_dirty, _) = GitService::check_dirty_status(repo_root);
                if main_dirty {
                    if let Err(e) = Self::discard_worktree_changes(&repo_root.to_string_lossy(), &mut warnings) {
                        errors.push(e);
                    } else {
                        worktrees_discarded += 1;
                    }
                }

                // 5. Delete all local branches except the default branch
                let local_raw = GitService::run_git(
                    repo_root,
                    &["branch", "--list", "--format=%(refname:short)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    match GitService::run_git(repo_root, &["branch", "-D", branch]) {
                        Ok(_) => {
                            branches_deleted += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Failed to delete branch {}: {}", branch, e));
                        }
                    }
                }
            }
        }

        QuickActionResult {
            preset,
            success: errors.is_empty(),
            worktrees_removed,
            worktrees_discarded,
            branches_deleted,
            warnings,
            errors,
        }
    }

    /// Discards uncommitted tracked and untracked changes in a worktree path.
    /// Strictly adheres to Invariant 3 (git clean -ffd, NEVER -x) and Invariant 6 (lock warnings).
    fn discard_worktree_changes(wt_path: &str, warnings: &mut Vec<String>) -> Result<(), String> {
        let path = Path::new(wt_path);
        if !path.exists() {
            return Ok(());
        }

        // 1. Revert tracked modifications
        if let Err(e) = GitService::run_git(path, &["reset", "--hard", "HEAD"]) {
            let lower = e.to_lowercase();
            if lower.contains("permission denied")
                || lower.contains("being used by another process")
                || lower.contains("unlink")
            {
                warnings.push(format!("Lock warning during reset in {}: {}", wt_path, e));
            } else {
                return Err(format!("git reset failed in {}: {}", wt_path, e));
            }
        }

        // 2. Remove untracked files without removing gitignored files (NO -x)
        if let Err(e) = GitService::run_git(path, &["clean", "-ffd"]) {
            let lower = e.to_lowercase();
            if lower.contains("permission denied")
                || lower.contains("being used by another process")
                || lower.contains("unlink")
            {
                warnings.push(format!("Lock warning during clean in {}: {}", wt_path, e));
            } else {
                return Err(format!("git clean failed in {}: {}", wt_path, e));
            }
        }

        Ok(())
    }
}
