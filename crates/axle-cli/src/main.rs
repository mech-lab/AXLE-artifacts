use anyhow::{Context, Result};
use axle_artifact::{ArtifactDirectoryExt, new_artifact, verify_dir};
use axle_client::{
    AxleClient, BuildArtifactContext, CheckRequest, ExtractDeclsRequest, artifact_from_responses,
};
use axle_core::VerificationStatus;
use clap::{Args, CommandFactory, Parser, Subcommand};
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Build(args)) => build(args),
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
            Ok(())
        }
    }
}

fn artifact_new(path: PathBuf) -> Result<()> {
    let artifact = new_artifact();
    artifact
        .save_dir(&path)
        .with_context(|| format!("failed to create artifact at {}", path.display()))?;

    println!("created {}", path.display());
    println!("artifact_id: {}", artifact.artifact_digest()?);
    Ok(())
}

fn build(args: BuildArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let output_path = match args.output {
        Some(output) => output,
        None => default_output_path(&args.input)?,
    };
    let ignore_imports = args.ignore_imports || !args.respect_imports;

    let client = AxleClient::from_env(args.api_url, Option::<String>::None)?;
    let check = client.check(&CheckRequest {
        content: source.clone(),
        environment: args.environment.clone(),
        mathlib_options: args.mathlib_options.then_some(true),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    })?;
    let extract_decls = client.extract_decls(&ExtractDeclsRequest {
        content: source,
        environment: args.environment.clone(),
        ignore_imports: Some(ignore_imports),
        timeout_seconds: args.timeout_seconds,
    })?;

    let artifact = artifact_from_responses(
        BuildArtifactContext {
            source_path: &args.input,
            environment: &args.environment,
        },
        &check,
        &extract_decls,
    )?;
    artifact
        .save_dir(&output_path)
        .with_context(|| format!("failed to write artifact to {}", output_path.display()))?;

    println!("built {}", output_path.display());
    println!("artifact_id: {}", artifact.artifact_digest()?);
    Ok(())
}

fn inspect(path: PathBuf) -> Result<()> {
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

    Ok(())
}

fn verify(path: PathBuf) -> Result<()> {
    let report = verify_dir(&path)
        .with_context(|| format!("failed to verify artifact at {}", path.display()))?;

    if report.is_valid() {
        println!("valid {}", report.artifact_id);
        return Ok(());
    }

    println!("invalid {}", report.artifact_id);
    for error in report.errors {
        println!("error: {error}");
    }

    anyhow::bail!("artifact verification failed");
}

fn hash(path: PathBuf) -> Result<()> {
    let artifact = axle_core::AxleArtifact::load_dir(&path)
        .with_context(|| format!("failed to load artifact from {}", path.display()))?;
    println!("{}", artifact.artifact_digest()?);
    Ok(())
}

fn render_status(status: &VerificationStatus) -> &'static str {
    match status {
        VerificationStatus::Verified => "verified",
        VerificationStatus::Unverified => "unverified",
        VerificationStatus::Failed => "failed",
        VerificationStatus::Unknown => "unknown",
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
