# LumioGameEngine V1 ADR（唯一权威索引）

本目录是全仓公共架构与框架决策的唯一落点。`docs/adr` 只保留兼容入口与指向本目录的文件级软链接；`docs/architecture/ADR_INDEX.md` 只保留入口指针。

ADR-001 至 ADR-020 已随 `LGE-V1.1-2026-08-27` 转为 `Accepted`；ADR-021 至 ADR-023 随 `LGE-V1.2-2026-08-27` 接受。`Accepted` 后不可改写，只能由新 ADR 取代并在双方记录取代关系。`ADR-015` 保持 `Reserved`，因为 Mod 是 P2。尚未批准的实现选型和运维默认值不进入 ADR，统一记录在 [`DECISIONS_PENDING.md`](../../docs/architecture/DECISIONS_PENDING.md)。

## 写作与变更契约

- 一个决策一个 `ADR-NNN-<slug>.md`，沿用现有三位编号并递增；无 frontmatter。
- 正文必须包含背景、决策、替代方案、接口/Schema、失败语义、兼容影响、迁移方案和验证 Fixture。
- 公共语义变化同步更新 Schema/ID、正向与失败 Fixture、架构正文、README、BaselineId/Hash 和七仓镜像。
- `Accepted`/`Superseded` 历史不改写；取代时新增 ADR，并在旧 ADR 状态与本索引中记录关系。

## 索引

| 编号 | 主题 | 状态 |
| --- | --- | --- |
| [ADR-001](ADR-001-session-lifecycle.md) | Session Ownership、World Lifecycle、Clock Split | Accepted |
| [ADR-002](ADR-002-tick-determinism.md) | Tick Phase、Processor Scheduling、Determinism | Accepted |
| [ADR-003](ADR-003-cross-world-txn.md) | CrossWorldTxnV1、Revision 和 SnapshotCut | Accepted |
| [ADR-004](ADR-004-entity-identity.md) | Entity Identity、Tombstone、Ownership Revision | Accepted |
| [ADR-005](ADR-005-replication-prediction.md) | Replication Baseline、Prediction、Resync | Accepted |
| [ADR-006](ADR-006-native-managed-abi.md) | NativeManagedAbiV1、Loader 和 Fault Domain | Accepted |
| [ADR-007](ADR-007-contract-toolchain.md) | Contract Toolchain、ID Namespace、Dependency DAG | Accepted |
| [ADR-008](ADR-008-gas-state.md) | GAS Core State Model | Accepted |
| [ADR-009](ADR-009-local-transport.md) | Local Transport Fidelity 和 Fault Injection | Accepted |
| [ADR-010](ADR-010-persistence-config.md) | Persistence、Serialization、Config Snapshot | Accepted |
| [ADR-011](ADR-011-observability.md) | Logging、Metrics、Trace、Audit 和 Failure Bundle | Accepted |
| [ADR-012](ADR-012-release-update-maintenance.md) | Release Catalog、Rolling Update、Forced Maintenance | Accepted |
| [ADR-013](ADR-013-migration-dag.md) | Migration DAG、Staging、Atomic Activation | Accepted |
| [ADR-014](ADR-014-platform-capability.md) | Unity/HybridCLR Platform Capability | Accepted |
| [ADR-015](ADR-015-mod-extension-boundary.md) | P2 Mod SDK Extension Boundary | Reserved |
| [ADR-016](ADR-016-benchmark-workload.md) | Benchmark Workload、TickBudget、Hardware Profile | Accepted |
| [ADR-017](ADR-017-root-abi-generatable-contract.md) | Root ABI 可生成契约粒度（Slot/签名/Handle/Buffer/ErrorDetail） | Accepted |
| [ADR-018](ADR-018-coreengine-manifest-canonicalization.md) | CoreEngineManifestBody 规范化与分离式 SignatureEnvelope | Accepted |
| [ADR-019](ADR-019-loader-state-machine-package-identity.md) | Loader 状态机、PackageIdentity 与单进程锁定 | Accepted |
| [ADR-020](ADR-020-target-profile-orthogonalization.md) | TargetProfile / PackagingProfile / LoadBackend 正交化 | Accepted |
| [ADR-021](ADR-021-client-authority-update.md) | 客户端权威更新单一 Runtime 事务 | Accepted |
| [ADR-022](ADR-022-protocol-permission-gate.md) | 生成的 Protocol/Permission 门与字段集 | Accepted |
| [ADR-023](ADR-023-generated-contract-artifact.md) | 生成契约 Artifact 发布方与零实现依赖 | Accepted |

尚未确认的实现选型和运维默认值见 [`DECISIONS_PENDING.md`](../../docs/architecture/DECISIONS_PENDING.md)；确认前只能使用其中明确标注的临时默认值。
