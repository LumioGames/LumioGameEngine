# LumioGameEngine V1 ADR（唯一权威索引）

本目录是全仓公共架构与框架决策的唯一落点，也是全仓唯一文档根 `.spec/` 的一部分。

本目录分两段历史，读之前先认这条线：

- **ADR-001 ~ ADR-048（`Historical`）**——全部产生于 2026-08-31 之前的「架构源 + Baseline 门禁 + 七仓镜像」旧制度，其赖以成立的 `schemas/` / `fixtures/` / `ids/` / `packages/` 与 Baseline 校验链已随制度废止一并删除（见 git 历史）。它们的**技术结论仍可被新 ADR 引用**，但「`Accepted` 后不可改写」的约束随旧制度一并失效：允许就地修正失效的引用路径，不必为一处路径新增一号。状态列写作 `Historical · <原状态>`，原状态保留可读。
- **ADR-049 起（在册）**——Living Architecture 制度下的决策。ADR-049 于 2026-09-01 在新制度下重新 `Accepted`（明确为 pre-launch wire 契约、非 baseline event），因此不属旧制度、不打 `Historical`。此段的 `Accepted` 不可改写规则正常生效。

`ADR-015` 保持 `Reserved`（Mod 是 P2）。NativeCore 内部实现决策在 [`nativecore/`](nativecore/README.md) 子命名空间，另起一套 `000N` 编号，与本表互不占号。

## 写作与变更契约

- 一个决策一个 `ADR-NNN-<slug>.md`，沿用现有三位编号并递增；无 frontmatter。
- 正文必须包含背景、决策、替代方案、接口/Schema、失败语义、兼容影响、迁移方案和验证 Fixture。
- 跨边界语义变化同步更新 [`knowledge/features/architecture.md`](../knowledge/features/architecture.md) 与受影响的接口定义（`engine/abi/`、`engine/wire/`）；Living Architecture 不再维护 BaselineId、Schema/Fixture 全量门与七仓镜像。
- ADR-049 起，`Accepted`/`Superseded` 历史不改写；取代时新增 ADR，并在旧 ADR 状态与本索引中记录关系。`Historical` 段不受此约束（见上）。

## 索引

