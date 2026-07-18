# Independent Proof Artifact Engine

AXLE-rs is a Rust-native platform for creating durable, verifiable proof artifacts designed for compliance, insurance, and legal applications. Unlike traditional proof systems that focus on formal methods research, AXLE-rs provides a production-ready infrastructure for generating, attesting, and independently verifying legally significant artifacts that meet regulatory requirements.

## Why AXLE-rs Exists

Modern compliance workflows require more than just verification - they need auditable trails, regulatory compliance, and independent verification capabilities. AXLE-rs addresses this gap by providing:

- **Independent Verification**: Create artifacts that can be verified by third parties without requiring the original issuer
- **Regulatory Compliance**: Built-in support for insurance risk assessments, compliance controls, and legal disclosures
- **Audit Trails**: Cryptographic receipts and signing for immutable audit trails
- **Pluggable Policies**: Configurable verification policies for different regulatory domains

## Core Concepts

### .axle Artifact Format
A standardized directory structure that bundles:
- `manifest.json`: Artifact metadata and schema version
- Typed JSON files: `claim.json`, `evidence.json`, `verification_policy.json`
- `hashes.json`: Canonical hashes for content addressing
- Optional signed components

### Key Innovations
- **Signing Layer**: ed25519 digital signatures with `receipt_id` generation
- **Verification Policy**: Schema-versioned policies for different claim types
- **ProofMail**: Structured data model for delivery and receipt
- **Independent Verification**: CLI/endpoint that only requires artifact + public key

### Claim Types
- `insurance_risk`: Risk assessment proofs
- `compliance_control`: Regulatory compliance verification
- `legal_disclosure`: Legal statement attestation
- `decision_proof`: Algorithmic decision justification

## System Model

```
formal statement? → Lean source / proof candidate → AXLE processing → .axle artifact → 
Signing Layer → Receipt Binding → Independent Verification
```

## Getting Started

```bash
# Create a signed artifact
axle-rs issue claim.json evidence.json --signing-key private.key

# Verify an artifact independently
axle-rs verify --artifact-dir .axle --public-key public.key

# Generate a receipt for audit trails
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

## Documentation

- [Artifact Format Specification](spec/artifact-format.md)
- [Verification Policy Guide](spec/verification-policy.md)
- [Receipt Binding Protocol](spec/receipt-binding.md)
- [Delivery Channels](spec/delivery-channels.md)
- [API Reference](docs/api-reference.md)

## Compliance Focus

AXLE-rs is built for organizations that need to:
- Maintain auditable proof trails for regulatory compliance
- Provide independent verification to auditors or regulators
- Generate legally significant artifacts with cryptographic assurance
- Integrate with existing compliance workflows and systems
- Meet documentation requirements for insurance claims and legal proceedings

## Recent Announcements

AXLE-rs is now focused on compliance and regulatory applications. The platform provides:

- **Independent Verification**: Third-party verification without requiring issuer access
- **Regulatory Compliance**: Built-in support for insurance, legal, and compliance use cases
- **Audit Trails**: Cryptographic receipts and signing for immutable audit trails
- **Pluggable Policies**: Configurable verification policies for different regulatory domains

The platform is designed to work with existing AXLE infrastructure while providing a new layer for artifact creation, signing, and independent verification.

## Technical Report

For detailed technical information about the AXLE-rs architecture and implementation, see our [technical report](https://arxiv.org/abs/2606.26442).
