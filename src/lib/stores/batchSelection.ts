import { writable, derived } from 'svelte/store';
import type { BatchDeleteTarget, RepositoryWorktrees } from '../types';

/**
 * Re-derives each selected target from the latest scan: a selection made before a rescan, a
 * stash or a discard would otherwise carry a stale dirty flag into the delete confirmation.
 * Targets whose worktree is gone (or became the main one) are dropped.
 */
export function rebuildTargetsFromScan(
  selection: Map<string, BatchDeleteTarget>,
  repos: RepositoryWorktrees[]
): Map<string, BatchDeleteTarget> {
  const next = new Map<string, BatchDeleteTarget>();
  for (const repo of repos) {
    for (const wt of repo.worktrees) {
      const previous = selection.get(wt.path);
      if (!previous || wt.isMain) continue;
      next.set(wt.path, {
        ...previous,
        repoPath: repo.repoPath,
        repoName: repo.repoName,
        branch: wt.branch,
        isDirty: wt.isDirty,
        uncommittedFilesCount: wt.uncommittedFilesCount
      });
    }
  }
  return next;
}

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

    clear: () => set(new Map()),

    syncWithScan: (repos: RepositoryWorktrees[]) => {
      update((map) => rebuildTargetsFromScan(map, repos));
    },

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
