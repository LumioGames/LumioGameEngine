---
name: workflow
description: 开发工作流——分支/提交/合并·PR 与知识同步义务;动手改代码、开 PR 前查
metadata:
  type: doc
  status: 已交付
---

# 开发工作流（分支 / 提交 / 合并）

> 本文是“开发这件事**怎么做**”的手册。Agent 之间**怎么协作**（拆解 → 实现 → reviewer 对抗审查 → 收口）在 [`AGENTS.md`](../../AGENTS.md) 的「调度核心」与「编码约定」里，不在这里。
> “禁止碰什么”的硬性护栏在 [`rules/system.md`](../../rules/system.md)；本文只描述流程，遇到护栏处**引用**它，不重复定义。

## 分支策略

- `main` 是发布给七个实现仓库的架构基线；Architecture Policy 在 `push` 与 `pull_request` 到 `main` 时执行。
- 架构与公共契约改动使用短生命周期分支并经 PR 合入 `main`；push/PR 等对外发布动作仍受 [`rules/system.md`](../../rules/system.md) 的确认要求约束。
- 多文件契约变更按 ADR -> Schema/ID -> 正向/失败 Fixture -> README/基线 -> 实现仓镜像组织，不允许只改镜像或只改生成结果。

## 提交规范（通用）

- 格式：`type(scope): subject`，例如 `feat(agents): 新增 reviewer`、`fix(skills): 修复 TDD 步骤`。
- 常用 type：`feat` / `fix` / `refactor` / `chore` / `docs` / `ci`。scope 可省略。
- **一次提交只做一类事**；文档、脚手架、功能、测试修复不混在一起。
- 提交前自检：验证命令通过（见 `AGENTS.md`「收口门槛」与 `rules/system.md`）、无调试残留、知识已同步（见下节）。
- 机器兜底：Claude Code 宿主经入库的 `.claude/settings.json` hooks 在 `git commit` 前自动跑结构校验，未过即阻断（known gap：仅 Claude Code 生效，Codex 等宿主无机器兜底，依赖上一条自检自觉执行；「reviewer 通过前不得提交」机器不可判，同属自觉项，红线见 `rules/system.md`）。

## 合并 / PR 流程

- PR 交付说明必须包含 Summary、公共语义变化、兼容/迁移影响、Schema/ID/Fixture 清单、验证命令与关键输出、七仓同步范围和 known gaps。
- 根 `.github/CODEOWNERS` 将全仓及 `.spec/` 指派给 `@Go1c`；所有架构变化必须由架构所有者审查。
- 预上线合并前运行 SDK 开发态收口命令，证明 Host 加载最新 BuildId；完整 Contract validate、Baseline 和 reviewer 闭环仅在正式硬化阶段启用。

## 改动完成 = 知识已同步

一处改动只有在**知识沉淀完成**后才算 Done：用 `spec-steward` 技能更新对应 `knowledge/` 文档、`status` 与 `knowledge/README.md` 导航（交付历史在 git，不进文档）。豁免口径与 `AGENTS.md`「编码约定」的交付标准一致：纯修复 / 机械套用既有模式可豁免，但**豁免必须在交回物里声明**，不得静默跳过。

## 相关

- 验收与测试：[`testing.md`](./testing.md)
- 注释与命名：[`code-style.md`](./code-style.md)
- 护栏（禁止项）：[`rules/system.md`](../../rules/system.md)
- 沉淀方法：[`skills/spec-steward`](../../skills/spec-steward/SKILL.md)
