# AXLE-rs API Reference

This documentation covers the command-line interface (CLI) and API for creating and verifying independent proof artifacts in compliance, insurance, and legal contexts.

## CLI Commands

### `axle-rs issue`
Create a signed proof artifact from claim and evidence bundles.

**Usage:**
```bash
axle-rs issue [OPTIONS] claim.json evidence.json

Options:
  --signing-key PATH       Path to private key for signing
  --receipt-path PATH      Path to save receipt ID
  --verification-policy PATH  Path to verification policy file

Example:
```bash
axle-rs issue claim.json evidence.json --signing-key private.key --receipt-path receipt.json
```

### `axle-rs verify`
Independently verify an artifact using its directory and public key.

**Usage:**
```bash
axle-rs verify [OPTIONS] --artifact-dir PATH --public-key PATH

Options:
  --strict                Enforce strict verification
  --output PATH           Path to save verification report

Example:
```bash
axle-rs verify --artifact-dir .axle --public-key public.key
```

### `axle-rs attest`
Generate a receipt for audit trails.

**Usage:**
```bash
axle-rs attest [OPTIONS] claim.json evidence.json

Options:
  --receipt-path PATH      Path to save receipt

Example:
```bash
axle-rs attest claim.json evidence.json --receipt-path receipt.json
```

## API Endpoints

### POST /api/v1/issue
Create a signed artifact via API.

**Request Body:**
```json
{
  "claim": "base64-encoded claim.json",
  "evidence": "base64-encoded evidence.json",
  "signing_key": "base64-encoded private key"
}

**Response:**
```json
{
  "artifact_id": "hash-of-manifest",
  "receipt_id": "unique-receipt-id",
  "verification_policy": "policy-hash"
}
```

### POST /api/v1/verify
Verify an artifact independently.

**Request Body:**
```json
{
  "artifact_dir": ".axle",
  "public_key": "base64-encoded public key"
}

**Response:**
```json
{
  "verified": true,
  "verification_policy": "policy-hash",
  "receipt_id": "receipt-hash"
}
```

## Verification Policy Integration

The API supports specifying verification policies via:
- Path to local policy file
- Base64-encoded policy JSON
- Policy hash for pre-registered policies

## Compliance Features

- Cryptographic receipts with `receipt_id` generation
- Schema-versioned verification policies
- Independent verification without issuer access
- Audit trail support for regulatory requirements