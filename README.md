# AXLE-rs

AXLE-rs is an experimental Rust-native artifact layer for AXLE-compatible Lean outputs.

This repository is a fork of `AxiomMath/axiom-lean-engine`. It preserves the upstream Python client, CLI, API-oriented documentation, and examples while adding a Rust workspace for portable `.axle` artifacts, canonical hashing, artifact inspection, and future proof-evidence infrastructure.

AXLE-rs is exploratory and not an official Axiom Math release unless explicitly stated otherwise.

## Why This Fork Exists

Upstream AXLE is already useful as proof-manipulation infrastructure: it checks Lean files, verifies candidate proofs, extracts declarations, and performs source-to-source transformations. That is the right place to ask questions such as "does this Lean development compile?" or "what declarations can I extract from this file?"

AXLE-rs explores the next systems layer: once AXLE has produced a result, how should that result become a durable artifact? The central question in this fork is not how to replace Lean or AXLE’s metaprogramming layer, but how to turn verification outputs into portable, inspectable, hash-bound objects that can be cached, transported, compared, and eventually attested.

That split is intentional:

- Upstream AXLE focuses on proof execution, checking, and transformation.
- AXLE-rs focuses on artifact representation, provenance, and downstream infrastructure.

## Current State

The current repository ships the first live AXLE-backed vertical slice. `axle-rs build` reads a Lean file, calls upstream AXLE `check` and `extract_decls`, converts the result into a `.axle` directory artifact, and preserves the raw AXLE API payloads as auxiliary metadata.

```bash
axle-rs build sample.lean \
  --environment lean-4.28.0 \
  -o sample.axle
```

Today’s artifact flow supports:

- `.axle` directory bundles with `manifest.json`, `source.json`, `declarations.json`, `diagnostics.json`, and `hashes.json`
- deterministic artifact IDs based on canonical JSON hashing
- `inspect`, `verify`, and `hash` commands for local artifact introspection
- optional `adapter.json` metadata containing raw upstream AXLE `check` and `extract_decls` responses

`adapter.json` is intentionally excluded from the artifact digest so AXLE timing differences and other volatile response fields do not destabilize content addressing.

## Research Goals

The long-term goal is to make proof outputs easier to treat as systems artifacts rather than ephemeral API responses. AXLE-rs is interested in proof bundles that are durable enough for research pipelines, reproducible evaluation, artifact exchange, and eventually registry-backed storage.

The research program in this fork points toward:

- portable proof artifacts that survive outside a single API call or runtime
- content-addressed object models for deduplication and comparison
- deterministic serialization suitable for caching and corpus construction
- artifact diffing and dependency-graph inspection
- receipt binding and later signature/attestation layers
- registry- and WASM-friendly verification surfaces for downstream tooling

## System Model

AXLE-rs currently assumes a layered pipeline:

```text
Lean source
   ↓
AXLE checking / extraction
   ↓
.axle artifact
   ↓
optional receipt / registry / verifier layers
```

In the present slice, AXLE remains the source of truth for processed Lean content and declaration extraction, while AXLE-rs is responsible for normalizing those outputs into a stable artifact form.

## Roadmap At A Glance

The roadmap is incremental rather than revolutionary.

- Milestone 1: fork bootstrap and Rust workspace foundation — completed
- Milestone 2: artifact v0 directory format, hashing, inspect/verify flows — completed
- Milestone 3: first AXLE-backed build slice — landed, with further adapter growth planned
- Milestone 4: Merkle-style object graphs, diffing, and graph export — planned
- Milestone 5: receipt binding and verification workflows — planned
- Milestone 6: WASM-facing inspection and verification surfaces — planned

The detailed status view lives in [ROADMAP.md](ROADMAP.md).

## Repo Guide

This repository now contains two overlapping but distinct documentation and implementation surfaces.

- `axle/`, `docs/`, and most of `examples/` preserve the upstream AXLE Python/API/CLI surface.
- `crates/` contains the Rust implementation of AXLE-rs.
- `spec/` contains the evolving artifact, receipt, and Merkle-DAG design notes for the fork.
- `examples/artifacts/` contains example `.axle` output.

The preserved `docs/` tree is still upstream AXLE-facing documentation. It should be read as the operational reference for the Python client, hosted API, and existing AXLE tools, not as a complete description of the AXLE-rs fork vision.

## Documentation Map

- [ROADMAP.md](ROADMAP.md): milestone status and near-to-mid-term implementation direction
- [SPEC.md](SPEC.md): spec index for artifact, receipt, and Merkle-DAG work
- [FORK.md](FORK.md): fork notice and relationship to the upstream project
- [UPSTREAM_README.md](UPSTREAM_README.md): preserved snapshot of the upstream root README
- [docs/](docs/): preserved upstream AXLE documentation for the Python/API/CLI surface
