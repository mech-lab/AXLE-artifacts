# Roadmap

This roadmap tracks the fork from bootstrap through a fuller proof-artifact research stack. It is meant to describe implementation status, not just aspirations.

## Milestone 1 — Fork Bootstrap And Positioning

Status: completed

This milestone established the fork identity, preserved the upstream AXLE surface, and created the Rust workspace that AXLE-rs builds on.

- fork notice and repository positioning
- initial Rust crate layout
- minimal `axle-rs` CLI surface

## Milestone 2 — Artifact V0 Foundations

Status: completed

This milestone established the first `.axle` directory artifact model and the deterministic local tooling around it.

- artifact v0 directory format
- canonical hashing
- load, save, inspect, verify, and hash flows
- example artifact fixtures and round-trip tests

## Milestone 3 — AXLE-Backed Build Slice

Status: current slice landed; further expansion planned

This milestone begins the adapter layer between upstream AXLE outputs and AXLE-rs artifacts.

- live `axle-rs build` path using AXLE `check` and `extract_decls`
- conversion from AXLE responses into `.axle` bundles
- auxiliary raw response preservation via `adapter.json`

Still open within this milestone:

- broader AXLE endpoint coverage
- richer declaration metadata policy
- clearer separation between artifact build, proof verification, and later attestation flows

## Milestone 4 — Merkle Object Graphs

Status: planned

This milestone turns flat artifact hashing into an explicit object model suitable for deeper systems work.

- Merkle-style node model
- artifact diffing
- graph export and dependency inspection

## Milestone 5 — Receipt Binding

Status: planned

This milestone introduces attestations about artifacts without collapsing artifacts and receipts into the same object.

- unsigned receipt format first
- digest binding between artifact and receipt
- later signature and policy hooks

## Milestone 6 — WASM And Distribution Surfaces

Status: planned

This milestone explores lightweight verification and inspection surfaces for downstream environments.

- WASM-facing inspection and verification targets
- browser or sandbox-friendly artifact tooling
- future registry-facing distribution work
