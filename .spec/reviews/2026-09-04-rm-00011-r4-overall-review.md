---
name: 2026-09-04-rm-00011-r4-overall-review
description: RM-00011 r4 六仓整体进度与收口复核——Workflow 读回、六仓 origin/main 代码阅读与实测、17 项 Fixture 逐项结论、R4-02 单独复核与门禁；裁定 r4 是否放行时查
metadata:
  type: doc
  status: 已交付
---

# RM-00011 r4 整体 Review（2026-09-04）

Reviewer：独立 reviewer（本会话未执行任何 r4 卡）。审查环境：六仓 `origin/main` 与 Server `feat/r-00388-r4-02-self-drive` 经 `git archive` 物化到会话 scratchpad 的只读快照（同级目录布局，`LumioRuntimeRoot` / `LumioClientRoot` / `LUMIO_GAME_ROOT` 指向快照）。判定以**阅读源码**为主（先读 Runtime ECS 框架与 `modules/ecs/samples/username/` 样板，再逐仓核对消费方是否按样板接入），单测与 grep 只作旁证。未改任何实现仓、契约、验收项、Workflow 状态或 oracle；未 push；未删除任何分支 / worktree。本报告只写实际执行过的命令与其真实输出（G7）；没跑的写「未执行」或「无法复核」。

提示词来源：`plans/2026-09-04-rm-00011-r4-overall-review-prompt.md`（`c0bcef2`）。

## 0. 基线 SHA（`git fetch --prune` 后 `git rev-parse origin/main`，2026-09-04 12:4x CST）

| 仓 | origin/main | 与提示词入口的差异 |
|---|---|---|
| Arch `LumioGameEngine` | `c0bcef224135bea993a6ef3b5b3250d29ea188c5` | 提示词未给 SHA；本机 main 在审查中途被另一会话 ff 到同一 SHA（reflog `merge @{u}: Fast-forward`）；远端分支 `docs/rm00011-r4-overall-review` 已被删除 |
| Runtime `LumioGameRuntime` | `010ae46f87eb9aa6ad0c6075ffa86054f9f6f335` | 一致（代码基线 `7f198e5` 为 PR #33 squash，`origin/feat/r-00385-r4-05-single-world-r2` 已删除） |
| NativeCore `LumioNativeCore` | `70b9834f211f689c74589806e7e9cc7dd15a4a23` | 一致 |
| Server `LumioServer` | `4c7688b7aacdd037f08ef22f053a3d9e6af7e5a7` | 一致；R4-02 分支 `f8aef7728f376b942c9b7cf00e5798fa3aa8534d`（PR #33 OPEN，merge-base = `4c7688b`，不落后 main） |
| Client `LumioClient` | `f06d5e6220541d91c3435d8f42d64ec763c8364d` | 一致（本机主 worktree 停在 `feat/t-00003-wss-transport-adapter @ cb3cd65`，未用作证据） |
| Game `LumioGame` | `e7afb5b41a1520abe697e2a69f986b4d84dff3f0` | 一致 |

工具链：dotnet 10.0.400、cargo/rustc 1.94.0、node v26.4.0、macOS（Darwin 25.5）。无 `pwsh`、无 GNU `timeout`。

## 1. 一页结论

**保持 blocked，不放行；ADR-057 / ADR-058 保持 Draft。**

| 口径 | 数值 | 说明 |
|---|---|---|
| Workflow 加权进度 | **65%** | `(6 × 100 + 50) / 10`；读回见 §2 |
| 已合入 origin/main 的卡 | **6 / 10** | R4-01 / R4-05 / R4-07 / R4-03 / R4-04 / R4-06 |
| 交付完成度（已合入 + 可复核证据 + 无未决 P0/P1） | **3 / 10 = 30%** | R4-03 / R4-06 / R4-07；R4-01 / R4-05 / R4-04 各挂 P1（§3） |
| 最终目标完成度 | **0%（门禁未过）** | R4-02 未合入且 Workflow 无交回；R4-08 / R4-09 / R4-10 未开始；17 项 Fixture 通过 3、部分 9、不通过 5 |

一句话：**Runtime 的 ECS 框架本体按样板做到了**——`WorldManager` / `World` / `Sync<T>` / 生成三件 / `[ServerRpc]`·`[ClientRpc]` / 128 位发号 / `CreateFromSnapshot` 与 `modules/ecs/samples/username/` 逐文件一致，进程内七步链路成立；Client 与 Game 也确实改为持同一个 `WorldManager` 并用样板写法（`ClientBootstrap.Boot()`、`Self.Get<ChatComponent>().SendMessage`、`Commands.CreateFor`）。**但网线两端没有按样板接上**：C-1 只有 `chat.event` 是 128 位，`entity.identity` / `field.write` / `ConnectionSuperseded` 仍是 u64，创建记录不带字段值、字段变化包不上网线、欢迎消息无 wire 形态，客户端只能用本地 `InstanceId = 0` 拼 id；Client 把 Runtime 出站 `chat.input` 原字节当 Text 帧直接发，而 Rust 宿主只解析 C-1 JSON `InputCommand` 信封——Bot.Host 的发言到不了服务器。加上 Server 自驱 / 删账号表只在未合入的 PR #33、Runtime 生成的声明表 sha 已与契约内嵌表分叉、六仓没有任何一份真实进程日志，ADR-056 / 057 的核心验证在本轮无一项有跨进程证据。

## 2. Workflow 读回（R-00384…R-00393，2026-09-04 只读 GET）

连接：profile `lumiogamesengine`，`/projects/current` = `proj_b6979c277715a6c6c490a541ac69709b`（subdomain `lumiogamesengine`）。每张卡读了正文 + `acceptance-items` + `comments` + `attachments`（附件均为 0）。

| 卡 | displayKey | status / progress | 验收项 systemSemantic | 评论 | 合入证据评论钉的 SHA |
|---|---|---|---|---|---|
| R4-01 | R-00384 | done / 100 | 4 × passed | 2（开工 + 合入证据） | Arch `3bd6fd7`（PR #71，内容 `b518390`） |
| R4-05 | R-00385 | done / 100 | 4 × passed | 4（开工、前置修订、二轮重派、合入证据） | Runtime `7f198e5`（PR #33 squash of `13a52d2`） |
| R4-07 | R-00386 | done / 100 | 4 × passed | 2 | NativeCore `70b9834`（PR #7） |
| R4-03 | R-00387 | done / 100 | 4 × passed | 2 | Server `8ba3fe3`（PR #32；Owner 书面豁免 11-scenario 与 mvp-host 两个红 job） |
| R4-02 | R-00388 | **in_progress / 50** | **4 × not_started** | **1（只有开工评论，无交回 / 合入评论）** | 无 |
| R4-04 | R-00389 | done / 100 | 4 × passed | 2 | Client `1473cc9`（PR #16） |
| R4-06 | R-00390 | done / 100 | 4 × passed | 2 | Game `a080edf`（PR #11） |
| R4-08 | R-00391 | backlog / 0 | 4 × not_started | 0 | 无 |
| R4-09 | R-00392 | backlog / 0 | 4 × not_started | 0 | 无 |
| R4-10 | R-00393 | backlog / 0 | 4 × not_started | 0 | 无 |

读回与提示词表一致。注意：提示词说 R4-02 handback 是 `DONE_WITH_CONCERNS`——该 handback 在派工机临时目录（`C:\Users\g923\AppData\Local\Temp\grok-goal-…\R-00388-handback.md`），**不在任何仓也不在 Workflow**，本报告无法复核其内容；六仓内唯一 `DONE_WITH_CONCERNS` 是 Server `.wf-report-R-00359.md`（旧卡）。按提示词口径，R4-02 视为未完成交付。

## 3. 样板对照（代码阅读）

样板 = `knowledge/features/ecs.md` §4.5 与 Runtime `modules/ecs/samples/username/`（ADR-058 指定「以后所有 ECS 代码都按它写」）。

### 3.1 Runtime 框架本体：与样板一致（读 `modules/ecs/src/Lumio.GameRuntime.Ecs/**`）

