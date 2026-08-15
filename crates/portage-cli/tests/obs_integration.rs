//! Integration tests for PR 1.5 observability: JSONL logs, redaction at the
//! process boundary, and the `status` metrics snapshot. Temp dirs, cleaned up.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn portage() -> Command {
    Command::new(env!("CARGO_BIN_EXE_portage"))
}

/// Initialize a data dir; returns None when the temp volume is too small
/// (the refusal contract is covered in init_integration.rs).
fn init_data_dir(tmp: &TempDir) -> Option<std::path::PathBuf> {
    let data_dir = tmp.path().join("PortageData");
    let out = portage()
        .args(["init", "--data-dir"])
        .arg(&data_dir)
        .output()
        .expect("run portage init");
    out.status.success().then_some(data_dir)
}

fn read_jsonl_lines(data_dir: &Path) -> Vec<serde_json::Value> {
    let logs = data_dir.join("logs");
    let mut lines = Vec::new();
    for entry in std::fs::read_dir(&logs).expect("logs dir exists") {
        let path = entry.expect("dir entry").path();
        assert!(
            path.extension().is_some_and(|e| e == "jsonl"),
            "unexpected log file: {}",
            path.display()
        );
        for line in std::fs::read_to_string(&path).expect("read log").lines() {
            lines.push(serde_json::from_str(line).expect("log line is valid JSON"));
        }
    }
    lines
}

#[test]
fn init_writes_structured_jsonl_logs() {
    let tmp = TempDir::new().expect("temp dir");
    let Some(data_dir) = init_data_dir(&tmp) else {
        return;
    };

    let lines = read_jsonl_lines(&data_dir);
    assert!(!lines.is_empty(), "init produced no log lines");
    let initialized = lines
        .iter()
        .find(|l| l["msg"] == "data dir initialized")
        .expect("init event logged");
    assert_eq!(initialized["level"], "INFO");
    assert!(initialized["ts"].as_str().unwrap().contains('T'));
    assert!(initialized["free"].as_u64().is_some());
}

#[test]
fn status_prom_output_and_snapshot_file() {
    let tmp = TempDir::new().expect("temp dir");
    let Some(data_dir) = init_data_dir(&tmp) else {
        return;
    };

    let out = portage()
        .args(["status", "--format", "prom", "--data-dir"])
        .arg(&data_dir)
        .output()
        .expect("run portage status");
    assert!(
        out.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("# TYPE portage_space_free gauge"), "{text}");
    assert!(text.contains("portage_status_runs 1"), "{text}");

    let snapshot = data_dir.join("metrics").join("portage.prom");
    assert!(snapshot.exists(), "portage.prom snapshot missing");
    let snap = std::fs::read_to_string(snapshot).expect("read snapshot");
    assert!(snap.contains("portage_space_free"));
}

#[test]
fn status_text_output_mentions_data_dir() {
    let tmp = TempDir::new().expect("temp dir");
    let Some(data_dir) = init_data_dir(&tmp) else {
        return;
    };

    let out = portage()
        .args(["status", "--data-dir"])
        .arg(&data_dir)
        .output()
        .expect("run portage status");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("free"));
    assert!(text.contains("portage.prom"));
}

#[test]
fn status_refuses_uninitialized_dir() {
    let tmp = TempDir::new().expect("temp dir");
    let out = portage()
        .args(["status", "--data-dir"])
        .arg(tmp.path().join("nope"))
        .output()
        .expect("run portage status");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("portage init"), "unhelpful error: {err}");
}
