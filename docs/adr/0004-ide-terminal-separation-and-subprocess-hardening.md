# ADR 0004: Dual IDE & Terminal Separation and Subprocess Launcher Hardening

## Status
Accepted

## Context
Developers work with a combination of graphical code editors (VS Code, Antigravity IDE, Cursor, Windsurf) and command-line interfaces or terminal multiplexers (Windows Terminal `wt.exe`, PowerShell, CMD, Git Bash, AGY CLI). 

Prior to this architectural update:
1. Editor and terminal launcher actions risked being conflated under a single configuration key.
2. If a user updated their preferences in `WatchFoldersModal`, any stale `localStorage` keys could cause state drift and split-brain UI across different worktree views.
3. Deserializing older `app_config.json` files missing newly introduced configuration fields would trigger serde deserialization errors, potentially overwriting user preferences.
4. Spawning external terminal commands via unescaped shell strings risked path quoting errors when handling worktree folders with spaces or special characters.

## Decision
1. **Explicit Dual Configuration**: Separate `defaultEditor` and `defaultTerminal` into distinct configuration parameters in `AppConfig`, complemented by a `showTerminalButton` visibility toggle.
2. **Backward-Compatible Deserialization**: Apply `#[serde(default)]` annotations across all Rust configuration fields to guarantee seamless migration for existing configuration files without data loss.
3. **Resilient Terminal Cascade**: Implement an automatic fallback hierarchy in `GitService::open_in_terminal` (e.g. `wt` -> `powershell` -> `cmd`) if the user's preferred terminal binary is not found on the host machine.
4. **Direct Subprocess Spawning**: Avoid shell wrapper injection by invoking binaries directly with `std::process::Command`, applying `CREATE_NO_WINDOW = 0x08000000` on background operations while spawning interactive terminals independently.
5. **Unified Reactive Store State**: Eradicate fragmented `localStorage` lookups in favor of the centralized reactive Svelte store (`$appConfig`).

## Consequences
- **Positive**:
  - Independent customization for both GUI editor and terminal/CLI workflows.
  - Zero terminal popups for background tasks, with reliable fallback for terminal launches.
  - Full backward compatibility for existing user configurations.
  - Safe handling of filesystem paths with spaces and special characters.
- **Negative**:
  - Requires maintaining a catalog of supported IDE and terminal binary identifiers and their platform-specific invocation semantics.