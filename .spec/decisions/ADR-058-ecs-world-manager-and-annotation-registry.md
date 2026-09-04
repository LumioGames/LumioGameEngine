# ADR-058：ECS World Manager、Sync<T> 字段与标注生成桥

状态：Draft（2026-09-03，ECS 架构专题会话 Owner 逐条裁决；依据 `reviews/2026-09-03-rm-00011-r4-owner-discussion.md` §3 与 `reviews/2026-09-03-rm-00011-r3-owner-review.md` P1-7 / P1-9 / P2-6 / P2-12）
取代：无（填实 ADR-056 §1「单一 ECS」与 ADR-057 第 7 条「单一世界原则」的结构；ADR-053 的 AttributeId 查询面保留为适配层，不取代）
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioGameRuntime`（World Manager / ECS 核心 / 生成器唯一实现）、`LumioClient`（客户端 World 消费方）、`LumioGame`（玩法组件内容层）、`LumioServer`（宿主路由与会话表）

> **修订（2026-09-03 第二轮）**：Owner 复审样板示例后逐条裁决（流水见 `reviews/2026-09-03-ecs-sample-owner-rulings.md`），第 2 / 4 / 5 / 7 / 8 / 12 / 14 / 17 / 18 条与「接口 / Schema」「失败语义」「替代方案」「验证 Fixture」按裁决改写：删 `[ServerOnly]` / `[ClientOnly]`（文件后缀即归属）与 `IdentityComponent.Kind`；加 `world.TypeOf(id)`、EntityType 用 C# 继承（abstract class）、WorldEntity 由游戏声明（`World = true`）、按字段生成的变化钩子与 `Notify`、客户端同一 `Create` + 欢迎消息绑 Self、同进程双端 = 两个 Manager + 环回；客户端聊天窗口归 UI 层。

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。**
- 本 ADR 新增：**AI Agent 友好。** 同一件事只在一处维护；调用点显式可辨（看一行代码就知道它会不会上网）；生成物入库可读；每件事只有一种写法。多份声明、隐形生成成员、两种读法并存，都按此原则否决。

## 背景

RM-00011 r3 深审（`reviews/2026-09-03-rm-00011-r3-owner-review.md`）实测 Runtime `origin/main`（`ff70401a`）：

- 三个世界：`EntityBindingQuery.cs:73-84` 建 WorldId 1（权威，零实体）/ 2（副本），只当线程牌子（`IsOwnerThread:921`）；`ChatIngressWorld` WorldId 370（maxEntities=128）才有实体；三者无数据通道。
- 查询答假值：`QueryAttribute:450` 只读私有字典 `_values`，唯一写入点 `SeedAttributes` 灌常量（`"mark"` / `"persisted"` / 空串 / 0）。
- 标注不驱动注册：标注类 3 字段、`ChatIngressWorld.cs:267-280` 手写 2 字段、测试 4 字段；注册靠反射取 `_componentRegistrationCapability` 破门；`ComponentTypeRegistry` 字段只有 `PersistOnly` 一维；`EntityTypeDefinition` 无子实体 / 依赖 / 互斥 / 模板。
- ECS 创建 / 写字段 / 查询 API 全 internal，没有面向组件代码的写法；`CreateWorld(WorldId, EcsBudget)` 不传注册表；无 World Manager。
- NetEntityId 是 64 位计数器 `x32` 补零，由绑定查询发号；`RestoreIdentityTable` private，`IdentityTableSnapshot` 公开构造（宿主自铸旁路）。
- 持久化按注册期枚举筛字段、不看 `[Persist]`；恢复只重建 LocalEntityId、不重建网络映射；生产零调用。
- `_eventsByRoomTick` 无界事件历史；`ChatTypedMapping._displayed` 每连接窗口副本；`SetMessage` 直写旁路不产事件；`_liveConnectionByAccount` 独立在线表。
- Client `ReplicaWorld` 是字符串属性袋（Delta 不改属性、手写 6 行声明表）、零 Runtime 引用；Runtime 9 个生产项目均 `net10.0;netstandard2.1`，Client 生产库 `netstandard2.1`。

根因：「世界」没有 owner、没有定义，谁需要谁 new；「声明」没有被当作唯一真源。ADR-057 第 7 条已定单一世界原则并留三条已拍板项（WorldEntity 存档触发、同步时裁可见性、身份表入档），本 ADR 定结构。

## 决策（Owner 逐条裁决，2026-09-03）

1. **声明真源。** 组件类是唯一真源，注册表 / 同步表 / 契约声明表全由生成器产出，手写注册路径删除。**未标注的普通字段 = 什么都不做**（不存档、不同步、本端本地值），恢复时按声明默认值初始化；忘打 `[Persist]` 是使用者 bug，引擎不兜底。
2. **WorldEntity。** 由**游戏**在 `EntityTypes/WorldEntity.cs` 声明，与其他 EntityType 同一写法：`[EntityType(Mode.CS, World = true)] [Has(typeof(WorldSaveComponent))] public abstract class WorldEntity {}`；注册表里标 `World = true` 的类型必须恰好一个（缺或多 = 生成报错）。引擎只提供世界级组件（`WorldSaveComponent` 等：存档 / Dump / Tick 配置是它们的字段），游戏的世界级状态（对局阶段、比分）再加 `[Has]` 挂上。World Manager 建世界时按它建单例，普通 CS 实体（有 NetEntityId，字段按声明同步）；两端按组件类型 `Single<T>()` 取；客户端不自建，它是第一条创建记录。世界存在 = WorldEntity 存在。
3. **创建与亮相。** 双端 World Manager 都是「收消息 → 在自己提交相生效」的入口：服务器收 InputCommand，客户端收「世界变化」——创建实体（EntityType + NetEntityId + 全部可见字段当前值）、字段变化、销毁实体三种记录，**同一条有序流，创建优先**，按 Tick 成包一次性生效。客户端建实体：按同一 EntityType 模板建 → Awake（同一套代码）→ **PostAttribute**（框架把创建消息携带的服务器字段值写入）→ Start；Awake 完整结束后客户端字段已与服务器一致。
4. **一套源码编两份程序集，文件后缀与宏都要，不打归属标注。** 字段归属只由文件后缀决定：`.Server.cs` 的成员只进服务器程序集、`.Client.cs` 只进客户端、无后缀共享文件两端都编；生成器按 csproj 各跑一遍，据此产两端各自的注册表 / 同步表，客户端程序集物理上不含服务器字段。第二轮删去 `[ServerOnly]` / `[ClientOnly]`——生成器本来就看不见另一端的文件，标注是同一件事的第二份声明，多一处就多一处不一致。逻辑块与敏感信息用 `#if LUMIO_SERVER` / `#if LUMIO_CLIENT` 物理剔除，防逆向 / 剥离。`[Persist]` 哪端编译哪端存（客户端 World 不存世界档，客户端文件里的 `[Persist]` 不起作用）。
5. **文件布局。** 一个组件类型、按端拆 partial 文件，按组件聚合一个文件夹：`Components/<名>/X.cs`（共享：Sync 字段 + RPC 声明 + 共享逻辑）/ `X.Server.cs`（服务器私有字段 + ServerRpc 处理体）/ `X.Client.cs`（客户端本地字段 + 表现钩子）/ `X.g.cs`（生成物，入库不手改）；`EntityTypes/` 一份声明（含 WorldEntity）。一棵源码树、两个 csproj：`*.Server.csproj` 排除 `**/*.Client.cs` 定义 `LUMIO_SERVER`，Client 反之；生成器各跑一遍。lint：共享文件里只许 `Sync` / `SyncList` / `SyncDict` 字段、RPC 声明与共享逻辑，非 Sync 的状态字段必须在 `.Server.cs` / `.Client.cs`（放共享文件 = 另一端多一个永远是默认值的死字段，读到假值不报错）；每文件首行注释列兄弟文件。
6. **上行参照 Unity Netcode for GameObjects。** 同步字段 = NetworkVariable 语义：`Sync<T>(scope, authority)`，默认 `Authority.Server`；`Authority.Owner` 显式 opt-in，仅绑定到该实体的连接可写自己实体的该字段，改了 `.Value` 自动上行、不写消息代码。上行字段变更与 `[ServerRpc]` 方法调用都是 ADR-049 InputCommand 信封的种类，进服务器同一条有序输入流，ApplyInputs 相按发送者 NetEntityId 排序后应用；组件可选 `partial void OnClientWrite(in SyncWrite w, ref bool accept)` 校验钩子（`accept` 进来是 true，置 false = 拒；带返回值的 partial 在 C# 里必须有实现，做不到「不写就没有」，所以走 ref），被拒即权威纠正推回。服务器→客户端动作 = `[ClientRpc]`。无通用 SetField RPC。存档命令 = 对 WorldEntity 的 ServerRpc。
7. **脏记账 = `Sync<T>` 包装类型（struct）。** 声明 `Sync<T>(Scope, Authority = Authority.Server, Notify = Notify.Remote)`，写 `.Value`、读隐式转换；容器 `SyncList<T>` / `SyncDict<K,V>`。标注集修订为：`[EcsComponent]`、`[Persist]`；不再有 `[Replicate]` / `[Visibility]`（类型即声明），也不再有 `[ServerOnly]` / `[ClientOnly]`（文件后缀即归属，第 4 条）。生成器只产表、内部模板类、**可选 partial 钩子声明**（入库可见、不写不生效）与 **RPC 发送桩**（`[ServerRpc]` 在客户端、`[ClientRpc]` 在服务器都是没有用户实现的 partial 声明，桩体由生成器产在该端），不在用户类型上生成任何隐形成员或脏位属性。EntityType 声明类无成员，按类型下单用泛型 `Commands.Create<T>()`，没有 `.Type`。下行写入走 Sync 内部接口，不记脏不回声。
   **变化钩子（第二轮）**：生成器为每个 Sync 字段在 `X.g.cs` 里产一对可选 partial 方法——标量 `partial void OnXChanging(T old, T @new, ChangeReason reason)`（内存值改之前，只通知不否决；否决已有 `OnClientWrite` 一条路）/ `OnXChanged(...)`（改之后）；`SyncList<T>` 为 `OnXChanged(in ListChange<T> c)`（`c.Op` Set / Insert / Remove / Clear、`c.Index`、`c.Old`、`c.New`、`c.Reason`），`SyncDict<K,V>` 为 `OnXChanged(in DictChange<K,V> c)`（`c.Op` Set / Remove / Clear、`c.Key`…）；容器按条目报不拷整个容器；嵌套 struct 整体当一个值。**默认 `Notify.Remote`：只收对端来的变化**，`reason` = `Sync`（同步到达）/ `Correction`（被拒后权威纠正）；本端写 `.Value` 不触发——服务器改服务器、客户端改客户端都不收，这是对的默认。`Notify.All` 时本端写也触发，`reason` = `Local`。创建 / 恢复时的首次填值不触发（归 PostAttribute / OnHydrate）。接收批语义：同一 Tick 包先全部写入、再统一触发 Changed，同帧写的多个字段在任一钩子里都已是新值。多字段 WhenAll 式组合器不进引擎（后置，见替代方案与 ecs.md §6）。
