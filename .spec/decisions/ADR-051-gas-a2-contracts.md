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
  visibility for every field from the following closed V1.4 matrix. `Owner`
  means the owning player replica (and the server authority); `ThirdParty`
  means every non-owner observer. `thirdPartyPublic=true` means the field is
  visible to all recipients, while `thirdPartyPublic=false` is owner-only.

  | Component field | Recipient interpretation | Authority | Sync/persisted/predicted/presentation |
  | --- | --- | --- | --- |
  | `AbilityComponent.state` | Owner only | Server | true / true / false / false |
  | `AbilityComponent.inputFrame` | Owner only | Client | true / false / true / false |
  | `EffectComponent.entries` | Owner only; Effect details stay private | Server | true / true / false / false |
  | `EffectComponent.presentationBuffer` | ThirdParty/public | Shared | true / false / false / true |
  | `AttributeComponent.current` | ThirdParty/public | Server | true / false / false / false |
  | `AttributeComponent.revision` | ThirdParty/public | Server | true / true / false / false |
  | `TagComponent.counts` | ThirdParty/public | Server | true / true / false / false |

  No other component-field pair is a V1.4 replication declaration: internal
  Handles, cost intermediates, prediction drafts, Modifier ledgers and an
  out-of-place `fx_key` are rejected. The server snapshot hash covers
  authoritative fields; the owner client confirmation hash covers only
  non-predicted synchronized fields, with explicit complementary exclusion
  lists.
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
leaks, fields outside the closed component-field matrix, standalone Modifier ledger declarations, prediction of a frozen
non-predictable action and server rollback are rejected deterministically.

## Alternatives

**Keeping replication fields open and relying on authority flags** was rejected. Flags and disjoint hashes do not establish which Effect details or internal values a recipient may see; the closed matrix makes the owner/public boundary machine-checkable.

**Adding a separate public Effect-details or Handle component** was rejected because it would expose internal accounting and create a second state projection. Effect details remain in the owner-only `entries` field, and Handles remain ECS identity probes rather than replicated fields.

**Treating `fx_key` as a free component field** was rejected because presentation data belongs to an Effect entry and `FxComponent` is explicitly forbidden. The only public presentation path is `EffectComponent.presentationBuffer` or an Effect entry's `fx_key` payload.

## Compatibility and migration

This is additive within `LGE-V1.4-2026-08-27`. Existing lifecycle states,
MessageType values and host phases are unchanged. Generated package hashes are
regenerated from the architecture schemas, IDs and positive fixtures; no
implementation repository is changed.

## Verification

Run `node .spec/tools/spec-lint.mjs`, `node --test
.spec/tools/spec-lint.test.mjs`, `python3 -m py_compile
tools/lumio_contract.py`, `python3 tools/lumio_contract.py validate`, the
targeted `gas/*` fixtures (including the five closed-matrix rejection records)
and `git diff --check`. The compatibility ADR entries must resolve as Git mode
`120000` links to these Draft files.
