# R5-02 Handback

## 1. Deliverables and actual scope

- Runtime `WorldManager` now owns one dense-array `World`; entity ids are derived tombstones, component arrays are pooled, and generated entity templates provide the component factory used by each generated registry.
- `ObserverComponent` owns connection projection state. Server projection emits one `Welcome` and observer-scoped `WorldChange` packs, with create batching, correction targeting, destroy records, RPC Scope filtering, and no tick advancement during bind/rebind.
- Runtime `WireCodec` is the only C-1 codec. It supports `Welcome`, `WorldChange`, `InputCommand`, `ConnectionSuperseded`, and `Error`; InputCommand accepts 0-16 command blocks, RPCs preserve all arguments, JSON members/UTF-8/hex/error codes are closed and canonical, and RPC arguments are capped at 64.
- `Sync<T>`, `SyncList<T>`, and `SyncDict<TKey,TValue>` are bound to the World host. Container mutation enters dirty projection and owner upload paths; persisted containers are encoded in snapshots and remote deltas apply on the replica.
- Generated declarations validate `claimBy`, derive Scope visibility from declarations, persist containers, bind containers, and use template factories. Username sample exercises owner/claim projection, snapshot restore, Welcome ordering, and container delta delivery.
- Entity binding mutation and World reads are owner-thread guarded. Takeover removes the old binding and emits `ConnectionSuperseded`; chat input validates room and connection generation before enqueueing.

## 2. Acceptance evidence

- R5-02 implementation commit: `3a14065e97c29c345b4332558392133eb223164b` (`rm00011/r5-02`).
- Runtime merge commit: `4137971` (`Merge RM-00011 r5-02 runtime delivery`).
- Container delta fix commit: `d003ccc`.
- Binding/codec boundary fix commit: `54bf50f`.
- Runtime `main` is at `54bf50f` and the Runtime worktree is clean.
- Username sample now has an explicit `FriendsContainerDeltaReachesClient` regression covering server mutation, C-1 encode/decode, and replica apply.
- Takeover, room/generation validation, multi-command input, multi-argument RPC, strict Welcome ordering, counter exhaustion, and multi-RPC chat mapping each have focused regression coverage.

## 3. Commands and actual outputs

- `dotnet test modules/ecs/tests/Lumio.GameRuntime.Ecs.Tests/Lumio.GameRuntime.Ecs.Tests.csproj --no-restore` -> 14 passed, 0 failed.
- `dotnet test modules/replication/tests/Lumio.GameRuntime.Replication.Tests/Lumio.GameRuntime.Replication.Tests.csproj --no-restore` -> 190 passed, 0 failed.
- `dotnet test modules/ecs/samples/username/tests/Lumio.GameRuntime.Samples.Username.Tests.csproj --no-restore` -> 6 passed, 0 failed.
- `dotnet test tools/gen-declarations/tests/Lumio.Tools.GenDeclarations.Tests/Lumio.Tools.GenDeclarations.Tests.csproj` -> 3 passed, 0 failed.
- Earlier Runtime main verification also passed Coordination 89/89 and Command 28/28; Config and GAS test projects report no test projects in this checkout.
- `git diff --check` -> no whitespace errors.

## 4. Deviations, risks, and incomplete items

- The generic Replication history/projection subsystem still contains its established `FullSnapshot`/`Delta` implementation for the broader runtime architecture. The gameplay C-1 contract source was rewritten to the five Runtime message types and no longer declares the legacy gameplay `chat.event`/`entity.identity` faces; removing the generic history subsystem would exceed R5-02 scope and break its existing contract tests.
- Admission APIs return an explicit `owner_thread_required` result when called off the Simulation owner thread. They do not mutate World from network threads; transport-facing callers must enqueue through `WorldManager` and perform the binding operation at the owner barrier.
- No native/provider or live socket smoke is available in this Runtime checkout. Verification is managed compile/test plus in-process server/client byte round-trips.
- Pre-existing command safety fixtures removed by the baseline R5 migration were not recreated; command tests still pass 28/28, but this remains a coverage follow-up rather than an acceptance blocker for the World Manager slice.

## 5. Downstream integration and knowledge

- Downstream consumers must use `WorldManager.Enqueue`, `Tick`, `DrainOutbox`, and the Runtime `WireCodec`; no second gameplay codec or local snapshot/delta envelope is valid.
- Server/client generated assemblies must be regenerated with `tools/gen-declarations` when component declarations change. Generated template factories and declaration JSON are checked in with their source changes.
- No architecture source contract or engine/wire file was changed by R5-02. This report is the architecture handback; Workflow R-00407 should be moved to acceptance only after reading this evidence back.

## TDD evidence

- RED: new tests initially failed for missing multi-command/container behavior and for accepting `WorldChange` before `Welcome`.
- GREEN: focused ECS, Replication, Username, and generator suites pass with the implementations above; the later container-delta test reproduced and then closed the replica-update gap.
- REFACTOR: generated templates, strict codec validation, owner-thread guards, and binding takeover behavior were kept in their existing module boundaries.
