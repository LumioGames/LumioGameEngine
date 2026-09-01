# 并行开发编排报告（RM-00011 之外的盘子）

日期：2026-09-01
范围：**排除 RM-00011「ECS Formal Entity and Chat」需求室及其 Account Server**——该室由另一会话的 Agent 管理，本报告不进入、不派活、不改其文件。
本文回答：在此边界之外，哪些内容可并行，先搭哪些底层通用模块。
证据基准：各仓 `origin/main`，2026-09-01 实测。**文件计数会腐烂，派活时须执行方现场重测，不得照抄。**

---

## 1. 结论

我这边最干净、最该先搭的底层通用模块是 **LumioConfig 配表系统**。

三个理由：

1. **它是独立新仓库**——阶段 0 与阶段 1 前段完全不碰任何现存实现仓的文件，与 RM-00011 天然零冲突。
2. **它是别人的共同上游**——GAS 的技能表/效果表、存档的终身编号、内容层的一切数值都从它出。先搭它，等 RM-00011 冻结 ECS 组件 schema，GAS 就能立刻接上，两条线正好错开而不是抢道。
3. **架构 8-30 已定稿**，`docs/specs/2026-08-30-lumioconfig-design-overview.md` 里 TODO 已排好顺序（阶段 0 六张 ADR → 阶段 1 七步），拿来就能开卡，不需要再开架构会。

配套三条永不堵的并行线：**Voxel 独立仓**、**工程债**、**重构善后**。

---

## 2. 先划防撞边界（两个 Agent 并仓作业的前提）

RM-00011 会动的面（**我方禁区，一律不碰**）：

| 禁区 | 位置 |
|---|---|
| Account Server | 全新，归对方 |
| Room admission / 实体分类 | LumioServer |
| 连接↔实体绑定、NetEntityId | Runtime + Server |
| ECS 组件 schema / AttributeId 查询面 | Runtime `modules/ecs` |
| ChatComponent、Chat 复制映射 | Runtime `modules/ecs`、`modules/replication` |
| ReplicaWorld、重连 | Client `modules/replica`、`session`、`connection` |
| **Native Timer Manager** | `engine/native/modules/timer`（RM-00011 的 Timer track）|
| **Snapshot/Restore + WAL** | Runtime `modules/persistence`（RM-00011 要用）|
| wire 契约 / `hello-wire-v1` 后继 | `engine/wire`、`engine/abi` |
| E2E 验收 | LumioGame |

> 上一版报告曾把 Timer 与 persistence 列为我方可起项——**按新边界撤回**，两者都在 RM-00011 的 track 里，归对方。

我方可动的面：**LumioConfig（新仓）· LumioVoxelEngine · 各仓工程债 · 重构善后 · 存量需求室重判**。

---

## 3. 为什么不能照旧铺量（这条教训跨边界仍然成立）

### 3.1 上一次铺量并行：11/12 与 5/5 全堵

`.sdd/progress.md`（提交 `7f054de`）：

| 批次 | 卡数 | done | BLOCKED |
|---|--:|--:|--:|
| GAS（R-00302…R-00313） | 12 | 1 | 11 |
| D-005（R-00141/228/231/236/245） | 5 | 0 | 5 |

BLOCKED 理由逐条读是同一句话的变体，没有一条是"做不完"：`Runtime public projects absent`、`persistence-host crates and required predecessors are absent`、`Runtime GAS sources/ports are absent`、`the Game checkout has no project/implementation files`。

**根因不是人手不够，是从中间派活——底座没人搭，中层卡永远等不到上游。** 这正好反过来印证：**要并行，就得从底座起**。

### 3.2 重构已铲掉旧铺量线的地基

架构仓 `59866ec`（09-01）：`415 files changed, 30932 deletions(-)`——fixtures 266 / packages 70 / schemas 57 / tools 5 / ids 2，整套 Baseline + Schema + Fixture + ID 契约系统删除。`LumioCoreEngine` 已 Deprecated 并入 `engine/native/`，`LumioNativeCore` ABI 所有权上交 SDK。

而下游还镜像着这套**已不存在的上游**：`LumioClient/contract-mirror/upstream/packages/`、`LumioVoxelEngine/crates/lumio-voxel-contracts/generated/`。

**RM-00004（CoreEngine，40 张卡）所属的仓本身已废弃。** 照旧单派活 = 按过期地图派活。

---

## 4. 我方盘子的地形（09-01 实测）

| 对象 | 现状 | 与 RM-00011 |
|---|---|---|
| **LumioConfig** | **仓库尚不存在**；架构 8-30 定稿，M1–M10 十模块 + TODO 已排序 | **零交集** |
| **LumioVoxelEngine** | 7 crates：contracts / domain / migration / ops / project / test-support / world | **零交集** |
| Runtime `modules/config` | **0 文件**（配表在运行时的落点，M8/M9）| 零交集 |
| Runtime `modules/gas` | **0 文件**；GAS 架构 8-30 定稿，9 模块 | **有交集**（GAS 的「四组件与 Schema」挂 ECS）|
| Runtime `modules/hot-reload` | **0 文件** | 待判 |
| 各仓工程债 | 见 §6 | 零交集 |
| 存量需求室 RM-00003/04/05/06/07/08 | 未按重构后重判 | 需重判 |

