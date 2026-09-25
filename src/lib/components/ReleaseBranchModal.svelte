<script lang="ts">
  import type { BranchStatusEntry, BranchBatchDeleteSummary, WorktreeInfo } from '../types';
  import { Unlock, X, AlertTriangle, ShieldAlert, Trash2, GitBranch, Loader2 } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { closeOnEscape } from '../actions/closeOnEscape';
  import { autofocus } from '../actions/autofocus';
  import { toErrorMessage } from '../utils/errors';
  import { scannedRepos } from '../stores/worktrees';
  import DiscardChangesPanel from './DiscardChangesPanel.svelte';

  export let isOpen: boolean = false;
  export let branch: BranchStatusEntry | null = null;

  const dispatch = createEventDispatcher<{
    close: void;
    worktreeDiscarded: { worktree: WorktreeInfo; repoPath: string };
    released: { repoPath: string; branchName: string; deletedWorktreePath?: string };
  }>();

  /** Same fixed literal as the batch dialogs: one keyword everywhere reads as "the irreversible one". */
  const FORCE_KEYWORD = 'FORCE';

  let isBusy = false;
  let errorMessage: string | null = null;
  let awaitingForceConfirm = false;
  let forceConfirmText = '';
  let deletedWorktreePathForPending: string | null = null;
  let lastBranchKey = '';

  // The worktree is looked up live from the scan store (not copied into local state) so that a
  // discard applied through `DiscardChangesPanel` — which only updates the store via the
  // `worktreeDiscarded` event — is reflected here automatically.
  $: worktree = branch?.checkedOutWorktreePath
    ? $scannedRepos
        .find((r) => r.repoPath === branch!.repoPath)
        ?.worktrees.find((w) => w.path === branch!.checkedOutWorktreePath) ?? null
    : null;
  $: isDirty = worktree?.isDirty ?? false;
  $: isForceConfirmed = forceConfirmText.trim().toUpperCase() === FORCE_KEYWORD;

  // Resets on a change of target branch, not on close: reopening over the same branch keeps intent.
  $: branchKey = branch ? `${branch.repoPath}::${branch.name}` : '';
  $: if (branchKey !== lastBranchKey) {
    lastBranchKey = branchKey;
    errorMessage = null;
    awaitingForceConfirm = false;
    forceConfirmText = '';
    deletedWorktreePathForPending = null;
  }

  function close() {
    if (isBusy) return;
    dispatch('close');
  }

  function handleDiscarded(updatedWt: WorktreeInfo) {
    if (!branch) return;
    dispatch('worktreeDiscarded', { worktree: updatedWt, repoPath: branch.repoPath });
  }

  /** Deletes the branch itself; a `skipped` result the first time means it needs an explicit force retry. */
  async function deleteBranch(force: boolean) {
    if (!branch) return;
    try {
      const summary = await invoke<BranchBatchDeleteSummary>('remove_branches_batch', {
        targets: [{ repoPath: branch.repoPath, branchName: branch.name, force }]
      });

      if (summary.deletedCount === 1) {
        dispatch('released', {
          repoPath: branch.repoPath,
          branchName: branch.name,
          deletedWorktreePath: deletedWorktreePathForPending ?? undefined
        });
        return;
      }

      const issue = summary.errors[0];
      if (!force && issue?.kind === 'skipped') {
        awaitingForceConfirm = true;
        return;
      }
      errorMessage = issue?.error ?? 'The branch could not be deleted.';
    } catch (err: unknown) {
      errorMessage = toErrorMessage(err, 'Failed to delete the branch');
    } finally {
      isBusy = false;
    }
  }

  async function handleDetachAndDelete() {
    if (!branch?.checkedOutWorktreePath || isBusy || isDirty) return;
    isBusy = true;
    errorMessage = null;
    try {
      await invoke('detach_worktree_head', { worktreePath: branch.checkedOutWorktreePath });
      await deleteBranch(false);
    } catch (err: unknown) {
      errorMessage = toErrorMessage(err, 'Failed to detach the worktree');
      isBusy = false;
    }
  }

  async function handleDeleteWorktreeAndBranch() {
    if (!branch?.checkedOutWorktreePath || isBusy || isDirty) return;
    isBusy = true;
    errorMessage = null;
    try {
      await invoke('remove_worktree', {
        repoPath: branch.repoPath,
        worktreePath: branch.checkedOutWorktreePath,
        force: Boolean(worktree?.locked)
      });
      deletedWorktreePathForPending = branch.checkedOutWorktreePath;
      await deleteBranch(false);
    } catch (err: unknown) {
      errorMessage = toErrorMessage(err, 'Failed to remove the worktree');
      isBusy = false;
    }
  }

  function confirmForceDelete() {
    if (!isForceConfirmed || isBusy) return;
    isBusy = true;
    errorMessage = null;
    deleteBranch(true);
  }
</script>

<svelte:window use:closeOnEscape={{ enabled: () => isOpen, onClose: close }} />

