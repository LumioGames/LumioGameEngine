# ADR-013: Migration DAG, Staging and Atomic Activation

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGame` (Game semantics), `LumioVoxelEngine` (Voxel semantics), `LumioServer` (orchestration)
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Refined by**: [ADR-034](ADR-034-hot-reload-dual-scope.md)

## Context

Game and Voxel data evolve at different rates, and a fixed hand-written order cannot represent dependencies or recover safely after a crash.

## Decision

Migration reads an immutable `SnapshotId + SessionRevisionVector`, executes a declared acyclic graph in a staging directory, validates references, quotas, Schema and target Manifest, then atomically activates a new version pointer. Game and Voxel nodes expose typed inputs/outputs and never mutate the source snapshot. Short maintenance pauses are acceptable in V1; online cross-Release Session migration is not required.

## Contract

Migration manifests identify source/target Release, node ids, dependencies, input/output hashes, tool version and idempotency. Snapshot headers and Failure Bundles retain the evidence needed to rerun a failed graph.

## Failure semantics

Cycle, missing dependency, unsupported source, reference error, quota violation, checksum/signature failure or node crash aborts staging and preserves the previous active pointer. A rerun starts from immutable inputs or a verified node checkpoint.

## Alternatives

Hard-coded Game-then-Voxel order was rejected for independent schema evolution. In-place mutation was rejected for crash safety. Automatic semantic inference was rejected because only Game/Voxel owners know business meaning.

## Compatibility and migration

A new graph version is required for changed semantics. Old snapshots and Releases remain available through retention; the graph itself is part of the target ReleaseManifest.

## Verification

Add golden upgrades, downgrade rejection, cycle/missing-reference, quota, crash-at-each-node and atomic-pointer tests before production rollout.
