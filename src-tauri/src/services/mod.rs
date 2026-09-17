use serde::{Deserialize, Serialize};

pub mod branch_cleaner;
pub mod config;
pub mod gh;
pub mod git;
pub mod worktree_cleaner;

/// Tells a target the cleaner intentionally refused apart from one where the git call itself
/// failed. Both end up in the same `errors` list, but only `Failed` means something went wrong:
/// a `Skipped` entry is a safety guard doing its job (ADR 0003, ADR 0006), so the UI can report
/// it without treating the batch as unsuccessful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchItemErrorKind {
    Skipped,
    Failed,
}
