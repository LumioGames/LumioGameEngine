# UE Dedicated Server Research 2026-08-29

本包是 Unreal Engine Dedicated Server 架构深度调研交付。研究任务依据委托方给出的完整任务书执行，范围覆盖 **A–M 共 13 章**，重点为 E/F/G/L/M。

## 建议阅读顺序

1. `report/00-executive-summary.md`
2. `report/ue-dedicated-server-research-2026-08-29.md` 的 E、F、L、M
3. C、D、G、I
4. A、B、H、J、K
5. `sources.md`
6. `appendix/`

## 包内结构

- `README.md` — 导读与执行摘要索引
- `report/00-executive-summary.md` — 可独立阅读的执行摘要
- `report/ue-dedicated-server-research-2026-08-29.md` — A–M 主报告
- `sources.md` — 来源总表与实际访问状态
- `appendix/portability-matrix.csv` — M.3 判定表
- `appendix/diagrams.md` — 两张硬指标图源码
- `appendix/terminology.md` — UE→通用术语对照

## 报告规模

- 章节：13（A–M）
- 主报告字符数：约 42,137
- 图：2 张 Mermaid 架构图
- 来源：46 条

## 最重要的可达性说明

指定源码镜像 `https://github.com/Go1c/UnrealEngine` 在本次会话可用的网页检索环境中无法可靠打开/搜索。因此本报告按任务书 R1 执行降级：
- 不伪造源码行号；
- 不把记忆中的路径当 Verified；
- 使用 Epic 官方文档/API 做文档级 Verified；
- 源码实现细节按 Reported/Estimated 标注。

## 执行摘要

见 `report/00-executive-summary.md`；主报告开头亦包含完整执行摘要。
