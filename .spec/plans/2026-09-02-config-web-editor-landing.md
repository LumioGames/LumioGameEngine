---
name: 2026-09-02-config-web-editor-landing
description: LumioConfig M6 网页表格编辑器落地方案——选型裁决、架构、提交/合并机制与分卡派活顺序;派 M6 卡或改编辑器边界前查
metadata:
  type: doc
  status: 实施中
---

# LumioConfig 网页表格编辑器（M6）落地方案（2026-09-02）

> 输入：调研稿《LumioConfig 网页在线表格编辑器最终技术方案与实施指引 v1.0》（Owner 提供，建议 Univer OSS + React + Rust Host）。本文是对照 LumioConfig 仓现状核对后的**落地定稿**：选型裁决、架构、提交与合并机制、分卡与派活顺序。每张卡的逐模块实现指引在 LumioConfig 仓 `docs/plans/2026-09-02-web-editor-design-prompt.md`（v2），本文不复述。
> 不变的底线（[`config-table.md`](../knowledge/features/config-table.md) 与 LumioConfig `docs/decisions/0-1`）：TXT/Schema/墓碑是唯一真身；编辑器保存 = 相对打开基线的逐格补丁走 M2；补丁只写 name 不写 id；没有上线按钮；Excel 不是源。

## 1. 与调研稿的四处偏差及裁决（Owner 2026-09-02）

| 偏差 | 仓库现状 | 裁决 |
|---|---|---|
| Host 语言 | Canonical Core 已是 Python（`src/lumio_config/`，M1 全部 + M2/M3 主体 + M4 投影落地，pyproject 只允许标准库，31 测试通过） | **Python**：`tools/lumio_config.py serve`；不建 Rust Host、不写第二套解析器（调研稿 Kill Criteria 第 8 条反向成立） |
| 表格内核 | 未提交的 R-00330 设计稿走「零依赖单文件 HTML」 | **Univer OSS + React + Vite**（`@univerjs/preset-sheets-core` 0.25.1，Apache-2.0，14 个依赖全在 `@univerjs/*`，无 pro 包）；Node 只进前端 `editor/` |
| 提交到版本库 | `apply_patch` 只写文件、无基线校验、不 commit | **设置项**：自动/手动 commit、自动/手动导出；版本库抽象 **Git / SVN / none** |
| 卡片 | R-00330 一张卡装全部 | R-00330 收窄为 Univer POC + 只读投影；新增 M6-A~E 五张卡 |

默认取舍（Owner 未反对即生效）：草稿存仓内 `.lumio/drafts/`（gitignored）；首版导出只做 CSV/TSV，XLSX 推迟到出现 Excel 交换需求；前端构建产物 `src/lumio_config/editor_static/` 随源提交、CI 重建 `git diff --exit-code` 校验；M6 决议落 LumioConfig `docs/decisions/0-7`、`0-8`，并行期不占架构仓 ADR 号。

## 2. 架构（定稿）

```
LumioConfig
├── src/lumio_config/              Python Canonical Core（已有）
│   ├── patch.py                   M2：扩 cell 级三方合并（归 R-00322）
│   └── editor/                    新增包：server / session / drafts / vcs / settings
│       └── ../editor_static/      前端构建产物（随源提交）
├── editor/                        前端源：Vite + React + Univer；src/{app,spreadsheet,panels,api}；tests
├── .lumio/                        drafts/（ignored）· editor.json（仓级设置，入库）· local.json（个人，ignored）
└── docs/decisions/0-7 · 0-8       M6 决议
```

所有权：TXT 语法 / 四态 / 指纹 / 补丁合法性 → Python Core；工作簿交互 / Undo-Redo → Univer；会话 / 基线 / 草稿 / 冲突 → Python Host；视图状态（列宽、冻结、筛选、排序、隐藏列）→ 浏览器 `localStorage`，永不进补丁；新行终身编号 → M3。

Univer 接入三件套（全在 `editor/src/spreadsheet/`）：`projection.ts` 把 Host 的表 JSON 投影成 `IWorkbookData` 并在内存建 `ProjectionMap`（rowIndex → stableRowId | draftRowKey，colIndex → columnName）；`interceptors.ts` 拦公式、合并单元格、插删列、改 `id` 列，粘贴含公式只取值，插行分配 `draftRowKey`；`extract.ts` 保存时按 `ProjectionMap` 取语义值、四态化、与打开基线逐格 diff 出补丁 ops。**永不 diff 两份 IWorkbookData，永不把行号或 `cell.custom` 当身份。**

## 3. 提交与合并机制

