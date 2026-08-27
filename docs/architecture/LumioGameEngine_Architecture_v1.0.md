# LumioGameEngine V3 (v1.0) 架构与开发规范

> **ArchitectureBaselineId**：`LGE-V1.0-2026-08-27`
> **状态**：Implementation Baseline
> **唯一架构源**：`LumioGameEngineArchitecture`
> **适用范围**：LumioNativeCore、LumioVoxelEngine、LumioCoreEngine、LumioGameRuntime、LumioServer、LumioClient、LumioGame
> **说明**：本文冻结跨仓公共语义；内部实现、存储布局和具体开源依赖可以在不破坏契约的前提下演进。

## 1. 目标、范围与优先级

LumioGameEngine 的目标是让同一套 Gameplay/Scenario 在最小环境、Native Headless、Local 双角色、独立进程、Dedicated Server、Replay 和 CI 中复用，并且能在真实网络、版本升级、崩溃和资源压力下产生可诊断、可恢复的结果。

本基线把需求分为两条轴：

- **架构必备**：必须现在定义所有权、接口、数据格式和失败语义。
- **实现优先级**：P0 先实现，P1 随基础闭环实现，P2 后置实现。P2 仍必须在本基线中有明确预留。

### 1.1 不可违背的原则

1. 越底层越通用；领域语义不能倒灌到通用 Kernel 或聚合发布层。
2. Server 与 Client 永远拥有独立的本地 World、Entity 和状态真相。
3. 所有跨线程、跨进程、跨语言和跨 World 操作都经过有版本的契约。
4. 权威状态只在规定的 Simulation Owner Thread 和 Tick Barrier 提交。
5. 失败必须可分类、可记录、可重放；不能以“调用方自行处理”代替协议。
6. 优先复用成熟开源框架；第三方 API 通过 Adapter 隔离，许可证和供应链风险可审计。
7. P2 是后置交付，不是永久不支持；任何后置能力都不能破坏 V1 的核心边界。

### 1.2 术语

| 术语 | 语义 |
| --- | --- |
| `ProductId` | 产品/游戏身份，例如 `A`、`BOE`。 |
| `GameReleaseId` | 一次可验证的 Server/Client/Content 组合发布身份。 |
| `WorldSlot` | Host 内承载一个逻辑 Session 的资源和故障单元。 |
| `SimulationSession` | Runtime 拥有的逻辑模拟上下文，不等于远端 Client 对象。 |
| `Revision` | 必须带域语义；禁止用一个无定义的整数表示所有版本。 |
| `SnapshotCut` | 在 Tick Barrier 固定的跨 World 一致读取切面。 |
| `Capability` | Host、平台、Native、测试和资源能力的声明，不是业务模式布尔值。 |

## 2. 仓库拓扑与所有权

### 2.1 七仓库职责

| 仓库 | 必须拥有 | 明确不拥有 |
| --- | --- | --- |
| `LumioNativeCore` | 领域无关 Rust Kernel、Handle、Error、Capability、内存、Job、空间、压缩和 ABI 基础。 | Voxel、ECS、GAS、Gameplay、网络、Session、CoreCLR。 |
| `LumioVoxelEngine` | VoxelWorld、Chunk、Block、World/Chunk Revision、Mutation、Streaming、Voxel Snapshot/Diff、Voxel Migration 和 Voxel Spatial Source。 | Gameplay 权限/扣费、Socket、Session、ECS Storage、Host 生命周期。 |
| `LumioCoreEngine` | Native 聚合构建、统一 Root ABI、单包 Loader、Manifest、Hash、签名、SBOM 和平台产物。 | 领域算法、World 状态、ECS、GAS、Gameplay、迁移业务语义。 |
| `LumioGameRuntime` | ECS 语义、Logical Tick/Phase、Game/Replica World、Coordinator、Replication 语义、GAS Framework、Snapshot Cut、Hot Reload 契约和 Determinism Kit。 | 进程、Socket、端口、Voxel 内部、具体玩法。 |
| `LumioServer` | Rust Host、网络、Auth/Connection、Release Pool、WorldSlot、Host Pacing、CoreCLR Hosting、维护和升级编排。 | ECS Storage、Logical Phase 语义、Gameplay 规则、Voxel 内部。 |
| `LumioClient` | Client Connection/Handshake、Replica Apply、Prediction 驱动、Input/Presentation Adapter、Unity Host、HybridCLR Capability 和 Headless Bot。 | Server 权威、协议 Schema 唯一来源、具体 UI/内容、Native 内部。 |
| `LumioGame` | Server/Client Component、Processor、RPC Payload、Mapping、GAS Content、配置/内容、Scenario、Release Composition 和业务 Migration。 | Runtime/Host 生命周期、通用 ABI、网络治理、Voxel 内部。 |

