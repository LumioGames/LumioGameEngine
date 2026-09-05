---
name: 2026-09-05-engine-repos-progress-assessment
description: 八个实现仓逐仓进度盘点（自最底层 NativeCore 起）——仓库事实、Workflow 对账、阶段判定与下一步；排下一阶段派活前查
metadata:
  type: doc
  status: 实施中
---

# 2026-09-05 引擎实现仓逐仓进度盘点

> 盘点流程按 `skills/td-progress-audit`。真值分层：需求状态以 Workflow（lumiogamesengine）为准；代码以 **已推送 origin 的提交** 为准；测试以 **本机实跑输出** 为准，`cargo check` 与「已通过」声称不算。
> 本文按仓逐节推进；尚未盘到的仓标「待盘」。全部盘完后补执行摘要与 wave 编排。

## 1. 执行摘要（待全部仓盘完后补）

| 仓 | origin/main | 阶段判定（一句话） | 状态 |
| --- | --- | --- | --- |
| LumioNativeCore | `70b9834`（2026-09-03） | 地基打完、门全绿、只有定时器被真用上；其余模块无消费者，整仓仍绑着已退役的 Baseline 契约；唯一未完成是 3 张与 ADR-064 脱节的 GAS 卡 | 已盘（§2.1，已合并 Codex 会话报告） |
| LumioVoxelEngine | `e5c056e`（2026-09-05） | 服务端半边（编码 / 存储 / 事务 / 派发 / 读 / 物理 / 绑定 / pin）已合入 origin 并经 SDK 14 槽可达，但没有一个上层消费者；15 张蓝图卡的 Workflow 回写落后于代码；整仓仍绑着 V1.4 合同制，活代码与 SDK 都在用它的死名字；27 张 V1.4 旧卡待作废 | 已盘（§2.2） |
| LumioGameRuntime | — | — | 待盘 |
| LumioServer | — | — | 待盘 |
| LumioClient | — | — | 待盘 |
| LumioGame | — | — | 待盘 |
| LumioConfig | — | — | 待盘 |
| LumioPlatform | — | — | 待盘 |

## 2. 各仓详情

### 2.1 LumioNativeCore（依赖图最底层 · 领域无关 Rust 内核）

> 本节合并了同日另一会话（Codex）的 `2026-09-05-nativecore-progress-audit.md` 与 `2026-09-05-nativecore-kickoff-dispatch-prompts.md`（两文件已删除，内容并入此处）。两份材料对仓内现状判断一致；对方多跑了 build / 本仓 spec-lint / spec-lint.test 并先发现 timer 文档漂移，本节采纳；对方「等架构仓发布 provider composition 契约」的下一步前提与 Living Architecture 冲突（见漂移 ①），本节改写。

#### 仓库侧事实（本机 macOS，2026-09-05，两会话各自实跑、结果一致）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `70b9834`（2026-09-03，PR #7 squash），0 ahead / 0 behind；工作区只有两个未跟踪 `.DS_Store` | `git status -sb`、`git rev-list --left-right --count` |
| PR / Issue / CI | 7 个 PR 全 MERGED，0 open；0 issue；main 最近 8 次 CI 全 success | `gh pr list`、`gh issue list`、`gh run list` |
| 规模 | 10 crate + xtask；`.rs` 合计 12,202 行 = 源码 6,051 + 测试 6,101 + bench 50；`#[test]` 129 个；测试目标 150；`todo!/unimplemented!/TODO` 0 处 | `find`、`cargo test -- --list` |
| 质量门 | `cargo fmt --check`、`cargo clippy --all-targets -D warnings`、`cargo build --workspace`、`cargo test --workspace`（150 passed / 0 failed）、`cargo xtask check-dep-dag`（11 crate 合规）、`cargo xtask dump-symbols`（0 未批准符号）、本仓 `spec-lint` OK、`spec-lint.test` 13/13：**全部 exit 0** | 本机实跑 |
| 被 SDK 消费 | 架构仓 `engine/native/modules/sdk-native/Cargo.toml` 路径依赖 `lumio-kernel`、`lumio-timer`；`cargo build/test -p lumio-engine-native` 通过（6 passed） | 本机实跑，rustc 1.89 |
| 跨仓依赖 | VoxelEngine / Server / Runtime / Client / Game / Config / Platform 对 NativeCore 任一 crate **零引用** | `grep` 各仓 `Cargo.toml` / `*.csproj` |
| 分支 | 本地 5 条 `claude/*` 与远端 3 条 `feat/*` 全部已合入 main，可清理（删前确认） | `git merge-base --is-ancestor` |

#### 模块状态（规划 × 代码 × 消费 × 缺口）

| 模块 | 规划 | 代码 / 测试 | 有没有人用 | 到下一阶段缺什么 |
| --- | --- | --- | --- | --- |
| contract-types | approved / I0 | 631 行 / 14 | 只被本仓其它 crate 用 | **锚在已退役的 Root ABI bundle**（`LGE-V1.4`、ADR-040 `lumio_core.h` golden 与 digest 漂移门），见漂移 ① |
| error / capability / handle / memory / kernel-context（`lumio-kernel`） | approved / I0 | 1,410 行 / 28 | SDK 只以 `TypeId::of::<HandleKey>()` 做编译期标记，**根表没有槽位通向它们** | 没有宿主消费；跨进程装载证明为零 |
| job | approved / I0 | 585 行 / 8 | 无 | 消费者（tick 第 6 相）未开卡；缺负载曲线、Sanitizer / Miri |
| spatial | approved / I1 | 413 行 / 6 | 无 | 现为通用 AABB 查询（grid 参考 + rstar），**不是 ds-server.md M5 要的「双半径候选进 / 出对有序清单」**；缺跨平台 Benchmark |
| timer | 新增（ADR 0008） | 1,281 行 / 40 | **唯一被真用上的模块**：SDK 根表 14 个 `timer_*` 槽转发到它；Server / Client 经 SDK 消费（R-00374 / 375 / 389 done） | 仓内三处文档口径不一（漂移 ⑥） |
| native-core-ffi | approved / I0 | 824 行 / 16 | 无。按设计不导出 C 符号；`lumio_core_api` provider 表对应已退役的 `lumio_core.h`，`lumio_core_init` 槽为 `None`（源码注释「Still blocked, R-00179」） | Living Architecture 下**没有装载路径**；不是「等契约」，是「契约已退役」 |
| codec / diagnostics | pending / I1，feature-gated 默认关 | 277 / 302 行，5 / 5 | 无 | 等架构源批准公共语义；ADR 0005 前不进 ABI |

#### Workflow 对账（RM-00002 · 只读；已完成的卡只做一次机器复核，不再逐张看）

