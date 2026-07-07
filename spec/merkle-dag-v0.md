# Merkle DAG v0

This document reserves the object-graph design that would carry AXLE-rs beyond flat bundle hashing into a richer content-addressed proof-object model. It is the planned bridge from artifact v0 toward diffing, deduplication, and registry-oriented storage.

## Scope

- Define object kinds for source, environment, declarations, statements, proofs, and diagnostics.
- Specify stable node hashing rules.
- Enable future diffing, deduplication, and registry storage.

## Deferred work

- Canonical node encoding
- Dependency ordering rules
- Root object selection
- Registry addressing
