use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use workspace_companion::services::git::GitService;
use workspace_companion::services::worktree_cleaner::{
    BatchDeleteTarget, WorktreeCleanerService,
};

/// Helper to execute git commands on a path without flashing console windows
fn run_git<P: AsRef<Path>>(dir: P, args: &[&str]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut cmd = Command::new("git");
    cmd.current_dir(dir.as_ref()).args(args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| format!("Failed to spawn git: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Helper that initializes a real temporary git repository with an initial commit on 'main'
fn create_test_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let p = temp.path();

    run_git(p, &["init", "-b", "main"]).expect("git init failed");
    run_git(p, &["config", "user.name", "Integration Tester"]).expect("config user.name failed");
    run_git(p, &["config", "user.email", "tester@workspace-companion.test"]).expect("config user.email failed");

    let readme = p.join("README.md");
    fs::write(&readme, "# Integration Test Workspace\n").expect("write readme failed");
    run_git(p, &["add", "README.md"]).expect("git add failed");
    run_git(p, &["commit", "-m", "Initial commit on main"]).expect("git commit failed");

    temp
}

#[test]
fn test_real_worktree_creation_and_listing() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap();

    let wt_temp = tempfile::tempdir().expect("failed to create wt temp dir");
    let wt_path = wt_temp.path().join("wt-feature");
    let wt_path_str = wt_path.to_str().unwrap();

    // 1. Create a real worktree via GitService
    let create_res = GitService::create_worktree(
        repo_path_str,
        wt_path_str,
        "main",
        Some("feat/e2e-test"),
    );
    assert!(create_res.is_ok(), "create_worktree failed: {:?}", create_res.err());

    let res = create_res.unwrap();
    assert!(res.success, "Result should mark success: {}", res.message);
    assert!(wt_path.exists(), "Worktree directory must exist on disk");

    // 2. Parse real porcelain worktree list
    let raw = GitService::run_git(repo_path_str, &["worktree", "list", "--porcelain"])
        .expect("raw list failed");
    let worktrees = GitService::parse_worktree_porcelain(&raw);
    assert_eq!(worktrees.len(), 2, "Should list exactly 2 worktrees");

    let main_wt = worktrees.iter().find(|w| w.branch.as_deref() == Some("refs/heads/main"));
    assert!(main_wt.is_some(), "Main branch worktree should be present");
    assert!(!main_wt.unwrap().bare);

    let feat_wt = worktrees.iter().find(|w| w.branch.as_deref() == Some("refs/heads/feat/e2e-test"));
    assert!(feat_wt.is_some(), "Feature branch worktree should be present");
    assert!(!feat_wt.unwrap().bare);
}

#[test]
fn test_dirty_worktree_safety_protection() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap();

    let wt_temp = tempfile::tempdir().expect("failed to create wt temp dir");
    let wt_path = wt_temp.path().join("wt-dirty-protection");
    let wt_path_str = wt_path.to_str().unwrap();

    GitService::create_worktree(
        repo_path_str,
        wt_path_str,
        "main",
        Some("feat/dirty-test"),
    ).expect("create wt failed");

    // Add uncommitted / dirty files
    let secret_file = wt_path.join("uncommitted_work.txt");
    fs::write(&secret_file, "critical developer changes").expect("write dirty file failed");

    // Pre-flight check dirty status
    let (is_dirty, uncommitted_count) = GitService::check_dirty_status(&wt_path);
    assert!(is_dirty, "Worktree must be detected as dirty");
    assert_eq!(uncommitted_count, 1, "Must detect exactly 1 uncommitted file");

    // Attempt removal WITHOUT force -> MUST be blocked and preserve data
    let remove_err = WorktreeCleanerService::remove_worktree(repo_path, wt_path_str, false);
    assert!(remove_err.is_err(), "remove_worktree must fail for dirty worktrees when force is false");
    let msg = remove_err.unwrap_err();
    assert!(msg.contains("dirty") || msg.contains("uncommitted"), "Error message should cite dirty state: {}", msg);

    // Verify critical file is still intact on disk
    assert!(secret_file.exists(), "Dirty file must NOT be deleted when force is false");
}

#[test]
fn test_worktree_removal_clean_and_forced() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap();

    // 1. Clean worktree removal succeeds without force
    let wt_clean_temp = tempfile::tempdir().expect("failed to create clean wt dir");
    let wt_clean_path = wt_clean_temp.path().join("wt-clean");
    let wt_clean_str = wt_clean_path.to_str().unwrap();

    GitService::create_worktree(
        repo_path_str,
        wt_clean_str,
        "main",
        Some("feat/clean-remove"),
    ).expect("create clean wt failed");
    assert!(wt_clean_path.exists());

    let clean_res = WorktreeCleanerService::remove_worktree(repo_path, wt_clean_str, false);
    assert!(clean_res.is_ok(), "Clean removal should succeed: {:?}", clean_res.err());
    assert!(!wt_clean_path.exists(), "Clean worktree directory must be removed");

    // 2. Forced removal succeeds even if worktree contains uncommitted modifications
    let wt_forced_temp = tempfile::tempdir().expect("failed to create forced wt dir");
    let wt_forced_path = wt_forced_temp.path().join("wt-forced");
    let wt_forced_str = wt_forced_path.to_str().unwrap();

    GitService::create_worktree(
        repo_path_str,
        wt_forced_str,
        "main",
        Some("feat/forced-remove"),
    ).expect("create forced wt failed");
    fs::write(wt_forced_path.join("dirty_draft.ts"), "export const a = 1;").unwrap();

    let forced_res = WorktreeCleanerService::remove_worktree(repo_path, wt_forced_str, true);
    assert!(forced_res.is_ok(), "Forced removal must succeed: {:?}", forced_res.err());
    assert!(!wt_forced_path.exists(), "Worktree directory must be removed after force delete");
}

