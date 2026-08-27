# LumioClient 公共契约确认 — `LGE-V1.2-2026-08-27`

给 LumioClient 的同步说明。裁决对象是其设计文档「待上游契约确认」8 项与 ADR 0002 的开工门禁。公共语义只在本仓定义；下文提到的 client 模块名仅用于对照其内部 ADR，不是公共契约的一部分。

## 新 Baseline

- **ArchitectureBaselineId**：`LGE-V1.2-2026-08-27`
- **规范正文**：`docs/architecture/LumioGameEngine_Architecture_v1.2.md`
- **新增 ADR**：021 / 022 / 023
- **新增 Schema**：`client-authority-update`、`protocol-permission-gate`、`generated-contract-artifact`
- **新增 ErrorCode**：`MessagePermissionDenied` (1031)、`StaleConnectionGeneration` (1032)

## 8 项裁决

| # | 结论 | 落点 | 一句话理由 |
| --- | --- | --- | --- |
| 1 | **新增** | ADR-021，§7.2，`client-authority-update.schema.json`，Fixture `authority/committed` / `authority/visible-on-abort` | 步骤序已在 v1.1 §7.2，但缺少单一事务 API、提交可见性与不可知结果的 FaultClass。 |
| 2 | **新增** | ADR-022，§7.3，`protocol-permission-gate.schema.json`，Fixture `gate/accept` / `gate/stale-generation` | 由本仓工具链生成 Validator；字段集冻结为 Session/Release/MessageId/Role/Claims/Connection Generation；会话级反重放归 `ClientReplicaSession` 所有者。不冻结 D-009。 |
| 3 | **新增** | ADR-023，§11.2，`generated-contract-artifact.schema.json`，Fixture `gencfg/validator` / `gencfg/client-implementation-dep` | 本仓工具链唯一发布纯生成物；零依赖于 LumioClient/LumioGame 实现工程；双方与 Runtime/Server 可引用。 |
| 4 | **已存在** | ADR-010，§11.3，`config-table.schema.json`（`configRevision`、`activation`） | Config Port 已定义 typed materialization、staging/Active 快照、`ConfigRevision` 与 Tick Barrier 原子激活；请求时机属宿主编排，不进公共契约。 |
| 5 | **已存在** | ADR-001，§3.1 / §3.3 | 两个独立 Handle：`ReplicaWorld` 与 `VoxelReplicaWorld`；逆序销毁先 Voxel 后 ECS；权威更新事务跨越二者，不是合成单一 Handle。 |
| 6 | **拒绝** | §7.3，D-012 | V1 不提供 Session Resume Token；新连接代次一律重做通道认证与完整 Handshake。替代边界即现有 Handshake/Resync 分流。 |
| 7 | **拒绝** | §13.1，D-007，ADR-014 | Active Session 精确绑定 `GameReleaseId`，禁止跨 Release 替换 Gameplay Scope。同 Release 热更失败回滚不是 Release 切换。 |
| 8 | **已存在** | ADR-010 / ADR-011，§11.2 / §12.2 | 规范格式归本仓 Canonical Serializer；权威 Command Log 持久化归 Host 持久化；Client Observability 只导出同格式证据，不是状态真相。 |

## 其「待上游契约确认」可关闭项

1、2、3、4、5、6、7、8 **全部可关闭**。1–3 以本 Baseline 的新契约为准；4、5、8 指向既有章节；6、7 以拒绝 + 既有替代边界关闭。

## 与 LumioClient ADR 0002 的对照

不改写 0002；仅指出是否需要它新增 ADR 取代条款。

| 0002 条款 | 与本裁决 | 是否要它新增 ADR |
| --- | --- | --- |
| D1 单一 Runtime 权威更新事务 | 已由 ADR-021 确认，步骤序与「失败不推进 Baseline/Ack」一致 | 否。可关闭待确认，按 0002 实现。 |
| D2 生成 Validator 与字段集 | ADR-022 确认生成门与字段；V1 的 `MessageId` = `MessageType` 命名空间 | 否。不要把 D-009 RPC 分发写进实现。 |
| D3 重连重做 Handshake；无 Token 前不复用认证 | 与拒绝项 6 一致 | 否。 |
| D3 Resync 不重新握手 | 与 §3.2 / §7.3 一致 | 否。 |
| D4 Config 由 Runtime Port materialize/原子切换，session 只请求时机 | 与已存在项 4 一致；公共契约不承认 `session` 模块名 | 否。内部编排可保留。 |
| D4 两个 Runtime Handle 与逆序销毁 | 与已存在项 5 一致 | 否。 |
| D5 生成 Artifact 零依赖 Client/Game 实现 | 已由 ADR-023 确认 | 否。 |
| 「一个 Session 的 Gameplay Scope 固定」 | 与拒绝项 7 一致 | 否。 |
| Replay/Command Stream 作可观测证据 | 与已存在项 8 一致 | 否。 |

**结论：ADR 0002 无需被取代。** 它冻结的是 Client 内部角色；本 Baseline 补的是它声明「不得自行发明」的公共语义。C# 开工门禁在同步本 Baseline 后可以解除（仍须它自己的工程引用图 CI）。

## 七仓同步范围

LumioClient（关闭待确认并引用新 Schema）、LumioGameRuntime（实现权威更新事务与 Config Port）、LumioGame（只引用生成 Artifact，不把实现工程暴露给 Client）、LumioServer（同一 Validator 字段集）。其余仓只读镜像 BaselineId/Hash。
