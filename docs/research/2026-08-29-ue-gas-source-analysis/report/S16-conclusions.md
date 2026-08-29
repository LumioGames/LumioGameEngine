# S16 · 源码级结论与可迁移性再判定

> 结论先行
> 1. 数据同步：GAS 是**双层模型——权威状态层（FastArray×3 + 属性组，可补、可对账）+ 键控事件层（RPC + PredictionKey 吸收，不可补）**。ECS 权威存储的引擎能整体继承状态层，事件层必须重表达为状态投影。
> 2. 状态回滚：**乐观收敛，终裁成立**；GAS 的"回滚"=按副作用注册的 delegate 撤销，确认单元 = 预测键（逻辑作用域），与目标引擎"整帧单一确认单元"差一个数量级的粒度，但 `FScopedAggregatorOnDirtyBatch` 已具备"作用域末尾统一提交"的形状。
> 3. 裸 GAS 不可哈希、不可任意帧快照，但**改造点是有界清单**（S11.3 六条），其中三条（求值公式纯函数化、顺序键显式化、时间源替换）在 GAS 代码里已有可平移的雏形。

## 16.1 十条源码级洞察

1. **通道内公式先于通道间顺序**。求值正确性的关键不在"通道按序"（那是容易看到的），而在通道内 `((Base+Add)*Mult/Div*Compound)+FinalAdd` 的**固定结合次序**与 Override 短路（GameplayEffectAggregator.cpp:76-99）。文档写不出这件事，因为它既不是配置也不是策略，是算式。对目标引擎：把这条算式写进 Schema 注释并指定浮点求值规范，求值顺序问题就一次性解决。
2. **乘法修饰是加性聚合**（`(1+Σ(m−1))`，SumMods 带 bias，216-229），只有 MultiplyCompound 真连乘。这是"两个 +10% 该得 +20% 还是 +21%"的语义根——GAS 的选择是**可交换的**（因而对顺序鲁棒），这个选择本身值得照抄。
3. **句柄是进程级身份，不是实体级身份**。三个计数器全是进程静态（GameplayAbilitySpecHandle.cpp:9-14 等），客户端 AGE handle 自铸（GameplayEffect.cpp:2870-2871）。GAS 敢这么做是因为它从不需要跨进程对账；目标引擎需要，所以句柄必须换成（世界 ID, 实体 ID, 代数）三元组——GAS 的代码恰好演示了**不该**怎么做，以及为什么它在自己 的约束下又完全够用。
4. **预测的撤销是"注册制"而非"日志制"**。每个副作用在产生处注册自己的清理 delegate（GameplayEffect.cpp:4519-4544），Reject 时广播（GameplayPrediction.cpp:299-338）。没有中心化的 undo log。这暴露设计意图：GAS 认为副作用种类有限且都愿意配合注册；目标引擎的 fail-stop 整帧回滚则相反——不信任副作用自觉，靠快照整体恢复。两者是光谱两端，中间形态（帧内 undo 栈）更接近目标需求。
5. **确认是两阶段的，且依赖属性复制顺序**（ClientActivateAbilitySucceed 立即 → ReplicatedPredictionKeyMap 追平才删预测副本；后者必须最后注册，AbilitySystemComponent.h:1951-1953）。目标引擎的"有界预测窗口"若要可证明，必须把这个隐含顺序变成线协议的显式阶段号——GAS 用声明顺序编码了协议阶段，这是 UObject 世界特有的暗契约。
6. **Epic 自己承认预测的四个洞**：链式激活不回滚（GameplayPrediction.h:218-226）、乘法预测基数错（237-247）、meta 属性不可预测（228-235）、触发事件不复制（220）。这四个洞恰好是目标引擎冻结语义里 RolledBack/Expired 必须处理的场景——UE 没解决它们不代表不可解，而是乐观收敛模型的结构性边界：**凡不能被"权威覆盖"无损覆盖的副作用，都不能预测**。
7. **抑制是"存在但摘除"而非"求值跳过"**（SetActiveGameplayEffectInhibit 物理移除/挂回 mods，AbilitySystemComponent.cpp:362-406）。这让抑制对求值顺序零影响、对状态哈希零加项（bIsInhibited 不复制）。目标引擎把抑制表达为 Active 内事件时，应保留这个"摘除"语义而不是加 if 分支——求值路径统一是确定性的一部分。
8. **GAS 对到达顺序问题的答案是"客户端缓存 + 延迟到批次末尾"**（PostReplicatedChange 用 ClientCachedStackCount/CachedStartServerWorldTime 区分变更类型 2913-2936；cue 推迟到 PostReplicatedReceive 5271-5309；base 重算用 NetUpdateID 去重 3463-3498）。模式统一：**不信任事件顺序，只信任状态 diff + 本地缓存对账**。这正是目标引擎快照对账可以借用的形状。
9. **Iris 适配清单是官方的"线协议结构清单"**（EffectContext/TargetData/TagCount/CueProxy/AnimMontage，Public/Serialization/ 九个序列化器），且 TagCount 未打通的注释（GameplayEffect.cpp:4669-4670）标出了最难序列化的结构：**带计数语义的集合**。目标引擎 Schema 从这张清单入手，等于让 Epic 替你做了一轮筛选。
10. **可测试性分层是设计出来的**：PredictionKey 单测免 World（PredictionKeyTests.cpp:15-138），因为它没碰 UObject；GE 套件必须建 World（GameplayEffectTests.cpp:790）。GAS 的句柄层/求值层/tag 层天然可单测，激活层/复制层不可——目标引擎按"纯数据机制 vs 世界机制"切模块，测试成本曲线会自动复刻这个分界。

