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
2. **Four Supported Presets**:
   - `DiscardUncommitted`: `git reset --hard HEAD` and `git clean -ffd`. **Strictly forbids `-x`**, preserving `.env` and files in `.gitignore` intact.
   - `NukeWorktrees`: Discards dirty changes in linked worktrees, executes `git worktree remove --force --force`, and runs `git worktree prune`. The main repository worktree is strictly protected.
   - `CleanBranches`: Deletes all secondary local branches (`git branch -D`). The default branch is strictly immune (ADR 0006). Pre-flight audit inspects and warns about unpushed commits.
   - `TotalFreshStart`: A sequential 5-phase pipeline:
     1. Discard dirty changes in linked worktrees.
     2. Remove all linked worktrees and prune.
     3. Switch main repository worktree to default branch (`git checkout <default>`).
     4. Discard uncommitted changes on main worktree.
     5. Delete all secondary local branches.
3. **Pre-flight Live Impact Audit (`compute_preview`)**:
   - Before executing, the UI queries `get_quick_action_preview`, displaying affected counts of repositories, worktrees, dirty files, and branches. If any branch has unpushed commits, an amber/red warning is explicitly presented.
4. **Armed Confirmation UI**:
   - Presets 1-3 use a two-step armed confirmation button with a 3-second countdown.
   - Preset 4 (`TotalFreshStart`) requires typing `"RESET"` to unlock execution.

## Consequences
- **Positive**:
  - Developers can reset and clean up cluttered worktrees with zero manual terminal juggling.
  - Guaranteed protection of root repositories, default branches, and `.gitignore`d secrets.
  - Complete elimination of Windows console flashing via `CREATE_NO_WINDOW = 0x08000000` and `Stdio::null()`.
- **Negative**:
  - Pre-flight preview computation adds a minor round-trip to compute dirty status and unpushed commits before showing the modal's armed trigger.