---

## 5. 主推：LumioConfig 配表系统

### 5.1 为什么是它

配表是整个内容管线的地基：GAS 的技能/效果表、掉落表、经济数值、存档要认的终身编号，全从这里出。RM-00011 完全不碰它。它现在**连仓库都还没建**，等于零冲突起步。

四条底线（定稿已锁，砍功能也不能碰）：表的真身是文本文件 / 内容指纹与存储格式无关 / 一台服务器一个版本 / AI 没有上线按钮。

### 5.2 阶段 0（先立规矩，落架构仓 `docs/adr/`）

六张 ADR，**编号落笔时现查最高号**（当前最高 ADR-052，但会被抢占，必须重 fetch 核对）：

| 卡 | 内容 | 模块 |
|---|---|---|
| 0-1 | 权威源与补丁通道（文本格式三纪律、双门） | M1/M2 |
| 0-2 | ID 命名空间与发号（扩展 ADR-007） | M3 |
| 0-3 | 内容指纹与数值规则（**含定稿时未钉的 Unicode 归一化，此卡必须补**）| M4 |
| 0-4 | 产物容器与三端切分（四层清单、S/C/V 标签、披露门禁、签名防回滚） | M4/M5 |
| 0-5 | 版本生命周期（实例绑定、开发 reload、备齐纪律、回放钉版） | M9/M10 |
| 0-6 | 工具面契约（补丁格式、报错格式、AI 五动作） | M2/M7 |

**这六张之间可并行**——文件集是六个独立 ADR 文件。唯一串行点是编号分配。

### 5.3 阶段 1（垂直切片，七步）

| 步 | 内容 | 可并行性 |
|---|---|---|
| 1 | 建 LumioConfig 仓 + M1 最小版（目录、格式化工具、两三张真表）| **必须先行** |
| 2 | M2 机器门最小版（查错 + 结构化报错）| M1 后 |
| 3 | M3 发号台 | 可与 2 并行 |
| 4 | M4 导表器最小版（文本产物 + 切三份 + 盖指纹）| 2/3 后 |
| 5 | M8 查表代码（Rust + C# 的 `TryGet`）| 4 后，**Rust 与 C# 两路可并行** |
| 6 | M9 开机装载 + 开发 reload | 5 后 |
| 7 | 验收三条链 | 串行收口 |

**验收三条链**（定稿原文）：① 加一张混合标签新表 → 三端各读到该读的、读不到不该读的；② 改一行数值 → reload 原子生效 → 旧回放照样能放；③ AI 五动作全流程提补丁（中途故意报错让它自修），全程无上线权，Git 留痕完整。

### 5.4 起步建议

**今天起 2 路**：一路走阶段 0 的 ADR（0-1/0-2/0-3 先行，它们挡后面）；一路建仓 + M1。两者文件集不重叠（架构仓 `docs/adr/` vs 新仓）。

**注意接触点**：M8 生成的查表代码最终要进各实现仓、M9 装载器要进运行时 `modules/config`——那是阶段 1 后段。届时若 RM-00011 仍在改 Runtime，需先对一次文件边界。**阶段 0 与阶段 1 前四步不涉及。**

---

## 6. 配套三条永不堵的线

### 6.1 工程债（零依赖，永不 BLOCKED）

出处 `2026-08-29-td-handoff-final.md` §5.2：

- **LumioServer 两处闸门哈希可碰撞**（**P1 安全缺陷**，`tools/xtask/src/contracts.rs:1033` 与 `:471`，有生产调用方，碰撞即闸门静默放行）← 我方盘子里最该先修的一件
- `contracts/*.lock.toml` 内嵌 Windows 绝对路径，致 `contracts verify` 在非 Windows 宿主整份失效
- 各仓 CI 接入（当前守护只在本地生效）
- Client flaky 测试（会卡 CI）
- Voxel / Runtime / Server 的 `eng/*.ps1` 至今无任何机器实跑过
- Client 两个 adapter 测试工程零测试方法却 `dotnet test` 返回 0

这些不依赖任何未写出的上游，是"免费"并行带宽。但注意：**改 Server 的 xtask 不碰 RM-00011 的 Server 业务面**，边界要在派活卡里写死。

### 6.2 重构善后

- 清 `LumioClient/contract-mirror/` 与 `LumioVoxelEngine/.../generated/` 中指向已删上游的镜像。**不清，RM-00011 那边迟早也会撞上。**
- 判 RM-00004 整室（40 张）是否随 CoreEngine 废弃而作废。
- 存量 RM-00003/05/06/07/08 按新地基分三类：**仍有效 / 需重写 / 应作废**。

### 6.3 Voxel 独立线

7 个 crate 与 RM-00011 零交集，可长期单开一路。但**先做 §6.2 的重判再派活**——RM-00003 的 16 张待核销卡里，有一部分验收口径建立在已删的 Fixture 系统上。

