<p align="center">
  <img src="icon.png" alt="workspace-companion Logo" width="120" />
</p>

# Worktree & Workspace Companion

[English](README.md) | [Español](README.es.md)

[![CI](https://github.com/AnaCataVC/workspace-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/AnaCataVC/workspace-companion/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.97%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri&logoColor=white)](https://v2.tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-v5-ff3e00?logo=svelte&logoColor=white)](https://svelte.dev)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-v3-38bdf8?logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
[![GitHub CLI](https://img.shields.io/badge/GitHub%20CLI-Integrated-2088FF?logo=github&logoColor=white)](https://cli.github.com)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

---

### 1. Project Description
**Worktree & Workspace Companion** is an ultra-lightweight, native system tray utility and Spotlight-style floating dashboard for Windows designed to streamline developer workflows with Git Worktrees and multi-account GitHub CLI setups.

#### ✨ Core Capabilities:
- 📊 **Active Worktrees Dashboard**: Discovers local repositories, parses active worktrees, detached heads, locked states, and dirty working trees with zero console flickering.
- 💻 **Independent IDE & Terminal Launchers**: Configurable 1-click launching for preferred code editors (VS Code, Antigravity IDE, Cursor, Windsurf) and consoles (Windows Terminal, PowerShell, CMD, Git Bash, AGY CLI) with optional terminal quick-button toggle.
- 🧹 **Orphaned Worktree Cleaner**: Safely identifies worktrees whose remote upstream branch has been deleted or merged, equipped with pre-flight dirty checks to prevent accidental loss of uncommitted work.
- 🌿 **Branch Cleaner**: Lists every local branch across managed repos — including ones with no worktree of their own — flagged as merged, remote-gone, or protected, for safe bulk deletion. The default branch and any checked-out branch are never deletable, even when forcing an unmerged one.
- 🔄 **1-Click GitHub CLI Profile Switcher**: Instantly toggle between personal and corporate GitHub CLI identities (e.g. `gh auth switch`) and prevent author mismatch.
- ⚡ **Quick Actions & Bulk Cleanup**: One-click atomic workspace maintenance across a single repository or all watched folders simultaneously. Offers four bounded presets: discarding uncommitted changes without touching `.gitignore`d secrets, nuking secondary worktrees, cleaning merged/local branches, or performing a 5-phase **Total Fresh Start** reset to default branch. Protected by pre-flight impact previews (unpushed commit audits) and armed safety confirmation triggers (3-second countdown or typed `"RESET"` verification).
- 🪟 **Desktop Window with System Tray Integration**: Resides silently in the Windows system tray (<40 MB RAM in background) and opens next to the tray icon.
  - **Toggle**: single-click the tray icon, or press **Alt+Space** from anywhere (global shortcut; if another app already owns it, the tray still works).
  - **Hide**: the window's **X** and **Esc** (with no dialog open) hide the panel to the tray instead of quitting; quit from the tray menu.
  - **Always on Top (Pin)**: the window stays visible as a standard desktop application without disappearing when moved or blurred; clicking the **Pin** button toggles *Always on Top* so it stays above other windows. The pin state is saved and restored on next launch.

#### 🎬 Visual Product Interface:
Each worktree row displays the **Git Branch Badge** (interactive button to switch branches) and the **Physical Folder Path** (directory name referenced by CLI tools and AI agents) side-by-side:
```text
┌──────────────────────────────────────────────────────────────────────────┐
│ [Icon] WORKSPACE COMPANION             [▦/▤ View] [+ New] [@gh-user] [⚙] [⚡] [📌] [↻]│ Top Toolbar
├──────────────────────────────────────────────────────────────────────────┤
│ 🔍 Filter worktree, branch or path...                                    │ Quick Search
├──────────────────────────────────────────────────────────────────────────┤
│ [📁 workspace-companion] [3 worktrees] [@AnaCataVC]        [Select All] [🧹 Clean]│ Repo Section Header
│                                                                          │
│  ⚓ [ ⎇ main ▼ ]             workspace-companion  [● clean]      [🚀] [>_] [📁]    │ Root Worktree
│  ├─ [ ⎇ feat/e2e-benchmarks▼] companion-benchmarks [▲ Orphan]     [🚀] [>_] [📁] [🗑]│ Linked Worktree 1
│  └─ [ ⎇ fix/lock-safety   ▼] companion-safety     [● dirty (2)]  [🚀] [>_] [📁] [🗑]│ Linked Worktree 2
│                                                                          │
│  [+ Fast Worktree: branch-name                       ] [⚡ Create & Launch]│ Inline Quick Creator
├──────────────────────────────────────────────────────────────────────────┤
│ [ 2 worktrees selected ]                          [✕ Cancel] [🗑 Delete Selected]│ Floating Batch Bar
└──────────────────────────────────────────────────────────────────────────┘
```
For a comprehensive breakdown of all toolbar buttons, status indicators, and view density modes, see the **[User Interface & Navigation Guide](docs/ui-guide.md)**.

---

### 2. Safety Model & Data Protection Guardrails

Worktree & Workspace Companion automates high-consequence Git operations without sacrificing repository integrity or risking uncommitted developer work:

1. 🛡️ **Dirty-State Preflight Guard**: Before deleting any worktree or switching branches, the engine executes a non-destructive pre-flight audit via `git status --porcelain`. Worktrees with uncommitted, unstaged, or untracked changes cannot be removed without explicit two-stage override.
2. 🔍 **Unpushed-Commit & Remote Sync Audit**: Prior to destructive bulk actions (such as Quick Actions or Branch Cleaner), the system audits commits against upstream (`@{u}`) and push targets (`@{push}`). Any unpushed local commit is flagged in the impact summary with commit count and author metadata.
3. 🔒 **Protected & Checked-Out Invariants**: The repository's default branch (`main` / `master`) and any branch currently checked out in an active worktree are strictly protected. They cannot be deleted or forced-deleted under any circumstance.
4. 🔄 **Latest-State Revalidation**: Target states are dynamically re-verified against the physical filesystem immediately before batch deletion begins, preventing race conditions or execution on stale filter results.
5. ⏱️ **Two-Stage Arming & Confirmation Barriers**: High-impact bulk cleanups feature armed confirmation triggers: destructive presets require a 3-second arming delay or explicit typed confirmation (`"RESET"` or `"FORCE"`).
6. 📁 **Windows Long Paths & Verbatim Filesystem Resilience**: To prevent `Filename too long` (`ENAMETOOLONG`) or `AccessDenied (os error 5)` on deep folder structures (e.g. nested `node_modules`), directory cleanup uses Windows verbatim paths (`\\?\`), automatically strips `FILE_ATTRIBUTE_READONLY` flags before deletion, and safely unlinks symlinks/junctions without recursing into target trees.

---

### 3. Technologies Used
- **Backend**: Rust 1.97+, Tauri v2 (`tauri::tray::TrayIconBuilder`, `tauri-plugin-positioner`, `tauri-plugin-global-shortcut`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Protocols & OS Integration**: Git Porcelain Protocol (`git worktree list --porcelain`), GitHub CLI (`gh`), Windows Win32 API (`CREATE_NO_WINDOW`).

---

### 4. Key Learnings & Architectural Decisions
- **Subprocess Ergonomics in Windows**: Avoiding flashing terminal prompts when executing background developer CLIs requires low-level process creation flags (`CREATE_NO_WINDOW = 0x08000000`).
- **IDE vs. CLI Disambiguation**: Differentiating between GUI desktop applications (e.g. `Antigravity.exe`) and command-line companion utilities (`agy.exe`) ensures predictable workspace launching.
- **Resilient Configuration Migration**: Using `#[serde(default)]` in Rust and nullish coalescing in TypeScript guarantees zero data loss and prevents deserialization crashes when extending configuration schemas.
- **Safety Guardrails in Tooling**: Automating `git worktree remove` demands pre-flight dirty checks (`git status --porcelain`) before triggering destructive actions.
- **Repo-Wide Context Caching & Delta-Patch Updates**: Computing orphan-detection facts (default branch, `branch -vv`, `branch --merged`) once per repository instead of once per worktree cuts subprocess spawns roughly 5x on multi-worktree repos ([see benchmark](docs/benchmarks/context-caching-benchmark.md)); patching the in-memory worktree list after a single mutation keeps the UI from flashing empty.
- **Atomic Bulk Cleanup & Git Lock Isolation**: Chaining destructive Git commands across multiple repositories requires strictly bounded concurrency: parallelizing across distinct repositories via Rayon (`par_iter`) while strictly sequencing steps within each repository prevents `.git/index.lock` collisions and Windows OS file lock conflicts.

---

### 5. Local Setup Instructions

#### Prerequisites
- [Node.js](https://nodejs.org/) (v20+ recommended) & `npm`
- [Rust & Cargo](https://rustup.rs/) (v1.80+)
- [Git](https://git-scm.com/) & [GitHub CLI](https://cli.github.com/)

#### Installation & Development
```powershell
# 1. Clone the repository
git clone https://github.com/AnaCataVC/workspace-companion.git
cd workspace-companion

# 2. Install frontend dependencies
npm install

# 3. Run in desktop development mode (with Hot Module Reloading)
npx tauri dev

# 4. Run frontend standalone preview
npm run dev
```

#### Automated Testing & Verification
Workspace Companion maintains a comprehensive automated test pyramid across both backend Rust services and frontend Svelte components:

```powershell
# Run backend Rust test suite (Unit, Integration & Benchmarks)
cd src-tauri; cargo test; cd ..

# Run reproducible context caching benchmark (~5x subprocess reduction)
cd src-tauri; cargo test --test context_caching_benchmark -- --nocapture; cd ..

# Check frontend types and production build
npm run check
npm run build
```

**Quality Gates Automated in CI:**
- 🛡️ **Disposable Git Repositories**: Real temporary repositories created via `tempfile` testing 9 orthogonal scenarios: clean/dirty worktrees, detached `HEAD`, remote-gone upstream branches, merged branches, protected/default branch guards, unpushed commit audits, locked worktrees (`git worktree lock`), long paths, and safe directory removal.
- ⚡ **Zero Regressions & Bounded Concurrency**: 100% automated test pass rate with strict concurrency boundaries (Rayon `par_iter` across repos; strictly sequential within each repo to eliminate `.git/index.lock` collisions).

---

### 6. Documentation & Architectural Decisions
- 🧭 [User Interface & Navigation Guide](docs/ui-guide.md)
- 📖 [Technical Architecture & Design](docs/architecture.md)
- 📊 [Benchmark: Subprocess Reduction & Context Caching](docs/benchmarks/context-caching-benchmark.md)
- 🏛️ [Architectural Decision Records (ADRs)](docs/adr/)
  - [ADR 0001: Tauri v2 System Tray & Window Positioning](docs/adr/0001-tauri-v2-system-tray-and-window-positioning.md)
  - [ADR 0002: Windows Subprocess Creation Flag](docs/adr/0002-windows-subprocess-no-window-flag.md)
  - [ADR 0003: Git Porcelain Protocol & Pre-flight Safety](docs/adr/0003-git-porcelain-preflight-safety.md)
  - [ADR 0004: Dual IDE & Terminal Separation and Subprocess Launcher Hardening](docs/adr/0004-ide-terminal-separation-and-subprocess-hardening.md)
  - [ADR 0005: Delta-Patch State Updates and Repo-Wide Context Caching Over Full Rescans](docs/adr/0005-delta-patch-state-and-repo-context-caching.md)
  - [ADR 0006: Branch Cleaner Safety Guards](docs/adr/0006-branch-cleaner-safety-guards.md)
  - [ADR 0007: Quick Actions and Atomic Bulk Cleanup Architecture](docs/adr/0007-quick-actions-and-atomic-bulk-cleanup.md)
  - [Formal Contract: Quick Actions & Bulk Cleanup](docs/contracts/quick-actions-and-bulk-cleanup.contract.md)
- 🤝 [Contributing Guidelines](CONTRIBUTING.md)

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
