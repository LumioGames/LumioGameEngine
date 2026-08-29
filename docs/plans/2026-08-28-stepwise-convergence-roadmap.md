# 2026-08-28 · 分步收敛路线图(后 NativeCore 阶段,r2)

> 来源盘点:[`../reviews/2026-08-28-nativecore-convergence-audit.md`](../reviews/2026-08-28-nativecore-convergence-audit.md);
> 裁决底账:[`../reviews/2026-08-28-gate-p0-delivery-and-escalations.md`](../reviews/2026-08-28-gate-p0-delivery-and-escalations.md)(D-1…D-12);
> MVP 锚点:MS-00001(目标 2026-10-31),两轨四阶段见 [`mvp-browser-voxel-multiplayer.md`](mvp-browser-voxel-multiplayer.md)。
> 用户指令口径:**一小步一小步收敛,不要全部收敛**——本路线图即该指令的执行形。
> r2(2026-08-29):经三视角对抗审查(30 条发现,8 条 P1)修订——S2 执行仓更正为 CoreEngine、补 A2 浏览器接入阶段、拆散 S2/S4/S7 隐性大步、接住 D-7 独立审查欠账、完成条件统一为三证模板。

## 0 · 原则(每步都适用)

1. **一步只收一件事**:一个仓、或一个裁决簇;上一步完成条件未过,不进下一步(W0「不完成不进下一 wave」纪律的推广)。旁路步(编号 Sx 前缀 R/V/A)才允许与主线并行,且必须逐条声明与主线无文件重叠、无裁决依赖、无卡集重叠。
2. **完成条件三证模板**:每步收口必须同时给 ① origin 提交号(逐项列明)② 门禁/命令真实输出 ③ Workflow 状态读回清单。本文各步所列条件若有缺项,以开步派活卡补齐为准——缺三证之一不得宣称收敛完成。
3. **裁决与执行解耦**:裁决(D-*)是用户动作,不占执行带宽——挡最长链的裁决应最早给;执行严格小步。
4. **滚动计划**:S5 及之后为**粗排**,卡号与计数是 2026-08-28 快照,开步时必须以当时的 Workflow 读回与仓库状态重核,不得照抄本文;每步收口跑一次轻量 td 对账。
5. **QA 独立**:验收判定复用 NC-B/C 双道模板,写的人 ≠ 判的人。

## 1 · 主线步骤

### S0 · NC-W2:NativeCore 收口(触发:NC-A/B/C 三道回报齐)

- 内容:抽样复核两道 QA 判定 → 批量 acceptance→done 流转(逐卡 reason)→ R-00007 按其变更控制条款收尾 → 产出**收口报告** `docs/reviews/2026-08-29-nativecore-closeout.md`(含:RM-00002 逐卡状态读回清单;上游缺口清单——已知预期 OperationId namespace 未发布,可能新增 ids/ 覆盖缺口)。
- 完成条件(三证):① NativeCore `origin/main` HEAD 提交号 + 收口报告所在提交号;② 门槛三命令真实输出全绿——`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo build --workspace --benches`;③ RM-00002 逐卡读回:全部 done 或明确 blocked(带解铃条件)。
- 需要用户:若 A 道上报需架构仓立新卡(注册表增补),逐次授权。
- 回退口径:QA 发现「验收口径与代码事实不符」→ 该卡不流转,进收口报告差异清单,由本步裁决(退回重做 or 修订口径)。

### S1 · VoxelEngine 解阻测量(一件事:把 D-12「从未测量」变成「测过了」)

- 为什么是它:依赖图紧贴 NativeCore 的 Rust 层;心结不是 53 张卡,而是 **D-12——决策门测量从未链接执行**(VOX-D-005..008 blocked,evidence §4 是 plan 不是 results),连带架构室 R-00257 前提不成立。
- 内容(刻意收窄,53 卡核销**不在本步**,见旁路 SV):macOS 宿主真实链接跑 decision-gate 测量缝 → 四份 evidence §4 从 plan 改 results → 推 origin → R-00257 卡面「已完成测量」一句修正。
- 完成条件(三证,判别式针对「plan 冒充 results」病灶):① 四份 evidence 逐份 origin 提交号 + R-00257 卡面修订提交号;② 四份 §4 各含 host triple + 数值表 + 生成命令与运行日志引用(可复现);③ R-00257 读回显示测量证据链接,VOX-D-005..008 的 blockedReason 已更新。
- 派活形态:单会话一单(cwd=LumioVoxelEngine)。裁决本身(逐门按 D-013 模式)归用户,在数字出来后进行。

