# Formal Contract: Worktree Discard & Popup Stability

**Feature Slug:** `worktree-discard-stability`  
**Target Systems:** Rust Backend (`src-tauri/src/services/git.rs`, `src-tauri/src/services/worktree_cleaner.rs`), Svelte 5 Frontend (`src/lib/actions/closeOnEscape.ts`, `src/lib/components/DiscardChangesPanel.svelte`, `src/lib/components/BranchSwitcherModal.svelte`)

---

## 1. Public Interfaces & Signatures

### Backend (Rust)

#### 1.1 `GitService::build_command`
```rust
pub fn build_command<P: AsRef<Path>>(working_dir: P, args: &[&str]) -> Command
```
- **Inputs:** `working_dir` (path to repository/worktree directory), `args` (slice of CLI arguments for `git`).
- **Guarantees:**
  - Standard input MUST be isolated with `Stdio::null()` so subprocesses never inherit GUI stdin handles or hang waiting for terminal input.
  - Non-interactive environment variables MUST be explicitly enforced:
    - `GIT_TERMINAL_PROMPT = 0`
    - `GIT_OPTIONAL_LOCKS = 0`
  - On Windows, `CREATE_NO_WINDOW = 0x08000000` creation flag MUST be applied.

#### 1.2 `GitService::discard_worktree_changes`
```rust
pub fn discard_worktree_changes<P: AsRef<Path>>(worktree_path: P) -> Result<String, String>
```
- **Inputs:** `worktree_path` (path to worktree).
- **Behavior:**
  - Executes `git reset --hard HEAD` to discard tracked modifications.
  - Executes `git clean -ffd` (double `-f` to force removal of untracked directories including nested repositories/submodules).
  - Returns `Ok(String)` summarizing reset and clean output.
  - If untracked files are locked by Windows (e.g., in use by editor/terminal with `warning: failed to remove` or `Permission denied`), it MUST NOT fail the entire reset operation; it reports tracked changes reset with a clear notification of locked files.

#### 1.3 `WorktreeCleanerService::build_single_worktree_info`
```rust
pub fn build_single_worktree_info(repo_path: &str, worktree_path: &str) -> Result<WorktreeEntry, String>
```
- **Inputs:** `repo_path` (repository root or worktree path), `worktree_path` (target worktree path to query).
- **Behavior:**
  - Matches the target worktree in `git worktree list --porcelain` using normalized, case-insensitive comparison (`comparable_path`).
  - MUST NOT fail due to Windows drive letter casing differences (`C:\` vs `c:\`), UNC verbatim prefixes (`\\?\`), or slash direction differences (`\` vs `/`).

---

### Frontend (TypeScript / Svelte)

#### 1.4 `closeOnEscape` Action (`src/lib/actions/closeOnEscape.ts`)
```typescript
export interface CloseOnEscapeOptions {
  enabled: () => boolean;
  onClose: () => void;
}
export const closeOnEscape: Action<Window, CloseOnEscapeOptions>;
```
- **Behavior:**
  - When `Escape` key is pressed while `enabled()` is `true`:
    - Calls `event.preventDefault()`
    - Calls `event.stopPropagation()`
    - Calls `onClose()`
  - Prevents the `Escape` key from bubbling up to the top-level window handler (`App.svelte`), ensuring the main desktop panel is NOT hidden to the system tray when closing a popup or modal.

#### 1.5 `DiscardChangesPanel` Component (`src/lib/components/DiscardChangesPanel.svelte`)
- **Props:**
  - `worktreePath: string`
  - `uncommittedFilesCount: number`
  - `disabled?: boolean`
  - `ondiscarded?: (updatedWorktree: WorktreeInfo) => void`
  - `oncancel?: () => void`
  - `onbusychange?: (busy: boolean) => void`
- **Behavior:**
  - When discarded: calls `invoke('git_discard_worktree_changes')`.
  - When component is destroyed (`onDestroy`): MUST emit `onbusychange(false)` so parent components and modals are never permanently locked in a busy state.
  - Exposes a dedicated cancel/close button (`X`) in addition to "Keep changes".
  - If discard fails, displays the error message and allows dismissing or retrying without freezing the interface.

#### 1.6 `BranchSwitcherModal` Component (`src/lib/components/BranchSwitcherModal.svelte`)
- **Behavior:**
  - Modal close button (`X`) and `Escape` key handler MUST be accessible whenever the modal is open, unless a Git branch checkout (`isSwitching`) is actively in progress.
  - Dirty state resolutions (stashing or discarding) MUST NOT permanently disable modal closure.
  - Cancelling the discard prompt explicitly resets `isConfirmingDiscard = false` and `isDiscarding = false`.

---

## 2. Behavioral Acceptance Criteria (Given-When-Then)

### Scenario 1: Clean discard on dirty worktree
- **Given** a worktree with modified tracked files and new untracked files.
- **When** `discard_worktree_changes` is invoked.
- **Then** `git reset --hard HEAD` and `git clean -ffd` execute without hanging, returning `Ok` and leaving a clean working tree.

### Scenario 2: Locked file tolerance on Windows
- **Given** an untracked file held open by a process with exclusive read lock.
- **When** `discard_worktree_changes` is executed.
- **Then** tracked changes are successfully reset and the operation does not crash or hang indefinitely on stdin.

### Scenario 3: Worktree matching with casing discrepancies
- **Given** a target worktree queried with lowercase drive letter (`c:/path`) while git reports uppercase (`C:/path`).
- **When** `build_single_worktree_info` is called.
- **Then** the entry is successfully located and returned rather than throwing `Worktree not found at path`.

### Scenario 4: Escape key dismissal
- **Given** the discard changes popup or modal is open.
- **When** the user presses `Escape`.
- **Then** `closeOnEscape` intercepts the event, calls `preventDefault()` and `stopPropagation()`, closes the popup, and the main application window remains visible.

### Scenario 5: Cleanup on unmount
- **Given** `DiscardChangesPanel` is rendered inside `BranchSwitcherModal`.
- **When** the user dismisses the discard panel or closes the modal.
- **Then** `onDestroy` triggers `onbusychange(false)` and `isDiscarding` does not linger in parent state.

---

## 3. Boundary Values & State Invariants

1. **Subprocess Isolation:** `Command::new("git")` must have stdin detached (`Stdio::null()`).
2. **Terminal Invariant:** `GIT_TERMINAL_PROMPT` must be `"0"`.
3. **Event Invariant:** `closeOnEscape` must mark `event.defaultPrevented == true`.
4. **Idempotence:** Dismissing a modal or popup multiple times must not underflow action counts or throw errors.

---

## 4. Prohibited Details

- Tests and implementations must NOT rely on private internal functions outside the specified module interfaces.
- Tests must NOT attempt to test specific UI theme styling (colors, margins) — only accessibility, event handling, and operational state.
