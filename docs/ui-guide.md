# 🧭 User Interface & Navigation Guide

This document provides a comprehensive visual reference and operational guide for the **Workspace Companion** desktop interface.

---

## 1. Floating Dashboard Overview

**Workspace Companion** operates as a Spotlight-style floating window that slides out from the Windows System Tray upon clicking the tray icon or pressing the global shortcut (`Ctrl+Shift+W`).

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ [Icon] WORKSPACE COMPANION             [▦/▤ View] [+ New] [@gh-user] [⚙] [📌] [↻]│ Top Toolbar
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

You can switch between view layouts using the **View Mode** toggle button (`▤ / ▦`) in the top-right toolbar:

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

1. **Select Worktrees**: Click the checkbox on any secondary worktree row, or click **`Select All`** on the repository header.
2. **Review Pending Count**: The bottom floating bar indicates `N worktrees selected`.
3. **Trigger Deletion**: Click **`Delete Selected`**.
4. **Safety Verification**: Workspace Companion executes pre-flight dirty checks (`git status --porcelain`) on each selected worktree. If any worktree contains uncommitted modifications, it prompts for explicit confirmation, preventing accidental code loss.

---

## 5. Multi-Account GitHub CLI Management

- **Global CLI Identity**: Click the **`@username`** badge in the top toolbar to switch your active global GitHub CLI account (`gh auth switch`).
- **Repository Context**: If a repository belongs to a specific organization or work account (e.g. `@CataVillalobosC`), Workspace Companion highlights the active profile and ensures commit authorship and PR commands target the correct account.

---

## 6. Fast Worktree Creation

At the bottom of each repository section:
1. Type a new branch name into the **`+ Fast Worktree: branch-name`** field.
2. Press `Enter` or click **`⚡ Create & Launch`**.
3. In under 500ms, the tool creates the Git worktree, checks out the branch, and launches your configured IDE. The new worktree is appended straight into the list — no rescan, no flash.

---

## 7. Keyboard Shortcuts

Workspace Companion is designed to be operated without leaving the keyboard:

| Shortcut | Context | Action |
| :--- | :--- | :--- |
| `Ctrl+Shift+W` | Global | Toggle the floating window |
| `Escape` | Any open modal | Close it (Branch Switcher, New Worktree, Watched Folders, Orphan Cleaner, Batch Delete, GitHub Account) |
| `↑` / `↓` | Branch Switcher Modal | Move the highlighted branch |
| `Enter` | Branch Switcher Modal | Checkout the highlighted branch |
| `Enter` / `Escape` | Fast Worktree Creator & New Worktree Modal branch field | Create now / clear and collapse |
