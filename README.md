# AXLE-rs

AXLE-rs is an experimental Rust-native artifact layer for AXLE-compatible proof outputs.

This repository is a fork-oriented extension of `AxiomMath/axiom-lean-engine`. The upstream Python client, CLI, docs, and examples are preserved in place, while a Rust workspace is added for portable `.axle` artifacts, canonical hashing, CLI inspection and verification, and a path toward optional receipt binding.

The immediate goal of this fork skeleton is to define artifact v0, stand up a Rust workspace, and provide a minimal CLI for creating, inspecting, hashing, and verifying `.axle` directory bundles.

## Layout

- Upstream Python package and CLI live under `axle/`.
- Upstream docs and examples live under `docs/` and `examples/`.
- Rust workspace crates live under `crates/`.
- Artifact specifications live under `spec/`.
- The preserved upstream root README is copied to `UPSTREAM_README.md`.
