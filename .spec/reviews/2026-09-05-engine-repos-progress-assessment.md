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
| LumioVoxelEngine | — | — | 待盘 |
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
| N-W0 | 退役制度残留清理（**新建**） | 小卡，纯本仓 | 删 `docs/architecture/` 镜像与 `.baseline.sha256`；删 CI `readme` job 基线断言；`lumio-contract-types` 去掉 Root ABI bundle golden / 漂移门，改锚 `native-abi.json` 或只保留本仓内部类型；删 `lumio-native-ffi` 退役 provider 表（保留 panic 边界 / 句柄校验若仍有用，否则整 crate 删）；README、`.spec` 与模块 README 去掉 `LGE-V1.4` / `LumioGameEngineArchitecture` / CoreEngine 口径；timer 三处文档统一 | 无；D1 裁决 |
| N-GAS | R-00302 / R-00308 / R-00309 | 既有 3 张 | **挂起**并补评论说明原因（ADR-064 阶段 2）；重启前先裁 D2，再重写卡面并修编码 | D2 裁决 |
| N-M5 | 空间粗筛内核契约卡 + 实现卡（**新建，暂不派**） | 契约先行 | 契约：`(viewer, target, enter|leave)` 有序清单、排序键、双半径、确定性义务、在第 6 相收回、根表槽位形状；实现：`lumio-spatial` 改形 + `lumio-job` 运载 | DS 视野排期；ECS 视野表真值就位 |
| N-ACC | R-00007 / R-00083 验收项补记 | 写操作 | 9 条 `not_started` → 按 PR #3 / #4 证据补跑或补记 | 写授权 |
| — | 分支清理 | 卫生 | 删 5 本地 + 3 远端已合入分支 | 确认 |

不开的卡及原因：Production Hardening（§6 推到正式硬化）；wasm32 目标（等 D3）；codec / diagnostics 转正（等架构源批准公共语义）。

### 2.2 LumioVoxelEngine（待盘）

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
| RM-00014 | 九项必备能力 | 14 | 0 | 0 | 0 | 14 | 无 NativeCore 卡 |

## 4. 漂移对照与证据核验结果

见 §2.1「漂移与问题」①–⑤。本轮**未做任何 Workflow 写入**。

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

## 7. 本次已执行动作 / 待授权事项

- 已执行：NativeCore 全量事实采集（git / gh / cargo 实跑）；RM-00002 全量只读拉取与 68 张 done 卡的机器复核；合并 Codex 会话的两份文档并删除原文件；本文 §2.1 落盘。
- 未执行：任何 Workflow 写入（验收项补记、挂起评论、新卡）；任何分支删除；开工提示词（等 D1 / D2 裁决后按 `td-progress-audit` 第 5 步成文，沿用 Codex 版的守门优先与公共纪律格式）。
- 待授权：① N-W0 新建卡；② R-00302 / 308 / 309 挂起评论；③ R-00007 / R-00083 验收项补记；④ 删除已合入的 5 本地 + 3 远端分支。
