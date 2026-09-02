<script lang="ts">
  import { notifications } from '../stores/notifications';
  import { AlertCircle, CheckCircle2, AlertTriangle, Info, X } from 'lucide-svelte';
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full pointer-events-none select-none px-3">
  {#each $notifications as notif (notif.id)}
    <div
      class="pointer-events-auto flex items-start gap-2.5 p-3 rounded-xl shadow-2xl border backdrop-blur-md transition-all duration-200 animate-in slide-in-from-bottom-3 fade-in
        {notif.type === 'error'
          ? 'bg-neutral-900/95 border-rose-800/80 text-rose-200'
          : notif.type === 'success'
          ? 'bg-neutral-900/95 border-emerald-800/80 text-emerald-200'
          : notif.type === 'warning'
          ? 'bg-neutral-900/95 border-amber-800/80 text-amber-200'
          : 'bg-neutral-900/95 border-neutral-700 text-neutral-200'}"
    >
      <div class="flex-shrink-0 mt-0.5">
        {#if notif.type === 'error'}
          <AlertCircle size={16} class="text-rose-400" />
        {:else if notif.type === 'success'}
          <CheckCircle2 size={16} class="text-emerald-400" />
        {:else if notif.type === 'warning'}
          <AlertTriangle size={16} class="text-amber-400" />
        {:else}
          <Info size={16} class="text-indigo-400" />
        {/if}
      </div>

      <div class="flex-1 min-w-0 flex flex-col gap-0.5">
        <p class="text-xs font-semibold leading-tight text-neutral-100">
          {notif.title}
        </p>
        {#if notif.message}
          <p class="text-[11px] leading-normal opacity-85 break-words font-normal">
            {notif.message}
          </p>
        {/if}
      </div>

      <button
        type="button"
        on:click={() => notifications.dismiss(notif.id)}
        class="flex-shrink-0 p-1 rounded hover:bg-white/10 text-neutral-400 hover:text-neutral-200 transition-colors"
        title="Dismiss"
      >
        <X size={13} />
      </button>
    </div>
  {/each}
</div>
