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
  import ReleaseBranchModal from './lib/components/ReleaseBranchModal.svelte';
  import ToastContainer from './lib/components/ToastContainer.svelte';
  import { scannedRepos, filteredRepos, isScanning, isPinned, scanError, highlightedWorktreePath, searchFilter, openWorktreeActionCount } from './lib/stores/worktrees';
  import { scannedBranches, isScanningBranches, branchScanError } from './lib/stores/branchCleaner';
  import { ghAccounts, activeGhAccount, isGhLoading } from './lib/stores/ghAuth';
  import { appConfig, selectedAccountFilter, selectedStatusFilter } from './lib/stores/appConfig';
  import { batchSelection, selectedWorktreeList } from './lib/stores/batchSelection';
  import { branchSelection, selectedBranchList } from './lib/stores/branchSelection';
  import { installedEditors } from './lib/stores/editors';
  import { notifications } from './lib/stores/notifications';
  import { invoke } from '@tauri-apps/api/core';
  import { toErrorMessage } from './lib/utils/errors';
  import { isPathWithin, normalizePath } from './lib/utils/paths';
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
    RepoScannedEvent,
    RepoScanFailedEvent,
    ScanCompleteEvent,
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

  // Release Branch Modal state (frees a branch checked out in a worktree so it can be deleted)
  let isReleaseBranchModalOpen: boolean = false;
  let selectedBranchForRelease: BranchStatusEntry | null = null;

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

  async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      return await invoke<T>(cmd, args);
    } else {
      // Mock data for web development/preview outside Tauri
      const mockArgs = args as Record<string, any>;
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
        return mockArgs.config as unknown as T;
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
          repoPath: mockArgs.repoPath || '',
          worktreePath: mockArgs.worktreePath || '',
          currentBranch: 'main',
          branches: [
            { name: 'main', shortName: 'main', isRemote: false, isCurrent: true, isLockedByOther: false, lastCommitSha: 'a1b2c3d', lastCommitMessage: 'feat: add UI' },
            { name: 'develop', shortName: 'develop', isRemote: false, isCurrent: false, isLockedByOther: false, lastCommitSha: 'b2c3d4e', lastCommitMessage: 'chore: update packages' }
          ]
        } as unknown as T;
      }
      if (cmd === 'suggest_worktree_path') {
        const repo = String(mockArgs.repoPath || 'repo').split(/[\/\\]/).pop() || 'repo';
        const clean = String(mockArgs.branchName || 'branch').replace(/[\/\\]/g, '-');
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
        const repoPaths: string[] = mockArgs.repoPaths || [];
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
        isPinned.set(cfg.isPinned ?? false);
      }
    } catch (err: unknown) {
      notifications.error('Configuration Error', toErrorMessage(err, 'Failed to load app config'));
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
    } catch (err: unknown) {
      notifications.error('Failed to save configuration', toErrorMessage(err));
    } finally {
      isSavingConfig = false;
    }
  }

  // Each refresh gets a new id; scan events carrying an older id belong to a superseded scan.
  // The previous list stays on screen until `scan-complete`, which then drops repos this scan
  // no longer found (a repo that failed to scan keeps its last known state).
  let currentScanId = 0;
  let reposSeenInScan = new Set<string>();
  let reposFailedInScan: RepoScanFailedEvent[] = [];

  async function refreshWorktrees() {
    const scanId = ++currentScanId;
    reposSeenInScan = new Set();
    reposFailedInScan = [];
    isScanning.set(true);
    scanError.set(null);

    try {
      await invokeTauri('scan_worktrees', { scanId });
    } catch (err: unknown) {
      if (scanId !== currentScanId) return;
      scanError.set(toErrorMessage(err, 'Failed to scan worktrees'));
      isScanning.set(false);
    }
  }

  function handleRepoScanned(event: RepoScannedEvent) {
    if (event.scanId !== currentScanId) return;
    const repo = event.repo;
    reposSeenInScan.add(repo.repoPath);
    scannedRepos.update((repos) => {
      const existingIndex = repos.findIndex((r) => r.repoPath === repo.repoPath);
      if (existingIndex >= 0) {
        const updated = [...repos];
        updated[existingIndex] = repo;
        return updated;
      }
      return [...repos, repo];
    });
  }

  function handleRepoScanFailed(event: RepoScanFailedEvent) {
    if (event.scanId !== currentScanId) return;
    // Scanned paths are the canonical repo root, which may differ in form from the discovered path.
    const previous = $scannedRepos.find((r) => normalizePath(r.repoPath) === normalizePath(event.repoPath));
    reposSeenInScan.add(previous?.repoPath ?? event.repoPath);
    reposFailedInScan = [...reposFailedInScan, event];
  }

  function handleScanComplete(event: ScanCompleteEvent) {
    if (event.scanId !== currentScanId) return;
    scannedRepos.update((repos) => repos.filter((r) => reposSeenInScan.has(r.repoPath)));
    isScanning.set(false);
    if (reposFailedInScan.length > 0) {
      const details = reposFailedInScan.map((f) => `${f.repoPath}: ${f.error}`).join(' | ');
      scanError.set(`Could not scan ${reposFailedInScan.length} repository(ies): ${details}`);
    }
  }

  async function refreshGhAccounts() {
    isGhLoading.set(true);
    try {
      const accounts = await invokeTauri<GhAccount[]>('get_gh_accounts');
      ghAccounts.set(accounts || []);
      const active = accounts?.find(a => a.active);
      activeGhAccount.set(active ? active.username : null);
    } catch (err: unknown) {
      notifications.error('GitHub CLI Error', toErrorMessage(err, 'Failed to load GH accounts'));
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
    } catch (err: unknown) {
      notifications.error('Failed to switch GitHub account', toErrorMessage(err));
    } finally {
      isGhLoading.set(false);
    }
  }

  /** The repo owning `targetPath`: the deepest repo or worktree root that contains it. */
  function findRepoForPath(targetPath: string): RepositoryWorktrees | undefined {
    let best: { repo: RepositoryWorktrees; depth: number } | undefined;
    for (const repo of $scannedRepos) {
      for (const root of [repo.repoPath, ...repo.worktrees.map((w) => w.path)]) {
        if (!isPathWithin(targetPath, root)) continue;
        const depth = normalizePath(root).length;
        if (!best || depth > best.depth) best = { repo, depth };
      }
    }
    return best?.repo;
  }

  // Smart Context Switcher check before action
  async function ensureMatchingAccountForPath(targetPath: string) {
    if (!$appConfig.autoSwitchAccount) return;
    const account = findRepoForPath(targetPath)?.associatedAccount;
    if (!account || account.toLowerCase() === ($activeGhAccount ?? '').toLowerCase()) return;
    try {
      await invokeTauri('switch_gh_account', { username: account });
      await refreshGhAccounts();
      notifications.info('GitHub account switched', `Active gh account is now ${account}.`);
    } catch (err: unknown) {
      notifications.error(`Could not switch gh account to ${account}`, toErrorMessage(err));
    }
  }

  function handleOpenPath(event: CustomEvent<string>) {
    const path = event.detail;
    ensureMatchingAccountForPath(path);
    invokeTauri('open_path', { path }).catch((err: unknown) => {
      notifications.error('Could not open folder', toErrorMessage(err));
    });
  }

  function handleOpenEditor(event: CustomEvent<{ editor: SupportedEditor; path: string }>) {
    const { editor, path } = event.detail;
    // Account switch runs concurrently, not awaited: it must not delay the editor launch itself.
    ensureMatchingAccountForPath(path);
    invokeTauri('open_in_editor', { editor, path })
      .then(() => notifications.success(`Opening in ${editor}`, path, 2500))
      .catch((err: unknown) => {
        notifications.error(`Could not open editor (${editor})`, toErrorMessage(err));
      });
  }

  function handleOpenTerminal(event: CustomEvent<{ terminal: SupportedTerminal; path: string }>) {
    const { terminal, path } = event.detail;
    if (terminal === 'none') return;
    ensureMatchingAccountForPath(path);
    invokeTauri('open_in_terminal', { terminal, path })
      .then(() => notifications.success(`Opening ${terminal} terminal`, path, 2500))
      .catch((err: unknown) => {
        notifications.error(`Could not open terminal (${terminal})`, toErrorMessage(err));
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
    } catch (err: unknown) {
      branchSwitchError = toErrorMessage(err, 'Failed to list branches');
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
    } catch (err: unknown) {
      branchSwitchError = toErrorMessage(err, 'Failed to switch branch');
    } finally {
      isSwitchingBranch = false;
    }
  }

  function handleWorktreeUpdated(event: CustomEvent<WorktreeInfo>) {
    const freshInfo = event.detail;
    if (!freshInfo) return;
    selectedWorktreeForBranchSwitch = freshInfo;
    const repoPath = selectedRepoForBranchSwitch;
    scannedRepos.update((repos) =>
      repos.map((r) =>
        r.repoPath === repoPath
          ? { ...r, worktrees: r.worktrees.map((w) => (w.path === freshInfo.path ? freshInfo : w)) }
          : r
      )
    );
  }

  function handleWorktreeDiscarded(event: CustomEvent<{ worktree: WorktreeInfo; repoPath: string }>) {
    const { worktree: updatedWt, repoPath } = event.detail;
    scannedRepos.update((repos) =>
      repos.map((r) =>
        r.repoPath === repoPath
          ? { ...r, worktrees: r.worktrees.map((w) => (w.path === updatedWt.path ? updatedWt : w)) }
          : r
      )
    );
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
    } catch (err: unknown) {
      newWorktreeError = toErrorMessage(err, 'Failed to create worktree');
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
      notifications.success('Worktree removed', `${worktree.path} removed. Branch ${worktree.branch?.replace('refs/heads/', '') ?? '(detached)'} was kept.`);

      scannedRepos.update((repos) =>
        repos
          .map((r) =>
            r.repoPath === repoPath
              ? { ...r, worktrees: r.worktrees.filter((w) => w.path !== worktree.path) }
              : r
          )
          .filter((r) => r.worktrees.length > 0)
      );
    } catch (err: unknown) {
      notifications.error('Error deleting worktree', toErrorMessage(err));
    } finally {
      isDeletingWorktree = false;
    }
  }

  function handleOpenBatchDeleteModal() {
    batchSelection.syncWithScan($scannedRepos);
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
    } catch (err: unknown) {
      batchDeleteError = toErrorMessage(err, 'Failed to remove worktrees');
    } finally {
      isBatchDeleting = false;
    }
  }

  function handleBatchItemDeleted(event: CustomEvent<{ worktreePath: string; repoPath: string }>) {
    const { worktreePath, repoPath } = event.detail;
    scannedRepos.update((repos) =>
      repos
        .map((r) =>
          r.repoPath === repoPath
            ? { ...r, worktrees: r.worktrees.filter((w) => w.path !== worktreePath) }
            : r
        )
        .filter((r) => r.worktrees.length > 0)
    );
    batchSelection.deselect(worktreePath);
    notifications.success('Worktree removed', `Force unlocked and removed: ${worktreePath}`);
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

    const orphanPaths = new Set(targets.map((t) => t.worktreePath));
    const otherSelected = $selectedWorktreeList.filter((t) => !orphanPaths.has(t.worktreePath)).length;
    if (
      otherSelected > 0 &&
      !window.confirm(`Replace your current selection (${otherSelected} other worktree(s)) with the ${targets.length} orphan(s) of ${repo.repoName}?`)
    ) {
      return;
    }

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
    } catch (err: unknown) {
      branchScanError.set(toErrorMessage(err, 'Failed to scan branches'));
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

  function handleRequestReleaseBranch(event: CustomEvent<BranchStatusEntry>) {
    selectedBranchForRelease = event.detail;
    isReleaseBranchModalOpen = true;
  }

  async function handleBranchReleased(event: CustomEvent<{ repoPath: string; branchName: string; deletedWorktreePath?: string }>) {
    const { repoPath, branchName, deletedWorktreePath } = event.detail;
    isReleaseBranchModalOpen = false;
    selectedBranchForRelease = null;

    scannedBranches.update((existing) => existing.filter((b) => !(b.repoPath === repoPath && b.name === branchName)));
    branchSelection.prune(new Set($scannedBranches.map((b) => `${b.repoPath}::${b.name}`)));

    if (deletedWorktreePath) {
      scannedRepos.update((repos) =>
        repos
          .map((r) =>
            r.repoPath === repoPath
              ? { ...r, worktrees: r.worktrees.filter((w) => w.path !== deletedWorktreePath) }
              : r
          )
          .filter((r) => r.worktrees.length > 0)
      );
      batchSelection.deselect(deletedWorktreePath);
    }

    notifications.success('Branch released', `${branchName} was deleted.`);
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
    } catch (err: unknown) {
      branchBatchDeleteError = toErrorMessage(err, 'Failed to delete branches');
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
    } catch (err: unknown) {
      // Non-fatal: default fallback editors remain active in store
    }
  }

  async function handleTogglePin() {
    const pinned = !$isPinned;
    isPinned.set(pinned);
    try {
      const saved = await invokeTauri<AppConfig>('save_app_config', { config: { ...$appConfig, isPinned: pinned } });
      if (saved) appConfig.set(saved);
    } catch (err: unknown) {
      notifications.error('Could not save pin state', toErrorMessage(err));
    }
  }

  // "Busy" keeps the panel visible on focus loss: blurring mid-modal or mid-action would hide the
  // dialog or the progress/result the user is waiting for.
  $: visibleWorktreePaths = new Set($filteredRepos.flatMap((r) => r.worktrees.map((w) => w.path)));

  $: isAnyModalOpen =
    isDeleteModalOpen ||
    isBatchDeleteModalOpen ||
    isBranchBatchDeleteModalOpen ||
    isReleaseBranchModalOpen ||
    isGhModalOpen ||
    isSettingsModalOpen ||
    isBranchSwitcherOpen ||
    isNewWorktreeOpen;
  $: isActionRunning =
    isDeletingWorktree || isBatchDeleting || isBranchBatchDeleting || isSavingConfig || isSwitchingBranch || isCreatingWorktree || $openWorktreeActionCount > 0;
  $: syncPanelState($isPinned, isAnyModalOpen || isActionRunning);

  function syncPanelState(pinned: boolean, busy: boolean) {
    invokeTauri('set_panel_state', { pinned, busy }).catch((err: unknown) => {
      notifications.error('Window state error', toErrorMessage(err, 'Failed to sync pin/always-on-top state'));
    });
  }

  // Captured before any modal's own Escape handler runs, so the Escape that closes a modal does
  // not also hide the panel.
  let modalOpenAtEscape = false;

  function handleWindowKeydownCapture(event: KeyboardEvent) {
    if (event.key === 'Escape') modalOpenAtEscape = isAnyModalOpen;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    // Inline widgets that consume Escape themselves call preventDefault.
    if (event.key !== 'Escape' || modalOpenAtEscape || event.defaultPrevented) return;
    invokeTauri('hide_panel').catch(() => {
      // Hiding is best-effort; the tray and Alt+Space still toggle the panel.
    });
  }

  let unlistenRepo: (() => void) | null = null;
  let unlistenRepoFailed: (() => void) | null = null;
  let unlistenDone: (() => void) | null = null;
  let unlistenTrayRefresh: (() => void) | null = null;

  onMount(async () => {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlistenRepo = await listen<RepoScannedEvent>('repo-scanned', (event) => handleRepoScanned(event.payload));
        unlistenRepoFailed = await listen<RepoScanFailedEvent>('repo-scan-failed', (event) => handleRepoScanFailed(event.payload));
        unlistenDone = await listen<ScanCompleteEvent>('scan-complete', (event) => handleScanComplete(event.payload));

        unlistenTrayRefresh = await listen('refresh-worktrees', () => {
          refreshWorktrees();
        });
      } catch (err: unknown) {
        notifications.error('Event Listener Error', toErrorMessage(err, 'Failed to setup scan event listeners'));
      }
    }

    await loadAppConfig();
    refreshInstalledEditors();
    refreshWorktrees();
    refreshGhAccounts();
  });

  onDestroy(() => {
    if (unlistenRepo) unlistenRepo();
    if (unlistenRepoFailed) unlistenRepoFailed();
    if (unlistenDone) unlistenDone();
    if (unlistenTrayRefresh) unlistenTrayRefresh();
  });
