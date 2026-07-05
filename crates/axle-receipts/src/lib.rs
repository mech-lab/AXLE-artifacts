use axle_hash::Digest;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
