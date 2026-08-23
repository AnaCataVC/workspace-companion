# ADR 0003: Git Porcelain Protocol and Pre-flight Dirty Checks

## Status
Accepted

## Context
Automating worktree discovery and cleanup poses risks:
1. Localized Git outputs (e.g. non-English OS) cause regex parsing failures if standard text output is parsed.
2. Destructive operations (`git worktree remove`) can irreversibly discard uncommitted changes or work in progress.

## Decision
1. **Machine-readable Parsing**: Strictly parse porcelain formats (`git worktree list --porcelain`, `git status --porcelain`).
2. **Mandatory Pre-flight Dirty Guardrails**: Before invoking any worktree removal or branch prune, the system executes `git status --porcelain` on the target worktree path. If any uncommitted, modified, or untracked changes exist, the operation is blocked and requires explicit confirmation.

## Consequences
- **Positive**: Resilient against different Git locales and completely prevents accidental code loss.
- **Negative**: Adds a minor pre-flight I/O overhead before deletions, which is negligible compared to the safety benefit.
