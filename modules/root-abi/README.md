# root-abi

> 将架构源 Schema 组合为唯一 Root C ABI，并生成跨语言绑定。优先级：P0；状态：设计中。

## 负责什么

- 发布唯一 Root API Table，包括 `abi_version`、`struct_size` 和 `capability_bits`。
- 统一导出符号前缀、稳定 Error Code 和不透明 Handle 边界。
- 生成 Header、Managed Adapter 和 P/Invoke 元数据，并记录 Compiler、Input Hash、Output Hash。

## 明确不负责什么

- 不在本仓手写第二套 ABI、P/Invoke 布局或领域绑定语义。
- 不拥有 VoxelWorld、ECS、Gameplay、Session 或 Runtime 生命周期。

## 输入与输出

- 输入：架构源发布的 Native/Voxel Schema 和 `composition` 产物。
- 输出：只读的 Root Header、Managed Contract、绑定元数据和 ABI 兼容报告。

## 生命周期与失败行为

`Consume Schema -> Generate -> Check Layout/Symbols -> Publish Immutable Artifacts`。布局漂移、符号冲突、版本不匹配和无法转换的 panic/exception 必须失败并产生稳定诊断。

## 验收范围

覆盖 ABI Layout、符号导出、跨语言调用、内存归属、失效 Handle、错误转换和负向兼容 Fixture。

## 相关文档

- [模块索引](../README.md)
- [架构基线镜像](../../docs/architecture/LumioGameEngine_Architecture_v1.0.md)
