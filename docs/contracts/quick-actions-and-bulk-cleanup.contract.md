# Formal Contract: Quick Actions & Bulk Cleanup

**Feature Slug:** `quick-actions-and-bulk-cleanup`  
**Target Systems:**
- Rust Backend: `src-tauri/src/services/quick_actions.rs`, `src-tauri/src/commands/quick_actions.rs`
- TypeScript Frontend: `src/lib/types.ts`, `src/lib/stores/quickActions.ts`, `src/lib/components/QuickActionsModal.svelte`

---

## 1. Public Interfaces & Signatures

### 1.1 Backend (Rust)

#### Data Structures (`src-tauri/src/services/quick_actions.rs`)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionPreset {
    SyncDefaultBranch,
    DiscardUncommitted,
    NukeWorktrees,
    CleanBranches,
    TotalFreshStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionScope {
    SingleRepo {
        #[serde(rename = "repoPath", alias = "repo_path")]
        repo_path: String,
    },
    RepoList {
        #[serde(rename = "repoPaths", alias = "repo_paths")]
        repo_paths: Vec<String>,
    },
    AllWatchedRepos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionPreview {
    pub preset: QuickActionPreset,
    pub repos_count: usize,
    pub worktrees_to_remove: Vec<String>,
    pub worktrees_to_discard: Vec<String>,
    pub dirty_files_count: usize,
    pub branches_to_delete: Vec<String>,
    pub unpushed_branches_count: usize,
    pub total_unpushed_commits: usize,
    pub protected_default_branches: Vec<String>,
    #[serde(default)]
    pub repos_to_switch_to_default: Vec<String>,
    #[serde(default)]
    pub repos_ready_to_pull: Vec<String>,
    #[serde(default)]
    pub repos_skipped_dirty_or_locked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionResult {
    pub preset: QuickActionPreset,
    pub success: bool,
    pub worktrees_removed: usize,
    pub worktrees_discarded: usize,
    pub branches_deleted: usize,
    #[serde(default)]
    pub repos_fetched: usize,
    #[serde(default)]
    pub repos_switched_to_default: usize,
    #[serde(default)]
    pub repos_pulled: usize,
    #[serde(default)]
    pub repos_up_to_date: usize,
    #[serde(default)]
    pub repos_skipped: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
```

#### Service Interface (`QuickActionsService`)
```rust
pub struct QuickActionsService;

impl QuickActionsService {
    /// Resolves target repository root directories based on scope and watch folders.
    pub fn resolve_repos(
        scope: &QuickActionScope,
        watch_folders: &[crate::services::config::WatchFolder],
    ) -> Result<Vec<std::path::PathBuf>, String>;

    /// Computes a dry-run pre-flight audit for the selected preset and scope.
    pub fn compute_preview(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[crate::services::config::WatchFolder],
    ) -> Result<QuickActionPreview, String>;

    /// Executes the atomic pipeline across repositories using default config options.
    pub fn execute_action(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[crate::services::config::WatchFolder],
    ) -> Result<QuickActionResult, String>;

    /// Executes the atomic pipeline honoring full `AppConfig` (including `auto_switch_account`
    /// sequential GitHub CLI credential grouping for `SyncDefaultBranch`).
    pub fn execute_action_with_config(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        config: &crate::services::config::AppConfig,
    ) -> Result<QuickActionResult, String>;
}
```

#### Tauri IPC Commands (`src-tauri/src/commands/quick_actions.rs`)
```rust
#[tauri::command]
pub async fn get_quick_action_preview(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionPreview, String>;

#[tauri::command]
pub async fn execute_quick_action(
    preset: QuickActionPreset,
    scope: QuickActionScope,
) -> Result<QuickActionResult, String>;
```

---

### 1.2 Frontend (TypeScript Contracts: `src/lib/types.ts`)

```typescript
export type QuickActionPreset =
  | 'syncDefaultBranch'
  | 'discardUncommitted'
  | 'nukeWorktrees'
  | 'cleanBranches'
  | 'totalFreshStart';

export type QuickActionScope =
  | { singleRepo: { repoPath: string } }
  | { repoList: { repoPaths: string[] } }
  | 'allWatchedRepos';

export interface QuickActionPreview {
  preset: QuickActionPreset;
  reposCount: number;
  worktreesToRemove: string[];
  worktreesToDiscard: string[];
  dirtyFilesCount: number;
  branchesToDelete: string[];
  unpushedBranchesCount: number;
  totalUnpushedCommits: number;
  protectedDefaultBranches: string[];
  reposToSwitchToDefault: string[];
  reposReadyToPull: string[];
  reposSkippedDirtyOrLocked: string[];
}

export interface QuickActionResult {
  preset: QuickActionPreset;
  success: boolean;
  worktreesRemoved: number;
  worktreesDiscarded: number;
  branchesDeleted: number;
  reposFetched: number;
  reposSwitchedToDefault: number;
  reposPulled: number;
  reposUpToDate: number;
  reposSkipped: number;
  warnings: string[];
  errors: string[];
}
```

---

## 2. Behavioral Acceptance Criteria

### Scenario 1: `DiscardUncommitted`
- **Given** one or more worktrees containing modified tracked files and untracked files:
- **When** `execute_action(QuickActionPreset::DiscardUncommitted, scope, watch_folders)` is executed:
- **Then**:
  1. Every modified tracked file is restored to commit `HEAD` (`git reset --hard HEAD`).
  2. Every untracked file is removed using `git clean -ffd`.
  3. Files listed in `.gitignore` (such as `.env`, certificates, `node_modules`, `target/`) MUST NOT be deleted (flag `-x` is strictly prohibited).
  4. Returns `worktrees_discarded` matching the number of processed worktrees.

### Scenario 2: `NukeWorktrees`
- **Given** a repository with one main worktree and one or more linked worktrees (which may have uncommitted changes or untracked files):
- **When** `execute_action(QuickActionPreset::NukeWorktrees, scope, watch_folders)` is executed:
- **Then**:
  1. If any linked worktree is dirty, it is automatically discarded first.
  2. All linked worktrees (`isMain == false`) are removed using `git worktree remove --force --force`.
  3. The main repository worktree (`isMain == true`) MUST NOT be removed and MUST remain intact.
  4. `git worktree prune` is executed on the repository.
  5. Returns `worktrees_removed` reflecting the count of deleted linked worktrees.

### Scenario 3: `CleanBranches`
- **Given** a repository with a default branch (`main` or `master`) and multiple secondary local branches:
- **When** `execute_action(QuickActionPreset::CleanBranches, scope, watch_folders)` is executed:
- **Then**:
  1. The default branch is strictly preserved and NEVER deleted.
  2. Any branch checked out in an active worktree is protected unless worktrees have been nuked.
  3. All other local branches are deleted using `git branch -D`.
  4. Returns `branches_deleted` count.

### Scenario 4: `TotalFreshStart` (Nuclear Wipe)
- **Given** a repository with dirty linked worktrees, unpushed branches, and dirty main worktree:
- **When** `execute_action(QuickActionPreset::TotalFreshStart, scope, watch_folders)` is executed:
- **Then**:
  1. Discards dirty changes across all linked worktrees.
  2. Removes all linked worktrees (`--force --force`) and runs `git worktree prune`.
  3. Switches the main repository worktree to its default branch (`git checkout <default_branch>`).
  4. Discards uncommitted changes on the main repository worktree (`reset --hard` + `clean -ffd`).
  5. Deletes all local branches except the default branch (`git branch -D`).
  6. The repository is left in a pristine state matching the clean default branch.

### Scenario 5: Pre-flight Audit Preview (`compute_preview`)
- **Given** repositories in various states (clean, dirty, linked worktrees, branches with unpushed commits, root worktrees off default branch):
- **When** `compute_preview(preset, scope, watch_folders)` is queried:
- **Then**:
  1. For destructive presets, accurate counts for `repos_count`, `worktrees_to_remove`, `worktrees_to_discard`, `dirty_files_count`, and `branches_to_delete` are returned, along with `unpushed_branches_count` and `total_unpushed_commits`.
  2. For `QuickActionPreset::SyncDefaultBranch`:
     - If a repository's root worktree (`is_main == true`) is dirty OR if its `<default_branch>` is currently checked out in any secondary linked worktree (`is_main == false`), its root path is added to `repos_skipped_dirty_or_locked` (and NOT to `repos_to_switch_to_default` or `repos_ready_to_pull`).
     - Otherwise, its root path is added to `repos_ready_to_pull`; additionally, if the root worktree's current branch differs from `<default_branch>`, its root path is also added to `repos_to_switch_to_default`.
  3. `protected_default_branches` lists the deduplicated names of default branches across targeted repositories.

### Scenario 6: `SyncDefaultBranch` (Constructive Non-Destructive Sync)
- **Given** one or more repositories (with or without remotes, clean or dirty root worktrees, secondary worktrees, or diverged local branches):
- **When** `execute_action(QuickActionPreset::SyncDefaultBranch, scope, watch_folders)` is executed:
- **Then**:
  1. **Zero Data Destruction:** Under NO circumstances may `SyncDefaultBranch` discard modified tracked files, delete untracked files, remove linked worktrees, or delete local branches. Linked worktrees remain untouched.
  2. **Remote Fetch & Prune:** If the repository has at least one configured git remote (`git remote` is non-empty), `git fetch --all --prune` is executed. On success, `repos_fetched` increments by 1. If `fetch` fails (e.g. unreachable remote), an error is recorded in `errors` for that repository and `pull` is not attempted against stale network state.
  3. **Dirty Root Worktree Guard:** If the root/main worktree has uncommitted tracked or untracked changes (`git status --porcelain=v1` is non-empty), `checkout` and `pull` MUST be skipped for this repository, `repos_skipped` increments by 1, and an explanatory message is appended to `warnings` (`errors` remains empty so `success` stays `true`).
  4. **Linked Worktree Branch-Lock Guard:** If `<default_branch>` is currently checked out in a secondary linked worktree (`!is_main`), root `checkout` and `pull` MUST be skipped, `repos_skipped` increments by 1, and an explanatory message naming the secondary worktree is appended to `warnings` (`errors` remains empty).
  5. **Checkout Default Branch:** If the root worktree is clean and not already on `<default_branch>`, `git checkout <default_branch>` is executed and `repos_switched_to_default` increments by 1.
  6. **Fast-Forward Pull (`git pull --ff-only`):**
     - If the repository has no remote or `<default_branch>` has no upstream tracking reference (`<default_branch>@{u}` fails to resolve), `pull` is not required and `repos_up_to_date` increments by 1.
     - Otherwise, `git pull --ff-only` is executed on the root worktree:
       - If `HEAD` commit SHA advances after `pull --ff-only`, `repos_pulled` increments by 1.
       - If `HEAD` commit SHA is unchanged (`Already up to date`), `repos_up_to_date` increments by 1.
       - If `pull --ff-only` fails due to diverging local commits (non-fast-forward), the working tree remains clean and unmutated, `repos_skipped` increments by 1, and a divergence warning is appended to `warnings` (`errors` remains empty).

### Scenario 7: `QuickActionScope::RepoList` (Filtered Repository Subset)
- **Given** multiple repositories on disk (e.g. Repo A, Repo B, Repo C) and `QuickActionScope::RepoList { repo_paths: vec![repo_a, repo_b] }`:
- **When** `compute_preview` or `execute_action` is invoked with that scope:
- **Then**:
  1. Only the deduplicated repository roots resolved from `repo_paths` are previewed or mutated; Repo C is completely untouched.
  2. If `repo_paths` is empty, `resolve_repos` returns `Ok(vec![])` and both `compute_preview` (`repos_count == 0`) and `execute_action` (`success == true`) succeed as clean no-ops.
  3. If any path in `repo_paths` does not exist or is not a valid git repository, `resolve_repos` returns `Err(...)`.

---

## 3. Boundary Values & State Invariants

1. **Main Worktree Immunity:**
   - The root/main worktree (`.git` is a directory or path equals `repo_path`) MUST NEVER be passed to `git worktree remove`.
2. **Default Branch Immunity (ADR 0006):**
   - The default branch (`main`, `master`, `trunk`, or resolved `origin/HEAD`) MUST NEVER be deleted under any circumstances, even with `force: true`.
3. **No-Secrets Destruction Invariant:**
   - Under no circumstances shall `git clean` be invoked with `-x` or `-X`. Only `-ffd` is permitted in destructive presets, and `SyncDefaultBranch` MUST NEVER invoke `git reset --hard` or `git clean`.
4. **Subprocess Invariants (Windows & Terminal):**
   - Every `std::process::Command` MUST have `CREATE_NO_WINDOW = 0x08000000` (on Windows), `stdin(Stdio::null())`, and environment variables `GIT_TERMINAL_PROMPT = "0"`, `GIT_OPTIONAL_LOCKS = "0"` via `GitService::build_command`.
5. **Rayon Inter-Repo Concurrency & Multi-Account Credential Grouping:**
   - Parallel iteration (`par_iter()`) is permitted ONLY across distinct repositories; intra-repo operations MUST be sequential.
   - When `execute_action_with_config` runs `SyncDefaultBranch` with `config.auto_switch_account == true`, repositories associated with distinct GitHub accounts (`WatchFolder.account_username`) are processed by account group sequentially (switching `GhService::switch_account` before each named group and restoring the initial active account afterwards), using `par_iter()` across repositories within each group.
6. **Graceful OS Lock & Safe Skip Handling:**
   - Safe skips in `SyncDefaultBranch` (dirty root, branch locked in linked worktree, or diverged non-FF branch) record entries in `warnings`, NOT `errors`, preserving `success == true`.

---

## 4. Prohibited Details

- Do NOT prescribe internal helper names or private struct layouts inside `quick_actions.rs`.
- Do NOT prescribe CSS animations or internal DOM tree structure of `QuickActionsModal.svelte`.
