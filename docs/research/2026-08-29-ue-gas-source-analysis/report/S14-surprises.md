# S14 · 源码里的意外发现（跨章汇总）

> 结论先行
> 1. Epic 在 GAS 里留下 **33 条 TODO/FIXME/@todo 告解**（GAS 模块）+ 7 条（GameplayTags），最重的三块：堆叠规则未做完数据校验（目标版本写着 "5.11"）、周期执行的 cue 复制策略未按配置过滤、属性体系正计划整体升级为 struct-based attributes。
> 2. 废弃史清晰可读：**5.3 单体 GE → 组件化**（28 个符号一次性弃用）、**5.5 NonInstanced + Spec.ActivationInfo 弃用**、**5.7 Tag 复制双容器弃用**、**5.8 ActiveGEHandle 全局 TMap 移除**——每一波都是一次架构承认。
> 3. 编译期分支带来的行为差异集中在 `#if WITH_EDITOR`（123 处，多为编辑器数据校验）与 `!UE_BUILD_SHIPPING`（24 处，含预测键 RPC 拦截调试、DenyClientActivations、循环依赖全量扫描）——**Shipping 构建少掉的恰恰是"抓 bug 的网"**。

## 14.1 Epic 的自白（TODO/FIXME 转述，坐标为准）

### 直接影响正确性的
| 坐标 | 自白（转述） |
|---|---|
| GameplayEffect.h:2405 | 堆叠相关字段缺 EditCondition 校验，TODO 目标版本写 **"5.11"**——5.8 的堆叠 UI 还能配出非法组合 |
| GameplayEffect.cpp:4547-4548 | 代码"假设（可能不正确）堆叠不改变抑制状态"，复杂动态 tag 场景未处理 |
| GameplayEffect.cpp:4263-4267 | 堆叠时新旧动态授予/资产 tag 不同 → 只 ensure 不 diff（"come back to this later"） |
| GameplayEffect.cpp:3200 / 3352 | 周期执行 cue "现在每个 execute 都走 multicast RPC"，未检查复制策略 |
| AbilitySystemComponent_Abilities.cpp:2399-2400 | "Fixme: We need a better way to link up/reconcile predictive replicated abilities"——预测实例与服务器实例的对账是已知债务 |
| AbilitySystemComponent_Abilities.cpp:2685 | 服务器处理某调用时"应该检查 client/server ability type 但没查" |
| GameplayModMagnitudeCalculation.cpp:57 | 自定义幅度计算的 filter "从未被应用"（死参数） |
| GameplayEffectAggregator.cpp:525 | AddModsFrom 不确定该不该广播脏（"should this broadcast dirty?"） |
| AttributeSet.h:226-234 | @todo 计划弃用裸 float 属性；Pre/PostGameplayEffectExecute 的 execute-only 语义写死 |
| GameplayPrediction.h:218-263 | 整节 "Unsupported / Issues / Todo"：触发事件不复制、链式激活无法回滚、meta 属性不可预测、乘法预测基数错误、弱预测构想（正文各章已引） |

### 工程债自白（摘选）
| 坐标 | 自白 |
|---|---|
| AbilitySystemComponent.cpp:58 | 组件默认开 tick 是"临时措施"，直到想清楚 timer 处理 |
| AbilitySystemComponent_Abilities.cpp:1414 | 作者怀疑把能力标记为垃圾而不调 EndAbility 是错的 |
| AbilitySystemComponent_Abilities.cpp:3437 | 动画通知丢失问题："surprised nobody noticed"（没人发现才奇怪） |
| GameplayEffect.cpp:4908-4911 | "Hack: force netupdate on owner... Open issue with network team" |
| AbilitySystemComponent.cpp:1570-1590 | 三段 "Original Hack"：Mixed 模式下用服务器发起预测键防止拥有客户端双播 cue |
| GameplayCueManager.cpp:479 | "Animation preview hack"：CDO 上播 cue 跳过回收与 owner 赋值 |
| GameplayEffectAggregator.h:372 | @todo 想尽量消除聚合器上的 friend 声明（亲密耦合自白） |
| AbilityTask_MoveToLocation.cpp:53 | 自评 "an awful way to do this" |

## 14.2 废弃史 = 设计演进史（按版本，摘最有信息量的）

