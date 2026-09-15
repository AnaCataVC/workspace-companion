import { writable } from 'svelte/store';
import type { BranchStatusEntry, BranchStatusFilterType } from '../types';

export const scannedBranches = writable<BranchStatusEntry[]>([]);
export const isScanningBranches = writable<boolean>(false);
export const branchScanError = writable<string | null>(null);
export const selectedBranchStatusFilter = writable<BranchStatusFilterType>('ALL');
