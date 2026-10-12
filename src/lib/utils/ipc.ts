import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  BatchDeleteTarget,
  BranchBatchDeleteSummary,
  BranchDeleteTarget,
  BranchStatusEntry,
  CreateWorktreeResult,
  SuggestWorktreePathResult,
  WorktreeBranchesResponse,
  WorktreeDiffSummary,
  WorktreeInfo
} from '../types';

export const isTauriEnvironment =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/**
 * Unified Tauri IPC wrapper. In desktop Tauri mode, routes directly to `@tauri-apps/api/core`.
 * When running in a standard web browser (preview / mock mode), supplies safe deterministic mock responses.
 */
export async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauriEnvironment) {
    return await invoke<T>(cmd, args);
  }

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
      worktreePath: mockArgs.targetPath,
      branchName: mockArgs.newBranchName || mockArgs.baseBranch,
      message: 'Worktree created',
      worktreeInfo: {
        path: mockArgs.targetPath,
        head: '0000000',
        branch: mockArgs.newBranchName || mockArgs.baseBranch,
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
      newBranch: mockArgs.targetBranch,
      headSha: '1a2b3c4',
      message: 'Checked out',
      worktreeInfo: {
        path: mockArgs.worktreePath,
        head: '1a2b3c4',
        branch: mockArgs.targetBranch,
        bare: false,
        locked: null,
        prunable: null,
        isOrphaned: false,
        isDirty: false,
        uncommittedFilesCount: 0
      }
    } as unknown as T;
  }

  if (
    cmd === 'open_in_editor' ||
    cmd === 'open_in_terminal' ||
    cmd === 'switch_gh_account' ||
    cmd === 'remove_worktree' ||
    cmd === 'detach_worktree_head' ||
    cmd === 'git_unlock_worktree'
  ) {
    return { success: true } as unknown as T;
  }

  if (cmd === 'git_stash_worktree' || cmd === 'git_discard_worktree_changes') {
    return {
      path: mockArgs.worktreePath || '',
      head: '1a2b3c4',
      branch: 'main',
      bare: false,
      locked: null,
      prunable: null,
      isMain: false,
      isOrphaned: false,
      isDirty: false,
      uncommittedFilesCount: 0
    } as unknown as T;
  }

  if (cmd === 'get_worktree_diff_summary') {
    return {
      files: [],
      filesCount: 0,
      insertions: 0,
      deletions: 0
    } as unknown as T;
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
    const targets = (mockArgs.targets || []) as BranchDeleteTarget[];
    return {
      totalRequested: targets.length,
      deletedCount: targets.length,
      skippedCount: 0,
      deletedBranches: targets.map((t) => ({ repoPath: t.repoPath, branchName: t.branchName })),
      errors: []
    } as unknown as T;
  }

  if (cmd === 'get_quick_action_preview') {
    return {
      affectedReposCount: 1,
      impactItems: [],
      safetyWarnings: []
    } as unknown as T;
  }

  if (cmd === 'execute_quick_action') {
    return {
      affectedReposCount: 1,
      totalCleanedItems: 0,
      errors: []
    } as unknown as T;
  }

  return null as unknown as T;
}
