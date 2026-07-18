use axle_hash::Digest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claim types for different domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    InsuranceRisk,
    ComplianceControl,
    LegalDisclosure,
    DecisionProof,
    /// Generic fallback for custom claim types
    Custom(String),
}

impl ClaimType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "insurance_risk" => Self::InsuranceRisk,
            "compliance_control" => Self::ComplianceControl,
            "legal_disclosure" => Self::LegalDisclosure,
            "decision_proof" => Self::DecisionProof,
            _ => Self::Custom(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::InsuranceRisk => "insurance_risk",
            Self::ComplianceControl => "compliance_control",
            Self::LegalDisclosure => "legal_disclosure",
            Self::DecisionProof => "decision_proof",
            Self::Custom(s) => s,
        }
    }
}

/// Subject of a claim - who or what is being claimed about
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimSubject {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// Evidence bundle attached to a claim
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub payload_hash: Digest,
    #[serde(default)]
    pub attachment_hashes: Vec<Digest>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Core claim structure - replaces source.json, declarations.json, diagnostics.json
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub schema: String,
    pub claim_type: ClaimType,
    pub subject: ClaimSubject,
    pub issuer: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub evidence: Evidence,
    #[serde(default)]
    pub verification_policy: VerificationPolicy,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Claim {
    pub fn new(
        claim_type: ClaimType,
        subject: ClaimSubject,
        issuer: String,
        evidence: Evidence,
    ) -> Self {
        Self {
            schema: "axle.claim.v0".to_string(),
            claim_type,
            subject,
            issuer,
            created_at: Utc::now(),
            evidence,
            verification_policy: VerificationPolicy::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Verification policy for claims
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub schema_version: String,
    #[serde(default)]
    pub required_signers: Vec<String>,
    #[serde(default)]
    pub verification_methods: Vec<VerificationMethod>,
    #[serde(default)]
    pub constraints: HashMap<String, serde_json::Value>,
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            required_signers: Vec::new(),
            verification_methods: Vec::new(),
            constraints: HashMap::new(),
        }
    }
}

/// Verification method for claims
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub method: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Evidence structure - replaces evidence.json
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub schema: String,
    pub payload_hash: Digest,
    #[serde(default)]
    pub attachment_hashes: Vec<Digest>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EvidenceBundle {
    pub fn new(payload_hash: Digest) -> Self {
        Self {
            schema: "axle.evidence.v0".to_string(),
            payload_hash,
            attachment_hashes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}