| 项 | 实测 |
| --- | --- |
| 总数 | 71 张：**68 done、3 backlog**；phase `ready`；0 工作项、0 缺陷 |
| 68 张 done 的机器复核 | 全部有含 origin 可核提交号的证据评论（68 / 68）；验收项引用的 69 处文件路径与 72 处标识符在 `70b9834` 全部存在（1 处 `modules/*/README.md` 为 glob 假阳性）；**66 / 68 验收项全 passed**，R-00007（蓝图源卡）与 R-00083 的 9 条验收项停在 `not_started`（交付有 PR #4 `e2a801e` 证据，只是验收项没跑）。**结论：done 卡可信，不再复核。** |
| 3 张未完成 | R-00302 GAS-A3 帧调度器契约、R-00308 GAS-N01 无状态求值 / Tag / 堆叠内核、R-00309 GAS-N02 帧调度器实现。均 P0、2026-08-30 建、0 评论、无负责人、正文有编码损坏（`??????`） |
| 3 张卡的前置 | 全部 backlog：架构室 R-00299 / 300 / 301（GAS-A0 / A1 / A2）、Runtime 室 R-00303 ~ 307（GAS-R1 ~ R5）。也就是说 **2026-08-30 那套 GAS 卡族（架构 4 + Runtime 5 + NativeCore 3）整体没动**，而 GAS 方向已由 ADR-064（炸弹人切片，C# 一份实现，Rust 下沉推到阶段 2）与 RM-00014 R-00468（最小 GAS 可执行纵链）接管 |
| 其它 Room | RM-00011 的 R-00352 / 372 / 386 done 且有 PR 证据；**RM-00013（炸弹人）与 RM-00014（九项必备能力）0 张 NativeCore 卡** |

#### 漂移与问题（按严重度）

① **整仓仍绑在已退役的 Baseline 制度上（结构性）。** 架构仓已按 ADR-059 / `architecture.md` §6–§7 转入 Living Architecture：Baseline、`packages/`、`tools/lumio_contract.py`、contract mirror 全部删除（本机实测不存在），唯一 ABI 真值是 `engine/abi/native-abi.json`。NativeCore 这边：README「架构基线 `LGE-V1.4-2026-08-27`」；`docs/architecture/` 保留 5 版架构正文 + `abi/` bundle 镜像（276 KB）；`lumio-contract-types` 的 golden / 漂移门锚在退役 bundle 的 digest 上；CI `readme` job 断言基线字符串与镜像 sha256；`lumio-native-ffi` 的 provider 表对应退役的 `lumio_core.h`。`architecture.md` §7 第 4 条「活动源码和 CI 不再依赖 LumioCoreEngine、Baselines 或 contract mirror」是迁移完成条件，**NativeCore 目前不满足**。另：R4 整体审查（2026-09-04）称「旧仓名 NativeCore 零命中」不准确，`git grep -l LumioGameEngineArchitecture` 命中 20 个已跟踪文件，`CoreEngine` 命中 16 个。

② **库存与消费严重不对称。** 约 6,000 行内核里只有 timer（约 1,300 行）有真实消费链；handle / memory / job / kernel-context / spatial 五块地基没有任何宿主能摸到。这不是代码质量问题（门全绿），是**没有需求方**：两个在途 Room 都没给 NativeCore 排卡；设计里唯一的预期消费者是 ds-server.md M5 空间粗筛内核经 tick 第 6 相 `NativeJobBarrier` 收回，但无卡。

③ **3 张 GAS 卡与现行设计脱节。** 卡按 2026-08-30 裁决（板 11a：帧调度器落 NativeCore）写成，要新建 `lumio-gas-scheduler` / `lumio-gas-eval` crate；两天后 `lumio-timer` tickFrame 模式已提供「按 Tick 推进、每帧一次 drain 到期清单」，与「帧调度器批量取件」职责重叠；ADR-064 把 Rust 下沉定为阶段 2，gas.md 明说 0-6 帧调度器不在炸弹人切片。按「如无必要勿增实体」，重启前必须先回答：M9 帧调度器是不是 lumio-timer tickFrame 的扩展？

④ **Workflow 状态欠账（轻）。** R-00007、R-00083 共 9 条验收项 `not_started`。

⑤ **卫生项（轻）。** 5 本地 + 3 远端已合入分支可清理；3 张 GAS 卡正文编码损坏，重派前要重写。

⑥ **timer 归属口径三处不一致（轻，纯文档；Codex 会话先发现）。** 事实：内核 `lumio-timer` 在 NativeCore，C ABI 插头 `timer_*` 在架构仓 `engine/native/modules/sdk-native/src/timer.rs`，经 `native-abi.json` 到达托管侧（ADR 0008 修订记录 / ADR-057 第 9 条）。仓内：`modules/timer/README.md` 与 `docs/specs/native-core-module-map.md` 仍写 ADR 0007 时代的「不进 native-abi.json、不导出」；`.spec/knowledge/standards/repository-architecture.md` 写成「本仓……经 `native-abi.json` 的 `timer_*` 槽导出」。三处应统一成同一句话，否则下游会再造一份 timer FFI 副本。

⑦ **发布硬化不足（P2，Codex 会话提出）。** CI Native job 只跑 Ubuntu，无 Windows / macOS 产物与 ABI 装载证据；无 Sanitizer / Miri / SBOM / 可复现构建。按 `architecture.md` §6 这些属正式硬化阶段，预上线不开卡，只登记。

#### 近期架构演进对 NativeCore 的影响（逐项核对）

| 演进项 | 对 NativeCore 的含义 | 现状 | 需要的动作 |
| --- | --- | --- | --- |
| ADR-059 CoreEngine 退役 + Living Architecture（ABI 唯一真值 `native-abi.json`） | Root ABI bundle / Baseline / mirror 整套制度作废 | 仓内残留见漂移 ① | **W0 卫生卡**：删镜像与 CI 基线断言；`contract-types` 改锚 `native-abi.json` 或删掉 golden 门；`lumio-native-ffi` 退役 provider 表删除；README 按 Living Architecture 重写 |
| ADR-056 §7 / ADR-057 第 9 条：单一定时内核，插头归架构仓 | timer 只留内核 | 已落地（R-00372 / R-00386） | 三处文档统一（并入 W0） |
| ADR-064 炸弹人 GAS：C# 一份实现，Rust 下沉 = 阶段 2 | 3 张 GAS 卡失去当前需求方 | 3 张 backlog，前置全 backlog | **挂起**；重启时先裁「帧调度器 = lumio-timer 扩展 or 新 crate」，再重写卡面 |
| ds-server.md M5 空间粗筛内核 + tick.md 第 6 相 `NativeJobBarrier` | NativeCore 下一个真实消费者：候选进 / 出对有序清单、逐字节确定性、双端复用 | 现 `lumio-spatial` 是通用 AABB 查询，形状不对；`lumio-job` 是它的运载体，无卡 | 待 DS 视野排期时开 **契约卡 + 实现卡**；不在炸弹人 Stage 0（19×19、8 Bot 不需要 AOI） |
| ADR-063「爆炸传播下沉 Rust」被否；ADR-065 F05：spatial 只证明数值类型 | 炸弹人不向 NativeCore 要新东西 | — | 无 |
| ADR-062 体素物理查询 C 签名进 `native-abi.json` | 同一根表的 VoxelEngine 槽位，不是 NativeCore 的 | 槽数 0，待 VoxelEngine 开卡 | 无（只提醒：根表扩展只追加不插入） |
| CL-1 WASM 调研（LumioClient PR #18）：浏览器客户端用 .NET browser-wasm 跑 Runtime，**不装 Native 库** | 「单一定时内核」「空间粗筛双端复用」在浏览器端不成立；帧驱动靠浏览器定时器 | 调研未触及 NativeCore | **需 Owner 裁决 D3**：浏览器端允许 C# / JS 兜底（承认「单一内核」只在有 Native 的宿主成立），还是 NativeCore 出 wasm32 目标（`lumio-job` 依赖 crossbeam 线程，wasm 默认无线程，成本不低） |
| LumioServer Rust 宿主 timer ABI loader Windows-only（`native_timer.rs` `cfg(not(windows))` BLOCKED） | 影响 NativeCore 在 mac / Linux 的端到端证明，但归 LumioServer | NativeCore 自身 mac 构建测试全绿 | 记入 Server 仓盘点，不在本仓开卡 |
| `architecture.md` §6 预上线质量边界 | Production Hardening 推到正式硬化 | 未开始 | 不开卡，只登记（漂移 ⑦） |

