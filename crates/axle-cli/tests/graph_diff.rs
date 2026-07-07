use assert_cmd::prelude::*;
use axle_artifact::ArtifactDirectoryExt;
use axle_core::{
    AxleArtifact, Declaration, DeclarationKind, Diagnostic, DiagnosticLevel, VerificationMode,
    VerificationResultStatus, VerificationStatus, VerificationSummary,
};
use axle_hash::Digest;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn build_artifact_dir() -> PathBuf {
    let temp = tempdir().unwrap();
    let path = temp.path().join("build.axle");

    let mut artifact = AxleArtifact::new_v0();
    artifact.source.module = Some("Sample".to_owned());
    artifact.source.path = Some("Sample.lean".to_owned());
    artifact.source.source_text = Some("theorem Sample.bar : True := trivial".to_owned());
    artifact.manifest.environment.lean_version = Some("lean-4.28.0".to_owned());
    artifact.declarations = vec![
        Declaration {
            name: "Sample.foo".to_owned(),
            kind: DeclarationKind::Def,
            statement_digest: Some(Digest::sha256("Nat")),
            body_digest: Some(Digest::sha256("1")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        },
        Declaration {
            name: "Sample.bar".to_owned(),
            kind: DeclarationKind::Theorem,
            statement_digest: Some(Digest::sha256("Sample.foo = 1")),
            body_digest: Some(Digest::sha256("rfl")),
            dependencies: vec!["Sample.foo".to_owned()],
            verification_status: VerificationStatus::Verified,
        },
    ];
    artifact.diagnostics = vec![Diagnostic {
        level: DiagnosticLevel::Warning,
        message: "sample warning".to_owned(),
        code: Some("graph.test".to_owned()),
    }];

    artifact.save_dir(&path).unwrap();
    let artifact_path = path.clone();
    let _ = temp.keep();
    artifact_path
}

fn verified_artifact_dir() -> PathBuf {
    let temp = tempdir().unwrap();
    let path = temp.path().join("verified.axle");

    let mut artifact = AxleArtifact::new_v0();
    artifact.source.module = Some("Sample".to_owned());
    artifact.source.path = Some("Sample.lean".to_owned());
    artifact.source.source_text = Some("theorem Sample.bar : Sample.foo = 1 := rfl".to_owned());
    artifact.manifest.environment.lean_version = Some("lean-4.28.0".to_owned());
    artifact.declarations = vec![
        Declaration {
            name: "Sample.foo".to_owned(),
            kind: DeclarationKind::Def,
            statement_digest: Some(Digest::sha256("Nat")),
            body_digest: Some(Digest::sha256("1")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        },
        Declaration {
            name: "Sample.bar".to_owned(),
            kind: DeclarationKind::Theorem,
            statement_digest: Some(Digest::sha256("Sample.foo = 1")),
            body_digest: Some(Digest::sha256("rfl")),
            dependencies: vec!["Sample.foo".to_owned()],
            verification_status: VerificationStatus::Verified,
        },
    ];
    artifact.verification = Some(VerificationSummary {
        mode: VerificationMode::VerifyProof,
        status: VerificationResultStatus::Pass,
        formal_statement_digest: Digest::sha256("theorem Sample.bar : Sample.foo = 1"),
        failed_declarations: Vec::new(),
    });

    artifact.save_dir(&path).unwrap();
    let artifact_path = path.clone();
    let _ = temp.keep();
    artifact_path
}

fn changed_artifact_dir() -> PathBuf {
    let temp = tempdir().unwrap();
    let path = temp.path().join("changed.axle");

    let mut artifact = AxleArtifact::new_v0();
    artifact.source.module = Some("Sample".to_owned());
    artifact.source.path = Some("Sample.lean".to_owned());
    artifact.source.source_text = Some("theorem Sample.bar : True := by omega".to_owned());
    artifact.manifest.environment.lean_version = Some("lean-4.28.0".to_owned());
    artifact.declarations = vec![
        Declaration {
            name: "Sample.foo".to_owned(),
            kind: DeclarationKind::Def,
            statement_digest: Some(Digest::sha256("Nat")),
            body_digest: Some(Digest::sha256("1")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        },
        Declaration {
            name: "Sample.bar".to_owned(),
            kind: DeclarationKind::Theorem,
            statement_digest: Some(Digest::sha256("True")),
            body_digest: Some(Digest::sha256("by omega")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Failed,
        },
    ];

    artifact.save_dir(&path).unwrap();
    let artifact_path = path.clone();
    let _ = temp.keep();
    artifact_path
}

#[test]
fn graph_json_succeeds_on_build_artifact() {
    let path = build_artifact_dir();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("graph")
        .arg(&path)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\": \"axle.graph.v0\""))
        .stdout(predicate::str::contains("\"kind\": \"artifact\""))
        .stdout(predicate::str::contains("\"type\": \"declaration\""));
}

#[test]
fn graph_dot_succeeds_on_verified_artifact() {
    let path = verified_artifact_dir();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("graph")
        .arg(&path)
        .arg("--format")
        .arg("dot")
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph axle {"))
        .stdout(predicate::str::contains("verification\\npass"));
}

#[test]
fn diff_text_reports_summary_changes() {
    let old_path = build_artifact_dir();
    let new_path = changed_artifact_dir();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("diff")
        .arg(&old_path)
        .arg(&new_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("root: changed"))
        .stdout(predicate::str::contains("changed_declarations: 1"))
        .stdout(predicate::str::contains(
            "changed_declaration: Sample.bar [statement, body, verification_status, dependencies]",
        ));
}

#[test]
fn diff_json_emits_structured_output() {
    let old_path = build_artifact_dir();
    let new_path = changed_artifact_dir();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("diff")
        .arg(&old_path)
        .arg(&new_path)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\": \"axle.diff.v0\""))
        .stdout(predicate::str::contains("\"changed_declarations\""))
        .stdout(predicate::str::contains("\"statement_changed\": true"));
}

#[test]
fn invalid_artifact_fails_before_graph_derivation() {
    let path = build_artifact_dir();
    fs::write(path.join("declarations.json"), "[]").unwrap();

    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("graph")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact verification failed"));

    let other = changed_artifact_dir();
    Command::cargo_bin("axle-rs")
        .unwrap()
        .arg("diff")
        .arg(&path)
        .arg(&other)
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact verification failed"));
}
