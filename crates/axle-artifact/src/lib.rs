use anyhow::{Context, Result, bail};
use axle_core::{
    ARTIFACT_SCHEMA_V0, AxleArtifact, DEFAULT_MANIFEST_FILE, Diagnostic, Environment, HashesFile,
    Manifest, ObjectPaths, SourceInfo, SourceSummary,
};
use axle_hash::Digest;
use std::fs;
use std::path::Path;

pub trait ArtifactDirectoryExt {
    fn load_dir<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;

    fn save_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn artifact_digest(&self) -> Result<Digest>;

    fn verify_integrity(&self) -> Result<IntegrityReport>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrityReport {
    pub artifact_id: Digest,
    pub errors: Vec<String>,
}

impl IntegrityReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl ArtifactDirectoryExt for AxleArtifact {
    fn load_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let manifest: Manifest = read_json(path.join(DEFAULT_MANIFEST_FILE))
            .with_context(|| format!("failed to read {}", DEFAULT_MANIFEST_FILE))?;

        let source: SourceInfo = read_json(path.join(&manifest.objects.source))
            .with_context(|| format!("failed to read {}", manifest.objects.source))?;
        let declarations = read_json(path.join(&manifest.objects.declarations))
            .with_context(|| format!("failed to read {}", manifest.objects.declarations))?;
        let diagnostics: Vec<Diagnostic> = read_json(path.join(&manifest.objects.diagnostics))
            .with_context(|| format!("failed to read {}", manifest.objects.diagnostics))?;
        let hashes: HashesFile = read_json(path.join(&manifest.objects.hashes))
            .with_context(|| format!("failed to read {}", manifest.objects.hashes))?;
        let verification =
            read_manifest_optional_json(path, manifest.objects.verification.as_deref())
                .with_context(|| {
                    let file = manifest
                        .objects
                        .verification
                        .as_deref()
                        .unwrap_or(axle_core::DEFAULT_VERIFICATION_FILE);
                    format!("failed to read {}", file)
                })?;
        let adapter = read_optional_json(path, manifest.objects.adapter.as_deref())?;

        Ok(Self {
            manifest,
            source,
            declarations,
            diagnostics,
            hashes,
            verification,
            adapter,
        })
    }

    fn save_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let normalized = normalize_artifact(self)?;
        fs::create_dir_all(path)
            .with_context(|| format!("failed to create artifact directory {}", path.display()))?;

        write_json(path.join(DEFAULT_MANIFEST_FILE), &normalized.manifest)?;
        write_json(
            path.join(&normalized.manifest.objects.source),
            &normalized.source,
        )?;
        write_json(
            path.join(&normalized.manifest.objects.declarations),
            &normalized.declarations,
        )?;
        write_json(
            path.join(&normalized.manifest.objects.diagnostics),
            &normalized.diagnostics,
        )?;
        write_json(
            path.join(&normalized.manifest.objects.hashes),
            &normalized.hashes,
        )?;
        if let (Some(verification_path), Some(verification)) = (
            normalized.manifest.objects.verification.as_deref(),
            normalized.verification.as_ref(),
        ) {
            write_json(path.join(verification_path), verification)?;
        }
        if let (Some(adapter_path), Some(adapter)) = (
            normalized.manifest.objects.adapter.as_deref(),
            normalized.adapter.as_ref(),
        ) {
            write_json(path.join(adapter_path), adapter)?;
        }

