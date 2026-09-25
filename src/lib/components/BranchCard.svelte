<script lang="ts">
  import type { BranchStatusEntry } from '../types';
  import { GitBranch, Lock, FolderGit2, FolderSymlink, GitBranchPlus, Unlock } from 'lucide-svelte';
  import { branchSelection, selectedBranchKeys, branchSelectionKey } from '../stores/branchSelection';
  import { branchProtectionReason } from '../utils/protectionReason';
  import BranchStatusBadges from './BranchStatusBadges.svelte';
  import { createEventDispatcher } from 'svelte';

  export let branch: BranchStatusEntry;
  export let repoName: string = '';

  /*
   * Deliberately has no batch-style delete button, unlike WorktreeCard: ordinary branch deletion
   * is batch-only, so the only destructive path stays the reviewed batch flow. The one exception
   * is "Release", since a checked-out branch can never be selected for that batch flow at all.
   */
  const dispatch = createEventDispatcher<{
    requestCheckout: BranchStatusEntry;
    requestRelease: BranchStatusEntry;
  }>();

  $: isProtected = branch.isDefault || branch.isCheckedOut;
  $: isReleasable = branch.isCheckedOut && !branch.isDefault;
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
      {:else if isReleasable}
        <button
          type="button"
          on:click={() => dispatch('requestRelease', branch)}
          title={`${branchProtectionReason(branch)} — release to delete it`}
          class="w-3.5 h-3.5 flex items-center justify-center text-amber-400 hover:text-amber-300 flex-shrink-0"
        >
          <Unlock size={10} />
        </button>
      {:else}
        <span title={branchProtectionReason(branch)} class="w-3.5 h-3.5 flex items-center justify-center text-neutral-400 flex-shrink-0">
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
      <FolderGit2 size={11} class="text-neutral-400 flex-shrink-0" />
      <span class="truncate">{repoName || branch.repoPath}</span>
    </div>

    {#if branch.lastCommitSha}
      <span class="font-mono text-[11px] text-neutral-400">
        {branch.lastCommitSha}
      </span>
    {/if}
  </div>

  {#if branch.lastCommitMessage}
    <p class="text-[11px] text-neutral-400 truncate italic font-sans" title={branch.lastCommitMessage}>
      "{branch.lastCommitMessage}"
    </p>
  {/if}

  <div class="flex items-center justify-end gap-1.5 pt-1 border-t border-neutral-800/50 opacity-85 group-hover:opacity-100 transition-opacity">
    {#if isReleasable}
      <button
        type="button"
        on:click={() => dispatch('requestRelease', branch)}
        title="Free this branch from its worktree and delete it"
        class="px-2 py-1 rounded bg-neutral-800/80 hover:bg-amber-950/80 border border-neutral-700/60 hover:border-amber-700/60 text-neutral-300 hover:text-amber-300 flex items-center gap-1.5 transition-colors text-[11px]"
      >
        <Unlock size={12} />
        <span>Release</span>
      </button>
    {/if}
    <button
      type="button"
      on:click={() => dispatch('requestCheckout', branch)}
      title={hasWorktree
        ? `Go to the worktree at ${branch.checkedOutWorktreePath}`
        : 'Create a worktree that checks out this branch'}
      class="px-2 py-1 rounded bg-neutral-800/80 hover:bg-indigo-950/80 border border-neutral-700/60 hover:border-indigo-700/60 text-neutral-300 hover:text-indigo-300 flex items-center gap-1.5 transition-colors text-[11px]"
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
