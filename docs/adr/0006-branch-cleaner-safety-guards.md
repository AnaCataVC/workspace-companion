# ADR 0006: Branch Cleaner Safety Guards

## Status
Accepted

## Context
The Orphaned Worktree Cleaner (ADR 0003, ADR 0005) only ever evaluates branches that currently have
a worktree. Most branches that pile up after a PR merges never had a worktree of their own — they
were created, checked out briefly on the main worktree or a throwaway one, then left behind — so
today's cleaner never sees them and the user is left running `git branch -d` by hand across every
managed repo.

Extending the same merged/upstream-gone detection to *every* local branch, not just worktree-linked
ones, reopens the exact risk ADR 0003 was written for: a batch delete driven by a possibly-stale
frontend selection could target a branch that is no longer safe to delete by the time the delete
actually runs (checked out in the meantime, or the default branch). Unlike worktree deletion, branch
deletion has no `git status --porcelain` equivalent to fall back on — the two facts that make a
branch undeletable are "is it the default branch" and "is it checked out anywhere," not "is it
dirty."

## Decision
1. **Server-side guard, not a client-side one.** `BranchCleanerService::remove_branches_batch`
   re-derives branch status (`GitService::list_local_branches_with_status`) once per repository
   immediately before deleting, instead of trusting the `isDefault`/`isCheckedOut` flags the
   frontend last saw during its scan. A branch merged, checked out, or turned into the default
   branch since that scan is still protected. **Fails closed**: if the re-derivation itself errors
   for a repository (a corrupt ref, a repo that vanished between scan and delete), every target in
   that repository is refused with an error instead of falling back to an empty status list — an
   empty list would make every branch look unprotected, turning a lookup failure into a silent
   bypass of the guard `force` is never allowed to skip.
2. **The default branch and any checked-out branch are never deletable, `force` or not.** This
   mirrors the existing "cannot remove the main working tree" guard in
   `WorktreeCleanerService::remove_worktrees_batch` — `force` only ever controls whether an
   *unmerged* branch can be deleted (`git branch -D` instead of `-d`), never whether a *protected*
   one can.
3. **Git's own merge check is the safety net for everything else.** Rather than reimplementing "is
   this branch merged," `delete_branch` calls `git branch -d` by default, which refuses to delete an
   unmerged branch on its own; `force` switches to `-D`. The merged/remote-gone flags computed by
   `list_local_branches_with_status` are informational for the UI (badges, filters), not a
   duplicate gate the Rust code has to keep in sync with git's.
4. **Reuses `RepoOrphanContext` rather than adding a second detection path.** The merged/gone
   classification is split out of `check_orphan_status` into `branch_status_flags`, which both the
   worktree cleaner's collapsed "orphaned" check and the new per-branch listing call — so the two
   features can never disagree about what counts as merged or gone.

## Consequences
- **Positive**:
  - A batch that spans dozens of branches across several repos can't delete the default branch or a
    branch someone just checked out mid-scan, even under `force: true`.
  - No new "is this branch safe to delete" logic to maintain — git's own `-d`/`-D` distinction does
    that work, exactly as it would from the command line.
- **Negative**:
  - Re-deriving status per repo right before deleting adds a `branch -vv`/`branch --merged`/
    `worktree list` round-trip to every batch delete, on top of the scan that already ran a moment
    earlier. Negligible next to the safety it buys, same trade-off ADR 0003 made for dirty checks.
