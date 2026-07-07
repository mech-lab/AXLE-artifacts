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

## Milestone 3 — Verification-First AXLE Adapter

Status: completed

This milestone completes the first coherent adapter layer between upstream AXLE outputs and AXLE-rs artifacts.

- live `axle-rs build` path using AXLE `check` and `extract_decls`
- live `axle-rs verify-proof` path using AXLE `verify_proof` and `extract_decls`
- a unified `.axle` format for build and proof-verification artifacts
- hashed `verification.json` summaries for proof-verification artifacts
- auxiliary raw request and response preservation via `adapter.json`

## Milestone 4 — Merkle Object Graphs

Status: completed

This milestone turns flat artifact hashing into an explicit derived object model suitable for deeper systems work while keeping the persisted `.axle` format unchanged.

- derived-first Merkle graph model over verified artifact core data
- `axle-rs graph` with JSON and DOT export
- `axle-rs diff` with summary-first text and JSON output
- declaration-centric node model covering source, environment, verification, declarations, statements, bodies, and diagnostics

## Milestone 5 — Receipt Binding

Status: planned

This milestone introduces attestations about artifacts without collapsing artifacts and receipts into the same object. It depends on the current artifact and verification split remaining stable.

- unsigned receipt format first
- digest binding between artifact and receipt
- later signature and policy hooks

## Milestone 6 — WASM And Distribution Surfaces

Status: planned

This milestone explores lightweight verification and inspection surfaces for downstream environments.

- WASM-facing inspection and verification targets
- browser or sandbox-friendly artifact tooling
- future registry-facing distribution work
