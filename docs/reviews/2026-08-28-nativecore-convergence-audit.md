# 2026-08-28 · NativeCore 收敛盘点(TD 进度盘点 · 第三轮)

> 盘点会话:架构仓 TD 会话(macOS,`~/LumioGames`)。基线 `LGE-V1.4-2026-08-27` 未变。
> 范围:全局八室状态速览 + **LumioNativeCore 深盘与收尾编排**(用户指令:自底向上收敛,先收 NativeCore,派单并行干活)。
> 上一轮:[`2026-08-28-seven-repo-progress-assessment.md`](2026-08-28-seven-repo-progress-assessment.md)、[`2026-08-28-gate-p0-delivery-and-escalations.md`](2026-08-28-gate-p0-delivery-and-escalations.md)。

## 1 · 执行摘要

**全局完成度总览**(数据源:Workflow `rooms/{id}/overview`,2026-08-28 实拉;连接三方一致 `lumiogamesengine`):

| Room | 仓 | 需求卡 | backlog | 进行中桶 | completed | 验收项未过 | 未解缺陷 |
|---|---|--:|--:|--:|--:|--:|--:|
| RM-00001 | Architecture | 8 | 2 | 0 | 6 | 10 | 0 |
| RM-00002 | **NativeCore** | **68** | **1** | **67** | **0** | **289** | 0 |
| RM-00003 | VoxelEngine | 53 | 14 | 39 | 0 | 212 | 0 |
| RM-00004 | CoreEngine | 38 | 34 | 0 | 4 | 149 | 3 |
| RM-00005 | GameRuntime | 31 | 26 | 5 | 0 | 246 | 0 |
| RM-00006 | Server | 54 | 48 | 6 | 0 | 264 | 0 |
| RM-00007 | Client | 10 | 4 | 6 | 0 | 48 | 5(工作项) |
| RM-00008 | Game | 1 | 0 | 1 | 0 | 0 | 0 |

**核心结论(3 条):**

1. **NativeCore 代码面已达仓库门槛全绿,单据面整体落后于代码。** HEAD `c180bdd`(= origin/main)本机实跑:`cargo test --workspace` 90 通过 0 失败(82 个 suite)、`cargo clippy --workspace --all-targets -- -D warnings` exit 0、`cargo build --workspace --benches` exit 0。而 Workflow 侧 67 张执行卡全部停在「验收中」,289 条验收项全部 `not_started`,0 张 completed——属**仓库领先型漂移**,收尾主体是验收核销,不是补代码。
2. **唯一实质功能缺口是契约绑定。** 上游架构仓已发布 Root ABI Bundle(`packages/abi/lumio_core.h` + `root-abi-bundle.json`,`packages/index.json` 的 `rootAbi.consumers` 登记了 LumioNativeCore),`ids/index.json` 已发布 ErrorCode(43 值)与 Capability(9 值)数值注册表(均 Architecture 所有)。但本仓 `crates/lumio-contract-types/src/generated.rs` 自述 "**Binding is not done yet**"(`c180bdd` 只收敛了注释),五个 newtype 仍是 `_private: ()` 空壳,FFI `exports.rs` 仍禁 `#[no_mangle]`。R-00007 冻结的 BLOCKED_ABI 前提对 8 张卡已(部分)解除,按其变更控制条款走「蓝图修订评论 + 重开受波及卡」。
3. **收尾编排为一波三道并行**(文件集与卡集互不重叠,各自隔离 worktree):A 契约绑定(代码,8 卡)∥ B 验收判定批次一(28 卡,纯 QA)∥ C 验收判定批次二(31 卡,纯 QA);随后总调度核验、批量 acceptance→done 流转与 R-00007 收尾(NC-W2)。

## 2 · NativeCore 仓详情(每条结论附出处)

### 2.1 代码面

