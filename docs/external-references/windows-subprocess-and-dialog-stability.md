# Windows Subprocess Ergonomics & Dialog Stability

This document outlines key architectural findings and hardening patterns implemented to ensure high stability in Windows desktop environments (Tauri v2 + Rust + Svelte 5).

---

## 1. Subprocess Ergonomics & Non-Interactive Safety in Rust

When executing command-line utilities (such as `git` or `gh`) from a native desktop application on Windows:

1. **Standard Input Detachment (`Stdio::null()`):**
   - By default, `std::process::Command` inherits handles if not specified. In a GUI without an attached console, subcommands that attempt to query `stdin` or interactive prompts will block the calling thread indefinitely.
   - **Fix:** Always enforce `cmd.stdin(Stdio::null())`.

2. **Environment Variable Disabling of Prompts:**
   - Git may attempt credential queries, GPG passphrases, or pager invocations:
     - `GIT_TERMINAL_PROMPT = 0`: Instantly errors instead of waiting on terminal input.
     - `GIT_OPTIONAL_LOCKS = 0`: Prevents non-essential operations from attempting index locks.
     - `CREATE_NO_WINDOW = 0x08000000`: Eliminates flashing cmd windows on Windows.

3. **Handling Windows NTFS File Locks During `git clean`:**
   - On Windows, if any process (such as VS Code, Cursor, terminal, or language server) holds an open file handle, `git clean -fd` fails with exit code 1 (`warning: failed to remove ... Permission denied`).
   - Hardening pattern: Combine `git reset --hard HEAD` with `git clean -ffd` (double `-f` removes untracked nested repos). If `clean` fails specifically due to locked files while `reset` succeeded, the operation should report partial success with a clean explanation rather than aborting the entire pipeline.

4. **Path Normalization for Windows (`comparable_path`):**
   - Direct binary equality comparisons (`PathBuf == PathBuf`) fail on Windows when comparing paths across different sources due to:
     - UNC verbatim prefixes (`\\?\C:\...`)
     - Drive letter casing differences (`C:\` vs `c:\`)
     - Slash direction differences (`\` vs `/`)
   - Normalizing paths before lookup (`strip_verbatim_prefix`, forward slashes, lowercase on Windows) prevents false "not found" errors.

---

## 2. Event Containment & Modal Lifecycle in Svelte

1. **`Escape` Key Containment (`closeOnEscape`):**
   - Any modal or popover closing on `Escape` MUST call `event.preventDefault()` and `event.stopPropagation()`.
   - Failing to call `preventDefault()` allows the key event to bubble to parent window handlers (such as Tauri's `hide_panel`), causing the entire desktop application to vanish into the Windows system tray.

2. **Guaranteed Child Unmount Cleanup:**
   - Any child component that reports busy state to a parent (e.g. `onbusychange(busy: boolean)`) must emit `onbusychange(false)` in its `onDestroy` hook.
   - This ensures that if the component is dismissed or unmounted unexpectedly, parent containers (such as `BranchSwitcherModal`) never remain permanently locked in a disabled state.
