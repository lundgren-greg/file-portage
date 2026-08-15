//! Integration tests for `portage doctor` (PR 3 stub).
//!
//! Boundary covered: the CLI process against a real data dir + SQLite
//! catalog. Uses temp dirs and cleans up via `tempfile`.

use std::process::Command;

use tempfile::TempDir;

fn portage() -> Command {
    Command::new(env!("CARGO_BIN_EXE_portage"))
}

/// Init a data dir; returns `None` when the temp volume is below the 8 GiB
/// minimum (that refusal path is covered by the init tests).
fn init_data_dir(tmp: &TempDir) -> Option<std::path::PathBuf> {
    let data_dir = tmp.path().join("PortageData");
    let out = portage()
        .args(["init", "--data-dir"])
        .arg(&data_dir)
        .output()
        .expect("run portage init");
    out.status.success().then_some(data_dir)
}

#[test]
fn doctor_reports_ok_on_a_fresh_catalog() {
    let tmp = TempDir::new().expect("temp dir");
    let Some(data_dir) = init_data_dir(&tmp) else {
        return;
    };

    let out = portage()
        .args(["doctor", "--data-dir"])
        .arg(&data_dir)
        .output()
        .expect("run portage doctor");
    assert!(
        out.status.success(),
        "doctor failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("schema       : v2"), "stdout: {text}");
    assert!(text.contains("integrity    : ok"), "stdout: {text}");
    assert!(text.contains("foreign keys : ok"), "stdout: {text}");
    assert!(
        data_dir.join("catalog.sqlite").exists(),
        "doctor migrated a fresh catalog in place"
    );
}

#[test]
fn doctor_refuses_an_uninitialized_dir() {
    let tmp = TempDir::new().expect("temp dir");
    let out = portage()
        .args(["doctor", "--data-dir"])
        .arg(tmp.path().join("nowhere"))
        .output()
        .expect("run portage doctor");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("portage init"), "stderr: {err}");
}
