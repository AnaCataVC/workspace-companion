# ADR 0002: Windows Subprocess Creation Flag (CREATE_NO_WINDOW)

## Status
Accepted

## Context
When executing external command-line binaries (`git`, `gh`, or code editors) via Rust's `std::process::Command` on Windows, the operating system by default flashes a brief, distracting console window for each background invocation.

## Decision
We enforce the Win32 creation flag `CREATE_NO_WINDOW = 0x08000000` on all background subprocess invocations:

```rust
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

let output = Command::new("git")
    .creation_flags(CREATE_NO_WINDOW)
    .args(&["worktree", "list", "--porcelain"])
    .output()?;
```

## Consequences
- **Positive**: Completely eliminates terminal flashing and provides a smooth desktop UI experience.
- **Negative**: Subprocess debugging requires inspecting structured error logs rather than standard console outputs.
