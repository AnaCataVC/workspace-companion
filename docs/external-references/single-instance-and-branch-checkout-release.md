# Technical Reference: Single Instance Enforcement & Checked-Out Branch Release

## 1. Single Instance Architecture in Tauri v2 (Windows)

### Problem Statement
In desktop utilities operating as background System Tray residents (such as Workspace Companion), closing the UI window only hides it (`window.hide()`) rather than terminating the process. Without a strict single-instance lock:
1. Launching the executable again (via Start Menu, shortcut, `tauri dev`, or secondary autorun trigger) spawns a duplicate OS process (`workspace-companion.exe`).
2. The secondary process initializes a duplicate tray icon with `TrayIconBuilder::new().build(app)`, resulting in multiple identical icons in the Windows Notification Area.
3. The secondary process fails to register the global shortcut `Alt+Space` because the OS hotkey table already assigned the key combination to the primary process PID.
4. Concurrent background processes operate on the same local `app_config.json`, causing cache desynchronization, file lock race conditions, and phantom tray icons when one process exits without clearing its tray handle.

### Tauri v2 Single Instance Mechanism
Tauri v2 provides the official plugin `tauri-plugin-single-instance` (version `2.x`), which uses an OS-level named pipe / mutex on Windows:
- **First Process:** Owns the named pipe and registers the handler callback.
- **Second Process:** Attempts to connect to the named pipe. Upon detection, it forwards its command-line arguments and working directory to the existing process, and then immediately exits cleanly without creating windows, tray icons, or hotkeys.
- **Handler Execution:** On the primary process, the plugin triggers `tauri_plugin_single_instance::init(|app, _args, _cwd| { ... })`. In Workspace Companion, this must invoke `tray::show_main_window(app)`, unminimizing, repositioning near the tray, revealing, and focusing the existing panel.
- **Initialization Order Invariant:** `tauri_plugin_single_instance::init` MUST be registered as the very first plugin in `tauri::Builder::default()`, before `tauri_plugin_positioner` or any tray initialization.

---

## 2. Git Checked-Out Branch Release & Deletion Ergonomics

### Problem Statement
Git strictly prohibits deleting a branch that is currently checked out by any worktree:
```text
error: Cannot delete branch '<branch-name>' checked out at '<worktree-path>'
```
In Workspace Companion:
- `BranchCleanerService::remove_branches_batch` enforces this server-side with zero bypass (ADR 0006).
- In the frontend, checked-out branches cannot be selected for batch deletion and display a lock / unlock icon.
- Previously, the only way a user could delete such a branch was:
  1. Manually navigate to the **Worktrees** tab.
  2. Locate the worktree that checked out that branch.
  3. Open the Branch Switcher modal.
  4. Select `main` (or default branch).
  5. Wait for the checkout to complete.
  6. Manually navigate back to the **Branches** tab.
  7. Locate the branch again (now unlocked).
  8. Select and delete the branch.
- The existing `ReleaseBranchModal` only offered:
  - `Detach & delete branch`: Leaves the worktree in a `detached HEAD` state, which confuses developers and causes terminal warnings.
  - `Delete worktree & branch`: Deletes the entire worktree folder from disk; furthermore, if the branch is checked out in the main repository root (`isMain: true`), this operation is invalid and fails with a Git error.

### Architectural Solution
Provide an atomic **"Switch to default branch & delete branch"** operation directly within the branch release workflow:
1. **Repository Context Discovery:** Identify the target worktree and determine the repository's default branch (`main`, `master`, or derived default branch).
2. **Worktree Type Awareness:**
   - **Main Worktree (`isMain: true`):** Hide "Delete worktree" (prohibited) and make "Switch to `<default_branch>` & delete" the primary recommended action.
   - **Secondary Worktree (`isMain: false`):** Offer both:
     - Option A: "Switch worktree to `<default_branch>` & delete branch" (retains the worktree directory on `main`).
     - Option B: "Delete worktree & branch" (removes the entire worktree folder and deletes the branch).
     - Option C: "Detach & delete branch" (advanced fallback).
3. **Atomic Execution Flow:**
   - Step 1: Call `checkout_worktree_branch` (`git checkout <default_branch>`) on the worktree.
   - Step 2: Once checked out, call `remove_branches_batch` with target `[{ repoPath, branchName, force }]`.
   - Step 3: Emit `released` event, updating both `scannedBranches` and `scannedRepos` in the frontend store without requiring a view switch.
4. **UI Affordances:**
   - Replace the ambiguous "Go to worktree" primary button on checked-out branches with a clearer action button in both Compact and Detailed cards.
