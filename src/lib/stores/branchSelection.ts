import { writable, derived } from 'svelte/store';
import type { BranchDeleteTarget } from '../types';

/**
 * Branch names are not globally unique across repos (every repo has a "main"), unlike worktree
 * paths, so the selection map must key on repo+branch instead of branch name alone.
 */
export function branchSelectionKey(target: Pick<BranchDeleteTarget, 'repoPath' | 'branchName'>): string {
  return `${target.repoPath}::${target.branchName}`;
}

function createBranchSelectionStore() {
  const { subscribe, set, update } = writable<Map<string, BranchDeleteTarget>>(new Map());

  return {
    subscribe,

    toggle: (target: BranchDeleteTarget) => {
      update((map) => {
        const next = new Map(map);
        const key = branchSelectionKey(target);
        if (next.has(key)) {
          next.delete(key);
        } else {
          next.set(key, target);
        }
        return next;
      });
    },

    selectRepo: (targets: BranchDeleteTarget[]) => {
      update((map) => {
        const next = new Map(map);
        for (const t of targets) {
          next.set(branchSelectionKey(t), t);
        }
        return next;
      });
    },

    deselectRepo: (targets: Pick<BranchDeleteTarget, 'repoPath' | 'branchName'>[]) => {
      update((map) => {
        const next = new Map(map);
        for (const t of targets) {
          next.delete(branchSelectionKey(t));
        }
        return next;
      });
    },

    selectAll: (targets: BranchDeleteTarget[]) => {
      update(() => {
        const next = new Map<string, BranchDeleteTarget>();
        for (const t of targets) {
          next.set(branchSelectionKey(t), t);
        }
        return next;
      });
    },

    clear: () => set(new Map()),

    /** Drops any selected branch whose repo+name key is not in the given set (called after a delete). */
    prune: (validKeys: Set<string>) => {
      update((map) => {
        let changed = false;
        const next = new Map(map);
        for (const key of next.keys()) {
          if (!validKeys.has(key)) {
            next.delete(key);
            changed = true;
          }
        }
        return changed ? next : map;
      });
    }
  };
}

export const branchSelection = createBranchSelectionStore();

export const selectedBranchList = derived(branchSelection, ($map) => Array.from($map.values()));
export const selectedBranchCount = derived(branchSelection, ($map) => $map.size);
export const hasUnmergedBranchSelected = derived(selectedBranchList, ($list) => $list.some((b) => !b.isMerged));
export const selectedBranchKeys = derived(branchSelection, ($map) => new Set($map.keys()));
