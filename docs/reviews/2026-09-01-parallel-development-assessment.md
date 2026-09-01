# 并行开发评估与编排报告（底层通用模块优先）

日期：2026-09-01
提问：两条线（验证线 / 铺量线）能否并行；最新形态下哪些仓库内容可并行；如何先把底层通用模块搭起来。
证据基准：各仓 `origin/main`，2026-09-01 实测 fetch。**下列文件计数会随提交腐烂，派活时须由执行方现场重测，不得照抄。**

---

## 第一部分 · 结论

**并行要做，但"多仓铺量"这个并行方式是错的。** 按旧需求单挑低耦合卡去铺，会原样重演 8 月 29 日那一批：派了 17 个会话，产出接近 0。

原因一句话：**瓶颈不是人手不够，是上游不存在。** 加会话数不会让不存在的上游长出来，只会把一份等待变成十七份等待。

你说的"先把底层通用模块搭好"——**这个方向是对的，而且正是解药**。下面把它落成可派活的清单。

---

## 第二部分 · 为什么不能照旧铺量（三条硬证据）

### 2.1 上一次铺量并行：11/12 与 5/5 全堵

`.sdd/progress.md`（提交 `7f054de`）：

| 批次 | 卡数 | done | BLOCKED |
|---|--:|--:|--:|
| GAS（R-00302…R-00313） | 12 | 1 | 11 |
| D-005（R-00141/228/231/236/245） | 5 | 0 | 5 |

BLOCKED 理由逐条读，是同一句话的变体，没有一条是"做不完"：

- `Runtime public projects absent`
- `persistence-host/host-runtime crates and required predecessors are absent`
- `Runtime GAS sources/ports are absent`
- `the Game checkout has no project/implementation files`
- `no executable LumioBinV1 codec is published and forbids a local replacement`

这些卡在需求单上写着"低耦合"，但真实依赖藏在"上游那个模块还没人写"里，需求单上看不见。

**推论：按需求单标注的耦合度挑卡，挑出来的"低耦合"是假的。**

### 2.2 昨天和今天的重构，把旧铺量线的地基铲了

架构仓 `59866ec`（09-01）：`415 files changed, 30932 deletions(-)`——fixtures 266 / packages 70 / schemas 57 / tools 5 / ids 2。整套 Baseline + Schema + Fixture + ID 契约系统被删除。同期 `LumioCoreEngine` 标记 Deprecated 并入 `engine/native/`，`LumioNativeCore` 的 ABI 所有权上交 SDK。

而 `LumioClient/contract-mirror/upstream/packages/` 与 `LumioVoxelEngine/crates/lumio-voxel-contracts/generated/` 还镜像着这套**已经不存在的上游**。

**推论：旧需求单大量围绕那套契约写成，技术前提昨天消失。RM-00004（40 张）所属的仓本身已废弃。照旧单派活 = 按过期地图派活。**

### 2.3 MS-00002 成功的方式，恰好是铺量的反面

Hello World 不是横着铺的，是竖着切的——一个里程碑同时动四个仓：架构仓出 `hello-wire-v1` 契约与 clr-host Root ABI（ADR-052），Server 出权威世界循环，Client 出 web + headless bot，Game 出 launcher 与证据（`integration/hello/evidence-run1/` 有双轮独立运行、截图、`trace.zip`、`verify-report.json`）。

**它没有等任何上游——它自己造上游。** 这是它跑通、而铺量批零产出的全部差别。

---

## 第三部分 · 最新地形图（09-01 实测）

先看清家底，再谈谁能并行。括号内为非测试源文件数。

### SDK 根 · LumioGameEngineArchitecture `engine/`

| 模块 | 量 | 状态 |
|---|--:|---|
| `native/modules/root-abi` | 28 | 有 |
| `native/modules/composition` | 17 | 有 |
| `native/modules/platform` | 13 | 有 |
| `native/modules/signing` | 6 | 有 |
| `native/modules/sdk-native` | 4 | 有 |
| `native/modules/clr-host` | 3 | 有（MS-00002 新增）|
| `native/modules/manifest` / `smoke` / `loader` / `diagnostics` | 2/2/1/1 | 薄 |
| `abi/native-abi.json` | — | 唯一 ABI 定义 |
| `wire/hello-wire-v1.json` | — | 唯一 wire 契约 |
| **`native/modules/timer`** | **不存在** | **RM-00011 要求的共享 Timer Manager，尚未建** |

### LumioGameRuntime（托管运行时，C#）

| 模块 | 量 | 状态 |
|---|--:|---|
| simulation | 34 | 有 |
| coordination | 29 | 有 |
| replication | 26 | 有 |
| command | 24 | 有 |
| observability | 15 | 有 |
| ecs | 12 | **薄**——RM-00011 要的 component schema / AttributeId 查询面基本没有 |
| hello | 8 | MS-00002 闭环 |
| **config / gas / hot-reload / persistence / testing** | **0** | **五个空壳目录** |

### LumioServer（Rust）