| 编号 | 主题 | 状态 |
| --- | --- | --- |
| [ADR-001](ADR-001-session-lifecycle.md) | Session Ownership、World Lifecycle、Clock Split | Historical · Accepted |
| [ADR-002](ADR-002-tick-determinism.md) | Tick Phase、Processor Scheduling、Determinism | Historical · Accepted |
| [ADR-003](ADR-003-cross-world-txn.md) | CrossWorldTxnV1、Revision 和 SnapshotCut | Historical · Accepted |
| [ADR-004](ADR-004-entity-identity.md) | Entity Identity、Tombstone、Ownership Revision | Historical · Accepted |
| [ADR-005](ADR-005-replication-prediction.md) | Replication Baseline、Prediction、Resync | Historical · Accepted |
| [ADR-006](ADR-006-native-managed-abi.md) | NativeManagedAbiV1、Loader 和 Fault Domain | Historical · Accepted |
| [ADR-007](ADR-007-contract-toolchain.md) | Contract Toolchain、ID Namespace、Dependency DAG | Historical · Accepted |
| [ADR-008](ADR-008-gas-state.md) | GAS Core State Model | Historical · Accepted |
| [ADR-009](ADR-009-local-transport.md) | Local Transport Fidelity 和 Fault Injection | Historical · Accepted |
| [ADR-010](ADR-010-persistence-config.md) | Persistence、Serialization、Config Snapshot | Historical · Accepted |
| [ADR-011](ADR-011-observability.md) | Logging、Metrics、Trace、Audit 和 Failure Bundle | Historical · Accepted |
| [ADR-012](ADR-012-release-update-maintenance.md) | Release Catalog、Rolling Update、Forced Maintenance | Historical · Accepted |
| [ADR-013](ADR-013-migration-dag.md) | Migration DAG、Staging、Atomic Activation | Historical · Accepted |
| [ADR-014](ADR-014-platform-capability.md) | Unity/HybridCLR Platform Capability | Historical · Accepted |
| [ADR-015](ADR-015-mod-extension-boundary.md) | P2 Mod SDK Extension Boundary | Historical · Reserved |
| [ADR-016](ADR-016-benchmark-workload.md) | Benchmark Workload、TickBudget、Hardware Profile | Historical · Accepted |
| [ADR-017](ADR-017-root-abi-generatable-contract.md) | Root ABI 可生成契约粒度（Slot/签名/Handle/Buffer/ErrorDetail） | Historical · Accepted |
| [ADR-018](ADR-018-coreengine-manifest-canonicalization.md) | CoreEngineManifestBody 规范化与分离式 SignatureEnvelope | Historical · Accepted |
| [ADR-019](ADR-019-loader-state-machine-package-identity.md) | Loader 状态机、PackageIdentity 与单进程锁定 | Historical · Accepted |
| [ADR-020](ADR-020-target-profile-orthogonalization.md) | TargetProfile / PackagingProfile / LoadBackend 正交化 | Historical · Accepted |
| [ADR-021](ADR-021-client-authority-update.md) | 客户端权威更新单一 Runtime 事务 | Historical · Accepted |
| [ADR-022](ADR-022-protocol-permission-gate.md) | 生成的 Protocol/Permission 门与字段集 | Historical · Accepted |
| [ADR-023](ADR-023-generated-contract-artifact.md) | 生成契约 Artifact 发布方与零实现依赖 | Historical · Accepted |
| [ADR-024](ADR-024-voxel-p0-contract-set.md) | Voxel P0 公共契约集（World/Port、Chunk/Page、Revision Stamp、Query 一致性） | Historical · Accepted |
| [ADR-025](ADR-025-voxel-participant-receipt-durability.md) | Voxel participant receipt 耐久与 pruning handshake（扩展 ADR-003） | Historical · Accepted |
| [ADR-026](ADR-026-crossworld-commandbuffer-markers.md) | CommandBuffer 状态机与参与者枚举标记（refine ADR-003） | Historical · Accepted |
| [ADR-027](ADR-027-tick-fail-stop.md) | Tick Fail-stop 与 13 相契约矩阵（refine ADR-002） | Historical · Accepted |
| [ADR-028](ADR-028-replication-typed-bodies.md) | Replication typed body 与 MessageType 三方一致（refine ADR-005） | Historical · Accepted |
| [ADR-029](ADR-029-entity-namespace-required.md) | Entity namespace 必填与域约束（refine ADR-004） | Historical · Accepted |
| [ADR-030](ADR-030-processor-structural-commands.md) | mayEmitStructuralCommands 与自重叠合法（refine ADR-002） | Historical · Accepted |
| [ADR-031](ADR-031-gas-lifecycle.md) | GAS Ability/Effect 通用状态机（refine ADR-008） | Historical · Accepted |
| [ADR-032](ADR-032-durable-recovery-records.md) | TxnJournal/CommandLog/WAL 恢复记录（refine ADR-010/011） | Historical · Accepted |
| [ADR-033](ADR-033-config-typed-columns.md) | Config typed columns 动态校验（refine ADR-010） | Historical · Accepted |
| [ADR-034](ADR-034-hot-reload-dual-scope.md) | Hot Reload 双 Scope 原子激活（refine ADR-013） | Historical · Accepted |
| [ADR-035](ADR-035-voxel-snapshot-payload.md) | Voxel Snapshot/Diff payload 与 Canonical Capture（载荷≠Envelope、Cut 投影、Pin 栅栏） | Historical · Accepted |
| [ADR-036](ADR-036-voxel-streaming-durability-ack.md) | Voxel Streaming DurabilityAck、驻留模式与驱逐栅栏（DS 默认不逐出 Dirty） | Historical · Accepted |
| [ADR-037](ADR-037-contract-common-primitives.md) | 契约公共原语归一（$defs 下沉、缺陷修复、词汇冻结） | Historical · Accepted |
| [ADR-038](ADR-038-state-machine-descriptor.md) | 状态机描述符契约（12 机冻结注册表与一致性门） | Historical · Accepted |
| [ADR-039](ADR-039-contract-runtime-artifact.md) | ContractRuntime Artifact 类别（refine ADR-023） | Historical · Accepted |
| [ADR-040](ADR-040-root-abi-generated-bundle.md) | Root ABI Generated Bundle（编译器身份、typeRef 映射、布局 Golden） | Historical · Draft |
| [ADR-041](ADR-041-canonical-digest-profiles.md) | Canonical 与 Digest Profiles（CanonicalJsonV1、四类 Digest 域、自引用规则、Golden） | Historical · Draft |
| [ADR-042](ADR-042-signature-trust-profile.md) | Signature 与 Trust Profile（LumioSignatureV1、域分离 preimage、keyId 派生、拒绝优先级） | Historical · Draft |
| [ADR-043](ADR-043-loader-reentry-error-priority.md) | Loader 重入与错误优先级（终态即终态、身份闩锁、根因优先于回滚） | Historical · Draft |
| [ADR-044](ADR-044-evidence-profiles.md) | Evidence Profiles（三档格式/版本/媒体类型、原始字节摘要、双向覆盖） | Historical · Draft |
| [ADR-045](ADR-045-replication-body-closure.md) | Replication body 闭合、mappingSetHash 摘要域与 length 上界（ADR-028 立论的机器强制） | Historical · Draft |
| [ADR-046](ADR-046-native-kernel-status-band.md) | Native Kernel Status Band（ErrorCode 内核 band 1044–1053、状态值 int32 范围门） | Historical · Draft |
| [ADR-047](ADR-047-lumio-bin-canonical-profile.md) | LumioBinV1 二进制 canonical profile（定宽小端、u32 长度前缀、声明序无填充、Golden 与拒绝向量、snapshot checksum B 档） | Historical · Draft |
| [ADR-048](ADR-048-generated-consumable-surface.md) | Generated 面可消费化（八类闭合契约类型本体、可执行 Protocol/Permission 门、netstandard2.1;net8.0 双目标、capability 常量三形态） | Historical · Draft |
| [ADR-049](ADR-049-replication-state-payload-and-input-command.md) | Replication 状态载荷与 InputCommand 承载（开发态 wire 契约 `gameplay-command-envelope-v1`：InputCommand / stateBlocks / changedBlocks、LumioBinV1、Chat 租户；非 V1.5 基线事件） | Accepted |
| [ADR-050](ADR-050-gas-a1-contracts.md) | GAS A1 lifecycle admission, deterministic evaluation and same-Tick Effect events | Draft |
| [ADR-051](ADR-051-gas-a2-contracts.md) | GAS A2 ECS components, Tag handshake, replication visibility and frame prediction | Draft |
| [ADR-052](ADR-052-ms00002-hello-wire-and-clr-host-abi.md) | MS-00002 开发态 Hello wire 契约与 CLR 装载 ABI | Accepted |
| [ADR-053](ADR-053-entity-binding-and-attribute-query.md) | 连接绑定五元组与 NetEntityId Attribute Query（开发态 `engine/wire/entity-binding-and-query-v1.json`） | Accepted |
| [ADR-054](ADR-054-account-server-topology-and-port.md) | Account Server 第三服务拓扑与账号/准入端口（开发态 `engine/wire/account-port-v1.json`） | Accepted |
| [ADR-055](ADR-055-native-timer-abi.md) | Native Timer ABI 与双层定时（开发态 `engine/wire/native-timer-abi-v1.json`） | Accepted |
| [ADR-056](ADR-056-rm00011-architecture-convergence.md) | RM-00011 架构收敛：单一 ECS / 单一绑定查询 / 字段标注声明 / 单一定时内核 / 真实广播与落盘（第一性原理：如无必要勿增实体） | Accepted |
| [ADR-057](ADR-057-rm00011-r4-owner-rulings.md) | RM-00011 r4 Owner 裁决：ADR-056 六项 Fixture 在 r3 未成立的事实与补救范围（顺序一致、证据即日志、Bot 归 Client、在线名单只在 Runtime、自驱主循环、单一世界原则） | Draft |
| [ADR-058](ADR-058-ecs-world-manager-and-annotation-registry.md) | ECS World Manager、Sync<T> 字段与标注生成桥（单进程单世界、WorldEntity、Netcode 式上行、一套源码两份程序集、组件式 API + 模板内联存储、128 位 NetEntityId、事件即 ClientRpc） | Draft |
| [ADR-059](ADR-059-lumiocoreengine-repository-retirement.md) | LumioCoreEngine 仓库退役与 Owner 指针归属（远端删除、旧 ADR Owner 前向重定向、生成物 URL 经生成源更新） | Accepted |
| [ADR-060](ADR-060-rm00011-r5-owner-rulings-pack-wire-and-observer-projection.md) | RM-00011 r5 Owner 裁决：World Manager 包上网线（C-1″ 一份 codec）、按观察者投影（ObserverComponent、先全量后增量、Scope 裁剪、出视野即删、墓碑推导）、admit 不同步回 id、声明表只从组件生成、模板内联存储做到底、删旧世界；彻底清理不留兼容 | Draft |
| [ADR-061](ADR-061-lumioplatform-repository-and-account-authority.md) | LumioPlatform 第八实现仓与账号权威归属：账号服迁入平台、一库两端口（WS 契约不改 + HTTP `platform-port-v1`）、PostgreSQL 持久真值、AccountWorld 保留、注册策略 profile、launch 端口、`LumioServer/account-server/` 退役；部分取代 ADR-054 | Draft |

