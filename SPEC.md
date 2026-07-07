# Specification Index

- [artifact-v0](spec/artifact-v0.md)
- [manifest schema](spec/manifest-v0.schema.json)
- [receipt-binding-v0](spec/receipt-binding-v0.md)
- [merkle-dag-v0](spec/merkle-dag-v0.md)

AXLE-rs is organized around three linked specification threads. `artifact-v0` defines the portable proof-bundle shape that exists today, including both build-oriented artifacts and proof-verification artifacts with hashed `verification.json` summaries. `merkle-dag-v0` now defines the derived-first graph layer used for `graph` and `diff`, without introducing new persisted artifact files. `receipt-binding-v0` reserves the later attestation layer that can speak about an artifact without becoming the artifact. Receipts and distribution work remain deferred even though the Merkle graph layer is now implemented.
