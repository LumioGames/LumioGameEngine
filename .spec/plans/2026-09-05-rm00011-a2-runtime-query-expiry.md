---
name: 2026-09-05-rm00011-a2-runtime-query-expiry
description: RM-00011 A2 Runtime owner-thread query and expiry integration plan
metadata:
  type: doc
  status: 实施中
---

# RM-00011 A2 Runtime Query and Expiry Integration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development to implement this plan task-by-task with a fresh implementer and an independent reviewer for each task.

**Goal:** Finish R-00408 and the downstream R-00392/R-00393 gates by providing Runtime owner-thread expiry/query controls and completing Server, Client, Game, integration, and deep review evidence.

**Architecture:** Runtime owns all lifecycle and C-2 query authority. Typed in-process messages enter `WorldManager.Enqueue`, execute on the owner thread during `Tick`, and return internal query records through `DrainOutbox.queries`; C-1 network frames remain unchanged. Server, Client, and Game remain byte-forwarding consumers and do not store a second world, binding map, codec, timer, or oracle.

**Tech Stack:** C# .NET Runtime and tests, Rust Server host/tests, C# Client/tests, Node.js Game oracle/tests, JSON wire contract, Orca supervised workers and reviewers.

## Global Constraints

- Preserve ADR-060 C-1 message set and the HostEntry six-operation allowlist: `boot`, `enqueue`, `tick`, `drain`, `snapshot`, `restore`.
- All lifecycle and query mutations enter Runtime through `WorldManager.Enqueue`; no synchronous host bypass, host-minted NetEntityId, local Welcome, local FullSnapshot/Delta, u64 identity fallback, second codec, second binding/query map, or second timer.
- Runtime query results are internal bridge records in `drain.queries`; they are not C-1 wire frames and `WireCodec.EncodePack` must reject internal controls/results.
- Preserve S5/S10 deferred behavior, existing oracle judgment paths, generated source ownership, and the single World Manager per process.
- Every implementation task uses TDD: add a focused failing test, run the expected failure, implement the minimum behavior, run focused and repository gates, then commit.
- Every task has an independent reviewer; no Client or Game task starts until Server review is clean.
- Do not modify Workflow card states or acceptance items until evidence and review are complete; all write-backs use a local bundle, idempotency key, immediate GET read-back, and truthful status.

---

### Task 1: A2 architecture and C-2 contract

**Files:**
- Modify: `engine/wire/entity-binding-and-query-v1.json`
- Modify: `.spec/decisions/README.md`
- Add: `.spec/decisions/ADR-063-rm00011-a2-runtime-query-expiry.md`
- Modify: `.spec/knowledge/README.md`
- Modify: `.spec/knowledge/features/runtime-manager-controls.md`
- Add: `.spec/knowledge/features/runtime-manager-query-expiry.md`
- Test: `eng/verify-wire.mjs` and its existing contract tests

**Interfaces:**
- Consumes: ADR-060 C-1/C-2 shapes and Runtime `WorldManager.Enqueue`/`DrainOutbox`.
- Produces: exact names and fields for `ExpireEntityMessage`, `ResolveBindingMessage`, `AttributeQueryMessage`, and `drain.queries` records.

- [ ] Write a failing wire-contract assertion for the three request types, result fields, and unchanged C-1 message set.
- [ ] Run `node eng/verify-wire.mjs` and capture the expected missing-contract failure.
- [ ] Update the JSON contract, ADR index, feature index, and feature design with the exact A2 decision.
- [ ] Run `node eng/verify-wire.mjs` and `node --test eng/verify-wire.mjs`.
- [ ] Commit the architecture contract and document changes.

### Task 2: Runtime owner-thread controls and query results

**Files:**
- Modify: `C:/Work/LumioGames/LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldMessages.cs`
- Modify: `C:/Work/LumioGames/LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/IWorldControlAdapter.cs`
- Modify: `C:/Work/LumioGames/LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs`
- Modify: `C:/Work/LumioGames/LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WireCodec.cs`
- Modify: `C:/Work/LumioGames/LumioGameRuntime/modules/replication/src/Lumio.GameRuntime.Replication/Binding/EntityBindingQuery.cs`
- Modify: Runtime ECS/Replication tests covering controls, queries, expiry, and drain serialization

**Interfaces:**
- Consumes: Task 1 contract; existing `EntityBindingQuery` C-2 result methods.
- Produces: owner-thread `WorldMessage` request types, adapter response, and internal result records usable by HostEntry reflection.

- [ ] Add failing tests proving expiry/query requests are accepted as WorldMessages, rejected by `WireCodec`, and not applied before owner Tick.
- [ ] Add failing tests for all C-2 query outcomes and expiry tombstone behavior through queued messages.
- [ ] Implement the minimal typed messages, adapter response path, owner Tick dispatch, and result records.
- [ ] Run focused ECS/Replication tests, then full Runtime suites and formatting/build gates.
- [ ] Commit Runtime A2 changes and write the exact evidence report.

### Task 3: Server consume-only A2 bridge

**Files:**
- Modify: `C:/Users/g923/orca/workspaces/LumioServer/r5-03-server/entity-chat-host/src/Lumio.Server.EntityChat.HostEntry/HostEntry.cs`
- Modify: `C:/Users/g923/orca/workspaces/LumioServer/r5-03-server/modules/process/src/entity_chat/clr.rs`
- Modify: `C:/Users/g923/orca/workspaces/LumioServer/r5-03-server/modules/process/src/entity_chat/runtime.rs`
- Modify: Server host/tests and architecture assertions