## 16.2 两个判断题的终裁（证据链）

**数据同步**：状态层（S7.2 十五组复制属性 + FastArray×3）继承价值高——它已经把"权威真相"组织为可增量、可补发、可对账的结构；事件层（R14-R23 的 RPC 族）不可继承——其正确性依赖 PredictionKey 协议与 UObject 生命周期。目标引擎的分法：**权威存储 diff 出状态层，订阅投影生成事件层**，即可获得 GAS 的可用性而不继承其脆弱性。

**状态回滚**：终裁=**乐观应用 + 权威覆盖收敛**（S8.2.7 三步证据链）。窗口等价物=FScopedPredictionWindow（调用栈级）；与"整帧单一确认单元"的距离=①粒度（键 vs 帧）②边界（RPC 往返 vs 提交点）③覆盖面（注册了 delegate 的副作用 vs 帧内全部写入，含 ECS/体素）。结论：**不能把 GAS 预测层搬进目标引擎，但可以把它的"确认信号 + 副作用吸收"语义映射到帧模型**：帧号即预测键、提交点即 ScopedPredictionWindow 析构、权威帧到达即 catch-up。

## 16.3 状态机对照表（升级版）

完整 CSV：`appendix/state-machine-crosswalk.csv`。要点（UE 侧载体均有坐标）：
- UE 的显式态：EGameplayAbilityActivationMode{Authority,NonAuthority,Predicting,Confirmed,Rejected}（GameplayAbilitySpec.h:25-45）→ 映射目标的 Rejected/预测中/已确认；**Executing/Expired 无对应物**（执行=实例存活；超时=无）。
- UE 的隐式态（目标侧必须显式化的）：blocked（BlockedAbilityTags 计数，ASC.h 层）、inhibited（bIsInhibited 不复制）、等待目标数据（AbilityTargetDataMap 条目存在）、等待动画（task 挂起）、pending-remove（IsPendingRemove 标志）、bActivateOnce 等——每个的载体与坐标见 CSV。
- 反向缺口：目标的 Expired（执行中超时）在 GAS **不存在**（无执行时限概念，只有 GE 时长）；RolledBack 对应 GAS 的 Rejected+清理 delegate 序列（是过程不是状态）。

## 16.4 不可照搬清单（升级版，含耦合强度）

