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

/**
 * `skipped` is a safety guard refusing a target on purpose; `failed` is the git call itself
 * going wrong. Only `failed` means the batch needs the user's attention.
 */
export type BatchItemErrorKind = 'skipped' | 'failed';

export interface BatchItemError {
  worktreePath: string;
  error: string;
  kind: BatchItemErrorKind;
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
  /** Status of the checked-out branch, so a worktree row can name why it is disposable. */
  isBranchMerged?: boolean;
  isBranchRemoteGone?: boolean;
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

/** Payloads of the `scan_worktrees` events; `scanId` identifies the refresh that emitted them. */
export interface RepoScannedEvent {
  scanId: number;
  repo: RepositoryWorktrees;
}

export interface RepoScanFailedEvent {
  scanId: number;
  repoPath: string;
  error: string;
}

export interface ScanCompleteEvent {
  scanId: number;
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
  /** Absent in configs saved before pinning was persisted; treat as false. */
  isPinned?: boolean;
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
  worktreeInfo: WorktreeInfo;
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
  worktreeInfo: WorktreeInfo;
}

export type BranchStatusFilterType = 'ALL' | 'MERGED' | 'REMOTE_GONE' | 'PROTECTED';

export interface BranchStatusEntry {
  repoPath: string;
  name: string;
  isCurrent: boolean;
  isDefault: boolean;
  isMerged: boolean;
  isRemoteGone: boolean;
  isCheckedOut: boolean;
  checkedOutWorktreePath?: string | null;
  lastCommitSha?: string | null;
  lastCommitMessage?: string | null;
}

export interface BranchDeleteTarget {
  repoPath: string;
  branchName: string;
  force: boolean;
  repoName?: string;
  isMerged?: boolean;
  isRemoteGone?: boolean;
}

export interface BranchBatchItemError {
  branchName: string;
  error: string;
  kind: BatchItemErrorKind;
}

/** Identifies a deleted branch by repo + name — a bare name is ambiguous when a batch spans repos. */
export interface DeletedBranchRef {
  repoPath: string;
  branchName: string;
}

export interface BranchBatchDeleteSummary {
  totalRequested: number;
  deletedCount: number;
  skippedCount: number;
  deletedBranches: DeletedBranchRef[];
  errors: BranchBatchItemError[];
}
