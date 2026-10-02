<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { WorktreeInfo, WorktreeDiffSummary } from '../types';
  import { invoke } from '@tauri-apps/api/core';
  import { toErrorMessage } from '../utils/errors';
  import { notifications } from '../stores/notifications';
  import { AlertTriangle, Loader2, X } from 'lucide-svelte';

  export let worktreePath: string;
  export let uncommittedFilesCount: number = 0;
  /** Extra guard from the host (e.g. a sibling action for the same worktree is running). */
  export let disabled: boolean = false;
  /** Fired once the backend discard call succeeds, with the refreshed worktree. */
  export let ondiscarded: (updatedWorktree: WorktreeInfo) => void = () => {};
  /** Fired when the user dismisses this panel without discarding. */
  export let oncancel: () => void = () => {};
  /** Fired whenever this panel's own async work is in flight, so the host can sync `set_panel_state`. */
  export let onbusychange: (busy: boolean) => void = () => {};

  // The confirm button only arms after a delay, so a double-click on the trigger that opened this
  // panel can never land on the destructive confirm.
  const DISCARD_ARM_DELAY_MS = 800;

  let isDiscardArmed = false;
  let armTimer: ReturnType<typeof setTimeout> | null = null;
  let discardPreview: WorktreeDiffSummary | null = null;
  let discardPreviewError: string | null = null;
  let discardError: string | null = null;
  let isDiscarding = false;

  $: onbusychange(isDiscarding);

  // The file list is capped by the backend, so the remainder is reported as a count.
  $: hiddenDiscardCount = Math.max(0, uncommittedFilesCount - (discardPreview?.modifiedFiles.length ?? 0));

  onMount(async () => {
    armTimer = setTimeout(() => {
      armTimer = null;
      isDiscardArmed = true;
    }, DISCARD_ARM_DELAY_MS);
    try {
      discardPreview = await invoke<WorktreeDiffSummary>('get_worktree_diff_summary', { worktreePath });
    } catch (err: unknown) {
      discardPreviewError = toErrorMessage(err, 'Could not list the files that will be discarded');
    }
  });

  onDestroy(() => {
    if (armTimer !== null) clearTimeout(armTimer);
    onbusychange(false);
  });

  async function handleConfirm() {
    if (!isDiscardArmed || isDiscarding || disabled) return;
    isDiscarding = true;
    discardError = null;
    try {
      const updatedWt = await invoke<WorktreeInfo>('git_discard_worktree_changes', { worktreePath });
      ondiscarded(updatedWt);
    } catch (err: unknown) {
      const msg = toErrorMessage(err, 'Failed to discard changes');
      discardError = msg;
      notifications.error('Failed to discard changes', msg);
    } finally {
      isDiscarding = false;
    }
  }
</script>

<div class="rounded-md bg-neutral-950/90 border border-rose-800/60 p-2.5 flex flex-col gap-2 shadow-xl" role="alertdialog" aria-label="Confirm discarding changes">
  <div class="flex items-center justify-between gap-1">
    <p class="text-[11px] text-rose-200 font-medium">
      These changes will be permanently discarded:
    </p>
    <button
      type="button"
      on:click={oncancel}
      class="text-neutral-400 hover:text-neutral-200 p-0.5 rounded hover:bg-neutral-800 transition-colors"
      aria-label="Cancel"
      title="Close panel"
    >
      <X size={13} />
    </button>
  </div>

  {#if discardPreview}
    <div class="max-h-28 overflow-y-auto space-y-0.5 font-mono text-[11px] text-neutral-300">
      {#each discardPreview.modifiedFiles as file}
        <div class="truncate" title={file}>{file}</div>
      {/each}
      {#if hiddenDiscardCount > 0}
        <div class="text-neutral-400">...and {hiddenDiscardCount} more</div>
      {/if}
    </div>
  {:else if discardPreviewError}
    <p class="text-[11px] text-rose-300">{discardPreviewError}</p>
  {:else}
    <p class="text-[11px] text-neutral-400 flex items-center gap-1.5">
      <Loader2 size={11} class="animate-spin" /> Listing files...
    </p>
  {/if}

  {#if discardError}
    <div class="p-1.5 rounded bg-rose-950/80 border border-rose-700/60 text-rose-300 text-[11px] flex flex-col gap-0.5">
      <span class="font-semibold text-rose-200">Error:</span>
      <span class="break-words font-mono text-[10px]">{discardError}</span>
    </div>
  {/if}

  <div class="flex items-center justify-end gap-2 pt-1 border-t border-neutral-800/60">
    <button
      type="button"
      on:click={oncancel}
      class="px-2.5 py-1 rounded-md bg-neutral-800 hover:bg-neutral-700 text-neutral-200 text-[11px]"
    >
      Keep changes
    </button>
    <button
      type="button"
      disabled={!isDiscardArmed || isDiscarding || disabled}
      on:click={handleConfirm}
      class="px-2.5 py-1 rounded-md bg-rose-700 hover:bg-rose-600 border border-rose-500 text-white font-medium text-[11px] flex items-center gap-1.5 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
      title="Permanently discard all modifications (git reset + clean)"
    >
      {#if isDiscarding}
        <Loader2 size={12} class="animate-spin" />
      {:else}
        <AlertTriangle size={12} />
      {/if}
      <span>{isDiscarding ? 'Discarding...' : 'Discard permanently'}</span>
    </button>
  </div>
</div>
