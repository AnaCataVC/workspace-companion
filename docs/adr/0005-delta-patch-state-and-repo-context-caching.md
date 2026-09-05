# ADR 0005: Delta-Patch State Updates and Repo-Wide Context Caching Over Full Rescans

## Status
Accepted

## Context
The initial implementation treated every worktree mutation (create, delete, branch checkout) as a trigger for a full rescan: `scan_worktrees` re-walked every configured watch folder and re-shelled out to `git` for every worktree in every repository, and the frontend cleared its entire `scannedRepos` store before repopulating it from the resulting `repo-scanned` events.

This had two compounding costs:

1. **Backend.** Within a single repository, orphan detection (`check_orphan_status`) recomputed the same repo-wide facts — the default branch, `git branch -vv`, `git branch --merged` — once per worktree, even though that result is identical for every worktree in the repo. A repository with N worktrees fired roughly 5N git subprocesses instead of ~5. Several command handlers (`remove_worktree`, `list_branches`, `checkout_worktree_branch`, `suggest_worktree_path`, `create_worktree`, `prune_worktrees`, `get_worktree_diff_summary`) also ran these blocking subprocess calls directly inside their `async fn`, unlike `scan_worktrees`, risking a stalled Tokio worker thread under concurrent IPC.
2. **Frontend.** Any single mutation — deleting one worktree, switching one branch, creating one worktree — cleared the visible list and re-triggered the full backend scan pipeline above, both flashing the UI empty and paying the full N+1 subprocess cost for a change that touched exactly one worktree. Additionally, `handleOpenEditor`/`handleOpenTerminal`/`handleConfirmCreateWorktree` awaited the GitHub CLI "smart account switch" (`gh auth switch`, itself a subprocess) *before* launching the editor/terminal or creating the worktree, serializing an unrelated CLI call onto the app's core "1-click, sub-500ms" action path.

## Decision
1. **Repo-wide context computed once per repo.** `GitService::build_repo_orphan_context` runs `symbolic-ref`/`branch --list`/`branch -vv`/`branch --merged` exactly once per repository scan, producing a `RepoOrphanContext` that every worktree in that repo reuses. `check_orphan_status` takes this context as a parameter instead of recomputing it.
2. **Single-worktree enrichment as a first-class capability.** The per-worktree enrichment logic (dirty check, last-commit info, orphan check, main-worktree marking) used by the scan loop lives in shared helpers (`enrich_worktree_info`, `mark_main_worktree`), reusable from a new `build_single_worktree_info(repo_path, worktree_path)` entry point. `create_worktree` and `checkout_worktree_branch` now return the freshly enriched `WorktreeInfo` for the one worktree they affected (`worktreeInfo` on `CreateWorktreeResult`/`CheckoutBranchResult`).
3. **Frontend patches instead of rescanning.** `App.svelte` uses the `worktreeInfo` above (and, for deletion, the known worktree path) to patch `scannedRepos` in place for create/delete/branch-switch, mirroring the delta-update pattern the batch-delete flow already used. Full rescans (`refreshWorktrees()`) are reserved for the manual Refresh action and as a defensive fallback if a mutation response doesn't include `worktreeInfo`.
4. **Account switching runs concurrently, never blocking.** `ensureMatchingAccountForPath` is invoked without `await` ahead of editor/terminal launch or worktree creation, since `gh`'s auth state is a global config file read dynamically at call time, not a value the launch action needs to wait on.
5. **Consistent `spawn_blocking` usage.** Every command handler that shells out to `git` wraps that work in `tauri::async_runtime::spawn_blocking`, matching the pattern `scan_worktrees` already used.
6. **Parallel enrichment and discovery.** Per-worktree enrichment within a repository (`worktrees.par_iter_mut()`) and watch-folder discovery (`enabled_watches.par_iter()`) run in parallel via `rayon`, since decision (1) removed the repo-wide git calls that made the loop body sequential-by-necessity.

## Consequences
- **Positive**:
  - A repository with N worktrees now fires ~5 git subprocesses for orphan detection instead of ~5N.
  - Deleting, creating, or switching a branch on one worktree no longer flashes the list empty or re-scans every watched repository.
  - Launching an editor/terminal is no longer serialized behind a `gh auth switch` call.
  - IPC command handlers no longer risk stalling a Tokio worker thread on blocking subprocess I/O.
- **Negative**:
  - Two enrichment entry points (`scan_repository`'s loop and `build_single_worktree_info`) must be kept behaviorally identical by construction. They already share `enrich_worktree_info`/`mark_main_worktree`, but a future change to worktree enrichment must remember both call sites exist.
  - The frontend's delta-patch path silently falls back to a full rescan if `worktreeInfo` is ever missing from a mutation response (e.g. an older or mocked backend), which reads as ordinary "why did the whole list refresh" behavior rather than a hard error — worth checking first if that fallback ever fires in practice.
