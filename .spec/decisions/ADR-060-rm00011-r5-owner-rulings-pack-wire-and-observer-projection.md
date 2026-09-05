# ADR-060：RM-00011 r5 Owner 裁决——World Manager 包上网线、按观察者投影、模板内联存储与不留兼容的框架清理

状态：Draft（2026-09-04，Owner 逐条裁决；依据 `reviews/2026-09-04-rm-00011-r4-overall-review.md` §3「样板对照」与附录 B 讨论记录）
取代：无（填实 ADR-058 §3 / §9 / §11 / §14 / §16 / §18 与 ADR-057 第 5 / 7 条在 r4 交付上未成立的部分；ADR-058 Accepted 前本 ADR 与之一并复核）
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioGameRuntime`（World Manager / 生成器 / wire codec 唯一实现）、`LumioServer` / `LumioClient` / `LumioGame`（只转发字节的消费方）

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。**
- 沿用 ADR-058：**AI Agent 友好**——同一件事只在一处维护，每件事只有一种写法。
- 本 ADR 新增：**彻底清理，不留兼容。** 引擎底层（ECS / Runtime / 契约 / 宿主接入）的整改是代码框架清理：必须彻底修复、彻底重构；不允许兼容层（u64 兜底、别名、常量答案、第二份编解码、反射兜底）；不保留历史过程或过渡性决策（ADR 里不写「两套模型并存过渡态」）；一切以设计原文为准，设计原文不合适就改设计原文，不在代码里绕。

## 背景

r4 整体复核（`reviews/2026-09-04-rm-00011-r4-overall-review.md`）以读代码为主，对照 `knowledge/features/ecs.md` §4.5 与 Runtime `modules/ecs/samples/username/` 样板逐项核对。结论：**Runtime 的 ECS 框架本体按样板做到了**（`WorldManager` / `World` / `Sync<T>` / 生成三件 / `[ServerRpc]`·`[ClientRpc]` / 128 位发号 / `CreateFromSnapshot` 进程内成立），但网线两端没有按样板接上，且若干处偏离 ADR-058 字面：

1. Runtime `WorldManager` 产出的「欢迎 / 创建（带字段值）/ 字段变化 / 销毁 / ClientRpc」包从未上网线。宿主只抽 ClientRpc 重编成旧 C-1 `chat.event`，自编 `entity.identity` 普查 FullSnapshot（只有 counter + entityType）；客户端反向拼零字段创建记录、本地合成欢迎消息（`InstanceId = 0`）。两套复制模型并存。
2. 投影无观察者概念：全局「已发给客户端」集合，后连客户端收不到已有实体；`Hydrated` 实体永不下发；销毁记录写死空数组从未进包；`Scope` 未裁剪。
3. 声明表四行（`EntityIdentity.entityType / claimedMark / unmappedMark`、`ChatComponent.lastMessagePersistOnly`）由生成器硬编码，不来自任何组件类；`claimedMark` 答常量；Runtime 生成表（11 行）与 C-1 / C-2 内嵌表（6 行）sha 分叉。
4. 准入 / 过期 / 生成在适配层自己 `Tick()` 以同步返回 id，根子是 C-2 `admit` 同步回 `netEntityId` 与 ADR-058 §16「提交相发号」冲突；每次登录推进一拍。
5. C-1 信封编解码三份（Runtime `ChatEnvelope`、Server `host.rs` 手写、Client `GameplayCodec`），Client 出站 `chat.input` 未包信封，宿主解析失败静默丢弃。
6. 旧 `EcsWorld` / `EcsModule.CreateWorld` / `EcsPersistSnapshotPipeline` 仍在生产源，`command` 模块引用。
7. ADR-058 §9 模板内联存储未实现：字典 + `Component[]` + 每 `Sync<T>` 字段一个堆槽 + 生产实体上的测试探针 `List<string>`；`Get<T>()` 顺序扫描。

## 决策（Owner 逐条裁决，2026-09-04）

1. **C-1 升级为 World Manager 包的网线形态。** 消息集收敛为：`Welcome`（世界实例 ID + 自己的 NetEntityId + 连接代数）、`WorldChange`（tick + 创建记录[类型 + 128 位 id + 可见字段当前值] + 字段变化[id + 组件 + 字段 + 值 + 原因 sync|correction] + 销毁[id] + ClientRpc[目标 + 组件 + 方法 + 参数 + messageId + roomSequence + sender + appliedTick]，同一条有序流、创建优先）、`InputCommand`（`chat.input` / `field.write` / 生成的 ServerRpc 种类，信封含 mappingId + LumioBinV1 payload hex + sha）、`ConnectionSuperseded`、`Error`。`FullSnapshot` / `Delta` / `entity.identity` / `chat.event` 映射删除：聊天事件就是 `ChatComponent.OnChatMessage` 这条 ClientRpc 的记录，不再单独造格式。所有 `netEntityId` 一律 128 位（instance u64 + counter u64，wire 32-hex 或两段 u64）。
2. **编解码只在 Runtime 一份。** Runtime 提供 C-1 信封 codec（服务器编包 / 解 InputCommand，客户端解包 / 编 InputCommand），宿主与客户端会话层只搬字节，不得手写第二份解析或编码。
3. **投影按观察者，连上先全量再增量。** 每拍为每个在线观察者打一个包；观察者绑定（首次准入、重连、顶号后 rebind）那一拍先收 `Welcome`，再收它可见的全部活体实体的创建记录（WorldEntity 永远第一），之后只收增量。首包规则写成分批：每拍至多 N 条创建记录，其余按优先级（本切片 = 创建顺序；以后 = AOI 距离 / 权重）；本切片 N 不限。恢复出的实体（`Hydrated`）与新建实体在投影上无区别。
4. **引擎提供服务器侧 `ObserverComponent`。** 字段：`Connected`、`ConnectionGeneration`、`DisconnectedAtTick`、`ProjectedTick`（追到哪一拍）；不存档，重启即离线。游戏在可被连接绑定的 EntityType（Player / Bot）上 `[Has(typeof(ObserverComponent))]`，与 `WorldEntity` 挂 `WorldSaveComponent` 同一写法；准入时实体类型没挂它 = 结构性失败。Manager 按类型零查找读，不再按字段名字符串读游戏组件。`IdentityComponent` 只剩 `Name` + `AccountId`。以后 AOI 的视野 / 权重也放这里。包按观察者 NetEntityId 寻址，宿主用自己的会话表换成连接；**Manager 不再持任何连接字符串表**。
5. **字段可见性在服务器打包时按 `Scope` 裁，创建记录与字段变化同一规则。** `Room` 全发；`Aoi` 按视野（本切片 = 全房间）；`Owner` 只发绑定者；`Claim` 按凭证（写法见第 12 条）。接受的 Owner 写不回声给写者；被拒只推 `Correction` 给本人。客户端本地读不再判定。
6. **AOI 接口预留：出视野即删。** 服务器每拍按观察者算可见集（本切片 = 全房间）；出视野 = 销毁记录 + `OnLeaveAOI`，客户端回池；进视野 = 创建记录（带当前值）+ `OnEnterAOI`。客户端只有「有 / 没有」两态，不做「保留但停更」。
7. **墓碑按计数器推导，不存集合。** 墓碑 = 计数器 < 下一个号 且不在活体；快照只存「下一个号」；客户端按见过的最大号同理。销毁记录真正进包，客户端跑 `OnDisable` / `OnDestroy` 并回池。
8. **断线重连 = 重新模拟完整登录进入游戏。** 退出 → 登录 → 进入；`ProjectedTick` 归零，全量重发，不补断线期间增量；服务器不留任何按观察者的历史；UI 聊天窗口默认清空（UI 层事）。
9. **admit 不再同步返回 `netEntityId`。** 准入只回「已受理 / 拒绝」；id 在下一拍提交相发号，经 `Welcome` 送达客户端。准入 / 过期 / 生成代码不再自己 `Tick()`；世界只在宿主的内核节拍处推进。C-2 `admit` 结果形状相应修改。
10. **声明表只从组件类生成。** 删除生成器硬编码的四行与 `claimedMark` 常量、`lastMessagePersistOnly` 别名；`entityType`（player | bot）作为唯一明确标注的派生项由 `TypeOf` 回答，在 C-2 查询面单独列出；契约副本原样嵌 Runtime 生成物，Arch 在 Runtime 重生成后同步；S5「凭证才能看」用例标 deferred。
11. **模板内联存储按 ADR-058 §9 做到底，组件保持 class。** ① 生成器为每个 EntityType 产内部模板类，实体与组件一起分配、整块入池复用；② `Get<T>()` 走生成定位表零查找；③ `Sync<T>` 的值存结构体内（值 + 所属组件引用 + 字段序号），不再另起堆槽；④ NetEntityId → 实体走密集数组（计数器即下标，墓碑为空位）；⑤ 测试探针（`Lifecycle`）搬出生产实体。不换 struct 组件。
12. **Claim 的写法定稿，语义与实现随真字段。** `Scope.Claim` 字段声明时指定凭证名单字段：`Sync<string> RealName = new(Scope.Claim, claimBy: nameof(Friends))`，名单是同一实体上的普通 Sync 字段（如 `SyncList<NetEntityId> Friends = new(Scope.Owner)`）；打包时只发给名单里的观察者；不另建凭证表。删除现有按「连接 × 字段名」发凭证的 `GrantClaim`。样板示例（`ecs.md` §4.5 与 Runtime `samples/username`）以此写法为准；本切片没有 Claim 验收场景，规则随 Runtime 卡实现，不留假字段。
13. **现在删旧世界。** `EcsWorld`、`EcsModule.CreateWorld`、`EcsPersistSnapshotPipeline` 及 `command` 模块中依赖它们的代码删除；`command` 以后按 `WorldManager.Commands` 重做；结构断言扩到全部 `modules/*/src`。`EntityBindingQuery.Create()` 写死实例 ID的兜底构造、`World` 的反射兜底读字段、`TryParseLoose` 一并删除。
14. **宿主与客户端只搬字节。** Server `HostEntry` 删自编 FullSnapshot / `entity.identity` 编码与 `list_bindings` 入口；`host.rs` 删手写信封解析；Client 删 `ReplicaNetIds` u64 兜底、本地合成欢迎消息、`GameplayCodec` / `LiteJsonParser` 第二份解析；客户端 Manager 改为主线程每帧 `Tick()` 一次，不再每收一条消息 `Tick()`；聊天窗口挪到 UI 层；Bot.Host 经 NativeLoader 的 timer 包装取根表槽，不自读槽位。
15. **下游默认项。** S10 标 deferred（ADR-058 §11）；Server ADR 0008 追加修订记录指向 ADR-057；`knowledge/lessons.md` 记 ADR-057 第 10 条。

## 替代方案

- **保留旧 C-1 到切片验收后、把两套模型写成 ADR 过渡态**：被 Owner 否决——那样 11 场景验收的仍不是设计里的架构，与 r3 被退回的原因相同；且违反「不留兼容」。
- **只补齐欢迎消息与带字段的创建记录，字段变化 / 销毁流推后**：被否——仍是两套编码器。
- **观察者状态归宿主**（Manager 全局视角，宿主记谁看见过什么并向 Manager 要全量）：被否——宿主要读世界内容，与「只转发」相悖，AOI 来时还要再搬。
- **观察者状态放 WorldEntity 的组件**（按观察者索引的表）：被否——每观察者状态集中在单例上，AOI 以后也得进这张表；先例是连接态字段放在被绑定实体自己身上。
- **观察者状态为 Manager 私有字典**：被否——不是组件字段，与「每实体状态只能在组件里」相抵。
- **一拍全量、分批以后再说**：被否——以后加分批要改包语义（客户端要能处理「没收全」）；规则写成分批、本切片 N 不限，不多任何东西。
- **补发断线期间增量**：被否——需要按观察者的事件历史缓冲，正是 ADR-058 §18 否掉的。
- **保留同步回 id、下单时就发号**：被否——与「提交相发号」原文不一致，要改 ADR-058 §16 措辞；欢迎消息本来就要走，改 admit 结果形状不多加东西。
- **四行假字段保留但标「探针」**：被否——声明表不再是「只从组件类生成」。
- **删假字段并给样板加一个真的凭证字段只为 S5 可验**：被否——为验收而加实体。
- **struct 组件、DOTS 式连续内存块**：被否——是换设计不是清理，样板写法、partial 钩子、RPC 方法、ADR-058 §7 §8 §9 §12 全部重议。
- **旧 EcsWorld 先留、只加禁新引用断言**：被否——「只有一条 CreateWorld 路径」字面继续不成立。
- **出视野保留实体但停更、再进补差**：被否——客户端多第三种状态与补差机制。
- **墓碑保留集合**：被否——计数器单调、id 永不复用，集合可推导，且只增不减。
- **全发、客户端自己过滤可见性**：被否——与 ADR-057 第 7 条相悖，`Owner` 字段会泄露给全房间。

## 接口 / Schema

- **C-1″ `engine/wire/gameplay-command-envelope-v1.json`**：消息集 = `Welcome` / `WorldChange` / `InputCommand` / `ConnectionSuperseded` / `Error`；删除 `FullSnapshot` / `Delta` 消息与 `entity.identity` / `chat.event` / `chat.component` 映射；`InputCommand.commands[].mappingId` ∈ {`chat.input`, `field.write`, 生成的 ServerRpc 种类}，payload 仍为 LumioBinV1 hex + sha256；`WorldChange` 各记录的 `netEntityId` 为 128 位（两段 u64 LE，JSON 上 32-hex）；`roomSequence` = 世界内严格递增序号，盖在 ClientRpc 记录上；transport（websocket utf8-json-text-frame，`maxFrameBytes`）不变；`limits` 增 `createsPerPack`（本切片 = 0 表示不限）。具体字段名、`fieldOrder`、`payloadSha256` 由契约卡填实并经 `eng/verify-wire.mjs` 编码器重算。
- **C-2″ `engine/wire/entity-binding-and-query-v1.json`**：`binding.operations.admit.result` = `accepted` | 拒绝码（`invalid_binding_shape` / `account_already_online` / `bot_namespace_admission_forbidden` 等），不再含 `netEntityId`；`connectionGeneration` 与 `netEntityId` 由 `Welcome` 交付；删除 `listBindings`；`attributeDeclarations.table` = Runtime `modules/ecs/generated/attribute-declarations.json` 逐字节副本（sha 由 verify-wire 重算），`EntityIdentity.*` 与 `lastMessagePersistOnly` 行删除；`attributeQuery` 增「派生项」小节：`entityType` 由 `TypeOf` 回答；`tombstoned` 结局定义改为「计数器 < 下一个号 且不在活体」；`claim` 小节：凭证 = 目标实体上 `claimBy` 指名的名单字段。
- **Runtime 公开 API（R5 Runtime 卡落地）**：`WorldManager.Create(registry, instanceId?)` / `CreateFromSnapshot(snapshot)` / `Start(ownerThread)` / `Enqueue(WorldMessage)` / `Tick()` / `DrainOutbox()`（每观察者一包，按观察者 NetEntityId 寻址）/ `CaptureSnapshot()`；`Bind(observerNetEntityId)` / `Unbind(...)` 只作用于 `ObserverComponent`；`Commands.Create<T>()` 下单、提交相发号；`World.TypeOf(id)`；`ObserverComponent`（引擎组件，服务器侧，四字段）；`Sync<T>(Scope, Authority = Server, Notify = Remote, string? claimBy = null)`，`SyncList<T>` / `SyncDict<K,V>` 同；Runtime wire codec：`WireCodec.EncodePack(pack)` / `DecodePack(bytes)` / `EncodeInput(...)` / `DecodeInput(bytes)`（名称由 Runtime 卡定，两端同一程序集）。
- **生成物**：模板类（每 EntityType 一个，含定位表）、注册表、同步表、`attribute-declarations.json`；`ObserverComponent` 缺失校验；`claimBy` 指向的字段必须存在于同一组件且为 Sync 容器，否则生成失败。

## 失败语义

- 宿主或客户端源码出现第二份 C-1 编解码（手写 JSON 解析 / 自编 FullSnapshot / u64 兜底）——结构断言失败，收口审查退回。
- 生产源出现 `EcsWorld` / `EcsModule.CreateWorld` / 反射读组件字段 / 适配层内 `Tick()` / Manager 内连接字符串表——结构断言失败。
- 生成器发现 `Scope.Claim` 字段无 `claimBy` 或指向不存在 / 非容器字段——生成失败。
- 可被绑定的 EntityType 未 `[Has(typeof(ObserverComponent))]`——准入结构性失败，不得运行时静默不发包。
- 创建记录出现 `Scope.Owner` 字段发给非绑定者、或 `Scope.Aoi` 字段发给视野外观察者——打包断言失败。
- C-1 / C-2 内嵌声明表 sha 与 Runtime 生成物不一致——`verify-wire` 失败。

## 兼容影响

- ADR-058：§3（创建记录按观察者；首包分批规则）、§9（按第 11 条落实）、§14（连接态字段移入引擎 `ObserverComponent`，`IdentityComponent` 只剩 `Name` + `AccountId`）、§16（墓碑推导；快照不存墓碑集合）、§17（客户端 `Welcome` 来自服务器，不本地合成）、§18（事件 = ClientRpc 记录进 `WorldChange`）由本 ADR 修订；ADR-058 正文不改写，追加「修订记录（2026-09-04，ADR-060）」。
- ADR-057 第 5 条（admit 结局）与第 7 条（读权限同步时裁剪）按第 9 / 5 条填实；ADR-053 / ADR-049 各追加修订记录段。
- `knowledge/features/ecs.md`：M1a ④ ⑥、M3 ⑦、M6「不干什么」、M9 ①、§4.5 样板（Claim 写法、ObserverComponent 的 `[Has]`）按本 ADR 改写；`ecs-entity-chat.md` §4 重连、§6 S5 / S10 deferred。
- 六仓：Runtime `modules/ecs` / `modules/replication` / `modules/command` / `tools/gen-declarations`；Server `entity-chat-host` / `modules/process/src/entity_chat`；Client `modules/replica` / `modules/session` / `modules/bot`；Game `integration/entity-chat`（oracle 读新日志字段）；Arch `engine/wire/*.json`、`eng/verify-wire.mjs`、`engine/managed/Lumio.Engine.NativeLoader`（timer 包装）。
- Workflow：R4-02 / R4-04 退回按本 ADR 重接；R4-01 / R4-05 的 done 不代表 ADR-058 Fixture 已落地；R4-08 / R4-09 / R4-10 顺延到 r5 之后。r5 卡见 `plans/2026-09-04-rm-00011-r5-cards.md`。

## 迁移方案

`plans/2026-09-04-rm-00011-r5-cards.md`：R5-01 契约与文档（Arch）→ R5-02 Runtime 框架清理 → R5-03 宿主与客户端接入（Server + Client + Game）→ R4-09 集成重跑 → R4-10 独立深审。串行，不并行：三张卡文件集互不重叠但语义上后者消费前者的产物。

## 验证 Fixture

沿用 ADR-056 六项、ADR-057 四项、ADR-058 七项原文，追加：

1. **包上网线**：真网线上客户端收到的第一帧是 `Welcome`，第二帧 `WorldChange` 的第一条创建记录是 WorldEntity，创建记录带 `IdentityComponent.name` 当前值；改名后其他客户端日志出现 `name <旧> -> <新> (Sync)`（真进程、真 socket，不是环回）。
2. **一份 codec**：六仓 grep 只有 Runtime 一处 C-1 编解码；Server / Client 生产源无 `"messageType"` 字符串拼装 / 解析。
3. **晚进者全量**：第 101 个连接的首包创建记录数 = 当时活体数（WorldEntity 第一）；恢复后新连接同样收到全量；断线重连收到全量且不收断线期间事件。
4. **可见性裁剪**：样板增 `Scope.Owner` 字段后，非绑定者的包里不出现该字段；接受的 Owner 写无回声；被拒仅本人收 `Correction`。
5. **销毁与墓碑**：到期实体的销毁记录出现在每个观察者的包里；快照无墓碑集合；查询已销毁 id 两端均答 `tombstoned`。
6. **登录不偷跑**：100 个 Bot 登录期间 `World.Tick` 增量 = 内核节拍数；两轮 `appliedTicks` 逐位一致与登录节奏无关。
7. **声明表**：Runtime 生成物与 C-1 / C-2 内嵌表 sha 一致；生成器源码无硬编码 attributeId；查询面无常量答案。
8. **紧凑存储**：建一个 PlayerEntity 的堆分配对象数 = 1 模板 + 组件数；销毁再建同类型零新分配；`Get<T>()` 无循环；实体查找无字典。
9. **旧世界清零**：全部 `modules/*/src` grep `EcsWorld` / `EcsModule.CreateWorld` / `EcsPersistSnapshotPipeline` 零命中。
10. **Manager 无连接表**：`WorldManager` 源码无以连接字符串为键的容器；包按观察者 NetEntityId 寻址。

## 修订记录（2026-09-05，ADR-063）

以下各条的措辞由 [ADR-063](ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) 修订，本文正文不改写：

- 第 5 / 12 条：可见性本身变化也是同步事件——`Claim` / `Owner` 名单新增观察者补发当前值、移除发失效记录；持久名单存 `AccountId`（ADR-063 第 8 条）。
- 第 7 条：墓碑推导只在服务器成立；客户端不按最大号推导，只有三态（有副本 / 未知 / 收到过已终结）；销毁记录带 `reason ∈ { left_aoi, terminated }`（ADR-063 第 2 条）。「快照只存『下一个号』」被取代：快照存发号器「已占到哪」，崩溃后从已占段之后继续（ADR-063 第 3 条）。
- 第 11 条 ①：「整块入池复用」保留，「整块内存拷贝」删除——批量创建 = 池化 + 生成的按类型克隆，验收看耗时 / 分配 / GC 三个数（ADR-063 第 9 条）。
- 第 11 条 ④：「NetEntityId → 实体走密集数组」由必须改为参考做法，查找结构归实现仓（ADR-063 第 3 条）。
- 「接口 / Schema」C-1″：追加 `WorldChange.destroys[].reason` 与包级 `appliedInputSequence`，随 R5-01 一并落（ADR-063 接口段）。
