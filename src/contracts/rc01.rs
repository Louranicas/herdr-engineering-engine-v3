//! RC01 numeric limits for `rust-library-change/1` (docs/contract-decisions.md, RC01 "Attempts and
//! loop" and "No-progress stop"). The task loop guard owns the policy and re-exports these; they are
//! defined here because every consumer may import `contracts`, while `store` may not import `task`.
use std::time::Duration;

/// Total monotonic elapsed per task, shared by every retry.
pub const TASK_LIMIT: Duration = Duration::from_mins(20);
/// Final share of [`TASK_LIMIT`] kept for verification and cleanup; no new candidate work starts in it.
pub const CLEANUP_RESERVE: Duration = Duration::from_mins(5);
/// Attempts per task.
pub const MAX_ATTEMPTS: u8 = 3;
/// Consecutive attempts without a newly satisfied criterion before the task stops.
pub const MAX_NO_PROGRESS: u8 = 2;
