---
name: 2026-09-04-rm-00011-r4-overall-review-prompt
description: RM-00011 r4 六仓整体进度与收口复核提示词——按 Workflow、origin/main、17 项 Fixture 和前置关系判定真实完成度
metadata:
  type: prompt
  status: 设计中
---

# RM-00011 r4 整体 Review 提示词

你是 RM-00011 r4 的独立 reviewer。目标不是复述各仓提交说明，而是判断当前实现是否满足 ADR-056、ADR-057、ADR-058 及 r4 卡片的字面验收条件。你只读验证，不改代码、不改验收尺子、不替任何仓补实现、不把 `done/100` 或“测试通过”当作充分证据。

## 1. 权威来源与时间点

按以下优先级取真值：

1. Workflow `lumiogamesengine` 中 RM-00011 的 R4 卡片与验收项状态。
2. 各仓 `origin/main` 的实际提交与可复核命令输出。
3. 架构仓 `.spec/decisions/ADR-056-rm00011-architecture-convergence.md`、`ADR-057-rm00011-r4-owner-rulings.md`、`ADR-058-ecs-world-manager-and-annotation-registry.md`。
4. 架构仓 `.spec/knowledge/features/ecs.md`、`ecs-entity-chat.md`，以及 `plans/2026-09-03-rm-00011-r4-blueprint.md`、`r4-cards.md`。
5. 各卡 handback、评审记录和日志；它们只能作为证据或偏离说明，不能替代前四项。

审计日期：2026-09-04。复核必须记录每个仓当时的 `origin/main` SHA，不使用本地未推送分支作为“已交付”证据。

## 2. 当前 Workflow 基线

先读取 R-00384…R-00393 的状态、progress、验收项和评论，并把读回结果原样摘要到报告：

| 卡片 | 领域 | 当前读回状态 |
|---|---|---|
| R-00384 / R4-01 | Arch 契约 | `done / 100` |
| R-00385 / R4-05 | Runtime ECS | `done / 100` |
| R-00386 / R4-07 | NativeCore Timer | `done / 100` |
| R-00387 / R4-03 | Server Bot 注入清理 | `done / 100` |
| R-00388 / R4-02 | Server 自驱主循环 | `in_progress / 50` |
| R-00389 / R4-04 | Client World / Bot.Host | `done / 100` |
| R-00390 / R4-06 | Game / oracle | `done / 100` |
| R-00391 / R4-08 | 多仓清理 | `backlog / 0` |
| R-00392 / R4-09 | 11 场景集成 | `backlog / 0` |
| R-00393 / R4-10 | 独立深审 | `backlog / 0` |

不要把这张表直接当结论。报告同时给出：

- Workflow 加权进度：`(6 * 100 + 50) / 10 = 65%`。
- 交付完成度：只统计已在目标仓 `origin/main`、有可复核证据且没有未解决 P0/P1 的卡。
- 最终目标完成度：R4-09、R4-10 未通过前，不能标为完成；R4-02 的 handback 仍是 `DONE_WITH_CONCERNS`。

## 3. 仓库与已知提交入口

核对以下目标仓 `origin/main`，并标注“已合入 / 仅分支 / 未开始 / 有风险”：

| 卡片 | 仓库 | 当前应检查的入口 |
|---|---|---|
| R4-01 | `LumioGameEngineArchitecture` | R-00384 合入主线的 C-1′/C-2′ 与 ADR 附录 |
| R4-05 | `LumioGameRuntime` | `7f198e5`；另有 `origin/feat/r-00385-r4-05-single-world-r2` 的 `3e52258`、`13a52d2`，必须判断其未合入修正是否影响验收 |
| R4-07 | `LumioNativeCore` | `70b9834` |
| R4-03 | `LumioServer` | `8ba3fe3`；检查 Server 是否仍有 `DOTNET_STARTUP_HOOKS` 或第二 oracle |
| R4-02 | `LumioServer` | `be9c28d` 仅在本地 handback 分支，未 push、未开 PR、未写 Workflow 证据 |
| R4-04 | `LumioClient` | `1473cc9` |
| R4-06 | `LumioGame` | `a080edf` |
| R4-08 | 多仓 | 尚未发现可审计的独立交付分支 |
| R4-09 | `LumioGame` + `LumioServer` | 尚未发现集成交付分支或入库日志目录 |
| R4-10 | 架构仓 | 尚未发现独立深审报告 |

上述 SHA 是审计入口，不是预先判定；若 `origin/main` 已变化，使用新 SHA 并说明变化。

## 4. 必查的 17 项 Fixture

逐项给出命令、关键输出、结论（通过 / 部分 / 不通过 / 无法复核）和引用路径。任何一项只凭文件存在、单元测试、发送计数或常量字段都不得判通过。

### ADR-056 六项

