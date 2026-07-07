# `axle.artifact.v0`

`axle.artifact.v0` is the current concrete output format of the AXLE-rs fork. It takes AXLE-compatible Lean processing results and freezes them into a deterministic, inspectable directory artifact that can survive outside a single API response or runtime session.

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

- `adapter.json`

`adapter.json` stores raw upstream AXLE responses such as `check` and `extract_decls`. It is auxiliary metadata for provenance and debugging.

## Hashing rules

- Object digests use `sha256`.
- Canonicalization uses recursively sorted JSON object keys.
- The artifact digest is computed from the full artifact body with `manifest.artifact_id` omitted.
- `adapter.json` and the optional `manifest.objects.adapter` pointer are excluded from the artifact digest.

## Manifest fields

- `schema`: must be `axle.artifact.v0`
- `artifact_id`: content-derived digest of the artifact body
- `producer`: artifact producer metadata
- `source`: language and digest summary for `source.json`
- `environment`: Lean and engine environment summary
- `objects`: relative file names for bundled objects

## Declaration fields

- `name`: declaration name
- `kind`: declaration kind, matching AXLE declaration kinds where possible
- `statement_digest`: digest of the AXLE declaration type text
- `body_digest`: digest of the AXLE single-declaration source text
- `dependencies`: local declaration names referenced by the declaration
- `verification_status`: file-level status derived from AXLE `check`

## Validation rules

- Every required file must exist.
- `manifest.schema` must match the v0 schema string.
- Stored digests must match recomputed canonical digests.
- `manifest.artifact_id` must match the recomputed artifact digest.
- If `adapter.json` is present, it must parse as JSON, but it is not part of core hash validation.