### 2.2 三张依赖图

**源码依赖约束**（`A -> B` 表示 A 只能编译依赖 B 的已发布 API、Schema 或 Artifact；箭头不是运行时调用方向）：

```text
LumioVoxelEngine -> LumioNativeCore
LumioCoreEngine  -> LumioNativeCore + LumioVoxelEngine (仅源 Schema/Artifact)
LumioGameRuntime -> Generated Native/Voxel Contracts
LumioServer      -> LumioGameRuntime + LumioCoreEngine Package
LumioClient      -> LumioGameRuntime + Server/Runtime Contracts
LumioGame        -> LumioGameRuntime + Server/Client Host Contracts
```

Gameplay Assembly、Config/Content 和生成契约作为版本化构建产物向 Host 输入，不形成对 Host 实现源码的反向依赖；任何生成物依赖环都必须在 Contract Toolchain 阶段拒绝。

**Generated Artifact 图**：

```text
Native/Voxel Source Schema
        -> Native ABI + Voxel Contract
        -> CoreEngine Package + Managed Adapter
        -> Runtime/Host Contract
        -> Game Gameplay Schema + Mapping
        -> Server/Client Release Package
```

**运行时加载图**：

```text
ReleaseCatalog
  -> Server/Client Host
  -> one CoreEngine package per process
  -> stable Runtime
  -> role-specific Gameplay Assembly
  -> Config/Content/Snapshot
```

不得通过运行时反射或隐式源码依赖绕过上述 DAG。

### 2.3 RACI 与时钟所有权

| 能力 | 负责者 | 其他仓库边界 |
| --- | --- | --- |
| Wall Clock、节拍、暂停和维护 | Server/Client Host | Runtime 只消费 Host Tick 入口。 |
| Logical `TickId`、Phase Graph、Determinism | GameRuntime | Server 不定义阶段语义。 |
| Server ECS World | GameRuntime | Server 只编排，Game 提供 Factory/Hook。 |
| VoxelWorld | VoxelEngine | Host 创建实例，不能复制内部状态。 |
| Client ReplicaWorld | Client + GameRuntime | Server 只保存 Replication Context。 |
| Gameplay 规则与迁移 | Game | Framework 做结构校验和资源限制。 |
| Native 加载与 ABI | CoreEngine | NativeCore/VoxelEngine 提供源契约。 |

## 3. Session、World 与生命周期

### 3.1 物理与逻辑模型

Dedicated Server 中：

```text
Server Process
└─ WorldSlotHost (one GameRelease per process)
   └─ ServerSimulationSession
      ├─ GameWorld (Runtime, authoritative)
      ├─ VoxelWorld (VoxelEngine, authoritative)
      ├─ Coordinator + SnapshotCut
      └─ per-client ReplicationContext

Client Process
└─ ClientReplicaSession (same SessionId, separate process)
   ├─ ReplicaWorld (Runtime)
   ├─ VoxelReplicaWorld (VoxelEngine)
   └─ PredictionHistory + Presentation Adapter
```

LocalEmbedded 只把两棵树放入同一进程，不共享 World、Storage、Entity 或对象引用。

### 3.2 状态机

```text
WorldSlotHost:
Allocated -> Bootstrapping -> NativeReady -> ManagedReady
 -> LoadingSession -> Running <-> Quiescing
 -> Snapshotting / Reloading / Migrating -> Stopping -> Destroyed
Any active state -> Faulted

SimulationSession:
Created -> Initialized -> Ready -> Running <-> Paused
 -> Draining -> Snapshotted -> Disposed
Any active state -> Faulted

ClientReplicaSession:
Disconnected -> Connecting -> Negotiating -> Synchronizing -> Active
Active -> Resyncing -> Active
Active/Resyncing -> Reconnecting -> Synchronizing
Any state -> Closed/Faulted
```

