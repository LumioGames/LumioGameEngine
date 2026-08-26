# LumioGameEngine V3 (v0.3) 架构与开发规范

> 状态：V3 架构基线草案（内部版本号 v0.3），作为所有实现仓库的共同契约。本文描述边界和行为，不替代具体 API 设计。

## 1. 目标与原则

LumioGameEngine 的首要目标是让开发者在最小环境中重复测试 bug、性能、表现和玩法行为，并能把同一场景提升到 Local、Dedicated Server、Bot、Replay 和 CI，而不重写 Gameplay 代码。

### 1.1 不可违背的原则

1. **越底层越通用。** Native Kernel 只做跨项目通用能力；领域、Runtime、宿主和具体玩法依次向上收敛。
2. **可测试性是一等能力。** 每个模块都有 Headless 入口、确定性时钟、故障注入和可导出的失败证据。
3. **World 按业务域分开。** `GameWorld` 与 `VoxelWorld` 各自拥有状态、Entity/Component 数据和生命周期；跨域通过 Port 与 Coordinator 协作。
4. **Server/Client 角色分开。** 两端各自创建本地 ECS Entity，Component 可以不对称；跨端身份依赖 `NetEntityId`，本地存储依赖 `LocalEntityId`。
5. **兼容性由玩法作者负责。** Framework 只负责版本、结构安全、Manifest、启动校验和 Migration Hook；玩法语义是否兼容由 `LumioGame` 决定。
6. **运行模式由 Host 决定。** Gameplay 不写 `if (IsOffline)`、`if (IsLocal)` 等模式分支；它只依赖 Role、Command、Event、Port 和 Capability。
7. **一次发布同步更新。** Server 与 Client 的 Gameplay Assembly、Schema、配置和内容属于同一 `GameReleaseId`，必须一起验证和发布。

## 2. 仓库拓扑与所有权

```text
LumioNativeCore
└─ 跨项目通用 Rust Native Kernel / HPC

LumioVoxelEngine
└─ 可复用 VoxelWorld / Chunk / Revision 领域实现

LumioCoreEngine
└─ NativeCore + VoxelEngine 聚合、统一 C ABI、平台产物、Manifest/Hash

LumioGameRuntime
└─ ECS Runtime、GameWorld 生命周期、GAS Framework、Processor、Coordinator、Hot Reload Host

LumioServer
└─ Rust DS、网络、Connection/Session、WorldSlot、CoreCLR Hosting、升级编排

LumioClient
└─ Client Replica、Prediction/Correction/Rollback、平台 Host、Headless Client/Bot

LumioGame
└─ Server/Client Component、RPC、Gameplay、GAS Content、Config、Content、Migration、Scenario/Test
```

### 2.1 实现语言边界

- `LumioNativeCore`、`LumioVoxelEngine` 和 `LumioCoreEngine` 的 Native/高性能模块统一使用 Rust。
- `LumioServer` 的 Dedicated Server Host、网络、进程治理和 CoreCLR Hosting 使用 Rust；它加载托管层但不把 Gameplay 编译进 Rust Host。
- `LumioGameRuntime` 的稳定 ECS、GAS Framework、Processor 和 Hot Reload Host 使用 C#。
- `LumioGame` 的 Server/Client Gameplay Assembly、GAS Content、Component、Processor、Migration 和测试场景使用 C#，通过 Runtime 热更加载。
- `LumioClient` 的通用连接/Replica Host 可以按平台实现，但 Client Gameplay 热更代码仍是 C#；平台 Renderer/Adapter 不改变该边界。

Rust 负责可复用的底层和高性能数据处理，C# 负责可热更的托管运行时与具体玩法。两者只能通过版本化 C ABI、Managed Adapter 和 Generated Contract 交互；禁止以脚本语言或未经契约的反射桥接替代这条边界。

### 2.2 依赖规则

```text
NativeCore ──> VoxelEngine ──┐
                             ├─> CoreEngine package ──> Server / Client
GameRuntime ─────────────────┘                  └─────> Game composition
Server Host ──> GameRuntime
Client Host ──> GameRuntime
Game Content ──> Runtime + Host contracts
```

- `LumioCoreEngine` 是发布聚合仓库，不是 Voxel 领域实现仓库。
- Server 与 Client 只加载一个 `LumioCoreEngine` 平台包，不直接分别装载 NativeCore/VoxelEngine。
- `LumioGameRuntime` 只通过 `IVoxelWorldPort` 和 Generated Contract 使用 Voxel 能力，不依赖 VoxelEngine 内部实现。
- 上层仓库不得反向成为底层的源码依赖。

## 3. Session、World 与状态所有权

