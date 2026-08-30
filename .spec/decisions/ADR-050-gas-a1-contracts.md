# ADR-050: GAS A1 Lifecycle, Evaluation and Effect Contracts

- **Status**: Draft
- **Owner**: `LumioGameEngineArchitecture` (contract), `LumioGameRuntime` (lifecycle consumer)
- **Baseline**: `LGE-V1.4-2026-08-27`
- **Relation**: Refines [ADR-008](ADR-008-gas-state.md) and [ADR-031](ADR-031-gas-lifecycle.md)

## Context

The frozen GAS V1.4 architecture specifies lifecycle admission, deterministic evaluation and same-Tick Effect ordering, but the prior lifecycle schema only represented a generic transition event. Runtime, NativeCore and Game need one additive contract surface with machine-validated failure behavior.

## Decision

- Ability states remain exactly `Requested`, `Activated`, `Executing`, `Completed`, `Rejected`, `Cancelled`, `Expired` and `RolledBack`; Effect states remain exactly `Pending`, `Active`, `Expired`, `Removed`, `Rejected` and `RolledBack`. Existing legal transitions remain explicit, and every terminal transition requires `handleValid=false`.
- Admission records declare the ordered checks `HandlePermission -> Cooldown -> Cost -> Tag -> GameCustom`; each declared check ordinal matches its list position, the first failed check produces `Rejected` and no charge. Commit records recheck only `Cooldown -> Cost` with matching ordinals; a failed recheck produces `Cancelled` atomically with no charge.
- Prepared and CommitIntent records cannot carry a later business rejection or more than one charge. Commit charging is represented by an explicit bounded `chargeCount`.
- Evaluation is one channel using `(Base + SigmaAdd) * (1 + SigmaPercent)`, additive percentage aggregation, and explicit numeric priority followed by descending sequence as the override tie-break.
- Effect same-Tick events order `Hit -> Overflow -> SnapshotReplacement/Stack -> Duration -> Period -> Removal`. `Suppress` is an Active-internal event; applying and removing in one Tick has `Cancelled` outcome. Duration and period values are Tick numbers.

## Contract

The source contracts are `schemas/gas-lifecycle.schema.json`, `schemas/gas-evaluation.schema.json` and `schemas/gas-effect-events.schema.json`. Positive and negative records are registered in `fixtures/index.json`; `tools/lumio_contract.py validate` is the single cross-field semantic gate.

## Failure semantics

Unknown states, illegal transitions, wrong check/event order, unsupported operators, ambiguous overrides, non-finite results, wall-clock timing fields, invalid suppression transitions and terminal handles left valid are rejected deterministically.

## Compatibility and migration

This is additive within the existing baseline: the original generic lifecycle transition shape remains valid, no public state or numeric ID is added, and no implementation repository is changed. Consumers should read the generated contract artifacts and migrate to the new record kinds before relying on Admission, Commit or EffectTick fields.

## Verification

The registered GAS positive and negative fixtures, the full contract validator, schema lint and deterministic generation are the acceptance evidence for this ADR.