**基线 = 底稿指纹（按表），不用版本库修订号**，Git / SVN / none 行为一致。补丁在 `0-6` 格式上扩可选字段：

```json
{"table":"skills",
 "base":{"sourceFingerprint":"<打开时该表底稿指纹>"},
 "ops":[{"op":"update","name":"fireball","set":{"damage":130},"expect":{"damage":"120"}}]}
```

M2（`patch.py`，AI 与编辑器共用，归 R-00322）：当前指纹 == base → 直接 apply；否则逐格三方——current == base 采 draft；draft == current 无操作；三者互异报 `STALE_BASELINE`（携 base / current / draft 三值）；行已被删而 draft 改它报 `DELETED_ROW_CONFLICT`。比较对象是四态 Cell 不是显示串。冲突解决后必须重跑完整 validate + apply。

**提交到版本库 = 设置项** `.lumio/editor.json`（个人覆盖 `local.json`）：`vcs: git|svn|none`、`submit.autoCommit`、`submit.autoExport`、`export.outDir`、`allowDirtyWorkingTree`。autoCommit 时 Host 在 apply 成功后按白名单路径 commit（message = 人话摘要 + 补丁 JSON），界面提交前显示目标分支 / 工作副本与变更摘要；打开时 `tables/ registry/ schemas/` 有未提交改动默认拒绝（`working_tree_policy_violation`）。前端不拼任何版本库命令。

## 4. 分卡（Workflow RM-00009，module = LumioConfig；2026-09-02 已落单，蓝图 `lumioconfig-m6-editor-20260902/r1`）

| 卡 | 内容 | 前置 | 文件集 |
|---|---|---|---|
| R-00322（扩） | 机器门 + cell 级三方合并、`base` / `expect` 字段、`STALE_BASELINE` / `DELETED_ROW_CONFLICT` | R-00321 | `patch.py` `validate.py` `fingerprint.py` `tests/` `docs/reference/` |
| R-00330（收窄） | Univer POC + 只读投影：`editor/` 脚手架、projection / interceptors、四态显示、10k×50 基准、Playwright 冒烟；数据用静态 fixture，**不碰 Python** | 无 | `editor/` |
| M6-A（R-00360） | Host 会话与安全 + 设置 + VcsAdapter（status / revision） | R-00322 | `src/lumio_config/editor/` `cli.py` |
| M6-B（R-00361） | 编辑与草稿：四态操作、类型编辑器、draftRowKey、Draft Store、恢复、多标签版本 | R-00330 M6-A | `editor/src/` `editor/drafts.py` |
| M6-C（R-00362） | 语义提取 → 带 base 补丁 → validate / apply → autoCommit / autoExport；VcsAdapter.commit（git / svn / none） | M6-B R-00322 | `extract.ts` `vcs.py` `server.py` |
| M6-D（R-00363） | 冲突面板 + 修订监视（SSE）+ Stale 状态机 | M6-C | `editor/src/panels/` `session.py` |
| M6-E（R-00364） | CSV/TSV 导出、`editor_static` 内嵌、CI 可复现构建、E2E 矩阵、`docs/reference/editor.md` | M6-D | `.github/workflows/` `editor_static/` `docs/` |

不在本批（触发条件写进 `0-7`）：技能卡视图、XLSX 导出、领域插件 API、多人协作。

## 5. 派活顺序

Python 主线 R-00321 → 322 → 323 → 324 → 331 → 329 串行；R-00330 文件集仅 `editor/`，与主线不重叠，**作为「同仓串行」的显式例外并行**（独立 worktree，Owner 已知）；随后 M6-A → E 串行；最后 R-00326、R-00327。每卡一分支一 PR，reviewer 放行后合 main；派活骨架按 [`cross-repo-delivery`](../skills/cross-repo-delivery/SKILL.md)，卡正文按 LumioConfig 设计提示词 v2 的对应节粘贴。

## 6. 验收硬指标（每张 M6 卡收口时逐条对）

- 无修改打开再提交 → 空补丁；仅视图操作（排序、筛选、冻结、列宽、隐藏列、缩放）→ 空补丁。
- 四态在打开、编辑、复制、粘贴、填充、删除、撤销、重做、草稿恢复、提交十个动作下无损。
- AI 并行改不同表 / 同表不同格 → 自动合并；改同格 → 结构化冲突不丢数据；AI 删除用户正在改的行 → 删除/修改冲突；AI 改名 → 稳定 id 仍识别。
- 新行先有 `draftRowKey`，提交后由 M3 发号；改名不换 id；复制行不复制 id。
- 非 loopback、无 token、错误 Origin、路径穿越一律拒绝；前端不能执行任何版本库命令。
