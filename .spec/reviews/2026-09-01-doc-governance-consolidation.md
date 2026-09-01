---
name: 2026-09-01-doc-governance-consolidation
description: 文档治理收敛的审计证据与执行记录——四个文档根如何形成、删了什么、留了什么;复核本次收敛或再遇文档漂移时查
metadata:
  type: doc
  status: 已交付
---

# 2026-09-01 · 文档治理收敛 · 审计与执行记录

> 审计基线 `935a8a9`，执行基线 `2b7e321`（执行期间 origin/main 前进 19 个提交，重新对齐后重做）。
> 结论落点：全仓文档收敛为单一根 `.spec/`;各分区口径见 [`knowledge/README.md`](../knowledge/README.md) 与 [`.spec/AGENTS.md`](../AGENTS.md)「项目是什么」。

## 1. 病根：制度换了一半

2026-08-31/09-01 仓库从「架构源 + Baseline 门禁 + 七仓镜像」换成 Living Architecture：

| | 旧制度 | 新制度 |
|---|---|---|
| 仓库自称 | 唯一架构源、公共契约目录 | SDK 组装根、Native 聚合根 |
| 约束 | Baseline `LGE-V1.4` + ADR 门禁 + 七仓镜像 + 全量 Fixture | 明写「不要求 Baseline、七仓镜像或全量 Fixture」 |
| CI | `Architecture Policy`（校验 baseline/schema/fixture/packages/hash） | `LumioGameEngine Development`（build SDK + 证明 Host 装载） |

旧制度的**可执行部分**（`packages/` `tools/` `schemas/` `fixtures/` `ids/`）在 `59866ec` 已删，`.spec/` 也跟着改写了。**只有 `docs/` 原封未动**——于是两套互相矛盾的制度的文档平铺在同一棵树上，外观完全一样，没有任何标记能区分。

## 2. 为什么没人发现：`docs/` 是机器校验的盲区

`spec-lint.mjs` 的遍历起点只有 `.spec/`。实测（审计基线）：

- `docs/` 下 198 份 md，**带 frontmatter 的 0 份**——没有状态、没有 owner、没有时效字段，所以连「打个废弃标记」都无处下手。
- `docs/` 无顶层索引；7 个子目录里只有 `adr/` 和 `research/` 有 README。
- 治理层（根 README + `.spec/` + CI）"知道"的 docs 文件只有 68/181，其中 49 份是 ADR 兼容软链。**真正有内容的 132 份里只有约 19 份被提到过。**
- 具体漏检实例：`docs/adr/` 缺 ADR-045 软链无人发现（修复被挂在一个从未获批的 V1.5 批规划上）；`docs/research/README.md` 开篇写「五份调研」而实际有 7 个包。

## 3. 四个并行文档根

| 根 | 规模 | 被治理文档承认 |
|---|---|---|
| `.spec/` | 86 md | 是 |
| `docs/` | 198 md | 部分 |
| `.sdd/` | 53 md | **否** |
| `.workflow-drafts/` | 11 md | **否** |
| `engine/native/.spec/` | 完整第二套 LumioAgent 框架（含自己的 `lumio-agent.lock`） | **否** |

重复度：旧架构正文 10 份副本（`docs/architecture/` 5 + `engine/native/docs/architecture/` 5）；`ADR-001` 4 份副本；`lessons.md` 2 份；决策体系 5 套。

## 4. 决策落点曾散在 11 处

`.spec/AGENTS.md` 写死「决策**一律**记 `decisions/`——唯一落点」，实际散在 11 处，其中 6 处没有任何治理文档承认过。两处已经出事：

1. 配表定稿板 D.3 把 **Accepted** 的 ADR-034 适用面砍掉一半（「生产环境换幕机从 V1 砍掉」），未走 ADR。
2. GAS 定稿流水直接裁决 ADR 状态（「ADR-008/031 维持 Accepted」），该裁决只存在于 `docs/specs/`。

另有两套冲突的 `D-N` 编号命名空间（`DECISIONS_PENDING` 的 D-001~016 vs gate-p0 的 D-1~12），已导致一份文档标题混排两套编号。

## 5. 执行

### 迁移（19 份活文档）