#[test]
fn test_orphan_detection_lifecycle() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap();

    // Create a feature branch and worktree
    let wt_temp = tempfile::tempdir().expect("failed to create wt dir");
    let wt_path = wt_temp.path().join("wt-merged-orphan");
    let wt_path_str = wt_path.to_str().unwrap();

    GitService::create_worktree(
        repo_path_str,
        wt_path_str,
        "main",
        Some("feat/merged-feature"),
    ).expect("create wt failed");

    // Add a commit on the feature branch worktree
    let feat_file = wt_path.join("feature.txt");
    fs::write(&feat_file, "New feature complete\n").expect("write feat file failed");
    run_git(&wt_path, &["add", "feature.txt"]).expect("add feat failed");
    run_git(&wt_path, &["commit", "-m", "Complete feat/merged-feature"]).expect("commit feat failed");

    // Merge feature branch into main in the root repository
    run_git(repo_path, &["merge", "feat/merged-feature", "--no-ff", "-m", "Merge feat/merged-feature into main"])
        .expect("merge into main failed");

    // Scan repository via WorktreeCleanerService
    let repo_worktrees = WorktreeCleanerService::scan_repository(repo_path, None, None)
        .expect("scan_repository failed");

    // Verify main worktree is protected from orphan status
    let main_wt = repo_worktrees.worktrees.iter().find(|w| w.is_main).expect("main worktree not found");
    assert!(!main_wt.is_orphaned, "Root repository worktree must never be marked as orphaned");

    // Verify merged feature worktree is correctly detected as orphaned
    let orphan_wt = repo_worktrees.worktrees.iter().find(|w| {
        w.branch.as_deref() == Some("refs/heads/feat/merged-feature")
    }).expect("merged feature worktree not found");

    assert!(orphan_wt.is_orphaned, "Merged worktree must be flagged as orphaned");
    assert!(
        orphan_wt.orphan_reason.as_deref().unwrap_or("").contains("Merged into main"),
        "Orphan reason should cite merged into main: {:?}",
        orphan_wt.orphan_reason
    );
}

#[test]
fn test_batch_delete_skips_dirty_and_protects_main() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap().to_string();

    let wt_temp1 = tempfile::tempdir().unwrap();
    let wt_clean = wt_temp1.path().join("wt-batch-clean");
    let wt_clean_str = wt_clean.to_str().unwrap().to_string();

    let wt_temp2 = tempfile::tempdir().unwrap();
    let wt_dirty = wt_temp2.path().join("wt-batch-dirty");
    let wt_dirty_str = wt_dirty.to_str().unwrap().to_string();

    GitService::create_worktree(&repo_path_str, &wt_clean_str, "main", Some("feat/batch-clean")).unwrap();
    GitService::create_worktree(&repo_path_str, &wt_dirty_str, "main", Some("feat/batch-dirty")).unwrap();

    fs::write(wt_dirty.join("unsaved.txt"), "precious changes").unwrap();

    let targets = vec![
        // Target 1: Root repo (MUST be skipped)
        BatchDeleteTarget {
            repo_path: repo_path_str.clone(),
            worktree_path: repo_path_str.clone(),
            force: false,
        },
        // Target 2: Clean worktree (MUST be deleted)
        BatchDeleteTarget {
            repo_path: repo_path_str.clone(),
            worktree_path: wt_clean_str.clone(),
            force: false,
        },
        // Target 3: Dirty worktree with force: false (MUST be skipped)
        BatchDeleteTarget {
            repo_path: repo_path_str.clone(),
            worktree_path: wt_dirty_str.clone(),
            force: false,
        },
    ];

    let summary = WorktreeCleanerService::remove_worktrees_batch(targets);
    assert_eq!(summary.total_requested, 3);
    assert_eq!(summary.deleted_count, 1, "Only the clean worktree should be deleted");
    assert_eq!(summary.skipped_count, 2, "Main repo and dirty worktree must be skipped");
    assert!(!wt_clean.exists(), "Clean worktree must be deleted");
    assert!(wt_dirty.exists(), "Dirty worktree must remain on disk");
    assert!(repo_path.exists(), "Root repo must remain on disk");
}
