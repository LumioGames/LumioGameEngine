# MVP 大纲：多浏览器联机体素世界

> **基线**：`LGE-V1.4-2026-08-27`（本文不改公共契约，只是对既有架构的 MVP 级裁剪与排期）
> **状态**：Draft（讨论共识沉淀，未立项）
> **日期**：2026-08-27
> **来源**：架构可行性讨论——C# 热更双端执行、无引擎 MVP、浏览器客户端、在线/离线形态

## 1. 讨论确认的五个结论

以下结论全部有 v1.4 基线明文出处，MVP 不需要任何架构变更：

1. **C# 热更代码天然双端执行。** Gameplay/GAS 逻辑是标准 IL Assembly：客户端经 HybridCLR 加载（AOT 平台的 IL 执行拐杖），服务器经 CoreCLR `AssemblyLoadContext` 加载（原生能力，不需要 HybridCLR，ADR-014 明确拒绝其为 V1 前提）。服务器热更走 ADR-034 双 Scope ALC 切换；两端版本由 `ReleaseManifest` Assembly Hash + Handshake 锁定。
2. **无游戏引擎的纯 C# 形态是默认开发路径，不是妥协。** `PureHeadless`（`NoNative` + `ReferenceVoxelPort`）到 `LocalSplitProcess` 四个无引擎 Preset 覆盖 Vertical Slice 全部验收（§16.1）；Unity 只是 `LumioClient` 边缘的一个 Presentation/Host Adapter，P2 才深度接入。
3. **浏览器客户端可行，且不需要 Rust 进浏览器。** 权威 `VoxelWorld`（Rust）跑在服务器；浏览器只跑 `VoxelReplicaWorld` 副本，由 `ReferenceVoxelPort`（纯 C#，架构本来强制要求存在）承担，.NET WASM 运行时直接执行。Rust→WASM `StaticLink`（与 iOS 同档）留作 P2 独立 ADR，不进 MVP。
4. **浏览器的真实成本在传输层，不在语言运行时。** 浏览器无裸 UDP/TCP，MVP 用 WebSocket（全可靠有序，MVP 规模够用）；Envelope/Serializer/权限校验原样复用（§7.3 规则：可换传输，不可绕业务协议）。WebTransport（QUIC 不可靠数据报）留作后续。
5. **"浏览器在线瘦客户端 + 桌面/移动离线本地计算"是 Host Profile 原生能力。** 在线 = `RemoteDS`；离线 = `LocalEmbedded`/`MobileLocal`——离线的本质是同一份权威代码搬进本地进程跑两棵树，Gameplay 禁读 `IsOffline`（ADR-014），存档在线离线互转（§13.4）。离线模式不进 MVP，但 MVP 的纪律保证它以后能直接接上。

## 2. MVP 目标

**一句话：N 个浏览器登录同一个体素世界联机，挖/放方块实时互见。**

这与架构里程碑 3（Vertical Slice：`PlaceVoxelAbility` 跑通事务、复制、预测、Replay、存档）高度重合——MVP 即 Vertical Slice 换上浏览器宿主，一份工作两份验收。

### 2.1 目标范围内

- 多个浏览器客户端经登录（简单 token 存根）接入同一 `WorldSlot`。
- 服务器权威模拟：ECS + GAS（挖/放方块 Ability）+ 体素世界，完整十三相 Tick 与 CrossWorldTxn。
- 复制链路：FullSnapshot → BaselineAck → Delta → Resync（§7.1 全状态机）。
- 客户端预测 + 单一权威更新事务（§7.2 六步，含预测回滚）。
- 浏览器 WebGL 粗糙渲染（Presentation Adapter 的第一个 Renderer）。
- Headless Bot 客户端（CI/压测用，与浏览器客户端同一套逻辑）。
- 存档：Snapshot + WAL 最小闭环，能存能读。

### 2.2 明确不做（非目标）

| 不做 | 归属 |
| --- | --- |
| Unity / HybridCLR 接入 | P2（架构阶段 5） |
| UDP / WebTransport 传输 | 后续 TransportProfile 扩展 |
| 离线模式（LocalEmbedded 产品形态） | MVP 后第一批 |
| Rust 体素进浏览器（StaticLink wasm32） | P2 独立 ADR，决策依赖三个测量：单线程副本负载、WASM 性能折损、内存预算 |
| Release Pool 滚动更新、强制维护、Migration DAG | 架构阶段 4（Production Hardening） |
| Mod、Sharding、Authority Transfer | P2 |
| 正式 Auth 线格式 | D-011 悬决，MVP 用存根 |

