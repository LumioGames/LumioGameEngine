# ADR-012: Release Catalog, Rolling Update and Forced Maintenance

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioServer` (orchestration), `LumioGame` (release composition)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

The service must support rolling updates, an explicit force-stop that kicks every target user, and concurrent products/releases such as A 1.1 and BOE 2.1. A single global version flag cannot express that scope.

## Decision

`ReleaseCatalog` routes by `ProductId + GameReleaseId` and records signed artifacts, capabilities, endpoint and Pool state. One process loads one Release; multiple processes/Pools serve products/releases concurrently. V1 requires exact Server/Client Release matching.

Rolling update is `Published -> Verified -> Warmup -> Serving`; the old Pool becomes `Draining -> Empty -> Retired`, with Rollback/Faulted exits. It stops new admissions but keeps existing sessions until drain, deadline or explicit migration.

Cluster desired state — which Pools exist, which Release each Pool serves, and when an old instance is replaced by a target instance — is owned by an external control plane (deployment supervisor/orchestrator), never by a DS process. The DS process exposes a local agent boundary: it verifies and executes signed commands scoped to itself, reports readiness, drain progress and exit evidence, and terminates at `ReadyToExit`/exit. Target-instance activation happens outside the old process, after its exit evidence, guarded by a control-plane fencing token; a command carrying a stale fencing token is rejected with the stable error `FencingTokenStale`. `MaintenanceId` is the idempotency key: a duplicate command returns current progress instead of starting a second execution, and a replay after completion returns the terminal state.

Maintenance commands always scope `ProductId + GameReleaseId + ReleasePoolId`. Maintenance deadlines live in the wall/monotonic clock domain, never the Logical Tick domain. A command carries `issuedAt` (audit ordering and replay dedup only) and `graceDeadlineSeconds` (a duration); the Host converts the duration once, at command receipt, into a monotonic-clock deadline, so the deadline converges even when no WorldSlot is active, Ticks are paused or the wall clock jumps. Any Slot-level tick cut needed for a consistent snapshot is derived internally by the Runtime at the quiesce barrier and is not part of the management contract.

`Graceful` (`graceDeadlineSeconds >= 1`) closes ingress, broadcasts reason and remaining grace window, drains transactions and persists Snapshot/WAL, waiting for the persistence commit acknowledgment and the Audit durable acknowledgment as two independent completions, then sends `MaintenanceKick` to remaining users at the deadline. `Forced` (`graceDeadlineSeconds = 0`) stops input/Tick submission immediately, persists best effort, broadcasts `MaintenanceKick` and disconnects every target-Pool connection; uncommitted commands are not assumed applied.

## Contract

`release-manifest.schema.json` and `maintenance-command.schema.json` define release identity, command scope, the grace window and the optional fencing token. Routing, drain and kick events use the logging correlation fields.

## Failure semantics

Manifest, ABI, signature, SBOM, capability or health failure prevents Serving. Rollback keeps the old verified Pool and snapshots. A process crash recovers from the last valid checkpoint/WAL and records all kicked, disconnected and indeterminate sessions.

## Alternatives

A single process loading multiple incompatible Releases was rejected for ABI/ALC and fault isolation. Global maintenance was rejected because it would affect unrelated products. Transparent live Session migration is deferred.

## Compatibility and migration

V1 exact matching rejects mismatched clients with a stable error. N/N-1 compatibility and live Session migration require future ADRs and explicit handshake/Save migrations; they cannot be inferred from semver.

## Verification

Validate A 1.1 and BOE 2.1 manifests, mismatch failure, warmup/drain/rollback, graceful deadline and forced all-user kick tests, including concurrent Pool isolation, the forced-with-grace failure fixture, duplicate-command idempotency and stale-fencing-token rejection.
