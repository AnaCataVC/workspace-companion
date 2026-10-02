<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import {
    Zap,
    X,
    AlertTriangle,
    CheckCircle2,
    FileX2,
    Trash2,
    GitBranch,
    Flame,
    RotateCcw,
    FolderGit2,
    ShieldCheck,
    Layers
  } from 'lucide-svelte';
  import { closeOnEscape } from '../actions/closeOnEscape';
  import { autofocus } from '../actions/autofocus';
  import {
    activeQuickActionPreset,
    quickActionScope,
    quickActionPreview,
    isLoadingQuickActionPreview,
    quickActionPreviewError,
    isExecutingQuickAction,
    quickActionResult,
    quickActionError,
    fetchQuickActionPreview,
    runQuickAction,
    resetQuickActionsState
  } from '../stores/quickActions';
  import { scannedRepos } from '../stores/worktrees';
  import type { QuickActionPreset, QuickActionScope } from '../types';

  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    actionCompleted: void;
  }>();

  let selectedRepoPath: string = '';
  let isSingleRepoMode: boolean = false;

  // Nuclear wipe confirmation text
  const NUCLEAR_KEYWORD = 'RESET';
  let nuclearInputText: string = '';

  // Armed 2-step button with countdown for presets 1-3
  let isArmed: boolean = false;
  let countdown: number = 3;
  let countdownTimer: ReturnType<typeof setInterval> | null = null;

  $: isNuclearPreset = $activeQuickActionPreset === 'totalFreshStart';
  $: isNuclearConfirmed = nuclearInputText.trim().toUpperCase() === NUCLEAR_KEYWORD;

  // Scope binding
  $: {
    if (isSingleRepoMode) {
      if (!selectedRepoPath && $scannedRepos.length > 0) {
        selectedRepoPath = $scannedRepos[0].repoPath;
      }
      $quickActionScope = { singleRepo: { repoPath: selectedRepoPath } };
    } else {
      $quickActionScope = 'allWatchedRepos';
    }
  }

  // Reload preview whenever preset or scope changes
  $: if (isOpen && !isExecutingQuickAction && !$quickActionResult) {
    refreshPreview($activeQuickActionPreset, $quickActionScope);
  }

  function refreshPreview(preset: QuickActionPreset, scope: QuickActionScope) {
    disarm();
    nuclearInputText = '';
    fetchQuickActionPreview(preset, scope);
  }

  function handleSelectPreset(preset: QuickActionPreset) {
    if ($isExecutingQuickAction) return;
    activeQuickActionPreset.set(preset);
  }

  function handleScopeChange(singleRepo: boolean) {
    if ($isExecutingQuickAction) return;
    isSingleRepoMode = singleRepo;
  }

  function handleRepoChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedRepoPath = target.value;
    $quickActionScope = { singleRepo: { repoPath: selectedRepoPath } };
  }

  function armAction() {
    isArmed = true;
    countdown = 3;
    if (countdownTimer) clearInterval(countdownTimer);
    countdownTimer = setInterval(() => {
      countdown -= 1;
      if (countdown <= 0) {
        disarm();
      }
    }, 1000);
  }

  function disarm() {
    isArmed = false;
    countdown = 3;
    if (countdownTimer) {
      clearInterval(countdownTimer);
      countdownTimer = null;
    }
  }

  async function handleExecute() {
    if ($isExecutingQuickAction) return;

    if (isNuclearPreset && !isNuclearConfirmed) {
      return;
    }

    if (!isNuclearPreset && !isArmed) {
      armAction();
      return;
    }

    disarm();
    const result = await runQuickAction($activeQuickActionPreset, $quickActionScope);
    if (result && result.success) {
      dispatch('actionCompleted');
    }
  }

  function close() {
    if ($isExecutingQuickAction) return;
    disarm();
    resetQuickActionsState();
    dispatch('close');
  }

  onMount(() => {
    return () => {
      disarm();
    };
  });
</script>

<svelte:window use:closeOnEscape={{ enabled: () => isOpen, onClose: close }} />

