use anyhow::{Context, Result};
use axle_artifact::{ArtifactDirectoryExt, new_artifact, verify_dir};
use axle_core::VerificationStatus;
use clap::{CommandFactory, Parser, Subcommand};
use std::collections::BTreeMap;
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
