# Runtime Command / Coordination Review

Base: `ef822a7`  →  Head: `7952804` (`feat(runtime): add command and coordination foundation`)

## Verdict

**Spec Compliance: BLOCKED. Ready for merge: NO.** The implementation is a useful foundation and the focused unit suites pass, but it has P1 contract/recovery defects. Per review policy, any P1 blocks merge.

## Strengths

- Command buffers enforce an explicit lifecycle (`Open -> Sealed -> Merged -> Prepared -> Applied`) and reject writes after sealing (`modules/command/src/Lumio.GameRuntime.Command/Buffers/ProcessorCommandBuffer.cs:236-247`).
- Per-buffer command and byte budgets are checked before mutation, and failed appends leave usage unchanged (`.../Buffers/ProcessorCommandBuffer.cs:169-177`, `.../Budgets/CommandBufferBudget.cs:47-57`).
- Merge validates tick/world/scope, command sequence, structural capability, byte accounting, and uses the required phase/processor/sequence ordering (`.../Merge/CommandBufferMerger.cs:93-165`).
- Preflight validates deferred-token scope, target existence, duplicate/conflicting writes, capacities, and marks the batch prepared only after validation (`.../Prepare/CommandPreflightValidator.cs:115-295`).
- ECS apply serializes concurrent replay, resolves deferred creates before later references, and records idempotent receipts (`.../Apply/EcsCommandCommitExecutor.cs:29-113`).
- Transaction state and participant markers are guarded and monotonic; recovery does not guess unavailable participant state (`modules/coordination/src/Lumio.GameRuntime.Coordination/Transactions/TxnRecord.cs:117-209`, `.../Recovery/TxnRecoveryResolver.cs:124-167`).
- Snapshot cuts enforce barrier/paused-session gating, exact shared revision equality, deterministic participant ordering, and reverse-order release (`.../Snapshot/SnapshotCutCoordinator.cs:102-149`, `.../Snapshot/SnapshotCutLease.cs:27-40`).

## Issues

### P1 - Blocking

1. **Default composition uses an in-memory journal as the supposedly durable CommitIntent boundary.** `CrossWorldCoordinator(SessionRevisionVectorStore)` constructs `new InMemoryTxnJournalPort()` and injects it into `CommitIntentCoordinator` (`modules/coordination/src/Lumio.GameRuntime.Coordination/Transactions/CrossWorldCoordinator.cs:90-102`). `CoordinationModule.Create` always selects this constructor (`.../CoordinationModule.cs:20-24`). A process restart therefore loses CommitIntent, participant markers, and committed markers, so the default public composition cannot satisfy the architecture requirement that recovery be based on durable `TxnJournal` records. The in-memory implementation is appropriate as an explicitly supplied test/host adapter, but must not be the default runtime durability implementation.

2. **The terminal `Committed` journal record is durable before the shared revision is advanced.** Commit appends `Committed` at lines `192-196`, then computes/advances the revision at `198-205`, and only then transitions the record to `Committed` at `208-211` (`modules/coordination/src/Lumio.GameRuntime.Coordination/Commit/CommitIntentCoordinator.cs:192-211`). If revision advancement fails, a durable committed marker remains while the record is moved to `Indeterminate`; recovery treats that marker as proof of commit (`.../Recovery/TxnRecoveryResolver.cs:90-101`, `235-249`) but does not advance or verify the revision in `ConvergeCommittedMarker`. This can produce a committed transaction with an unadvanced/inconsistent shared Revision Vector and violates the Snapshot/transaction consistency contract.

3. **The voxel adapter introduces a public, manually authored wire-shaped contract with no generated schema/ID source.** `IGeneratedVoxelWorldPort` and all `GeneratedVoxel*` request/result records are hand-written (`modules/coordination/src/Lumio.GameRuntime.Coordination.VoxelAdapters/GeneratedVoxelWorldPortAdapter.cs:8-73`). The adapter then exposes these as a public package API (`:75-132`). The architecture explicitly requires generated contract types to come from the locked schema and forbids inventing a second public schema/IDs; the source tree has no generated contract corresponding to these types. This must either consume the actual generated voxel port contract or remain an internal adapter over a schema-owned type.

