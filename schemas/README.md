# Contract Schemas

This directory is the versioned contract input for the `LGE-V1.4-2026-08-27` baseline.

`index.json` is the registry. A schema entry names its stable contract id, file, owning repository and implementation priority. JSON Schema files use Draft 2020-12 and may reference definitions in `common.schema.json`.

The schemas describe wire and persistence shape, but they do not replace runtime ownership or state-machine checks. `tools/lumio_contract.py validate` runs structural validation and the small set of semantic checks that are safe to express at the architecture gate. Generated serializers and language bindings must consume these files; handwritten duplicate MessageId, field-layout or ABI definitions are not permitted.

P0 schemas are required before cross-repository integration. `mod-manifest.schema.json` is a P2 reservation: it describes the extension boundary but third-party Mods are not loaded by V1.

## GAS A1 Contracts

`gas-lifecycle.schema.json` keeps the ADR-031 Ability and Effect state sets closed and adds machine-readable Admission and Commit records without adding states. `gas-evaluation.schema.json` freezes the three V1 operators and `(Base + SigmaAdd) * (1 + SigmaPercent)` evaluation. `gas-effect-events.schema.json` freezes same-Tick Effect ordering, suppression as an Active-internal event, and Tick-only duration/period fields.

Cross-field rules that JSON Schema cannot express, including first-failure order, Commit charging, evaluation arithmetic, override tie-breaks and same-Tick cancellation, are enforced by `tools/lumio_contract.py validate` against the registered positive and negative fixtures.

## Change Rule

Changing a required field, enum, revision meaning, phase, ID layout or compatibility rule requires an ADR, a new positive and negative fixture, regenerated artifacts, a new baseline id, and synchronized repository mirrors.
