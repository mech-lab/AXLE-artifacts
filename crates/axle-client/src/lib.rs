use axle_core::{
    AdapterMetadata, AxleArtifact, Declaration, DeclarationKind, Diagnostic, DiagnosticLevel,
    SourceInfo, VerificationStatus,
};
use axle_hash::Digest;
use reqwest::blocking::Client as HttpClient;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::Path;
use thiserror::Error;

pub const DEFAULT_AXLE_API_URL: &str = "https://axle.axiommath.ai";

#[derive(Debug)]
pub struct AxleClient {
    base_url: String,
    api_key: Option<String>,
    http: HttpClient,
}

impl AxleClient {
    pub fn from_env(
        api_url_override: Option<impl Into<String>>,
        api_key_override: Option<impl Into<String>>,
    ) -> Result<Self, AxleClientError> {
        let base_url = api_url_override
            .map(Into::into)
            .or_else(|| env::var("AXLE_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_AXLE_API_URL.to_owned())
            .trim_end_matches('/')
            .to_owned();
        let api_key = api_key_override
            .map(Into::into)
            .or_else(|| env::var("AXLE_API_KEY").ok());

        Ok(Self {
            base_url,
            api_key,
            http: HttpClient::builder().build()?,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn check(
        &self,
        request: &CheckRequest,
    ) -> Result<ParsedResponse<CheckResponse>, AxleClientError> {
        self.post("check", request, request.timeout_seconds)
    }

    pub fn extract_decls(
        &self,
        request: &ExtractDeclsRequest,
    ) -> Result<ParsedResponse<ExtractDeclsResponse>, AxleClientError> {
        self.post("extract_decls", request, request.timeout_seconds)
    }

    fn post<T, R>(
        &self,
        method: &str,
        request: &R,
        timeout_seconds: Option<f64>,
    ) -> Result<ParsedResponse<T>, AxleClientError>
    where
        T: DeserializeOwned,
        R: Serialize,
    {
        let url = format!("{}/api/v1/{method}", self.base_url);
        let mut builder = self
            .http
            .post(url.clone())
            .header("X-Request-Source", "axle-rs")
            .json(request);

        if let Some(api_key) = &self.api_key {
            builder = builder.bearer_auth(api_key);
        }

        if let Some(timeout_seconds) = timeout_seconds {
            builder = builder.timeout(std::time::Duration::from_secs_f64(timeout_seconds + 30.0));
        }

        let response = builder.send()?;
        let status = response.status();
        let body = response.text()?;

        if !status.is_success() {
            return Err(AxleClientError::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }

        let raw: Value = serde_json::from_str(&body)?;
        if let Some(message) = raw.get("internal_error").and_then(Value::as_str) {
            return Err(AxleClientError::Internal(message.to_owned()));
        }
        if let Some(message) = raw.get("user_error").and_then(Value::as_str) {
            return Err(AxleClientError::User(message.to_owned()));
        }
        if let Some(message) = raw.get("error").and_then(Value::as_str) {
            return Err(AxleClientError::Runtime(message.to_owned()));
        }

        let value = serde_json::from_value(raw.clone())?;
        Ok(ParsedResponse { value, raw })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedResponse<T> {
    pub value: T,
    pub raw: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CheckRequest {
    pub content: String,
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mathlib_options: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_imports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExtractDeclsRequest {
    pub content: String,
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_imports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Messages {
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub infos: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckResponse {
    #[serde(default)]
    pub okay: bool,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub lean_messages: Messages,
    #[serde(default)]
    pub tool_messages: Messages,
    #[serde(default)]
    pub failed_declarations: Vec<String>,
    #[serde(default)]
    pub timings: BTreeMap<String, u64>,
    #[serde(default)]
    pub info: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractDeclsResponse {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub lean_messages: Messages,
    #[serde(default)]
    pub tool_messages: Messages,
    #[serde(default)]
    pub documents: BTreeMap<String, Document>,
    #[serde(default)]
    pub timings: BTreeMap<String, u64>,
    #[serde(default)]
    pub info: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub declaration: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tokens: Vec<String>,
    #[serde(default)]
    pub signature: String,
    #[serde(rename = "type", default)]
    pub type_text: String,
    #[serde(default)]
    pub type_hash: i64,
    #[serde(default)]
    pub type_depth: u64,
    #[serde(default)]
    pub term_depth: u64,
    #[serde(default)]
    pub is_sorry: bool,
    #[serde(default)]
    pub index: i64,
    #[serde(default)]
    pub line_pos: u64,
    #[serde(default)]
    pub end_line_pos: u64,
    #[serde(default)]
    pub proof_length: u64,
    #[serde(default)]
    pub tactic_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub wall_ms: u64,
    #[serde(default)]
    pub heartbeats: u64,
    #[serde(default)]
    pub local_type_dependencies: Vec<String>,
    #[serde(default)]
    pub local_value_dependencies: Vec<String>,
    #[serde(default)]
    pub external_type_dependencies: Vec<String>,
    #[serde(default)]
    pub external_value_dependencies: Vec<String>,
    #[serde(default)]
    pub local_syntactic_dependencies: Vec<String>,
    #[serde(default)]
    pub external_syntactic_dependencies: Vec<String>,
    #[serde(default)]
    pub declaration_messages: Messages,
    #[serde(default)]
    pub theorem_messages: Messages,
}

pub struct BuildArtifactContext<'a> {
    pub source_path: &'a Path,
    pub environment: &'a str,
}

pub fn artifact_from_responses(
    context: BuildArtifactContext<'_>,
    check: &ParsedResponse<CheckResponse>,
    extract_decls: &ParsedResponse<ExtractDeclsResponse>,
) -> Result<AxleArtifact, AxleAdapterError> {
    if check.value.content != extract_decls.value.content {
        return Err(AxleAdapterError::ContentMismatch);
    }

    let failed_declarations: BTreeSet<_> =
        check.value.failed_declarations.iter().cloned().collect();
    let mut artifact = AxleArtifact::new_v0();
    artifact.source = SourceInfo {
        language: "lean4".to_owned(),
        module: context
            .source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(ToOwned::to_owned),
        path: Some(context.source_path.display().to_string()),
        source_text: Some(check.value.content.clone()),
    };
    artifact.manifest.environment.lean_version = Some(context.environment.to_owned());
    artifact.declarations =
        declarations_from_documents(&extract_decls.value.documents, &failed_declarations)?;
    artifact.diagnostics = collect_diagnostics(check, extract_decls);
    artifact.adapter = Some(AdapterMetadata::new(
        check.raw.clone(),
        extract_decls.raw.clone(),
    ));

    Ok(artifact)
}

fn declarations_from_documents(
    documents: &BTreeMap<String, Document>,
    failed_declarations: &BTreeSet<String>,
) -> Result<Vec<Declaration>, AxleAdapterError> {
    let mut ordered: Vec<_> = documents.iter().collect();
    ordered.sort_by(|(left_name, left), (right_name, right)| {
        left.index
            .cmp(&right.index)
            .then_with(|| left_name.cmp(right_name))
    });

    ordered
        .into_iter()
        .map(|(name, document)| {
            Ok(Declaration {
                name: name.clone(),
                kind: DeclarationKind::from_axle_kind(&document.kind),
                statement_digest: Some(Digest::from_canonical_json(&document.type_text)?),
                body_digest: Some(Digest::from_canonical_json(&document.declaration)?),
                dependencies: local_dependencies(document),
                verification_status: if failed_declarations.contains(name) {
                    VerificationStatus::Failed
                } else {
                    VerificationStatus::Verified
                },
            })
        })
        .collect()
}

fn local_dependencies(document: &Document) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.extend(document.local_type_dependencies.iter().cloned());
    names.extend(document.local_value_dependencies.iter().cloned());
    names.extend(document.local_syntactic_dependencies.iter().cloned());
    names.into_iter().collect()
}

fn collect_diagnostics(
    check: &ParsedResponse<CheckResponse>,
    extract_decls: &ParsedResponse<ExtractDeclsResponse>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    push_messages(&mut diagnostics, "check.lean", &check.value.lean_messages);
    push_messages(&mut diagnostics, "check.tool", &check.value.tool_messages);
    push_messages(
        &mut diagnostics,
        "extract_decls.lean",
        &extract_decls.value.lean_messages,
    );
    push_messages(
        &mut diagnostics,
        "extract_decls.tool",
        &extract_decls.value.tool_messages,
    );
    diagnostics
}

fn push_messages(diagnostics: &mut Vec<Diagnostic>, code: &str, messages: &Messages) {
    diagnostics.extend(messages.errors.iter().cloned().map(|message| Diagnostic {
        level: DiagnosticLevel::Error,
        message,
        code: Some(code.to_owned()),
    }));
    diagnostics.extend(messages.warnings.iter().cloned().map(|message| Diagnostic {
        level: DiagnosticLevel::Warning,
        message,
        code: Some(code.to_owned()),
    }));
    diagnostics.extend(messages.infos.iter().cloned().map(|message| Diagnostic {
        level: DiagnosticLevel::Info,
        message,
        code: Some(code.to_owned()),
    }));
}

#[derive(Debug, Error)]
pub enum AxleClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("AXLE returned HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },
    #[error("AXLE internal error: {0}")]
    Internal(String),
    #[error("AXLE user error: {0}")]
    User(String),
    #[error("AXLE runtime error: {0}")]
    Runtime(String),
    #[error("failed to parse AXLE JSON response: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum AxleAdapterError {
    #[error("processed content mismatch between AXLE check and extract_decls responses")]
    ContentMismatch,
    #[error("failed to digest AXLE declaration content: {0}")]
    Digest(#[from] axle_hash::HashError),
}

#[cfg(test)]
mod tests {
    use super::{
        BuildArtifactContext, CheckResponse, ExtractDeclsResponse, ParsedResponse,
        artifact_from_responses,
    };
    use axle_core::DeclarationKind;
    use serde_json::Value;
    use std::path::Path;

    fn fixture(name: &str) -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("workspace root");
        std::fs::read_to_string(root.join("tests/fixtures").join(name)).unwrap()
    }

    #[test]
    fn parses_check_response_fixture() {
        let response: CheckResponse =
            serde_json::from_str(&fixture("check_response.json")).unwrap();

        assert!(response.okay);
        assert_eq!(response.failed_declarations, vec!["sample.example_fail"]);
        assert!(response.content.contains("theorem sample.bar"));
    }

    #[test]
    fn parses_extract_decls_response_fixture() {
        let response: ExtractDeclsResponse =
            serde_json::from_str(&fixture("extract_decls_response.json")).unwrap();

        assert_eq!(response.documents.len(), 4);
        assert_eq!(response.documents["sample.foo"].kind, "def");
        assert_eq!(
            response.documents["sample.bar"].local_value_dependencies,
            vec!["sample.foo"]
        );
    }

    #[test]
    fn converts_axle_responses_into_artifact() {
        let check_value: CheckResponse =
            serde_json::from_str(&fixture("check_response.json")).unwrap();
        let extract_value: ExtractDeclsResponse =
            serde_json::from_str(&fixture("extract_decls_response.json")).unwrap();

        let check = ParsedResponse {
            raw: serde_json::from_str::<Value>(&fixture("check_response.json")).unwrap(),
            value: check_value,
        };
        let extract = ParsedResponse {
            raw: serde_json::from_str::<Value>(&fixture("extract_decls_response.json")).unwrap(),
            value: extract_value,
        };

        let artifact = artifact_from_responses(
            BuildArtifactContext {
                source_path: Path::new("sample.lean"),
                environment: "lean-4.28.0",
            },
            &check,
            &extract,
        )
        .unwrap();

        assert_eq!(artifact.source.path.as_deref(), Some("sample.lean"));
        assert_eq!(
            artifact
                .declarations
                .iter()
                .map(|decl| (&decl.name, &decl.kind, &decl.verification_status))
                .collect::<Vec<_>>(),
            vec![
                (
                    &"sample.foo".to_owned(),
                    &DeclarationKind::Def,
                    &axle_core::VerificationStatus::Verified
                ),
                (
                    &"sample.bar".to_owned(),
                    &DeclarationKind::Theorem,
                    &axle_core::VerificationStatus::Verified
                ),
                (
                    &"sample.instNat".to_owned(),
                    &DeclarationKind::Instance,
                    &axle_core::VerificationStatus::Verified
                ),
                (
                    &"sample.example_fail".to_owned(),
                    &DeclarationKind::Example,
                    &axle_core::VerificationStatus::Failed
                ),
            ]
        );
        assert_eq!(
            artifact.declarations[1].dependencies,
            vec!["sample.foo".to_owned()]
        );
        assert!(artifact.adapter.is_some());
    }
}
