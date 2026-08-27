# LumioGameEngine V3 最终架构评审合并稿

> **评审日期**：2026-08-27
> **合并来源**：`5.5Max-ReviewV3`、`Fable-5LumioGameEngine_V3_Architecture_Review_2026-08-27.md`、`Pro-LumioGameEngine_V3_Architecture_Review_2026-08-26.md`
> **最终基线**：`LGE-V1.0-2026-08-27`
> **对应规范**：[`LumioGameEngine_Architecture_v1.0.md`](../architecture/LumioGameEngine_Architecture_v1.0.md)

## 1. 评审结论

七仓库的宏观分层、Rust/C# 边界、GameWorld/VoxelWorld 双权威域、Server/Client 非对称 Component、Local 双角色和 Headless/Replay 方向正确，不需要推倒重来。

三份报告共同指出：原 v0.3 是边界草案，不是可并行实现的执行契约。Tick、Revision、Cross-World、Entity 生命周期、Replication、ABI、发布兼容和失败恢复仍停留在名词层；Runtime、Server、Game 对 Session/World/Clock 的表述存在重叠。

本次最终判定分开表达：

- **架构方向**：READY，作为 `v1.0 Implementation Baseline` 进入实现规划。
- **公共契约**：v1.0 已冻结公共语义；架构源仓库已提供第一批状态机约束、Schema、正向/失败 Fixture 和契约校验工具，作为跨仓实现的 Architecture Gate 输入。
- **代码实现**：当前尚未开始；允许独立 Spike 和基础设施骨架，不把 Spike 当作已完成能力。

## 2. 三份报告的共识

### P0：必须先定义的协议

1. Session、World、Role 和 Clock 的唯一所有权。
2. Tick Phase、Processor 调度、CommandBuffer 提交和 Determinism。
3. Cross-World Prepare/Commit、Revision、幂等和崩溃恢复。
4. NetEntityId/LocalEntityId 生命周期、Tombstone、重连和预测重映射。
5. Snapshot/Delta/Ack/Resync、Mapping、Prediction/Correction 的端到端协议。
6. Rust/C# ABI、Handle、Buffer、线程、异常、取消和单加载规则。
7. Generated Contract、ID Namespace、构建 DAG 和 Manifest 兼容判断。
8. 协调 Snapshot Cut、Replay、State Hash 和 Failure Bundle。

### P1：基础实现前必须有边界

- GAS Core 状态模型和 ECS 单一真相。
- LocalEmbedded 的完整序列化保真度和故障注入。
- Server/Client 线程、队列、背压、资源配额和故障域。
- Voxel Revision、Chunk Streaming、AOI/Spatial Projection 边界。
- Hot Reload/ALC/Task/Timer/Native Lease 清理。
- 日志、Metrics、Trace、审计、持久化和配置快照管线。
- Release Catalog、滚动更新、强制维护和 Migration。
- 统一 CLI、Schema/Codegen、CI 和文档同步。

### P2：必须预留、实现后置

- 签名审核的 Managed/Data Mod SDK。
- 跨 Server Sharding、Authority Transfer 和跨服事务。
- 生产级在线跨 Release Session 迁移。
- 深度 Voxel-aware AOI、多后端插件和复杂 GAS Trigger/Formula VM。
- 更深的移动端内存优化和 Server HybridCLR 支持。

## 3. 报告分歧与最终裁决

| 议题 | 报告差异 | 最终裁决 |
| --- | --- | --- |
| 准入状态 | Fable 为 `CONDITIONALLY_READY`，Max/Pro 为 `NOT_READY`。 | 文档基线和 Architecture Gate 产物 READY for implementation planning；代码实现、具体依赖选型和跨仓主干集成仍需按退出条件推进。 |
| Cross-World | 从真正 2PC、Saga 到 Validate-then-Apply 均有讨论。 | 单 Owner Thread + Tick Barrier 的 `CrossWorldTxnV1`，Reservation/幂等/Journaling/状态查询；不引入通用 XA/2PC。 |
| Entity ID | 延迟回收或 Session 内不复用。 | 128 位不透明组合 ID；Session 内不复用，Tombstone、临时预测 ID 和确认重映射。 |
| Tick | 6、8、13 阶段建议不同。 | 采用 v1.0 的 13 个可审计阶段；内部可合并实现，但语义顺序不可变。 |
| Migration | Game→Voxel 固定顺序与 DAG 两种意见。 | 不可变 Snapshot + Staging + 声明式 DAG + 校验 + 原子激活；允许短暂停服，不要求 V1 无缝跨版本。 |
| Tooling | 归 LumioGame、Runtime 或新增仓库。 | 架构源仓库发布规范和 Tooling Contract；实现阶段以独立版本化包交付，不塞进 CoreEngine/Game。 |
| Host Profile | 固定枚举或正交 Capability。 | 正交能力维度 + 命名 Preset。 |
| 文档源 | 七份副本或单一实现仓库。 | 新建 `LumioGameEngineArchitecture` 为唯一源，七仓镜像并做 Baseline Hash 校验。 |

## 4. 重点优化清单

### 4.1 生命周期与时序