8. **读。** 玩法代码只有类型化读：`Get<T>()` 读自己（组件里取同一实体的另一个组件也是它）、`Get<T>(id)` 读别人、`world.Each<T>()` 系统遍历——一种写法，不用知道对方实体类型；要知道类型时 `world.TypeOf(id)` 返回类型句柄，`.Is<PlayerEntity>()` 对子类型也为 true（类型不编进 NetEntityId：世界本来就存着每个实体的模板，一次数组定位）。C-2 AttributeId 查询面保留但退成生成的薄适配层（字符串名 → 同一世界同一字段），供宿主探针 / 验收 / 工具，无自有存储；四种结局由世界状态派生。
9. **实体结构 = 组件式写法 + 生成的实体模板内联存储。** 生成器按 EntityType 产**内部**模板类：实体对象 + 其组件对象相邻分配、整块入池，`Get<T>()` 走生成定位表无字典；模板类玩法不引用。`Sync<T>` 必须是 struct。
10. **恢复 = World Manager 从快照建新世界** `CreateFromSnapshot`，与 `Create` 同一条路只是来源不同。快照 = 全部实体 `[Persist]` 字段 + NetEntityId 身份表与发号器状态 + WorldEntity 自身组件 + Tick 号。恢复只跑 OnHydrate；未标 `[Persist]` 取声明默认值。
11. **Room。** 一个进程一个 World Manager 一个 GameWorld；世界内部没有 Room 概念，事件也没有。多房间 = Unreal 专用服务器方案：多个服务器进程 + 匹配服 / 宿主路由把连接送到哪个进程。C-2 五元组的 roomId = 宿主路由键（哪个进程 / 实例），Runtime 接口按实例隐含；`roomSequence` 语义 = 世界内严格递增序号（字段名保留）；cross_room 结局归宿主路由层。本轮不实现多房间；需求 `ecs-entity-chat.md` §6.10 标为待多房间阶段。
12. **生成桥。** 一个生成命令产三件：组件注册表 + 实体模板类（`.g.cs`）、同步表、C-2 契约声明表（json）；扩现有 `gen-declarations`，做成 MSBuild 目标每次 build 自动跑（秒级增量）；生成物入库、测试断言零 diff；非法组合生成时拒；世界只收生成注册表，手写注册与反射破门删除。EntityType 声明式 abstract class（不可实例化、无成员）：`[EntityType(Mode.CS)] [Has(typeof(X))]… [Child("Weapon", typeof(WeaponEntity))] public abstract class PlayerEntity {}`；继承就是 C# 继承——`public abstract class VipPlayerEntity : PlayerEntity {}` 再加自己的 `[Has]`，组件集 = 基类 ∪ 自己（`[Child]` 同理），生成器读基类链并预算父链供 `TypeOf(id).Is<T>()`；`World = true` 的类型恰好一个。**开发期热重载（仅开发构建）**：`dotnet watch`；改方法体 = .NET Hot Reload 原地生效；改字段 / `[Persist]` / Scope / EntityType 组件集 = 生成器重跑 → 世界热重载 = 快照 → 换程序集 → `CreateFromSnapshot`，新字段取默认值，进程不重启、连接不断；改 wire 契约 = 重新握手。不用 Roslyn 源生成器。
13. **World Manager 职责。** 唯一持有世界句柄；准入 / 属性查询 / 聊天 ingress / 存档 / 复制都是它的服务，构造注入 Manager；宿主只持一个 Manager 门面；`Manager.OwnerThread` 在 Start 时记下，所有入口统一校验；网络线程只能 Enqueue 到 Manager inbox。
14. **绑定 = 实体字段 + 派生索引，无独立绑定表。** `IdentityComponent`：`[Persist] Sync<string> Name`（共享，`Scope.Room` + `Authority.Owner`）、`[Persist] AccountId`（Server.cs）、`Connected / ConnectionGeneration / DisconnectedAtTick`（Server.cs，不存档，重启即离线）；不设 `Kind` 字段——实体是 Player 还是 Bot 由 EntityType 决定（`TypeOf`），五元组里的 `entityType` 由此派生。Manager 维护可从世界重建的 accountId → NetEntityId 索引；宿主只持连接 → NetEntityId 会话表；顶号 = 查实体 Connected；过期 = DisconnectedAtTick + 内核定时；C-2 五元组由实体字段 + 宿主会话表拼出。
15. **私有字段措辞。** 私有字段 = 组件上不是 `Sync<T>` 的普通字段（同世界服务器代码可读、客户端读不到、不上网；存档打 `[Persist]`）。硬规则：每实体状态只能存在组件字段里；系统 / 模块私有状态只能是可从世界重建的派生缓存（索引、排序结果），不得替代组件存实体数据；模块不得自建世界。
16. **NetEntityId = 128 位 = 世界实例 ID（64 位，宿主建世界时给，入档）+ 世界内计数器（64 位，入档）。** 世界在提交相创建实体时发号；不随机（确定性义务 + 按 NetEntityId 排序的序号）；wire 32-hex 不变；跨进程唯一、跨重启不复用；`IdentityTableSnapshot` 公开构造 / 宿主自铸旁路删除。
17. **客户端 World。** Client 直接引用 Runtime ECS（`Lumio.GameRuntime.Ecs` + 复制客户端模块 + 玩法 `*.Client` 程序集，全部 netstandard2.1）；删 `ReplicaWorld` 属性袋与手写 `AttributeDeclarationTable`；客户端 World 由同一 World Manager 类建，写法与服务器相同：`WorldManager.Create(GeneratedRegistry.Instance)`，只差不传 `instanceId`（客户端不发号；生成注册表自带端别，不传模式参数）；同样 `Start(ownerThread)`，网络线程只 `Enqueue`，主线程每帧 `Tick()`。连上后前两条消息：欢迎消息（世界实例 ID + 自己的 NetEntityId，Manager 在提交相绑 `World.Self`）、创建记录（第一条是 WorldEntity）。客户端收世界变化消息进提交相、不存世界档、字段上行只按 `Authority.Owner`；Bot.Host 用同一客户端 World。**同进程双端**（单机 / 本地联调）= 两个 Manager（服务器程序集一个、客户端程序集一个）+ 内存环回代替网络：`server.outbox → client.Enqueue` 同一行代码，回调 / 同步 / 权限 / 校验与联网零差异；不共用一个 World（那要第三种编译配置，且 partial 方法两端体会撞）。
18. **事件。** 字段 = 最后状态（可存可查可同步）；事件 = 一次性通知 = `[ClientRpc]`（不存不查不回放）。事件在提交相发出、与字段变化同一 Tick 包下发，投影后服务器即丢（每 Tick outbox）；可靠有序由每连接有界传输队列 + 游标（宿主）保证；重连全量快照不回放；客户端聊天窗口归 UI 层，ECS 不存事件（`OnChatMessage` 到达即交给 UI，组件上不留窗口字段）；Runtime 内 `_displayed` 与 `_eventsByRoomTick` 删除。

