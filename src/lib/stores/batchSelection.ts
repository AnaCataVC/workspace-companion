import { writable, derived } from 'svelte/store';
import type { BatchDeleteTarget } from '../types';

function createBatchSelectionStore() {
  const { subscribe, set, update } = writable<Map<string, BatchDeleteTarget>>(new Map());

  return {
    subscribe,

    toggle: (target: BatchDeleteTarget) => {
      update((map) => {
        const next = new Map(map);
        if (next.has(target.worktreePath)) {
          next.delete(target.worktreePath);
        } else {
          next.set(target.worktreePath, target);
        }
        return next;
      });
    },

    selectRepo: (targets: BatchDeleteTarget[]) => {
      update((map) => {
        const next = new Map(map);
        for (const t of targets) {
          next.set(t.worktreePath, t);
        }
        return next;
      });
    },

    deselectRepo: (paths: string[]) => {
      update((map) => {
        const next = new Map(map);
        for (const path of paths) {
          next.delete(path);
        }
        return next;
      });
    },

    deselect: (path: string) => {
      update((map) => {
        if (!map.has(path)) return map;
        const next = new Map(map);
        next.delete(path);
        return next;
      });
    },

    selectAll: (targets: BatchDeleteTarget[]) => {
      update(() => {
        const next = new Map<string, BatchDeleteTarget>();
        for (const t of targets) {
          next.set(t.worktreePath, t);
        }
        return next;
      });
    },

    clear: () => set(new Map()),

    prune: (validPaths: Set<string>) => {
      update((map) => {
        let changed = false;
        const next = new Map(map);
        for (const path of next.keys()) {
          if (!validPaths.has(path)) {
            next.delete(path);
            changed = true;
          }
        }
        return changed ? next : map;
      });
    }
  };
}

export const batchSelection = createBatchSelectionStore();

export const selectedWorktreeList = derived(batchSelection, ($map) => Array.from($map.values()));
export const selectedCount = derived(batchSelection, ($map) => $map.size);
export const hasDirtySelected = derived(selectedWorktreeList, ($list) => $list.some((w) => Boolean(w.isDirty)));
export const selectedPaths = derived(batchSelection, ($map) => new Set($map.keys()));
