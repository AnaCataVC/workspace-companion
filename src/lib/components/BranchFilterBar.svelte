<script lang="ts">
  import { scannedBranches, selectedBranchStatusFilter } from '../stores/branchCleaner';
  import type { BranchStatusFilterType } from '../types';
  import { Layers, GitMerge, GitPullRequestClosed, ShieldCheck } from 'lucide-svelte';

  // Single-pass O(N) tally, same pattern as AccountFilterBar's statusCounts.
  $: statusCounts = (() => {
    let all = 0;
    let merged = 0;
    let remoteGone = 0;
    let protectedCount = 0;

    const branches = $scannedBranches;
    for (let i = 0; i < branches.length; i++) {
      const b = branches[i];
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
</script>

<div class="px-3 py-1 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar border-b border-neutral-800/80 bg-neutral-950/70 select-none">
  <button
    type="button"
    on:click={() => setFilter('ALL')}
    class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
      {$selectedBranchStatusFilter === 'ALL'
        ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-xs'
        : 'text-neutral-400 hover:text-neutral-300 hover:bg-neutral-900/80 border border-transparent'}"
  >
    <Layers size={11} />
    <span>All</span>
    <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-400">
      {statusCounts.all}
    </span>
  </button>

  <button
    type="button"
    on:click={() => setFilter('MERGED')}
    title="Branches merged into the default branch"
    class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
      {$selectedBranchStatusFilter === 'MERGED'
        ? 'bg-emerald-950/80 text-emerald-200 border border-emerald-800/70 shadow-xs'
        : 'text-neutral-400 hover:text-emerald-300 hover:bg-neutral-900/80 border border-transparent'}"
  >
    <GitMerge size={11} class={$selectedBranchStatusFilter === 'MERGED' ? 'text-emerald-400' : 'text-neutral-500'} />
    <span>Merged</span>
    <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedBranchStatusFilter === 'MERGED' ? 'bg-emerald-900/80 text-emerald-200' : 'bg-neutral-900 text-neutral-400'}">
      {statusCounts.merged}
    </span>
  </button>

  <button
    type="button"
    on:click={() => setFilter('REMOTE_GONE')}
    title="Branches whose upstream remote branch was deleted"
    class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
      {$selectedBranchStatusFilter === 'REMOTE_GONE'
        ? 'bg-amber-950/80 text-amber-200 border border-amber-800/70 shadow-xs'
        : 'text-neutral-400 hover:text-amber-300 hover:bg-neutral-900/80 border border-transparent'}"
  >
    <GitPullRequestClosed size={11} class={$selectedBranchStatusFilter === 'REMOTE_GONE' ? 'text-amber-400' : 'text-neutral-500'} />
    <span>Remote gone</span>
    <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedBranchStatusFilter === 'REMOTE_GONE' ? 'bg-amber-900/80 text-amber-200' : 'bg-neutral-900 text-neutral-400'}">
      {statusCounts.remoteGone}
    </span>
  </button>

  <button
    type="button"
    on:click={() => setFilter('PROTECTED')}
    title="Default branch or checked out in a worktree — never deletable"
    class="flex items-center gap-1 px-2 py-0.5 rounded-md font-medium transition-all flex-shrink-0
      {$selectedBranchStatusFilter === 'PROTECTED'
        ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/70 shadow-xs'
        : 'text-neutral-400 hover:text-indigo-300 hover:bg-neutral-900/80 border border-transparent'}"
  >
    <ShieldCheck size={11} class={$selectedBranchStatusFilter === 'PROTECTED' ? 'text-indigo-400' : 'text-neutral-500'} />
    <span>Protected</span>
    <span class="px-1.5 py-0.2 rounded-full text-[10px] font-mono {$selectedBranchStatusFilter === 'PROTECTED' ? 'bg-indigo-900/80 text-indigo-200' : 'bg-neutral-900 text-neutral-400'}">
      {statusCounts.protectedCount}
    </span>
  </button>
</div>