状态迁移只能由所属者发起；Game 的初始化、迁移和销毁回调不能改变 Host 状态机。

### 3.3 创建、暂停、销毁规则

1. Host 校验 Release、ABI、Capability 和资源预算后创建 WorldSlot。
2. Runtime 创建 GameWorld；VoxelEngine 创建 VoxelWorld；两者完成 Ready 后才进入 Running。
3. 暂停/维护先关闭 Ingress，再排空或记录在途事务，固定 SnapshotCut，最后停止 Tick。
4. 销毁顺序为停止新输入、完成/中止事务、导出证据、卸载 Gameplay Scope、释放 Voxel、释放 ECS、关闭 Host。
5. 任一失败都必须进入明确 Faulted 状态，不能留下半初始化对象。

## 4. Tick、调度与确定性

### 4.1 时钟与阶段

Host 决定何时进入一个逻辑 Tick；Runtime 决定 Tick 内部语义。V1 默认单一权威写线程，每个 active WorldSlot 一个 Simulation Owner Thread。

```text
IngressCapture
 -> DecodeAndCanonicalize
 -> ApplyInputs
 -> ProcessorPlan
 -> CrossWorldPrepare
 -> NativeJobBarrier
 -> CommitDecision
 -> VoxelCommit
 -> EcsCommandBufferCommit
 -> GasAndEventFinalize
 -> ReplicationProjection
 -> SnapshotHashMetrics
 -> EgressPublish
```

每个阶段的输入、可写状态、错误处理和可见性必须在 API Contract 中声明。任何 Native Job 结果只能在 `NativeJobBarrier` 或之后应用。

### 4.2 ProcessorDescriptor

所有 Processor 必须声明：

```text
ProcessorId, Role, Phase, Query, ReadSet, WriteSet,
StructuralWrites, Before/After Dependencies,
DeterminismClass, Budget, DiagnosticName
```

Scheduler 先验证读写冲突和依赖图，再决定是否并行。V1 只允许无共享写集且有稳定归并顺序的并行任务；Archetype/Storage 布局不属于公共契约。

### 4.3 队列与迟到输入

- Network、IO、Native Job、平台回调只能写入有界 Queue/Batch。
- Queue 满载按来源和优先级执行丢弃、降级或断开策略，禁止无界增长。
- 输入按 `SessionId + ClientCommandSeq + ArrivalClass` 规范化；迟到输入进入明确的当前 Tick、下一 Tick 或拒绝分支。
- 多 CommandBuffer 按 `Phase + ProcessorId + LocalSequence` 稳定合并。

### 4.4 Determinism 分级

- **Level 1**：同平台、同二进制、同 Release 可位级重放。
- **Level 2**：不同 Host Profile 在同一确定性核心上语义一致，能定位首个差异 Tick。
- 跨 x86/ARM、SIMD/非 SIMD 的位级一致不作为 V1 承诺；跨平台使用固定规则、容差和业务断言。
- 冻结 RNG Seed/Stream、整数/定点边界、浮点舍入规则、时间单位、事件排序和 Canonical Hash。

## 5. World、ECS 与 Entity

### 5.1 状态域

- `GameWorld` 保存 Server Gameplay/ECS/GAS 权威状态。
- `VoxelWorld` 保存 Chunk/Block/Streaming/Revision 权威状态。
- `ReplicaWorld` 和 `VoxelReplicaWorld` 只保存 Client 投影、预测和本地表现所需状态。
- Runtime 不保存完整 Chunk 第二真相；VoxelEngine 不直接修改 ECS。

### 5.2 Entity Identity V1

