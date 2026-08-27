# LumioCoreEngine

> `LumioNativeCore` 与 `LumioVoxelEngine` 的聚合发布、统一 Native ABI、Loader 和平台产物边界。

## 架构基线

- Baseline：`LGE-V1.4-2026-08-27`
- 唯一架构源：`LumioGameEngineArchitecture`
- 本地镜像：[`docs/architecture/LumioGameEngine_Architecture_v1.2.md`](docs/architecture/LumioGameEngine_Architecture_v1.2.md)

`LumioCoreEngine` 是 Native 发布层，不是新的运行时或领域引擎。它把锁定版本的 NativeCore/VoxelEngine 组合为一个可验证的平台包，负责 Root ABI、Loader、ArtifactIndex、签名、SBOM 和平台兼容性。

## Architecture Gate

Root ABI、Capability、Manifest/ID Schema 和失败 Fixture 的唯一来源是 `LumioGameEngineArchitecture`。Loader 或平台包变更必须消费已发布 Schema/Artifact，保留 Compiler/Input/Output Hash，并在架构源执行 `python3 tools/lumio_contract.py validate`；不得在聚合层手写第二套 P/Invoke 布局。

## 拥有的状态与生命周期

- NativeCore/VoxelEngine 的 Source Commit、Feature、Compiler、编译参数和平台矩阵。
- 组合构建图、导出符号、ABI/Capability、ArtifactIndex、Artifact Set Digest 和发布签名状态。
- 平台包、调试符号、SBOM、许可证清单、CoreEngineManifestBody 和 Loader Registry 元数据。
- Loader 的进程级 PackageIdentity 锁定、预检、Lease 和诊断状态；不拥有运行中的 World、Session 或 Chunk。V1 不做物理卸载（No-Physical-Unload，架构源 ADR-019 已冻结，V2 以新 ADR 重审）。

## 子模块

| 子模块 | 责任 | 类型 | 优先级 |
| --- | --- | --- | --- |
| [`composition`](modules/composition/README.md) | 锁定 NativeCore/VoxelEngine Source 与 Feature，产出不可变 BuildPlan | 生产 | P0 |
| [`root-abi`](modules/root-abi/README.md) | 统一 API Table、符号前缀、Header 和绑定生成 | 生产 | P0 |
| [`loader`](modules/loader/README.md) | 进程级 PackageIdentity 锁定、预检和 LoaderLease | 生产 | P0 |
| [`manifest`](modules/manifest/README.md) | CoreEngineManifestBody、Digest Chain 和平台描述 | 生产 | P0 |
| [`signing`](modules/signing/README.md) | 供应链证据、离线签名、运行时验证和信任策略 | 生产 | P0 最小子集 / P1 完整 |
| [`platform`](modules/platform/README.md) | TargetProfile 规范化与唯一构建执行入口 | 生产 | P0 最小子集 / P1 完整矩阵 |
| [`smoke`](modules/smoke/README.md) | NativeHeadless、ABI Layout、包完整性和回归 Fixture | 验证平面 | 验证门 |
| [`diagnostics`](modules/diagnostics/README.md) | 事件契约、同步验证结果和 Failure Evidence Fragment | 观测适配平面 | P1 |

模块边界、依赖方向、产物流、术语表和验收范围见 [`modules/README.md`](modules/README.md)。

## 职责

- 锁定并组合 NativeCore/VoxelEngine 版本，生成每个平台唯一 Native 包。
- 统一 Root C ABI、导出符号、结构版本、Capability、Error Code 和 Loader Contract。
- 根据源 Schema 生成统一 Header、托管绑定和 P/Invoke 元数据；不得产生第二套领域绑定语义。
- 保证同一进程只锁定一个 PackageIdentity；对 ABI、Capability、Digest 或符号不匹配给出稳定拒绝。
- 生成可复现构建证明、ArtifactIndex、SignatureEnvelope、SBOM、License 清单和平台目录。
- 提供 NativeHeadless、ABI Smoke、重复加载、失败诊断和包完整性测试。

## 明确不负责什么

