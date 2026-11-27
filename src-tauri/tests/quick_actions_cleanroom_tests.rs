//! Cleanroom Black-Box Test Suite: Quick Actions & Bulk Cleanup
//! Contract: docs/contracts/quick-actions-and-bulk-cleanup.contract.md
//!
//! Formulated purely against formal interface contracts and behavioral acceptance criteria,
//! in strict isolation from internal service implementations.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use workspace_companion::services::quick_actions::{
    QuickActionPreset, QuickActionScope, QuickActionsService,
};

// ============================================================================
// Git Test Harness & Fixtures
// ============================================================================

struct GitTestEnv {
    _root_tempdir: TempDir,
    repo_path: PathBuf,
}

impl GitTestEnv {
    fn new() -> Self {
        let tempdir = tempfile::tempdir().expect("Failed to create temporary directory for git test");
        let repo_path = tempdir.path().join("main_repo");
        fs::create_dir_all(&repo_path).expect("Failed to create main repo folder");

        let env = Self {
            _root_tempdir: tempdir,
            repo_path,
        };

        // Initialize Git repository
        env.run_git(&["init"]);
        env.run_git(&["config", "user.name", "Cleanroom Tester"]);
        env.run_git(&["config", "user.email", "cleanroom@test.local"]);
        env.run_git(&["config", "commit.gpgsign", "false"]);
        env.run_git(&["config", "core.autocrlf", "false"]);

        // Create baseline README and initial commit on main
        let readme_path = env.repo_path.join("README.md");
        fs::write(&readme_path, "# Test Repository\nBaseline content.\n").unwrap();
        env.run_git(&["add", "README.md"]);
        env.run_git(&["commit", "-m", "Initial commit"]);
        env.run_git(&["branch", "-M", "main"]);

        env
    }

    fn run_git(&self, args: &[&str]) -> String {
        self.run_git_in_dir(&self.repo_path, args)
    }

