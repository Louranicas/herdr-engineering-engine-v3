#![forbid(unsafe_code)]
pub use hee3_fixed_task_runtime::{prepare, support, u64_receipt};
pub mod frontend;
mod inputs;
mod manifest;
mod probes;
pub type Result<T> = std::result::Result<T, String>;
pub(crate) fn checked<T, E: std::fmt::Debug>(value: std::result::Result<T, E>) -> Result<T> {
    value.map_err(|error| format!("{error:?}"))
}
pub mod recovery;
pub mod recovery_cancel;