- 仓库结构:9 crate + xtask,136 个 `.rs`,源码 8,876 行;各 crate 源/测试行数——kernel 1379/1409、job 585/799、spatial 413/651、native-ffi 520/284、diagnostics 302/452、codec 277/260、test-support 227/121、contract-types 161/120、platform 101/71(`find`/`wc` 实测)。
- `main` = `origin/main` = `c180bdd`,0 ahead / 0 behind;工作区仅 2 条未跟踪 `.DS_Store`(仓务瑕疵,见 §6)。
- 门槛实跑(本机 `x86_64-apple-darwin` via Rosetta,rustc 1.94.0):
  - `cargo test --workspace` → **90 passed / 0 failed**(82 suites);
  - `cargo clippy --workspace --all-targets -- -D warnings` → **exit 0**;
  - `cargo build --workspace --benches` → **exit 0**。
  即 R-00261 声称的 CI native job 修复(`03d6bd7`,PR #1)在 HEAD 复验成立。
- 契约绑定缺口出处:`crates/lumio-contract-types/src/generated.rs:9`("Binding is not done yet, so this module is still the internal seam only");`AbiVersion`/`ArchitectureErrorCode`/`ArchitectureOperationId`/`CapabilityBits`/`StructSize` 均为 `_private: ()`;`crates/lumio-native-ffi/src/exports.rs:10`("Do not add `#[no_mangle]` or `extern \"C\"` names here")。

### 2.2 上游可消费面(架构仓 origin/main `d812617` 实查)

- `packages/abi/lumio_core.h` + `packages/abi/root-abi-bundle.json` 在库(R-00003 交付,`44f617b` 起);bundle 顶层含 `abi`(entrySymbol `lumio_core_get_api_v1`、symbolPrefix `lumio_`、little-endian、ptr64)、`layoutProfile`(`linux-x86_64-glibc`,root/table header 16B)、`tables`、`typeMapping`、`compiler.digest`、`inputHash`。
- `packages/index.json` 的 `rootAbi.consumers = [LumioCoreEngine, LumioNativeCore]`(`b8f8c50` 登记)——本仓是 Root ABI 的登记消费方,且**刻意不是** Rust/C# generated packages 的消费方(`generated.rs` 注释口径)。
- `ids/index.json`(baselineId `LGE-V1.4-2026-08-27`):namespace MessageType(8,GameRuntime)、**ErrorCode(43,Architecture)**、**Capability(9,Architecture)**、FaultClass(3,GameRuntime)。
- SHA-256 K[28] 已修复入 main(`bcc8eb9`),D-8 normalization 已数据化(`7bdad78`)——上游对 digest 核对的禁令已解除(出处:`2026-08-28-gate-p0-delivery-and-escalations.md` 状态更新块)。

### 2.3 单据面

- RM-00002 共 68 卡:R-00007(原始需求/蓝图,backlog)+ 66 张执行卡 + R-00261(CI 修复),67 张全在 `acceptance`;`GET /requirements?roomId=` cursor 取全量核对,与 overview 计数一致。
- 每卡已有 2–3 条评论:交付评论(锚 `e3c382c`/`801a3a5` 等)+ 2026-08-28「状态欠账补平」对账评论(统一锚 `origin/main 0e18106`,验证宿主与工作区状态齐全);R-00261 证据锚 `03d6bd7`(PR #1,mergedAt 2026-08-28T11:54:47Z)。抽样核验 R-00087/R-00102/R-00261 评论,锚点均可在 origin 找到(HEAD `c180bdd` 含其祖先)。
- **289 条验收项 `systemSemantic: not_started`**(68 卡逐卡拉 `acceptance-items` 汇总)——验收判定从未在系统内执行过,是纯单据欠账。

## 3 · Workflow 现状

见 §1 总览表。全项目 263 张需求卡,里程碑 MS-00001(MVP,目标 2026-10-31)。NativeCore 之外的显著事实:CoreEngine 34 张 backlog 等待 D-2/D-10 裁决(architecture lock 升级)后解锁;Architecture 室 2 张 backlog 卡(R-00257 前提不成立、R-00009 需基线跃迁)已在上一轮记为 blocked(`d812617`)。本轮不动其他室。

## 4 · 漂移对照与证据核验

| 方向 | 判定 | 说明 |
|---|---|---|
| 仓库领先 | **是(主要形态)** | 代码/证据齐备,验收项未判定、卡未 completed。处置:派 QA 核销(B/C 道),总调度核验后流转 |
| Workflow 领先 | 否 | 67 张验收中卡的证据锚点(`0e18106`、`03d6bd7`、各交付提交)均已核实在 origin/main 祖先链上;无不可复核卡,无解铃条件挂账 |

本轮对 Workflow **零写入**(纯读盘点);写入全部交由派出的执行会话按公共纪律进行(既有卡评论/流转/验收项判定,在 cross-repo-delivery 持续授权内;**不建新卡/新 Room/新里程碑**)。

## 5 · 关键路径与下一阶段编排(NC-W1 → NC-W2)

**NC-W1(本轮派出,三道并行;完成前不进 NC-W2):**

| 道 | 性质 | 卡集 | 文件集 |
|---|---|---|---|
| A 契约绑定 | 代码 | R-00056、R-00069、R-00072、R-00074、R-00079、R-00083、R-00179、R-00180(+R-00007 蓝图修订评论) | `crates/lumio-contract-types`、`crates/lumio-native-ffi`、(若解锁)kernel 错误映射/capability 相关文件、xtask 守护 |
| B 验收判定批次一 | QA(不改代码) | R-00010、R-00075、R-00077、R-00082、R-00084…R-00102、R-00129、R-00130、R-00132、R-00160、R-00161、R-00165、R-00168(28 卡) | 无(只读 + 隔离 worktree 实跑) |
| C 验收判定批次二 | QA(不改代码) | R-00103…R-00185(job/spatial/codec/diagnostics/test-support 系,31 卡)+ R-00261 | 无(同上) |

三道卡集互不重叠;A 独占其 8 卡的全部 Workflow 写入。派活提示词全文见 [`../plans/2026-08-28-nativecore-convergence-dispatch.md`](../plans/2026-08-28-nativecore-convergence-dispatch.md)。

**NC-W2(A/B/C 全部回报后,总调度执行):** 抽样复核 QA 判定 → 批量 acceptance→done 流转(逐卡 reason)→ R-00007 按其变更控制条款收尾(全部执行需求 + 全仓负向 Gate + 整体审查有证据后)→ 视 A 道上报决定是否需要架构仓侧新卡(届时另行请授权)。

## 6 · 风险与开放决策

1. **三平台 smoke 无从实跑**:本机只有 macOS(且 rustup 默认 x86_64/Rosetta,另有 aarch64 腿);Windows/Linux 侧证据缺宿主。QA 道纪律:验不了的项**不判通过**,留未判定 + 缺口评论。建议裁决:三平台项以 CI(GitHub Actions linux runner)+ 后续 Windows 机实跑补齐,不阻塞其余项核销。
2. **ids/ 数值覆盖度未逐项核对**:ErrorCode 43 值是否覆盖 NativeCore 内部类别(InvalidArgument/BufferTooSmall/CapacityExceeded 等)的映射需要,由 A 道逐 namespace 核对;不覆盖部分标 BLOCKED 上报,**不得本地发明数值**(可能演化为对架构仓的注册表增补需求 → 届时需用户授权立卡)。
3. **OperationId namespace 未发布**(ids/ 只有 MessageType/ErrorCode/Capability/FaultClass):`ArchitectureOperationId` 绑定预计仍 BLOCKED,A 道如实上报,属预期内残余。
4. **并发会话共用工作区**:三道并行 + 本会话同仓,已在提示词硬性要求各开隔离 git worktree(教训见 `lessons`/记忆)。
5. **仓务瑕疵**:NativeCore 工作区 2 条未跟踪 `.DS_Store`(`.gitignore` 未覆盖)——不派卡,留给 A 道顺路判断是否值得单独一笔 chore(不得夹带进契约绑定提交)。
6. **本报告分支未推送**:见 §7 待授权。

## 7 · 本次已执行动作 / 待授权事项

**已执行(全部只读或本地):**
- 八仓 fetch + 状态采集(未 pull,共享工作区不动);NativeCore 门槛三命令实跑(§2.1 输出);
- Workflow 连接验证(`/me` + `/projects/current` + profile 三方一致)、8 室 overview、RM-00002 全量 68 卡 + 289 验收项 + 评论逐卡拉取(cursor 至空);
- 本报告与派活提示词落盘于隔离 worktree 分支 `docs/2026-08-28-nativecore-convergence`;
- `spawn_task` 派出 A/B/C 三个 NativeCore 会话(cwd 指向 `~/LumioGames/LumioNativeCore`)。

**待授权 / 待用户裁决:**
- 本分支 push 到 origin 并开 PR(对外发布动作,未获持续授权);
- §6.1 三平台 smoke 的补齐路径;
- 若 A 道上报注册表缺口 → 架构仓侧新卡授权。

**豁免声明:** 本提交为纯文档(评估报告 + 派活提示词),按快速模式白名单收口(收口门槛四命令实跑通过,见提交信息),不派 reviewer。