1. 依赖方向与宿主无绑定表：Runtime 是唯一绑定/查询实现，Server/Client/Game 不自建第二份表。
2. 标注生成：组件声明是唯一真源，注册表、同步表、C-2 声明表由同一生成命令产出且零 diff。
3. 广播与两轮一致：同 Tick 输入按发送者 `NetEntityId` 排序，事件顺序和 `appliedTicks` 两轮逐位一致；oracle 读取真实 Server/Client 日志。
4. 快照：进程 A 落盘，进程 B `CreateFromSnapshot`，101 个实体逐实体比对 `ChatComponent.lastMessageText`，身份不变。
5. 定时：生产 owner loop 自驱 `Tick` 与 wall-clock pump；五分钟过期由内核回调触发；生产没有 `advance_ms` 后门。
6. 顶号：Runtime 返回 `account_already_online`，旧连接先收到 `ConnectionSuperseded` 再关闭，客户端回登录界面且不自动重连。

### ADR-057 四项

7. 两轮一致仍是顺序一致，不得改成多重集比较。
8. 证据只认结构化 Server/Client 日志，日志可拉取，oracle 对行尾归一化字节自校验 SHA。
9. Bot 行为确实在 `LumioClient` 的 `Lumio.Client.Bot.Host` 进程中执行；Server 不注入启动钩子。
10. Server 自驱主循环和 Runtime Tick 来源真实成立，不能由 harness 假推进。

### ADR-058 七项

11. 单进程单 World Manager 单 GameWorld；WorldEntity 由游戏声明且唯一。
12. `Sync<T>`、`SyncList<T>`、`SyncDict<K,V>` 和字段钩子语义成立；共享文件不能放普通状态字段。
13. EntityType 使用 `abstract class` 与 C# 继承；`TypeOf(id).Is<Base>()` 对子类型成立；不把类型编码进 ID。
14. 客户端使用同一 `WorldManager.Create(GeneratedRegistry.Instance)`，不传 `instanceId`；创建记录按 `Awake → PostAttribute → Start` 生效。
15. `NetEntityId` 是世界实例 ID + 计数器的 128 位值；快照包含身份表和发号器状态，恢复后不复用。
16. 同进程双端是两个 Manager + 内存环回，不共用一个 World；事件是 `[ClientRpc]`，聊天窗口归 UI 层。
17. 绑定、查询、聊天、存档均通过同一个 World Manager；不存在 `_values`、第二个 ChatComponent、无界事件历史或模块自建世界。

## 5. R4-02 handback 的单独复核

把 R-00388 的 `DONE_WITH_CONCERNS` 当作未完成交付，逐条核对：

- `be9c28d` 是否已推送到远端、是否有 PR、是否有 Workflow 评论和验收项回写。
- Windows `cargo test`、`cargo fmt`、`spec-lint` 的输出是否可复核；不要把缺少 `LUMIO_GAME_ROOT` 的 11 场景当作通过。
- `clippy` 在未改 `bots.rs` 上的既有失败是否仍存在。
- Ubuntu/macOS 编译是否只有 CI 配置，还是已有真实运行输出。
- HostEntry 活体 CLR Boot、R4-09 常驻 Tick、S10 多房间后置是否明确标为风险/延后，而不是隐含为通过。
- C-1 `entity.identity` 的 128 位身份编码缺口是否已经被 R4-01/Runtime/Client/Game 全链路解决。

## 6. 输出格式

报告写入 `.spec/reviews/2026-09-04-rm-00011-r4-overall-review.md`，包含：

1. 一页结论：Workflow 进度、真实交付进度、最终目标进度。
2. Findings，按 P0/P1/P2 排序；每条必须有仓库、SHA、文件/行号、复现命令和影响。
3. 十张 R4 卡逐卡状态表：Workflow 状态、目标仓 `origin/main`、证据、阻塞、下一动作。
4. 17 项 Fixture 逐项结论与证据链接。
5. 前置 DAG 是否满足；明确 R4-02 → R4-09 → R4-10 的门禁。
6. 清理建议：可删除的 worktree/分支与必须保留的未交付分支，不能直接删除有未提交改动的目录。
7. 最终处置建议：放行、退回某卡、或保持 blocked；若 ADR-058 仍未满足条件，不得把 ADR-057/058 改成 Accepted。

## 7. 禁止事项

- 不改代码、契约、验收项、Workflow 状态或 oracle。
- 不用本地未推送 commit 证明交付，不用 `cargo check` 代替测试，不用“文件存在”代替行为证据。
- 不接受 `DONE_WITH_CONCERNS` 作为完成，不接受缺失日志、缺失跨进程读回或 harness 合成字段的“通过”。
- 不删除仍有未提交改动的 worktree；不删除 R4-05-r2、R4-02 等仍可能承载修正的分支。
