# ADR-038: State Machine Descriptor Contract

- **Status**: Accepted (enters Implementation Baseline `LGE-V1.4-2026-08-27`, accepted 2026-08-27)
- **Owner**: Architecture (descriptor registry), every repository that owns a machine (semantics)
- **Baseline**: `LGE-V1.4-2026-08-27`
- **Relation**: Makes machine-readable the state machines frozen in prose by [ADR-001](ADR-001-session-lifecycle.md) (WorldSlotHost, SimulationSession), [ADR-005](ADR-005-replication-prediction.md) (ClientReplicaSession), [ADR-003](ADR-003-cross-world-txn.md)/[ADR-026](ADR-026-crossworld-commandbuffer-markers.md) (CrossWorldTxn, EcsCommandBuffer), [ADR-012](ADR-012-release-update-maintenance.md) (ReleasePool), [ADR-019](ADR-019-loader-state-machine-package-identity.md) (CoreEngineLoader), [ADR-031](ADR-031-gas-lifecycle.md) (GasAbility, GasEffect), [ADR-034](ADR-034-hot-reload-dual-scope.md) (GameplayScopeActivation), [ADR-024](ADR-024-voxel-p0-contract-set.md)/[ADR-036](ADR-036-voxel-streaming-durability-ack.md) (VoxelChunkResidency) and [ADR-035](ADR-035-voxel-snapshot-payload.md) (VoxelSnapshotCapture). Consumes the `stateTransitionEvent`/`stateName` primitives of [ADR-037](ADR-037-contract-common-primitives.md). Supersedes none; each source ADR stays authoritative for what its machine *means*.

## Context

The architecture freezes at least twelve state machines, but V1.3 records them only as prose bullets in ADRs and the architecture document. Two of them (GasAbility, GasEffect) additionally live as hand-written transition tables inside `tools/lumio_contract.py`, and seven have their state *enum* copied into some domain schema. Every implementation repository is about to re-transcribe the same prose into Rust and C# by hand, which is exactly the drift channel the contract gate exists to close: a renamed state in one repository, a missing fault edge in another, and no machine ever notices. A descriptor contract makes the transition tables generatable artifacts (ADR-023 `ContractTypes`/`StateTransitionTable` family) instead of twelve independent transcriptions.

## Decision

### 1. Descriptor schema

New P0 schema `state-machine-descriptor.schema.json` (owner: Architecture). One document describes one machine: `machineId`, `owner` (the repository that runs the machine), `sourceAdr` (the ADR that owns its semantics), `initialState`, `states`, `terminalStates`, `transitions` (`from`/`to`/`event` triples), optional `anyActiveTo` (the fault idiom: an implicit edge from *every non-terminal state* to each listed target, used for `Faulted`/`Closed`/`Cancelled`-style exits so the explicit table stays free of N×fault noise), optional `selfEvents` (state-internal events that do not change state, e.g. Effect `Stack`/`Duration`/`Refresh`), optional `notes`. State names use the ADR-037 `stateName` shape; events use the shared `id` shape.

### 2. Descriptor registry: twelve machines, frozen

The registered machine set is frozen by this ADR; adding, removing or renaming a machine requires a new ADR. Each machine has exactly one valid descriptor fixture:

| `machineId` | Owner | Source ADR | Schema enum cross-check |
| --- | --- | --- | --- |
| `WorldSlotHost` | Server | ADR-001 | — (descriptor is the truth) |
| `SimulationSession` | Server | ADR-001 | — |
| `ClientReplicaSession` | Client | ADR-005 | — |
| `EcsCommandBuffer` | GameRuntime | ADR-026 | `cross-world-txn.commandBufferState` |
| `CrossWorldTxn` | Server | ADR-003 | `cross-world-txn.state` |
| `CoreEngineLoader` | GameRuntime | ADR-019 | `failure-bundle.coreEngine.loaderState` |
| `GasAbility` | GameRuntime | ADR-031 | Python `_ABILITY_TRANSITIONS`/`_ABILITY_TERMINAL` |
| `GasEffect` | GameRuntime | ADR-031 | Python `_EFFECT_TRANSITIONS`/`_EFFECT_TERMINAL` + Active self events |
| `ReleasePool` | Server | ADR-012 | `release-catalog.entries.state` |
| `GameplayScopeActivation` | GameRuntime | ADR-034 | `gameplay-scope-activation.stage` |
| `VoxelSnapshotCapture` | VoxelEngine | ADR-035 | `voxel-snapshot-payload.$defs.voxelCaptureState` |
| `VoxelChunkResidency` | VoxelEngine | ADR-024 | `voxel-chunk-page.$defs.voxelChunkState` |

