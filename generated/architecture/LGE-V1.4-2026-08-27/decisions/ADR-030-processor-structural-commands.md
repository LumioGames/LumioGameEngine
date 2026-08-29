# ADR-030: Processor Structural Commands and Self Overlap

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-002](ADR-002-tick-determinism.md). The Accepted ADR-002 Decision text is unchanged.

## Context

The validator required `structuralWrites` processors to sit in `EcsCommandBufferCommit` and rejected a Stable Processor whose ReadSet overlapped WriteSet. Gameplay processors emit structural commands in business phases; only the Runtime Commit Executor applies them. Self-overlap (read inventory, write inventory) is legal.

## Decision

Replace `structuralWrites` with `mayEmitStructuralCommands`. A business phase (`ApplyInputs`, `ProcessorPlan`, `CrossWorldPrepare`, `CommitDecision`, `GasAndEventFinalize`) may set it true. Only the Runtime Commit Executor applies those commands in `EcsCommandBufferCommit`. Declaring `mayEmitStructuralCommands` in a non-business phase (for example `SnapshotHashMetrics`) is invalid.

ReadSet/WriteSet self-overlap is allowed. The Scheduler validates conflicts *between* Processors, not within one Processor.

## Contract

`processor-descriptor.schema.json`.

## Failure semantics

A non-business phase that emits structural commands is rejected. A Stable self-overlapping Processor is accepted.

## Alternatives

Keeping the self-overlap ban was rejected because it forbids ordinary stat updates.

## Compatibility and migration

Field rename in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `processor/place-voxel`, `processor/stable-self-overlap`, `processor/structural-in-metrics`.