## 3. 运行拓扑

```text
Server 进程 (Linux/本机, CoreCLR)
└─ ServerSimulationSession
   ├─ GameWorld        (ECS/GAS 权威)
   ├─ VoxelWorld       (权威体素；轨道 B 汇合前为 ReferenceVoxelPort，汇合后为 Rust Native)
   ├─ Coordinator + SnapshotCut
   └─ per-client ReplicationContext × N

        ▲ WebSocket (WSS)          ▲ 内存传输 (开发调试)
        │                          │
浏览器 A/B/C… (.NET WASM)          Headless Bot (CoreCLR 控制台)
└─ ClientReplicaSession            └─ ClientReplicaSession（同一套代码）
   ├─ ReplicaWorld (ECS 副本)
   ├─ VoxelReplicaWorld = ReferenceVoxelPort (纯 C#)
   ├─ PredictionHistory
   └─ Presentation Adapter → WebGL/Canvas
```

挖方块端到端链路（全部为 v1.4 已冻结语义）：浏览器输入 → `ClientCommandSeq` 规范化上行（§4.3）→ 可选本地预测 overlay → 服务器 `ApplyInputs` → GAS Ability → `CrossWorldPrepare` 校验 → `VoxelCommit` → `GasAndEventFinalize`（唯一 Commit Point）→ `ReplicationProjection` 产出 delta → 各浏览器单一权威更新事务应用 → `EmitPresentationDiff` → WebGL 重绘。

## 4. 工作分解：两条轨 + 四个阶段

### 轨道 A：C# 逻辑闭环（主轨）

| 阶段 | 形态 | 内容 | 退出条件 |
| --- | --- | --- | --- |
| A0 | `PureHeadless` + `LocalEmbedded`，单 C# 进程 | Runtime ECS/Tick 骨架、GAS 最小生命周期、ReferenceVoxelPort、CrossWorldTxn、两棵树 + 内存传输、挖/放 Ability | 单进程内挖方块经完整事务提交并复制到副本树；Replay 一致 |
| A1 | `LocalSplitProcess`，两个 C# 进程 | WebSocket 传输适配（Envelope/Serializer 复用）、Handshake/Auth 存根、Headless Bot、断线 Resync | Bot 客户端跨进程联机挖方块；断连重连走 Full Resync 恢复 |
| A2 | 浏览器接入 | .NET WASM 宿主壳（rAF 驱动 Host Tick）、客户端 Assembly WASM 面收窄、WebGL Presentation Adapter、登录页 | 两个浏览器互见挖方块；预测回滚肉眼可验 |
| A3 | MVP 收口 | 多浏览器（≥5）并发、Snapshot/WAL 存档闭环、确定性 Replay 门、故障注入（丢连接/慢客户端） | 见 §6 验收标准 |

### 轨道 B：Rust 体素权威（并行轨）

| 阶段 | 内容 | 汇合点 |
| --- | --- | --- |
| B0 | VoxelEngine 首批子模块（world/chunk/revision/mutation/snapshot）+ Native ABI 按 §8.1 | — |
| B1 | CoreEngine 最小 Loader（`NativeHeadless` 档）+ Managed Adapter | — |
| B2 | **Differential 测试：ReferenceVoxelPort vs Rust Native 语义一致**（§15.2） | 通过后服务器权威体素从 ReferencePort 切换为 Rust Native——契约不变，副本端无感知 |

> 轨道 B 不阻塞 A0–A2；A3 收口时服务器权威体素以 B2 汇合结果为准（未汇合则 MVP 演示以 ReferencePort 权威出，Rust 切换为紧随其后的第一项）。

### 各仓工作包归属（首批子模块视角）

