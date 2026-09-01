# ADR-024: Voxel P0 Public Contract Set (World/Port, Chunk/Page, Revision Stamp, Query Consistency)

- **Status**: Historical · Accepted (accepted 2026-08-27 additive to `LGE-V1.2-2026-08-27`; formally entered Implementation Baseline `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioVoxelEngine` (contract source), Architecture (registry)
- **Baseline**: `LGE-V1.2-2026-08-27` line
- **Relation**: Completes the Voxel milestone of the Architecture Gate. Refines the World/Role ownership of [ADR-001](ADR-001-session-lifecycle.md), the Barrier semantics of [ADR-002](ADR-002-tick-determinism.md) and the Revision meanings of [ADR-003](ADR-003-cross-world-txn.md); supersedes none of them. The write-side participant receipt is [ADR-025](ADR-025-voxel-participant-receipt-durability.md).

## Context

The VoxelEngine module map (world/chunk/revision/query/mutation) kept candidate interfaces only: no public schema existed for world creation, chunk/page encoding, revision stamps or query consistency, so Rust, C ABI and C# bindings could not be generated and module READMEs risked becoming a second truth. The voxel architecture review requires the architecture source to freeze these contracts before Foundation coding (P0 contract set, VXL gaps §1–§4), while chunk dimensions and storage backends explicitly stay open (voxel decision gates VOX-D-001/002/003/005).

## Decision

Freeze the read-side Voxel P0 contract as four schemas plus shared definitions in `common.schema.json` (`voxelChunkCoord`, `voxelBlockCoord`, `voxelChunkId`, `voxelBlockValue`, `voxelContext`, `voxelChunkRevisionSet`):

- **`voxel-world-port.schema.json`** — one record per world instance: `role` is `Authority` or `Replica` (LocalEmbedded runs two independent instances, never a shared one); `context` is `{contextId, generation}` and every late result is rejected against it; `schemaEpoch` is pinned at creation; capabilities and a required resource budget (`maxResidentChunks`, `snapshotPinBudgetBytes`, `maxConcurrentQueries`, `maxPendingMutations`, `receiptRetentionEntries`) come from immutable config; the handle model is `IndexGenerationContext`/`GenerationBump`/`StableError` (same vocabulary as ADR-017); the port surface is exactly `createWorld, query, prepareMutation, commit, abort, status, capture, applyDurabilityAck, restore, quiesce, destroy`; lifecycle states are `Created…Faulted` as declared by the schema enum. Query poll/cancel and continuations ride the `query` entry and the `voxel-query` contract.
- **`voxel-chunk-page.schema.json`** — chunk coordinates are signed 32-bit per axis and negative coordinates are first-class; the canonical `ChunkId` string is `c:<x>:<y>:<z>` with no leading zeros and no `-0`, and it is the required key format for every voxel-owned chunk-keyed map; block values cross the wire as unsigned 32-bit; pages carry `pageIndex`, `pageVersion`, `encoding` (`Dense`/`Sparse`), `payloadLength`, SHA-256 `hash` and `compression` (`None`/`Zstd`/`Lz4`, same enum as `snapshot-header`); chunk data states are `Unallocated, Loading, Ready, Dirty, Evicting, Unloaded, Failed`.
- **`voxel-revision-stamp.schema.json`** — the read stamp every result carries: `{worldId, context, worldRevision, chunkRevisionSet}` plus optional `tickId`/`schemaEpoch`. `WorldRevision` is monotonic per world, `ChunkRevision` monotonic per chunk; the same shape is the projection of a coordinator-owned SnapshotCut onto the voxel domain.
- **`voxel-query.schema.json`** — consistency enum `voxelQueryConsistency` is `ExplicitRevision` or `LatestAtBegin`: the target revision is bound when the request begins, and one `boundWorldRevision` covers every chunk in the read set. A continuation carries its token together with the same `boundWorldRevision` and never rebinds; if the bound revision has been reclaimed the request terminates with `TargetRevisionUnavailable` instead of silently reading latest. Per-chunk presence is `Ready, NotLoaded, Pending, Unavailable` (`voxelChunkPresence`); a missing chunk must never be served as empty blocks. Block coordinates are 64-bit signed, constrained in JSON to the IEEE-754-safe integer range.

New stable identifiers: ErrorCodes `ChunkUnavailable` (1033), `TargetRevisionUnavailable` (1034), `BudgetExceeded` (1035), `QueueFull` (1036), `CoordinateOutOfBounds` (1037); Capabilities `VoxelSnapshot` (4), `VoxelStreaming` (5), `VoxelSpatial` (6), `VoxelMeshCollision` (7) for the optional module hookup declared at world creation.

## Contract

`schemas/voxel-world-port.schema.json`, `schemas/voxel-chunk-page.schema.json`, `schemas/voxel-revision-stamp.schema.json`, `schemas/voxel-query.schema.json`, the shared `$defs` in `schemas/common.schema.json`, and the registry rows in `schemas/index.json` (owner `VoxelEngine`, priority P0) and `ids/index.json`. Implementation repositories must consume generated types; module READMEs may no longer copy field layouts.

## Failure semantics

Out-of-range coordinates fail structurally (`CoordinateOutOfBounds` at the port); a corrupt page hash is rejected, never silently zero-filled; an unloaded, pending or failed chunk is reported through `voxelChunkPresence` and `ChunkUnavailable`, never as an empty world; a reclaimed bound revision returns `TargetRevisionUnavailable`; budget and queue exhaustion return `BudgetExceeded`/`QueueFull` and truncated batches carry an explicit terminal status, never a silent success.

## Alternatives

Freezing concrete chunk dimensions and page sizes was rejected: VOX-D-001/002 keep numeric profiles open, so the contract freezes wire widths and envelopes only (see `DECISIONS_PENDING.md` D-013). Deriving ChunkId implicitly per repository was rejected because chunk-keyed revision maps need one canonical string key. Allowing continuations to re-acquire the latest revision was rejected: it would make multi-batch reads internally inconsistent and mask lost pins.

## Compatibility and migration

These schemas are additive to the `LGE-V1.2-2026-08-27` line; no existing schema field changes meaning. There is no deployed voxel wire consumer yet, so no migration window is required. Changing coordinate width, the ChunkId format, a state/enum value or the consistency binding later requires a new ADR, new fixtures and a new BaselineId.

## Verification

Fixtures `world/authority-ready` and `world/bad-role`; `chunk/negative-coord` (INT32 boundary, negative coordinates), `chunk/coord-overflow`, `chunk/page-bad-hash`; `voxelrev/stamp` and `voxelrev/negative-chunk-revision`; `query/multi-chunk-bound` (bound revision shared by every chunk, tri-state missing chunks, bound continuation) and `query/continuation-unbound`.
