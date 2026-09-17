<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import AccountFilterBar from './lib/components/AccountFilterBar.svelte';
  import WorktreeList from './lib/components/WorktreeList.svelte';
  import RemoveWorktreeModal from './lib/components/RemoveWorktreeModal.svelte';
  import GhAccountModal from './lib/components/GhAccountModal.svelte';
  import BranchSwitcherModal from './lib/components/BranchSwitcherModal.svelte';
  import NewWorktreeModal from './lib/components/NewWorktreeModal.svelte';
  import WatchFoldersModal from './lib/components/WatchFoldersModal.svelte';
  import BatchActionBar from './lib/components/BatchActionBar.svelte';
  import BatchDeleteModal from './lib/components/BatchDeleteModal.svelte';
  import BranchFilterBar from './lib/components/BranchFilterBar.svelte';
  import BranchList from './lib/components/BranchList.svelte';
  import BranchActionBar from './lib/components/BranchActionBar.svelte';
  import BranchBatchDeleteModal from './lib/components/BranchBatchDeleteModal.svelte';
  import ToastContainer from './lib/components/ToastContainer.svelte';
  import { scannedRepos, isScanning, isPinned, scanError, highlightedWorktreePath, searchFilter } from './lib/stores/worktrees';
  import { scannedBranches, isScanningBranches, branchScanError } from './lib/stores/branchCleaner';
  import { ghAccounts, activeGhAccount, isGhLoading } from './lib/stores/ghAuth';
  import { appConfig, selectedAccountFilter, selectedStatusFilter } from './lib/stores/appConfig';
  import { batchSelection, selectedWorktreeList } from './lib/stores/batchSelection';
  import { branchSelection, selectedBranchList } from './lib/stores/branchSelection';
  import { installedEditors } from './lib/stores/editors';
  import { notifications } from './lib/stores/notifications';
  import type {
    RepositoryWorktrees,
    WorktreeInfo,
    GhAccount,
    SupportedEditor,
    SupportedTerminal,
    EditorInfo,
    WorktreeBranchesResponse,
    BranchEntry,
    SuggestWorktreePathResult,
    CreateWorktreeResult,
    CheckoutBranchResult,
    AppConfig,
    BatchDeleteTarget,
    BatchDeleteSummary,
    BranchStatusEntry,
    BranchDeleteTarget,
    BranchBatchDeleteSummary
  } from './lib/types';

  // Top-level view toggle: worktrees vs. the Branch Cleaner
  let activeView: 'worktrees' | 'branches' = 'worktrees';

  // Branch cleaner batch deletion state
  let isBranchBatchDeleteModalOpen: boolean = false;
  let isBranchBatchDeleting: boolean = false;
  let branchBatchDeleteSummary: BranchBatchDeleteSummary | null = null;
  let branchBatchDeleteError: string | null = null;

  // Deletion state
  let selectedWorktreeForDelete: WorktreeInfo | null = null;
  let selectedRepoPathForDelete: string = '';
  let isDeleteModalOpen: boolean = false;
  let isDeletingWorktree: boolean = false;

  // Batch deletion state
  let isBatchDeleteModalOpen: boolean = false;
  let isBatchDeleting: boolean = false;
  let batchDeleteSummary: BatchDeleteSummary | null = null;
  let batchDeleteError: string | null = null;

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
  let initialExistingBranchForNewWorktree: string = '';
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
        return {
          success: true,
          worktreePath: args.targetPath,
          branchName: args.newBranchName || args.baseBranch,
          message: 'Worktree created',
          worktreeInfo: {
            path: args.targetPath,
            head: '0000000',
            branch: args.newBranchName || args.baseBranch,
            bare: false,
            locked: null,
            prunable: null,
            isMain: false,
            isOrphaned: false,
            isDirty: false,
            uncommittedFilesCount: 0
          }
        } as unknown as T;
      }
      if (cmd === 'checkout_worktree_branch') {
        return {
          success: true,
          newBranch: args.targetBranch,
          headSha: '1a2b3c4',
          message: 'Checked out',
          worktreeInfo: {
            path: args.worktreePath,
            head: '1a2b3c4',
            branch: args.targetBranch,
            bare: false,
            locked: null,
            prunable: null,
            isOrphaned: false,
            isDirty: false,
            uncommittedFilesCount: 0
          }
        } as unknown as T;
      }
      if (cmd === 'open_in_editor' || cmd === 'open_in_terminal' || cmd === 'switch_gh_account' || cmd === 'remove_worktree') {
        return { success: true } as unknown as T;
      }
      if (cmd === 'scan_branches_for_cleanup') {
        const repoPaths: string[] = args.repoPaths || [];
        return repoPaths.flatMap((repoPath) => [
          {
            repoPath, name: 'main', isCurrent: true, isDefault: true, isMerged: false,
            isRemoteGone: false, isCheckedOut: true, checkedOutWorktreePath: repoPath,
            lastCommitSha: 'a1b2c3d', lastCommitMessage: 'feat: add UI'
          },
          {
            repoPath, name: 'feat/old-merged', isCurrent: false, isDefault: false, isMerged: true,
            isRemoteGone: false, isCheckedOut: false, checkedOutWorktreePath: null,
            lastCommitSha: 'b2c3d4e', lastCommitMessage: 'Merged feature'
          },
          {
            repoPath, name: 'feat/stale-remote', isCurrent: false, isDefault: false, isMerged: false,
            isRemoteGone: true, isCheckedOut: false, checkedOutWorktreePath: null,
            lastCommitSha: 'c3d4e5f', lastCommitMessage: 'Upstream deleted'
          }
        ]) as unknown as T;
      }
      if (cmd === 'remove_branches_batch') {
        const targets = (args.targets || []) as BranchDeleteTarget[];
        return {
          totalRequested: targets.length,
          deletedCount: targets.length,
          skippedCount: 0,
          deletedBranches: targets.map((t) => ({ repoPath: t.repoPath, branchName: t.branchName })),
          errors: []
        } as unknown as T;
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
        notifications.success('Settings saved', 'Configuration updated successfully.');
      }
      isSettingsModalOpen = false;
      await refreshWorktrees();
    } catch (err: any) {
      notifications.error('Failed to save configuration', err?.message || String(err));
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
    } catch (err: any) {
      notifications.error('Failed to switch GitHub account', err?.message || String(err));
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

  function handleOpenPath(event: CustomEvent<string>) {
    const path = event.detail;
    ensureMatchingAccountForPath(path);
    invokeTauri('open_path', { path }).catch((err: any) => {
      notifications.error('Could not open folder', err?.message || String(err));
    });
  }

  function handleOpenEditor(event: CustomEvent<{ editor: SupportedEditor; path: string }>) {
    const { editor, path } = event.detail;
    // Account switch runs concurrently, not awaited: it must not delay the editor launch itself.
    ensureMatchingAccountForPath(path);
    invokeTauri('open_in_editor', { editor, path }).catch((err: any) => {
      notifications.error(`Could not open editor (${editor})`, err?.message || String(err));
    });
  }

  function handleOpenTerminal(event: CustomEvent<{ terminal: SupportedTerminal; path: string }>) {
    const { terminal, path } = event.detail;
    if (terminal === 'none') return;
    ensureMatchingAccountForPath(path);
    invokeTauri('open_in_terminal', { terminal, path }).catch((err: any) => {
      notifications.error(`Could not open terminal (${terminal})`, err?.message || String(err));
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
    const repoPath = selectedRepoForBranchSwitch;
    isSwitchingBranch = true;
    branchSwitchError = null;

    ensureMatchingAccountForPath(worktree.path);

    try {
      const res = await invokeTauri<CheckoutBranchResult>('checkout_worktree_branch', {
        worktreePath: worktree.path,
        targetBranch
      });
      isBranchSwitcherOpen = false;
      selectedWorktreeForBranchSwitch = null;

      if (res?.worktreeInfo) {
        const freshInfo = res.worktreeInfo;
        scannedRepos.update((repos) =>
          repos.map((r) =>
            r.repoPath === repoPath
              ? { ...r, worktrees: r.worktrees.map((w) => (w.path === worktree.path ? freshInfo : w)) }
              : r
          )
        );
      } else {
        await refreshWorktrees();
      }
    } catch (err: any) {
      branchSwitchError = err?.message || err?.toString() || 'Failed to switch branch';
    } finally {
      isSwitchingBranch = false;
    }
  }

  // New Worktree Modal handlers
  function handleOpenNewWorktree(repoPath?: string) {
    initialRepoForNewWorktree = repoPath || ($scannedRepos[0]?.repoPath || '');
    initialExistingBranchForNewWorktree = '';
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
    const repoPath = event.detail.repoPath;

    ensureMatchingAccountForPath(repoPath);

    try {
      const res = await invokeTauri<CreateWorktreeResult>('create_worktree', {
        repoPath,
        targetPath: event.detail.targetPath,
        baseBranch: event.detail.baseBranch,
        newBranchName: event.detail.newBranchName || null
      });
      isNewWorktreeOpen = false;

      if (res?.worktreeInfo) {
        const freshInfo = res.worktreeInfo;
        scannedRepos.update((repos) =>
          repos.map((r) => (r.repoPath === repoPath ? { ...r, worktrees: [...r.worktrees, freshInfo] } : r))
        );
      } else {
        await refreshWorktrees();
      }
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

      scannedRepos.update((repos) =>
        repos
          .map((r) =>
            r.repoPath === repoPath
              ? { ...r, worktrees: r.worktrees.filter((w) => w.path !== worktree.path) }
              : r
          )
          .filter((r) => r.worktrees.length > 0)
      );
    } catch (err: any) {
      notifications.error('Error deleting worktree', err?.message || String(err));
    } finally {
      isDeletingWorktree = false;
    }
  }

  function handleOpenBatchDeleteModal() {
    batchDeleteSummary = null;
    batchDeleteError = null;
    isBatchDeleteModalOpen = true;
  }

  async function handleConfirmBatchDelete(event: CustomEvent<{ targets: BatchDeleteTarget[]; force: boolean }>) {
    const { targets } = event.detail;
    isBatchDeleting = true;
    batchDeleteError = null;

    try {
      const summary = await invokeTauri<BatchDeleteSummary>('remove_worktrees_batch', {
        targets
      });

      batchDeleteSummary = summary;

      // Delta Update: Evict deleted worktrees directly from $scannedRepos in memory
      if (summary && summary.deletedPaths && summary.deletedPaths.length > 0) {
        const deletedSet = new Set(summary.deletedPaths);
        scannedRepos.update((repos) =>
          repos
            .map((repo) => ({
              ...repo,
              worktrees: repo.worktrees.filter((w) => !deletedSet.has(w.path))
            }))
            .filter((repo) => repo.worktrees.length > 0)
        );

        // Auto-prune batch selection store
        const remainingPaths = new Set<string>();
        for (const repo of $scannedRepos) {
          for (const wt of repo.worktrees) {
            remainingPaths.add(wt.path);
          }
        }
        batchSelection.prune(remainingPaths);
      }

      // A skipped target is a protection doing its job, not an outcome the user has to review
      // in the modal — only a genuine failure keeps the summary phase open.
      if (summary && !summary.errors.some((e) => e.kind === 'failed')) {
        batchSelection.clear();
        isBatchDeleteModalOpen = false;
        const skippedNote = summary.skippedCount > 0 ? ` ${summary.skippedCount} protected worktree(s) skipped.` : '';
        notifications.success('Worktrees removed', `Successfully removed ${summary.deletedCount} worktree(s).${skippedNote}`);
      }
    } catch (err: any) {
      batchDeleteError = err?.message || err?.toString() || 'Failed to remove worktrees';
    } finally {
      isBatchDeleting = false;
    }
  }

  function handleCleanAllOrphans(event: CustomEvent<string>) {
    const repoPath = event.detail;
    const repo = $scannedRepos.find((r) => r.repoPath === repoPath);
    if (!repo) return;

    // Collect non-main orphaned worktrees
    const orphans = repo.worktrees.filter((w) => !w.isMain && w.isOrphaned);

    if (orphans.length === 0) {
      notifications.info('No orphaned worktrees found to clean in this repository.');
      return;
    }

    // Populate batch selection with these orphaned targets and open BatchDeleteModal
    const targets: BatchDeleteTarget[] = orphans.map((w) => ({
      repoPath: repo.repoPath,
      worktreePath: w.path,
      branch: w.branch,
      isDirty: w.isDirty,
      uncommittedFilesCount: w.uncommittedFilesCount,
      repoName: repo.repoName,
      force: false
    }));

    batchSelection.clear();
    batchSelection.selectRepo(targets);
    batchDeleteSummary = null;
    batchDeleteError = null;
    isBatchDeleteModalOpen = true;
  }

  function handleQuickWorktreeCreated(event: CustomEvent<{ repoPath: string; worktreeInfo?: WorktreeInfo }>) {
    const { repoPath, worktreeInfo } = event.detail;
    if (worktreeInfo) {
      scannedRepos.update((repos) =>
        repos.map((r) => (r.repoPath === repoPath ? { ...r, worktrees: [...r.worktrees, worktreeInfo] } : r))
      );
    } else {
      refreshWorktrees();
    }
  }

  // Branch Cleaner: scan / toggle view / batch delete
  async function refreshBranches() {
    const paths = $scannedRepos.map((r) => r.repoPath);
    if (paths.length === 0) {
      scannedBranches.set([]);
      return;
    }

    isScanningBranches.set(true);
    branchScanError.set(null);

    try {
      const entries = await invokeTauri<BranchStatusEntry[]>('scan_branches_for_cleanup', { repoPaths: paths });
      scannedBranches.set(entries || []);
    } catch (err: any) {
      branchScanError.set(err?.message || err?.toString() || 'Failed to scan branches');
    } finally {
      isScanningBranches.set(false);
    }
  }

  function handleSetView(view: 'worktrees' | 'branches') {
    if (activeView === view) return;
    highlightedWorktreePath.set(null);
    activeView = view;
    if (activeView === 'branches') {
      refreshBranches();
    }
  }

  function handleRequestCheckoutBranch(event: CustomEvent<BranchStatusEntry>) {
    const branch = event.detail;

    // The branch is already checked out somewhere: there is nothing to check out, so the useful
    // action is taking the user to that worktree instead of creating a second one.
    // Reset filters and query so the target worktree is guaranteed to be rendered in the view.
    if (branch.checkedOutWorktreePath) {
      searchFilter.set('');
      selectedStatusFilter.set('ALL');
      selectedAccountFilter.set('ALL');
      highlightedWorktreePath.set(branch.checkedOutWorktreePath);
      activeView = 'worktrees';
      return;
    }

    initialRepoForNewWorktree = branch.repoPath;
    initialExistingBranchForNewWorktree = branch.name;
    newWorktreeError = null;
    isNewWorktreeOpen = true;
  }

  function handleOpenBranchBatchDeleteModal() {
    branchBatchDeleteSummary = null;
    branchBatchDeleteError = null;
    isBranchBatchDeleteModalOpen = true;
  }

  async function handleConfirmBranchBatchDelete(event: CustomEvent<{ targets: BranchDeleteTarget[]; force: boolean }>) {
    const { targets } = event.detail;
    isBranchBatchDeleting = true;
    branchBatchDeleteError = null;

    try {
      const summary = await invokeTauri<BranchBatchDeleteSummary>('remove_branches_batch', { targets });
      branchBatchDeleteSummary = summary;

      // `deletedBranches` is repo-qualified (see `DeletedBranchRef`), so it can be matched
      // directly against `scannedBranches` without ambiguity even if the same branch name was
      // targeted in more than one repo in this batch.
      const deletedKeys = new Set(summary.deletedBranches.map((d) => `${d.repoPath}::${d.branchName}`));
      if (deletedKeys.size > 0) {
        scannedBranches.update((existing) =>
          existing.filter((b) => !deletedKeys.has(`${b.repoPath}::${b.name}`))
        );
      }

      const remainingKeys = new Set($scannedBranches.map((b) => `${b.repoPath}::${b.name}`));
      branchSelection.prune(remainingKeys);

      if (summary && !summary.errors.some((e) => e.kind === 'failed')) {
        branchSelection.clear();
        isBranchBatchDeleteModalOpen = false;
        const skippedNote = summary.skippedCount > 0 ? ` ${summary.skippedCount} protected branch(es) skipped.` : '';
        notifications.success('Branches deleted', `Successfully deleted ${summary.deletedCount} branch(es).${skippedNote}`);
      }
    } catch (err: any) {
      branchBatchDeleteError = err?.message || err?.toString() || 'Failed to delete branches';
    } finally {
      isBranchBatchDeleting = false;
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

<main class="w-full h-screen flex flex-col bg-neutral-900 text-neutral-100 overflow-hidden select-none relative">
  <Header
    {activeView}
    isRefreshing={activeView === 'worktrees' ? $isScanning : $isScanningBranches}
    onRefresh={activeView === 'worktrees' ? refreshWorktrees : () => refreshBranches()}
    onSetView={handleSetView}
    onOpenGhModal={() => (isGhModalOpen = true)}
    onOpenNewWorktreeModal={() => handleOpenNewWorktree()}
    onOpenSettingsModal={() => (isSettingsModalOpen = true)}
  />

  {#if activeView === 'worktrees'}
    {#if $scanError}
      <div class="p-2 mx-3 mt-2 rounded bg-rose-950/60 border border-rose-800/50 text-rose-300 text-xs">
        {$scanError}
      </div>
    {/if}

    <AccountFilterBar />

    <WorktreeList
      on:openPath={handleOpenPath}
      on:openEditor={handleOpenEditor}
      on:openTerminal={handleOpenTerminal}
      on:requestSwitchBranch={handleRequestSwitchBranch}
      on:newWorktreeForRepo={(e) => handleOpenNewWorktree(e.detail)}
      on:switchGhAccount={(e) => handleSwitchGhAccount(e.detail)}
      on:openSettings={() => (isSettingsModalOpen = true)}
      on:requestDelete={handleRequestDelete}
      on:cleanAllOrphans={handleCleanAllOrphans}
      on:worktreeCreated={handleQuickWorktreeCreated}
    />

    <!-- Floating Batch Action Bar -->
    <BatchActionBar on:openBatchDeleteModal={handleOpenBatchDeleteModal} />

    <!-- Batch Delete Modal -->
    <BatchDeleteModal
      isOpen={isBatchDeleteModalOpen}
      targets={$selectedWorktreeList}
      isDeleting={isBatchDeleting}
      summary={batchDeleteSummary}
      errorMessage={batchDeleteError}
      on:close={() => (isBatchDeleteModalOpen = false)}
      on:confirmDelete={handleConfirmBatchDelete}
    />
  {:else}
    <BranchFilterBar />

    <BranchList on:requestCheckout={handleRequestCheckoutBranch} />

    <!-- Floating Branch Batch Action Bar -->
    <BranchActionBar on:openBatchDeleteModal={handleOpenBranchBatchDeleteModal} />

    <!-- Branch Batch Delete Modal -->
    <BranchBatchDeleteModal
      isOpen={isBranchBatchDeleteModalOpen}
      targets={$selectedBranchList}
      isDeleting={isBranchBatchDeleting}
      summary={branchBatchDeleteSummary}
      errorMessage={branchBatchDeleteError}
      on:close={() => (isBranchBatchDeleteModalOpen = false)}
      on:confirmDelete={handleConfirmBranchBatchDelete}
    />
  {/if}

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
    initialExistingBranch={initialExistingBranchForNewWorktree}
    isCreating={isCreatingWorktree}
    errorMessage={newWorktreeError}
    onSuggestPath={handleSuggestPath}
    onFetchBranches={handleFetchBranchesForRepo}
    on:close={() => (isNewWorktreeOpen = false)}
    on:create={handleConfirmCreateWorktree}
  />

  <RemoveWorktreeModal
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

  <ToastContainer />
</main>