    fn run_git_in_dir(&self, dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_AUTHOR_NAME", "Cleanroom Tester")
            .env("GIT_AUTHOR_EMAIL", "cleanroom@test.local")
            .env("GIT_COMMITTER_NAME", "Cleanroom Tester")
            .env("GIT_COMMITTER_EMAIL", "cleanroom@test.local")
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute git {:?} in {:?}: {}", args, dir, e));

        assert!(
            output.status.success(),
            "Git command failed: git {:?} in {:?}\nStderr: {}",
            args,
            dir,
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn repo_path_str(&self) -> String {
        self.repo_path.to_string_lossy().to_string()
    }

    fn add_worktree(&self, folder_name: &str, branch_name: &str) -> PathBuf {
        let wt_path = self._root_tempdir.path().join(folder_name);
        let wt_path_str = wt_path.to_string_lossy().to_string();
        self.run_git(&["worktree", "add", "-b", branch_name, &wt_path_str]);
        wt_path
    }

    fn add_worktree_existing_branch(&self, folder_name: &str, branch_name: &str) -> PathBuf {
        let wt_path = self._root_tempdir.path().join(folder_name);
        let wt_path_str = wt_path.to_string_lossy().to_string();
        self.run_git(&["worktree", "add", &wt_path_str, branch_name]);
        wt_path
    }

    /// Creates a local bare `origin` remote connected to `self.repo_path` and returns
    /// `(origin_bare_path, peer_clone_path)` for 100% offline upstream simulation.
    fn setup_bare_origin_and_peer(&self) -> (PathBuf, PathBuf) {
        let origin_bare = self._root_tempdir.path().join("origin.git");
        fs::create_dir_all(&origin_bare).expect("Failed to create bare origin directory");
        self.run_git_in_dir(&origin_bare, &["init", "--bare"]);
        self.run_git_in_dir(&origin_bare, &["symbolic-ref", "HEAD", "refs/heads/main"]);

        let origin_str = origin_bare.to_string_lossy().to_string();
        self.run_git(&["remote", "add", "origin", &origin_str]);
        self.run_git(&["push", "-u", "origin", "main"]);
        self.run_git(&["remote", "set-head", "origin", "main"]);

        let peer_repo = self._root_tempdir.path().join("peer_clone");
        let peer_str = peer_repo.to_string_lossy().to_string();
        self.run_git_in_dir(
            self._root_tempdir.path(),
            &["clone", &origin_str, &peer_str],
        );
        self.run_git_in_dir(&peer_repo, &["config", "user.name", "Cleanroom Peer"]);
        self.run_git_in_dir(&peer_repo, &["config", "user.email", "peer@test.local"]);
        self.run_git_in_dir(&peer_repo, &["config", "commit.gpgsign", "false"]);
        self.run_git_in_dir(&peer_repo, &["config", "core.autocrlf", "false"]);

        (origin_bare, peer_repo)
    }

    fn push_commit_from_dir(
        &self,
        dir: &Path,
        filename: &str,
        content: &str,
        message: &str,
    ) -> String {
        fs::write(dir.join(filename), content).expect("Failed to write file for commit");
        self.run_git_in_dir(dir, &["add", filename]);
        self.run_git_in_dir(dir, &["commit", "-m", message]);
        self.run_git_in_dir(dir, &["push", "origin", "HEAD"]);
        self.run_git_in_dir(dir, &["rev-parse", "HEAD"])
    }
}

fn paths_match(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

fn assert_contains_repo_path(list: &[String], expected: &Path) {
    assert!(
        list.iter().any(|entry| paths_match(Path::new(entry), expected)),
        "Expected repo path {:?} to be present in {:?}",
        expected,
        list
    );
}

// ============================================================================
// Scenario 1: DiscardUncommitted
// ============================================================================

#[test]
fn test_scenario_1_discard_uncommitted_restores_tracked_and_removes_untracked_preserving_ignored() {
    let env = GitTestEnv::new();

    // 1. Setup .gitignore with secrets and ignored directories
    let gitignore_path = env.repo_path.join(".gitignore");
    fs::write(&gitignore_path, ".env\nignored_dir/\n").unwrap();
    env.run_git(&["add", ".gitignore"]);
    env.run_git(&["commit", "-m", "Add .gitignore"]);

    // 2. Create ignored secrets (.env and ignored directory)
    let env_file_path = env.repo_path.join(".env");
    fs::write(&env_file_path, "SUPER_SECRET_KEY=123456\n").unwrap();

    let ignored_dir = env.repo_path.join("ignored_dir");
    fs::create_dir_all(&ignored_dir).unwrap();
    let ignored_nested_file = ignored_dir.join("local_config.json");
    fs::write(&ignored_nested_file, "{\"env\": \"local\"}").unwrap();

    // 3. Create modified tracked file
    let readme_path = env.repo_path.join("README.md");
    fs::write(&readme_path, "DIRTY MODIFICATION!\n").unwrap();

    // 4. Create untracked file and untracked folder
    let untracked_file = env.repo_path.join("scratch_untracked.txt");
    fs::write(&untracked_file, "untracked content").unwrap();

    let untracked_dir = env.repo_path.join("untracked_scratch_dir");
    fs::create_dir_all(&untracked_dir).unwrap();
    fs::write(untracked_dir.join("temp.log"), "log").unwrap();

    // Sanity check: git status shows modified tracked and untracked files
    let status_before = env.run_git(&["status", "--porcelain"]);
    assert!(status_before.contains("README.md"));
    assert!(status_before.contains("scratch_untracked.txt"));
    assert!(!status_before.contains(".env"), ".env must be ignored by git");

    // Execute DiscardUncommitted
    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };
    let result = QuickActionsService::execute_action(
        QuickActionPreset::DiscardUncommitted,
        &scope,
        &[],
    )
    .expect("execute_action should succeed");

    // Assertions according to Scenario 1 Contract:
    assert!(result.success, "Action result must be successful");
    assert_eq!(result.preset, QuickActionPreset::DiscardUncommitted);
    assert!(
        result.worktrees_discarded >= 1,
        "Should count at least 1 worktree discarded"
    );

    // 1. Every modified tracked file is restored to commit HEAD
    let readme_content = fs::read_to_string(&readme_path).unwrap();
    assert!(
        readme_content.contains("Baseline content."),
        "Tracked README.md must be restored to commit HEAD"
    );

    // 2. Untracked files and folders are deleted
    assert!(!untracked_file.exists(), "Untracked file must be removed");
    assert!(!untracked_dir.exists(), "Untracked directory must be removed");

    // 3. Files in .gitignore MUST NOT be deleted (No-Secrets Destruction Invariant)
    assert!(
        env_file_path.exists(),
        "CRITICAL: .env file MUST NOT be deleted by git clean"
    );
    assert_eq!(
        fs::read_to_string(&env_file_path).unwrap(),
        "SUPER_SECRET_KEY=123456\n"
    );
    assert!(
        ignored_nested_file.exists(),
        "Ignored directory contents MUST NOT be deleted"
    );
}

// ============================================================================
// Scenario 2: NukeWorktrees
// ============================================================================

#[test]
fn test_scenario_2_nuke_worktrees_removes_linked_worktrees_and_preserves_main_worktree() {
    let env = GitTestEnv::new();

    // 1. Create 2 linked worktrees
    let wt1_path = env.add_worktree("wt_feature_1", "feat/feature-1");
    let wt2_path = env.add_worktree("wt_feature_2", "feat/feature-2");

    // 2. Make wt1 dirty with untracked and modified files
    let wt1_file = wt1_path.join("wt1_dirty.txt");
    fs::write(&wt1_file, "dirty untracked file").unwrap();

    assert!(wt1_path.exists());
    assert!(wt2_path.exists());

    // Execute NukeWorktrees
    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };
    let result = QuickActionsService::execute_action(
        QuickActionPreset::NukeWorktrees,
        &scope,
        &[],
    )
    .expect("NukeWorktrees should succeed");

    // Assertions according to Scenario 2 Contract:
    assert!(result.success);
    assert_eq!(result.preset, QuickActionPreset::NukeWorktrees);
    assert_eq!(
        result.worktrees_removed, 2,
        "Must report exactly 2 linked worktrees removed"
    );

    // 1. Linked worktrees are removed from disk
    assert!(!wt1_path.exists(), "Linked worktree 1 must be removed from disk");
    assert!(!wt2_path.exists(), "Linked worktree 2 must be removed from disk");

    // 2. Main repository worktree MUST NOT be removed and MUST remain intact
    assert!(
        env.repo_path.exists(),
        "CRITICAL: Main repository worktree must remain intact"
    );
    assert!(
        env.repo_path.join(".git").exists(),
        "Main repo .git must remain intact"
    );

    // 3. git worktree prune was executed (only main worktree remains in porcelain list)
    let wt_list = env.run_git(&["worktree", "list", "--porcelain"]);
    let worktree_entries = wt_list
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .count();
    assert_eq!(
        worktree_entries, 1,
        "Only the main worktree must remain in git worktree list"
    );
}

// ============================================================================
// Scenario 3: CleanBranches
// ============================================================================

#[test]
fn test_scenario_3_clean_branches_deletes_secondary_branches_preserving_default_branch() {
    let env = GitTestEnv::new();

    // 1. Create multiple secondary branches
    env.run_git(&["branch", "feat/alpha"]);
    env.run_git(&["branch", "feat/beta"]);
    env.run_git(&["branch", "bugfix/gamma"]);

    // Ensure we are currently on main
    let current_branch = env.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(current_branch, "main");

    // Execute CleanBranches
    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };
    let result = QuickActionsService::execute_action(
        QuickActionPreset::CleanBranches,
        &scope,
        &[],
    )
    .expect("CleanBranches should succeed");

