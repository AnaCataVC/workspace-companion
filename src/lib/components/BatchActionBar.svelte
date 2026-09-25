<script lang="ts">
  import { batchSelection, selectedCount, hasDirtySelected } from '../stores/batchSelection';
  import { Trash2, X, Flame } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    openBatchDeleteModal: void;
  }>();

  function clearSelection() {
    batchSelection.clear();
  }

  function handleOpenModal() {
    dispatch('openBatchDeleteModal');
  }
</script>

{#if $selectedCount > 0}
  <div class="fixed bottom-3 inset-x-3 z-40 bg-neutral-950/95 border border-rose-800/60 rounded-xl p-2.5 shadow-2xl backdrop-blur-md flex items-center justify-between animate-in slide-in-from-bottom-2 duration-150 select-none">
    <!-- Left Info -->
    <div class="flex items-center gap-2 text-xs">
      <span class="flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full bg-rose-600 text-white font-mono text-[11px] font-bold">
        {$selectedCount}
      </span>
      <span class="text-neutral-200 font-medium">
        {$selectedCount === 1 ? '1 worktree selected' : `${$selectedCount} worktrees selected`}
      </span>

      {#if $hasDirtySelected}
        <span class="px-1.5 py-0.5 rounded bg-rose-950 text-rose-300 border border-rose-800/80 text-[11px] flex items-center gap-1 font-mono">
          <Flame size={10} class="text-rose-400 animate-pulse" />
          <span>contains dirty</span>
        </span>
      {/if}
    </div>

    <!-- Right Actions -->
    <div class="flex items-center gap-2">
      <button
        type="button"
        on:click={clearSelection}
        class="px-2.5 py-1 text-xs text-neutral-400 hover:text-neutral-200 rounded hover:bg-neutral-900 transition-colors flex items-center gap-1"
      >
        <X size={12} />
        <span>Cancel</span>
      </button>

      <button
        type="button"
        on:click={handleOpenModal}
        class="px-3 py-1 text-xs font-semibold rounded-lg bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-600/25 flex items-center gap-1.5 transition-all"
      >
        <Trash2 size={12} />
        <span>Delete Selected</span>
      </button>
    </div>
  </div>
{/if}
