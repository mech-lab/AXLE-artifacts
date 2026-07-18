# Comprehensive TDD Test Plan for Experimental Flight Recorder

## Current State Analysis

### Existing Tests
1. **Unit tests** (`tests/unit/flight_recorder_test.rs`):
   - `test_record_and_replay` - Basic record/replay cycle
   - `test_replay_verify_proof` - VerifyProofCommand replay

2. **Integration tests** (`crates/axle-cli/tests/cli_recording_test.rs`):
   - `cli_build_records_flight_log` - Build with --record creates log
   - `cli_replay_reads_flight_log` - Replay reads log
   - `cli_replay_missing_log_errors` - Missing log error handling
   - `cli_replay_corrupted_log_errors` - Corrupted JSON error handling

### Gaps Identified

| Category | Missing Coverage |
|----------|------------------|
| **Event Types** | Only `BuildCommand`, `ArtifactCreated`, `VerifyProofCommand` tested. Missing: `ReceiptIssued`, `ArtifactVerified`, `ReceiptVerified`, `InspectCommand`, `VerifyCommand`, `HashCommand` |
| **CLI Commands** | Only `build --record` tested. Missing: `verify-proof --record`, `inspect --record`, `verify --record`, `hash --record` |
| **Replay Logic** | Replay only prints - doesn't actually re-execute commands |
| **Error Handling** | No tests for partial writes, concurrent access, large logs |
| **Serialization** | No tests for JSON format stability, Digest serialization |
| **Edge Cases** | Empty logs, very large logs, special characters in paths |
| **Thread Safety** | No concurrent recorder tests |

---

## TDD Test Plan

### Phase 1: Unit Tests - Core Recorder Functions

#### 1.1 Recorder Initialization
- [ ] `test_new_recorder_creates_empty_state` - Verify new_recorder creates empty events vec
- [ ] `test_new_recorder_with_various_paths` - Test with absolute, relative, nested paths

#### 1.2 Event Recording
- [ ] `test_record_event_appends_to_events` - Verify events vector grows
- [ ] `test_record_all_event_types` - Record each FlightEvent variant
- [ ] `test_record_event_preserves_order` - Events maintain insertion order
- [ ] `test_record_event_with_complex_digest` - Digest with special characters

#### 1.3 Finalization & Persistence
- [ ] `test_finalize_recording_writes_valid_json` - Output is valid JSON array
- [ ] `test_finalize_recording_creates_file` - File exists after finalize
- [ ] `test_finalize_recording_overwrites_existing` - Second finalize replaces content
- [ ] `test_finalize_recording_handles_io_error` - Returns error on unwritable path
- [ ] `test_finalize_recording_empty_events` - Empty events array written correctly

#### 1.4 Replay Function
- [ ] `test_replay_events_reads_all_events` - All events processed
- [ ] `test_replay_events_handles_missing_file` - Returns error for missing file
- [ ] `test_replay_events_handles_corrupted_json` - Returns error for invalid JSON
- [ ] `test_replay_events_handles_wrong_type` - Returns error for non-array JSON
- [ ] `test_replay_events_empty_log` - Empty array succeeds silently
- [ ] `test_replay_events_unknown_variant` - Handles future enum variants gracefully

#### 1.5 Serialization Round-trip
- [ ] `test_flight_event_serialization_roundtrip` - Each variant serializes/deserializes
- [ ] `test_digest_serialization` - Digest format preserved
- [ ] `test_pathbuf_serialization` - Paths serialize correctly (cross-platform)

---

### Phase 2: Unit Tests - CLI Integration

#### 2.1 Global --record Flag
- [ ] `test_record_flag_creates_recorder` - Flag initializes recorder state
- [ ] `test_record_flag_without_command` - Flag alone doesn't crash
- [ ] `test_record_flag_with_invalid_path` - Handles unwritable paths

#### 2.2 Build Command Recording
- [ ] `test_build_records_build_command` - BuildCommand event recorded
- [ ] `test_build_records_artifact_created` - ArtifactCreated event on success
- [ ] `test_build_records_on_failure` - Events recorded even if build fails
- [ ] `test_build_without_record_no_recorder` - No recorder created without flag

#### 2.3 Verify-Proof Command Recording
- [ ] `test_verify_proof_records_verify_proof_command` - VerifyProofCommand event
- [ ] `test_verify_proof_records_artifact_created` - ArtifactCreated on success
- [ ] `test_verify_proof_records_on_failure` - Events recorded on failure

