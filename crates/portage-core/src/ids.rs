//! Typed identifiers. The canonical content id is BLAKE3-256 (`b3:<64 hex>`).
//!
//! Provider hashes (Google MD5, OneDrive SHA1/QuickXor/SHA256) are bindings,
//! not identity (design K4).

use std::fmt;
use std::str::FromStr;

use crate::error::Error;

/// BLAKE3-256 content identity, displayed and parsed as `b3:<64 hex>`.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ContentId([u8; 32]);

impl ContentId {
    /// Wrap a raw BLAKE3-256 digest.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw digest.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b3:{}", hex::encode(self.0))
    }
}

impl fmt::Debug for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentId({self})")
    }
}

impl FromStr for ContentId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || Error::InvalidId {
            what: "ContentId",
            input: s.chars().take(80).collect(),
        };
        let hex_part = s.strip_prefix("b3:").ok_or_else(invalid)?;
        if hex_part.len() != 64 {
            return Err(invalid());
        }
        let bytes = hex::decode(hex_part).map_err(|_| invalid())?;
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes);
        Ok(Self(digest))
    }
}

macro_rules! string_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
        pub struct $name(String);

        impl $name {
            /// Wrap a raw id string.
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// The id as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(
    /// A configured provider (`local-d`, `gdrive`, `onedrive`, …).
    ProviderId
);
string_id!(
    /// A stored plan (`file-plan-7f3c`). The user types this to apply.
    PlanId
);

/// A `blobs` row (catalog-assigned, PR 3).
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct BlobId(pub i64);

/// A `plan_ops` row (catalog-assigned, PR 3).
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct OpId(pub i64);

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blob-{}", self.0)
    }
}

impl fmt::Display for OpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "op-{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Official BLAKE3 test vector: hash of the empty input.
    const EMPTY_B3: &str = "b3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

    #[test]
    fn display_parse_round_trip() {
        let id = ContentId::from_bytes(*blake3::hash(b"").as_bytes());
        assert_eq!(id.to_string(), EMPTY_B3);
        let parsed: ContentId = EMPTY_B3.parse().unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn parse_rejects_bad_input() {
        for bad in [
            "",
            "b3:",
            "af1349b9",                             // no prefix
            "md5:d41d8cd98f00b204e9800998ecf8427e", // wrong algo
            "b3:zz1349b9f5f9a1a6a0404dee36dcc9499bcb25c9adc112b7cc9a93cae41f3262", // not hex
            "b3:af1349b9f5f9a1a6a0404dee36dcc9499bcb25c9adc112b7cc9a93cae41f32", // short
        ] {
            assert!(bad.parse::<ContentId>().is_err(), "accepted: {bad}");
        }
    }

    #[test]
    fn string_ids_display_verbatim() {
        assert_eq!(PlanId::new("file-plan-7f3c").to_string(), "file-plan-7f3c");
        assert_eq!(ProviderId::new("local-d").as_str(), "local-d");
        assert_eq!(BlobId(7).to_string(), "blob-7");
        assert_eq!(OpId(3).to_string(), "op-3");
    }
}
