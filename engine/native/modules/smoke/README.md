# smoke

> CoreEngine 的验证平面，覆盖 NativeHeadless、ABI、包完整性和回归 Fixture。类型：验证平面（非生产模块）；优先级：验证门，随各阶段交付；状态：设计中。

## 负责什么

- 验证 ABI Layout、Header/Binding 一致性和导出符号。
- 验证平台包、ManifestBody、Digest、签名和目录完整性。
- 验证单包加载、PackageIdentity 冲突拒绝、能力缺失和错误诊断。
- 保存正向、失败、Golden、Fuzz 和回归 Fixture。
- 作为每个阶段的验证门贯穿一条端到端 Slice；不参与生产模块的 P0/P1 依赖。

## 明确不负责什么

- 不定义新的 Schema、Error Code、Capability 或业务语义。
- 不代替 Runtime、Server、Client 的集成测试，也不产生生产运行时依赖。
- 不生产最终 Failure Bundle：失败时输出测试报告并引用 Failure Evidence Fragment，Bundle Assembly 归 Host 或独立诊断工具。

## 输入与输出

- 输入：所有 CoreEngine 生成物与公开契约、架构源 Fixture、平台矩阵和基线信息。
- 输出：可审计的测试报告、失败原因（基于稳定 ErrorCode 断言，不依赖字符串消息）、Fixture 引用。

## 依赖关系

- 消费：全部生产模块的公开契约与产物、架构源正负 Fixture。
- 被消费：CI、Release Gate、开发者（测试报告）。
- 任何生产模块不得依赖本模块。

## 生命周期与失败行为

`Discover Fixtures -> Prepare Under-Test Artifacts -> Execute -> Assert -> Report`。断言基于稳定 ErrorCode 和结构化结果；被测对象失败与 smoke 自身失败必须可区分并分别报告；报告不完整或 Fixture 缺失时验证门失败，不得静默跳过。

## 验收范围

至少覆盖包损坏、签名失败、ABI mismatch、符号冲突、重复库、资源不足、Loader 超时和不同平台包可用性；每个 P0 稳定错误至少一组正向和一组失败断言。

## 相关文档

- [模块索引](../README.md)
- [测试与验收标准](../../../../.spec/knowledge/standards/testing.md)
