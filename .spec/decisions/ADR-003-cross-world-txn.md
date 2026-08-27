# ADR-003: CrossWorldTxnV1, Revision and SnapshotCut

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (Coordinator), `LumioVoxelEngine` and Runtime (participants)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

Gameplay actions such as placing a block update Game/ECS resources and Voxel state together. XA/2PC would introduce distributed locks and failure modes that do not fit the single-process Tick model, while validate-then-apply alone cannot recover a crash between participants.

## Decision

Use a single-owner `CrossWorldTxnV1` with Tick Barrier, Expected Revisions, lease-backed Reservation, idempotent Apply and a durable `TxnJournal`. Prepare has no visible side effect. Before the first participant write, persist `CommitIntent`; apply in the fixed order `VoxelCommit` then `EcsCommandBufferCommit`; append a marker after each participant and `Committed` after both. Query resolves `Indeterminate` after recovery. No generic XA/2PC is part of V1.

## Contract

`cross-world-txn.schema.json` defines `Created`, `Prepared`, `CommitIntent`, `Committed`, `Aborted` and `Indeterminate`. Every result carries a `SessionRevisionVector`; reads return the revision observed. `GameRevision`, `VoxelWorldRevision`, `ChunkRevisionSet` and `ReplicationRevision` are distinct.

## Failure semantics

Revision conflict, unloaded Chunk, permission/resource failure, timeout and cancellation abort without visible writes. A crash between participant commits yields `Indeterminate`; recovery consults journal markers and replays only missing idempotent steps. Duplicate `TxnId` returns the original result and never charges resources twice.

## Alternatives

Generic distributed 2PC was rejected for lock duration and operational complexity. Saga compensation was rejected for non-compensatable voxel/resource effects. Validate-then-apply without journaling was rejected for crash ambiguity.

## Compatibility and migration

Commit order, state names and revision meanings are wire/persistence semantics. A change requires a new transaction schema epoch and a migrator for journal records. Participant storage remains private.

## Verification

Use committed, revision-conflict and partial-commit fixtures. Inject duplicate command, lost result, deadline, Chunk-not-loaded and process crash at each journal boundary; assert replay and no double debit.
