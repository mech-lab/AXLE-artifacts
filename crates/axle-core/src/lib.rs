use axle_hash::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const ARTIFACT_SCHEMA_V0: &str = "axle.artifact.v0";
pub const DEFAULT_MANIFEST_FILE: &str = "manifest.json";
pub const DEFAULT_SOURCE_FILE: &str = "source.json";
pub const DEFAULT_DECLARATIONS_FILE: &str = "declarations.json";
pub const DEFAULT_DIAGNOSTICS_FILE: &str = "diagnostics.json";
pub const DEFAULT_CLAIM_FILE: &str = "claim.json";
pub const DEFAULT_EVIDENCE_FILE: &str = "evidence.json";
pub const DEFAULT_HASHES_FILE: &str = "hashes.json";
pub const DEFAULT_VERIFICATION_FILE: &str = "verification.json";
pub const DEFAULT_ADAPTER_FILE: &str = "adapter.json";
pub const ADAPTER_SCHEMA_V0: &str = "axle.adapter.v0";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxleArtifact {
    pub manifest: Manifest,
    pub source: SourceInfo,
    #[serde(default)]
    pub declarations: Vec<Declaration>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub hashes: HashesFile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<AdapterMetadata>,
}

impl Default for AxleArtifact {
    fn default() -> Self {
        Self::new_v0()
    }
}

