> **Created:** 2026-11-07
> **Last Updated:** 2026-11-07
> **Status:** Active
> **Scope:** Architecture / Git Subsystem Migration (Native Crate vs Git CLI)

# Adversarial Stress-Test: Migrating Git Read Path to Native Rust Crates (`gix` / `git2`)

## Executive Summary (The Premortem)
**Hypothesis:** Replacing `git.exe` subprocess invocations (`CreateProcessW`) with in-memory Git libraries (`gix` or `git2`) + `rayon` will achieve instantaneous, zero-overhead worktree discovery and dirty checking.
**Premortem Reality (6 months post-deployment):** Users report phantom dirty files, UI freezes when IDEs write commits, crashes due to C-ABI segfaults in `libgit2`, and broken CI/CD builds on Windows machines without CMake. The apparent 10x-50x speedup in isolated benchmarks triggers insidious real-world edge cases that Git CLI solved over 15 years of battle-testing.

---

## The 5-Dimension Attack Matrix

### 1. Premortem & Operational Failure Modes

| Vulnerability | Severity | Root Cause & Failure Mode |
| :--- | :---: | :--- |
| **Phantom Dirty Status (`core.autocrlf` & Stat Cache Drift)** | **[Critical / Blocker]** | On Windows, Git CLI writes cached `stat()` metadata (`mtime`, `ctime`, file size) into `.git/index`. If files use CRLF/LF conversions or nano-second timestamp rounding on NTFS, `libgit2`/`gix` dirty checks that inspect working files without running the full clean/smudge filter pipeline will flag clean files as modified (`is_dirty = true`). The UI will falsely report uncommitted changes when `git status` in terminal says clean. |
| **Git LFS & Custom Smudge/Clean Filters** | **[Critical / Blocker]** | Repositories using Git LFS store 130-byte pointer files in Git objects. In the working tree, the real multi-megabyte binary is present. `git.exe` uses the LFS filter to compare them seamlessly. `libgit2` and `gix` do NOT execute external filter drivers (`filter.lfs.smudge`) out of the box; they compare the pointer hash to the checked-out asset and report every LFS asset as modified/dirty. |
| **Windows Long Paths (> 260 chars) Boundary** | **[Major / Hardening]** | Workspace Companion enforces `-c core.longpaths=true` on all Git CLI calls. `libgit2` historically truncates paths exceeding `MAX_PATH` unless explicitly initialized with `git_libgit2_opts(GIT_OPT_ENABLE_LONGPATHS)` AND running on Windows 10+ with system-wide long paths. Unhandled deep trees (`node_modules`) trigger silent lookup errors or panic. |
| **Linked Worktree Pointer Divergence** | **[Major / Hardening]** | Git 2.5+ linked worktrees contain a `.git` file with `gitdir: <path>`. If the main repository moves, or if worktree metadata has prune/lock flags (`.git/worktrees/<name>/locked`), native libraries must manually parse and resolve `commondir`. A malformed or disconnected worktree can cause library panics rather than returning a clean porcelain error. |

---

### 2. Concurrency, Race Conditions & State Drift

| Vulnerability | Severity | Root Cause & Failure Mode |
| :--- | :---: | :--- |
| **Torn Index Reads during Background IDE/CLI Writes** | **[Critical / Blocker]** | When an external tool (VS Code, Cursor, terminal `git commit`) writes `.git/index`, it writes `.git/index.lock` and atomically renames it over `.git/index`. In-memory readers opening `.git/index` at that millisecond can read a partially flushed file or encounter `ERROR_SHARING_VIOLATION` on Windows NTFS. Git CLI handles lock-wait retries internally; raw crate readers throw deserialization errors. |
| **Thread-Safety & Rayon Par-Iter Incompatibilities** | **[Major / Hardening]** | In `git2`, `git2::Repository` is `!Sync` (raw C pointer wrapper). You cannot share a single repository handle across Rayon threads. Each thread must independently open, read, and close repository handles. If 20 Rayon threads open the same repository simultaneously to check worktrees, file handles to `.git/objects` and `.git/refs` contend on the Windows filesystem filter driver (antivirus / Defender). |

---

### 3. Build & Runtime Resource Explosion

| Vulnerability | Severity | Root Cause & Failure Mode |
| :--- | :---: | :--- |
| **C-Toolchain Dependency in CI/CD (`libgit2-sys`)** | **[Major / Hardening]** | `git2` depends on `libgit2`, requiring a native C compiler, CMake, and Python for vendored builds. This increases CI build matrix fragility and binary size (~3.5 MB increase in NSIS installer). |
| **Cargo Dependency Tree & Compile-Time Bloat (`gix`)** | **[Major / Hardening]** | `gix` is 100% pure Rust but pulls ~90 transitive crates into `Cargo.lock`. Clean compile times on developer machines increase by 45–90 seconds, and memory consumption during `cargo check` / `rust-analyzer` expands significantly. |

---

### 4. Security & Process Crash Vectors

| Vulnerability | Severity | Root Cause & Failure Mode |
| :--- | :---: | :--- |
| **Process Crash vs Subprocess Isolation** | **[Critical / Blocker]** | In the current architecture, if `git.exe` encounters a corrupt object or out-of-memory error, the subprocess exits with an error code and the Rust app logs `repo-scan-failed`. If `libgit2` (C code) encounters a memory corruption or segfaults, it takes down the **entire Tauri desktop host process** without triggering Rust's `panic::catch_unwind`. |

---

### 5. Developer Friction & Maintenance Debt

| Vulnerability | Severity | Root Cause & Failure Mode |
| :--- | :---: | :--- |
| **Behavioral Divergence ("Bug in Companion vs Bug in Git")** | **[Major / Hardening]** | Users trust Git CLI as the single source of truth. When native crate heuristics disagree with terminal `git` (e.g. untracked nested submodules, `.gitattributes` line ending conversions, sparse checkouts), users report issues to Companion that cannot be fixed without patching upstream crate libraries. |

---

## Required Hardening Countermeasures & Invariants

If migration is pursued, the following architectural invariants are mandatory:

1. **Subprocess Fallback Circuit-Breaker:**
   Never eliminate `git.exe` subprocess execution entirely. Treat native Rust (`gix`/`git2`) as an optimistic **Fast-Path Cache**. If the native check encounters errors, lock contention, or unusual attributes, cleanly fall back to `GitService::run_git`.
2. **Read-Only Invariant with Atomic Retry:**
   Native operations must strictly be read-only. Wrap index/reference reading in retry loops (3 attempts with exponential backoff of 5ms–20ms) to accommodate transient `.git/index.lock` renames by IDEs.
3. **Stat-Only Dirty Fast-Check with Git CLI Fallback:**
   Only use native Rust to inspect commit OIDs, branch head pointers, and basic uncommitted status counts where no custom filters (LFS) exist. If `repo.has_filter("lfs")` or special `.gitattributes` are detected, delegate dirty status directly to `git status --porcelain`.