    // Assertions according to Scenario 3 Contract:
    assert!(result.success);
    assert_eq!(result.preset, QuickActionPreset::CleanBranches);
    assert_eq!(
        result.branches_deleted, 3,
        "All 3 secondary branches must be deleted"
    );

    // 1. Default branch is strictly preserved and NEVER deleted
    let branches = env.run_git(&["branch", "--list"]);
    assert!(
        branches.contains("main"),
        "CRITICAL: Default branch 'main' must NEVER be deleted"
    );

    // 2. Secondary branches are deleted
    assert!(!branches.contains("feat/alpha"));
    assert!(!branches.contains("feat/beta"));
    assert!(!branches.contains("bugfix/gamma"));
}

// ============================================================================
// Scenario 4: TotalFreshStart (Nuclear Wipe)
// ============================================================================

#[test]
fn test_scenario_4_total_fresh_start_nuclear_wipe_returns_pristine_default_branch() {
    let env = GitTestEnv::new();

    // 1. Create a linked worktree with dirty uncommitted changes
    let wt_path = env.add_worktree("wt_nuclear", "feat/nuclear-wt");
    fs::write(wt_path.join("uncommitted.txt"), "dirty linked file").unwrap();

    // 2. Create standalone extra branch
    env.run_git(&["branch", "feat/orphan-branch"]);

    // 3. Dirty the main worktree (modified tracked file and untracked file)
    fs::write(env.repo_path.join("README.md"), "MAIN DIRTY MODIFICATION").unwrap();
    fs::write(env.repo_path.join("scratch_temp.txt"), "temp untracked").unwrap();

    // Execute TotalFreshStart
    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };
    let result = QuickActionsService::execute_action(
        QuickActionPreset::TotalFreshStart,
        &scope,
        &[],
    )
    .expect("TotalFreshStart should succeed");

    // Assertions according to Scenario 4 Contract:
    assert!(result.success);
    assert_eq!(result.preset, QuickActionPreset::TotalFreshStart);
    assert!(result.worktrees_removed >= 1, "Must have removed linked worktree");
    assert!(result.branches_deleted >= 1, "Must have deleted secondary branches");

    // Phase 1 & 2: Linked worktree removed from disk
    assert!(!wt_path.exists(), "Linked worktree must be removed from disk");

    // Phase 3: Main worktree checked out on default branch
    let current_branch = env.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(current_branch, "main", "Must be checked out on main branch");

    // Phase 4: Discard uncommitted changes on main worktree (clean state)
    let status = env.run_git(&["status", "--porcelain"]);
    assert!(
        status.is_empty(),
        "Main repo must be in a pristine clean state. Status: {}",
        status
    );
    let readme_content = fs::read_to_string(env.repo_path.join("README.md")).unwrap();
    assert_eq!(
        readme_content.replace("\r\n", "\n"),
        "# Test Repository\nBaseline content.\n",
        "README.md must be restored to baseline HEAD"
    );
    assert!(!env.repo_path.join("scratch_temp.txt").exists());

    // Phase 5: Secondary branches deleted, default branch preserved
    let branches = env.run_git(&["branch", "--list"]);
    assert!(branches.contains("main"), "Default branch must exist");
    assert!(!branches.contains("feat/nuclear-wt"));
    assert!(!branches.contains("feat/orphan-branch"));
}

