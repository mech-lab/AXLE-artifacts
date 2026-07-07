use anyhow::{Context, Result};
use axle_artifact::{ArtifactDirectoryExt, new_artifact, verify_dir};
use axle_client::{
    AxleClient, BuildArtifactContext, CheckRequest, ExtractDeclsRequest,
    VerifyProofArtifactContext, VerifyProofRequest, build_artifact_from_responses,
    verify_proof_artifact_from_responses,
};
use axle_core::{VerificationResultStatus, VerificationStatus};
use axle_graph::{ArtifactDiff, MerkleGraph};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    bin_name = "axle-rs",
    version,
    about = "Rust-native artifact tooling for AXLE proof outputs"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Build(BuildArgs),
    VerifyProof(VerifyProofArgs),
    Graph(GraphArgs),
    Diff(DiffArgs),
    Artifact {
        #[command(subcommand)]
        command: ArtifactCommand,
    },
    Inspect {
        path: PathBuf,
    },
    Verify {
        path: PathBuf,
    },
    Hash {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum ArtifactCommand {
    New { path: PathBuf },
}

#[derive(Args)]
struct BuildArgs {
    input: PathBuf,
    #[arg(long)]
    environment: String,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long)]
    api_url: Option<String>,
    #[arg(long)]
    timeout_seconds: Option<f64>,
    #[arg(long)]
    mathlib_options: bool,
    #[arg(long, conflicts_with = "respect_imports")]
    ignore_imports: bool,
    #[arg(long, conflicts_with = "ignore_imports")]
    respect_imports: bool,
}

#[derive(Args)]
struct VerifyProofArgs {
    formal_statement: PathBuf,
    content: PathBuf,
    #[arg(long)]
    environment: String,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long)]
    api_url: Option<String>,
    #[arg(long)]
    timeout_seconds: Option<f64>,
    #[arg(long)]
    mathlib_options: bool,
    #[arg(long, conflicts_with = "respect_imports")]
    ignore_imports: bool,
    #[arg(long, conflicts_with = "ignore_imports")]
    respect_imports: bool,
    #[arg(long, value_delimiter = ',')]
    permitted_sorries: Vec<String>,
    #[arg(long)]
    no_use_def_eq: bool,
}

#[derive(Args)]
struct GraphArgs {
    path: PathBuf,
    #[arg(long, default_value = "json")]
    format: GraphFormat,
}

#[derive(Args)]
struct DiffArgs {
    old: PathBuf,
    new: PathBuf,
    #[arg(long, default_value = "text")]
    format: DiffFormat,
}

#[derive(Clone, Debug, ValueEnum)]
enum GraphFormat {
    Json,
    Dot,
}

#[derive(Clone, Debug, ValueEnum)]
enum DiffFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Build(args)) => build(args),
        Some(Command::VerifyProof(args)) => verify_proof(args),
        Some(Command::Graph(args)) => graph(args),
        Some(Command::Diff(args)) => diff(args),
        Some(Command::Artifact { command }) => match command {
            ArtifactCommand::New { path } => artifact_new(path),
        },
        Some(Command::Inspect { path }) => inspect(path),
        Some(Command::Verify { path }) => verify(path),
        Some(Command::Hash { path }) => hash(path),
        None => {
            let mut command = Cli::command();
            command.print_help()?;
            println!();
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn artifact_new(path: PathBuf) -> Result<ExitCode> {
    let artifact = new_artifact();
    artifact
        .save_dir(&path)
        .with_context(|| format!("failed to create artifact at {}", path.display()))?;

    println!("created {}", path.display());
    println!("artifact_id: {}", artifact.artifact_digest()?);
    Ok(ExitCode::SUCCESS)
}

fn build(args: BuildArgs) -> Result<ExitCode> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let output_path = match args.output {
        Some(output) => output,
        None => default_output_path(&args.input)?,
    };
    let ignore_imports = args.ignore_imports || !args.respect_imports;

    let client = AxleClient::from_env(args.api_url, Option::<String>::None)?;
    let check_request = CheckRequest {
        content: source.clone(),
        environment: args.environment.clone(),
        mathlib_options: args.mathlib_options.then_some(true),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    };
    let extract_request = ExtractDeclsRequest {
        content: source,
        environment: args.environment.clone(),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    };
    let check = client.check(&check_request)?;
    let extract_decls = client.extract_decls(&extract_request)?;

    let artifact = build_artifact_from_responses(
        BuildArtifactContext {
            source_path: &args.input,
            environment: &args.environment,
        },
        &serde_json::to_value(&check_request)?,
        &serde_json::to_value(&extract_request)?,
        &check,
        &extract_decls,
    )?;
    artifact
        .save_dir(&output_path)
        .with_context(|| format!("failed to write artifact to {}", output_path.display()))?;

    println!("built {}", output_path.display());
    println!("artifact_id: {}", artifact.artifact_digest()?);
    Ok(ExitCode::SUCCESS)
}