| 样板要求 | 实现落点 | 结论 |
|---|---|---|
| ② 两端同一 `WorldManager.Create(registry, instanceId?)`，服务器必传、客户端不传；`Start(ownerThread)`；网络线程只 `Enqueue`；主线程 `Tick()` | `World/WorldManager.cs:45-60`（Server 缺 instanceId 抛、Client 传了抛）、`:109-120 Enqueue` 唯一跨线程入口、`:123-150 Tick` = ApplyInputs → CommitCreates → StampAndProject → ConsumeSave；`:101-102` 唯一 `CreateWorld` | 一致 |
| WorldEntity 由游戏声明、`World = true` 恰好一个、随世界诞生、`Single<T>()` 取 | `samples/username/EntityTypes/WorldEntity.cs`；`WorldManager.cs:225-232 SpawnWorldEntity`；`World.cs:142-156 Single<T>`；生成器 `tools/gen-declarations/SourceModel.cs:257` 非恰好一个即失败 | 一致 |
| ③ `Commands.Create<T>()` 下单、提交相发号（实例 ID + 计数器）、Awake → Start；客户端收创建记录按同模板建 → Awake → PostAttribute → Start | `EntityRecord.cs` `CommandBuffer.Create<T>` / `EntityOrder`；`WorldManager.cs:333-355 CommitCreates` + `World.cs:178-183 IssueId`；`:357-380 Appear`；客户端 `:548-575 ApplyWorldChange`（Awake → WriteField(silent) → InvokePostAttribute → Start） | 一致 |
| ④ `Sync<T>` struct，写 `.Value` 记脏、owner 客户端自动上行；`OnClientWrite(in SyncWrite, ref bool accept)` 拒则权威纠正；`[ServerRpc]` 客户端桩 / `[ClientRpc]` 服务器桩由生成器产 | `Sync/SyncTypes.cs:218-298`（struct + `SyncSlot` 绑定）、`World.cs:264-274 OnLocalWrite`（非服务器 + `Authority.Owner` → `EnqueueOwnerWrite`）；`WorldManager.cs:289-330 ApplyFieldWrite`（非本人 / 非 Owner 字段 / 钩子拒 → `PushCorrection`）；`generated/*/ChatComponent.g.cs` 客户端 `SendMessage => EmitServerRpc`、服务器 `OnChatMessage => EmitClientRpc` | 一致 |
| ⑤ 帧末打包下发；接收批先全部写入再统一触发 `OnXChanged`；默认 `Notify.Remote`（自己写不收） | `WorldManager.cs:400-480 StampAndProject` / `FieldsFor`（SuppressWriterEcho）；`:577-596` 先 WriteField 再集中 Invoke 钩子；`World.cs:264-270` 本端写只在 `Notify.All` 入 PendingHooks | 一致 |
| ⑥ `Get<T>()` 自己 / `Get<T>(id)` 别人 / `Each<T>` / `TypeOf(id).Is<T>()`；类型不进 ID | `Component.cs:24-36`；`World.cs:81-85,130-140,166-171`；`EntityTypeRef.cs`；`NetEntityId.cs` 仅 instance + counter | 一致 |
| ⑦ 存档 = `WorldSaveComponent.Save` ServerRpc → 提交相消费 → 字节交宿主；恢复 = `CreateFromSnapshot` 新世界只跑 `OnHydrate`，未 `[Persist]` 取默认 | `World/WorldSaveComponent.cs`；`WorldManager.cs:640-648 ConsumeSave`（`ISnapshotSink`）；`:63-99 CreateFromSnapshot`（头含 instanceId / nextCounter / tick / 序号 + 墓碑，`RestorePersist` + `OnHydrate`，`RebuildAccountIndex`）；`Snapshot/WorldSnapshotCodec.cs:17-70` | 一致 |
| 文件后缀即归属；一句命令产三件；共享文件不许非 Sync 状态字段 | `samples/username/*.Server.csproj` `DefaultItemExcludes **/*.Client.cs` + `LUMIO_SERVER`；`Directory.Build.targets:31-72 GenerateLumioEcsDeclarations`；`SourceModel.cs:165-171` lint | 一致 |
| 事件不存：每 Tick outbox，无历史 | `WorldManager.cs:650-656 ClearTickEphemera`；`World.cs` 无事件容器 | 一致 |

### 3.2 Runtime 内部偏离样板之处

- **适配层自己 `Tick()`**：`modules/replication/.../Binding/EntityBindingQuery.cs:107`（Admit）、`:181`（Expire）、`:270`（Spawn）在 `lock` 内直接 `_manager.Tick()` 以同步拿到 `AssignedId`。样板 ③ 说准入服务只在 ApplyInputs 相下单、提交由 owner 的 `Tick()` 做。后果：每次准入推进一次 `World.Tick`（101 个实体准入后 tick ≥ 101），Tick 号不再只由内核节拍驱动。
- **声明表有不来自组件类的行**：`tools/gen-declarations/CodeEmitter.cs:60-84` 硬编码 `EntityIdentity.entityType / claimedMark / unmappedMark` 与 `ChatComponent.lastMessagePersistOnly` 四行；`EntityBindingQuery.cs:332-333,350-351` 与 Client `ReplicaWorld.cs` `ReadAttribute` 对 `claimedMark` 答常量 `"mark"`。违反 ADR-058 §1「组件类是唯一真源」与 §8「薄适配层无自有存储」。
- **恢复后的实体永远不会作为创建记录下发**：`WorldManager.cs:443` `if (record.Hydrated) continue;` 把 `CreateFromSnapshot` 建出的实体排除在 `newCreates` 之外；`:402-410` 另有一份算出即丢的 `creates` 死代码。样板 ⑦ 的恢复世界若再接客户端，客户端收不到 101 个实体。
- **反射兜底**：`World.cs:236,254` 在非生成组件上 `GetField("AccountId") / GetField("Connected")`（生产路径有生成物时不走，但「反射破门」路径仍在）。
- **兜底 Manager**：`EntityBindingQuery.cs:48` `WorldManager.Create(registry, instanceId: 0x1000000000000001UL)`（第二个建 Manager 入口，实例 ID 写死）。
- **旧世界仍在**：`modules/ecs/src/.../EcsModule.cs:34 new EcsWorld(request)`，`EcsWorld` 被 `command` 模块 3 个生产文件引用（共 11 个生产文件）；`StructureAssertionTests` 扫不到 `command`。

### 3.3 消费方是否按样板接入

**Server（分支 `f8aef77`，main 未含）**
- 装载：`HostEntry.cs:178-206` 反射调 `ServerBootstrap.Boot(instanceId)`（样板 ②）→ 持一个 `WorldManager`，再用 Runtime 适配层 `EntityBindingQuery.Create(manager)` / `ChatCommandRuntime.Create(bindings)`；`Restore` 走 `ServerBootstrap.Restore`（`CreateFromSnapshot`）。**符合**「宿主只托管、只转发」。
- Tick：owner loop `host.rs:233-241` 每 `OWNER_PUMP_INTERVAL_MS` `drive_wall()`，`run_tick` 要内核 `advance_tick_frame` 真返回 `DISPATCH_TICK`（`:706-716`）→ `HostEntry.Tick` → `ChatCommandRuntime.RunTick` → `Manager.Tick()`。**符合**。
- 网线：`ChatCommandRuntime.cs:74-84 RunTick` 只把 outbox 里的 `ClientRpc` 转成 `ChatMessageEvent`；`WelcomeMessage` / 创建记录 / 字段变化包全部丢弃。`HostEntry.cs:507-580` 自己再实现一份 `entity.identity` FullSnapshot 编码（与 Runtime `ChatEnvelope.FullSnapshot` / `ChatPayload.EncodeIdentity` 重复），只写 `counter` + `entityType`，不带任何 Sync 字段值。**样板 ⑤「改名后其他客户端 log 是 name old -> ABCD (Sync)」在真网线上不可能发生。**
- 入站：`host.rs:815-843 on_wire` 只收 Text 帧，`parse_input_command_json`（`:952-977`）要求 `{"messageType":"InputCommand","commands":[{mappingId,payload(hex),payloadSha256}]}`。
- 会话表只存 `connection → (netEntityId, generation)`（`host.rs:137-170 Session`）；顶号 = Runtime `account_already_online` → 旧 egress 发 `ConnectionSuperseded` 再 close → `Rebind`（`:513-585`）。**符合** ADR-057 第 5 条 / ADR-058 §14。

**Client（`f06d5e6`）**
- 建世界：`modules/replica/src/Public/ReplicaWorld.cs:35` `ClientBootstrap.Boot()`（样板 ② 不传 instanceId）；`ReplicaWorld` 只是门面，无属性袋。**符合**。
- 发言：`modules/bot/host/BotHostResidentLoop.cs:51` / `modules/bot/src/Internal/HeadlessBotHost.cs:161` `world.Manager.World.Self.Get<ChatComponent>().SendMessage(...)`（样板 ④）→ 生成桩 `EmitServerRpc` → `WorldManager.EnqueueServerRpc` → outbox `InputCommandMessage(chat.input, payload)`。**符合**。
- **出站上网线不符**：`modules/session/src/Internal/ClientSession.cs:485-494 DrainReplicaOutbound` 把 `input.Payload`（LumioBinV1 4 字节长度前缀 + UTF-8 文本）原样 `TrySend(new EncodedFrame(...))`，`modules/connection/.../WebSocketClientConnection.cs:413-416` 以 Text 帧发出，**没有包成 C-1 `InputCommand` JSON 信封**（无 `messageType` / `mappingId` / hex `payload` / `payloadSha256`）。Server 的 `parse_input_command_json` 解析失败即静默丢弃（`host.rs:837-842` `if let Ok(...)`，无 else）。R4-04 的单测只在进程内断言 outbox（`ReplicaWorldRuntimeTests.cs:99`）与日志（`BotCadenceTests`），没有任何测试把出站字节喂给 C-1 解析器。
- 入站：`ReplicaWorld.cs:353-425 ApplyCommitted` 把 `entity.identity`（u64）拼成 `new NetEntityId(instanceId, counter)` 且 `Fields = Array.Empty`（`:376`），`chat.event` 转成 `OnChatMessage` ClientRpc；`:438-440` 自己合成 `WelcomeMessage(_manager.World.InstanceId /*=0*/, selfId)`；`:442` 每条消息后 `_manager.Tick()`（不是「主线程每帧 Tick」）。创建记录无字段值 → `PostAttribute` 收不到 `Name`；`OnNameChanged` 永远不会跨网线触发。
- 聊天窗口：`ReplicaWorld.cs:21 _chat` / `:224 CopyChatWindow` 留在 replica 层（ADR-058 §18 归 UI 层）。

**Game（`e7afb5b`）**
- `modules/server-gameplay/src/Lumio.Game.ServerGameplay/Chat/ChatSetMessageSystem.cs`：`Admit` → `manager.TryGetSession` + `manager.Enqueue(InputCommandMessage)`；`SetMessage` → owner 线程校验后 `manager.World.Get<ChatComponent>(id).SendMessage(text)`（样板 ④）；`ChatComponent` 来自 `Lumio.GameRuntime.Samples.Username.Server`（csproj `:19-20`）。**符合**，Game 不再持世界或队列。`Lumio.Game.EntityChat.Suite` 只做账号登录证据。