#### 阶段判定

按仓自己的路线图：**Architecture Gate ✔ → Foundation ✔（9 模块全落地）→ NativeHeadless 半程**（spatial 有了但形状待改；codec / diagnostics 仍是默认关闭原型；「CoreEngine 包加载」路线随 CoreEngine 退役作废，改为 SDK 路径依赖）**→ Production Hardening 未开始**。

完成度口径（沿用 Codex 会话的两分法，数值为判断非测量）：**底层算法 / 生命周期约 70–80%**；**可被上层直接用上的部分约 10–15%**（10 个模块里只有 timer 通向宿主；对方给的 35–45% 把「等契约发布就能装载」算了进去，而该契约已退役，故下调）。

放到全项目里看：NativeCore 是「**一座打好地基、只亮了一间房的仓库**」，对当前切片不构成阻塞，也没有在途需求；下一阶段完全取决于上层何时提出真实消费（DS M5 空间粗筛、GAS 阶段 2），不取决于它自己再堆代码。

#### 剩下的单子怎么补（RM-00002 · 待 Owner 裁决后落卡，新建卡须逐次授权）

| # | 卡 | 性质 | 内容要点 | 前置 |
| --- | --- | --- | --- | --- |
| N-W0 | 退出旧合同制清理（**已建：R-00473**，8 条验收项） | 小卡，纯本仓 | 删 `docs/architecture/` 镜像与 `.baseline.sha256`；删 CI `readme` job 基线断言；`lumio-contract-types` 去掉 Root ABI bundle golden / 漂移门，改锚 `native-abi.json` 或只保留本仓内部类型；删 `lumio-native-ffi` 退役 provider 表（保留 panic 边界 / 句柄校验若仍有用，否则整 crate 删）；README、`.spec` 与模块 README 去掉 `LGE-V1.4` / `LumioGameEngineArchitecture` / CoreEngine 口径；timer 三处文档统一 | 无；D1 裁决 |
| N-GAS | R-00302 / R-00308 / R-00309 | 既有 3 张 | **已作废（已否决）并各补一条评论**：前提失效、重开条件 = GAS 阶段 2 且 benchmark 证明需要下沉 | D2 已裁 |
| N-M5 | 空间粗筛内核契约卡 + 实现卡（**新建，暂不派**） | 契约先行 | 契约：`(viewer, target, enter|leave)` 有序清单、排序键、双半径、确定性义务、在第 6 相收回、根表槽位形状；实现：`lumio-spatial` 改形 + `lumio-job` 运载 | DS 视野排期；ECS 视野表真值就位 |
| N-ACC | R-00007 / R-00083 验收项补记 | 写操作 | 9 条 `not_started` → 按 PR #3 / #4 证据补跑或补记 | 写授权 |
| — | 分支清理 | 卫生 | **已完成**：远端 3 条 `feat/*` 删除（`git push origin --delete`）；本地 `claude/*` 与 `.claude/worktrees` 复核时已不存在，读回只剩 `main` / `origin/main` 与单一 worktree | 已确认执行 |

不开的卡及原因：Production Hardening（§6 推到正式硬化）；wasm32 目标（等 D3）；codec / diagnostics 转正（等架构源批准公共语义）。

### 2.2 LumioVoxelEngine（体素世界 · 唯一 Rust 实现）

> 盘点当天另一会话（体素落地总指挥）刚把蓝图 `voxel-impl-2026-09-04` 的 15 张卡合入两仓 origin/main（VoxelEngine `e5c056e`、架构仓 `4d6d2c3`），Workflow 回写还没跟上。本节以两仓 **origin HEAD** 为准：VoxelEngine 本地已 `git pull --ff-only` 到 `e5c056e`；架构仓本地 main 领先 origin 2 个未推送提交（NativeCore 盘点 `4a5f596` / `94a2540`）、落后 59 个，不能快进，故架构仓 origin 用只读快照（`git archive origin/main`）核验，本地 main 未动。

#### 仓库侧事实（本机 macOS，2026-09-05 实跑）

