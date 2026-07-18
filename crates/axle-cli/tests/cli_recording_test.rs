use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Locate the built `axle-rs` binary for integration testing.
fn axle_bin() -> PathBuf {
    // The binary is produced by `cargo build` in target/debug at the workspace root.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("..");
    path.push("target");
    path.push("release");
    path.push("axle-rs");
    path
}

/// Integration test: build with --record produces a flight log, and replay reads it back.
#[test]
fn cli_build_records_flight_log() {
    let bin = axle_bin();
    assert!(bin.exists(), "axle-rs binary not found at {:?}", bin);

    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("source.lean");
    fs::write(&input, "def foo : Nat := 0").unwrap();
    let log_path = temp_dir.path().join("flight.log");

    let output = Command::new(&bin)
        .arg("build")
        .arg(&input)
        .arg("--environment")
        .arg("lean4")
        .arg("--record")
        .arg(&log_path)
        .output()
        .expect("failed to execute axle-rs");

    // The build may fail if no API is reachable, but the flight log must still be written.
    assert!(
        log_path.exists(),
        "flight log was not created at {:?}; stdout: {}, stderr: {}",
        log_path,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log_content = fs::read_to_string(&log_path).unwrap();
    assert!(!log_content.trim().is_empty(), "flight log is empty");
    assert!(
        log_content.contains("BuildCommand"),
        "flight log should contain BuildCommand event"
    );
}

/// Integration test: replay subcommand consumes a previously written flight log.
#[test]
fn cli_replay_reads_flight_log() {
    let bin = axle_bin();
    assert!(bin.exists(), "axle-rs binary not found at {:?}", bin);

    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("flight.log");

    // Write a minimal valid flight log (one BuildCommand event).
    let event = r#"[{"BuildCommand":"."}]"#;
    fs::write(&log_path, format!("{}\n", event)).unwrap();

    let output = Command::new(&bin)
        .arg("replay")
        .arg(&log_path)
        .output()
        .expect("failed to execute axle-rs replay");

    assert!(
        output.status.success(),
        "replay failed; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Build command for"),
        "replay output should mention Build command; got: {}",
        stdout
    );
}

/// Integration test: replay with a missing log file returns a non-zero exit code.
#[test]
fn cli_replay_missing_log_errors() {
    let bin = axle_bin();
    assert!(bin.exists(), "axle-rs binary not found at {:?}", bin);

    let temp_dir = TempDir::new().unwrap();
    let missing_log = temp_dir.path().join("does_not_exist.log");

    let output = Command::new(&bin)
        .arg("replay")
        .arg(&missing_log)
        .output()
        .expect("failed to execute axle-rs replay");

    assert!(
        !output.status.success(),
        "replay should fail for a missing log file"
    );
}

/// Integration test: replay with a corrupted log file returns a non-zero exit code.
#[test]
fn cli_replay_corrupted_log_errors() {
    let bin = axle_bin();
    assert!(bin.exists(), "axle-rs binary not found at {:?}", bin);

    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("corrupted.log");
    fs::write(&log_path, "this is not valid json\n").unwrap();

    let output = Command::new(&bin)
        .arg("replay")
        .arg(&log_path)
        .output()
        .expect("failed to execute axle-rs replay");

    assert!(
        !output.status.success(),
        "replay should fail for a corrupted log file"
    );
}
