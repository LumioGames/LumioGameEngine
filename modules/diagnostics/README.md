# diagnostics

> 提供 CoreEngine 的诊断事件、Metrics、Trace 和 Failure Bundle 适配。优先级：P1；状态：设计中。

## 负责什么

- 统一 Loader、ABI、Package、签名和 SBOM 事件的关联字段。
- 记录 `ProductId`、`GameReleaseId`、`CoreEngineArtifactHash`、`Platform` 和 `TraceId`。
- 提供有界异步批次、队列满载策略和 Error/Fatal 应急落盘。
- 组装可校验、可下载和可重建的 Failure Bundle。

## 明确不负责什么

- 不拥有服务器最终 Sink、业务审计规则、Txn Journal 或 Command Log。
- 不用普通 Diagnostic Log 替代不可静默丢失的发布审计和签名验证结果。

## 输入与输出

- 输入：Loader、Manifest、Signing、Platform、Smoke 的结构化事件和关联上下文。
- 输出：规范化事件、指标、Trace 片段和 Failure Bundle；最终 Sink 由 Host 提供。

## 生命周期与失败行为

事件进入有界队列后批量发送；队列满载按事件类别执行采样、降级或阻止新接入。审计、签名验证和 SBOM 结果不得静默丢失。

## 验收范围

覆盖字段完整性、Trace 关联、脱敏、队列满载、应急落盘、Failure Bundle 校验和跨线程顺序重建。

## 相关文档

- [模块索引](../README.md)
- [测试与验收标准](../../.spec/knowledge/standards/testing.md)