- 拆分 `WorldSlotHost`、`SimulationSession` 和 `ClientReplicaSession`。
- Server/Client Host 负责 Wall Clock；Runtime 负责 Logical TickId 和 Phase Graph。
- 明确创建、Ready、Running、Paused、Draining、Snapshotting、Migrating、Faulted、Disposed 状态。
- Snapshot 只能在协调 Barrier 生成；Game/Voxel 必须共享同一 `SessionRevisionVector`。

### 4.2 数据、事务与复制

- GameRevision、VoxelWorldRevision、ChunkRevision、ReplicationRevision 分开定义。
- 所有跨域写入携带 Expected Revision；所有读取返回读取 Revision。
- Replication Mapping 覆盖 Entity/Component/Field/Role/Owner/AOI/可靠性/生命周期。
- Transport ACK 和 Baseline ACK 分开；缺口、未知 Baseline、旧 Revision 进入 Full Resync。
- ECS、GAS、Voxel Overlay 在 Client 的同一 PredictionFrame 中原子确认或回滚。

### 4.3 跨语言与主机

- 用 `NativeManagedAbiV1` API Table、固定宽度 POD、版本化 Buffer、不透明 Handle 和统一 Error Code。
- Rust panic、Managed Exception、Native Job、CoreCLR/ALC 回收和重复 Native 加载都必须有明确故障路径。
- 每个进程只加载一个 CoreEngine/Release；同一集群可运行多个 Release Pool。

### 4.4 运营基础设施

- 多线程异步日志使用成熟 Rust/C# 框架和统一 Event Schema。
- Diagnostic、Audit、Txn Journal、Command Log、Metrics、Trace、Failure Bundle 分离存储和丢失策略。
- Snapshot + WAL/Command Log 支持原子写入、校验、压缩、恢复、部分加载和 Migration。
- 配表由 Schema 编译为 typed binary；Tick 使用不可变配置快照，生产显式版本切换。
- Release Catalog 支持多产品、多版本路由、滚动更新、Session 排空和强制维护。

## 5. 最小验证切片

使用 `PlaceVoxelAbility` 作为第一条端到端切片：客户端预测一个放置请求，服务器验证资源/权限/Chunk/Revision，Coordinator 同 Tick 提交资源和体素变更，产生 Replication Delta、Audit/Txn/Command 记录，并支持重复命令、Revision 冲突、Chunk 未加载、丢包、断线重连、Replay、Save/Load 和 Release 不匹配。

同一 Scenario 至少运行于 Reference/PureHeadless、NativeHeadless、LocalEmbedded 和 LocalSplitProcess；RemoteDS、移动端和滚动更新作为后续验收面。

## 6. 进入实现的退出条件

1. v1.0 规范中的每个 P0 都有状态机或 Schema、正向 Fixture 和失败 Fixture。
2. 七仓 README 的职责、子模块、公共契约、日志、持久化、配置、版本和阶段路线无冲突。
3. ReleaseManifest/Catalog、Snapshot/WAL、日志事件和 Failure Bundle 的关联字段已统一。
4. 同一基线镜像通过 `BaselineId + ContentHash` 检查。
5. 依赖的开源方案通过许可证、SBOM、漏洞、AOT、确定性和性能门槛。

当前已完成第 1、2、3、4 项的文档/契约门槛输入：唯一架构源已提交 19 个版本化 Schema（16 个 P0、2 个 P1、1 个 P2 预留）、ID Registry、43 个正向/失败 Fixture、`tools/lumio_contract.py` 和 16 份 ADR 草案。第 5 项以及各仓库的代码实现必须在 Foundation 阶段实际验证，不能由文档样例代替；临时默认值集中记录在 `DECISIONS_PENDING.md`。

## 7. 实现前需确认的非语义选择

以下选择默认不改变 v1.0 的所有权、状态机和兼容边界，但会影响 Foundation 的具体实现；若选择改变 Wire/Schema/兼容语义，就必须升级 ADR 和 Baseline。默认按 OSS-first 逐项验证，并在对应 ADR/Manifest 中记录：

1. 生产 Transport（TCP/QUIC/UDP 组合）、Wire Codec 和压缩库。
2. Rust/C# 日志框架、外部 Sink、审计持久化介质和目标保留时长。
3. Snapshot/WAL 的耐久级别（同步、Group Commit 或可声明的异步模式）及磁盘/对象存储部署。
4. 移动端 `MobileLocal` 的内存、启动时间和热更装载预算；未通过 Spike 前不提升为默认能力。
5. 是否在未来通过 ADR 开放 N/N-1 Release 兼容；V1 当前保持精确匹配和强制更新路径。
6. 是否允许同一进程加载多个 `ProductId/GameReleaseId`；V1 默认一个进程一个 Release，通过多个进程/Pool 支持 A 1.1 与 BOE 2.1 并行。

## 8. 说明

`5.5Max-ReviewV3` 在当前工作区末尾中断于“ADR-003 Cross-World”；本合并稿已使用该文件可见主体，并以 Fable/Pro 的完整协议和 ADR 清单补足缺失部分，不把未知尾部当作额外事实。