- `NetEntityId` 是 128 位不透明值，逻辑上包含 `AuthorityDomain`、`WorldEpoch`、`Sequence`、`Generation`；具体位分配由版本化 Schema 冻结。
- 同一 Session 内已销毁 ID 不复用；跨 Session 通过新 Epoch 隔离。
- `LocalEntityId = Index + Generation`，只在一个 ECS World 有效，不能作为网络身份。
- Destroy 产生 Tombstone，至少保留到相关 Baseline Ack 或失效；迟到 Delta 不得复活实体。
- Respawn 默认新建 NetEntityId；Authority Transfer V1 不启用，但 ID/Manifest 预留 AuthorityDomain。
- Client provisional ID 使用独立命名空间，服务器确认包提供重映射；Replay/Migration 保留原 NetEntityId。

### 5.3 非对称 Component 与 Mapping

Mapping 必须声明 Source Entity/Component/Field、Target、Role、Owner、AOI/Visibility、Initial/Continuous、Reliable/Unreliable、Quantization、Predicted/Authoritative、Add/Remove/Tombstone 策略。

Server/Client 可以有不同字段、不同生命周期甚至不同 Entity 子集。生成器产出 Spawn、Despawn、Transfer、Stale-update 和未知字段测试。

### 5.4 CommandBuffer

Processor 只能写自己的 CommandBuffer；结构变化在固定阶段提交。必须定义 Deferred Entity Token、同 Tick Create/Write/Destroy、无效目标、重复命令和稳定合并规则。

## 6. Cross-World Transaction 与 Revision

### 6.1 Revision 模型

```text
SessionRevisionVector {
  TickId,
  GameRevision,
  VoxelWorldRevision,
  ChunkRevisionSet,
  ReplicationRevision,
  ConfigRevision,
  SchemaEpoch
}
```

Game 状态使用 `GameRevision`；Voxel 使用单调 `WorldRevision` 和每 Chunk `ChunkRevision`。所有读取返回读取 Revision；SnapshotCut 固定同一向量。

### 6.2 CrossWorldTxnV1

```text
SessionId
TxnId                  // Session 内唯一，重复调用幂等
TickId
CommandId / PredictionKey
ExpectedGameRevision
ExpectedVoxelRevision / ChunkRevisionSet
DeadlineTick
PreparedGameDelta
PreparedVoxelToken
ResultRevisionVector
```

状态：

```text
Created -> Prepared -> CommitIntent -> Committed
       \-> Aborted
Prepared -> Indeterminate
```

规则：

1. Prepare 只做验证和有租约的 Reservation，不产生可见业务副作用。
2. 所有可能失败的检查、容量检查和 Chunk 可用性检查都在 Prepare 完成。
3. Coordinator 在固定 Barrier 决定 Commit；Apply 必须幂等且不可再次发生业务校验失败。
4. 不在 Rust 锁内调用 C#，不由 Native Worker 回调 Hot Gameplay，不跨 FFI 持有锁。
5. V1 按 `VoxelCommit -> EcsCommandBufferCommit` 顺序 Apply；写入第一个参与者前先持久化 `CommitIntent`，每个参与者完成后追加结果，双方完成后追加 `Committed` 标记。
6. `TxnJournal` 记录 Intent、参与者 Token、每步提交结果和失败原因；结果丢失时用状态查询解决 `Indeterminate`，恢复只重放带提交标记且尚未完成的参与者步骤。
7. 进程恢复从最近协调 Snapshot + Command Log 重放；不引入通用跨进程 Durable 2PC。
8. Duplicate、Timeout、Revision Conflict、Chunk Unloaded、Lost Result、Crash Between Commits 都必须有 Failure Fixture。

## 7. Replication、Prediction 与网络

### 7.1 端到端流程

```text
HandshakeAccepted
 -> FullSnapshot(SnapshotId, TickId, RevisionVector)
 -> BaselineAck
 -> Delta(BaseSnapshotId, FromRevision, ToRevision, Sequence)
 -> DeltaAck / GapDetected
 -> ResyncRequest
 -> FullSnapshot or ResyncPatch
```

Transport ACK 与 Baseline ACK 分开。未知 Baseline、Schema 不匹配、历史窗口不足和 Tombstone 冲突直接进入 Full Resync。

### 7.2 PredictionFrame

Client 收到权威 Delta 后：验证 Baseline/Revision，恢复最近 Confirmed Frame，原子应用 ECS/GAS/Voxel 权威结果，删除已确认命令，按原序重放未确认命令，再输出表现差异。ECS、GAS 和 Voxel Overlay 属于同一确认/回滚单元。

