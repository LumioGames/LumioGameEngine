# 执行摘要 · UE GAS 源码级分析（2026-08-29）

**基线**：Unreal Engine **5.8.2**（git `ff8421f2b`，分支 `5.8`，2026-08-25）。本报告所有行号与符号相对该版本。
**对象**：Gameplay Ability System 全链路源码（GameplayAbilities 插件 + GameplayTags/GameplayTasks 模块），函数体级阅读 ~8,000 行，覆盖激活、Effect 应用与求值、堆叠/时长/抑制、复制、预测键、Cue、Task、确定性、调试设施、Iris/Mass 集成。
**置信度**：正文论断以 Verified-Src 为主（124 条证据入 `appendix/evidence-index.csv`，每条带 路径:行号·符号 三件套）；源码注释级引用标 Verified-Doc；对预研（wave-1）的转述标 Reported。

## 十个最重要的源码级发现

**1. 预测的终裁：乐观应用 + 权威覆盖收敛，不是回滚重放（证实预研）。**
全代码库不存在「存快照→恢复→重放」路径；撤销唯一原语是 `FPredictionKeyDelegates`（进程级 TMap）里各副作用注册的 delegate（GameplayPrediction.cpp:299-355）。收敛靠三条权威覆盖：属性 OnRep 重算（GameplayEffect.cpp:3452-3511）、AGE FastArray 回调（2804-2940）、`SetBaseAttributeValueFromReplication` 的回绕-求值-再设（3743-3772）。确认是两阶段的：`ClientActivateAbilitySucceed`（可靠 RPC，立即）只标 Confirmed，预测副本要等 `ReplicatedPredictionKeyMap`（32 槽 ring buffer FastArray，GameplayPrediction.cpp:686）在属性复制通道追平才删除。

**2.（修正预研）Modifier 求值不是「逐 mod 顺序应用」，通道内是一条固定算式。**
`((Base + Additive) × Multiplicitive ÷ Division × CompoundMultiply) + FinalAdd`（GameplayEffectAggregator.cpp:76-99）；**Override 是首个符合条件者短路返回（先加者赢）**；乘法类 mod 是 `(1+Σ(m−1))` 的**加性聚合**而非连乘（216-229）；操作符实际有 **9 种**（447-479 的 switch），不是社区常说的 4 种。通道间按枚举升序串行，且通道容器是 TMap——靠**每次插入后 KeySort** 保序（231-243）。

**3.（修正预研）抑制（Inhibition）机制的旧函数在 5.8 已不存在。**
`CheckOngoingTagRequirements` / `InhibitActiveGameplayEffect` 全模块 grep 零命中；现行入口是 `UAbilitySystemComponent::SetActiveGameplayEffectInhibit`（AbilitySystemComponent.cpp:362-406），触发源是 GameplayEffectComponent 的 tag 事件；抑制的语义是**把 mods/tags 从聚合器物理摘除**而非求值时跳过。`bIsInhibited` 字段**不复制**——Epic 在字段旁注释"Not sure if this should replicate or not"（GameplayEffect.h:1440-1442），客户端靠组件回调独立重算。

**4.（证伪「有保护」的想象）GameplayTag 索引表的一致性没有任何运行时校验。**
两端 NetIndex 表按「tag 名排序 + 常用前置」构建（GameplayTagsManager.cpp:767-837），全表 CRC32 哈希**被计算、被写进日志、但全引擎零消费者**（GameplayTagsManager.h:634 仅定义）；表错位的实际行为：索引越界 → ensure + 静默返回 NAME_None；**索引在界内但映射错位 → 静默错认 tag，不断线**（839-850）。这是「静默数据损坏」的教科书案例，也是目标引擎握手哈希必须堵的洞。

**5.（修正预研）Iris 适配「存在但未打通」，两头的极端说法都不成立。**
GameplayAbilities.Build.cs 调 `SetupIrisSupport`，Public/Serialization 下有 9 个 NetSerializer 适配（EffectContext/TargetData/TagCount/CueProxy/AnimMontage…）；但 `AbilitySystem.Fix.ReplicateTagCountContainerWithIris` **默认 false**，源码注释直说"we do not have working Tag Count replication through GameplayTagCountContainerNetSerializer"（GameplayEffect.cpp:4669-4670）；Iris 插件本体 `IsBetaVersion: true`（Experimental 目录）。

