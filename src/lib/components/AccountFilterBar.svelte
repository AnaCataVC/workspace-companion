<script lang="ts">
  import { selectedAccountFilter } from '../stores/appConfig';
  import { scannedRepos } from '../stores/worktrees';
  import { ghAccounts } from '../stores/ghAuth';
  import { Github, Layers } from 'lucide-svelte';

  $: allCount = $scannedRepos.length;

  // Extract unique accounts from accounts store and scanned repos
  $: accounts = Array.from(
    new Set([
      ...$ghAccounts.map(a => a.username),
      ...$scannedRepos.map(r => r.associatedAccount).filter((a): a is string => Boolean(a))
    ])
  );

  function getCountForAccount(acc: string): number {
    return $scannedRepos.filter(r => r.associatedAccount === acc).length;
  }

  $: unassignedCount = $scannedRepos.filter(r => !r.associatedAccount).length;
</script>

{#if accounts.length > 0 || unassignedCount > 0}
  <div class="px-3 py-1.5 bg-neutral-950/60 border-b border-neutral-800 flex items-center gap-1.5 overflow-x-auto text-[11px] no-scrollbar">
    <!-- All Repositories Pill -->
    <button
      type="button"
      on:click={() => selectedAccountFilter.set('ALL')}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-medium transition-all flex-shrink-0
        {$selectedAccountFilter === 'ALL'
          ? 'bg-neutral-800 text-neutral-100 border border-neutral-700 shadow-xs'
          : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
    >
      <Layers size={11} />
      <span>All</span>
      <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-400">
        {allCount}
      </span>
    </button>

    <!-- Account Specific Pills -->
    {#each accounts as acc (acc)}
      {@const count = getCountForAccount(acc)}
      <button
        type="button"
        on:click={() => selectedAccountFilter.set(acc)}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-medium transition-all flex-shrink-0
          {$selectedAccountFilter === acc
            ? 'bg-indigo-950/80 text-indigo-200 border border-indigo-700/60 shadow-xs'
            : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
      >
        <Github size={11} class={$selectedAccountFilter === acc ? 'text-indigo-400' : 'text-neutral-500'} />
        <span class="font-mono text-[10px]">@{acc}</span>
        <span class="px-1.5 py-0.2 rounded-full bg-neutral-900/80 text-[10px] font-mono {$selectedAccountFilter === acc ? 'text-indigo-300' : 'text-neutral-500'}">
          {count}
        </span>
      </button>
    {/each}

    <!-- Unassigned Pill (Only if there are repos without account) -->
    {#if unassignedCount > 0 && accounts.length > 0}
      <button
        type="button"
        on:click={() => selectedAccountFilter.set('UNASSIGNED')}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-medium transition-all flex-shrink-0
          {$selectedAccountFilter === 'UNASSIGNED'
            ? 'bg-neutral-800 text-neutral-200 border border-neutral-700 shadow-xs'
            : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-900 border border-transparent'}"
      >
        <span>Unassigned</span>
        <span class="px-1.5 py-0.2 rounded-full bg-neutral-900 text-[10px] font-mono text-neutral-500">
          {unassignedCount}
        </span>
      </button>
    {/if}
  </div>
{/if}
