# ADR-051: GAS A2 ECS Components, Tags, Replication and Prediction Contracts

- **Status**: Draft
- **Owner**: `LumioGameEngineArchitecture` (contract), `LumioGameRuntime` (consumer)
- **Baseline**: `LGE-V1.4-2026-08-27`
- **Relation**: Refines [ADR-021](ADR-021-client-authority-update.md), [ADR-031](ADR-031-gas-lifecycle.md) and the ECS architecture baseline

## Context

The GAS A1 contracts freeze lifecycle and Effect evaluation, but do not yet
describe the ECS component projection, Tag vocabulary handshake, field-level
replication boundary or frame prediction rollback. These boundaries must remain
additive and must not create a second state store, an RPC channel or a new
prediction state machine.

## Decision

- `gas-components` declares exactly `AbilityComponent`, `EffectComponent`,
  `AttributeComponent` and `TagComponent`. Presentation `fx_key` is an Effect
  entry field; `FxComponent` is forbidden.
- The Modifier ledger is only a derived view of `EffectComponent.entries`; it
  has no independent ECS or replication field and is never synchronized or
  persisted.
- Each row carries `typeId`, `instanceId`, an ECS row index and a Handle bound
  to `worldId`, index and generation. Stale, cross-world and terminal Handles
  are rejected or invalidated deterministically.
- `gas-tag` consumes the complete permanent `Tag` namespace in
  `ids/index.json`. Counts are positive integers; Exact, Parent and Child
  queries use the dotted hierarchy. A canonical hash of the full table and of
  the schema document must agree before `WorldReady` or `Running`.
- `gas-replication` declares authority, owner, third-party-public and hidden
  visibility for every field. The server snapshot hash covers authoritative
  fields; the client confirmation hash covers only non-predicted synchronized
  fields, with explicit complementary exclusion lists.
- `gas-prediction` uses the input frame as the prediction key and
  `GasAndEventFinalize` as the boundary. Effect removal, Effect period and
  out-of-simulation actions are non-predictable. Rejection rolls back exactly
  one client ECS/GAS/Voxel frame, replays later inputs deterministically and
  never rolls back the server.

## Contract

The source schemas are `schemas/gas-components.schema.json`,
`schemas/gas-tag.schema.json`, `schemas/gas-replication.schema.json` and
`schemas/gas-prediction.schema.json`. Registered positive and negative fixtures
exercise each S01-S05 criterion. Cross-field rules that JSON Schema cannot
express are enforced by `tools/lumio_contract.py validate`.

## Failure semantics

Unknown or duplicate component containers, `FxComponent`, mismatched row and
Handle identity, stale or cross-world probes, Tag table/schema disagreement,
hierarchy result drift, hidden/public visibility contradictions, hash-domain
leaks, standalone Modifier ledger declarations, prediction of a frozen
non-predictable action and server rollback are rejected deterministically.

## Compatibility and migration

This is additive within `LGE-V1.4-2026-08-27`. Existing lifecycle states,
MessageType values and host phases are unchanged. Generated package hashes are
regenerated from the architecture schemas, IDs and positive fixtures; no
implementation repository is changed.

## Verification

Run `node .spec/tools/spec-lint.mjs`, `node --test
.spec/tools/spec-lint.test.mjs`, `python3 -m py_compile
tools/lumio_contract.py`, `python3 tools/lumio_contract.py validate`, the
targeted `gas/*` fixtures and `git diff --check`.
