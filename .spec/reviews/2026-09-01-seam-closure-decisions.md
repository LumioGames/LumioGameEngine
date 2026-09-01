---
name: 2026-09-01-seam-closure-decisions
description: 文档重构与 19 处接缝补齐的逐条裁决流水;追某条接缝为何这样定时查
metadata:
  type: doc
  status: 已交付
---

# 2026-09-01 · 文档重构与接缝补齐 · 裁决流水

> 实时落盘，防对话中断。产出的设计概要见 `../knowledge/features/ecs.md` / `../knowledge/features/ds-server.md` / `../knowledge/features/gas.md` / `../knowledge/features/save-load.md`。
> 本轮起因：四份「定稿 + 流水」双文件的正文掺满「怎么吵出来的」，且已被 2026-09-01 的 RM-00011 裁决部分改写而未更新。目标形态照 `../knowledge/features/config-table.md`。

## 会议规则

- **正文只回答五个问题**：这是什么 / 长什么样 / 每块干什么 / 按什么顺序做 / 什么不许做。历史（裁决号、过账表、UE 对照、反向意见）一律迁出到流水或审计附录。
- **每个模块四段式**：干什么 / 能干什么 / 不干什么 / **做完的标准**。写不出验收标准的地方 = 设计没定，就地开板裁决。
- **裁决方式**：主 loop 给带理由和游戏例子的推荐，Owner 放行或改指。拍完立即追加本表。

## 裁决流水

