# Benchmark: Subprocess Reduction & Context Caching (ADR 0005)

This benchmark provides reproducible evidence for the performance optimizations documented in [ADR 0005: Delta-Patch State Updates and Repo-Wide Context Caching Over Full Rescans](../adr/0005-delta-patch-state-and-repo-context-caching.md).

---

## 1. Problem Statement & Hypothesis

Prior to ADR 0005, each worktree entry in a repository independently queried repo-wide Git facts:
1. `git symbolic-ref` / default branch resolution (`1-2` calls).
2. `git branch -vv` to detect upstream deletion (`[gone]`) (`1` call).
3. `git branch --merged <default_branch>` to detect merged branches (`1` call).

In a repository with $N$ worktrees, this naive approach spawned $3N$ to $5N$ subprocesses, resulting in linear $O(N)$ execution latency and process contention on Windows.

### Architectural Optimization
By computing the repository-wide facts exactly once per scan via `GitService::build_repo_orphan_context` (`3` Git calls total per repository) and sharing this immutable `RepoOrphanContext` across all $N$ worktree evaluations in-memory, the number of subprocesses drops from $O(N)$ to $O(1)$ relative to the worktree count.

---

## 2. Reproducible Empirical Results

The benchmark is executed against a real, disposable temporary Git repository created via `tempfile` and tested across varying worktree/branch densities.

### Test Execution Command
```powershell
cd src-tauri
cargo test --test context_caching_benchmark -- --nocapture
```

### Benchmark Measurement Matrix

| Worktrees ($N$) | Naive Subprocesses | Cached Subprocesses | Subprocess Reduction | Latency: Naive | Latency: Cached | Effective Speedup |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **1** | 3 | 3 | **1.0x** | ~888 ms | ~904 ms | **1.0x** |
| **3** | 9 | 3 | **3.0x** | ~2,711 ms | ~880 ms | **3.1x** |
| **5** | 15 | 3 | **5.0x** | ~4,503 ms | ~889 ms | **5.1x** |
| **10** | 30 | 3 | **10.0x** | ~9,038 ms | ~887 ms | **10.2x** |

> **Key Takeaway:** For a standard repository with 5 worktrees, subprocess spawning drops by **5.0x** (15 processes down to 3), and scan latency is reduced from **4.5 seconds to 889 milliseconds** (a **5.1x** speedup), keeping the desktop UI responsive and eliminating process thrashing.

---

## 3. Output Parity & Invariant Verification

The benchmark asserts that for every $N \in \{1, 3, 5, 10\}$:
$$\text{Output}(\text{Naive}) \equiv \text{Output}(\text{Cached})$$

Both engines produce identical boolean status tuples `(is_remote_gone, is_merged)` for every evaluated branch, guaranteeing zero behavioral drift.
