<script lang="ts">
  import { scannedRepos, searchFilter } from '../stores/worktrees';
  import WorktreeCard from './WorktreeCard.svelte';
  import type { RepositoryWorktrees, WorktreeInfo } from '../types';
  import { FolderGit2, Sparkles, Inbox } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    openPath: string;
    requestDelete: { worktree: WorktreeInfo; repoPath: string };
    cleanAllOrphans: string; // repoPath
  }>();

  $: filteredRepos = $scannedRepos.map(repo => {
    const q = $searchFilter.toLowerCase().trim();
    if (!q) return repo;

    const matchedWorktrees = repo.worktrees.filter(wt => 
      (wt.branch && wt.branch.toLowerCase().includes(q)) ||
      wt.path.toLowerCase().includes(q) ||
      (wt.lastCommitMessage && wt.lastCommitMessage.toLowerCase().includes(q))
    );

    return {
      ...repo,
      worktrees: matchedWorktrees
    };
  }).filter(repo => repo.worktrees.length > 0);
</script>

<div class="flex-1 overflow-y-auto p-3 space-y-4">
  {#if filteredRepos.length === 0}
    <div class="h-48 flex flex-col items-center justify-center text-center text-neutral-500 gap-2">
      <Inbox size={28} class="text-neutral-600" />
      <p class="text-xs">No active worktrees found</p>
      {#if $searchFilter}
        <p class="text-[11px] text-neutral-600">Try adjusting your filter search.</p>
      {:else}
        <p class="text-[11px] text-neutral-600">Scan folders or add repositories to track.</p>
      {/if}
    </div>
  {:else}
    {#each filteredRepos as repo (repo.repoPath)}
      {@const orphanCount = repo.worktrees.filter(w => w.isOrphaned).length}
      <div class="space-y-2">
        <!-- Repository Header -->
        <div class="flex items-center justify-between px-1">
          <div class="flex items-center gap-1.5 min-w-0">
            <FolderGit2 size={13} class="text-neutral-400 flex-shrink-0" />
            <span class="text-xs font-medium text-neutral-300 truncate" title={repo.repoPath}>
              {repo.repoName}
            </span>
            <span class="px-1.5 py-0.2 rounded-full bg-neutral-800 text-[10px] text-neutral-400 font-mono">
              {repo.worktrees.length}
            </span>
          </div>

          {#if orphanCount > 0}
            <button
              on:click={() => dispatch('cleanAllOrphans', repo.repoPath)}
              class="flex items-center gap-1 px-2 py-0.5 rounded bg-amber-950/60 hover:bg-amber-900/80 text-amber-300 border border-amber-800/40 text-[10px] transition-colors"
            >
              <Sparkles size={10} />
              Clean {orphanCount} orphans
            </button>
          {/if}
        </div>

        <!-- Worktrees grid/cards -->
        <div class="grid grid-cols-1 gap-2">
          {#each repo.worktrees as wt (wt.path)}
            <WorktreeCard
              worktree={wt}
              repoPath={repo.repoPath}
              on:openPath={(e) => dispatch('openPath', e.detail)}
              on:requestDelete={(e) => dispatch('requestDelete', { worktree: e.detail, repoPath: repo.repoPath })}
            />
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>
