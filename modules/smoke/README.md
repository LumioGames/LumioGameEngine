# smoke

> CoreEngine 的验证支撑模块，覆盖 NativeHeadless、ABI、包完整性和回归 Fixture。优先级：P0；状态：设计中。

## 负责什么

- 验证 ABI Layout、Header/Binding 一致性和导出符号。
- 验证平台包、Manifest、Hash、签名和目录完整性。
- 验证单包加载、重复加载拒绝、能力缺失和错误诊断。
- 保存正向、失败、Golden、Fuzz 和回归 Fixture。

## 明确不负责什么

- 不定义新的 Schema、Error Code、Capability 或业务语义。
- 不代替 Runtime、Server、Client 的集成测试，也不产生生产运行时依赖。

## 输入与输出

- 输入：所有 CoreEngine 生成物、架构源 Fixture、平台矩阵和基线信息。
- 输出：可审计的测试报告、失败原因、Artifact Hash 和 Failure Bundle。

## 验收范围

至少覆盖包损坏、签名失败、ABI mismatch、符号冲突、重复库、资源不足、Loader 超时和不同平台包可用性。

## 相关文档

- [模块索引](../README.md)
- [测试与验收标准](../../.spec/knowledge/standards/testing.md)
