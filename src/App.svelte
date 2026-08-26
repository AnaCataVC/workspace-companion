<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import AccountFilterBar from './lib/components/AccountFilterBar.svelte';
  import WorktreeList from './lib/components/WorktreeList.svelte';
  import OrphanCleanerModal from './lib/components/OrphanCleanerModal.svelte';
  import GhAccountModal from './lib/components/GhAccountModal.svelte';
  import BranchSwitcherModal from './lib/components/BranchSwitcherModal.svelte';
  import NewWorktreeModal from './lib/components/NewWorktreeModal.svelte';
  import WatchFoldersModal from './lib/components/WatchFoldersModal.svelte';
  import { scannedRepos, isScanning, isPinned, scanError } from './lib/stores/worktrees';
  import { ghAccounts, activeGhAccount, isGhLoading } from './lib/stores/ghAuth';
  import { appConfig, selectedAccountFilter } from './lib/stores/appConfig';
  import { installedEditors } from './lib/stores/editors';
  import type {
    RepositoryWorktrees,
    WorktreeInfo,
    GhAccount,
    SupportedEditor,
    EditorInfo,
    WorktreeBranchesResponse,
    BranchEntry,
    SuggestWorktreePathResult,
    CreateWorktreeResult,
    CheckoutBranchResult,
    AppConfig
  } from './lib/types';

  // Deletion state
  let selectedWorktreeForDelete: WorktreeInfo | null = null;
  let selectedRepoPathForDelete: string = '';
  let isDeleteModalOpen: boolean = false;
  let isDeletingWorktree: boolean = false;

  // GitHub account modal state
  let isGhModalOpen: boolean = false;

  // Settings & Watched Folders modal state
  let isSettingsModalOpen: boolean = false;
  let isSavingConfig: boolean = false;

  // Branch switcher modal state
  let isBranchSwitcherOpen: boolean = false;
  let selectedWorktreeForBranchSwitch: WorktreeInfo | null = null;
  let selectedRepoForBranchSwitch: string = '';
  let branchesResponse: WorktreeBranchesResponse | null = null;
  let isLoadingBranches: boolean = false;
  let isSwitchingBranch: boolean = false;
  let branchSwitchError: string | null = null;

  // New worktree modal state
  let isNewWorktreeOpen: boolean = false;
  let initialRepoForNewWorktree: string = '';
  let isCreatingWorktree: boolean = false;
  let newWorktreeError: string | null = null;

  async function invokeTauri<T>(cmd: string, args: Record<string, any> = {}): Promise<T> {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(cmd, args);
    } else {
      // Mock data for web development/preview
      console.warn(`[Mock IPC] Invoked "${cmd}" with args:`, args);
      if (cmd === 'get_app_config') {
        return {
          version: 1,
          watchFolders: [
            { id: '1', path: 'C:/Projects/Personal', accountUsername: 'alex-dev', enabled: true, maxDepth: 1 },
            { id: '2', path: 'C:/Projects/Work', accountUsername: 'acme-corp', enabled: true, maxDepth: 4 }
          ],
          autoSwitchAccount: true,
          defaultEditor: 'vscode'
        } as unknown as T;
      }
      if (cmd === 'save_app_config') {
        return args.config as unknown as T;
      }
      if (cmd === 'scan_worktrees') {
        return null as unknown as T;
      }

      if (cmd === 'get_gh_accounts') {
        return [
          { username: 'alex-dev', active: true, host: 'github.com' },
          { username: 'acme-corp', active: false, host: 'github.com' }
        ] as unknown as T;
      }
      if (cmd === 'list_branches') {
        return {
          repoPath: args.repoPath || '',
          worktreePath: args.worktreePath || '',
          currentBranch: 'main',
          branches: [
            { name: 'main', shortName: 'main', isRemote: false, isCurrent: true, isLockedByOther: false, lastCommitSha: 'a1b2c3d', lastCommitMessage: 'feat: add UI' },
            { name: 'develop', shortName: 'develop', isRemote: false, isCurrent: false, isLockedByOther: false, lastCommitSha: 'b2c3d4e', lastCommitMessage: 'chore: update packages' }
          ]
        } as unknown as T;
      }
      if (cmd === 'suggest_worktree_path') {
        const repo = args.repoPath.split(/[\/\\]/).pop() || 'repo';
        const clean = (args.branchName || 'branch').replace(/[\/\\]/g, '-');
        return {
          suggestedPath: `C:/Projects/Personal/${repo}-${clean}`,
          alreadyExists: false
        } as unknown as T;
      }
      if (cmd === 'create_worktree') {
        return { success: true, worktreePath: args.targetPath, branchName: args.baseBranch, message: 'Worktree created' } as unknown as T;
      }
      if (cmd === 'checkout_worktree_branch') {
        return { success: true, newBranch: args.targetBranch, headSha: '1a2b3c4', message: 'Checked out' } as unknown as T;
      }
      if (cmd === 'open_in_editor' || cmd === 'switch_gh_account' || cmd === 'remove_worktree') {
        return { success: true } as unknown as T;
      }
      return null as unknown as T;
    }
  }

  async function loadAppConfig() {
    try {
      const cfg = await invokeTauri<AppConfig>('get_app_config');
      if (cfg) {
        appConfig.set(cfg);
      }
    } catch (err) {
      console.error('Failed to load app config:', err);
    }
  }

  async function handleSaveAppConfig(event: CustomEvent<AppConfig>) {
    isSavingConfig = true;
    try {
      const saved = await invokeTauri<AppConfig>('save_app_config', { config: event.detail });
      if (saved) {
        appConfig.set(saved);
      }
      isSettingsModalOpen = false;
      await refreshWorktrees();
    } catch (err: any) {
      alert(`Failed to save configuration: ${err?.message || err}`);
    } finally {
      isSavingConfig = false;
    }
  }

  async function refreshWorktrees() {
    isScanning.set(true);
    scanError.set(null);
    scannedRepos.set([]);

    try {
      await invokeTauri('scan_worktrees');
    } catch (err: any) {
      console.error('Failed to scan worktrees:', err);
      scanError.set(err?.toString() || 'Failed to scan worktrees');
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

  async function handleSwitchGhAccount(event: CustomEvent<string> | string) {
    const targetUser = typeof event === 'string' ? event : event.detail;
    if (!targetUser) return;
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

  // Smart Context Switcher check before action
  async function ensureMatchingAccountForPath(targetPath: string) {
    if (!$appConfig.autoSwitchAccount) return;
    const repo = $scannedRepos.find(r => targetPath.startsWith(r.repoPath) || r.worktrees.some(w => targetPath.startsWith(w.path)));
    if (repo?.associatedAccount && repo.associatedAccount !== $activeGhAccount) {
      try {
        await invokeTauri('switch_gh_account', { username: repo.associatedAccount });
        await refreshGhAccounts();
      } catch (e) {
        console.warn('Smart switch account warning:', e);
      }
    }
  }

  async function handleOpenPath(event: CustomEvent<string>) {
    const path = event.detail;
    await ensureMatchingAccountForPath(path);
    invokeTauri('open_path', { path });
  }

  async function handleOpenEditor(event: CustomEvent<{ editor: SupportedEditor; path: string }>) {
    const { editor, path } = event.detail;
    await ensureMatchingAccountForPath(path);
    invokeTauri('open_in_editor', { editor, path }).catch((err: any) => {
      alert(`Could not open editor (${editor}): ${err?.message || err}`);
    });
  }

  // Branch Switcher Modal handlers
  async function handleRequestSwitchBranch(event: CustomEvent<{ worktree: WorktreeInfo; repoPath: string }>) {
    selectedWorktreeForBranchSwitch = event.detail.worktree;
    selectedRepoForBranchSwitch = event.detail.repoPath;
    branchSwitchError = null;
    branchesResponse = null;
    isBranchSwitcherOpen = true;
    isLoadingBranches = true;

    try {
      const res = await invokeTauri<WorktreeBranchesResponse>('list_branches', {
        repoPath: selectedRepoForBranchSwitch,
        worktreePath: selectedWorktreeForBranchSwitch.path
      });
      branchesResponse = res;
    } catch (err: any) {
      branchSwitchError = err?.message || err?.toString() || 'Failed to list branches';
    } finally {
      isLoadingBranches = false;
    }
  }

  async function handleConfirmSwitchBranch(event: CustomEvent<{ worktree: WorktreeInfo; targetBranch: string }>) {
    const { worktree, targetBranch } = event.detail;
    isSwitchingBranch = true;
    branchSwitchError = null;

    try {
      await ensureMatchingAccountForPath(worktree.path);
      await invokeTauri<CheckoutBranchResult>('checkout_worktree_branch', {
        worktreePath: worktree.path,
        targetBranch
      });
      isBranchSwitcherOpen = false;
      selectedWorktreeForBranchSwitch = null;
      await refreshWorktrees();
    } catch (err: any) {
      branchSwitchError = err?.message || err?.toString() || 'Failed to switch branch';
    } finally {
      isSwitchingBranch = false;
    }
  }

  // New Worktree Modal handlers
  function handleOpenNewWorktree(repoPath?: string) {
    initialRepoForNewWorktree = repoPath || ($scannedRepos[0]?.repoPath || '');
    newWorktreeError = null;
    isNewWorktreeOpen = true;
  }

  async function handleSuggestPath(repoPath: string, branchName: string): Promise<SuggestWorktreePathResult> {
    return await invokeTauri<SuggestWorktreePathResult>('suggest_worktree_path', {
      repoPath,
      branchName
    });
  }

  async function handleFetchBranchesForRepo(repoPath: string): Promise<BranchEntry[]> {
    const res = await invokeTauri<WorktreeBranchesResponse>('list_branches', {
      repoPath,
      worktreePath: repoPath
    });
    return res?.branches || [];
  }

  async function handleConfirmCreateWorktree(event: CustomEvent<{
    repoPath: string;
    targetPath: string;
    baseBranch: string;
    newBranchName?: string;
  }>) {
    isCreatingWorktree = true;
    newWorktreeError = null;

    try {
      await ensureMatchingAccountForPath(event.detail.repoPath);
      await invokeTauri<CreateWorktreeResult>('create_worktree', {
        repoPath: event.detail.repoPath,
        targetPath: event.detail.targetPath,
        baseBranch: event.detail.baseBranch,
        newBranchName: event.detail.newBranchName || null
      });
      isNewWorktreeOpen = false;
      await refreshWorktrees();
    } catch (err: any) {
      newWorktreeError = err?.message || err?.toString() || 'Failed to create worktree';
    } finally {
      isCreatingWorktree = false;
    }
  }

  // Worktree Deletion Handlers
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

    // Collect non-dirty orphans — dirty ones require explicit user confirmation
    const orphans = $scannedRepos
      .find((r) => r.repoPath === repoPath)
      ?.worktrees.filter((w) => w.isOrphaned && !w.isDirty) ?? [];

    if (orphans.length === 0) {
      alert('No safe orphaned worktrees to clean (all have uncommitted changes).');
      return;
    }

    const confirmed = window.confirm(
      `Remove ${orphans.length} orphaned worktree(s) in this repository?\n\n` +
      orphans.map((w) => `  • ${w.branch ?? '(detached)'} — ${w.path}`).join('\n') +
      '\n\nWorktrees with uncommitted changes are skipped.'
    );
    if (!confirmed) return;

    const errors: string[] = [];
    for (const wt of orphans) {
      try {
        await invokeTauri('remove_worktree', {
          repoPath,
          worktreePath: wt.path,
          force: false
        });
      } catch (err: any) {
        errors.push(`${wt.branch ?? wt.path}: ${err?.message ?? err}`);
      }
    }

    // Clean up stale admin references afterwards
    try {
      await invokeTauri('prune_worktrees', { repoPath });
    } catch (_) { /* non-critical */ }

    await refreshWorktrees();

    if (errors.length > 0) {
      alert(`Some worktrees could not be removed:\n\n${errors.join('\n')}`);
    }
  }

  async function refreshInstalledEditors() {
    try {
      const editors = await invokeTauri<EditorInfo[]>('detect_installed_editors');
      if (editors && editors.length > 0) {
        installedEditors.set(editors);
      }
    } catch (err) {
      console.error('Failed to detect editors:', err);
    }
  }

  // Sync window AlwaysOnTop with isPinned store
  $: {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
        getCurrentWindow().setAlwaysOnTop($isPinned).catch((err) => {
          console.warn('Failed to update window alwaysOnTop:', err);
        });
      });
    }
  }

  let unlistenRepo: (() => void) | null = null;
  let unlistenDone: (() => void) | null = null;

  onMount(async () => {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenRepo = await listen<RepositoryWorktrees>('repo-scanned', (event) => {
          scannedRepos.update((repos) => {
            const existingIndex = repos.findIndex((r) => r.repoPath === event.payload.repoPath);
            if (existingIndex >= 0) {
              const updated = [...repos];
              updated[existingIndex] = event.payload;
              return updated;
            }
            return [...repos, event.payload];
          });
        });

        unlistenDone = await listen('scan-complete', () => {
          isScanning.set(false);
        });
      } catch (err) {
        console.error('Failed to setup scan event listeners:', err);
      }
    }

    await loadAppConfig();
    refreshInstalledEditors();
    refreshWorktrees();
    refreshGhAccounts();
  });

  onDestroy(() => {
    if (unlistenRepo) unlistenRepo();
    if (unlistenDone) unlistenDone();
  });
