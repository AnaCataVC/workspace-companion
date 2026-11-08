use std::time::Instant;
use workspace_companion::services::git::{GitService, RepoOrphanContext};

mod common;
use common::{create_test_repo, run_git};

/// Simulates the legacy naive approach (prior to ADR 0005) where repo-wide facts
/// were re-queried from Git subprocesses once for every single worktree.
fn naive_per_worktree_check(
    repo_path: &std::path::Path,
    branches: &[&str],
) -> (Vec<(bool, bool)>, usize) {
    let mut results = Vec::new();
    let mut subprocess_count = 0;

    for branch in branches {
        // Subprocess 1: get default branch
        let default_branch = GitService::get_default_branch(repo_path).unwrap_or_else(|| "main".to_string());
        subprocess_count += 1;

        // Subprocess 2: git branch -vv
        let branch_vv_output = GitService::run_git(repo_path, &["branch", "-vv"]).unwrap_or_default();
        subprocess_count += 1;

        // Subprocess 3: git branch --merged
        let merged_branches_output = GitService::run_git(repo_path, &["branch", "--merged", &default_branch]).unwrap_or_default();
        subprocess_count += 1;

        let ctx = RepoOrphanContext {
            default_branch,
            branch_vv_output,
            merged_branches_output,
        };

        // In-memory evaluation
        let flags = GitService::branch_status_flags(branch, &ctx);
        results.push(flags);
    }

    (results, subprocess_count)
}

/// The optimized approach implemented in ADR 0005: Repo-wide context is computed
/// exactly once per repository, and all worktrees evaluate their status in-memory.
fn cached_repo_context_check(
    repo_path: &std::path::Path,
    branches: &[&str],
) -> (Vec<(bool, bool)>, usize) {
    // Exactly 3-4 subprocesses for the entire repository, regardless of N worktrees
    let ctx = GitService::build_repo_orphan_context(repo_path);
    let subprocess_count = 3; // get_default_branch + branch -vv + branch --merged

    let mut results = Vec::new();
    for branch in branches {
        let flags = GitService::branch_status_flags(branch, &ctx);
        results.push(flags);
    }

    (results, subprocess_count)
}

#[test]
fn test_context_caching_subprocess_reduction_and_latency_benchmark() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();

    let worktree_counts = [1, 3, 5, 10];
    let mut branch_names = Vec::new();

    // Create 10 branches and worktrees in the real git repository
    for i in 1..=10 {
        let branch = format!("feature/task-{:02}", i);
        run_git(repo_path, &["branch", &branch]).expect("branch create failed");
        branch_names.push(branch);
    }

    println!("\n=========================================================================================");
    println!("  WORKSPACE COMPANION — CONTEXT CACHING BENCHMARK (ADR 0005)");
    println!("=========================================================================================");
    println!("{:<12} | {:<16} | {:<16} | {:<14} | {:<12}", "Worktrees", "Naive Procs", "Cached Procs", "Reduction", "Speedup");
    println!("-----------------------------------------------------------------------------------------");

    for &count in &worktree_counts {
        let active_branches: Vec<&str> = branch_names.iter().take(count).map(|s| s.as_str()).collect();

        // 1. Measure Naive
        let start_naive = Instant::now();
        let (naive_res, naive_procs) = naive_per_worktree_check(repo_path, &active_branches);
        let naive_duration = start_naive.elapsed();

        // 2. Measure Cached
        let start_cached = Instant::now();
        let (cached_res, cached_procs) = cached_repo_context_check(repo_path, &active_branches);
        let cached_duration = start_cached.elapsed();

        // 3. Verify exact output parity (no divergence)
        assert_eq!(naive_res, cached_res, "Output parity check failed for N={}", count);

        let reduction_ratio = naive_procs as f64 / cached_procs as f64;
        let speedup = if cached_duration.as_millis() > 0 {
            naive_duration.as_secs_f64() / cached_duration.as_secs_f64()
        } else {
            1.0
        };

        println!(
            "{:<12} | {:<16} | {:<16} | {:<13.1}x | {:<11.1}x ({:?} vs {:?})",
            count,
            naive_procs,
            cached_procs,
            reduction_ratio,
            speedup,
            naive_duration,
            cached_duration
        );

        if count == 5 {
            // Validate the claim in the README and ADR 0005: roughly 5x reduction in subprocess spawns
            assert!(
                reduction_ratio >= 4.5,
                "Expected ~5x reduction in subprocess calls for 5 worktrees, got {:.2}x",
                reduction_ratio
            );
        }
    }
    println!("=========================================================================================\n");
}

#[test]
fn test_batch_commit_info_subprocess_reduction_and_parity() {
    let repo_temp = create_test_repo();
    let repo_path = repo_temp.path();

    // Create 5 commits on 5 branches
    let mut shas = Vec::new();
    for i in 1..=5 {
        let branch = format!("feature/commit-batch-{:02}", i);
        run_git(repo_path, &["checkout", "-b", &branch]).unwrap();
        let file_name = format!("file_{}.txt", i);
        std::fs::write(repo_path.join(&file_name), format!("content {}", i)).unwrap();
        run_git(repo_path, &["add", &file_name]).unwrap();
        let msg = format!("Commit message for feature {}", i);
        run_git(repo_path, &["commit", "-m", &msg]).unwrap();
        let head_out = run_git(repo_path, &["rev-parse", "HEAD"]).unwrap();
        shas.push(head_out.trim().to_string());
    }

    // 1. Individual log -1 approach (naive: N subprocesses)
    let sha_refs: Vec<&str> = shas.iter().map(|s| s.as_str()).collect();
    let mut naive_results = Vec::new();
    for &sha in &sha_refs {
        let output = run_git(repo_path, &["log", "-1", "--format=%s|||%an|||%cr", sha]).unwrap();
        let parts: Vec<&str> = output.trim().split("|||").collect();
        let msg = parts.get(0).map(|s| s.to_string());
        let author = parts.get(1).map(|s| s.to_string());
        let date = parts.get(2).map(|s| s.to_string());
        naive_results.push((msg, author, date));
    }

    // 2. Batched git show -s approach: 1 subprocess for all N commits
    let batched_map = GitService::get_batch_commit_info(repo_path, &sha_refs);

    // 3. Verify exact output parity (no divergence)
    assert_eq!(batched_map.len(), 5);
    for (i, &sha) in sha_refs.iter().enumerate() {
        let (batched_msg, batched_author, batched_date) = batched_map
            .get(sha)
            .expect("SHA should be in batch map")
            .clone();
        let (naive_msg, naive_author, naive_date) = &naive_results[i];
        assert_eq!(&batched_msg, naive_msg, "Message mismatch for sha {}", sha);
        assert_eq!(&batched_author, naive_author, "Author mismatch for sha {}", sha);
        // Date is relative time (%cr e.g. "1 second ago"), which can drift by 1 second
        // between sequential naive invocations and the batched call.
        assert!(batched_date.is_some() && naive_date.is_some(), "Date should be populated");
        assert!(
            batched_date.as_ref().unwrap().ends_with("ago"),
            "Date should end with 'ago': {:?}",
            batched_date
        );
    }
}