{#if isOpen}
  <div
    use:autofocus
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-100 select-none"
  >
    <div
      class="w-full max-w-2xl bg-neutral-900 border border-neutral-800 rounded-xl p-5 shadow-2xl flex flex-col gap-4 max-h-[90vh] overflow-y-auto"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-neutral-800 pb-3">
        <div class="flex items-center gap-2 text-indigo-400">
          <Zap size={18} class="fill-indigo-400/20" />
          <h2 class="text-sm font-semibold tracking-wide text-neutral-100 uppercase">
            Quick Actions & Bulk Cleanup
          </h2>
        </div>
        {#if !$isExecutingQuickAction}
          <button
            on:click={close}
            class="text-neutral-400 hover:text-neutral-200 p-1 rounded transition-colors"
            aria-label="Close"
          >
            <X size={16} />
          </button>
        {/if}
      </div>

      {#if $quickActionResult}
        <!-- Completion Summary View -->
        <div class="flex flex-col gap-4 py-2">
          <div
            class="p-4 rounded-lg bg-neutral-950/80 border border-neutral-800 flex items-start gap-3"
          >
            {#if $quickActionResult.success}
              <CheckCircle2 size={20} class="text-emerald-400 shrink-0 mt-0.5" />
              <div class="flex flex-col gap-1">
                <h3 class="text-sm font-medium text-neutral-100">Quick action completed successfully</h3>
                <p class="text-xs text-neutral-400">
                  Worktrees removed: <strong class="text-neutral-200">{$quickActionResult.worktreesRemoved}</strong> |
                  Worktrees discarded: <strong class="text-neutral-200">{$quickActionResult.worktreesDiscarded}</strong> |
                  Branches deleted: <strong class="text-neutral-200">{$quickActionResult.branchesDeleted}</strong>
                </p>
              </div>
            {:else}
              <AlertTriangle size={20} class="text-rose-400 shrink-0 mt-0.5" />
              <div class="flex flex-col gap-1">
                <h3 class="text-sm font-medium text-rose-300">Action finished with errors</h3>
                <p class="text-xs text-neutral-400">
                  Some steps could not be completed. Review the logs below.
                </p>
              </div>
            {/if}
          </div>

          {#if $quickActionResult.warnings.length > 0}
            <div class="p-3 rounded-lg bg-amber-950/40 border border-amber-800/50 flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-amber-300 uppercase tracking-wider">
                Lock Warnings ({$quickActionResult.warnings.length})
              </span>
              <ul class="text-xs text-amber-200/90 list-disc list-inside space-y-1 font-mono text-[11px]">
                {#each $quickActionResult.warnings as warning}
                  <li class="truncate">{warning}</li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if $quickActionResult.errors.length > 0}
            <div class="p-3 rounded-lg bg-rose-950/40 border border-rose-800/50 flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-rose-300 uppercase tracking-wider">
                Errors ({$quickActionResult.errors.length})
              </span>
              <ul class="text-xs text-rose-200/90 list-disc list-inside space-y-1 font-mono text-[11px]">
                {#each $quickActionResult.errors as error}
                  <li class="truncate">{error}</li>
                {/each}
              </ul>
            </div>
          {/if}

          <div class="flex justify-end pt-2">
            <button
              on:click={close}
              class="px-4 py-1.5 text-xs font-medium bg-neutral-800 hover:bg-neutral-700 text-neutral-200 rounded-md transition-colors"
            >
              Done
            </button>
          </div>
        </div>
      {:else}
        <!-- Scope Selector -->
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between text-xs font-medium text-neutral-400">
            <span>Target Scope</span>
            {#if isSingleRepoMode && $scannedRepos.length > 0}
              <span class="text-[11px] text-neutral-500 font-mono">1 Repository</span>
            {:else if !isSingleRepoMode}
              <span class="text-[11px] text-neutral-500 font-mono">All Watched Repositories</span>
            {/if}
          </div>

          <div class="grid grid-cols-2 gap-2 bg-neutral-950/60 p-1 rounded-lg border border-neutral-800">
            <button
              type="button"
              on:click={() => handleScopeChange(false)}
              class="flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-md text-xs font-medium transition-all {
                !isSingleRepoMode
                  ? 'bg-indigo-600 text-white shadow-xs'
                  : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50'
              }"
            >
              <Layers size={13} />
              <span>All Watched Repos</span>
            </button>

            <button
              type="button"
              on:click={() => handleScopeChange(true)}
              class="flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-md text-xs font-medium transition-all {
                isSingleRepoMode
                  ? 'bg-indigo-600 text-white shadow-xs'
                  : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50'
              }"
            >
              <FolderGit2 size={13} />
              <span>Single Repo</span>
            </button>
          </div>

          {#if isSingleRepoMode}
            <div class="mt-1">
              {#if $scannedRepos.length === 0}
                <div class="p-2 text-xs text-neutral-400 bg-neutral-950 border border-neutral-800 rounded">
                  No repositories detected in watch folders.
                </div>
              {:else}
                <select
                  value={selectedRepoPath}
                  on:change={handleRepoChange}
                  class="w-full bg-neutral-950 border border-neutral-800 rounded-md px-3 py-1.5 text-xs text-neutral-200 focus:outline-none focus:border-indigo-500"
                >
                  {#each $scannedRepos as repo}
                    <option value={repo.repoPath}>
                      {repo.repoName} ({repo.repoPath})
                    </option>
                  {/each}
                </select>
              {/if}
            </div>
          {/if}
        </div>

        <!-- 4 Preset Cards -->
        <div class="grid grid-cols-2 gap-2.5">
          <!-- Preset 1: Discard Uncommitted -->
          <button
            type="button"
            on:click={() => handleSelectPreset('discardUncommitted')}
            class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 {
              $activeQuickActionPreset === 'discardUncommitted'
                ? 'bg-indigo-950/30 border-indigo-500/70 ring-1 ring-indigo-500/30'
                : 'bg-neutral-950/40 border-neutral-800 hover:border-neutral-700'
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5 text-neutral-200 font-medium text-xs">
                <FileX2 size={14} class="text-indigo-400" />
                <span>Discard Changes</span>
              </div>
              <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-400">
                Safe
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 leading-snug">
              Hard resets tracked files and cleans untracked items (<code class="text-[10px]">clean -ffd</code>). Preserves <code class="text-[10px]">.env</code> and gitignores.
            </p>
          </button>

          <!-- Preset 2: Nuke Worktrees -->
          <button
            type="button"
            on:click={() => handleSelectPreset('nukeWorktrees')}
            class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 {
              $activeQuickActionPreset === 'nukeWorktrees'
                ? 'bg-indigo-950/30 border-indigo-500/70 ring-1 ring-indigo-500/30'
                : 'bg-neutral-950/40 border-neutral-800 hover:border-neutral-700'
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5 text-neutral-200 font-medium text-xs">
                <Trash2 size={14} class="text-rose-400" />
                <span>Nuke Worktrees</span>
              </div>
              <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-400">
                Linked WTs
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 leading-snug">
              Force-removes all linked worktrees and prunes git registry. Main root worktree remains immune.
            </p>
          </button>

          <!-- Preset 3: Clean Branches -->
          <button
            type="button"
            on:click={() => handleSelectPreset('cleanBranches')}
            class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 {
              $activeQuickActionPreset === 'cleanBranches'
                ? 'bg-indigo-950/30 border-indigo-500/70 ring-1 ring-indigo-500/30'
                : 'bg-neutral-950/40 border-neutral-800 hover:border-neutral-700'
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5 text-neutral-200 font-medium text-xs">
                <GitBranch size={14} class="text-amber-400" />
                <span>Clean Branches</span>
              </div>
              <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-400">
                Local
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 leading-snug">
              Deletes secondary local branches not checked out in active worktrees. Default branch (<code class="text-[10px]">main</code>) is preserved.
            </p>
          </button>

          <!-- Preset 4: Total Fresh Start -->
          <button
            type="button"
            on:click={() => handleSelectPreset('totalFreshStart')}
            class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 {
              $activeQuickActionPreset === 'totalFreshStart'
                ? 'bg-rose-950/30 border-rose-500/70 ring-1 ring-rose-500/30'
                : 'bg-neutral-950/40 border-neutral-800 hover:border-neutral-700'
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5 text-rose-300 font-medium text-xs">
                <Flame size={14} class="text-rose-400" />
                <span>Total Fresh Start</span>
              </div>
              <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-rose-950/80 text-rose-300 border border-rose-800/50">
                Nuclear
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 leading-snug">
              Nukes all linked worktrees, switches main worktree to default branch, cleans all modifications, and deletes all secondary branches.
            </p>
          </button>
        </div>

        <!-- Live Impact Pre-flight Audit -->
        <div class="p-3 rounded-lg bg-neutral-950/70 border border-neutral-800 flex flex-col gap-2.5">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold text-neutral-300 uppercase tracking-wider flex items-center gap-1.5">
              <ShieldCheck size={14} class="text-indigo-400" />
              <span>Pre-flight Live Impact Audit</span>
            </span>
            {#if $isLoadingQuickActionPreview}
              <span class="text-[11px] text-neutral-500 animate-pulse">Calculating impact...</span>
            {/if}
          </div>

          {#if $quickActionPreviewError}
            <div class="p-2 text-xs text-rose-300 bg-rose-950/40 border border-rose-800/50 rounded">
              {$quickActionPreviewError}
            </div>
          {:else if $quickActionPreview}
            <div class="grid grid-cols-4 gap-2 text-center">
              <div class="p-2 rounded bg-neutral-900 border border-neutral-800">
                <span class="text-xs font-semibold text-neutral-200">{$quickActionPreview.reposCount}</span>
                <p class="text-[10px] text-neutral-500 uppercase mt-0.5">Repos</p>
              </div>

              <div class="p-2 rounded bg-neutral-900 border border-neutral-800">
                <span class="text-xs font-semibold text-rose-300">{$quickActionPreview.worktreesToRemove.length}</span>
                <p class="text-[10px] text-neutral-500 uppercase mt-0.5">WTs to Nuke</p>
              </div>

              <div class="p-2 rounded bg-neutral-900 border border-neutral-800">
                <span class="text-xs font-semibold text-amber-300">{$quickActionPreview.dirtyFilesCount}</span>
                <p class="text-[10px] text-neutral-500 uppercase mt-0.5">Dirty Files</p>
              </div>

              <div class="p-2 rounded bg-neutral-900 border border-neutral-800">
                <span class="text-xs font-semibold text-rose-300">{$quickActionPreview.branchesToDelete.length}</span>
                <p class="text-[10px] text-neutral-500 uppercase mt-0.5">Branches to Delete</p>
              </div>
            </div>

            <div class="flex items-center justify-between text-[11px] text-neutral-400 pt-1">
              <span>Protected Default Branches:</span>
              <span class="font-mono text-neutral-300">
                {$quickActionPreview.protectedDefaultBranches.join(', ') || 'main'}
              </span>
            </div>

            <!-- Prominent Unpushed Commits Warning Banner -->
            {#if $quickActionPreview.unpushedBranchesCount > 0 || $quickActionPreview.totalUnpushedCommits > 0}
              <div class="p-2.5 rounded bg-amber-950/50 border border-amber-700/60 flex items-start gap-2 text-amber-300">
                <AlertTriangle size={16} class="shrink-0 mt-0.5 text-amber-400" />
                <div class="text-xs leading-tight">
                  <strong>Warning:</strong> {$quickActionPreview.totalUnpushedCommits} unpushed commit(s) across {$quickActionPreview.unpushedBranchesCount} branch(es) will be permanently deleted and cannot be recovered!
                </div>
              </div>
            {/if}
          {/if}
        </div>

        {#if $quickActionError}
          <div class="p-2.5 rounded bg-rose-950/50 border border-rose-800/60 text-xs text-rose-300">
            {$quickActionError}
          </div>
        {/if}

        <!-- Confirmation Trigger Controls -->
        <div class="border-t border-neutral-800 pt-3 flex flex-col gap-2.5">
          {#if isNuclearPreset}
            <div class="flex flex-col gap-1.5">
              <label for="nuclear-confirm-input" class="text-xs text-rose-300">
                To confirm the Total Fresh Start wipe, type <strong class="font-mono">RESET</strong>:
              </label>
              <input
                id="nuclear-confirm-input"
                type="text"
                bind:value={nuclearInputText}
                placeholder="Type RESET to confirm"
                class="w-full bg-neutral-950 border border-neutral-800 rounded px-3 py-1.5 text-xs text-neutral-200 focus:outline-none focus:border-rose-500 font-mono tracking-wider"
              />
            </div>
          {/if}

          <div class="flex items-center justify-between">
            <button
              type="button"
              on:click={close}
              disabled={$isExecutingQuickAction}
              class="px-3 py-1.5 text-xs font-medium text-neutral-400 hover:text-neutral-200 transition-colors"
            >
              Cancel
            </button>

            <div class="flex items-center gap-2">
              {#if isNuclearPreset}
                <button
                  type="button"
                  on:click={handleExecute}
                  disabled={!isNuclearConfirmed || $isExecutingQuickAction}
                  class="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-md bg-rose-600 hover:bg-rose-500 text-white transition-all disabled:opacity-40 disabled:cursor-not-allowed shadow-sm"
                >
                  <Flame size={13} />
                  <span>{$isExecutingQuickAction ? 'Executing Nuclear Wipe...' : 'Execute Nuclear Wipe'}</span>
                </button>
              {:else}
                <button
                  type="button"
                  on:click={handleExecute}
                  disabled={$isExecutingQuickAction}
                  class="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-md transition-all shadow-sm {
                    isArmed
                      ? 'bg-rose-600 hover:bg-rose-500 text-white animate-pulse'
                      : 'bg-indigo-600 hover:bg-indigo-500 text-white'
                  }"
                >
                  {#if $isExecutingQuickAction}
                    <RotateCcw size={13} class="animate-spin" />
                    <span>Executing...</span>
                  {:else if isArmed}
                    <AlertTriangle size={13} />
                    <span>Click again to confirm ({countdown}s)</span>
                  {:else}
                    <Zap size={13} />
                    <span>Execute Cleanup</span>
                  {/if}
                </button>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}
