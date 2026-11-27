import { derived, writable } from 'svelte/store';
import type { BranchStatusEntry, BranchStatusFilterType } from '../types';
import { selectedAccountFilter } from './appConfig';
import { scannedRepos } from './worktrees';

export const scannedBranches = writable<BranchStatusEntry[]>([]);
export const isScanningBranches = writable<boolean>(false);
export const branchScanError = writable<string | null>(null);
export const selectedBranchStatusFilter = writable<BranchStatusFilterType>('ALL');
export const branchSearchFilter = writable<string>('');

export function branchMatchesSearch(
  branch: BranchStatusEntry,
  query: string,
  repoName?: string,
  associatedAccount?: string | null
): boolean {
  if (!query) return true;
  const accountText = associatedAccount ? `@${associatedAccount.toLowerCase()}` : '';
  return Boolean(
    branch.name.toLowerCase().includes(query) ||
      branch.repoPath.toLowerCase().includes(query) ||
      (repoName && repoName.toLowerCase().includes(query)) ||
      (branch.lastCommitMessage && branch.lastCommitMessage.toLowerCase().includes(query)) ||
      (accountText && accountText.includes(query))
  );
}

/**
 * The branches the Branch Cleaner is actually showing, honoring account filter, status filter,
 * and search query consistently with `filteredRepos`.
 */
export const filteredBranches = derived(
  [scannedBranches, scannedRepos, selectedAccountFilter, selectedBranchStatusFilter, branchSearchFilter],
  ([$scannedBranches, $scannedRepos, $account, $filter, $search]) => {
    const q = $search.toLowerCase().trim();
    const repoByPath = new Map($scannedRepos.map((r) => [r.repoPath, r]));

    return $scannedBranches.filter((b) => {
      const repo = repoByPath.get(b.repoPath);
      const repoAccount = repo?.associatedAccount ?? null;

      if ($account === 'UNASSIGNED') {
        if (repoAccount) return false;
      } else if ($account !== 'ALL') {
        if (repoAccount?.toLowerCase() !== $account.toLowerCase()) return false;
      }

      if ($filter === 'MERGED' && !b.isMerged) return false;
      if ($filter === 'REMOTE_GONE' && !b.isRemoteGone) return false;
      if ($filter === 'PROTECTED' && !(b.isDefault || b.isCheckedOut)) return false;

      return branchMatchesSearch(b, q, repo?.repoName, repoAccount);
    });
  }
);

export const hasActiveBranchFilters = derived(
  [branchSearchFilter, selectedBranchStatusFilter, selectedAccountFilter],
  ([$search, $status, $account]) =>
    Boolean($search.trim()) || $status !== 'ALL' || $account !== 'ALL'
);

export function resetBranchFilters(): void {
  branchSearchFilter.set('');
  selectedBranchStatusFilter.set('ALL');
  selectedAccountFilter.set('ALL');
}
