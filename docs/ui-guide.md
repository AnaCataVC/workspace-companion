# 🧭 User Interface & Navigation Guide

This document provides a comprehensive visual reference and operational guide for the **Workspace Companion** desktop interface.

---

## 1. Floating Dashboard Overview

**Workspace Companion** operates as a Spotlight-style floating window that slides out from the Windows System Tray upon clicking the tray icon or pressing the global shortcut (`Alt+Space`).

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ [Icon] WORKSPACE COMPANION             [▦/▤ View] [+ New] [@gh-user] [⚙] [⚡] [📌] [↻]│ Top Toolbar
├──────────────────────────────────────────────────────────────────────────┤
│ 🔍 Filter worktree, branch or path...                                    │ Quick Search
├──────────────────────────────────────────────────────────────────────────┤
│ [📁 cud-system] [5 worktrees] [@CataVillalobosC]          [Select All] [🧹 Clean]│ Repo Section Header
│                                                                          │
│  ⚓ [ ⎇ main ▼ ]             cud-system          [● dirty (1)]  [🚀] [>_] [📁]    │ Root / Main Worktree
│  ├─ [ ⎇ cld/rekey-edited..▼] epic-sutherland-..  [▲ Orphan]     [🚀] [>_] [📁] [🗑]│ Linked Worktree 1
│  ├─ [ ⎇ cld/reacunado-r.. ▼] jose-observations.. [● dirty (3)]  [🚀] [>_] [📁] [🗑]│ Linked Worktree 2
│  └─ [ ⎇ FIX-Suggest_reas..▼] suggest-reason-..   [● dirty (2)]  [🚀] [>_] [📁] [🗑]│ Linked Worktree 3
│                                                                          │
│  [+ Fast Worktree: branch-name                       ] [⚡ Create & Launch]│ Inline Quick Creator
├──────────────────────────────────────────────────────────────────────────┤
│ [ 4 worktrees selected ]                          [✕ Cancel] [🗑 Delete Selected]│ Floating Batch Bar
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Worktree Item Anatomy: Git Branch vs. Directory Path

A common point of confusion arises when working with Git CLI, AI agents (Claude Code, Cursor, Windsurf, Copilot), or external terminals that refer to worktrees by their **filesystem path** (e.g., `.claude/worktrees/epic-sutherland-e8bdda`).

In **Workspace Companion**, every worktree entry presents both the **Git Branch** and the **Physical Directory Name** side-by-side:

```text
    ┌─── [1] Branch Selector Badge           ┌─── [2] Directory Name
    │                                        │
┌───┴────────────────────────┐       ┌───────┴────────┐
│ [⎇] cld/rekey-addresses [▼]│       │ epic-sutherland│   [● 3 dirty]   [🚀 IDE] [>_] [📁] [🗑]
└────────────────────────────┘       └────────────────┘   └────┬────┘   └───────────┬───────────┘
                                                               │                    │
                                          [3] Status Badges ───┘                    └── [4] 1-Click Action Dock
```

### Component Breakdown

| Callout | UI Element | Visual Icon / Style | Description & User Interaction |
| :--- | :--- | :---: | :--- |
| **[1]** | **Git Branch Badge** | `<GitBranch />` + `<ChevronDown />` (Green / Dark Pill) | **Active Git Branch:** Displays the branch currently checked out in this worktree. **Clicking this badge** opens the in-place **Branch Switcher Modal** to switch branches or checkout a new branch without touching the terminal. |
| **[2]** | **Directory Name** | Muted Monospace Text / `<Folder />` | **Physical Folder Name:** Displays the local directory basename where this worktree lives on disk. Hovering displays the full absolute path in a tooltip. When AI coding agents (Claude, Copilot) say *"remove `.claude/worktrees/epic-sutherland-e8bdda`"*, this is the corresponding entry. |
| **[3]** | **Status Indicators** | Badges (`dirty`, `orphan`, `locked`, `bare`) | **Worktree Health:**<br>• **`● dirty (N)`**: Worktree has `N` uncommitted files. Hovering reveals the modified file list.<br>• **`▲ Orphan`**: Remote branch was merged or deleted on upstream.<br>• **`🔒 Locked`**: Worktree is locked against automatic pruning.<br>• **`📦 Bare`**: Bare repository root. |
| **[4]** | **1-Click Action Dock** | Action Buttons | **Instant Tools:**<br>• **Editor Launcher (Split Button):** Opens the worktree directly in your preferred IDE (Cursor, VS Code, Windsurf, Antigravity).<br>• **Windows Terminal (`>_`):** Spawns Windows Terminal (`wt.exe`) directly in this worktree folder.<br>• **File Explorer (`📁`):** Opens Windows File Explorer at this folder.<br>• **Safe Delete (`🗑`):** Removes the worktree safely with pre-flight uncommitted work verification (disabled for the root main worktree). |