| # | 议题 | 裁决 | 理由 | 落点 |
|---|---|---|---|---|
| 0a | 交付链之争（2026-09-01 两条裁决打架） | **Living Architecture 算数**。接口真值 = `engine/abi/native-abi.json` + `engine/wire/*.json`；开发态不跑 Baseline / Schema / Fixture / 七仓镜像 / 发布门 | `59866ec`（09:39）已实际删除整套 baseline 机器（57 schema + fixtures + ID registry + 生成物 + python 门禁，415 文件 / 30932 行），理由写明主线已切换为预上线 Living Architecture；ADR-052 同步记「ADR-049 的 V1.5 基线化路线未被采纳执行」。较新且已执行的一条胜出 | 四份设计概要的「TODO / 阶段 0」一律按 Living Architecture 写；**待修项**见下表 W-1 |
| 0b | 两份定稿被 RM-00011 改写的五处怎么处理 | **直接写成新现状**。新正文只写今天的真值，旧裁决归流水附录，正文不提「原本怎么定的」 | 文档要当规范用，不能让读者在正文里做考古 | 五处逐条见「RM-00011 并入清单」 |
| 1 | ECS 一帧四步 → 13 相映射（原定稿 §5.1 自留的空白） | **记单 = `ApplyInputs`/`ProcessorPlan`；结算 + 亮相 + 全体 Awake + 全体 Start 全部在 `EcsCommandBufferCommit` 相内跑完；发货可见点 = `GasAndEventFinalize`；取样打包 = `ReplicationProjection`** | 13 相里**只有 `EcsCommandBufferCommit` 的可写域是 `GameWorld`**（`LumioGameEngine_Architecture_v1.4.md` §4.5 权威摘要表）。而 `Awake` 明写要初始化 Attribute 初值 = 写 `GameWorld`，所以它跑不到别的相去。原定稿只写了「落 ADR 时对照定名」，实现仓看不到这个约束，会自然把 Awake/Start 挂到提交点之后——写完才发现违约 | ECS 设计概要 M3 ①；阶段 0 卡 0-1 |
| 1b | 钩子禁令补两条 | **①钩子不许写 `GasEvents` 域**（那是提交点的可写域，跨相即违约）；**②钩子不许做不可回滚动作**（发消息/写文件/播表现记 outbox，帧成功收尾后统一执行） | ①是板 1 的直接推论；②原定稿 §5.3 有，重构清点时发现差点丢 | ECS 设计概要 M3「不干什么」 |
| 6 | 状态哈希频率（ECS 保留意见① vs DS「漂移一等告警」的唯一真矛盾） | **拆成两轨**：①每帧轻量哈希（只覆盖 `LogicTransform` + `Attribute` 当前值，SoA 连续内存），在 `SnapshotHashMetrics` 相跑，作漂移告警信号源；②按需全量快照哈希，走恢复/排查路径做定位 | 保留意见①担心的是「全量哈希每帧」的成本，DS 要的是「有没有漂移」的信号——两件事被混成了一件。告警要的是「有没有歪」，定位要的是「歪在哪」，拆开后两边都成立 | ECS 设计概要 M10 ④ |
| 7 | 「进视野排队」归 ECS 还是 DS（两边 ADR 候选撞车） | **语义归 ECS，预算归 DS**。ECS 定队列的键与顺序语义（重要度类别由内容层声明、饥饿上限的存在性）；DS 定预算、曲线、与另两处回流队列的统一纪律 | 同一个队列被两张候选 ADR 各覆盖一半，边界没写死就会两边都写或两边都不写 | ECS 设计概要 M6 ⑧；DS 侧待写 |
| 4 | 空间粗筛清单的接缝形状 | **`(viewer, target, enter\|leave)` 三元组的有序数组**；序按 `(viewer 创建序, target 创建序)` 排死；在 `NativeJobBarrier` 相收回 | 粗筛下沉 Native 是既有分工，但清单形状与确定性判据此前零定义。排序键取创建序是承 ECS「顺序稳定」义务，不新造第二套排序规则 | ECS 设计概要 M6 ③；阶段 0 卡 0-5 |
| E-1 | ECS 新增模块 M9（实体绑定与属性查询面） | **立为独立模块**：连接↔实体绑定（`AccountId + RoomId + NetEntityId + EntityType + 连接代次`）+ 受控 `AttributeId` 查询面，四种明确结果（存在/可见/权限/过期） | RM-00011 决策日志明写这是「shared runtime capability, not Chat-specific logic」。它既不属生命周期也不属同步，原两份定稿都没有它的位置 | ECS 设计概要 M9；阶段 0 卡 0-8 |
| D-1 | DS 分层按什么切 | **按层切，不按语言切**：「连接与字节」层 / 「语义与真值」层。公共契约与宿主无关；当前切片两层都在 C# 宿主，Rust 宿主并行起建后接替下层 | 原裁决 7 写的是「Rust = 连接与字节，C# = 语义」，把层和语言绑死了。RM-00011 裁定 C# 宿主先行，若照原文写，文档当场自相矛盾。按层切之后两种排布都成立，且 Rust 宿主接替时契约零改动 | DS 设计概要 §2「两层，不是两种语言」 |
| 5 | 每帧预算余量接缝（原图画了、底下零定义） | **单位 = 字节**（token 是桶的内部实现，不上接缝）；**语义 = 本帧配额，不是瞬时余量**；`ProcessorPlan` 相前置读一次并锁定，`ReplicationProjection` 相消费 | 若是瞬时余量，打包过程中数值会变，**同一帧同一状态可能打出不同的包**——确定性义务当场作废。这条接缝原定稿吹成「UE 缺失我们补上」，却是全图定义最少的一条 | DS 设计概要 M4 ②；阶段 0 卡 0-4 |
| 8 | 慢客户端阶梯的切换信号（原定稿只说「数值归实现仓」） | **统一用未确认 Delta 的 revision 落后深度**。不用 RTT、不用队列字节数 | 第三级（超变更集历史窗 → Full Resync）本来就是这个量；前两级复用它就不必造第二套仪表，且双端都算得出。「用什么信号」是语义不是数值，不写死两个仓会造出两套不兼容的阶梯 | DS 设计概要 M7 ⑤；阶段 0 卡 0-6 |
| 9 | 握手凭据契约（原触发条件写错位置） | **握手携带 Account Server 签发的不透明凭据 + 反重放窗口语义**；凭据的签发、轮换、密钥管理归 Account Server，**不进 wire 契约** | 原定稿把凭据线格式的解冻触发写成「对外网测/安全审计前」，但验收出口第一句就是「浏览器登录」——MVP 第一个动作无契约可依。RM-00011 已裁 Account Server 出「签名的、有期限的、不透明的」凭据，顺着它切分即可：wire 只认「一串不透明字节 + 反重放」，密钥学留在账号服务 | DS 设计概要 M1 ③、M2 ④；阶段 0 卡 0-1/0-2 |
| 10 | 关闭原因码增列（**落点已变**） | **走 wire 契约的 snake_case 词表**，增列 `connection_timeout` / `protocol_violation` / `input_rate_exceeded` / `send_buffer_overflow` / `normal_logout`；其中 `send_buffer_overflow`（出站积压）与既有 `queue_full`（入站队列）是两个方向，分开两个码 | 原推荐是「数字码 1054–1058、不另立 band」，依据是 `ids/index.json` 的 ErrorCode 命名空间——**该文件已于 2026-09-01 09:39 删除**。现行错误码是 `engine/wire/hello-wire-v1.json` 里的字符串词表（`bad_envelope`/`queue_full`/…），数字与 band 的问题不存在了 | DS 设计概要 M1 ⑤；阶段 0 卡 0-1 |
| 3 | 空间粗筛内核的身份（**落点已变**） | **在 root ABI 增函数槽**，照 `create_clr_host` / `clr_host_call` / `destroy_clr_host` 三槽先例（create / call / destroy），状态码复用既有 `status` 词表；具体签名归实现卡 | 原推荐是「新增 Capability `EntitySpatial`，numeric 10」，依据是 `ids/index.json` 的 Capability 命名空间——**已删除**。现行 ABI 真值是 `engine/abi/native-abi.json`，root 表目前只有 4 个槽（`ping` + 三个 CLR host），没有 capability 位的概念 | DS 设计概要 M5；阶段 0 卡 0-5 |
| 7b | 「进视野排队」的 DS 半边 | **预算、曲线、与另两处回流队列（实体变更待发集 / 体素差量队列）的统一纪律归 DS**；三处队列共守「截断不丢弃、回流升权、关键类别硬饥饿上限」 | 承板 7 的「语义归 ECS，预算归 DS」 | DS 设计概要 M7 ③④ |
| D-2 | 原定稿 16 条冻结语义的清点结果 | 15 条已在新正文有落点；**第 11 条的「两端对畸形输入的策略允许不同」重构时漏了，已补回**（服务器收到畸形一律断连；客户端收到服务器畸形按致命级处理，不要求对称） | 逐条清点而非抽查——这类「一句话挂在长列表里」的规则最容易在重写时蒸发 | DS 设计概要 M1「不干什么」 |
| 2 | GAS 把 Attribute Current 写回 ECS 落哪一相 | **`EcsCommandBufferCommit` 相尾**。`GasAndEventFinalize` 只做状态取样与事件产出，不写实体字段 | 写 Attribute 组件字段 = 写 `GameWorld` 域，13 相里只有 `EcsCommandBufferCommit` 可写该域；而提交点相的可写域是 `GasEvents`。原文只说「提交点前」，没指相 | GAS 设计概要 §2「一帧里 GAS 在哪几格干活」、M4 ⑤ |
| G-1 | 表现缓冲住哪儿（原定稿真歧义） | **表现缓冲产出在 `GasAndEventFinalize` 相、属 `GasEvents` 域，不是 ECS 组件字段**；Effect 条目上的 `fx_key` **静态载荷**仍住 ECS。一句话：**载荷住 Effect，触发走 GasEvents** | 原文一边说「表现载荷在 Effect 条目上，无 FxComponent」（= ECS 组件字段 = `GameWorld` 域），一边说「提交点产出表现缓冲条目」（= `GasEvents` 域）——两句指向两个不同的可写域，实现方必然二选一猜。定论依据是原文自己的语义 25：**表现缓冲不进哈希、不存档**——不进哈希不存档的东西不该住在 ECS 组件里（ECS 权威字段默认进存档） | GAS 设计概要 M8 ③；阶段 0 卡 0-3 |
| S-1 | 存档垂直切片的范围 | **本轮收窄到组件级**：只验证单个组件最后状态字段的存档/恢复往返；整房间进程重启恢复留给持久化主线 | RM-00011 Owner 裁决（本室非目标） | 存档设计概要 §5 阶段 1 第 4 项 |
| R-1 | 五份文档改名与全仓链接策略 | 设计概要类**去日期前缀**（活文档）：`lumio-{ecs,ds,gas,save,config}-design-overview.md`，全部 `git mv` 保历史；历史类**保留日期前缀**（某天的记录）。全仓 `.md`/`.json` 内的旧文件名链接一律改写为新名；**`.sdd/*.diff` 四个文件刻意不改**——路径是 diff 语法的一部分，改了会破坏那份历史 diff 的可用性 | 设计概要会一直改，带日期会让人误以为是某天的快照；而流水/审计确实是某天的记录，该带日期 | 五份文件已改名；18 个文件的链接已更新；全仓旧名零残留 |
| R-2 | 「原文照搬」附录里的链接怎么办 | 附录内文的**文档链接同步更新为新文件名**，其余原文不改；并在附录里显式写明这一点 | 不更新就是一堆死链；更新了却仍宣称「原文照搬」就是撒谎——所以写明 | GAS / 存档两份流水的附录 C 开头 |
| R-3 | 本轮与 2026-09-01 PR #52（C-1 玩法命令信封）的 blast radius | 四份设计概要的「交付按 Living Architecture」前言全部改写为 `repository-architecture.md`「变更顺序」的精确口径：**其余公共语义各落一份独立的 `engine/wire/<name>-v1.json`，不得扩展 `hello-wire-v1.json`**，校验走 `node eng/verify-wire.mjs`；DS 阶段 0 卡 0-3 标记为已落地 | 本分支 rebase 到 `935a8a9` 后重算——`main` 推进极快，rebase 后不重算 blast radius 就会发布一份当天就过期的规范 | 四份设计概要 §5 前言；DS 卡 0-3；本表 D-009 行与 W-1 行 |

