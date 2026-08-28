# 2026-08-28 · 七仓进度盘点与对账（第二轮）

> 盘点会话：`lumiogameenginearchitecture-stoic-golick`。**只读盘点，未做任何 Workflow 写入。**
> 测量时刻：**2026-08-28T12:21–12:28Z**。跨仓状态几十分钟即过期——本次盘点期间架构仓 `origin/main` 前进 4 笔。
> 前一轮报告：[`2026-08-28-seven-repo-progress-assessment.md`](2026-08-28-seven-repo-progress-assessment.md)；待裁决项：[`2026-08-28-gate-p0-delivery-and-escalations.md`](2026-08-28-gate-p0-delivery-and-escalations.md)。

## 1. 执行摘要

**261 张需求卡**：`backlog` 130（49.8%）· `acceptance` 85（32.6%）· `in_review` 33（12.6%）· `in_progress` 6 · `done` 7（2.7%）。

七仓在过去 5 小时内全部有推送，多数在 30 分钟内。项目在高速推进，**不是停滞，是积压在验收环节**。

### 三条核心结论

1. **`acceptance` 那 85 张不是证据问题，是流转欠账。** 抽样核验 NativeCore 5 张 + CoreEngine 4 张 done 的证据评论，引用的提交 SHA **全部实测为 `origin/main` 祖先**。证据质量合格，缺的是有人去走验收。而下游卡的「前置满足」按**卡状态**判定（escalations D-7 已确认此原理），因此这 85 张正在实际挡住下游开工。
2. **LumioClient 是全项目最严重的 Workflow 漂移。** `origin/main` 上有 **11 个模块 / 242 个 .cs / 12,224 行 / 130 个测试实体**，而它在 Workflow 里只有 10 张卡，其中**没有任何一张实现卡**——4 张设计/计划（`in_review`）、1 张 manifest、1 张 audit、4 张 SPIKE（`backlog`）。12k 行产出全部兜在 R-00031 一张「Wave 0-6 基础实现」卡下。
3. **LumioServer 无漂移，48 张 backlog 是准确的。** 其 7,838 行里 **6,844 行是 xtask 治理工具**、950 行 testkit、24 行 generated 镜像，`modules/` 下的真实现只有 **44 行**。它先修护栏后写业务，卡状态如实反映这一点。

## 2. 各仓详情（数据全部取自 `origin/main` 已提交对象，未读任何仓的工作区）

| 仓 | origin/main | 源文件 | 源码行 | 测试实体 | 卡数 | 主状态 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| LumioVoxelEngine | `4ced801` | 141 | **25,041** | **186** | 53 | 25 in_review / 14 acceptance |
| LumioClient | `6b7e834` | 242 | **12,224** | 130 | 10 | 5 in_review / 5 backlog |
| LumioNativeCore | `03d6bd7` | 136 | 8,854 | 83 | 68 | **67 acceptance** |
| LumioServer | `5031feb` | 18 | 7,838 | 53 | 54 | **48 backlog** |
| LumioGameRuntime | `7b89dc9` | 29 | 2,064 | 22 | 31 | 26 backlog / 5 in_progress |
| LumioCoreEngine | `fa8c412` | 22 | 241 | 0 | 36 | **32 backlog** / 4 done |
| LumioGame | `9bc46ed` | 0 | 0 | 0 | 1 | 1 acceptance |
| 架构仓 | `c712ff4` | — | — | — | 8 | 3 done / 1 in_progress / 4 backlog |

### 2.1 LumioCoreEngine —— 4 张 done 的证据链是范本，但仓被架构源挡死

**证据核验通过（本次独立复现）**：R-00011/00012/00013/00014 的四路评论呈现完整闭环——原始证据 → 差异记录（引用的 commit 不在 origin）→ 推送后 macOS 独立复现并订正 SHA → 后续修复补记。实测：

```
订正后 SHA  48f109b / 0af3f63 / 28559aa / 11f5326 / d87e12e  → 5/5 均为 origin/main 祖先 ✅
失效锚点    015035b / d668426 / 06c954f / 68e1442            → 4/4 均非祖先（与评论所述一致）✅
```

241 行 / 0 测试**不构成问题**——这 4 张是 workspace 脚手架 + 3 份 ADR，本就低代码。

**但 32/36 backlog 卡在架构源。** 本次第三方独立复核 `architecture.lock.json`：

```
architectureBaselineId: LGE-V1.2-2026-08-27
commit:                 2d7980d95b163404e33cc6212db13ac948d30d40
requiredPaths:          131
```

与 escalations **D-2 的数字逐项吻合**（第三个独立来源确认）。`packages/` 不在 requiredPaths 内，CoreEngine 物理上无法消费 V1.4 的新产物。

### 2.2 LumioClient —— 12k 行产出在 Workflow 上不可见

11 个模块全部有实现：`bot / connection / handshake / hybridclr-adapter / input / observability / persistence / prediction / replica / session / unity-adapter`，`src/Public` + `src/Internal` + `tests/Unit` 三层结构齐整。

而 10 张卡里实现类为零。**后果**：无法按卡判断哪些模块已完成、哪些验收项已满足，下游（含 MVP A1 客户端侧）无法用卡状态做前置判定；这 12k 行也没有任何一张卡承载其验收证据。

### 2.3 LumioNativeCore —— 67 张 acceptance，证据合格，等验收

抽样 R-00056 / R-00075 / R-00107 / R-00144 / R-00179 的末条评论，提取到的提交 SHA `0e18106 / b92a84f / c72c460 / f3e4399 / 2110ac2 / d2e460f` **6/6 实测为 origin/main 祖先**（另两个 9 位串 `85eff7c80` / `4a4ef493e` 是 hash 值非提交号）。

