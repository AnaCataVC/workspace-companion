> **Created:** 2026-09-02
> **Last Updated:** 2026-09-02

# IDE & Editor Launcher Architecture & Windows Path Resolution

## 1. Overview & Problem Definition

In **Workspace Companion** (Tauri v2 + Rust backend), opening worktree folders in external IDEs previously delegated to `cmd /C start "" <editor_bin> <path>`.

On Windows, invoking `code` triggers `code.cmd` (a batch file wrapper located in `...\Microsoft VS Code\bin\code.cmd`). Running batch scripts through `cmd.exe /C start` forces Windows to instantiate an interactive console window (`conhost.exe`), parse environment variables, run Node.js wrapper scripts, and only then launch the Electron binary (`Code.exe`).

This caused:
1. An intrusive black console window popup flashing on the user's screen.
2. Latency of 2–4 seconds per launch.
3. Vulnerability to Windows cmd command-line argument splitting on paths with spaces, ampersands, or parentheses.

---

## 2. Windows IDE Installation Paths Benchmark

| Editor ID | Application Name | Default User Setup Path (`%LOCALAPPDATA%`) | System Setup Path (`%ProgramFiles%`) | PATH Bin Name | GUI Executable Target |
|---|---|---|---|---|---|
| `vscode` | Visual Studio Code | `%LOCALAPPDATA%\Programs\Microsoft VS Code\Code.exe` | `%ProgramFiles%\Microsoft VS Code\Code.exe` | `code` | `Code.exe` |
| `vscode` | VS Code (32-bit fallback) | N/A | `%ProgramFiles(x86)%\Microsoft VS Code\Code.exe` | `code` | `Code.exe` |
| `vscode` | VS Code Insiders | `%LOCALAPPDATA%\Programs\Microsoft VS Code Insiders\Code - Insiders.exe` | `%ProgramFiles%\Microsoft VS Code Insiders\Code - Insiders.exe` | `code-insiders` | `Code - Insiders.exe` |
| `cursor` | Cursor IDE | `%LOCALAPPDATA%\Programs\cursor\Cursor.exe` | `%ProgramFiles%\Cursor\Cursor.exe` | `cursor` | `Cursor.exe` |
| `windsurf` | Windsurf IDE | `%LOCALAPPDATA%\Programs\windsurf\Windsurf.exe` | `%ProgramFiles%\Windsurf\Windsurf.exe` | `windsurf` | `Windsurf.exe` |
| `antigravity` | Antigravity IDE | `%LOCALAPPDATA%\Programs\Antigravity\Antigravity.exe` | `%ProgramFiles%\Antigravity\Antigravity.exe` | `antigravity` | `Antigravity.exe` |
| `antigravity` | Antigravity IDE (alt) | `%LOCALAPPDATA%\Programs\Antigravity IDE\Antigravity.exe` | `%ProgramFiles%\Antigravity IDE\Antigravity.exe` | `antigravity` | `Antigravity.exe` |
| `explorer` | Windows File Explorer | N/A (Native Shell API `open::that`) | N/A | `explorer` | `explorer.exe` |

---

## 3. Dynamic PATH & Custom Installation Resolution

In addition to standard hardcoded directories, portable or custom installations are resolved via `where.exe <bin>`:
- When `where.exe code` returns `<install_dir>\bin\code.cmd` or `<install_dir>\bin\code`, the parent directory `<install_dir>\Code.exe` is inspected.
- If `<install_dir>\Code.exe` exists, it is selected as the primary launch target.

---

## 4. Subprocess Execution & Windows Flags

Direct execution uses Rust's `std::process::Command` without shell wrapping:

```rust
use std::process::{Command, Stdio};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn launch_gui_editor(exe_path: &Path, target_dir: &str) -> Result<(), String> {
    let mut cmd = Command::new(exe_path);
    cmd.arg(target_dir);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn().map_err(|e| format!("Failed to launch {:?}: {}", exe_path, e))?;
    Ok(())
}
```

### Benefits:
1. **0 Console Allocation**: Native Win32 GUI subsystem executables spawn instantly with no window flash.
2. **No Command Injection**: Arguments are passed directly via `CreateProcessW` array parameter, completely bypassing `cmd.exe` string interpolation risks.
3. **No Zombie Pipes**: `Stdio::null()` detaches standard I/O handles.
