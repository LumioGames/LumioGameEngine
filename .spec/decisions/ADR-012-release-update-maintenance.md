# ADR-012: Release Catalog, Rolling Update and Forced Maintenance

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioServer` (orchestration), `LumioGame` (release composition)
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

The service must support rolling updates, an explicit force-stop that kicks every target user, and concurrent products/releases such as A 1.1 and BOE 2.1. A single global version flag cannot express that scope.

## Decision

`ReleaseCatalog` routes by `ProductId + GameReleaseId` and records signed artifacts, capabilities, endpoint and Pool state. One process loads one Release; multiple processes/Pools serve products/releases concurrently. V1 requires exact Server/Client Release matching.

Rolling update is `Published -> Verified -> Warmup -> Serving`; the old Pool becomes `Draining -> Empty -> Retired`, with Rollback/Faulted exits. It stops new admissions but keeps existing sessions until drain, deadline or explicit migration.

Maintenance commands always scope `ProductId + GameReleaseId + ReleasePoolId`. `Graceful` closes ingress, broadcasts reason/deadline, drains transactions and persists Snapshot/WAL/Audit, then sends `MaintenanceKick` to remaining users at deadline. `Forced` stops input/Tick submission immediately, persists best effort, broadcasts `MaintenanceKick` and disconnects every target-Pool connection; uncommitted commands are not assumed applied.

## Contract

`release-manifest.schema.json` and `maintenance-command.schema.json` define release identity and command scope. Routing, drain and kick events use the logging correlation fields.

## Failure semantics

Manifest, ABI, signature, SBOM, capability or health failure prevents Serving. Rollback keeps the old verified Pool and snapshots. A process crash recovers from the last valid checkpoint/WAL and records all kicked, disconnected and indeterminate sessions.

## Alternatives

A single process loading multiple incompatible Releases was rejected for ABI/ALC and fault isolation. Global maintenance was rejected because it would affect unrelated products. Transparent live Session migration is deferred.

## Compatibility and migration

V1 exact matching rejects mismatched clients with a stable error. N/N-1 compatibility and live Session migration require future ADRs and explicit handshake/Save migrations; they cannot be inferred from semver.

## Verification

Validate A 1.1 and BOE 2.1 manifests, mismatch failure, warmup/drain/rollback, graceful deadline and forced all-user kick tests, including concurrent Pool isolation.
