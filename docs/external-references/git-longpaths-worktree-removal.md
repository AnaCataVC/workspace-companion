# Windows Git Long Paths & Resilient Worktree Removal Architecture

## 1. Overview & Problem Statement

When managing Git worktrees in modern software ecosystems (notably frontend repositories with deeply nested package managers like pnpm, npm, and bun), full directory paths inside `node_modules` or build artifacts frequently exceed the legacy Windows `MAX_PATH` limit (260 characters), often reaching 280 to 300+ characters.

On Windows systems, executing `git worktree remove` without explicit long path configurations results in immediate fatal errors:
```text
error: failed to delete '<path>': Filename too long
```

### Consequences of Unhandled Failure:
1. **Partial / Inconsistent Deregistration**: Git deletes the `.git` pointer file in the target worktree and unlinks/prunes its metadata entry from `.git/worktrees/`, but halts file deletion midway through upon encountering the first path exceeding 260 characters.
2. **Orphaned Zombie Folders**: The physical worktree directory remains on disk in an unlinked state.
3. **Subsequent Deletion Failures**: Standard filesystem deletion utilities (such as Rust's default `std::fs::remove_dir_all`) fail when confronted with paths longer than 260 characters (unless verbatim prefixes are used) or files marked with the `FILE_ATTRIBUTE_READONLY` attribute (common in package caches and Git object packs), yielding `os error 5 (Access is denied)`.

---

## 2. Root Cause Analysis

### Git for Windows `core.longpaths` Default
Even when the Windows operating system has long path support enabled globally via registry:
```text
HKLM\SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled = 1
```
Git for Windows defaults `core.longpaths` to `false` when unset. Git's MSYS2/MinGW POSIX translation layer enforces `MAX_PATH` (260 characters) during recursive traversal and unlinking (`unlink()` / `rmdir()`), returning `ENAMETOOLONG` ("Filename too long").

### Rust Standard Library Filesystem Traversal
Rust's `std::fs::remove_dir_all` on Windows relies on standard Win32 file APIs unless the path explicitly uses the Windows verbatim prefix (`\\?\`). Furthermore, Win32 `DeleteFileW` rejects attempts to delete files with `readonly` flags set, which commonly exist in package managers and build tools.

---

## 3. Two-Tier Resilient Architecture

To guarantee 100% reliability during worktree creation, checkout, and deletion across high-density repositories, Workspace Companion implements a two-tier resilience architecture:

```
┌────────────────────────────────────────────────────────┐
│                   Worktree Deletion                    │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
          Tier 1: Subprocess Invariant
   GitService::build_command injects:
       `git -c core.longpaths=true ...`
                           │
              ┌────────────┴────────────┐
              │                         │
           Success                    Error
              │                         │
              ▼                         ▼
        Clean Complete       Tier 2: Resilient Fallback
                             - Check if Git pruned worktree
                             - Fallback if target unregistered:
                               WorktreeCleanerService::force_remove_directory()
                                 ├── Verbatim `\\?\` prefix (up to 32,767 chars)
                                 ├── Pre-removal readonly attribute stripping
                                 └── Safe symlink/junction unlinking
```

### Tier 1: Git Subprocess Long Path Enablement (`GitService::build_command`)
Every Git command constructed by `GitService::build_command` on Windows automatically prepends global configuration argument `-c core.longpaths=true`:
```rust
#[cfg(target_os = "windows")]
{
    cmd.arg("-c");
    cmd.arg("core.longpaths=true");
}
```
This ensures Git's internal POSIX runtime operates without the 260-character ceiling across all operations (`worktree remove`, `checkout`, `clean`, `reset`, `status`).

### Tier 2: Resilient Filesystem Removal (`force_remove_directory`)
For unregistered leftover directories, or when a forced removal requires disk cleanup following a partial Git abort:
1. **Verbatim Path Normalization (`to_verbatim_path`)**: Converts relative or drive paths to Windows verbatim syntax (`\\?\C:\path\...`), enabling filesystem APIs to address paths up to 32,767 characters.
2. **Permission Relaxation**: Traverses entries recursively, unsetting `readonly` attributes (`set_readonly(false)`) prior to deletion.
3. **Symlink Safety**: Detects symbolic links and junction points via `symlink_metadata` to remove the link itself without recursing into target structures.

---

## 4. User Terminal Recommendation

While Workspace Companion handles long paths autonomously, developers working directly in PowerShell or Command Prompt are encouraged to enable long paths globally in Git:

```powershell
git config --global core.longpaths true
```
