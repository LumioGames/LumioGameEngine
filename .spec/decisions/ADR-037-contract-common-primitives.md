# ADR-037: Contract Common Primitive Consolidation

- **Status**: Historical · Accepted (enters Implementation Baseline `LGE-V1.4-2026-08-27`, accepted 2026-08-27)
- **Owner**: Architecture (schema registry), all implementation repositories (consumers)
- **Baseline**: `LGE-V1.4-2026-08-27`
- **Relation**: Refines the contract surfaces of [ADR-003](ADR-003-cross-world-txn.md) (chunk revision keys), [ADR-005](ADR-005-replication-prediction.md)/[ADR-028](ADR-028-replication-typed-bodies.md) (envelope integrity), [ADR-010](ADR-010-persistence-config.md) (snapshot header), [ADR-011](ADR-011-observability.md) (error vocabulary), [ADR-018](ADR-018-coreengine-manifest-canonicalization.md)/[ADR-019](ADR-019-loader-state-machine-package-identity.md) (package identity, signatures), [ADR-022](ADR-022-protocol-permission-gate.md) (gate reject reasons) and [ADR-032](ADR-032-durable-recovery-records.md) (recovery record chain). Consumes the voxel `ChunkId` key format frozen by [ADR-024](ADR-024-voxel-p0-contract-set.md) and completes the semantic-validation mandates of [ADR-035](ADR-035-voxel-snapshot-payload.md)/[ADR-036](ADR-036-voxel-streaming-durability-ack.md). Supersedes none of them; every refined ADR stays authoritative for its domain semantics.

## Context

The V1.3 contract set froze forty schemas that repeat the same primitives with drifting shapes: three recovery record schemas re-declare the identical hash-chain skeleton; package identity appears three times with three different required sets; two signature blocks disagree on the field name (`value` vs `signature`); `trustDomain` and the compression codec enum are copy-pasted; chunk-keyed revision maps accept the canonical `c:x:y:z` key in the voxel family but the legacy `chunk-0-0` key everywhere else; `snapshot-header.checksum` accepts any short string while every other digest is a SHA-256; the replication envelope accepts any `integrity.value` regardless of the declared algorithm; and the permission gate invents `MessageNotPermitted` although the ID Registry froze `MessagePermissionDenied` (1031). Each drift is a place where seven implementation repositories and the upcoming contract generator would encode a different truth. Before the first generated artifact ships is the last cheap moment to consolidate: nothing is deployed, so shape fixes are breaking-on-paper only.

## Decision

### 1. Common `$defs` downshift

`schemas/common.schema.json` gains the following shared definitions; domain schemas must reference them instead of re-declaring the shape:

| `$defs` | Shape | Referencing schemas |
| --- | --- | --- |
| `abiIdentity` | `^NativeManagedAbiV[0-9]+$` | `packageIdentity`, `release-manifest` |
| `packageIdentity` | closed object; **all five** of `manifestDigest`, `artifactSetDigest`, `abiIdentity`, `targetProfileDigest`, `capabilitySetDigest` required (the ADR-019 five-tuple) | `verified-package-descriptor`, `failure-bundle.coreEngine` |
| `signatureAlgorithm` | enum `Ed25519`, `ECDSA-P256` | `signature-envelope`, `signatureBlock` |
| `signatureValue` | string, 32–4096 chars | `signature-envelope`, `signatureBlock` |
| `signatureBlock` | closed object `{algorithm, keyId, signature}` all required | `release-manifest.signature` |
| `trustDomain` | enum `Production`, `Staging`, `Test` | `signature-envelope`, `verified-package-descriptor` |
| `compressionCodec` | enum `None`, `Zstd`, `Lz4` | `snapshot-header`, `voxel-chunk-page` |
| `recoveryRecordChain` | open mixin; `recordVersion`, `recordSeq`, `previousHash`, `payloadHash`, `length`, `checksum` all required | `txn-journal-record`, `command-log-record`, `wal-record-envelope` |
| `recoveryCommitFields` | open mixin; `commitState` (`Pending`/`Committed`/`Aborted`) and `durabilityState` (`Buffered`/`Durable`) required | `txn-journal-record`, `command-log-record` |
| `sessionReleaseTriple` | open mixin; `sessionId`, `productId`, `gameReleaseId` required | `replication-envelope`, `protocol-permission-gate`, `snapshot-header` |
| `stateTransitionEvent` | open mixin; `machine`, `instanceId`, `fromState`, `toState`, `event` required | `gas-lifecycle` (and every future lifecycle event record) |

