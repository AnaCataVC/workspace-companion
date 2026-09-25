<script lang="ts">
  import type { WorktreeInfo, SupportedEditor, SupportedTerminal } from '../types';
  import {
    GitBranch,
    Folder,
    AlertTriangle,
    Trash2,
    Eraser,
    Lock,
    Code2,
    Sparkles,
    Compass,
    Terminal,
    ChevronDown,
    Bot,
    Anchor,
    GitMerge,
    GitPullRequestClosed
  } from 'lucide-svelte';
  import { batchSelection, selectedPaths } from '../stores/batchSelection';
  import { highlightedWorktreePath, openWorktreeActionCount } from '../stores/worktrees';
  import { appConfig } from '../stores/appConfig';
  import { createEventDispatcher, onDestroy } from 'svelte';
  import DirtyDiffPopover from './DirtyDiffPopover.svelte';
  import DiscardChangesPanel from './DiscardChangesPanel.svelte';
  import { closeOnEscape } from '../actions/closeOnEscape';
  import { autofocus } from '../actions/autofocus';
  import { worktreeProtectionReason } from '../utils/protectionReason';

  export let worktree: WorktreeInfo;
  export let repoPath: string = '';
  export let repoName: string = '';
  export let isLast: boolean = false;

  const dispatch = createEventDispatcher<{
    openPath: string;
    openEditor: { editor: SupportedEditor; path: string };
    openTerminal: { terminal: SupportedTerminal; path: string };
    requestSwitchBranch: { worktree: WorktreeInfo; repoPath: string };
    requestDelete: WorktreeInfo;
    worktreeDiscarded: WorktreeInfo;
  }>();

  let isDiscardPopoverOpen = false;

  function openDiscardPopover() {
    isDiscardPopoverOpen = true;
    openWorktreeActionCount.update((n) => n + 1);
  }

  function closeDiscardPopover() {
    if (!isDiscardPopoverOpen) return;
    isDiscardPopoverOpen = false;
    openWorktreeActionCount.update((n) => Math.max(0, n - 1));
  }

  function handleDiscarded(updatedWt: WorktreeInfo) {
    closeDiscardPopover();
    dispatch('worktreeDiscarded', updatedWt);
  }

  onDestroy(() => {
    if (isDiscardPopoverOpen) openWorktreeActionCount.update((n) => Math.max(0, n - 1));
  });

  $: preferredEditor = ($appConfig.defaultEditor as SupportedEditor) || 'vscode';
  $: preferredTerminal = ($appConfig.defaultTerminal as SupportedTerminal) || 'wt';
  $: showTerminalBtn = ($appConfig.showTerminalButton !== false) && preferredTerminal !== 'none';

  let isHovered = false;
  let rootEl: HTMLDivElement;

  let highlightTimer: ReturnType<typeof setTimeout> | null = null;
  $: isSelected = $selectedPaths.has(worktree.path);
  $: isHighlighted = $highlightedWorktreePath === worktree.path;
  $: if (isHighlighted && rootEl) {
    rootEl.scrollIntoView({ block: 'center', behavior: 'smooth' });
    if (highlightTimer) clearTimeout(highlightTimer);
    highlightTimer = setTimeout(() => {
      highlightedWorktreePath.set(null);
    }, 2500);
  }

  function toggleSelection() {
    if (worktree.isMain) return;
    batchSelection.toggle({
      repoPath,
      repoName: repoName || getDirectoryName(repoPath),
      worktreePath: worktree.path,
      force: false,
      branch: worktree.branch,
      isDirty: worktree.isDirty,
      uncommittedFilesCount: worktree.uncommittedFilesCount
    });
  }

  function getShortBranch(fullBranch: string | null): string {
    if (!fullBranch) return '(detached HEAD)';
    return fullBranch.replace('refs/heads/', '');
  }

  function getDirectoryName(fullPath: string): string {
    const parts = fullPath.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || fullPath;
  }

</script>

<svelte:window use:closeOnEscape={{ enabled: () => isDiscardPopoverOpen, onClose: closeDiscardPopover }} />

<div
  bind:this={rootEl}
  role="group"
  class="group relative flex items-center justify-between px-2.5 py-1.5 rounded-md border transition-all text-xs
    {isHighlighted ? 'ring-2 ring-indigo-500/70' : ''}
    {isSelected
      ? 'bg-rose-950/30 border-rose-800/60 shadow-xs'
      : worktree.isMain
      ? 'bg-indigo-950/20 hover:bg-indigo-950/35 border-indigo-900/40 hover:border-indigo-800/60 shadow-xs'
      : 'bg-neutral-900/60 hover:bg-neutral-850/90 border-neutral-800/60 hover:border-neutral-700/80'}"
  on:mouseenter={() => (isHovered = true)}
  on:mouseleave={() => (isHovered = false)}
