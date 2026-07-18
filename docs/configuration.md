# Configuration

AXLE-rs can be configured via environment variables, configuration files, or command-line arguments. This document covers the configuration options for the independent proof artifact engine.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AXLE_RS_HOME` | `~/.axle-rs` | Base directory for AXLE-rs data |
| `AXLE_RS_KEY_DIR` | `~/.axle-rs/keys` | Directory for signing keys |
| `AXLE_RS_POLICY_DIR` | `~/.axle-rs/policies` | Directory for verification policies |
| `AXLE_RS_LOG_LEVEL` | `info` | Logging level (debug, info, warn, error) |
| `AXLE_RS_API_URL` | `http://localhost:8080` | API server URL for remote operations |
| `AXLE_RS_TIMEOUT_SECONDS` | `30` | Request timeout in seconds |
| `AXLE_RS_MAX_CONCURRENCY` | `4` | Max concurrent operations |

### Example

```bash
export AXLE_RS_HOME=/var/lib/axle-rs
export AXLE_RS_KEY_DIR=/etc/axle-rs/keys
export AXLE_RS_POLICY_DIR=/etc/axle-rs/policies
export AXLE_RS_LOG_LEVEL=debug
export AXLE_RS_API_URL=https://api.axle-rs.example.com
export AXLE_RS_TIMEOUT_SECONDS=60
export AXLE_RS_MAX_CONCURRENCY=8
```

## Configuration File

AXLE-rs supports a TOML configuration file at `$AXLE_RS_HOME/config.toml`:

```toml
[general]
home = "/var/lib/axle-rs"
log_level = "info"

[keys]
directory = "/etc/axle-rs/keys"
default_key = "underwriter.key"

[policies]
directory = "/etc/axle-rs/policies"
default_policy = "insurance_risk_v1.json"

[api]
url = "https://api.axle-rs.example.com"
timeout_seconds = 60
max_concurrency = 8

[verification]
strict = false
require_receipt = true
```

## CLI Configuration

The CLI reads configuration from:
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file
4. Built-in defaults (lowest priority)

### Global Options

```bash
# Custom configuration file
axle-rs --config /path/to/config.toml issue claim.json evidence.json

# Custom log level
axle-rs --log-level debug verify --artifact-dir .axle --public-key public.key

# Custom home directory
axle-rs --home /var/lib/axle-rs issue claim.json evidence.json
```

## Signing Keys

### Key Generation

```bash
# Generate a new ed25519 key pair
axle-rs key generate --output private.key --public-key public.key

# Generate with a specific name
axle-rs key generate --name underwriter --output /etc/axle-rs/keys/
```

### Key Management

```bash
# List available keys
axle-rs key list

# Show key information
axle-rs key show --name underwriter

# Import an existing key
axle-rs key import --path /path/to/private.key --name imported_key
```

## Verification Policies

### Policy Registration

```bash
# Register a verification policy
axle-rs policy register --path insurance_risk_v1.json --name insurance_risk_v1

# List registered policies
axle-rs policy list

# Show policy details
axle-rs policy show --name insurance_risk_v1
```

### Policy Configuration

Policies can be configured via:
- Local file path
- Base64-encoded JSON
- Pre-registered policy name

## Compliance Configuration

### Audit Trail Settings

```toml
[audit]
enabled = true
receipt_dir = "/var/lib/axle-rs/receipts"
retention_days = 365
```

### Regulatory Compliance

```toml
[compliance]
jurisdiction = "US"
regulatory_framework = "SOX"
require_independent_verification = true
```

## Integration

### CI/CD Integration

```yaml
# Example GitHub Actions workflow
- name: Verify Compliance Artifact
  run: |
    axle-rs verify --artifact-dir .axle --public-key ${{ secrets.AXLE_RS_PUBLIC_KEY }}
```

### API Integration

```bash
# Set API URL for remote operations
export AXLE_RS_API_URL=https://api.axle-rs.example.com

# Use API for artifact creation
axle-rs issue claim.json evidence.json --api
```

## Security

### Key Security

- Private keys should be stored securely (e.g., HSM, encrypted storage)
- Public keys can be distributed freely
- Key rotation should be performed regularly

### Policy Security

- Policies should be versioned and immutable
- Policy changes should be audited
- Policy validation should be strict

## Troubleshooting

See [Troubleshooting](troubleshooting.md) for common configuration issues.
