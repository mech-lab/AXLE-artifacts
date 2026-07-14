# AXLE Receipt Binding v0 Specification

## Flight Recorder Capabilities

The flight recorder is a diagnostic and replay tool that captures and reconstructs artifact processing workflows. It records key events during CLI operations and allows deterministic replay of the workflow.

### Key Features
- Records artifact creation, receipt issuance, verification, and CLI command executions
- Stores events in JSON format for replay
- Supports replay of artifact workflows to verify deterministic outcomes
- Integrates with existing CLI commands via `--record` flag

### Usage
1. Enable recording with `--record <log_path>`
2. Events are automatically recorded during build, verify, and other commands
3. Replay with `axle-rs replay <log_path>` (to be implemented)

### Event Types
- `ArtifactCreated(Digest)`
- `ReceiptIssued(Digest)`
- `ArtifactVerified(Digest)`
- `ReceiptVerified(Digest)`
- `BuildCommand(PathBuf)`
- `VerifyProofCommand(PathBuf, PathBuf)`

### Implementation Notes
- Uses `serde` for JSON serialization/deserialization
- Maintains state in `RecorderState` struct
- Events are stored in a log file specified by the user
