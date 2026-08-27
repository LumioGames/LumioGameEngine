# ADR-001: Session Ownership, World Lifecycle and Clock Split

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioGameRuntime` (logical state), `LumioServer`/`LumioClient` (host clock)
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

The earlier review used `Session`, `World` and `Host` interchangeably. That makes pause, maintenance, recovery and LocalEmbedded behavior ambiguous. A server process must also serve multiple products and release pools without sharing authoritative state.

## Decision

`WorldSlotHost` owns process resources, admission, Wall Clock, pacing and host lifecycle. `SimulationSession` in Runtime owns Logical `TickId`, Phase Graph, GameWorld and the Coordinator. VoxelEngine owns each VoxelWorld. A client owns a separate `ClientReplicaSession`; LocalEmbedded places server and client trees in one process but never shares World, Storage, Entity or object references.

The state machines in Architecture v1.0 are normative. Only the owner may initiate a transition. A failed initialization, unload or recovery enters `Faulted` and cannot leave a half-live object. V1 loads one `GameReleaseId` per process; multiple products/releases use separate processes or ReleasePools.

## Contract

The owner and clock split are represented by the lifecycle diagrams in Architecture v1.0 section 3. `SessionRevisionVector` is the shared snapshot identity. Host calls Runtime at a Tick boundary; Runtime never reads a Wall Clock directly.

## Failure semantics

Ingress closes before pause, drain or destruction. In-flight work is completed or explicitly aborted, then evidence and a SnapshotCut are written. A stale callback after disposal is rejected with a stable fault and cannot mutate a new session.

## Alternatives

Sharing one World between Local roles was rejected because it bypasses authority, replication and prediction tests. Letting Runtime own Wall Clock was rejected because it couples simulation to platform scheduling and maintenance.

## Compatibility and migration

Adding a lifecycle state or changing ownership requires a new ADR and Baseline. Existing v0.3 consumers follow the deprecated pointer to v1.0; no runtime migration is implied.

## Verification

Use `fixtures/valid/session-revision-vector.json`, plus the negative revision fixture, and host tests for every transition, duplicate dispose, callback-after-dispose and LocalEmbedded two-tree isolation.