- 不实现或拥有 VoxelWorld、Chunk、Mutation、Streaming、ECS、GAS、Gameplay 或 Migration 业务逻辑。
- 不增加 NativeCore 通用算法，不替代 VoxelEngine/NativeCore 的领域所有权。
- 不实现 Connection、Session、WorldSlot、Release Pool、CoreCLR、Client Replica 或 Hot Gameplay。
- 不决定产品语义兼容；只验证结构、ABI、Capability、ManifestBody 和 Artifact 完整性。
- 不把所有跨仓工具、日志后端或持久化数据库塞进聚合层。

## Root ABI 与 Loader 契约

CoreEngine 只发布一份 Root API Table。每个表和结构带 `abi_version`、`struct_size`、`capability_bits`；所有下层布局来自锁定源 Schema。Loader 必须对**实际打开的文件句柄**校验平台、架构、Compiler Feature、ABI 主版本、符号前缀、ArtifactIndex 逐文件 Digest 和签名（验证与映射针对同一组句柄，防止 TOCTOU）。

- 首次成功 Acquire 后进程锁定唯一 PackageIdentity（Manifest Digest + Artifact Set Digest + ABI Identity + TargetProfile）；同一身份重复 Acquire 幂等，任何不同身份一律返回稳定 `PackageIdentityConflict`，不做「看起来兼容」判断；拒绝重复 Worker Pool。
- Loader 输出 LoaderLease + RootApiTableView，不输出裸 Library Handle；只消费运行时 `runtime-verifier` 的 VerifiedPackageDescriptor，不信任离线验证结论。
- 绑定由源 Schema 生成，记录 Compiler Version、Input Hash、Output Hash；生成物只读。
- 不跨 ABI 传对象引用、异常、Rust/C# 容器或未版本化指针。
- Rust panic、失效 Handle、Capability 缺失、加载失败都转换为稳定错误和诊断事件。
- 静态链接或动态加载方式按平台唯一声明（LoadBackend），两个 Backend 共享同一逻辑状态机；上层不得自行复制选择逻辑。

## 线程、资源与故障域

Loader 在规定 Host 线程初始化；Native Worker 的完成批次由 Runtime/Host 在 Tick Barrier 应用。CoreEngine 不创建业务线程和 World。重复加载、符号冲突、ABI mismatch、包损坏和资源不足属于可诊断 Fault，由 Server/Client 决定 Session、进程或发布级处置。

## 日志与观测

输出 Loader/ABI/Package Diagnostic Event、Metrics、Trace 和 Failure Evidence Fragment，不拥有服务器最终 Sink；队列、批处理与持久化由 Host 注入的 EventSink 负责。事件带 `ProductId、GameReleaseId、PackageIdentity、TargetProfile、TraceId`。发布审计、签名验证和 SBOM 结果通过同步 `VerificationResult` 返回并进入审计，不可静默丢失，不得只依赖异步日志。

## 序列化与持久化边界

CoreEngine 只提供 ABI/绑定所需的机械 Buffer、Hash/Checksum 和压缩依赖，不能定义 Game/Voxel Snapshot 或 WAL 语义。ManifestBody 使用规范化、可复现的序列化；生成时间等非确定字段不进入 ManifestBody，只存在于 SignatureEnvelope 或外部元数据。

## Source / Compile-Time Dependencies

- `LumioNativeCore`：通用 Kernel、ABI 源和 Error/Capability。
- `LumioVoxelEngine`：Voxel 领域 crate、Schema 和 Voxel ABI 源。
- Rust toolchain、平台 SDK、构建/签名工具和经过供应链审查的 crates。
- 不允许任何 Runtime、Server、Client 或 Game 反向成为 CoreEngine 源码依赖。

## Generated Contract Dependencies

消费 NativeCore/VoxelEngine 源 Schema，生成统一 Root Header、Managed Adapter、P/Invoke 元数据、Capability、Error 和平台清单。生成物必须记录 Source Commit/Hash；上层只消费 CoreEngine 发布包，不自行重新生成布局。

## Runtime Loading Relationships

