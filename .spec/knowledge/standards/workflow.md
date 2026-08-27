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

- `main` 是共享集成基线；现有 Repository Policy 在 `push` 与 `pull_request` 到 `main` 时执行。
- 需要协作或审查的改动使用短生命周期分支并合入 `main`。任何 push/PR/发包等对外发布动作仍受 [`rules/system.md`](../../rules/system.md) 的确认要求约束。
- 公共架构变更不从本仓直接发起：先在 `LumioGameEngineArchitecture` 完成 ADR、Schema、正向/失败 Fixture、Baseline 与契约校验，再同步本仓只读镜像。

## 提交规范（通用）

- 格式：`type(scope): subject`，例如 `feat(agents): 新增 reviewer`、`fix(skills): 修复 TDD 步骤`。
- 常用 type：`feat` / `fix` / `refactor` / `chore` / `docs` / `ci`。scope 可省略。
- **一次提交只做一类事**；文档、脚手架、功能、测试修复不混在一起。
- 提交前自检：验证命令通过（见 `AGENTS.md`「收口门槛」与 `rules/system.md`）、无调试残留、知识已同步（见下节）。
- 机器兜底：Claude Code 宿主经入库的 `.claude/settings.json` hooks 在 `git commit` 前自动跑结构校验，未过即阻断（known gap：仅 Claude Code 生效，Codex 等宿主无机器兜底，依赖上一条自检自觉执行；「reviewer 通过前不得提交」机器不可判，同属自觉项，红线见 `rules/system.md`）。

## 合并 / PR 流程

- PR 交付说明必须包含 Summary、改动清单、验证命令与关键输出、known gaps、知识沉淀落点或豁免声明。
- 根 `.github/CODEOWNERS` 当前将全仓指派给 `@Go1c`；涉及公共 ABI、Loader、平台包、签名或供应链的改动必须由对应所有者审查。
- 合并/发布前必须从干净来源重建并比对 Hash，通过 Repository Policy、收口门槛与 reviewer 闭环；不得发布手改生成物。

## 改动完成 = 知识已同步

一处改动只有在**知识沉淀完成**后才算 Done：用 `spec-steward` 技能更新对应 `knowledge/` 文档、`status` 与 `knowledge/README.md` 导航（交付历史在 git，不进文档）。豁免口径与 `AGENTS.md`「编码约定」的交付标准一致：纯修复 / 机械套用既有模式可豁免，但**豁免必须在交回物里声明**，不得静默跳过。

## 相关

- 验收与测试：[`testing.md`](./testing.md)
- 注释与命名：[`code-style.md`](./code-style.md)
- 护栏（禁止项）：[`rules/system.md`](../../rules/system.md)
- 沉淀方法：[`skills/spec-steward`](../../skills/spec-steward/SKILL.md)
