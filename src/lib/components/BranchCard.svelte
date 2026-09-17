<script lang="ts">
  import type { BranchStatusEntry } from '../types';
  import { GitBranch, Lock, FolderGit2, FolderSymlink, GitBranchPlus } from 'lucide-svelte';
  import { branchSelection, selectedBranchKeys, branchSelectionKey } from '../stores/branchSelection';
  import { branchProtectionReason } from '../utils/protectionReason';
  import BranchStatusBadges from './BranchStatusBadges.svelte';
  import { createEventDispatcher } from 'svelte';

  export let branch: BranchStatusEntry;
  export let repoName: string = '';

  /*
   * Deliberately has no delete button, unlike WorktreeCard: branch deletion is batch-only, so the
   * only destructive path stays the reviewed batch flow. Do not port WorktreeCard's action row.
   */
  const dispatch = createEventDispatcher<{
    requestCheckout: BranchStatusEntry;
  }>();

  $: isProtected = branch.isDefault || branch.isCheckedOut;
  $: isSelected = $selectedBranchKeys.has(branchSelectionKey({ repoPath: branch.repoPath, branchName: branch.name }));
  $: hasWorktree = Boolean(branch.checkedOutWorktreePath);

  function toggleSelection() {
    if (isProtected) return;
    branchSelection.toggle({
      repoPath: branch.repoPath,
      branchName: branch.name,
      force: false,
      repoName,
      isMerged: branch.isMerged,
      isRemoteGone: branch.isRemoteGone
    });
  }
</script>

<div class="group relative rounded-lg border p-2.5 transition-all text-xs flex flex-col gap-1.5 shadow-sm
  {isSelected
    ? 'bg-rose-950/30 border-rose-800/70 shadow-xs'
    : isProtected
    ? 'bg-neutral-900/40 border-neutral-850/70'
    : 'bg-neutral-900/90 hover:bg-neutral-850 border-neutral-800/80 hover:border-neutral-700'}">
  <div class="flex items-center justify-between gap-2">
    <div class="flex items-center gap-1.5 min-w-0 flex-1">
      {#if !isProtected}
        <input
          type="checkbox"
          checked={isSelected}
          on:change={toggleSelection}
          title="Select branch for batch delete"
          class="w-3.5 h-3.5 rounded border-neutral-700 bg-neutral-950 text-rose-500 focus:ring-rose-500/30 focus:ring-offset-0 cursor-pointer flex-shrink-0"
        />
      {:else}
        <span title={branchProtectionReason(branch)} class="w-3.5 h-3.5 flex items-center justify-center text-neutral-600 flex-shrink-0">
          <Lock size={10} />
        </span>
      {/if}

      <GitBranch size={12} class="text-indigo-400 flex-shrink-0" />
      <span class="font-mono font-medium truncate text-[11px] text-neutral-200" title={branch.name}>
        {branch.name}
      </span>
    </div>

    <BranchStatusBadges {branch} size="md" />
  </div>

  <!-- Repository and last commit info -->
  <div class="flex items-center justify-between text-neutral-400 text-[11px]">
    <div class="flex items-center gap-1 truncate max-w-[260px]" title={branch.repoPath}>
      <FolderGit2 size={11} class="text-neutral-500 flex-shrink-0" />
      <span class="truncate">{repoName || branch.repoPath}</span>
    </div>

    {#if branch.lastCommitSha}
      <span class="font-mono text-[10px] text-neutral-500">
        {branch.lastCommitSha}
      </span>
    {/if}
  </div>

  {#if branch.lastCommitMessage}
    <p class="text-[11px] text-neutral-400 truncate italic font-sans" title={branch.lastCommitMessage}>
      "{branch.lastCommitMessage}"
    </p>
  {/if}

  <div class="flex items-center justify-end pt-1 border-t border-neutral-800/50 opacity-85 group-hover:opacity-100 transition-opacity">
    <button
      type="button"
      on:click={() => dispatch('requestCheckout', branch)}
      title={hasWorktree
        ? `Go to the worktree at ${branch.checkedOutWorktreePath}`
        : 'Create a worktree that checks out this branch'}
      class="px-2 py-1 rounded bg-neutral-800/80 hover:bg-indigo-950/80 border border-neutral-700/60 hover:border-indigo-700/60 text-neutral-300 hover:text-indigo-300 flex items-center gap-1.5 transition-colors text-[10px]"
    >
      {#if hasWorktree}
        <FolderSymlink size={12} />
        <span>Go to worktree</span>
      {:else}
        <GitBranchPlus size={12} />
        <span>Checkout</span>
      {/if}
    </button>
  </div>
</div>