| 项 | 实测 | 出处 |
| --- | --- | --- |
| origin/main = 本地 main | `e5c056e`（2026-09-05，「merge: integrate R-00434 voxel public layer」），0 ahead / 0 behind，工作区干净 | `git status -sb`、`git rev-list --left-right --count` |
| 最近 5 个提交 | R-00441 键契约（`58746f1` → `d76b9ab`）；「integrate reviewed voxel implementation batch」`10bf536`（+6,297 行实现、13 个新测试文件，一次性覆盖 I-2 ~ I-11 全部实现卡）；「ratify block resolution and catalog precedence」`8c88efd`（ADR-066 落地）；`e5c056e` 合入。**后两批直接推到 main，没有 PR**（PR 列表停在 #15，2026-09-04） | `git log`、`gh pr list` |
| PR / Issue / CI | 15 PR 全 MERGED、0 open；0 issue；main 最近 8 次 CI success（含 `e5c056e`） | `gh` |
| 规模 | 7 crate；`.rs` 源码 26,900 行 + 测试 15,570 行；`#[test]` 392 个；`todo!/unimplemented!/TODO` 0 | `find`、`grep` |
| 质量门 | `cargo fmt --check`、`clippy --all-targets --all-features -D warnings`、`check --no-default-features`、`build`、`check-crate-dag`（7 crate）、`check-generated-clean`、本仓 spec-lint：**全部 exit 0**；`cargo test --workspace --all-features --no-fail-fast`：**388 passed / 1 failed**。唯一失败 `vendored_copy_matches_upstream_when_available`——它把仓内契约副本与**同级架构仓工作区**逐字节比对，而本机架构仓 main 落后 origin；用 `LUMIO_ENGINE_WIRE_DIR` 指向架构仓 origin 快照后 12/12 通过。**不是仓的问题，是本机架构仓没拉** | 本机实跑，rustc 1.98.0（`rust-toolchain.toml` 与 CI 同钉） |
| 契约副本 | `crates/lumio-voxel-contracts/wire/voxel-world-v1.json` SHA-256 `56d555fd…` **= 架构仓 origin 同名文件**（52 错误码 / 57 规则 / 53 + 57 用例）；仓内 `CONTRACT_SHA256` 常量与副本一致，一致性测试逐字段断言。**是消费活契约，不是又复印一份**。架构仓本地 main 那份仍是 51 / 56（差 ADR-066 加的 `unregistered_block_type` 与规则 `blockType.resolution-domain`），拉 origin 即齐 | `shasum`、`node` 对比 |
| 被 SDK 消费 | 架构仓 origin `sdk-native/Cargo.toml` 路径依赖 4 个 crate（world / domain / ops / contracts）；`sdk-native/src/voxel.rs` 1,486 行把根表体素槽转发到这些 crate；`native-abi.json` `root.fields` 35 个，其中体素 14 个（`block_read_cell / box / column`、`block_write_prepare / commit / abort`、`section_revision_query`、`residency_pin_declare / release / status`、`raycast / sweep / overlap`）；托管侧 `VoxelFacade.cs` 779 行 + 测试 10 个。origin 快照上 `cargo build / test -p lumio-engine-native` 通过（`root_api` 14 passed）、`verify-wire` 30/30、`generate-abi.test` 19/19、spec-lint OK | 本机实跑（快照） |
| 跨仓依赖 | Runtime / Server / Client / Game / Config / Platform / NativeCore 对 VoxelEngine crate **零引用**。Runtime / Server / Client 各自仍持有 V1.4 生成物复印件，体素代码里 `VoxelChunkResidency` 72 处、`ChunkRevisionSet` 45 处、`VoxelChunkPage` 18 处，`Section` 0 处——**Section 改名一处都没到消费方**。LumioGame 体素相关代码 0 个文件（炸弹人地形只在 ADR 0019 与本地草稿卡里） | `grep` 各仓 |
| 分支 / worktree | 本地 16 条分支（14 `claude/*` + `fix/vox-d-001-004-post-sha256-retest` + `test/r-00290-sha256-kat`）全部已合入 main；远端只有 main；`.claude/worktrees/` 下 3 个 detached worktree 指向的提交都已在 main | `git merge-base --is-ancestor` |

#### 模块状态（`voxel.md` 模块图 × 代码 × 卡 × 消费）

| 模块 | 代码（origin `e5c056e`） | 卡 | 有没有人用 | 到下一阶段缺什么 |
| --- | --- | --- | --- | --- |
| M1 分层与方块编码 + M1a 目录 | `domain/block.rs` 877 行（BlockId 位段 / 段表 / 材质类表 / 行为模板 / 目录校验 / `cellOffset` 唯一算式）、`key.rs` 326 行；测试 42 + 12 + 9 + 22 | R-00434（实现中）、R-00441（已完成） | SDK `voxel.rs` | 无；ADR-066 三条裁决已进契约与代码 |
| M2 三态存储 | `section/block_storage.rs` 353 + `block_payload.rs` 329；测试 10 + 16 | R-00435（评审中） | SDK | 无 |
| M3 光照 | **0** | 无（R-00433 列为非目标） | — | 客户端要画地形时立项 |
| M4 网格生成与零拷贝 | **0**（`lumio-voxel-project` 里只有 physics_query） | 无（非目标） | — | 同上 |
| M5 改动层与派发 | `section/dispatch.rs` 120 + `modification_layer.rs` 115 + `delta.rs` 92；测试 11 | R-00436、R-00458（评审中） | SDK 未暴露派发面 | 派发到 DS 的接线在 Server 仓 |
| M6 权威写入与事务 | `ops/mutation/*`（结构化条目、prepare / commit、幂等回执、原子发布）；测试 6 + 既有 | R-00438（评审中）；旧 R-00096 / R-00104（评审中） | SDK `block_write_*` | 无 |
| M6a 方块与实体绑定 | `domain/binding.rs` 878；测试 7 | R-00447（评审中） | 无（`NetEntityId` 接线归 Runtime） | 跨域同提交点要 Runtime 把 `IVoxelWorldPort` 公开（R-00469） |
| M7 物理检测 | `project/physics_query.rs` 1,209；测试 16 | R-00448（评审中） | **无**——根表 `raycast / sweep / overlap` 三槽只有声明，`voxel.rs` 没有路由（R-00443 按卡边界 declaration-only） | 一张「路由物理槽」卡（架构仓） |
| M7a 批量读 | `ops/query/block_read.rs` 1,133；测试 6 | R-00437（评审中） | SDK `block_read_*` + C# `VoxelFacade` | Runtime / Game 零消费（R-00469 未派） |
| M8 驻留 / pin | `world/residency.rs` 865；测试 9 + 7 | R-00440、R-00452（评审中） | SDK `residency_pin_*` | 真·按玩家位置的流式加载无卡；炸弹人整图 pin 不需要 |
| M9 存档与恢复 | 只有 V1.4 时代的 snapshot / restore 脊柱（capture、shadow root、restore） | 旧 R-00134 / R-00136（评审中，QA 不通过） | 无 | 非目标；新口径（体素与实体成组原子激活）未开卡 |
| M10 离线检查器 | 0 | 无（非目标） | — | — |

#### Workflow 对账（RM-00003 + 蓝图 `voxel-impl-2026-09-04` · 只读；done 卡只做一次机器复核）

