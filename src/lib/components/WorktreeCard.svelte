<script lang="ts">
  import type { WorktreeInfo } from '../types';
  import { GitBranch, Folder, AlertTriangle, Trash2, ExternalLink, Lock } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let worktree: WorktreeInfo;
  export let repoPath: string = '';

  const dispatch = createEventDispatcher<{
    openPath: string;
    requestDelete: WorktreeInfo;
  }>();

  function getShortBranch(fullBranch: string | null): string {
    if (!fullBranch) return '(detached HEAD)';
    return fullBranch.replace('refs/heads/', '');
  }

  function getDirectoryName(fullPath: string): string {
    const parts = fullPath.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || fullPath;
  }
</script>

<div class="group relative rounded-lg bg-neutral-900/90 hover:bg-neutral-850 border border-neutral-800/80 hover:border-neutral-700 p-2.5 transition-all text-xs flex flex-col gap-1.5 shadow-sm">
  <div class="flex items-center justify-between">
    <!-- Branch name & icon -->
    <div class="flex items-center gap-1.5 min-w-0">
      <GitBranch size={13} class={worktree.isOrphaned ? 'text-amber-400' : 'text-indigo-400'} />
      <span class="font-mono font-medium text-neutral-200 truncate text-[11px]" title={worktree.branch || ''}>
        {getShortBranch(worktree.branch)}
      </span>
      {#if worktree.locked}
        <span title={`Locked: ${worktree.locked}`} class="text-neutral-400">
          <Lock size={11} />
        </span>
      {/if}
    </div>

    <!-- Status Badges -->
    <div class="flex items-center gap-1">
      {#if worktree.isOrphaned}
        <span class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[10px] flex items-center gap-0.5 font-sans">
          <AlertTriangle size={10} />
          Orphan
        </span>
      {/if}

      {#if worktree.isDirty}
        <span class="px-1.5 py-0.5 rounded bg-rose-950/70 text-rose-300 border border-rose-800/50 text-[10px] font-sans">
          {worktree.uncommittedFilesCount ? `${worktree.uncommittedFilesCount} uncommitted` : 'Dirty'}
        </span>
      {/if}

      {#if worktree.bare}
        <span class="px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-400 text-[10px]">
          Bare
        </span>
      {/if}
    </div>
  </div>

  <!-- Path and last commit info -->
  <div class="flex items-center justify-between text-neutral-400 text-[11px]">
    <div class="flex items-center gap-1 truncate max-w-[260px]" title={worktree.path}>
      <Folder size={11} class="text-neutral-500 flex-shrink-0" />
      <span class="font-mono truncate">{getDirectoryName(worktree.path)}</span>
    </div>

    <span class="font-mono text-[10px] text-neutral-500">
      {worktree.head.substring(0, 7)}
    </span>
  </div>

  {#if worktree.lastCommitMessage}
    <p class="text-[11px] text-neutral-400 truncate italic font-sans" title={worktree.lastCommitMessage}>
      "{worktree.lastCommitMessage}"
    </p>
  {/if}

  <!-- Action buttons -->
  <div class="flex items-center justify-end gap-1 pt-1 border-t border-neutral-800/50 opacity-80 group-hover:opacity-100 transition-opacity">
    <button
      on:click={() => dispatch('openPath', worktree.path)}
      title="Open folder in File Explorer / Editor"
      class="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors"
    >
      <ExternalLink size={12} />
    </button>

    <button
      on:click={() => dispatch('requestDelete', worktree)}
      title="Remove worktree"
      class="p-1 rounded hover:bg-rose-950/50 text-neutral-400 hover:text-rose-400 transition-colors"
    >
      <Trash2 size={12} />
    </button>
  </div>
</div>
