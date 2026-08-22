<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { RepositoryWorktrees, BranchEntry } from '../types';
  import {
    Plus,
    GitBranch,
    FolderPlus,
    AlertCircle,
    X,
    Loader2,
    Check,
    FolderGit2,
    Sparkles
  } from 'lucide-svelte';

  export let isOpen: boolean = false;
  export let repositories: RepositoryWorktrees[] = [];
  export let initialRepoPath: string = '';
  export let isCreating: boolean = false;
  export let errorMessage: string | null = null;
  export let onSuggestPath: ((repoPath: string, branchName: string) => Promise<{ suggestedPath: string; alreadyExists: boolean }>) | null = null;
  export let onFetchBranches: ((repoPath: string) => Promise<BranchEntry[]>) | null = null;

  const dispatch = createEventDispatcher<{
    close: void;
    create: {
      repoPath: string;
      targetPath: string;
      baseBranch: string;
      newBranchName?: string;
    };
  }>();

  let selectedRepoPath: string = '';
  let mode: 'new' | 'existing' = 'new';
  let baseBranch: string = 'main';
  let existingBranch: string = '';
  let newBranchName: string = '';
  let targetPath: string = '';
  let isPathColliding: boolean = false;
  let availableBranches: BranchEntry[] = [];
  let isLoadingBranches: boolean = false;
  let isSuggestingPath: boolean = false;

  $: if (isOpen && repositories.length > 0) {
    if (!selectedRepoPath || !repositories.some(r => r.repoPath === selectedRepoPath)) {
      selectedRepoPath = initialRepoPath || repositories[0]?.repoPath || '';
    }
  }

  $: if (selectedRepoPath) {
    loadBranchesForRepo(selectedRepoPath);
  }

  async function loadBranchesForRepo(repoPath: string) {
    if (!onFetchBranches || !repoPath) return;
    isLoadingBranches = true;
    try {
      const branches = await onFetchBranches(repoPath);
      availableBranches = branches;
      const mainBranch = branches.find(b => b.shortName === 'main' || b.shortName === 'master');
      if (mainBranch) {
        baseBranch = mainBranch.shortName;
      } else if (branches.length > 0) {
        baseBranch = branches[0].shortName;
      }
      
      const unassigned = branches.filter(b => !b.isLockedByOther && !b.isCurrent);
      if (unassigned.length > 0) {
        existingBranch = unassigned[0].shortName;
      }
    } catch (e) {
      console.error('Failed to load branches:', e);
    } finally {
      isLoadingBranches = false;
      updateSuggestedPath();
    }
  }

  $: activeBranchToSuggest = mode === 'new' ? (newBranchName || 'feature') : (existingBranch || 'branch');

  $: if (selectedRepoPath && activeBranchToSuggest) {
    updateSuggestedPath();
  }

  async function updateSuggestedPath() {
    if (!onSuggestPath || !selectedRepoPath) return;
    isSuggestingPath = true;
    try {
      const res = await onSuggestPath(selectedRepoPath, activeBranchToSuggest);
      targetPath = res.suggestedPath;
      isPathColliding = res.alreadyExists;
    } catch (e) {
      console.error('Failed to suggest path:', e);
    } finally {
      isSuggestingPath = false;
    }
  }

  function handleCreate() {
    if (!selectedRepoPath || !targetPath.trim() || isCreating) return;

    if (mode === 'new') {
      const cleanBranch = newBranchName.trim();
      if (!cleanBranch) return;
      dispatch('create', {
        repoPath: selectedRepoPath,
        targetPath: targetPath.trim(),
        baseBranch,
        newBranchName: cleanBranch
      });
    } else {
      if (!existingBranch) return;
      dispatch('create', {
        repoPath: selectedRepoPath,
        targetPath: targetPath.trim(),
        baseBranch: existingBranch
      });
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      dispatch('close');
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-lg rounded-xl bg-neutral-900 border border-neutral-800 shadow-2xl p-4 flex flex-col gap-3.5 text-neutral-200"
    >
      <!-- Header -->
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-indigo-950/80 text-indigo-400 border border-indigo-800/40">
            <FolderPlus size={16} />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-neutral-100">Create New Git Worktree</h3>
            <p class="text-[11px] text-neutral-400">
              Spin up an isolated working tree for parallel development
            </p>
          </div>
        </div>
        <button
          on:click={() => dispatch('close')}
          disabled={isCreating}
          class="text-neutral-500 hover:text-neutral-300 p-1 rounded-md hover:bg-neutral-800 transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      <!-- Error alert -->
      {#if errorMessage}
        <div class="rounded-lg bg-rose-950/80 border border-rose-800/60 p-2.5 flex items-start gap-2 text-rose-300 text-xs">
          <AlertCircle size={14} class="flex-shrink-0 mt-0.5" />
          <span>{errorMessage}</span>
        </div>
      {/if}

      <!-- Target Repository Selection -->
      <div class="space-y-1">
        <label for="new-wt-repo-select" class="text-[11px] font-medium text-neutral-400 flex items-center gap-1">
          <FolderGit2 size={12} /> Target Repository
        </label>
        <select
          id="new-wt-repo-select"
          bind:value={selectedRepoPath}
          disabled={isCreating}
          class="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-2.5 py-1.5 text-xs text-neutral-200 focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        >
          {#each repositories as repo (repo.repoPath)}
            <option value={repo.repoPath}>{repo.repoName} ({repo.repoPath})</option>
          {/each}
        </select>
      </div>

      <!-- Mode Selector Tabs -->
      <div class="flex rounded-lg bg-neutral-950 p-1 border border-neutral-800 text-xs">
        <button
          type="button"
          on:click={() => (mode = 'new')}
          class="flex-1 py-1 rounded-md font-medium transition-all flex items-center justify-center gap-1.5 {mode === 'new' ? 'bg-indigo-600 text-white shadow-xs' : 'text-neutral-400 hover:text-neutral-200'}"
        >
          <Sparkles size={12} /> New Branch
        </button>
        <button
          type="button"
          on:click={() => (mode = 'existing')}
          class="flex-1 py-1 rounded-md font-medium transition-all flex items-center justify-center gap-1.5 {mode === 'existing' ? 'bg-indigo-600 text-white shadow-xs' : 'text-neutral-400 hover:text-neutral-200'}"
        >
          <GitBranch size={12} /> Existing Branch
        </button>
      </div>

      {#if mode === 'new'}
        <!-- New Branch Inputs -->
        <div class="grid grid-cols-2 gap-2.5">
          <div class="space-y-1">
            <label for="new-wt-branch-input" class="text-[11px] font-medium text-neutral-400 flex items-center gap-1">
              <GitBranch size={12} /> New Branch Name
            </label>
            <input
              id="new-wt-branch-input"
              type="text"
              bind:value={newBranchName}
              placeholder="e.g. feat/dashboard-redesign"
              class="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-2.5 py-1.5 text-xs text-neutral-200 placeholder-neutral-600 font-mono focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
            />
          </div>

          <div class="space-y-1">
            <label for="new-wt-base-branch" class="text-[11px] font-medium text-neutral-400">
              Base Branch (Source)
            </label>
            <select
              id="new-wt-base-branch"
              bind:value={baseBranch}
              disabled={isCreating || isLoadingBranches}
              class="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-2.5 py-1.5 text-xs text-neutral-200 font-mono focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
            >
              {#each availableBranches as b (b.name)}
                <option value={b.shortName}>{b.shortName} {b.isRemote ? '(remote)' : ''}</option>
              {/each}
            </select>
          </div>
        </div>
      {:else}
        <!-- Existing Branch Selector -->
        <div class="space-y-1">
          <label for="new-wt-existing-branch" class="text-[11px] font-medium text-neutral-400 flex items-center gap-1">
            <GitBranch size={12} /> Choose Existing Branch
          </label>
          <select
            id="new-wt-existing-branch"
            bind:value={existingBranch}
            disabled={isCreating || isLoadingBranches}
            class="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-2.5 py-1.5 text-xs text-neutral-200 font-mono focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
          >
            {#each availableBranches.filter(b => !b.isCurrent && !b.isLockedByOther) as b (b.name)}
              <option value={b.shortName}>{b.shortName} {b.isRemote ? '(remote)' : ''}</option>
            {/each}
          </select>
        </div>
      {/if}

      <!-- Target Worktree Directory Path -->
      <div class="space-y-1">
        <div class="flex items-center justify-between text-[11px]">
          <label for="new-wt-target-path" class="font-medium text-neutral-400">Target Folder Location</label>
          <span class="text-[10px] text-indigo-400 font-sans">
            Auto-suggested sibling layout
          </span>
        </div>
        <input
          id="new-wt-target-path"
          type="text"
          bind:value={targetPath}
          placeholder="C:\Users\...\Repos\project-feature"
          class="w-full bg-neutral-950 border border-neutral-800 rounded-lg px-2.5 py-1.5 text-xs text-neutral-200 font-mono focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
        />
        {#if isPathColliding}
          <p class="text-[10px] text-amber-400 flex items-center gap-1">
            <AlertCircle size={10} /> Folder already exists on disk.
          </p>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="pt-2 border-t border-neutral-800 flex items-center justify-end gap-2 text-xs">
        <button
          type="button"
          on:click={() => dispatch('close')}
          disabled={isCreating}
          class="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 transition-colors"
        >
          Cancel
        </button>

        <button
          type="button"
          on:click={handleCreate}
          disabled={isCreating || (mode === 'new' && !newBranchName.trim()) || (mode === 'existing' && !existingBranch)}
          class="px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium flex items-center gap-1.5 shadow-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isCreating}
            <Loader2 size={13} class="animate-spin" />
            <span>Creating Worktree...</span>
          {:else}
            <Plus size={13} />
            <span>Create Worktree</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
