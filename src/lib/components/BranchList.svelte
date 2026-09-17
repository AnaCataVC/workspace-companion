<script lang="ts">
  import { filteredBranches, isScanningBranches, branchScanError, scannedBranches } from '../stores/branchCleaner';
  import { scannedRepos } from '../stores/worktrees';
  import { viewDensity } from '../stores/appConfig';
  import { branchSelection, selectedBranchKeys, branchSelectionKey } from '../stores/branchSelection';
  import BranchItemRow from './BranchItemRow.svelte';
  import BranchCard from './BranchCard.svelte';
  import type { BranchStatusEntry } from '../types';
  import { FolderGit2, CheckSquare, Square, Inbox } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    requestCheckout: BranchStatusEntry;
  }>();

  $: repoNameByPath = new Map($scannedRepos.map((r) => [r.repoPath, r.repoName]));

  $: groupedByRepo = (() => {
    const groups = new Map<string, BranchStatusEntry[]>();
    for (const b of $filteredBranches) {
      const list = groups.get(b.repoPath) || [];
      list.push(b);
      groups.set(b.repoPath, list);
    }
    return Array.from(groups.entries());
  })();

  function getDeletable(branches: BranchStatusEntry[]): BranchStatusEntry[] {
    return branches.filter((b) => !b.isDefault && !b.isCheckedOut);
  }

  function isRepoAllSelected(branches: BranchStatusEntry[]): boolean {
    const deletable = getDeletable(branches);
    if (deletable.length === 0) return false;
    return deletable.every((b) => $selectedBranchKeys.has(branchSelectionKey({ repoPath: b.repoPath, branchName: b.name })));
  }

  function toggleSelectRepo(repoPath: string, branches: BranchStatusEntry[]) {
    const deletable = getDeletable(branches);
    if (deletable.length === 0) return;

    if (isRepoAllSelected(branches)) {
      branchSelection.deselectRepo(deletable.map((b) => ({ repoPath: b.repoPath, branchName: b.name })));
    } else {
      branchSelection.selectRepo(
        deletable.map((b) => ({
          repoPath: b.repoPath,
          branchName: b.name,
          force: false,
          repoName: repoNameByPath.get(repoPath) || '',
          isMerged: b.isMerged,
          isRemoteGone: b.isRemoteGone
        }))
      );
    }
  }
</script>

<div class="flex-1 overflow-y-auto p-3 space-y-3 custom-scroll">
  {#if $branchScanError}
    <div class="p-2 rounded bg-rose-950/60 border border-rose-800/50 text-rose-300 text-xs">
      {$branchScanError}
    </div>
  {:else if $isScanningBranches && $scannedBranches.length === 0}
    <div class="flex items-center justify-center h-48 text-neutral-500 text-xs">
      Scanning branches...
    </div>
  {:else if groupedByRepo.length === 0}
    <div class="flex flex-col items-center justify-center h-48 text-neutral-500 gap-2 select-none">
      <Inbox size={20} />
      <p class="text-xs">No branches match this filter.</p>
    </div>
  {:else}
    {#each groupedByRepo as [repoPath, branches] (repoPath)}
      {@const repoName = repoNameByPath.get(repoPath) || repoPath}
      {@const deletable = getDeletable(branches)}
      {@const allSelected = isRepoAllSelected(branches)}
      <div class="space-y-1.5 bg-neutral-950/40 border border-neutral-850 rounded-lg p-2.5">
        <div class="flex items-center justify-between px-1 mb-1">
          <div class="flex items-center gap-1.5 min-w-0">
            <FolderGit2 size={13} class="text-indigo-400 flex-shrink-0" />
            <span class="text-xs font-semibold text-neutral-200 truncate" title={repoPath}>{repoName}</span>
            <span class="px-1.5 py-0.2 rounded-full bg-neutral-800 text-[10px] text-neutral-400 font-mono">
              {branches.length}
            </span>
          </div>

          {#if deletable.length > 0}
            <button
              type="button"
              on:click={() => toggleSelectRepo(repoPath, branches)}
              title={allSelected ? 'Deselect all deletable branches in this repo' : 'Select all deletable branches in this repo'}
              class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] transition-colors
                {allSelected
                  ? 'bg-rose-950/60 text-rose-300 border border-rose-800/50 hover:bg-rose-900/70'
                  : 'bg-neutral-850 hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-neutral-750'}"
            >
              {#if allSelected}
                <CheckSquare size={11} class="text-rose-400" />
                <span>Deselect All</span>
              {:else}
                <Square size={11} />
                <span>Select All ({deletable.length})</span>
              {/if}
            </button>
          {/if}
        </div>

        <!-- Branch Presentation (Compact Rows vs Detailed Cards), same density switch as WorktreeList -->
        {#if $viewDensity === 'compact'}
          <div class="flex flex-col gap-1">
            {#each branches as branch (branch.name)}
              <BranchItemRow
                {branch}
                repoName={repoName}
                on:requestCheckout={(e) => dispatch('requestCheckout', e.detail)}
              />
            {/each}
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-2">
            {#each branches as branch (branch.name)}
              <BranchCard
                {branch}
                repoName={repoName}
                on:requestCheckout={(e) => dispatch('requestCheckout', e.detail)}
              />
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>
