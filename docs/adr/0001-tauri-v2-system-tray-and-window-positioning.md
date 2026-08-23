# ADR 0001: Tauri v2 System Tray and Window Positioning

## Status
Accepted

## Context
Developers need instant access to their Git worktrees and GitHub profiles without keeping a heavy GUI open in the taskbar. Traditional background applications either consume significant memory or cause intrusive window popups.

## Decision
We chose **Tauri v2** with `tauri::tray::TrayIconBuilder` and `tauri-plugin-positioner` for the following reasons:
1. **Ultra-lightweight Footprint**: Memory consumption under 40 MB RAM in background tray mode.
2. **Spotlight Behavior**: The floating dashboard positions dynamically adjacent to the tray icon and automatically hides on blur (`tauri::WindowEvent::Focused(false)`).
3. **Native Desktop Experience**: Instant responsiveness using the OS native WebView2 engine without the overhead of Electron.

## Consequences
- **Positive**: Near-instant startup, seamless background execution, low resource footprint.
- **Negative**: Requires handling platform-specific window events and position coordinates on Windows.
