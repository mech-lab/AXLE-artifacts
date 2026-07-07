# `axle.receipt.v0`

This document reserves the receipt-binding layer for a later AXLE-rs milestone. The intent is to let verifiers, policies, and eventually signatures speak about a specific artifact digest without collapsing artifact content and attestation content into one file format.

## Scope

- Bind a verification statement to a specific `axle.artifact.v0` digest.
- Keep receipts separate from artifact bundles.
- Support unsigned receipts first.

## Deferred work

- Signature encoding
- Policy identifiers
- Environment capture policy
- Receipt verification CLI flows
