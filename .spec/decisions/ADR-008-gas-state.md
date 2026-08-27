# ADR-008: GAS Core State Model

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioGameRuntime` (framework), `LumioGame` (content)
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

Ability, Effect and Attribute behavior was previously split between Runtime and Game, with no rule for handles, prediction or ECS ownership.

## Decision

Runtime owns the generic Ability/Effect/Attribute/Tag state machines, TypeId/InstanceId/Handle distinction, Stack/Duration/Cancel ordering, PredictionContext, Snapshot/Restore and deterministic evaluation hooks. Game owns concrete content, formulas, targeting, cost, cooldown, permissions and presentation events. ECS is the single authoritative storage for replicated Gameplay state; GAS keeps only framework-owned indexes and transient execution context.

Server validates and commits; Client may predict inside a bounded PredictionFrame. GAS, ECS and Voxel overlays confirm or roll back as one frame.

## Contract

Content schemas must declare stable TypeId, version, formula inputs, authority role and replication mapping. Runtime API exposes lifecycle/error results, not product names or socket types.

## Failure semantics

Unknown type, invalid handle, exceeded stack/quotas, formula error or permission failure rejects activation without partial effects. A rejected predicted ability produces a correction and deterministic history replay.

## Alternatives

Putting all ability logic in Game was rejected because it duplicates lifecycle and rollback. Making GAS a second state truth was rejected because ECS and replication would diverge.

## Compatibility and migration

Changing modifier order, formula semantics or TypeId is a gameplay schema break and needs a Game Migration plus new Release. Framework additive APIs can remain backward compatible.

## Verification

Add activation/stack/cancel/expiry, snapshot restore, deterministic formula, permission, prediction reject and Save/Load golden fixtures before marking this ADR Accepted.
