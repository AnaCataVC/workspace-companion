<script lang="ts">
  import {
    scannedBranches,
    selectedBranchStatusFilter,
    filteredBranches,
    branchSearchFilter,
    branchMatchesSearch,
    hasActiveBranchFilters,
    resetBranchFilters
  } from '../stores/branchCleaner';
  import { selectedAccountFilter } from '../stores/appConfig';
  import { scannedRepos } from '../stores/worktrees';
  import { branchSelection, selectedBranchKeys, branchSelectionKey } from '../stores/branchSelection';
  import AccountPillsBar from './AccountPillsBar.svelte';
  import type { BranchStatusFilterType, BranchDeleteTarget } from '../types';
  import {
    Layers,
    GitMerge,
    GitPullRequestClosed,
    ShieldCheck,
    CheckSquare,
    Square,
    FilterX
  } from 'lucide-svelte';

  $: searchQuery = $branchSearchFilter.toLowerCase().trim();
  $: repoByPath = new Map($scannedRepos.map((r) => [r.repoPath, r]));

  $: accountStats = (() => {
    const counts: Record<string, number> = {};
    let unassigned = 0;
    let all = 0;

    for (const b of $scannedBranches) {
      const repo = repoByPath.get(b.repoPath);
      const acc = repo?.associatedAccount ?? null;
      if (!branchMatchesSearch(b, searchQuery, repo?.repoName, acc)) continue;

      all++;
      if (acc) {
        const key = acc.toLowerCase();
        counts[key] = (counts[key] || 0) + 1;
      } else {
        unassigned++;
      }
    }

    return { counts, unassigned, all };
  })();

  // Single-pass O(N) tally respecting both account filter and search box so every pill's number
  // matches what clicking it renders.
  $: statusCounts = (() => {
    let all = 0;
    let merged = 0;
    let remoteGone = 0;
    let protectedCount = 0;

    const targetAccount = $selectedAccountFilter;
    const branches = $scannedBranches;

    for (let i = 0; i < branches.length; i++) {
      const b = branches[i];
      const repo = repoByPath.get(b.repoPath);
      const repoAccount = repo?.associatedAccount ?? null;

      if (targetAccount !== 'ALL') {
        if (targetAccount === 'UNASSIGNED' && repoAccount) continue;
        if (targetAccount !== 'UNASSIGNED' && repoAccount?.toLowerCase() !== targetAccount.toLowerCase()) continue;
      }

      if (!branchMatchesSearch(b, searchQuery, repo?.repoName, repoAccount)) continue;

      all++;
      if (b.isDefault || b.isCheckedOut) protectedCount++;
      if (b.isMerged) merged++;
      if (b.isRemoteGone) remoteGone++;
    }

    return { all, merged, remoteGone, protectedCount };
  })();

  function setFilter(filter: BranchStatusFilterType) {
    selectedBranchStatusFilter.set(filter);
  }

  // The default branch and any checked-out branch are refused outright by the backend (ADR 0006).
  $: selectableTargets = $filteredBranches
    .filter((b) => !b.isDefault && !b.isCheckedOut)
    .map<BranchDeleteTarget>((b) => ({
      repoPath: b.repoPath,
      branchName: b.name,
      force: false,
      repoName: repoByPath.get(b.repoPath)?.repoName || '',
      isMerged: b.isMerged,
      isRemoteGone: b.isRemoteGone
    }));

  $: allFilteredSelected =
    selectableTargets.length > 0 &&
    selectableTargets.every((b) => $selectedBranchKeys.has(branchSelectionKey(b)));

  function toggleSelectAllFiltered() {
    if (allFilteredSelected) {
      branchSelection.deselectRepo(
        selectableTargets.map((b) => ({ repoPath: b.repoPath, branchName: b.branchName }))
      );
    } else {
      branchSelection.selectRepo(selectableTargets);
    }
  }
</script>

