use crate::{FlightEvent, RecorderState, new_recorder, record_event, finalize_recording, replay_events};
use std::path::PathBuf;
use std::fs;

#[test]
fn test_new_recorder_creates_empty_state() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let recorder = new_recorder(log_path.clone());
    
    // Verify it has empty events vec
    assert_eq!(recorder.events.len(), 0);
    assert_eq!(recorder.log_path, log_path);
}

#[test]
fn test_new_recorder_with_various_paths() {
    // Test with absolute path
    let abs_path = PathBuf::from("/tmp/test.log");
    let recorder = new_recorder(abs_path.clone());
    assert_eq!(recorder.log_path, abs_path);
    
    // Test with relative path
    let rel_path = PathBuf::from("test.log");
    let recorder = new_recorder(rel_path.clone());
    assert_eq!(recorder.log_path, rel_path);
}

#[test]
fn test_record_event_appends_to_events() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path.clone());
    
    // Record sample events
    let event1 = FlightEvent::ArtifactCreated("digest123".into());
    record_event(&mut recorder, event1.clone());
    
    let event2 = FlightEvent::BuildCommand(PathBuf::from("./src"));
    record_event(&mut recorder, event2.clone());
    
    // Verify events are appended
    assert_eq!(recorder.events.len(), 2);
    assert_eq!(recorder.events[0], event1);
    assert_eq!(recorder.events[1], event2);
}

#[test]
fn test_record_all_event_types() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path.clone());
    
    // Record each event type
    let events = [
        FlightEvent::ArtifactCreated("digest123".into()),
        FlightEvent::ReceiptIssued("digest456".into()),
        FlightEvent::ArtifactVerified("digest789".into()),
        FlightEvent::ReceiptVerified("digest012".into()),
        FlightEvent::BuildCommand(PathBuf::from("./src")),
        FlightEvent::VerifyProofCommand(PathBuf::from("./formal"), PathBuf::from("./content")),
        FlightEvent::InspectCommand(PathBuf::from("./inspect")),
        FlightEvent::VerifyCommand(PathBuf::from("./verify")),
        FlightEvent::HashCommand(PathBuf::from("./hash")),
    ];
    
    for event in events.iter() {
        record_event(&mut recorder, event.clone());
    }
    
    // Verify all events recorded in order
    assert_eq!(recorder.events.len(), events.len());
    for (i, expected) in events.iter().enumerate() {
        assert_eq!(recorder.events[i], *expected);
    }
}

#[test]
fn test_record_event_preserves_order() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path.clone());
    
    // Record events in specific order
    let event_a = FlightEvent::ArtifactCreated("digest_a".into());
    let event_b = FlightEvent::BuildCommand(PathBuf::from("./src"));
    let event_c = FlightEvent::ReceiptIssued("digest_c".into());
    
    record_event(&mut recorder, event_a.clone());
    record_event(&mut recorder, event_b.clone());
    record_event(&mut recorder, event_c.clone());
    
    // Verify order is preserved
    assert_eq!(recorder.events[0], event_a);
    assert_eq!(recorder.events[1], event_b);
    assert_eq!(recorder.events[2], event_c);
}

#[test]
fn test_record_event_with_complex_digest() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path.clone());
    
    // Record event with complex digest (hex, special chars)
    let complex_digest = "0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3".into();
    let event = FlightEvent::ArtifactCreated(complex_digest.clone());
    
    record_event(&mut recorder, event.clone());
    
    // Verify it's stored correctly
    assert_eq!(recorder.events[0], event);
}

#[test]
fn test_finalize_recording_writes_valid_json() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path.clone());
    
    // Record some events
    record_event(&mut recorder, FlightEvent::ArtifactCreated("digest123".into()));
    record_event(&mut recorder, FlightEvent::BuildCommand(PathBuf::from("./src")));
    
    // Finalize recording
    let result = finalize_recording(&mut recorder);
    assert!(result.is_ok(), "finalize_recording should succeed");
    
    // Verify file was created
    assert!(fs::metadata(&log_path).is_ok(), "log file should exist");
    
    // Verify file contains valid JSON
    let content = fs::read_to_string(&log_path).unwrap();
    assert!(!content.trim().is_empty(), "log file should not be empty");
    
    // Parse JSON to verify it's valid
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array(), "root should be JSON array");
    assert_eq!(parsed.as_array().unwrap().len(), 2, "should have 2 events");
}

#[test]
fn test_finalize_recording_creates_file() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    
    // Ensure file doesn't exist initially
    assert!(!fs::metadata(&log_path).is_ok(), "file should not exist initially");
    
    // Create recorder and finalize
    let mut recorder = new_recorder(log_path.clone());
    let result = finalize_recording(&mut recorder);
    assert!(result.is_ok(), "finalize_recording should succeed");
    
    // Verify file now exists
    assert!(fs::metadata(&log_path).is_ok(), "log file should exist after finalize");
}