| 位置 | 量 | 状态 |
|---|--:|---|
| `modules/process` | 10 | MVP host 进程 |
| `crates/lumio-host-testkit` | 7 | 有 |
| `tools/xtask` | 6 | 有 |
| `generated/*`（三个契约 crate） | 各 1 | 占位 |
| **Account Server / Room 管理 / admission** | **不存在** | **RM-00011 主干，全新** |

Server 是全局最薄的一环，且 RM-00011 对它的要求最重。

### LumioClient（C#）

| 模块 | 量 | 模块 | 量 |
|---|--:|---|--:|
| session | 31 | replica | 15 |
| connection | 24 | persistence | 13 |
| observability | 20 | handshake | 10 |
| prediction | 16 | bot | 6 |
| input | 15 | hello | 5 |
| web | 2 | **hybridclr-adapter / unity-adapter** | **0 / 0** |

`replica` 已有 15 个文件——ReplicaWorld 有地基，不是从零起。

### 其余

- **LumioVoxelEngine**：7 个 crate（contracts / domain / migration / ops / project / test-support / world），与 RM-00011 **零交集**。
- **LumioGame**：0 源文件，只有 integration launcher 与验收证据。
- **LumioNativeCore**：ABI 所有权已上交 SDK。**LumioCoreEngine**：已 Deprecated。

---

## 第四部分 · 底层通用模块清单（你要的重点）

RM-00011 的决策日志里，被明确写成"**shared / common / not X-specific**"的有这些。它们就是该先搭的地基——Chat 只是第一个使用者。

| # | 通用模块 | 原文定性 | 现状 | 归属 |
|---|---|---|---|---|
| G1 | **Native Timer Manager** | "shared Server/Client infrastructure" | **不存在** | `engine/native/modules/timer` |
| G2 | **连接↔实体绑定** | "a shared runtime capability, **not Chat-specific logic**" | 不存在 | Runtime + Server |
| G3 | **Attribute Query 表面 + 生成 AttributeId** | "controlled ECS Attribute Query surface"，非 SQL、非任意属性名 | 不存在 | Runtime ecs（生成面在 SDK）|
| G4 | **NetEntityId 身份契约** | 不复用、tombstone、generation 安全 | 不存在 | 契约（SDK）|
| G5 | **ECS component schema 三维标注** | 每个属性独立声明 persist / replicate / visibility | 不存在 | Runtime ecs |
| G6 | **ReplicaWorld** | 客户端权威副本 | replica 15 文件，需扩展 | Client |
| G7 | **Snapshot/Restore + WAL** | "existing architecture"，但 Runtime `persistence` 是 0 文件 | **空壳** | Runtime persistence |
| G8 | **Account Server** | "mandatory for this slice"，不可被 Game Server 假账号表替代 | 不存在 | 新建 |

**关键判断：G1、G7、G8 三件不吃任何未冻结的契约，现在就能开工。** 其余五件（G2–G6）要等契约期定型，否则就是 2.1 的抢跑返工。

---

## 第五部分 · 并行编排

结构不是"几个仓一起搞"，是"**一段串行契约期 → 一次扇出实现期 → 串行收口**"，另加一条**全程不受影响的独立线**。

```
独立线 ─────────────────────────────────────────────────────  全程并行
        A1 Timer  A2 Account Server  A3 persistence 骨架  A4 工程债  A5 镜像清理

主  线  [契约期·独占串行]  →  [实现期·6 路并行]  →  [收口·串行]
          C1…C5 冻结            B1…B6                   E2E / 重连 / 隔离
```

### A 组 · 现在就能开，互不冲突（建议今天起 3–5 路）

| # | 内容 | 落点（新目录，零冲突） | 依赖 |
|---|---|---|---|
| **A1** | **Native Timer Manager 首片**：固定 Tick/Frame 定时器、one-shot / repeating、取消、scope/generation 校验、`CallbackSlot` 回调 | `engine/native/modules/timer/` | 无（新模块）|
| **A2** | **Account Server**：账号注册表、login-or-register、AccountEntity 生命周期、默认口令档、admission credential 签发 | 新建服务 | 只吃 credential 格式一个契约点 |
| **A3** | **Runtime persistence 骨架**：WAL / Command Log / Snapshot 落盘与恢复的**存储层** | `modules/persistence/`（现 0 文件）| 存储层无依赖；序列化面等 G5 |
| **A4** | **工程债清理**（见 5.3） | 各仓分散 | 无 |
| **A5** | **下游镜像清理**：Client / Voxel 里指向已删上游的 `contract-mirror`、`Lumio.Gen.*` | Client / Voxel | 无 |

A1 是最干净的一路：全新目录、需求已定型、Server 与 Client 都要用。**这是"先搭底层通用模块"的最佳第一刀。**

A3 有个诚实的边界：**存储层**（写 WAL、原子换页、恢复重放）可以现在写；**序列化什么**要等 G5 的 component schema。派活时必须把这条边界写进卡里，否则会做成半成品返工。

### B 组 · 契约冻结后扇出（6 路，文件集互不重叠）

先冻结这五件共同上游（**C 段，必须独占串行，谁抢跑谁返工**）：