运行时的最小隔离单元是 `WorldSlot`。一个房间进入后创建一个 `SimulationSession`：

```text
WorldSlot
└─ SimulationSession
   ├─ Server GameWorld (GameRuntime)
   ├─ Authoritative VoxelWorld (VoxelEngine)
   ├─ Client ReplicaWorld (GameRuntime, 每个 Client 一份)
   ├─ Client VoxelReplicaWorld (VoxelEngine, 每个 Client 一份)
   ├─ Cross-World Coordinator (GameRuntime)
   ├─ Replication Context
   └─ Session Snapshot / Replay Metadata
```

### 3.1 GameWorld

`GameWorld` 保存 C# ECS/Gameplay 权威状态，包括具体游戏的 Server Component、GAS Content 状态、玩家会话映射和可复制投影。只有 Server Role 能提交权威 Gameplay 结果。

### 3.2 VoxelWorld

`VoxelWorld` 保存 Rust Voxel/Chunk/Revision/Mutation 权威状态。它负责 Chunk 数据、Streaming、Snapshot/Diff、Mesh Source、Collision Source 和体素空间查询。C# ECS 不保存完整 Chunk 作为第二真相来源。

### 3.3 跨 World 协调

例如建造玩法需要同时修改玩家资源和体素块时：

1. Gameplay Processor 通过 `IVoxelWorldPort` 发出带 `TickId` 的 Prepare 请求。
2. Coordinator 收集 GameWorld 资源校验和 VoxelWorld Mutation 结果。
3. 两边均成功时在同一个 Tick 的 Commit 阶段提交；任一失败则返回可记录、可重试的失败结果。
4. Snapshot、Replay 和 Metrics 记录请求、Revision、Commit 结果和失败原因。

跨 World 禁止直接访问对方 Storage、指针或句柄。体素感知的 AOI、Streaming、Collision 优化可以使用 Voxel 数据，但必须位于 VoxelEngine/Coordinator 边界，不把 Voxel 语义下沉到 NativeCore。

## 4. ECS、Entity 与 Processor

### 4.1 双层身份

```text
NetEntityId   跨 Server/Client、Snapshot、Replay 稳定的逻辑身份
LocalEntityId 当前 ECS World 内部的存储身份
```

Server、每个 Client 和 LocalEmbedded 的 Server/Client Role 都创建自己的 `LocalEntityId`。`NetEntityId` 由权威侧分配或由生成契约定义映射；不得把网络 ID 当成数组下标或直接复用为 Local ID。

### 4.2 非对称 Component

`LumioGame` 为同一 `NetEntityId` 分别声明：

```text
Server Component: HealthAuthority, InventoryState, BuildPermission
Client Component:  HealthReplica, HealthPresentation, PredictedInput
```

两侧可以字段、生命周期、甚至 Entity 子集都不同。`Replication Mapping` 明确哪些 Server 字段投影到哪些 Client Component，哪些字段只在本地存在。Runtime 只提供存储、查询、变更追踪和 Mapping 接口，不强迫名称或结构对称。

### 4.3 执行模型

- `Processor/Handler` 是主要执行抽象，按 Query、Role、Phase 和读取/写入集合注册。
- `System` 不是必需概念；仅在需要传统调度注册语义时作为 Processor 的一种实现。
- 所有结构变更通过 CommandBuffer，在固定 Tick 阶段统一提交。
- 网络线程、Native Job 和平台回调只能向 Typed Queue/Channel 写入数据；Gameplay Processor 在确定性 Tick 中消费。

## 5. GAS Framework 与 Gameplay Content

### 5.1 Runtime 所有的 GAS Framework

`LumioGameRuntime` 提供跨游戏可复用的：

- Ability、Effect、Attribute、Tag 的通用数据模型和状态机。
- Handle、生命周期、激活/取消/结束、Stack、持续时间和依赖管理。
- Server/Client Role、Authority、Prediction、Correction、Rollback 和 Snapshot/Replication 接口。
- Tick、Determinism Context、Typed Event Channel、错误和能力校验。

GAS Framework 只表达 Gameplay 事件与状态语义。它不依赖 Socket，不解析原始网络字节；Server/Client Host 负责网络传输和 Envelope。

### 5.2 Game 所有的 GAS Content

`LumioGame` 提供具体的：

- Ability、Buff、Effect、AttributeSet、Tag 内容和资源引用。
- Formula、Targeting、资源消耗、Cooldown、权限和玩法触发条件。
- Server/Client 入口 Processor、表现事件、音效/动画桥接和测试 Fixture。

内容必须通过 Runtime GAS API 注册，不能把产品语义塞回 Runtime。

## 6. 运行模式与 Host Profile

