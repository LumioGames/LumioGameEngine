# ADR-031: GAS Ability and Effect Lifecycle

- **Status**: Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (framework), `LumioGame` (content hooks only)
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-008](ADR-008-gas-state.md). The Accepted ADR-008 Decision text is unchanged. State names below are frozen and must not be added or removed.

## Context

ADR-008 assigned lifecycle ownership to Runtime but did not list states or legal transitions. Game content could invent incompatible Activated/Running names.

## Decision

Ability instance states are exactly: `Requested`, `Activated`, `Executing`, `Completed`, `Rejected`, `Cancelled`, `Expired`, `RolledBack`. Transitions:

- `Requested -> Activated -> Executing -> Completed`
- `Requested -> Rejected`, `Activated -> Rejected`
- any non-terminal -> `Cancelled`
- `Executing -> Expired`
- predicted instance rejected by authority -> `RolledBack`
- terminal states invalidate the Handle

Effect instance states are exactly: `Pending`, `Active`, `Expired`, `Removed`, `Rejected`, `RolledBack`. Transitions:

- `Pending -> Active -> Expired | Removed`
- `Pending -> Rejected`
- prediction rollback -> `RolledBack`
- Stack, Duration and Refresh are events inside `Active`, not states

Game content may define business substates only while Ability is `Executing` or Effect is `Active`. Content must not change the generic transitions, terminal semantics, rollback window or Handle invalidation.

## Contract

`gas-lifecycle.schema.json` and Architecture §9.

## Failure semantics

An undeclared state name or illegal transition is invalid.

## Alternatives

Delegating the generic machine to Game was rejected because Snapshot, Replication and Prediction need one recovery rule.

## Compatibility and migration

Additive in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `gas/ability-complete` and `gas/illegal-transition`.
