# 0003 · diagnostics 收窄为观测适配平面，smoke 定位为验证平面

- 日期:2026-08-27
- 状态:生效

## 背景

架构 Review（CE-008/CE-009/CE-014）指出:diagnostics 声称拥有有界异步队列、批处理、应急落盘并可阻塞新入口，越过了 CoreEngine Adapter 边界（队列满载可能反向死锁 Loader）;smoke 与 diagnostics 同时声称生产 Failure Bundle;公共架构第 16 章的 CoreEngine 首批模块只有六个生产模块，smoke/diagnostics 的定位未在模块地图中说明。

## 决策

- `diagnostics` 收窄为**观测适配平面**（非生产模块）:只拥有 Diagnostic Event Contract、同步 `VerificationResult` 传递、Host 注入的 EventSink Adapter、Failure Evidence Fragment 组装和 Sink 无关的 Metrics/Trace 属性。队列、批处理、采样、磁盘 Spool、跨进程传输与 Durable Audit 归 Host/Server Observability;本平面不得阻塞 Loader 或 Host 入口。
- 供应链准入结果（发布审计、签名验证、SBOM 结果）必须是 Loader 的同步返回值和审计输入，不得只依赖异步日志。
- `smoke` 定位为**验证平面**（非生产模块、每阶段验证门）:输出测试报告与 Fixture 引用，不生产最终 Failure Bundle;Bundle Assembly 归 Host 或独立诊断工具，CoreEngine 各模块只输出 Failure Evidence Fragment。
- 生产模块不得依赖两个平面;模块地图中两平面的公共登记随架构源第 16 章更新（ADR-020）。

## 后果

- Loader 生命周期与 Host 日志基础设施解耦，消除队列满载反向阻塞风险。
- Host 必须提供 EventSink 与 Bundle Assembler 实现，CoreEngine 单独跑时只有 Fragment 与同步结果可用。
- Failure Bundle 格式唯一（架构源所有），CoreEngine 不再定义第二套 Bundle。