Machines without a schema enum (the three session/host lifecycles) are registered presence-only: their descriptor *is* the canonical table. `gas-lifecycle` transition events remain the runtime evidence shape (per ADR-037 `stateTransitionEvent`); the descriptor is the static table those events are judged against.

### 3. Descriptor coherence rules (per instance)

The semantic gate rejects a descriptor whose `initialState`, `terminalStates`, `anyActiveTo`, transition endpoints or `selfEvents` reference undeclared states; whose terminal states own outgoing transitions or internal events; whose `(from, event)` pair is reused for two different transitions (machines are deterministic); whose states are not all reachable from `initialState` (counting `anyActiveTo` edges); or whose non-terminal states have no exit when the machine declares no `anyActiveTo` idiom.

### 4. Registry-level consistency (cross-artifact)

At registry load the tool enforces: the valid-descriptor set covers exactly the frozen twelve machines with no duplicates; every schema-enum cross-check in the table above is *set equality* (a state added to a schema enum without updating the descriptor — or vice versa — invalidates the registry, same failure class as ADR-037 vocabulary drift); the GasAbility/GasEffect descriptors equal the frozen ADR-031 transition tables, terminal sets and Active-internal event set that the gate already enforces on runtime `gas-lifecycle` events. Semantically equal, dually encoded: when the contract generator lands, the Python tables become derived from the descriptors and the duplication collapses.

### 5. Modeling judgments recorded

`VoxelChunkResidency` has no `Dirty -> Evicting` edge: the ADR-036 fence means a dirty chunk re-enters `Ready` via durability coverage before it may evict — the fence is a *missing edge*, not a guard annotation. `GameplayScopeActivation` failure after `BarrierSwitch` is session fail-stop (ADR-027), not a stage, so the machine keeps `OldUnloaded` as its only terminal; pre-switch failure is the explicit `NewValidated -> OldActiveNewStaging` discard edge. `CrossWorldTxn.Indeterminate` stays non-terminal with the single `IntentReplayed -> Committed` exit, encoding "post-intent may only resolve forward". `WorldSlotHost` snapshot/reload return to `Quiescing` (the slot keeps serving) while migration exits through `Stopping`.

## Contract

New: `schemas/state-machine-descriptor.schema.json`; fixtures `valid/state-machine-*.json` (twelve machines), `invalid/state-machine-terminal-outgoing.json`, `invalid/state-machine-undeclared-state.json`. Changed: `schemas/index.json` (P0 count 35 → 36), `fixtures/index.json` (+14), `tools/lumio_contract.py` (`_STATE_MACHINE_SOURCES`, descriptor semantic branch, `state_machine_consistency_errors` at registry load).

## Failure semantics

An incoherent descriptor (undeclared state, terminal outgoing edge, nondeterministic event reuse, unreachable state, dead non-terminal state) is a fixture-level semantic rejection. A registry whose descriptors do not cover the frozen twelve, drift from a schema enum, or contradict the frozen GAS tables fails at registry load — the contract set itself is invalid, nothing validates until the drift is resolved. Runtime transition legality stays where it was: the `gas-lifecycle` event gate (and future per-machine event gates) judge *events*; this ADR guarantees the *table* they judge against is single-sourced.

## Alternatives

Embedding transition tables in each domain schema was rejected: half the machines have no domain schema, and tables inside schemas are not independently addressable by the generator. Guard/action annotations in the descriptor were rejected for V1: guards are domain code, not contract shape; the fence-as-missing-edge idiom plus `notes` covers the P0 machines without inventing an expression language. Making `sourceAdr` an array for dual-authority machines (residency: ADR-024 states + ADR-036 fence) was rejected: one primary authority plus `notes` keeps the field mechanical.

## Compatibility and migration

Purely additive at the wire level: no existing message shape changes. The registry-level checks are new gate obligations that land together with the `LGE-V1.4-2026-08-27` cut. When the contract generator (Foundation first card) ships `StateTransitionTable` artifacts, implementation repositories must consume the generated tables instead of transcribing this ADR.

## Verification

`python3 tools/lumio_contract.py validate` — twelve valid descriptors pass structural + semantic gates; the two invalid fixtures are rejected for terminal-outgoing and undeclared-state; registry-level checks prove descriptor↔schema-enum equality (seven machines), descriptor↔Python-table equality (two GAS machines) and frozen-set coverage. Mutating any cross-checked schema enum without the descriptor (or vice versa) fails registry load, which is exercised implicitly on every validate run.
