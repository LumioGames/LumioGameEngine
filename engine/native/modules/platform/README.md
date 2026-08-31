# platform

> TargetProfile 规范化与唯一构建执行入口，产出平台包与 ArtifactIndex。优先级：P0 最小子集（单一 Linux Server TargetProfile）/ P1 完整矩阵；状态：设计中。

## 负责什么

- 规范化 **TargetProfile**：OS、CPU Architecture、ABI/libc、Minimum OS、SDK 与 Compiler 约束。
- 作为**唯一权威构建执行入口**：消费 `composition` 的不可变 BuildPlan 执行编译与链接。
- 声明 **PackagingProfile**（文件命名、目录布局、调试符号布局、归档格式）与 **LoadBackend**（StaticLinked / DynamicLibrary）。
- 输出 PlatformArtifactSet 与逐文件 **ArtifactIndex**（规范路径、类型、大小、Digest），并记录上一阶段 Input Digest。

## 明确不负责什么

- 不定义或修改 BuildPlan、Source Lock 或 Feature（`composition` 所有）。
- 不实现 Unity 渲染、Host Profile 语义、网络传输或移动端业务逻辑。
- 不允许上层重复实现平台选择逻辑。
- 不把供应商 SDK 类型泄漏进稳定 ABI。
- PureHeadless 是 No-Native/ProcessAbsent 路径，不产生本模块产物。

## 输入与输出

- 输入：`composition` 不可变 BuildPlan、`root-abi` 生成产物、平台 SDK 和工具链。
- 输出：PlatformArtifactSet、ArtifactIndex、调试符号、LoadBackend 声明和平台兼容矩阵。

## 依赖关系

- 消费：`composition::BuildPlan`、`root-abi` 生成产物、平台 SDK/工具链约束。
- 被消费：`manifest`（ArtifactIndex）、`signing`（ArtifactIndex 与包产物）、`loader`（LoadBackend 契约）、`smoke`（包完整性验证）。

## 生命周期与失败行为

`Normalize TargetProfile -> Execute BuildPlan -> Layout -> Verify -> Publish ArtifactSet + ArtifactIndex`。每阶段记录上一阶段 Input Digest。Target 不支持、SDK 缺失、链接模式或符号前缀不一致、布局校验失败必须给出稳定原因并失败；失败不得发布部分 ArtifactSet。

## 验收范围

P0 先覆盖单一 TargetProfile（Linux Server、x86_64、glibc、DynamicLibrary）端到端产出；每个声明平台必须通过包完整性检查、Loader 预检和 NativeHeadless Smoke；平台不支持或 SDK 缺失必须给出稳定原因。

## 相关文档

- [模块索引](../README.md)
- [Composition](../composition/README.md)
- [Loader](../loader/README.md)