        Ok(())
    }

    fn artifact_digest(&self) -> Result<Digest> {
        Ok(normalize_artifact(self)?
            .manifest
            .artifact_id
            .expect("artifact digest is always populated during normalization"))
    }

    fn verify_integrity(&self) -> Result<IntegrityReport> {
        let normalized = normalize_artifact(self)?;
        let normalized_id = normalized
            .manifest
            .artifact_id
            .clone()
            .expect("artifact digest is always populated during normalization");

        let mut errors = Vec::new();

        if self.manifest.schema != ARTIFACT_SCHEMA_V0 {
            errors.push(format!(
                "manifest schema mismatch: expected {}, found {}",
                ARTIFACT_SCHEMA_V0, self.manifest.schema
            ));
        }

        compare_optional_digest(
            "manifest.source.source_digest",
            self.manifest.source.source_digest.as_ref(),
            normalized.manifest.source.source_digest.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "manifest.environment.environment_digest",
            self.manifest.environment.environment_digest.as_ref(),
            normalized.manifest.environment.environment_digest.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "hashes.source",
            self.hashes.source.as_ref(),
            normalized.hashes.source.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "hashes.declarations",
            self.hashes.declarations.as_ref(),
            normalized.hashes.declarations.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "hashes.diagnostics",
            self.hashes.diagnostics.as_ref(),
            normalized.hashes.diagnostics.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "hashes.environment",
            self.hashes.environment.as_ref(),
            normalized.hashes.environment.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "hashes.verification",
            self.hashes.verification.as_ref(),
            normalized.hashes.verification.as_ref(),
            &mut errors,
        );
        compare_optional_digest(
            "manifest.artifact_id",
            self.manifest.artifact_id.as_ref(),
            normalized.manifest.artifact_id.as_ref(),
            &mut errors,
        );

        Ok(IntegrityReport {
            artifact_id: normalized_id,
            errors,
        })
    }
}

pub fn new_artifact() -> AxleArtifact {
    AxleArtifact::new_v0()
}

fn normalize_artifact(artifact: &AxleArtifact) -> Result<AxleArtifact> {
    let mut normalized = artifact.clone();
    normalized.manifest.schema = ARTIFACT_SCHEMA_V0.to_owned();
    normalized.manifest.objects = normalized_object_paths(
        &normalized.manifest.objects,
        normalized.verification.is_some(),
        normalized.adapter.is_some(),
    );
    normalized.manifest.source = SourceSummary {
        language: normalized.source.language.clone(),
        source_digest: None,
    };

    let source_digest = Digest::from_canonical_json(&normalized.source)?;
    let declarations_digest = Digest::from_canonical_json(&normalized.declarations)?;
    let diagnostics_digest = Digest::from_canonical_json(&normalized.diagnostics)?;
    let environment_digest = digest_environment(&normalized.manifest.environment)?;
    let verification_digest = normalized
        .verification
        .as_ref()
        .map(Digest::from_canonical_json)
        .transpose()?;

    normalized.manifest.source.source_digest = Some(source_digest.clone());
    normalized.manifest.environment.environment_digest = Some(environment_digest.clone());
    normalized.hashes = HashesFile {
        source: Some(source_digest),
        declarations: Some(declarations_digest),
        diagnostics: Some(diagnostics_digest),
        environment: Some(environment_digest),
        verification: verification_digest,
    };

    let mut hashed = normalized.clone();
    hashed.adapter = None;
    hashed.manifest.objects.adapter = None;
    hashed.manifest.artifact_id = None;

    let artifact_id = Digest::from_canonical_json(&hashed)?;
    normalized.manifest.artifact_id = Some(artifact_id);

    Ok(normalized)
}

fn digest_environment(environment: &Environment) -> Result<Digest> {
    let mut normalized = environment.clone();
    normalized.environment_digest = None;
    Ok(Digest::from_canonical_json(&normalized)?)
}

fn normalized_object_paths(
    paths: &ObjectPaths,
    has_verification: bool,
    has_adapter: bool,
) -> ObjectPaths {
    ObjectPaths {
        source: non_empty_or_default(&paths.source, axle_core::DEFAULT_SOURCE_FILE.to_owned()),
        declarations: non_empty_or_default(
            &paths.declarations,
            axle_core::DEFAULT_DECLARATIONS_FILE.to_owned(),
        ),
        diagnostics: non_empty_or_default(
            &paths.diagnostics,
            axle_core::DEFAULT_DIAGNOSTICS_FILE.to_owned(),
        ),
        hashes: non_empty_or_default(&paths.hashes, axle_core::DEFAULT_HASHES_FILE.to_owned()),
        verification: has_verification.then(|| {
            non_empty_or_default(
                paths.verification.as_deref().unwrap_or_default(),
                axle_core::DEFAULT_VERIFICATION_FILE.to_owned(),
            )
        }),
        adapter: has_adapter.then(|| {
            non_empty_or_default(
                paths.adapter.as_deref().unwrap_or_default(),
                axle_core::DEFAULT_ADAPTER_FILE.to_owned(),
            )
        }),
    }
}

