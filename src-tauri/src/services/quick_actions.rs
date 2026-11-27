use crate::services::config::{AppConfig, WatchFolder};
use crate::services::gh::GhService;
use crate::services::git::GitService;
use crate::services::worktree_cleaner::WorktreeCleanerService;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionPreset {
    SyncDefaultBranch,
    DiscardUncommitted,
    NukeWorktrees,
    CleanBranches,
    TotalFreshStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionScope {
    SingleRepo {
        #[serde(rename = "repoPath", alias = "repo_path")]
        repo_path: String,
    },
    RepoList {
        #[serde(rename = "repoPaths", alias = "repo_paths")]
        repo_paths: Vec<String>,
    },
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
    #[serde(default)]
    pub repos_to_switch_to_default: Vec<String>,
    #[serde(default)]
    pub repos_ready_to_pull: Vec<String>,
    #[serde(default)]
    pub repos_skipped_dirty_or_locked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionResult {
    pub preset: QuickActionPreset,
    pub success: bool,
    pub worktrees_removed: usize,
    pub worktrees_discarded: usize,
    pub branches_deleted: usize,
    #[serde(default)]
    pub repos_fetched: usize,
    #[serde(default)]
    pub repos_switched_to_default: usize,
    #[serde(default)]
    pub repos_pulled: usize,
    #[serde(default)]
    pub repos_up_to_date: usize,
    #[serde(default)]
    pub repos_skipped: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl QuickActionResult {
    fn empty(preset: QuickActionPreset) -> Self {
        Self {
            preset,
            success: true,
            worktrees_removed: 0,
            worktrees_discarded: 0,
            branches_deleted: 0,
            repos_fetched: 0,
            repos_switched_to_default: 0,
            repos_pulled: 0,
            repos_up_to_date: 0,
            repos_skipped: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

pub struct QuickActionsService;

impl QuickActionsService {
    /// Normalizes a path for reliable set/map lookups while falling back to the raw path if needed.
    fn canonical_key(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    /// Deduplicates repository root paths while preserving insertion order.
    fn deduplicate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut unique = Vec::with_capacity(paths.len());
        for path in paths {
            if seen.insert(Self::canonical_key(&path)) {
                unique.push(path);
            }
        }
        unique
    }

    /// Validates and resolves a single repository or worktree path to its main repository root.
    fn resolve_single_repo_path(repo_path: &str) -> Result<PathBuf, String> {
        let trimmed = repo_path.trim();
        if trimmed.is_empty() {
            return Err("Repository path cannot be empty".to_string());
        }

        let path = PathBuf::from(trimmed);
        if !path.exists() {
            return Err(format!("Repository path does not exist: {}", repo_path));
        }

        if path.join(".git").is_dir() {
            return Ok(path);
        }

        if let Ok(raw_wt) = GitService::run_git(&path, &["worktree", "list", "--porcelain"]) {
            let worktrees = GitService::parse_worktree_porcelain(&raw_wt);
            if let Some(first) = worktrees.first() {
                return Ok(PathBuf::from(first.path.trim()));
            }
        }

        if let Ok(top) = GitService::run_git(&path, &["rev-parse", "--show-toplevel"]) {
            let top_trimmed = top.trim();
            if !top_trimmed.is_empty() {
                return Ok(PathBuf::from(top_trimmed));
            }
        }

        Err(format!("Path is not a git repository: {}", repo_path))
    }

    /// Resolves a repository's default branch, checking `symbolic-ref --short HEAD` for unborn
    /// or custom-initialized repositories before falling back to `"main"`.
    fn resolve_default_branch(repo_root: &Path) -> String {
        GitService::get_default_branch(repo_root)
            .or_else(|| {
                GitService::run_git(repo_root, &["symbolic-ref", "--short", "HEAD"])
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| "main".to_string())
    }

    /// Resolves target repository root directories based on scope and watch folders.
    pub fn resolve_repos(
        scope: &QuickActionScope,
        watch_folders: &[WatchFolder],
    ) -> Result<Vec<PathBuf>, String> {
        match scope {
            QuickActionScope::SingleRepo { repo_path } => {
                let resolved = Self::resolve_single_repo_path(repo_path)?;
                Ok(vec![resolved])
            }
            QuickActionScope::RepoList { repo_paths } => {
                if repo_paths.is_empty() {
                    return Ok(Vec::new());
                }
                let mut resolved = Vec::with_capacity(repo_paths.len());
                for repo_path in repo_paths {
                    resolved.push(Self::resolve_single_repo_path(repo_path)?);
                }
                Ok(Self::deduplicate_paths(resolved))
            }
            QuickActionScope::AllWatchedRepos => {
                let config = AppConfig {
                    watch_folders: watch_folders.to_vec(),
                    ..Default::default()
                };
                let discovered = WorktreeCleanerService::discover_repositories(&config);
                let paths: Vec<PathBuf> = discovered.into_iter().map(|r| r.path).collect();
                Ok(Self::deduplicate_paths(paths))
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
                repos_to_switch_to_default: Vec::new(),
                repos_ready_to_pull: Vec::new(),
                repos_skipped_dirty_or_locked: Vec::new(),
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
        let mut repos_to_switch_to_default = Vec::new();
        let mut repos_ready_to_pull = Vec::new();
        let mut repos_skipped_dirty_or_locked = Vec::new();

        for prev in per_repo_previews {
            worktrees_to_remove.extend(prev.worktrees_to_remove);
            worktrees_to_discard.extend(prev.worktrees_to_discard);
            dirty_files_count += prev.dirty_files_count;
            branches_to_delete.extend(prev.branches_to_delete);
            unpushed_branches_count += prev.unpushed_branches_count;
            total_unpushed_commits += prev.total_unpushed_commits;
            protected_default_branches.extend(prev.protected_default_branches);
            repos_to_switch_to_default.extend(prev.repos_to_switch_to_default);
            repos_ready_to_pull.extend(prev.repos_ready_to_pull);
            repos_skipped_dirty_or_locked.extend(prev.repos_skipped_dirty_or_locked);
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
            repos_to_switch_to_default,
            repos_ready_to_pull,
            repos_skipped_dirty_or_locked,
        })
    }

    /// Previews a single repository.
    fn preview_single_repo(preset: QuickActionPreset, repo_root: &Path) -> QuickActionPreview {
        let default_branch = Self::resolve_default_branch(repo_root);
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
        let mut repos_to_switch_to_default = Vec::new();
        let mut repos_ready_to_pull = Vec::new();
        let mut repos_skipped_dirty_or_locked = Vec::new();

        match preset {
            QuickActionPreset::SyncDefaultBranch => {
                let repo_root_str = repo_root.to_string_lossy().to_string();
                let (root_dirty, _) = GitService::check_dirty_status(repo_root);
                let locked_in_secondary = worktrees.iter().any(|wt| {
                    !wt.is_main
                        && wt
                            .branch
                            .as_deref()
                            .map(|b| b.trim_start_matches("refs/heads/"))
                            == Some(default_branch.as_str())
                });

                if root_dirty || locked_in_secondary {
                    repos_skipped_dirty_or_locked.push(repo_root_str);
                } else {
                    repos_ready_to_pull.push(repo_root_str.clone());

                    let root_branch = worktrees
                        .iter()
                        .find(|wt| wt.is_main)
                        .and_then(|wt| {
                            wt.branch
                                .as_deref()
                                .map(|b| b.trim_start_matches("refs/heads/").to_string())
                        })
                        .or_else(|| {
                            if worktrees.is_empty() {
                                GitService::run_git(repo_root, &["symbolic-ref", "--short", "HEAD"])
                                    .ok()
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                            } else {
                                None
                            }
                        });

                    if root_branch.as_deref() != Some(default_branch.as_str()) {
                        repos_to_switch_to_default.push(repo_root_str);
                    }
                }
            }
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
                    &["branch", "--list", "--format=%(refname:lstrip=2)"],
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
                    &["branch", "--list", "--format=%(refname:lstrip=2)"],
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
            repos_to_switch_to_default,
            repos_ready_to_pull,
            repos_skipped_dirty_or_locked,
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

    /// Executes the atomic pipeline across repositories using default config options.
    pub fn execute_action(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[WatchFolder],
    ) -> Result<QuickActionResult, String> {
        let config = AppConfig {
            watch_folders: watch_folders.to_vec(),
            ..Default::default()
        };
        Self::execute_action_with_config(preset, scope, &config)
    }

    /// Executes the atomic pipeline honoring full `AppConfig` (including `auto_switch_account`
    /// sequential GitHub CLI credential grouping for `SyncDefaultBranch`).
    pub fn execute_action_with_config(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        config: &AppConfig,
    ) -> Result<QuickActionResult, String> {
        let repos = Self::resolve_repos(scope, &config.watch_folders)?;

        if repos.is_empty() {
            return Ok(QuickActionResult::empty(preset));
        }

        let mut top_level_warnings = Vec::new();
        let per_repo_results: Vec<QuickActionResult> =
            if preset == QuickActionPreset::SyncDefaultBranch && config.auto_switch_account {
                let discovered = WorktreeCleanerService::discover_repositories(config);
                let mut repo_accounts: HashMap<PathBuf, Option<String>> = HashMap::new();
                for disc in discovered {
                    let acct = disc
                        .associated_account
                        .map(|a| a.trim().to_string())
                        .filter(|a| !a.is_empty());
                    repo_accounts.insert(Self::canonical_key(&disc.path), acct);
                }

                let mut groups: BTreeMap<Option<String>, Vec<PathBuf>> = BTreeMap::new();
                for repo in &repos {
                    let acct = repo_accounts
                        .get(&Self::canonical_key(repo))
                        .cloned()
                        .flatten();
                    groups.entry(acct).or_default().push(repo.clone());
                }

                let has_named_group = groups.keys().any(|k| k.is_some());
                if has_named_group {
                    let initial_active = GhService::get_accounts()
                        .ok()
                        .and_then(|accs| accs.into_iter().find(|a| a.active).map(|a| a.username));
                    let mut current_active = initial_active.clone();
                    let mut switched_any = false;
                    let mut combined = Vec::new();

                    for (group_account, group_repos) in groups {
                        if let Some(ref acc) = group_account {
                            let already_active = current_active
                                .as_deref()
                                .map(|cur| cur.eq_ignore_ascii_case(acc))
                                .unwrap_or(false);
                            if !already_active {
                                match GhService::switch_account(acc) {
                                    Ok(_) => {
                                        current_active = Some(acc.clone());
                                        switched_any = true;
                                    }
                                    Err(w) => {
                                        top_level_warnings.push(format!(
                                            "Could not switch GitHub CLI account to '{}': {}",
                                            acc, w
                                        ));
                                    }
                                }
                            }
                        }

                        let group_results: Vec<QuickActionResult> = group_repos
                            .par_iter()
                            .map(|repo_root| Self::execute_single_repo(preset, repo_root))
                            .collect();
                        combined.extend(group_results);
                    }

                    if let Some(ref orig) = initial_active {
                        let still_on_orig = current_active
                            .as_deref()
                            .map(|cur| cur.eq_ignore_ascii_case(orig))
                            .unwrap_or(false);
                        if switched_any && !still_on_orig {
                            let _ = GhService::switch_account(orig);
                        }
                    }

                    combined
                } else {
                    repos
                        .par_iter()
                        .map(|repo_root| Self::execute_single_repo(preset, repo_root))
                        .collect()
                }
            } else {
                // Rayon inter-repo parallel execution; intra-repo sequential
                repos
                    .par_iter()
                    .map(|repo_root| Self::execute_single_repo(preset, repo_root))
                    .collect()
            };

        let mut worktrees_removed = 0;
        let mut worktrees_discarded = 0;
        let mut branches_deleted = 0;
        let mut repos_fetched = 0;
        let mut repos_switched_to_default = 0;
        let mut repos_pulled = 0;
        let mut repos_up_to_date = 0;
        let mut repos_skipped = 0;
        let mut warnings = top_level_warnings;
        let mut errors = Vec::new();

        for res in per_repo_results {
            worktrees_removed += res.worktrees_removed;
            worktrees_discarded += res.worktrees_discarded;
            branches_deleted += res.branches_deleted;
            repos_fetched += res.repos_fetched;
            repos_switched_to_default += res.repos_switched_to_default;
            repos_pulled += res.repos_pulled;
            repos_up_to_date += res.repos_up_to_date;
            repos_skipped += res.repos_skipped;
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
            repos_fetched,
            repos_switched_to_default,
            repos_pulled,
            repos_up_to_date,
            repos_skipped,
            warnings,
            errors,
        })
    }

    /// Executes the non-destructive default branch sync on a single repository.
    fn execute_sync_default_branch(repo_root: &Path) -> QuickActionResult {
        let preset = QuickActionPreset::SyncDefaultBranch;
        let repo_display = repo_root.to_string_lossy().to_string();

        let mut repos_fetched = 0;
        let mut repos_switched_to_default = 0;
        let mut repos_pulled = 0;
        let mut repos_up_to_date = 0;
        let mut repos_skipped = 0;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let has_remote = GitService::run_git(repo_root, &["remote"])
            .map(|out| !out.trim().is_empty())
            .unwrap_or(false);

        if has_remote {
            match GitService::run_git(repo_root, &["fetch", "--all", "--prune"]) {
                Ok(_) => {
                    repos_fetched += 1;
                }
                Err(e) => {
                    errors.push(format!("Failed to fetch remotes in {}: {}", repo_display, e));
                    return QuickActionResult {
                        preset,
                        success: false,
                        worktrees_removed: 0,
                        worktrees_discarded: 0,
                        branches_deleted: 0,
                        repos_fetched,
                        repos_switched_to_default,
                        repos_pulled,
                        repos_up_to_date,
                        repos_skipped,
                        warnings,
                        errors,
                    };
                }
            }
        }

        let default_branch = Self::resolve_default_branch(repo_root);

        let raw_wt = GitService::run_git(repo_root, &["worktree", "list", "--porcelain"])
            .unwrap_or_default();
        let mut worktrees = GitService::parse_worktree_porcelain(&raw_wt);
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }

        if let Some(locked_wt) = worktrees.iter().find(|wt| {
            !wt.is_main
                && wt
                    .branch
                    .as_deref()
                    .map(|b| b.trim_start_matches("refs/heads/"))
                    == Some(default_branch.as_str())
        }) {
            repos_skipped += 1;
            warnings.push(format!(
                "Skipped {}: default branch '{}' is currently checked out in linked worktree {}",
                repo_display, default_branch, locked_wt.path
            ));
            return QuickActionResult {
                preset,
                success: true,
                worktrees_removed: 0,
                worktrees_discarded: 0,
                branches_deleted: 0,
                repos_fetched,
                repos_switched_to_default,
                repos_pulled,
                repos_up_to_date,
                repos_skipped,
                warnings,
                errors,
            };
        }

        let (is_dirty, dirty_count) = GitService::check_dirty_status(repo_root);
        if is_dirty {
            repos_skipped += 1;
            warnings.push(format!(
                "Skipped {}: root worktree has {} uncommitted or untracked file(s)",
                repo_display, dirty_count
            ));
            return QuickActionResult {
                preset,
                success: true,
                worktrees_removed: 0,
                worktrees_discarded: 0,
                branches_deleted: 0,
                repos_fetched,
                repos_switched_to_default,
                repos_pulled,
                repos_up_to_date,
                repos_skipped,
                warnings,
                errors,
            };
        }

        let current_branch = GitService::run_git(repo_root, &["symbolic-ref", "--short", "HEAD"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if current_branch.as_deref() != Some(default_branch.as_str()) {
            let checkout_res =
                GitService::run_git(repo_root, &["checkout", &default_branch]).or_else(|first_err| {
                    let local_ref = format!("refs/heads/{}", default_branch);
                    let origin_ref = format!("refs/remotes/origin/{}", default_branch);
                    let has_local = GitService::run_git(
                        repo_root,
                        &["show-ref", "--verify", "--quiet", &local_ref],
                    )
                    .is_ok();
                    let has_origin = GitService::run_git(
                        repo_root,
                        &["show-ref", "--verify", "--quiet", &origin_ref],
                    )
                    .is_ok();
                    if !has_local && has_origin {
                        let track_target = format!("origin/{}", default_branch);
                        GitService::run_git(
                            repo_root,
                            &["checkout", "-b", &default_branch, "--track", &track_target],
                        )
                    } else {
                        Err(first_err)
                    }
                });

            match checkout_res {
                Ok(_) => {
                    repos_switched_to_default += 1;
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to checkout default branch '{}' in {}: {}",
                        default_branch, repo_display, e
                    ));
                    return QuickActionResult {
                        preset,
                        success: false,
                        worktrees_removed: 0,
                        worktrees_discarded: 0,
                        branches_deleted: 0,
                        repos_fetched,
                        repos_switched_to_default,
                        repos_pulled,
                        repos_up_to_date,
                        repos_skipped,
                        warnings,
                        errors,
                    };
                }
            }
        }

        let upstream_spec = format!("{}@{{u}}", default_branch);
        let has_upstream =
            has_remote && GitService::run_git(repo_root, &["rev-parse", "--abbrev-ref", &upstream_spec]).is_ok();

        if !has_upstream {
            repos_up_to_date += 1;
        } else {
            let head_before = GitService::run_git(repo_root, &["rev-parse", "HEAD"])
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            match GitService::run_git(repo_root, &["pull", "--ff-only"]) {
                Ok(_) => {
                    let head_after = GitService::run_git(repo_root, &["rev-parse", "HEAD"])
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    if head_after != head_before {
                        repos_pulled += 1;
                    } else {
                        repos_up_to_date += 1;
                    }
                }
                Err(e) => {
                    let lower = e.to_lowercase();
                    if lower.contains("not possible to fast-forward")
                        || lower.contains("diverging")
                        || lower.contains("divergent")
                        || lower.contains("ff-only")
                        || lower.contains("fast-forward")
                    {
                        repos_skipped += 1;
                        warnings.push(format!(
                            "Skipped pull in {}: local branch '{}' has diverged from upstream (non-fast-forward)",
                            repo_display, default_branch
                        ));
                    } else {
                        errors.push(format!(
                            "Failed to pull default branch '{}' in {}: {}",
                            default_branch, repo_display, e
                        ));
                    }
                }
            }
        }

        QuickActionResult {
            preset,
            success: errors.is_empty(),
            worktrees_removed: 0,
            worktrees_discarded: 0,
            branches_deleted: 0,
            repos_fetched,
            repos_switched_to_default,
            repos_pulled,
            repos_up_to_date,
            repos_skipped,
            warnings,
            errors,
        }
    }

    /// Executes the selected preset on a single repository.
    fn execute_single_repo(preset: QuickActionPreset, repo_root: &Path) -> QuickActionResult {
        if preset == QuickActionPreset::SyncDefaultBranch {
            return Self::execute_sync_default_branch(repo_root);
        }

        let mut worktrees_removed = 0;
        let mut worktrees_discarded = 0;
        let mut branches_deleted = 0;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let default_branch = Self::resolve_default_branch(repo_root);

        let raw_wt = GitService::run_git(repo_root, &["worktree", "list", "--porcelain"])
            .unwrap_or_default();
        let mut worktrees = GitService::parse_worktree_porcelain(&raw_wt);
        if let Some(first) = worktrees.first_mut() {
            first.is_main = true;
        }

        match preset {
            QuickActionPreset::SyncDefaultBranch => unreachable!(),
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
                        // Main worktree immunity
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
                            let p = Path::new(&wt.path);
                            if p.exists() {
                                let _ = GitService::run_git(repo_root, &["worktree", "prune"]);
                                if WorktreeCleanerService::force_remove_directory(p).is_ok() {
                                    worktrees_removed += 1;
                                    continue;
                                }
                            }
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
                    &["branch", "--list", "--format=%(refname:lstrip=2)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    // Default branch immunity
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    if checked_out_branches.contains(branch) {
                        continue;
                    }

                    match GitService::delete_branch(repo_root, branch, true) {
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
                                let p = Path::new(&wt.path);
                                if p.exists() {
                                    let _ = GitService::run_git(repo_root, &["worktree", "prune"]);
                                    if WorktreeCleanerService::force_remove_directory(p).is_ok() {
                                        worktrees_removed += 1;
                                        continue;
                                    }
                                }
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
                    &["branch", "--list", "--format=%(refname:lstrip=2)"],
                )
                .unwrap_or_default();

                for line in local_raw.lines().filter(|l| !l.trim().is_empty()) {
                    let branch = line.trim();
                    if branch == default_branch || branch == "main" || branch == "master" {
                        continue;
                    }
                    match GitService::delete_branch(repo_root, branch, true) {
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
            repos_fetched: 0,
            repos_switched_to_default: 0,
            repos_pulled: 0,
            repos_up_to_date: 0,
            repos_skipped: 0,
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
