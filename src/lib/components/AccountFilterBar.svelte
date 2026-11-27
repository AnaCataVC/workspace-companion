<script lang="ts">
  import { selectedAccountFilter, selectedStatusFilter } from '../stores/appConfig';
  import {
    scannedRepos,
    filteredRepos,
    searchFilter,
    worktreeMatchesSearch,
    isRepoOffDefaultBranch,
    hasActiveWorktreeFilters,
    resetWorktreeFilters,
    collapsedRepos,
    collapseAllRepos,
    expandAllRepos,
    isFilterActive
  } from '../stores/worktrees';
  import { batchSelection, selectedPaths } from '../stores/batchSelection';
  import AccountPillsBar from './AccountPillsBar.svelte';
  import type { StatusFilterType, BatchDeleteTarget } from '../types';
  import {
    ArrowLeftRight,
    CircleDot,
    GitFork,
    AlertTriangle,
    CheckCircle2,
    CheckSquare,
    Square,
    FilterX,
    ChevronsDownUp,
    ChevronsUpDown
  } from 'lucide-svelte';

  $: searchQuery = $searchFilter.toLowerCase().trim();
  $: searchedRepos = $scannedRepos.map((repo) => ({
    ...repo,
    worktrees: repo.worktrees.filter((wt) =>
      worktreeMatchesSearch(wt, searchQuery, repo.repoName, repo.associatedAccount)
    )
  }));

  $: accountStats = (() => {
    const counts: Record<string, number> = {};
    let unassigned = 0;
    let all = 0;
    for (const repo of searchedRepos) {
      const count = repo.worktrees.length;
      all += count;
      if (repo.associatedAccount) {
        const key = repo.associatedAccount.toLowerCase();
        counts[key] = (counts[key] || 0) + count;
      } else {
        unassigned += count;
      }
    }
    return { counts, unassigned, all };
  })();

  // Worktree counts per status filter within the selected account scope
  $: statusCounts = (() => {
    let all = 0;
    let offMain = 0;
    let dirty = 0;
    let multiWt = 0;
    let orphans = 0;
    let clean = 0;

    const targetAccount = $selectedAccountFilter;

    for (const repo of $scannedRepos) {
      if (targetAccount !== 'ALL') {
        if (targetAccount === 'UNASSIGNED' && repo.associatedAccount) continue;
        if (targetAccount !== 'UNASSIGNED' && repo.associatedAccount?.toLowerCase() !== targetAccount.toLowerCase()) continue;
      }

      const isMultiWt = repo.worktrees.length > 1;
      const repoOffMain = isRepoOffDefaultBranch(repo);

      for (const wt of repo.worktrees) {
        if (!worktreeMatchesSearch(wt, searchQuery, repo.repoName, repo.associatedAccount)) continue;
        all++;
        if (repoOffMain) offMain++;
        if (isMultiWt) multiWt++;
        if (wt.isDirty) dirty++;
        else clean++;
        if (!wt.isMain && wt.isOrphaned) orphans++;
      }
    }

    return { all, offMain, dirty, multiWt, orphans, clean };
  })();

  function setStatusFilter(filter: StatusFilterType) {
    selectedStatusFilter.set(filter);
  }

  $: allVisibleCollapsed =
    $filteredRepos.length > 0 && $filteredRepos.every((r) => $collapsedRepos.has(r.repoPath));

  function toggleCollapseAll() {
    if (allVisibleCollapsed) {
      expandAllRepos();
    } else {
      collapseAllRepos($scannedRepos.map((r) => r.repoPath));
    }
  }

  // The main worktree is refused by the backend, so it is never a selectable deletion target.
  $: selectableTargets = $filteredRepos.flatMap<BatchDeleteTarget>((repo) =>
    repo.worktrees
      .filter((wt) => !wt.isMain)
      .map((wt) => ({
        repoPath: repo.repoPath,
        repoName: repo.repoName,
        worktreePath: wt.path,
        force: false,
        branch: wt.branch,
        isDirty: wt.isDirty,
        uncommittedFilesCount: wt.uncommittedFilesCount
      }))
  );

  $: allFilteredSelected =
    selectableTargets.length > 0 && selectableTargets.every((t) => $selectedPaths.has(t.worktreePath));

  function toggleSelectAllFiltered() {
    if (allFilteredSelected) {
      batchSelection.deselectRepo(selectableTargets.map((t) => t.worktreePath));
    } else {
      batchSelection.selectRepo(selectableTargets);
    }
  }
</script>