样板示例（用户名的声明 → 建世界 → 创建 → 写 → 同步 → 读 → 存档全链路）落 `knowledge/features/ecs.md` §4.5，作为以后所有 ECS 代码与讨论的标准样例。最小 Demo（第二轮定）：建世界 → 建 WorldEntity → 建 PlayerEntity（Identity + Chat）→ Chat 取自己实体的 Name 发消息（名字 + 内容）→ 两端 log 验证，改名后下一句话的 log 就是新名字。

## 替代方案

- **未标注字段默认存档 + `[Transient]` 退出**（ecs.md M4 原文）：被否——Owner 定「未标注 = 什么都不做」，忘打 Persist 是使用者 bug。
- **独立 schema 文件为真源**：被否——多一种声明语言。
- **WorldEntity 只在服务器（Local 实体）/ 由玩法第一帧下单**：被否——多一条通道 / 单例无人保证。
- **独立 CreateEntity RPC + 字段另走同步流**：被否——幽灵实体与中间态。
- **只用标注不用宏 / 一份程序集运行时判角色**：被否——客户端二进制仍含服务器逻辑与 schema。
- **两个独立类型 ClientChat / ServerChat**、**三个类型（共享 + 两端附加）**、**按端分顶层目录**、**单 csproj 双配置**：被否——同步字段两份声明是 r3 漂移的翻版；AI Agent 一处维护优先。
- **只有 InputCommand 一条写路径**（主会话原提议）：被 Owner 否——要有字段自动上行与 RPC 两种；收敛为 Netcode 模型。
- **任何 Sync 字段 owner 默认可写 / 上行直写世界不进输入流 / 世界级命令走独立管理通道**：被否——作弊面、破确定性与线程红线、第二种写机制。
- **标注 + 生成带脏位属性 / 手动 MarkDirty / 影子副本比对**：被否——隐形成员、靠人记、运行时反射近亲。
- **删 AttributeId 查询面**：被否——需取代已 Accepted 的 ADR-053，宿主探针与验收要换方式。
- **Entity 真类字段长在实体上 / 纯组件独立池 / 实体类持有组件成员作公开 API**：被否——组件不可复用、对象多 C 倍、两种读法并存。
- **原地灌入运行中的世界**（现状）：被否——映射不重建、冲突处理。
- **世界内保留 Room 字段过滤 / 同进程多 World Manager**：被否——处处过滤；Owner 定多房间 = 多进程。
- **Roslyn 源生成器 / 改字段重启进程**：被否——IDE 每键触发、生成物不可见；开发期要热重载。
- **静态 `World.Current` / 各模块自持世界句柄**：被否——隐式依赖 / 三个世界的来路。
- **Manager 内独立绑定表**：被否——存档与世界分开维护。
- **组件外零状态 / 模块私有状态不限**：被否——过严 / 正是 `_values` 的来路。
- **64 位计数器搬进世界 / 随机 GUID**：被否——跨进程重号 / 非确定性。
- **Client 自实现轻量 World / 保留属性袋只补 Delta**：被否——第二套 ECS。
- **有界环形事件历史 N Tick**：被否——与「重连不回放」需求矛盾，多一份状态。
- **类型编进 NetEntityId**（第二轮）：被否——类型编号须跨版本稳定、热重载改类型集时旧 ID 说谎；世界本来存着模板，`TypeOf` 一次数组定位。
- **`[Extends]` 标注表达 EntityType 继承 / 只用组合不要继承**（第二轮）：被否——C# 继承已是现成表达，多一个词是两套；Owner 明确有继承关系需求。
- **`Sync<EntityKind> Kind` 字段**（第二轮）：被否——EntityType 已决定类型，同一件事两处维护。
- **`[ServerOnly]` / `[ClientOnly]` 与文件后缀双份声明**（第二轮）：被否——生成器按 csproj 跑本来看不见另一端文件，标注是第二份声明。
- **单入口 `OnSyncChanged(in SyncChange)` 变化回调 / 每字段 `+=` 事件 / 不要回调靠读字段**（第二轮）：被否——单入口内部 n 个 `c.Is()` 分支且容器 payload 越做越厚；struct 上挂事件随拷贝丢失；Owner 明确要 old / new + 改前改后各一次。
- **本端写默认触发变化钩子**（第二轮）：被否——Owner 定「自己改自己不收」才是对的默认，可选 `Notify.All` 打开。
- **多字段 WhenAll 式组合器进引擎**（第二轮）：后置——需求成立、可实现、复杂度高（续体在提交相调度、跨 Tick 中间态）；同 Tick 靠批语义与一致性组，创建靠 PostAttribute，跨 Tick 玩法自判就绪。
- **同进程双端共用一个 World**（第二轮）：被否——需要第三种编译配置，partial 方法两端体相撞；两个 Manager + 环回零特例。
- **客户端专用 `CreateClient(registry, instanceId, selfId)`**（第二轮）：被否——多一个入口，同进程双端下宿主还得从服务器 Manager 掏参数；欢迎消息经 Enqueue 同一条路。
- **引擎内置固定 WorldEntity 类型**（第二轮）：被否——游戏没处放世界级状态，只能另建第二个单例。
- **客户端聊天窗口挂在实体组件 / WorldEntity 客户端组件**（第二轮）：被否——每客户端一份的 UI 状态声明成每实体一份，每个实体白分配空 List；事件不存，窗口归 UI 层。