**Interfaces:**
- Consumes: Runtime A2 request/result names and `drain.queries` records.
- Produces: `RuntimeSurface.expire`, `resolve_by_net_entity_id`, and `query_attribute` backed by enqueue/tick/drain correlation while HostEntry remains six-op.

- [ ] Add failing Server tests for expiry, binding resolution, query outcomes, malformed query records, and no extra HostEntry op.
- [ ] Implement raw JSON request construction and strict internal query-result parsing; C-1 frame parsing remains strict and opaque.
- [ ] Run locked Server focused tests, full workspace tests, fmt, clippy, HostEntry build, and scoped source scans.
- [ ] Commit Server A2 changes and write a five-part handback.

### Task 4: Independent Server review and R-00408 acceptance evidence

**Files:**
- Read-only: Server commit range and Runtime A2 commit
- Add: `.spec/reviews/2026-09-05-r5-03-a2-server-review.md`
- Modify: local Workflow bundle under `.workflow-drafts/rm00011/r5-03-a2-writeback/`

**Interfaces:**
- Consumes: Task 2/3 reports, exact commit SHAs, architecture contract, and acceptance items.
- Produces: clean review verdict, acceptance evidence, and truthful R-00408 transition/write-back only if all required evidence exists.

- [ ] Create a review package from the Server merge base through the A2 commit.
- [ ] Run independent source scans and the full applicable Server/Runtime gates in a separate review environment.
- [ ] Fix and re-review every P0/P1 finding before acceptance.
- [ ] Upload evidence comment and acceptance-item updates through a local bundle, then GET read back.

### Task 5: Client Runtime integration

**Files:**
- Modify: `C:/Users/g923/orca/workspaces/LumioClient/r5-03-client-exec/modules/replica/**`
- Modify: `C:/Users/g923/orca/workspaces/LumioClient/r5-03-client-exec/modules/session/**`
- Modify: `C:/Users/g923/orca/workspaces/LumioClient/r5-03-client-exec/modules/bot/**`
- Modify: `eng/project-reference-allowlist.json`, `modules/web/**`, and relevant tests only where required by the card

**Interfaces:**
- Consumes: Runtime `WorldManager`, `WireCodec.DecodePack/DecodeInput`, Welcome/WorldChange shapes, and Server A2 evidence.
- Produces: Client World/Replica and Bot.Host consuming Runtime only, one Tick per frame, no duplicate codec/parser/u64 fallback/local Welcome, and outbound DecodeInput coverage.

- [ ] Add failing tests for Runtime outbound DecodeInput, Welcome self binding, one Tick per frame, and removal of duplicate parser/codec paths.
- [ ] Implement minimal Client migration and UI ownership move to `modules/web` where required.
- [ ] Run focused and full Client tests/builds, source scans, and commit.

### Task 6: Game oracle integration

**Files:**
- Modify: `C:/Work/LumioGames/LumioGame/r5-03-game/integration/entity-chat/**`
- Modify: `verify-evidence.mjs`, scenario/log readers, and `modules/server-gameplay/**` only where required

**Interfaces:**
- Consumes: Server/Client Runtime A2 evidence and current WorldChange RPC fields.
- Produces: oracle reading `WorldChange.rpcs`, no mvp-host path, stable deterministic event/tick assertions, and updated ChatSetMessageSystem only if the Runtime signature requires it.

- [ ] Add failing oracle tests for the new fields and forbidden legacy paths.
- [ ] Implement the smallest oracle and gameplay adjustments.
- [ ] Run Node tests, dotnet tests/builds, source scans, and commit.

### Task 7: R-00408 integration smoke and R-00392

**Files:**
- Read-only all three implementation worktrees and Runtime
- Add: `.spec/reviews/2026-09-05-r4-09-integration.md`
- Modify: local Workflow bundle for R-00392 only after evidence exists

**Interfaces:**
- Consumes: reviewed Server, Client, Game commits and Runtime A2; environment variables `LumioRuntimeRoot`, `LumioClientRoot`, `LUMIO_GAME_ROOT`.
- Produces: real process/socket smoke logs, 101-entity/chat/reconnect/expiry evidence, deterministic repeatability comparison, and R-00392 evidence.

- [ ] Run the actual two-sided smoke and record Welcome -> WorldChange -> chat -> expiry logs without writing SUCCESS early.
- [ ] Run repository gates for all three repositories and compare two identical runs byte-for-byte after newline normalization.
- [ ] Write the integration review and upload truthful R-00392 evidence only after read-back.

### Task 8: R-00393 independent deep review and final handback

**Files:**
- Read-only final commits and all reports
- Add: `.spec/reviews/2026-09-05-r4-10-deep-review.md`
- Modify: local Workflow bundle for R-00393 and final RM-00011 handback

**Interfaces:**
- Consumes: R-00408/R-00392 evidence, ADR-056/057/058/060/061, all source scans and smoke logs.
- Produces: independent P0/P1/P2 verdict, final acceptance state, and Owner report with unresolved gaps.

- [ ] Review the complete diff in an isolated snapshot, including architecture contract and all three consumer repositories.
- [ ] Re-run representative gates and inspect evidence provenance, no duplicate authorities, no forbidden compatibility paths, and no unverified claims.
- [ ] Fix and re-review blocking findings.
- [ ] Upload R-00393 evidence and final RM-00011 handback; mark the goal complete only after every required item is verified.