```text
LumioServer / LumioClient / NativeHeadless Host
  -> CoreEngine Loader（运行时验证 + PackageIdentity 锁定）
  -> one verified platform package
  -> NativeCore + VoxelEngine symbols
  -> Runtime/Voxel adapters
```

LocalEmbedded 的两个 Role 共享同一 Native 包，但各自创建 VoxelWorld/ReplicaWorld；Server/Client 不得直接加载下层第二份库。

## Release Composition Relationships

每个上层 Release 通过不可变键精确引用一个 CoreEngine 包：PackageIdentity、Manifest Digest、Artifact Set Digest、ABI Identity、TargetProfile 和 Signature 引用。CoreEngine 独立发布，`GameManifest` 锁定其组合；产品滚动更新由 Server Release Pool 编排。

## Room Modes / Host Profiles

为 `NativeHeadless`、`LocalEmbedded`、`LocalSplitProcess`、`RemoteDS`、`MobileLocal` 提供统一加载入口。`PureHeadless` 是 No-Native/ProcessAbsent 路径（ReferenceVoxelPort），不经过 CoreEngine Loader，不加载空包。Scenario 通过 Capability 匹配，不在 CoreEngine 内分叉业务。

## Headless Test Surface

- 可复现组合构建、ManifestBody/Digest/签名/SBOM/License 检查。
- ABI 结构布局、Header/Binding 一致性、导出符号和平台目录检查。
- Loader 单加载、PackageIdentity 冲突拒绝、Capability 缺失、版本不匹配和错误诊断。
- NativeHeadless Kernel/Voxel Smoke 和不同平台包可用性矩阵。
- Fault：包损坏、签名失败、ABI mismatch、符号冲突、重复库、资源不足、Loader 超时。

## Version / Manifest

`CoreEngineManifestBody` 是确定性、可复现、无自引用的规范化载荷，至少包含 Source Commit、TargetProfile、Compiler、Feature、ABI/Capability、ArtifactIndex、Artifact Set Digest、EvidenceSet（SBOM/License/Provenance 的 Digest）和生成工具版本。Signature 位于分离的 `SignatureEnvelope`，不进入 ManifestBody；Evidence 以 Digest 绑定，签名后不可替换。ABI 主版本变化必须显式升级并拒绝不兼容组合。

## 开源优先与供应链

优先采用成熟开源的构建、Loader、签名、序列化和诊断方案；通过 Adapter 隔离，锁定版本/Commit，检查许可证、漏洞、AOT、确定性和性能。默认偏好宽松许可证；强传染许可证需要法务审核。运行时发布包只含 `runtime-verifier` 与只读信任元数据，不含 Signer 或私钥访问代码。

## 开发规范

- 只做聚合、适配、发布和校验，不把领域逻辑偷偷下沉。
- 任何 ABI/符号/平台变化都必须同步 Header、Binding、ManifestBody、Compatibility 和 Smoke Test。
- 发布前从干净来源重建并比对 Digest；禁止手工修改生成物后发布。
- Loader 错误必须含稳定 Code、缺失能力、Artifact 和 Trace 关联信息。

## 当前阶段与开发节奏

1. **Architecture Gate**：在架构源冻结 Root ABI、CoreEngineManifestBody/SignatureEnvelope、TargetProfile、Loader 状态机与错误登记，再同步本仓镜像。
2. **Foundation（P0 最小垂直切片）**：单一 TargetProfile（Linux Server、x86_64、glibc、DynamicLibrary）+ 测试密钥 Signer/运行时 Verifier + Loader 状态机 + 基础事件契约，贯穿一条端到端 Slice 并由 `smoke` 验证。
3. **Vertical Slice**：接入 Runtime/Server/Client，验证 NativeHeadless、LocalEmbedded 和 Release 校验。
4. **Production Hardening（P1 扩展）**：完整平台矩阵、生产 Key Management/Rotation、远程签名、完整 SBOM/License 自动化、故障注入和滚动发布包。
5. **P2**：更多平台和可选构建后端；不改变 V1 Root ABI 语义。
