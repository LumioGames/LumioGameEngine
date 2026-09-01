---
name: 2026-09-01-ecs-formal-entity-chat-room-review-prompt
description: RM-00011 需求室审查的派活提示词——只审需求与架构、不实现;复用该审查流程时查
metadata:
  type: doc
  status: 已交付
---

# ECS Formal Entity and Chat Requirement Room Review Prompt

将以下提示词交给 Review Agent，要求它只做需求与架构审查，不实现代码、不修改 Workflow 对象：

```text
你是 LumioGameEngine 的需求与架构评审 Agent。请审查 Workflow 需求室 RM-00011「ECS Formal Entity and Chat Vertical Slice」及其 11 张需求卡 R-00344 至 R-00354。

审查材料：
1. ../reviews/2026-09-01-ecs-formal-entity-chat-decision-log.md
2. ../knowledge/features/ecs-entity-chat.md
3. Workflow 需求室 RM-00011 及其全部需求正文、验收项、依赖关系和当前状态
4. 相关前置需求：R-00149、R-00150、R-00152、R-00172、R-00178、R-00189、R-00212、R-00218、R-00231、R-00240、R-00247、R-00272、R-00279、R-00295

已知事实：
- 主场景是 100 个 Bot 客户端 + 1 个 Browser 客户端。
- Game Server 应创建 100 个 BotEntity + 1 个 PlayerEntity，共 101 个 Game ECS Entity。
- Account Server 必须存在；Bot01-Bot100 在请求时 login-or-register，默认测试密码为 123456。
- Bot 加数字的账号命名空间只能由 Bot 工具注册或声明，普通客户端不能抢占。
- AccountId 是持久业务身份；NetEntityId 是永不复用的运行时实体引用。
- 断线后服务器继续运行，实体保留 5 分钟；重连是重新登录和完整握手；过期后销毁 A，后续使用同一 AccountId 创建新的 B。
- ChatComponent 是第一个 ECS 组件；ChatInput 只包含文本；SetMessage 在 Simulation Owner Thread 的固定 Tick 中提交并产生有序 ChatMessageEvent。
- RM-00010 保持 archived，不创建新里程碑。
- 当前线上已写入 RM-00011、11 张需求卡和 39 条验收项；23 条 Requirement reference 边因平台 bindRequirementReference 返回 HTTP 500 尚未完成，不能把它们当作已建立关系。

请从以下方向重新判断整个需求室是否合理：

A. 总目标与范围
- 目标是否清晰、可交付、可验证？
- 是否仍然属于一个合理的 Hello World 后第一条 ECS vertical slice？
- 是否把过多长期能力（账号、身份、查询、复制、重连、持久化、Timer、E2E）压进同一阶段？
- 是否存在未声明的隐含目标、歧义或相互冲突的决策？

B. 需求卡正确性
- 逐卡检查 R-00344 至 R-00354 的标题、背景、目标、前置、要求、验收和边界是否一致。
- 判断每张卡是否能由一个明确责任方独立交付和验证。
- 检查是否有重复卡、遗漏卡、职责重叠、职责空洞、把实现细节误写成产品需求，或把前置条件误当成当前交付物。
- 检查 AccountEntity、PlayerEntity/BotEntity、AccountId、NetEntityId、LocalEntityId、ConnectionGeneration 的边界是否稳定。
- 特别检查「Bot 命名空间只能由 Bot 工具注册」是否在 Account Server 和 Game Server 两侧闭环，是否存在仅靠用户名即可伪造 BotEntity 的漏洞。

C. 验收质量
- 检查 39 条验收项是否客观、可执行、无重复、无不可测的形容词。
- 每条验收项是否有明确输入、状态变化、输出证据和失败结果？
- 101 Entity、固定 Tick、可靠有序 Chat、Attribute Query、5 分钟重连、过期销毁、Snapshot/Restore 是否都有可复现实验？
- 是否误把「计划存在」或「接口已定义」当作「功能完成」？
- 是否需要删减、合并或新增验收项？

D. 依赖与排期
- 审查 Wave 1 到 Wave 5 是否符合真实技术依赖。
- 判断哪些卡可以并行，哪些必须等待公共契约、Schema、ABI、Fixture 或前置实现。
- 检查当前 conditional 卡是否应该继续保留为规划卡，还是应先拆出公共接口/契约卡。
- 检查是否存在环依赖、缺失前置、错误方向或把 implementation 顺序误写成 interface 依赖。
- 对每条建议给出明确的 upstream -> downstream 关系和理由。

E. 风险与非目标
- 检查五分钟保留、进程重启、AccountId 与实体生命周期、tombstone/non-reuse、跨 Room 隔离、权限与可见性是否足够明确。
- 检查 Chat 是否错误承担历史、审核、离线投递或持久化职责。
- 检查是否遗漏容量、速率限制、失败恢复、幂等、观测性或安全边界；只指出本阶段必须补的内容，避免扩张范围。
- 将高风险但未冻结的 Account Auth/Profile、Entity Query、Chat mapping、Native Timer ABI、Room snapshot key 单独列为决策门，不要擅自替项目做最终决定。

输出要求：
1. 先给结论：`通过`、`有条件通过` 或 `退回重规划`，并说明是否建议继续按当前需求室开工。
2. Findings 按 P0/P1/P2/P3 排序，每条包含：问题、证据（Room/Requirement/验收项/前置卡）、影响、建议动作。
3. 单独给出「应删除/合并/拆分/新增」清单，逐条指向具体 Requirement。
4. 单独给出修订后的目标句、推荐的卡片分组和 Wave 顺序；若认为现有安排合理，要说明理由。
5. 列出必须由产品/架构负责人确认的决策问题，最多 7 条。
6. 不修改线上对象、不创建 WorkItem、不流转状态；只输出审查报告和建议的变更清单。
7. 明确区分：已验证事实、需求缺口、推测和建议；不得把当前 23 条未写入的引用边描述为已完成。
```