### 7.3 Wire 与 Transport

Envelope 必须包含版本、长度、序号、Session/Release、消息类型、可靠性、完整性校验和 TraceId。必须定义最大长度、分片、重传、反重放、认证和三类错误：可重试、可拒绝、可致命。

LocalEmbedded 必须复用同一 Schema、Serializer、Envelope、权限校验、大小限制和有界队列；可以绕过 Socket/TLS/OS 网络栈，但不能绕过业务协议。Fault Decorator 支持延迟、抖动、丢包、乱序、重复、断线、重连和 QueueFull。

## 8. Native、Managed 与 CoreEngine

### 8.1 NativeManagedAbiV1

- 单一 Root API，例如 `lumio_core_get_api_v1(requested_version, out_table)`。
- API Table 包含 `abi_version`、`struct_size`、`capability_bits`。
- 只跨边界传固定宽度 POD、版本化 Buffer 和不透明 Handle；不传 Rust/C# 容器、对象引用或异常。
- 内存由创建侧释放，优先使用调用方提供 Buffer；Handle 为 Index+Generation+Context。
- Rust 捕获 panic，Managed Entry 捕获 Exception，统一转换为稳定 Error Code。
- Simulation Owner Thread 是唯一 Managed Tick 入口；Native Worker 不回调 Hot Gameplay。
- Managed 调用期间不得持有可能阻塞的 Rust 锁；取消、超时和世界销毁后的异步结果必须有明确定义。

### 8.2 Loader

CoreEngine 统一打包并在一个进程内只加载一套 Native 组合。Loader Registry 拒绝第二版本、符号冲突、ABI/Capability 不匹配和重复释放；平台使用静态或动态链接的方式必须在 Manifest 中唯一声明。

## 9. GAS Framework

Runtime 提供宿主无关的 Ability/Effect/Attribute/Tag 生命周期、Handle、时间、Snapshot/Restore 和 Prediction Context；Game 提供具体类型、Formula、Cost、Cooldown、Targeting 和表现事件；Server 负责权威确认传输；Client 负责预测历史。

V1 必须定义：TypeId/InstanceId/Handle 区别、Ability/Effect 状态机、Stack/Duration/Cancel 顺序、Modifier 求值顺序、PredictionKey、确认/拒绝/回滚窗口、GAS 与 ECS 的单一真相、Snapshot/State Hash 投影。高级 Trigger Graph、Formula VM 和复杂依赖求解器为 P2。

## 10. Host Profile、平台与能力

用户模式与运行能力正交：

```text
RoomMode, DeploymentProfile, ProcessTopology, RoleSet,
TransportProfile, NativeProfile, RenderProfile,
ClockProfile, FaultProfile, PlatformProfile
```

提供命名 Preset：`PureHeadless`、`NativeHeadless`、`LocalEmbedded`、`LocalSplitProcess`、`RemoteDS`、`MobileLocal`。Gameplay 只读取 Role、Capability 和 Port，不读取 `IsOffline`/`IsLocal`。

目标平台为 Linux/Windows Server、Desktop Client、iOS/Android Unity Client。所有 Unity Client 可使用 HybridCLR；HybridCLR 通过 Platform Capability、签名、Hash 和 Release 校验接入。Server 默认 CoreCLR，Server HybridCLR 只是后续兼容性验证，不是 V1 硬依赖。

## 11. 持久化、序列化与配置

### 11.1 Snapshot/WAL

- 使用版本化 Snapshot + WAL/Command Log；本地文件/目录是第一阶段权威存储，对象存储/数据库通过 Adapter 预留。
- Snapshot、Chunk、Gameplay、GAS 和 Config 都有 Magic、SchemaVersion、Length、Hash、Checksum、Compression 和可选加密元数据。
- 写入使用临时文件、校验、fsync/原子替换；保留最近有效 Checkpoint 和失败 Bundle。
- Dedicated Server 默认在权威确认前保证可恢复；Singleplayer 可选择轻量落盘模式，但必须声明丢失边界。
- 支持部分加载、Chunk 流式加载、旧版本 Migrator、引用校验和恢复演练。

### 11.2 Canonical Serialization