</script>

<svelte:window on:keydown|capture={handleWindowKeydownCapture} on:keydown={handleWindowKeydown} />

<main class="w-full h-screen flex flex-col bg-neutral-900 text-neutral-100 overflow-hidden select-none relative">
  <Header
    {activeView}
    isRefreshing={activeView === 'worktrees' ? $isScanning : $isScanningBranches}
    onRefresh={activeView === 'worktrees' ? refreshWorktrees : () => refreshBranches()}
    onSetView={handleSetView}
    onOpenGhModal={() => (isGhModalOpen = true)}
    onOpenNewWorktreeModal={() => handleOpenNewWorktree()}
    onOpenSettingsModal={() => (isSettingsModalOpen = true)}
    onTogglePin={handleTogglePin}
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
      on:worktreeDiscarded={handleWorktreeDiscarded}
      on:cleanAllOrphans={handleCleanAllOrphans}
      on:worktreeCreated={handleQuickWorktreeCreated}
    />

    <!-- Floating Batch Action Bar -->
    <BatchActionBar on:openBatchDeleteModal={handleOpenBatchDeleteModal} />

    <!-- Batch Delete Modal -->
    <BatchDeleteModal
      isOpen={isBatchDeleteModalOpen}
      targets={$selectedWorktreeList}
      visiblePaths={visibleWorktreePaths}
      isDeleting={isBatchDeleting}
      summary={batchDeleteSummary}
      errorMessage={batchDeleteError}
      on:close={() => (isBatchDeleteModalOpen = false)}
      on:confirmDelete={handleConfirmBatchDelete}
      on:itemDeleted={handleBatchItemDeleted}
    />
  {:else}
    <BranchFilterBar />

    <BranchList on:requestCheckout={handleRequestCheckoutBranch} on:requestRelease={handleRequestReleaseBranch} />

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

    <!-- Release Branch Modal: frees a branch checked out in a worktree so it can be deleted -->
    <ReleaseBranchModal
      isOpen={isReleaseBranchModalOpen}
      branch={selectedBranchForRelease}
      on:close={() => (isReleaseBranchModalOpen = false)}
      on:worktreeDiscarded={handleWorktreeDiscarded}
      on:released={handleBranchReleased}
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
    on:worktreeUpdated={handleWorktreeUpdated}
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
    onCheckTargetOccupied={(targetPath) => invokeTauri<boolean>('is_worktree_target_occupied', { targetPath })}
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

