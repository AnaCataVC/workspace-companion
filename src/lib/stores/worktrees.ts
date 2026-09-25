import { derived, writable } from 'svelte/store';
import type { RepositoryWorktrees, WorktreeInfo } from '../types';
import { selectedAccountFilter, selectedStatusFilter } from './appConfig';

export const scannedRepos = writable<RepositoryWorktrees[]>([]);
export const isScanning = writable<boolean>(false);
export const searchFilter = writable<string>('');
export const isPinned = writable<boolean>(false);
export const scanError = writable<string | null>(null);

/** Whether a worktree matches the search box; `query` must already be lowercased and trimmed. */
export function worktreeMatchesSearch(wt: WorktreeInfo, query: string): boolean {
  if (!query) return true;
  return Boolean(
    (wt.branch && wt.branch.toLowerCase().includes(query)) ||
      wt.path.toLowerCase().includes(query) ||
      (wt.lastCommitMessage && wt.lastCommitMessage.toLowerCase().includes(query))
  );
}

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

          return worktreeMatchesSearch(wt, q);
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

/**
 * Counts open per-card interactive popovers (e.g. the discard-changes confirmation) so the panel
 * can stay visible on blur while one is open, the same way the app's modals hold `isAnyModalOpen`.
 */
export const openWorktreeActionCount = writable<number>(0);
