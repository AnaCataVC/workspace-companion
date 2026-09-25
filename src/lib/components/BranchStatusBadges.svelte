<script lang="ts">
  import type { BranchStatusEntry } from '../types';
  import { GitMerge, GitPullRequestClosed, ShieldCheck, FolderGit2 } from 'lucide-svelte';
  import { branchProtectionReason } from '../utils/protectionReason';

  export let branch: BranchStatusEntry;
  /** Matches the two densities the lists already use: rows run one step smaller than cards. */
  export let size: 'sm' | 'md' = 'sm';

  $: textClass = size === 'sm' ? 'text-[11px]' : 'text-[11px]';
  $: iconSize = size === 'sm' ? 9 : 10;
  $: protectionLabel = branch.isDefault ? 'Default' : branch.isCheckedOut ? 'In worktree' : '';
</script>

<div class="flex items-center gap-1 flex-shrink-0">
  {#if protectionLabel}
    <span
      class="px-1.5 py-0.5 rounded bg-indigo-950/70 text-indigo-300 border border-indigo-800/50 {textClass} flex items-center gap-0.5"
      title={branchProtectionReason(branch)}
    >
      {#if branch.isDefault}
        <ShieldCheck size={iconSize} />
      {:else}
        <FolderGit2 size={iconSize} />
      {/if}
      {protectionLabel}
    </span>
  {/if}

  {#if branch.isMerged}
    <span class="px-1.5 py-0.5 rounded bg-emerald-950/70 text-emerald-300 border border-emerald-800/50 {textClass} flex items-center gap-0.5">
      <GitMerge size={iconSize} />
      Merged
    </span>
  {/if}

  {#if branch.isRemoteGone}
    <span class="px-1.5 py-0.5 rounded bg-amber-950/70 text-amber-300 border border-amber-800/50 {textClass} flex items-center gap-0.5">
      <GitPullRequestClosed size={iconSize} />
      Remote gone
    </span>
  {/if}
</div>
