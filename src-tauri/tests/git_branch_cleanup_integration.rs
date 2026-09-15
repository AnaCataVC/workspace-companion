use std::fs;
use workspace_companion::services::branch_cleaner::{BranchCleanerService, BranchDeleteTarget};
use workspace_companion::services::git::GitService;

mod common;
use common::{create_test_repo, run_git};

#[test]
fn test_branch_status_classification() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap();

    // Merged branch: committed via a throwaway worktree, merged into main, worktree removed
    // afterwards — so at scan time it's a plain merged branch with no worktree of its own.
    run_git(repo_path, &["branch", "feat/merged-no-worktree"]).expect("create merged branch failed");
    let wt_temp = tempfile::tempdir().expect("failed to create wt temp dir");
    let wt_path = wt_temp.path().join("wt-merge-source");
    GitService::create_worktree(
        repo_path_str,
        wt_path.to_str().unwrap(),
        "feat/merged-no-worktree",
        None,
    )
    .expect("create worktree for merge source failed");
    fs::write(wt_path.join("feature.txt"), "done\n").expect("write feature file failed");
    run_git(&wt_path, &["add", "feature.txt"]).expect("add failed");
    run_git(&wt_path, &["commit", "-m", "Complete feat/merged-no-worktree"]).expect("commit failed");
    run_git(repo_path, &["worktree", "remove", "--force", wt_path.to_str().unwrap()])
        .expect("remove merge-source worktree failed");
    run_git(
        repo_path,
        &["merge", "feat/merged-no-worktree", "--no-ff", "-m", "Merge feat/merged-no-worktree"],
    )
    .expect("merge into main failed");

    // Unmerged branch with no worktree — needs a commit that main doesn't have, otherwise it's
    // trivially "merged" (identical to main) by git's own definition. Built via a throwaway
    // worktree, then the worktree is removed without merging the commit back.
    let open_wt_temp = tempfile::tempdir().expect("failed to create open-branch wt dir");
    let open_wt_path = open_wt_temp.path().join("wt-open-source");
    GitService::create_worktree(
        repo_path_str,
        open_wt_path.to_str().unwrap(),
        "main",
        Some("feat/still-open"),
    )
    .expect("create worktree for open branch failed");
    fs::write(open_wt_path.join("wip.txt"), "not merged\n").expect("write wip file failed");
    run_git(&open_wt_path, &["add", "wip.txt"]).expect("add failed");
    run_git(&open_wt_path, &["commit", "-m", "WIP on feat/still-open"]).expect("commit failed");
    run_git(repo_path, &["worktree", "remove", "--force", open_wt_path.to_str().unwrap()])
        .expect("remove open-branch worktree failed");

    let statuses = GitService::list_local_branches_with_status(repo_path_str)
        .expect("list_local_branches_with_status failed");

    let main_entry = statuses.iter().find(|s| s.name == "main").expect("main branch missing");
    assert!(main_entry.is_default, "main must be flagged as the default branch");
    assert!(main_entry.is_current, "main must be flagged as current in the main worktree");
    assert!(main_entry.is_checked_out, "main must be flagged as checked out");
    assert!(!main_entry.is_merged, "the default branch is never 'merged into itself'");

    let merged_entry = statuses
        .iter()
        .find(|s| s.name == "feat/merged-no-worktree")
        .expect("merged branch missing");
    assert!(merged_entry.is_merged, "branch merged into main must be flagged is_merged");
    assert!(!merged_entry.is_default);
    assert!(!merged_entry.is_checked_out, "branch has no worktree left, so it isn't checked out");

    let open_entry = statuses
        .iter()
        .find(|s| s.name == "feat/still-open")
        .expect("open branch missing");
    assert!(!open_entry.is_merged, "unmerged branch must not be flagged is_merged");
    assert!(!open_entry.is_remote_gone);
    assert!(!open_entry.is_checked_out);
}