**6. Mass 与 GAS 双向零桥接（一手定论）。**
GameplayAbilities 模块 grep `Mass` 唯一命中是注释里的英文单词 "massive"；反向在 MassGameplay 插件全部 ~280 个源文件里 grep GAS 符号 = 0 命中；两者唯一共享底座是 GameplayTags 模块（13 处引用）。「Epic 在把 GAS 搬上 Mass」在 5.8 引擎源码中无证据。

**7. 句柄是进程级身份：三个计数器全是进程静态量，无代数位、无回收。**
SpecHandle = `static int32 GHandle=1` 单调递增（GameplayAbilitySpecHandle.cpp:9-14）；ActiveGEHandle = 匿名命名空间 int32、溢出回绕（ActiveGameplayEffectHandle.cpp:8-28）；PredictionKey = **int16**、约 3.3 万回绕（GameplayPrediction.cpp:189-197）。ABA 防护靠 WeakPtr 所有权 + 容器反查，不是 generation。5.8 刚删除全局 handle→ASC 注册表，改为句柄内嵌 WeakOwningASC。**客户端收到的 AGE 会自铸本地 handle**（GameplayEffect.cpp:2870-2871）——两端句柄身份各自独立。

**8. 激活链的失败出口与检查顺序被穷举钉死。**
`CanActivateAbility` 固定 9 步：角色/安全 → ASC → Spec → 用户抑制 → **冷却 → 消耗 → Tag** → 输入阻塞 → 蓝图覆盖（GameplayAbility.cpp:457-575）——冷却排在消耗与 Tag 之前。Commit 三段式里 `CommitExecute` **先冷却后消耗**，且 CommitCheck 刻意不复查 Tag（注释自述：自己激活带来的 tag 会误拒自己）。服务器对客户端激活的「真校验」就是重跑一遍 InternalTryActivateAbility；目标数据上行的「验证」只有指针有效性（AbilitySystemComponent_Abilities.cpp:4033-4045）——**服务器原样信任客户端目标数据**。

**9. 确定性裁决：裸 GAS 不能产出稳定状态哈希，但改造点是有界清单。**
拦路的四类：顺序敏感容器（AGE `RemoveAtSwap`、mod 数组 `RemoveAllSwap` 会改写 Override 赢家）、墙钟时间源（时长/周期全走 FTimerManager + World time）、进程级全局单例（键计数器、当前 GE 指针、委托表、脏集）、无归约规范的浮点累加。S11.3 给出六条坐标级清单——其中求值公式本身已是纯函数，确定性是「没做」而非「做不到」。

**10. Epic 自己写下的限制清单是全报告最贵的一页。**
GameplayPrediction.h:22-264：链式激活无法回滚（"not possible out of the box"）、乘法类 Effect 客户端预测基数错误（+10%+10% 得 605 而非 600 的自述）、meta 属性不可预测、触发事件不复制、"we do not predict over multiple frames"。这四个洞恰好框定了乐观收敛模型的结构边界：**凡不能被权威覆盖无损覆盖的副作用，都不能预测**。

## 对目标引擎的三句话

1. **能继承**：权威状态层的全部结构（FastArray 语义、四层身份模型中的 Spec 快照层、通道求值算式、摘除式抑制、客户端缓存对账模式）与 CanActivate 九步顺序语义。
2. **不能继承**：delegate 注册式撤销、UObject 属性宿主与 OnRep 重算、FTimerManager 时间源、RPC 事件层——目标引擎的帧级提交点与 ECS 权威存储要求把这几块整体重建。
3. **必须自建**（GAS 没有可抄的）：Executing/Expired 显式态、执行时限、静态依赖求值序（GAS 用运行时递归+10 层上限硬扛，注释承认"values are not what you expect"）、状态哈希规范。

## Known gaps（读源码仍无答案）

见 README 同名节——最重要的一条：**wave-1 预研原文不在本机**，S15 勘误以任务书转述的预研论断为对象；拿到原文后 S15 可补内部章节号映射。
