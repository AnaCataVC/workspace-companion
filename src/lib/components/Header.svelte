<script lang="ts">
  import { isPinned, searchFilter } from '../stores/worktrees';
  import { branchSearchFilter } from '../stores/branchCleaner';
  import { activeGhAccount } from '../stores/ghAuth';
  import { viewDensity } from '../stores/appConfig';
  import {
    RefreshCw,
    Pin,
    PinOff,
    Github,
    Search,
    Plus,
    Settings2,
    LayoutList,
    LayoutGrid,
    GitBranch,
    FolderTree,
    FolderSync,
    Zap,
    X
  } from 'lucide-svelte';

  export let activeView: 'worktrees' | 'branches' = 'worktrees';
  export let isRefreshing: boolean = false;
  export let isSyncingMain: boolean = false;
  export let hasFilteredScope: boolean = false;
  export let canSyncMain: boolean = true;
  export let onRefresh: () => void;
  export let onSyncMain: () => void;
  export let onSetView: (view: 'worktrees' | 'branches') => void;
  export let onOpenGhModal: () => void;
  export let onOpenNewWorktreeModal: () => void;
  export let onOpenSettingsModal: () => void;
  export let onOpenQuickActionsModal: () => void;
  export let onTogglePin: () => void;
  export let onToggleDensity: () => void;

  let searchInputEl: HTMLInputElement | null = null;

  export function focusSearchInput() {
    searchInputEl?.focus();
    searchInputEl?.select();
  }
</script>