### S2 · 解锁 CoreEngine(单簇:D-2 卡①② + D-10 卡面修订,三件一起)

- **前置(D-7 欠账,二选一,须你裁决)**:三张 P0 Gate 卡(R-00003/R-00004/R-00005)的交付是同上下文自审、PR 无人 review(gate 文档 §四);S2 会把这些交付物字节级冻进 CoreEngine 只读镜像并钉进 lock。开步前要么 ① CODEOWNERS `@Go1c` 补独立审查,要么 ② 你显式裁决「接受不审直接 pin」并记录——若 pin 后才发现缺陷,重新登记 280+ 条路径的 churn 自负。
- 内容:落 D-2 两张卡草稿——卡①升 `architecture.lock` 到 V1.4 pin(≥ `c712ff4`,以开步时重核为准)、卡②按 `packages/index.json` consumers 关系收敛纳入 `packages/`(**①→② 严格串行**,两卡都动 `sync-architecture.sh` 与 `compilerSha256`);**同批**修订 R-00015 / R-00012 卡面的钉死基线句(D-10:漏了这件,前两件白做)。
- **执行形态(r2 更正)**:D-2 卡①② 的文件集在 **LumioCoreEngine 仓**(`tools/sync-architecture.sh` 只存在于该仓)——执行会话 cwd=LumioCoreEngine;架构仓侧只有 Workflow 动作(立卡、卡面修订)与可能的 consumers 数据核对。「与 S1 并行」仍成立(异仓)。
- 需要用户:**立卡授权**(D-2 两张是新卡)+ 卡②时机子决定(按 D-2 排序约束 4 原口径:**现在冻 Draft、接受 V1.5 转 Accepted 时一次重登记**,还是**等 V1.5 再做卡②**——不存在「先转 Accepted」的第三选项,那等价于提前跳基线,归 V1.5 批)+ tools/** 是否进契约镜像的口径(D-5 子项,在卡②定投影规则时一并裁最省)。
- 完成条件(三证):① 卡①②交付提交号(CoreEngine origin);② CoreEngine `sync-architecture.sh` / check-contracts 门禁在新 pin 上的真实输出全绿;③ R-00015/R-00012 卡面读回自洽、两张新卡状态读回。

### S3 · CoreEngine 消费链路打通(只收 R-00015 + 既有 3 个 open bugs)

- 内容:R-00015 消费 runtime crate 落地(逐字节消费上游制品)+ 清 RM-00004 现有 3 个未解缺陷(**开步时枚举单号回填派活卡**;执行期间新报 bug 不计入本步,进差异清单)。**34 张 backlog 不动**——链路通了才谈铺量,铺量另起一步且以 MVP 关键路径优先级裁剪。
- 完成条件(三证):① R-00015 交付提交号(origin);② check-contracts 门禁真实输出绿;③ R-00015 done + 三个 bug 单号逐一读回 closed(经 QA 判定)。

### S4 · D-1 裁决与落地(单簇:MVP 最长链的钥匙)

- 内容:**D-1 下行状态载荷编码 + 上行输入承载 ADR**(裁决讨论可提前至任意时点,见 §2)→ ADR + Schema + 正反 Fixture + 七仓镜像同步。仅此一件;D-9/D-3 挪到 S5 开步裁决,D-4 挪到 S7。附加项:**ids/ 注册表增补**若 S0 缺口清单要求(~~OperationId namespace~~ **已于 2026-08-29 裁决为不适用/终态,不再列为缺口**;其余项若有),作为本步的第二张卡一并落(新卡需你授权)——它与 D-1 同为「架构仓公共面增发」,同一执行会话、不同卡。
- 完成条件(三证):① ADR/Schema/Fixture 提交号;② `python3 tools/lumio_contract.py validate` 全绿 + **复现 `.github/workflows/repository-policy.yml` 的 Hash/文件检查**(AGENTS.md 收口门槛对涉基线改动的硬性要求)真实输出;③ 相关卡状态读回,Server A1-β 与 GameRuntime 受阻卡的 BLOCKED 评论已更新引用新 ADR。

### S5 · GameRuntime 收敛(粗排;开步裁决 + 两小批)

- **S5-0 开步裁决(你)**:D-9(二进制 canonical:补并列 profile,还是 ADR-010:20 改引用、域自定)+ D-3(generated 面 catalog-only 与否、ID ordinal 权威来源)。**条件性前置**:若 D-9 裁为域自定,R-00141 系须先补「Runtime 持久化域 primitive 编码」域级 ADR 才能开工(gate 文档 D-9 原文)。
- **S5a**:被 D-3/D-9 解锁的卡先行(2026-08-28 快照:R-00138/139/141/149/150 系 8 条验收项;开步重核)。**注意:S5a 主要是 config/persistence/ecs 域,不含 replication 实现**(七仓评估 §5:replication 七卡在 W2)。
- **S5b**:replication 系与其余 backlog 按 wave 分批,**A1-β 所需的 replication 子集优先**。
- 完成条件:按三证模板开步时定,逐批收口。

### S6 · Server A1-β 合龙(前置:S4 落地 + S5b 的 replication 子集)

- **r2 更正**:A1-β(第二个客户端看见方块被挖)的 Runtime 前置**不止 S5a**——复制链路(ReplicationProjection→delta→客户端应用)在 S5b;开步时以卡号显式核对前置集。A1-α 不在本步(见旁路 SA,随时可做)。
- 完成条件(三证):① Server/Client 侧交付提交号;② 双客户端端到端实测记录(操作序列 + 双端状态一致性断言输出,非「演示成功」一句话);③ 相关卡读回。

### S7 · Client 解锁与收敛(D-4 + 10 卡 + 5 缺陷)

- 开步裁决(你):D-4(netstandard2.1 多目标 or Unity 发布形态)。内容:包引用切换 + RM-00007 的 10 卡收敛 + **5 条未解缺陷(工作项)**(r2 补;开步枚举单号)。
- 完成条件:三证模板,开步时定。

### S8 · A2 浏览器接入(r2 补——MVP 计划 §4 轨道 A 的整段阶段,此前漏排)

- 内容(粗排,出处 MVP 计划 §4):.NET WASM 宿主壳、客户端 Assembly WASM 面收窄、WebGL Presentation Adapter、登录页;退出条件「两个**浏览器**互见挖方块」。落卡归属(Client 10 卡是否已覆盖)在 S7 收口时核对,不足则请授权补卡。
- 完成条件:三证 + 双浏览器实测记录。

### S9 · Game 内容(粗排)

- 前置裁决(你):美术三方向比稿(出处:LumioGame `origin/main` `9bc46ed`「美术风格推翻归零进入三方向比稿」)。内容卡铺开按比稿结果,开步时以 RM-00008 读回为准。

### S10 · A3 整体收口(MVP 验收)

- 按 MVP 计划 §6 验收清单逐条实测(≥5 浏览器客户端互见等);整体 reviewer 收口审查;MS-00001 读回。

## 旁路步(与主线并行,逐条声明无冲突)

| 旁路 | 内容 | 起跑条件 | 与主线的隔离声明 |
|---|---|---|---|
| **SA · Server A1-α** | WSS 握手→admission→FullSnapshot→BaselineAck→revision→DeltaAck→断连重连 Full Resync(gate 文档 D-1:只依赖已冻结面,可交付) | **随时**(r2 更正:不依赖 S2,更不依赖 S4;唯一考量是执行带宽) | 仓=LumioServer,与 S0–S3 无文件/卡集交集;完成条件:三证 + 链接执行的序列测试输出 |
| **SV · VoxelEngine 53 卡验收核销** | 复用 NC-B/C QA 模板拆两道 | **不早于 S1 收口**(r2 更正:53 卡含 R-00057..R-00064,与 S1 改写中的 evidence 卡集重叠,同时跑会对着移动靶判卡) | S1 收口后无重叠;不改代码 |
| **SW · Windows 机三平台 smoke 证据** | 补 NativeCore smoke 的 Windows 腿 | S0 之后;**前置:先装 MSVC Build Tools(link.exe)并留证**(r2 补:该机已知缺 link.exe,cargo check 不算证据,否则复现 Voxel 同款验证债) | 异机异仓,无交集 |

## 2 · 裁决队列(用户视角,按挡链长短排序)

| 裁决 | 挡什么 | 建议给出时机 |
|---|---|---|
| **D-1** 状态载荷/输入承载 ADR | MVP 验收 1(最长链:S4→S5→S6→S8→S10) | **现在就可开始讨论**,S4 前定稿 |
| **D-7 残余**:三张 Gate 卡独立审查(补审 or 显式豁免) | S2 开步(pin 之前必须了结) | S0 收口前后 |
| D-2 立卡授权 + 卡②时机(冻 Draft or 等 V1.5)+ tools/** 口径 | CoreEngine 全部(S2→S3) | S0 收口后立即 |
| D-12→R-00257 Voxel 决策门逐门裁决 | Voxel P2 方向 | S1 数字出来后 |
| D-9 + D-3(GameRuntime 簇) | S5(R-00141 系可能还需域级 ADR) | S4 期间 |
| D-4(Client 簇) | S7/S8 | S5 期间 |
| **B 轨汇合与否**(r2 补):9 周内是否追求 B1/B2 汇合;默认建议按 MVP 计划 §4 既有兜底——ReferencePort 权威出演示,B1/B2 顺延 MVP+1(MVP 验收 7 相应不达成) | A3 验收口径 | S3 前后 |
| D-5 其余(tag 发布节奏、compilerHash 拆分)+ D-6 其余 + D-11 | 下游 pin 稳定性 | **并入一次 V1.5 规划批**,不单独跳基线(D-6 在 S2 只出「卡②时机」一个子决定,不在 S2 转 Accepted) |
| 美术三方向比稿 | S9 | S7 期间 |

## 3 · 风险与时间盒

- MS-00001 目标 2026-10-31,余 ~9 周。**量化提示(r2 补)**:主线 S0–S10 十一步默认串行,其上还挂着未排期的铺量批次(CoreEngine 34 卡、GameRuntime 26 卡、Voxel 53 卡核销、V1.5 批、Game 内容、A2 落卡)——全部做完 9 周**不够**。口径:关键路径(D-1→S5→S6→S8→S9→S10)优先,铺量批次按「MVP 需要才做」裁剪,裁剪决定权在你,每次开步对账时确认。
- 关键路径不在 NativeCore/Voxel——S1/S3 保持小步不铺量,带宽留给 S4 之后。
- 本路线图与盘点报告所在分支 `docs/2026-08-28-nativecore-convergence` **尚未推送**(待你授权 push/PR)——证据锚定纪律要求引用已推送提交,授权前派活提示词一律自足、不引用本分支路径(现状已如此)。
- 多会话并发纪律不变:隔离 worktree、证据锚 origin、spawn_task 派活、SendMessage 回报。

## 4 · 与既有编排的衔接

- S0 操作细则:[`2026-08-28-nativecore-convergence-dispatch.md`](2026-08-28-nativecore-convergence-dispatch.md)「收口(NC-W2)」节;S0 收口报告落 `docs/reviews/2026-08-29-nativecore-closeout.md`。
- S1/S2 及之后的派活提示词开步时按 dispatch.md 模板现写(指路/立规/设禁区),不预写——防旧快照;S5 起的卡号/计数一律开步重核(原则 4)。
- 全局 wave 底图(W0–W3)见 `2026-08-28-seven-repo-progress-assessment.md` §5;本路线图是其 NativeCore 收口后的细化与顺延,冲突时以本文为准并回写该文。
- 对抗审查全文(30 条发现)存于会话工作流记录;P1 已全部吸收,P2 择要吸收,未吸收项:无。