## 接口 / Schema

- **C-2 `engine/wire/entity-binding-and-query-v1.json`**：`roomId` = 宿主路由键（一进程一 World Manager 一 GameWorld），Runtime 接口按实例隐含；`cross_room` 由宿主路由层判定；Admit 结局 `account_already_online`（正用例 `admit_second_connection_account_already_online`，形状错误仍 `invalid_binding_shape`）。绑定五元组由 `IdentityComponent` + 宿主会话表拼出。`NetEntityId` 128 位 = 实例 ID（高 64）+ 计数器（低 64），32-hex 小写。AttributeId 查询面是生成的薄适配层，无自有存储。
- **C-1 `engine/wire/gameplay-command-envelope-v1.json`**：`roomSequence` = 世界内严格递增序号（字段名不变）；`entity.identity` = 创建记录；新增 InputCommand 种类 `field.write`（正用例 `input/field-write-owner-name`，反用例 `runtime/field-write-other-entity` / `runtime/field-write-server-authority` → `unauthorized`）。`chat.event` 的 `senderNetEntityId` 编码为 `senderNetEntityIdInstanceId`（u64 LE）+ `senderNetEntityIdCounter`（u64 LE），16 字节定宽，与 C-2 32-hex 是同一 128 位值；不引入 ADR-047 `u128` 原语。`chat.input` = `ChatComponent.SendMessage` ServerRpc；`chat.event` = `ChatComponent.OnChatMessage(string line)` ClientRpc，`line` = 服务器拼好的「名字: 内容」，走 `text` 字段，**C-1 不加名字字段**（第二轮：Owner 定「两端都行，重点是演示取自己的名字」，取不改契约的路）。
- **Runtime 公开 API 面（新增，R4-05 落地；第二轮修订）**：`WorldManager.Create(registry, instanceId?)`（服务器必传、客户端不传）/ `CreateFromSnapshot(snapshot)` / `Enqueue(WorldMessage)`（`WorldMessage` = InputCommand / 欢迎消息 / 世界变化记录的统一入口类型）/ `OwnerThread` / `Tick()`；`World.Self` / `World.Commands.Create<T>()` / `Get<T>(netId)` / `Each<T>()` / `Single<T>()` / `TypeOf(netId)`（类型句柄，`.Is<T>()`）；`Sync<T>(Scope, Authority, Notify)` / `SyncList<T>` / `SyncDict<K,V>`、`Notify { Remote, All }`、`ChangeReason { Sync, Correction, Local }`、`ListChange<T>` / `DictChange<K,V>`；标注 `[EcsComponent]` / `[Persist]` / `[EntityType(Mode, World)]` / `[Has]` / `[Child]` / `[ServerRpc]` / `[ClientRpc]`；组件基类上的 `Get<T>()` / `Get<T>(id)` / `World` / `Rpc` / `OnClientWrite(in SyncWrite, ref bool accept)` / 九回调；生成的每字段 `OnXChanging` / `OnXChanged` 可选钩子与 RPC 发送桩。
- **生成三件**：`<Assembly>.Registry.g.cs`（组件注册表 + 实体模板类 + 每字段可选钩子声明 + RPC 发送桩）、`<Assembly>.Sync.g.cs`（同步表）、`generated/attribute-declarations.json`（C-2 声明表，sha 与契约一致）。