fn verify_proof(args: VerifyProofArgs) -> Result<ExitCode> {
    let formal_statement = fs::read_to_string(&args.formal_statement)
        .with_context(|| format!("failed to read {}", args.formal_statement.display()))?;
    let content = fs::read_to_string(&args.content)
        .with_context(|| format!("failed to read {}", args.content.display()))?;
    let output_path = match args.output {
        Some(output) => output,
        None => default_verify_output_path(&args.content)?,
    };
    let ignore_imports = args.ignore_imports || !args.respect_imports;

    let client = AxleClient::from_env(args.api_url, Option::<String>::None)?;
    let verify_request = VerifyProofRequest {
        formal_statement: formal_statement.clone(),
        content: content.clone(),
        environment: args.environment.clone(),
        permitted_sorries: (!args.permitted_sorries.is_empty()).then_some(args.permitted_sorries),
        mathlib_options: args.mathlib_options.then_some(true),
        use_def_eq: args.no_use_def_eq.then_some(false),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    };
    let extract_request = ExtractDeclsRequest {
        content,
        environment: args.environment.clone(),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    };

    let verify_proof = client.verify_proof(&verify_request)?;
    let extract_decls = client.extract_decls(&extract_request)?;
    let artifact = verify_proof_artifact_from_responses(
        VerifyProofArtifactContext {
            content_path: &args.content,
            environment: &args.environment,
            formal_statement: &formal_statement,
        },
        &serde_json::to_value(&verify_request)?,
        &verify_proof,
        &extract_decls,
    )?;
    artifact
        .save_dir(&output_path)
        .with_context(|| format!("failed to write artifact to {}", output_path.display()))?;

    println!("built {}", output_path.display());
    println!("artifact_id: {}", artifact.artifact_digest()?);

    if verify_proof.value.okay {
        return Ok(ExitCode::SUCCESS);
    }

    println!("verification_status: fail");
    println!(
        "failed_declarations: {}",
        verify_proof.value.failed_declarations.len()
    );
    if !verify_proof.value.failed_declarations.is_empty() {
        println!(
            "failed_declaration_names: {}",
            verify_proof.value.failed_declarations.join(", ")
        );
    }

    Ok(ExitCode::FAILURE)
}

