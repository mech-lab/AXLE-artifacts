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
fn verify_proof_success_emits_artifact_and_inspect_reports_pass() {
    let server = MockServer::start();
    let verify_body = fixture("verify_proof_pass_response.json");
    let extract_body = fixture("extract_decls_verified_response.json");

    server.mock(|when, then| {
        when.method(POST).path("/api/v1/verify_proof");
        then.status(200)
            .header("content-type", "application/json")
            .body(verify_body.clone());
    });
    server.mock(|when, then| {
        when.method(POST).path("/api/v1/extract_decls");
        then.status(200)
            .header("content-type", "application/json")
            .body(extract_body.clone());
    });

    let temp = tempdir().unwrap();
    let formal_statement_path = temp.path().join("statement.lean");
    let content_path = temp.path().join("proof.lean");
    let output_path = temp.path().join("proof.verified.axle");
    std::fs::write(
        &formal_statement_path,
        "theorem sample.bar : sample.foo = 1 := by\n  rfl\n",
    )
    .unwrap();
    std::fs::write(
        &content_path,
        "import Mathlib\n\ndef sample.foo : Nat := 1\n\ntheorem sample.bar : sample.foo = 1 := rfl\n\ninstance sample.instNat : Inhabited Nat where\n  default := sample.foo\n\nexample : sample.foo = sample.foo := by\n  rfl\n",
    )
    .unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify-proof")
        .arg(&formal_statement_path)
        .arg(&content_path)
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

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("inspect")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("adapter_metadata: present"))
        .stdout(predicate::str::contains("verification_mode: verify_proof"))
        .stdout(predicate::str::contains("verification_status: pass"))
        .stdout(predicate::str::contains(
            "verification_failed_declarations: 0",
        ));

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid sha256:"));
}

#[test]
fn verify_proof_failure_emits_artifact_and_returns_nonzero() {
    let server = MockServer::start();
    let verify_body = fixture("verify_proof_fail_response.json");
    let extract_body = fixture("extract_decls_response.json");

    server.mock(|when, then| {
        when.method(POST).path("/api/v1/verify_proof");
        then.status(200)
            .header("content-type", "application/json")
            .body(verify_body.clone());
    });
    server.mock(|when, then| {
        when.method(POST).path("/api/v1/extract_decls");
        then.status(200)
            .header("content-type", "application/json")
            .body(extract_body.clone());
    });

    let temp = tempdir().unwrap();
    let formal_statement_path = temp.path().join("statement.lean");
    let content_path = temp.path().join("proof.lean");
    let output_path = temp.path().join("proof.verified.axle");
    std::fs::write(
        &formal_statement_path,
        "theorem sample.bar : sample.foo = 1 := by\n  rfl\n",
    )
    .unwrap();
    std::fs::write(
        &content_path,
        "import Mathlib\n\ndef sample.foo : Nat := 1\n\ntheorem sample.bar : sample.foo = 1 := rfl\n\ninstance sample.instNat : Inhabited Nat where\n  default := sample.foo\n\nexample : sample.foo = sample.foo := by\n  sorry\n",
    )
    .unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify-proof")
        .arg(&formal_statement_path)
        .arg(&content_path)
        .arg("--environment")
        .arg("lean-4.28.0")
        .arg("--api-url")
        .arg(server.base_url())
        .arg("-o")
        .arg(&output_path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("built"))
        .stdout(predicate::str::contains("verification_status: fail"))
        .stdout(predicate::str::contains("failed_declarations: 1"));

    assert!(output_path.is_dir());

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("inspect")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("verification_mode: verify_proof"))
        .stdout(predicate::str::contains("verification_status: fail"))
        .stdout(predicate::str::contains(
            "verification_failed_declarations: 1",
        ));

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify")
        .arg(&output_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid sha256:"));
}

#[test]
fn verify_proof_mismatch_fails_without_writing_artifact() {
    let server = MockServer::start();
    let mut verify_value: Value =
        serde_json::from_str(&fixture("verify_proof_pass_response.json")).unwrap();
    verify_value["content"] = Value::String("theorem broken : False := by\n  sorry\n".to_owned());
    let verify_body = serde_json::to_string(&verify_value).unwrap();
    let extract_body = fixture("extract_decls_response.json");

    server.mock(|when, then| {
        when.method(POST).path("/api/v1/verify_proof");
        then.status(200)
            .header("content-type", "application/json")
            .body(verify_body.clone());
    });
    server.mock(|when, then| {
        when.method(POST).path("/api/v1/extract_decls");
        then.status(200)
            .header("content-type", "application/json")
            .body(extract_body.clone());
    });

    let temp = tempdir().unwrap();
    let formal_statement_path = temp.path().join("statement.lean");
    let content_path = temp.path().join("proof.lean");
    let output_path = temp.path().join("proof.verified.axle");
    std::fs::write(
        &formal_statement_path,
        "theorem sample.bar : sample.foo = 1",
    )
    .unwrap();
    std::fs::write(&content_path, "theorem sample.bar : sample.foo = 1 := rfl").unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("verify-proof")
        .arg(&formal_statement_path)
        .arg(&content_path)
        .arg("--environment")
        .arg("lean-4.28.0")
        .arg("--api-url")
        .arg(server.base_url())
        .arg("-o")
        .arg(&output_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "processed content mismatch between AXLE verify_proof and extract_decls responses",
        ));

    assert!(!output_path.exists());
}
