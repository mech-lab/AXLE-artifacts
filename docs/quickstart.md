# Quick Start

AXLE-rs is an experimental Rust-native platform for creating durable, verifiable proof artifacts designed for compliance, insurance, and legal applications. This guide will help you get started with creating and verifying proof artifacts.

## Prerequisites

- Rust 1.70 or higher with Cargo
- A text editor (VS Code, IntelliJ, etc.)

## Installation

If you haven't installed AXLE-rs yet:

```bash
# Clone the repository
git clone https://github.com/AxiomMath/axle-rs.git
cd axle-rs

# Build the CLI tool
cargo build --release

# Install the binary (optional)
cargo install --path crates/axle-cli
```

## Creating Your First Proof Artifact

### Step 1: Create a Claim

A claim represents what you're attesting. For compliance purposes, it typically includes:
- The type of claim (e.g., `insurance_risk`, `compliance_control`)
- The subject (who or what is being claimed)
- The issuer (who is making the claim)
- The payload (the actual claim content)

Create a `claim.json` file:

```json
{
  "type": "insurance_risk",
  "subject": {
    "entity": "Acme Corporation",
    "policy_number": "POL-12345"
  },
  "issuer": {
    "name": "Risk Assessment Division",
    "id": "risk-div-001"
  },
  "created_at": "2026-07-18T03:32:29.585Z",
  "payload": {
    "risk_score": 0.75,
    "risk_factors": ["market_volatility", "credit_rating"],
    "assessment_date": "2026-07-18"
  },
  "attachments": []
}
```

### Step 2: Create Evidence

Evidence supports the claim with verifiable data:

```json
{
  "type": "audit_trail",
  "subject": {
    "entity": "Acme Corporation",
    "policy_number": "POL-12345"
  },
  "issuer": {
    "name": "Risk Assessment Division",
    "id": "risk-div-001"
  },
  "created_at": "2026-07-18T03:32:29.585Z",
  "payload": {
    "source": "internal_risk_system",
    "data_hashes": ["sha256:abc123...", "sha256:def456..."],
    "verification_status": "verified"
  },
  "attachments": []
}
```

### Step 3: Create a Verification Policy

For compliance, you need a verification policy that defines how the claim should be verified:

```json
{
  "schema_version": "1.0",
  "claim_type": "insurance_risk",
  "required_fields": ["risk_score", "risk_factors"],
  "validation_rules": {
    "risk_score_range": [0.0, 1.0],
    "required_factors": ["market_volatility", "credit_rating"]
  },
  "issuers": ["risk-div-001"],
  "validity_period": "P1Y"
}
```

### Step 4: Generate a Signing Key

For cryptographic signing, you'll need an ed25519 key pair:

```bash
# Generate a key pair (using the axle-rs CLI or external tool)
axle-rs key generate --output private.key --public-key public.key
```

### Step 5: Create a Signed Artifact

Now create your signed artifact:

```bash
# Create a signed artifact
axle-rs issue claim.json evidence.json --signing-key private.key

# This creates a .axle directory with:
# - manifest.json (artifact metadata)
# - claim.json (your claim)
# - evidence.json (your evidence)
# - verification_policy.json (if provided)
# - hashes.json (content hashes)
# - receipt.json (cryptographic receipt)
```

### Step 6: Verify Independently

The key advantage of AXLE-rs is independent verification:

```bash
# Verify the artifact independently
axle-rs verify --artifact-dir .axle --public-key public.key

# This verifies:
# 1. The artifact is cryptographically valid
# 2. The claim matches the evidence
# 3. The verification policy is satisfied
# 4. The artifact hasn't been tampered with
```

### Step 7: Generate a Receipt

For audit trails, generate a receipt:

```bash
# Generate a receipt for audit trails
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

## Compliance Workflow Example

Here's a complete compliance workflow for insurance:

```bash
# 1. Underwriter creates a risk assessment claim
axle-rs issue claim.json evidence.json --signing-key underwriter.key

# 2. Compliance officer verifies independently
axle-rs verify --artifact-dir .axle --public-key underwriter.pub

# 3. Auditor generates a receipt for the audit trail
axle-rs attest claim.json evidence.json --receipt-path audit_receipt.json

# 4. Regulator can verify using the public key
axle-rs verify --artifact-dir .axle --public-key underwriter.pub
```

## Next Steps

- [Python API Reference](python-api.md) - For programmatic access
- [CLI Reference](cli-reference.md) - All available commands
- [Configuration](configuration.md) - Environment variables and options
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Integration

AXLE-rs can be integrated into:
- CI/CD pipelines for automated compliance checks
- Legal document workflows
- Insurance underwriting systems
- Regulatory reporting systems

The independent verification capability makes it ideal for scenarios where third-party auditors or regulators need to verify claims without requiring access to the issuer's systems.
