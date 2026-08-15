//! Streaming hashing: BLAKE3 identity plus provider-native digests in one read.
//!
//! Never `read_to_end` — everything streams through a 1 MiB buffer (design K4).
//! QuickXor (OneDrive) lands with PR 8 alongside its documented test vector;
//! requesting it before then is `Error::UnsupportedHashAlgo`, never a wrong hash.

use std::fmt;
use std::io::{self, Read};
use std::path::Path;

use md5::{Digest as _, Md5};
use sha1::Sha1;
use sha2::Sha256;

use crate::error::Error;
use crate::ids::ContentId;

/// Streaming buffer size: 1 MiB.
pub const HASH_BUF_SIZE: usize = 1024 * 1024;

/// Bytes covered by [`QuickHash`]: the first 64 KiB.
pub const QUICK_PROBE_BYTES: usize = 64 * 1024;

/// Hash algorithms Portage can bind. Only BLAKE3 is identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HashAlgo {
    /// BLAKE3-256 — the canonical `ContentId`.
    Blake3,
    /// Google Drive `md5Checksum`.
    Md5,
    /// OneDrive `hashes.sha1Hash`.
    Sha1,
    /// OneDrive `hashes.sha256Hash`.
    Sha256,
    /// OneDrive `hashes.quickXorHash` (implemented in PR 8).
    QuickXor,
}

impl fmt::Display for HashAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Blake3 => "blake3",
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::QuickXor => "quickxor",
        };
        f.write_str(name)
    }
}

/// Digests produced in a single read of a source or staging file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferDigests {
    /// BLAKE3 identity of the bytes read.
    pub content_id: ContentId,
    /// Dest-provider algos hashed in the same pass, hex lowercase.
    pub native: Vec<(HashAlgo, String)>,
}

/// Cheap duplicate prefilter over the first 64 KiB. **Not identity** — used
/// only to skip expensive full hashes on obviously different files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuickHash {
    /// Full file size in bytes.
    pub size: u64,
    /// BLAKE3 XOF of the first 64 KiB, extended to 64 bytes.
    pub head: [u8; 64],
    /// How many bytes the probe actually covered (< 64 KiB for small files).
    pub head_len: u32,
}

impl QuickHash {
    /// Probe a reader: consumes at most 64 KiB. `size` is the full file size
    /// from metadata (the probe cannot know it).
    pub fn probe(reader: &mut impl Read, size: u64) -> io::Result<Self> {
        let mut buf = vec![0u8; QUICK_PROBE_BYTES];
        let mut filled = 0usize;
        while filled < buf.len() {
            match reader.read(&mut buf[filled..])? {
                0 => break,
                n => filled += n,
            }
        }
        let mut hasher = blake3::Hasher::new();
        hasher.update(&buf[..filled]);
        let mut head = [0u8; 64];
        hasher.finalize_xof().fill(&mut head);
        Ok(Self {
            size,
            head,
            head_len: filled as u32,
        })
    }
}

/// One-pass multi-digest hasher: BLAKE3 plus requested native algos, all fed
/// from the same buffer so a multi-GB clip is read exactly once.
#[derive(Debug)]
pub struct MultiHasher {
    blake3: blake3::Hasher,
    md5: Option<Md5>,
    sha1: Option<Sha1>,
    sha256: Option<Sha256>,
}

impl MultiHasher {
    /// BLAKE3 is always computed; `native` lists the dest provider's algos.
    pub fn new(native: &[HashAlgo]) -> Result<Self, Error> {
        let mut hasher = Self {
            blake3: blake3::Hasher::new(),
            md5: None,
            sha1: None,
            sha256: None,
        };
        for algo in native {
            match algo {
                HashAlgo::Blake3 => {}
                HashAlgo::Md5 => hasher.md5 = Some(Md5::new()),
                HashAlgo::Sha1 => hasher.sha1 = Some(Sha1::new()),
                HashAlgo::Sha256 => hasher.sha256 = Some(Sha256::new()),
                HashAlgo::QuickXor => {
                    return Err(Error::UnsupportedHashAlgo(
                        "quickxor arrives with the OneDrive provider (PR 8)".into(),
                    ))
                }
            }
        }
        Ok(hasher)
    }

    /// Feed a chunk to every active digest.
    pub fn update(&mut self, chunk: &[u8]) {
        self.blake3.update(chunk);
        if let Some(md5) = &mut self.md5 {
            md5.update(chunk);
        }
        if let Some(sha1) = &mut self.sha1 {
            sha1.update(chunk);
        }
        if let Some(sha256) = &mut self.sha256 {
            sha256.update(chunk);
        }
    }

    /// Finish all digests.
    pub fn finalize(self) -> TransferDigests {
        let content_id = ContentId::from_bytes(*self.blake3.finalize().as_bytes());
        let mut native = Vec::new();
        if let Some(md5) = self.md5 {
            native.push((HashAlgo::Md5, hex::encode(md5.finalize())));
        }
        if let Some(sha1) = self.sha1 {
            native.push((HashAlgo::Sha1, hex::encode(sha1.finalize())));
        }
        if let Some(sha256) = self.sha256 {
            native.push((HashAlgo::Sha256, hex::encode(sha256.finalize())));
        }
        TransferDigests { content_id, native }
    }