| 项 | 耦合点坐标 | 强度 | 说明 |
|---|---|---|---|
| delegate 注册式撤销 | GameplayPrediction.h:435-467（全局 TMap+UObject 绑定） | **深**（换掉=重做预测协议） | 目标引擎用帧快照替代 |
| 属性 UObject 子对象 + OnRep 重算 | AttributeSet.h:184-265；GameplayEffect.cpp:3452-3511 | **深**（值语义 ECS 无法挂 delegate） | 求值公式可搬，宿主重写 |
| FastArray + 复制回调体系 | GameplayEffect.cpp:2767-2940 | 中（机制可替换，语义可搬） | ECS diff 替代 |
| FTimerManager 时长/周期 | GameplayEffect.cpp:4481-4508 | 中 | 换 tick 计数 |
| PredictionKey 定向序列化（只回源客户端） | GameplayPrediction.cpp:115-187 | 浅（纯位协议技巧） | 可直接借鉴 |
| Tag NetIndex 两段压缩 | GameplayTagContainer.cpp:69-126 | 浅 | 加握手校验后可抄 |
| 进程级计数器三件 | 各 handle cpp | 浅（但必须换） | 内容 ID 替代 |
| CanActivate 九步顺序 + 失败 tag 体系 | GameplayAbility.cpp:457-575 | 浅 | 可近乎照抄 |
| 通道求值算式 | GameplayEffectAggregator.cpp:76-99 | 浅（纯函数） | 必抄 |

## 16.5 推迟能力的风险再评估（量化）

| 推迟项 | UE 对应实现 | 量化 | 风险评估 |
|---|---|---|---|
| 触发图 | FGameplayTagQuery（token 流求值器，GameplayTagContainer.cpp:129-164 的 FQueryEvaluator + 全模块 60+ 引用点） | 1 个类族、约 500 行核心 | **低风险**：其本质是可序列化的布尔表达式树，目标引擎用 Schema 直接建模即可；GAS 的价值在于证明了"查询语言必须可序列化" |
| 公式虚拟机 | FGameplayEffectModifierMagnitude + AttributeBasedFloat + ScalableFloat + SetByCaller（GameplayEffect.h:123-405 一带；Calculation 三个类） | 4 个结构 + 2 个 UCLASS，~700 行 | **中低风险**：GAS 自己也在往"声明式量（curve/scaler/attribute 捕获）+ 逃生舱（C++ Calc）"收敛；目标引擎先做声明式三件套（常量/曲线/属性捕获）+ 外部计算接口即可覆盖 90% |
| 复杂依赖求解器 | linked aggregator 依赖（RegisterLinkedAggregatorCallbacks + OnMagnitudeDependencyChange，GameplayEffect.cpp:2474-2486/3513-3570）+ 循环防护（BroadcastOnDirty 的 MAX_BROADCAST_DIRTY=10） | ~250 行 + 全局批量器 | **最高风险的一项**：GAS 用"运行时回调和递归上限"而不是"静态依赖图"求解，正因如此才有循环警告与顺序敏感。目标引擎若依赖属性间派生（MaxHealth←Strength），**必须**先定静态求值序（拓扑排序）再谈其他——GAS 的痛苦（cyclic 警告、"values are not what you expect"）就是这个缺口的活证 |

## 16.6 如果从零重写（ECS + 确定性 Tick）

**保留**：四层身份模型（尤其 Spec 快照层）；CanActivate 九步与失败 tag 语义；通道求值算式与加性乘法聚合；"摘除式"抑制；两段 Tag 压缩 + 握手哈希；客户端缓存对账模式（PostReplicatedChange 三分法）；GE 应用的权限门/免疫/自定义条件的**顺序**（S3.2 步 3-7）。
**删掉**：delegate 注册式撤销、UObject 属性宿主、FTimerManager 时间源、FastArray（换 ECS diff）、三个进程计数器、RPC 事件层（换状态投影 + 显式阶段号）。
**改造**：预测键→帧号；ScopedPredictionWindow→提交点作用域；ReplicatedPredictionKeyMap→帧确认表（保留 ring buffer 有界性证明）；NetUpdateID 去重→快照 epoch；GameplayTask→显式挂起状态数据（可序列化，S10.4 的三张缓存表是反面教材）。
**必须新增**（GAS 没有可抄的）：Executing/Expired 显式态、执行时限、静态依赖求值序、状态哈希规范。

## 16.7 给决策者的一句话

GAS 的价值不在可搬运，而在**把每个问题的第一份工业级答案连同它的伤疤一起展示出来**：读它的实现等于读一份带勘误的考卷——目标引擎的冻结语义（Effect 六态、整帧回滚、封闭 Schema）几乎每一条都能在 GAS 源码里找到"为什么需要这样"的反面证据或正面雏形。
