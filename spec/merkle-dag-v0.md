# Merkle DAG v0

This document defines the current derived-first graph layer used by AXLE-rs for `graph` export and artifact-to-artifact diffing. It carries AXLE-rs beyond flat bundle hashing into a richer content-addressed proof-object model without changing the persisted `.axle` bundle format.

## Scope

- Define declaration-centric node kinds over existing artifact v0 data.
- Specify stable derived node hashing rules.
- Support JSON graph export, DOT graph export, and summary-first artifact diffing.

## Derived-First Storage Policy

The Merkle graph is not stored inside `.axle` in v0. It is derived from verified artifact core data at read time.

- `adapter.json` is excluded from graph derivation.
- No `graph.json`, `merkle.json`, or new manifest object is added in this milestone.
- Existing `artifact_id` semantics remain unchanged and continue to identify the graph root.

## Node Kinds

The current graph model uses these node kinds:

- `artifact`
- `source`
- `environment`
- `verification`
- `declaration`
- `statement`
- `body`
- `diagnostic`

External dependencies are not modeled as nodes in this milestone because the current artifact core stores only local declaration dependency names.

## Node Identity Rules

- `artifact` node ID = existing `artifact_id`
- `source` node ID = digest of `source.json`
- `environment` node ID = digest of normalized environment metadata
- `verification` node ID = digest of `verification.json`
- `statement` node ID = digest of a payload that contains the statement digest reference
- `body` node ID = digest of a payload that contains the body digest reference
- `diagnostic` node ID = digest of the diagnostic payload

Declaration nodes are real Merkle nodes. Their payload includes only stable declaration metadata:

- declaration name
- declaration kind
- declaration verification status

Each declaration node hash is computed from canonical JSON of:

- `kind`
- `label`
- `payload`
- `edges`

## Edges And Ordering

The root artifact node references:

- source
- environment
- verification when present
- every declaration node
- every diagnostic node

Declaration nodes reference:

- one statement node when a statement digest is present
- one body node when a body digest is present
- zero or more local declaration dependency nodes

Edges are sorted deterministically by `(label, target_id)`.

If a local declaration dependency name does not resolve to another declaration in the same artifact, graph derivation fails. Cyclic local declaration dependencies also fail in the current implementation because declaration node hashes depend on dependency targets.

## Diff Surface

`axle-rs diff` compares two verified artifacts by first deriving both graphs and then reporting:

- root change
- source change
- environment change
- verification summary change
- added declarations
- removed declarations
- changed declarations

For changed declarations, the current summary identifies changes in:

- statement target
- body target
- verification status
- local dependency set

## Deferred Work

- persisted graph storage inside artifacts
- external dependency nodes
- deeper proof- or syntax-level node granularity
- registry addressing and graph transport
