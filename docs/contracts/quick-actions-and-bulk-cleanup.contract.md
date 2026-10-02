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
    DiscardUncommitted,
    NukeWorktrees,
    CleanBranches,
    TotalFreshStart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionScope {
    SingleRepo { repo_path: String },
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionResult {
    pub preset: QuickActionPreset,
    pub success: bool,
    pub worktrees_removed: usize,
    pub worktrees_discarded: usize,
    pub branches_deleted: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}
```

#### Service Interface (`QuickActionsService`)
```rust
pub struct QuickActionsService;

impl QuickActionsService {
    /// Computes a dry-run pre-flight audit for the selected preset and scope.
    pub fn compute_preview(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[crate::services::config::WatchFolder],
    ) -> Result<QuickActionPreview, String>;

    /// Executes the atomic cleanup pipeline across repositories.
    pub fn execute_action(
        preset: QuickActionPreset,
        scope: &QuickActionScope,
        watch_folders: &[crate::services::config::WatchFolder],
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
export type QuickActionPreset = 'discardUncommitted' | 'nukeWorktrees' | 'cleanBranches' | 'totalFreshStart';

export type QuickActionScope = 
  | { singleRepo: { repoPath: string } }
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
}

export interface QuickActionResult {
  preset: QuickActionPreset;
  success: boolean;
  worktreesRemoved: number;
  worktreesDiscarded: number;
  branchesDeleted: number;
  warnings: string[];
  errors: string[];
}
```

---

## 2. Behavioral Acceptance Criteria

### Scenario 1: `DiscardUncommitted`
- **Given** one or more worktrees containing modified tracked files and untracked files:
- **When** `execute_action(QuickActionPreset::DiscardUncommitted, scope)` is executed:
- **Then**:
  1. Every modified tracked file is restored to commit `HEAD` (`git reset --hard HEAD`).
  2. Every untracked file is removed using `git clean -ffd`.
  3. Files listed in `.gitignore` (such as `.env`, certificates, `node_modules`, `target/`) MUST NOT be deleted (flag `-x` is strictly prohibited).
  4. Returns `worktrees_discarded` matching the number of processed worktrees.

### Scenario 2: `NukeWorktrees`
- **Given** a repository with one main worktree and one or more linked worktrees (which may have uncommitted changes or untracked files):
- **When** `execute_action(QuickActionPreset::NukeWorktrees, scope)` is executed:
- **Then**:
  1. If any linked worktree is dirty, it is automatically discarded first.
  2. All linked worktrees (`isMain == false`) are removed using `git worktree remove --force --force`.
  3. The main repository worktree (`isMain == true`) MUST NOT be removed and MUST remain intact.
  4. `git worktree prune` is executed on the repository.
  5. Returns `worktrees_removed` reflecting the count of deleted linked worktrees.

### Scenario 3: `CleanBranches`
- **Given** a repository with a default branch (`main` or `master`) and multiple secondary local branches:
- **When** `execute_action(QuickActionPreset::CleanBranches, scope)` is executed:
- **Then**:
  1. The default branch is strictly preserved and NEVER deleted.
  2. Any branch checked out in an active worktree is protected unless worktrees have been nuked.
  3. All other local branches are deleted using `git branch -D`.
  4. Returns `branches_deleted` count.

### Scenario 4: `TotalFreshStart` (Nuclear Wipe)
- **Given** a repository with dirty linked worktrees, unpushed branches, and dirty main worktree:
- **When** `execute_action(QuickActionPreset::TotalFreshStart, scope)` is executed:
- **Then**:
  1. **Phase 1:** Discards dirty changes across all linked worktrees.
  2. **Phase 2:** Removes all linked worktrees (`--force --force`) and runs `git worktree prune`.
  3. **Phase 3:** Switches the main repository worktree to its default branch (`git checkout <default_branch>`).
  4. **Phase 4:** Discards uncommitted changes on the main repository worktree (`reset --hard` + `clean -ffd`).
  5. **Phase 5:** Deletes all local branches except the default branch (`git branch -D`).
  6. The repository is left in a pristine state matching the clean default branch.

### Scenario 5: Pre-flight Audit Preview (`compute_preview`)
- **Given** repositories in various states (clean, dirty, linked worktrees, branches with unpushed commits):
- **When** `compute_preview(preset, scope)` is queried:
- **Then**:
  1. Accurate counts for `repos_count`, `worktrees_to_remove`, `worktrees_to_discard`, `dirty_files_count`, and `branches_to_delete` are returned.
  2. If any branch scheduled for deletion has local commits not pushed to its upstream remote, `unpushed_branches_count` and `total_unpushed_commits` MUST reflect the exact count (calculated via `git rev-list --count @{u}..HEAD` or counting all commits on branches lacking upstream).
  3. `protected_default_branches` lists the names of default branches that will be preserved.

---

## 3. Boundary Values & State Invariants

1. **Main Worktree Immunity:**
   - The root/main worktree (`.git` is a directory or path equals `repo_path`) MUST NEVER be passed to `git worktree remove`.
2. **Default Branch Immunity (ADR 0006):**
   - The default branch (`main`, `master`, `trunk`, or resolved `origin/HEAD`) MUST NEVER be deleted under any circumstances, even with `force: true`.
3. **No-Secrets Destruction Invariant:**
   - Under no circumstances shall `git clean` be invoked with `-x` or `-X`. Only `-ffd` is permitted to protect `.env` and local secrets.
4. **Subprocess Invariants (Windows & Terminal):**
   - Every `std::process::Command` MUST have `CREATE_NO_WINDOW = 0x08000000`, `stdin(Stdio::null())`, and environment variables `GIT_TERMINAL_PROMPT = "0"`, `GIT_OPTIONAL_LOCKS = "0"`.
5. **Rayon Inter-Repo Concurrency:**
   - Parallel iteration (`par_iter()`) is permitted ONLY across distinct repositories. All operations on a single repository MUST be sequential.
6. **Graceful OS Lock Handling:**
   - If Windows locks an untracked file (`Permission denied` or in use by another process), the operation logs a warning in `warnings` rather than panicking or failing the entire batch.

---

## 4. Prohibited Details

- Do NOT prescribe internal helper names or private struct layouts inside `quick_actions.rs`.
- Do NOT prescribe CSS animations or internal DOM tree structure of `QuickActionsModal.svelte` beyond the requirement to have a scope selector, preset cards, impact preview with unpushed commit warning, and armed confirmation triggers.
