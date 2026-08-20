<script lang="ts">
  import type { WorktreeInfo } from '../types';
  import { AlertTriangle, Trash2, X, ShieldAlert } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let isOpen: boolean = false;
  export let worktree: WorktreeInfo | null = null;
  export let repoPath: string = '';
  export let isDeleting: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    confirmDelete: { worktree: WorktreeInfo; repoPath: string; force: boolean };
  }>();

  let forceDelete: boolean = false;

  function close() {
    forceDelete = false;
    dispatch('close');
  }

  function handleConfirm() {
    if (!worktree) return;
    dispatch('confirmDelete', {
      worktree,
      repoPath,
      force: forceDelete
    });
  }
</script>

{#if isOpen && worktree}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-100">
    <div class="w-full max-w-sm bg-neutral-900 border border-neutral-800 rounded-xl p-4 shadow-2xl flex flex-col gap-3">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-rose-400">
          <Trash2 size={16} />
          <h2 class="text-xs font-semibold uppercase tracking-wider text-neutral-200">
            Remove Worktree
          </h2>
        </div>
        <button on:click={close} class="text-neutral-500 hover:text-neutral-300 p-1 rounded">
          <X size={14} />
        </button>
      </div>

      <!-- Warning & Details -->
      <div class="bg-neutral-950/80 rounded-lg p-3 border border-neutral-800/80 flex flex-col gap-2 text-xs">
        <div class="flex items-start gap-2">
          <AlertTriangle size={14} class="text-amber-400 flex-shrink-0 mt-0.5" />
          <div class="flex flex-col">
            <span class="font-mono text-neutral-200 font-medium break-all">
              {worktree.branch || '(detached HEAD)'}
            </span>
            <span class="text-neutral-500 text-[11px] break-all">
              {worktree.path}
            </span>
          </div>
        </div>

        {#if worktree.isDirty}
          <div class="mt-1 p-2 rounded bg-rose-950/60 border border-rose-800/50 flex items-start gap-1.5 text-rose-200 text-[11px]">
            <ShieldAlert size={14} class="flex-shrink-0 text-rose-400 mt-0.5" />
            <div>
              <p class="font-medium">Uncommitted changes detected!</p>
              <p class="text-rose-300/80 text-[10px]">
                This directory has uncommitted files. Removing it will discard any uncommitted work permanently.
              </p>
            </div>
          </div>
        {/if}
      </div>

      {#if worktree.isDirty}
        <label class="flex items-center gap-2 text-xs text-neutral-300 cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={forceDelete}
            class="rounded border-neutral-700 bg-neutral-950 text-rose-500 focus:ring-rose-500 focus:ring-offset-neutral-900"
          />
          <span>Force delete even with uncommitted changes</span>
        </label>
      {/if}

      <!-- Actions -->
      <div class="flex items-center justify-end gap-2 pt-1">
        <button
          on:click={close}
          disabled={isDeleting}
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 transition-colors"
        >
          Cancel
        </button>
        <button
          on:click={handleConfirm}
          disabled={isDeleting || (worktree.isDirty && !forceDelete)}
          class="px-3 py-1.5 rounded-lg text-xs font-medium bg-rose-600 hover:bg-rose-500 disabled:opacity-40 disabled:hover:bg-rose-600 text-white transition-colors flex items-center gap-1.5 shadow-lg shadow-rose-600/20"
        >
          {#if isDeleting}
            <span>Removing...</span>
          {:else}
            <Trash2 size={13} />
            <span>Remove Worktree</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