## 4. Findings（P0 = 0，P1 = 5，P2 = 16）

### P1

**P1-1 生成声明表与契约内嵌表 sha 分叉（ADR-056 Fixture 2 字面不成立）**
- 仓 / SHA：Runtime `010ae46` `modules/ecs/generated/attribute-declarations.json`；Arch `c0bcef2` `engine/wire/entity-binding-and-query-v1.json`（`attributeDeclarations.sha256`）与 `engine/wire/gameplay-command-envelope-v1.json`（`generatedAttributeDeclarations.sha256`）。
- 事实：契约内嵌表 6 条（`ChatComponent.lastMessage*` ×3、`EntityIdentity.*` ×3），sha `a47e92d6…`（1153 字节，与 ADR-056 接口段一致）。Runtime 当前生成物 11 条（多出 `IdentityComponent.accountId / connected / connectionGeneration / disconnectedAtTick / name`），2103 字节，sha `b659e4c1…`。`Directory.Build.targets:49/60` 把 server 侧生成物直接写到该文件。另见 §3.2：其中四行由 `CodeEmitter.cs:60-84` 硬编码，不来自任何组件类。
- 复现：`shasum -a 256 modules/ecs/generated/attribute-declarations.json` → `b659e4c1…`；`python3` 读两份 JSON 对比 attributeId 集合（本报告实跑）。`node eng/verify-wire.mjs` 41/41 绿只证明内嵌表自洽，不比对 Runtime 产物。
- 影响：ADR-056「生成的声明表与 C-1 / C-2 契约内嵌表逐字节一致」与 ADR-058 §12「C-2 契约声明表（json），sha 与契约一致」都不成立；宿主 / 客户端若照契约表做可见性判断，看不到 `IdentityComponent.name`。属契约缺口：R4-01 卡只改了 notes 没重嵌表，R4-05 把样板组件字段并入了 C-2 表；两卡交回物都没升级此项。需 Owner 裁决重嵌（Arch）还是收窄生成范围（Runtime）。

**P1-2 128 位 NetEntityId 与创建记录 / 字段同步未贯通 wire**
- 仓 / SHA：Arch `c0bcef2` C-1 `gameplay-command-envelope-v1.json:91`（`field.write.netEntityId: u64`）、`:113`（`entity.identity.netEntityId: u64`，且只有 `entityType` / `unmappedMark`，无 Sync 字段值）；Runtime `010ae46` `modules/replication/.../Chat/ChatEnvelope.cs:310`（`ConnectionSuperseded` 用 `TryRequiredUInt64`）、`modules/ecs/.../World/WireCodec.cs:37,51`（`field.write` 只编 `Counter`）、`.../Chat/ChatCommandRuntime.cs:74-84`（`RunTick` 丢弃 `WelcomeMessage` / 创建记录 / 字段变化包）；Server 分支 `HostEntry.cs:507-580`（自编 FullSnapshot 只含 counter + entityType）；Client `f06d5e6` `modules/replica/src/Internal/ReplicaNetIds.cs:14-18`（u64 兜底 = `new NetEntityId(本地 World.InstanceId, counter)`）、`ReplicaWorld.cs:376,438-440`（创建记录零字段、本地合成 Welcome，`InstanceId = 0`）。
- 事实：只有 `chat.event` 改成了 16 字节两段 u64（R4-01 交付）。客户端 World 的 `InstanceId` 只在收到 `WelcomeMessage` 时改（`WorldManager.cs:522-524`），而 C-1 没有欢迎消息形态，宿主也不发。R4-01 合入评论 known gaps 已自述「`entity.identity` / `field.write` / `ConnectionSuperseded` 的 `netEntityId` 仍为单 `u64`」，但没有卡承接。
- 复现：`grep -n '"netEntityId": { "type": "u64"' engine/wire/gameplay-command-envelope-v1.json`；`grep -n 'TryRequiredUInt64(root, "netEntityId"' …/ChatEnvelope.cs`。
- 影响：ADR-058 §3「创建记录 = EntityType + NetEntityId + 全部可见字段当前值」、§16「跨进程唯一」、Fixture 6「改名后其他客户端 `OnNameChanged` 收到 Sync」与 Fixture 15 在网线两端不成立；S7 / S8 / S11 的 sender 比对在客户端侧是 `(0, counter)` 而非服务器的 `(instanceId, counter)`。这是提示词 §5 最后一问的答案：**未被全链路解决**。

**P1-3 R4-02 未合入：Server `origin/main` 仍是 r3 形态，PR #33 两个 job 红，Workflow 无回写**
- 仓 / SHA：Server `4c7688b`：`modules/process/src/entity_chat/host.rs:154`（`account_sessions: HashMap<String,String>`）、`:458`（宿主自判顶号）；`modules/host-runtime/src/clock.rs:87`（生产 `SystemMonotonicClock::advance_ms`）；owner loop 无周期 `pump_wall_clock`。分支 `f8aef77` 删除了这些（`entity_chat_architecture.rs` 32/32 绿含 `host_crate_has_no_account_keyed_maps`、`production_system_clock_has_no_advance_ms_backdoor`、`owner_loop_expire_fires_without_harness_drive_kernel`）。
- PR #33（`gh pr view 33`）：`mergeStateStatus=UNSTABLE`；`README policy` ✓、`Cargo entity-chat acceptance (windows-latest)` ✓、`(ubuntu-latest)` ✓、`Cargo entity-chat 11-scenario` ✗（`BLOCKED: LUMIO_GAME_ROOT is not set`，0 秒失败）、`MVP C# host policy` ✗（`LiveElevenPathTests.GameplayAssemblyDiscoveryFindsSiblingLumioGame` / `BindingsQueryChatTickExpireSnapshotAndSecondRoomRunOnTestControl` 失败 + `MVP_HOST_MIRROR_UPSTREAM gone contract-mirror/fixtures/valid/replication-error.json`）。run `33831568845`。
- 本机 macOS（快照 `f8aef77`）：`cargo test --locked --workspace --no-fail-fast` → host-runtime 13 ✓、process lib 95 ✓、architecture 32 ✓、host 19 ✓、wire 10 ✓；**2 targets failed**：`entity_chat_acceptance`（BLOCKED，设 `LUMIO_GAME_ROOT` 后变为 `BLOCKED: account-server dll not found`）、`xtask policy::tests::current_tree_satisfies_policy`（`bots.rs` / `suite.rs` / `wire.rs` 直接 `thread::sleep`，`suite.rs` / `lib.rs` / `server.rs` / `world.rs` 直接 `thread::spawn` / `tokio::spawn`）。`cargo fmt --all -- --check` exit 0。`cargo clippy -p lumio-server-process --all-targets --locked -- -D warnings` **exit 101**：`modules/host-runtime/src/native_timer.rs:368` `needless_return`（分支新增的 `cfg(not(windows))` 段）；因 host-runtime 先编译失败，`bots.rs` 上的既有 clippy 失败在 macOS 无法观测。
- Workflow：R-00388 只有开工评论，4 条验收项 `not_started`，无交回 / 合入证据。
- 影响：ADR-056 Fixture 1 / 5、ADR-057 Fixture「自驱」、ADR-058 Fixture 1（宿主侧）全部只能在分支上成立；R4-09 前置未满足。

**P1-4 没有任何真实服务器 / 客户端日志可拉取；11 场景从未在 Rust 宿主上跑过（R4-09 未开始）**
- 仓 / SHA：Game `e7afb5b` `integration/entity-chat/logs/` 只有 `README.md`；`integration/entity-chat/fixtures/oracle-min/` 是 R-00390 自造最小样本（README 明言「不是收口日志」）。Server CI 11-scenario 每次 0 秒 BLOCKED（PR #32、#33 同）。
- 复现：`find integration/entity-chat/logs -type f` → 仅 README；`gh run view 33831568845 --log-failed`。
- 影响：ADR-056 Fixture 3（广播两轮一致）、4（S7 跨进程逐实体）、ADR-057「日志可复核」「自驱」、ADR-058 Fixture 4 / 5 在本轮无证据；R4-09 验收 4 条全部无法评估。

**P1-5 Client 出站 `chat.input` 没有 C-1 InputCommand 信封，Rust 宿主解析不到 Bot 发言**
- 仓 / SHA：Client `f06d5e6` `modules/session/src/Internal/ClientSession.cs:485-494`（`DrainReplicaOutbound` 直接 `TrySend(new EncodedFrame(input.Payload))`）、`modules/connection/src/Internal/Transport/WebSocket/WebSocketClientConnection.cs:413-416`（Text 帧原样发）；Runtime `010ae46` `WorldManager.cs:210-219 EnqueueServerRpc`（payload = `WireCodec.EncodeUtf8` 的 4 字节长度前缀 + UTF-8，不含信封）；Server `f8aef77` `host.rs:835-843 on_wire` + `:952-977 parse_input_command_json`（要求 JSON `messageType=InputCommand` + `commands[{mappingId,payload hex,payloadSha256}]`，解析失败静默丢弃）；C-1 `gameplay-command-envelope-v1.json` `InputCommand` 定义同此。
- 事实：Client 仓内没有任何生产代码把 Runtime 出站 `InputCommandMessage` 包成 C-1 信封（`grep -rn '"messageType"' modules/*/src` 只命中 hello 契约）；R4-04 的证据是进程内 outbox 断言与 `bot-host.ndjson` 日志计数，从未把出站字节交给 C-1 解析器；Server 侧 `bots.rs` 只读 Bot.Host 日志目录，不校验宿主是否收到。
- 影响：R4-09 一旦真跑，100 个 Bot 的 `chat.input` 全部在 `on_wire` 被丢，S6 / S11 不可能出现 `chat.event`；ADR-057 Fixture「Bot 归属」的「Bot.Host 常驻逐 Tick 发言」在网线上等于零发言。归属：R4-04（Client 会话层封包）+ R4-02（契约消费方对齐），需 R4-09 前修。

