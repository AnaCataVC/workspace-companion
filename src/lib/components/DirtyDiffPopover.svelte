<script lang="ts">
  import { Flame } from 'lucide-svelte';
  import type { WorktreeDiffSummary } from '../types';
  import { invoke } from '@tauri-apps/api/core';
  import { toErrorMessage } from '../utils/errors';

  export let worktreePath: string;
  export let label: string;
  export let badgeClass: string;
  export let iconSize: number = 10;
  export let popoverPositionClass: string = 'right-0 top-full mt-1.5';

  const DIFF_FETCH_DEBOUNCE_MS = 180;

  let showDiffPopover = false;
  let diffSummary: WorktreeDiffSummary | null = null;
  let isLoadingDiff = false;
  let diffError: string | null = null;
  let diffFetchTimer: ReturnType<typeof setTimeout> | null = null;

  function handleMouseEnter() {
    showDiffPopover = true;
    if (diffSummary || isLoadingDiff) return;
    clearDiffFetchTimer();
    diffFetchTimer = setTimeout(() => {
      diffFetchTimer = null;
      fetchDiffSummary();
    }, DIFF_FETCH_DEBOUNCE_MS);
  }

  function handleMouseLeave() {
    showDiffPopover = false;
    clearDiffFetchTimer();
  }

  function clearDiffFetchTimer() {
    if (diffFetchTimer) {
      clearTimeout(diffFetchTimer);
      diffFetchTimer = null;
    }
  }

  async function fetchDiffSummary() {
    if (diffSummary || isLoadingDiff) return;
    isLoadingDiff = true;
    diffError = null;
    try {
      diffSummary = await invoke<WorktreeDiffSummary>('get_worktree_diff_summary', {
        worktreePath
      });
    } catch (err: unknown) {
      diffError = toErrorMessage(err, 'Failed to inspect diff');
    } finally {
      isLoadingDiff = false;
    }
  }
</script>

<div class="relative" role="group" on:mouseenter={handleMouseEnter} on:mouseleave={handleMouseLeave}>
  <span class={badgeClass}>
    <Flame size={iconSize} class="text-rose-400 animate-pulse" />
    {label}
  </span>

  <!-- Lazy Diff Popover -->
  {#if showDiffPopover}
    <div
      class="absolute {popoverPositionClass} z-40 w-64 p-2 rounded-lg bg-neutral-900 border border-neutral-700 shadow-2xl text-[11px] text-neutral-200 pointer-events-none"
    >
      <div class="font-semibold text-[10px] text-rose-400 mb-1 flex items-center justify-between border-b border-neutral-800 pb-1">
        <span>Uncommitted Changes</span>
        {#if isLoadingDiff}
          <span class="text-neutral-500 animate-pulse text-[9px]">Analyzing...</span>
        {/if}
      </div>

      {#if diffSummary}
        <p class="text-neutral-300 font-mono text-[10px] mb-1.5">
          {diffSummary.summaryText}
        </p>
        {#if diffSummary.modifiedFiles.length > 0}
          <div class="max-h-28 overflow-y-auto space-y-0.5 font-mono text-[9px] text-neutral-400">
            {#each diffSummary.modifiedFiles as file}
              <div class="truncate text-neutral-300">{file}</div>
            {/each}
          </div>
        {/if}
      {:else if diffError}
        <span class="text-rose-400 text-[10px]">{diffError}</span>
      {:else if !isLoadingDiff}
        <span class="text-neutral-400 text-[10px]">Hovered to inspect git diff</span>
      {/if}
    </div>
  {/if}
</div>