Wire、Replay、Snapshot Hash 和迁移输入使用生成的 Canonical Serializer；实体、Component、字段和 Chunk 按稳定顺序编码，不使用运行时对象图或遍历地址。Serializer 必须同时提供 `Encode` 和 `Decode`：反序列化先校验 Magic、SchemaVersion、Length、Compression、Hash/Checksum 和边界，再 materialize typed 状态；同时执行最大消息/分配/解压比限制。版本不兼容、截断、未知必需字段和重复字段都必须拒绝，不得执行输入中的代码。每个 Schema 至少有 round-trip、旧版本读取、损坏输入和未知可选字段 Fixture。JSON/文本导出只用于诊断，不作为高性能权威存储。

### 11.3 配置/配表

人类可读源文件经 Schema 校验、默认值合并和编译，运行时通过版本化 Table Reader 读取 typed binary table；行/键 ID、类型、范围和引用在激活前校验。层级顺序固定为 Engine → Platform → Server → Product → Environment → User/Session；每个 Tick 使用不可变配置快照，配置切换只在 Tick 边界原子生效。开发可热载，生产只能通过带 Hash/签名的版本切换生效；Secret 与普通配表分离。

## 12. 日志、Metrics、Trace 与审计

### 12.1 实现原则

Rust 和 C# 使用各自成熟、经过维护的日志框架，通过 Adapter 统一到 Lumio Event Schema；不自研底层日志库，不把具体供应商 SDK 写入稳定契约。

多线程日志使用有界异步队列和专用 Sink 批量写入。Simulation Thread 不等待 Diagnostic Sink；队列满载按级别、类别和采样策略处理。Audit、Txn Journal 和 Command Log 使用独立的持久化队列，满载时停止新接入或进入维护，不得静默丢失。Error/Fatal 具备同步应急落盘。日志必须支持文件、控制台和可选外部 Sink，包含轮转、保留、脱敏和权限策略。

### 12.2 事件类别

| 类别 | 用途 | 丢失策略 |
| --- | --- | --- |
| Diagnostic Log | 排障和运行状态 | 可采样/按级别丢弃。 |
| Audit Log | 登录、权限、管理和发布审计 | 默认不可静默丢失，需持久化。 |
| Txn Journal | Cross-World 恢复 | 由持久化策略保护，不能依赖 Diagnostic Log。 |
| Command Log | 确定性 Replay/恢复 | 按 Release/Session 策略保存。 |
| Metrics/Trace | 性能和调用链 | 可采样，但保留聚合指标。 |
| Failure Bundle | 失败重建 | 必须可下载、校验和重放。 |

共享字段至少包括 `ProductId、GameReleaseId、ReleasePoolId、MaintenanceId、SessionId、WorldId、WorldSlotId、TickId、TxnId、NetEntityId、PredictionKey、SnapshotId、TraceId、ProducerId、EventSeq`。异步 Sink 不承诺跨线程的实时全局顺序，但必须保留每个 Producer 的 `EventSeq` 和 Tick 关联以便重建；日志不能替代 Txn Journal 或 Command Log。网络时间戳和队列状态进入 Diagnostic Hash，不进入权威 Simulation Hash。

## 13. Release、版本共存与更新

### 13.1 发布身份

`ReleaseManifest` 至少包含 `ProductId、GameReleaseId、ManifestHash、Server/Client Assembly Hash、Gameplay Contract Hash、Runtime API、CoreEngine ABI/Capability、Network/Replication Protocol、Voxel Schema/Migration、Config/Content Hash、Signature、SBOM`。

`ReleaseCatalog` 是签名、版本化的产品/版本/Artifact/Capability/路由清单，路由键至少为 `ProductId + GameReleaseId`，并记录 Pool 状态、Endpoint 和兼容判定。A 1.1 与 BOE 2.1 可以同时在线，但每个进程/Runtime 实例只加载一个 Release；同一 Session 精确匹配并固定 Release。V1 默认不接受跨 Release 连接，N/N-1 窗口只能通过后续 ADR 显式启用。

### 13.2 滚动更新

```text
Published -> Verified -> Warmup -> Serving
Old Serving -> Draining -> Empty -> Retired
任一阶段 -> Rollback / Faulted
```

