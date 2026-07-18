use axle_hash::Digest;
use chrono::{DateTime, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Receipt identifier for binding artifacts to attestations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptId {
    pub id: String,
    pub version: u32,
}

impl ReceiptId {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let mut bytes = [0u8; 8];
        rng.fill_bytes(&mut bytes);
        Self {
            id: format!("rec-{}", hex::encode(bytes)),
            version: 1,
        }
    }
}

/// Signing layer for proof artifacts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SigningLayer {
    pub schema: String,
    pub signer_public_key: String,
    pub signature: String,
    pub signed_at: DateTime<Utc>,
    pub receipt_id: ReceiptId,
    pub verification_policy: VerificationPolicy,
}

impl SigningLayer {
    pub fn new(
        signing_key: &SigningKey,
        artifact_data: &[u8],
        verification_policy: VerificationPolicy,
    ) -> Self {
        let signature = signing_key.sign(artifact_data);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            schema: "axle.signing.v0".to_string(),
            signer_public_key: hex::encode(verifying_key.to_bytes()),
            signature: hex::encode(signature.to_bytes()),
            signed_at: Utc::now(),
            receipt_id: ReceiptId::generate(),
            verification_policy,
        }
    }

    pub fn verify(&self, artifact_data: &[u8], public_key_bytes: &[u8]) -> bool {
        let verifying_key = match VerifyingKey::from_bytes(public_key_bytes.try_into().unwrap()) {
            Ok(k) => k,
            Err(_) => return false,
        };
        
        let signature_bytes = match hex::decode(&self.signature) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        let signature = match Signature::from_bytes(&signature_bytes.try_into().unwrap()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        
        verifying_key.verify(artifact_data, &signature).is_ok()
    }
}

/// Verification policy for claims
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub schema_version: String,
    pub claim_type: String,
    pub required_signers: Vec<String>,
    pub verification_methods: Vec<String>,
    pub constraints: HashMap<String, serde_json::Value>,
}

impl VerificationPolicy {
    pub fn new(claim_type: &str) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            claim_type: claim_type.to_string(),
            required_signers: vec![],
            verification_methods: vec![],
            constraints: HashMap::new(),
        }
    }

    pub fn for_claim_type(claim_type: &str) -> Self {
        let mut policy = Self::new(claim_type);
        
        match claim_type {
            "insurance_risk" => {
                policy.required_signers = vec!["insurer".to_string(), "regulator".to_string()];
                policy.verification_methods = vec!["clinical_data".to_string(), "policy_docs".to_string()];
                policy.constraints.insert("min_confidence".to_string(), serde_json::json!(0.95));
            }
            "compliance_control" => {
                policy.required_signers = vec!["auditor".to_string()];
                policy.verification_methods = vec!["system_logs".to_string(), "manual_review".to_string()];
                policy.constraints.insert("max_age_days".to_string(), serde_json::json!(365));
            }
            "legal_disclosure" => {
                policy.required_signers = vec!["legal_counsel".to_string(), "compliance_officer".to_string()];
                policy.verification_methods = vec!["document_signatures".to_string(), "witness_testimony".to_string()];
                policy.constraints.insert("jurisdiction".to_string(), serde_json::json!("US"));
            }
            "decision_proof" => {
                policy.required_signers = vec!["data_steward".to_string(), "decision_maker".to_string()];
                policy.verification_methods = vec!["model_outputs".to_string(), "input_data".to_string()];
                policy.constraints.insert("model_version".to_string(), serde_json::json!("v2.1"));
            }
            _ => {
                policy.required_signers = vec!["verifier".to_string()];
                policy.verification_methods = vec!["custom".to_string()];
            }
        }
        
        policy
    }
}

/// Complete proof artifact with signing layer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofArtifact {
    pub schema: String,
    pub claim: serde_json::Value,
    pub evidence: serde_json::Value,
    pub signing_layer: SigningLayer,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ProofArtifact {
    pub fn new(
        claim: serde_json::Value,
        evidence: serde_json::Value,
        signing_key: &SigningKey,
        verification_policy: VerificationPolicy,
    ) -> Self {
        let claim_bytes = serde_json::to_vec(&claim).unwrap();
        let evidence_bytes = serde_json::to_vec(&evidence).unwrap();
        let artifact_data = [&claim_bytes[..], &evidence_bytes[..]].concat();
        
        let signing_layer = SigningLayer::new(&signing_key, &artifact_data, verification_policy);
        
        Self {
            schema: "axle.proof_artifact.v0".to_string(),
            claim,
            evidence,
            signing_layer,
            metadata: HashMap::new(),
        }
    }

    pub fn verify(&self, public_key_bytes: &[u8]) -> bool {
        let claim_bytes = serde_json::to_vec(&self.claim).unwrap();
        let evidence_bytes = serde_json::to_vec(&self.evidence).unwrap();
        let artifact_data = [&claim_bytes[..], &evidence_bytes[..]].concat();
        
        self.signing_layer.verify(&artifact_data, public_key_bytes)
    }

    pub fn get_receipt_id(&self) -> &ReceiptId {
        &self.signing_layer.receipt_id
    }

    pub fn get_verification_policy(&self) -> &VerificationPolicy {
        &self.signing_layer.verification_policy
    }
}