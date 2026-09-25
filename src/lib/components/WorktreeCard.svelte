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
    GitMerge,
    GitPullRequestClosed
  } from 'lucide-svelte';
  import { installedEditors } from '../stores/editors';
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

  const dispatch = createEventDispatcher<{
    openPath: string;
    openEditor: { editor: SupportedEditor; path: string };
    openTerminal: { terminal: SupportedTerminal; path: string };
    requestSwitchBranch: { worktree: WorktreeInfo; repoPath: string };
    requestDelete: WorktreeInfo;
    worktreeDiscarded: WorktreeInfo;
  }>();

  let isEditorMenuOpen = false;
  let isDiscardPopoverOpen = false;
  let rootEl: HTMLDivElement;

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

  function checkAvailable(id: SupportedEditor): boolean {
    const found = $installedEditors.find(e => e.id === id);
    return found ? found.isAvailable : true;
  }

  function selectAndLaunchEditor(editor: SupportedEditor) {
    if (!checkAvailable(editor)) return;
    isEditorMenuOpen = false;
    dispatch('openEditor', { editor, path: worktree.path });
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

<svelte:window
  on:keydown={(e) => {
    // Consumed so the panel-level Escape does not also hide the window.
    if (e.key === 'Escape' && isEditorMenuOpen) {
      e.preventDefault();
      isEditorMenuOpen = false;
    }
  }}
  use:closeOnEscape={{ enabled: () => isDiscardPopoverOpen, onClose: closeDiscardPopover }}
/>

<div
  bind:this={rootEl}
  class="group relative rounded-lg border p-2.5 transition-all text-xs flex flex-col gap-1.5 shadow-sm
  {isHighlighted ? 'ring-2 ring-indigo-500/70' : ''}
  {isSelected
    ? 'bg-rose-950/30 border-rose-800/70 shadow-xs'
    : 'bg-neutral-900/90 hover:bg-neutral-850 border-neutral-800/80 hover:border-neutral-700'}">
  <div class="flex items-center justify-between">
    <!-- Branch name & interactive switcher button -->
    <div class="flex items-center gap-1.5 min-w-0">
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

      <button
        type="button"
        on:click={() => dispatch('requestSwitchBranch', { worktree, repoPath })}
        title="Click to switch or checkout branch in this worktree"
        class="flex items-center gap-1.5 px-1.5 py-0.5 rounded bg-neutral-800/80 hover:bg-indigo-950/80 border border-neutral-700/50 hover:border-indigo-700/60 text-neutral-200 hover:text-indigo-300 transition-all text-left group/branch max-w-[210px]"
      >
        <GitBranch size={12} class={worktree.isOrphaned ? 'text-amber-400' : 'text-indigo-400'} />
        <span class="font-mono font-medium truncate text-[11px]">
          {getShortBranch(worktree.branch)}
        </span>
        <ChevronDown size={10} class="text-neutral-400 group-hover/branch:text-indigo-400 transition-colors flex-shrink-0" />
      </button>

      {#if worktree.locked}
        <span title={`Locked: ${worktree.locked}`} class="text-neutral-400">
          <Lock size={11} />
        </span>
      {/if}
    </div>

    <!-- Status Badges -->
    <div class="flex items-center gap-1">
      {#if worktree.locked}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[11px] flex items-center gap-0.5 font-sans"
          title={`Locked: ${worktree.locked}`}
        >
          <Lock size={10} />
          Locked
        </span>
      {/if}

      <!-- Named reason first, generic "Orphan" only when neither branch flag explains it. -->
      {#if worktree.isBranchMerged}
        <span
          class="px-1.5 py-0.5 rounded bg-emerald-950/70 text-emerald-300 border border-emerald-800/50 text-[11px] flex items-center gap-0.5 font-sans"
          title={worktree.orphanReason || 'Branch merged into the default branch'}
        >
          <GitMerge size={10} />
          Merged
        </span>
      {/if}

      {#if worktree.isBranchRemoteGone}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[11px] flex items-center gap-0.5 font-sans"
          title={worktree.orphanReason || 'Upstream remote branch was deleted'}
        >
          <GitPullRequestClosed size={10} />
          Remote gone
        </span>
      {/if}

      {#if worktree.isOrphaned && !worktree.isBranchMerged && !worktree.isBranchRemoteGone}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[11px] flex items-center gap-0.5 font-sans"
          title={worktree.orphanReason || 'Orphaned worktree'}
        >
          <AlertTriangle size={10} />
          Orphan
        </span>
      {/if}

      {#if worktree.isDirty}
        <DirtyDiffPopover
          worktreePath={worktree.path}
          label={worktree.uncommittedFilesCount ? `${worktree.uncommittedFilesCount} uncommitted` : 'Dirty'}
          badgeClass="px-1.5 py-0.5 rounded bg-rose-950/70 text-rose-300 border border-rose-800/50 text-[11px] font-sans flex items-center gap-0.5 cursor-pointer"
          iconSize={10}
          uncommittedCount={worktree.uncommittedFilesCount ?? 0}
          popoverPositionClass="right-0 top-full mt-1.5"
        />
      {/if}

      {#if worktree.bare}
        <span class="px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-400 text-[11px]">
          Bare
        </span>
      {/if}
    </div>
  </div>

  <!-- Path and last commit info -->
  <div class="flex items-center justify-between text-neutral-400 text-[11px]">
    <div class="flex items-center gap-1 truncate max-w-[260px]" title={worktree.path}>
      <Folder size={11} class="text-neutral-400 flex-shrink-0" />
      <span class="font-mono truncate">{getDirectoryName(worktree.path)}</span>
    </div>

    <span class="font-mono text-[11px] text-neutral-400">
      {worktree.head.substring(0, 7)}
    </span>
  </div>

  {#if worktree.lastCommitMessage}
    <p class="text-[11px] text-neutral-400 truncate italic font-sans" title={worktree.lastCommitMessage}>
      "{worktree.lastCommitMessage}"
    </p>
  {/if}

  <!-- Action buttons -->
  <div class="flex items-center justify-end gap-1.5 pt-1 border-t border-neutral-800/50 opacity-85 group-hover:opacity-100 transition-opacity">
    <!-- IDE Launcher Split Button -->
    <div class="relative flex items-center rounded bg-neutral-800/80 border border-neutral-700/60 shadow-xs">
      <button
        type="button"
        on:click={() => dispatch('openEditor', { editor: preferredEditor, path: worktree.path })}
        title={`Open in ${preferredEditor.toUpperCase()}`}
        class="px-1.5 py-1 text-neutral-300 hover:text-white hover:bg-neutral-700 rounded-l flex items-center gap-1 transition-colors text-[11px]"
      >
        {#if preferredEditor === 'antigravity'}
          <Bot size={12} class="text-indigo-400" />
        {:else if preferredEditor === 'vscode'}
          <Code2 size={12} class="text-blue-400" />
        {:else if preferredEditor === 'cursor'}
          <Sparkles size={12} class="text-purple-400" />
        {:else if preferredEditor === 'windsurf'}
          <Compass size={12} class="text-emerald-400" />
        {:else if preferredEditor === 'wt'}
          <Terminal size={12} class="text-neutral-300" />
        {:else}
          <Folder size={12} class="text-amber-400" />
        {/if}
        <span class="capitalize font-mono text-[11px]">{preferredEditor === 'antigravity' ? 'Antigravity' : preferredEditor}</span>
      </button>

      <button
        type="button"
        on:click={() => (isEditorMenuOpen = !isEditorMenuOpen)}
        title="Select editor or terminal"
        aria-label="Select editor or terminal"
        aria-haspopup="menu"
        aria-expanded={isEditorMenuOpen}
        class="min-w-6 min-h-6 flex items-center justify-center px-1 py-1 text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 rounded-r border-l border-neutral-700/60 transition-colors"
      >
        <ChevronDown size={10} />
      </button>

      {#if isEditorMenuOpen}
        <!-- Backdrop to close dropdown -->
        <div
          class="fixed inset-0 z-20"
          on:click={() => (isEditorMenuOpen = false)}
          role="presentation"
        ></div>

        <!-- Dropdown Menu -->
        <div role="menu" class="absolute right-0 top-7 z-30 w-44 rounded-lg bg-neutral-900 border border-neutral-700 shadow-xl p-1 flex flex-col gap-0.5">
          {#each $installedEditors as ed (ed.id)}
            <button
              type="button"
              disabled={!ed.isAvailable}
              on:click={() => selectAndLaunchEditor(ed.id)}
              class="flex items-center justify-between px-2 py-1 text-[11px] rounded transition-colors text-left
                {ed.isAvailable ? 'text-neutral-300 hover:text-white hover:bg-neutral-800 cursor-pointer' : 'text-neutral-400 opacity-40 cursor-not-allowed'}"
            >
              <div class="flex items-center gap-2">
                {#if ed.id === 'antigravity'}
                  <Bot size={12} class={ed.isAvailable ? 'text-indigo-400' : 'text-neutral-400'} />
                {:else if ed.id === 'vscode'}
                  <Code2 size={12} class={ed.isAvailable ? 'text-blue-400' : 'text-neutral-400'} />
                {:else if ed.id === 'cursor'}
                  <Sparkles size={12} class={ed.isAvailable ? 'text-purple-400' : 'text-neutral-400'} />
                {:else if ed.id === 'windsurf'}
                  <Compass size={12} class={ed.isAvailable ? 'text-emerald-400' : 'text-neutral-400'} />
                {:else if ed.id === 'wt'}
                  <Terminal size={12} class={ed.isAvailable ? 'text-neutral-300' : 'text-neutral-400'} />
                {:else}
                  <Folder size={12} class={ed.isAvailable ? 'text-amber-400' : 'text-neutral-400'} />
                {/if}
                <span>{ed.name}</span>
              </div>
              {#if !ed.isAvailable}
                <span class="text-[11px] text-neutral-400 font-sans">Not installed</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

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
        class="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-transparent hover:border-neutral-700 transition-colors"
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
      class="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 hover:text-amber-300 border border-transparent hover:border-neutral-700 transition-colors"
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
          class="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 hover:text-rose-300 border border-transparent hover:border-neutral-700 transition-colors"
        >
          <Eraser size={12} />
        </button>

        {#if isDiscardPopoverOpen}
          <div class="fixed inset-0 z-20" on:click={closeDiscardPopover} role="presentation"></div>
          <div use:autofocus class="absolute right-0 bottom-full mb-1.5 z-30 w-64">
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

    <!-- Delete Worktree button. Deliberately larger than its siblings and behind a persistent
         divider so the destructive action never reads as part of the safe launcher cluster. -->
    {#if !worktree.isMain}
      <button
        type="button"
        on:click={() => dispatch('requestDelete', worktree)}
        title="Remove worktree"
        aria-label="Remove worktree"
        class="p-2 pl-2.5 ml-1 border-l border-neutral-800/60 rounded hover:bg-rose-950/60 text-neutral-400 hover:text-rose-400 transition-colors"
      >
        <Trash2 size={14} />
      </button>
    {/if}
  </div>
</div>