---

## 3. View Density Modes

You can switch between view layouts using the **View Mode** toggle button (`▤ / ▦`) in the top-right
toolbar. The setting is global: it applies to the worktree list and to the Branch Cleaner alike.

### A. Compact Tree View (`▤`)
- Displays repositories as a hierarchical tree:
  - **`⚓ main`**: The primary root worktree anchor.
  - **`├─` / `└─`**: Connected linked worktrees.
- Optimized for scanning dozens of worktrees across multiple repositories simultaneously.
- Offers instant inline hover popovers for dirty Git file diffs without opening modal dialogs.

### B. Detailed Cards View (`▦`)
- Card-based layout with two distinct rows per worktree.
- Displays multi-select checkboxes, commit hash (`HEAD`), last commit message, and full branch name badges.
- Features an expanded IDE selector dropdown menu for launching non-default code editors on demand.
- Same 1-Click Action Dock as the Compact view: Windows Terminal, File Explorer, and a dirty-file diff popover on hover — no action is exclusive to either view.

---

## 4. Multi-Select & Batch Cleaning Workflow

When cleaning up multiple feature worktrees or stale AI agent worktrees:

1. **Select Worktrees**: Click the checkbox on any secondary worktree row, click **`Select All`** on the repository header, or click **`Select all (N)`** in the filter bar to take every worktree matching the active filters across all repositories at once.
2. **Review Pending Count**: The bottom floating bar indicates `N worktrees selected`.
3. **Trigger Deletion**: Click **`Delete Selected`**.
4. **Safety Verification**: Workspace Companion executes pre-flight dirty checks (`git status --porcelain`) on each selected worktree. Worktrees with uncommitted modifications are skipped unless you tick **"Force delete dirty worktrees permanently"** and type `FORCE` into the confirmation field that appears.
5. **Partial results**: A batch where the only refusals were protections (a dirty worktree, the root worktree) closes on its own and reports how many were skipped. The dialog stays open only when something genuinely failed.

---

## 5. Branch Cleaner Workflow

Switch to **Branches** with the segmented `Worktrees | Branches` control in the top toolbar for cross-repository bulk cleanup, or expand the inline **`N local branches without worktree`** drawer inside any repository card in the **Worktrees** view to mount (`+ Worktree`) or delete individual unmounted branches in place.

1. **Filter**: Both **Worktrees** and **Branches** share the top `@account` filter pills (`All Accounts`, `@username`, `Unassigned`). In **Worktrees**, status chips include `All Worktrees`, `Off main` (repositories whose root worktree is not on `main`/`master`), `Dirty`, `Multi-WT`, `Orphans`, and `Clean`, plus `Reset` and `Collapse all / Expand all`. In **Branches**, status chips include `All Branches`, `Merged`, `Remote gone`, and `Protected`, dynamically reflecting active account and search filters.
2. **Select**: Click a branch's checkbox, click **`Select All`** on a repository's header, or click **`Select all (N)`** in the filter bar to take every deletable branch matching the active filter across all repositories at once.
3. **Review & Confirm**: The floating bar shows the selected count. Clicking **`Delete Selected`** opens a review dialog listing each branch as `merged` or `unmerged`.
4. **Force unmerged branches (optional)**: Unmerged branches are skipped by default — check **"Force delete unmerged branches permanently"** and type `FORCE` to delete them anyway (`git branch -D`).
5. **Safety guarantee**: The default branch and any branch checked out in a worktree are never deletable, even with force enabled — enforced server-side.
6. **Checkout**: Every branch row offers an action on the right to jump to an existing worktree or preload the New Worktree dialog.

---

## 6. Quick Actions, Sync to Main & Bulk Cleanup (`Sync main` / `⚡`)

Workspace Companion separates **constructive daily sync** from **destructive bulk cleanup**:

- **1-Click Toolbar Sync (`Sync main` / `Sync filtered`)**: Clicking the emerald sync button in the top toolbar immediately runs `SyncDefaultBranch` across active filtered repositories (when any filter is active) or all watched repositories. Clean completions show a summary toast; repositories safely skipped due to uncommitted changes or branch locks open the detailed summary report.
- **Quick Actions Modal (`⚡`)**: Opens the two-zone maintenance dialog.

### A. Scope Selector
Choose the target scope at the top of the modal:
- **All Watched (`N`)**: Executes across all discovered repositories in configured watch folders.
- **Filtered (`M`)**: Appears and auto-selects when any `@account`, status, or search filter is active on the dashboard.
- **Single Repo**: Narrows the operation to one repository selected from the dropdown.