// ============================================================================
// Scenario 5: Pre-flight Audit Preview (compute_preview)
// ============================================================================

#[test]
fn test_scenario_5_compute_preview_audit_counts_match_repository_state() {
    let env = GitTestEnv::new();

    // 1. Setup repository with known artifacts:
    // - 2 linked worktrees
    let _wt1 = env.add_worktree("wt_preview_1", "feat/preview-1");
    let _wt2 = env.add_worktree("wt_preview_2", "feat/preview-2");

    // - 2 dirty files on main
    fs::write(env.repo_path.join("dirty_file_1.txt"), "dirty 1").unwrap();
    fs::write(env.repo_path.join("dirty_file_2.txt"), "dirty 2").unwrap();

    // - 1 standalone branch
    env.run_git(&["branch", "feat/preview-standalone"]);

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    // Audit Preview for DiscardUncommitted
    let preview_discard = QuickActionsService::compute_preview(
        QuickActionPreset::DiscardUncommitted,
        &scope,
        &[],
    )
    .expect("compute_preview DiscardUncommitted failed");

    assert_eq!(preview_discard.preset, QuickActionPreset::DiscardUncommitted);
    assert_eq!(preview_discard.repos_count, 1);
    assert_eq!(
        preview_discard.dirty_files_count, 2,
        "Must count exactly 2 dirty untracked files"
    );

    // Audit Preview for NukeWorktrees
    let preview_nuke = QuickActionsService::compute_preview(
        QuickActionPreset::NukeWorktrees,
        &scope,
        &[],
    )
    .expect("compute_preview NukeWorktrees failed");

    assert_eq!(preview_nuke.preset, QuickActionPreset::NukeWorktrees);
    assert_eq!(
        preview_nuke.worktrees_to_remove.len(), 2,
        "Must preview 2 linked worktrees for removal"
    );

    // Audit Preview for CleanBranches
    let preview_branches = QuickActionsService::compute_preview(
        QuickActionPreset::CleanBranches,
        &scope,
        &[],
    )
    .expect("compute_preview CleanBranches failed");

    assert_eq!(preview_branches.preset, QuickActionPreset::CleanBranches);
    assert!(
        preview_branches
            .protected_default_branches
            .contains(&"main".to_string()),
        "Protected default branches must list 'main'"
    );
    assert!(
        !preview_branches.branches_to_delete.contains(&"main".to_string()),
        "Default branch 'main' MUST NOT be scheduled for deletion"
    );

    // Audit Preview for TotalFreshStart
    let preview_total = QuickActionsService::compute_preview(
        QuickActionPreset::TotalFreshStart,
        &scope,
        &[],
    )
    .expect("compute_preview TotalFreshStart failed");

    assert_eq!(preview_total.preset, QuickActionPreset::TotalFreshStart);
    assert_eq!(preview_total.repos_count, 1);
    assert_eq!(preview_total.worktrees_to_remove.len(), 2);
    assert_eq!(preview_total.dirty_files_count, 2);
    assert!(
        preview_total.protected_default_branches.contains(&"main".to_string())
    );
}

