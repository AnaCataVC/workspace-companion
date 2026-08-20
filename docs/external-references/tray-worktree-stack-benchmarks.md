> **Created:** 2026-08-19
> **Last Updated:** 2026-08-19

# Architectural & Stack Benchmarks: Worktree & Workspace Companion

Research reference for the system tray desktop micro-tool managing Git Worktrees and GitHub CLI profiles on Windows.

---

## 1. System Tray Framework Comparison

### A. Tauri v2 (Rust + WebView2)
- **Tray Support**: Native first-class support in Tauri v2 core (`tauri::tray::TrayIconBuilder`).
- **Tray Window Positioning**: Supported via official plugin `tauri-plugin-positioner` (`Position::TrayBottomRight`, `Position::TrayCenter`).
- **Memory Footprint**: ~30 - 50 MB RAM (utilizing system-installed Windows WebView2).
- **Executable Size**: ~5 - 12 MB (lightweight compiled native Rust binary).
- **Subprocess Execution**: `std::process::Command` with Windows flags `CREATE_NO_WINDOW (0x08000000)` to execute background `git` and `gh` processes without command prompt flashes.
- **Reference**: https://v2.tauri.app

### B. Wails v2 / v3 (Go + WebView2)
- **Tray Support**:
  - **Wails v2**: No official native tray API. Requires external third-party Go packages like `github.com/energye/systray`.
  - **Wails v3**: Introduced native `SystemTray` API, but v3 is in alpha/preview status.
- **Memory Footprint**: ~40 - 70 MB RAM.
- **Subprocess Execution**: `os/exec` in Go with `SysProcAttr { HideWindow: true }`.
- **Verdict**: Viable, but Tauri v2 has more mature stable native tray plugins for Windows.

### C. Electron (Node.js + Chromium)
- **Tray Support**: Native `Tray` API in Electron core.
- **Memory Footprint**: ~150 - 300+ MB RAM.
- **Verdict**: Unnecessarily heavy for a micro-utility designed to stay open in the system tray.

---

## 2. Git Worktree & GitHub CLI Machine-Readable Protocols

### Git Worktree Porcelain Protocol
- Command: `git worktree list --porcelain` (or with `-z` for null-terminated output).
- Structure:
  ```text
  worktree <absolute-path>
  HEAD <commit-hash>
  branch refs/heads/<branch-name>
  [locked <reason>]
  [prunable <reason>]
  ```
- **Orphan Detection**:
  - Compare branch upstream via `git branch -vv` to identify `[gone]`.
  - Check merged branches with `git branch --merged` against default branch (`main` / `master`).
  - Safe removal: `git worktree remove <path>` and `git worktree prune`.

### GitHub CLI Protocol
- Query active account: `gh auth status`
- Switch account: `gh auth switch -u <username>`
- Fast non-interactive JSON profile inspection: `gh auth status --json` or parse `~/.config/gh/hosts.yml` / `%APPDATA%\GitHub CLI\hosts.yml`.