### P2

- **P2-1 Server `origin/main` 在 macOS 编不过（R4-08 AC1）**：`.cargo/config.toml` `rustflags = ["-Dwarnings"]` 下 `modules/host-runtime/src/native_timer.rs:10`（`ENTRY_SYMBOL` 未用）、`:87`（`GetApiV1` 未用）报 dead_code，`cargo test --locked --workspace` exit 101。分支 `f8aef77` 已 cfg 修正。
- **P2-2 Arch `eng/dev-build.sh` / `eng/dev-run.sh` 在仓内 mode 100644 且脚本只支持 Linux**：`git ls-tree origin/main eng/dev-build.sh` → `100644`；`dev-run.sh:25` 直接执行 `"$ROOT/eng/dev-build.sh"` → `Permission denied`（本机实跑 exit 126）。`chmod +x` 后 `dev-build.sh` 仍写死 `liblumio_engine_native.so` / `linux-x64`，且 `engine/native` workspace 依赖同级 `../LumioVoxelEngine/crates/*`（快照布局下 `failed to read …/LumioVoxelEngine/crates/lumio-voxel-world/Cargo.toml`）。R4-01 AC4「dev-run 两次 exit 0」只在 Windows `dev-run.ps1` 上证明过；本仓收口门槛在 macOS 不可用（附录 A4 / A14）。
- **P2-3 Runtime 第二条建 Manager 入口与旧世界**：见 §3.2（`EntityBindingQuery.cs:48`、`EcsModule.cs:34`、`Directory.Build.targets:49/60` 生成器 `--namespace Lumio.GameRuntime.Samples.Username` 写死样板命名空间）。R4-05 合入评论已列为 known gaps。
- **P2-4 Runtime 两轮确定性测试用同一投递顺序**：`modules/ecs/samples/username/tests/UsernameSevenStepTests.cs` `RunChatRound()` 两轮都 `Reverse()` 后投递，`DeterministicChatOrderAcrossTwoRuns` 证明不了「不同到达序 → 相同 roomSequence」。排序本身在 `WorldManager.cs:252`（按 `Sender` 排）。
- **P2-5 Client 聊天窗口仍在 replica 层而非 UI 层**：`modules/replica/src/Public/ReplicaWorld.cs:21` `_chat`、`:224 CopyChatWindow()`、`:397-419` 每 Delta 追加；ADR-058 §18「客户端聊天窗口归 UI 层，ECS 不存事件」。类名 `ReplicaWorld` 保留但已是 `WorldManager` 门面（`ReplicaWorld.cs:35`），无属性袋、无手写声明表（`ReplicaWorldRuntimeTests.cs:118` 断言 `class AttributeDeclarationTable` 不存在）。
- **P2-6 Client Bot.Host 自读根表槽位**：`modules/bot/host/NativeLoaderTimerAbi.cs:57-58` 反射取 `NativeEngineLease` 私有字段 `_library`，`:249` 自定义 `RootApiWithTimers` 结构按自己的槽布局 `Marshal.PtrToStructure`。不自写 `LoadLibrary`（用 `NativeEngineLoader.LoadFromBuildInfo`），但槽偏移是自写的；R4-04 合入评论以「NativeLoader 无 timer 包装（P2）」记录。另 `Lumio.Client.Bot.Host.csproj:24-25` 以旧仓名 `LumioGameEngineArchitecture` 做兄弟目录发现，本机布局（`LumioGameEngine`）下不设 `LumioArchRoot` 就编不出 `LUMIO_NATIVE_LOADER`，生产入口直接 `BLOCKED: Lumio.Engine.NativeLoader project was not found`（`FoundationHostCommand.cs:127`）。
- **P2-7 EntityType 继承无子类型测试**：`EntityTypeRef.Is<T>()` → `Registry.g.cs:88-95` 沿 `BaseType` 链判定、`SourceModel.cs:50,131` 解析基类名，实现存在；但六仓测试 `grep ': PlayerEntity'` 零命中，`Is<Base>()` 对子类型成立只在同类型上断言过（`UsernameSevenStepTests.cs:66-67`、Client `ReplicaWorldRuntimeTests.cs:33`）。
- **P2-8 ADR-057 第 1 / 10 条的落点未做**：Server `.spec/decisions/0008-csharp-mvp-host-frozen-after-adr-056.md`（23 行）无「修订记录」指向 ADR-057；Arch `knowledge/lessons.md` 无「合入闸 / 三次返工阈值 / 审与盖章分离敏捷期暂停」条目（`grep` 零命中）。无卡承接。
- **P2-9 旧仓名 `LumioGameEngineArchitecture` 引用仍在（R4-08 AC1）**：Runtime（`README.md`、`eng/generate-contracts.sh:143,185`、`.github/workflows/repository-policy.yml:93`、`src/Lumio.GameRuntime.GeneratedContracts/**` 生成头）、Server（`deny.toml`、`README.md`、`Cargo.lock`、`generated/lumio-architecture-contracts/**`、`tools/xtask/src/contracts.rs`、`contracts/architecture-contracts.lock.toml`）、Game（`README.md`、`.spec/**`、`integration/hello/**`）、Client（`modules/bot/host/Lumio.Client.Bot.Host.csproj:24-25`）。NativeCore 零命中。
- **P2-10 Game `scenarios.mjs` 仍走 mvp-host（R4-08 AC2）**：`integration/entity-chat/scenarios.mjs:17,226,239,266,464,518,534,549,576` `connectMvpHost` / `process: 'lumio-mvp-host'`。
- **P2-11 R4-02 分支 S10 未按 ADR-058 §11 标 deferred**：`f8aef77` `modules/process/src/entity_chat/suite.rs:835-880` 仍在同一进程用第二个 `ISO_ROOM` 跑「第二房间」并写 `scenarios.10.ok`；R4-08 AC2 / R4-09 AC1 要求 `deferred（ADR-058 §11）`。分支的 `rust-entity-chat-host.md`「待解决」也没提。
- **P2-12 R4-02 分支 xtask policy 红**：见 P1-3；`bots.rs` 未被本卡改动，`suite.rs` / `wire.rs` 是本卡文件。main 因 P2-1 编不过，无法判断是否既有。
- **P2-13 适配层自己 `Tick()`**：见 §3.2（`EntityBindingQuery.cs:107,181,270`）与 §3.3（Client `ReplicaWorld.cs:442` 每条消息后 `Tick()`）。样板是「owner loop / 主线程每帧 `Tick()`」；现状使 `World.Tick` 随准入次数和消息条数跳变，`appliedTick` 语义与「Tick 号来自内核节拍」脱钩。
- **P2-14 生成器硬编码声明行与常量答案**：`CodeEmitter.cs:60-84` 四行 + `claimedMark` 常量 `"mark"`（`EntityBindingQuery.cs:332-333,350-351`、Client `ReplicaWorld.cs` `ReadAttribute`）。与 P1-1 同源。
- **P2-15 恢复后的实体不下发创建记录 + 死代码**：`WorldManager.cs:443`（`Hydrated` 跳过）、`:402-410`（算出即丢的 `creates`）。
- **P2-16 HostEntry 自带第二份 FullSnapshot / `entity.identity` 编码器**：`HostEntry.cs:507-580`，与 Runtime `ChatEnvelope.FullSnapshot` / `ChatPayload.EncodeIdentity` 重复；`:375` 仍暴露 `list_bindings` op（未再作兜底覆盖，但入口保留）。

## 5. 十张 R4 卡逐卡状态

