//! Core types shared by every Portage crate.
//!
//! PR 1 scope: errors and byte units. Identifiers, hashing, paths, and config
//! arrive in PR 2 per `docs/design.md`.

pub mod error;
pub mod units;

pub use error::Error;
pub use units::ByteSize;
