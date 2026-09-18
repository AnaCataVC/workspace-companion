<script lang="ts">
  import { Plus, Settings2, Loader2, CornerDownLeft, AlertCircle } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { CreateWorktreeResult, SuggestWorktreePathResult, SupportedEditor, WorktreeInfo } from '../types';
  import { appConfig } from '../stores/appConfig';
  import { toErrorMessage } from '../utils/errors';

  export let repoPath: string;
  export let defaultBranch: string = 'main';

  const dispatch = createEventDispatcher<{
    worktreeCreated: { repoPath: string; worktreeInfo?: WorktreeInfo };
    openAdvancedModal: string;
    openEditor: { editor: SupportedEditor; path: string };
  }>();

  let isExpanded = false;
  let branchInput = '';
  let suggestedPath = '';
  let isCreating = false;
  let errorMessage: string | null = null;

  function sanitizeSlug(name: string): string {
    return name
      .trim()
      .toLowerCase()
      .replace(/[\/\\]/g, '-')
      .replace(/[^a-z0-9-_]/g, '');
  }

  $: {
    if (branchInput.trim()) {
      const slug = sanitizeSlug(branchInput);
      const parts = repoPath.replace(/\\/g, '/').split('/');
      const repoName = parts[parts.length - 1] || 'repo';
      suggestedPath = `${repoName}-${slug}`;
    } else {
      suggestedPath = '';
    }
  }

  function handleFocus() {
    isExpanded = true;
    errorMessage = null;
  }

  function handleBlur() {
    if (!branchInput.trim() && !isCreating) {
      isExpanded = false;
      errorMessage = null;
    }
  }

  async function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      await submitCreate();
    } else if (e.key === 'Escape') {
      branchInput = '';
      isExpanded = false;
      errorMessage = null;
    }
  }

  async function submitCreate() {
    const branchName = branchInput.trim();
    if (!branchName || isCreating) return;

    isCreating = true;
    errorMessage = null;

    try {
      // 1. Get full suggested path from backend
      const pathRes = await invoke<SuggestWorktreePathResult>('suggest_worktree_path', {
        repoPath,
        branchName
      });

      // 2. Create worktree
      const createRes = await invoke<CreateWorktreeResult>('create_worktree', {
        repoPath,
        targetPath: pathRes.suggestedPath,
        baseBranch: defaultBranch,
        newBranchName: branchName
      });

      if (createRes.success) {
        const editor = ($appConfig.defaultEditor as SupportedEditor) || 'vscode';
        
        dispatch('openEditor', { editor, path: createRes.worktreePath });
        dispatch('worktreeCreated', {
          repoPath,
          worktreeInfo: createRes.worktreeInfo
        });

        // Reset
        branchInput = '';
        isExpanded = false;
      }
    } catch (err: unknown) {
      errorMessage = toErrorMessage(err, 'Failed to create worktree');
    } finally {
      isCreating = false;
    }
  }
</script>

<div class="mt-1">
  <div
    class="flex items-center justify-between gap-1.5 px-2 py-1 rounded-md border transition-all text-xs
      {isExpanded
        ? 'bg-neutral-900 border-indigo-500/60 ring-1 ring-indigo-500/20'
        : 'bg-neutral-900/40 hover:bg-neutral-900/80 border-dashed border-neutral-800/80 hover:border-neutral-700'}"
  >
    <div class="flex items-center gap-1.5 min-w-0 flex-1">
      {#if isCreating}
        <Loader2 size={12} class="text-indigo-400 animate-spin flex-shrink-0" />
      {:else}
        <Plus size={12} class="text-neutral-500 flex-shrink-0" />
      {/if}

      <input
        bind:value={branchInput}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeyDown}
        disabled={isCreating}
        type="text"
        placeholder="Quick branch name (e.g. feat/auth, fix/bug)... [Enter to create]"
        class="bg-transparent text-xs text-neutral-200 placeholder-neutral-500 focus:outline-none w-full font-mono"
      />

      {#if suggestedPath && !isCreating}
        <span class="text-[10px] text-indigo-300 font-mono hidden md:inline-block bg-indigo-950/70 border border-indigo-800/50 px-1.5 py-0.2 rounded truncate max-w-[150px]">
          📁 {suggestedPath}
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-1 flex-shrink-0">
      {#if branchInput.trim() && !isCreating}
        <button
          type="button"
          on:click={submitCreate}
          title="Create worktree now (Enter)"
          class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[10px] font-medium transition-colors"
        >
          <span>Create</span>
          <CornerDownLeft size={9} />
        </button>
      {/if}

      <!-- Advanced Modal Launcher -->
      <button
        type="button"
        on:click={() => dispatch('openAdvancedModal', repoPath)}
        title="Open Advanced Worktree Creator"
        class="p-1 rounded hover:bg-neutral-800 text-neutral-500 hover:text-neutral-300 transition-colors"
      >
        <Settings2 size={11} />
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="flex items-center gap-1 px-2 py-1 mt-1 rounded bg-rose-950/70 border border-rose-800/50 text-rose-300 text-[10px]">
      <AlertCircle size={10} class="flex-shrink-0" />
      <span class="truncate">{errorMessage}</span>
    </div>
  {/if}
</div>