| 卡 | Workflow | 目标仓 origin/main | 状态判定 | 本报告实测证据 | 阻塞 | 下一动作 |
|---|---|---|---|---|---|---|
| R4-01 / R-00384 | done / 100 | Arch `c0bcef2`（含 `3bd6fd7`） | **已合入，有 P1** | `node eng/verify-wire.mjs` 41/41 pass（含 `account_already_online`、`field.write`、`senderNetEntityIdInstanceId/Counter` 三条新用例）；`node .spec/tools/spec-lint.mjs` OK；`node eng/generate-abi.mjs` 生成物零 diff；ADR-049 / 053 各有「修订记录（2026-09-03，ADR-058）」；ADR-057 / 058 接口段无待填项；`ecs-entity-chat.md:107` 已写日志口径；`eng/dev-run.*` 改启 Rust 宿主 | P1-1（契约内嵌表未随生成物重嵌）、P1-2（`entity.identity` / `field.write` / `ConnectionSuperseded` 仍 u64、创建记录无字段）、P2-2（`.sh` 无执行位且仅 Linux） | Owner 裁决 P1-1 / P1-2 归属后补契约卡；`dev-run.sh` 修执行位与平台 |
| R4-05 / R-00385 | done / 100 | Runtime `010ae46`（代码 `7f198e5`） | **已合入，有 P1** | 框架本体与样板逐项一致（§3.1）；`dotnet build` 4 个测试工程 exit 0；`dotnet exec`：Ecs.Tests 9/9、Replication.Tests 181/181、Username.Tests 3/3、GenDeclarations.Tests 3/3；`dotnet test` Ecs.Tests 9/9；重建两个 username csproj 后 `diff -r` 生成物零 diff（仅 `.gitignore` 忽略的 `gen.hash` 新增）；禁词零命中；`Sync<T>` 为 struct；`NetEntityId` 128 位；`CreateFromSnapshot` 后 id 不变、新建不重号、账号索引重建 | P1-1、P2-3、P2-4、P2-7、P2-13、P2-14、P2-15 | P1-1 与 Arch 一起裁决；P2 留 R4-08 或后续卡 |
| R4-07 / R-00386 | done / 100 | NativeCore `70b9834` | **已交付** | `cargo test -p lumio-timer --locked` 全绿（wall_clock_mode 7/7 等，exit 0）；`crates/` grep `extern "C" fn timer_|provider_engine_root_api|ClientTimerManager|ServerTimerManager` 零命中；`lumio-native-ffi/src/` 无 `timer.rs`；ADR 0008 有「修订记录（2026-09-03，ADR-057 第 9 条）」 | 无 | 无 |
| R4-03 / R-00387 | done / 100 | Server `4c7688b` | **已交付** | 六仓 grep `DOTNET_STARTUP_HOOKS` / `bot_startup_hook` 仅命中 Arch 文档；`modules/process/src/entity_chat/bots.rs` 只做发现 + 拉起 `Lumio.Client.Bot.Host` + 读 `bot-host.ndjson`，缺日志时 `BLOCKED: 等 R4-04`；`tests/verify_rust_evidence.mjs` 不存在（分支架构测试 `rust_second_oracle_verify_rust_evidence_is_removed` ✓）。main 的 `cargo test` 在 macOS 因 P2-1 编不过，Windows 证据只在 CI（PR #32） | P2-1（macOS 编译，归 R4-08） | 无 |
| R4-02 / R-00388 | in_progress / 50 | Server `4c7688b`（未含）；分支 `f8aef77` PR #33 | **仅分支，未完成** | 见 P1-3、§3.3；分支实测：owner loop `host.rs:233-241` 每 `OWNER_PUMP_INTERVAL_MS` `drive_wall()`（真 `pump_wall_clock`）、`run_tick` 要求内核 `advance_tick_frame` 真返回 `DISPATCH_TICK`、Tick 号来自 Runtime `RunTick`；`clock.rs` 生产钟无 `advance_ms`（结构测试）；会话表只存 `connection → (netEntityId, generation)`；顶号 `account_already_online → ConnectionSuperseded → Rebind` 有真 socket 测试；`entity_chat_replay.rs --restore-snapshot` 进程 B 逐实体比对代码存在；`log.rs` 写 `server.ndjson` | PR 两个 job 红；clippy 红（macOS）；xtask policy 红；S10 未 deferred；Workflow 无交回；无真实运行日志；P1-5 对端（入站信封）；P2-16 | 退回：修 clippy / policy、S10 deferred、Workflow 五段交回；与 R4-04 对齐 `chat.input` 信封；11-scenario 与 MVP policy 由 Owner 决定豁免或修 |
| R4-04 / R-00389 | done / 100 | Client `f06d5e6` | **已合入，有 P1** | 按样板接入（§3.3）；`dotnet test LumioClient.slnx` exit 0：ArchitectureTests 38/38（含 `ReplicaAndBotReferenceRuntimeEcs`）、Replica 40/40（4 skip）、Session 35/35（含真 WebSocket loopback 顶号不重连两例）、Bot 12/12（含 `ProductionResidentLoopTicksAndLogsChatInputAfterAwait`）、IntegrationTests 12/12 等；`eng/project-reference-allowlist.json` Replica / Bot 引用 Runtime `Ecs` / `Replication` / `Samples.Username.Client`；`ReplicaWorld` 门面用 `ClientBootstrap.Boot()`（不传 instanceId）；`CreateRecordRunsAwakePostAttributeStart` ✓；`GameplayCodec` 非 JSON / 非 C-1 → `bad_envelope`；`FoundationHostCommand.cs:134-138` 生产构造 `NativeLoaderTimerAbi` + `ClientTimerManager`；`BotHostResidentLoop.cs:51` 逐 Tick `SendMessage` ServerRpc 并写 `chat.input` 日志 | P1-5（出站无 C-1 信封）、P1-2 影响（u64 兜底、创建记录零字段）、P2-5、P2-6、P2-13 | 退回修 P1-5（会话层把 `InputCommandMessage` 包成 C-1 `InputCommand` 信封，并加「出站字节可被 C-1 解析器解出」的测试） |
| R4-06 / R-00390 | done / 100 | Game `e7afb5b` | **已交付（P2）** | 按样板接入（§3.3）；`node --test integration/entity-chat/verify-evidence.mjs integration/entity-chat/web/chat-window.test.mjs` 15/15 pass；`verify-evidence.mjs --dir fixtures/oracle-min` exit 0（单测样本，非收口日志）；`compareRuns` 逐位（`:371-390`）+ 「同多重集不同顺序必须 FAIL」「只比长度必须 FAIL」两例；`oracleSha256` `\r\n → \n`（`:33-34`）；`dotnet exec` ServerGameplay.Tests 23/23（`dotnet test` MTP 报 Zero tests exit 5，与交回一致）；`typeof(ChatComponent).Assembly == Lumio.GameRuntime.Samples.Username.Server` 断言；`server-gameplay` csproj 引用 Runtime `Ecs` + `Samples.Username.Server` | P2-10（scenarios.mjs 归 R4-08）；logs/ 空（归 R4-09） | 无 |
| R4-08 / R-00391 | backlog / 0 | 无分支 | **未开始** | 三仓旧仓名仍在（P2-9）；`scenarios.mjs` 仍 mvp-host（P2-10）；Server main macOS 编不过（P2-1）；`EntityChatSuite` S10 未 deferred（P2-11） | 等 R4-02 合入（热点文件） | 派卡 |
| R4-09 / R-00392 | backlog / 0 | 无分支、无日志目录 | **未开始** | `logs/` 仅 README；CI 11-scenario 从未真跑 | R4-02 / R4-08 未完成；P1-5 会让 Bot 零发言；P1-2 会让 sender 比对失真 | 等前置 |
| R4-10 / R-00393 | backlog / 0 | 无报告 | **未开始** | 本报告不是 R4-10（R4-10 前置是 R4-09） | R4-09 | 等前置 |

## 6. 17 项 Fixture 逐项结论

口径：只凭文件存在、单测、发送计数或常量字段不判通过。「通过」= 字面条件在 `origin/main` 上有可复核行为证据；「部分」= 部分成立或只在未合入分支 / 进程内成立；「不通过」= 字面条件不成立或无证据。

### ADR-056 六项

| # | Fixture | 结论 | 命令 / 关键输出 | 引用 |
|---|---|---|---|---|
| 1 | 依赖方向 / 宿主无绑定表 | **部分** | Game csproj `ProjectReference $(LumioRuntimeRoot)/modules/ecs/...Ecs.csproj` + `Samples.Username.Server.csproj`；Client allowlist + `ReplicaAndBotReferenceRuntimeEcs` ✓（38/38）；Server **main** `grep -rn account_sessions modules/process/src` → `host.rs:154,214,458` 命中；分支 `host_crate_has_no_account_keyed_maps` ✓ | Game `modules/server-gameplay/src/Lumio.Game.ServerGameplay/*.csproj:19-20`；Client `eng/project-reference-allowlist.json`；Server `host.rs:154` |
| 2 | 标注生成零 diff 且与契约一致 | **不通过** | 一句命令产三件 ✓（`Registry.g.cs` / `Sync.g.cs` / `attribute-declarations.json`，MSBuild 目标 `GenerateLumioEcsDeclarations`）；重建后 `diff -r -x gen.hash` 零 diff ✓；但 Runtime 表 sha `b659e4c1…`（11 条）≠ C-1 / C-2 内嵌 `a47e92d6…`（6 条）；四行由生成器硬编码 | P1-1、P2-14 |
| 3 | 广播两轮一致（顺序） | **不通过（无真实日志）** | Runtime `WorldManager.cs:252` 按 `Sender` 排序 ✓；oracle 逐位 ✓；`DeterministicChatOrderAcrossTwoRuns` ✓ 但两轮同序投递（P2-4）；Game `logs/` 无任何 server / client 日志；CI 11-scenario BLOCKED；且 P1-5 使 Bot 发言到不了宿主 | P1-4、P1-5 |
| 4 | 快照：进程 A 落盘 → 进程 B `CreateFromSnapshot` 101 逐实体 | **不通过（无跨进程证据）** | Runtime 进程内：`CreateFromSnapshotRebuildsAccountIndexAndAdmitRebinds` ✓、Username 恢复后 `Name`/`AccountId` 回来、`Connected=false` ✓；Server 分支 `entity_chat_replay.rs:65-190` 进程 B 逐实体比对代码存在但从未运行（本机 BLOCKED，CI BLOCKED） | Runtime `WorldManager.cs:63-99`；Server `entity_chat_replay.rs` |
| 5 | 定时：自驱 Tick + 墙钟 pump；到期内核回调；无 `advance_ms` | **部分（仅分支）** | 分支 `host.rs:233-241` owner loop `drive_wall()` + `recv_timeout(OWNER_PUMP_INTERVAL_MS)`；`owner_loop_expire_fires_without_harness_drive_kernel` / `owner_loop_pumps_wall_clock` / `wall_clock_kernel_expire_tombstones_a_and_creates_b` / `production_system_clock_has_no_advance_ms_backdoor` 全 ✓（Fake clock 注入）；main `clock.rs:87` 仍有生产 `advance_ms`；真内核回调只在 Windows 可跑（`native_timer.rs:365-369` 非 Windows 直接 BLOCKED）；Runtime 侧准入自 `Tick()`（P2-13） | P1-3 |
| 6 | 顶号：`account_already_online` → 旧连接先收 `ConnectionSuperseded` 再关 → 客户端回登录不重连 | **部分** | Runtime `AdmitIssues128BitIdAndSecondConnectionIsAlreadyOnline` / `ShapeErrorIsNotAccountAlreadyOnline` ✓（`EntityBindingQuery.cs:84-93`）；Server 分支 `takeover_sends_connection_superseded_before_close` / `second_live_admit_is_account_already_online_then_superseded_rebind` ✓（`host.rs:513-585`；main 无）；Client `SessionConnectionSupersededTests` 两例（真 `TcpListener` WebSocket）→ `ClientSessionState.Superseded`，`HandleDisconnect` 不走 `StartGeneration`（`ClientSession.cs:450-457`），需显式 `Login` 才离开 | P1-3；Client `ClientSession.cs:426-466` |