4. **Contract validation is not green.** `python C:\Work\LumioGames\LumioGameEngineArchitecture\tools\lumio_contract.py validate` fails: published Root ABI compiler digest `0aaf61...64bff` differs from the locked compiler hash `6f51b9...9e745`, and requests regeneration with `python3 tools/lumio_contract.py generate --out packages`. This is a required generated-contract gate failure and must be resolved or explicitly reconciled before merge. The PowerShell/bash wrapper checks could not run in this Windows environment because `pwsh` and `bash` are unavailable; the direct Python validator failure is authoritative evidence.

### P2 - Non-blocking quality gaps

1. `DeferredEntityToken` accepts `localSequence == 0` and `IsValid` does not reject it (`modules/command/src/Lumio.GameRuntime.Command/Tokens/DeferredEntityToken.cs:19-35`, `48-53`). Tokens created by the buffer start at one, but externally constructed malformed tokens can pass parts of preflight scope checking. Require a positive local token sequence and reject malformed tokens consistently.
2. `CommandPreflightOptions.SchemaEpoch` is copied into `PreparedGameDelta` without checking against `GeneratedContractManifest.SchemaEpoch` (`modules/command/src/Lumio.GameRuntime.Command/Prepare/CommandPreflightValidator.cs:18-30`, `325-335`). The coordination path checks epoch equality, but direct command prepare can manufacture a delta for an unsupported epoch.
3. `CommandBufferMerger` rejects duplicate `ProcessorId` even when phases differ (`modules/command/src/Lumio.GameRuntime.Command/Merge/CommandBufferMerger.cs:90-109`), while the frozen ordering key is `(Phase, ProcessorId, LocalSequence)`. If one processor can legally emit buffers in more than one declared phase, this is over-restrictive; either document the single-phase invariant or key uniqueness by phase plus processor.
4. `CommandServices.Buffers` hard-casts the interface to `CommandBufferFactory` (`modules/command/src/Lumio.GameRuntime.Command/CommandServices.cs:25-31`), making the public convenience property throw when a custom `ICommandBufferFactory` is supplied. Return the interface or remove the cast.

## Task quality / scope

The change stays within command, coordination, voxel-adapter, observability package metadata, and repository SDK/test configuration. It does not add gameplay/storage/native implementation, and the command/coordination package references are otherwise reasonable. The added tests cover lifecycle, budgets, merge conflicts, deferred tokens, prepare side effects, transaction state, crash/lost-result recovery, revisions, and snapshot pin release, but they do not exercise a real durable journal restart, committed-marker/revision failure ordering, or generated-contract schema validation for the voxel adapter.

## Commands and results

- `dotnet build --no-restore modules/command/src/Lumio.GameRuntime.Command/Lumio.GameRuntime.Command.csproj`: passed, 0 warnings, 0 errors.
- `dotnet build --no-restore modules/coordination/src/Lumio.GameRuntime.Coordination/Lumio.GameRuntime.Coordination.csproj`: passed, 0 errors; one transient MSB3026 file-lock warning from concurrent Defender access.
- `dotnet test --project modules/command/tests/Lumio.GameRuntime.Command.Tests/Lumio.GameRuntime.Command.Tests.csproj --no-restore --no-build`: passed, 14/14.
- `dotnet test --project modules/coordination/tests/Lumio.GameRuntime.Coordination.Tests/Lumio.GameRuntime.Coordination.Tests.csproj --no-restore --no-build`: passed, 12/12.
- `node .spec/tools/spec-lint.mjs`: failed on pre-existing workspace mirror/spec fixture drift (soft-link, hidden knowledge entry, stale demo task, and related index checks); `node --test .spec/tools/spec-lint.test.mjs`: passed, 13/13.
- `python C:\Work\LumioGames\LumioGameEngineArchitecture\tools\lumio_contract.py validate`: failed on Root ABI compiler digest mismatch as described above.
- `git diff --check ef822a7..7952804`: passed.

## Merge decision

**Do not merge.** P1 items 1-4 are blocking. At minimum, inject a real durable journal in the default runtime composition, make terminal marker/revision publication atomic or recoverably ordered, replace the invented voxel contract with schema-generated types, and obtain a green generated-contract validation result. Then add restart and revision-failure recovery fixtures before re-review.
