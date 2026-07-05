use axle_hash::Digest;
use serde::{Deserialize, Serialize};

pub const ARTIFACT_SCHEMA_V0: &str = "axle.artifact.v0";
pub const DEFAULT_MANIFEST_FILE: &str = "manifest.json";
pub const DEFAULT_SOURCE_FILE: &str = "source.json";
pub const DEFAULT_DECLARATIONS_FILE: &str = "declarations.json";
pub const DEFAULT_DIAGNOSTICS_FILE: &str = "diagnostics.json";
pub const DEFAULT_HASHES_FILE: &str = "hashes.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxleArtifact {
    pub manifest: Manifest,
    pub source: SourceInfo,
    #[serde(default)]
    pub declarations: Vec<Declaration>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub hashes: HashesFile,
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
}

impl Default for ObjectPaths {
    fn default() -> Self {
        Self {
            source: DEFAULT_SOURCE_FILE.to_owned(),
            declarations: DEFAULT_DECLARATIONS_FILE.to_owned(),
            diagnostics: DEFAULT_DIAGNOSTICS_FILE.to_owned(),
            hashes: DEFAULT_HASHES_FILE.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DeclarationKind {
    Theorem,
    Definition,
    Lemma,
    Axiom,
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
    pub proof_digest: Option<Digest>,
    #[serde(default)]
    pub dependencies: Vec<Digest>,
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
}
