<script lang="ts">
  import type { BatchDeleteTarget, BatchDeleteSummary } from '../types';
  import { Trash2, X, ShieldAlert, CheckCircle2, Flame, GitBranch } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';
  import { closeOnEscape } from '../actions/closeOnEscape';
  import { forceBatchWorktreeDelete } from '../stores/forceDeleteIntent';
  import { worktreeProtectionReason } from '../utils/protectionReason';

  export let isOpen: boolean = false;
  export let targets: BatchDeleteTarget[] = [];
  export let isDeleting: boolean = false;
  export let summary: BatchDeleteSummary | null = null;
  export let errorMessage: string | null = null;

  const dispatch = createEventDispatcher<{
    close: void;
    confirmDelete: { targets: BatchDeleteTarget[]; force: boolean };
  }>();

  /**
   * Typed against a fixed literal rather than each worktree's branch name: a batch confirms N
   * differently-named targets at once, so one keyword is both simpler and easier to recognise as
   * "this is the irreversible one".
   */
  const FORCE_KEYWORD = 'FORCE';

  let forceConfirmText: string = '';
  let lastTargetsKey: string = '';

  $: hasDirty = targets.some((t) => Boolean(t.isDirty));
  $: dirtyTargets = targets.filter((t) => Boolean(t.isDirty));
  $: dirtyCount = dirtyTargets.length;
  // Mirrors the backend's skip-dirty-unless-forced rule (see worktree_cleaner.rs) so the button
  // label reflects what will actually happen before the click, not just an intent to delete
  $: willDeleteCount = $forceBatchWorktreeDelete ? targets.length : targets.length - dirtyCount;
  $: skippedCount = targets.length - willDeleteCount;

  // Resets on a change of selection, not on close: reopening the dialog over the same selection
  // keeps the intent the user already expressed.
  $: targetsKey = targets.map((t) => t.worktreePath).sort().join('|');
  $: if (targetsKey !== lastTargetsKey) {
    lastTargetsKey = targetsKey;
    forceBatchWorktreeDelete.set(false);
    forceConfirmText = '';
  }

  $: if (!$forceBatchWorktreeDelete) {
    forceConfirmText = '';
  }

  $: isForceConfirmed = forceConfirmText.trim().toUpperCase() === FORCE_KEYWORD;

  function close() {
    if (isDeleting) return;
    dispatch('close');
  }

  function handleConfirm() {
    if (targets.length === 0) return;
    const force = $forceBatchWorktreeDelete;
    const finalTargets = targets.map((t) => ({
      ...t,
      force
    }));
    dispatch('confirmDelete', {
      targets: finalTargets,
      force
    });
  }

  function getShortBranch(fullBranch: string | null | undefined): string {
    if (!fullBranch) return '(detached HEAD)';
    return fullBranch.replace('refs/heads/', '');
  }
</script>

