<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import WorktreeList from './lib/components/WorktreeList.svelte';
  import OrphanCleanerModal from './lib/components/OrphanCleanerModal.svelte';
  import GhAccountModal from './lib/components/GhAccountModal.svelte';
  import { scannedRepos, isScanning, isPinned, scanError } from './lib/stores/worktrees';
  import { ghAccounts, activeGhAccount, isGhLoading } from './lib/stores/ghAuth';
  import type { RepositoryWorktrees, WorktreeInfo, GhAccount } from './lib/types';

  let selectedWorktreeForDelete: WorktreeInfo | null = null;
  let selectedRepoPathForDelete: string = '';
  let isDeleteModalOpen: boolean = false;
  let isGhModalOpen: boolean = false;
  let isDeletingWorktree: boolean = false;

  async function invokeTauri<T>(cmd: string, args: Record<string, any> = {}): Promise<T> {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    } else {
      // Mock data for web development/preview
      console.warn(`[Mock IPC] Invoked "${cmd}" with args:`, args);
      if (cmd === 'scan_worktrees') {
        return [
          {
            repoName: 'workspace-companion',
            repoPath: 'C:/Users/anaca/Repos/workspace-companion',
            worktrees: [
              {
                path: 'C:/Users/anaca/Repos/workspace-companion',
                head: 'a1b2c3d4e5f6',
                branch: 'refs/heads/main',
                bare: false,
                locked: null,
                prunable: null,
                isOrphaned: false,
                isDirty: false,
                lastCommitMessage: 'feat: add initial architecture and system tray scaffolding'
              },
              {
                path: 'C:/Users/anaca/Repos/workspace-companion-feature',
                head: 'f6e5d4c3b2a1',
                branch: 'refs/heads/feat/orphan-cleaner',
                bare: false,
                locked: null,
                prunable: null,
                isOrphaned: true,
                orphanReason: 'Remote branch merged',
                isDirty: false,
                lastCommitMessage: 'feat(cleaner): implement safe orphan cleanup check'
              }
            ]
          }
        ] as unknown as T;
      }
      if (cmd === 'get_gh_accounts') {
        return [
          { username: 'AnaCataVC', active: true, host: 'github.com' },
          { username: 'CataVillalobosC', active: false, host: 'github.com' }
        ] as unknown as T;
      }
      if (cmd === 'switch_gh_account') {
        return { success: true } as unknown as T;
      }
      if (cmd === 'remove_worktree') {
        return { success: true } as unknown as T;
      }
      return null as unknown as T;
    }
  }

  async function refreshWorktrees() {
    isScanning.set(true);
    scanError.set(null);
    try {
      const repos = await invokeTauri<RepositoryWorktrees[]>('scan_worktrees');
      scannedRepos.set(repos || []);
    } catch (err: any) {
      console.error('Failed to scan worktrees:', err);
      scanError.set(err?.toString() || 'Failed to scan worktrees');
    } finally {
      isScanning.set(false);
    }
  }

  async function refreshGhAccounts() {
    isGhLoading.set(true);
    try {
      const accounts = await invokeTauri<GhAccount[]>('get_gh_accounts');
      ghAccounts.set(accounts || []);
      const active = accounts?.find(a => a.active);
      activeGhAccount.set(active ? active.username : null);
    } catch (err) {
      console.error('Failed to load GH accounts:', err);
    } finally {
      isGhLoading.set(false);
    }
  }

  async function handleSwitchGhAccount(event: CustomEvent<string>) {
    const targetUser = event.detail;
    isGhLoading.set(true);
    try {
      await invokeTauri('switch_gh_account', { username: targetUser });
      await refreshGhAccounts();
    } catch (err) {
      console.error('Failed to switch GH account:', err);
    } finally {
      isGhLoading.set(false);
    }
  }

  function handleOpenPath(event: CustomEvent<string>) {
    const path = event.detail;
    invokeTauri('open_path', { path });
  }

  function handleRequestDelete(event: CustomEvent<{ worktree: WorktreeInfo; repoPath: string }>) {
    selectedWorktreeForDelete = event.detail.worktree;
    selectedRepoPathForDelete = event.detail.repoPath;
    isDeleteModalOpen = true;
  }

  async function handleConfirmDelete(event: CustomEvent<{ worktree: WorktreeInfo; repoPath: string; force: boolean }>) {
    const { worktree, repoPath, force } = event.detail;
    isDeletingWorktree = true;
    try {
      await invokeTauri('remove_worktree', {
        repoPath,
        worktreePath: worktree.path,
        force
      });
      isDeleteModalOpen = false;
      selectedWorktreeForDelete = null;
      await refreshWorktrees();
    } catch (err: any) {
      alert(`Error deleting worktree: ${err?.message || err}`);
    } finally {
      isDeletingWorktree = false;
    }
  }

  async function handleCleanAllOrphans(event: CustomEvent<string>) {
    const repoPath = event.detail;
    const confirm = window.confirm(`Are you sure you want to prune and clean all safe orphaned worktrees in this repository?`);
    if (!confirm) return;

    try {
      await invokeTauri('prune_worktrees', { repoPath });
      await refreshWorktrees();
    } catch (err: any) {
      alert(`Error pruning worktrees: ${err?.message || err}`);
    }
  }

  onMount(() => {
    refreshWorktrees();
    refreshGhAccounts();
  });
</script>

<main class="w-full h-screen flex flex-col bg-neutral-900 text-neutral-100 overflow-hidden select-none">
  <Header
    onRefresh={refreshWorktrees}
    onOpenGhModal={() => (isGhModalOpen = true)}
  />

  {#if $scanError}
    <div class="p-2 mx-3 mt-2 rounded bg-rose-950/60 border border-rose-800/50 text-rose-300 text-xs">
      {$scanError}
    </div>
  {/if}

  <WorktreeList
    on:openPath={handleOpenPath}
    on:requestDelete={handleRequestDelete}
    on:cleanAllOrphans={handleCleanAllOrphans}
  />

  <OrphanCleanerModal
    isOpen={isDeleteModalOpen}
    worktree={selectedWorktreeForDelete}
    repoPath={selectedRepoPathForDelete}
    isDeleting={isDeletingWorktree}
    on:close={() => (isDeleteModalOpen = false)}
    on:confirmDelete={handleConfirmDelete}
  />

  <GhAccountModal
    isOpen={isGhModalOpen}
    on:close={() => (isGhModalOpen = false)}
    on:switchAccount={handleSwitchGhAccount}
    on:refreshAccounts={refreshGhAccounts}
  />
</main>
