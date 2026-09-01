# composition

> 锁定 NativeCore/VoxelEngine 的组合来源，产出不可变 BuildPlan 与来源证明。优先级：P0；状态：设计中。

## 负责什么

- 锁定 `LumioNativeCore`、`LumioVoxelEngine` 的 Source Commit、Feature Resolution 和构建参数。
- 冻结工具链版本约束，生成**不可变 BuildPlan**（Source Lock、Feature、构建参数、Input Digest）。
- 记录 Source Tree Digest、Build Recipe Digest 和可复现构建的 ProvenanceRecord。

## 明确不负责什么

- 不执行实际编译、链接或产物布局；唯一构建执行入口在 `platform`。
- 不拥有平台构建产物、目录布局或 ArtifactIndex（`platform` 所有）。
- 不定义 Root ABI、Capability、Error 或领域语义。
- 不负责运行时 Loader、签名信任根或产品 Release 路由。
- 不修改 NativeCore/VoxelEngine 的源契约。

## 输入与输出

- 输入：已发布的 Native/Voxel 源描述、Schema、Feature 和工具链约束。
- 输出：不可变 BuildPlan（含 Source Lock 与 Input Digest）、ProvenanceRecord。

## 依赖关系

- 消费：NativeCore/VoxelEngine 已发布源描述与 Feature 约束。
- 被消费：`platform`（执行 BuildPlan）、`root-abi`（读取 Source Lock）、`manifest`（记录 Provenance）、`smoke`（校验可复现性）。
- BuildPlan 一经发布不可变；`platform` 只能消费，不得反向修改。

## 生命周期与失败行为

`Resolve -> Validate -> Freeze BuildPlan -> Record Provenance -> Publish`。未锁定依赖、工具链漂移、Feature 冲突或来源 Digest 不一致必须失败，并保留诊断信息；失败不得发布部分 BuildPlan。

## 验收范围

同一来源和参数可重复产出字节一致的 BuildPlan；BuildPlan、Source Commit、Compiler 约束和 Input Digest 可被 `platform`、`manifest` 与 `smoke` 校验。

## 相关文档

- [模块索引](../README.md)
- [Platform](../platform/README.md)
- [仓库边界与架构契约](../../../../.spec/knowledge/standards/repository-architecture.md)