**Composition rule.** Two forms only, both expressible in the bootstrap validator subset (`$ref`, `allOf`, `oneOf`, `enum`, `pattern`, `patternProperties`): a *closed* common object replaces the inline object wholesale via `$ref`; an *open mixin* is attached via `allOf` while the outer schema keeps its full closed property list for `additionalProperties: false`. The mixin owns the required set and canonical field shapes; the outer schema owns closure and domain extensions. Field-level duplication inside the outer property list is deliberate and machine-reconciled: the `allOf` re-validation makes a drifted outer shape fail its own fixtures.

### 2. Defect fixes (breaking before first implementation)

1. **Chunk-keyed revision maps use the ADR-024 canonical key everywhere.** `sessionRevisionVector.chunkRevisionSet` and `cross-world-txn.chunkRevisionSet` now reference `voxelChunkRevisionSet` (keys `^c:x:y:z$` via `patternProperties`, `additionalProperties: false`); the semantic gate re-checks the key format inside replication `FullSnapshot` bodies, which are structurally untyped. A legacy `chunk-0-0` key is rejected wherever it appears.
2. **`snapshot-header.checksum` is a SHA-256 digest.** The field tightens from a free 1–128 char string to `hash256`. This freezes the *format* only; the byte domain the checksum covers is fixed together with the canonical serializer (D-002 family) by the contract generator card — implementations must not invent an interim formula and must treat the field as opaque until then. `hash` remains the digest of the uncompressed payload as before.
3. **`replication-envelope.integrity` is branch-constrained by algorithm.** `None` → value is the literal `"none"`; `CRC32C` → 8 lowercase hex chars; `SHA256` → 64 lowercase hex chars; `AEAD` → a 24–256 char base64/hex tag. Enforced structurally (`oneOf`) and mirrored by the semantic gate. A value that does not match its declared algorithm is rejected.
4. **Gate reject vocabulary aligns with the ID Registry.** `protocol-permission-gate.rejectReason` renames `MessageNotPermitted` to the registered `MessagePermissionDenied` (1031); the remaining gate-only reasons `SessionMismatch` (1040), `RoleMismatch` (1041), `ClaimNotGranted` (1042) and `SessionAntiReplay` (1043) are registered in the `ErrorCode` namespace. The contract tool enforces that the gate reject enum is a subset of registered `ErrorCode` ids, so the gate can never invent an unregistered symbol again.
5. **`abortReason` stays domain-owned; the shared subset is frozen.** `cross-world-txn.abortReason` and `voxel-mutation-receipt.voxelMutationAbortReason` remain two enums: the transaction owns `PermissionDenied` (business permission is not a voxel concern per ADR-024), the voxel participant owns `LeaseExpired` (reservation lease is participant-local). The six shared members — `RevisionConflict`, `ChunkUnloaded`, `ValidationFailed`, `DeadlineExceeded`, `Cancelled`, `InsufficientResource` — must keep identical spellings; the contract tool freezes exactly this intersection and the two domain-only remainders, so silent divergence or silent merging both fail the gate.
6. **Package identity is the full five-tuple everywhere.** `verified-package-descriptor.packageIdentity` and `failure-bundle.coreEngine.packageIdentity` reference the common closed `packageIdentity`; `release-manifest.coreEnginePackage` keeps its flat extended shape (`packageId`, the five identity fields, optional `signatureEnvelopeDigest`) but its required set now includes `capabilitySetDigest`. The contract tool checks that `coreEnginePackage` requires every `packageIdentity` member, so the flat copy cannot drift.
7. **One signature block.** `release-manifest.signature` renames `value` to `signature` and references the common `signatureBlock`; `signature-envelope` keeps its envelope-level fields but references the common `signatureAlgorithm`, `signatureValue` and `trustDomain`. Lifecycle enums that merely share member *names* across domains (entity lifecycle, package trust decisions) are explicitly *not* merged: same spelling, domain-owned meaning, per the same reasoning as abort reasons.

### 3. Voxel semantic completion (closes the ADR-035/036 mandates)