### ADR-057 四项

| # | Fixture | 结论 | 命令 / 关键输出 | 引用 |
|---|---|---|---|---|
| 7 | 两轮一致 = 顺序一致，非多重集 | **通过（尺子）** | `verify-evidence.mjs:371-390` 逐位；测试 608 / 623 两例 ✓（15/15）；Server `entity_chat_acceptance.rs:337-352` 仍 `assert_eq!(order1, order2)` 且调用 Game oracle（`:46-59`），一把尺；`tests/verify_rust_evidence.mjs` 不存在 | Game `verify-evidence.mjs` |
| 8 | 证据只认结构化日志，可拉取，行尾归一化自校验 | **不通过（无日志）** | oracle 只读 `round-N/server/*.ndjson` + `client/*.ndjson`、sha 先 `\r\n→\n`（`:33-34`，测试 540 ✓）；Server 分支 `log.rs` 写 `{log-dir}/server.ndjson`；Client 写 `bot-host.ndjson`；但 Game `logs/` 只有 README | P1-4 |
| 9 | Bot 行为在 `Lumio.Client.Bot.Host` 进程执行；Server 不注入 | **不通过** | Server 六仓 grep `DOTNET_STARTUP_HOOKS` 零命中（仅 Arch 文档）✓；Client 生产入口 `FoundationHostCommand.cs:134-138` → `BotHostResidentLoop.cs:51` `Self.Get<ChatComponent>().SendMessage` 在 Client 进程执行 ✓；**但**出站没有 C-1 信封（`ClientSession.cs:485-494`），宿主 `parse_input_command_json` 解不出即丢（P1-5）——Bot 的发言到不了服务器；另 Bot.Host 自读槽位（P2-6） | P1-5、Client `modules/bot/host/*` |
| 10 | Server 自驱与 Runtime Tick 来源真实 | **部分（仅分支）** | 分支 `run_tick` 要求 `advance_tick_frame` 返回 `DISPATCH_TICK`（`host.rs:711-716`），`kernel_tick_frame_runs_runtime_tick` ✓；`HostEntry.cs:439` → `ChatCommandRuntime.RunTick` → `Manager.Tick()`，Tick 号 = `World.Tick`；main 不满足；无 harness 的真实进程运行只能在 Windows；Runtime 准入自 `Tick()` 使 Tick 号额外跳变（P2-13） | P1-3 |

### ADR-058 七项

| # | Fixture | 结论 | 命令 / 关键输出 | 引用 |
|---|---|---|---|---|
| 11 | 单进程单 Manager 单 GameWorld；WorldEntity 游戏声明且唯一 | **部分** | `WorldManager.cs:101-102` 唯一 `CreateWorld`；`EntityTypes/WorldEntity.cs` `[EntityType(Mode.CS, World = true)]`；`SourceModel.cs:257` 非恰好一个即生成失败；`grep WorldId\((1|2|370)\)` 生产源零命中；但 `EntityBindingQuery.cs:48` 第二入口 + 旧 `EcsWorld`（P2-3）；宿主侧单 Manager 只在分支 `HostEntry.cs` | P2-3 |
| 12 | `Sync<T>` / `SyncList` / `SyncDict` 与字段钩子；共享文件禁普通状态字段 | **通过（Runtime 内）** | `SyncTypes.cs:218-298` struct + `SyncSlot`；`:300,345` 容器；`CodeEmitter.cs:295-299` 产 `OnXChanging/Changed` + `OnClientWrite`；`SourceModel.cs:171` 共享文件非 Sync 状态字段 → LintErrors，GenDeclarations.Tests 3/3；Username 七步：`this-name-is-way-too-long -> ABCD (Correction)` 出现、`ABCD -> ABCD (Sync)` 不出现 ✓ | Runtime `Sync/SyncTypes.cs`、`tools/gen-declarations/*` |
| 13 | EntityType `abstract class` + C# 继承；`TypeOf(id).Is<Base>()` 对子类型成立；类型不进 ID | **部分** | 样板三个 EntityType 均 `abstract class` 无成员；`Registry.g.cs:88-95` 沿 `BaseType` 判定；`NetEntityId` 只含 instance + counter ✓；但无子类型测试（P2-7） | P2-7 |
| 14 | 客户端同一 `WorldManager.Create(GeneratedRegistry.Instance)` 不传 instanceId；Awake → PostAttribute → Start | **通过（进程内）** | `ClientBootstrap.Client.cs:14` ✓；`WorldManager.cs:57-59` 传了 instanceId 抛异常；`SevenStepUsernameDemo` `LifecycleOf(player) == [Awake, PostAttribute, Start]` ✓；Client `CreateRecordRunsAwakePostAttributeStart` ✓（40/40）。注：真网线上创建记录零字段、Welcome 本地合成（P1-2） | Runtime / Client 测试 |
| 15 | NetEntityId = 实例 ID + 计数器 128 位；快照含身份表与发号器；恢复后不复用 | **部分** | `NetEntityId.cs` 两段 u64 + 32-hex ✓；`WorldSnapshotCodec` 头含 `InstanceId / NextCounter / Tick / tombstones`，`CreateFromSnapshot` 回填 ✓；Username 恢复后新建 `Counter >= 旧` 且 ≠ ✓；但 wire 只有 `chat.event` 是 128 位（P1-2） | P1-2 |
| 16 | 同进程双端 = 两个 Manager + 环回；事件 `[ClientRpc]`；聊天窗口归 UI | **部分** | `UsernameSevenStepTests.Pump()` 两个 Manager `DrainOutbox → Enqueue` ✓；`ChatComponent.OnChatMessage` `[ClientRpc(Scope.Room)]` ✓；Client `ReplicaWorld._chat` 仍在 replica 层（P2-5） | P2-5 |
| 17 | 绑定 / 查询 / 聊天 / 存档同一 Manager；无 `_values` / 第二 ChatComponent / 无界历史 / 模块自建世界 | **部分** | `EntityBindingQuery` / `ChatCommandRuntime` / Game `ChatSetMessageSystem` 全部持 `WorldManager` ✓；禁词（`_values` / `_liveConnectionByAccount` / `_eventsByRoomTick` / `_displayed` / `ChatIngressWorld`）生产源零命中（只在测试断言与文档）；Game `class ChatComponent` 零命中 + 程序集断言 ✓；`EventOutboxDoesNotGrowAcrossOneThousandTicks` ✓（只数每 Tick 必清的三个 list，弱证据）；`command` 模块仍持旧 `EcsWorld`（P2-3）；`EntityIdentity.*` 伪属性无组件后盾（P2-14） | P2-3、P2-14 |

汇总：通过 3（#7 #12 #14）、部分 9（#1 #5 #6 #10 #11 #13 #15 #16 #17）、不通过 5（#2 #3 #4 #8 #9）。

## 7. R4-02（R-00388）单独复核

| 复核项 | 结论 | 证据 |
|---|---|---|
| 分支 `f8aef77` 与 `origin/main` 对齐 | 是 | `git merge-base origin/main f8aef77` = `4c7688b`（= origin/main）；`git log origin/main..f8aef77` 5 个提交（`be9c28d` + 3 个 cfg-gate 修正 + merge） |
| PR #33 检查完成 | 完成但两个 job 红 | README ✓、cargo acceptance windows ✓ / ubuntu ✓、11-scenario ✗（BLOCKED `LUMIO_GAME_ROOT`）、MVP C# host policy ✗；`mergeStateStatus=UNSTABLE`，`reviewDecision` 空 |
| Workflow 评论 / 验收项回写 | 未回写 | 1 条开工评论，4 条验收项 `not_started` |
| Windows `cargo test` / `cargo fmt` / `spec-lint` 可复核 | 部分 | Windows cargo 只在 CI job（run `33831568845`）可见，本报告未取其日志；fmt 本机 exit 0；`spec-lint` 未跑（Server 仓） |
| 11-scenario | 不得当作通过 | CI 与本机均 `BLOCKED`；本机设 `LUMIO_GAME_ROOT` 后下一道 BLOCKED 是 `account-server dll not found`，再建好 account-server / HostEntry 后是 `hostfxr missing (set LUMIO_HOSTFXR or DOTNET_ROOT)`，再设 `DOTNET_ROOT` 后是 `LUMIO_ENGINE_NATIVE missing`（macOS 原生库无法在快照布局下构建，附录 A14）；即便原生库存在，`sdk_loader.rs:555-559` 与 `native_timer.rs:365-369` 在非 Windows 直接 `UnsupportedPlatform` / BLOCKED |
| clippy 既有失败 | 无法证实「既有」；分支引入新失败 | macOS clippy 先在 `native_timer.rs:368` `needless_return` 失败（分支新增段），`bots.rs` 不可观测 |
| Ubuntu / macOS 编译 | Ubuntu 有真实 CI 运行（SUCCESS）；macOS 本机真实运行 | 见 P1-3 |
| HostEntry 活体 CLR Boot / R4-09 常驻 Tick / S10 后置 | 未标为风险 | 分支 `rust-entity-chat-host.md`「待解决」只写产物路径依赖与 mvp-host 冻结；S10 仍在跑（P2-11）；CLR Boot 只有单元 / 结构断言，无活体日志 |
| C-1 `entity.identity` 128 位缺口全链路解决 | **否** | P1-2 |
| 入站 `chat.input` 信封 | 与 Client 出站不匹配 | P1-5 |

