<script lang="ts">
  import { selectedAccountFilter, selectedStatusFilter } from '../stores/appConfig';
  import { scannedRepos, filteredRepos } from '../stores/worktrees';
  import { ghAccounts } from '../stores/ghAuth';
  import { batchSelection, selectedPaths } from '../stores/batchSelection';
  import type { StatusFilterType, BatchDeleteTarget } from '../types';
  import { Github, Layers, Flame, GitFork, AlertTriangle, CheckCircle2, CheckSquare, Square } from 'lucide-svelte';

  $: allCount = $scannedRepos.length;

  // Extract unique accounts from accounts store and scanned repos
  $: accounts = Array.from(
    new Set([
      ...$ghAccounts.map(a => a.username),
      ...$scannedRepos.map(r => r.associatedAccount).filter((a): a is string => Boolean(a))
    ])
  );

  // Single-pass O(N) account tally, shared by every account pill instead of
  // each pill re-filtering the full repo list
  $: accountStats = (() => {
    const counts: Record<string, number> = {};
    let unassigned = 0;
    const repos = $scannedRepos;

    for (let i = 0; i < repos.length; i++) {
      const acc = repos[i].associatedAccount;
      if (acc) {
        counts[acc] = (counts[acc] || 0) + 1;
        const lower = acc.toLowerCase();
        if (lower !== acc) {
          counts[lower] = (counts[lower] || 0) + 1;
        }
      } else {
        unassigned++;
      }
    }

    return { counts, unassigned };
  })();

  // Single-pass O(N) calculation for status filter counts within the selected account scope
  $: statusCounts = (() => {
    let all = 0;
    let dirty = 0;
    let multiWt = 0;
    let orphans = 0;
    let clean = 0;

    const targetAccount = $selectedAccountFilter;
    const repos = $scannedRepos;

    for (let i = 0; i < repos.length; i++) {
      const repo = repos[i];
      if (targetAccount !== 'ALL') {
        if (targetAccount === 'UNASSIGNED' && repo.associatedAccount) continue;
        if (targetAccount !== 'UNASSIGNED' && repo.associatedAccount?.toLowerCase() !== targetAccount.toLowerCase()) continue;
      }

      all++;
      if (repo.worktrees.length > 1) {
        multiWt++;
      }

      let repoHasDirty = false;
      let repoHasOrphan = false;

      for (let j = 0; j < repo.worktrees.length; j++) {
        const wt = repo.worktrees[j];
        if (wt.isDirty) repoHasDirty = true;
        if (!wt.isMain && wt.isOrphaned) repoHasOrphan = true;
      }

      if (repoHasDirty) dirty++;
      else clean++;
      if (repoHasOrphan) orphans++;
    }

    return { all, dirty, multiWt, orphans, clean };
  })();

  function setStatusFilter(filter: StatusFilterType) {
    selectedStatusFilter.set(filter);
  }

  // The main worktree is the one the backend refuses outright, so it's never a selectable target.
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

  $: allFilteredSelected = selectableTargets.length > 0 && selectableTargets.every((t) => $selectedPaths.has(t.worktreePath));

  function toggleSelectAllFiltered() {
    if (allFilteredSelected) {
      batchSelection.clear();
    } else {
      batchSelection.selectAll(selectableTargets);
    }
  }
</script>

