use axle_core::{AxleArtifact, Diagnostic, SourceInfo};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxleClient {
    base_url: String,
}

impl AxleClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxleVerifyResponse {
    pub source: SourceInfo,
    #[serde(default)]
    pub declarations: Vec<axle_core::Declaration>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

impl From<AxleVerifyResponse> for AxleArtifact {
    fn from(response: AxleVerifyResponse) -> Self {
        let mut artifact = AxleArtifact::new_v0();
        artifact.source = response.source;
        artifact.declarations = response.declarations;
        artifact.diagnostics = response.diagnostics;
        artifact
    }
}