| 项 | 实测 |
| --- | --- |
| RM-00003 总数 | 55 张：28 done、**13「评审中」（`in_review`）**、14 backlog；0 实现中 / 验收中。`in_review` 在本项目状态机里是**「评审中」——需求池之后、已评审 / 实现中之前的早期状态**（transitions 实查：需求池 → 评审中 → 已评审 → 实现中 → 验收中 → 已完成，另有已否决），不是「已开工」 |
| 28 张 done 的机器复核 | 28 / 28 评论里都有 origin 可核提交号；验收项与评论引用的 91 处路径 76 处在 origin HEAD 存在，15 处缺失全部是 ADR 0013 改名前的旧路径（`src/chunk/` → `src/section/`、`crates/lumio-voxel-persistence`、`tools/lumio_contract.py` 等）；**13 张 done 卡的验收项没跑**（12 张 × 4 条 `not_started`，R-00203 1 条 `failed`），R-00264 / R-00290 0 验收项。**结论：done 卡交付可信；验收项欠账是 V1.4 旧制度遗留，不再复核** |
| 13 张「评审中」 | 全是 2026-08-27 建的 **V1.4 框架卡**：R-00002 原始需求、R-00066 配置快照、R-00068 OriginToken、R-00070 Revision 分配器、R-00076 Staged Delta、R-00078 PublishedState Root、R-00080 查询计划器、R-00096 Prepare、R-00104 Commit、R-00116 World 生命周期、R-00134 快照、R-00136 恢复、R-00142 Port 适配。每张都有 8 月 28 日的交付评论（提交号在 origin）和 **8 月 29 日独立 QA「不通过」评论**（验收项无载体 / 自证循环），此后无人再动；52 条验收项全 `not_started`。它们的代码今天仍是仓的脊柱（revision / publication / mutation / query / snapshot / restore / world / port，约 2 万行），9 月 5 日的新批次直接叠在上面 |
| 14 张 backlog | 同一批 V1.4 卡，全部「未开工」：Streaming ×3（R-00151 / 153 / 155）、Spatial ×2（R-00163 / 166）、Migration ×2（R-00169 / 170）、Project ×1（R-00182）、Mesh / Collision Source ×2（R-00193 / 194）、测试 ×2（R-00196 / 198）、QA 发布门 ×2（R-00204 / 208）。卡面按 V1.4 的 Demand / Ticket 流式、Revision-scoped Source、生成 Manifest 迁移、LocalEmbedded 双树写；新模块图的对应物（M8 流式、M4 网格、M9 转档）形状都不一样；Spatial 两张与「不为地形建空间划分树」直接冲突 |
| 蓝图 15 张 + 来源卡 | **27 张卡没有 Room**（`roomId` 为空）：R-00432（已否决）、R-00433（backlog，来源卡）、15 张活卡、10 张 9 月 4 日重复建出来的已否决卡（R-00442 / 444 / 446 / 449 / 450 / 451 / 453 / 454 / 455 / 457）。活卡状态：R-00441 已完成（6 / 6 passed）；R-00434 实现中；R-00439 验收中；其余 12 张（435 / 436 / 437 / 438 / 440 / 443 / 445 / 447 / 448 / 452 / 456 / 458）**停在「评审中」**，但每张都有 9 月 5 日 12:31–12:36 的「Batch final independent review PASS」评论，引用的是**本地** main（VoxelEngine `d76b9ab` + 补丁 SHA；架构仓 `766e1ae`）并注明「未创建新 commit / push」——这些补丁现已在 origin（`10bf536` / `8c88efd` / `e5c056e`；架构仓 `4d6d2c3`）。**93 条验收项全 `not_started`**（Voxel 66、架构 27）。R-00434 只有 9 月 5 日 01:33 的深审 RETURN 评论（三条契约冲突），之后 ADR-066 裁决落地、代码合入，卡上没有后续评论 |
| 其它 Room | RM-00013 R-00427（LumioGame `ITerrainStore` 内存版，backlog，P0）；RM-00014 R-00469（Runtime 消费体素批量读写 + 跨域提交接线，backlog）——这两张是「体素真后端接到游戏」的全部剩余路径 |

#### 漂移与问题（按严重度）

① **仓库领先 Workflow：15 张蓝图卡的代码已在两仓 origin，卡还停在「评审中」。** 事实链：9 月 5 日 12:3x 独立复审 PASS（引用本地 main）→ 14:55 前后推到 VoxelEngine origin（`10bf536` / `8c88efd` / `e5c056e`）与架构仓 origin（`4d6d2c3`）→ 卡未流转、评论没有 origin 提交号、93 条验收项没跑。解铃条件：归 Room → 评审中 → 实现中 → 验收中 → 评论补 origin 提交号 → QA 逐条跑验收项 → 已完成。在此之前不得向已完成流转。

② **整仓仍绑在已退役的 V1.4 合同制上（结构性，与 NativeCore 漂移 ① 同款，且深一层）。** 复印件与门：README「架构基线 `LGE-V1.4-2026-08-27` / 唯一架构源 `LumioGameEngineArchitecture` / `python3 tools/lumio_contract.py validate`」；CI `readme` job grep 基线字符串并 `sha256sum -c docs/architecture/.baseline.sha256`；`docs/architecture/` 6 版正文（152 KB）+ `docs/LumioVoxelEngine_Framework_Design_LGE-V1.3/`（264 KB）+ `docs/plans/lve-v1.4-implementation-blueprint.md` + `docs/evidence/decision-gates/VOX-D-001~008`；`crates/lumio-voxel-contracts/generated/`（420 KB：`lumio_core.h`、`RootAbi.cs`、6 个 C# 生成目录、descriptors）由 `tools/architecture/generated-lock.json` + `check-generated-clean` 锁住并进 CI；`legacy_baseline.rs` 保留 `voxel-chunk-page` / `VoxelChunkResidency` 两个旧 id；`.spec/AGENTS.md` 收口门槛仍写「公共契约变更必须在 `LumioGameEngineArchitecture` 通过 `lumio_contract.py`」；`modules/README.md` 10 模块图是旧图（mesh-collision / migration / spatial / streaming …）；`lumio-voxel-migration` crate 5 行空壳。机器计数：`LGE-V1` 118 个已跟踪文件、`LumioGameEngineArchitecture` 66、`CoreEngine` 21、`Root ABI` 11。**比 NativeCore 深的一层在活代码里**：33 个源文件用 `Generated*` 类型（`GeneratedVoxelConfig` / `GeneratedRevisionStamp` / `GeneratedVoxelWorldPortAdapter` / `GeneratedVoxelQueryRequest` …）、21 个用 `STABLE_ERROR_IDS`（第二套错误 id 命名空间）、8 个用 `BASELINE_ID` / `SCHEMA_EPOCH`、6 个用 VOX-D 决策门常量；test-support 9,657 行里 b0 / b2 / mvp / reference / fixture_runner 全是 V1.4 fixture 骨架；**架构仓 `sdk-native/voxel.rs` 也直接引用这些**（`GeneratedVoxelWorldPortAdapter` 5 处、`P0_DECISION_GATES` 3 处、`BASELINE_ID` / `SCHEMA_EPOCH` 各 2 处）——SDK 根表正在把 V1.4 的死名字往托管侧递。`architecture.md` §7 第 4 条的迁移完成条件本仓同样不满足。

③ **27 张 V1.4 旧卡无人认领。** 13 张 QA 不通过后停在评审中 8 天；14 张 backlog 卡面与新模块图形状不同。继续挂着会让「RM-00003 还有 27 张没做」这个数字一直是假的。

④ **消费方一个都没接上，且拿着死复印件。** Runtime `IVoxelWorldPort` 仍 `internal`（`TxnPrepareCoordinator.cs:73`，只有 Prepare / Commit / Abort / Query / ReadRevision 五个跨域事务方法，没有读写方块）；Runtime / Server / Client 三仓生成物复印件里 `VoxelChunk*` 旧名 135 处。今天体素唯一的消费者是架构仓 SDK 聚合层（14 槽 + C# `VoxelFacade`），托管侧再往上零消费。消费方清理归各仓那一站。