// ============================================================================
// Boundary & Invariant Tests: Idempotency & Clean Repository
// ============================================================================

#[test]
fn test_invariant_clean_repo_idempotence() {
    let env = GitTestEnv::new();

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    // Discard on clean repo succeeds with 0 changes
    let res_discard = QuickActionsService::execute_action(
        QuickActionPreset::DiscardUncommitted,
        &scope,
        &[],
    )
    .expect("Discard on clean repo should succeed");
    assert!(res_discard.success);

    // Clean branches on repo with only default branch succeeds with 0 branches deleted
    let res_branches = QuickActionsService::execute_action(
        QuickActionPreset::CleanBranches,
        &scope,
        &[],
    )
    .expect("Clean branches on default branch should succeed");
    assert!(res_branches.success);
    assert_eq!(res_branches.branches_deleted, 0);

    // Nuke worktrees on repo with no linked worktrees succeeds with 0 removed
    let res_nuke = QuickActionsService::execute_action(
        QuickActionPreset::NukeWorktrees,
        &scope,
        &[],
    )
    .expect("Nuke worktrees on single worktree should succeed");
    assert!(res_nuke.success);
    assert_eq!(res_nuke.worktrees_removed, 0);
}

// ============================================================================
// Scenario 6: SyncDefaultBranch (Constructive Non-Destructive Sync)
// ============================================================================