    /// Stream a reader to completion through the 1 MiB buffer.
    pub fn digest_reader(mut self, reader: &mut impl Read) -> io::Result<TransferDigests> {
        let mut buf = vec![0u8; HASH_BUF_SIZE];
        loop {
            match reader.read(&mut buf)? {
                0 => break,
                n => self.update(&buf[..n]),
            }
        }
        Ok(self.finalize())
    }
}

/// BLAKE3 a file with the streaming buffer. Convenience for the indexer.
pub fn hash_file(path: &Path) -> Result<ContentId, Error> {
    let mut file = std::fs::File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let digests = MultiHasher::new(&[])
        .expect("empty native set is always supported")
        .digest_reader(&mut file)
        .map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(digests.content_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Deterministic pseudo-random bytes without a rand dependency.
    fn test_bytes(len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        let mut state = 0x9E3779B97F4A7C15u64;
        while out.len() < len {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            out.extend_from_slice(&state.to_le_bytes());
        }
        out.truncate(len);
        out
    }

    #[test]
    fn known_vectors_for_every_algo() {
        let mut h = MultiHasher::new(&[HashAlgo::Md5, HashAlgo::Sha1, HashAlgo::Sha256]).unwrap();
        h.update(b"abc");
        let d = h.finalize();
        // Public test vectors (RFC 1321, FIPS 180).
        assert!(d
            .native
            .contains(&(HashAlgo::Md5, "900150983cd24fb0d6963f7d28e17f72".into())));
        assert!(d.native.contains(&(
            HashAlgo::Sha1,
            "a9993e364706816aba3e25717850c26c9cd0d89d".into()
        )));
        assert!(d.native.contains(&(
            HashAlgo::Sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
        )));
        assert_eq!(
            d.content_id,
            ContentId::from_bytes(*blake3::hash(b"abc").as_bytes())
        );
    }

    #[test]
    fn empty_input_blake3_vector() {
        let d = MultiHasher::new(&[]).unwrap().finalize();
        assert_eq!(
            d.content_id.to_string(),
            "b3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn streaming_equals_one_shot_at_boundary_sizes() {
        // Design test strategy: 0, 1, 64 KiB - 1, 64 KiB, 20 MiB.
        for len in [0usize, 1, 64 * 1024 - 1, 64 * 1024, 20 * 1024 * 1024] {
            let data = test_bytes(len);
            let streamed = MultiHasher::new(&[HashAlgo::Md5])
                .unwrap()
                .digest_reader(&mut Cursor::new(&data))
                .unwrap();
            let mut one_shot = MultiHasher::new(&[HashAlgo::Md5]).unwrap();
            one_shot.update(&data);
            assert_eq!(streamed, one_shot.finalize(), "len {len}");
            assert_eq!(
                streamed.content_id,
                ContentId::from_bytes(*blake3::hash(&data).as_bytes()),
                "len {len}"
            );
        }
    }

    #[test]
    fn multihasher_matches_standalone_hashes() {
        let data = test_bytes(3 * 1024 * 1024 + 17);
        let d = MultiHasher::new(&[HashAlgo::Md5, HashAlgo::Sha1])
            .unwrap()
            .digest_reader(&mut Cursor::new(&data))
            .unwrap();
        let standalone_md5 = hex::encode(Md5::digest(&data));
        let standalone_sha1 = hex::encode(Sha1::digest(&data));
        assert!(d.native.contains(&(HashAlgo::Md5, standalone_md5)));
        assert!(d.native.contains(&(HashAlgo::Sha1, standalone_sha1)));
    }

    #[test]
    fn quickxor_is_refused_not_faked() {
        let err = MultiHasher::new(&[HashAlgo::QuickXor]).unwrap_err();
        assert!(matches!(err, Error::UnsupportedHashAlgo(_)));
    }

    #[test]
    fn quickhash_probes_only_the_head() {
        let mut a = test_bytes(200 * 1024);
        let b = a.clone();
        let qa = QuickHash::probe(&mut Cursor::new(&a), a.len() as u64).unwrap();
        let qb = QuickHash::probe(&mut Cursor::new(&b), b.len() as u64).unwrap();
        assert_eq!(qa, qb);
        assert_eq!(qa.head_len as usize, QUICK_PROBE_BYTES);

        // Change a byte beyond 64 KiB: quickhash must NOT notice (prefilter only).
        a[100 * 1024] ^= 0xFF;
        let qa2 = QuickHash::probe(&mut Cursor::new(&a), a.len() as u64).unwrap();
        assert_eq!(qa.head, qa2.head);

        // Change a byte inside the head: quickhash must notice.
        a[10] ^= 0xFF;
        let qa3 = QuickHash::probe(&mut Cursor::new(&a), a.len() as u64).unwrap();
        assert_ne!(qa.head, qa3.head);
    }

    #[test]
    fn quickhash_short_file_records_partial_head() {
        let data = b"tiny";
        let q = QuickHash::probe(&mut Cursor::new(data), 4).unwrap();
        assert_eq!(q.head_len, 4);
        assert_eq!(q.size, 4);
    }

    #[test]
    fn hash_file_streams_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clip.bin");
        let data = test_bytes(2 * 1024 * 1024);
        std::fs::write(&path, &data).unwrap();
        let id = hash_file(&path).unwrap();
        assert_eq!(id, ContentId::from_bytes(*blake3::hash(&data).as_bytes()));
    }
}