处置：**退回 R4-02**（清单：修 clippy `needless_return`、xtask policy 对 `suite.rs` / `wire.rs` 的 sleep / spawn、S10 改 deferred、Workflow 五段交回、与 R4-04 对齐 `chat.input` 信封；11-scenario / MVP policy 两个红 job 需 Owner 书面决定豁免或修）。合入前不得标 done。

## 8. 前置 DAG 与门禁

```text
Wave 0  R4-01 ✓合入（P1-1 / P1-2 待裁决）
Wave 1  R4-05 ✓合入（P1-1）  ‖  R4-07 ✓  ‖  R4-03 ✓
Wave 2  R4-02 ✗仅分支 PR #33  ‖  R4-04 ✓合入（P1-5）  ‖  R4-06 ✓
Wave 2b R4-08 ✗未开始（热点文件等 R4-02）
Wave 3  R4-09 ✗未开始（前置 R4-02 / R4-08 未满足；P1-5 令 Bot 零发言、P1-2 令 sender 比对失真）
Wave 4  R4-10 ✗未开始（前置 R4-09）
```

门禁（顺序不可换）：
1. **R4-02 合入**：PR #33 必过 job 绿（或 Owner 对 11-scenario / MVP policy 的书面豁免，照 PR #32 先例）+ clippy / policy 绿 + Workflow 交回 + reviewer 通过。
2. **P1-5 修复**（R4-04 会话层封 C-1 信封 + 出站字节可被 C-1 解析器解出的测试）——否则 R4-09 必然 BLOCKED。
3. **P1-1 / P1-2 契约裁决**：由 Owner 决定重嵌表与 128 位 wire 编码 / 创建记录字段 / 欢迎消息的归属卡；否则 R4-09 的 sender 比对与可见性判断以哪份表为准无法确定。
4. **R4-08**：macOS / Linux 可编译、旧仓名、`scenarios.mjs`、S10 deferred。
5. **R4-09**：真实两轮日志入库 + macOS / Windows `verify-evidence.mjs --dir` exit 0 + Server acceptance job 真跑绿。
6. **R4-10**：独立深审复核 17 项 → 才能提请 Owner 会话把 ADR-057 / ADR-058 转 Accepted。

## 9. 清理建议（只建议，本报告未删任何东西）

原则：有未提交改动的目录不删；仍可能承载修正的分支不删；Runtime / NativeCore / Server / Client / Game 用 squash 合入，`git branch -r --merged` 判不出 squash 分支，删前以 PR 状态为准。

| 仓 | 可删（干净且已合入） | 必须保留 | 备注 |
|---|---|---|---|
| Arch | worktree `.wt-docs-consolidation`（`docs/2026-09-01-spec-consolidation @ 161c85d`，dirty=0，已合入）、`.wt-rm00011-rulings`（`@ f878034`，dirty=0，已合入）；远端已合入分支 12 条（`docs/2026-09-01-*`、`docs/2026-09-02-rm00011-architecture-rulings`、`docs/ecs-sample-r2-rulings`、`feat/r-00365-*`、`feat/r-00367-c2-binding`、`feat/r-00368-*`、`feat/r-00377-adr-accepted`、`feat/r-00384-r4-01-c1-c2-contract`、`rm00011/merge-wave0`） | worktree `LumioGameEngineArchitecture-parallel-eval`（`eval/2026-09-01-parallel-strategy @ 8e6408f`，未合入）；`origin/docs/2026-09-02-config-web-editor`（未合入）；本地分支 `worktree-agent-ad37c7ef876f6d507 @ 3287bba`（无上游，未核对内容） | 本地 main 已被另一会话 ff 到 `c0bcef2` |
| Runtime | `.claude/worktrees/exciting-chaplygin-77e103`（`ef822a7`）、`priceless-meitner-bb6110`（`6a2ab80`）dirty=0 已合入；远端已合入分支 16 条（`feat/r-00139…r-00371-*`、`docs/ecs-username-sample-r2`） | `.claude/worktrees/distracted-moser-154e2f`（`8afac31` = `docs/spec-baseline-v14`，未合入）；`origin/ci/supply-chain-gates-admission-path`、`origin/docs/spec-baseline-v14`、`origin/feat/dependency-policy-forbidden-packages`（未合入） | `feat/r-00385-r4-05-single-world-r2` 远端已不存在 |
| NativeCore | 三个 `.claude/worktrees/*`（`e192459` / `c180bdd`，dirty=0，已合入） | 主 worktree 有 2 个未跟踪 `.DS_Store`（不要顺手删目录）；`origin/feat/r-00386-r4-07-timer-ffi-delete` 经 PR #7 squash 合入，确认后再删 | |
| Server | `.claude/worktrees/epic-kalam-a04752`（`688be03`）、`friendly-moser-be15c3`（`37d4af4`）dirty=0 已合入；远端已合入分支 13 条（`feat/r-00277…r-00377-*`） | **`origin/feat/r-00388-r4-02-self-drive`（PR #33 open）**；`origin/feat/r-00346-admission`（未合入，需核） | |
| Client | 三个 `.claude/worktrees/*`（dirty=0，已合入）；`origin/feat/r-00349-replica-chat`、`origin/feat/r-00375-replica-snapshot` | 主 worktree 停在 `feat/t-00003-wss-transport-adapter`（本地分支 `cb3cd65` 无上游）——切回 main 前确认该分支去向 | |
| Game | 三个 `.claude/worktrees/*`（dirty=0，已合入）；`origin/feat/r-00348-chat-component`、`feat/r-00373-chatcomponent-runtime`、`feat/r-00376-eleven-scenarios` | `origin/claude/confident-dhawan-567439`（未合入） | |

## 10. 最终处置建议

- **整体：保持 blocked。** 不放行 r4，不进入 R4-09 / R4-10。
- **退回 R4-02（R-00388）**：见 §7 清单。
- **退回 R4-04（R-00389）修 P1-5**：会话层把 Runtime `InputCommandMessage` 包成 C-1 `InputCommand` 信封（`messageType` / `commands[{mappingId, payload hex, payloadSha256}]`），并补一条把出站字节喂给 C-1 解析器的测试；Workflow 状态是否回退由 Owner 决定（本报告不流转）。
- **升级 Owner 裁决两条契约缺口**：P1-1（声明表重嵌 vs 生成范围收窄；顺带处理生成器硬编码四行）、P1-2（`entity.identity` / `field.write` / `ConnectionSuperseded` 128 位编码 + 创建记录携带 Sync 字段值 + 字段变化包 + 欢迎消息的 wire 形态）。裁决后各补一张契约卡，R4-05 / R4-04 / R4-02 按卡跟进。
- **R4-01 / R4-05 保持 Workflow done 不动**（本报告不流转），但两卡的 done 不代表 ADR-056 Fixture 2 / ADR-058 Fixture 3、5、6 已落地；建议在 R-00345 变更控制评论登记。
- **ADR-057 / ADR-058 保持 Draft。** ADR-058 转 Accepted 的条件（R4-05 / R4-04 合入并由独立深审复核七项 Fixture）只满足前半；七项中通过 2、部分 5。ADR-057 转 Accepted 的条件（r4 全部 P1 关闭并逐项复核）不满足。
- **本报告不是 R4-10 交付物**：R4-10 前置 R4-09 未开始；本报告是对 r4 当前状态的整体复核，可作为 R4-10 的基线。

## 附录 A：本机（macOS）复跑记录

只记实际执行过的命令；每条给 exit 与关键输出。快照根 = 会话 scratchpad `snap/<仓>`。