| 版本 | 弃用 | 替代 | 意味着什么 |
|---|---|---|---|
| 4.26 | ASC::RepAnimMontageInfo 公开成员 | getter/setter（将转 private） | montage 复制状态收权 |
| **5.3** | UGameplayEffect 上 **28 个字段/函数**（OngoingTagRequirements、GrantedApplicationImmunityTags、ChanceToApply、ConditionalGameplayEffects、GrantedAbilities、TargetEffectSpecs、UIData、各类 tag 容器…） | UGameplayEffectComponent 家族（11 个组件） | **单体数据资产 → 组合式组件**；免疫/持续条件/附加效果全部改为组件回调驱动（这是 S4 抑制机制重构的源头） |
| 5.4-5.5 | EGameplayAbilityInstancingPolicy::NonInstanced（UE_DEPRECATED_FORGAME）；FGameplayAbilitySpec::ActivationInfo；AbilitySystemGlobals 上 23 个配置项 | InstancedPerActor；每实例 ActivationInfo；UGameplayAbilitiesDeveloperSettings | **非实例化路线正式放弃**；配置迁到 DeveloperSettings |
| 5.6 | GetModifierMagnitude(int32,bool) 等计算接口 | 单参重载 + IsDataValid | 编辑期校验前移 |
| 5.7 | MinimalReplicationTags / ReplicatedLooseTags 两个复制属性 | EGameplayTagReplicationState（TagOnly/CountToOwner）+ 单一 GameplayTagCountContainer | **tag 复制从双容器归一到单容器**（Iris 未完工前的过渡态） |
| 5.8 | FActiveGameplayEffectHandle(int32) 构造器、ResetGlobalHandleMap/RemoveFromGlobalMap、AbilitiesGameplayEffectComponent 两个按 handle 的重载 | GenerateNewHandle/GetInstantExecutedHandle；内嵌 WeakOwningASC；按 FActiveGameplayEffect 重载 | **句柄自证化**；全局注册表退场 |
| 永久弃用（类） | UGameplayCueNotify_HitImpact（整类） | AGameplayCueNotify_Burst | 类注释让用户"用 UFortGameplayCueNotify_Burst"——**Fortnite 类名 UFort\* 泄漏在引擎源码里**（GameplayCueNotify_HitImpact.h:20） |

## 14.3 编译期分支与"编辑器对、打包错"

- `#if WITH_EDITOR`：123 处（59 文件）——大头是 IsDataValid/PostEditChangeProperty（数据校验只在编辑器跑）与 GameplayCueManager 的对象库/预览世界逻辑（Private/GameplayCueManager.cpp:56-1658 十余处）。**行为差异风险**：编辑器里 PIE 有 preview world 分支、dedicated server PIE 吸收 cue（:110）。
- `#if !UE_BUILD_SHIPPING`（含 `!(SHIPPING||TEST)` 24 处）：预测键 RPC 拦截与泄漏告警（GameplayPrediction.cpp:439-509）、DenyClientActivations、循环聚合器全量扫描（GameplayEffectAggregator.cpp:618-624）、CanActivate 失败日志、TargetActor 的非权威 spec 告警（GameplayAbility.cpp:1358）。**Shipping 少掉的是诊断网，不是逻辑分支**——但也意味着 Shipping 下同类 bug 静默。
- 正向 SHIPPING 分支仅 2 处（AbilityTask.cpp:24 的记录阈值、GameplayEffectTypes.cpp:1641 的错误日志详略）。
- 版本门控不用宏：GAS 内 **0 处** UE_VERSION_NEWER_THAN；一律 CVar 兼容开关（AbilitySystemPrivate.h:15-23 的注释明说 0x2/0x4 位是"重新打开 UE5.3 引入的 bug"的开关）。

## 14.4 死代码 / 实验开关 / 项目补丁痕迹

1. **`AbilitySystem.Fix.*` 家族 11 个 CVar**——每个都是"曾出过 bug、修了、留开关"的化石层（ActiveGEReplicationFix 位掩码默认 15 = 四个修复全开）。
2. `AbilitySystem.GameplayEffects.MaxVersion`（默认 CurrentVersion）+ FGameplayEffectVersion 序列化版本（Monolithic→Modular53，GameplayEffect.h:94-98）——资产升级路径的逃生门。
3. GameplayAbility.cpp:86 的 FIXME"temporary code to work around a crash"仍在主干。
4. `UGameplayCueNotify_HitImpact` 的 tooltip 指向不存在的 `UFortGameplayCueNotify_Burst`——Fort 项目补丁被带上引擎后未清理（本源码树 grep `UFort|Fortnite|Paragon` 仅此一处真实类名引用；README.md 提到 Lyra 两次但无代码）。
5. GameplayTags 的 NativeGameplayTags.cpp:37 "either those are wrong or this check is wrong"（自相矛盾的校验注释）。