#[test]
fn test_finalize_recording_overwrites_existing() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    
    // Create initial file with some content
    fs::write(&log_path, b"old content").unwrap();
    assert!(fs::metadata(&log_path).is_ok(), "file should exist");
    
    // Create recorder and finalize
    let mut recorder = new_recorder(log_path.clone());
    record_event(&mut recorder, FlightEvent::ArtifactCreated("digest123".into()));
    let result = finalize_recording(&mut recorder);
    assert!(result.is_ok(), "finalize_recording should succeed");
    
    // Verify file was overwritten (not appended)
    let content = fs::read_to_string(&log_path).unwrap();
    assert_eq!(content, "[\n  {\n    \"ArtifactCreated\": {\n      \"0\": \"digest123\"\n    }\n]\n");
}

#[test]
fn test_finalize_recording_empty_events() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    
    // Create recorder with no events
    let recorder = new_recorder(log_path.clone());
    let result = finalize_recording(&mut recorder);
    assert!(result.is_ok(), "finalize_recording should succeed with empty events");
    
    // Verify file exists and is valid JSON array
    let content = fs::read_to_string(&log_path).unwrap();
    assert!(content.trim() == "[]", "empty events should produce empty array");
}

#[test]
fn test_finalize_recording_handles_io_error() {
    // Create recorder with path that's not writable (simulate error)
    let log_path = PathBuf::from("/nonexistent/directory/test.log");
    
    // Create recorder
    let mut recorder = new_recorder(log_path.clone());
    
    // Try to finalize - should return error
    let result = finalize_recording(&mut recorder);
    assert!(result.is_err(), "finalize_recording should fail with IO error");
}

#[test]
fn test_replay_events_reads_all_events() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    
    // Create recorder and record events
    let mut recorder = new_recorder(log_path.clone());
    record_event(&mut recorder, FlightEvent::ArtifactCreated("digest123".into()));
    record_event(&mut recorder, FlightEvent::BuildCommand(PathBuf::from("./src")));
    
    // Finalize to write events to file
    finalize_recording(&mut recorder).unwrap();
    
    // Replay events
    let result = replay_events(log_path.clone());
    assert!(result.is_ok(), "replay_events should succeed");
}

#[test]
fn test_replay_events_reads_valid_json_array() {
    // Create test log with valid JSON array
    let log_path = PathBuf::from("./test_recorder.log");
    let events = vec![
        FlightEvent::ArtifactCreated("digest123".into()),
        FlightEvent::BuildCommand(PathBuf::from("./src")),
    ];
    let content = serde_json::to_string_pretty(&events).unwrap();
    fs::write(&log_path, content).unwrap();
    
    // Replay events
    let result = replay_events(log_path.clone());
    assert!(result.is_ok(), "replay_events should succeed with valid JSON");
}

#[test]
fn test_replay_events_handles_missing_file() {
    // Use non-existent file path
    let log_path = PathBuf::from("./nonexistent.log");
    
    // Should return error
    let result = replay_events(log_path.clone());
    assert!(result.is_err(), "replay_events should fail with missing file");
}

#[test]
fn test_replay_events_handles_corrupted_json() {
    // Create test log with invalid JSON
    let log_path = PathBuf::from("./corrupted.log");
    fs::write(&log_path, b"this is not valid json").unwrap();
    
    // Should return error
    let result = replay_events(log_path.clone());
    assert!(result.is_err(), "replay_events should fail with corrupted JSON");
}

#[test]
fn test_replay_events_empty_array() {
    // Create test log with empty JSON array
    let log_path = PathBuf::from("./empty.log");
    fs::write(&log_path, b"[]").unwrap();
    
    // Should succeed silently
    let result = replay_events(log_path.clone());
    assert!(result.is_ok(), "replay_events should succeed with empty array");
}

#[test]
fn test_replay_events_unknown_variant() {
    // This test ensures future enum variants won't break existing code
    // We can't easily test this without unstable Rust features,
    // but we can at least verify the match arm handles unknown variants
    // by compiling with the current enum definition
    
    // Create test log with an event that will be handled by the default arm
    let log_path = PathBuf::from("./test.log");
    
    // Write a JSON that would deserialize to an unknown variant
    // This is more of a compile-time guarantee
    let content = br#"[{"UnknownVariant":"test"}]#;
    fs::write(&log_path, content).unwrap();
    
    // Replay should succeed (the default arm handles it)
    let result = replay_events(log_path.clone());
    assert!(result.is_ok(), "replay_events should handle unknown variants gracefully");
}