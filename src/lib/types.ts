export interface WorktreeInfo {
  path: string;
  head: string;
  branch: string | null;
  bare: boolean;
  locked: string | null;
  prunable: string | null;
  isOrphaned: boolean;
  orphanReason?: string;
  isDirty?: boolean;
  uncommittedFilesCount?: number;
  lastCommitMessage?: string;
  lastCommitAuthor?: string;
  lastCommitDate?: string;
}

export interface RepositoryWorktrees {
  repoPath: string;
  repoName: string;
  worktrees: WorktreeInfo[];
}

export interface GhAccount {
  username: string;
  active: boolean;
  scopes?: string[];
  host: string;
}

export interface ScanResult {
  repositories: RepositoryWorktrees[];
  totalWorktrees: number;
  totalOrphaned: number;
}
