> **Created:** 2026-04-10
> **Last Updated:** 2026-11-27

# Quick Actions & Bulk Cleanup Architecture

## Executive Summary
Developers working with multiple repositories and Git worktrees frequently encounter accumulated debris:
1. **Uncommitted dirty state / untracked files**: stray build artifacts, test runs, or WIP experiments across multiple worktrees.
2. **Accumulated linked worktrees**: temporary feature branches or cleanroom agent workspaces that need to be completely purged, including dirty files.
3. **Stale local branches**: branches left behind after merging PRs or abandoning work, where only `main` (or the default branch) should survive.

This document establishes the architecture for **Quick Actions** (Acciones Rápidas) in Workspace Companion, balancing one-click developer velocity with strict guardrails against catastrophic data loss.

---

## 1. Safety & Ergonomic Patterns from CLI & Desktop Tools

### A. LazyGit & Git CLI Reference
- **Nuke working directory**: In Git CLI and LazyGit, wiping uncommitted changes requires both `git reset --hard HEAD` and `git clean -ffd` (recursive force clean ignoring directories).
- **Worktree Force Removal**: `git worktree remove --force --force <path>` allows removing worktrees with uncommitted files or locked states, but lingering untracked ignore files or permission-locked OS handles can cause removal failures if not unlinked or cleaned first.
- **Branch Bulk Prune**: Git protects against deleting unmerged branches via `-d`, requiring `-D` (force). Crucially, Git refuses to delete a branch that is currently checked out in any linked worktree (`error: Cannot delete branch 'foo' checked out at '...'`). Therefore, a comprehensive "nuke all worktrees then prune all non-main branches" pipeline must execute in strict sequence:
  1. If cleaning worktrees: Discard/unmount worktrees -> `git worktree remove --force --force` -> `git worktree prune`.
  2. If pruning branches: Once worktrees are removed, those branches become un-checked-out and can safely be deleted with `git branch -D`.
  3. Always preserve the default/main branch (e.g. `main`, `master`, `trunk`, or remote default).

### B. Safety Guardrails & User Friction
Because bulk cleanup actions are destructive and cannot be undone via standard Git undo (`git reflog` preserves commit references but **uncommitted files wiped by `git clean` are lost forever**):
1. **Explicit Pre-flight Preview**: Before executing any bulk quick action, the user must be shown a summary of what will be affected:
   - Number of repositories affected.
   - Number of worktrees to be removed.
   - Number of uncommitted files that will be wiped.
   - Number of branches to be deleted.
2. **Two-Step / Armed Confirmation**: Similar to `DiscardChangesPanel.svelte`, confirm buttons for bulk destruction should require explicit confirmation (or typing a confirmation or an arming delay) to prevent accidental double-clicks.
3. **Repository Scope Selector**: The user must be able to choose whether the quick action applies:
   - **Globally** (across all watched repositories).
   - **Per Repository** (scoped to the currently selected or active repository).

---

## 2. Technical Capabilities in Workspace Companion Backend

| Action Preset | Underlying Rust Operations | Safety Check |
| :--- | :--- | :--- |
| **Discard All Uncommitted Changes** | For target worktrees: `GitService::discard_worktree_changes` (`git reset --hard HEAD` + `git clean -ffd`) | Confirm total count of dirty files across target worktrees. |
| **Nuke All Linked Worktrees** | For all non-main worktrees: Discard dirty changes if needed, execute `WorktreeCleanerService::remove_worktrees_batch` with `force: true`, followed by `git worktree prune` | Main repository worktree is strictly protected (`isMain == true` cannot be deleted). |
| **Prune All Branches Except Main** | Collect non-default local branches; check if checked out; if checked out in active worktrees, notify or require worktree release first; execute `BranchCleanerService::remove_branches_batch` with `force: true` | Default branch (`isDefault == true`) is permanently protected server-side (ADR 0006). |
| **Total Workspace Reset (Nuclear)** | 1. Discard all dirty files in worktrees.<br>2. Remove all linked worktrees.<br>3. Discard any dirty files in main repo.<br>4. Delete all local branches except main. | Highest danger level; requires two-step confirmation dialog with full audit count. |

---

## 3. UI/UX Placement Options

1. **Header "Quick Actions" Menu / Dropdown**:
   - A dedicated lightning bolt (`Zap`) icon or "Quick Actions" button in `Header.svelte`.
   - Opens a modal or dropdown with available presets, with badge indicators showing counts (e.g. "3 dirty worktrees", "5 linked worktrees", "12 stale branches").
2. **Contextual Action in Repository Card**:
   - Within each repository group header in `WorktreeList`, a small quick action menu allows nuking worktrees or discarding changes specifically for that repo.
