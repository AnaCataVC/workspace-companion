<p align="center">
  <img src="icon.png" alt="workspace-companion Logo" width="120" />
</p>

# Worktree & Workspace Companion

[English](README.md) | [Español](README.es.md)

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
- 🛡️ **Safer destructive actions**: discarding changes before a branch switch lists the files that will be lost and only arms the confirm button after a short delay; batch delete re-checks each selected worktree against the latest scan and flags selections hidden by the current filter.

#### 🧭 UI Anatomy at a Glance:
Each worktree row displays the **Git Branch Badge** (left interactive button to switch branches) and the **Physical Folder Path** (center/right name referenced by CLI tools and AI agents) side-by-side:
```text
┌────────────────────────────┐       ┌────────────────┐
│ [⎇] feature/branch-name [▼]│       │ worktree-folder│   [● dirty]   [🚀 IDE] [>_] [📁] [🗑]
└────────────────────────────┘       └────────────────┘
```
For a comprehensive breakdown of all toolbar buttons, status indicators, and view density modes, see the **[User Interface & Navigation Guide](docs/ui-guide.md)**.

---

### 2. Technologies Used
- **Backend**: Rust 1.97+, Tauri v2 (`tauri::tray::TrayIconBuilder`, `tauri-plugin-positioner`, `tauri-plugin-global-shortcut`).
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, Lucide Icons.
- **Protocols & OS Integration**: Git Porcelain Protocol (`git worktree list --porcelain`), GitHub CLI (`gh`), Windows Win32 API (`CREATE_NO_WINDOW`).

---

### 3. Key Learnings
- **Subprocess Ergonomics in Windows**: Avoiding flashing terminal prompts when executing background developer CLIs requires low-level process creation flags (`CREATE_NO_WINDOW = 0x08000000`).
- **IDE vs. CLI Disambiguation**: Differentiating between GUI desktop applications (e.g. `Antigravity.exe`) and command-line companion utilities (`agy.exe`) ensures predictable workspace launching.
- **Resilient Configuration Migration**: Using `#[serde(default)]` in Rust and nullish coalescing in TypeScript guarantees zero data loss and prevents deserialization crashes when extending configuration schemas.
- **Safety Guardrails in Tooling**: Automating `git worktree remove` demands pre-flight dirty checks (`git status --porcelain`) before triggering destructive actions.
- **Repo-Wide Context Caching & Delta-Patch Updates**: Computing orphan-detection facts (default branch, `branch -vv`, `branch --merged`) once per repository instead of once per worktree cuts subprocess spawns roughly 5x on multi-worktree repos; patching the in-memory worktree list after a single mutation — instead of re-scanning every watched folder — keeps the UI from flashing empty and avoids paying that cost for a one-worktree change.
- **Atomic Bulk Cleanup & Git Lock Isolation**: Chaining destructive Git commands across multiple repositories requires strictly bounded concurrency: parallelizing across distinct repositories via Rayon (`par_iter`) while strictly sequencing steps within each repository prevents `.git/index.lock` collisions and Windows OS file lock conflicts.

---

### 4. Local Setup Instructions

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

#### Run Tests
```powershell
# Run backend Rust tests
cd src-tauri; cargo test; cd ..

# Check frontend types and build
npm run check
npm run build
```

---

### 5. Documentation & Architectural Decisions
- 🧭 [User Interface & Navigation Guide](docs/ui-guide.md)
- 📖 [Technical Architecture & Design](docs/architecture.md)
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


---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

