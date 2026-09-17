# 🏗️ Technical Architecture & Design

This document details the system architecture, component breakdown, concurrency model, and data contracts for **Worktree & Workspace Companion**.

---

## 1. System Overview

**Worktree & Workspace Companion** is a native Windows system tray utility and Spotlight-style floating dashboard built with **Tauri v2**, **Rust**, **Svelte 5**, and **Tailwind CSS**.

```mermaid
graph TD
    User([User / System Tray]) -->|Click Tray Icon / Hotkey| Window[Floating Spotlight Window]
    Window -->|Svelte 5 UI| Frontend[Frontend Stores & Components]
    Frontend -->|Tauri IPC invoke| Commands[Rust IPC Commands]
    Commands -->|Modular Services| Services[Rust Services Layer]
    Services -->|CREATE_NO_WINDOW| GitCLI[Git CLI Engine]
    Services -->|CREATE_NO_WINDOW| GhCLI[GitHub CLI Engine]
    Services -->|Local Storage| ConfigEngine[Config & Watch Folders Engine]
```

---

## 2. Backend Architecture (`src-tauri/`)

The Rust backend handles low-level OS interactions, system tray lifecycle, and safe subprocess execution.

### Key Subsystems:

1. **System Tray & Window Lifecycle (`tray.rs`)**:
   - Manages the tray icon using `tauri::tray::TrayIconBuilder`.
   - Utilizes `tauri-plugin-positioner` to position the floating dashboard adjacent to the taskbar.
   - Listens to window focus events (`tauri::WindowEvent::Focused(false)`) to automatically hide the window when unfocused (*Spotlight auto-hide*).

2. **Git Engine & Process Service (`services/git.rs`)**:
   - Parses machine-readable output from `git worktree list --porcelain` and `git status --porcelain`.
   - Detects active worktrees, detached HEAD states, locked worktrees, and dirty working trees.
   - Executes commands with Windows creation flag `CREATE_NO_WINDOW = 0x08000000` and detached standard I/O (`Stdio::null()`) to eliminate terminal popups.
   - Provides decoupled, zero-console launchers:
     - `open_in_editor`: Resolves native Win32 GUI executables directly via `resolve_gui_binary` (inspecting User `%LOCALAPPDATA%`, System `%ProgramFiles%`, Insiders, and dynamic PATH parent un-nesting) to launch VS Code (`Code.exe`), Antigravity IDE (`Antigravity.exe`), Cursor (`Cursor.exe`), or Windsurf (`Windsurf.exe`) instantly with 0 console windows.
     - `open_in_terminal`: Dispatches to terminal environments (Windows Terminal `wt`, PowerShell, CMD, Git Bash, AGY CLI) with fallback cascade.
   - `CreateWorktreeResult`/`CheckoutBranchResult` carry a fully-enriched `worktree_info` for the one worktree they affected, letting the frontend patch its list in place instead of re-scanning (see ADR 0005).
   - Every command handler that shells out to `git` runs its blocking work inside `tauri::async_runtime::spawn_blocking`, so a slow subprocess never stalls a Tokio worker thread handling other IPC calls.

3. **GitHub CLI Service (`services/gh.rs`)**:
   - Queries authenticated GitHub accounts via `gh auth status`.
   - Switches active identities seamlessly via `gh auth switch --hostname github.com -u <user>`.
   - Invoked concurrently with, and never blocking, editor/terminal launches and worktree creation — `gh`'s auth state is a config file read fresh at call time, not something the launch action needs to wait on.

4. **Orphaned Worktree Cleaner (`services/worktree_cleaner.rs`)**:
   - Compares local worktree branches against remote upstream tracking branches.
   - Identifies branches deleted or merged on the remote.
   - Enforces pre-flight dirty checks (`git status --porcelain`) to guarantee no uncommitted work is deleted.
   - Computes the repo-wide orphan context (default branch, `branch -vv`, `branch --merged`) once per repository (`build_repo_orphan_context`) and reuses it for every worktree in that repo, instead of recomputing it per worktree.
   - Per-worktree enrichment (`enrich_worktree_info`: dirty check, last-commit info, orphan check) runs in parallel across a repository's worktrees via `rayon::par_iter_mut`, and is also reachable standalone through `build_single_worktree_info` for a single worktree path.
   - Leverages `rayon` for parallel repository discovery across configured watch folders and parallel per-repo scanning.

