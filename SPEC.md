# Specification Index

- [artifact-v0](spec/artifact-v0.md)
- [manifest schema](spec/manifest-v0.schema.json)
- [receipt-binding-v0](spec/receipt-binding-v0.md)
- [merkle-dag-v0](spec/merkle-dag-v0.md)

AXLE-rs is organized around three linked specification threads. `artifact-v0` defines the portable proof-bundle shape that exists today. `receipt-binding-v0` reserves the later attestation layer that can speak about an artifact without becoming the artifact. `merkle-dag-v0` sketches the longer-term object model needed for diffing, deduplication, registry storage, and richer proof-corpus infrastructure. The current implementation is intentionally concentrated in artifact v0 first.