The Python semantic gate gains the checks those ADRs assigned to "semantic validation" but V1.3 shipped only as prose: a `DiffPayload` must satisfy `cutProjection.worldRevision == base.baseWorldRevision + worldRevisionAdvance`; payload chunk tables must be in canonical `CoordXYZAscending` order with contiguous, ascending byte ranges that sum to `payloadLength`; a `DurabilityAck` covered-chunk set must not name the same `chunkId` twice. Where the published JSON Schema already fully expresses a voxel rule (the `oneOf` discriminations of ADR-035/036), JSON-Schema-only enforcement is *intentional* and needs no Python twin; this ADR records that judgment so the asymmetry is not read as an omission.

### 4. Bootstrap validator subset

The fallback validator in `tools/lumio_contract.py` adds `patternProperties` to its supported keyword subset (required by the canonical chunk-key constraint). The supported subset is now: `$ref`, `const`, `enum`, `type`, `allOf`, `anyOf`, `oneOf`, `not`, `pattern`, `patternProperties`, string/number/array/object bounds, `required`, `properties`, `additionalProperties`, `uniqueItems`. Schemas must stay inside this subset so the registry remains checkable before `jsonschema` is installed.

## Contract

Changed files: `schemas/common.schema.json` (new `$defs`), `schemas/txn-journal-record.schema.json`, `schemas/command-log-record.schema.json`, `schemas/wal-record-envelope.schema.json`, `schemas/session-revision-vector.schema.json` (via common), `schemas/cross-world-txn.schema.json`, `schemas/replication-envelope.schema.json`, `schemas/snapshot-header.schema.json`, `schemas/protocol-permission-gate.schema.json`, `schemas/verified-package-descriptor.schema.json`, `schemas/failure-bundle.schema.json`, `schemas/release-manifest.schema.json`, `schemas/signature-envelope.schema.json`, `schemas/voxel-chunk-page.schema.json`, `schemas/gas-lifecycle.schema.json`; `ids/index.json` (`ErrorCode` 1040–1043); `tools/lumio_contract.py` (patternProperties, integrity mirror, chunk-key mirror, gate-vocabulary check, abort-reason intersection check, voxel semantic completion).

## Failure semantics

A legacy chunk key, a non-SHA-256 snapshot checksum, an integrity value that contradicts its algorithm, an unregistered gate reject reason, a package identity missing any of its five digests, and a signature block missing `algorithm`/`keyId`/`signature` are all structural rejections. A drifted abort-reason intersection, a gate enum outside the ErrorCode registry, and a `coreEnginePackage` required set that loses an identity member fail the contract tool at registry load, not at fixture level — the registry itself becomes invalid. Diff payloads whose revision equation does not hold, non-canonical chunk ordering, byte ranges that do not tile the payload, and duplicate covered chunks are semantic rejections with the envelope intact.

## Alternatives

A single shared `abortReason` enum was rejected: it either forces `PermissionDenied` into a domain that must not own permission or forces `LeaseExpired` into a domain that has no leases; drift is prevented more precisely by freezing the intersection. Object-level `$ref` composition with `unevaluatedProperties` was rejected because the bootstrap validator cannot express it; the open-mixin-plus-closed-outer idiom keeps both validators authoritative. Restructuring `release-manifest.coreEnginePackage` to nest a `packageIdentity` sub-object was rejected as an unnecessary wire-shape change: the required-set alignment plus the tool check achieves the same drift protection. Freezing the snapshot-header checksum *formula* now was rejected: the byte domain belongs to the canonical serializer decision, and an interim formula would create a migration for no benefit.

## Compatibility and migration

Breaking on paper, compatible in practice: no implementation repository has shipped code against the loosened shapes, so the fixes land as part of the `LGE-V1.4-2026-08-27` cut with no migration window. Mirrors sync in the same wave. Any future loosening or re-tightening of a common `$defs` member requires a new ADR and a BaselineId bump; adding a *new* common def stays additive.

## Verification

Updated fixtures: every `cross-world-txn`, `session-revision-vector`, `snapshot-header`, `client-authority-update` and replication `FullSnapshot` fixture now uses canonical chunk keys; `snapshot-active` carries a SHA-256 checksum; release manifests carry the renamed `signature` field and the full five-tuple. New failure fixtures: `revision/legacy-chunk-key` (legacy key in a session revision vector), `snapshot/legacy-checksum` (prefixed checksum string), `replication/integrity-value-mismatch` (SHA256 algorithm with an 8-char value), `release/missing-capability-digest` (four-tuple package identity). Registry-level checks (gate vocabulary subset, abort-reason intersection, coreEnginePackage required superset) are exercised by `python3 tools/lumio_contract.py validate` on every run.
