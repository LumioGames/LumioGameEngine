---
status: pending
---

# 补上 RM-00011 里 R-00345 → R-00344 那条 500 失败的需求引用边，并向 Workflow 反馈该 500

2026-09-01 上传 RM-00011 需求室的 52 项操作中，51 项 verified，只有 `edge-source-account` 这一条引用边失败且**从未重试**：`PUT /requirements/01a05b5a-75ad-7062-bfb9-b9df66ea7ca2/references/01a05b5a-75a6-77f1-a568-7b044e1a0053` 返回 `500 urn:gameflow:problem:internal-error`（`detail: 需求引用操作失败`，`traceId=ed48c86f-1659-4fc2-b58b-9bc520fd84d6`）。两端分别是 `R-00345`（source，remoteId `01a05b5a-75ad-7062-bfb9-b9df66ea7ca2`）与 `R-00344`（account，remoteId `01a05b5a-75a6-77f1-a568-7b044e1a0053`）。原上传工件 `.workflow-drafts/wf-ecs-entity-chat-20260901/` 已随本次治理收敛删除，本卡是该失败的唯一留存记录（工件内容见 git 历史 `2cce508`）。

## 涉及范围

- 无本仓文件改动；操作对象是 Workflow(`lumiogamesengine`，projectId `proj_b6979c277715a6c6c490a541ac69709b`)。
- 主 loop 是唯一 Workflow 写入方。

## 验收标准

- [ ] 在 Workflow 上确认 `R-00345` 是否已引用 `R-00344`；若缺失则补建该引用边，并记录操作返回。
- [ ] 复核 RM-00011 其余 23 条需求引用边是否齐全（上传日志只覆盖本次 bundle 声明的边，不排除别处遗漏）。
- [ ] 按「WorkFlow 工具 bug 反馈」纪律，向 Workflow 提交一次该 500 的反馈提示词：含 `traceId=ed48c86f-1659-4fc2-b58b-9bc520fd84d6`、请求方法与路径、以及「同一 bundle 内 429 限流可自动重试而 500 不重试」这一批量上传器的行为缺陷建议。
- [ ] 本卡删除（目录纪律：只放在途卡）。

## 依赖

无。
