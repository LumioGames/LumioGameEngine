---
name: cross-repo-delivery
description: 架构仓主会话向七个实现仓提需求、按 wave 派活、核验交回物、做阶段聚合验收时使用;实现仓上报公共契约缺口需要回路处理时也用。
---

# Cross-Repo Delivery（跨仓需求驱动交付）

把架构文档的子模块地图变成 Workflow 需求卡，按 wave 派子 Agent 到各实现仓并行开发，核验交回物后自动流转，阶段级用集成卡聚合验收。本技能只定义**跨仓协作机制**；单仓内怎么开发以目标仓自己的 `.spec/` 为准，Workflow API 怎么调以 `workflow-planning` / `workflow-ops` 技能为准，此处不复述。

## 前置条件

- Workflow Agent 插件已安装（`workflow-setup` / `workflow-planning` / `workflow-ops` 技能可用），本仓 `.workflow` 已绑定 lumiogamesengine 项目，凭证经环境变量解析可用（未绑定先走 `workflow-setup`）。
- 七个实现仓在本机 `~/LumioGames/<仓名>`，各仓 `repository-policy` CI 可用。

## 角色与真值

- **总调度 = 本架构仓主会话**。只有总调度持有 Workflow 凭证并读写 Workflow（单一写入方）；派出的子 Agent 不碰 token，一切状态经会话内交回物回报。
- **跨仓需求真值在 Workflow**（lumiogamesengine.workflow.games）：一张卡 = 一个实现仓可独立交付、独立验证的成果。仓内执行粒度真值仍是各仓自己的任务机制，不回写 Workflow。
- **公共契约真值在本仓**：Baseline / Schema / ID / Fixture 的变更只走[变更顺序](../../knowledge/standards/repository-architecture.md)，任何实现仓不得自行改镜像。

## 卡片结构（落单时必须齐全）

| 字段 | 口径 |
|------|------|
| module | 目标仓库名（LumioNativeCore 等七仓之一；集成卡归验证平面所在仓） |
| 标题 | `[仓库名] 成果描述`，与 module 一致 |
| 正文 | 自包含 Agent 提示词（不依赖原对话）+ 结构化验收项 |
| BaselineId | 卡片开发所基于的架构基线（如 `LGE-V1.4-2026-08-27`），正文首行声明 |
| wave | 并行批次号；同 wave 各卡目标仓互不相同（天然文件集不重叠），同仓多卡必落不同 wave |
| 前置 | 依赖的上游卡（按源码依赖图 / Generated Artifact 图判定） |

蓝图生成与落单走 `workflow-planning`（含其全部硬闸门）；需求来源默认是架构文档 §16「子模块地图」+ §16.1「阶段退出条件」，每阶段一个 Requirement Room。

## 状态映射

| Workflow 状态（按项目工作流现查） | 开发阶段 | 谁触发 |
|------|------|--------|
| 初始态 | 已落单待派 | workflow-planning 落单 |
| 开发中 | 子 Agent 已派出 | 总调度派活前流转 |
| 待验收 | 交回物核验通过、已合入该仓 main | 总调度核验后流转 + 证据评论 |
| 完成 | 卡级验收（全自动）| 总调度流转；阶段汇总报告是人的唯一触点 |
| 退回/重开 | 核验不过或契约变更波及 | 总调度附核验报告流转 |

状态名词表以项目实配为准，流转前先查 transitions 端点（`workflow-ops` 口径），不硬写枚举。

## 操作步骤

### 流程 1 · 提需求

1. 读架构文档 §16 + §16.1 与当前 Baseline，用 `workflow-planning` 生成阶段蓝图（Room + 分 wave 卡片，含集成卡）。
2. 蓝图展示后取得用户一次性写入授权，落单并逐卡读回。

### 流程 2 · 派活（wave 扇出）

1. 取当前 wave 全部卡，确认前置卡已完成；把卡流转到「开发中」。
2. 每卡派一个后台子 Agent，工作目录 = `~/LumioGames/<目标仓>`。派遣 prompt 在 [dispatch.md](../../knowledge/standards/dispatch.md) implementer 骨架上补：

