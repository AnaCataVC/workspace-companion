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