>
  <!-- Left Side: Hierarchy Connector + Selection Checkbox + Branch / Name / Badges -->
  <div class="flex items-center gap-2 min-w-0 flex-1">
    <!-- Visual Tree Connector -->
    <div class="flex items-center text-neutral-400 select-none flex-shrink-0 font-mono text-[11px] w-4 text-center">
      {#if worktree.isMain}
        <span title="Main / Root Worktree" class="inline-flex">
          <Anchor size={12} class="text-indigo-400" />
        </span>
      {:else if isLast}
        <span class="text-neutral-400 font-bold">└─</span>
      {:else}
        <span class="text-neutral-400 font-bold">├─</span>
      {/if}
    </div>

    <!-- Batch Selection Checkbox (disabled for main) -->
    {#if !worktree.isMain}
      <input
        type="checkbox"
        checked={isSelected}
        on:change={toggleSelection}
        title="Select worktree for batch delete"
        class="w-3.5 h-3.5 rounded border-neutral-700 bg-neutral-950 text-rose-500 focus:ring-rose-500/30 focus:ring-offset-0 cursor-pointer flex-shrink-0"
      />
    {:else}
      <span title={worktreeProtectionReason(worktree)} class="w-3.5 h-3.5 flex items-center justify-center text-neutral-400 flex-shrink-0">
        <Lock size={10} />
      </span>
    {/if}

    <!-- Branch Button (Interactive Switcher) -->
    <button
      type="button"
      on:click={() => dispatch('requestSwitchBranch', { worktree, repoPath })}
      title="Click to switch or checkout branch in this worktree"
      class="flex items-center gap-1.5 px-2 py-0.5 rounded transition-all text-left group/btn truncate max-w-[210px]
        {worktree.isMain
          ? 'bg-indigo-900/40 hover:bg-indigo-800/60 text-indigo-200 border border-indigo-700/40'
          : 'bg-neutral-800/80 hover:bg-neutral-700/80 text-neutral-200 border border-neutral-700/40'}"
    >
      <GitBranch
        size={11}
        class={worktree.isOrphaned ? 'text-amber-400' : worktree.isMain ? 'text-indigo-400' : 'text-emerald-400'}
      />
      <span class="font-mono font-medium truncate text-[11px]">
        {getShortBranch(worktree.branch)}
      </span>
      <ChevronDown size={10} class="text-neutral-400 group-hover/btn:text-neutral-300 transition-colors flex-shrink-0" />
    </button>

    <!-- Path directory name -->
    <span
      class="font-mono text-[11px] text-neutral-400 truncate max-w-[130px] hidden sm:inline-block"
      title={worktree.path}
    >
      {getDirectoryName(worktree.path)}
    </span>

    <!-- Status Badges -->
    <div class="flex items-center gap-1 flex-shrink-0">
      {#if worktree.locked}
        <span
          class="px-1.5 py-0.5 rounded-full bg-amber-950/80 text-amber-300 border border-amber-800/60 text-[11px] font-sans flex items-center gap-0.5"
          title={`Locked: ${worktree.locked}`}
        >
          <Lock size={9} />
          locked
        </span>
      {/if}

      {#if worktree.isDirty}
        <DirtyDiffPopover
          uncommittedCount={worktree.uncommittedFilesCount ?? 0}
          worktreePath={worktree.path}
          label={worktree.uncommittedFilesCount ? `${worktree.uncommittedFilesCount} dirty` : 'dirty'}
          badgeClass="px-1.5 py-0.5 rounded-full bg-rose-950/80 text-rose-300 border border-rose-800/60 text-[11px] font-sans flex items-center gap-0.5 cursor-pointer shadow-2xs"
          iconSize={9}
          popoverPositionClass="left-0 bottom-full mb-1.5"
        />
      {/if}

      <!-- Named reason first, generic "Orphan" only when neither branch flag explains it. -->
      {#if worktree.isBranchMerged}
        <span
          class="px-1.5 py-0.5 rounded bg-emerald-950/70 text-emerald-300 border border-emerald-800/50 text-[11px] font-sans flex items-center gap-0.5"
          title={worktree.orphanReason || 'Branch merged into the default branch'}
        >
          <GitMerge size={9} />
          Merged
        </span>
      {/if}

      {#if worktree.isBranchRemoteGone}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[11px] font-sans flex items-center gap-0.5"
          title={worktree.orphanReason || 'Upstream remote branch was deleted'}
        >
          <GitPullRequestClosed size={9} />
          Remote gone
        </span>
      {/if}

      {#if worktree.isOrphaned && !worktree.isBranchMerged && !worktree.isBranchRemoteGone}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[11px] font-sans flex items-center gap-0.5"
          title={worktree.orphanReason || 'Orphaned worktree'}
        >
          <AlertTriangle size={9} />
          Orphan
        </span>
      {/if}

      {#if worktree.locked}
        <span title={`Locked: ${worktree.locked}`} class="text-neutral-400">
          <Lock size={10} />
        </span>
      {/if}
    </div>
  </div>

  <!-- Right Side: Mini-Dock 1-Click Action Buttons -->
  <div class="flex items-center gap-1 flex-shrink-0 ml-2">
    <!-- 1-Click Launch Editor -->
    <button
      type="button"
      on:click={() => dispatch('openEditor', { editor: preferredEditor, path: worktree.path })}
      title={`Open in ${preferredEditor === 'antigravity' ? 'Antigravity IDE' : preferredEditor.toUpperCase()}`}
      class="min-w-6 min-h-6 flex items-center justify-center p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-white border border-transparent hover:border-neutral-700 transition-colors gap-1"
    >
      {#if preferredEditor === 'antigravity'}
        <Bot size={12} class="text-indigo-400" />
      {:else if preferredEditor === 'vscode'}
        <Code2 size={12} class="text-blue-400" />
      {:else if preferredEditor === 'cursor'}
        <Sparkles size={12} class="text-purple-400" />
      {:else if preferredEditor === 'windsurf'}
        <Compass size={12} class="text-emerald-400" />
      {:else}
        <Folder size={12} class="text-amber-400" />
      {/if}
    </button>

    <!-- 1-Click Launch Terminal / CLI (configurable & toggleable) -->
    {#if showTerminalBtn}
      <button
        type="button"
        on:click={() => dispatch('openTerminal', { terminal: preferredTerminal, path: worktree.path })}
        aria-label="Open terminal in worktree"
        title={preferredTerminal === 'agy'
          ? 'Open AGY CLI in worktree'
          : preferredTerminal === 'powershell'
          ? 'Open PowerShell in worktree'
          : preferredTerminal === 'cmd'
          ? 'Open Command Prompt in worktree'
          : preferredTerminal === 'git-bash'
          ? 'Open Git Bash in worktree'
          : 'Open Windows Terminal in worktree'}
        class="min-w-6 min-h-6 flex items-center justify-center p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-transparent hover:border-neutral-700 transition-colors"
      >
        {#if preferredTerminal === 'agy'}
          <Bot size={12} class="text-indigo-400" />
        {:else}
          <Terminal size={12} />
        {/if}
      </button>
    {/if}

    <!-- 1-Click Open File Explorer -->
    <button
      type="button"
      on:click={() => dispatch('openPath', worktree.path)}
      title="Open in File Explorer"
      aria-label="Open in File Explorer"
      class="min-w-6 min-h-6 flex items-center justify-center p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-amber-300 border border-transparent hover:border-neutral-700 transition-colors"
    >
      <Folder size={12} />
    </button>

    <!-- Discard uncommitted changes (dirty worktrees only) -->
    {#if worktree.isDirty}
      <div class="relative">
        <button
          type="button"
          on:click={openDiscardPopover}
          title="Discard uncommitted changes"
          aria-label="Discard uncommitted changes"
          aria-haspopup="dialog"
          aria-expanded={isDiscardPopoverOpen}
          class="min-w-6 min-h-6 flex items-center justify-center p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-rose-300 border border-transparent hover:border-neutral-700 transition-colors"
        >
          <Eraser size={12} />
        </button>

        {#if isDiscardPopoverOpen}
          <div class="fixed inset-0 z-20" on:click={closeDiscardPopover} role="presentation"></div>
          <div use:autofocus class="absolute right-0 top-full mt-1.5 z-30 w-64">
            <DiscardChangesPanel
              worktreePath={worktree.path}
              uncommittedFilesCount={worktree.uncommittedFilesCount ?? 0}
              ondiscarded={handleDiscarded}
              oncancel={closeDiscardPopover}
            />
          </div>
        {/if}
      </div>
    {/if}

    <!-- 1-Click Remove / Clean (hidden for main anchor). Deliberately larger than its siblings
         and behind a persistent divider so the destructive action never reads as part of the
         safe launcher cluster, even when the row isn't hovered. -->
    {#if !worktree.isMain}
      <button
        type="button"
        on:click={() => dispatch('requestDelete', worktree)}
        title="Remove worktree"
        aria-label="Remove worktree"
        class="p-1.5 pl-2 ml-1 border-l border-neutral-800/60 rounded hover:bg-rose-950/80 text-neutral-400 hover:text-rose-400 transition-colors"
      >
        <Trash2 size={14} />
      </button>
    {/if}
  </div>
</div>