3. **Global Command Palette / Modal**:
   - A comprehensive "Quick Actions" modal with tabbed or card-based presets, showing exact previews of what will be deleted before the user clicks "Execute".


---

## 4. Non-Destructive Multi-Repo Default-Branch Sync (`SyncDefaultBranch`)

### A. Constructive vs. Destructive Action Partition
Unlike cleanup presets (`DiscardUncommitted`, `NukeWorktrees`, `CleanBranches`, `TotalFreshStart`), synchronizing managed repositories to their default branch (`main` / `master`) and pulling upstream commits is a **daily constructive workflow**. Combining constructive sync with destructive wipes under a single red armed trigger violates usability heuristics (error prevention and mental-model alignment). Separating Quick Actions into **Daily Sync & Maintenance** (1-click execution, non-destructive) and **Destructive Bulk Cleanup** (armed 3s countdown / `RESET` keyword) prevents accidental data loss while keeping daily sync frictionless.

### B. Git Worktree & Fast-Forward Safety Invariants
1. **Always-Safe Remote Fetch (`git fetch --all --prune`)**:
   - Fetching remote refs and pruning deleted remote-tracking branches (`refs/remotes/origin/*`) never mutates the working tree or local branches, so it can safely run even on repositories whose root worktree is dirty.
2. **Pre-Flight Root Worktree Dirty Guard (`git status --porcelain=v1`)**:
   - If the root/main worktree (`is_main == true`) has uncommitted tracked or untracked changes, `git checkout <default_branch>` or `git pull` could either fail mid-operation or carry dirty modifications across branches.
   - Safe non-destructive contract: perform `git fetch --all --prune`, skip `checkout` and `pull` for that repository, and record a structured warning (`Skipped checkout/pull in <repo>: uncommitted changes in main worktree`).
3. **Worktree Branch-Lock Contention (`fatal: '<branch>' is already used by worktree at ...`)**:
   - Git enforces that a local branch cannot be checked out in two worktrees simultaneously ([git-worktree documentation](https://git-scm.com/docs/git-worktree)).
   - Before invoking `git checkout <default_branch>` on the root worktree, inspect `git worktree list --porcelain`. If `<default_branch>` is currently checked out in a linked secondary worktree, skip root `checkout`/`pull` and emit a clear warning naming the linked worktree holding `<default_branch>`.
4. **Deterministic Fast-Forward Only Pull (`git pull --ff-only`)**:
   - Headless background tasks (`GIT_TERMINAL_PROMPT=0`) must never create unexpected merge commits or leave a repository stranded mid-rebase conflict.
   - Using `git pull --ff-only` ([git-pull documentation](https://git-scm.com/docs/git-pull)) advances `<default_branch>` cleanly when local history is a strict ancestor of `@{u}`. If local `<default_branch>` has diverged (unpushed local commits conflicting with upstream), Git exits cleanly without mutating the working tree and the service reports a warning (`Diverged from upstream; fast-forward not possible`).
   - If `<default_branch>` has no configured upstream tracking ref, skip `pull --ff-only` gracefully after `fetch --all --prune` (local-only repo or unpushed default branch).

### C. Multi-Account GitHub CLI (`gh auth switch`) Credential Grouping
- When HTTPS remotes rely on GitHub CLI as a Git credential helper (`gh auth git-credential`), `gh` stores **one active account per host (`github.com`)** in `hosts.yml` ([GitHub CLI Issue #8678](https://github.com/cli/cli/issues/8678), [Why `git pull` Says "Repository Not Found" — Multi-Account Keychain Analysis](https://dev.to/trivedivatsal/why-git-pull-says-repository-not-found-when-the-repo-exists-4f51), [Running Multiple GitHub Accounts with `gh`](https://accuweaver.com/2026/07/02/how-to-painlessly-run-multiple-github-accounts-on-one-machine/)).
- Spawning `par_iter()` across repositories belonging to different `@account` profiles while invoking `gh auth switch` concurrently causes a global credential race condition.
- **Bounded Account-Group Concurrency**:
  1. Partition target repositories by `associated_account` (`Option<String>`).
  2. Iterate account groups **sequentially**: when `auto_switch_account` is enabled and `Some(username)` is present, invoke `GhService::switch_account(username)` once before processing that group (logging a non-fatal warning if `gh` fails so SSH/GCM setups continue uninterrupted).
  3. Within each account group, execute `fetch`/`checkout`/`pull` across distinct repositories in parallel via Rayon (`par_iter()`), preserving intra-repo sequential execution.
  4. Restore the originally active `gh` account after multi-group sync completes so the user's global CLI session is not unexpectedly mutated.