<div class="flex flex-col border-b border-neutral-800/80 bg-neutral-950/70 select-none">
  <AccountPillsBar
    countsByAccountLower={accountStats.counts}
    unassignedCount={accountStats.unassigned}
    totalCount={accountStats.all}
  />

  <div class="px-3 py-1 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar">
    <button
      type="button"
      on:click={() => setFilter('ALL')}
      aria-pressed={$selectedBranchStatusFilter === 'ALL'}
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedBranchStatusFilter === 'ALL'
          ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-sm'
          : 'text-neutral-400 hover:text-neutral-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <Layers size={11} />
      <span>All Branches</span>
      <span class="px-1.5 py-0.5 rounded-full bg-neutral-900 text-[11px] font-mono text-neutral-400">
        {statusCounts.all}
      </span>
    </button>

    <button
      type="button"
      on:click={() => setFilter('MERGED')}
      aria-pressed={$selectedBranchStatusFilter === 'MERGED'}
      title="Branches merged into the default branch"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedBranchStatusFilter === 'MERGED'
          ? 'bg-emerald-950/80 text-emerald-200 border border-emerald-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-emerald-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <GitMerge size={11} class={$selectedBranchStatusFilter === 'MERGED' ? 'text-emerald-400' : 'text-neutral-400'} />
      <span>Merged</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedBranchStatusFilter === 'MERGED' ? 'bg-emerald-900/80 text-emerald-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.merged}
      </span>
    </button>

    <button
      type="button"
      on:click={() => setFilter('REMOTE_GONE')}
      aria-pressed={$selectedBranchStatusFilter === 'REMOTE_GONE'}
      title="Branches whose upstream remote branch was deleted"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedBranchStatusFilter === 'REMOTE_GONE'
          ? 'bg-amber-950/80 text-amber-200 border border-amber-800/70 shadow-sm'
          : 'text-neutral-400 hover:text-amber-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <GitPullRequestClosed size={11} class={$selectedBranchStatusFilter === 'REMOTE_GONE' ? 'text-amber-400' : 'text-neutral-400'} />
      <span>Remote gone</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedBranchStatusFilter === 'REMOTE_GONE' ? 'bg-amber-900/80 text-amber-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.remoteGone}
      </span>
    </button>

    <button
      type="button"
      on:click={() => setFilter('PROTECTED')}
      aria-pressed={$selectedBranchStatusFilter === 'PROTECTED'}
      title="Default branch or checked out in a worktree — never deletable"
      class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedBranchStatusFilter === 'PROTECTED'
          ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/70 shadow-sm'
          : 'text-neutral-400 hover:text-indigo-300 hover:bg-neutral-900/80 border border-transparent'}"
    >
      <ShieldCheck size={11} class={$selectedBranchStatusFilter === 'PROTECTED' ? 'text-indigo-400' : 'text-neutral-400'} />
      <span>Protected</span>
      <span class="px-1.5 py-0.5 rounded-full text-[11px] font-mono {$selectedBranchStatusFilter === 'PROTECTED' ? 'bg-indigo-900/80 text-indigo-200' : 'bg-neutral-900 text-neutral-400'}">
        {statusCounts.protectedCount}
      </span>
    </button>

    <div class="ml-auto flex items-center gap-1.5 flex-shrink-0">
      {#if $hasActiveBranchFilters}
        <button
          type="button"
          on:click={resetBranchFilters}
          title="Reset all active filters"
          class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium bg-indigo-950/70 hover:bg-indigo-900/80 text-indigo-300 border border-indigo-800/60 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
        >
          <FilterX size={11} />
          <span>Reset</span>
        </button>
      {/if}

      {#if selectableTargets.length > 0}
        <button
          type="button"
          on:click={toggleSelectAllFiltered}
          title={allFilteredSelected
            ? 'Deselect all filtered branches'
            : 'Select every deletable branch matching the active filter, across all repositories'}
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
