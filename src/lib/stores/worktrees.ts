import { derived, writable } from 'svelte/store';
import type { RepositoryWorktrees } from '../types';
import { selectedAccountFilter, selectedStatusFilter } from './appConfig';

export const scannedRepos = writable<RepositoryWorktrees[]>([]);
export const isScanning = writable<boolean>(false);
export const searchFilter = writable<string>('');
export const isPinned = writable<boolean>(false);
export const scanError = writable<string | null>(null);

/**
 * The repositories and worktrees the Worktrees view is actually showing. Derived here rather than
 * computed inside `WorktreeList` because the "select all filtered" control has to act on exactly
 * what the list renders — two independent copies of this filter would eventually disagree about
 * what "all" means. Single-pass over the repo list, same as the filter bars' tallies.
 */
export const filteredRepos = derived(
  [scannedRepos, selectedAccountFilter, selectedStatusFilter, searchFilter],
  ([$scannedRepos, $account, $status, $search]) =>
    $scannedRepos
      .filter((repo) => {
        if ($account === 'UNASSIGNED') {
          if (repo.associatedAccount) return false;
        } else if ($account !== 'ALL') {
          if (repo.associatedAccount?.toLowerCase() !== $account.toLowerCase()) return false;
        }

        // Multi-WT is a repository-level property, so it filters whole repos out.
        if ($status === 'MULTI_WT' && repo.worktrees.length <= 1) {
          return false;
        }

        return true;
      })
      .map((repo) => {
        const q = $search.toLowerCase().trim();

        const matchedWorktrees = repo.worktrees.filter((wt) => {
          if ($status === 'DIRTY' && !wt.isDirty) return false;
          if ($status === 'ORPHANS' && (wt.isMain || !wt.isOrphaned)) return false;
          if ($status === 'CLEAN' && wt.isDirty) return false;

          if (!q) return true;
          return (
            (wt.branch && wt.branch.toLowerCase().includes(q)) ||
            wt.path.toLowerCase().includes(q) ||
            (wt.lastCommitMessage && wt.lastCommitMessage.toLowerCase().includes(q))
          );
        });

        return {
          ...repo,
          worktrees: matchedWorktrees
        };
      })
      .filter((repo) => repo.worktrees.length > 0)
);

/** Worktree the user asked to be taken to (from the Branches view); cleared on view change. */
export const highlightedWorktreePath = writable<string | null>(null);