| 仓库 | MVP 工作包 |
| --- | --- |
| `LumioGameRuntime` | `ecs`、`simulation`（十三相最小实现）、`coordination`（CrossWorldTxn）、`replication`、`gas`（通用状态机）、`persistence`（Snapshot+WAL 最小）、`config` 最小 Port |
| `LumioVoxelEngine` | 轨道 B 全部；ReferenceVoxelPort 归 Runtime/架构仓契约侧，语义以 Differential 测试对齐 |
| `LumioServer` | MVP 期用 C# 测试宿主承担 Host（`lumio test` CLI 形态）；`transport`（WebSocket）、`auth` 存根、`session`、`world-slot` 最小实现；Rust Host + `coreclr-host` 延后 |
| `LumioClient` | `connection`、`handshake`、`replica`、`prediction`、`input`、`bot`；新增浏览器宿主壳与 WebGL Renderer（"更多 Renderer" 的第一个） |
| `LumioGame` | `server-gameplay`/`client-gameplay` 最小集、`gas-content`（挖/放 Ability）、`mapping`、最小 `config`/`content` |
| `LumioCoreEngine` | 轨道 B1 最小 Loader；`PureHeadless` 走 `NoNative` 无 Loader 路径 |
| `LumioNativeCore` | 轨道 B 依赖的 `contract-types`/`error`/`handle`/`native-core-ffi` 最小集 |
| 架构仓（本仓） | WebSocket 档的 TransportProfile Capability 登记；如需新增公共字段/错误码，按 ADR → Schema → Fixture 流程走 |

## 5. 必守纪律（MVP 期的六条红线）

MVP 无引擎跑起来没有悬念，风险全在"省事欠债"。以下六条从第一天强制：

1. **表现只走事件**：渲染/日志只消费 `EmitPresentationDiff` 与副本世界只读投影；Gameplay 不得直接输出表现。
2. **两棵树不焊死**：LocalEmbedded/浏览器形态下渲染不得直读权威世界（§17 禁止清单第一条）；违反即毁掉在线/离线一致性与存档互转。
3. **传输可换、协议不可绕**：WebSocket/内存传输都复用同一 Schema、Serializer、Envelope、权限校验、有界队列（§7.3）。
4. **共享 Assembly API 面从第一天收窄**：target 未来两端交集（netstandard2.1 量级 + WASM 支持子集），analyzer 禁越界引用（UnityEngine、Server Host、WASM 不可用 API）。
5. **确定性规则第一天冻结**：RNG Seed/Stream、定点/舍入、事件排序、Canonical Hash（§4.4）；GAS Formula 求值用定点或冻结舍入。A0 起每阶段跑 Replay 一致性。
6. **Differential 测试与 ReferencePort 同期交付**：ReferenceVoxelPort 在本 MVP 中是用户可见路径（浏览器副本），不再只是 CI 工具；它与 Rust 实现的语义漂移 = 玩家可见 bug。

## 6. 验收标准（MVP DoD）

1. ≥5 个浏览器客户端登录同一 `WorldSlot`，任一客户端挖/放方块，其余客户端在一个复制周期内可见。
2. 预测路径可验：本地挖方块立即反馈；构造服务器拒绝场景时预测回滚正确（`RolledBack` 语义，画面还原）。
3. 断线重连：杀掉任一浏览器连接后重连，经 Full Resync 恢复到一致世界，无实体复活/幽灵方块。
4. Headless Bot 以同一套客户端代码通过上述 1–3 的自动化版本。
5. 确定性：同 Seed 同输入序列的 Replay 产出逐 Tick 一致的 Snapshot Hash（Level 1，§4.4）。
6. 存档：服务器 Snapshot+WAL 落盘后重启进程，世界状态恢复且各客户端可重新接入。
7. （轨道 B 汇合后）Differential 测试通过：同一 Mutation 序列下 ReferenceVoxelPort 与 Rust Native 的 Snapshot Hash 一致。
8. 全程无一处违反 §5 六条纪律（代码审查项）。

## 7. MVP 之后的演进路径（已铺好的格子）

| 步骤 | 内容 | 架构依据 |
| --- | --- | --- |
| +1 | 服务器权威体素切 Rust Native（若 MVP 内未汇合）；Rust Host + `coreclr-host` 替换 C# 测试宿主 | §16 Server 子模块 |
| +2 | 桌面客户端 + 离线模式（`LocalEmbedded`）：同一权威代码本地跑，存档与 DS 互转 | §10、§13.4、ADR-014 |
| +3 | WebTransport 不可靠数据报档，激活 Envelope 可靠性分级的真实语义 | §7.3 TransportProfile |
| +4 | Unity 接入：`unity-adapter` 作为又一个 Presentation/Host Adapter；HybridCLR 按 ADR-014 Capability 门接入 | ADR-014 |
| +5 | 移动端 `MobileLocal`（安卓 `.so` 动态库 / iOS `StaticLink`） | ADR-020 |
| P2 | 浏览器真客户端跑 Rust 体素（wasm32 `StaticLink`）——立独立 ADR，凭三个测量数字决策 | ADR-020 轴系 |