## 失败语义

- 生产源出现第二条 CreateWorld 路径、空世界当线程牌子、手写组件注册、组件之外存每实体状态、事件历史容器、通用 SetField 入口——结构断言直接失败，收口审查退回。
- 共享文件（无后缀）里出现非 Sync 的状态字段——lint 失败（另一端程序集里会出现永远是默认值的死字段）。
- EntityType 缺必需组件 / 依赖成环、阶梯外类型、`World = true` 的类型不是恰好一个、EntityType 声明类不是 abstract 或带成员——生成命令失败，不留到运行时。
- 生成物与源码不一致（零 diff 断言失败）——视同手改生成物。
- 非 owner 写 `Authority.Owner` 字段、任何客户端写 `Authority.Server` 字段——服务器拒绝并权威纠正，记日志。
- 生产构建开启世界热重载——违反 `rules/system.md`「dev-only 开关不得在生产开启」。

## 兼容影响

- `knowledge/features/ecs.md` M1 / M2 / M3 / M4 / M8 / M9 按本 ADR 改写，新增 M1a「World Manager 与 WorldEntity」与 §4.5 样板示例；「ReplicaWorld」全部改「客户端 World」；八回调改九回调（加 PostAttribute）。第二轮：ecs.md 规范词表 / M1a / M2 / M4 / M9 / §4.5 / §5 / §6 / §7、`ecs-entity-chat.md`「entity kind」措辞、r4 蓝图与 R4-05 卡片正文、Runtime `modules/ecs/samples/username/` 同步。
- ADR-056 §1「单一 ECS」与 ADR-057 第 7 条由本 ADR 填实结构；ADR-053 保留，AttributeId 查询面降为适配层。
- Client `ReplicaWorld` 退役；Client 引用图开放 Runtime ECS；`ecs-entity-chat.md` §6.10 后置到多房间阶段。
- 本 ADR 转 Accepted 的条件：R4-05 / R4-04 合入并由独立深审复核本 ADR 验证 Fixture。

