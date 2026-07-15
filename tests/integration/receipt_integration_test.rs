use std::fs;
use std::path::Path;
use tempfile::TempDir;

use axle_rs::Cli;
use axle_rs::receipt::{receipt_issue, receipt_verify};

/// Integration test for receipt issue and verify commands
#[test]
fn receipt_integration_roundtrip() {
    // Create temporary directory for artifact and receipt files
    let temp_dir = TempDir::new().unwrap();
    let artifact_path = temp_dir.path().join("artifact.axle");
    let receipt_path = temp_dir.path().join("receipt.axle");

    // Initialize CLI instance
    let cli = Cli::new();

    // Issue receipt using CLI command
    let issue_args = receipt_issue_args { path: artifact_path.clone() };
    let exit_code = receipt_issue(issue_args.path.clone()).unwrap();
    assert_eq!(exit_code, 0);

    // Verify receipt using CLI command
    let verify_args = receipt_verify_args {
        receipt_path: receipt_path.clone(),
        artifact_path: artifact_path.clone(),
    };
    let exit_code = receipt_verify(verify_args).unwrap();
    assert_eq!(exit_code, 0);
}

#[test]
fn receipt_integration_negative_verification() {
    // Create temporary directory for artifact and receipt files
    let temp_dir = TempDir::new().unwrap();
    let artifact_path = temp_dir.path().join("artifact.axle");
    let receipt_path = temp_dir.path().join("receipt.axle");

    // Initialize CLI instance
    let cli = Cli::new();

    // Issue receipt using CLI command
    let issue_args = receipt_issue_args { path: artifact_path.clone() };
    receipt_issue(issue_args.path.clone()).unwrap();

    // Tamper with artifact by rewriting it
    fs::write(&artifact_path, b"tampered content").unwrap();

    // Attempt to verify receipt against tampered artifact
    let verify_args = receipt_verify_args {
        receipt_path: receipt_path.clone(),
        artifact_path: artifact_path.clone(),
    };
    let exit_code = receipt_verify(verify_args);
    assert!(exit_code.is_err());
}