impl AxleArtifact {
    pub fn new_v0() -> Self {
        Self {
            manifest: Manifest {
                schema: ARTIFACT_SCHEMA_V0.to_owned(),
                artifact_id: None,
                producer: Producer {
                    name: "axle-rs".to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                },
                source: SourceSummary {
                    language: "lean4".to_owned(),
                    source_digest: None,
                },
                environment: Environment {
                    engine: "axiom-lean-engine".to_owned(),
                    engine_version: None,
                    lean_version: None,
                    mathlib_digest: None,
                    environment_digest: None,
                },
                objects: ObjectPaths::default(),
            },
            source: SourceInfo {
                language: "lean4".to_owned(),
                module: None,
                path: None,
                source_text: None,
            },
            declarations: Vec::new(),
            diagnostics: Vec::new(),
            hashes: HashesFile::default(),
            verification: None,
            adapter: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub artifact_id: Option<Digest>,
    pub producer: Producer,
    pub source: SourceSummary,
    pub environment: Environment,
    pub objects: ObjectPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSummary {
    pub language: String,
    pub source_digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInfo {
    pub language: String,
    pub module: Option<String>,
    pub path: Option<String>,
    pub source_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub engine: String,
    pub engine_version: Option<String>,
    pub lean_version: Option<String>,
    pub mathlib_digest: Option<Digest>,
    pub environment_digest: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectPaths {
    pub source: String,
    pub declarations: String,
    pub diagnostics: String,
    pub hashes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
}

impl Default for ObjectPaths {
    fn default() -> Self {
        Self {
            source: DEFAULT_SOURCE_FILE.to_owned(),
            declarations: DEFAULT_CLAIM_FILE.to_owned(),
            diagnostics: DEFAULT_EVIDENCE_FILE.to_owned(),
            hashes: DEFAULT_HASHES_FILE.to_owned(),
            verification: None,
            adapter: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum DeclarationKind {
    #[serde(rename = "theorem")]
    Theorem,
    #[serde(rename = "lemma")]
    Lemma,
    #[serde(rename = "def")]
    Def,
    #[serde(rename = "abbrev")]
    Abbrev,
    #[serde(rename = "axiom")]
    Axiom,
    #[serde(rename = "opaque")]
    Opaque,
    #[serde(rename = "structure")]
    Structure,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "class inductive")]
    ClassInductive,
    #[serde(rename = "inductive")]
    Inductive,
    #[serde(rename = "instance")]
    Instance,
    #[serde(rename = "example")]
    Example,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    pub name: String,
    pub kind: DeclarationKind,
    pub statement_digest: Option<Digest>,
    pub body_digest: Option<Digest>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub verification_status: VerificationStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashesFile {
    pub source: Option<Digest>,
    pub declarations: Option<Digest>,
    pub diagnostics: Option<Digest>,
    pub environment: Option<Digest>,
    pub verification: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMode {
    VerifyProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationResultStatus {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationSummary {
    pub mode: VerificationMode,
    pub status: VerificationResultStatus,
    pub formal_statement_digest: Digest,
    #[serde(default)]
    pub failed_declarations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterOperation {
    Build,
    VerifyProof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdapterMetadata {
    pub schema: String,
    pub operation: AdapterOperation,
    pub requests: BTreeMap<String, Value>,
    pub responses: BTreeMap<String, Value>,
}

impl AdapterMetadata {
    pub fn new(
        operation: AdapterOperation,
        requests: BTreeMap<String, Value>,
        responses: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            schema: ADAPTER_SCHEMA_V0.to_owned(),
            operation,
            requests,
            responses,
        }
    }

    pub fn build(requests: BTreeMap<String, Value>, responses: BTreeMap<String, Value>) -> Self {
        Self::new(AdapterOperation::Build, requests, responses)
    }

    pub fn verify_proof(
        requests: BTreeMap<String, Value>,
        responses: BTreeMap<String, Value>,
    ) -> Self {
        Self::new(AdapterOperation::VerifyProof, requests, responses)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AdapterMetadataCurrent {
    pub schema: String,
    pub operation: AdapterOperation,
    #[serde(default)]
    pub requests: BTreeMap<String, Value>,
    #[serde(default)]
    pub responses: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AdapterMetadataLegacy {
    pub schema: String,
    pub check: Value,
    pub extract_decls: Value,
}

impl Serialize for AdapterMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        AdapterMetadataCurrent {
            schema: self.schema.clone(),
            operation: self.operation.clone(),
            requests: self.requests.clone(),
            responses: self.responses.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AdapterMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;

        if raw.get("operation").is_some() {
            let current: AdapterMetadataCurrent =
                serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
            return Ok(Self {
                schema: current.schema,
                operation: current.operation,
                requests: current.requests,
                responses: current.responses,
            });
        }

        let legacy: AdapterMetadataLegacy =
            serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
        let mut responses = BTreeMap::new();
        responses.insert("check".to_owned(), legacy.check);
        responses.insert("extract_decls".to_owned(), legacy.extract_decls);

        Ok(Self {
            schema: legacy.schema,
            operation: AdapterOperation::Build,
            requests: BTreeMap::new(),
            responses,
        })
    }
}

impl DeclarationKind {
    pub fn from_axle_kind(kind: &str) -> Self {
        match kind {
            "theorem" => Self::Theorem,
            "lemma" => Self::Lemma,
            "def" => Self::Def,
            "abbrev" => Self::Abbrev,
            "axiom" => Self::Axiom,
            "opaque" => Self::Opaque,
            "structure" => Self::Structure,
            "class" => Self::Class,
            "class inductive" => Self::ClassInductive,
            "inductive" => Self::Inductive,
            "instance" => Self::Instance,
            "example" => Self::Example,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdapterMetadata, AdapterOperation, DeclarationKind};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn maps_upstream_axle_declaration_kinds() {
        assert_eq!(
            DeclarationKind::from_axle_kind("theorem"),
            DeclarationKind::Theorem
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("lemma"),
            DeclarationKind::Lemma
        );
        assert_eq!(DeclarationKind::from_axle_kind("def"), DeclarationKind::Def);
        assert_eq!(
            DeclarationKind::from_axle_kind("abbrev"),
            DeclarationKind::Abbrev
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("axiom"),
            DeclarationKind::Axiom
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("opaque"),
            DeclarationKind::Opaque
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("structure"),
            DeclarationKind::Structure
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("class"),
            DeclarationKind::Class
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("class inductive"),
            DeclarationKind::ClassInductive
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("inductive"),
            DeclarationKind::Inductive
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("instance"),
            DeclarationKind::Instance
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("example"),
            DeclarationKind::Example
        );
        assert_eq!(
            DeclarationKind::from_axle_kind("mystery"),
            DeclarationKind::Unknown
        );
    }

    #[test]
    fn serializes_current_adapter_envelope_shape() {
        let mut requests = BTreeMap::new();
        requests.insert(
            "verify_proof".to_owned(),
            json!({ "formal_statement": "theorem foo" }),
        );
        let mut responses = BTreeMap::new();
        responses.insert("verify_proof".to_owned(), json!({ "okay": true }));

        let adapter = AdapterMetadata::verify_proof(requests, responses);
        let value = serde_json::to_value(adapter).unwrap();

        assert_eq!(value["operation"], "verify_proof");
        assert!(value.get("requests").is_some());
        assert!(value.get("responses").is_some());
    }

    #[test]
    fn deserializes_legacy_adapter_shape() {
        let adapter: AdapterMetadata = serde_json::from_value(json!({
            "schema": "axle.adapter.v0",
            "check": { "okay": true },
            "extract_decls": { "documents": {} }
        }))
        .unwrap();

        assert_eq!(adapter.operation, AdapterOperation::Build);
        assert!(adapter.requests.is_empty());
        assert_eq!(adapter.responses["check"]["okay"], json!(true));
        assert_eq!(adapter.responses["extract_decls"]["documents"], json!({}));
    }
}
