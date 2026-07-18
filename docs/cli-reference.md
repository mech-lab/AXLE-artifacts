# AXLE-rs CLI Reference

This documentation covers the command-line interface for creating and verifying independent proof artifacts in compliance, insurance, and legal contexts.

## Global Options

```
axle-rs [OPTIONS] COMMAND [ARGS]

Options:
  --version          Show version and exit
  --help             Print help information
```

## Commands

### `axle-rs issue`

Create a signed proof artifact from claim and evidence bundles.

**Usage:**
```bash
axle-rs issue [OPTIONS] claim.json evidence.json

Options:
  --signing-key PATH       Path to private key for signing (required)
  --receipt-path PATH      Path to save receipt ID
  --verification-policy PATH  Path to verification policy file
  --help                   Print help information

Example:
```bash
axle-rs issue claim.json evidence.json --signing-key private.key --receipt-path receipt.json
```

**Description:**
Creates a signed `.axle` artifact bundle containing the claim, evidence, and cryptographic signature. The artifact can be independently verified by third parties using the public key.

### `axle-rs verify`

Independently verify an artifact using its directory and public key.

**Usage:**
```bash
axle-rs verify [OPTIONS] --artifact-dir PATH --public-key PATH

Options:
  --strict                Enforce strict verification
  --output PATH           Path to save verification report
  --help                  Print help information

Example:
```bash
axle-rs verify --artifact-dir .axle --public-key public.key
```

**Description:**
Verifies that an artifact is cryptographically valid and matches its verification policy. This command can be run by any party with the public key, without requiring access to the private key or issuer.

### `axle-rs attest`

Generate a receipt for audit trails.

**Usage:**
```bash
axle-rs attest [OPTIONS] claim.json evidence.json

Options:
  --receipt-path PATH      Path to save receipt
  --help                   Print help information

Example:
```bash
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

**Description:**
Creates a cryptographic receipt that binds the claim and evidence to a specific artifact. The receipt includes a `receipt_id` for audit trail purposes and can be independently verified.

## Exit Codes

- `0` - Success (operation completed without errors)
- `1` - Failure (general error)
- `2` - File exists error (use -f to overwrite)
- `3` - Validation failed (when using --strict flag)
- `130` - Interrupted (Ctrl+C)

## Configuration

The CLI reads configuration from:
- Environment variables
- Configuration files
- Command-line arguments

## Compliance Features

- **Independent Verification**: Third parties can verify artifacts without issuer access
- **Cryptographic Assurance**: ed25519 signatures for tamper-proof artifacts
- **Audit Trails**: `receipt_id` generation for immutable audit trails
- **Regulatory Compliance**: Built-in support for insurance, legal, and compliance use cases

## Example Workflows

### Creating a Compliance Artifact

```bash
# Create a claim for insurance risk assessment
axle-rs issue claim.json evidence.json --signing-key private.key

# Verify independently for regulatory audit
axle-rs verify --artifact-dir .axle --public-key public.key
```

### Legal Disclosure Workflow

```bash
# Generate a receipt for legal disclosure
axle-rs attest claim.json evidence.json --receipt-path receipt.json

# Verify the disclosure independently
axle-rs verify --artifact-dir .axle --public-key public.key
```

## Integration

The CLI can be integrated into:
- CI/CD pipelines for automated compliance checks
- Legal document workflows
- Insurance underwriting systems
- Regulatory reporting systems

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