#### 2.4 Other Command Recording
- [ ] `test_inspect_records_inspect_command` - InspectCommand event
- [ ] `test_verify_records_verify_command` - VerifyCommand event
- [ ] `test_hash_records_hash_command` - HashCommand event
- [ ] `test_graph_records_no_events` - Graph doesn't record (read-only)
- [ ] `test_diff_records_no_events` - Diff doesn't record (read-only)

#### 2.5 Replay Subcommand
- [ ] `test_replay_subcommand_exists` - Command registered
- [ ] `test_replay_subcommand_requires_log_path` - Missing arg error
- [ ] `test_replay_calls_replay_events` - Delegates to library function

---

### Phase 3: Integration Tests - End-to-End Workflows

#### 3.1 Complete Build Workflow
- [ ] `test_build_record_replay_cycle` - Build → record → replay produces same events
- [ ] `test_build_record_with_environment` - Environment captured in events
- [ ] `test_build_record_with_custom_output` - Output path in events

#### 3.2 Complete Verify-Proof Workflow
- [ ] `test_verify_proof_record_replay_cycle` - Verify-proof → record → replay
- [ ] `test_verify_proof_with_sorries` - Permitted sorries captured

#### 3.3 Multi-Command Session
- [ ] `test_multiple_commands_single_recording` - Multiple commands in one session
- [ ] `test_record_across_commands` - Events from different commands interleaved

#### 3.4 Replay Verification
- [ ] `test_replay_outputs_all_event_types` - Each event type produces output
- [ ] `test_replay_preserves_event_order` - Output order matches log order
- [ ] `test_replay_with_real_artifact` - Replay with actual artifact files

---

### Phase 4: Edge Cases & Stress Tests

#### 4.1 File System Edge Cases
- [ ] `test_log_file_in_nonexistent_directory` - Creates parent dirs or errors
- [ ] `test_log_file_permissions` - Read-only, write-only scenarios
- [ ] `test_log_file_symlinks` - Symlink handling
- [ ] `test_unicode_paths` - Non-ASCII paths in events

#### 4.2 Large Scale
- [ ] `test_large_event_log` - 10,000+ events performance
- [ ] `test_large_digest_values` - Very long digest strings
- [ ] `test_deeply_nested_paths` - Deep path hierarchies

#### 4.3 Concurrency
- [ ] `test_concurrent_recorder_access` - Mutex prevents data races
- [ ] `test_recorder_state_isolation` - Separate recorders don't interfere

#### 4.4 Error Recovery
- [ ] `test_partial_write_recovery` - Crash during write doesn't corrupt
- [ ] `test_disk_full_handling` - Graceful error on ENOSPC

---

### Phase 5: Regression & Compatibility Tests

#### 5.1 Format Stability
- [ ] `test_json_format_v1_compatibility` - Current format parseable
- [ ] `test_additive_field_compatibility` - New fields don't break old logs

#### 5.2 CLI Compatibility
- [ ] `test_help_includes_record_flag` - --help shows --record
- [ ] `test_help_includes_replay_command` - --help shows replay
- [ ] `test_version_includes_recorder` - Version info consistent

---

## Test Implementation Priority

### High Priority (Blockers for Release)
1. All 9 FlightEvent variants recorded and replayed
2. All 5 CLI commands with --record flag
3. Error handling for missing/corrupted logs
4. Serialization round-trip for all types

### Medium Priority (Quality)
1. Multi-command recording sessions
2. Large log performance
3. Unicode path handling
4. Concurrent access safety

### Low Priority (Nice to Have)
1. Disk full handling
2. Symlink edge cases
3. Format versioning tests

---

## Test File Structure

```
tests/
├── unit/
│   ├── flight_recorder_test.rs          # Existing - expand
│   ├── flight_recorder_serialization_test.rs  # NEW
│   ├── flight_recorder_edge_cases_test.rs     # NEW
│   └── flight_recorder_concurrency_test.rs    # NEW
├── integration/
│   ├── cli_recording_test.rs            # Existing - expand
│   ├── cli_recording_all_commands_test.rs     # NEW
│   ├── cli_recording_workflows_test.rs        # NEW
│   └── cli_recording_stress_test.rs           # NEW
```

---

## TDD Workflow

For each test:
1. **Red** - Write failing test first
2. **Green** - Implement minimal code to pass
3. **Refactor** - Clean up implementation
4. **Commit** - Atomic commit per test group

---

## Success Criteria

- [ ] 100% FlightEvent variant coverage in tests
- [ ] 100% CLI command --record coverage
- [ ] All error paths tested
- [ ] Performance baseline established (<100ms for 1000 events)
- [ ] Zero flaky tests
- [ ] CI passes on Linux, macOS, Windows