玩家可见的模式只有：

```text
Online       连接某个 Dedicated Server
Singleplayer 同进程运行 Server Role + Client Role
```

`Online` 的 Endpoint 可以是公共 DS、玩家启动的独立 DS 或本机 localhost DS。这三者都是独立 Dedicated Server，只由 Endpoint/发现方式区分，不改变 Gameplay。

### 6.1 Host Profile

| RoomMode | Host Profile | 进程与 World |
| --- | --- | --- |
| `Online` | `PublicDedicatedServer` | Client 进程连接公共 DS；两端各自 World。 |
| `Online` | `PlayerHostedDedicatedServer` | 玩家启动独立 DS 进程后连接；两端各自 World。 |
| `Online` | `LocalhostDedicatedServer` | 本机独立 DS 进程；两端各自 World。 |
| `Singleplayer` | `LocalEmbedded` | 同一进程内 Server Role + Client Role + 两套 ECS/体素 World。 |

第一阶段移动端支持 `LocalEmbedded` 和加入远程 DS，不负责启动 Player-hosted DS。移动端 Local 也使用完整双角色，后续只在 Host/Runtime 层做性能优化。

### 6.2 LocalEmbedded 的 Entity 规则

LocalEmbedded 不是把两端合并成一个 World：

```text
同一进程
├─ Server Role
│  ├─ Server ECS World + Server LocalEntityId
│  └─ Authoritative VoxelWorld
├─ Client Role
│  ├─ Client ECS World + Client LocalEntityId
│  └─ VoxelReplicaWorld
└─ InMemoryTransport (仍经过消息/快照边界)
```

同一个逻辑对象通常拥有一个 `NetEntityId`，但在 Server 和 Client World 中分别有不同 `LocalEntityId` 和 Component 集合。这样 Local 测试可以覆盖真实 DS 的权限、复制、预测和校正路径。

## 7. 更新、热更与兼容性

### 7.1 开发期热更

- Rust Host、Network、CoreEngine 和 VoxelWorld 保持运行。
- Server Gameplay Assembly 与 Client Gameplay Assembly 同时重载，并以同一 `GameReleaseId` 进行校验。
- Component/RPC/Schema 结构变化时重建托管 World、重新注册 Mapping 和 Processor。
- 默认重置开发态 GameWorld/Client Replica；无需让旧状态兼容。
- 若测试迁移本身，则显式运行 Migration Fixture，不把开发热更误当成生产无缝升级。

### 7.2 生产更新

1. Server 停止接入并冻结 Tick，导出 Session Snapshot。
2. `LumioGame` 执行 GameWorld Migration；`LumioVoxelEngine` 执行 VoxelWorld/Chunk Migration。
3. Server 校验新 Manifest、ABI、Schema、Capability、Migration 结果和签名。
4. Server/Client 使用同一 `GameReleaseId` 的新包启动；校验失败则按编排策略回滚或保持停服。

玩法作者决定旧数据是否语义兼容并提供 Migration；框架不推断 Ability、经济、任务等业务语义。

## 8. 测试架构

### 8.1 统一 CLI

```text
lumio test
lumio test simulation
lumio test local
lumio test ds
lumio test bots
lumio test replay
lumio test perf
```

CLI 负责选择仓库原生测试、Host Profile、数据集、随机种子、Tick 上限、故障注入和产物目录。仓库可以提供更细的原生命令，但结果格式保持统一。

### 8.2 Host Test Matrix

| Host | 主要用途 |
| --- | --- |
| `PureHeadless` | 不加载 Native/Renderer，快速验证 Gameplay、ECS、GAS 和确定性。 |
| `NativeHeadless` | 加载 CoreEngine，验证 Native/Voxel 绑定、数据和性能。 |
| `LocalEmbedded` | 同进程双角色，验证真实消息边界、复制、预测和校正。 |
| `LocalSplitProcess` | 独立 Server/Client 进程，验证启动、连接、资源和端口。 |
| `RemoteDS` | 公共/玩家 DS 环境的网络、重连和部署回归。 |
| `MobileLocal` | 移动端双角色资源预算、输入和表现边界。 |

同一个 Scenario 必须只通过 Host 配置切换这些环境，不复制一套 Offline Gameplay。

### 8.3 Scenario、Bot 与 Replay

- C# 编写 Scenario 初始状态、输入命令、Bot 行为、断言和结束条件。
- Scenario 可被 Pure Headless、Local、DS、Bot 和 CI 复用；测试数据与执行逻辑分开。
- 失败时导出 `Command Stream`、关键 Snapshot、`State Hash`、Metrics、结构化日志、Manifest 和随机种子。
- Replay 必须能在不同 Host 重放并指出第一个 Tick、World、Entity、Component 或 Revision 差异。

