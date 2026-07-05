# Roadmap

## Milestone 1

- Add fork notice and repository positioning.
- Stand up a Rust workspace with the initial crate split.
- Ship a minimal `axle-rs` CLI with `--version` and `artifact new`.

## Milestone 2

- Define `.axle` directory artifact v0.
- Implement deterministic load, save, inspect, hash, and verify flows.
- Add golden fixtures and round-trip tests.

## Milestone 3

- Add an AXLE adapter crate for parsing upstream-style verification output.
- Convert AXLE verification responses into `.axle` artifacts.

## Milestone 4

- Extend hashing into a Merkle-style object graph.
- Add artifact diffing and graph export.

## Milestone 5

- Add unsigned receipt binding first.
- Reserve room for later signatures and policy handling.

## Milestone 6

- Add a WASM target for no-network artifact inspection and verification.