<header class="p-3 bg-neutral-950/80 backdrop-blur border-b border-neutral-800 flex flex-col gap-2">
  <div class="flex items-center justify-between gap-2">
    <!-- Left Navigation: Segmented View Switcher and Single Primary CTA -->
    <div class="flex items-center gap-2">
      <div class="flex items-center rounded-md bg-neutral-900 border border-neutral-800 p-0.5 gap-0.5">
        <button
          type="button"
          on:click={() => onSetView('worktrees')}
          title="Show worktrees"
          aria-pressed={activeView === 'worktrees'}
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
            {activeView === 'worktrees'
              ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-sm'
              : 'text-neutral-400 hover:bg-neutral-850 hover:text-neutral-200 border border-transparent'}"
        >
          <FolderTree size={13} class={activeView === 'worktrees' ? 'text-indigo-400' : ''} />
          <span>Worktrees</span>
        </button>
        <button
          type="button"
          on:click={() => onSetView('branches')}
          title="Show branches"
          aria-pressed={activeView === 'branches'}
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
            {activeView === 'branches'
              ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-sm'
              : 'text-neutral-400 hover:bg-neutral-850 hover:text-neutral-200 border border-transparent'}"
        >
          <GitBranch size={13} class={activeView === 'branches' ? 'text-indigo-400' : ''} />
          <span>Branches</span>
        </button>
      </div>

      {#if activeView === 'worktrees'}
        <!-- Sole Primary CTA in Header -->
        <button
          type="button"
          on:click={onOpenNewWorktreeModal}
          title="Create new Git worktree"
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-md bg-indigo-600 hover:bg-indigo-500 text-white transition-colors shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
        >
          <Plus size={13} />
          <span>New</span>
        </button>
      {/if}
    </div>

    <!-- Right Controls: 1-Click Sync Main, View Density & Global Utility Buttons -->
    <div class="flex items-center gap-1.5">
      <!-- 1-Click Constructive Sync to Default Branch -->
      <button
        type="button"
        on:click={onSyncMain}
        disabled={isSyncingMain || !canSyncMain}
        aria-busy={isSyncingMain}
        title={hasFilteredScope
          ? 'Switch clean filtered repositories to main/master and run fetch --prune + pull --ff-only'
          : 'Switch all clean watched repositories to main/master and run fetch --prune + pull --ff-only'}
        class="flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-md bg-emerald-950/70 hover:bg-emerald-900/80 text-emerald-300 border border-emerald-800/60 transition-colors disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400/70"
      >
        <FolderSync size={13} class={isSyncingMain ? 'animate-spin text-emerald-400' : 'text-emerald-400'} />
        <span class="hidden sm:inline">
          {isSyncingMain ? 'Syncing...' : hasFilteredScope ? 'Sync filtered' : 'Sync main'}
        </span>
      </button>

      <!-- View Density Toggle -->
      <button
        type="button"
        on:click={onToggleDensity}
        title={$viewDensity === 'compact' ? 'Switch to Detailed Cards view' : 'Switch to Compact Tree view'}
        aria-label={$viewDensity === 'compact' ? 'Switch to Detailed Cards view' : 'Switch to Compact Tree view'}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-indigo-300 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        {#if $viewDensity === 'compact'}
          <LayoutList size={14} class="text-indigo-400" />
        {:else}
          <LayoutGrid size={14} class="text-neutral-400" />
        {/if}
      </button>

      <!-- GitHub Account Badge Button -->
      <button
        type="button"
        on:click={onOpenGhModal}
        title="Switch GitHub CLI account"
        class="flex items-center gap-1.5 px-2 py-1 text-xs rounded-md bg-neutral-800/80 hover:bg-neutral-700 text-neutral-300 transition-colors border border-neutral-700/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        <Github size={13} class="text-neutral-400" />
        <span class="font-mono text-[11px] truncate max-w-[90px]">
          {$activeGhAccount || 'No auth'}
        </span>
      </button>

      <!-- Watched Folders / Settings Button -->
      <button
        type="button"
        on:click={onOpenSettingsModal}
        title="Watched Folders & Account Settings"
        aria-label="Watched Folders & Account Settings"
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        <Settings2 size={14} />
      </button>

      <!-- Quick Actions Button -->
      <button
        type="button"
        on:click={onOpenQuickActionsModal}
        title="Quick Actions & Workspace Maintenance"
        aria-label="Quick Actions & Workspace Maintenance"
        class="p-1.5 rounded-md hover:bg-indigo-950/50 text-indigo-400 hover:text-indigo-300 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        <Zap size={14} class="fill-indigo-400/20" />
      </button>

      <!-- Pin Window Toggle -->
      <button
        type="button"
        on:click={onTogglePin}
        title={$isPinned ? 'Unpin window (keep normal window order)' : 'Pin window (always on top)'}
        aria-label={$isPinned ? 'Unpin window (keep normal window order)' : 'Pin window (always on top)'}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        {#if $isPinned}
          <Pin size={14} class="text-indigo-400" />
        {:else}
          <PinOff size={14} />
        {/if}
      </button>

      <!-- Refresh Scan -->
      <button
        type="button"
        on:click={onRefresh}
        disabled={isRefreshing}
        title={activeView === 'worktrees' ? 'Refresh Worktrees' : 'Refresh Branches'}
        aria-label={activeView === 'worktrees' ? 'Refresh Worktrees' : 'Refresh Branches'}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
      >
        <RefreshCw size={14} class={isRefreshing ? 'animate-spin text-indigo-400' : ''} />
      </button>
    </div>
  </div>

  <!-- Unified Search Filter Input -->
  <div class="relative flex items-center">
    <Search size={13} class="absolute left-2.5 text-neutral-400 pointer-events-none" />
    {#if activeView === 'worktrees'}
      <input
        bind:this={searchInputEl}
        type="text"
        bind:value={$searchFilter}
        placeholder="Filter worktree, branch, repo or @account... (/ to focus, ↑↓ Enter)"
        class="w-full bg-neutral-900 border border-neutral-800 rounded-md pl-8 pr-7 py-1 text-xs text-neutral-200 placeholder-neutral-500 focus:outline-none focus:border-indigo-500/70 focus:ring-1 focus:ring-indigo-500/30 transition-all font-sans"
      />
      {#if $searchFilter}
        <button
          type="button"
          on:click={() => {
            searchFilter.set('');
            searchInputEl?.focus();
          }}
          title="Clear search (Esc)"
          aria-label="Clear search"
          class="absolute right-2 p-0.5 rounded text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
        >
          <X size={12} />
        </button>
      {/if}
    {:else}
      <input
        bind:this={searchInputEl}
        type="text"
        bind:value={$branchSearchFilter}
        placeholder="Filter branch, commit, repo or @account... (/ to focus)"
        class="w-full bg-neutral-900 border border-neutral-800 rounded-md pl-8 pr-7 py-1 text-xs text-neutral-200 placeholder-neutral-500 focus:outline-none focus:border-indigo-500/70 focus:ring-1 focus:ring-indigo-500/30 transition-all font-sans"
      />
      {#if $branchSearchFilter}
        <button
          type="button"
          on:click={() => {
            branchSearchFilter.set('');
            searchInputEl?.focus();
          }}
          title="Clear search (Esc)"
          aria-label="Clear search"
          class="absolute right-2 p-0.5 rounded text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60"
        >
          <X size={12} />
        </button>
      {/if}
    {/if}
  </div>
</header>
