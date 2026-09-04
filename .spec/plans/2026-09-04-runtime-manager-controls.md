---
name: 2026-09-04-runtime-manager-controls
description: Runtime 准入、断开与重绑定控制消息的契约与实现计划
metadata:
  type: doc
  status: 实施中
---

# Runtime Manager Controls Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development to implement this plan task-by-task (hosts without subagents: its Inline Fallback section). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a single Runtime owner-thread path for admission, disconnect, and rebind intent without adding a second wire codec or Manager-owned connection table.

**Architecture:** ECS owns typed in-process control messages and a narrow control-adapter interface. `WorldManager` dispatches controls during the server tick before normal input projection; Replication's `EntityBindingQuery` implements the adapter and remains the only binding authority. The C-1 network envelope remains unchanged; C-2 documents the internal control surface.

**Tech Stack:** C#/.NET 10 and netstandard2.1 Runtime, xUnit, JSON contract files, Node.js `eng/verify-wire.mjs`.

## Global Constraints

- C-1 wire messages remain exactly `Welcome`, `WorldChange`, `InputCommand`, `ConnectionSuperseded`, and `Error`.
- Network threads may only call `WorldManager.Enqueue`; World mutation and binding mutation run on the Simulation Owner Thread during `Tick`.
- `EntityBindingQuery` remains the sole binding authority; `WorldManager` must not own a connection-keyed binding table or persist connection refs.
- Host and client code continue to forward Runtime bytes; no local Welcome, FullSnapshot, Delta, u64 fallback, or second codec is added.
- Admission does not synchronously return `NetEntityId`; success is observed through the projected `Welcome` packet.
- Every behavior change has a failing test first and a focused regression test after implementation.

---

### Task 1: Extend the C-2 contract with internal Manager controls

**Files:**
- Modify: `engine/wire/entity-binding-and-query-v1.json`
- Modify: `eng/verify-wire.mjs`
- Test: existing `eng/verify-wire.mjs` contract tests plus new control assertions

**Interfaces:** Produces the authoritative `runtimeManagerControls` table. Controls are in-process and must not be added to the C-1 gameplay envelope.

- [ ] **Step 1: Write the failing contract assertions**

Require a closed `runtimeManagerControls.messages` object with `admit`, `disconnect`, and `rebind`; require `transport: in-process`, `entryPoint: WorldManager.Enqueue`, required field names, `rebind.mode` values `reconnect|takeover`, and a result that never returns `netEntityId`. Assert the C-1 message set remains the five existing wire messages.

- [ ] **Step 2: Run the contract tests**

Run `node --test eng/verify-wire.mjs`. Expected: FAIL because the C-2 control table is absent.

- [ ] **Step 3: Add the minimal schema and validator**

Add this object to `entity-binding-and-query-v1.json`:

```json
{
  "transport": "in-process",
  "entryPoint": "WorldManager.Enqueue",
  "messages": {
    "admit": { "type": "AdmitConnectionMessage", "required": ["connection", "accountId", "roomId", "entityType"], "entityType": ["player", "bot"] },
    "disconnect": { "type": "DisconnectConnectionMessage", "required": ["connection"] },
    "rebind": { "type": "RebindConnectionMessage", "required": ["connection", "accountId", "roomId", "mode"], "mode": ["reconnect", "takeover"] }
  },
  "result": "accepted-or-error-without-netEntityId",
  "connectionRouting": "adapter-callback",
  "persistence": "none"
}
```

Extend `eng/verify-wire.mjs` to reject unknown control keys, missing required fields, unsupported modes, a non-in-process transport, or a result description that permits synchronous entity IDs.

- [ ] **Step 4: Run `node --test eng/verify-wire.mjs` and confirm all existing and new tests pass.**
- [ ] **Step 5: Commit:** `git add engine/wire/entity-binding-and-query-v1.json eng/verify-wire.mjs; git commit -m "feat(wire): define runtime manager lifecycle controls"`

### Task 2: Add ECS control messages and the adapter boundary

**Files:**
- Create: `modules/ecs/src/Lumio.GameRuntime.Ecs/World/IWorldControlAdapter.cs`
- Modify: `modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldMessages.cs`
- Modify: `modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs`
- Modify: `modules/ecs/tests/Lumio.GameRuntime.Ecs.Tests/R5RuntimeContractTests.cs`

**Interfaces:**

```csharp
public interface IWorldControlAdapter
{
    bool TryHandle(WorldMessage message, out ErrorMessage? error);
    bool TryResolveConnection(NetEntityId observerId, out string connection);
}
```

`AdmitConnectionMessage`, `DisconnectConnectionMessage`, and `RebindConnectionMessage` are immutable `WorldMessage` values. `WorldManager.AttachControlAdapter` accepts one adapter; `DetachControlAdapter` clears the same instance.

- [ ] **Step 1: Add failing ECS tests**

Assert each control is a `WorldMessage`; assert `WireCodec.EncodePack` rejects each control; attach a recording adapter, enqueue an admission from a background task, assert the adapter runs only during owner `Tick`, and assert a returned error retains the original connection.

- [ ] **Step 2: Run focused tests and verify RED**

