import { derived, writable } from 'svelte/store';
import type { BranchStatusEntry, BranchStatusFilterType } from '../types';

export const scannedBranches = writable<BranchStatusEntry[]>([]);
export const isScanningBranches = writable<boolean>(false);
export const branchScanError = writable<string | null>(null);
export const selectedBranchStatusFilter = writable<BranchStatusFilterType>('ALL');

/**
 * The branches the Branch Cleaner is actually showing. Derived here for the same reason as
 * `filteredRepos`: the "select all filtered" control must act on exactly what the list renders.
 */
export const filteredBranches = derived(
  [scannedBranches, selectedBranchStatusFilter],
  ([$scannedBranches, $filter]) =>
    $scannedBranches.filter((b) => {
      if ($filter === 'MERGED') return b.isMerged;
      if ($filter === 'REMOTE_GONE') return b.isRemoteGone;
      if ($filter === 'PROTECTED') return b.isDefault || b.isCheckedOut;
      return true;
    })
);