⑤ **物理三槽只声明不路由。** R-00443 按卡边界 declaration-only，R-00456 的 A-2 只路由了读 / 写 / revision / pin，`raycast / sweep / overlap` 没接到 `physics_query`；炸弹人 Stage 0 用不到物理查询，但 ABI 面「有槽无实现」会误导消费方。

⑥ **契约三处已知缺陷未修**（`reviews/2026-09-04-voxel-card-contract-drift.md` §6 第 1–3 条：rule 49 `onViolation` 错位、全量编码携带 `baseSectionRevision` 无错误码、pin 预算无常量）；R-00440 验收项 2「超出驻留预算的 pin 当场失败」因此没有机器可判的界。

⑦ **Workflow 卫生（轻）。** 27 张卡无 Room；10 张重复建出来的已否决卡；R-00433 正文停在「44 / 49 / 98」与旧轨道（缺 I-6 ~ I-11、A-2 ~ A-4）；R-00441 收口评论中文全部是 `??????`（Windows 侧编码损坏）。

⑧ **仓库卫生（轻）。** 16 本地已合入分支 + 3 个 stale worktree；最近两批合入没走 PR（审查轨迹只在 Workflow 评论与架构仓 `reviews/2026-09-05-r-00439-*.md`）。

⑨ **架构仓侧顺带发现（不归本站开卡，只登记）。** origin main 根目录被 RM-00011 会话提交了工作文件：`R-00406.json`、`baseline-binding.json`、`baseline-gameplay.json`、`.wf-report-R-0035x*.md` × 8、`.wf-evidence-r00357.txt`、`.wf-selfcheck-r00357.mjs`、`r5-01-fix-request.md`、`r5-01-review-findings.md`、`.sdd-scratch/`；`lumio-clr-host` 在 `cargo clippy -D warnings` 下有一个 dead-code 常量（`HDT_LOAD_ASSEMBLY_AND_GET_FUNCTION_POINTER`，本地 main 与 origin 同，不在本仓收口门槛内）；本机架构仓 main 领先 origin 2 个未推送提交、落后 59。

#### 近期架构演进对 VoxelEngine 的影响（逐项核对）

| 演进项 | 对 VoxelEngine 的含义 | 现状 | 需要的动作 |
| --- | --- | --- | --- |
| ADR-062 文末「明确不冻结的」三行（9 月 4 日口径） | ABI 体素 slot = 0 / SDK 只有编译期标记 / 实现面全零 | **三行全部过时**：14 槽已进 `native-abi.json`（origin）；SDK 4 crate 路径依赖 + `voxel.rs` 1,486 行 + `VoxelFacade.cs` 779 行；实现面 6,297 行 | 改写缺口表；Draft → Accepted 的条件（M1 / M2 实现验证）已满足，等 93 条验收项跑完再转 |
| ADR-066（R-00434 三条 Owner 裁决） | 哨兵 0..3、4..255 不可解析、目录校验结构优先 | 契约、ADR-062、`voxel.md`、VoxelEngine `block.rs`、SDK 已同步（origin） | 无 |
| ADR-063 世界模型 / `tick.md` 第 5、8 相 | 体素是参与者不是协调者；帧初读、帧末一批写、同帧多批合一 | 体素侧 prepare / commit 两段 + `expectedSectionRevision` + 幂等回执在；协调者（Runtime）侧 `IVoxelWorldPort` internal、没有 Tick 接线 | R-00469（RM-00014）负责，未派 |
| 炸弹人地形接 Voxel 真后端 | LumioGame `ITerrainStore` → 换 Voxel 实现 | LumioGame ADR 0019 已把口径对齐（y 竖直、BlockId u32、blockRead / blockWrite 形状、九种方块进官方段）；代码 0 行；R-00427 内存版 backlog；R-00469 backlog | 顺序：R-00427（内存版，Game 可先跑）→ R-00469（Runtime 经 SDK `VoxelFacade` 消费）→ Game 换实现；本仓无新卡 |
| Section / Chunk 改名 | 本仓 ✓（ADR 0013）；架构仓 ✓；消费方 ✗（死复印件） | — | 各仓那一站的 W0 卡 |
| 体素与 NativeCore spatial 边界（D4） | 体素派生碰撞归本仓（`physics_query` ✓），实体间粗筛归 NativeCore M5 | 一致；本仓对 NativeCore 零引用 | 无；RM-00003 两张 Spatial 旧卡与此冲突 → 作废 |
| D3（浏览器端没有 Native） | `voxel.md` 写「客户端体素 Rust 编成 WASM」；炸弹人 Stage 0 体素不进预测世界、客户端只按 Delta 重画 | 7 个 crate `#![forbid(unsafe_code)]`、活代码无线程 / 文件 / 网络依赖（test-support 除外）——wasm32 目标技术上便宜 | 不开卡，登记；客户端要画地形（M3 / M4）时一起立项 |
| 已裁原则 ①③（唯一最干净版本、如无必要勿增实体） | 本仓 26,900 行里约 2 万行是 V1.4 制度的脊柱与 fixture 骨架，新实现叠在上面；错误 id 有两套命名空间 | — | D8 |

#### 阶段判定

按 `voxel.md` §5：阶段 0「先立规矩」8 张里 7 张有代码（0-5 存档自描述是非目标）；阶段 1 垂直切片 6 步里**服务端半边**（挖一格 → prepare / commit → Delta；批量读；DDA 三种检测；箱子绑定；改动层派发）有代码，**光照、网格、客户端画面三步为零**，所以「老王挖一格土，另一个玩家看见」这条主线今天在任何宿主里都跑不通；阶段 2 / 3 未开始。

完成度口径（判断非测量）：**服务端算法与存储约 60–70%**；**可被上层用上的部分：经 SDK 根表 14 槽可达，但 Runtime / Game 零消费，端到端 0%**。放到全项目里看：VoxelEngine 是「**货已经上架、柜台（SDK）也开了、还没有一个顾客走进来**」的仓库；下一步都在别的仓（R-00469 Runtime、R-00427 Game），本仓自己剩的是把旧制度的架子拆掉。

#### 剩下的单子怎么补（RM-00003 / RM-00001 · 待 Owner 裁决后落卡，新建卡须逐次授权）

