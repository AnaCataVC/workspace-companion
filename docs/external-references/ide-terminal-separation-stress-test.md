> **Created:** 2026-08-30
> **Last Updated:** 2026-08-30

# Adversarial Stress-Test: Dual Architecture (IDE vs Terminal/CLI Separation & Launcher Hardening)

This stress-test conducts a premortem and adversarial vulnerability analysis on **Option 1 (Dual IDE & Terminal Configuration with Visibility Toggles and GUI/CLI Disambiguation)** for Workspace Companion.

---

## 1. Premortem & Operational Failure Modes

### 💥 Vulnerability 1.1: Missing or Non-Standard Binary Paths (Severity: Major)
- **Scenario:** The user configures "Antigravity IDE" or "Cursor", but the IDE is installed in a non-standard directory, has been updated/moved, or is not in `%LOCALAPPDATA%\Programs`.
- **Current Behavior:** Returns a raw error string or fails silently depending on the invocation.
- **Hardening Mitigation:**
  1. In `git.rs`, implement structured binary fallback: check Registry / `%LOCALAPPDATA%` / `%PROGRAMFILES%` / `where.exe`.
  2. If the preferred IDE is unavailable, trigger a user-friendly diagnostic notification and gracefully offer to open in VS Code or File Explorer instead of failing abruptly.

### 💥 Vulnerability 1.2: Windows Terminal (`wt.exe`) Missing on Older/Stripped Windows Installs (Severity: Minor)
- **Scenario:** `wt` is the default terminal, but on Windows 10 without Microsoft Store or on certain enterprise Windows installations, `wt.exe` is absent.
- **Hardening Mitigation:**
  - In `git.rs`, when `open_in_terminal` targets `wt` and fails, automatically fallback to `powershell.exe` or `cmd.exe`.

---

## 2. Concurrency, Race Conditions & State Drift

### 💥 Vulnerability 2.1: Deserialization Crash on Existing `app_config.json` (Severity: Critical / Blocker)
- **Scenario:** Users upgrading from prior versions already have an `app_config.json` without `defaultTerminal` or `showTerminalButton`. If Rust's `AppConfig` does not use `#[serde(default)]`, `serde_json::from_str` fails, resetting user configuration to default.
- **Hardening Mitigation:**
  - Add explicit default functions for all new fields in Rust:
    ```rust
    #[serde(default = "default_terminal")]
    pub default_terminal: String,
    #[serde(default = "default_true")]
    pub show_terminal_button: bool,
    ```
  - In TypeScript, always provide nullish coalescing defaults:
    ```typescript
    localDefaultTerminal = config.defaultTerminal ?? 'wt';
    localShowTerminalButton = config.showTerminalButton ?? true;
    ```

### 💥 Vulnerability 2.2: Dual-State Drift between `localStorage` and `AppConfig` (Severity: Major)
- **Scenario:** If any component still reads `localStorage.getItem('workspace_preferred_editor')`, changing settings in `WatchFoldersModal` will create a split-brain UI where some cards update and others keep stale cached settings.
- **Hardening Mitigation:**
  - Completely eradicate `localStorage.getItem('workspace_preferred_editor')` and `localStorage.setItem` from all components (`WorktreeCard`, `WorktreeItemRow`, `QuickWorktreeInline`).
  - Derive reactive values directly from `$appConfig`.

---

## 3. Cost & Resource Explosion (Performance & OS Vectors)

### 💥 Vulnerability 3.1: Subprocess Flooding on Rapid Multi-Clicks (Severity: Minor)
- **Scenario:** An impatient user clicks the IDE or Terminal button 5 times rapidly.
- **Hardening Mitigation:**
  - Implement a 400ms UI debounce / busy lock on frontend launcher buttons.

---

## 4. Security, Authorization & Abuse Vectors

### 💥 Vulnerability 4.1: Windows `cmd.exe` Argument Injection in `start` (Severity: Major)
- **Scenario:** `git.rs` currently spawns:
  `cmd /C start "" <bin> <path>`
  If `<path>` contains spaces, ampersands (`&`), or parentheses (common in worktree folder names), `cmd.exe` can misinterpret arguments or open the wrong directory.
- **Hardening Mitigation:**
  - Avoid passing raw shell strings through `cmd /C start`.
  - For standard binaries in PATH (`code`, `cursor`, `powershell`, `wt`), invoke them directly via `Command::new(bin).arg(path).creation_flags(CREATE_NO_WINDOW).spawn()`.
  - Only use `cmd /C start` when launching non-executable document associations, and ensure proper argument escaping.

---

## 5. Developer Friction & Backward Compatibility Breakage

### 💥 Vulnerability 5.1: Breaking Tauri IPC Signatures (Severity: Minor)
- **Scenario:** Changing IPC command signatures from `open_in_editor(editor, path)` could break mock tests or existing callers.
- **Hardening Mitigation:**
  - Keep `open_in_editor(editor: String, path: String)` backwards compatible, and add `open_in_terminal(terminal: String, path: String)`.

---

## Hardening Checklist Summary

| # | Vulnerability | Severity | Hardening Measure |
|---|---|---|---|
| 1 | `app_config.json` deserialization breakage | **Critical** | Add `#[serde(default)]` in Rust + TypeScript fallbacks |
| 2 | `localStorage` state drift | **Major** | Remove `localStorage` and bind directly to `$appConfig` |
| 3 | `cmd /C start` path quoting bugs | **Major** | Launch binaries directly with `Command::new()` and proper args |
| 4 | Missing IDE/Terminal executables | **Minor** | Add smart fallback cascade (wt -> powershell -> cmd) |
| 5 | Multi-click subprocess spawn | **Minor** | Add click debounce in Svelte components |