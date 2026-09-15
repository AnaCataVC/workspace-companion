use crate::services::git::{BranchStatusEntry, GitService};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchDeleteTarget {
    pub repo_path: String,
    pub branch_name: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBatchItemError {
    pub branch_name: String,
    pub error: String,
}

/// Identifies a deleted branch by repo + name, since a bare branch name is ambiguous when a
/// batch spans multiple repositories (e.g. "main" exists in every repo).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedBranchRef {
    pub repo_path: String,
    pub branch_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBatchDeleteSummary {
    pub total_requested: usize,
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub deleted_branches: Vec<DeletedBranchRef>,
    pub errors: Vec<BranchBatchItemError>,
}

pub struct BranchCleanerService;

impl BranchCleanerService {
    /// Lists every local branch, with status, across all given repositories.
    pub fn scan_branches(repo_paths: &[String]) -> Vec<BranchStatusEntry> {
        repo_paths
            .par_iter()
            .filter_map(|repo_path| GitService::list_local_branches_with_status(repo_path).ok())
            .flatten()
            .collect()
    }

    /// Deletes multiple branches safely and efficiently across repositories.
    ///
    /// The default branch and any branch checked out in a worktree (this repo's main worktree or
    /// a linked one) are never deleted, even with `force: true` — mirrors the "can't remove the
    /// main working tree" guard in `WorktreeCleanerService::remove_worktrees_batch`. Status is
    /// recomputed once per repo right before deleting, instead of trusting the caller's targets,
    /// so a branch that was merged/checked-out since the frontend's last scan is still protected.
    pub fn remove_branches_batch(targets: Vec<BranchDeleteTarget>) -> BranchBatchDeleteSummary {
        let total_requested = targets.len();
        if targets.is_empty() {
            return BranchBatchDeleteSummary {
                total_requested: 0,
                deleted_count: 0,
                skipped_count: 0,
                deleted_branches: Vec::new(),
                errors: Vec::new(),
            };
        }

        // Group by repository so the fresh status lookup below runs once per repo, not once per
        // branch, and so per-repo git calls stay sequential (same rationale as the worktree
        // cleaner's grouping, even though branch deletion doesn't hold a `.git/worktrees` lock).
        let mut grouped: HashMap<String, Vec<BranchDeleteTarget>> = HashMap::new();
        for target in targets {
            grouped
                .entry(target.repo_path.clone())
                .or_default()
                .push(target);
        }

        let repo_results: Vec<(Vec<DeletedBranchRef>, usize, Vec<BranchBatchItemError>)> = grouped
            .into_par_iter()
            .map(|(repo_path, repo_targets)| {
                let mut local_deleted = Vec::new();
                let mut local_skipped = 0;
                let mut local_errors = Vec::new();

                // If status can't be re-derived, every target in this repo is refused outright.
                // Falling back to an empty status list here would make `is_protected` default to
                // false for every branch — including the default branch — turning a lookup
                // failure into a silent bypass of the one guard `force` is never allowed to skip.
                let statuses = match GitService::list_local_branches_with_status(&repo_path) {
                    Ok(statuses) => statuses,
                    Err(err) => {
                        for item in repo_targets {
                            local_skipped += 1;
                            local_errors.push(BranchBatchItemError {
                                branch_name: item.branch_name,
                                error: format!(
                                    "Cannot verify which branches are safe to delete in this repo: {}",
                                    err
                                ),
                            });
                        }
                        return (local_deleted, local_skipped, local_errors);
                    }
                };

                for item in repo_targets {
                    let is_protected = statuses
                        .iter()
                        .find(|s| s.name == item.branch_name)
                        .map(|s| s.is_default || s.is_checked_out)
                        .unwrap_or(false);

                    if is_protected {
                        local_skipped += 1;
                        local_errors.push(BranchBatchItemError {
                            branch_name: item.branch_name.clone(),
                            error: "Cannot delete the default branch or a branch checked out in a worktree.".to_string(),
                        });
                        continue;
                    }

                    match GitService::delete_branch(
                        Path::new(&repo_path),
                        &item.branch_name,
                        item.force,
                    ) {
                        Ok(()) => local_deleted.push(DeletedBranchRef {
                            repo_path: repo_path.clone(),
                            branch_name: item.branch_name,
                        }),
                        Err(err) => local_errors.push(BranchBatchItemError {
                            branch_name: item.branch_name,
                            error: err,
                        }),
                    }
                }

                (local_deleted, local_skipped, local_errors)
            })
            .collect();

        let mut deleted_branches = Vec::new();
        let mut skipped_count = 0;
        let mut errors = Vec::new();

        for (branches, skipped, mut errs) in repo_results {
            deleted_branches.extend(branches);
            skipped_count += skipped;
            errors.append(&mut errs);
        }

        let deleted_count = deleted_branches.len();

        BranchBatchDeleteSummary {
            total_requested,
            deleted_count,
            skipped_count,
            deleted_branches,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_remove_empty_targets() {
        let summary = BranchCleanerService::remove_branches_batch(Vec::new());
        assert_eq!(summary.total_requested, 0);
        assert_eq!(summary.deleted_count, 0);
        assert_eq!(summary.skipped_count, 0);
        assert!(summary.deleted_branches.is_empty());
        assert!(summary.errors.is_empty());
    }
}
