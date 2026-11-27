import { derived, writable } from 'svelte/store';
import type { RepositoryWorktrees, WorktreeInfo } from '../types';
import { selectedAccountFilter, selectedStatusFilter } from './appConfig';

export const scannedRepos = writable<RepositoryWorktrees[]>([]);
export const isScanning = writable<boolean>(false);
export const searchFilter = writable<string>('');
export const isPinned = writable<boolean>(false);
export const scanError = writable<string | null>(null);

/** Resolves a repository's effective default branch name (`main` fallback when unconfigured). */
export function getRepoDefaultBranch(repo: RepositoryWorktrees): string {
  return repo.defaultBranch?.trim() || 'main';
}

/** Whether a repository's root/main worktree is currently checked out off its default branch. */
export function isRepoOffDefaultBranch(repo: RepositoryWorktrees): boolean {
  const mainWt = repo.worktrees.find((wt) => wt.isMain);
  if (!mainWt) return false;
  if (!mainWt.branch) return true; // detached HEAD is off the default branch
  const shortBranch = mainWt.branch.replace('refs/heads/', '');
  const expected = getRepoDefaultBranch(repo);
  if (shortBranch === expected) return false;
  // Treat master as on-default only when defaultBranch was not explicitly resolved
  if (!repo.defaultBranch && shortBranch === 'master') return false;
  return true;
}

/** Whether a worktree matches the search box; `query` must already be lowercased and trimmed. */
export function worktreeMatchesSearch(
  wt: WorktreeInfo,
  query: string,
  repoName?: string,
  associatedAccount?: string | null
): boolean {
  if (!query) return true;
  const accountText = associatedAccount ? `@${associatedAccount.toLowerCase()}` : '';
  return Boolean(
    (wt.branch && wt.branch.toLowerCase().includes(query)) ||
      wt.path.toLowerCase().includes(query) ||
      (wt.lastCommitMessage && wt.lastCommitMessage.toLowerCase().includes(query)) ||
      (repoName && repoName.toLowerCase().includes(query)) ||
      (accountText && accountText.includes(query))
  );
}

/**
 * The repositories and worktrees the Worktrees view is actually showing. Derived here rather than
 * computed inside `WorktreeList` because the "select all filtered" control and filtered Quick
 * Actions scope must act on the exact set rendered on screen.
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

        // Repository-level properties filter whole repositories.
        if ($status === 'MULTI_WT' && repo.worktrees.length <= 1) {
          return false;
        }
        if ($status === 'OFF_MAIN' && !isRepoOffDefaultBranch(repo)) {
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

          return worktreeMatchesSearch(wt, q, repo.repoName, repo.associatedAccount);
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

/** Worktree currently focused via Up/Down arrow keyboard navigation. */
export const keyboardFocusedPath = writable<string | null>(null);

/** Session-only set of collapsed repository paths; ignored while a search or status filter is active. */
export const collapsedRepos = writable<Set<string>>(new Set());

export function toggleRepoCollapsed(repoPath: string): void {
  collapsedRepos.update((prev) => {
    const next = new Set(prev);
    if (next.has(repoPath)) next.delete(repoPath);
    else next.add(repoPath);
    return next;
  });
}

export function collapseAllRepos(repoPaths: string[]): void {
  collapsedRepos.set(new Set(repoPaths));
}

export function expandAllRepos(): void {
  collapsedRepos.set(new Set());
}

/** Whether any search, status, or account filter narrows the Worktrees view. */
export const hasActiveWorktreeFilters = derived(
  [searchFilter, selectedStatusFilter, selectedAccountFilter],
  ([$search, $status, $account]) =>
    Boolean($search.trim()) || $status !== 'ALL' || $account !== 'ALL'
);

/** While a search or status filter is active, collapsed repos expand automatically so matches stay visible. */
export const isFilterActive = derived(
  [searchFilter, selectedStatusFilter],
  ([$search, $status]) => Boolean($search.trim()) || $status !== 'ALL'
);

export function resetWorktreeFilters(): void {
  searchFilter.set('');
  selectedStatusFilter.set('ALL');
  selectedAccountFilter.set('ALL');
}

/** Worktrees currently rendered on screen, in display order; the arrow-key navigation walks this list. */
export const navigableWorktrees = derived(
  [filteredRepos, collapsedRepos, isFilterActive],
  ([$repos, $collapsed, $filterActive]) =>
    $repos
      .filter((repo) => $filterActive || !$collapsed.has(repo.repoPath))
      .flatMap((repo) => repo.worktrees.map((worktree) => ({ worktree, repoPath: repo.repoPath })))
);

/**
 * Counts open per-card interactive popovers (e.g. the discard-changes confirmation) so the panel
 * can stay visible on blur while one is open, the same way the app's modals hold `isAnyModalOpen`.
 */
export const openWorktreeActionCount = writable<number>(0);
