use axle_hash::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ARTIFACT_SCHEMA_V0: &str = "axle.artifact.v0";
pub const DEFAULT_MANIFEST_FILE: &str = "manifest.json";
pub const DEFAULT_SOURCE_FILE: &str = "source.json";
pub const DEFAULT_DECLARATIONS_FILE: &str = "declarations.json";
pub const DEFAULT_DIAGNOSTICS_FILE: &str = "diagnostics.json";
pub const DEFAULT_HASHES_FILE: &str = "hashes.json";
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
    pub adapter: Option<String>,
}

impl Default for ObjectPaths {
    fn default() -> Self {
        Self {
            source: DEFAULT_SOURCE_FILE.to_owned(),
            declarations: DEFAULT_DECLARATIONS_FILE.to_owned(),
            diagnostics: DEFAULT_DIAGNOSTICS_FILE.to_owned(),
            hashes: DEFAULT_HASHES_FILE.to_owned(),
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdapterMetadata {
    pub schema: String,
    pub check: Value,
    pub extract_decls: Value,
}

impl AdapterMetadata {
    pub fn new(check: Value, extract_decls: Value) -> Self {
        Self {
            schema: ADAPTER_SCHEMA_V0.to_owned(),
            check,
            extract_decls,
        }
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
    use super::DeclarationKind;

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
}
