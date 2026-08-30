<script lang="ts">
  import type { WorktreeInfo, SupportedEditor, SupportedTerminal, WorktreeDiffSummary } from '../types';
  import {
    GitBranch,
    Folder,
    AlertTriangle,
    Trash2,
    Lock,
    Code2,
    Sparkles,
    Compass,
    Terminal,
    ChevronDown,
    Bot,
    Anchor,
    Flame
  } from 'lucide-svelte';
  import { installedEditors } from '../stores/editors';
  import { batchSelection, selectedPaths } from '../stores/batchSelection';
  import { appConfig } from '../stores/appConfig';
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

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
  }>();

  $: preferredEditor = ($appConfig.defaultEditor as SupportedEditor) || 'vscode';
  $: preferredTerminal = ($appConfig.defaultTerminal as SupportedTerminal) || 'wt';
  $: showTerminalBtn = ($appConfig.showTerminalButton !== false) && preferredTerminal !== 'none';

  let isHovered = false;
  let showDiffPopover = false;
  let diffSummary: WorktreeDiffSummary | null = null;
  let isLoadingDiff = false;

  $: isSelected = $selectedPaths.has(worktree.path);

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

  async function handleMouseEnterDirty() {
    if (!worktree.isDirty || diffSummary || isLoadingDiff) return;
    isLoadingDiff = true;
    try {
      const res = await invoke<WorktreeDiffSummary>('get_worktree_diff_summary', {
        worktreePath: worktree.path
      });
      diffSummary = res;
    } catch (err) {
      console.error('Failed to load worktree diff summary:', err);
    } finally {
      isLoadingDiff = false;
    }
  }
</script>

<div
  role="group"
  class="group relative flex items-center justify-between px-2.5 py-1.5 rounded-md border transition-all text-xs
    {isSelected
      ? 'bg-rose-950/30 border-rose-800/60 shadow-xs'
      : worktree.isMain
      ? 'bg-indigo-950/20 hover:bg-indigo-950/35 border-indigo-900/40 hover:border-indigo-800/60 shadow-xs'
      : 'bg-neutral-900/60 hover:bg-neutral-850/90 border-neutral-800/60 hover:border-neutral-700/80'}"
  on:mouseenter={() => (isHovered = true)}
  on:mouseleave={() => {
    isHovered = false;
    showDiffPopover = false;
  }}
>
  <!-- Left Side: Hierarchy Connector + Selection Checkbox + Branch / Name / Badges -->
  <div class="flex items-center gap-2 min-w-0 flex-1">
    <!-- Visual Tree Connector -->
    <div class="flex items-center text-neutral-600 select-none flex-shrink-0 font-mono text-[11px] w-4 text-center">
      {#if worktree.isMain}
        <span title="Main / Root Worktree" class="inline-flex">
          <Anchor size={12} class="text-indigo-400" />
        </span>
      {:else if isLast}
        <span class="text-neutral-500 font-bold">└─</span>
      {:else}
        <span class="text-neutral-500 font-bold">├─</span>
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
      <span title="Main / Root Worktree is protected from batch deletion" class="w-3.5 h-3.5 flex items-center justify-center text-neutral-600 flex-shrink-0">
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
      <ChevronDown size={10} class="text-neutral-500 group-hover/btn:text-neutral-300 transition-colors flex-shrink-0" />
    </button>

    <!-- Path directory name -->
    <span
      class="font-mono text-[11px] text-neutral-500 truncate max-w-[130px] hidden sm:inline-block"
      title={worktree.path}
    >
      {getDirectoryName(worktree.path)}
    </span>

    <!-- Status Badges -->
    <div class="flex items-center gap-1 flex-shrink-0">
      {#if worktree.isDirty}
        <div
          class="relative"
          role="group"
          on:mouseenter={() => {
            showDiffPopover = true;
            handleMouseEnterDirty();
          }}
          on:mouseleave={() => (showDiffPopover = false)}
        >
          <span
            class="px-1.5 py-0.5 rounded-full bg-rose-950/80 text-rose-300 border border-rose-800/60 text-[9px] font-sans flex items-center gap-0.5 cursor-pointer shadow-2xs"
          >
            <Flame size={9} class="text-rose-400 animate-pulse" />
            {worktree.uncommittedFilesCount ? `${worktree.uncommittedFilesCount} dirty` : 'dirty'}
          </span>

          <!-- Lazy Diff Popover -->
          {#if showDiffPopover}
            <div
              class="absolute left-0 bottom-full mb-1.5 z-40 w-64 p-2 rounded-lg bg-neutral-900 border border-neutral-700 shadow-2xl text-[11px] text-neutral-200 pointer-events-none"
            >
              <div class="font-semibold text-[10px] text-rose-400 mb-1 flex items-center justify-between border-b border-neutral-800 pb-1">
                <span>Uncommitted Changes</span>
                {#if isLoadingDiff}
                  <span class="text-neutral-500 animate-pulse text-[9px]">Analyzing...</span>
                {/if}
              </div>

              {#if diffSummary}
                <p class="text-neutral-300 font-mono text-[10px] mb-1.5">
                  {diffSummary.summaryText}
                </p>
                {#if diffSummary.modifiedFiles.length > 0}
                  <div class="max-h-28 overflow-y-auto space-y-0.5 font-mono text-[9px] text-neutral-400">
                    {#each diffSummary.modifiedFiles as file}
                      <div class="truncate text-neutral-300">{file}</div>
                    {/each}
                  </div>
                {/if}
              {:else if !isLoadingDiff}
                <span class="text-neutral-400 text-[10px]">Hovered to inspect git diff</span>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      {#if worktree.isOrphaned}
        <span
          class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 text-[9px] font-sans flex items-center gap-0.5"
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
      class="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-white border border-transparent hover:border-neutral-700 transition-colors flex items-center gap-1"
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
        title={preferredTerminal === 'agy'
          ? 'Open AGY CLI in worktree'
          : preferredTerminal === 'powershell'
          ? 'Open PowerShell in worktree'
          : preferredTerminal === 'cmd'
          ? 'Open Command Prompt in worktree'
          : preferredTerminal === 'git-bash'
          ? 'Open Git Bash in worktree'
          : 'Open Windows Terminal in worktree'}
        class="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 border border-transparent hover:border-neutral-700 transition-colors"
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
      class="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-amber-300 border border-transparent hover:border-neutral-700 transition-colors"
    >
      <Folder size={12} />
    </button>

    <!-- 1-Click Remove / Clean (hidden for main anchor) -->
    {#if !worktree.isMain}
      <button
        type="button"
        on:click={() => dispatch('requestDelete', worktree)}
        title="Remove worktree"
        class="p-1 rounded hover:bg-rose-950/80 text-neutral-500 hover:text-rose-400 border border-transparent hover:border-rose-900/50 transition-colors"
      >
        <Trash2 size={12} />
      </button>
    {/if}
  </div>
</div>
