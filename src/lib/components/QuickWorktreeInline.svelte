<script lang="ts">
  import { Plus, Settings2, Loader2, CornerDownLeft, AlertCircle } from 'lucide-svelte';
  import { createEventDispatcher, onDestroy } from 'svelte';
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

  const SUGGEST_DEBOUNCE_MS = 200;

  let isExpanded = false;
  let branchInput = '';
  let suggestedPath = '';
  let isCreating = false;
  let errorMessage: string | null = null;
  let suggestTimer: ReturnType<typeof setTimeout> | null = null;
  // Only the latest suggestion may apply: responses can arrive out of order while typing.
  let suggestSeq = 0;

  // The preview shows the exact folder the backend will create, not a local approximation.
  $: scheduleSuggestion(branchInput.trim());

  function scheduleSuggestion(branchName: string) {
    const seq = ++suggestSeq;
    if (suggestTimer) clearTimeout(suggestTimer);
    if (!branchName) {
      suggestedPath = '';
      return;
    }
    suggestTimer = setTimeout(async () => {
      try {
        const res = await invoke<SuggestWorktreePathResult>('suggest_worktree_path', { repoPath, branchName });
        if (seq === suggestSeq) suggestedPath = res.suggestedPath;
      } catch {
        // Preview only: the create call reports any real error.
        if (seq === suggestSeq) suggestedPath = '';
      }
    }, SUGGEST_DEBOUNCE_MS);
  }

  onDestroy(() => {
    if (suggestTimer) clearTimeout(suggestTimer);
  });

  function folderName(fullPath: string): string {
    const parts = fullPath.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || fullPath;
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
      // Consumed here: the panel-level Escape must not also hide the window.
      e.preventDefault();
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
      } else {
        errorMessage = createRes.message || 'Worktree was not created';
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
        <Plus size={12} class="text-neutral-400 flex-shrink-0" />
      {/if}

      <input
        bind:value={branchInput}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeyDown}
        disabled={isCreating}
        type="text"
        placeholder="Quick branch name (e.g. feat/auth, fix/bug)... [Enter to create]"
        aria-label="Quick new worktree branch name"
        class="bg-transparent text-xs text-neutral-200 placeholder-neutral-500 focus:outline-none w-full font-mono"
      />

      {#if suggestedPath && !isCreating}
        <span
          class="text-[11px] text-indigo-300 font-mono hidden md:inline-block bg-indigo-950/70 border border-indigo-800/50 px-1.5 py-0.2 rounded truncate max-w-[220px]"
          title={`New branch from ${defaultBranch}, created at ${suggestedPath}`}
        >
          from {defaultBranch} → {folderName(suggestedPath)}
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-1 flex-shrink-0">
      {#if branchInput.trim() && !isCreating}
        <button
          type="button"
          on:click={submitCreate}
          title="Create worktree now (Enter)"
          class="flex items-center gap-1 px-2 min-h-6 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[11px] font-medium transition-colors"
        >
          <span>Create</span>
          <CornerDownLeft size={10} />
        </button>
      {/if}

      <!-- Advanced Modal Launcher -->
      <button
        type="button"
        on:click={() => dispatch('openAdvancedModal', repoPath)}
        title="Open Advanced Worktree Creator"
        aria-label="Open Advanced Worktree Creator"
        class="min-w-6 min-h-6 flex items-center justify-center rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-300 transition-colors"
      >
        <Settings2 size={12} />
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="flex items-center gap-1 px-2 py-1 mt-1 rounded bg-rose-950/70 border border-rose-800/50 text-rose-300 text-[11px]" role="alert">
      <AlertCircle size={11} class="flex-shrink-0" />
      <span class="truncate" title={errorMessage}>{errorMessage}</span>
    </div>
  {/if}
</div>
