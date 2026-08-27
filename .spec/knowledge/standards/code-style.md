---
name: code-style
description: 代码与文档风格——语言约定、命名、注释原则、生成物纪律;写代码/建文档时查
metadata:
  type: doc
  status: 已交付
---

# 代码与文档风格

> 能交给工具（formatter / linter）强制的，优先交给工具；本文只写工具管不了、需要人 / Agent 判断的部分。

## 语言与文件命名（通用）

- **规范主体使用中文**；例外是根 `CLAUDE.md`、既有英文 Skill，以及从原架构目录迁入的英文公共 ADR。单份文档内保持语言一致，不做无语义收益的整篇翻译。
- 文件与目录命名一律 **kebab-case**；agent 文件 `<name>.agent.md`、skill 目录 `skills/<name>/`；公共 ADR 沿用 `ADR-NNN-<slug>.md`。

## 注释原则（通用）

- 注释只写**代码表达不了的约束**（为什么这样做、边界条件、外部依赖的坑）。
- 不写「改动说明」式注释（改了什么、为什么正确）——那是给评审人的话，进交回物或提交信息，不进代码。
- 注释密度、命名、习语向**周边既有代码**看齐。

## 生成物纪律（通用）

- 生成物不得手改，只能经生成源与生成命令更新，并与生成源一起提交（红线见 [`rules/system.md`](../../rules/system.md)）。

## 语言 / 框架特定风格

- 架构正文与 Agent 规范使用中文，公共协议名、Schema 字段、状态值、错误码和命令保持其精确英文拼写；Markdown 与 JSON/YAML 保持 LF（见根 `.gitattributes`）。
- 契约工具使用 Python 3，依赖只来自 `requirements-dev.txt`；Schema 使用 JSON Schema Draft 2020-12，JSON 文件必须保持确定性、可由 `tools/lumio_contract.py` 校验。
- Python 遵循周边代码的类型、命名与标准库习惯；新增依赖前记录许可证、供应链和为何不能用标准库/现有依赖，不能把未固定的第三方类型写进输出 Contract。
- Schema 文件、Fixture、ID Registry 和索引中的 ID/路径必须一致；已发布 ID 永不复用，弃用项保留可诊断状态。
- Serializer、ABI Header、Binding 等未来生成物不得手改，必须记录 Compiler/Input/Output Hash，并能从干净输入重建。
