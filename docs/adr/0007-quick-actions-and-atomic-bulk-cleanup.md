# ADR 0007: Quick Actions and Atomic Bulk Cleanup Architecture

## Status
Accepted

## Context
Developers working with Git worktrees and multiple repositories accumulate residual clutter over time:
1. Uncommitted file edits and untracked artifacts across multiple worktrees.
2. Temporary linked worktrees that should be completely removed once tasks are finished.
3. Abandoned or merged local branches that pile up behind the default branch.

Prior to this feature, cleaning up required individual discards, batch worktree deletes, and branch prunes across disjoint modal windows. Chaining these destructive actions from the frontend UI presented severe risks:
- **Lock Contention**: Concurrently modifying `.git/worktrees` and `.git/index` causes fatal Git lock collisions.
- **De-synchronization**: Git forbids deleting a branch that is checked out in any worktree (`error: Cannot delete branch '<branch>' checked out at '<path>'`).
- **Windows File Locks**: Active language servers, IDEs, or terminal handles holding files open can cause `Permission denied` (`EPERM`) errors.
- **Unintended Data Loss**: Deleting branches with unpushed commits or purging gitignored credentials (`.env`).

## Decision
1. **Unified Atomic Service in Rust (`QuickActionsService`)**:
   - Rather than chaining IPC calls from the frontend, all bulk cleanup pipelines are executed server-side in Rust.
   - Concurrency is strictly bounded: **Rayon parallel iteration occurs only across distinct repositories (`par_iter`)**, while operations within a single repository run strictly sequentially.
2. **Five Supported Presets (2-Zone Constructive vs. Destructive Partition)**:
   - **Constructive Daily Sync (`SyncDefaultBranch`)**: Executes `git fetch --all --prune`, safely skips `checkout`/`pull` on dirty root worktrees or when `<default_branch>` is checked out in a linked secondary worktree (reporting safe skip warnings), switches clean root worktrees to `<default_branch>`, and advances via `git pull --ff-only`. Never discards files, deletes branches, or creates unexpected merge commits. When `auto_switch_account` is enabled, repositories are grouped by `associated_account` so `GhService::switch_account` runs sequentially per account group while Rayon parallelizes within each group, restoring the initial active account afterwards.
   - **Destructive Cleanups**:
     - `DiscardUncommitted`: `git reset --hard HEAD` and `git clean -ffd`. **Strictly forbids `-x`**, preserving `.env` and files in `.gitignore` intact.
     - `NukeWorktrees`: Discards dirty changes in linked worktrees, executes `git worktree remove --force --force`, and runs `git worktree prune`. The main repository worktree is strictly protected.
     - `CleanBranches`: Deletes secondary local branches (`git branch -D`). The default branch and checked-out branches are strictly immune (ADR 0006). Pre-flight audit inspects and warns about unpushed commits.
     - `TotalFreshStart`: Sequential 5-phase wipe (discard linked worktrees, remove linked worktrees & prune, switch root worktree to default branch, discard uncommitted changes on root worktree, delete secondary local branches).
3. **Three Target Scopes (`QuickActionScope`)**:
   - `AllWatchedRepos`, `RepoList { repo_paths }` (automatic subset targeting active `@account`, status, or search filters), and `SingleRepo { repo_path }`.
4. **Pre-flight Live Impact Audit (`compute_preview`) & Trigger Discipline**:
   - Before executing, the UI queries `get_quick_action_preview`. Constructive `SyncDefaultBranch` executes in 1 click (both from the top toolbar `Sync main` button and the modal's top zone). Destructive presets disable execution when impact is zero (`Nothing to clean`), use a two-step armed button with a 3-second countdown for presets 1–3, and require typing `"RESET"` for `TotalFreshStart`.

## Consequences
- **Positive**:
  - Developers can synchronize all or filtered repositories to `main`/`master` in 1 click without risking uncommitted work or cross-account `gh` credential collisions, and clean up cluttered worktrees without manual terminal juggling.
  - Guaranteed protection of root repositories, default branches, dirty worktrees during sync, and `.gitignore`d secrets.
  - Complete elimination of Windows console flashing via `CREATE_NO_WINDOW = 0x08000000` and `Stdio::null()`.
- **Negative**:
  - Pre-flight preview computation adds a minor round-trip to compute dirty status, default-branch state, and unpushed commits before rendering modal metrics.
