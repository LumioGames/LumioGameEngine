# ADR-025: Voxel Participant Receipt Durability and Pruning Handshake

- **Status**: Accepted (accepted 2026-08-27 additive to `LGE-V1.2-2026-08-27`; formally entered Implementation Baseline `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioVoxelEngine` (participant), `LumioGameRuntime` (coordinator journal)
- **Baseline**: `LGE-V1.2-2026-08-27` line
- **Relation**: Extends [ADR-003](ADR-003-cross-world-txn.md) with the participant-side receipt durability, status protocol and retention handshake. ADR-003 stays Accepted and unchanged: global states, the `VoxelCommit -> EcsCommandBufferCommit` order, CommitIntent-before-first-write and Indeterminate resolution are not modified here.

## Context

ADR-003 requires that a duplicate `TxnId` returns the original result and that recovery replays only missing idempotent steps, but it does not say where the voxel participant's idempotency record lives, when it may be deleted, or what `status(txnId)` returns after cache eviction, snapshot restore or journal truncation. A bounded in-memory cache alone cannot survive the documented danger window — CommitIntent persisted, voxel data applied, crash before the coordinator writes the voxel participant marker — so recovery could re-apply blocks and double-increment revisions (voxel review VXL-002; voxel gate VOX-D-004).

## Decision

- **Participant states are `Prepared`, `Applied`, `Aborted`, `Duplicate`** (`voxelParticipantState`). The voxel participant never owns a `CommitIntent` state; it only consumes the coordinator's proof that the intent is persisted, then applies.
- **Receipt durability model: `CoDurableWithWorldState`.** The participant receipt is recorded inside the same infallible CommitBatch publish that publishes chunk pages, `ChunkRevisionSet` and `WorldRevision` at the Voxel Barrier. The receipt is therefore part of voxel authoritative state: every SnapshotCut that captures the write captures its receipt, and restore materializes both together. Invariant: a recovered voxel state contains a transaction's receipt **iff** it contains that transaction's writes. Receipts are queryable by `SessionId + TxnId`.
- **Crash between markers**: after recovery the coordinator finds the persisted CommitIntent without the voxel marker and re-issues Apply or queries status. If the recovered state includes the write, the participant answers with the original receipt (`Duplicate` on re-Apply, `Applied` on status) and nothing is re-applied. If the recovered state predates the write, the receipt is equally absent, status answers `Unknown`, and replaying the Apply is safe by the invariant above. The global transaction remains `Indeterminate` until resolved this way, exactly as ADR-003 requires.
- **`status(txnId)` protocol**: the request carries `sessionId`, `txnId` and the transaction's `txnDeadlineTick` (the coordinator knows it from its journal). The response (`StatusResponse` in `voxel-mutation-receipt.schema.json`) returns `statusResult` from `voxelParticipantStatusResult`: `Unknown`, `Prepared`, `Applied`, `Aborted` or `ResultPruned`, plus the participant's current `pruneHorizonTick`. `Applied`/`Aborted` responses re-serve the original result revisions or abort reason.
- **No unilateral eviction**: the receipt table may only shrink through the pruning handshake. If `receiptRetentionEntries` is exhausted, new Prepares are rejected with `BudgetExceeded`; existing receipts are never silently evicted, so "cache eviction" cannot forge `Unknown`.
- **Pruning handshake**: (1) the coordinator persists a TxnJournal checkpoint covering every finalized transaction with `deadlineTick <= H`; (2) it sends the participant a prune acknowledgment carrying `pruneHorizonTick = H`; (3) the participant durably records `H` (same co-durable rule) and may drop receipts with `deadlineTick <= H`. Afterwards: no receipt and `txnDeadlineTick <= pruneHorizonTick` → `ResultPruned` (the journal is authoritative; the coordinator must not replay); no receipt and `txnDeadlineTick > pruneHorizonTick` → `Unknown` (never reached this participant; replay per journal is safe).
- **`Duplicate` semantics**: `Duplicate` is the idempotent re-serve of an `Applied` receipt for a repeated Commit (lost result included); a repeated Abort re-serves the `Aborted` receipt unchanged. Neither path re-applies writes, re-increments a revision or double-charges resources (charging stays on the Game side per ADR-003).

## Contract

`schemas/voxel-mutation-receipt.schema.json` (`kind` = `ParticipantReceipt` | `StatusResponse`), registered in `schemas/index.json` as P0 `VoxelEngine`; abort reasons `voxelMutationAbortReason` are `RevisionConflict, ChunkUnloaded, ValidationFailed, DeadlineExceeded, LeaseExpired, Cancelled, InsufficientResource` — permission and charging failures belong to the Game participant. The retention budget field is `resourceBudget.receiptRetentionEntries` in `voxel-world-port.schema.json`.

## Failure semantics

An `Applied` or `Duplicate` receipt without its original result revisions is malformed. A participant record claiming `CommitIntent` is malformed. A `ResultPruned` response never carries result revisions and always carries the prune horizon. Losing the receipt table while keeping the writes (or vice versa) violates the co-durability invariant and is a fatal world fault, not a retryable error.

## Alternatives

Rebuilding participant state by coordinated snapshot plus deterministic command replay was rejected: it cannot distinguish an already-applied transaction without an in-state receipt and couples voxel recovery to global replay infrastructure. Unbounded retention was rejected for unbounded memory. TTL/LRU eviction was rejected because silent eviction breaks Duplicate detection and permits double apply.

## Compatibility and migration

Additive to ADR-003; journal record formats and the global transaction schema are unchanged. Changing the durability model, the status result set or the handshake ordering later requires a new ADR, a transaction schema epoch bump and a migrator for retained receipts.

## Verification

Fixtures `mutation/receipt-applied`, `mutation/receipt-duplicate` (duplicate returns the original result), `mutation/receipt-aborted-revision-conflict`, `mutation/status-lost-result` (lost result recovered via status), `mutation/status-result-pruned` (post-handshake semantics), `mutation/receipt-global-commit-intent` (participant must not own CommitIntent), `mutation/receipt-applied-missing-result`, and `txn/crash-between-markers` (Indeterminate with exactly the voxel marker set). Implementations must additionally inject a crash at each journal boundary and assert no double apply, per ADR-003.