</script>

<main class="w-full h-screen flex flex-col bg-neutral-900 text-neutral-100 overflow-hidden select-none">
  <Header
    onRefresh={refreshWorktrees}
    onOpenGhModal={() => (isGhModalOpen = true)}
    onOpenNewWorktreeModal={() => handleOpenNewWorktree()}
    onOpenSettingsModal={() => (isSettingsModalOpen = true)}
  />

  {#if $scanError}
    <div class="p-2 mx-3 mt-2 rounded bg-rose-950/60 border border-rose-800/50 text-rose-300 text-xs">
      {$scanError}
    </div>
  {/if}

  <AccountFilterBar />

  <WorktreeList
    on:openPath={handleOpenPath}
    on:openEditor={handleOpenEditor}
    on:requestSwitchBranch={handleRequestSwitchBranch}
    on:newWorktreeForRepo={(e) => handleOpenNewWorktree(e.detail)}
    on:switchGhAccount={(e) => handleSwitchGhAccount(e.detail)}
    on:openSettings={() => (isSettingsModalOpen = true)}
    on:requestDelete={handleRequestDelete}
    on:cleanAllOrphans={handleCleanAllOrphans}
    on:worktreeCreated={refreshWorktrees}
  />

  <WatchFoldersModal
    isOpen={isSettingsModalOpen}
    config={$appConfig}
    ghAccounts={$ghAccounts}
    isSaving={isSavingConfig}
    on:close={() => (isSettingsModalOpen = false)}
    on:save={handleSaveAppConfig}
  />

  <BranchSwitcherModal
    isOpen={isBranchSwitcherOpen}
    worktree={selectedWorktreeForBranchSwitch}
    repoPath={selectedRepoForBranchSwitch}
    branchesResponse={branchesResponse}
    isLoadingBranches={isLoadingBranches}
    isSwitching={isSwitchingBranch}
    errorMessage={branchSwitchError}
    on:close={() => (isBranchSwitcherOpen = false)}
    on:switchBranch={handleConfirmSwitchBranch}
  />

  <NewWorktreeModal
    isOpen={isNewWorktreeOpen}
    repositories={$scannedRepos}
    initialRepoPath={initialRepoForNewWorktree}
    isCreating={isCreatingWorktree}
    errorMessage={newWorktreeError}
    onSuggestPath={handleSuggestPath}
    onFetchBranches={handleFetchBranchesForRepo}
    on:close={() => (isNewWorktreeOpen = false)}
    on:create={handleConfirmCreateWorktree}
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

