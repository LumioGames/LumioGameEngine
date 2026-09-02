# ADR-056：RM-00011 架构收敛——单一 ECS、单一绑定/查询、字段标注声明、单一定时内核、真实广播与落盘

状态：Draft（2026-09-02，Owner 与独立深审 reviewer 七步对照 `knowledge/features/ecs-entity-chat.md` 与实现后的裁决；待 Owner 定稿为 Accepted）
取代：无（修订 ADR-055 的双层定时分层与 ADR-045 在 Room 路径上的 FullSnapshot 形状，见「兼容影响」）
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioGameRuntime`（ECS / 绑定 / 查询 / 持久化唯一实现）、`LumioNativeCore`（定时内核）、`LumioServer` / `LumioClient` / `LumioGame`（消费方）

## 治理原则

本 ADR 及后续全部架构裁决的最高原则：**第一性原理——如无必要，勿增实体。** 同一职责只允许一份实现；已存在多份的一律收敛到一处；提案先证明「必要」再新增层、副本或旁路。RM-00011 的全部偏离都是这一条被违反的结果。

## 背景

RM-00011（ECS 正式实体与聊天纵切）已在 Workflow 全部标记 done，独立深审（`reviews/2026-09-02-rm-00011-architecture-deviation.md`）逐维对照设计真值后发现：切片验收跑通的不是设计里的那套架构，而是围绕设计另建的三到四套平行实现。具体：

- Room 世界不是 Runtime `EcsWorld`，而是 Game 仓自带的 `ChatRoomWorld`（字典 + 私有队列，csproj 零依赖）。
- 连接绑定与 Attribute Query 有四份：Runtime `EntityBindingQuery`（按契约实现、无人消费）、C# 宿主 `RoomAdmissionRegistry` + `LiveElevenHost.Query`（硬编码 switch）、Rust 宿主 `host.rs`（硬编码 match）。
- 属性三维标签手写在 C-1 `dimensions`、Runtime `DeclareDefaults()`、Game `ChatComponentSchema` 三处，查询时一处都不读。
- 聊天事件在两个宿主都未发到网线；`FullSnapshot` 发的是 ADR-045 五字段空基线，客户端等的是 C-1 带 `stateBlocks` 的完整快照；顶号通知只存字典未下发。
- 定时：宿主闹钟三份（C# `ITimerService`、Rust 轮询、NativeCore 闲置 `HostTimerService`）；Native Timer Manager 为 Rust rlib 且不进 C ABI，C# 消费者无通道，所谓「真实消费者」是 crate 内测试。
- 持久化：Runtime `EcsPersistSnapshotPipeline` 无人调用，三条内存旁路各自往返，全切片无落盘。

## 决策

1. **单一 ECS。** Room 世界必须是 `LumioGameRuntime` 的 `EcsWorld`；`ChatComponent` 是其普通组件；聊天输入走 Runtime 统一命令提交路径（IngressCapture → CommandBuffer → 固定 Tick 结算）。`ChatRoomWorld` 及其私有队列退役。
2. **单一绑定与查询。** 连接↔实体绑定、自查、解析与 Attribute Query 只在 Runtime 一处实现；宿主只托管、只转发，不得保有第二份绑定表或查询 switch。
3. **单一发号。** `NetEntityId` 由 Runtime 身份表发号并登记墓碑；宿主准入时向 Runtime 要号。绑定记录只保留冻结五元组（AccountId / RoomId / NetEntityId / EntityType / ConnectionGeneration）；会话号归连接层会话表，宿主内部句柄归宿主私有映射，不得混入绑定记录。
4. **字段标注即声明。** 属性三维（持久化 / 复制 / 可见性）的唯一来源是组件字段上的标注（`knowledge/features/ecs.md` §M4）；契约声明表、Runtime 注册表、客户端可见性判断全部由标注生成，不许手写第二份。`AccountId` 不声明为可查属性，只在登录回应中给出。
5. **真实广播。** `ChatMessageEvent` 必须经最小可靠有序 Room 广播真正送达每个客户端；验收以客户端（含 Chromium 页面）实际收到为准，宿主进程内替客户端维护的窗口不算。
6. **完整快照与顶号通知上网线。** Room 路径的 `FullSnapshot` 以 C-1（ADR-049）带 `stateBlocks` 的完整快照为准，宿主必须发出实体状态；ADR-045 五字段体仅为 MS-00001 基线，不再是 Room 路径的 FullSnapshot。顶号时旧连接必须收到 `connection_superseded` 通知后再关闭。
7. **单一定时内核。** 定时内核只有一个，在 NativeCore（Rust），经 `engine/abi/native-abi.json` 暴露给托管侧；同时支持单调墙钟到期（承载五分钟断线保留）与 Tick/帧节拍两种模式。Game / Server / Client / Worker 各自的 Timer Manager 只是适配层，不得自建定时器。C# `ITimerService`、Rust host-runtime `HostTimer` 与轮询式 `expire_due`、NativeCore `HostTimerService` 门面一并收敛。
8. **单一持久化路径并真落盘。** 组件级快照/恢复只走 Runtime `EcsPersistSnapshotPipeline`，快照按 ADR-032 记录头写成规范字节落盘，验收必须证明进程重启后读回；WAL/命令日志本切片不做（如无必要勿增实体），留持久化主线，但快照记录头预留兼容。

## 替代方案

- **维持现状，按宿主各自实现验收**：被否决——四份实现已经造成同一查询三种答案、证据观测深度不一致（见深审 P1-1），且与 ecs.md「同一套玩法代码两边都跑」直接矛盾。
- **只收敛 C# 宿主，Rust 宿主保留自己的绑定表**：被否决——Rust 宿主是接力交付面，若它保有第二份绑定真值，收敛等于没做。
- **定时保留双层各自独立基础设施（ADR-055 原文）**：被 Owner 修订——两层是一个内核的两种模式，不是两套基础设施；保留双套即「增实体」。
- **本切片先用空基线 FullSnapshot / 内存快照往返**：被否决——需要 Owner 在卡上签字的替代条款从未签，且与设计「完整权威快照重建」「既有 ECS 快照路径」字面相悖。

## 接口 / Schema

- `engine/wire/native-timer-abi-v1.json`（C-4）与 ADR-055：修订为「单内核双模式」，并把定时内核的托管可达面纳入 `engine/abi/native-abi.json`（ABI 变更须经 `eng/generate-abi.mjs` 重生成，`DEFINITION_SHA256` 随之变化）。
- `engine/wire/entity-binding-and-query-v1.json`（C-2）：`attributeDeclarations` 改为「由字段标注生成」的生成物，不再手写示例表；增加 `NetEntityId` 由 Runtime 发号的措辞。
- `engine/wire/gameplay-command-envelope-v1.json`（C-1）：`mappings.*.dimensions` 改为生成物；`FullSnapshot.stateBlocks` 为 Room 路径唯一快照载体的措辞加固。
- 顶号通知：在 C-3 或 C-1 增加 `connection_superseded` 下行通知消息形状（待定稿时选定落点，原则是不新增第四份契约）。
- 组件字段标注：在 Runtime 定义标注类型（命名待定稿），生成器扫描标注产出声明表；`ChatComponent` 作为第一个打标组件。

## 失败语义

- 宿主若保有第二份绑定表或查询 switch，架构测试（Runtime 依赖方向 + 宿主禁用词表）直接失败。
- 未打标注的字段不得上网、不得入档；查询未声明属性返回 `undeclared_attribute`，不得映射为 `unauthorized`。
- 广播路径缺失时验收器不得凭发送计数生成 `eventOrder` / `appliedTicks`；证据字段必须来自客户端收到的事件。
- FullSnapshot 缺 `stateBlocks` 或为 ADR-045 形状时，客户端按 C-1 拒绝，验收 S8 不得通过。
- 快照未落盘或重启后读不回，S7 不得通过。

## 兼容影响

- ADR-055「双层定时、两者一等公民」分层被本 ADR 修订为「单内核双模式」；ADR-055 其余（TimerHandle / CallbackSlot / 确定性投递）保留。
- ADR-045 的 FullSnapshot 闭合体对 Room 路径不再适用，由 ADR-049 / C-1 接管；ADR-045 仍约束 MS-00001 基线信封。
- 已合入的 Game `ChatRoomWorld`、Server `mvp-host/` 切片编排（`LiveElevenHost` 等）、Rust `entity_chat::host` 绑定表、NativeCore `HostTimerService`、Server ADR 0006「C# 冻结」的前提（identical suite 已过）——全部需按本 ADR 重做验收；Workflow R-00346 / R-00347 / R-00348 / R-00350 / R-00351 / R-00352 / R-00353 / R-00354 / R-00359 的 done 状态不再代表设计已落地。

## 迁移方案

按 `reviews/2026-09-02-rm-00011-architecture-deviation.md` §3 的重拆卡建议执行：Wave 0 先修三份契约（C-1 / C-2 / C-4）与标注生成器，Wave 1 让 Runtime 成为唯一实现并让两个宿主改为消费方，Wave 2 以「客户端真收到、快照真落盘真读回」重跑 11 场景。C# 宿主冻结（Server ADR 0006）在 Rust 宿主按本 ADR 重新通过前撤回。

## 验证 Fixture

- 依赖方向：Game / Server / Client 对 `Lumio.GameRuntime.Ecs` 与 `Lumio.GameRuntime.Replication` 的引用不为零；宿主源码内不得出现绑定表/查询 switch（禁用符号表）。
- 标注生成：从 `ChatComponent` 标注生成的声明表与 C-1 / C-2 契约内嵌表逐字节一致（生成物零 diff）。
- 广播：Playwright 页面 `__lumioChat.window.lines` 长度 = 101 且顺序与服务器 `roomSequence` 一致；两轮一致。
- 快照：进程 A 落盘 → 进程 B 读回 → `ChatComponent.lastMessageText` 逐实体相等、`historyCount` = 0。
- 定时：五分钟到期由 NativeCore 内核回调触发（trace 有内核事件），非任何调用方顺手轮询；Bot 发言由 Client Timer Manager 每 N Tick 触发并在客户端 trace 可见。
- 顶号：旧连接收到 `connection_superseded` 帧后再收到关闭。