Run `dotnet test modules/ecs/tests/Lumio.GameRuntime.Ecs.Tests/Lumio.GameRuntime.Ecs.Tests.csproj --no-restore --filter FullyQualifiedName~R5RuntimeContractTests`. Expected: missing-type/API failures.

- [ ] **Step 3: Implement the ECS boundary**

Add constructor validation and the single adapter slot. In server `ApplyInputs`, dispatch lifecycle controls through the adapter before `InputCommandMessage` values, enqueue returned `ErrorMessage` responses, and continue with normal commit/projection. During `Project`, ask the adapter for the observer connection and use it only on emitted `WelcomeMessage` and `WorldChangeMessage`; do not store a connection dictionary in the Manager. Make `WireCodec.EncodePack` explicitly reject internal controls.

- [ ] **Step 4: Run focused tests and then the full ECS project; both must pass.**
- [ ] **Step 5: Commit:** `git add modules/ecs/src modules/ecs/tests; git commit -m "feat(ecs): route lifecycle controls through world manager"`

### Task 3: Connect EntityBindingQuery to the Manager control queue

**Files:**
- Modify: `modules/replication/src/Lumio.GameRuntime.Replication/Binding/EntityBindingQuery.cs`
- Modify: `modules/replication/tests/Lumio.GameRuntime.Replication.Tests/EntityBindingQueryTests.cs`
- Modify: `modules/replication/tests/Lumio.GameRuntime.Replication.Tests/TestBindingFactory.cs` only if registration requires fixture setup

**Interfaces:** `EntityBindingQuery` implements `IWorldControlAdapter`; `Create(manager)` registers it exactly once; `Dispose()` detaches it. `TryHandle` delegates to existing `Admit`, `Disconnect`, and `Rebind` methods and maps rejected outcomes to `ErrorMessage` without exposing a binding ID. `TryResolveConnection` synchronizes pending admissions before resolving the existing connection index.

- [ ] **Step 1: Add failing Replication tests**

Cover background-thread enqueue plus owner-tick admission and `Welcome.Connection == "C1"`; duplicate admission produces an outbox error for `C2`; disconnect unbinds on the next tick; takeover preserves the entity, increments generation, sends `ConnectionSuperseded` to the old connection, and sends the new `Welcome` to the new connection; disposing the query detaches the adapter.

- [ ] **Step 2: Run `dotnet test modules/replication/tests/Lumio.GameRuntime.Replication.Tests/Lumio.GameRuntime.Replication.Tests.csproj --no-restore --filter FullyQualifiedName~EntityBindingQueryTests` and verify RED.**
- [ ] **Step 3: Implement the adapter**

Dispatch the three control classes, parse `Mode` to the existing `RebindMode`, preserve direct API owner-thread guards, register/detach the adapter, and keep all connection/room indexes inside `EntityBindingQuery`.

- [ ] **Step 4: Run the focused tests and then the full Replication project; both must pass.**
- [ ] **Step 5: Commit:** `git add modules/replication/src modules/replication/tests; git commit -m "feat(replication): enqueue binding lifecycle controls"`

### Task 4: Regenerate, verify, and hand back the Runtime contract

**Files:**
- Modify only tool-produced generated files if the Runtime generator requires them.
- Create: `.sdd-scratch/task-runtime-manager-controls-report.md`

- [ ] **Step 1: Run architecture verification**

Run `node eng/verify-wire.mjs` and `git diff --check` from `LumioGameEngineArchitecture`. Expected: all five contracts green and no whitespace errors.

- [ ] **Step 2: Run Runtime suites**

```text
dotnet test modules/ecs/tests/Lumio.GameRuntime.Ecs.Tests/Lumio.GameRuntime.Ecs.Tests.csproj --no-restore
dotnet test modules/replication/tests/Lumio.GameRuntime.Replication.Tests/Lumio.GameRuntime.Replication.Tests.csproj --no-restore
dotnet test modules/ecs/samples/username/tests/Lumio.GameRuntime.Samples.Username.Tests.csproj --no-restore
dotnet test tools/gen-declarations/tests/Lumio.Tools.GenDeclarations.Tests/Lumio.Tools.GenDeclarations.Tests.csproj --no-restore
```

Expected: zero failures, including the new enqueue admission/rebind tests.

- [ ] **Step 3: Run a byte round-trip fixture**

Using the generated server registry, enqueue admission, tick, drain, encode/decode the `Welcome` and `WorldChange`, assert `Welcome.Connection == "C1"`, and repeat takeover to assert supersession precedes the new Welcome. Reflect over `WorldManager` to prove it has no connection-keyed table.

- [ ] **Step 4: Write the five-part handback**

Record actual commands/outputs, commit SHAs, owner-thread evidence, internal-vs-wire boundary, generated artifact status, and the downstream Server resume path. State that live socket smoke belongs to the resumed R5-03 Server task.

- [ ] **Step 5: Commit the report:** `git add .sdd-scratch/task-runtime-manager-controls-report.md; git commit -m "docs: record runtime manager controls verification"`

Finish with `git status --short --branch` in both repositories; both implementation worktrees must be clean before resuming R5-03 Server.