### 2.4 LumioServer —— 卡状态准确，业务实现确实未开始

| 分类 | 行数 |
| --- | ---: |
| `tools/xtask`（DAG/契约/策略/队列/源码扫描守卫） | 6,844 |
| `crates/lumio-host-testkit` | 950 |
| `modules/`（真实现） | **44** |
| `generated/`（只读镜像） | 24 |

设计已出（R-00260 `acceptance`），实现待开工。MVP 主线 A1-β 仍被 D-1 挡着。

### 2.5 架构仓 —— P0 Gate 5 张已推进到 3 张

盘点期间 `origin/main` 前进 4 笔：`7bdad78`（D-8 normalization 数据化）→ `b8f8c50`（登记 ABI consumers）→ `f5ce0e3`（更新 P0 报告）→ `c712ff4`（**冻结 ADR-042 Signature/Trust Profile，即 R-00005**）。

| 卡 | 状态 | 备注 |
| --- | --- | --- |
| R-00003 / R-00004 / R-00258 | `done` | escalations D-7 的流转欠账已清 |
| R-00005 | `in_progress` | 交付已落 `c712ff4`，**卡状态待流转** |
| R-00006 / R-00008 / R-00009 / R-00257 | `backlog` | — |

## 3. 双向漂移对照

| 方向 | 仓 | 事实 | 处置建议 |
| --- | --- | --- | --- |
| **仓库领先** | Client | 12,224 行 / 11 模块 / 130 测试，无实现卡 | **补建实现卡**（需授权）：按 11 个模块或 Wave 0-6 边界建卡，把已交付的补上证据并流转 |
| **仓库领先** | 架构仓 | R-00005 交付已在 `c712ff4` | 补证据评论 + 流转到「验收中」 |
| **流转欠账** | NativeCore | 67 张 `acceptance`，证据可核 | 逐批走验收流转到 `done`；证据已足，不需重跑 |
| **流转欠账** | VoxelEngine | 25 张 `in_review` + 14 `acceptance` | 同上，按模块批量清 |
| **无漂移** | Server / CoreEngine / GameRuntime / Game | 卡状态与 origin 事实一致 | 无需对账，等解阻 |

## 4. 关键路径与下一阶段 wave

### W0 · 解阻（不完成不进下一 wave）

| # | 事项 | 阻塞对象 | 归属 |
| --- | --- | --- | --- |
| W0-1 | **D-1 状态载荷 + 上行承载裁决**（已定方向：甲·拆两步） | MVP A1-β（主线验收 1） | 架构源 |
| W0-2 | **D-2 `architecture.lock` 升级到 V1.4 + 投影纳入 `packages/`** | CoreEngine 32 张 | 架构源 + CoreEngine，两卡严格串行 |
| W0-3 | **D-3 generated 面能力边界裁决** | GameRuntime 26 张、Client、Server | 架构源 |
| W0-4 | **NativeCore 67 张验收流转** | 下游按卡状态判前置者 | 总调度 |

W0-1/2/3 都是**架构源单点**——三条下游主干（CoreEngine、GameRuntime、MVP 主线）全部堵在同一个仓的裁决上。这是当前项目的真实关键路径。

### W1 · 解阻后并行

- CoreEngine：R-00015 起的 P0 链（lock 升级后）
- GameRuntime：R-00138/00139/00141/00149/00150（generated 面裁决后）
- Server：R-00206 起的服务端实现（D-1 裁决后可开 A1-α 部分）
- Client：补卡后按模块流转 + 4 张 SPIKE

## 5. 风险与开放决策

1. **架构源是单点瓶颈。** 三条下游主干堵在同一处，且该仓同时有多个会话在改（本次盘点期间 4 笔推送、K[28] 被重复修复两次）。建议对 W0-1/2/3 明确排他归属，避免并发裁决。
2. **`acceptance` 85 张的验收由谁做没有定义。** 证据齐全但无人流转，形成系统性积压。需要指定验收责任人或批量放行规则。
3. **Client 补卡的粒度需裁决**：按 11 个模块建 11 张，还是按 Wave 0-6 建 7 张，或只补一张总卡挂证据。
4. **MS-00001 的 `targetOn` 本次未读到**——`/schedule/milestones` 列表与详情端点均返回 405，只能从 `/search` 拿到 `status=planned`。记忆记载的 2026-10-31 **本轮未验证**，不得当已核实数据使用。

## 6. 本次已执行 / 待授权

**已执行（全部只读）**：七仓 `git fetch` + `origin/main` 事实采集；Workflow 全量 261 卡 cursor 取全；9 张关键卡四路读；14 个证据 SHA 的祖先关系实测；CoreEngine `architecture.lock.json` 独立复核。

**待授权（本次一律未做）**：
- Client 补建实现卡（数量与粒度待定）
- NativeCore 67 张 + VoxelEngine 39 张的验收流转
- 架构仓 R-00005 补证据评论并流转
- 上述任何 Workflow 写入

## 7. Known gaps

- NativeCore 67 张、VoxelEngine 39 张为**抽样**核验（各 5 张 / 0 张），非逐卡；VoxelEngine 的 `in_review` 卡本轮未读评论。
- 未核验各仓测试是否**实际执行通过**——只统计了测试实体数量（`#[test]` / `[Fact]` 等），未跑任何仓的测试。
- MS-00001 的 `targetOn` 与需求归属未读到（端点 405）。
- 架构仓 `origin/main` 在盘点期间移动 4 次，表中架构仓行取 `c712ff4`（12:28Z），其余仓取 12:21Z。
