# Independent Review: Runtime Command/Coordination Delivery

## Scope and Verdict

Reviewed `ef822a7..1c53e2b7bc14d7f01a24a38ce6dc5d52448b2708` from the supplied review package (`76` changed files, `7,421` added lines), with the live v1.4 architecture source and command/coordination module READMEs. The review was read-only; no Workflow operation, commit, push, or source edit was performed.

**Decision: NOT READY FOR MERGE.** The implementation has multiple P1 acceptance blockers. In particular, the checked-in test configuration makes the required `dotnet test` command fail, and the commit/recovery paths can publish a committed transaction while using an inconsistent or stale revision vector. P1 findings must be corrected and covered by regression tests before acceptance.

## Verification Evidence

Commands were run against a clean `git archive` of commit `1c53e2b` in `C:\Temp\lumio-review-1c53e-2`:

| Command | Result |
|---|---|
| `dotnet build modules/command/src/Lumio.GameRuntime.Command/Lumio.GameRuntime.Command.csproj --no-restore` | PASS, net10.0/netstandard2.1, 0 warnings/errors |
| `dotnet restore modules/coordination/tests/Lumio.GameRuntime.Coordination.Tests/Lumio.GameRuntime.Coordination.Tests.csproj` | PASS |
| `dotnet build modules/coordination/tests/Lumio.GameRuntime.Coordination.Tests/Lumio.GameRuntime.Coordination.Tests.csproj --no-restore` | PASS, 0 warnings/errors |
| `dotnet test modules/command/tests/Lumio.GameRuntime.Command.Tests/Lumio.GameRuntime.Command.Tests.csproj` | **FAIL**: Microsoft.Testing.Platform reports “Testing with VSTest target is no longer supported ... opt-in to the new dotnet test experience.” |
| `dotnet modules/command/tests/.../Lumio.GameRuntime.Command.Tests.dll` | PASS, 14 passed, 0 failed |
| `dotnet modules/coordination/tests/.../Lumio.GameRuntime.Coordination.Tests.dll` | PASS, 18 passed, 0 failed |
| `node .spec/tools/spec-lint.mjs` | FAIL, 3 dangling-link findings for `.claude/agents`, `.claude/skills`, `.agents/skills` in the archived checkout |
| `node --test .spec/tools/spec-lint.test.mjs` | PASS, 13 passed |
| `python3 -m py_compile tools/lumio_contract.py` (architecture checkout) | PASS |
| `python3 tools/lumio_contract.py validate` (architecture checkout) | **FAIL**: published Root ABI compiler digest `0aaf61...` differs from locked compiler hash `6f51b9...`; validator requests regeneration |
| `git diff --check ef822a7..1c53e2b` | PASS, no whitespace errors |

The direct DLL runs demonstrate that the implementation compiles and the focused tests execute when bypassing the broken `dotnet test` integration, but they do not remove the configuration blocker.

## Spec Compliance

- **Lifecycle and ordering:** The command buffer state machine and `Phase + ProcessorId + LocalSequence` sorting are implemented; the commit coordinator follows `Voxel -> ECS` and writes an intent before the first apply.
- **Budgets and prepare/apply:** Command/byte/entity/change reservations, conflict checks, deferred-token scope checks, and fail-closed ECS apply are present. Prepared batches are immutable views and apply rejects a non-prepared batch.
- **CrossWorldTxn:** Guarded states, participant four-valued markers, duplicate lookup, abort/expire, bounded reservation leases, and journal hash-chain fields are present.
- **Revision/snapshot:** Revision vectors are defensively copied and monotonicity is checked; SnapshotCut compares all participant vectors and releases pins in reverse order.
- **Generated/public boundary:** The Voxel adapter is source-defined and merely “generated-contract-shaped”; it is not generated from the published `voxel-world-port` artifact. See P1-3.
- **Project scope:** Changes stay under command, coordination, the Voxel adapter, observability package metadata, and central package metadata. No ECS/Voxel storage implementation was introduced.
- **Test evidence:** Focused unit/property tests exist (14 command, 18 coordination), but required durable failure fixtures are not present as data artifacts and the standard test command fails.

## Strengths

1. `EcsCommandCommitExecutor` serializes apply/replay and caches receipts by tick/digest, reducing check-then-apply races (`modules/command/src/Lumio.GameRuntime.Command/Apply/EcsCommandCommitExecutor.cs:20-55`).
2. Preflight rejects stale direct targets, duplicate destroys, conflicting field writes, malformed identifiers, and budget overflow before `Prepared` (`modules/command/src/Lumio.GameRuntime.Command/Prepare/CommandPreflightValidator.cs:117-301`).
3. `InMemoryTxnJournalPort` enforces idempotency-key equality, sequence continuity, previous-hash continuity, and checksum verification (`modules/coordination/src/Lumio.GameRuntime.Coordination/Journal/ITxnJournalPort.cs:94-182`).
4. Recovery does not guess an unavailable participant: it records `Unknown` and leaves the transaction `Indeterminate` (`modules/coordination/src/Lumio.GameRuntime.Coordination/Recovery/TxnRecoveryResolver.cs:124-167`).

## P1 Issues (Acceptance Blockers)

### P1-1: Checked-in test configuration breaks the required `dotnet test` workflow

