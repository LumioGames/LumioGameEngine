# ADR-010: Persistence, Canonical Serialization and Config Snapshots

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (format contract), `LumioVoxelEngine`/`LumioGame` (domain schemas), `LumioServer` (host durability)
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Refined by**: [ADR-032](ADR-032-durable-recovery-records.md), [ADR-033](ADR-033-config-typed-columns.md)

## Context

Save/Load, replay and migration require deterministic bytes and safe rejection of corrupt input. Configuration must not change halfway through a Tick. The reports specifically called out serialization/deserialization and table reading as first-class requirements.

## Decision

Snapshots and WAL/Command Logs use versioned Canonical `Encode`/`Decode`, with Magic, SchemaVersion, Length, Hash/Checksum, Compression and optional encryption metadata. Decode validates bounds and metadata before materializing typed state; unknown required fields, duplicates, truncation and decompression bombs are rejected. Snapshot writes use staging, verification, fsync/atomic activation and retention of the last valid checkpoint.

Human-readable config is schema-validated, defaults merged in the fixed Engine -> Platform -> Server -> Product -> Environment -> User/Session order, then compiled to typed binary tables. A Tick receives an immutable ConfigSnapshot; production switches a signed version only at a Tick boundary. Secrets are separate from ordinary tables.

## Contract

`snapshot-header.schema.json` and `config-table.schema.json` define the minimum envelope. Domain payload schemas are owned by Voxel/Game and reference the same canonical codec rules.

## Failure semantics

Corrupt, incompatible or over-budget data never becomes active and leaves the previous snapshot untouched. Recovery starts at the last valid checkpoint and replays only authenticated, committed log records. A failed migration leaves staging evidence and the source snapshot intact.

## Alternatives

Runtime object graphs/addresses were rejected for nondeterminism. JSON as the authoritative hot format was rejected for size, ordering and validation cost. In-place migration was rejected because a crash could destroy the only valid save.

## Compatibility and migration

SchemaVersion and SchemaEpoch identify format breaks. Every breaking release ships a migrator and old/new golden fixtures. Storage backend changes are adapters and do not change the canonical bytes.

## Verification

Run active/length-mismatch snapshot fixtures, round-trip and old-version golden tests, corruption/fuzz/decompression-limit tests, atomic activation crash tests and config duplicate-key/priority tests.
