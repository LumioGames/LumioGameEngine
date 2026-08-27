# ADR-026: CrossWorldTxn CommandBuffer and Participant Markers

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (Coordinator, CommandBuffer)
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-003](ADR-003-cross-world-txn.md). The Accepted ADR-003 Decision text is unchanged; this ADR is the normative participant/CommandBuffer contract.

## Context

ADR-003 requires all business checks in Prepare and forbids Apply-time business rejection, but it does not name the ECS CommandBuffer states or encode an Apply-succeeded-marker-unpersisted window. Boolean participant markers cannot represent `Unknown`.

## Decision

CommandBuffer states are exactly `Open -> Sealed -> Merged -> Prepared -> Applied`. During `CrossWorldPrepare` the ECS participant finishes Generation, target existence, component capacity, command-conflict, permission and budget checks and produces an immutable `PreparedGameDelta`. After `CommitIntent` is persisted, ECS Apply may return only `Applied`, `AlreadyApplied`, `Indeterminate` or `Faulted`. A business reject after CommitIntent is not a legal contract object.

Transaction graph: pre-intent failures are `Prepared -> Aborted` or `Prepared -> Expired`. `Indeterminate` is reachable only from an Apply phase that already persisted `CommitIntent`.

Participant markers are `NotStarted | Unknown | Applied | Failed` (not booleans). Recovery:

1. No persisted CommitIntent: abort or expire; do not Apply.
2. CommitIntent persisted and marker `Applied`: treat as done; never re-Apply.
3. CommitIntent persisted and marker `NotStarted` or `Unknown`: query the participant. `Applied`/`AlreadyApplied` writes the marker. `Unknown` stays `Indeterminate` until a later query or journal replay. `Failed`/`Faulted` stays `Indeterminate` and recovers from Snapshot + journal.
4. Apply succeeded but the marker was not persisted: the participant receipt/status answers `Applied`/`AlreadyApplied`; the coordinator writes `Applied` and must not re-Apply (ADR-025 for Voxel; the same query rule for ECS).

## Contract

`cross-world-txn.schema.json` records `commandBufferState`, `ecsApplyResult`, enum `participantMarkers` and `Expired`.

## Failure semantics

A post-intent business reject is rejected by the Architecture Gate. Partial boolean-style commits remain invalid. Lost-result and marker-crash records must be expressible as `Indeterminate` with `Unknown` or mixed enum markers.

## Alternatives

Keeping boolean markers was rejected because the apply-vs-persist window is unrepresentable. Allowing Apply-time business reject was rejected because Voxel may already have committed.

## Compatibility and migration

Additive field replacement in `LGE-V1.3-2026-08-27`. No deployed journal consumer exists.

## Verification

Fixtures `txn/committed`, `txn/aborted-revision-conflict`, `txn/partial-commit`, `txn/indeterminate`, `txn/lost-result`, `txn/marker-crash`, `txn/post-intent-business-reject`.
