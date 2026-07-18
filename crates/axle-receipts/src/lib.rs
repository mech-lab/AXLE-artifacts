use anyhow::{Context, Result};
use axle_hash::Digest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxleReceipt {
    pub schema: String,
    pub subject: ReceiptSubject,
    pub verification: VerificationSummary,
    pub environment: ReceiptEnvironment,
    pub issued_at: Option<DateTime<Utc>>,
    pub signature: Option<String>,
}

impl AxleReceipt {
    pub fn issue_unsigned(
        artifact_id: Digest,
        verifier: impl Into<String>,
        verifier_version: impl Into<String>,
        policy: impl Into<String>,
    ) -> Self {
        Self {
            schema: "axle.receipt.v0".to_owned(),
            subject: ReceiptSubject {
                artifact_id,
                artifact_schema: "axle.artifact.v0".to_owned(),
            },
            verification: VerificationSummary {
                status: VerificationOutcome::Pass,
                verifier: verifier.into(),
                verifier_version: verifier_version.into(),
                policy: policy.into(),
            },
            environment: ReceiptEnvironment {
                os: None,
                engine: Some("axiom-lean-engine".to_owned()),
                lean_version: None,
            },
            issued_at: Some(Utc::now()),
            signature: None,
        }
    }

    pub fn verifies_artifact(&self, artifact_id: &Digest) -> bool {
        &self.subject.artifact_id == artifact_id
    }

    pub fn artifact_digest(&self) -> &Digest {
        &self.subject.artifact_id
    }

    pub fn save_dir(&self, path: &Path) -> Result<()> {
        let target = if path.is_dir() {
            path.join("receipt.axle")
        } else {
            path.to_path_buf()
        };
        let json = serde_json::to_string_pretty(self)
            .context("failed to serialize receipt to JSON")?;
        fs::write(&target, json)
            .with_context(|| format!("failed to write receipt to {}", target.display()))?;
        Ok(())
    }

    pub fn load_dir(path: &Path) -> Result<Self> {
        let target = if path.is_dir() {
            path.join("receipt.axle")
        } else {
            path.to_path_buf()
        };
        let data = fs::read_to_string(&target)
            .with_context(|| format!("failed to read receipt from {}", target.display()))?;
        serde_json::from_str(&data)
            .with_context(|| format!("failed to parse receipt from {}", target.display()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptSubject {
    pub artifact_id: Digest,
    pub artifact_schema: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub status: VerificationOutcome,
    pub verifier: String,
    pub verifier_version: String,
    pub policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationOutcome {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptEnvironment {
    pub os: Option<String>,
    pub engine: Option<String>,
    pub lean_version: Option<String>,
}
