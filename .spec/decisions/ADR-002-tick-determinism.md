# ADR-002: Tick Phases, Processor Scheduling and Determinism

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Refined by**: [ADR-027](ADR-027-tick-fail-stop.md), [ADR-030](ADR-030-processor-structural-commands.md)

## Context

The reports proposed incompatible phase counts and left parallel writes, late input and replay guarantees undefined. Deterministic replay needs a stable semantic order even when implementation workers differ.

## Decision

V1 exposes the 13 phases in Architecture v1.0 section 4, from `IngressCapture` through `EgressPublish`. Internal fusion is permitted only when observable ordering is unchanged. One Simulation Owner Thread commits authoritative state per active WorldSlot. Processors declare `ReadSet`, `WriteSet`, structural writes, dependencies, determinism class and budget using `processor-descriptor.schema.json`.

Only disjoint write sets may run concurrently, and reductions use a stable merge order. Network, IO, platform callbacks and Native workers write bounded queues; they never mutate a World directly. Late input is classified as current tick, next tick or rejected.

## Contract

`ProcessorId + Phase + LocalSequence` defines command merge order. RNG stream, time units, integer/floating rules, event ordering and canonical hash inputs are part of a Release Contract. Level 1 promises bit-level replay on the same platform/binary; Level 2 promises semantic consistency and first-difference diagnostics across profiles.

## Failure semantics

Cycle or read/write conflict rejects the Processor plan before a Tick. Queue overflow applies its declared priority policy and emits a diagnostic; it cannot silently grow. A budget overrun is attributed to the Processor and follows Host policy without applying partial structural writes.

## Alternatives

Unordered parallel ECS was rejected for authoritative writes. A single monolithic phase was rejected because it prevents audit, barriers and deterministic replay.

## Compatibility and migration

Changing phase meaning, merge order or determinism class is a protocol/replay break and requires a new `SchemaEpoch` and migration policy. Storage layout and worker count remain private.

## Verification

`fixtures/valid/processor-place-voxel.json` and `fixtures/invalid/processor-read-write-conflict.json` cover declaration and rejection. Add replay golden files with identical command streams and a first-difference Tick assertion.
