# S15 · 勘误：对预研报告的证实、证伪与修正

> 结论先行
> 1. 第 4 节待证清单 **33 条全部裁决完毕**：证实 12、修正/部分成立 14、证伪 2、源码中不存在 2、升级为 Verified-Src 若干（明细见下表与 CSV）。
> 2. 被源码**推翻或大幅修正**的预研结论集中在四处：Modifier 求值的通道内语义（不是逐 mod 顺序应用）、抑制机制的函数形态（旧函数已不存在）、「Executions 完全不预测」（存在预测执行路径）、Tag 索引表的一致性保护（哈希只算不用、错位静默）。
> 3. **重要前置声明**：委托方描述的第一波报告目录 `docs/research/2026-08-29-ue-gas/` **在本机不存在**（全盘搜索无果，见 search-log.md）。本章以任务书第 4 节所载预研论断（其转述了预研原文）为裁决对象；「预研章节」栏使用任务书编号。若后续拿到预研原文，本章可直接补齐其内部章节号。

## 15.1 裁决总表

| # | 预研章节 | 预研论断（摘要） | 预研置信度 | 裁决 | 源码结论（新） | 关键坐标 | 影响 |
|---|---|---|---|---|---|---|---|
| 1 | 1.1 | SpecHandle 有生成 API，计数器/回绕/回收未核实 | Reported | **证实+细化** | `static int32 GHandle=1; Handle=GHandle++`，int32、无回绕处理、无回收、进程级全局 | GameplayAbilitySpecHandle.cpp:9-14 | 句柄设计决策 |
| 2 | 1.2 | ActiveGEHandle 分配与回收未核实；若是全局静态计数器意味着什么 | Reported | **证实** | 匿名命名空间 int32 计数器、前置自增、溢出回绕到 1；**多 ASC/多世界共享号空间**；5.8 起句柄内嵌 WeakOwningASC（全局 TMap 已删） | ActiveGameplayEffectHandle.cpp:8-28；.h:66-70 | 多世界撞号只差时间窗 |
| 3 | 1.3 | 句柄复用与 ABA 防了没有无一手证据 | Reported | **修正** | 无 generation 位；防护=WeakPtr 所有权+容器反查；PredictionKey 的 int16 会回绕（3.3 万），但有 32 槽 ring + stale 清扫兜底 | ActiveGameplayEffectHandle.cpp:43-55；GameplayPrediction.cpp:189-197/594-684 | ABA 论断成立但防线形态特殊 |
| 4 | 1.4 | 类型/实例/句柄三层区分 | Reported | **修正** | 实为**四层**：类型(Def/CDO)→Spec→运行实例(FActiveGE/能力实例)→句柄 | S01.2 表 | Spec 是独立身份层，快照设计必须含它 |
| 5 | 1.5 | KeyRingBufferSize=32 来自 API 文档 | Reported | **证实（升级 Verified-Src）** | =32；FastArray 32 槽、键按 Current%32 入槽、构造全槽标脏；溢出走 stale 清扫 | GameplayPrediction.cpp:686-695/702-707 | 窗口边界量化 |
| 6 | 3.1 | Modifier 顺序「基础值→各聚合通道→当前值」为推断 | Reported | **修正（重要）** | 通道间确为升序串行；但**通道内不是逐 mod 应用**，是固定公式 `((Base+Add)*Mult/Div*Compound)+FinalAdd`，Override 首个符合者直接短路返回；且 op 有 9 种非 4 种 | GameplayEffectAggregator.cpp:76-99/447-479 | 直接推翻"按 op 分组顺序应用"的常见转述 |
| 7 | 3.2 | 同组 Effect 不同应用顺序结果是否相同未确证 | Reported | **修正** | 多数可交换；三个例外：Override 先加者赢且 RemoveAllSwap 会换赢家；Division 和≈0 强制 1；float 求和按数组序 | GameplayEffectAggregator.cpp:78-84/92-96/150-162/216-229 | 确定性结论 |
| 8 | 3.3 | ModChannel 容器与遍历顺序 | Reported | **证实** | TMap<enum,Channel>，每次插入 KeySort 保升序；迭代即升序 | GameplayEffectAggregator.cpp:231-243/250-261 | 通道序确定 |
| 9 | 3.4 | 同优先级 tie-break | Reported | **修正** | **没有优先级/稳定排序**；tie-break=插入序（RemoveAllSwap 破坏） | GameplayEffectAggregator.cpp:137-162 | 哈希可行性 |
| 10 | 3.5 | 快照×来源/目标四象限 | Reported | **证实** | Source 快照=Spec 创建时；Target 捕获=应用时；非快照=聚合器回调 | GameplayEffect.cpp:1838-1859/4387/2474-2486 | capture 时机表 |
| 11 | 4.1 | 堆叠/时长刷新/取消顺序证据缺口 | Reported | **证实（已钉死）** | 完整 12 步时序表（结构固定：堆叠→溢出→Spec 替换→时长→周期→锁外取消） | GameplayEffect.cpp:4171-4561（S4.2 表） | 冻结语义映射 |
| 12 | 4.2 | 堆叠被拒返回值 | Reported | **证实** | nullptr→无效 handle；与免疫/无权限/周期不可预测同值，**调用方不可区分**；仅 instant 成功有哨兵 handle | GameplayEffect.cpp:4245-4252；ActiveGameplayEffectHandle.h:40-48 | API 设计教训 |
| 13 | 4.3 | Inhibition 的进入/退出条件与 bIsInhibited 维护点 | Reported | **源码中不存在（旧形态）** | `InhibitActiveGameplayEffect`/`CheckOngoingTagRequirements` 在 5.8 已不存在；现为 `SetActiveGameplayEffectInhibit`+组件 tag 事件驱动；抑制=mods/tags 物理摘除 | AbilitySystemComponent.cpp:362-406；GameplayEffect.h:2360-2361（旧字段弃用） | 章节结论需重写 |
| 14 | 4.4 | 抑制态复制与否、客户端能否独立算出 | Reported | **证实** | bIsInhibited **NotReplicated**（Epic 注释自述"Not sure if this should replicate"）；客户端靠组件 tag 回调独立重算；cue 靠 pending 标志推迟 | GameplayEffect.h:1440-1446；GameplayEffect.cpp:2883-2885/5271-5309 | Effect 六态映射的关键输入 |
| 15 | 6.1 | Tag 内部表示 | Reported | **证实** | FName + 节点树；网络身份 uint16 NetIndex | GameplayTagContainer.cpp:1066-1123；GameplayTagsManager.cpp:839-864 | — |
| 16 | 6.2 | NetIndex 位布局与分段编码 | Reported | **证实（细化）** | 两段变长 [N bit][more bit][Max−N bit]；N=ini 可配、Max=ceil(log2(n+1)) | GameplayTagContainer.cpp:69-126；GameplayTagsManager.cpp:810-814 | 可照抄 |
| 17 | 6.3 | NetIndex 表构建时机与排序 | Reported | **证实** | 启动/树变更时；名字排序+常用前置；"两端一致"靠配置不靠协议 | GameplayTagsManager.cpp:767-837 | 隐含契约 |
| 18 | 6.4 | 表不一致的实际行为 | Reported | **证伪（重要）** | **没有任何运行时校验**：CRC32 哈希计算+打日志但全引擎 0 消费者；越界=ensure+静默 NAME_None；界内错位=**静默错认 tag，不断线** | GameplayTagsManager.cpp:818-836（哈希）；:843-848（越界）；全树 grep 仅定义处 | 若预研学过"有保护"则推翻；对目标引擎是硬教训 |
| 19 | 6.5 | MatchesTag vs Exact 的实现与代价 | Reported | **证实** | 节点树父链比对 | GameplayTagContainer.cpp 匹配区（FQueryEvaluator 同文件 129-164） | — |
| 20 | 6.6 | CountContainer 为何是 count | Reported | **证实** | TMap<tag,count>+父聚合；加减点=Loose API/GE 授予/堆叠变化 | GameplayEffectTypes.h:1059-1101；GameplayEffect.cpp:4663-4664/4971-4972/3585-3588 | — |
| 21 | 6.7 | Loose 与 Replicated 两套路径 | Reported | **修正** | 实为**三态**（None/SimulatedTagOnly/CountToOwner）+ 5.7 起双容器弃用、5.8 走单一 GameplayTagCountContainer；Iris 序列化器存在但未启用 | AbilitySystemComponent.cpp:1842-1874；GameplayEffectTypes.cpp:47 一带；GameplayEffect.cpp:4669-4670 | 双路径描述已过时 |
| 22 | 7.1 | 「不能简单说 GAS 只复制 Base」 | Reported | **证实（细化）** | Base/Current 都在 UPROPERTY；复制=AttributeSet 子对象通道；客户端把复制值当新 base 重算（legacy float 反推） | AttributeSet.h:48-53；GameplayEffect.cpp:3452-3511 | — |
| 23 | 7.2 | FastArray 增量粒度 | Reported | **证实** | 项级（变项整发；项内属性 delta 属引擎 FastArray 机制，DS 篇） | GameplayEffect.cpp:5264；GameplayAbilitySpec.h:300-338 | — |
| 24 | 7.3 | 三种 ReplicationMode 分支 | Reported | **证实（钉死）** | 仅两处分支：GetReplicationCondition（COND_* 映射）+ NetDeltaSerialize（连接过滤） | GameplayEffect.cpp:5183-5217/5219-5269 | — |
| 25 | 7.4 | Mixed 模式挂载陷阱 | Reported | **证实（源码形态钉死）** | `ParentOwner->IsOwnedBy(Connection->OwningActor) \|\| GetNetConnection()==Connection`（含子连接遍历）；owner actor 必须被接收连接拥有 | GameplayEffect.cpp:5230-5261 | 陷阱的机理表述 |
| 26 | 7.5 | RPC batching | Reported | **证实（细化）** | 恰好打包 3 调用共享一键；乱序入队放行规则；服务器 FakeInfo 自认 bogus | AbilitySystemComponent_Abilities.cpp:4184-4334 | — |
| 27 | 7.6 | 属性与 Effect 到达顺序不一致的坑 | Reported | **证实（防护清单）** | 五层防护：预测键表最后注册（头注释）、容器锁、cue 延迟到数组收完、NetUpdateID 去重、3 秒 cue 规则 | AbilitySystemComponent.h:1951-1953；AbilitySystemComponent.cpp:1959-1975；GameplayEffect.cpp:5271-5309/3463-3498/2842-2858 | 该坑有系统性防护 |
| 28 | 8.1–8.8 | 预测键系列（8 条） | Reported/待核 | **全部裁决**（见 S8 裁决表）：乐观收敛终裁成立；窗口=调用栈；确认=RPC+复制追平两阶段；撤销=delegate 注册式；不预测清单已源码穷举 | S08 全章 | 预研核心结论「乐观应用+权威覆盖收敛」**被源码证实** |
| 29 | S9（预研若称） | 「Executions 完全不预测」 | Reported | **修正** | 头注释这么说（GameplayPrediction.h:39），但 `PredictivelyExecuteEffectSpec` 在客户端预测路径**会**跑 ExecCDO->Execute（3138-3172）；主应用路径不预测 Execution | GameplayEffect.cpp:3069-3207 | 文档与实现的偏差 |
| 30 | S12（名称待核） | 各 CVar/命令/枚举名 | 待核 | **全部钉死** | 76 CVar + 30 命令入 CSV；含三个拼写陷阱（Threshhold 双 h、Recalcuate、ServerRPCBatching.Log） | cvar-and-commands.csv | 清账 |
| 31 | S13-Iris | Iris 适配现状 | Reported | **修正** | 适配已存在（9 序列化器+Build 接线），但 TagCountContainer 序列化器未打通（CVar 默认 false+注释自认）、Iris 本体 Beta | GameplayAbilities.Build.cs:42；GameplayEffect.cpp:4669-4670；Iris.uplugin | 「无适配」与「已就绪」都不成立 |
| 32 | S13-Mass | GAS↔Mass 桥接 | Reported | **证实（零桥接）** | 双向 grep 0 命中；唯一共享=GameplayTags（13 处） | S13.2 搜索表 | — |
| 33 | S13-uplugin | 插件成熟度字段待核 | 待核 | **裁决** | uplugin **无** Experimental/Beta 成熟度字段（IsBetaVersion: false 且无 IsExperimentalVersion 键） | GameplayAbilities.uplugin:13-16 | — |

## 15.2 没有被改变、但被升级为 Verified-Src 的预研要点（摘要）

预激活链失败出口、CanActivate 顺序（冷却→消耗→Tag）、Commit 三段式、GE 应用路径分叉（Instant/Duration 行号级）、到期路径（timer 驱动+策略分叉）、免疫检查点位置、预测键生命周期、FastArray 三回调行为、late-join 3 秒规则、Minimal/Mixed/Full 的行为差异——全部从 Reported/推断升级为 Verified-Src（坐标见各章）。

## 15.3 机器可读版

`appendix/corrections-to-wave1.csv`（编号/预研章节/原论断摘要/预研置信度/裁决/新结论摘要/证据坐标/影响面）。evidence-index.csv 中「是否改变预研结论」列与此对应。
