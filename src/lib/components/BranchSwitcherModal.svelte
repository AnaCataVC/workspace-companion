<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { WorktreeInfo, WorktreeBranchesResponse, BranchEntry } from '../types';
  import {
    GitBranch,
    Search,
    Lock,
    Check,
    AlertTriangle,
    X,
    Loader2,
    Globe,
    GitCommit
  } from 'lucide-svelte';
  import { closeOnEscape } from '../actions/closeOnEscape';

  export let isOpen: boolean = false;
  export let worktree: WorktreeInfo | null = null;
  export let repoPath: string = '';
  export let branchesResponse: WorktreeBranchesResponse | null = null;
  export let isLoadingBranches: boolean = false;
  export let isSwitching: boolean = false;
  export let errorMessage: string | null = null;

  const dispatch = createEventDispatcher<{
    close: void;
    switchBranch: { worktree: WorktreeInfo; targetBranch: string };
  }>();

  let searchQuery: string = '';
  let highlightedIndex: number = 0;
  let branchButtons: (HTMLButtonElement | null)[] = [];

  $: filteredBranches = (branchesResponse?.branches || []).filter(b => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    return b.name.toLowerCase().includes(q) || b.shortName.toLowerCase().includes(q) || (b.lastCommitMessage && b.lastCommitMessage.toLowerCase().includes(q));
  });

  // Reset the highlight to the top result whenever the search filter changes the visible set
  $: {
    searchQuery;
    highlightedIndex = 0;
  }

  // Clamp defensively in case the list shrinks (e.g. branches reload) without the query changing
  $: if (filteredBranches.length > 0 && highlightedIndex > filteredBranches.length - 1) {
    highlightedIndex = filteredBranches.length - 1;
  } else if (filteredBranches.length === 0) {
    highlightedIndex = 0;
  }

  $: branchButtons[highlightedIndex]?.scrollIntoView({ block: 'nearest' });

  function getShortBranch(fullBranch: string | null): string {
    if (!fullBranch) return '(detached HEAD)';
    return fullBranch.replace('refs/heads/', '');
  }

  function handleSelectBranch(branch: BranchEntry) {
    if (!worktree || branch.isCurrent || branch.isLockedByOther || isSwitching || worktree.isDirty) {
      return;
    }
    dispatch('switchBranch', {
      worktree,
      targetBranch: branch.isRemote ? branch.name : branch.shortName
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    if (e.key === 'ArrowDown') {
      if (filteredBranches.length === 0) return;
      e.preventDefault();
      highlightedIndex = (highlightedIndex + 1) % filteredBranches.length;
      return;
    }

    if (e.key === 'ArrowUp') {
      if (filteredBranches.length === 0) return;
      e.preventDefault();
      highlightedIndex = (highlightedIndex - 1 + filteredBranches.length) % filteredBranches.length;
      return;
    }

    if (e.key === 'Enter') {
      const branch = filteredBranches[highlightedIndex];
      if (branch) {
        e.preventDefault();
        handleSelectBranch(branch);
      }
    }
  }
</script>

<svelte:window
  on:keydown={handleKeydown}
  use:closeOnEscape={{ enabled: () => isOpen, onClose: () => dispatch('close') }}
/>

{#if isOpen && worktree}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-md rounded-xl bg-neutral-900 border border-neutral-800 shadow-2xl p-4 flex flex-col gap-3 text-neutral-200 max-h-[85vh]"
    >
      <!-- Header -->
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-indigo-950/80 text-indigo-400 border border-indigo-800/40">
            <GitBranch size={16} />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-neutral-100">Switch Worktree Branch</h3>
            <p class="text-[11px] text-neutral-400 font-mono truncate max-w-[280px]" title="{repoPath} • {worktree.path}">
              Current: <span class="text-indigo-300 font-medium">{getShortBranch(worktree.branch)}</span>
            </p>
          </div>
        </div>
        <button
          on:click={() => dispatch('close')}
          disabled={isSwitching}
          class="text-neutral-500 hover:text-neutral-300 p-1 rounded-md hover:bg-neutral-800 transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      <!-- Warning: Dirty Worktree -->
      {#if worktree.isDirty}
        <div class="rounded-lg bg-rose-950/70 border border-rose-800/60 p-2.5 flex items-start gap-2 text-rose-300 text-xs">
          <AlertTriangle size={14} class="flex-shrink-0 mt-0.5" />
          <div class="space-y-0.5">
            <p class="font-medium">Cannot switch branch: Uncommitted changes</p>
            <p class="text-[11px] text-rose-300/80">
              {worktree.uncommittedFilesCount ? `${worktree.uncommittedFilesCount} modified/untracked files` : 'Modified files'} detected. Please commit, stash, or discard them first.
            </p>
          </div>
        </div>
      {/if}

      <!-- Error Message -->
      {#if errorMessage}
        <div class="rounded-lg bg-rose-950/80 border border-rose-800/60 p-2 text-rose-300 text-xs">
          {errorMessage}
        </div>
      {/if}

      <!-- Search Box -->
      <div class="relative">
        <Search size={13} class="absolute left-2.5 top-2.5 text-neutral-500" />
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Filter branches (e.g. main, feat/login)..."
          class="w-full bg-neutral-950 border border-neutral-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-neutral-200 placeholder-neutral-500 focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all font-mono"
        />
      </div>

      <!-- Branch List -->
      <div class="flex-1 overflow-y-auto min-h-[160px] max-h-[260px] space-y-1 pr-1">
        {#if isLoadingBranches}
          <div class="h-36 flex flex-col items-center justify-center gap-2 text-neutral-500">
            <Loader2 size={20} class="animate-spin text-indigo-400" />
            <span class="text-xs">Loading repository branches...</span>
          </div>
        {:else if filteredBranches.length === 0}
          <div class="h-36 flex flex-col items-center justify-center text-neutral-500 text-xs text-center p-4">
            <span>No matching branches found</span>
          </div>
        {:else}
          {#each filteredBranches as branch, idx (branch.name)}
            <button
              bind:this={branchButtons[idx]}
              on:click={() => handleSelectBranch(branch)}
              on:mouseenter={() => (highlightedIndex = idx)}
              disabled={branch.isCurrent || branch.isLockedByOther || isSwitching || (worktree.isDirty ?? false)}
              class="w-full text-left p-2 rounded-lg border transition-all flex items-center justify-between text-xs font-mono
                {branch.isCurrent
                  ? 'bg-indigo-950/40 border-indigo-800/50 text-indigo-300 cursor-default'
                  : branch.isLockedByOther
                    ? 'bg-neutral-950/40 border-neutral-800/40 text-neutral-500 opacity-60 cursor-not-allowed'
                    : worktree.isDirty
                      ? 'bg-neutral-950/20 border-neutral-800/30 text-neutral-400 opacity-60 cursor-not-allowed'
                      : 'bg-neutral-950/50 hover:bg-neutral-800 border-neutral-800/60 hover:border-neutral-700 text-neutral-200 cursor-pointer'}
                {idx === highlightedIndex ? 'ring-2 ring-indigo-500/80 ring-offset-1 ring-offset-neutral-900 border-indigo-500/60' : ''}"
            >
              <div class="flex items-center gap-2 min-w-0 flex-1">
                {#if branch.isRemote}
                  <Globe size={13} class="text-neutral-500 flex-shrink-0" />
                {:else}
                  <GitBranch size={13} class={branch.isCurrent ? 'text-indigo-400' : 'text-neutral-400'} />
                {/if}

                <div class="flex flex-col min-w-0 flex-1">
                  <div class="flex items-center gap-1.5">
                    <span class="truncate font-medium {branch.isCurrent ? 'text-indigo-300' : 'text-neutral-200'}">
                      {branch.shortName}
                    </span>
                    {#if branch.isRemote}
                      <span class="text-[9px] font-sans px-1 py-0.2 rounded bg-neutral-800 text-neutral-400">
                        remote
                      </span>
                    {/if}
                  </div>

                  {#if branch.lastCommitMessage}
                    <span class="text-[10px] text-neutral-500 truncate font-sans italic">
                      {branch.lastCommitMessage}
                    </span>
                  {/if}
                </div>
              </div>

              <!-- Status badge or action -->
              <div class="flex items-center gap-1 flex-shrink-0 ml-2">
                {#if branch.isCurrent}
                  <span class="text-[10px] text-indigo-400 flex items-center gap-0.5 font-sans">
                    <Check size={12} /> Active
                  </span>
                {:else if branch.isLockedByOther}
                  <span class="text-[10px] text-amber-500/80 flex items-center gap-1 font-sans" title={`Checked out in ${branch.lockedWorktreePath}`}>
                    <Lock size={11} /> Locked
                  </span>
                {:else if branch.lastCommitSha}
                  <span class="text-[10px] text-neutral-600 font-mono flex items-center gap-0.5">
                    <GitCommit size={10} /> {branch.lastCommitSha}
                  </span>
                {/if}
              </div>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Footer -->
      <div class="pt-2 border-t border-neutral-800 flex items-center justify-between text-[11px] text-neutral-400">
        <span class="text-neutral-500 font-sans">
          ↑↓ to navigate, Enter to switch
        </span>
        <button
          on:click={() => dispatch('close')}
          disabled={isSwitching}
          class="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
