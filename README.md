# LumioCoreEngine

> `LumioNativeCore` 与 `LumioVoxelEngine` 的独立发布聚合仓库、统一 Native C ABI 和平台产物边界。

## 定位

`LumioCoreEngine` 不是新的 Voxel 领域实现，也不是游戏 Runtime。它解决 Native 组件的组合、构建、签名、版本与加载一致性问题：锁定 `LumioNativeCore` 和 `LumioVoxelEngine` 的来源，生成一个可被 Server/Client 统一加载的平台 Native 包，并发布与之对应的 Manifest/Hash/ABI 契约。

总架构基线见 [`docs/architecture/LumioGameEngine_Architecture_v0.3.md`](docs/architecture/LumioGameEngine_Architecture_v0.3.md)。

本仓库聚合和发布 Rust Native 产物；它不承载 C# Runtime 或 Gameplay 热更实现。

## 拥有的状态与生命周期

- NativeCore/VoxelEngine 的来源 Commit、Feature、编译参数和平台矩阵。
- 组合构建图、导出符号、ABI 版本、Capability 表和依赖 Artifact Hash。
- 平台包、调试符号、SBOM、签名和发布 Manifest 的生成状态。
- Loader 的一次性加载、ABI/Capability 校验、卸载和诊断元数据；不拥有运行中的 World 数据。

## 职责

- 以锁定版本组合 `LumioNativeCore` 与 `LumioVoxelEngine`，统一生成静态/动态/移动平台 Native 产物。
- 维护统一 C ABI、导出符号、结构版本、Capability、错误码和 Native Loader Contract。
- 生成给 C# Runtime、Server、Client 使用的托管绑定、P/Invoke 元数据和平台选择清单。
- 确保同一进程只有一套 NativeCore/VoxelEngine 组合被加载，避免重复库和符号漂移。
- 提供可复现构建、Artifact Hash、签名、SBOM、符号包和兼容矩阵。
- 提供 Native Headless Smoke Test、ABI/Loader Test 和跨平台包完整性检查。

## 明确不负责什么

- 不实现或拥有 VoxelWorld、Chunk、Mutation、Streaming 等 Voxel 领域逻辑。
- 不增加 NativeCore 的通用算法，不替代 `LumioNativeCore` 的 Kernel 所有权。
- 不实现 ECS、GAS、Gameplay、Connection、Session、DS、Client Replica 或 Hot Reload。
- 不让 Server/Client/Game 直接依赖下游仓库源码，也不决定玩法兼容语义。

## 对外产物与契约

- `LumioCoreEngine.<platform>.<version>`：统一 Native 动态库/静态库、Loader 和目录布局。
- `CoreEngineManifest.json`：NativeCore/VoxelEngine Commit、版本、平台、ABI、Capability、Feature 和 Artifact Hash。
- 统一 C ABI Header、托管绑定、P/Invoke 源生成元数据、错误码与符号包。
- SBOM、签名、可复现构建证明、License 清单和兼容矩阵。

## Source / Compile-Time Dependencies

- `LumioNativeCore`：通用 Native Kernel。
- `LumioVoxelEngine`：VoxelWorld 领域实现。
- Rust toolchain、平台 SDK、构建/签名工具和经审核的 crates。

`LumioCoreEngine` 可以在编译期依赖这两个下层仓库；任何上层 Runtime、Server、Client、Game 都不得反向成为其源码依赖。

## Generated Contract Dependencies

聚合并重新导出 NativeCore/VoxelEngine 生成的 ABI Header、Capability、错误码和 Voxel Contract，生成统一版本前缀和绑定包。生成结果必须记录源 Commit 与 Hash，禁止手工修改后发布。

## Runtime Loading Relationships

```text
LumioServer / LumioClient / NativeHeadless Host
  -> CoreEngine Loader
  -> one LumioCoreEngine platform package
  -> NativeCore + VoxelEngine symbols
  -> Runtime adapters / VoxelWorld instances
```

CoreCLR 与 C# Runtime 只使用统一 Managed Contract；Server/Client 不直接探测或加载 NativeCore/VoxelEngine 的第二份库。

## Release Composition Relationships

每个上层 Server、Client、Runtime 或 Game Release 只声明一个 CoreEngine 版本与 Hash。CoreEngine 发布独立于玩法发布，但 `GameManifest` 必须锁定其 ABI/Capability；升级时通过兼容矩阵明确允许的组合。

## Room Modes / Host Profiles

为 `PureHeadless`、`NativeHeadless`、`LocalEmbedded`、`LocalSplitProcess`、`RemoteDS`、`MobileLocal` 提供同一加载 API。`LocalEmbedded` 同进程只加载一套 CoreEngine，但创建独立的 Server/Client VoxelWorld 实例；模式差异不进入 Native Kernel。

## Headless Test Surface

- Manifest/Hash/签名、平台目录、导出符号和 ABI 结构布局检查。
- Loader 一次加载、重复加载保护、Capability 缺失、版本不匹配和错误诊断。
- NativeHeadless Kernel + Voxel Smoke Test，以及不同 Host Profile 的包可用性矩阵。
- 构建可复现性、SBOM、License 和最小运行时依赖检查。

## Version / Manifest

- CoreEngine 使用独立 SemVer；ABI 主版本变化必须显式升级。
- Manifest 必须包含组合来源、平台/架构、编译器、Feature、ABI/Capability、Artifact Hash、签名、SBOM 和生成时间。
- 所有 Host 在启动时校验 Manifest；不匹配时拒绝加载并输出缺失能力。

## 开发规范

- 只做聚合、适配、发布和校验，不把领域逻辑偷偷下沉到聚合仓库。
- 任何导出符号或结构变化都要更新 ABI、绑定、兼容矩阵和 Smoke Test。
- 平台差异通过 Manifest/Loader 表达；禁止在上层复制平台选择逻辑。
- 发布前必须从干净来源重建，验证 Hash、签名、SBOM 和 Native Headless 测试。

## 当前阶段任务

- 建立 NativeCore + VoxelEngine 的可复现组合构建和统一 ABI 最小闭环。
- 定义 `CoreEngineManifest`、平台目录布局、Loader 一次加载规则和 Artifact 签名流程。
- 为 Server、Client、LocalEmbedded 和 NativeHeadless 提供包加载 Smoke Test。
