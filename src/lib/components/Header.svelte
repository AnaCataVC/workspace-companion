<script lang="ts">
  import { isPinned, searchFilter } from '../stores/worktrees';
  import { activeGhAccount } from '../stores/ghAuth';
  import { viewDensity } from '../stores/appConfig';
  import { RefreshCw, Pin, PinOff, Github, Search, Plus, Settings2, LayoutList, LayoutGrid, GitBranch, FolderTree } from 'lucide-svelte';

  export let activeView: 'worktrees' | 'branches' = 'worktrees';
  export let isRefreshing: boolean = false;
  export let onRefresh: () => void;
  export let onSetView: (view: 'worktrees' | 'branches') => void;
  export let onOpenGhModal: () => void;
  export let onOpenNewWorktreeModal: () => void;
  export let onOpenSettingsModal: () => void;

  function togglePin() {
    isPinned.update(p => !p);
  }

  function toggleDensity() {
    viewDensity.update(d => (d === 'compact' ? 'detailed' : 'compact'));
  }
</script>

<header class="p-3 bg-neutral-950/80 backdrop-blur border-b border-neutral-800 flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <!-- Left Navigation: Segmented View Switcher and Quick Action -->
    <div class="flex items-center gap-2">
      <!-- Segmented view switch: each half is its own target, so the label always names the view
           it takes you to instead of flipping meaning with the current state. -->
      <div class="flex items-center rounded-md bg-neutral-900 border border-neutral-800 p-0.5 gap-0.5">
        <button
          type="button"
          on:click={() => onSetView('worktrees')}
          title="Show worktrees"
          aria-pressed={activeView === 'worktrees'}
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded transition-colors
            {activeView === 'worktrees'
              ? 'bg-indigo-600 text-white shadow-xs'
              : 'text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'}"
        >
          <FolderTree size={13} />
          <span>Worktrees</span>
        </button>
        <button
          type="button"
          on:click={() => onSetView('branches')}
          title="Show branches"
          aria-pressed={activeView === 'branches'}
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded transition-colors
            {activeView === 'branches'
              ? 'bg-indigo-600 text-white shadow-xs'
              : 'text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'}"
        >
          <GitBranch size={13} />
          <span>Branches</span>
        </button>
      </div>

      {#if activeView === 'worktrees'}
        <!-- New Worktree Button -->
        <button
          on:click={onOpenNewWorktreeModal}
          title="Create new Git worktree"
          class="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-md bg-indigo-600 hover:bg-indigo-500 text-white transition-colors shadow-xs"
        >
          <Plus size={13} />
          <span>New</span>
        </button>
      {/if}
    </div>

    <!-- Right Controls: View Density & Global Utility Buttons -->
    <div class="flex items-center gap-1.5">
      <!-- View Density Toggle -->
      <button
        type="button"
        on:click={toggleDensity}
        title={$viewDensity === 'compact' ? "Switch to Detailed Cards view" : "Switch to Compact Tree view"}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-indigo-300 transition-colors"
      >
        {#if $viewDensity === 'compact'}
          <LayoutList size={14} class="text-indigo-400" />
        {:else}
          <LayoutGrid size={14} class="text-neutral-400" />
        {/if}
      </button>

      <!-- GitHub Account Badge Button -->
      <button
        on:click={onOpenGhModal}
        title="Switch GitHub CLI account"
        class="flex items-center gap-1.5 px-2 py-1 text-xs rounded-md bg-neutral-800/80 hover:bg-neutral-700 text-neutral-300 transition-colors border border-neutral-700/50"
      >
        <Github size={13} class="text-neutral-400" />
        <span class="font-mono text-[11px] truncate max-w-[90px]">
          {$activeGhAccount || 'No auth'}
        </span>
      </button>

      <!-- Watched Folders / Settings Button -->
      <button
        on:click={onOpenSettingsModal}
        title="Watched Folders & Account Settings"
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors"
      >
        <Settings2 size={14} />
      </button>

      <!-- Pin Window Toggle -->
      <button
        on:click={togglePin}
        title={$isPinned ? "Unpin window (auto-hide on blur)" : "Pin window (keep visible)"}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors"
      >
        {#if $isPinned}
          <Pin size={14} class="text-indigo-400" />
        {:else}
          <PinOff size={14} />
        {/if}
      </button>

      <!-- Refresh Scan -->
      <button
        on:click={onRefresh}
        disabled={isRefreshing}
        title={activeView === 'worktrees' ? 'Refresh Worktrees' : 'Refresh Branches'}
        class="p-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors disabled:opacity-50"
      >
        <RefreshCw size={14} class={isRefreshing ? 'animate-spin text-indigo-400' : ''} />
      </button>
    </div>
  </div>

  {#if activeView === 'worktrees'}
    <!-- Search Filter Input -->
    <div class="relative flex items-center">
      <Search size={13} class="absolute left-2.5 text-neutral-500 pointer-events-none" />
      <input
        type="text"
        bind:value={$searchFilter}
        placeholder="Filter worktree, branch or path..."
        class="w-full bg-neutral-900 border border-neutral-800 rounded-md pl-8 pr-3 py-1 text-xs text-neutral-200 placeholder-neutral-500 focus:outline-none focus:border-indigo-500/70 focus:ring-1 focus:ring-indigo-500/30 transition-all font-sans"
      />
    </div>
  {/if}
</header>
