export type ViewDensity = 'compact' | 'detailed';

export interface WorktreeInfo {
  path: string;
  head: string;
  branch: string | null;
  bare: boolean;
  locked: string | null;
  prunable: string | null;
  isMain?: boolean;
  isOrphaned: boolean;
  orphanReason?: string;
  isDirty?: boolean;
  uncommittedFilesCount?: number;
  lastCommitMessage?: string;
  lastCommitAuthor?: string;
  lastCommitDate?: string;
}

export interface WorktreeDiffSummary {
  filesChanged: number;
  insertions: number;
  deletions: number;
  summaryText: string;
  modifiedFiles: string[];
}

export interface RepositoryWorktrees {
  repoPath: string;
  repoName: string;
  associatedAccount?: string | null;
  watchFolderPath?: string | null;
  worktrees: WorktreeInfo[];
}

export interface WatchFolder {
  id: string;
  path: string;
  accountUsername: string | null;
  enabled: boolean;
  maxDepth: number; // 1 to 5
}

export interface AppConfig {
  version: number;
  watchFolders: WatchFolder[];
  autoSwitchAccount: boolean;
  defaultEditor: SupportedEditor;
}

export interface GhAccount {
  username: string;
  active: boolean;
  scopes?: string[];
  host: string;
}


export type SupportedEditor = 'antigravity' | 'vscode' | 'cursor' | 'windsurf' | 'explorer' | 'wt';

export interface EditorInfo {
  id: SupportedEditor;
  name: string;
  isAvailable: boolean;
  iconName: string;
}

export interface BranchEntry {
  name: string;
  shortName: string;
  isRemote: boolean;
  isCurrent: boolean;
  isLockedByOther: boolean;
  lockedWorktreePath?: string;
  lastCommitSha?: string;
  lastCommitMessage?: string;
}

export interface WorktreeBranchesResponse {
  repoPath: string;
  worktreePath: string;
  currentBranch: string | null;
  branches: BranchEntry[];
}

export interface CheckoutBranchResult {
  success: boolean;
  newBranch: string;
  headSha: string;
  message: string;
}

export interface SuggestWorktreePathResult {
  suggestedPath: string;
  alreadyExists: boolean;
}

export interface CreateWorktreeResult {
  success: boolean;
  worktreePath: string;
  branchName: string;
  message: string;
}
