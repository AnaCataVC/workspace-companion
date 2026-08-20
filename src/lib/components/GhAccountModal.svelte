<script lang="ts">
  import { ghAccounts, activeGhAccount, isGhLoading } from '../stores/ghAuth';
  import type { GhAccount } from '../types';
  import { Github, Check, X, RefreshCw } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';

  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    switchAccount: string;
    refreshAccounts: void;
  }>();

  function close() {
    dispatch('close');
  }

  function handleSelect(account: GhAccount) {
    if (account.username === $activeGhAccount) return;
    dispatch('switchAccount', account.username);
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-100">
    <div class="w-full max-w-sm bg-neutral-900 border border-neutral-800 rounded-xl p-4 shadow-2xl flex flex-col gap-3">
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-neutral-200">
          <Github size={16} />
          <h2 class="text-xs font-semibold uppercase tracking-wider">
            GitHub CLI Accounts
          </h2>
        </div>
        <div class="flex items-center gap-1">
          <button
            on:click={() => dispatch('refreshAccounts')}
            disabled={$isGhLoading}
            class="text-neutral-500 hover:text-neutral-300 p-1 rounded"
            title="Refresh accounts"
          >
            <RefreshCw size={13} class={$isGhLoading ? 'animate-spin text-indigo-400' : ''} />
          </button>
          <button on:click={close} class="text-neutral-500 hover:text-neutral-300 p-1 rounded">
            <X size={14} />
          </button>
        </div>
      </div>

      <p class="text-[11px] text-neutral-400">
        Select which account you want to be actively authenticated in GitHub CLI (`gh`).
      </p>

      <!-- Account list -->
      <div class="space-y-1.5 max-h-56 overflow-y-auto">
        {#if $ghAccounts.length === 0}
          <div class="p-3 text-center text-xs text-neutral-500 bg-neutral-950/60 rounded-lg border border-neutral-800">
            No accounts found in GitHub CLI. Run <code>gh auth login</code> in terminal.
          </div>
        {:else}
          {#each $ghAccounts as acc (acc.username)}
            <button
              on:click={() => handleSelect(acc)}
              disabled={$isGhLoading || acc.username === $activeGhAccount}
              class="w-full flex items-center justify-between p-2.5 rounded-lg border transition-all text-left text-xs {acc.username === $activeGhAccount ? 'bg-indigo-950/40 border-indigo-500/50 text-indigo-200' : 'bg-neutral-950/70 border-neutral-800/80 hover:bg-neutral-800/70 text-neutral-300'}"
            >
              <div class="flex items-center gap-2">
                <div class="w-6 h-6 rounded-full bg-neutral-800 flex items-center justify-center font-mono text-[10px] font-bold text-neutral-300">
                  {acc.username.substring(0, 2).toUpperCase()}
                </div>
                <div class="flex flex-col">
                  <span class="font-mono font-medium">{acc.username}</span>
                  <span class="text-[10px] text-neutral-500">{acc.host}</span>
                </div>
              </div>

              {#if acc.username === $activeGhAccount}
                <div class="flex items-center gap-1 text-[11px] text-indigo-400 font-medium">
                  <Check size={14} />
                  <span>Active</span>
                </div>
              {:else}
                <span class="text-[10px] text-neutral-500 opacity-0 hover:opacity-100 transition-opacity">
                  Switch
                </span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <!-- Footer info -->
      <div class="pt-2 border-t border-neutral-800/70 flex justify-between items-center text-[10px] text-neutral-500">
        <span>Runs: <code>gh auth switch -u &lt;user&gt;</code></span>
        <button on:click={close} class="hover:text-neutral-300">Done</button>
      </div>
    </div>
  </div>
{/if}
