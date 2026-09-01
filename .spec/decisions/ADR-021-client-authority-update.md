# ADR-021: Client Authority Update Transaction

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.2-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (transaction API), `LumioClient` (submit/ack orchestration only)
- **Baseline**: `LGE-V1.2-2026-08-27`
- **Relation**: Refines [ADR-005](ADR-005-replication-prediction.md) with an explicit Runtime transaction boundary; does not reorder the §7.2 step chain.

## Context

ADR-005 and Architecture §7.2 already name the client authority-update step order, but they do not publish a single Runtime transaction API. Independent Restore/Apply/Replay entry points would let a client advance Baseline, Revision, Confirmed Point or Ack after a partial apply. LumioClient cannot start implementation until the commit visibility and unknown-result fault class are public.

## Decision

`LumioGameRuntime` publishes one client authority-update transaction. FullSnapshot, Delta and Resync share that boundary. The step order is fixed and matches Architecture §7.2:

1. `ValidateBaselineAndRevision`
2. `RestoreConfirmedPredictionFrame`
3. `ApplyAuthoritativeEcsGasVoxel`
4. `DropConfirmedCommands`
5. `ReplayUnconfirmedInOrder`
6. `EmitPresentationDiff`

Commit is all-or-nothing: only `Committed` makes replica-world writes, Baseline/Revision/Confirmed Point and Ack visible. Any failed step yields `Aborted` with zero visible side effects. If the Runtime cannot prove whether apply committed or rolled back, the result is `Indeterminate` and the attested `FaultClass` is:

- `SessionLocalProven` only when the Runtime proves replica worlds were not committed or were rolled back.
- `SlotStateUnproven` when replica-world integrity is unproven; the `ClientReplicaSession` must enter `Faulted` and recover by Full Resync or session restart. This is the client-side meaning of the same ID; it does not authorize a server WorldSlot recovery from a client fault.
- `ProcessFault` for process-level collapse.

The Host/Client never infers success from a missing result. Independent Restore/Apply/Replay APIs are not a substitute for this transaction.

## Contract

`client-authority-update.schema.json` records `updateKind`, the fixed `stepOrder`, per-step results, `state`, visibility flags and `faultClass`. Runtime owns ReplicaWorld and VoxelReplicaWorld storage; the client submits a staged plan and may advance session metadata only after `Committed`.

## Failure semantics

A rejected plan or failed step returns `Aborted` + `SessionLocalProven` and forbids Ack. An unknown commit result returns `Indeterminate` + `SlotStateUnproven` or `ProcessFault`. Visible side effects on any non-Committed state are a contract violation.

## Alternatives

Composing Restore/Apply/Replay as three public calls was rejected because a crash between them is indistinguishable from success. A different step order for Resync was rejected because it would fork the prediction/rollback unit.

## Compatibility and migration

Additive in `LGE-V1.2-2026-08-27`. No deployed client binary consumes a public Restore/Apply/Replay split. Future step insertions require a new ADR; reordering is forbidden.

## Verification

Fixtures `authority/committed` (positive) and `authority/visible-on-abort` (failure: Abort with visible side effects). Runtime must also test crash-between-steps classifying `Indeterminate`.
