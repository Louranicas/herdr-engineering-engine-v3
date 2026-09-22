//! Herdr Engineering Engine v3: typed operational control.
//!
//! Modules become public here as their implementation boundaries are introduced.
//! A callable API is not evidence of module or release admission.
//!
//! ```
//! use habitat_engine::contracts::{ScalarError, parse_u64_decimal};
//! assert_eq!(parse_u64_decimal("18446744073709551615"), Ok(u64::MAX));
//! assert_eq!(parse_u64_decimal("01"), Err(ScalarError::LeadingZero));
//! ```

#![forbid(unsafe_code)]

pub mod contracts;
pub mod task;

pub mod worker;

pub mod store;

pub mod roster;

pub mod check;

pub mod app;

pub mod numerical;
pub mod recovery;
pub mod service;
