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
    FolderSync,
    ShieldCheck,
    Layers,
    Filter
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
  export let filteredRepoPaths: string[] = [];

  const dispatch = createEventDispatcher<{
    close: void;
    actionCompleted: void;
  }>();

  type ScopeMode = 'all' | 'filtered' | 'single';

  let scopeMode: ScopeMode = 'all';
  let selectedRepoPath: string = '';
  let filteredPathsSnapshot: string[] = [];

  const NUCLEAR_KEYWORD = 'RESET';
  let nuclearInputText: string = '';

  let isArmed: boolean = false;
  let countdown: number = 3;
  let countdownTimer: ReturnType<typeof setInterval> | null = null;

  $: isSyncPreset = $activeQuickActionPreset === 'syncDefaultBranch';
  $: isNuclearPreset = $activeQuickActionPreset === 'totalFreshStart';
  $: isNuclearConfirmed = nuclearInputText.trim().toUpperCase() === NUCLEAR_KEYWORD;

  let wasOpen = false;
  $: if (isOpen && !wasOpen) {
    filteredPathsSnapshot = [...filteredRepoPaths];
    const scope = $quickActionScope;
    if (typeof scope === 'object' && 'singleRepo' in scope) {
      scopeMode = 'single';
      selectedRepoPath = scope.singleRepo.repoPath;
    } else if (typeof scope === 'object' && 'repoList' in scope) {
      scopeMode = 'filtered';
      filteredPathsSnapshot = [...scope.repoList.repoPaths];
    } else {
      scopeMode = 'all';
      selectedRepoPath = $scannedRepos[0]?.repoPath || '';
    }
  }
  $: wasOpen = isOpen;

  $: if (isOpen && !$isExecutingQuickAction && !$quickActionResult) {
    refreshPreview($activeQuickActionPreset, $quickActionScope);
  }

  function refreshPreview(preset: QuickActionPreset, scope: QuickActionScope) {
    disarm();
    nuclearInputText = '';
    expandedAuditKey = null;
    fetchQuickActionPreview(preset, scope);
  }

  function handleSelectPreset(preset: QuickActionPreset) {
    if ($isExecutingQuickAction) return;
    activeQuickActionPreset.set(preset);
  }

  function handleScopeModeChange(mode: ScopeMode) {
    if ($isExecutingQuickAction) return;
    scopeMode = mode;
    if (mode === 'single') {
      selectedRepoPath = selectedRepoPath || $scannedRepos[0]?.repoPath || '';
      $quickActionScope = { singleRepo: { repoPath: selectedRepoPath } };
    } else if (mode === 'filtered') {
      $quickActionScope = { repoList: { repoPaths: filteredPathsSnapshot } };
    } else {
      $quickActionScope = 'allWatchedRepos';
    }
  }

  function handleRepoChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    selectedRepoPath = target.value;
    $quickActionScope = { singleRepo: { repoPath: selectedRepoPath } };
  }

  let expandedAuditKey: string | null = null;

  $: auditCards = $quickActionPreview
    ? isSyncPreset
      ? [
          {
            key: 'repos',
            label: 'Repos',
            count: $quickActionPreview.reposCount,
            items: [] as string[],
            color: 'text-neutral-200'
          },
          {
            key: 'switchMain',
            label: 'Switch → Main',
            count: ($quickActionPreview.reposToSwitchToDefault ?? []).length,
            items: $quickActionPreview.reposToSwitchToDefault ?? [],
            color: 'text-emerald-300'
          },
          {
            key: 'readyPull',
            label: 'Ready to Pull',
            count: ($quickActionPreview.reposReadyToPull ?? []).length,
            items: $quickActionPreview.reposReadyToPull ?? [],
            color: 'text-indigo-300'
          },
          {
            key: 'skippedSync',
            label: 'Safe Skips',
            count: ($quickActionPreview.reposSkippedDirtyOrLocked ?? []).length,
            items: $quickActionPreview.reposSkippedDirtyOrLocked ?? [],
            color: 'text-amber-300'
          }
        ]
      : [
          {
            key: 'repos',
            label: 'Repos',
            count: $quickActionPreview.reposCount,
            items: [] as string[],
            color: 'text-neutral-200'
          },
          {
            key: 'nuke',
            label: 'WTs to Nuke',
            count: $quickActionPreview.worktreesToRemove.length,
            items: $quickActionPreview.worktreesToRemove,
            color: 'text-rose-300'
          },
          {
            key: 'dirty',
            label: 'Dirty Files',
            count: $quickActionPreview.dirtyFilesCount,
            items: $quickActionPreview.worktreesToDiscard,
            color: 'text-amber-300'
          },
          {
            key: 'branches',
            label: 'Branches',
            count: $quickActionPreview.branchesToDelete.length,
            items: $quickActionPreview.branchesToDelete,
            color: 'text-rose-300'
          }
        ]
    : [];
  $: expandedCard = auditCards.find((c) => c.key === expandedAuditKey) ?? null;

  $: hasDestructiveImpact = (() => {
    const p = $quickActionPreview;
    if (!p) return false;
    if ($activeQuickActionPreset === 'discardUncommitted') {
      return p.dirtyFilesCount > 0 || p.worktreesToDiscard.length > 0;
    }
    if ($activeQuickActionPreset === 'nukeWorktrees') {
      return p.worktreesToRemove.length > 0;
    }
    if ($activeQuickActionPreset === 'cleanBranches') {
      return p.branchesToDelete.length > 0;
    }
    return p.reposCount > 0;
  })();

  $: canExecuteSync = Boolean($quickActionPreview && $quickActionPreview.reposCount > 0);

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

    if (isSyncPreset) {
      if (!canExecuteSync) return;
      const result = await runQuickAction('syncDefaultBranch', $quickActionScope);
      if (result) {
        dispatch('actionCompleted');
      }
      return;
    }

    if (isNuclearPreset && !isNuclearConfirmed) {
      return;
    }

    if (!isNuclearPreset && !hasDestructiveImpact) {
      return;
    }

    if (!isNuclearPreset && !isArmed) {
      armAction();
      return;
    }

    disarm();
    const result = await runQuickAction($activeQuickActionPreset, $quickActionScope);
    if (result) {
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
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm select-none"
  >
    <div
      class="w-full max-w-2xl bg-neutral-900 border border-neutral-800 rounded-xl p-5 shadow-2xl flex flex-col gap-4 max-h-[90vh] overflow-y-auto"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-neutral-800 pb-3">
        <div class="flex items-center gap-2 text-indigo-400">
          <Zap size={18} class="fill-indigo-400/20" />
          <h2 class="text-sm font-semibold tracking-wide text-neutral-100 uppercase">
            Quick Actions & Workspace Maintenance
          </h2>
        </div>
        {#if !$isExecutingQuickAction}
          <button
            type="button"
            on:click={close}
            class="text-neutral-400 hover:text-neutral-200 p-1 rounded transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
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
                {#if $quickActionResult.preset === 'syncDefaultBranch'}
                  <h3 class="text-sm font-medium text-neutral-100">
                    {$quickActionResult.warnings.length > 0
                      ? `Repository sync completed with ${$quickActionResult.warnings.length} safe skip(s)`
                      : 'All targeted repositories synced to default branch'}
                  </h3>
                  <p class="text-xs text-neutral-400">
                    Fetched: <strong class="text-neutral-200">{$quickActionResult.reposFetched ?? 0}</strong> |
                    Switched to main: <strong class="text-emerald-300">{$quickActionResult.reposSwitchedToDefault ?? 0}</strong> |
                    Fast-forwarded: <strong class="text-indigo-300">{$quickActionResult.reposPulled ?? 0}</strong> |
                    Up to date: <strong class="text-neutral-200">{$quickActionResult.reposUpToDate ?? 0}</strong>
                    {#if ($quickActionResult.reposSkipped ?? 0) > 0}
                      | Skipped: <strong class="text-amber-300">{$quickActionResult.reposSkipped}</strong>
                    {/if}
                  </p>
                {:else}
                  <h3 class="text-sm font-medium text-neutral-100">Quick action completed successfully</h3>
                  <p class="text-xs text-neutral-400">
                    Worktrees removed: <strong class="text-neutral-200">{$quickActionResult.worktreesRemoved}</strong> |
                    Worktrees discarded: <strong class="text-neutral-200">{$quickActionResult.worktreesDiscarded}</strong> |
                    Branches deleted: <strong class="text-neutral-200">{$quickActionResult.branchesDeleted}</strong>
                  </p>
                {/if}
              </div>
            {:else}
              <AlertTriangle size={20} class="text-rose-400 shrink-0 mt-0.5" />
              <div class="flex flex-col gap-1">
                <h3 class="text-sm font-medium text-rose-300">Action finished with errors</h3>
                <p class="text-xs text-neutral-400">
                  Some repositories encountered errors. Review the details below.
                </p>
              </div>
            {/if}
          </div>

          {#if $quickActionResult.warnings.length > 0}
            <div class="p-3 rounded-lg bg-amber-950/40 border border-amber-800/50 flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-amber-300 uppercase tracking-wider">
                {$quickActionResult.preset === 'syncDefaultBranch' ? 'Safe Skips & Warnings' : 'Lock Warnings'} ({$quickActionResult.warnings.length})
              </span>
              <ul class="max-h-36 overflow-y-auto text-xs text-amber-200/90 list-disc list-inside space-y-1 font-mono text-[11px]">
                {#each $quickActionResult.warnings as warning}
                  <li class="break-words">{warning}</li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if $quickActionResult.errors.length > 0}
            <div class="p-3 rounded-lg bg-rose-950/40 border border-rose-800/50 flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-rose-300 uppercase tracking-wider">
                Errors ({$quickActionResult.errors.length})
              </span>
              <ul class="max-h-36 overflow-y-auto text-xs text-rose-200/90 list-disc list-inside space-y-1 font-mono text-[11px]">
                {#each $quickActionResult.errors as error}
                  <li class="break-words">{error}</li>
                {/each}
              </ul>
            </div>
          {/if}

          <div class="flex justify-end pt-2">
            <button
              type="button"
              on:click={close}
              class="px-4 py-1.5 text-xs font-medium bg-neutral-800 hover:bg-neutral-700 text-neutral-200 rounded-md transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
            >
              Done
            </button>
          </div>
        </div>
      {:else}
        <!-- Scope Selector (Elevated surface active pills — never solid primary CTA) -->
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between text-xs font-medium text-neutral-400">
            <span>Target Scope</span>
            {#if scopeMode === 'single'}
              <span class="text-[11px] text-neutral-400 font-mono">1 Repository</span>
            {:else if scopeMode === 'filtered'}
              <span class="text-[11px] text-indigo-300 font-mono">{filteredPathsSnapshot.length} Filtered Repositories</span>
            {:else}
              <span class="text-[11px] text-neutral-400 font-mono">All Watched Repositories ({$scannedRepos.length})</span>
            {/if}
          </div>

          <div
            class="grid {filteredPathsSnapshot.length > 0 ? 'grid-cols-3' : 'grid-cols-2'} gap-1.5 bg-neutral-950/60 p-1 rounded-lg border border-neutral-800"
          >
            <button
              type="button"
              on:click={() => handleScopeModeChange('all')}
              aria-pressed={scopeMode === 'all'}
              class="flex items-center justify-center gap-1.5 py-1.5 px-2.5 rounded-md text-xs font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
                scopeMode === 'all'
                  ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-sm'
                  : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-850 border border-transparent'
              }"
            >
              <Layers size={13} />
              <span>All Watched ({$scannedRepos.length})</span>
            </button>

            {#if filteredPathsSnapshot.length > 0}
              <button
                type="button"
                on:click={() => handleScopeModeChange('filtered')}
                aria-pressed={scopeMode === 'filtered'}
                class="flex items-center justify-center gap-1.5 py-1.5 px-2.5 rounded-md text-xs font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
                  scopeMode === 'filtered'
                    ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/70 shadow-sm'
                    : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-850 border border-transparent'
                }"
              >
                <Filter size={13} class={scopeMode === 'filtered' ? 'text-indigo-400' : ''} />
                <span>Filtered ({filteredPathsSnapshot.length})</span>
              </button>
            {/if}

            <button
              type="button"
              on:click={() => handleScopeModeChange('single')}
              aria-pressed={scopeMode === 'single'}
              class="flex items-center justify-center gap-1.5 py-1.5 px-2.5 rounded-md text-xs font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
                scopeMode === 'single'
                  ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-sm'
                  : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-850 border border-transparent'
              }"
            >
              <FolderGit2 size={13} />
              <span>Single Repo</span>
            </button>
          </div>

          {#if scopeMode === 'single'}
            <div class="mt-0.5">
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
                  {#each $scannedRepos as repo (repo.repoPath)}
                    <option value={repo.repoPath}>
                      {repo.repoName} ({repo.repoPath})
                    </option>
                  {/each}
                </select>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Zone 1: Daily Sync & Constructive Maintenance -->
        <div class="flex flex-col gap-1.5">
          <span class="text-[11px] font-semibold uppercase tracking-wider text-emerald-400">
            Daily Sync & Maintenance
          </span>
          <button
            type="button"
            on:click={() => handleSelectPreset('syncDefaultBranch')}
            class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500/60 {
              isSyncPreset
                ? 'bg-emerald-950/30 border-emerald-500/70 ring-1 ring-emerald-500/30'
                : 'bg-neutral-950/40 border-neutral-800 hover:border-neutral-700'
            }"
          >
            <div class="flex items-center justify-between gap-2">
              <div class="flex items-center gap-1.5 text-neutral-100 font-medium text-xs">
                <FolderSync size={14} class="text-emerald-400" />
                <span>Sync All to Main (Checkout Default + Fetch & Pull)</span>
              </div>
              <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-emerald-950/90 text-emerald-300 border border-emerald-800/60">
                Safe · Non-destructive
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 leading-snug">
              Runs <code class="text-[10px]">git fetch --all --prune</code>, switches clean root worktrees to <code class="text-[10px]">main</code>/<code class="text-[10px]">master</code>, and fast-forwards (<code class="text-[10px]">pull --ff-only</code>). Dirty roots and locked branches are safely skipped.
            </p>
          </button>
        </div>

        <!-- Zone 2: Destructive Bulk Cleanup -->
        <div class="flex flex-col gap-1.5">
          <span class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400">
            Destructive Bulk Cleanup
          </span>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <!-- Discard Uncommitted -->
            <button
              type="button"
              on:click={() => handleSelectPreset('discardUncommitted')}
              class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
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
                <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-amber-950/80 text-amber-300 border border-amber-800/50">
                  Destructive
                </span>
              </div>
              <p class="text-[11px] text-neutral-400 leading-snug">
                Hard resets tracked files and cleans untracked items (<code class="text-[10px]">clean -ffd</code>). Preserves <code class="text-[10px]">.env</code> and gitignores.
              </p>
            </button>

            <!-- Nuke Worktrees -->
            <button
              type="button"
              on:click={() => handleSelectPreset('nukeWorktrees')}
              class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
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

            <!-- Clean Branches -->
            <button
              type="button"
              on:click={() => handleSelectPreset('cleanBranches')}
              class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60 {
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

            <!-- Total Fresh Start -->
            <button
              type="button"
              on:click={() => handleSelectPreset('totalFreshStart')}
              class="text-left p-3 rounded-lg border transition-all flex flex-col gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500/60 {
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
        </div>

        <!-- Live Impact Pre-flight Audit -->
        <div class="p-3 rounded-lg bg-neutral-950/70 border border-neutral-800 flex flex-col gap-2.5">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold text-neutral-300 uppercase tracking-wider flex items-center gap-1.5">
              <ShieldCheck size={14} class="text-indigo-400" />
              <span>Pre-flight Live Impact Audit</span>
            </span>
            {#if $isLoadingQuickActionPreview}
              <span class="text-[11px] text-neutral-400 animate-pulse">Calculating impact...</span>
            {/if}
          </div>

          {#if $quickActionPreviewError}
            <div class="p-2 text-xs text-rose-300 bg-rose-950/40 border border-rose-800/50 rounded">
              {$quickActionPreviewError}
            </div>
          {:else if $quickActionPreview}
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-center">
              {#each auditCards as card (card.key)}
                {@const expandable = card.items.length > 0}
                <button
                  type="button"
                  disabled={!expandable}
                  aria-expanded={expandable ? expandedAuditKey === card.key : undefined}
                  title={expandable ? 'Show exact items' : undefined}
                  on:click={() => (expandedAuditKey = expandedAuditKey === card.key ? null : card.key)}
                  class="p-2 rounded bg-neutral-900 border transition-colors disabled:cursor-default focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
                    {expandedAuditKey === card.key ? 'border-indigo-500/60' : 'border-neutral-800'}
                    {expandable ? 'hover:border-neutral-600 cursor-pointer' : ''}"
                >
                  <span class="text-xs font-semibold {card.color}">{card.count}</span>
                  <p class="text-[10px] text-neutral-400 uppercase mt-0.5">{card.label}</p>
                </button>
              {/each}
            </div>

            {#if expandedCard}
              <ul class="max-h-28 overflow-y-auto space-y-0.5 p-2 rounded bg-neutral-900 border border-neutral-800 text-[11px] font-mono text-neutral-300 no-scrollbar">
                {#each expandedCard.items as item (item)}
                  <li class="truncate" title={item}>{item}</li>
                {/each}
              </ul>
            {/if}

            <div class="flex items-center justify-between text-[11px] text-neutral-400 pt-1">
              <span>{isSyncPreset ? 'Target Default Branches:' : 'Protected Default Branches:'}</span>
              <span class="font-mono text-neutral-300">
                {$quickActionPreview.protectedDefaultBranches.join(', ') || 'main'}
              </span>
            </div>

            <!-- Prominent Unpushed Commits Warning Banner (Destructive Branch Deletions) -->
            {#if !isSyncPreset && ($quickActionPreview.unpushedBranchesCount > 0 || $quickActionPreview.totalUnpushedCommits > 0)}
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

        <!-- Confirmation / Execution Trigger Controls -->
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
              class="px-3 py-1.5 text-xs font-medium text-neutral-400 hover:text-neutral-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
            >
              Cancel
            </button>

            <div class="flex items-center gap-2">
              {#if isSyncPreset}
                <button
                  type="button"
                  on:click={handleExecute}
                  disabled={$isExecutingQuickAction || $isLoadingQuickActionPreview || !canExecuteSync}
                  class="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-md bg-emerald-600 hover:bg-emerald-500 text-white transition-all disabled:opacity-40 disabled:cursor-not-allowed shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400"
                >
                  {#if $isExecutingQuickAction}
                    <RotateCcw size={13} class="animate-spin" />
                    <span>Syncing repositories...</span>
                  {:else}
                    <FolderSync size={13} />
                    <span>
                      Sync {$quickActionPreview?.reposCount ?? 0} {$quickActionPreview?.reposCount === 1 ? 'Repository' : 'Repositories'}
                    </span>
                  {/if}
                </button>
              {:else if isNuclearPreset}
                <button
                  type="button"
                  on:click={handleExecute}
                  disabled={!isNuclearConfirmed || $isExecutingQuickAction}
                  class="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-md bg-rose-600 hover:bg-rose-500 text-white transition-all disabled:opacity-40 disabled:cursor-not-allowed shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-400"
                >
                  <Flame size={13} />
                  <span>{$isExecutingQuickAction ? 'Executing Nuclear Wipe...' : 'Execute Nuclear Wipe'}</span>
                </button>
              {:else}
                <button
                  type="button"
                  on:click={handleExecute}
                  disabled={$isExecutingQuickAction || $isLoadingQuickActionPreview || !hasDestructiveImpact}
                  class="flex items-center gap-1.5 px-4 py-1.5 text-xs font-semibold rounded-md transition-all shadow-sm disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 {
                    isArmed
                      ? 'bg-rose-600 hover:bg-rose-500 text-white animate-pulse'
                      : 'bg-indigo-600 hover:bg-indigo-500 text-white'
                  }"
                >
                  {#if $isExecutingQuickAction}
                    <RotateCcw size={13} class="animate-spin" />
                    <span>Executing...</span>
                  {:else if !hasDestructiveImpact && !$isLoadingQuickActionPreview}
                    <span>Nothing to clean</span>
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
