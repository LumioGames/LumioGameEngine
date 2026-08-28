# LumioGameEngine V3 · 七仓进度评估与 Workflow 对账报告

> **日期**:2026-08-28 · **基线**:`LGE-V1.4-2026-08-27` · **评估者**:架构仓总调度(macOS 会话)
> **方法**:八仓全量文档与代码调研 + Workflow(lumiogamesengine.workflow.games)全量在线核对(8 Room / 256 需求卡)+ 本机实测(工具链、验收脚本、git 远端比对)。每条结论附出处;未实际执行的验证一律标注「未执行」。
> **配套动作**:MVP 已立项(见 [mvp-browser-voxel-multiplayer.md](../plans/mvp-browser-voxel-multiplayer.md) §0);对账写入已执行(见 §7)。

## 1. 执行摘要

| 仓库 | 实现完成度(对 V1 目标) | 一句话状态 |
|------|------|------|
| LumioGameEngineArchitecture | Architecture Gate **完成** | V1.4 冻结,契约生成器与六类 Artifact 已发布;5+1 张 Gate 裁决卡(AG 族)待做 |
| LumioNativeCore | **60–70%** | 最健康:9 crate 全有实现与测试(~8.8k 行),I0 完成、I1 进行中 |
| LumioVoxelEngine | P0 代码 **~85% 但未验证** | ~24k 行**从未链接执行过测试**(Windows 缺 link.exe);RETURN/BLOCKED 均为环境性 |
| LumioGameRuntime | **10–15%** | 全局最长杆:11 模块仅 observability 有实现;31 卡链是轨道 A 心脏 |
| LumioClient | **40–50%** | Foundation Wave0–6 完成(11/11 模块有代码);Wave7 未开始 |
| LumioServer | **0% 实现 / 90% 设计** | origin 零代码;51 张 Rust 卡已落单;**不在 MVP 关键路径**(MVP 用 C# 宿主) |
| LumioCoreEngine | **5–10%(origin)** | origin 零代码;Windows 侧有未推送交付(见 §4);瓶颈是架构仓 AG 裁决吞吐 |
| LumioGame | **2–3%** | 零设计文档、零模块目录;MVP 尾部风险(PlaceVoxelAbility 无处落地) |

**三个核心结论:**

1. **关键路径 = 架构仓 Gate 裁决吞吐 → GameRuntime 31 卡串行链 → LumioGame 玩法内容 → MVP A3 收口。** VoxelEngine 解阻是一周期可清的短杆;Server Rust 51 卡与 MVP 无关,推迟到 Production Hardening。
2. **Workflow 与仓库现实存在系统性双向漂移**:256 张卡无一张「已完成」,而 NativeCore/VoxelEngine/Client 的大量交付已在 origin/main;反向地,CoreEngine 4 张、Server 2 张「验收中」卡的证据引用了 **不在 origin 的提交**(Windows 工作区未推送)——其中 R-00188 的证据与其引用提交内容**不符**。详见 §4。
3. **开发横跨两台机器(Windows `C:/Work/LumioGames` 与 macOS `~/LumioGames`),origin 是唯一可信汇合点。** 两侧工具链差异已造成实际损失:VoxelEngine 2.4 万行代码因 Windows 缺链接器从未跑过测试;GameRuntime 的 SDK pin(`10.0.11`,rollForward=disable)在任何机器上都无法字面满足,Mac 上直接 `SDK_MISMATCH` 无法构建。

## 2. 各仓完成度详情(证据)

### 2.1 LumioGameEngineArchitecture(架构源)

- **已完成**:V1.4 基线冻结(36 P0 + 4 P1 + 1 P2 Schema、160 Fixture、12 状态机描述符);ADR-001..039(除 ADR-015 Reserved);契约生成器与六类 Artifact × Rust/C# 发布(`packages/`,commit `15d96f3`);ContractRuntime clippy 清洁(`3d5e29d`);D-013 / VOX-D-001..004 架构所有者确认(`5f06822`,确认书 id `LGE-V1.4-VOX-D-P0-2026-08-28`)。
- **待办**:Workflow RM-00001 有 6 张 Gate 卡全部 backlog——R-00003(Root ABI Bundle,W1 发布物可能已大部覆盖,差逐项核对与 Golden)、R-00004(Canonical/Digest Profiles)、R-00005(Signature/Trust)、R-00006(Loader Fixture)、R-00008(Evidence Profiles)、R-00009(P1 统一 TargetProfile 族)。D-009(RPC dispatch)/D-011(Auth wire)**维持有意阻塞**;D-014 只闸 Voxel P2,不阻 MVP。
- **治理缺口**:契约生成器(Foundation W1 卡)当时未走 Workflow 落单,造成真值分叉先例。

### 2.2 LumioNativeCore(Rust Kernel)

- HEAD `0fcb1f0`;71 src + 64 test 文件,~8,776 行;9 crate(contract-types/kernel/job/spatial/codec/diagnostics/native-ffi/platform/test-support)全部有实质实现与测试;无 TODO/FIXME。
- `.spec/tasks/` 四张卡全 completed;I1(codec/diagnostics feature-gated 私有原型)推进中。
- Workflow RM-00002:67 卡,**66 张停在「评审中」、1 张 backlog、0 张完成**——交付远超卡状态,属「仓库领先」漂移,需按核验清单批量补验收流转(派活阶段)。

### 2.3 LumioVoxelEngine(Rust 体素域)

- HEAD `ea7d6e7`;89 src + 33 test 文件,~24,308 行(含生成契约);P0 domain/ops/world 深度实现;project/migration crate 仅骨架(P2)。
- **唯一硬伤是环境**:全部测试仅过 `cargo check`,`cargo test` 在 Windows 因缺 `link.exe` exit 101 **从未链接执行**。R-00203(MVP 审查)四轮评论链最终裁决 RETURN 的仅存理由 = 「linked tests 未跑 + R-00204 QA 无线上环境」;R-00204 放行门 BLOCKED 同源。
- 决策门:R-00057..R-00064(VOX-D-001..008)全在「验收中」;**001..004 已获架构所有者裁决**(本次已同步至卡,见 §7),005..008 属 D-014 家族,只闸 P2。
- Workflow RM-00003:53 卡 = 28 评审中 + 11 验收中 + 14 backlog(P2 12 卡 + R-00204 等)。

### 2.4 LumioGameRuntime(C# Runtime)

- HEAD `fbaca12`(工程基线 + GeneratedContracts + observability 模块 + 测试,~28 文件 ~1.7k 行);其余 10 模块(ecs/simulation/command/coordination/replication/gas/persistence/config/hot-reload/testing)零代码。
- **本机实测**:`bash eng/verify-sdk.sh` → `SDK_MISMATCH expected=10.0.11 actual=<unavailable>`(exit 0 但报告失配)。`global.json` 锁 `10.0.11` 且 `rollForward: disable`——SDK 版本族不存在 10.0.11(是 runtime 版本号;Windows 曾用 SDK 10.0.111,本机 10.0.400)。**在修正 pin 之前,本仓在本机不可构建、验收不可复跑。**
- Workflow RM-00005:31 卡 21 wave;本次对账后 5 张(R-00112/00127/00131/00133/00138)已流转到「实现中」并附证据与差距评论,其余 26 张 backlog。R-00131 的执行门(架构六类 Artifact 正式发布)已满足。

### 2.5 LumioClient(C# 客户端)

- HEAD `22eae37`;~211 个 .cs(src ~6.9k 行 + tests ~5.1k 行);11/11 模块有实现;Foundation Wave0–6 完成,8-28 仍有三笔修复提交;Wave7(persistence filesystem/remote transport/Serilog/Unity/HybridCLR)未开始。
- Workflow RM-00007:10 卡(5 评审中——五文件规划记录,5 backlog);Wave7 执行卡未落单(有意后置,多数项等 Runtime/Server 产物)。

### 2.6 LumioServer(Rust Host)

- origin/main HEAD `58741d9`:**纯文档**(51 张任务卡的实现设计包);origin 全树无 Cargo.toml、无 .rs。
- Workflow RM-00006:53 卡(51 backlog + 2 验收中);两张验收中卡的证据问题见 §4。
- **MVP 定位修正**:MVP §4 明确 Server 职责由 C# 测试宿主承担(WebSocket transport/auth 存根/session/world-slot 最小实现);Rust Host 主线是 Production Hardening 方向。Foundation 退出条件只需 Rust wave0(workspace)一张卡。

### 2.7 LumioCoreEngine(Rust 聚合/Loader)

- origin/main HEAD `f3c9920`:纯文档(3161 行设架说明书);origin 零代码。
- Workflow RM-00004:36 卡(32 backlog + 4 验收中);四张验收中卡(R-00011..14:workspace/镜像锁定/ADR-004/ADR-006)证据引用的提交均不在 origin(见 §4)——**Windows 侧有约 53 文件/+1100 行的未推送交付**。
- 设计书明文允许 AG 关闭前先建骨架与「缺契约即失败」门禁;实现的真正前置是架构仓 8 项 P0 AG 裁决(≈RM-00001 的 R-00003..R-00008)。

### 2.8 LumioGame(C# 游戏内容)

- 零设计文档、零模块目录,仅 README + 架构评审镜像;Workflow RM-00008 空。
- **尾部风险**:契约面已冻结、依赖已成熟,若不在 W1 启动设计卡,A0 收口时名义验收物 `PlaceVoxelAbility` 无处落地。

## 3. Workflow 全量现状(2026-08-28 对账后)

- 项目:LumioGamesEngine(lumiogamesengine.workflow.games),单成员(admin@lumio.games,项目管理员)。
- 里程碑:`MS-00001`「MVP · 多浏览器联机体素世界」,目标日 2026-10-31(Server 批次落单时建立,全项目统一归属锚点)。
- 需求工作流:需求池(backlog)→ 评审中(in_review)→ 已评审(approved)→ 实现中(in_progress)→ 验收中(acceptance)→ 已完成;另有已否决(rejected)。

| Room | 仓库 | 卡数 | 状态分布(对账后) |
|------|------|-----:|------|
| RM-00001 | 架构仓 | 6 | backlog=6 |
| RM-00002 | NativeCore | 67 | 评审中=66,backlog=1 |
| RM-00003 | VoxelEngine | 53 | 评审中=28,验收中=11,backlog=14 |
| RM-00004 | CoreEngine | 36 | backlog=32,验收中=4 |
| RM-00005 | GameRuntime | 31 | **实现中=5(本次流转)**,backlog=26 |
| RM-00006 | Server | 53 | backlog=51,验收中=2 |
| RM-00007 | Client | 10 | 评审中=5,backlog=5 |
| RM-00008 | Game | 0 | — |
| 合计 | | 256 | **已完成=0** |

## 4. 漂移对照与证据核验结果

### 4.1 仓库领先于 Workflow(状态欠账)

| 范围 | 事实 | 处置建议 |
|------|------|----------|
| NativeCore 66 卡评审中 | 交付已在 origin/main(HEAD `0fcb1f0`),测试在 macOS 可跑 | 派活阶段按核验清单批量重验 → 流转;不逐卡人工补历史 |
| VoxelEngine 28 卡评审中 | P0 代码在树,证据链完整,仅测试未链接执行 | W0 macOS 首跑测试后随 R-00203 重审一并流转 |
| Client 5 卡 backlog(Foundation 已完成) | Wave0–6 交付在 origin | 补录进 Room 备注或随 Wave7 落单一并对账 |
| GameRuntime 5 卡 | 已对账:流转到「实现中」+ 证据/差距评论(§7) | 验收流转待 SDK pin 修正后实跑 |

### 4.2 Workflow 领先于 origin(证据不可复核 —— 本次核验的核心发现)

| 卡 | 状态 | 证据声称 | 核验结果 |
|----|------|----------|----------|
| R-00011(CoreEngine workspace) | 验收中 | commit `015035b`,15 crate workspace,53 文件/+1100 行 | **`015035b` 不在 origin**(origin HEAD=`f3c9920`,无任何 Cargo 文件) |
| R-00012/13/14(CoreEngine) | 验收中 | commits `d668426`/`06c954f`/`68e1442` | **均不在 origin** |
| R-00188(Server workspace) | 验收中 | commit `58741d9` 交付 Cargo.toml/modules/process/xtask 等 | **`58741d9` 在 origin,但只是文档提交,不含任何所声称文件**——证据与引用提交内容不符 |
| R-00186(Server 规则同步) | 验收中 | commit `58741d9` 同步规则文件;spec-lint 在 Windows 退出码 1 | 该提交未触及所声称文件;收口门槛未通过(评论已如实记录) |

**结论**:六张「验收中」卡在 origin 上均不可复核。根因是 Windows 工作区交付后未推送(CoreEngine 四张)与证据引用错误(Server 两张)。已在六张卡各补差异记录评论(不改状态);**解铃人是 Windows 侧推送 + 总调度重核**。在重核通过前,这些卡不得向「已完成」流转。

### 4.3 防复发规矩(自本报告起生效)

1. **证据以 origin 为准**:任何交回物评论必须引用已推送 origin 的提交号;总调度核验第一步是 `git ls-remote` + 提交内容比对。
2. **动手前先对账**:任何仓开工前,先把该仓 Room 里与 origin 不一致的卡状态清账。
3. **架构仓自身待办一律入 Workflow**(W1 生成器绕行是最后一次例外)。

## 5. 关键路径与下一阶段 wave 编排

依赖链(MVP 视角):架构仓(契约/Gate 裁决)→ GameRuntime(31 卡,轨道 A 心脏)→ LumioGame(玩法内容)→ A3 收口;并行短线:VoxelEngine 解阻 → CoreEngine B1 Loader → B2 Differential 汇合;Server C# 宿主支撑 A1;NativeCore/Client 按既有节奏。

| Wave | 各仓动作(→ 为前置) | 完成判据 |
|------|------|------|
| **W0 解阻与对账**(不完成不进 W1) | VoxelEngine:固定 aarch64 工具链(rustup 当前默认 x86_64/Rosetta,pin 1.98.0 需下载)→ 首次真实 `cargo test --workspace --all-features` → 修复(预算 1–2 轮)→ evidence 翻绿 → R-00203 重审/R-00204 放行;GameRuntime:修 SDK pin 口径 → Mac 实跑五卡验收 → 验收流转;CoreEngine/Server:Windows 侧推送 → 总调度重核六张验收中卡;架构仓:R-00003 对照 W1 发布物核对补 Golden;NativeCore:66 卡批量补验收流转 | 测试矩阵绿;六张问题卡重核完毕;R-00203=APPROVE |
| **W1** | GameRuntime R-00139/00140/00141 + R-00149/00150/00152(config/persistence/ecs 六卡串行);架构仓 R-00004(Canonical/Digest 族 ADR);LumioGame 设计卡(待授权落单);Server C# 宿主设计卡(待授权落单);Client 可选 persistence/Serilog | 每卡收口门槛输出 + 流转 |
| **W2** | GameRuntime command/gas/coordination/replication 七卡;架构仓 R-00005/00006/00008(签名信任/Loader/Evidence 族,8 项 P0 Gate 全关);CoreEngine 骨架启动(重核后);Server C# 宿主首批实现 + Rust wave0 卡 R-00188(重核后) | P0 Gate 全关;C# 宿主与 Client 回环连通 |
| **W3** | GameRuntime simulation/testing 收口(R-00199 = Foundation 收口);LumioGame Ability 实现;Server–Client A1 联调(跨进程挖方块 + 断连 Resync);CoreEngine B1 最小 Loader;VoxelEngine B2 准备 | A0/A1 退出条件;B1 冒烟 |

W3 之后:A2(浏览器 WASM/WebGL)→ A3 收口(≥5 浏览器、存档、Replay、故障注入);B2 汇合则权威体素切 Rust,未汇合按 MVP 计划以 ReferenceVoxelPort 出演示。Server Rust 51 卡随 Production Hardening 立项批量启动。

## 6. 风险与开放决策

1. **VoxelEngine 首跑失败面未知**(2.4 万行首次链接执行):预算 1–2 轮修复;失败面大则 B2 顺延,MVP 按计划 fallback ReferencePort,不让轨道 B 拖 A。
2. **工具链漂移是当前第一工程风险**:双机三处失配(link.exe 缺失已致 Voxel 验证债;SDK pin 字面不可满足;rustup Rosetta 默认)。W0 统一:各 Rust 仓明确 aarch64 目标并记录环境入 evidence;GameRuntime pin 修正为「SDK 族 + runtime 10.0.11」双口径。
3. **A1 WebSocket 若暴露新公共字段/错误码** → 立即走契约变更回路(ADR→Schema→Fixture→镜像);TransportProfile 登记卡(待授权)提前探明。
4. **D-009/D-011 维持阻塞**(MVP 用 replication envelope + auth 存根);**D-014 不为 MVP 提前冻结**,P2 streaming 启动前按 D-013 模式出确认书即可。
5. **单人多机开发的证据纪律**(§4.3)不立稳,漂移必复发。

## 7. 本次会话已执行动作(2026-08-28)

- MVP 计划 Draft → **Adopted**(立项说明含定位/总纲/MS-00001 锚点/C# 宿主归属裁决)。
- GameRuntime 五卡(R-00112/00127/00131/00133/00138):各附对账评论(origin 证据 + SDK pin 缺陷 + 验证受阻说明),流转 需求池→评审中→已评审→**实现中**(读回确认)。
- R-00049 补验收定义 2 条(源记录卡收口条件)。
- VoxelEngine R-00057..R-00060:各附 VOX-D-001..004 架构所有者裁决同步评论(引用确认书 `LGE-V1.4-VOX-D-P0-2026-08-28`)。
- CoreEngine R-00011..14、Server R-00186/00188:各附核验差异记录评论(不改状态)。
- 全部写入均已读回核验;评论/流转在 cross-repo-delivery 既有持续授权范围内。

## 8. 待授权事项(本报告随附请求)

1. **新落单 4 张卡**(需一次性写入授权):RM-00001 两张(D-014 处置声明卡、TransportProfile WebSocket 档登记卡)、RM-00008 一张(LumioGame 设计文档卡)、RM-00006 一张(MVP C# 宿主设计卡)。
2. **Windows 侧推送**:CoreEngine/Server 本地 main 推送 origin(用户在 Windows 机器执行),推送后总调度重核六张问题卡。
3. **本报告与 MVP 立项提交推送**至架构仓 origin/main。

---

*附:本机工具链事实(2026-08-28)——dotnet SDK 10.0.400;rustc/cargo 1.94.0,rustup default `stable-x86_64-apple-darwin`(Rosetta,需切 aarch64);Apple clang 21.0(链接器可用)。*
