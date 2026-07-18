# Independent Proof Artifact Engine Documentation

This documentation covers the AXLE-rs independent proof artifact engine, designed for compliance, insurance, and legal applications.

## Overview

AXLE-rs provides a Rust-native platform for creating durable, verifiable proof artifacts that can be independently verified by third parties. The system is designed for regulatory compliance workflows where audit trails, cryptographic assurances, and independent verification are required.

## Key Components

### Artifact Format (.axle)
Standardized directory structure containing:
- `manifest.json`: Metadata and schema version
- `claim.json`: The core claim being attested
- `evidence.json`: Supporting evidence for the claim
- `verification_policy.json`: Policy governing how the claim should be verified
- `hashes.json`: Canonical hashes for content addressing
- Optional signed components and receipts

### Signing and Verification
- **ed25519 signatures** for cryptographic assurance
- **receipt_id generation** for audit trails
- **Independent verification** requiring only artifact + public key
- **Verification policies** for different regulatory domains

### Claim Types
- `insurance_risk`: Risk assessment proofs for underwriting and claims
- `compliance_control`: Regulatory compliance verification (SOX, GDPR, etc.)
- `legal_disclosure`: Legal statement attestation for contracts and disclosures
- `decision_proof`: Algorithmic decision justification for AI/ML systems

## Getting Started

### Installation
```bash
# Install the CLI
cargo install --path crates/axle-cli

# Or build from source
cargo build --release
```

### Basic Usage

Create a signed artifact:
```bash
axle-rs issue claim.json evidence.json --signing-key private.key
```

Verify an artifact independently:
```bash
axle-rs verify --artifact-dir .axle --public-key public.key
```

Generate a receipt for audit trails:
```bash
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

## Documentation

- [Artifact Format](artifact-format.md)
- [Verification Policies](verification-policy.md)
- [Receipt Binding](receipt-binding.md)
- [Delivery Channels](delivery-channels.md)
- [CLI Reference](cli-reference.md)
- [API Reference](api-reference.md)

## Compliance Focus

AXLE-rs is built for organizations that need to:
- Maintain auditable proof trails for regulatory compliance
- Provide independent verification to auditors or regulators
- Generate legally significant artifacts with cryptographic assurance
- Integrate with existing compliance workflows and systems
- Meet documentation requirements for insurance claims and legal proceedings

## Tools Reference

While AXLE-rs focuses on artifact creation and verification, the underlying AXLE tools remain available for proof processing:

- `check`: Validate Lean code for errors
- `verify_proof`: Validate proofs against formal statements
- `extract_decls`: Extract declarations with dependencies
- And other proof manipulation tools

These tools can be used to generate the inputs for AXLE-rs artifact creation.