沿用 `475393f` 定的约定——**活文档去日期前缀、历史记录留日期前缀**：

- `knowledge/features/`（7）：`architecture` `ecs` `ds-server` `gas` `save-load` `config-table` `ecs-entity-chat`
- `reviews/`（11）：四场定稿会流水、ECS 审计过账附录、接缝裁决流水、RM-00011 三份报告、Owner wire 确认件、架构收口记录
- `plans/`（1）：RM-00011 需求室审查提示词

全部补齐 frontmatter；20 处跨文档引用按新路径重写。**迁移当场暴露一条此前不可见的断链**（`architecture.md` → `engine/wire/hello-wire-v1.json`），因为 `docs/` 从来不被扫描。

### 删除

`docs/`(219) `.sdd/`(136) `.workflow-drafts/`(15)；`engine/native/` 内的第二套 LumioAgent 框架及其宿主入口（`.claude/` `.agents/`）、旧架构正文第二套副本、`.old-sync-4724` 临时残留、失效的嵌套 CI、重复的 `.workflow` 指针。

### 保留（一处必要的偏离）

`engine/native/generated/architecture/LGE-V1.4-2026-08-27/`（301 文件）**不删**——它不是文档副本，是 `modules/composition` 测试实际读取的 fixtures/schemas 底座（`tests/common/mod.rs`、`tests/config_parsing.rs`）。删了 `cargo test` 会红。

### ADR

ADR-001~048 标 `Historical · <原状态>`，其「`Accepted` 不可改写」约束随旧制度失效，以便就地修正失效引用（顺带解决了 ADR-002 引用一个 v1.3 就删掉的 fixture、制度上却修不了的死结）。**ADR-049 不打标**——它于 2026-09-01 在新制度下重新 `Accepted`，正文明写「pre-launch living-architecture wire contract, not a baseline event」。

NativeCore 的 9 条生效决策并入 `decisions/nativecore/` 子命名空间，**保留 `000N` 文件名**：`engine/native/` 下 51 个源码与配置文件以「ADR-0006」这类写法引用它们，改号要动 51 处代码注释、收益为零。

### 未决项不随文件消失

两条落成任务卡：八仓盘点的 `AUDIT_INCOMPLETE`（缺 Server 正式报告 + 两条 hard hold）、RM-00011 上传里唯一未重试的 500 引用边（`edge-source-account`，`traceId=ed48c86f-1659-4fc2-b58b-9bc520fd84d6`）。

### 防复发

`spec-lint` 新增三项，各配反例测试：

1. `plans/` `reviews/` 下每份文档必须有 frontmatter（沿用既有 status 枚举）
2. ADR 状态纳入枚举校验（中英两种写法都收）——此前 ADR 状态完全不受校验
3. **无并行文档根**：`docs/` `.sdd/` `.workflow-drafts/` 不得在仓内任何层级重现（历史病灶包括嵌套的 `engine/native/docs/`）；仓根之外不得有第二个 `.spec/`（防止 subtree 合并再带一套框架进来）。`subagent-driven-development` 技能的临时工作区因此从 `.sdd/` 改名 `.sdd-scratch/`，避开禁名单

## 6. 已知缺口

- NativeCore 决策的 `0005` / 部分 `0007` 编号未随迁移进入本仓（原目录即无对应文件）；曾指向它们的历史引用已随 `engine/native/` 第二套框架副本一并删除，编号空洞保留，需要时按新决策补号。
- `engine/native/generated/architecture/LGE-V1.4-2026-08-27/` 里仍有一份旧 ADR 快照。它是测试底座的一部分，随底座保留；不是文档入口，不被任何治理索引引用。
- 仍保留的 `docs/` 字样均为对废止制度的元引用（禁令声明、删除记录）或非本仓路径（LumioClient 的 `docs/spikes/`、git 分支名），无一是本仓活路径。

## 7. 验证

```text
node .spec/tools/spec-lint.mjs                     → OK
node --test .spec/tools/spec-lint.test.mjs         → 23/23 pass
node eng/generate-abi.mjs                          → exit 0，无生成物漂移
cargo test -p lumio-engine-native -p lumio-core-composition
                                                   → 49 passed / 0 failed
```

`composition` 全绿即证明第 5 节保留的镜像底座未被破坏。