### 8.4 性能基线

第一阶段以约 100 名真实玩家规模建立基线，Bot 数量可配置扩展。每次基准至少记录：

- 玩家数、Bot 数、Entity/Component 数、Voxel Chunk 数、AOI 半径和 Streaming 队列。
- Server/Client Tick、Processor、GAS、Voxel Query、Replication、网络吞吐和 p95/p99 延迟。
- CPU、峰值内存、分配次数、Native Job 队列、包大小、丢包/重传和帧时间。
- Commit、Compiler、平台、随机种子、GameReleaseId、CoreEngine Hash 和数据集版本。

性能优化可以利用体素数据（例如体数、Chunk 密度和空间分布）来调节 AOI/加载策略，但优化边界必须记录在 Voxel/Coordinator 契约中。

## 9. 契约、Manifest 与发布

### 9.1 契约来源

- NativeCore：通用 ID、Handle、Error、Capability、ABI。
- VoxelEngine：Voxel Handle、Chunk、Mutation、Revision、Snapshot/Diff、Migration。
- GameRuntime：ECS、Role、Processor、GAS、Snapshot、Typed Channel、Hot Reload。
- Server/Client：Connection/Session、RPC Envelope、Endpoint、Host Adapter。
- Game：Server/Client Component、RPC Payload、Replication Mapping、GAS Content、Config 和 Migration。

契约生成器输出 C# 类型、序列化器、RPC ID、Schema、绑定和校验代码。所有生成物必须可从干净来源重建；禁止手写重复的布局、MessageId 或 Native Handle。

### 9.2 Manifest 最低字段

```text
GameReleaseId
Runtime version / Commit / API schema
CoreEngine version / Commit / Artifact Hash / ABI / Capability
Server Host version / network protocol
Client Host version / platform / adapter
Gameplay Server Assembly + Client Assembly Hash
Generated Contract Hash
Config / Content Hash
Game Migration / Voxel Migration version
Signature / SBOM / compatibility matrix
```

Host 启动和握手时校验这些字段；不匹配时拒绝加载或进入房间。Server 与 Client 的更新由同一 Game Release 编排，玩法作者在发布前决定语义兼容范围。

## 10. 开发规范与审查清单

提交新功能前至少回答：

- 状态属于哪个 World、哪个 Role、哪个仓库？谁创建、Tick、Snapshot、Migration 和销毁？
- Server/Client 是否需要不同 Component？`NetEntityId` 到各自 `LocalEntityId` 的 Mapping 是否明确？
- 是否通过 Processor/Typed Channel/Port 协作，而不是直接跨边界访问 Storage、Socket 或裸指针？
- LocalEmbedded 是否仍走双角色、双 World 和 InMemoryTransport？是否有对应 Headless Scenario？
- Voxel-aware 优化是否使用了正确的 Voxel 数据边界，且没有把玩法语义下沉到 NativeCore？
- 是否补齐 Schema、Manifest、Hash、Migration、Replay、Metrics 和故障路径？
- 是否在 Pure Headless、Native Headless、Local 和 DS 至少各运行一次同一 Scenario？

禁止事项：

- 禁止重新引入已废弃的旧控制平面或依赖其运行时。
- 禁止在 Local 模式共享 Server/Client ECS World 或直接调用对方 Processor。
- 禁止把 GAS Content 放入 `LumioGameRuntime`，也禁止让 GAS Framework 直接绑定网络字节。
- 禁止让 Client 作为权威状态来源，或把 Client Component 当作 Server Schema 的镜像要求。
- 禁止绕过 `LumioCoreEngine` 直接加载第二套 NativeCore/VoxelEngine 产物。

## 11. v0.3 实施顺序

1. 冻结各仓库 README、契约所有权、依赖图和 Manifest 字段。
2. 建立 CoreEngine 统一 Native 包、Loader、ABI Smoke Test 和 NativeHeadless。
3. 建立 Runtime ECS、双层 Entity、Processor、GAS Framework、Coordinator 和 LocalEmbedded 双 World。
4. 建立 Server/Client Headless Host、InMemoryTransport、Replica/Prediction/Rollback 和 DS 握手。
5. 在 Game 中加入不对称 Component、GAS Content、Voxel Port 场景、Migration、Replay 和 100 玩家性能基线。
6. 将同一 Scenario 接入 CI、Split Process、Remote DS 和 MobileLocal，并把失败证据纳入回归资产。

完成上述闭环后，再根据测量结果决定更深层的 AOI、Streaming、内存布局和移动端性能优化；优化不能反向破坏 World 所有权和测试入口。
