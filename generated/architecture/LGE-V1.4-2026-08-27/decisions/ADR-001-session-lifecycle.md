# ADR-001: Session Ownership, World Lifecycle and Clock Split

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (logical state), `LumioServer`/`LumioClient` (host clock)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

The earlier review used `Session`, `World` and `Host` interchangeably. That makes pause, maintenance, recovery and LocalEmbedded behavior ambiguous. A server process must also serve multiple products and release pools without sharing authoritative state.

## Decision

`WorldSlotHost` is the single Host-side aggregate root: it owns process-facing host resources, the Host Admission Gate, Wall Clock, pacing start/stop, the quiesce/drain/snapshot/stop sequence and the host lifecycle epoch. `SimulationSession` in Runtime owns Logical `TickId`, Phase Graph, GameWorld and the Coordinator. VoxelEngine owns each VoxelWorld. A client owns a separate `ClientReplicaSession`; LocalEmbedded places server and client trees in one process but never shares World, Storage, Entity or object references.

Host subcomponents (admission execution, pacing scheduling, transport, persistence, maintenance progress) may hold internal state, but only the aggregate owner initiates a host-aggregate transition. Subcomponents execute typed commands issued by the aggregate owner and report explicit acknowledgments. Every aggregate transition carries the lifecycle epoch; a command or acknowledgment stamped with an older epoch is rejected with the stable error `StaleEpoch` and cannot mutate the aggregate.

The state machines in Architecture v1.1 are normative. Only the owner may initiate a transition. A failed initialization, unload, recovery or partially completed multi-step transition enters `Faulted` and cannot leave a half-live object. V1 loads one `GameReleaseId` per process; multiple products/releases use separate processes or ReleasePools.

Server-side per-connection records (connection identity, admission result, reconnect retention, slot association) are host-private state owned by the server host; they must not be named, modeled or documented as the client-owned `ClientReplicaSession` machine and never cross the wire.

## Contract

The owner and clock split are represented by the lifecycle diagrams in Architecture v1.1 section 3. `SessionRevisionVector` is the shared snapshot identity. Host calls Runtime at a Tick boundary; Runtime never reads a Wall Clock directly.

## Failure semantics

Ingress closes before pause, drain or destruction. In-flight work is completed or explicitly aborted, then evidence and a SnapshotCut are written. A stale callback after disposal is rejected with a stable fault and cannot mutate a new session.

## Alternatives

Sharing one World between Local roles was rejected because it bypasses authority, replication and prediction tests. Letting Runtime own Wall Clock was rejected because it couples simulation to platform scheduling and maintenance.

## Compatibility and migration

Adding a lifecycle state or changing ownership requires a new ADR and Baseline. Existing v0.3 consumers follow the deprecated pointer to v1.0; no runtime migration is implied.

## Verification

Use `fixtures/valid/session-revision-vector.json`, plus the negative revision fixture, and host tests for every transition, duplicate dispose, callback-after-dispose, stale-epoch command rejection and LocalEmbedded two-tree isolation.
