<script lang="ts">
  import type { SupportedEditor } from '../types';
  import { installedEditors } from '../stores/editors';
  import { Bot, Code2, Sparkles, Compass, Terminal, Folder, ChevronDown } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let preferredEditor: SupportedEditor = 'vscode';
  export let path: string;
  export let compact: boolean = false;

  const dispatch = createEventDispatcher<{
    openEditor: { editor: SupportedEditor; path: string };
  }>();

  let isMenuOpen = false;

  function checkAvailable(id: SupportedEditor): boolean {
    const found = $installedEditors.find((e) => e.id === id);
    return found ? found.isAvailable : true;
  }

  function selectAndLaunch(editor: SupportedEditor) {
    if (!checkAvailable(editor)) return;
    isMenuOpen = false;
    dispatch('openEditor', { editor, path });
  }

  $: editorLabel = preferredEditor === 'antigravity' ? 'Antigravity' : preferredEditor.toUpperCase();
</script>

<svelte:window
  on:keydown={(e) => {
    if (e.key === 'Escape' && isMenuOpen) {
      e.preventDefault();
      isMenuOpen = false;
    }
  }}
/>

<div
  class="relative flex items-center rounded {compact
    ? 'bg-neutral-850/90 border border-neutral-750/70'
    : 'bg-neutral-800/80 border border-neutral-700/60 shadow-xs'}"
>
  <button
    type="button"
    on:click={() => dispatch('openEditor', { editor: preferredEditor, path })}
    title={`Open in ${editorLabel}`}
    class="{compact
      ? 'min-w-6 min-h-6 justify-center p-1'
      : 'px-1.5 py-1'} text-neutral-300 hover:text-white hover:bg-neutral-700 rounded-l flex items-center gap-1 transition-colors text-[11px]"
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
    {#if !compact}
      <span class="capitalize font-mono text-[11px]">{preferredEditor === 'antigravity' ? 'Antigravity' : preferredEditor}</span>
    {/if}
  </button>

  <button
    type="button"
    on:click={() => (isMenuOpen = !isMenuOpen)}
    title="Select editor or terminal"
    aria-label="Select editor or terminal"
    aria-haspopup="menu"
    aria-expanded={isMenuOpen}
    class="{compact
      ? 'min-w-4 min-h-6 px-0.5'
      : 'min-w-6 min-h-6 px-1 py-1'} flex items-center justify-center text-neutral-400 hover:text-neutral-200 hover:bg-neutral-700 rounded-r border-l border-neutral-700/60 transition-colors"
  >
    <ChevronDown size={9} />
  </button>

  {#if isMenuOpen}
    <div
      class="fixed inset-0 z-20"
      on:click={() => (isMenuOpen = false)}
      role="presentation"
    ></div>

    <div
      role="menu"
      class="absolute right-0 top-full mt-1 z-30 w-44 rounded-lg bg-neutral-900 border border-neutral-700 shadow-xl p-1 flex flex-col gap-0.5"
    >
      {#each $installedEditors as ed (ed.id)}
        <button
          type="button"
          disabled={!ed.isAvailable}
          on:click={() => selectAndLaunch(ed.id)}
          class="flex items-center justify-between px-2 py-1 text-[11px] rounded transition-colors text-left
            {ed.isAvailable
              ? 'text-neutral-300 hover:text-white hover:bg-neutral-800 cursor-pointer'
              : 'text-neutral-400 opacity-40 cursor-not-allowed'}"
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
