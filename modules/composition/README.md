# composition

> 锁定 NativeCore/VoxelEngine 的组合来源和可复现构建输入。优先级：P0；状态：设计中。

## 负责什么

- 锁定 `LumioNativeCore`、`LumioVoxelEngine` 的 Source Commit、Feature、Compiler 和构建参数。
- 描述组合构建图、平台矩阵和工具链版本。
- 记录 Source Commit、Input Hash 和可复现构建证明。

## 明确不负责什么

- 不定义 Root ABI、Capability、Error 或领域语义。
- 不负责运行时 Loader、签名信任根或产品 Release 路由。
- 不修改 NativeCore/VoxelEngine 的源契约。

## 输入与输出

- 输入：已发布的 Native/Voxel 源 Schema、Artifact、Feature 和工具链约束。
- 输出：锁定的组合描述、构建输入、平台构建产物及其来源证明。

## 生命周期与失败行为

`Resolve -> Validate -> Build -> Record Provenance -> Publish Input`。未锁定依赖、工具链漂移、平台不匹配或来源 Hash 不一致必须失败，并保留诊断信息。

## 验收范围

同一来源和参数可重复构建；构建结果、Source Commit、Compiler 和 Input Hash 可被后续 `manifest` 与 `smoke` 校验。

## 相关文档

- [模块索引](../README.md)
- [仓库边界与架构契约](../../.spec/knowledge/standards/repository-architecture.md)
