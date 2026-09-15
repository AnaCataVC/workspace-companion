<script lang="ts">
  import type { BranchStatusEntry } from '../types';
  import { GitBranch, Lock, GitMerge, GitPullRequestClosed, ShieldCheck } from 'lucide-svelte';
  import { branchSelection, selectedBranchKeys, branchSelectionKey } from '../stores/branchSelection';

  export let branch: BranchStatusEntry;
  export let repoName: string = '';

  $: isProtected = branch.isDefault || branch.isCheckedOut;
  $: isSelected = $selectedBranchKeys.has(branchSelectionKey({ repoPath: branch.repoPath, branchName: branch.name }));

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

  function protectionReason(b: BranchStatusEntry): string {
    if (b.isDefault) return 'Default branch — never deletable';
    if (b.isCheckedOut) return `Checked out at ${b.checkedOutWorktreePath || 'a worktree'}`;
    return '';
  }
</script>

<div
  role="group"
  class="group flex items-center justify-between px-2.5 py-1.5 rounded-md border transition-all text-xs
    {isSelected
      ? 'bg-rose-950/30 border-rose-800/60 shadow-xs'
      : isProtected
      ? 'bg-neutral-900/30 border-neutral-850/60'
      : 'bg-neutral-900/60 hover:bg-neutral-850/90 border-neutral-800/60 hover:border-neutral-700/80'}"
>
  <div class="flex items-center gap-2 min-w-0 flex-1">
    {#if !isProtected}
      <input
        type="checkbox"
        checked={isSelected}
        on:change={toggleSelection}
        title="Select branch for batch delete"
        class="w-3.5 h-3.5 rounded border-neutral-700 bg-neutral-950 text-rose-500 focus:ring-rose-500/30 focus:ring-offset-0 cursor-pointer flex-shrink-0"
      />
    {:else}
      <span title={protectionReason(branch)} class="w-3.5 h-3.5 flex items-center justify-center text-neutral-600 flex-shrink-0">
        <Lock size={10} />
      </span>
    {/if}

    <GitBranch size={11} class="text-neutral-500 flex-shrink-0" />
    <span class="font-mono font-medium truncate text-[11px] text-neutral-200" title={branch.name}>
      {branch.name}
    </span>

    {#if repoName}
      <span class="text-neutral-500 text-[10px] truncate max-w-[110px]" title={branch.repoPath}>
        ({repoName})
      </span>
    {/if}

    {#if branch.lastCommitMessage}
      <span class="text-neutral-600 text-[10px] truncate hidden sm:inline-block" title={branch.lastCommitMessage}>
        {branch.lastCommitMessage}
      </span>
    {/if}
  </div>

  <div class="flex items-center gap-1 flex-shrink-0 ml-2">
    {#if isProtected}
      <span
        class="px-1.5 py-0.5 rounded bg-indigo-950/70 text-indigo-300 border border-indigo-800/50 text-[9px] flex items-center gap-0.5"
        title={protectionReason(branch)}
      >
        <ShieldCheck size={9} />
        Protected
      </span>
    {/if}
    {#if branch.isMerged}
      <span class="px-1.5 py-0.5 rounded bg-emerald-950/70 text-emerald-300 border border-emerald-800/50 text-[9px] flex items-center gap-0.5">
        <GitMerge size={9} />
        Merged
      </span>
    {/if}
    {#if branch.isRemoteGone}
      <span class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[9px] flex items-center gap-0.5">
        <GitPullRequestClosed size={9} />
        Remote gone
      </span>
    {/if}
  </div>
</div>
