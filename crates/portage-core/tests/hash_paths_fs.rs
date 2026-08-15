//! Integration: hashing and path containment against the real filesystem.
//!
//! Exercises the on-disk boundary the indexer and executor will use: large
//! streamed files, files created then hashed, and containment on real dirs.

use std::io::Write;
use std::path::Path;

use portage_core::hash::{hash_file, HashAlgo, MultiHasher, QuickHash, QUICK_PROBE_BYTES};
use portage_core::paths::ensure_inside;
use portage_core::ContentId;

fn write_patterned(path: &Path, len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state = 0xDEADBEEFCAFEF00Du64;
    while data.len() < len {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        data.extend_from_slice(&state.to_le_bytes());
    }
    data.truncate(len);
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&data).unwrap();
    data
}

#[test]
fn twenty_mib_file_streams_to_the_same_digests_as_memory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.bin");
    let data = write_patterned(&path, 20 * 1024 * 1024);

    let mut file = std::fs::File::open(&path).unwrap();
    let streamed = MultiHasher::new(&[HashAlgo::Md5, HashAlgo::Sha256])
        .unwrap()
        .digest_reader(&mut file)
        .unwrap();

    let mut in_memory = MultiHasher::new(&[HashAlgo::Md5, HashAlgo::Sha256]).unwrap();
    in_memory.update(&data);
    assert_eq!(streamed, in_memory.finalize());
    assert_eq!(
        streamed.content_id,
        ContentId::from_bytes(*blake3::hash(&data).as_bytes())
    );
    assert_eq!(hash_file(&path).unwrap(), streamed.content_id);
}

#[test]
fn quickhash_from_disk_matches_metadata_size() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("clip.bin");
    write_patterned(&path, 3 * QUICK_PROBE_BYTES);

    let size = std::fs::metadata(&path).unwrap().len();
    let mut file = std::fs::File::open(&path).unwrap();
    let quick = QuickHash::probe(&mut file, size).unwrap();
    assert_eq!(quick.size, 3 * QUICK_PROBE_BYTES as u64);
    assert_eq!(quick.head_len as usize, QUICK_PROBE_BYTES);
}

#[test]
fn containment_gates_a_real_overlay_style_tree() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("media-root");
    std::fs::create_dir_all(root.join("footage").join("2024")).unwrap();

    // Legitimate destination the planner would emit.
    let ok = ensure_inside(&root, &Path::new("footage").join("2024").join("clip.mp4")).unwrap();
    assert!(ok.starts_with(&root));

    // Hostile destinations must be rejected before any I/O.
    assert!(ensure_inside(&root, &Path::new("..").join("escape.mp4")).is_err());
    assert!(ensure_inside(&root, Path::new("footage/clip.mp4:zone.identifier")).is_err());
    assert!(ensure_inside(&root, dir.path()).is_err());
}
