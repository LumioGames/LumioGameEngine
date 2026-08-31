# ADR-027: Tick Fail-Stop and Phase Contract Matrix

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime`
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-002](ADR-002-tick-determinism.md). The Accepted ADR-002 Decision text is unchanged.

## Context

ADR-002 forbids partial structural writes on budget overrun but does not say what happens to in-place component field writes when a Processor throws, is cancelled or exceeds budget. A 13-phase visibility matrix was required and missing.

## Decision

V1 uses **Fail-stop**. Processors may write existing component fields in place. Any pre-commit fault (Processor exception, cancel, or over-budget abort) makes the current World unusable: the `SimulationSession` becomes `Faulted` and is rebuilt from the pre-Tick Snapshot plus Journal. There is no field-level undo.

The unique authoritative Tick Commit Point is `GasAndEventFinalize`. Before that point a fault discards the in-progress World. After that point the Tick result is visible to Replication, Snapshot hash and Egress. Repeating the same Tick with the same canonical inputs is idempotent and yields the same Tick result.

`tick-phase-contract.schema.json` freezes the 13-phase matrix: inputs, writable domains, failure class, cancel point, over-budget action, visibility to later phases, repeat-Tick result and the single Commit Point flag.

## Contract

Architecture §4.5 and `tick-phase-contract.schema.json`.

## Failure semantics

A matrix that marks zero or more than one Commit Point, or that allows later-phase visibility before the Commit Point, is invalid. Fail-stop never reports a partial Tick as committed.

## Alternatives

Staged WriteSet/Overlay publish was rejected for V1 implementation cost; it remains a future ADR. Field-level undo journals were rejected as a second write path.

## Compatibility and migration

Additive in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `tick/phase-matrix` and `tick/duplicate-commit-point`.