| # | 仓 / 快照 | 命令 | 结果 |
|---|---|---|---|
| A1 | Arch `c0bcef2` | `node eng/verify-wire.mjs` | 41 pass / 0 fail（含 3 条 R4-01 新用例） |
| A2 | Arch | `node .spec/tools/spec-lint.mjs` | `spec-lint: OK` |
| A3 | Arch | `node eng/generate-abi.mjs` 后 `diff` `abi_generated.rs` / `AbiConstants.g.cs` | 零 diff，`DEFINITION_SHA256=ee2f6c6d…` |
| A4 | Arch | `bash eng/dev-run.sh`（`LumioServerRoot` 指向 Server main / r4-02 快照，各两次） | 四次 exit 126 `dev-build.sh: Permission denied`（仓内 mode 100644）；`chmod +x` 后 exit 1（`find …/../LumioVoxelEngine: No such file`）。**收口门槛在 macOS 不可用**（P2-2） |
| A5 | NativeCore `70b9834` | `cargo test -p lumio-timer --locked` | exit 0，全绿 |
| A6 | Server r4-02 `f8aef77` | `cargo test --locked --workspace --no-fail-fast` | exit 101；host-runtime 13 ✓、process lib 95 ✓、architecture 32 ✓、host 19 ✓、wire 10 ✓、testkit 10 ✓；`entity_chat_acceptance` BLOCKED（`LUMIO_GAME_ROOT`）；xtask `policy::tests::current_tree_satisfies_policy` FAILED |
| A7 | Server r4-02 | `cargo fmt --all -- --check` | exit 0 |
| A8 | Server r4-02 | `cargo clippy -p lumio-server-process --all-targets --locked -- -D warnings` | exit 101，`native_timer.rs:368` `needless_return` |
| A9 | Server r4-02 | `LUMIO_GAME_ROOT=<Game 快照> cargo test -p lumio-server-process --test entity_chat_acceptance --locked` | FAILED：`BLOCKED: account-server dll not found` |
| A10 | Server r4-02 | `dotnet build account-server/src/Lumio.Server.Account.App` / `entity-chat-host/src/Lumio.Server.EntityChat.HostEntry` 后 `./target/debug/lumio-entity-chat-replay --out … --log-dir …` | 两个 build exit 0；replay exit 2 `BLOCKED: hostfxr missing (set LUMIO_HOSTFXR or DOTNET_ROOT)` |
| A11 | Server main `4c7688b` | `cargo test --locked --workspace --no-fail-fast` | exit 101：`native_timer.rs:10` / `:87` dead_code（`-Dwarnings`），`lumio-host-runtime` 编译失败（P2-1） |
| A12 | Runtime `010ae46` | `dotnet build` ×4 测试工程；`dotnet exec` ×4；`dotnet test` Ecs.Tests；重建 username 两个 csproj 后 `diff -r` | Ecs 9/9、Replication 181/181、Username 3/3、GenDeclarations 3/3；`dotnet test` 9/9 exit 0；生成物零 diff（仅新增被 `.gitignore` 忽略的 `gen.hash`） |
| A13 | Client `f06d5e6` | `LumioRuntimeRoot=<Runtime 快照> dotnet test LumioClient.slnx` | exit 0：Connection 70、Session 35、Replica 40（4 skip）、Architecture 38、Bot 12、Integration 12、Prediction 14、Input 8、Handshake 5、Persistence 2、Hello 24 skip |
| A14 | Arch + Server r4-02 | `chmod +x` 后 `NATIVE_CORE_ROOT=<NativeCore 快照> VOXEL_ROOT=<本机 LumioVoxelEngine> bash eng/dev-build.sh`；`cargo build --manifest-path engine/native/Cargo.toml -p lumio-engine-native`；`DOTNET_ROOT=/usr/local/Cellar/dotnet/10.0.400/libexec` 重跑 replay | dev-build exit 101、native build exit 101（`failed to read …/snap/LumioVoxelEngine/crates/lumio-voxel-world/Cargo.toml`：`engine/native` workspace 以相对路径依赖同级 LumioVoxelEngine，快照布局不可构建）；replay exit 2 `BLOCKED: LUMIO_ENGINE_NATIVE missing`。**11 场景在本机无法复核**；按代码，即便原生库在，非 Windows 也会在 `sdk_loader.rs:555-559` / `native_timer.rs:365-369` BLOCKED |
| A15 | Game `e7afb5b` | `node --test integration/entity-chat/verify-evidence.mjs integration/entity-chat/web/chat-window.test.mjs`；`node verify-evidence.mjs --dir fixtures/oracle-min`；`dotnet test LumioGame.sln`；`dotnet exec …ServerGameplay.Tests.dll` | 15/15 pass；fixture `--dir` exit 0（非收口日志）；`dotnet test` MTP Zero tests exit 5；`dotnet exec` 23/23 |
| A16 | 六仓 | 禁词 grep：`DOTNET_STARTUP_HOOKS` / `account_sessions` / `ChatIngressWorld` / `_values` / `_eventsByRoomTick` / `_displayed` / `_liveConnectionByAccount` / `ReplicaWorld` / `AttributeDeclarationTable` / `ChatRoomWorld` / `advance_ms` / `verify_rust_evidence` / `LumioGameEngineArchitecture` / `bot_startup_hook` / `SeedAttributes` | 生产源命中：Server main `host.rs`（`account_sessions`）、Server main `clock.rs` / `timer.rs`（`advance_ms`）、Client `ReplicaWorld.cs`（门面，非属性袋）、Server `mvp-host/**`（冻结对照，`ChatRoomWorld`）、旧仓名见 P2-9；其余只在测试断言 / 文档 |

## 附录 B：2026-09-04 Owner 讨论裁决（只记录，未写 ADR、未拆卡、未流转）

逐条对话裁决，来源为本报告 §3 / §4 的设计层偏离。后续若立 ADR-059 与契约卡，以此为底稿。

| # | 问题 | Owner 裁决 |
|---|---|---|
| 1 | 网线上跑的是旧 C-1「普查 + chat.event」，World Manager 的包（欢迎 / 创建带字段 / 字段变化 / 销毁 / ClientRpc）从未上网线，两套复制模型并存 | **现在升级 C-1**：C-1 成为 Manager 包的网线形态，chat.event 是 ClientRpc 的一种；编解码只在 Runtime 一份；宿主与客户端只转发字节。先立契约卡，R4-02 / R4-04 退回重接，R4-09 顺延 |
| 2 | 投影无观察者概念：后连客户端收不到已有实体，恢复实体永不下发 | 客户端连入（含重连 = 重新登录）先收 AOI 范围内实体的创建记录把周围补全，之后只收增量；后续可按距离 / 权重排序先创建；本切片 AOI = 全房间。**细节（重开讨论后裁定）**：(a) 存储布局按 ADR-058 §9 做到底，组件保持 class：生成模板类 + 实体与组件一起分配入池复用、`Get<T>()` 走生成定位表零查找、`Sync<T>` 值存结构体内不再另起堆对象、NetEntityId → 实体走密集数组、测试探针搬出生产实体（Owner 要求内存紧凑、一开始就对性能友好）；(b) 首包规则写成分批：每拍至多 N 条创建记录，WorldEntity 永远第一，其余按优先级（本切片 = 创建顺序，以后 = AOI 距离 / 权重），本切片 N 不限；(c) 引擎提供服务器侧 `ObserverComponent`（Connected / ConnectionGeneration / DisconnectedAtTick / ProjectedTick，不存档），游戏在可被绑定的 EntityType 上 `[Has]`，Manager 按类型零查找读，修改 ADR-058 §14 措辞，`IdentityComponent` 只剩 Name + AccountId |
| 3 | 声明表四行假字段（`EntityIdentity.entityType / claimedMark / unmappedMark`、`ChatComponent.lastMessagePersistOnly`）由生成器硬编码；契约副本 6 行 vs Runtime 11 行 | **删掉**假字段；`entityType` 作为唯一明确标注的派生项由 `TypeOf` 回答；契约副本原样嵌 Runtime 生成物；S5「凭证才能看」用例标 deferred |
| 4 | 准入 / 过期 / 生成在适配层自己 `Tick()`，根子是 C-2 admit 同步回 id 与「提交相发号」冲突 | **admit 不再同步返回 netEntityId**，只回受理 / 拒绝；id 下一拍提交相发号，经欢迎消息送达；C-2 admit 结果形状相应改 |
| 5 | 信封编解码三份 + Client 出站无信封 | 并入第 1 条 |
| 6 | 旧 `EcsWorld` / `EcsModule.CreateWorld` / 旧快照管线仍在生产源，`command` 模块引用 | **现在删**；结构断言扩到全部 `modules/*/src`；command 以后按 `WorldManager.Commands` 重做 |

按设计原文默认处理、Owner 未反对的下游项：客户端聊天窗口挪到 UI 层；Bot.Host 不再自读根表槽位，由 NativeLoader 提供 timer 包装；S10 标 deferred；客户端 Manager 改主线程每帧 Tick；Server ADR 0008 补修订记录；lessons 记 ADR-057 第 10 条。

Owner 选择：**先停在这里**，只保留本记录。

**Owner 补充原则（2026-09-04，对六条全部适用）**：这是一次代码框架清理，必须彻底修复、彻底重构，不允许兼容层，不保留任何历史过程或过渡性决策；一切以设计原文为准。据此，下列在 review 中看到的「兼容 / 过渡」写法都在清理范围内，不再作为可选项：Client `ReplicaNetIds` 的 u64 兜底与本地合成欢迎消息；Runtime `EntityBindingQuery` 的 `TryParseLoose`、`lastMessagePersistOnly` 别名、`claimedMark` 常量；`World` 的反射兜底读字段；HostEntry 自编 FullSnapshot 与 `list_bindings` 入口；Server `host.rs` 手写信封解析；ADR 里的「两套模型并存过渡态」措辞。待 Owner 后续确认是否同样适用：Server 仓冻结对照的 C# `mvp-host/`、Runtime 旧 replication history / baseline 模块。

**第 2 条重开后的细节裁决（同日下午）**

| 细节 | 裁决 |
|---|---|
| a. 客户端销毁与墓碑 | 墓碑按计数器推导、不存集合（墓碑 = 计数器 < 下一个号 且不在活体；快照只存下一个号；客户端按见过的最大号同理）。销毁记录真正进包（现状打包代码 `Destroys` 写死空数组），客户端跑 OnDisable / OnDestroy 并回池 |
| b. 五分钟窗口内重连 | 只做「断线重连 = 重新模拟完整登录进入游戏（退出 → 登录 → 进入）」的粗暴做法：全量重发，不补断线期间增量，服务器不留按观察者历史；UI 聊天窗口默认清空 |
| c. 字段可见性裁剪 | 服务器打包时按 Scope 裁，创建记录与字段变化同一规则：Room 全发、Aoi 按视野（本切片 = 全房间）、Owner 只发绑定者、Claim 按凭证；接受的 Owner 写不回声，被拒只推 Correction 给本人；客户端不再判。现状打包对 Scope 一视同仁 |
| d. AOI 接口预留 | 出视野 = 销毁记录 + OnLeaveAOI（客户端回池），进视野 = 创建记录（带当前值）+ OnEnterAOI；服务器每拍按观察者算可见集，本切片 = 全房间；客户端只有「有 / 没有」两态 |
| Claim 语义 | Owner 澄清 Claim = 服务器点名发凭证的人才能看（好友 / 队友类）；现实现按「连接 × 字段名」发凭证，粒度粗于设计。裁决：留到真有 Claim 字段时再定；本轮只删 `claimedMark` 与 `GrantClaim`，`Scope.Claim` 保留枚举不定义语义 |
