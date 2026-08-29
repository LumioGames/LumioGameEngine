# ADR-034: Hot Reload Dual-Scope Activation

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (barrier), Host (ALC), `LumioGame` (migration nodes)
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-013](ADR-013-migration-dag.md). The Accepted ADR-013 Decision text is unchanged.

## Context

ADR-013 stages an immutable snapshot and atomically activates a pointer, but it does not name the dual Gameplay Scope machine or the post-switch failure rule.

## Decision

Dual-scope states are exactly:

`OldActive + NewStaging -> NewValidated -> BarrierSwitch -> OldQuiescing -> OldUnloaded`

- Failure before `BarrierSwitch`: discard `NewStaging`; `OldActive` is unchanged.
- Failure after `BarrierSwitch`: do not reactivate a quiesced or disposed old Scope; the Session is `Faulted` and recovers from a valid Snapshot/Release.
- Migration nodes read only the immutable Snapshot and write results in Staging.
- Ingress, subscriptions, timers, tasks and Native leases are linearized by Scope Generation at the switch.

`gameplay-scope-activation.schema.json` records the stage, generations and recovery action. Migration manifests add `scopeActivation` describing the same machine.

## Contract

`gameplay-scope-activation.schema.json` and `migration-manifest.schema.json`.

## Failure semantics

A record that reactivates an unloaded old Scope, or that mutates the source snapshot, is invalid.

## Alternatives

Unload-then-create was rejected because a failed new Scope would have no old pointer.

## Compatibility and migration

Additive in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `scope/barrier-switch`, `scope/reactivate-old`, `migration/dag` (with `scopeActivation`).