{#if isOpen && branch}
  <div
    use:autofocus
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-100"
    role="dialog"
    aria-modal="true"
  >
    <div class="w-full max-w-md bg-neutral-900 border border-neutral-800 rounded-xl p-4 shadow-2xl flex flex-col gap-3 max-h-[85vh]">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-neutral-800 pb-2">
        <div class="flex items-center gap-2 text-amber-400">
          <Unlock size={16} />
          <h2 class="text-xs font-semibold uppercase tracking-wider text-neutral-200">Release Branch</h2>
        </div>
        <button
          type="button"
          on:click={close}
          disabled={isBusy}
          aria-label="Close"
          class="text-neutral-400 hover:text-neutral-300 disabled:opacity-30 p-1.5 rounded transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      <div class="flex flex-col gap-0.5">
        <div class="flex items-center gap-1.5 text-xs">
          <GitBranch size={12} class="text-indigo-400 flex-shrink-0" />
          <span class="font-mono font-medium text-neutral-200 truncate" title={branch.name}>{branch.name}</span>
        </div>
        {#if branch.checkedOutWorktreePath}
          <span class="text-[11px] text-neutral-400 truncate" title={branch.checkedOutWorktreePath}>
            {branch.checkedOutWorktreePath}
          </span>
        {/if}
      </div>

      {#if isDirty && worktree}
        <div class="p-2.5 rounded-lg bg-rose-950/60 border border-rose-800/60 flex flex-col gap-2 text-xs">
          <div class="flex items-start gap-2">
            <AlertTriangle size={15} class="text-rose-400 flex-shrink-0 mt-0.5" />
            <p class="text-rose-200">
              This worktree has uncommitted changes. Discard them before the branch can be released.
            </p>
          </div>
          <DiscardChangesPanel
            worktreePath={worktree.path}
            uncommittedFilesCount={worktree.uncommittedFilesCount ?? 0}
            disabled={isBusy}
            ondiscarded={handleDiscarded}
            oncancel={close}
          />
        </div>
      {:else if awaitingForceConfirm}
        <div class="p-2.5 rounded-lg bg-rose-950/60 border border-rose-800/60 flex items-start gap-2 text-xs">
          <ShieldAlert size={15} class="text-rose-400 flex-shrink-0 mt-0.5" />
          <div>
            <p class="font-semibold text-rose-200 text-[11px]">This branch is not merged into the default branch</p>
            <p class="text-rose-300/80 text-[11px] mt-0.5">
              Deleting it now permanently discards commits that were never merged.
            </p>
          </div>
        </div>

        <div class="p-2.5 rounded-lg bg-rose-950/70 border border-rose-700/70 flex flex-col gap-2">
          <p class="text-[11px] font-semibold text-rose-100">This cannot be undone</p>
          <label for="release-branch-force-confirm" class="text-[11px] text-rose-200">
            Type <span class="font-mono font-bold text-rose-100">{FORCE_KEYWORD}</span> to confirm
          </label>
          <input
            id="release-branch-force-confirm"
            type="text"
            bind:value={forceConfirmText}
            on:keydown={(e) => {
              if (e.key === 'Enter' && isForceConfirmed && !isBusy) {
                e.preventDefault();
                confirmForceDelete();
              }
            }}
            autocomplete="off"
            spellcheck="false"
            placeholder={FORCE_KEYWORD}
            class="w-full bg-neutral-950 border border-rose-800/70 rounded px-2 py-1 text-[11px] font-mono text-rose-100 placeholder-rose-900 focus:outline-hidden focus:border-rose-500 focus:ring-1 focus:ring-rose-500/40"
          />
        </div>
      {:else}
        <p class="text-xs text-neutral-300">
          This branch is checked out in a worktree, which keeps it locked. Choose how to free it:
        </p>

        <div class="flex flex-col gap-2">
          <button
            type="button"
            on:click={handleDetachAndDelete}
            disabled={isBusy}
            class="w-full text-left px-3 py-2 rounded-lg bg-neutral-850 hover:bg-neutral-800 border border-neutral-750 hover:border-neutral-700 disabled:opacity-50 transition-colors"
          >
            <p class="text-xs font-medium text-neutral-100">Detach &amp; delete branch</p>
            <p class="text-[11px] text-neutral-400">Keeps the worktree folder, detaches its HEAD, then deletes the branch.</p>
          </button>

          <button
            type="button"
            on:click={handleDeleteWorktreeAndBranch}
            disabled={isBusy}
            class="w-full text-left px-3 py-2 rounded-lg bg-neutral-850 hover:bg-rose-950/60 border border-neutral-750 hover:border-rose-800/60 disabled:opacity-50 transition-colors"
          >
            <p class="text-xs font-medium text-neutral-100">Delete worktree &amp; branch</p>
            <p class="text-[11px] text-neutral-400">Removes the worktree folder entirely, then deletes the branch.</p>
          </button>
        </div>
      {/if}

      {#if errorMessage}
        <div class="p-2 rounded bg-rose-950/70 border border-rose-800 text-rose-200 text-[11px]">
          {errorMessage}
        </div>
      {/if}

      <!-- Footer Actions -->
      <div class="flex items-center justify-end gap-2 pt-1 border-t border-neutral-800">
        <button
          type="button"
          on:click={close}
          disabled={isBusy}
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 disabled:opacity-50 transition-colors"
        >
          Cancel
        </button>

        {#if awaitingForceConfirm}
          <button
            type="button"
            on:click={confirmForceDelete}
            disabled={isBusy || !isForceConfirmed}
            class="px-3.5 py-1.5 rounded-lg text-xs font-medium bg-rose-600 hover:bg-rose-500 disabled:opacity-40 disabled:hover:bg-rose-600 text-white transition-all flex items-center gap-1.5 shadow-lg shadow-rose-600/25"
          >
            {#if isBusy}
              <Loader2 size={13} class="animate-spin" />
              <span>Deleting...</span>
            {:else}
              <Trash2 size={13} />
              <span>Force Delete Branch</span>
            {/if}
          </button>
        {:else if isBusy}
          <span class="px-3.5 py-1.5 rounded-lg text-xs font-medium text-neutral-400 flex items-center gap-1.5">
            <Loader2 size={13} class="animate-spin" />
            <span>Working...</span>
          </span>
        {/if}
      </div>
    </div>
  </div>
{/if}
