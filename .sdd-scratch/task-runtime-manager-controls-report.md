# Runtime Manager Controls Handback

## Scope

方案 A 已在 Runtime 仓 `C:\Work\LumioGames\.wt-runtime-controls-runtime` 实现：
网络线程只把准入、断开、重绑定意图封装为 Runtime 内部 `WorldMessage`，经
`WorldManager.Enqueue` 进入 Simulation Owner Thread。`EntityBindingQuery` 仍是唯一
绑定真相，`WorldManager` 只保存一个 `IWorldControlAdapter` 引用。

## Commits

- Architecture contract: `f07add7` (cherry-pick of `85fc257`)
- Runtime ECS: `3bba165` (`feat(ecs): route lifecycle controls through world manager`)
- Runtime Replication: `a230a74` (`feat(replication): enqueue binding lifecycle controls`)

## Verification evidence

Architecture repository:

```text
node eng/verify-wire.mjs
verify-wire: all contracts green (5 contracts)
node --test eng/verify-wire.mjs
28 passed, 0 failed
```

Runtime repository, executed serially with the project executable runner (the local
Microsoft Testing Platform `dotnet test` integration reports zero tests / exit 5):

```text
ECS: 17 passed, 0 failed
Replication: 195 passed, 0 failed
Username sample: 6 passed, 0 failed
GenDeclarations: 3 passed, 0 failed
```

Focused lifecycle evidence:

```text
EntityBindingQuery: 14 passed, 0 failed
```

The RED phase was observed before implementation: ECS failed with missing
`IWorldControlAdapter`; Replication lifecycle tests initially failed because queued
controls had no adapter path and emitted no lifecycle outbox messages.

The byte fixture command was:

```text
dotnet run --project .sdd-scratch/roundtrip/roundtrip.csproj --no-restore
round-trip OK: initial route C1, takeover route C2, order 0<1, manager string-keyed tables 0
```

The fixture enqueued admission, ticked, encoded/decoded `Welcome` and `WorldChange`,
then repeated takeover and checked encoded payload fields and message ordering. The
opaque `Connection` value is intentionally outbox routing metadata and is not a C-1
wire field; therefore it is asserted before encoding while decoded business fields
are asserted after round-trip.

## Contract boundary

The C-1 network set remains exactly `Welcome`, `WorldChange`, `InputCommand`,
`ConnectionSuperseded`, and `Error`. `AdmitConnectionMessage`,
`DisconnectConnectionMessage`, and `RebindConnectionMessage` are rejected by
`WireCodec.EncodePack` and never cross the network codec.

Successful admission is observed through the projected `Welcome`; no synchronous
`NetEntityId` is returned. Takeover preserves the entity, increments generation,
routes `ConnectionSuperseded` to the old connection, and emits the new `Welcome` to
the new connection.

## Generated artifacts and worktrees

No generated artifact was required by the Runtime controls implementation. The Runtime
worktree has only a pre-existing line-ending-only modification in
`modules/ecs/generated/attribute-declarations.json`; it was not staged or committed.
The temporary round-trip fixture lives under architecture `.sdd-scratch` and is not a
production or generated source file.

## Downstream resume

R5-03 Server can resume from its existing worktree. Its transport layer should enqueue
`AdmitConnectionMessage`, `DisconnectConnectionMessage`, and
`RebindConnectionMessage`, drain Runtime outbox messages, and route each message by its
opaque `Connection`. Live socket smoke remains part of the resumed Server task; this
Runtime handback does not add a second admission envelope or codec.