```text
【目标仓】~/LumioGames/<仓名>。进场先读该仓 AGENTS.md 指向的 .spec 三件套,全程守该仓规范。
【任务卡】<Workflow 卡正文全文:自包含提示词 + 验收项 + BaselineId>
【边界】只动本仓文件;镜像 / 公共契约(schemas、ids、fixtures 镜像)一律不改,
发现契约缺口 → 停下,交回物标 BLOCKED + 缺口描述,不得本地绕过。
【执行口径】按该仓 .spec 流程的 Inline Fallback 执行(子 Agent 不得再派生子 Agent,
同上下文自审属已知降级);TDD 与该仓收口门槛照常强制。
【交回物】按该仓 AGENTS.md 交回物格式,另附:分支名与提交号、收口门槛命令的实际输出。
```

3. 同仓串行、异仓并行；wave 内全部交回后才进下一 wave。

### 流程 3 · 核验与流转（卡级，全自动）

逐卡过核验清单，全过才流转「待验收 → 完成」：

- [ ] 改动清单与目标仓实际 diff 一致（`git -C ~/LumioGames/<仓> diff/log` 抽查，不信口头声称）。
- [ ] 验证证据是实际命令输出：该仓收口门槛 + CI 等价检查；关键声称在目标仓重跑复核。
- [ ] 验收项逐条对得上；known gaps 已列明且不含 P0/P1。
- [ ] 短分支已合入该仓 main 并 push（授权口径见下），该仓 `repository-policy` CI 绿。

通过 → Workflow 流转 + 证据评论（含分支/提交号、验证输出摘要）。不过 → 退回：卡流转回「开发中」，附核验报告重派；**同一卡三次不过 → 停，升级用户**。

### 流程 4 · 契约变更回路

子 Agent 交回 BLOCKED + 契约缺口时：

1. 总调度在本仓按[变更顺序](../../knowledge/standards/repository-architecture.md)走 ADR → Schema/ID → Fixture → README/Baseline → 受影响仓镜像同步。
2. 在 Workflow 重开 / 更新受影响卡（更新 BaselineId 与提示词），再派。
3. 契约变更本身按本仓收口门槛与审查闭环交付，不走快速模式。

### 流程 5 · 聚合验收（阶段收口）

1. 阶段各 wave 全绿后执行集成卡：消费各仓**版本化 Artifact**（按 Generated Artifact 图），验证平面用 CoreEngine `smoke`；不做任何跨仓代码合并。
2. 对照架构文档 §16.1 该阶段退出条件逐条核验，产出**阶段汇总报告**给用户：各卡清单与证据链接、集成验证输出、退出条件核对表、known gaps。这是全自动模式下人的唯一验收触点。

## 持续授权口径（全自动模式的前提）

用户已授予本流程内的持续授权，范围仅限：

- 各 Lumio 实现仓短分支**合入本仓 main 并 push**（前提：该仓收口门槛过 + 核验清单全过）。
- 已授权蓝图所建卡片的 Workflow **流转 / 评论 / 附件**。

以下仍需逐次确认，授权不延伸：新蓝图落单、删除 Workflow 对象、发包 / 对外发布、生产环境操作、改各仓 CI 与访问控制。

## 注意事项（Pitfalls）

- **单一写入方不破例**：子 Agent 报「我顺手把卡流转了」= 核验不过。
- **Baseline 漂移先清**：派活前确认目标仓镜像与卡片 BaselineId 一致，不一致先走镜像同步再派。
- **集成不是合并**：聚合只消费版本化 Artifact；出现「把 A 仓代码拷进 B 仓」即停。
- **同 wave 同仓 = 拆错了**：回 `workflow-planning` 重排 wave，不靠运气串行。

## 验证

- 每张完成卡在 Workflow 上有证据评论（分支/提交号 + 实际验证输出）。
- 阶段汇总报告的每条退出条件都有对应命令输出或卡片链接，无裸声称。
- 本仓 `node .spec/tools/spec-lint.mjs` 通过（技能结构合规）。