### B. Zone 1 — Daily Sync & Maintenance (Constructive, 1-Click)
- **Sync All to Main (`FolderSync`)**:
  - Runs `git fetch --all --prune`, switches clean root worktrees to their default branch (`main`/`master`), and advances commits via `git pull --ff-only`.
  - **Non-Destructive Guarantee**: Dirty root worktrees, default branches checked out in secondary worktrees, and diverged non-fast-forward branches are safely skipped and listed under warnings without touching local files.
  - **Multi-Account CLI Grouping**: Sequentially switches `gh auth` per `@account` group when `autoSwitchAccount` is enabled, restoring your active CLI profile upon completion.

### C. Zone 2 — Destructive Bulk Cleanup
1. **Discard Changes (`FileX2`)**:
   - Reverts modified tracked files (`git reset --hard HEAD`) and removes untracked files (`git clean -ffd`), strictly preserving `.gitignore`d secrets (`.env`).
2. **Nuke Worktrees (`Trash2`)**:
   - Discards uncommitted changes in linked worktrees, force-removes them (`git worktree remove --force --force`), and runs `git worktree prune`. Main root worktree remains immune.
3. **Clean Branches (`GitBranch`)**:
   - Deletes secondary local branches (`git branch -D`) not checked out in active worktrees; default branch remains immune.
4. **Total Fresh Start (`Flame`)**:
   - 5-phase nuclear reset: discards linked worktrees, removes & prunes linked worktrees, checks out default branch on root, cleans root modifications, and deletes secondary local branches.

### D. Pre-Flight Impact Audit & Confirmation Triggers
- Displays live counts (and expandable item lists) before execution, warning prominently if unpushed commits would be deleted.
- **Sync All to Main**: Direct 1-click emerald button.
- **Destructive Presets (1–3)**: Disabled (`Nothing to clean`) when impact is `0`; otherwise uses a 2-step armed button with a 3-second countdown.
- **Total Fresh Start**: Requires typing `"RESET"` into the confirmation field.

---

## 7. Multi-Account GitHub CLI Management

- **Global CLI Identity**: Click the **`@username`** badge in the top toolbar to switch your active global GitHub CLI account (`gh auth switch`).
- **Repository Context**: If a repository belongs to a specific organization or work account (e.g. `@CataVillalobosC`), Workspace Companion highlights the active profile and ensures commit authorship and PR commands target the correct account.

---

## 8. Fast Worktree Creation

At the bottom of each repository section:
1. Type a new branch name into the **`+ Fast Worktree: branch-name`** field.
2. Press `Enter` or click **`⚡ Create & Launch`**.
3. In under 500ms, the tool creates the Git worktree, checks out the branch, and launches your configured IDE. The new worktree is appended straight into the list — no rescan, no flash.

---

## 9. Keyboard Shortcuts

Workspace Companion is designed to be operated without leaving the keyboard:

| Shortcut | Context | Action |
| :--- | :--- | :--- |
| `Alt+Space` | Global | Toggle the floating window |
| `Escape` | Any open modal | Close it (Branch Switcher, New Worktree, Watched Folders, Quick Actions, Remove Worktree, Batch Delete, Branch Batch Delete, GitHub Account) |
| `↑` / `↓` | Branch Switcher Modal | Move the highlighted branch |
| `Enter` | Branch Switcher Modal | Checkout the highlighted branch |
| `Enter` / `Escape` | Fast Worktree Creator & New Worktree Modal branch field | Create now / clear and collapse |


---

## 10. Visual Walkthrough & Demo Guide

For portfolio presentations and animated screencasts, the canonical 15-second visual demonstration covers:

1. **Tray Slide-Out (0s–3s)**: Click the system tray icon or trigger `Alt+Space` to reveal the Spotlight-style floating window next to the tray.
2. **Worktree Discovery (3s–6s)**: Automatic parsing of local repositories with dirty indicator badges (`● dirty (N)`), orphaned branch indicators, and commit hashes.
3. **Dirty-State Inspection (6s–9s)**: Hovering over the `● dirty` badge reveals the modified file list without leaving the main dashboard.
4. **Instant IDE Launch (9s–12s)**: 1-click launch of VS Code / Antigravity IDE and Windows Terminal directly scoped to the selected worktree's filesystem directory.
5. **Safe Batch Cleanup Barrier (12s–15s)**: Multi-selecting secondary worktrees and triggering `Delete Selected`, showcasing the pre-flight dirty check and safety confirmation barrier preventing data loss.