- **C1** 类型化 wire 映射：`InputCommand` 承载 `ChatInput`；`FullSnapshot.stateBlocks` / `Delta.changedBlocks` 承载状态（决策日志已定：**不扩展旧 Hello wire**）
- **C2** 生成 `AttributeId` 面 + component schema 三维标注（G3 + G5）
- **C3** `NetEntityId` + binding 契约：`AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration`（G2 + G4）
- **C4** Native Timer ABI 签名（与 A1 实现解耦：ABI 先定，实现照着做）
- **C5** admission credential 格式（**这一条应最先给，A2 在等它**）

冻结后：

| # | 实现面 | 仓 / 目录 | 吃哪个契约 |
|---|---|---|---|
| B1 | ChatComponent + SetMessage + 事件产出 | Runtime `modules/ecs` | C2 |
| B2 | Chat 复制映射 + 可靠有序投递 | Runtime `modules/replication` | C1 |
| B3 | Room admission + Bot 命名分类 + 实体创建 | Server（新 module）| C3 + C5 |
| B4 | ReplicaWorld 扩展 + 客户端自查 + 可见属性查询 | Client `modules/replica` | C1 + C3 |
| B5 | Bot 工具 100 路登录 | Client `modules/bot` | C5 |
| B6 | Browser 聊天窗渲染 | Client `modules/web` | C1 |

**冲突检查**：B1/B2 同仓不同 module；B4/B5/B6 同仓不同 module；B3 独占 Server。六路文件集两两不重叠，符合仓规"文件集互不重叠才可并行"。各自开独立 worktree。

### C 组 · 必须串行收口

101 实体规模、可靠有序、5 分钟重连、过期 tombstone、跨 Room 隔离、两轮可重复——要在同一套跑起来的系统上验，拆并行只会得到互相矛盾的证据。

### 5.3 工程债明细（A4，永不 BLOCKED）

出处 `2026-08-29-td-handoff-final.md` §5.2：

- 各仓 CI 接入（当前守护只在本地生效）
- Client flaky 测试（会卡 CI）
- **LumioServer 两处闸门哈希可碰撞**（P1，`tools/xtask/src/contracts.rs:1033` 与 `:471`，有生产调用方，碰撞即闸门静默放行）
- `contracts/*.lock.toml` 里的 Windows 绝对路径，致 `contracts verify` 在非 Windows 宿主整份失效
- Voxel / Runtime / Server 的 `eng/*.ps1` 至今无任何机器实跑过
- Client 两个 adapter 测试工程零测试方法却 `dotnet test` 返回 0

这些不依赖任何未写出的上游，因此**永远不会 BLOCKED**，是"免费"的并行带宽。

---

## 第六部分 · "多开几个仓是不是更快"

仓数不是并行度的单位。当前三个真实瓶颈，没有一个能靠加仓解决：

| 瓶颈 | 说明 | 加并行的后果 |
|---|---|---|
| 契约单点 | 所有跨仓卡共享同一个上游（C1–C5）| 无改善；抢跑则返工 |
| 审查带宽 | 写 ≠ 审是硬规；历史上 reviewer 大量 RETURN（R-00203 第三轮仍 1 CRITICAL + 7 HIGH）| **等比放大** |
| 你的裁决 | 历史上一批 D-* 长期挡着最长链 | **等比放大** |

后两个随并行度线性上涨，且都只有一个出口（你）。

**所以并行度上限该按"你每天能审掉 / 裁决掉多少"定，不是按仓数。**

建议起步配比：

- **A 组 3–5 路**——今天就能起，几乎不占你的裁决带宽
- **主线 1 路**（契约期 C1–C5），冻结后扇出 **B 组 6 路**
- Voxel 若要动，单独 1 路，与主线零交集

---

## 第七部分 · 建议的动作顺序

1. **今天**：起 A1（Timer）+ A2（Account Server）+ A4（工程债 1–2 路）。同时你给出 **C5 admission credential 格式**——它是 A2 唯一的阻塞点，一句话的裁决换一条并行线。
2. **同期主线**：把 RM-00011 的 Wave 1 收窄成**纯契约卡**（C1–C5），不混实现卡。
3. **契约冻结后**：B 组 6 路一次扇出，各自 worktree。
4. **收口**：C 组串行，按 MS-00002 的证据标准（双轮独立运行 + 原始日志 + 可复核 hash）。
5. **平行进行**：旧需求单有效性重判，分**仍有效 / 需重写 / 应作废**三类。RM-00004（40 张）大概率整室作废——仓已 deprecated。**不做这一步就铺量，2.1 的结果会原样再来一次。**

---

## 第八部分 · 需要你拍板

1. **admission credential 格式**——挡着 A2，最高优先级，给了就能起第二条并行线。
2. **Account Server 落哪**：新建独立仓，还是作为 LumioServer 的一个 module？影响 A2 的 worktree 与后续 CI。
3. **旧需求单重判现在做不做**——不做就无法安全铺量，做了大概率要作废一批（含 RM-00004 整室）。
4. **RM-00011 Wave 1 是否收窄成纯契约卡**——当前 11 张卡若混了实现卡，会重演抢跑返工。
5. **并行度上限定多少**——等价于问：你每天愿意花多少时间在审查和裁决上。