#[test]
fn test_batch_delete_skips_default_and_checked_out_even_with_force() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap().to_string();

    // A branch checked out in a live worktree.
    let wt_temp = tempfile::tempdir().expect("failed to create wt temp dir");
    let wt_path = wt_temp.path().join("wt-checked-out");
    GitService::create_worktree(
        &repo_path_str,
        wt_path.to_str().unwrap(),
        "main",
        Some("feat/checked-out"),
    )
    .expect("create worktree failed");

    // A plain unmerged branch with no worktree, safe to delete only with force.
    run_git(repo_path, &["branch", "feat/deletable"]).expect("create deletable branch failed");

    let targets = vec![
        // Target 1: the default branch (MUST be skipped, even forced).
        BranchDeleteTarget {
            repo_path: repo_path_str.clone(),
            branch_name: "main".to_string(),
            force: true,
        },
        // Target 2: a branch checked out in a worktree (MUST be skipped, even forced).
        BranchDeleteTarget {
            repo_path: repo_path_str.clone(),
            branch_name: "feat/checked-out".to_string(),
            force: true,
        },
        // Target 3: a free-standing unmerged branch, forced (MUST be deleted).
        BranchDeleteTarget {
            repo_path: repo_path_str.clone(),
            branch_name: "feat/deletable".to_string(),
            force: true,
        },
    ];

    let summary = BranchCleanerService::remove_branches_batch(targets);
    assert_eq!(summary.total_requested, 3);
    assert_eq!(summary.deleted_count, 1, "only the free-standing branch should be deleted");
    assert_eq!(summary.skipped_count, 2, "default branch and checked-out branch must be skipped");
    assert!(summary
        .deleted_branches
        .iter()
        .any(|d| d.repo_path == repo_path_str && d.branch_name == "feat/deletable"));

    let remaining = run_git(repo_path, &["branch", "--list"]).expect("branch --list failed");
    assert!(remaining.contains("main"), "main must still exist");
    assert!(remaining.contains("feat/checked-out"), "checked-out branch must still exist");
    assert!(!remaining.contains("feat/deletable"), "deletable branch must be gone");
}

#[test]
fn test_batch_delete_refuses_repo_when_status_lookup_fails() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap().to_string();

    // Move off main so it isn't the checked-out branch either — isolates the test to the
    // default-branch half of the guard, which has no git-level backstop (unlike checked-out
    // branches, which `git branch -D` refuses on its own).
    run_git(repo_path, &["checkout", "-b", "work"]).expect("checkout work failed");

    // Corrupt a ref so `git branch --list --format=...` (used by
    // `list_local_branches_with_status`) fails repo-wide with a non-zero exit.
    let broken_ref = repo_path.join(".git").join("refs").join("heads").join("broken");
    fs::write(&broken_ref, "1111111111111111111111111111111111111111\n")
        .expect("write broken ref failed");
    assert!(
        GitService::list_local_branches_with_status(repo_path_str.clone()).is_err(),
        "the broken ref must actually break status lookup, or this test proves nothing"
    );

    let summary = BranchCleanerService::remove_branches_batch(vec![BranchDeleteTarget {
        repo_path: repo_path_str,
        branch_name: "main".to_string(),
        force: true,
    }]);

    assert_eq!(
        summary.deleted_count, 0,
        "must refuse to delete when the default/checked-out status can't be verified, not fail open"
    );
    assert_eq!(summary.skipped_count, 1);
    assert!(!summary.errors.is_empty());
    assert!(
        repo_path.join(".git").join("refs").join("heads").join("main").exists(),
        "main branch ref must survive an unverifiable status lookup"
    );
}

#[test]
fn test_unmerged_branch_requires_force() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();
    let repo_path_str = repo_path.to_str().unwrap().to_string();

    // Give the branch its own commit so `git branch -d` has something to refuse.
    let wt_temp = tempfile::tempdir().expect("failed to create wt temp dir");
    let wt_path = wt_temp.path().join("wt-unmerged-source");
    GitService::create_worktree(
        &repo_path_str,
        wt_path.to_str().unwrap(),
        "main",
        Some("feat/unmerged"),
    )
    .expect("create worktree failed");
    fs::write(wt_path.join("wip.txt"), "not merged yet\n").expect("write wip file failed");
    run_git(&wt_path, &["add", "wip.txt"]).expect("add failed");
    run_git(&wt_path, &["commit", "-m", "WIP on feat/unmerged"]).expect("commit failed");
    run_git(repo_path, &["worktree", "remove", "--force", wt_path.to_str().unwrap()])
        .expect("remove worktree failed");

    // Without force: git's own safety net refuses to delete an unmerged branch.
    let without_force = BranchCleanerService::remove_branches_batch(vec![BranchDeleteTarget {
        repo_path: repo_path_str.clone(),
        branch_name: "feat/unmerged".to_string(),
        force: false,
    }]);
    assert_eq!(without_force.deleted_count, 0, "git -d must refuse an unmerged branch");
    assert_eq!(without_force.errors.len(), 1);

    let still_there = run_git(repo_path, &["branch", "--list"]).expect("branch --list failed");
    assert!(still_there.contains("feat/unmerged"), "branch must survive the unforced attempt");

    // With force: -D deletes it regardless of merge status.
    let with_force = BranchCleanerService::remove_branches_batch(vec![BranchDeleteTarget {
        repo_path: repo_path_str,
        branch_name: "feat/unmerged".to_string(),
        force: true,
    }]);
    assert_eq!(with_force.deleted_count, 1, "force delete must succeed on an unmerged branch");

    let gone = run_git(repo_path, &["branch", "--list"]).expect("branch --list failed");
    assert!(!gone.contains("feat/unmerged"), "branch must be gone after force delete");
}