新 Pool 健康检查通过后接收新 Session；旧 Pool 停止新接入但继续服务已有 Session，直至自然结束、显式迁移或达到维护期限。V1 不要求在线 Session 无感跨 Release 迁移。

### 13.3 强制维护

维护命令必须携带 `ProductId + GameReleaseId + ReleasePoolId` 作用域，并声明 `Graceful` 或 `Forced`；默认只影响目标 Pool，其他产品/Release 不受影响，集群级维护必须显式列出多个 Pool：

- `Graceful`：停止新接入、广播原因和截止时间、排空事务、完成 Snapshot/WAL/Audit 落盘；达到 deadline 后向所有仍在线用户返回稳定的 `MaintenanceKick` 错误并强制断开。
- `Forced`：立即停止新输入和 Tick 提交，先写入维护事件并尽最大努力完成当前 WAL/Failure Bundle，然后向全部连接广播 `MaintenanceKick` 并断开；未完成的命令不得假定已提交，恢复时从最近有效 Snapshot + WAL 重放。

两种模式都必须确保没有连接留在旧实例，再关闭旧实例并启动目标 Release。重连只能被路由到 Catalog 中允许的目标 Release；所有用户断开、未提交事务和恢复动作写入 Audit Log 和 Failure Bundle。

### 13.4 Migration

从不可变 `SnapshotId + SessionRevisionVector` 读取，按声明式 Migration DAG 在 Staging 目录执行 Game/Voxel 转换，做 Schema、引用、资源上限和 Manifest 校验，成功后通过原子版本指针激活。失败不得覆盖旧数据；保留旧 Release、旧 Snapshot 和可重跑证据。Singleplayer 与 DS 同 Release 存档可互转，跨版本必须显式 Migrator。

## 14. 安全、供应链与 P2 预留

### 14.1 开源优先

满足需求时遵循以下决策阶梯：

1. 先采用成熟、活跃维护且许可证可接受的开源框架，保持其标准行为并置于 Adapter 后。
2. 若标准能力不足，优先配置、组合或提交上游修复；只有无法上游化时才在 Adapter 内扩展或维护补丁。
3. 没有合适开源实现时，先建立参考实现和 Benchmark，再实现最小的产品专属代码。
4. 任何自研基础设施都必须在 ADR 中记录候选方案、许可证/供应链评估、维护责任和退出路径。

依赖必须锁定版本/Commit、许可证、SBOM、漏洞扫描、AOT、确定性、性能和平台验证。默认优先 MIT、Apache-2.0、BSD、Zlib 等宽松许可证；GPL/AGPL 等强传染许可证必须单独法务审核。

### 14.2 Mod（P2）

V1 只预留 `ModManifest`、Capability/权限、资源配额、生命周期、Schema、存档和 Release 挂接点。P2 才实现签名审核的 Managed/Data Mod；不得加载 Native DLL、访问裸指针/Socket 或绕过 Runtime Contract。

### 14.3 其他后置能力

跨 Server Sharding、任意 Authority Transfer、复杂 Durable 2PC、生产级无缝跨 Release 迁移、深度 Voxel-aware AOI 和多后端插件系统均为 P2 或更后，不得破坏 V1 单权威边界。

## 15. 工具链、测试与可观测性

架构仓库发布 Schema、ID Registry、Contract Compiler 规范、CLI 结果 Schema 和 ADR。实现阶段可将可执行 Tooling 作为本仓库的版本化包；不得把生成器隐式塞进 CoreEngine 或 Game。

统一命令约定：

```text
lumio test simulation|local|ds|bots|replay|perf
lumio release verify|catalog|rollout|maintenance
lumio save inspect|migrate|verify
lumio config validate|compile|diff
```

### 15.1 Host/Scenario 能力匹配

Scenario 声明 `RequiredCapabilities`，Host 声明 `ProvidedCapabilities`；CLI 只运行匹配组合。ReferenceVoxelPort 让核心体素语义可在 PureHeadless 运行；Native 布局、性能和真实 Streaming 场景至少在 NativeHeadless 验证。

### 15.2 测试类型与故障矩阵