`global.json` in the reviewed commit contains only the SDK block and omits the `test.runner` selection (`global.json:1-7`). The test projects opt into Microsoft.Testing.Platform (`modules/command/tests/Lumio.GameRuntime.Command.Tests/Lumio.GameRuntime.Command.Tests.csproj:1-9`), but `dotnet test ...csproj` therefore invokes the unsupported VSTest target and exits non-zero with the error captured above. A clean checkout cannot use the repository's canonical test command, so test evidence is not merge-grade. Restore the MTP runner configuration or otherwise make the standard command pass, then rerun both test projects through `dotnet test`.

### P1-2: Commit and recovery do not enforce participant/result revision consistency

Normal commit takes the caller-supplied `resultRevision` or synthesizes `NextRevision(record)` and advances the store without checking the revision returned by the Voxel participant (`Commit/CommitIntentCoordinator.cs:198-211`). Recovery similarly picks `voxel.ResultRevision ?? ecs.ResultRevision`, silently ignoring a disagreement between the two participants (`Recovery/TxnRecoveryResolver.cs:141-151`). The durable-`Committed` marker fast path marks both participants and transitions the record but never advances the revision store or records a revision from the marker (`Recovery/TxnRecoveryResolver.cs:235-249`). These paths can expose `Committed` with a stale, fabricated, or mismatched `SessionRevisionVector`, violating the architecture's single revision truth and snapshot/replay consistency. Require both participant result vectors to be present/equal (or an explicit contract-defined derivation), advance the store exactly once, and persist/recover the resulting vector; add mismatch and committed-marker recovery tests.

### P1-3: Public Voxel adapter duplicates an incomplete generated contract

`GeneratedVoxelWorldPortAdapter.cs` declares `IGeneratedVoxelWorldPort` and all request/result records manually (`modules/coordination/src/Lumio.GameRuntime.Coordination.VoxelAdapters/GeneratedVoxelWorldPortAdapter.cs:7-73`). The reviewed generated-contract manifest already publishes the `voxel-world-port` schema, whose required surface includes port version, world/role/context, capabilities, resource budgets, lifecycle, handle model, and the complete method/error set. The hand-authored adapter only carries a subset of prepare/commit/query fields and is not tied to generated serializer/validator artifacts. This creates a drift-prone public API boundary and bypasses the architecture rule that generated contracts are the sole source of public schema truth. Consume the generated binding/artifact (or add the generator output and drift gate) instead of defining a parallel contract in runtime source.

### P1-4: Default composition can report successful commits without applying game or Voxel state

`CommandModule.Create()` silently constructs `new EcsCommandCommitExecutor()` when no executor is supplied (`modules/command/src/Lumio.GameRuntime.Command/CommandModule.cs:39-42`), and that executor defaults to `NoOpEcsCommandCommitPort` (`modules/command/src/Lumio.GameRuntime.Command/Apply/EcsCommandCommitExecutor.cs:20-21,161-164`). `CoordinationModule.Create()` similarly builds `CrossWorldCoordinator(revisions)`, which installs `DefaultVoxelWorldPort` and the same no-op ECS executor (`modules/coordination/src/Lumio.GameRuntime.Coordination/Transactions/CrossWorldCoordinator.cs:90-102`; `:291-308`). Calling the public default composition can therefore advance revisions and mark a transaction `Committed` while no ECS/Voxel mutation occurred. A no-op port may be useful in isolated tests, but it must not be the production default: require explicit participant ports or make the default fail closed (`InfrastructureFault`) until real ports are configured.

## P2 Issues

### P2-1: Deferred-token canonical identity omits world and buffer generation

`DeferredEntityToken.CanonicalKey` intentionally serializes only tick, processor, and local sequence (`modules/command/src/Lumio.GameRuntime.Command/Tokens/DeferredEntityToken.cs:55-64`), while the token's value identity/equality includes `WorldId` and `BufferGeneration` (`:31-46`). `Command.BuildCanonicalBytes` also omits those fields (`modules/command/src/Lumio.GameRuntime.Command/Commands/CommandSortKey.cs:222-247`). Two otherwise identical commands from different worlds/generations can therefore share a canonical command digest/idempotency key even though their token values are not equal. Include every contract identity field in canonical encoding, with a regression test across world/generation boundaries.

### P2-2: Required durable failure fixtures are absent

The changed fixture directory contains only `modules/command/tests/fixtures/command/README.md`; there are no committed valid/invalid JSON fixtures for duplicate, timeout, lost-result, partial-commit, or crash-boundary cases called out by the architecture and module READMEs. The xUnit tests cover selected in-memory scenarios, but fixture-level replay/contract evidence is missing and should be added before broad integration acceptance.

### P2-3: Architecture validator evidence is not green for the live architecture checkout

`python3 tools/lumio_contract.py validate` fails on the Root ABI compiler digest mismatch shown above. This may be an external architecture checkout drift rather than a runtime source defect, but the final delivery cannot claim a green architecture gate until the published package/compiler hashes are aligned and validation is rerun.

## Task Quality and Merge Readiness

The implementation is substantial and focused, with good local tests and explicit failure enums. The main quality gap is that tests are optimized for direct DLL execution and do not exercise the repository's standard runner or durable/generated-contract integration; the default no-op composition also makes the public API deceptively successful. **Ready for merge: No.** Resolve all P1 findings, add regression/fixture evidence for revision/token identity, and provide green standard `dotnet test` plus architecture-contract validation results.
