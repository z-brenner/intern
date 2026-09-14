//! Durable queue state and safe file operations.
//!
//! This crate owns everything that must survive a crash: the queue database,
//! the leases, and the journalled rename/move machinery. It knows nothing about
//! documents, models, or filenames - that lives in `intern-engine`.

#![deny(unsafe_code)]

mod atomic_file;
mod domain;
mod error;
mod file_ops;
mod store;

pub use atomic_file::{DurableReplaceError, replace_file_durable};
pub use domain::*;
pub use error::*;
pub use file_ops::*;
pub use store::*;

pub const APPLICATION_NAME: &str = "Intern";
