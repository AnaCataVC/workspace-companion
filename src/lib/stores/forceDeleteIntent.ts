import { writable } from 'svelte/store';

/**
 * Survives a modal being closed and reopened over the same selection, so a user who ticks
 * "force", closes the dialog to double-check something, and comes back doesn't have to tick it
 * again. Each modal resets its own flag when the target it is pointed at changes.
 *
 * Forcing a dirty worktree and forcing an unmerged branch are separate intentions, so they never
 * share a flag: consenting to one must not silently consent to the other.
 */
export const forceSingleWorktreeDelete = writable<boolean>(false);
export const forceBatchWorktreeDelete = writable<boolean>(false);
export const forceBranchDelete = writable<boolean>(false);
