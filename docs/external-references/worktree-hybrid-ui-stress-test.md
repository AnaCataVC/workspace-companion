> **Created:** 2026-08-26
> **Last Updated:** 2026-08-26

# Adversarial Stress-Test: Hybrid Worktree UI Architecture & Inline Engine

**Target Proposal:** Option 3 — Hybrid Worktree UI (Tree Hierarchy, 1-Click Action Dock, Inline Quick Creator, Git Diff Badges, Density Toggle).

---

## 1. 💥 Critical Failure Modes & Premortem Analysis

### 🚨 Attack Vector A: The "Windows Process Spawning Storm" (N+1 Subprocess Bottleneck)
- **Failure Scenario:** A developer has 4 watched folders with 8 Git repositories, each having 3-6 active worktrees (approx. 35 total worktrees).
- **Vulnerability:** If every worktree executes `git status --porcelain`, `git log -1`, and `git diff --shortstat` on each scan/refresh, the backend spawns **over 100+ Windows subprocesses**. On Windows, `CreateProcessW` carries high overhead (~20ms per execution), resulting in a **2.5 to 5-second UI freeze** or high CPU spike every time the tray opens.
- **Severity:** `[Critical / Blocker]`
- **Hardening Mitigation:**
  1. **Lazy Loading for Diffs:** Do *not* fetch diff statistics eagerly during the global repository scan. Query `git diff --shortstat` on-demand (only when hovering or clicking a specific worktree).
  2. **Rayon / Parallel Iterator:** Use Rust's `rayon` parallel iterator for scanning independent repositories concurrently across CPU cores, while batching git queries.

---

### 🚨 Attack Vector B: `.git/index.lock` Race Conditions on 1-Click Actions
- **Failure Scenario:** The user rapidly clicks the inline creation button while a background agent or build tool is performing git operations, or double-clicks the "Delete/Clean" button.
- **Vulnerability:** Git fails immediately with `fatal: Unable to create '.git/index.lock': File exists`. If unhandled, the frontend shows an uninformative crash error and leaves the worktree list in a corrupted, half-created state on disk (directory created without registered git worktree).
- **Severity:** `[Major / Hardening Required]`
- **Hardening Mitigation:**
  1. **UI Mutation Locking:** Disable the inline input and show a micro-spinner on the card while a creation/deletion promise is pending.
  2. **Atomic Rollback in Rust:** If `git worktree add` fails after creating a directory, immediately clean up the orphaned directory.
  3. **Debounced Refresh:** Debounce window focus refresh events (300ms) to avoid triggering scan storms during rapid alt-tabs.

---

### 🚨 Attack Vector C: Fragile Default Branch & Detached HEAD Assumptions
- **Failure Scenario:** A repository has a detached `HEAD` in main worktree, or uses non-standard default branch names (`develop`, `staging`, `trunk`, or custom enterprise naming conventions), or is a bare repository.
- **Vulnerability:** An anchor-tree UI that hardcodes `main`/`master` as the pinned root node will fail to render the hierarchy properly or create new worktrees branching from a non-existent `main`.
- **Severity:** `[Major / Hardening Required]`
- **Hardening Mitigation:**
  1. **Dynamic Root Detection:** The first worktree returned in `git worktree list --porcelain` is by Git design the **main worktree** of the repository (regardless of its branch name). Use this entry as the anchor node.
  2. **Fallback Base Branch:** In the inline creator, default to the currently active branch of the main worktree if `origin/HEAD` is not resolved.

---

### 🚨 Attack Vector D: Input Sanitization & Path Collision in Inline Creation
- **Failure Scenario:** The user enters branch names with special characters (`feat/new#1?`, `../escape`, `feature with spaces`, or existing branch names).
- **Vulnerability:** Git command failure or inadvertent creation of messy directory paths on the Windows filesystem.
- **Severity:** `[Major / Hardening Required]`
- **Hardening Mitigation:**
  1. **Pre-flight Slugify:** Sanitize the input live: replace spaces and slashes with hyphens for the directory name, while keeping valid Git ref characters for the branch name.
  2. **Dry-Run Check:** Rust checks `git check-ref-format --branch <input>` and verifies directory non-existence before executing `git worktree add`.

---

### 🚨 Attack Vector E: Loss of Advanced Creation Capabilities
- **Failure Scenario:** A developer needs to create a worktree from a specific detached commit SHA, a remote tracking branch (`origin/feat-123`), or custom target folder.
- **Vulnerability:** Pure inline creation lacks input fields for base branch selection or custom target directory paths.
- **Severity:** `[Minor / Ergonomics]`
- **Hardening Mitigation:**
  1. **Hybrid Creation Access:** Keep a secondary icon button `[ ⚙️ Advanced... ]` next to the inline creator that opens the full `NewWorktreeModal.svelte` when complex options are required.

---

## 2. 🛡️ Summary of Hardening Architecture & Invariants

```text
[ Frontend (Svelte 5) ]
  ├── Density Mode Toggle (Stored in localStorage)
  ├── Debounced Input with Real-Time Sanitization Slug
  ├── Mutation Locks during Git execution (prevents double-clicks)
  └── Lazy Diff Popovers (query backend only on Hover/Focus)

[ Backend (Rust Services) ]
  ├── Lazy Diff Execution (no global diff scanning)
  ├── Main Worktree Anchor Detection (Index 0 of git worktree list)
  ├── Atomic Directory Rollback on Failed Worktree Creation
  └── Parallel Repo Scanning via Rayon / Threads
```
