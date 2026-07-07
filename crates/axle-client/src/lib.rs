use axle_core::{
    AdapterMetadata, AxleArtifact, Declaration, DeclarationKind, Diagnostic, DiagnosticLevel,
    SourceInfo, VerificationMode, VerificationResultStatus, VerificationStatus,
    VerificationSummary,
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

    pub fn verify_proof(
        &self,
        request: &VerifyProofRequest,
    ) -> Result<ParsedResponse<VerifyProofResponse>, AxleClientError> {
        self.post("verify_proof", request, request.timeout_seconds)
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VerifyProofRequest {
    pub formal_statement: String,
    pub content: String,
    pub environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permitted_sorries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mathlib_options: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_def_eq: Option<bool>,
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
pub struct VerifyProofResponse {
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

pub struct VerifyProofArtifactContext<'a> {
    pub content_path: &'a Path,
    pub environment: &'a str,
    pub formal_statement: &'a str,
}

pub fn artifact_from_responses(
    context: BuildArtifactContext<'_>,
    check: &ParsedResponse<CheckResponse>,
    extract_decls: &ParsedResponse<ExtractDeclsResponse>,
) -> Result<AxleArtifact, AxleAdapterError> {
    let environment = context.environment.to_owned();
    let check_request = serde_json::to_value(CheckRequest {
        content: check.value.content.clone(),
        environment: environment.clone(),
        mathlib_options: None,
        ignore_imports: None,
        timeout_seconds: None,
    })
    .expect("serializing synthetic check request should not fail");
    let extract_request = serde_json::to_value(ExtractDeclsRequest {
        content: extract_decls.value.content.clone(),
        environment,
        ignore_imports: None,
        timeout_seconds: None,
    })
    .expect("serializing synthetic extract request should not fail");

    build_artifact_from_responses(
        context,
        &check_request,
        &extract_request,
        check,
        extract_decls,
    )
}

pub fn build_artifact_from_responses(
    context: BuildArtifactContext<'_>,
    check_request: &Value,
    extract_decls_request: &Value,
    check: &ParsedResponse<CheckResponse>,
    extract_decls: &ParsedResponse<ExtractDeclsResponse>,
) -> Result<AxleArtifact, AxleAdapterError> {
    let failed_declarations: BTreeSet<_> =
        check.value.failed_declarations.iter().cloned().collect();
    ensure_processed_content_match(
        "check",
        &check.value.content,
        "extract_decls",
        &extract_decls.value.content,
    )?;

    let mut requests = BTreeMap::new();
    requests.insert("check".to_owned(), check_request.clone());
    requests.insert("extract_decls".to_owned(), extract_decls_request.clone());

    let mut responses = BTreeMap::new();
    responses.insert("check".to_owned(), check.raw.clone());
    responses.insert("extract_decls".to_owned(), extract_decls.raw.clone());

    artifact_from_documents(
        context.source_path,
        context.environment,
        check.value.content.clone(),
        &extract_decls.value.documents,
        &failed_declarations,
        collect_response_diagnostics(&[
            ("check.lean", &check.value.lean_messages),
            ("check.tool", &check.value.tool_messages),
            ("extract_decls.lean", &extract_decls.value.lean_messages),
            ("extract_decls.tool", &extract_decls.value.tool_messages),
        ]),
        None,
        Some(AdapterMetadata::build(requests, responses)),
    )
}

pub fn verify_proof_artifact_from_responses(
    context: VerifyProofArtifactContext<'_>,
    verify_proof_request: &Value,
    verify_proof: &ParsedResponse<VerifyProofResponse>,
    extract_decls: &ParsedResponse<ExtractDeclsResponse>,
) -> Result<AxleArtifact, AxleAdapterError> {
    let failed_declarations: BTreeSet<_> = verify_proof
        .value
        .failed_declarations
        .iter()
        .cloned()
        .collect();
    ensure_processed_content_match(
        "verify_proof",
        &verify_proof.value.content,
        "extract_decls",
        &extract_decls.value.content,
    )?;

    let mut requests = BTreeMap::new();
    requests.insert("verify_proof".to_owned(), verify_proof_request.clone());

    let mut responses = BTreeMap::new();
    responses.insert("verify_proof".to_owned(), verify_proof.raw.clone());
    responses.insert("extract_decls".to_owned(), extract_decls.raw.clone());

    let mut failed_names = verify_proof.value.failed_declarations.clone();
    failed_names.sort();
    failed_names.dedup();

    artifact_from_documents(
        context.content_path,
        context.environment,
        verify_proof.value.content.clone(),
        &extract_decls.value.documents,
        &failed_declarations,
        collect_response_diagnostics(&[
            ("verify_proof.lean", &verify_proof.value.lean_messages),
            ("verify_proof.tool", &verify_proof.value.tool_messages),
            ("extract_decls.lean", &extract_decls.value.lean_messages),
            ("extract_decls.tool", &extract_decls.value.tool_messages),
        ]),
        Some(VerificationSummary {
            mode: VerificationMode::VerifyProof,
            status: if verify_proof.value.okay {
                VerificationResultStatus::Pass
            } else {
                VerificationResultStatus::Fail
            },
            formal_statement_digest: Digest::from_canonical_json(&context.formal_statement)?,
            failed_declarations: failed_names,
        }),
        Some(AdapterMetadata::verify_proof(requests, responses)),
    )
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

fn artifact_from_documents(
    source_path: &Path,
    environment: &str,
    source_text: String,
    documents: &BTreeMap<String, Document>,
    failed_declarations: &BTreeSet<String>,
    diagnostics: Vec<Diagnostic>,
    verification: Option<VerificationSummary>,
    adapter: Option<AdapterMetadata>,
) -> Result<AxleArtifact, AxleAdapterError> {
    let mut artifact = AxleArtifact::new_v0();
    artifact.source = SourceInfo {
        language: "lean4".to_owned(),
        module: source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(ToOwned::to_owned),
        path: Some(source_path.display().to_string()),
        source_text: Some(source_text),
    };
    artifact.manifest.environment.lean_version = Some(environment.to_owned());
    artifact.declarations = declarations_from_documents(documents, failed_declarations)?;
    artifact.diagnostics = diagnostics;
    artifact.verification = verification;
    artifact.adapter = adapter;

    Ok(artifact)
}

fn local_dependencies(document: &Document) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.extend(document.local_type_dependencies.iter().cloned());
    names.extend(document.local_value_dependencies.iter().cloned());
    names.extend(document.local_syntactic_dependencies.iter().cloned());
    names.into_iter().collect()
}

fn collect_response_diagnostics(entries: &[(&str, &Messages)]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for &(code, messages) in entries {
        push_messages(&mut diagnostics, code, messages);
    }
    diagnostics
}

fn ensure_processed_content_match(
    left_label: &'static str,
    left_content: &str,
    right_label: &'static str,
    right_content: &str,
) -> Result<(), AxleAdapterError> {
    if left_content == right_content {
        return Ok(());
    }

    Err(AxleAdapterError::ContentMismatch {
        left: left_label,
        right: right_label,
    })
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
    #[error("processed content mismatch between AXLE {left} and {right} responses")]
    ContentMismatch {
        left: &'static str,
        right: &'static str,
    },
    #[error("failed to digest AXLE declaration content: {0}")]
    Digest(#[from] axle_hash::HashError),
    #[error("failed to serialize AXLE adapter request payload: {0}")]
    RequestEncoding(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::{
        BuildArtifactContext, CheckRequest, CheckResponse, ExtractDeclsRequest,
        ExtractDeclsResponse, ParsedResponse, VerifyProofArtifactContext, VerifyProofRequest,
        VerifyProofResponse, build_artifact_from_responses, verify_proof_artifact_from_responses,
    };
    use axle_core::{
        DeclarationKind, VerificationMode, VerificationResultStatus, VerificationStatus,
    };
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};
    use std::path::Path;

    fn fixture(name: &str) -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("workspace root");
        std::fs::read_to_string(root.join("tests/fixtures").join(name)).unwrap()
    }

    fn parsed_response<T>(name: &str) -> ParsedResponse<T>
    where
        T: DeserializeOwned,
    {
        ParsedResponse {
            raw: serde_json::from_str::<Value>(&fixture(name)).unwrap(),
            value: serde_json::from_str(&fixture(name)).unwrap(),
        }
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
    fn parses_verify_proof_pass_response_fixture() {
        let response: VerifyProofResponse =
            serde_json::from_str(&fixture("verify_proof_pass_response.json")).unwrap();

        assert!(response.okay);
        assert!(response.failed_declarations.is_empty());
        assert!(response.content.contains("theorem sample.bar"));
    }

    #[test]
    fn parses_verify_proof_fail_response_fixture() {
        let response: VerifyProofResponse =
            serde_json::from_str(&fixture("verify_proof_fail_response.json")).unwrap();

        assert!(!response.okay);
        assert_eq!(response.failed_declarations, vec!["sample.example_fail"]);
    }

    #[test]
    fn converts_build_responses_into_artifact() {
        let check = parsed_response::<CheckResponse>("check_response.json");
        let extract = parsed_response::<ExtractDeclsResponse>("extract_decls_response.json");
        let check_request = serde_json::to_value(CheckRequest {
            content: check.value.content.clone(),
            environment: "lean-4.28.0".to_owned(),
            mathlib_options: Some(true),
            ignore_imports: Some(true),
            timeout_seconds: Some(12.0),
        })
        .unwrap();
        let extract_request = serde_json::to_value(ExtractDeclsRequest {
            content: check.value.content.clone(),
            environment: "lean-4.28.0".to_owned(),
            ignore_imports: Some(true),
            timeout_seconds: Some(12.0),
        })
        .unwrap();

        let artifact = build_artifact_from_responses(
            BuildArtifactContext {
                source_path: Path::new("sample.lean"),
                environment: "lean-4.28.0",
            },
            &check_request,
            &extract_request,
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
        assert!(artifact.verification.is_none());
        assert_eq!(
            artifact.adapter.as_ref().unwrap().responses["check"]["okay"],
            json!(true)
        );
        assert_eq!(
            artifact.adapter.as_ref().unwrap().requests["check"]["environment"],
            json!("lean-4.28.0")
        );
    }

    #[test]
    fn converts_verify_proof_responses_into_artifact() {
        let verify = parsed_response::<VerifyProofResponse>("verify_proof_fail_response.json");
        let extract = parsed_response::<ExtractDeclsResponse>("extract_decls_response.json");
        let verify_request = serde_json::to_value(VerifyProofRequest {
            formal_statement: "theorem sample.bar : sample.foo = 1".to_owned(),
            content: verify.value.content.clone(),
            environment: "lean-4.28.0".to_owned(),
            permitted_sorries: Some(vec!["sample.helper".to_owned()]),
            mathlib_options: Some(true),
            use_def_eq: Some(false),
            ignore_imports: Some(true),
            timeout_seconds: Some(30.0),
        })
        .unwrap();

        let artifact = verify_proof_artifact_from_responses(
            VerifyProofArtifactContext {
                content_path: Path::new("proof.lean"),
                environment: "lean-4.28.0",
                formal_statement: "theorem sample.bar : sample.foo = 1",
            },
            &verify_request,
            &verify,
            &extract,
        )
        .unwrap();

        assert_eq!(artifact.source.path.as_deref(), Some("proof.lean"));
        assert_eq!(
            artifact
                .verification
                .as_ref()
                .expect("verification summary should be present")
                .mode,
            VerificationMode::VerifyProof
        );
        assert_eq!(
            artifact.verification.as_ref().unwrap().status,
            VerificationResultStatus::Fail
        );
        assert_eq!(
            artifact.verification.as_ref().unwrap().failed_declarations,
            vec!["sample.example_fail".to_owned()]
        );
        assert_eq!(
            artifact
                .declarations
                .iter()
                .find(|decl| decl.name == "sample.example_fail")
                .unwrap()
                .verification_status,
            VerificationStatus::Failed
        );
        assert_eq!(
            artifact.adapter.as_ref().unwrap().operation,
            axle_core::AdapterOperation::VerifyProof
        );
        assert_eq!(
            artifact.adapter.as_ref().unwrap().requests["verify_proof"]["formal_statement"],
            json!("theorem sample.bar : sample.foo = 1")
        );
    }

    #[test]
    fn preserves_failed_declarations_missing_from_extracted_documents() {
        let verify =
            parsed_response::<VerifyProofResponse>("verify_proof_missing_decl_response.json");
        let extract = parsed_response::<ExtractDeclsResponse>("extract_decls_response.json");
        let verify_request = serde_json::to_value(VerifyProofRequest {
            formal_statement: "theorem sample.main : True".to_owned(),
            content: verify.value.content.clone(),
            environment: "lean-4.28.0".to_owned(),
            permitted_sorries: None,
            mathlib_options: None,
            use_def_eq: None,
            ignore_imports: Some(true),
            timeout_seconds: None,
        })
        .unwrap();

        let artifact = verify_proof_artifact_from_responses(
            VerifyProofArtifactContext {
                content_path: Path::new("proof.lean"),
                environment: "lean-4.28.0",
                formal_statement: "theorem sample.main : True",
            },
            &verify_request,
            &verify,
            &extract,
        )
        .unwrap();

        assert_eq!(
            artifact.verification.as_ref().unwrap().failed_declarations,
            vec![
                "sample.example_fail".to_owned(),
                "sample.missing_theorem".to_owned()
            ]
        );
        assert!(
            artifact
                .declarations
                .iter()
                .all(|decl| decl.name != "sample.missing_theorem")
        );
    }
}