fn inspect(path: PathBuf) -> Result<ExitCode> {
    let artifact = axle_core::AxleArtifact::load_dir(&path)
        .with_context(|| format!("failed to load artifact from {}", path.display()))?;

    let mut status_counts: BTreeMap<VerificationStatus, usize> = BTreeMap::new();
    for declaration in &artifact.declarations {
        *status_counts
            .entry(declaration.verification_status.clone())
            .or_default() += 1;
    }

    println!("schema: {}", artifact.manifest.schema);
    println!(
        "artifact_id: {}",
        artifact
            .manifest
            .artifact_id
            .clone()
            .unwrap_or_else(|| artifact
                .artifact_digest()
                .expect("artifact digest should compute"))
    );
    println!("declarations: {}", artifact.declarations.len());
    println!("diagnostics: {}", artifact.diagnostics.len());
    println!(
        "adapter_metadata: {}",
        if artifact.adapter.is_some() {
            "present"
        } else {
            "absent"
        }
    );

    if status_counts.is_empty() {
        println!("verification_statuses: none");
    } else {
        let summary = status_counts
            .into_iter()
            .map(|(status, count)| format!("{}={count}", render_status(&status)))
            .collect::<Vec<_>>()
            .join(", ");
        println!("verification_statuses: {summary}");
    }

    if let Some(verification) = artifact.verification.as_ref() {
        println!(
            "verification_mode: {}",
            render_verification_mode(verification)
        );
        println!(
            "verification_status: {}",
            render_verification_result_status(&verification.status)
        );
        println!(
            "verification_failed_declarations: {}",
            verification.failed_declarations.len()
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn graph(args: GraphArgs) -> Result<ExitCode> {
    let artifact = load_verified_artifact(&args.path)?;
    let graph = MerkleGraph::derive(&artifact)
        .with_context(|| format!("failed to derive graph from {}", args.path.display()))?;

    match args.format {
        GraphFormat::Json => println!("{}", serde_json::to_string_pretty(&graph)?),
        GraphFormat::Dot => println!("{}", graph.to_dot()),
    }

    Ok(ExitCode::SUCCESS)
}

fn diff(args: DiffArgs) -> Result<ExitCode> {
    let old_artifact = load_verified_artifact(&args.old)?;
    let new_artifact = load_verified_artifact(&args.new)?;
    let old_graph = MerkleGraph::derive(&old_artifact)
        .with_context(|| format!("failed to derive graph from {}", args.old.display()))?;
    let new_graph = MerkleGraph::derive(&new_artifact)
        .with_context(|| format!("failed to derive graph from {}", args.new.display()))?;
    let diff = ArtifactDiff::between(&old_graph, &new_graph).with_context(|| {
        format!(
            "failed to diff {} and {}",
            args.old.display(),
            args.new.display()
        )
    })?;

    match args.format {
        DiffFormat::Text => println!("{}", diff.to_text()),
        DiffFormat::Json => println!("{}", serde_json::to_string_pretty(&diff)?),
    }

    Ok(ExitCode::SUCCESS)
}

fn verify(path: PathBuf) -> Result<ExitCode> {
    let report = verify_dir(&path)
        .with_context(|| format!("failed to verify artifact at {}", path.display()))?;

    if report.is_valid() {
        println!("valid {}", report.artifact_id);
        return Ok(ExitCode::SUCCESS);
    }

    println!("invalid {}", report.artifact_id);
    for error in report.errors {
        println!("error: {error}");
    }

    anyhow::bail!("artifact verification failed");
}

fn hash(path: PathBuf) -> Result<ExitCode> {
    let artifact = axle_core::AxleArtifact::load_dir(&path)
        .with_context(|| format!("failed to load artifact from {}", path.display()))?;
    println!("{}", artifact.artifact_digest()?);
    Ok(ExitCode::SUCCESS)
}

fn load_verified_artifact(path: &Path) -> Result<axle_core::AxleArtifact> {
    let report = verify_dir(path)
        .with_context(|| format!("failed to verify artifact at {}", path.display()))?;
    if !report.is_valid() {
        anyhow::bail!(
            "artifact verification failed for {}: {}",
            path.display(),
            report.errors.join("; ")
        );
    }

    axle_core::AxleArtifact::load_dir(path)
        .with_context(|| format!("failed to load artifact from {}", path.display()))
}

fn render_status(status: &VerificationStatus) -> &'static str {
    match status {
        VerificationStatus::Verified => "verified",
        VerificationStatus::Unverified => "unverified",
        VerificationStatus::Failed => "failed",
        VerificationStatus::Unknown => "unknown",
    }
}

fn render_verification_mode(verification: &axle_core::VerificationSummary) -> &'static str {
    match verification.mode {
        axle_core::VerificationMode::VerifyProof => "verify_proof",
    }
}

fn render_verification_result_status(status: &VerificationResultStatus) -> &'static str {
    match status {
        VerificationResultStatus::Pass => "pass",
        VerificationResultStatus::Fail => "fail",
    }
}

fn default_output_path(input: &Path) -> Result<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("failed to derive output path from {}", input.display()))?;
    let mut output_name = stem.to_os_string();
    output_name.push(".axle");
    Ok(parent.join(output_name))
}

fn default_verify_output_path(input: &Path) -> Result<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("failed to derive output path from {}", input.display()))?;
    let mut output_name = stem.to_os_string();
    output_name.push(".verified.axle");
    Ok(parent.join(output_name))
}