5. **Branch Cleaner (`services/branch_cleaner.rs`)**:
   - Lists every local branch in a repository — not only ones with a worktree — via
     `GitService::list_local_branches_with_status`, flagging each as current, default, merged,
     remote-gone, and/or checked out (in the main worktree or a linked one).
   - `remove_branches_batch` re-derives that status per repository immediately before deleting,
     rather than trusting the caller's targets, so the default branch and any checked-out branch are
     never deletable — not even with `force` — regardless of what the frontend last scanned. See
     ADR 0006.
   - Deletion itself uses `git branch -d` (git's own refusal for unmerged branches) unless `force`
     requests `-D`; the merged/remote-gone flags are informational for the UI, not a second gate.
   - Shares `RepoOrphanContext` and the merged/gone classification logic with the orphaned worktree
     cleaner via `GitService::branch_status_flags`, so the two features can't disagree about what
     counts as merged or gone.

6. **Configuration Service (`services/config.rs`)**:
   - Manages user preferences, watched root folders, default editor, default terminal, and terminal button visibility.
   - Employs atomic write commits (`.tmp` swap) and `#[serde(default)]` annotations for backward compatibility.

---

## 3. Frontend Architecture (`src/`)

The frontend is built with **Svelte 5** leveraging modern reactive stores and clean component modularity.

### Directory Structure:
- `src/lib/components/`:
  - `Header.svelte`: Top bar with GitHub active account badge, search bar, and action triggers.
  - `AccountFilterBar.svelte`: Filter worktrees by GitHub profile / organization.
  - `WorktreeList.svelte`, `WorktreeCard.svelte` & `WorktreeItemRow.svelte`: Worktree list components displaying branch details, status badges, lock indicators, and 1-click IDE/Terminal launchers. Both views share `DirtyDiffPopover.svelte` for the hover-triggered dirty-file diff, so neither view can drift out of parity with the other.
  - `DirtyDiffPopover.svelte`: Lazy, debounced fetch-and-display of a worktree's uncommitted diff summary, reused by both the Compact and Detailed views.
  - `NewWorktreeModal.svelte`: Modal to create a new worktree from existing or new branches.
  - `BranchSwitcherModal.svelte`: Quick switcher to checkout branches, with arrow-key + Enter keyboard navigation over the filtered branch list.
  - `RemoveWorktreeModal.svelte`: Single-worktree removal dialog with pre-flight safety summaries.
  - `WatchFoldersModal.svelte`: Configuration dialog for repository root scan paths, default IDE, and default terminal.
  - `GhAccountModal.svelte`: Account switcher modal.
  - `BranchList.svelte`, `BranchItemRow.svelte` & `BranchCard.svelte`: Branch Cleaner list grouped by repository, in the same Compact/Detailed densities as the worktree list. Neither branch view carries a per-branch delete action — branch deletion is batch-only by design, so the reviewed batch flow stays the single destructive path.
  - `BranchStatusBadges.svelte`: The Default / In-worktree / Merged / Remote-gone badge set, shared by the row and the card so the two densities can't describe the same branch differently.
  - `BranchFilterBar.svelte`: Status filter chips (All/Merged/Remote gone/Protected) for the Branch Cleaner, same single-pass tally pattern as `AccountFilterBar.svelte`, plus the cross-repository "select all filtered" control.
  - `BranchBatchDeleteModal.svelte` & `BranchActionBar.svelte`: Branch Cleaner's review/confirm modal and floating selection dock, mirroring `BatchDeleteModal.svelte`/`BatchActionBar.svelte` with "unmerged" in place of "dirty".
- `src/lib/actions/`:
  - `closeOnEscape.ts`: Shared Svelte action wiring `Escape` to a modal's close handler (`{ enabled, onClose }`), used by every modal so Escape-to-close can't silently go missing from a new one.
- `src/lib/utils/`:
  - `protectionReason.ts`: Single source for the wording of *why* a branch or worktree is protected or destructive to delete, so a row's tooltip and a dialog's warning can't drift apart. It describes the server-side guards (ADR 0003, ADR 0006); it never enforces them.
- `src/lib/stores/`:
  - `worktrees.ts`: Discovered worktrees, scan state, and `filteredRepos` — the filtered set both `WorktreeList` and the "select all filtered" control read, so neither can compute a different "all".
  - `ghAuth.ts`: Active GitHub account and switcher logic.
  - `appConfig.ts`: Application preferences, default editor, default terminal, and watch paths.
  - `editors.ts`: Installed editor (`installedEditors`) and terminal (`installedTerminals`) detection.
  - `branchCleaner.ts`: Scanned branch list, scan state, the selected branch status filter, and the matching `filteredBranches` derived set.
  - `branchSelection.ts`: Batch-selection map for the Branch Cleaner, keyed by `repoPath::branchName` since branch names — unlike worktree paths — aren't globally unique across repos.
  - `forceDeleteIntent.ts`: Whether the user has opted into forcing, kept separately for worktrees and branches, so closing and reopening a dialog over the same selection doesn't discard the choice.

### State Update Strategy
`App.svelte` patches the `scannedRepos` store in place for single-worktree mutations (create, delete, branch switch) using the fresh `worktreeInfo` the backend returns for create/checkout, or the known path for delete — mirroring the pattern the batch-delete flow already used. A full rescan (`scan_worktrees`) is reserved for the manual Refresh action and as a defensive fallback if a mutation response doesn't carry `worktreeInfo`. See ADR 0005 for the reasoning.

---

## 4. Security & Safety Principles

1. **Subprocess Isolation**: Background subprocesses are spawned without shell wrapper invocation (`CREATE_NO_WINDOW`) to protect against injection and prevent visual terminal glitches.
2. **Destructive Guardrails**: Any action executing `git worktree remove` or branch deletion MUST execute a pre-flight dirty check first.
3. **Configuration Resilience**: Backward-compatible schema deserialization prevents configuration loss during software updates.
4. **Path Privacy**: Local paths are kept internal and never exposed to unauthenticated endpoints or persistent tracking.