- Golden：Schema、Snapshot、Migration、Replay、Manifest。
- Property：Entity 生命周期、Revision 单调性、Mapping、配置优先级。
- Fuzz：ABI、Envelope、Serializer、Chunk、Migration 输入。
- Stress/Soak：Tick、队列、AOI、热更、内存和多小时运行。
- Differential：ReferenceVoxelPort 与 Native Voxel、Server 与 Replay。
- Fault：网络抖动/丢包/乱序/断线、Txn 冲突/超时/崩溃、Chunk 失败、ABI 错误、ALC 泄漏、OOM、磁盘满、签名错误。

### 15.3 性能基线

固定 Workload、硬件、TickRate、Entity/Chunk/AOI 分布、建造和网络频率、运行时长。测量 1/10/25/50/100/150/200 玩家或 Bot，记录 Tick p50/p95/p99/max、CPU、RSS、GC、Rust Heap、队列、复制字节、重传、FFI Batch 和日志吞吐；100 人是第一阶段目标负载，不是未经测量的容量上限。

## 16. 子模块地图与实现节奏

| 仓库 | 首批子模块 | 后续子模块 |
| --- | --- | --- |
| NativeCore | `abi`、`handle`、`error`、`capability`、`memory`、`job`、`spatial` | SIMD、压缩优化、更多 Kernel。 |
| VoxelEngine | `world`、`chunk`、`revision`、`mutation`、`snapshot`、`streaming` | AOI/Collision 优化、迁移工具。 |
| CoreEngine | `composition`、`root-abi`、`loader`、`manifest`、`signing`、`platform` | 多平台发布和供应链自动化。 |
| GameRuntime | `ecs`、`simulation`、`coordination`、`replication`、`gas`、`persistence`、`config`、`observability`、`hot-reload` | 并行存储、复杂 GAS、性能优化。 |
| Server | `process`、`network`、`auth`、`release-router`、`world-slot`、`pacing`、`maintenance`、`persistence-host` | 多 Slot、RemoteDS、自动扩缩。 |
| Client | `connection`、`handshake`、`replica`、`prediction`、`input`、`unity-adapter`、`hybridclr-adapter`、`bot` | 移动端优化、更多 Renderer。 |
| Game | `server-gameplay`、`client-gameplay`、`mapping`、`gas-content`、`config`、`content`、`scenario`、`migration`、`release` | 复杂玩法、P2 Mod 内容。 |

### 16.1 阶段退出条件

1. **Architecture Gate**：协议、状态机、Schema、RACI、日志/存档/配置/发布边界齐全；每个 P0 有正向和失败 Fixture 设计。
2. **Foundation**：Native ABI、Voxel 单域、Core Loader、Runtime ECS/Tick、Server Host、Client Connection 和 Tooling 骨架可构建。
3. **Vertical Slice**：`PlaceVoxelAbility` 在 Pure/Native Headless、LocalEmbedded、LocalSplitProcess 跑通，验证事务、复制、预测、Replay、存档和配置快照。
4. **Production Hardening**：Release Pool 滚动更新、强制维护、WAL 恢复、Migration DAG、故障矩阵、日志 Soak、RemoteDS 和标准性能曲线通过。
5. **P2 Expansion**：HybridCLR 深度优化、Mod SDK、Sharding、Authority Transfer 等逐项通过独立 ADR 后实现。

## 17. 开发审查清单与变更规则

每个功能必须回答：状态属于哪个 World/Role/仓库？谁创建、Tick、Snapshot、迁移和销毁？是否有 Schema/Manifest/Hash？失败如何分类、恢复和重放？是否经过正确 Host Capability？日志、审计、指标、存档和配置如何关联？是否优先采用 OSS，若自研为何没有合适方案及谁负责维护？

禁止：共享 Server/Client World、直接跨边界 Storage/裸指针、未经契约的反射桥接、Native 回调 Hot Gameplay、无界 Event Bus、第二套 Native 加载、Local 旁路 Gameplay API、把 P2 能力偷偷写成 V1 必须实现的耦合。

任何改变公共状态、字段、错误、时序、ID、版本或依赖图的变更必须：新增/更新 ADR → 更新 Schema/示例和失败语义 → 更新 README → 生成新 Baseline → 同步七仓镜像 → 通过文档和契约检查。