fn non_empty_or_default(value: &str, default: String) -> String {
    if value.trim().is_empty() {
        default
    } else {
        value.to_owned()
    }
}

fn compare_optional_digest(
    label: &str,
    actual: Option<&Digest>,
    expected: Option<&Digest>,
    errors: &mut Vec<String>,
) {
    match (actual, expected) {
        (Some(actual), Some(expected)) if actual == expected => {}
        (Some(actual), Some(expected)) => errors.push(format!(
            "{label} mismatch: expected {expected}, found {actual}"
        )),
        (None, Some(expected)) => {
            errors.push(format!("{label} missing: expected {expected}"));
        }
        (Some(actual), None) => {
            errors.push(format!("{label} present unexpectedly: found {actual}"));
        }
        (None, None) => {}
    }
}

fn read_json<T, P>(path: P) -> Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse JSON {}", path.display()))?)
}

fn read_optional_json<T>(base: &Path, relative_path: Option<&str>) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let Some(relative_path) = relative_path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(None);
    };

    let full_path = base.join(relative_path);
    if !full_path.exists() {
        return Ok(None);
    }

    read_json(full_path).map(Some)
}

fn read_manifest_optional_json<T>(base: &Path, relative_path: Option<&str>) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let Some(relative_path) = relative_path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(None);
    };

    read_json(base.join(relative_path)).map(Some)
}

fn write_json<T, P>(path: P, value: &T) -> Result<()>
where
    T: serde::Serialize,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let contents = serde_json::to_vec_pretty(value)
        .with_context(|| format!("failed to encode JSON {}", path.display()))?;
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn verify_dir<P: AsRef<Path>>(path: P) -> Result<IntegrityReport> {
    let path = path.as_ref();
    if !path.is_dir() {
        bail!("artifact path is not a directory: {}", path.display());
    }

    let artifact = AxleArtifact::load_dir(path)?;
    artifact.verify_integrity()
}

