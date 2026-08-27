# ADR-005: Replication Baselines, Prediction and Resynchronization

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (semantics), `LumioServer`/`LumioClient` (transport adapters)
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Refined by**: [ADR-028](ADR-028-replication-typed-bodies.md)

## Context

Transport acknowledgements do not prove that a Client applied a replication baseline. The reviews also identified unknown baselines, gaps, asymmetric Components and the need to roll back ECS, GAS and Voxel overlays together.

## Decision

Handshake is followed by reliable `FullSnapshot`, a separate `BaselineAck`, then revisioned `Delta` messages. Each Mapping declares source/target field, Role, Owner, AOI, reliability, quantization, prediction and lifecycle behavior. Client validation restores the last confirmed PredictionFrame, atomically applies authoritative ECS/GAS/Voxel state, removes confirmed commands and replays the remainder in order.

Unknown baseline, gap, stale revision, schema mismatch, history exhaustion or Tombstone conflict requests Full Resync. LocalEmbedded uses the same envelope, serializer, limits and permission path, even when sockets are bypassed.

## Contract

`replication-envelope.schema.json` separates sequence, SnapshotId, BaseSnapshotId, revision range and reliability. Mapping schemas and generated tests are owned by Game but consumed by Runtime/Client.

## Failure semantics

Malformed or oversized envelopes are rejected before allocation. A gap never gets silently patched; a Resync request carries a stable reason. Prediction rejection produces a correction event and bounded history cleanup.

## Alternatives

Using Transport ACK as the baseline was rejected. Sending complete state every Tick was rejected for bandwidth and does not solve prediction ordering. Symmetric Server/Client Components were rejected because presentation and authority have different needs.

## Compatibility and migration

Protocol, MappingHash and Snapshot schema changes require a new Release or explicitly declared compatibility window. V1 uses exact release matching; N/N-1 support is a future ADR.

## Verification

Run FullSnapshot/Delta fixtures, gap-without-resync failure, packet loss/duplication/reordering, unknown baseline, reconnect and prediction correction tests.
