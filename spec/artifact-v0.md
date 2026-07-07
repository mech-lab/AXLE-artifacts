# `axle.artifact.v0`

`axle.artifact.v0` is the current concrete output format of the AXLE-rs fork. It takes AXLE-compatible Lean processing results and freezes them into a deterministic, inspectable directory artifact that can survive outside a single API response or runtime session. The same artifact family is used for both `build` outputs and `verify-proof` outputs.

The emphasis in v0 is stability and transparency rather than compression or maximal fidelity. This is the layer where AXLE-rs turns proof-processing output into a durable research artifact.

## Goals

- Freeze AXLE-compatible proof outputs into a deterministic portable artifact.
- Keep the artifact human-inspectable by starting with a directory layout.
- Separate proof artifacts from later receipt attestations.

## Directory layout

```text
example.axle/
├── manifest.json
├── source.json
├── declarations.json
├── diagnostics.json
├── hashes.json
├── verification.json optional
└── adapter.json optional
```

`receipt.json` is intentionally excluded from artifact v0. Receipts bind to an artifact later and stay conceptually separate.

## Required files

- `manifest.json`
- `source.json`
- `declarations.json`
- `diagnostics.json`
- `hashes.json`

## Optional files

- `verification.json`
- `adapter.json`

`verification.json` is a hashed core object present on proof-verification artifacts. It records the verification mode, pass/fail status, the digest of the formal statement, and the artifact-level list of failed declarations.

`adapter.json` stores raw upstream AXLE request and response envelopes. In the current implementation it distinguishes operations such as `build` and `verify_proof`, and preserves the full upstream payloads needed for provenance and debugging.

## Hashing rules

- Object digests use `sha256`.
- Canonicalization uses recursively sorted JSON object keys.
- The artifact digest is computed from the full artifact body with `manifest.artifact_id` omitted.
- `verification.json` is included in the artifact digest when present.
- `adapter.json` and the optional `manifest.objects.adapter` pointer are excluded from the artifact digest.

## Manifest fields

- `schema`: must be `axle.artifact.v0`
- `artifact_id`: content-derived digest of the artifact body
- `producer`: artifact producer metadata
- `source`: language and digest summary for `source.json`
- `environment`: Lean and engine environment summary
- `objects`: relative file names for bundled objects

For build artifacts, `objects.verification` is absent. For proof-verification artifacts, `objects.verification` points to `verification.json`.

## Declaration fields

- `name`: declaration name
- `kind`: declaration kind, matching AXLE declaration kinds where possible
- `statement_digest`: digest of the AXLE declaration type text
- `body_digest`: digest of the AXLE single-declaration source text
- `dependencies`: local declaration names referenced by the declaration
- `verification_status`: declaration-level status derived from AXLE `check` for `build`, or from AXLE `verify_proof` for `verify-proof`

## Verification summary fields

- `mode`: currently `verify_proof`
- `status`: `pass` or `fail`
- `formal_statement_digest`: deterministic digest of the formal statement text
- `failed_declarations`: artifact-level failed declaration names reported by AXLE, including names not present in extracted declaration documents

## Validation rules

- Every required file must exist.
- `manifest.schema` must match the v0 schema string.
- Stored digests must match recomputed canonical digests.
- `manifest.artifact_id` must match the recomputed artifact digest.
- If `verification.json` is present, it is part of core hash validation.
- If `adapter.json` is present, it must parse as JSON, but it is not part of core hash validation.
