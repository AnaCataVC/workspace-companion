# AGENTS.md — AI Agent Guidelines & Architecture Manual

This document serves as the operational manual, architecture reference, and workflow guide for AI coding agents operating within the **Worktree & Workspace Companion** repository.

---

## 1. Project Overview & Architecture

**Worktree & Workspace Companion** is an ultra-lightweight, native Windows system tray utility and Spotlight-style floating dashboard built with **Tauri v2**, **Rust**, **Svelte 5**, and **Tailwind CSS**. It streamlines developer workflows by managing Git Worktrees and multi-account GitHub CLI setups.

### Core Architecture:
- **`src-tauri/` (Rust Backend)**:
  - System tray initialization & Spotlight floating window positioning (`tauri::tray::TrayIconBuilder`, `tauri-plugin-positioner`, `tray.rs`).
  - Safe subprocess execution with `CREATE_NO_WINDOW = 0x08000000` to prevent console flashing on Windows.
  - Modular services layer in `src-tauri/src/services/` (`git.rs`, `gh.rs`, `worktree_cleaner.rs`, `config.rs`).
  - Tauri IPC command handlers in `src-tauri/src/commands/` (`worktrees.rs`, `gh_auth.rs`).
- **`src/` (Svelte 5 & TypeScript Frontend)**:
  - `App.svelte`: Root floating panel, active views, and global shortcuts.
  - `lib/components/`: Modular Svelte 5 components (`WorktreeList`, `WorktreeCard`, `NewWorktreeModal`, `BranchSwitcherModal`, `RemoveWorktreeModal`, `WatchFoldersModal`, `GhAccountModal`, `Header`, `AccountFilterBar`).
  - `lib/stores/`: Reactive store modules (`worktrees.ts`, `ghAuth.ts`, `appConfig.ts`, `editors.ts`).
  - `lib/types.ts`: TypeScript contracts and data structures.
  - Modern pastel theming and Tailwind CSS styling.
- **`releases/`**: Standalone installers (`.msi`, `setup.exe`, Portable `.exe`).

---

## 2. Directory Structure

```text
workspace-companion/
├── src/                           # Svelte 5 frontend application
│   ├── lib/                       # Svelte components, stores, & utilities
│   │   ├── components/            # UI components (cards, modals, header, filters)
│   │   ├── stores/                # Reactive state management stores
│   │   └── types.ts               # Shared TypeScript interfaces & types
│   ├── app.css                    # Tailwind CSS directives and custom scrollbars
│   ├── App.svelte                 # Root application component
│   └── main.ts                    # Frontend entrypoint
├── src-tauri/                     # Rust backend & Tauri configuration
│   ├── src/                       # Rust source code
│   │   ├── commands/              # Tauri IPC command invocations
│   │   ├── services/              # Git engine, GitHub CLI, cleaner, and config services
│   │   ├── tray.rs                # System tray & window focus management
│   │   ├── lib.rs                 # Tauri plugin builder and setup logic
│   │   └── main.rs                # Rust binary entrypoint
│   ├── Cargo.toml                 # Rust dependencies & metadata
│   ├── tauri.conf.json            # Tauri v2 application configuration
│   └── capabilities/              # Tauri v2 security capabilities
├── docs/                          # Project documentation & references
│   └── external-references/       # Topic-specific research & technical references
├── public/                        # Static assets, logos, and tray icons
├── releases/                      # Compiled desktop installers (gitignored)
├── package.json                   # Frontend dependencies and npm scripts
└── README.md                      # Bilingual project documentation (EN/ES)
```

---

## 3. Mandatory Agent Rules & Directives

### 🌐 Language & Communication
- **Source Code**: All Rust and TypeScript code (types, functions, variables, comments) MUST be in **English**.
- **User Chat**: Communicate with the user in **Spanish** unless requested otherwise.
- **Git Commits**: Use **Conventional Commits** in **English** (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`).
- **README**: Maintain bilingual documentation (English and Spanish).

### 🔒 Security & Privacy
- **Absolute Paths**: NEVER leak absolute user paths (e.g., `C:\Users\...`) into code, documentation, or commit messages. Always use relative paths or placeholders.
- **Git Guardrails**: Before automating or calling `git worktree remove`, ALWAYS execute pre-flight dirty checks (`git status --porcelain`) to avoid deleting uncommitted changes.

### 💻 PowerShell Environment
- **Command Chaining**: NEVER use `&&` or `||` in terminal commands. Use `;` or separate sequential commands.
- **GitHub CLI Context**: Switch to personal account `AnaCataVC` (`gh auth switch -u AnaCataVC --hostname github.com 2>$null`).

---

## 4. Development & Build Commands (PowerShell)

### Development Mode
```powershell
# Install frontend dependencies
npm install

# Run application in desktop development mode (Hot Module Reloading)
npx tauri dev
```

### Type Checking & Build
```powershell
# Run Svelte and TypeScript type check
npm run check

# Run Rust backend tests
cd src-tauri; cargo test; cd ..

# Build production frontend
npm run build

# Build complete Tauri desktop installer and standalone release
npx tauri build
```

---

## 5. Architectural Standards & Best Practices

1. **Subprocess Ergonomics**: Any subprocess spawned from Rust (`std::process::Command`) to run `git` or `gh` must use Windows creation flag `0x08000000` (`CREATE_NO_WINDOW`) and `Stdio::null()` to eliminate console pop-ups and prevent lingering I/O handles.
2. **Git Porcelain Parsing**: Parse machine-readable porcelain formats (`git worktree list --porcelain`, `git status --porcelain`) to ensure compatibility across localized Git installations.
3. **Tray Auto-Hide**: The floating window must toggle visibility on tray icon click and auto-hide when losing window focus (`tauri::WindowEvent::Focused(false)`).
4. **Config Schema Evolution & Backward Compatibility**: Always annotate new fields in `AppConfig` with `#[serde(default = "...")]` in Rust and provide fallback defaults via nullish coalescing (`??`) in TypeScript. This prevents deserialization failures on existing `app_config.json` files.
5. **Launcher Disambiguation & Subprocess Safety**: Always distinguish between GUI code editors (`open_in_editor`) and interactive CLI/terminal environments (`open_in_terminal`). Escape literal paths when interpolating into shell commands (e.g. `path.replace('\'', "''")` for PowerShell).
6. **Direct GUI IDE Binary Resolution**: GUI code editors (VS Code, Cursor, Windsurf, Antigravity IDE) must be resolved directly to their native Win32 `.exe` via `resolve_gui_binary` (inspecting `%LOCALAPPDATA%`, `%ProgramFiles%`, Insiders, and dynamic PATH parent un-nesting) and spawned without shell wrappers to guarantee instant, zero-console execution.
