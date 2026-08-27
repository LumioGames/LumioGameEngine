# LumioCoreEngine

> `LumioNativeCore` 与 `LumioVoxelEngine` 的聚合发布、统一 Native ABI、Loader 和平台产物边界。

## 架构基线

- Baseline：`LGE-V1.0-2026-08-27`
- 唯一架构源：`LumioGameEngineArchitecture`
- 本地镜像：[`docs/architecture/LumioGameEngine_Architecture_v1.0.md`](docs/architecture/LumioGameEngine_Architecture_v1.0.md)

`LumioCoreEngine` 是 Native 发布层，不是新的运行时或领域引擎。它把锁定版本的 NativeCore/VoxelEngine 组合为一个可验证的平台包，负责 Root ABI、Loader、Hash、签名、SBOM 和平台兼容性。

## Architecture Gate

Root ABI、Capability、Manifest/ID Schema 和失败 Fixture 的唯一来源是 `LumioGameEngineArchitecture`。Loader 或平台包变更必须消费已发布 Schema/Artifact，保留 Compiler/Input/Output Hash，并在架构源执行 `python3 tools/lumio_contract.py validate`；不得在聚合层手写第二套 P/Invoke 布局。

## 拥有的状态与生命周期

- NativeCore/VoxelEngine 的 Source Commit、Feature、Compiler、编译参数和平台矩阵。
- 组合构建图、导出符号、ABI/Capability、Artifact Hash 和发布签名状态。
- 平台包、调试符号、SBOM、许可证清单、Manifest 和 Loader Registry 元数据。
- Loader 的一次性加载、版本拒绝、诊断和卸载状态；不拥有运行中的 World、Session 或 Chunk。

## 子模块

| 子模块 | 责任 | 优先级 |
| --- | --- | --- |
| [`composition`](modules/composition/README.md) | 锁定 NativeCore/VoxelEngine Source 和 Feature | P0 |
| [`root-abi`](modules/root-abi/README.md) | 统一 API Table、符号前缀、Header 和绑定输入 | P0 |
| [`loader`](modules/loader/README.md) | 单包加载、Registry、重复版本拒绝和能力校验 | P0 |
| [`manifest`](modules/manifest/README.md) | CoreEngineManifest、Hash、依赖和平台描述 | P0 |
| [`signing`](modules/signing/README.md) | 签名、信任根、Key Rotation 元数据和 SBOM | P1 |
| [`platform`](modules/platform/README.md) | Linux/Windows/Desktop/Mobile 目录与链接矩阵 | P1 |
| [`smoke`](modules/smoke/README.md) | NativeHeadless、ABI Layout、包完整性和回归 Fixture | P0 |
| [`diagnostics`](modules/diagnostics/README.md) | Loader/ABI 事件、Metrics、Trace 和 Failure Bundle | P1 |

模块边界、依赖方向、输入输出和验收范围见 [`modules/README.md`](modules/README.md)。

## 职责

- 锁定并组合 NativeCore/VoxelEngine 版本，生成每个平台唯一 Native 包。
- 统一 Root C ABI、导出符号、结构版本、Capability、Error Code 和 Loader Contract。
- 根据源 Schema 生成统一 Header、托管绑定和 P/Invoke 元数据；不得产生第二套领域绑定语义。
- 保证同一进程只加载一套 CoreEngine/Native 组合，拒绝 ABI、Capability、Hash 或符号不匹配。
- 生成可复现构建证明、Artifact Hash、签名、SBOM、License 清单和平台目录。
- 提供 NativeHeadless、ABI Smoke、重复加载、失败诊断和包完整性测试。

## 明确不负责什么

- 不实现或拥有 VoxelWorld、Chunk、Mutation、Streaming、ECS、GAS、Gameplay 或 Migration 业务逻辑。
- 不增加 NativeCore 通用算法，不替代 VoxelEngine/NativeCore 的领域所有权。
- 不实现 Connection、Session、WorldSlot、Release Pool、CoreCLR、Client Replica 或 Hot Gameplay。
- 不决定产品语义兼容；只验证结构、ABI、Capability、Manifest 和 Artifact 完整性。
- 不把所有跨仓工具、日志后端或持久化数据库塞进聚合层。

## Root ABI 与 Loader 契约

CoreEngine 只发布一份 Root API Table。每个表和结构带 `abi_version`、`struct_size`、`capability_bits`；所有下层布局来自锁定源 Schema。Loader 必须校验平台、架构、Compiler Feature、ABI 主版本、符号前缀、Artifact Hash 和签名。

