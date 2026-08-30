export type ViewDensity = 'compact' | 'detailed';

export type StatusFilterType = 'ALL' | 'DIRTY' | 'MULTI_WT' | 'ORPHANS' | 'CLEAN';

export interface StatusFilterCounts {
  all: number;
  dirty: number;
  multiWt: number;
  orphans: number;
  clean: number;
}

export interface BatchDeleteTarget {
  repoPath: string;
  worktreePath: string;
  force: boolean;
  branch?: string | null;
  isDirty?: boolean;
  uncommittedFilesCount?: number;
  repoName?: string;
}

export interface BatchItemError {
  worktreePath: string;
  error: string;
}

export interface BatchDeleteSummary {
  totalRequested: number;
  deletedCount: number;
  skippedCount: number;
  deletedPaths: string[];
  errors: BatchItemError[];
}

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

export type SupportedEditor = 'vscode' | 'antigravity' | 'cursor' | 'windsurf' | 'explorer' | 'wt';

export type SupportedTerminal = 'wt' | 'powershell' | 'cmd' | 'git-bash' | 'agy' | 'none';

export interface EditorInfo {
  id: SupportedEditor;
  name: string;
  isAvailable: boolean;
  iconName: string;
}

export interface TerminalInfo {
  id: SupportedTerminal;
  name: string;
  isAvailable: boolean;
  iconName: string;
}

export interface GhAccount {
  username: string;
  active: boolean;
  scopes?: string[];
  host: string;
}

export interface AppConfig {
  version: number;
  watchFolders: WatchFolder[];
  autoSwitchAccount: boolean;
  defaultEditor: SupportedEditor;
  defaultTerminal: SupportedTerminal;
  showTerminalButton: boolean;
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
