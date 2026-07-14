use anyhow::{Context, Result};
use axle_hash::Digest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Flight event types for recording artifact processing
#[derive(Debug, Serialize, Deserialize)]
pub enum FlightEvent {
    /// Artifact created or updated
    ArtifactCreated(Digest),
    /// Receipt issued for artifact
    ReceiptIssued(Digest),
    /// Artifact verified
    ArtifactVerified(Digest),
    /// Receipt verified
    ReceiptVerified(Digest),
    /// Build command executed
    BuildCommand(PathBuf),
    /// Verify proof command executed
    VerifyProofCommand(PathBuf, PathBuf),
    /// Inspect command executed
    InspectCommand(PathBuf),
    /// Verify command executed
    VerifyCommand(PathBuf),
    /// Hash command executed
    HashCommand(PathBuf),
}

/// Recorder state tracking current recording session
pub struct RecorderState {
    /// Path to log file
    pub log_path: PathBuf,
    /// Current event sequence
    pub events: Vec<FlightEvent>,
}

/// Initialize recorder with log path
pub fn new_recorder(log_path: PathBuf) -> RecorderState {
    RecorderState {
        log_path,
        events: Vec::new(),
    }
}

/// Record event to log
pub fn record_event(recorder: &mut RecorderState, event: FlightEvent) {
    recorder.events.push(event);
}

/// Finalize recording and save to file
pub fn finalize_recording(recorder: &mut RecorderState) -> Result<()> {
    let content = serde_json::to_string_pretty(&recorder.events)?;
    fs::write(recorder.log_path.clone(), content)
        .with_context(|| format!("failed to write log to {}", recorder.log_path.display()))?;
    Ok(())
}

/// Replay flight events to reconstruct workflow
pub fn replay_events(log_path: PathBuf) -> Result<()> {
    let content = fs::read_to_string(&log_path)
        .with_context(|| format!("failed to read log from {}", log_path.display()))?;
    let events: Vec<FlightEvent> = serde_json::from_str(&content)?;
    for event in events {
        match event {
            FlightEvent::ArtifactCreated(digest) => {
                println!("Replaying: Artifact created with digest {}", digest);
            }
            FlightEvent::ReceiptIssued(digest) => {
                println!("Replaying: Receipt issued for artifact {}", digest);
            }
            FlightEvent::ArtifactVerified(digest) => {
                println!("Replaying: Artifact verified with digest {}", digest);
            }
            FlightEvent::ReceiptVerified(digest) => {
                println!("Replaying: Receipt verified for artifact {}", digest);
            }
            FlightEvent::BuildCommand(path) => {
                println!("Replaying: Build command for {}", path.display());
            }
            FlightEvent::VerifyProofCommand(formal, content) => {
                println!("Replaying: Verify proof command for {} and {}", formal.display(), content.display());
            }
            FlightEvent::InspectCommand(path) => {
                println!("Replaying: Inspect command for {}", path.display());
            }
            FlightEvent::VerifyCommand(path) => {
                println!("Replaying: Verify command for {}", path.display());
            }
            FlightEvent::HashCommand(path) => {
                println!("Replaying: Hash command for {}", path.display());
            }
        }
    }
    Ok(())
}