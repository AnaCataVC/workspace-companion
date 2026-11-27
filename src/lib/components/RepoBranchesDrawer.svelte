<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    GitBranch,
    GitBranchPlus,
    Trash2,
    Lock,
    ChevronDown,
    ChevronRight,
    RotateCw
  } from 'lucide-svelte';
  import BranchStatusBadges from './BranchStatusBadges.svelte';
  import { invokeTauri } from '../utils/ipc';
  import { toErrorMessage } from '../utils/errors';
  import { notifications } from '../stores/notifications';
  import { branchProtectionReason } from '../utils/protectionReason';
  import type { BranchBatchDeleteSummary, BranchStatusEntry } from '../types';

  export let repoPath: string;
  export let repoName: string;
  export let branches: BranchStatusEntry[] = [];

  const dispatch = createEventDispatcher<{
    createWorktreeFromBranch: BranchStatusEntry;
    branchDeleted: { repoPath: string; branchName: string };
  }>();

  let isOpen: boolean = false;
  let armedBranchName: string | null = null;
  let deletingBranchName: string | null = null;

  $: mergedCount = branches.filter((b) => b.isMerged && !b.isDefault).length;
  $: goneCount = branches.filter((b) => b.isRemoteGone && !b.isDefault).length;

  async function deleteSingleBranch(branch: BranchStatusEntry, force: boolean) {
    if (branch.isDefault || deletingBranchName) return;

    // Unmerged branches require explicit 2-step confirmation before invoking `git branch -D`.
    if (!branch.isMerged && !force) {
      armedBranchName = branch.name;
      return;
    }

    deletingBranchName = branch.name;
    try {
      const summary = await invokeTauri<BranchBatchDeleteSummary>('remove_branches_batch', {
        targets: [
          {
            repoPath,
            branchName: branch.name,
            force,
            repoName,
            isMerged: branch.isMerged,
            isRemoteGone: branch.isRemoteGone
          }
        ]
      });

      const deleted = summary?.deletedBranches?.some(
        (d) => d.repoPath === repoPath && d.branchName === branch.name
      );

      if (deleted) {
        armedBranchName = null;
        dispatch('branchDeleted', { repoPath, branchName: branch.name });
        notifications.success('Branch deleted', `Deleted ${branch.name} in ${repoName}.`);
      } else {
        const errMsg = summary?.errors?.[0]?.error || `Could not delete branch ${branch.name}`;
        notifications.error('Could not delete branch', errMsg);
      }
    } catch (err: unknown) {
      notifications.error('Error deleting branch', toErrorMessage(err));
    } finally {
      deletingBranchName = null;
    }
  }
</script>

{#if branches.length > 0}
  <div class="pt-0.5">
    <button
      type="button"
      on:click={() => {
        isOpen = !isOpen;
        if (!isOpen) armedBranchName = null;
      }}
      aria-expanded={isOpen}
      class="w-full flex items-center justify-between px-2 py-1 rounded-md bg-neutral-900/50 hover:bg-neutral-850/90 border border-neutral-800/70 hover:border-neutral-750 text-[11px] text-neutral-400 hover:text-neutral-200 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
    >
      <div class="flex items-center gap-1.5 min-w-0">
        {#if isOpen}
          <ChevronDown size={11} class="text-neutral-400 flex-shrink-0" />
        {:else}
          <ChevronRight size={11} class="text-neutral-400 flex-shrink-0" />
        {/if}
        <GitBranch size={11} class="text-indigo-400 flex-shrink-0" />
        <span class="font-medium truncate">
          {branches.length} {branches.length === 1 ? 'local branch' : 'local branches'} without worktree
        </span>
      </div>

      <div class="flex items-center gap-1 flex-shrink-0">
        {#if mergedCount > 0}
          <span class="px-1.5 py-0.5 rounded bg-emerald-950/70 text-emerald-300 border border-emerald-800/50 font-mono text-[10px]">
            {mergedCount} merged
          </span>
        {/if}
        {#if goneCount > 0}
          <span class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 font-mono text-[10px]">
            {goneCount} gone
          </span>
        {/if}
      </div>
    </button>

    {#if isOpen}
      <div class="mt-1 pl-3 border-l border-neutral-800 space-y-1">
        {#each branches as branch (branch.name)}
          {@const isArmed = armedBranchName === branch.name}
          {@const isDeleting = deletingBranchName === branch.name}
          <div
            class="flex items-center justify-between gap-2 px-2 py-1 rounded bg-neutral-900/70 border border-neutral-800/80 text-[11px]"
          >
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <GitBranch size={10} class="text-neutral-400 flex-shrink-0" />
              <span class="font-mono text-neutral-200 truncate" title={branch.name}>
                {branch.name}
              </span>
              {#if branch.lastCommitMessage}
                <span class="text-neutral-400 truncate hidden sm:inline" title={branch.lastCommitMessage}>
                  — {branch.lastCommitMessage}
                </span>
              {/if}
            </div>

            <div class="flex items-center gap-1 flex-shrink-0">
              <BranchStatusBadges {branch} size="sm" />

              {#if isArmed}
                <span class="text-[10px] text-rose-300 font-medium pl-1">Unmerged:</span>
                <button
                  type="button"
                  disabled={isDeleting}
                  on:click={() => deleteSingleBranch(branch, true)}
                  class="px-1.5 py-0.5 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium text-[10px] transition-colors disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-400"
                >
                  {isDeleting ? 'Deleting...' : 'Force Delete'}
                </button>
                <button
                  type="button"
                  disabled={isDeleting}
                  on:click={() => (armedBranchName = null)}
                  class="px-1.5 py-0.5 rounded bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-[10px] transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
                >
                  Cancel
                </button>
              {:else}
                <button
                  type="button"
                  on:click={() => dispatch('createWorktreeFromBranch', branch)}
                  title={`Create a new worktree checking out ${branch.name}`}
                  aria-label={`Create worktree from ${branch.name}`}
                  class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-neutral-800/90 hover:bg-indigo-950/80 text-neutral-300 hover:text-indigo-200 border border-neutral-700/60 hover:border-indigo-700/60 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
                >
                  <GitBranchPlus size={10} />
                  <span>Worktree</span>
                </button>

                {#if branch.isDefault}
                  <span
                    title={branchProtectionReason(branch)}
                    aria-label={branchProtectionReason(branch)}
                    class="p-1 text-neutral-500 inline-flex items-center justify-center"
                  >
                    <Lock size={11} />
                  </span>
                {:else}
                  <button
                    type="button"
                    disabled={isDeleting}
                    on:click={() => deleteSingleBranch(branch, false)}
                    title={branch.isMerged
                      ? `Delete merged branch ${branch.name}`
                      : `Delete unmerged branch ${branch.name} (will ask confirmation)`}
                    aria-label={`Delete branch ${branch.name}`}
                    class="p-1 rounded hover:bg-rose-950/70 text-neutral-400 hover:text-rose-300 border border-transparent hover:border-rose-800/60 transition-colors disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500/60"
                  >
                    {#if isDeleting}
                      <RotateCw size={11} class="animate-spin text-rose-400" />
                    {:else}
                      <Trash2 size={11} />
                    {/if}
                  </button>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}
