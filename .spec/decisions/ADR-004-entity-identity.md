# ADR-004: Entity Identity, Tombstones and Ownership Revision

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Refined by**: [ADR-029](ADR-029-entity-namespace-required.md)

## Context

Server and Client have independent ECS storage, and prediction can create provisional entities. Reusing an array index or a destroyed network id lets delayed deltas resurrect the wrong object.

## Decision

`NetEntityId` is a 128-bit opaque value whose versioned logical fields are AuthorityDomain, WorldEpoch, Sequence and Generation. It is never an array index. `LocalEntityId` is an Index+Generation valid only inside one ECS World. A Session never reuses a destroyed NetEntityId; a Tombstone survives through the relevant Baseline Ack/history window. Client provisional ids use a separate namespace and an explicit confirmation remap. Respawn uses a new id; Authority Transfer is reserved for later.

## Contract

`entity-identity.schema.json` describes namespace, lifecycle and tombstone horizon. Exact bit allocation is a generated schema artifact, not a hand-coded assumption. Mapping and Replay preserve the opaque id.

## Failure semantics

Unknown, stale or tombstoned ids cause a no-op plus diagnostic, never resurrection. Generation/context mismatch returns a stable invalid-handle error. A provisional remap is atomic across prediction history and ReplicaWorld.

## Alternatives

Session-local integer ids without tombstones were rejected. Reusing ids after a short wall-clock timeout was rejected because network delay and replay windows are logical, not wall-clock based.

## Compatibility and migration

Changing width, namespace or reuse rules breaks snapshots and wire mappings. New epochs and an explicit id remap table are required; old snapshots remain readable until their retention window expires.

## Verification

Run tombstone, provisional-remap and reused-tombstone fixtures with delayed Delta, reconnect, replay and duplicate spawn tests.
