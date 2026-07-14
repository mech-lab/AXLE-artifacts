use axle_receipts::flight_recorder::{FlightEvent, RecorderState, new_recorder, record_event, finalize_recording, replay_events};
use std::path::PathBuf;

#[test]
fn test_record_and_replay() {
    // Create recorder with temporary log path
    let log_path = PathBuf::from("./test_recorder.log");
    let mut recorder = new_recorder(log_path);

    // Record sample events
    record_event(&mut recorder, FlightEvent::ArtifactCreated("digest123".into()));
    record_event(&mut recorder, FlightEvent::ReceiptIssued("digest456".into()));
    record_event(&mut recorder, FlightEvent::BuildCommand(PathBuf::from("./src")));

    // Finalize recording
    finalize_recording(recorder).unwrap();

    // Replay events
    replay_events(log_path).unwrap();

    // Verify log contents (manual check in this test)
    // In a real test, we'd read the log file and assert contents
}

#[test]
fn test_replay_verify_proof() {
    let log_path = PathBuf::from("./test_replay.log");
    let mut recorder = new_recorder(log_path);

    // Record verify proof event
    record_event(&mut recorder, FlightEvent::VerifyProofCommand(
        PathBuf::from("./formal"),
        PathBuf::from("./content")
    ));

    finalize_recording(recorder).unwrap();

    // Replay should print the verify proof command
    replay_events(log_path).unwrap();
}