- 同一进程拒绝第二份不兼容 Native 包或重复 Worker Pool。
- 绑定由源 Schema 生成，记录 Compiler Version、Input Hash、Output Hash；生成物只读。
- 不跨 ABI 传对象引用、异常、Rust/C# 容器或未版本化指针。
- Rust panic、失效 Handle、Capability 缺失、加载失败都转换为稳定错误和诊断事件。
- 静态链接或动态加载方式按平台唯一声明，不能由上层自行复制选择逻辑。

## 线程、资源与故障域

Loader 在规定 Host 线程初始化；Native Worker 的完成批次由 Runtime/Host 在 Tick Barrier 应用。CoreEngine 不创建业务线程和 World。重复加载、符号冲突、ABI mismatch、包损坏和资源不足属于可诊断 Fault，由 Server/Client 决定 Session、进程或发布级处置。

## 日志与观测

输出 Loader/ABI/Package Diagnostic Event、Metrics、Trace 和审计片段，不拥有服务器最终 Sink。事件带 `ProductId、GameReleaseId、CoreEngineArtifactHash、Platform、TraceId`，支持异步批次和 Failure Bundle；发布审计、签名验证和 SBOM 结果不可静默丢失。

## 序列化与持久化边界

CoreEngine 只提供 ABI/绑定所需的机械 Buffer、Hash/Checksum 和压缩依赖，不能定义 Game/Voxel Snapshot 或 WAL 语义。平台包和 Manifest 使用规范化、可复现的序列化；生成时间等非确定字段与 Artifact Hash 分离。

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
  -> CoreEngine Loader
  -> one signed platform package
  -> NativeCore + VoxelEngine symbols
  -> Runtime/Voxel adapters
```

LocalEmbedded 的两个 Role 共享同一 Native 包，但各自创建 VoxelWorld/ReplicaWorld；Server/Client 不得直接加载下层第二份库。

## Release Composition Relationships

每个上层 Release 只声明一个 CoreEngine 版本、Artifact Hash、ABI、Capability 和平台。CoreEngine 独立发布，`GameManifest` 锁定其组合；产品滚动更新由 Server Release Pool 编排。

## Room Modes / Host Profiles

为 `PureHeadless`、`NativeHeadless`、`LocalEmbedded`、`LocalSplitProcess`、`RemoteDS`、`MobileLocal` 提供统一加载入口。PureHeadless 可不加载 Native；Scenario 通过 Capability 匹配，不在 CoreEngine 内分叉业务。

## Headless Test Surface

- 可复现组合构建、Manifest/Hash/签名/SBOM/License 检查。
- ABI 结构布局、Header/Binding 一致性、导出符号和平台目录检查。
- Loader 单加载、重复加载拒绝、Capability 缺失、版本不匹配和错误诊断。
- NativeHeadless Kernel/Voxel Smoke 和不同平台包可用性矩阵。
- Fault：包损坏、签名失败、ABI mismatch、符号冲突、重复库、资源不足、Loader 超时。

## Version / Manifest

`CoreEngineManifest` 至少包含 Source Commit、Platform/Architecture、Compiler、Feature、ABI/Capability、Artifact Hash、Signature、SBOM、License 和生成工具版本。ABI 主版本变化必须显式升级并拒绝不兼容组合。

## 开源优先与供应链

优先采用成熟开源的构建、Loader、签名、序列化和诊断方案；通过 Adapter 隔离，锁定版本/Commit，检查许可证、漏洞、AOT、确定性和性能。默认偏好宽松许可证；强传染许可证需要法务审核。

## 开发规范

- 只做聚合、适配、发布和校验，不把领域逻辑偷偷下沉。
- 任何 ABI/符号/平台变化都必须同步 Header、Binding、Manifest、Compatibility 和 Smoke Test。
- 发布前从干净来源重建并比对 Hash；禁止手工修改生成物后发布。
- Loader 错误必须含稳定 Code、缺失能力、Artifact 和 Trace 关联信息。

## 当前阶段与开发节奏

1. **Architecture Gate**：冻结 Root ABI、Manifest 字段、平台链接矩阵和 Loader 状态机。
2. **Foundation**：组合 NativeCore/VoxelEngine、生成 Header/Binding、实现单加载和 ABI Smoke。
3. **Vertical Slice**：接入 Runtime/Server/Client，验证 NativeHeadless、LocalEmbedded 和 Release 校验。
4. **Production Hardening**：可复现构建、签名/SBOM、跨平台矩阵、故障注入和滚动发布包。
5. **P2**：更多平台和可选构建后端；不改变 V1 Root ABI 语义。