<div class="flex flex-col border-b border-neutral-800/80 bg-neutral-950/70 select-none">
  <AccountPillsBar
    countsByAccountLower={accountStats.counts}
    unassignedCount={accountStats.unassigned}
    totalCount={accountStats.all}
  />

  <!-- Smart Status Filter Chips -->
  <div class="px-3 py-1 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar">
    <button
      type="button"
      on:click={() => setStatusFilter('ALL')}
      aria-pressed={$selectedStatusFilter === 'ALL'}
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'ALL'
          ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-sm'
          : 'text-neutral-400 hover:text-neutral-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <span>All Worktrees</span>
      <span class="px-1.5 py-0.5 rounded-full bg-neutral-900 text-[11px] font-mono text-neutral-400">
        {statusCounts.all}
      </span>
    </button>

    <!-- Off Main Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('OFF_MAIN')}
      aria-pressed={$selectedStatusFilter === 'OFF_MAIN'}
      title="Filter repositories whose root worktree is not on its default branch (main/master)"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'OFF_MAIN'
          ? 'bg-amber-950/80 text-amber-200 border border-amber-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-amber-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <ArrowLeftRight size={11} class={$selectedStatusFilter === 'OFF_MAIN' ? 'text-amber-400' : 'text-neutral-400'} />
      <span>Off main</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedStatusFilter === 'OFF_MAIN' ? 'bg-amber-900/80 text-amber-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.offMain}
      </span>
    </button>

    <!-- Dirty Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('DIRTY')}
      aria-pressed={$selectedStatusFilter === 'DIRTY'}
      title="Filter worktrees with uncommitted changes"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'DIRTY'
          ? 'bg-rose-950/80 text-rose-200 border border-rose-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-rose-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <CircleDot size={11} class={$selectedStatusFilter === 'DIRTY' ? 'text-rose-400' : 'text-neutral-400'} />
      <span>Dirty</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedStatusFilter === 'DIRTY' ? 'bg-rose-900/80 text-rose-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.dirty}
      </span>
    </button>

    <!-- Multi-Worktree Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('MULTI_WT')}
      aria-pressed={$selectedStatusFilter === 'MULTI_WT'}
      title="Filter repositories with more than 1 worktree"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'MULTI_WT'
          ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/70 shadow-sm'
          : 'text-neutral-400 hover:text-indigo-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <GitFork size={11} class={$selectedStatusFilter === 'MULTI_WT' ? 'text-indigo-400' : 'text-neutral-400'} />
      <span>Multi-WT</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedStatusFilter === 'MULTI_WT' ? 'bg-indigo-900/80 text-indigo-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.multiWt}
      </span>
    </button>

    <!-- Orphans Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('ORPHANS')}
      aria-pressed={$selectedStatusFilter === 'ORPHANS'}
      title="Filter orphaned secondary worktrees"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'ORPHANS'
          ? 'bg-amber-950/80 text-amber-200 border border-amber-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-amber-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <AlertTriangle size={11} class={$selectedStatusFilter === 'ORPHANS' ? 'text-amber-400' : 'text-neutral-400'} />
      <span>Orphans</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedStatusFilter === 'ORPHANS' ? 'bg-amber-900/80 text-amber-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.orphans}
      </span>
    </button>

    <!-- Clean Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('CLEAN')}
      aria-pressed={$selectedStatusFilter === 'CLEAN'}
      title="Filter worktrees with no uncommitted changes"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedStatusFilter === 'CLEAN'
          ? 'bg-emerald-950/80 text-emerald-200 border border-emerald-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-emerald-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <CheckCircle2 size={11} class={$selectedStatusFilter === 'CLEAN' ? 'text-emerald-400' : 'text-neutral-400'} />
      <span>Clean</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedStatusFilter === 'CLEAN' ? 'bg-emerald-900/80 text-emerald-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.clean}
      </span>
    </button>

    <!-- Right-side Filter Bar Utilities -->
    <div class="ml-auto flex items-center gap-1.5 flex-shrink-0">
      {#if $hasActiveWorktreeFilters}
        <button
          type="button"
          on:click={resetWorktreeFilters}
          title="Reset all active filters"
          class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-indigo-950/70 hover:bg-indigo-900/80 text-indigo-300 border border-indigo-800/60 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
        >
          <FilterX size={11} />
          <span>Reset</span>
        </button>
      {/if}

      {#if !$isFilterActive && $filteredRepos.length > 1}
        <button
          type="button"
          on:click={toggleCollapseAll}
          title={allVisibleCollapsed ? 'Expand all repository sections' : 'Collapse all repository sections'}
          class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-neutral-850 hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-neutral-750 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
        >
          {#if allVisibleCollapsed}
            <ChevronsUpDown size={11} />
            <span>Expand all</span>
          {:else}
            <ChevronsDownUp size={11} />
            <span>Collapse all</span>
          {/if}
        </button>
      {/if}

      {#if selectableTargets.length > 0}
        <button
          type="button"
          on:click={toggleSelectAllFiltered}
          title={allFilteredSelected
            ? 'Deselect all filtered worktrees'
            : 'Select every worktree matching the active filters, across all repositories'}
          class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
            {allFilteredSelected
              ? 'bg-rose-950/80 hover:bg-rose-900/80 text-rose-300 border border-rose-800/70'
              : 'bg-neutral-850 hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-neutral-750'}"
        >
          {#if allFilteredSelected}
            <CheckSquare size={11} class="text-rose-400" />
            <span>Deselect all ({selectableTargets.length})</span>
          {:else}
            <Square size={11} />
            <span>Select all ({selectableTargets.length})</span>
          {/if}
        </button>
      {/if}
    </div>
  </div>
</div>
