import { derived, writable } from 'svelte/store';
import type { BranchStatusEntry, BranchStatusFilterType } from '../types';

export const scannedBranches = writable<BranchStatusEntry[]>([]);
export const isScanningBranches = writable<boolean>(false);
export const branchScanError = writable<string | null>(null);
export const selectedBranchStatusFilter = writable<BranchStatusFilterType>('ALL');
export const branchSearchFilter = writable<string>('');

/**
 * The branches the Branch Cleaner is actually showing. Derived here for the same reason as
 * `filteredRepos`: the "select all filtered" control must act on exactly what the list renders.
 */
export const filteredBranches = derived(
  [scannedBranches, selectedBranchStatusFilter, branchSearchFilter],
  ([$scannedBranches, $filter, $search]) => {
    const q = $search.toLowerCase().trim();

    return $scannedBranches.filter((b) => {
      if ($filter === 'MERGED' && !b.isMerged) return false;
      if ($filter === 'REMOTE_GONE' && !b.isRemoteGone) return false;
      if ($filter === 'PROTECTED' && !(b.isDefault || b.isCheckedOut)) return false;

      if (!q) return true;
      return b.name.toLowerCase().includes(q) || b.repoPath.toLowerCase().includes(q);
    });
  }
);
