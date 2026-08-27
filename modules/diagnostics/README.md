# diagnostics

> CoreEngine 观测适配平面：事件契约、同步验证结果与 Failure Evidence Fragment。类型：观测适配平面（非生产模块）；优先级：P1；状态：设计中。

## 负责什么

- 提供稳定的 **Diagnostic Event Contract**：统一 Loader、ABI、Package、签名和 SBOM 事件的关联字段。
- 记录 `ProductId`、`GameReleaseId`、`PackageIdentity`（Manifest Digest / Artifact Set Digest）、`TargetProfile`、`TraceId`、稳定 `ErrorCode` 等字段；字段以架构源 Logging Event Schema 为准。
- 定义 Host 注入的 **EventSink Adapter** 接口；事件发射为非阻塞调用。
- 组装 **Failure Evidence Fragment**（PackageIdentity、Manifest Digest、Artifact Digest、TrustDecision、LoaderState、TargetProfile、TraceId、稳定 ErrorCode）；无 Game Snapshot 时 Fragment 仍必须合法。
- 提供不依赖具体 Sink 的 Metrics/Trace 属性。

## 明确不负责什么

- 不拥有队列、批处理、采样、磁盘 Spool、跨进程传输或 Durable Audit——这些由 Host/Server Observability 实现。
- 不拥有服务器最终 Sink、业务审计规则、Txn Journal 或 Command Log。
- 不阻塞 Loader 或 Host 的任何入口；本平面故障不得反向控制生产模块生命周期。
- 不组装最终 Failure Bundle（Host 或独立诊断工具负责 Bundle Assembly）。
- 不用异步 Diagnostic Log 承载发布审计和签名验证结果：供应链准入结果必须是 Loader 的同步 `VerificationResult` 返回值和审计输入。

## 输入与输出

- 输入：`composition`、`root-abi`、`platform`、`manifest`、`signing`、`loader`、`smoke` 的结构化事件和公开状态快照。
- 输出：规范化事件、Metrics/Trace 属性、Failure Evidence Fragment；EventSink 由 Host 注入，最终持久化由 Host 负责。

## 依赖关系

- 消费：架构源 LoggingEvent/FailureBundle Schema、各模块公开事件契约。
- 被消费：Host Diagnostic Adapter（事件与 Fragment）、Host Bundle Assembler（Fragment）。
- 不读取任何模块私有状态。

## 生命周期与失败行为

事件经非阻塞调用交给 Host 注入的 EventSink Adapter；Sink 拒绝或不可用时按架构源事件类别丢失策略处理（Diagnostic 可采样丢弃，审计类结果不经此路径）。验证结果与审计证据通过同步返回值传递，不受 Sink 状态影响。本平面自身故障必须降级为可诊断事件缺失，不得使 Loader 或 Host 死锁。

## 验收范围

覆盖字段完整性、Trace 关联、脱敏、Sink 拒绝行为、Fragment 完整性（含无 Snapshot 场景）、每 Producer `EventSeq` 顺序重建。

## 相关文档

- [模块索引](../README.md)
- [Loader](../loader/README.md)
- [测试与验收标准](../../.spec/knowledge/standards/testing.md)