<div class="flex flex-col border-b border-neutral-800/80 bg-neutral-950/70 select-none">
  <!-- Top Row: GitHub Account Pills (if accounts exist) -->
  {#if accounts.length > 0 || accountStats.unassigned > 0}
    <div class="px-3 py-1.5 border-b border-neutral-850/60 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar">
      <!-- All Repositories Pill -->
      <button
        type="button"
        on:click={() => selectedAccountFilter.set('ALL')}
        class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0
          {$selectedAccountFilter === 'ALL'
            ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-xs'
            : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
      >
        <Layers size={11} />
        <span>All Accounts</span>
        <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-400">
          {allCount}
        </span>
      </button>

      <!-- Account Specific Pills -->
      {#each accounts as acc (acc)}
        <button
          type="button"
          on:click={() => selectedAccountFilter.set(acc)}
          class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0
            {$selectedAccountFilter === acc
              ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/60 shadow-xs'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
        >
          <Github size={11} class={$selectedAccountFilter === acc ? 'text-indigo-400' : 'text-neutral-500'} />
          <span class="font-mono text-[10px]">@{acc}</span>
          <span class="px-1.5 py-0.2 rounded-full bg-neutral-900/80 text-[10px] font-mono {$selectedAccountFilter === acc ? 'text-indigo-300' : 'text-neutral-500'}">
            {accountStats.counts[acc] ?? accountStats.counts[acc.toLowerCase()] ?? 0}
          </span>
        </button>
      {/each}

      <!-- Unassigned Pill -->
      {#if accountStats.unassigned > 0 && accounts.length > 0}
        <button
          type="button"
          on:click={() => selectedAccountFilter.set('UNASSIGNED')}
          class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0
            {$selectedAccountFilter === 'UNASSIGNED'
              ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-xs'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
        >
          <span>Unassigned</span>
          <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-500">
            {accountStats.unassigned}
          </span>
        </button>
      {/if}
    </div>
  {/if}

  <!-- Bottom Row: Smart Status Filter Chips -->
  <div class="px-3 py-1 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar">
    <!-- All Status Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('ALL')}
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
        {$selectedStatusFilter === 'ALL'
          ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-xs'
          : 'text-neutral-400 hover:text-neutral-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <span>All Repos</span>
      <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-400">
        {statusCounts.all}
      </span>
    </button>

    <!-- Dirty Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('DIRTY')}
      title="Filter repositories with uncommitted changes"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
        {$selectedStatusFilter === 'DIRTY'
          ? 'bg-rose-950/80 text-rose-200 border border-rose-800/70 shadow-xs'
          : 'text-neutral-400 hover:text-rose-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <Flame size={11} class={$selectedStatusFilter === 'DIRTY' ? 'text-rose-400 animate-pulse' : 'text-neutral-500'} />
      <span>Dirty</span>
      <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedStatusFilter === 'DIRTY' ? 'bg-rose-900/80 text-rose-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.dirty}
      </span>
    </button>

    <!-- Multi-Worktree Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('MULTI_WT')}
      title="Filter repositories with more than 1 worktree"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
        {$selectedStatusFilter === 'MULTI_WT'
          ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/70 shadow-xs'
          : 'text-neutral-400 hover:text-indigo-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <GitFork size={11} class={$selectedStatusFilter === 'MULTI_WT' ? 'text-indigo-400' : 'text-neutral-500'} />
      <span>Multi-WT</span>
      <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedStatusFilter === 'MULTI_WT' ? 'bg-indigo-900/80 text-indigo-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.multiWt}
      </span>
    </button>

    <!-- Orphans Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('ORPHANS')}
      title="Filter repositories with orphaned worktrees"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
        {$selectedStatusFilter === 'ORPHANS'
          ? 'bg-amber-950/80 text-amber-200 border border-amber-800/70 shadow-xs'
          : 'text-neutral-400 hover:text-amber-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <AlertTriangle size={11} class={$selectedStatusFilter === 'ORPHANS' ? 'text-amber-400' : 'text-neutral-500'} />
      <span>Orphans</span>
      <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedStatusFilter === 'ORPHANS' ? 'bg-amber-900/80 text-amber-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.orphans}
      </span>
    </button>

    <!-- Clean Chip -->
    <button
      type="button"
      on:click={() => setStatusFilter('CLEAN')}
      title="Filter repositories with no uncommitted changes"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
        {$selectedStatusFilter === 'CLEAN'
          ? 'bg-emerald-950/80 text-emerald-200 border border-emerald-800/70 shadow-xs'
          : 'text-neutral-400 hover:text-emerald-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <CheckCircle2 size={11} class={$selectedStatusFilter === 'CLEAN' ? 'text-emerald-400' : 'text-neutral-500'} />
      <span>Clean</span>
      <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedStatusFilter === 'CLEAN' ? 'bg-emerald-900/80 text-emerald-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.clean}
      </span>
    </button>

    <!-- Selects / deselects across every repo currently shown, not just one repo's block -->
    {#if selectableTargets.length > 0}
      <button
        type="button"
        on:click={toggleSelectAllFiltered}
        title={allFilteredSelected ? 'Deselect all filtered worktrees' : 'Select every worktree matching the active filters, across all repositories'}
        class="ml-auto flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-colors flex-shrink-0
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
