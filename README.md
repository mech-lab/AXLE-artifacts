# AXLE-rs

AXLE-rs is an experimental Rust-native artifact layer for AXLE-compatible proof outputs.

This repository is a fork-oriented extension of `AxiomMath/axiom-lean-engine`. The upstream Python client, CLI, docs, and examples are preserved in place, while a Rust workspace is added for portable `.axle` artifacts, canonical hashing, CLI inspection and verification, and a path toward optional receipt binding.

The current vertical slice adds the first live AXLE-backed flow: `axle-rs build` can call upstream AXLE `check` and `extract_decls`, convert the result into a `.axle` directory bundle, and preserve the raw AXLE API responses as auxiliary metadata.

## Build

```bash
axle-rs build sample.lean \
  --environment lean-4.28.0 \
  -o sample.axle
```

The resulting artifact contains deterministic hashed core files plus an optional `adapter.json` file with raw upstream AXLE `check` and `extract_decls` responses. `adapter.json` is intentionally excluded from the artifact digest so AXLE timing differences do not destabilize content addressing.

## Layout

- Upstream Python package and CLI live under `axle/`.
- Upstream docs and examples live under `docs/` and `examples/`.
- Rust workspace crates live under `crates/`.
- Artifact specifications live under `spec/`.
- The preserved upstream root README is copied to `UPSTREAM_README.md`.
