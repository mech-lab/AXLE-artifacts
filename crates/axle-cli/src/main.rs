use anyhow::{Context, Result};
use clap::{Command, Arg};
use serde_json::{Value, json};
use std::fs::File;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use hex::decode;

fn main() -> Result<()> {
    let matches = Command::new("axle-rs")
        .version("0.1.0")
        .about("Rust-native proof artifact tooling")
        .subcommand(
            Command::new("attest")
                .arg(Arg::new("claim").required(true))
                .arg(Arg::new("evidence").required(true))
        )
        .subcommand(
            Command::new("verify")
                .arg(Arg::new("artifact").required(true))
                .arg(Arg::new("public_key").required(true))
        )
        .subcommand(
            Command::new("issue")
                .arg(Arg::new("claim").required(true))
                .arg(Arg::new("evidence").required(true))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("attest", sub_matches)) => {
            let claim_path = sub_matches.get_one::<String>("claim").unwrap();
            let evidence_path = sub_matches.get_one::<String>("evidence").unwrap();
            attest(claim_path, evidence_path)?;
        }
        Some(("verify", sub_matches)) => {
            let artifact_path = sub_matches.get_one::<String>("artifact").unwrap();
            let public_key = sub_matches.get_one::<String>("public_key").unwrap();
            verify(artifact_path, public_key)?;
        }
        Some(("issue", sub_matches)) => {
            let claim_path = sub_matches.get_one::<String>("claim").unwrap();
            let evidence_path = sub_matches.get_one::<String>("evidence").unwrap();
            issue(claim_path, evidence_path)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn attest(claim_path: &str, evidence_path: &str) -> Result<()> {
    // Load claim and evidence
    let claim_file = File::open(claim_path).with_context(|| "Failed to open claim file")?;
    let claim: Value = serde_json::from_reader(claim_file).with_context(|| "Failed to parse claim.json")?;

    let evidence_file = File::open(evidence_path).with_context(|| "Failed to open evidence file")?;
    let evidence: Value = serde_json::from_reader(evidence_file).with_context(|| "Failed to parse evidence.json")?;

    // Generate keypair for signing
    let mut csprng = OsRng;
    let mut secret_key_bytes = [0u8; 32];
    csprng.fill_bytes(&mut secret_key_bytes);
    let signing_key = SigningKey::from_bytes(&secret_key_bytes);
    let verifying_key = signing_key.verifying_key();

    // Sign artifact and generate receipt
    let signed_artifact = sign_artifact(&claim, &evidence, &signing_key)?;
    let receipt_id = generate_receipt_id()?;

    // Save signed artifact and receipt
    save_signed_artifact(&signed_artifact, "signed_artifact.axle")?;
    save_receipt(&receipt_id, &claim, &evidence, &verifying_key)?;

    println!("Attestation successful. Artifact ID: {}", signed_artifact["artifact_id"]);
    Ok(())
}

fn verify(artifact_path: &str, public_key: &str) -> Result<()> {
    // Load artifact
    let artifact_file = File::open(artifact_path).with_context(|| "Failed to open artifact file")?;
    let artifact: Value = serde_json::from_reader(artifact_file).with_context(|| "Failed to parse artifact")?;

    // Verify signature
    let signature_hex = artifact["signature"].as_str().unwrap();
    let public_key_bytes = decode(public_key.trim_start_matches("0x"))?;

    // Verify using ed25519
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())?;
    let signature_bytes = decode(signature_hex.trim_start_matches("0x"))?;
    let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());

    // Verify signature against artifact data
    let data = serde_json::to_vec(&artifact["data"])?;
    verifying_key.verify(&data, &signature)?;

    println!("Verification successful for artifact: {}", artifact["artifact_id"]);
    Ok(())
}

fn issue(claim_path: &str, evidence_path: &str) -> Result<()> {
    // Issue a new proof artifact
    let claim_file = File::open(claim_path).with_context(|| "Failed to open claim file")?;
    let claim: Value = serde_json::from_reader(claim_file).with_context(|| "Failed to parse claim.json")?;

    let evidence_file = File::open(evidence_path).with_context(|| "Failed to open evidence file")?;
    let evidence: Value = serde_json::from_reader(evidence_file).with_context(|| "Failed to parse evidence.json")?;

    // Create artifact
    let artifact = create_artifact(&claim, &evidence)?;

    // Save artifact
    save_artifact(&artifact, "proof_artifact.axle")?;

    println!("Proof artifact issued successfully. Artifact ID: {}", artifact["artifact_id"]);
    Ok(())
}

fn sign_artifact(claim: &Value, evidence: &Value, signing_key: &SigningKey) -> Result<Value> {
    // Combine data for signing
    let data_to_sign = format!("claim:{:?}evidence:{:?}", claim, evidence);
    let signature = signing_key.sign(data_to_sign.as_bytes());

    // Create signed artifact structure
    Ok(json!({
        "artifact_id": "sha256:abc123...",
        "data": {
            "claim": claim,
            "evidence": evidence
        },
        "signature": hex::encode(signature.to_bytes())
    }))
}

fn generate_receipt_id() -> Result<String> {
    // Generate unique receipt ID
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    Ok(format!("rec-{}", hex::encode(bytes)).to_string())
}

fn save_signed_artifact(artifact: &Value, path: &str) -> Result<()> {
    let file = File::create(path).with_context(|| "Failed to create artifact file")?;
    serde_json::to_writer(file, artifact).with_context(|| "Failed to write artifact")?;
    Ok(())
}

fn save_receipt(receipt_id: &str, claim: &Value, evidence: &Value, verifying_key: &VerifyingKey) -> Result<()> {
    let receipt = json!({
        "id": receipt_id,
        "claim": claim.clone(),
        "evidence": evidence.clone(),
        "signed_at": "2026-07-18T02:00:00Z",
        "signer_public_key": hex::encode(verifying_key.to_bytes())
    });
    let file = File::create("receipt.json").with_context(|| "Failed to create receipt file")?;
    serde_json::to_writer(file, &receipt).with_context(|| "Failed to write receipt")?;
    Ok(())
}

fn create_artifact(claim: &Value, evidence: &Value) -> Result<Value> {
    // Create a new proof artifact from claim and evidence
    Ok(json!({
        "artifact_id": "sha256:xyz789...",
        "claim": claim.clone(),
        "evidence": evidence.clone()
    }))
}

fn save_artifact(artifact: &Value, path: &str) -> Result<()> {
    let file = File::create(path).with_context(|| "Failed to create artifact file")?;
    serde_json::to_writer(file, artifact).with_context(|| "Failed to write artifact")?;
    Ok(())
}