旧制度的 `DECISIONS_PENDING` 待决台账已随 `docs/` 一并删除（见 git 历史）；Living Architecture 下未定项直接落 ADR 或任务卡，不再另设台账。

## NativeCore 内部决策（`nativecore/` 子命名空间）

`engine/native/` 的内部实现决策，编号自 `0001` 起，与上表的 `ADR-NNN` 是两套编号、互不占号。合入自原 `engine/native/.spec/`（2026-09-01 文档治理收敛，全仓收敛为单一文档根 `.spec/`）。目录说明见 [`nativecore/README.md`](nativecore/README.md)。

| 编号 | 主题 | 状态 |
|------|------|------|
| [0001](nativecore/0001-build-orchestration-boundary.md) | composition 只产不可变 BuildPlan，platform 是唯一构建执行入口 | 生效 |
| [0002](nativecore/0002-supply-chain-domain-split.md) | signing 内部按四个安全域分拆，运行时只发布 runtime-verifier | 生效 |
| [0003](nativecore/0003-observation-validation-planes.md) | diagnostics 收窄为观测适配平面，smoke 定位为验证平面 | 生效 |
| [0004](nativecore/0004-workspace-runtime-boundary.md) | 固定单 Cargo workspace，运行时发布闭包按白名单冻结并机器校验 | Accepted |
| [0006](nativecore/0006-internal-build-plan-freeze.md) | BuildPlan 冻结为仓内确定性 JSON v1，sidecar Digest + 原子目录发布 | Accepted |
| [0007](nativecore/0007-composition-config-toml-parser.md) | compose 配置解析选定 `toml` crate（精确锁 `=1.1.4`） | 生效 |
| [0008](nativecore/0008-opened-artifact-set-construction-inversion.md) | `OpenedArtifactSet` / `MappedNativeImage` 用构造反转保持私有构造器 | 生效 |
| [0009](nativecore/0009-root-abi-generator-adapter-boundary.md) | root-abi generator 的适配器边界：摘要链锚点必须落在 lock 上 | 生效 |
| [0010](nativecore/0010-root-abi-runtime-unfrozen-semantics-seams.md) | Root ABI 运行时对「上游未冻结」语义保持缺位，只做不透明相等校验 | 生效 |

> `0005` 及原仓若干编号未随迁移进入本仓（原目录即无该文件）；曾指向 `0005-linux-same-object-loader.md` 的历史引用已随 `engine/native/` 内第二套框架副本一并删除，编号空洞保留，需要时按新决策补号，不回填旧号。