<svelte:window use:closeOnEscape={{ enabled: () => isOpen, onClose: close }} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-100 select-none">
    <div class="w-full max-w-md bg-neutral-900 border border-neutral-800 rounded-xl p-4 shadow-2xl flex flex-col gap-3 max-h-[85vh]">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-neutral-800 pb-2">
        <div class="flex items-center gap-2 text-rose-400">
          <Trash2 size={16} />
          <h2 class="text-xs font-semibold uppercase tracking-wider text-neutral-200">
            Batch Remove Worktrees
          </h2>
        </div>
        {#if !isDeleting}
          <button on:click={close} class="text-neutral-500 hover:text-neutral-300 p-1 rounded transition-colors">
            <X size={14} />
          </button>
        {/if}
      </div>

      <!-- Content Phases -->
      {#if summary}
        <!-- Phase 3: Completion & Summary Report -->
        <div class="flex flex-col gap-3 py-2">
          <div class="p-3 rounded-lg bg-neutral-950/80 border border-neutral-800 flex items-center gap-2.5">
            <CheckCircle2 size={18} class="text-emerald-400 flex-shrink-0" />
            <div>
              <p class="text-xs font-medium text-neutral-200">
                Batch operation completed
              </p>
              <p class="text-[11px] text-neutral-400">
                {summary.deletedCount} of {summary.totalRequested} worktrees removed.
                {#if summary.skippedCount > 0}
                  <span class="text-amber-400">({summary.skippedCount} skipped)</span>
                {/if}
              </p>
            </div>
          </div>

          {#if summary.errors && summary.errors.length > 0}
            <div class="space-y-1.5 max-h-48 overflow-y-auto pr-1">
              <span class="text-[10px] uppercase font-semibold text-rose-400">Reported issues:</span>
              {#each summary.errors as err}
                <div
                  class="p-2 rounded border text-[10px] font-mono
                    {err.kind === 'skipped'
                      ? 'bg-amber-950/50 border-amber-900/40 text-amber-200'
                      : 'bg-rose-950/50 border-rose-900/40 text-rose-200'}"
                >
                  <p class="font-semibold break-all">
                    <span class="uppercase font-sans tracking-wider mr-1">{err.kind === 'skipped' ? 'Skipped' : 'Failed'}</span>
                    {err.worktreePath}
                  </p>
                  <p class={err.kind === 'skipped' ? 'text-amber-300/80' : 'text-rose-300/80'}>{err.error}</p>
                </div>
              {/each}
            </div>
          {/if}

          <div class="flex justify-end pt-2">
            <button
              type="button"
              on:click={close}
              class="px-4 py-1.5 rounded-lg text-xs font-medium bg-neutral-800 hover:bg-neutral-700 text-neutral-100 transition-colors"
            >
              Done
            </button>
          </div>
        </div>
      {:else}
        <!-- Phase 1 & 2: Review / Confirmation or In-Progress -->
        <div class="flex flex-col gap-3 overflow-hidden">
          <p class="text-xs text-neutral-300">
            You are about to remove <span class="font-bold text-white">{targets.length}</span> worktree(s):
          </p>

          <!-- List of selected worktrees -->
          <div class="max-h-52 overflow-y-auto space-y-1 bg-neutral-950/70 p-2 rounded-lg border border-neutral-850 no-scrollbar">
            {#each targets as wt (wt.worktreePath)}
              <div class="flex items-center justify-between p-1.5 rounded bg-neutral-900/80 border border-neutral-800 text-[11px]">
                <div class="flex items-center gap-1.5 min-w-0 flex-1 mr-2">
                  <GitBranch size={11} class="text-indigo-400 flex-shrink-0" />
                  <span class="font-mono text-neutral-200 truncate" title={wt.branch || ''}>
                    {getShortBranch(wt.branch)}
                  </span>
                  {#if wt.repoName}
                    <span class="text-neutral-500 text-[10px] truncate max-w-[90px]" title={wt.repoPath}>
                      ({wt.repoName})
                    </span>
                  {/if}
                </div>

                <div class="flex items-center gap-1 flex-shrink-0">
                  {#if wt.isDirty}
                    <span
                      class="px-1.5 py-0.2 rounded bg-rose-950 text-rose-300 border border-rose-800 text-[9px] flex items-center gap-0.5"
                      title={worktreeProtectionReason(wt)}
                    >
                      <Flame size={9} class="text-rose-400" />
                      dirty
                    </span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          <!-- Dirty Warning Alert -->
          {#if hasDirty}
            <div class="p-2.5 rounded-lg bg-rose-950/60 border border-rose-800/60 flex items-start gap-2 text-xs">
              <ShieldAlert size={15} class="text-rose-400 flex-shrink-0 mt-0.5" />
              <div>
                <p class="font-semibold text-rose-200 text-[11px]">
                  {dirtyCount} worktree(s) have uncommitted local changes
                </p>
                <p class="text-rose-300/80 text-[10px] mt-0.5">
                  Removing dirty worktrees without forcing will skip them to protect your work.
                </p>
              </div>
            </div>

            <label class="flex items-center gap-2 text-xs text-neutral-300 cursor-pointer select-none px-1">
              <input
                type="checkbox"
                bind:checked={$forceBatchWorktreeDelete}
                class="w-3.5 h-3.5 rounded border-neutral-700 bg-neutral-950 text-rose-500 focus:ring-rose-500/30 cursor-pointer"
              />
              <span class="text-[11px] text-rose-300">Force delete dirty worktrees permanently</span>
            </label>

            {#if $forceBatchWorktreeDelete}
              <div class="p-2.5 rounded-lg bg-rose-950/70 border border-rose-700/70 flex flex-col gap-2">
                <p class="text-[11px] font-semibold text-rose-100">This cannot be undone</p>
                <div class="max-h-24 overflow-y-auto space-y-0.5 text-[10px] text-rose-200/90 font-mono no-scrollbar">
                  {#each dirtyTargets as wt (wt.worktreePath)}
                    <p class="break-all">{getShortBranch(wt.branch)} — {worktreeProtectionReason(wt)}</p>
                  {/each}
                </div>
                <label for="batch-delete-force-confirm" class="text-[10px] text-rose-200">
                  Type <span class="font-mono font-bold text-rose-100">{FORCE_KEYWORD}</span> to confirm
                </label>
                <input
                  id="batch-delete-force-confirm"
                  type="text"
                  bind:value={forceConfirmText}
                  on:keydown={(e) => {
                    if (e.key === 'Enter' && isForceConfirmed && !isDeleting && willDeleteCount > 0) {
                      e.preventDefault();
                      handleConfirm();
                    }
                  }}
                  autocomplete="off"
                  spellcheck="false"
                  placeholder={FORCE_KEYWORD}
                  class="w-full bg-neutral-950 border border-rose-800/70 rounded px-2 py-1 text-[11px] font-mono text-rose-100 placeholder-rose-900 focus:outline-hidden focus:border-rose-500 focus:ring-1 focus:ring-rose-500/40"
                />
              </div>
            {/if}
          {/if}

          {#if errorMessage}
            <div class="p-2 rounded bg-rose-950/70 border border-rose-800 text-rose-200 text-[11px]">
              {errorMessage}
            </div>
          {/if}

          <!-- Footer Actions -->
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-neutral-800">
            <button
              type="button"
              on:click={close}
              disabled={isDeleting}
              class="px-3 py-1.5 rounded-lg text-xs font-medium text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 transition-colors disabled:opacity-50"
            >
              Cancel
            </button>

            <button
              type="button"
              on:click={handleConfirm}
              disabled={isDeleting || willDeleteCount === 0 || ($forceBatchWorktreeDelete && !isForceConfirmed)}
              class="px-3.5 py-1.5 rounded-lg text-xs font-medium bg-rose-600 hover:bg-rose-500 disabled:opacity-40 disabled:hover:bg-rose-600 text-white transition-all flex items-center gap-1.5 shadow-lg shadow-rose-600/25"
            >
              {#if isDeleting}
                <div class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                <span>Removing {targets.length} worktrees...</span>
              {:else}
                <Trash2 size={13} />
                {#if skippedCount > 0}
                  <span>Delete {willDeleteCount} of {targets.length} (Skipping {skippedCount} dirty)</span>
                {:else}
                  <span>Delete {targets.length} Worktrees</span>
                {/if}
              {/if}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}
