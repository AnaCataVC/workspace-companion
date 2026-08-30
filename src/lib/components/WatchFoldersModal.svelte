<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { AppConfig, WatchFolder, GhAccount, SupportedEditor, SupportedTerminal } from '../types';
  import { installedEditors, installedTerminals } from '../stores/editors';
  import {
    FolderSync,
    Plus,
    Trash2,
    X,
    Github,
    Folder,
    Layers,
    Save,
    Settings2
  } from 'lucide-svelte';

  export let isOpen: boolean = false;
  export let config: AppConfig;
  export let ghAccounts: GhAccount[] = [];
  export let isSaving: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    save: AppConfig;
  }>();

  // Local working copy of config
  let localWatchFolders: WatchFolder[] = [];
  let localAutoSwitch: boolean = true;
  let localDefaultEditor: SupportedEditor = 'vscode';
  let localDefaultTerminal: SupportedTerminal = 'wt';
  let localShowTerminalButton: boolean = true;

  $: if (isOpen && config) {
    localWatchFolders = JSON.parse(JSON.stringify(config.watchFolders || []));
    localAutoSwitch = config.autoSwitchAccount ?? true;
    localDefaultEditor = config.defaultEditor || 'vscode';
    localDefaultTerminal = config.defaultTerminal || 'wt';
    localShowTerminalButton = config.showTerminalButton ?? true;
  }

  function addFolder() {
    const newId = `watch-${Date.now()}`;
    localWatchFolders = [
      ...localWatchFolders,
      {
        id: newId,
        path: '',
        accountUsername: ghAccounts.length > 0 ? ghAccounts[0].username : null,
        enabled: true,
        maxDepth: 2
      }
    ];
  }

  function removeFolder(index: number) {
    localWatchFolders = localWatchFolders.filter((_, i) => i !== index);
  }

  function handleSave() {
    const updatedConfig: AppConfig = {
      version: 1,
      watchFolders: localWatchFolders.filter(f => f.path.trim().length > 0),
      autoSwitchAccount: localAutoSwitch,
      defaultEditor: localDefaultEditor,
      defaultTerminal: localDefaultTerminal,
      showTerminalButton: localShowTerminalButton
    };
    dispatch('save', updatedConfig);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      dispatch('close');
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-xs p-4 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-xl rounded-xl bg-neutral-900 border border-neutral-800 shadow-2xl p-4 flex flex-col gap-3.5 text-neutral-200 max-h-[90vh]"
    >
      <!-- Modal Header -->
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-2">
          <div class="p-1.5 rounded-lg bg-indigo-950/80 text-indigo-400 border border-indigo-800/40">
            <Settings2 size={16} />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-neutral-100">Watched Repositories & Account Mappings</h3>
            <p class="text-[11px] text-neutral-400">
              Configure directories to scan and assign their GitHub CLI accounts
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => dispatch('close')}
          disabled={isSaving}
          class="text-neutral-500 hover:text-neutral-300 p-1 rounded-md hover:bg-neutral-800 transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      <!-- Watched Folders List -->
      <div class="flex-1 overflow-y-auto min-h-[180px] max-h-[340px] space-y-2.5 pr-1">
        {#if localWatchFolders.length === 0}
          <div class="h-32 flex flex-col items-center justify-center text-neutral-500 text-xs text-center border border-dashed border-neutral-800 rounded-lg p-4 gap-2">
            <FolderSync size={24} class="text-neutral-600" />
            <p>No watched folders configured yet.</p>
            <button
              type="button"
              on:click={addFolder}
              class="px-2.5 py-1 rounded bg-indigo-950/80 text-indigo-300 border border-indigo-800/50 hover:bg-indigo-900 text-[11px] transition-colors flex items-center gap-1"
            >
              <Plus size={12} /> Add First Folder
            </button>
          </div>
        {:else}
          {#each localWatchFolders as folder, idx (folder.id)}
            <div class="rounded-lg bg-neutral-950/80 border border-neutral-800/80 p-2.5 space-y-2 text-xs">
              <!-- Row 1: Path & Enable & Delete -->
              <div class="flex items-center gap-2">
                <input
                  type="checkbox"
                  bind:checked={folder.enabled}
                  title="Enable or disable scanning for this folder"
                  class="rounded bg-neutral-900 border-neutral-700 text-indigo-600 focus:ring-0 focus:outline-none"
                />

                <div class="relative flex-1">
                  <Folder size={12} class="absolute left-2.5 top-2.5 text-neutral-500" />
                  <input
                    type="text"
                    bind:value={folder.path}
                    placeholder="e.g. C:\Users\username\Repos or ~/Projects"
                    class="w-full bg-neutral-900 border border-neutral-800 rounded-md pl-7 pr-2.5 py-1 text-xs text-neutral-200 placeholder-neutral-600 font-mono focus:outline-hidden focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                  />
                </div>

                <button
                  type="button"
                  on:click={() => removeFolder(idx)}
                  title="Remove folder"
                  class="p-1 rounded text-neutral-500 hover:text-rose-400 hover:bg-rose-950/40 transition-colors"
                >
                  <Trash2 size={13} />
                </button>
              </div>

              <!-- Row 2: Account Mapping & Scan Depth -->
              <div class="flex items-center justify-between gap-3 text-[11px] pl-6">
                <!-- Account Selector -->
                <div class="flex items-center gap-1.5 flex-1">
                  <Github size={12} class="text-neutral-500 flex-shrink-0" />
                  <span class="text-neutral-400 flex-shrink-0">Account:</span>
                  <select
                    bind:value={folder.accountUsername}
                    class="bg-neutral-900 border border-neutral-800 rounded px-2 py-0.5 text-[11px] text-neutral-200 font-mono focus:outline-hidden focus:border-indigo-500"
                  >
                    <option value={null}>None / Unassigned</option>
                    {#each ghAccounts as acc (acc.username)}
                      <option value={acc.username}>@{acc.username}</option>
                    {/each}
                  </select>
                </div>

                <!-- Scan Depth Pills -->
                <div class="flex items-center gap-1.5 flex-shrink-0">
                  <Layers size={12} class="text-neutral-500 flex-shrink-0" />
                  <span class="text-neutral-400 flex-shrink-0">Depth:</span>
                  <div class="flex rounded bg-neutral-900 border border-neutral-800 p-0.5">
                    {#each [1, 2, 3, 4, 5] as depth}
                      <button
                        type="button"
                        on:click={() => (folder.maxDepth = depth)}
                        class="px-1.5 py-0.2 text-[10px] font-mono rounded transition-all
                          {folder.maxDepth === depth
                            ? 'bg-indigo-600 text-white font-medium shadow-2xs'
                            : 'text-neutral-400 hover:text-neutral-200'}"
                        title={`Scan up to ${depth} folder level(s) deep`}
                      >
                        {depth}
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            </div>
          {/each}

          <button
            type="button"
            on:click={addFolder}
            class="w-full py-1.5 rounded-lg border border-dashed border-neutral-800 hover:border-neutral-700 bg-neutral-950/40 hover:bg-neutral-900 text-neutral-400 hover:text-neutral-200 text-xs transition-all flex items-center justify-center gap-1"
          >
            <Plus size={12} /> Add Another Folder
          </button>
        {/if}
      </div>

      <!-- Global Settings & Preferences -->
      <div class="pt-2 border-t border-neutral-800 space-y-2.5 text-xs">
        <!-- Default Editor Selector -->
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-[11px] font-medium text-neutral-300">Default Editor / IDE</span>
            <span class="text-[10px] text-neutral-500">The primary IDE launched when clicking worktree action buttons</span>
          </div>
          <select
            bind:value={localDefaultEditor}
            class="bg-neutral-950 border border-neutral-800 rounded px-2 py-1 text-xs text-neutral-200 focus:outline-hidden focus:border-indigo-500"
          >
            {#each $installedEditors as editor (editor.id)}
              <option value={editor.id} disabled={!editor.isAvailable}>
                {editor.name} {!editor.isAvailable ? '(Not installed)' : ''}
              </option>
            {/each}
          </select>
        </div>

        <!-- Default Terminal / CLI Selector -->
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-[11px] font-medium text-neutral-300">Default Terminal / CLI</span>
            <span class="text-[10px] text-neutral-500">Console environment launched for worktree terminal actions</span>
          </div>
          <select
            bind:value={localDefaultTerminal}
            class="bg-neutral-950 border border-neutral-800 rounded px-2 py-1 text-xs text-neutral-200 focus:outline-hidden focus:border-indigo-500"
          >
            {#each $installedTerminals as terminal (terminal.id)}
              <option value={terminal.id} disabled={!terminal.isAvailable}>
                {terminal.name}
              </option>
            {/each}
          </select>
        </div>

        <!-- Show Terminal Quick Button Checkbox -->
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={localShowTerminalButton}
            class="rounded bg-neutral-950 border-neutral-700 text-indigo-600 focus:ring-0 focus:outline-hidden"
          />
          <div class="flex flex-col">
            <span class="text-[11px] font-medium text-neutral-300">
              Show quick Terminal / CLI button in worktree rows
            </span>
            <span class="text-[10px] text-neutral-500">
              When disabled, only the default IDE and folder explorer buttons are shown.
            </span>
          </div>
        </label>

        <!-- Auto Switch Account Checkbox -->
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={localAutoSwitch}
            class="rounded bg-neutral-950 border-neutral-700 text-indigo-600 focus:ring-0 focus:outline-hidden"
          />
          <div class="flex flex-col">
            <span class="text-[11px] font-medium text-neutral-300">
              Auto-switch GitHub CLI account on workspace actions
            </span>
            <span class="text-[10px] text-neutral-500">
              Automatically runs `gh auth switch` when opening or modifying repositories assigned to a different account.
            </span>
          </div>
        </label>
      </div>

      <!-- Action Buttons -->
      <div class="pt-2 border-t border-neutral-800 flex items-center justify-end gap-2 text-xs">
        <button
          type="button"
          on:click={() => dispatch('close')}
          disabled={isSaving}
          class="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 transition-colors"
        >
          Cancel
        </button>

        <button
          type="button"
          on:click={handleSave}
          disabled={isSaving}
          class="px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium flex items-center gap-1.5 shadow-sm transition-all"
        >
          <Save size={13} />
          <span>Save Settings</span>
        </button>
      </div>
    </div>
  </div>
{/if}