#[cfg(test)]
mod tests {
    use super::{ArtifactDirectoryExt, new_artifact};
    use axle_core::{
        AdapterMetadata, Declaration, DeclarationKind, VerificationMode, VerificationResultStatus,
        VerificationStatus, VerificationSummary,
    };
    use axle_hash::Digest;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn round_trip_preserves_integrity() {
        let temp = tempdir().unwrap();
        let artifact_path = temp.path().join("example.axle");

        let mut artifact = new_artifact();
        artifact.source.module = Some("Example".to_owned());
        artifact.source.path = Some("Example.lean".to_owned());
        artifact.source.source_text = Some("theorem foo : True := trivial".to_owned());
        artifact.declarations.push(Declaration {
            name: "Example.foo".to_owned(),
            kind: DeclarationKind::Theorem,
            statement_digest: Some(Digest::sha256("True")),
            body_digest: Some(Digest::sha256("trivial")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        });

        artifact.save_dir(&artifact_path).unwrap();
        let loaded = axle_core::AxleArtifact::load_dir(&artifact_path).unwrap();
        let report = loaded.verify_integrity().unwrap();

        assert!(report.is_valid(), "{:?}", report.errors);
    }

    #[test]
    fn changing_declarations_changes_artifact_digest() {
        let mut artifact = new_artifact();
        let first = artifact.artifact_digest().unwrap();

        artifact.declarations.push(Declaration {
            name: "Example.foo".to_owned(),
            kind: DeclarationKind::Theorem,
            statement_digest: Some(Digest::sha256("A")),
            body_digest: Some(Digest::sha256("B")),
            dependencies: Vec::new(),
            verification_status: VerificationStatus::Verified,
        });

        let second = artifact.artifact_digest().unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn adapter_metadata_does_not_change_artifact_digest() {
        let mut artifact = new_artifact();
        let mut requests = BTreeMap::new();
        requests.insert("check".to_owned(), json!({ "content": "theorem foo" }));
        let mut responses = BTreeMap::new();
        responses.insert("check".to_owned(), json!({ "timings": { "total_ms": 10 } }));
        responses.insert(
            "extract_decls".to_owned(),
            json!({ "timings": { "total_ms": 20 } }),
        );
        artifact.adapter = Some(AdapterMetadata::build(requests.clone(), responses));

        let first = artifact.artifact_digest().unwrap();

        let mut responses = BTreeMap::new();
        responses.insert(
            "check".to_owned(),
            json!({ "timings": { "total_ms": 999 } }),
        );
        responses.insert(
            "extract_decls".to_owned(),
            json!({ "timings": { "total_ms": 1000 } }),
        );
        artifact.adapter = Some(AdapterMetadata::build(requests, responses));

        let second = artifact.artifact_digest().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn missing_adapter_file_does_not_break_core_verification() {
        let temp = tempdir().unwrap();
        let artifact_path = temp.path().join("example.axle");

        let mut artifact = new_artifact();
        let mut responses = BTreeMap::new();
        responses.insert("check".to_owned(), json!({ "okay": true }));
        responses.insert("extract_decls".to_owned(), json!({ "documents": {} }));
        artifact.adapter = Some(AdapterMetadata::build(BTreeMap::new(), responses));
        artifact.save_dir(&artifact_path).unwrap();

        fs::remove_file(artifact_path.join("adapter.json")).unwrap();

        let loaded = axle_core::AxleArtifact::load_dir(&artifact_path).unwrap();
        assert!(loaded.adapter.is_none());

        let report = loaded.verify_integrity().unwrap();
        assert!(report.is_valid(), "{:?}", report.errors);
    }

    #[test]
    fn verification_summary_changes_artifact_digest() {
        let mut artifact = new_artifact();
        let first = artifact.artifact_digest().unwrap();

        artifact.verification = Some(VerificationSummary {
            mode: VerificationMode::VerifyProof,
            status: VerificationResultStatus::Pass,
            formal_statement_digest: Digest::sha256("theorem foo : True"),
            failed_declarations: Vec::new(),
        });

        let second = artifact.artifact_digest().unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn verification_file_is_part_of_integrity_validation() {
        let temp = tempdir().unwrap();
        let artifact_path = temp.path().join("verified.axle");

        let mut artifact = new_artifact();
        artifact.verification = Some(VerificationSummary {
            mode: VerificationMode::VerifyProof,
            status: VerificationResultStatus::Pass,
            formal_statement_digest: Digest::sha256("theorem foo : True"),
            failed_declarations: Vec::new(),
        });
        artifact.save_dir(&artifact_path).unwrap();

        fs::write(
            artifact_path.join("verification.json"),
            serde_json::to_vec_pretty(&json!({
                "mode": "verify_proof",
                "status": "fail",
                "formal_statement_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "failed_declarations": ["foo"]
            }))
            .unwrap(),
        )
        .unwrap();

        let loaded = axle_core::AxleArtifact::load_dir(&artifact_path).unwrap();
        let report = loaded.verify_integrity().unwrap();

        assert!(!report.is_valid());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("hashes.verification mismatch"))
        );
    }

    #[test]
    fn legacy_adapter_shape_remains_loadable() {
        let temp = tempdir().unwrap();
        let artifact_path = temp.path().join("legacy.axle");

        let mut artifact = new_artifact();
        let mut responses = BTreeMap::new();
        responses.insert("check".to_owned(), json!({ "okay": true }));
        responses.insert("extract_decls".to_owned(), json!({ "documents": {} }));
        artifact.adapter = Some(AdapterMetadata::build(BTreeMap::new(), responses));
        artifact.save_dir(&artifact_path).unwrap();

        fs::write(
            artifact_path.join("adapter.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "axle.adapter.v0",
                "check": { "okay": true },
                "extract_decls": { "documents": {} }
            }))
            .unwrap(),
        )
        .unwrap();

        let loaded = axle_core::AxleArtifact::load_dir(&artifact_path).unwrap();
        let report = loaded.verify_integrity().unwrap();

        assert!(report.is_valid(), "{:?}", report.errors);
        let adapter = loaded.adapter.expect("legacy adapter should parse");
        assert_eq!(adapter.operation, axle_core::AdapterOperation::Build);
        assert_eq!(adapter.responses["check"]["okay"], json!(true));
    }
}
