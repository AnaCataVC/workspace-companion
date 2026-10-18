<script lang="ts">
  import { Folder, Copy, Check } from 'lucide-svelte';
  import { onDestroy } from 'svelte';
  import { getDirectoryName } from '../utils/paths';
  import { notifications } from '../stores/notifications';

  export let path: string;
  export let showFolderIcon: boolean = false;
  export let maxWidthClass: string = 'max-w-[260px]';

  let copied = false;
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(path);
      copied = true;
      notifications.info('Copied to clipboard', path, 1800);
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        copied = false;
      }, 1500);
    } catch {
      notifications.error('Clipboard error', 'Could not copy path to clipboard');
    }
  }

  onDestroy(() => {
    if (timer) clearTimeout(timer);
  });
</script>

<button
  type="button"
  on:click={handleCopy}
  title={`Click to copy path: ${path}`}
  class="group/copy inline-flex items-center gap-1 px-1 py-0.5 -mx-1 rounded hover:bg-neutral-800/80 text-neutral-400 hover:text-neutral-200 transition-colors truncate {maxWidthClass}"
>
  {#if showFolderIcon}
    <Folder size={11} class="text-neutral-400 flex-shrink-0" />
  {/if}
  <span class="font-mono text-[11px] truncate">{getDirectoryName(path)}</span>
  {#if copied}
    <Check size={10} class="text-emerald-400 flex-shrink-0" />
  {:else}
    <Copy size={10} class="text-neutral-400 opacity-0 group-hover/copy:opacity-100 transition-opacity flex-shrink-0" />
  {/if}
</button>