| # | 卡 | 性质 | 内容要点 | 前置 |
| --- | --- | --- | --- | --- |
| V-SYNC | 15 张蓝图卡回写（既有卡） | 写操作 | 归 Room（Voxel 卡 → RM-00003，架构卡 R-00439 / 443 / 445 / 456 → RM-00001）；评审中 → 实现中 → 验收中；每张一条评论补 origin 提交号；R-00434 → 验收中 | D9 授权 |
| V-QA | 93 条验收项跑批（既有卡） | QA 提示词 | 独立环境按验收项逐条实跑，通过才 → 已完成；提示词见 `plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` §三 | V-SYNC |
| V-OLD | 27 张 V1.4 旧卡（既有卡） | 流转 | 作废（已否决）+ 每张一条取代评论（指向新卡 / 非目标 / 保留代码的测试落点） | D7 |
| V-W0 | 退出旧合同制清理（**新建**，RM-00003） | 一张卡三层 | ① 文档 / CI / 镜像 / 旧仓名；② `generated/` 树 + `generated-lock.json` + `check-generated-clean` + `legacy_baseline.rs` + VOX-D 门与 V1.4 fixture 骨架；③ 活代码里的 `Generated*` / `BASELINE_ID` / `SCHEMA_EPOCH` / `STABLE_ERROR_IDS` / `from_generated`，错误 id 只留契约 snake_case 一套；卡面见 `plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` §一 | D8；与 A-W0 串行 |
| A-W0 | 架构仓 `sdk-native/voxel.rs` 去掉对 V1.4 名字的引用（**新建**，RM-00001） | 配套小卡 | `GeneratedVoxelWorldPortAdapter` / `P0_DECISION_GATES` / `BASELINE_ID` / `SCHEMA_EPOCH` 8 处引用改走活契约类型 | D8；V-W0 之后 |
| A-5 | 物理三槽路由到 `physics_query`（**新建**，RM-00001） | 小卡 | `raycast / sweep / overlap` → `lumio-voxel-project`；`root_api` 补 3 条测试；C# facade 补三个入口 | D10；V-QA 之后 |
| C-FIX | 契约三处缺陷（**新建**，RM-00001） | 契约小卡 | rule 49 改挂「批被部分应用」的错误码、全量编码携带 `baseSectionRevision` 加错误码、`limits` 加 pin 预算常量；ADR-062 修订记录；VoxelEngine 复制副本 | D11 |
| — | R-00433 正文更正 | 评论 | 52 / 57 / 110、决策账本补 ADR-066、轨道补 I-6 ~ I-11 与 A-2 ~ A-4 | 写授权 |
| — | 分支 / worktree 清理 | 卫生 | 16 本地分支 + 3 worktree | 删前确认 |

不开的卡及原因：M3 光照 / M4 网格（客户端画地形时再开，届时连 wasm32 一起）；M8 真·流式加载（炸弹人整图 pin，用不上）；M9 新口径存档、M10 检查器（R-00433 非目标）；Runtime 侧 `IVoxelWorldPort` 公开与 Tick 接线（R-00469 已有）。

### 2.3 LumioGameRuntime（待盘）

### 2.4 LumioServer（待盘）

### 2.5 LumioClient（待盘）

### 2.6 LumioGame（待盘）

### 2.7 LumioConfig（待盘）

### 2.8 LumioPlatform（待盘）

## 3. Workflow 现状（Room × 状态；只填已核对的）

| Room | 名称 | 总数 | done | in_progress | acceptance | backlog | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| RM-00002 | LumioNativeCore | 71 | 68 | 0 | 0 | 3 | 3 张 backlog 为 GAS 阶段 2 卡；21 条验收项 not_started（9 条挂在 done 卡） |
| RM-00011 | ECS Formal Entity and Chat | 42 | 36 | 2 | 1 | 3 | NativeCore 相关 3 张（R-00352 / 372 / 386）均 done |
| RM-00013 | 体素炸弹人 Stage 0 | 10 | 0 | 0 | 0 | 10 | 无 NativeCore 卡 |
| RM-00014 | 九项必备能力 | 14 | 0 | 0 | 0 | 14 | 无 NativeCore 卡；R-00469 是体素接 Runtime 的唯一卡 |
| RM-00003 | LumioVoxelEngine | 55 | 28 | 0 | 0 | 14 | 另 13 张停在「评审中」（`in_review`，早期状态）；13 张 done 卡 49 条验收项未跑；全部 27 张未完成卡属已退役的 V1.4 蓝图 |
| （无 Room） | 蓝图 voxel-impl-2026-09-04 等 | 27 | 1 | 1 | 1 | 1 | 12 张「评审中」+ 11 张已否决（R-00432 + 10 张重复卡）；活卡 15 张的代码已在两仓 origin，93 条验收项未跑 |

## 4. 漂移对照与证据核验结果

见 §2.1「漂移与问题」①–⑦ 与 §2.2「漂移与问题」①–⑨。两站盘点本身**未做任何 Workflow 写入**（NativeCore 站的写入是 Owner 授权后另行执行的，见 §7）。

## 5. 关键路径与下一阶段 wave 编排（待全部仓盘完后补）

## 6. 风险与开放决策

