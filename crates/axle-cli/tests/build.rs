use assert_cmd::prelude::*;
use httpmock::prelude::*;
use predicates::prelude::*;
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn fixture(name: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("workspace root");
    std::fs::read_to_string(root.join("tests/fixtures").join(name)).unwrap()
}

#[test]
fn build_creates_a_valid_axle_artifact() {
    let server = MockServer::start();
    let check_body = fixture("check_response.json");
    let extract_body = fixture("extract_decls_response.json");

    let check_mock = server.mock(|when, then| {
        when.method(POST).path("/api/v1/check");
        then.status(200)
            .header("content-type", "application/json")
            .body(check_body.clone());
    });
    let extract_mock = server.mock(|when, then| {
        when.method(POST).path("/api/v1/extract_decls");
        then.status(200)
            .header("content-type", "application/json")
            .body(extract_body.clone());
    });

    let temp = tempdir().unwrap();
    let input_path = temp.path().join("sample.lean");
    let output_path = temp.path().join("sample.axle");
    std::fs::write(&input_path, "import Mathlib\n#check Nat\n").unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("build")
        .arg(&input_path)
        .arg("--environment")
        .arg("lean-4.28.0")
        .arg("--api-url")
        .arg(server.base_url())
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("built"))
        .stdout(predicate::str::contains("artifact_id: sha256:"));

    check_mock.assert();
    extract_mock.assert();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("inspect")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("adapter_metadata: present"))
        .stdout(predicate::str::contains(
            "verification_statuses: verified=3, failed=1",
        ));

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid sha256:"));

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("hash")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("sha256:"));

    let manifest = std::fs::read_to_string(output_path.join("manifest.json")).unwrap();
    assert!(manifest.contains("\"adapter\": \"adapter.json\""));
}

#[test]
fn build_fails_when_axle_processed_content_differs() {
    let server = MockServer::start();
    let check_body = fixture("check_response.json");
    let mut extract_value: Value =
        serde_json::from_str(&fixture("extract_decls_response.json")).unwrap();
    extract_value["content"] = Value::String("import Mathlib\n#check String\n".to_owned());
    let extract_body = serde_json::to_string(&extract_value).unwrap();

    server.mock(|when, then| {
        when.method(POST).path("/api/v1/check");
        then.status(200)
            .header("content-type", "application/json")
            .body(check_body.clone());
    });
    server.mock(|when, then| {
        when.method(POST).path("/api/v1/extract_decls");
        then.status(200)
            .header("content-type", "application/json")
            .body(extract_body.clone());
    });

    let temp = tempdir().unwrap();
    let input_path = temp.path().join("sample.lean");
    std::fs::write(&input_path, "import Mathlib\n#check Nat\n").unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("build")
        .arg(&input_path)
        .arg("--environment")
        .arg("lean-4.28.0")
        .arg("--api-url")
        .arg(server.base_url())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "processed content mismatch between AXLE check and extract_decls responses",
        ));
}
