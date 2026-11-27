<script lang="ts">
  import { selectedAccountFilter } from '../stores/appConfig';
  import { scannedRepos } from '../stores/worktrees';
  import { ghAccounts } from '../stores/ghAuth';
  import { Github, Layers } from 'lucide-svelte';

  export let countsByAccountLower: Record<string, number> = {};
  export let unassignedCount: number = 0;
  export let totalCount: number = 0;

  // Account names are compared case-insensitively everywhere; the gh CLI spelling wins when both exist.
  $: accounts = (() => {
    const byLower = new Map<string, string>();
    for (const name of [
      ...$ghAccounts.map((a) => a.username),
      ...$scannedRepos.map((r) => r.associatedAccount).filter((a): a is string => Boolean(a))
    ]) {
      if (!byLower.has(name.toLowerCase())) byLower.set(name.toLowerCase(), name);
    }
    return Array.from(byLower.values());
  })();
</script>

{#if accounts.length > 0 || unassignedCount > 0}
  <div
    class="px-3 py-1.5 border-b border-neutral-850/80 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar"
    role="group"
    aria-label="Filter by GitHub account"
  >
    <button
      type="button"
      on:click={() => selectedAccountFilter.set('ALL')}
      aria-pressed={$selectedAccountFilter === 'ALL'}
      class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
        {$selectedAccountFilter === 'ALL'
          ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-sm'
          : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
    >
      <Layers size={11} />
      <span>All Accounts</span>
      <span class="px-1.5 py-0.5 rounded-full bg-neutral-900 text-[11px] font-mono text-neutral-400">
        {totalCount}
      </span>
    </button>

    {#each accounts as acc (acc)}
      <button
        type="button"
        on:click={() => selectedAccountFilter.set(acc)}
        aria-pressed={$selectedAccountFilter === acc}
        class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
          {$selectedAccountFilter === acc
            ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/60 shadow-sm'
            : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
      >
        <Github size={11} class={$selectedAccountFilter === acc ? 'text-indigo-400' : 'text-neutral-400'} />
        <span class="font-mono text-[11px]">@{acc}</span>
        <span class="px-1.5 py-0.5 rounded-full bg-neutral-900/80 text-[11px] font-mono {$selectedAccountFilter === acc ? 'text-indigo-300' : 'text-neutral-400'}">
          {countsByAccountLower[acc.toLowerCase()] ?? 0}
        </span>
      </button>
    {/each}

    {#if unassignedCount > 0 && accounts.length > 0}
      <button
        type="button"
        on:click={() => selectedAccountFilter.set('UNASSIGNED')}
        aria-pressed={$selectedAccountFilter === 'UNASSIGNED'}
        class="flex items-center gap-1.5 px-2 py-0.5 rounded font-medium transition-all flex-shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500/60
          {$selectedAccountFilter === 'UNASSIGNED'
            ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-sm'
            : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
      >
        <span>Unassigned</span>
        <span class="px-1.5 py-0.5 rounded-full bg-neutral-900 text-[11px] font-mono text-neutral-400">
          {unassignedCount}
        </span>
      </button>
    {/if}
  </div>
{/if}
