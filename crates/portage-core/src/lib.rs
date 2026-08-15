//! Core types shared by every Portage crate.
//!
//! Errors, byte units, observability, typed identifiers, streaming hashes,
//! and path containment. Config loading arrives with the catalog work.

pub mod error;
pub mod hash;
pub mod ids;
pub mod obs;
pub mod paths;
pub mod units;

pub use error::Error;
pub use hash::{HashAlgo, MultiHasher, QuickHash, TransferDigests};
pub use ids::ContentId;
pub use units::ByteSize;