---

## 7. 建议的起步配比

| 线 | 路数 | 今天可起 |
|---|--:|---|
| LumioConfig 阶段 0（ADR）| 1–2 | 是 |
| LumioConfig 阶段 1 建仓 + M1 | 1 | 是 |
| 工程债（先修 Server 闸门哈希 P1）| 1–2 | 是 |
| 重构善后 / 存量重判 | 1 | 是 |
| Voxel | 1 | 重判后 |

合计 **5–7 路**，全部在 RM-00011 禁区之外，互不重叠。

上限不由仓数决定，而由**你每天能审掉多少**决定——写 ≠ 审是硬规，历史上 reviewer 大量 RETURN（R-00203 第三轮仍 1 CRITICAL + 7 HIGH）。并行度上去，审查排队也等比上去。

---

## 8. 需要你拍板

1. **LumioConfig 仓建在哪、叫什么**——挡着阶段 1 第 1 步。
2. **阶段 0 六张 ADR 落架构仓、由我方开卡** 是否确认（ADR 号需现查最高号，存在被另一会话抢占的风险，历史上发生过）。
3. **RM-00004（40 张）整室作废** 是否照准。
4. **GAS 归谁**——它的「四组件与 Schema」挂在 ECS 上，与 RM-00011 正在冻结的组件 schema 冲突。建议：**GAS 暂不开工，等对方冻结 ECS schema 后由我方接**，否则两边会对着同一个 schema 各写一版。
5. **存档 / DS 服务器 / ECS 框架三套定稿归谁**——均与 RM-00011 有交集，需与对方 Agent 对边界，我不单方面动。

---

## 9. 与 RM-00011 执行编排的冲突核对（2026-09-01 补）

收到对方会话的完整派活提示词后，逐条核对施工面：

| 对方施工面 | 我方原计划 | 冲突 | 处置 |
|---|---|---|---|
| 架构仓 `docs/adr/` + `.spec/decisions/`，**ADR 号合并时现查现占** | 配表阶段 0 落六张 ADR | **严重** | **改落 LumioConfig 新仓 `docs/decisions/`，不占架构仓号**；等对方 Phase 0 收完再搬 |
| 七仓镜像 / `contract-mirror` / `generated` 更新 | 清理指向已删上游的镜像 | **严重** | **撤回**，改为只读盘点 |
| Workflow（对方为**唯一写入方**） | 存量需求室重判 + 流转 | **严重** | **撤回写入**，只做本地只读盘点报告 |
| LumioServer（R-00344/346/350） | 修 xtask 闸门哈希 P1 | **隐性** | xtask 是对方跑门禁的工具，改它会改变其门禁行为——**本轮撤回**，先只出复现报告 |
| LumioGameRuntime（地基 6 卡 + 3 卡） | Runtime `modules/config`（M9 落点） | **严重** | **M9 暂缓**至对方阶段结束 |
| LumioClient / LumioGame / LumioNativeCore | 无 | 无 | 禁区 |
| LumioVoxelEngine | Voxel 独立线 | **五条管线不含 Voxel**，但镜像更新会碰其 `generated/` | 业务 crate 安全；**本轮不排**，避开镜像期 |

**净结果：我方可执行集合收窄为 LumioConfig 一条线，但该线 100% 零重叠。**
派活提示词见 [`../plans/2026-09-01-lumioconfig-parallel-dispatch.md`](../plans/2026-09-01-lumioconfig-parallel-dispatch.md)。

### 9.1 两处需要 Owner 知悉的事实偏差（对方提示词与 origin/main 实测不符）

只陈述实测结果，不代对方决定：

1. **收口门槛已有两条命令的目标不存在。** AGENTS.md 的收口门槛是四条命令，但 `59866ec`（09-01）
   已删除 `tools/lumio_contract.py`。实测 `origin/main`：`.spec/tools/spec-lint.mjs` 与
   `.spec/tools/spec-lint.test.mjs` **仍在**，`tools/lumio_contract.py` **已不存在**。
   对方提示词要求「过本仓收口门槛」，后两条会直接失败。
2. **对方第 3 条必读材料尚未在 origin/main 上。** 实测 `docs/reviews/` 无
   `2026-09-01-rm-00011-room-review.md`；同理其提示词引用的「Room Review Rulings 2026-09-01」
   在 `docs/specs/2026-09-01-ecs-formal-entity-chat-decision-log.md` 的线上版本中也未见。
   可能仍在对方本地未推送。若执行方按提示词去读会扑空。

另：对方提示词沿用「ADR→Schema/ID→正反 Fixture→README/Baseline→七仓镜像」的旧变更顺序，
而 `schemas/`、`ids/`、`fixtures/`、`packages/`、`tools/` 已在 `59866ec` 整体删除（415 文件 / 30932 行）。
**若对方因此重建这套系统，将与配表 M4 的指纹与产物容器产生新的重叠**——届时需要重新对边界。
这是本线唯一的远期风险点，现在无需处理，但值得记一笔。
