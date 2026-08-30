# Contract Schemas

This directory is the versioned contract input for the `LGE-V1.4-2026-08-27` baseline.

`index.json` is the registry. A schema entry names its stable contract id, file, owning repository and implementation priority. JSON Schema files use Draft 2020-12 and may reference definitions in `common.schema.json`.

The schemas describe wire and persistence shape, but they do not replace runtime ownership or state-machine checks. `tools/lumio_contract.py validate` runs structural validation and the small set of semantic checks that are safe to express at the architecture gate. Generated serializers and language bindings must consume these files; handwritten duplicate MessageId, field-layout or ABI definitions are not permitted.

P0 schemas are required before cross-repository integration. `mod-manifest.schema.json` is a P2 reservation: it describes the extension boundary but third-party Mods are not loaded by V1.

## GAS A1 Contracts

`gas-lifecycle.schema.json` keeps the ADR-031 Ability and Effect state sets closed and adds machine-readable Admission and Commit records without adding states. `gas-evaluation.schema.json` freezes the three V1 operators, exact-lexeme Decimal34 (`ROUND_HALF_EVEN`) accumulation with bounded exponents/coefficient size, sequence-ordered decimal accumulation and `(Base + SigmaAdd) * (1 + SigmaPercent)` evaluation. `gas-effect-events.schema.json` freezes same-Tick Effect ordering, suppression as an Active-internal event, and Tick-only duration/period fields.

Cross-field rules that JSON Schema cannot express, including first-failure order, Commit charging, exact Decimal34 evaluation arithmetic and bounds, override tie-breaks and same-Tick cancellation, are enforced by `tools/lumio_contract.py validate` against the registered positive and negative fixtures.

## GAS A2 Contracts

`gas-components.schema.json` closes the ECS projection to the four GAS containers and binds each row to a world/index/generation Handle. `gas-tag.schema.json` binds counted hierarchical tags to the permanent `Tag` ID namespace and its pre-World-Ready handshake; its table and schema hashes use the declared `SHA-256(CanonicalJsonV1)` construction. `gas-replication.schema.json` carries the exact seven component-field/recipient visibility pairs, field-level authority/owner/visibility and explicit server and client hash domains; hash exclusions do not remove a pair from the projection contract. The Modifier ledger remains only a derived view of Effect entries and is never a standalone replicated or persisted field. `gas-prediction.schema.json` freezes frame-keyed prediction rejection, one-frame ECS/GAS/Voxel rollback and deterministic replay.

The A2 schemas are additive under ADR-051. `FxComponent`, wall-clock timing, prediction-window/task concepts and RPC fields are not contract extensions.

## Change Rule

Changing a required field, enum, revision meaning, phase, ID layout or compatibility rule requires an ADR, a new positive and negative fixture, regenerated artifacts, a new baseline id, and synchronized repository mirrors.
