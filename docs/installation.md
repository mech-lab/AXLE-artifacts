# Installation

## Requirements

- Rust 1.70 or higher
- Cargo (Rust package manager)

## Install from Source

```bash
# Clone the repository
git clone https://github.com/AxiomMath/axle-rs.git
cd axle-rs

# Build the CLI tool
cargo build --release

# Install the binary (optional)
cargo install --path crates/axle-cli
```

## Development Installation

For development, you can install the Rust workspace with all features:

```bash
git clone https://github.com/AxiomMath/axle-rs.git
cd axle-rs
cargo build --workspace
```

This will build all crates in the workspace, including:
- `axle-core`: Core artifact engine
- `axle-cli`: Command-line interface
- `axle-artifact`: Artifact directory utilities
- `axle-receipts`: Receipt and signing layer

## Verify Installation

```bash
# Check CLI version
axle-rs --version

# Check that the binary is available
which axle-rs
```

## Quick Start

Once installed, you can start creating proof artifacts:

```bash
# Create a signed artifact from claim and evidence
axle-rs issue claim.json evidence.json --signing-key private.key

# Verify an artifact independently
axle-rs verify --artifact-dir .axle --public-key public.key

# Generate a receipt for audit trails
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

## Dependencies

AXLE-rs requires the following system dependencies:

- **Rust toolchain**: Install via rustup (https://rustup.rs/)
- **OpenSSL**: Required for cryptographic operations
- **Git**: For cloning the repository

On macOS, you can install Rust with:
```bash
/bin/bash -c "$(curl -fsSL https://install.rustup.rs)"
```

On Ubuntu/Debian:
```bash
sudo apt update
sudo apt install build-essential curl git
```

## Next Steps

See the [Quick Start](quickstart.md) tutorial to get started with creating your first proof artifact.