#[test]
fn test_scenario_6_sync_default_branch_happy_path_fetch_prune_switch_and_ff_pull() {
    let env = GitTestEnv::new();
    let (_origin_bare, peer_repo) = env.setup_bare_origin_and_peer();

    // Seed a remote branch, fetch it into target repo, then delete it on origin to verify --prune
    env.run_git_in_dir(
        &peer_repo,
        &["push", "origin", "main:refs/heads/feat/ephemeral-remote"],
    );
    env.run_git(&["fetch", "origin"]);
    assert!(
        env.run_git(&["branch", "-r"])
            .contains("origin/feat/ephemeral-remote"),
        "Target repo should initially track origin/feat/ephemeral-remote"
    );
    env.run_git_in_dir(
        &peer_repo,
        &["push", "origin", "--delete", "feat/ephemeral-remote"],
    );

    // Add a linked worktree on another branch to verify SyncDefaultBranch never deletes worktrees
    let secondary_wt = env.add_worktree("wt_secondary_keep", "feat/secondary-keep");

    // Park root worktree cleanly on a feature branch
    env.run_git(&["checkout", "-b", "feat/wip"]);

    // Advance origin/main from the peer clone
    let upstream_sha = env.push_commit_from_dir(
        &peer_repo,
        "upstream_feature.txt",
        "upstream fast-forward payload\n",
        "Advance origin/main",
    );

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    // Pre-flight preview assertions
    let preview = QuickActionsService::compute_preview(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("compute_preview SyncDefaultBranch should succeed");

    assert_eq!(preview.preset, QuickActionPreset::SyncDefaultBranch);
    assert_eq!(preview.repos_count, 1);
    assert_contains_repo_path(&preview.repos_to_switch_to_default, &env.repo_path);
    assert_contains_repo_path(&preview.repos_ready_to_pull, &env.repo_path);
    assert!(
        preview.repos_skipped_dirty_or_locked.is_empty(),
        "Clean root on feat/wip should not be marked skipped"
    );

    // Execute SyncDefaultBranch
    let result = QuickActionsService::execute_action(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("execute_action SyncDefaultBranch should succeed");

    assert!(result.success);
    assert_eq!(result.preset, QuickActionPreset::SyncDefaultBranch);
    assert_eq!(result.repos_fetched, 1);
    assert_eq!(result.repos_switched_to_default, 1);
    assert_eq!(result.repos_pulled, 1);
    assert_eq!(result.repos_skipped, 0);
    assert_eq!(result.worktrees_removed, 0);
    assert_eq!(result.branches_deleted, 0);
    assert!(result.errors.is_empty(), "Expected no errors: {:?}", result.errors);

    // Root worktree is now on main at the new upstream commit
    assert_eq!(env.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert_eq!(env.run_git(&["rev-parse", "HEAD"]), upstream_sha);
    assert!(env.repo_path.join("upstream_feature.txt").exists());

    // Deleted remote branch reference was pruned by fetch --all --prune
    assert!(
        !env.run_git(&["branch", "-r"])
            .contains("origin/feat/ephemeral-remote"),
        "Deleted remote branch must be pruned from remote-tracking refs"
    );

    // Linked secondary worktree and local feature branch remain intact
    assert!(secondary_wt.exists(), "Linked secondary worktree must remain untouched");
    assert!(env.run_git(&["branch", "--list"]).contains("feat/wip"));
}

#[test]
fn test_scenario_6_sync_default_branch_already_up_to_date_on_main() {
    let env = GitTestEnv::new();
    let (_origin_bare, _peer_repo) = env.setup_bare_origin_and_peer();

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    let result = QuickActionsService::execute_action(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("SyncDefaultBranch on up-to-date main should succeed");

    assert!(result.success);
    assert_eq!(result.repos_fetched, 1);
    assert_eq!(result.repos_switched_to_default, 0);
    assert_eq!(result.repos_pulled, 0);
    assert_eq!(result.repos_up_to_date, 1);
    assert_eq!(result.repos_skipped, 0);
    assert!(result.errors.is_empty());
}

#[test]
fn test_scenario_6_sync_default_branch_dirty_root_guard_preserves_all_changes() {
    let env = GitTestEnv::new();
    let (_origin_bare, peer_repo) = env.setup_bare_origin_and_peer();

    // Switch root to feat/dirty and introduce both modified tracked and untracked files
    env.run_git(&["checkout", "-b", "feat/dirty"]);
    let readme_path = env.repo_path.join("README.md");
    let modified_readme = "UNCOMMITTED_WORK_IN_PROGRESS\n";
    fs::write(&readme_path, modified_readme).unwrap();

    let untracked_path = env.repo_path.join("draft_notes.txt");
    let untracked_body = "precious local notes\n";
    fs::write(&untracked_path, untracked_body).unwrap();

    // Advance origin/main from peer clone
    let _upstream_sha = env.push_commit_from_dir(
        &peer_repo,
        "remote_advance.txt",
        "remote update\n",
        "Upstream advance while local is dirty",
    );

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    // Verify preview flags the dirty root worktree as skipped
    let preview = QuickActionsService::compute_preview(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("compute_preview should succeed for dirty root");

    assert_contains_repo_path(&preview.repos_skipped_dirty_or_locked, &env.repo_path);
    assert!(preview.repos_to_switch_to_default.is_empty());
    assert!(preview.repos_ready_to_pull.is_empty());

    // Execute SyncDefaultBranch: fetches remote, safely skips checkout & pull with warning
    let result = QuickActionsService::execute_action(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("execute_action should succeed with safe skip");

    assert!(result.success);
    assert_eq!(result.repos_fetched, 1);
    assert_eq!(result.repos_skipped, 1);
    assert_eq!(result.repos_switched_to_default, 0);
    assert_eq!(result.repos_pulled, 0);
    assert!(result.errors.is_empty());
    assert!(!result.warnings.is_empty(), "Expected dirty root skip warning");

    // Zero data loss verification
    assert_eq!(env.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]), "feat/dirty");
    assert_eq!(fs::read_to_string(&readme_path).unwrap(), modified_readme);
    assert!(untracked_path.exists());
    assert_eq!(fs::read_to_string(&untracked_path).unwrap(), untracked_body);
}

#[test]
fn test_scenario_6_sync_default_branch_linked_worktree_branch_lock_guard() {
    let env = GitTestEnv::new();
    let (_origin_bare, peer_repo) = env.setup_bare_origin_and_peer();
    let _upstream_sha = env.push_commit_from_dir(
        &peer_repo,
        "locked_upstream.txt",
        "upstream commit\n",
        "Upstream commit while main is locked in linked worktree",
    );

    // Park root cleanly on feat/other so `main` can be checked out in a linked worktree
    env.run_git(&["checkout", "-b", "feat/other"]);
    let linked_main_wt = env.add_worktree_existing_branch("wt_main_locked", "main");
    assert!(linked_main_wt.exists());

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    // Preview must classify the repo under repos_skipped_dirty_or_locked
    let preview = QuickActionsService::compute_preview(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("compute_preview should succeed when main is locked in linked worktree");

    assert_contains_repo_path(&preview.repos_skipped_dirty_or_locked, &env.repo_path);
    assert!(preview.repos_to_switch_to_default.is_empty());
    assert!(preview.repos_ready_to_pull.is_empty());

    // Execute SyncDefaultBranch: must safely skip without deleting the secondary worktree
    let result = QuickActionsService::execute_action(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("execute_action should succeed when main is locked in linked worktree");

    assert!(result.success);
    assert_eq!(result.repos_skipped, 1);
    assert_eq!(result.repos_switched_to_default, 0);
    assert_eq!(result.repos_pulled, 0);
    assert_eq!(result.worktrees_removed, 0);
    assert!(result.errors.is_empty());
    assert!(
        !result.warnings.is_empty(),
        "Expected branch-lock warning naming secondary worktree"
    );
    assert!(
        linked_main_wt.exists(),
        "Secondary linked worktree checking out main must not be deleted"
    );
    assert_eq!(env.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]), "feat/other");
}

#[test]
fn test_scenario_6_sync_default_branch_diverged_non_fast_forward_guard() {
    let env = GitTestEnv::new();
    let (_origin_bare, peer_repo) = env.setup_bare_origin_and_peer();

    // Advance origin/main from peer clone
    let _remote_sha = env.push_commit_from_dir(
        &peer_repo,
        "diverged_remote.txt",
        "remote side\n",
        "Remote commit on main",
    );

    // Create a diverging local commit on main without pushing
    fs::write(env.repo_path.join("diverged_local.txt"), "local side\n").unwrap();
    env.run_git(&["add", "diverged_local.txt"]);
    env.run_git(&["commit", "-m", "Unpushed local commit on main"]);
    let local_sha = env.run_git(&["rev-parse", "HEAD"]);

    let scope = QuickActionScope::SingleRepo {
        repo_path: env.repo_path_str(),
    };

    let result = QuickActionsService::execute_action(
        QuickActionPreset::SyncDefaultBranch,
        &scope,
        &[],
    )
    .expect("SyncDefaultBranch on diverged main should safely skip");

    assert!(result.success);
    assert_eq!(result.repos_fetched, 1);
    assert_eq!(result.repos_skipped, 1);
    assert_eq!(result.repos_pulled, 0);
    assert_eq!(result.repos_up_to_date, 0);
    assert!(result.errors.is_empty());
    assert!(!result.warnings.is_empty(), "Expected non-fast-forward warning");

    // Verify no merge commit was created and working tree remains clean at local_sha
    assert_eq!(env.run_git(&["rev-parse", "HEAD"]), local_sha);
    let rev_parents = env.run_git(&["rev-list", "--parents", "-n", "1", "HEAD"]);
    assert_eq!(
        rev_parents.split_whitespace().count(),
        2,
        "HEAD must have exactly 1 parent (no merge commit)"
    );
    assert!(
        env.run_git(&["status", "--porcelain"]).is_empty(),
        "Working tree must remain completely clean"
    );
}

// ============================================================================
// Scenario 7: QuickActionScope::RepoList (Filtered Repository Subset)
// ============================================================================

#[test]
fn test_scenario_7_repo_list_subset_isolation_empty_noop_and_invalid_path_error() {
    let repo_a = GitTestEnv::new();
    let repo_b = GitTestEnv::new();
    let repo_c = GitTestEnv::new();

    for env in [&repo_a, &repo_b, &repo_c] {
        fs::write(env.repo_path.join("README.md"), "DIRTY SUBSET MODIFICATION\n").unwrap();
        fs::write(env.repo_path.join("temp_untracked.txt"), "untracked\n").unwrap();
    }

    // 1. Subset isolation: target ONLY repo_a and repo_b, leaving repo_c untouched
    let subset_scope = QuickActionScope::RepoList {
        repo_paths: vec![repo_a.repo_path_str(), repo_b.repo_path_str()],
    };

    let preview = QuickActionsService::compute_preview(
        QuickActionPreset::DiscardUncommitted,
        &subset_scope,
        &[],
    )
    .expect("compute_preview on RepoList subset should succeed");
    assert_eq!(preview.repos_count, 2);

    let result = QuickActionsService::execute_action(
        QuickActionPreset::DiscardUncommitted,
        &subset_scope,
        &[],
    )
    .expect("execute_action on RepoList subset should succeed");
    assert!(result.success);
    assert_eq!(result.worktrees_discarded, 2);

    // repo_a and repo_b must now be clean
    for cleaned in [&repo_a, &repo_b] {
        assert!(
            cleaned.run_git(&["status", "--porcelain"]).is_empty(),
            "Targeted repo in RepoList must be clean"
        );
        assert!(!cleaned.repo_path.join("temp_untracked.txt").exists());
    }

    // repo_c must remain completely untouched and dirty
    assert!(
        !repo_c.run_git(&["status", "--porcelain"]).is_empty(),
        "Excluded repo_c must remain dirty"
    );
    assert_eq!(
        fs::read_to_string(repo_c.repo_path.join("README.md")).unwrap(),
        "DIRTY SUBSET MODIFICATION\n"
    );
    assert!(repo_c.repo_path.join("temp_untracked.txt").exists());

    // 2. Empty repo_paths clean no-op
    let empty_scope = QuickActionScope::RepoList { repo_paths: vec![] };
    let resolved_empty = QuickActionsService::resolve_repos(&empty_scope, &[])
        .expect("resolve_repos on empty RepoList should return Ok");
    assert!(resolved_empty.is_empty());

    let empty_preview = QuickActionsService::compute_preview(
        QuickActionPreset::DiscardUncommitted,
        &empty_scope,
        &[],
    )
    .expect("compute_preview on empty RepoList should succeed");
    assert_eq!(empty_preview.repos_count, 0);

    let empty_result = QuickActionsService::execute_action(
        QuickActionPreset::DiscardUncommitted,
        &empty_scope,
        &[],
    )
    .expect("execute_action on empty RepoList should succeed as no-op");
    assert!(empty_result.success);
    assert_eq!(empty_result.worktrees_discarded, 0);

    // 3. Non-existent path returns Err(...)
    let nonexistent_path = repo_a
        ._root_tempdir
        .path()
        .join("missing_repository_dir")
        .to_string_lossy()
        .to_string();
    let invalid_scope = QuickActionScope::RepoList {
        repo_paths: vec![nonexistent_path],
    };

    assert!(
        QuickActionsService::resolve_repos(&invalid_scope, &[]).is_err(),
        "resolve_repos must return Err for non-existent path"
    );
    assert!(
        QuickActionsService::compute_preview(
            QuickActionPreset::DiscardUncommitted,
            &invalid_scope,
            &[],
        )
        .is_err(),
        "compute_preview must return Err for non-existent path"
    );
    assert!(
        QuickActionsService::execute_action(
            QuickActionPreset::DiscardUncommitted,
            &invalid_scope,
            &[],
        )
        .is_err(),
        "execute_action must return Err for non-existent path"
    );
}