| # | 决策 | 建议方向 | 依据 |
| --- | --- | --- | --- |
| D1 | NativeCore 旧合同制残留：彻底清 vs 先留着 | **已裁决（Owner 2026-09-05）：三层全清，一张卡做完**——① 复印件目录 / README 合同口径 / CI 校对步骤 / 旧仓名；② `lumio-contract-types` 旧合同布局门与 xtask 生成命令、`lumio-native-ffi` 整 crate；③ kernel 内旧合同错误码 1044–1053 与 capability 注册表键，跨边界映射改由 SDK 插头对 `native-abi.json` 状态码 | ADR-059；`architecture.md` §7-4；Owner 原则「底层只保留唯一、最干净、最引擎的版本」 |
| D2 | 3 张 GAS-on-NativeCore 卡：派 / 挂起 / 作废 | **已裁决（Owner 2026-09-05）：作废** R-00302 / R-00308 / R-00309——前提已失效（新建帧调度器 crate 与 `lumio-timer` 重复；ADR-064 把 Rust 下沉推到阶段 2），正文编码损坏，无保留价值。阶段 2 真来时按新前提重开。gas.md 补哪一句（「GAS 到期调度在 Runtime 按帧计数」还是「帧调度器 = 定时内核扩展」）**等 D3 裁决**。同族另 9 张卡（架构室 R-00299 ~ 301、Runtime 室 R-00303 ~ 307）留待盘对应 Room 时处置 | ADR-064；如无必要勿增实体 |
| D3 | 浏览器客户端没有 Native：「单一定时内核」还成立吗 | **暂定（Owner 2026-09-05）：方案 A**——进预测世界的东西必须能在浏览器里跑，今天只有 Runtime C# 满足；预测世界里的到期是实体字段「第几帧到期」与当前帧号比较（炸弹 `FuseEndTick` 即此），不是定时器，不需要第二套内核；Native 定时内核只管宿主节拍（服务器推帧、Bot 节奏、断线保留窗）。两套并存自选回退被否（两套实现只在写出来那天一致；违反如无必要勿增实体 / 不留兼容层 / ADR-056 单一内核 / 确定性）。以后重计算内核进网页走「同一份 Rust 编成 WASM」，仍是一套；「参考 + 优化」双实现只允许用于有性能需求的重计算，编译期定死、逐字节差分测试。**待落文档**：gas.md M9 一节与 8 月 30 日裁决板 11a 改口；ADR-064 追加修订记录 | CL-1 调研；ADR-064 第 1 条；代码实测（Runtime 零处使用 Native timer） |
| D4 | 空间粗筛内核（DS M5）何时开卡 | **已裁决（Owner 2026-09-05）：B**——现在不开卡、不删；`lumio-spatial` / `lumio-job` 留作 M5 零件并标「形状待按 M5 契约改」；盘到 Server 仓看视野下发排期时，先开契约卡再开实现卡 | ds-server.md M5；tick 第 6 相；炸弹人 Stage 0（19×19、8 Bot）用不上 |
| D5 | `lumio-native-ffi` 整 crate 去留 | **已并入 D1 裁决：整 crate 删** | 漂移 ① |
| D6 | codec / diagnostics 两个默认关的私有原型留不留 | **已裁决（Owner 2026-09-05）：留着不动，不开卡**——全仓唯一一份、默认不编译不进产物、无运行时代价；DS 打包热路径过不了性能关时转正 codec，正式硬化阶段再谈 diagnostics | ds-server.md「下沉 native、边界不动」；ADR 0005 |
| D7 | RM-00003 的 27 张 V1.4 旧卡（13 张 QA 不通过后停在「评审中」+ 14 张 backlog）怎么处置 | **建议：整体作废（已否决）**，每张一条取代评论——写清交付提交号、QA 结论、现在由哪张新卡 / 哪条非目标 / 哪个测试文件接管。理由：卡面全按已退役的 V1.4 制度写（生成 Schema、VOX-D 门、LocalEmbedded 双树、Demand / Ticket 流式），验收项按那套制度根本跑不了；代码里有用的部分已被 9 月 5 日批次的测试覆盖。替代方案「重新 QA」要按旧验收项跑，等于给已作废的制度补考 | §2.2 Workflow 对账；已裁原则 ① |
| D8 | VoxelEngine 退出旧合同制清理的深度 | **建议：三层全清（同 D1），VoxelEngine 一张卡 + 架构仓一张配套小卡串行**——① 复印件 / README 合同口径 / CI 校对 / 旧模块图 / 空壳 crate；② `generated/` 树、`generated-lock.json`、`check-generated-clean`、`legacy_baseline.rs`、VOX-D 门与 V1.4 fixture 骨架；③ 活代码不再用 `Generated*` 类型与 `BASELINE_ID` / `SCHEMA_EPOCH` / `STABLE_ERROR_IDS`，错误 id 只剩契约 snake_case 一套，SHA-256 之类「与基线无关的水管」换成 crate 内自有实现或标准依赖。理由：错误 id 两套命名空间正是「第二份真值」；SDK 已经在把死名字往托管侧递，越晚清消费方越多 | 漂移 ②；已裁原则 ①③ |
| D9 | 15 张蓝图卡的 Workflow 回写口径 | **建议：归 Room + 评审中 → 实现中 → 验收中，评论补 origin 提交号；已完成只在 93 条验收项由 QA 实跑通过后流转**。R-00439 已在验收中不动；R-00434 实现中 → 验收中。理由：公共纪律 ⑦「做完流转验收中，已完成由总调度核验后流转」；独立复审已跑过全量测试，验收项没跑 | 漂移 ①；td-progress-audit 步骤 3 |
| D10 | 物理三槽（raycast / sweep / overlap）只声明不路由 | **建议：开一张架构仓小卡（A-5），排在 V-QA 之后**——路由到 `lumio-voxel-project::physics_query`，`root_api` 补 3 条测试，C# facade 补三个入口。替代方案「把三槽从根表删掉等以后再加」被否：根表只追加不插入，删了再加会换槽位 | 漂移 ⑤；ADR-062「C 签名属 native-abi.json」 |
| D11 | 契约三处已知缺陷（rule 49 错位 / 全量编码带 `baseSectionRevision` 无码 / pin 预算无常量）现在修还是等 | **建议：现在修，一张架构仓契约小卡 + ADR-062 修订记录**，VoxelEngine 复制副本并更新 `CONTRACT_SHA256`。理由：R-00440 验收项 2 没有机器可判的界，V-QA 会卡在这条上 | 漂移 ⑥；drift review §6 |

## 7. 本次已执行动作 / 待授权事项

- 已执行（2026-09-05，Owner 授权后逐笔读回）：
  - Workflow：新建 R-00473（RM-00002，8 条原生验收项，全部 not_started）；R-00302 / R-00308 / R-00309 流转「已否决」并各补 1 条作废评论；R-00007 5 条与 R-00083 4 条验收项改为 passed（读回 5/5、4/4）。共 1 建单 + 8 验收项 + 3 流转 + 3 评论 + 9 验收项更新。
  - NativeCore 仓：远端 `feat/r-00352-timer-manager` / `feat/r-00372-timer-abi` / `feat/r-00386-r4-07-timer-ffi-delete` 已删除；本地只剩 `main`，worktree 只剩主工作区。
  - 架构仓文档：本报告、`plans/2026-09-05-nativecore-w0-card-and-kickoff.md`（含 R-00473 开工提示词）、gas.md M9 改归 Runtime、ADR-064 修订记录；首批已由 Owner 提交（`4a5f596`），本轮补写卡号与执行记录待再提交。
- 未执行：R-00473 的实现（另开窗口按 plans 文件 §二 提示词派工）；其余七仓盘点（§2.2 ~ §2.8）。

- VoxelEngine 站（2026-09-05，本会话）：
  - 仓库侧已 `git pull --ff-only` 到 `e5c056e`；全仓门禁实跑全绿。
  - Workflow 执行（2026-09-06，Owner 授权后逐笔读回）：
    - 新建清理卡 **R-00474**（RM-00003，《[程序·工程] 退出旧合同制残留：删除 docs/architecture 镜像、CI 基线校验并重写 README 与注释》），状态 backlog。
    - 批量作废旧蓝图卡 27 张（R-00002、R-00066、R-00068、R-00070、R-00076、R-00078、R-00080、R-00096、R-00104、R-00116、R-00134、R-00136、R-00142 及 14 张 backlog）全部成功流转为 `rejected`（已否决），并附作废理由：“已由 ADR-062 与 Wave 1~4（R-00434~R-00458）新架构实现取代，按 2026-09-05 引擎总监盘点决策 D8（出路 A）作废关闭。”
    - RM-00003 读回：总数 56 张（28 done、27 rejected、1 backlog R-00474），旧需求彻底出清。
  - 产出：本报告 §2.2 / §3 / §6 D7–D11；`plans/2026-09-05-voxelengine-w0-card-and-kickoff.md`（已填入真实卡号 R-00474）。
  - 架构仓需 Owner 在终端执行：先 `git pull --rebase origin main`，再把本报告、`plans/2026-09-05-voxelengine-w0-card-and-kickoff.md` 加入暂存区提交并推送。
