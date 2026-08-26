<script lang="ts">
  import { scannedRepos, searchFilter } from '../stores/worktrees';
  import { selectedAccountFilter, viewDensity } from '../stores/appConfig';
  import { activeGhAccount } from '../stores/ghAuth';
  import WorktreeCard from './WorktreeCard.svelte';
  import WorktreeItemRow from './WorktreeItemRow.svelte';
  import QuickWorktreeInline from './QuickWorktreeInline.svelte';
  import type { RepositoryWorktrees, WorktreeInfo, SupportedEditor } from '../types';
  import { FolderGit2, Sparkles, Inbox, Plus, Github, Zap, Settings2 } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    openPath: string;
    openEditor: { editor: SupportedEditor; path: string };
    requestSwitchBranch: { worktree: WorktreeInfo; repoPath: string };
    requestDelete: { worktree: WorktreeInfo; repoPath: string };
    cleanAllOrphans: string; // repoPath
    newWorktreeForRepo: string; // repoPath
    switchGhAccount: string; // username
    openSettings: void;
    worktreeCreated: { worktreePath: string; branchName: string };
  }>();

  $: filteredRepos = $scannedRepos
    .filter(repo => {
      if ($selectedAccountFilter === 'ALL') return true;
      if ($selectedAccountFilter === 'UNASSIGNED') return !repo.associatedAccount;
      return repo.associatedAccount === $selectedAccountFilter;
    })
    .map(repo => {
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
    })
    .filter(repo => repo.worktrees.length > 0);

  function getAnchorBranchName(repo: RepositoryWorktrees): string {
    const mainWt = repo.worktrees.find(w => w.isMain) || repo.worktrees[0];
    if (mainWt?.branch) {
      return mainWt.branch.replace('refs/heads/', '');
    }
    return 'main';
  }
</script>

<div class="flex-1 overflow-y-auto p-3 space-y-4">
  {#if filteredRepos.length === 0}
    <div class="h-48 flex flex-col items-center justify-center text-center text-neutral-500 gap-2.5 p-4">
      <Inbox size={28} class="text-neutral-600" />
      {#if $searchFilter}
        <p class="text-xs">No worktrees match your filter.</p>
        <p class="text-[11px] text-neutral-600">Try adjusting your search query.</p>
      {:else if $selectedAccountFilter !== 'ALL'}
        <p class="text-xs">No repositories assigned to {$selectedAccountFilter}.</p>
        <button
          type="button"
          on:click={() => dispatch('openSettings')}
          class="px-2.5 py-1 rounded bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-[11px] transition-colors flex items-center gap-1.5"
        >
          <Settings2 size={12} /> Configure Watched Folders
        </button>
      {:else}
        <p class="text-xs">No active repositories found.</p>
        <button
          type="button"
          on:click={() => dispatch('openSettings')}
          class="px-3 py-1.5 rounded-lg bg-indigo-950/80 hover:bg-indigo-900 border border-indigo-800/60 text-indigo-300 text-xs transition-colors flex items-center gap-1.5 shadow-sm"
        >
          <Settings2 size={13} /> Add Watched Folders in Settings
        </button>
      {/if}
    </div>
  {:else}
    {#each filteredRepos as repo (repo.repoPath)}
      {@const orphanCount = repo.worktrees.filter(w => w.isOrphaned).length}
      {@const isDifferentAccount = repo.associatedAccount && $activeGhAccount && repo.associatedAccount !== $activeGhAccount}
      {@const anchorBranch = getAnchorBranchName(repo)}
      
      <div class="space-y-1.5 bg-neutral-950/40 border border-neutral-850 rounded-lg p-2.5">
        <!-- Repository Header -->
        <div class="flex items-center justify-between px-1 mb-1">
          <div class="flex items-center gap-1.5 min-w-0">
            <FolderGit2 size={13} class="text-indigo-400 flex-shrink-0" />
            <span class="text-xs font-semibold text-neutral-200 truncate" title={repo.repoPath}>
              {repo.repoName}
            </span>

            <span class="px-1.5 py-0.2 rounded-full bg-neutral-800 text-[10px] text-neutral-400 font-mono">
              {repo.worktrees.length}
            </span>

            <!-- Associated Account Badge -->
            {#if repo.associatedAccount}
              <span
                class="px-1.5 py-0.2 rounded bg-neutral-800/90 text-neutral-400 border border-neutral-700/40 text-[9px] font-mono flex items-center gap-1"
                title={`Associated with GitHub Account @${repo.associatedAccount}`}
              >
                <Github size={9} class="text-neutral-400" />
                <span>@{repo.associatedAccount}</span>
              </span>

              {#if isDifferentAccount}
                <button
                  type="button"
                  on:click={() => dispatch('switchGhAccount', repo.associatedAccount || '')}
                  title={`Switch GitHub CLI to @${repo.associatedAccount}`}
                  class="px-1.5 py-0.2 rounded bg-amber-950/60 hover:bg-amber-900/80 text-amber-300 border border-amber-800/50 text-[9px] font-mono flex items-center gap-0.5 transition-colors"
                >
                  <Zap size={9} />
                  <span>Switch CLI</span>
                </button>
              {/if}
            {/if}
          </div>

          <div class="flex items-center gap-1.5">
            {#if orphanCount > 0}
              <button
                type="button"
                on:click={() => dispatch('cleanAllOrphans', repo.repoPath)}
                class="flex items-center gap-1 px-2 py-0.5 rounded bg-amber-950/60 hover:bg-amber-900/80 text-amber-300 border border-amber-800/40 text-[10px] transition-colors"
              >
                <Sparkles size={10} />
                Clean {orphanCount} orphans
              </button>
            {/if}
          </div>
        </div>

        <!-- Worktrees Presentation (Compact Hierarchy vs Detailed Cards) -->
        {#if $viewDensity === 'compact'}
          <div class="flex flex-col gap-1">
            {#each repo.worktrees as wt, idx (wt.path)}
              <WorktreeItemRow
                worktree={wt}
                repoPath={repo.repoPath}
                isLast={idx === repo.worktrees.length - 1}
                on:openPath={(e) => dispatch('openPath', e.detail)}
                on:openEditor={(e) => dispatch('openEditor', e.detail)}
                on:requestSwitchBranch={(e) => dispatch('requestSwitchBranch', e.detail)}
                on:requestDelete={(e) => dispatch('requestDelete', { worktree: e.detail, repoPath: repo.repoPath })}
              />
            {/each}
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-2">
            {#each repo.worktrees as wt (wt.path)}
              <WorktreeCard
                worktree={wt}
                repoPath={repo.repoPath}
                on:openPath={(e) => dispatch('openPath', e.detail)}
                on:openEditor={(e) => dispatch('openEditor', e.detail)}
                on:requestSwitchBranch={(e) => dispatch('requestSwitchBranch', e.detail)}
                on:requestDelete={(e) => dispatch('requestDelete', { worktree: e.detail, repoPath: repo.repoPath })}
              />
            {/each}
          </div>
        {/if}

        <!-- Quick Inline Worktree Creator -->
        <QuickWorktreeInline
          repoPath={repo.repoPath}
          defaultBranch={anchorBranch}
          on:openAdvancedModal={(e) => dispatch('newWorktreeForRepo', e.detail)}
          on:openEditor={(e) => dispatch('openEditor', e.detail)}
          on:worktreeCreated={(e) => dispatch('worktreeCreated', e.detail)}
        />
      </div>
    {/each}
  {/if}
</div>
