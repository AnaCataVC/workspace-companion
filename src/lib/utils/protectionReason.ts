import type { BranchStatusEntry } from '../types';

/**
 * Why the batch cleaner will refuse this branch, or an empty string when it is freely deletable.
 * Mirrors the server-side guard in `BranchCleanerService::remove_branches_batch` (ADR 0006), which
 * remains the authority — this only puts the same rule into words for the UI.
 */
export function branchProtectionReason(branch: BranchStatusEntry): string {
  if (branch.isDefault) return 'Default branch — never deletable';
  if (branch.isCheckedOut) return `Checked out at ${branch.checkedOutWorktreePath || 'a worktree'}`;
  return '';
}

/**
 * Accepts both a scanned `WorktreeInfo` and a `BatchDeleteTarget`, which carries only the subset
 * of fields the batch modal knows about.
 */
export interface WorktreeProtectionInput {
  isMain?: boolean;
  isDirty?: boolean;
  uncommittedFilesCount?: number;
  isOrphaned?: boolean;
  orphanReason?: string;
  locked?: string | null;
}

/**
 * Why removing this worktree is blocked or destructive, or an empty string when it is an ordinary
 * clean removal. Ordered by how hard the block is: the main worktree can never be removed, a dirty
 * or locked one only with force, and an orphaned one is merely informative.
 */
export function worktreeProtectionReason(worktree: WorktreeProtectionInput): string {
  if (worktree.isMain) return 'Main / root worktree — never deletable';
  if (worktree.locked) {
    return `Locked worktree — ${worktree.locked}`;
  }
  if (worktree.isDirty) {
    const count = worktree.uncommittedFilesCount;
    return count
      ? `${count} uncommitted file(s) — removing discards that work permanently`
      : 'Uncommitted changes — removing discards that work permanently';
  }
  if (worktree.isOrphaned) return worktree.orphanReason || 'Orphaned worktree';
  return '';
}
