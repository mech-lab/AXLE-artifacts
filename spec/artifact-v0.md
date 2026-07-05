# `axle.artifact.v0`

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
└── hashes.json
```

`receipt.json` is intentionally excluded from artifact v0. Receipts bind to an artifact later and stay conceptually separate.

## Required files

- `manifest.json`
- `source.json`
- `declarations.json`
- `diagnostics.json`
- `hashes.json`

## Hashing rules

- Object digests use `sha256`.
- Canonicalization uses recursively sorted JSON object keys.
- The artifact digest is computed from the full artifact body with `manifest.artifact_id` omitted.

## Manifest fields

- `schema`: must be `axle.artifact.v0`
- `artifact_id`: content-derived digest of the artifact body
- `producer`: artifact producer metadata
- `source`: language and digest summary for `source.json`
- `environment`: Lean and engine environment summary
- `objects`: relative file names for bundled objects

## Validation rules

- Every required file must exist.
- `manifest.schema` must match the v0 schema string.
- Stored digests must match recomputed canonical digests.
- `manifest.artifact_id` must match the recomputed artifact digest.