## 迁移方案

按 `plans/2026-09-03-rm-00011-r4-blueprint.md`：R4-01（契约措辞）→ R4-05（Runtime：World Manager、生成三件、Sync<T>、ServerRpc/ClientRpc、128 位发号、CreateFromSnapshot、删三世界 / 字典 / 历史）→ R4-04（Client：引用 Runtime ECS、客户端模式 World、删属性袋、顶号回登录、Bot 常驻）→ R4-06 / R4-09 集成。开发期热重载与多房间只写设计概要，不进 r4。

## 验证 Fixture

1. **单一世界**：Runtime 生产源只有一条 CreateWorld 路径（结构断言）；WorldId 1 / 2 / 370 与 `ChatIngressWorld`、`_values`、`_liveConnectionByAccount`、`_eventsByRoomTick`、`_displayed` 不存在（grep）。
2. **生成三件**：一句命令产注册表 / 同步表 / 声明表，入库零 diff；手写注册与反射破门不存在；共享文件里放非 Sync 状态字段 lint 失败；`World = true` 缺失或重复生成失败。
3. **查询读真值**：`chat.input` 提交后 `QueryAttribute(ChatComponent.lastMessageText)` 返回该 Tick 写入的文本。
4. **顺序一致**：同 Tick 100 条上行按发送者 NetEntityId 排序编号，两轮日志逐位一致。
5. **恢复身份**：`CreateFromSnapshot` 后每个 NetEntityId 可达且不变，新建实体不与档内 id 重号；NetEntityId 高 64 位 = 实例 ID。
6. **客户端同套 ECS**：Client csproj 引用 Runtime ECS，属性袋与手写声明表不存在；创建记录按 EntityType 建实体，探针确认 Awake → PostAttribute → Start；子类型实体 `TypeOf(id).Is<基类>()` 为 true；改名后其他客户端 `OnNameChanged` 收到 `Sync`、被拒的 owner 收到 `Correction`，owner 自己写不触发。
7. **事件无历史**：连续 N Tick 聊天后 Runtime 常驻内存不随 Tick 增长；重连只收全量快照。
## 修订记录（2026-09-04，ADR-060）

ADR-060 supersedes the prior wire projection details without changing this ADR's decision or status. C-1 now uses Welcome/WorldChange/InputCommand/ConnectionSuperseded/Error; WorldChange carries creates, field changes, destroys, and ClientRpc records with 128-bit identifiers. Admission identity is delivered by Welcome and chat events are ClientRpc records.
