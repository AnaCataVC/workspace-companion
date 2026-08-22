> **Created:** 2026-08-20
> **Last Updated:** 2026-08-20

# Git Worktree Patterns, Layout Strategies & AI Agent Workspace Isolation

A comprehensive reference on Git Worktree architecture, directory layout strategies (Sibling, Nested, Bare Repository), and execution patterns utilized by AI coding agents (such as **Antigravity** and **Claude Code**) for sandboxed and parallel development.

---

## 1. Directory Layout Strategies for Git Worktrees

### A. Sibling Directories Pattern (Standard / Developer-Friendly)
Places worktrees at the same directory hierarchy as the primary clone.

```text
Repos/
├── project-core/                 # Main working tree (main branch)
├── project-core-feat-auth/       # Sibling worktree (feat/auth branch)
└── project-core-fix-sync/        # Sibling worktree (fix/sync branch)
```

- **Creation Command**:
  ```powershell
  git worktree add ../project-core-feat-auth -b feat/auth
  ```
- **Trade-offs**:
  - **Pros**: Zero impact on `.gitignore`, clean isolated IDE workspaces, trivial to delete without affecting parent directory.
  - **Cons**: Flat listing in the parent `Repos/` directory.

---

### B. Nested Subdirectory Pattern (`.worktrees/`)
Encapsulates all active worktrees inside a dedicated subfolder within the primary repository.

```text
Repos/
└── project-core/
    ├── .git/
    ├── .worktrees/               # Must be ignored in .gitignore
    │   ├── feat-auth/
    │   └── fix-sync/
    ├── src/
    └── package.json
```

- **Configuration**:
  ```gitignore
  # .gitignore
  .worktrees/
  ```
- **Creation Command**:
  ```powershell
  git worktree add .worktrees/feat-auth -b feat/auth
  ```
- **Trade-offs**:
  - **Pros**: Keeps the `Repos/` folder clean with a single root entry per project.
  - **Cons**: Language servers and linters (ESLint, TS Server) may traverse nested folders if not explicitly excluded in `.vscode/settings.json` or tool configs.

---

### C. Bare Repository Layout (Canonical / Pro Git Standard)
The standard architecture recommended for power users and heavy worktree workflows. The central Git database is initialized as a bare repository, and every working branch is treated symmetrically as a worktree.

```text
Repos/
└── project-core/
    ├── .bare/ (or .git)          # Bare repository containing Git metadata
    ├── .git                      # Gitdir file: "gitdir: ./.bare"
    ├── main/                     # Production worktree (main branch)
    ├── develop/                  # Development worktree
    └── feat-auth/                # Feature worktree
```

- **Setup Commands**:
  ```powershell
  # 1. Clone as bare
  mkdir project-core; cd project-core
  git clone --bare git@github.com:user/project-core.git .bare

  # 2. Wire top-level gitdir pointer
  "gitdir: ./.bare" | Out-File -Encoding ascii .git

  # 3. Configure remote refspecs
  cd .bare
  git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"
  git fetch origin
  cd ..

  # 4. Spawn worktrees symmetrically
  git worktree add main main
  git worktree add feat-auth -b feat/auth
  ```
- **Trade-offs**:
  - **Pros**: Perfectly symmetrical; eliminates the "privileged root worktree" dilemma where the initial clone cannot checkout branches used elsewhere.
  - **Cons**: Requires initial setup planning.

---

## 2. AI Coding Agent Execution & Isolation Patterns

Autonomous coding assistants execute concurrent tasks, background test suites, and subagent delegations by isolating the filesystem and Git states to avoid corrupting the user's active editor.

```text
                           ┌── [inherit] ──► Direct Shared Workspace (Live Active Editor)
AI Agent (Subagent Spawn) ──┼── [share]   ──► Native Git Worktree (Shared .git object db)
                           └── [branch]  ──► Sandbox Temp Environment (Isolated Clone/FS)
```

### A. Google Antigravity (AGY) Worktree & Workspace Architecture
Antigravity manages workspaces dynamically via the subagent orchestrator:
- **`Workspace: 'inherit'`**: Executes in the caller's active workspace directory.
- **`Workspace: 'share'`**: Uses native Git Worktree semantics (`hg share` / `git worktree`) to give the subagent an independent working tree without duplicating the object repository or bloating disk storage.
- **`Workspace: 'branch'`**: Creates an isolated sandbox inside the agent's runtime data directory (`~/.gemini/antigravity/brain/<conversation-id>/...`).
- **Lifecycle & Teardown**: Subagents report status and diffs back to the parent orchestrator. On subagent termination (`manage_subagents`), branched worktrees are pruned automatically.

### B. Claude Code Worktree Engine
Claude Code decouples execution across two layers:
1. **Worktree Engine (`claude --worktree <name>`)**:
   - Spawns parallel developer sessions on independent branches without file lock contention or hot-reloading disruption in the user's IDE.
   - Runs `git worktree add` targeting `.claude/worktrees/<task-id>` or sibling paths.
2. **Subagent Delegation**:
   - Subagents isolate conversation context and prompt logic, allowing parallel execution of linting, test suites, and refactors within a specific worktree environment.
   - Lifecycle concludes with automated `git worktree remove` and `git worktree prune`.

---

## 3. Orphan Worktree Mechanics & Safe Pruning

When AI agents or developer sessions terminate unexpectedly (e.g., process kills, network timeouts, power loss), worktree administrative metadata in `.git/worktrees/` can become desynchronized from the filesystem.

### Detection Criteria
1. **Missing Directory**: Worktree path registered in `.git/worktrees/` no longer exists on disk (`git worktree prune` candidate).
2. **Merged Upstream Branch**: Branch reference merged into the default branch (`main` / `master`) with no active uncommitted changes.
3. **Upstream Branch `[gone]`**: Remote branch deleted on GitHub/GitLab after PR merge (`git branch -vv` reports `: gone]`).

### Safe Removal Protocol (Porcelain Workflow)
```powershell
# 1. Pre-flight dirty check (MUST be executed first)
git -C <worktree-path> status --porcelain

# 2. Remove worktree safely
git worktree remove <worktree-path>

# 3. Clean stale administrative records
git worktree prune
```

---

## 4. References & Specifications
- [Official Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Bare Repo Worktree Best Practices (Meziantou)](https://www.meziantou.net/git-worktrees.htm)
- [Tauri v2 Native Desktop Guidelines](https://v2.tauri.app)