## RM-00011 并入清单（2026-09-01 Owner 裁决 → 写进新正文的新现状）

| 面 | 旧定稿写的 | 新正文写的 |
|---|---|---|
| 重连 | DS 裁决 11：断线 = 登出 + 重登；会话保留窗口**降为预留位** | **5 分钟保留窗口是正式功能**：服务器实体保留、房间照常模拟、显式 disconnected 状态；重连做全新握手 + rebind **同一 NetEntityId**，只重建客户端 ReplicaWorld；窗口用进程内单调钟、不跨重启；超时销毁，再登录建新实体 |
| 重复登入 | 无 | **接管（takeover）**：同账号新的已认证准入踢掉旧连接（带显式终止通知），走 rebind 路径接同一实体 |
| D-009 | DS 裁决 19：继续封锁、图上留洞 | **已解冻并已落地**（2026-09-01 PR #52）：ADR-049 转 Accepted，通用玩法命令信封的契约真值在 `engine/wire/gameplay-command-envelope-v1.json`，校验入口 `node eng/verify-wire.mjs`；ChatInput 是第一个租户，已按「挖方块」纸面套验过通用性 |
| 宿主 | DS 裁决 7：Rust = 连接与字节，C# = 语义与真值 | **C# 宿主先行**保 MS-00001 日期，切片级最小 Rust 宿主并行起建；C# 切片验收通过后 Rust 宿主重跑同一验收套，通过则冻结 C# 宿主退为参考。**公共契约与宿主无关** |
| 拓扑 | DS 分层总图：DS 核心 + 语义层 + 体素 | **多一个 Account Server**：LumioServer 仓下 `account-server/` 独立 C# 进程，自带低频 ECS World，AccountEntity 是 ECS 实体；凭据材料不进普通组件 |
| 多 World | ECS §2：V1 大世界单地图 = 单权威 World；U-1 副本/多地图**未决** | **Room = 多个互相隔离的 GameWorld 实例**，U-1 就此关闭。一个连接同时只在一个 Room |
| Timer | 两份定稿都无此概念 | **两层都是一等公民**：宿主 Timer 服务（单调时间，管 5 分钟重连期限）+ Native Tick/Frame Timer Manager（CallbackSlot，管节拍任务）；本切片两层都有真实消费者 |

## 待修项（本轮发现、不在本轮改）

| # | 问题 | 证据 | 建议处置 |
|---|---|---|---|
| ~~W-1~~ | ~~RM-00011 Wave 0 契约卡的交付口径与已删除的 Baseline 链冲突~~ | — | **已自行闭合**（2026-09-01 PR #52）：C-1 实际按 living-architecture 口径交付（ADR-049 明写「Delivered as a pre-launch living-architecture wire contract, not a baseline event」），且 `repository-architecture.md`「变更顺序」已重写为 ABI / `engine/wire/<name>-v1.json` + `verify-wire.mjs` 三步。**余下三张 Wave 0 卡的卡面文字若仍写旧链，修订权归 TD** |
| W-2 | ADR-027 的 Contract 段引用 `tick-phase-contract.schema.json`（已删）；13 相矩阵的权威摘要现只存于 `LumioGameEngine_Architecture_v1.4.md` §4.5 | ADR-027「Contract」段；v1.4 §4.5 表 | Accepted ADR 正文不可改写；需新增一张 ADR 记录「相位矩阵的现行落点」，或在 Living Architecture 下把该表迁入 `engine/` 契约。本轮先在 ECS 设计概要里把可写域约束写死，作为唯一在用出处 |
