# S1 · 句柄、标识与实例模型（源码级）

> 结论先行
> 1. 三类句柄全部是**进程级全局静态计数器**发号：SpecHandle = `static int32 GHandle = 1`（无回绕处理）、ActiveGEHandle = 匿名命名空间 `static int32 GHandleID = 0`（前置自增、int32 溢出回绕到 1）、PredictionKey = `static int16 GKey = 1`（回绕到 1）。**都没有 generation/version 位，都不回收**。
> 2. ABA 防护的实际形态不是代数位，而是**所有权弱引用 + 存活性反查**：`FActiveGameplayEffectHandle` 内嵌 `TWeakObjectPtr<UAbilitySystemComponent>`，`GetOwningAbilitySystemComponent()` 反查容器确认该 handle 仍活着（ActiveGameplayEffectHandle.cpp:43-55）。
> 3. 「类型/实例/句柄」在源码里是**四层**：类型 = UGameplayEffect/UGameplayAbility 类对象（Def/CDO 指针）；应用规格 = FGameplayEffectSpec（值拷贝、随 AGE 复制）；运行实例 = FActiveGameplayEffect（FastArray 项）/ UGameplayAbility 实例（UObject）；句柄 = 上述三个 int 族。预研说的「三层」漏掉了 Spec 这一独立身份层。

## 1.1 待证清单裁决表

| # | 预研说法 | 裁决 | 证据 |
|---|---|---|---|
| 1.1 | SpecHandle 计数器实现/回绕/回收未核实 | **裁决**：`static int32 GHandle = 1; Handle = GHandle++;`——int32、从 1 起、单调递增、**无回绕处理、无回收**；注释明言放 cpp 里是"避免跨执行单元的重复 static"。每 ASC 不独立，全进程共享 | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/GameplayAbilitySpecHandle.cpp:9-14 · FGameplayAbilitySpecHandle::GenerateNewHandle |
| 1.2 | ActiveGEHandle 分配与回收未核实 | **裁决**：匿名命名空间 `static int32 GHandleID = 0`，`++GHandleID`，溢出（<1）重置为 1；**全局静态、多 ASC/多世界共享同一号空间**——意味着跨 ASC 撞号只差时间窗；5.8 删除了全局 `TMap<handle,ASC>` 改为 handle 内嵌 WeakOwningASC | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/ActiveGameplayEffectHandle.cpp:8-28 · GetNextActiveGEHandleID/FActiveGameplayEffectHandle::FActiveGameplayEffectHandle；ActiveGameplayEffectHandle.h:66-70（UE_DEPRECATED(5.8) 的空壳 ResetGlobalHandleMap/RemoveFromGlobalMap，注释"You can remove this function"） |
| 1.3 | 句柄复用与 ABA 有没有防 | **裁决：无代数位；有弱所有权**。IsValid() 只查 `Handle != INDEX_NONE`；真正防悬空的是 WeakOwningASC + 反查。计数器 int32 从 1 到 21 亿才会回绕（PredictionKey 是 int16，约 3.3 万就回绕——但它的身份只活在确认窗口内，窗口有 ring buffer 兜底） | ActiveGameplayEffectHandle.h:50-60 · IsValid/WasSuccessfullyApplied；ActiveGameplayEffectHandle.cpp:43-55 · GetOwningAbilitySystemComponent；GameplayPrediction.cpp:189-197（int16 回绕） |
| 1.4 | 类型/实例/句柄实际几层 | **裁决：四层**（类型对象 → Spec → 运行实例 → 句柄），每层载体见 1.2 节表 | 见表 |
| 1.5 | KeyRingBufferSize=32 | **证实**：`const int32 FReplicatedPredictionKeyMap::KeyRingBufferSize = 32`；语义 = 每 ASC 的已确认键 FastArray **固定 32 槽**，键按 `Current % 32` 入槽，构造时 32 项全部 MarkItemDirty（晚加入客户端能收到全表）；溢出由 OnRep 的 stale 清扫处理（详见 S8） | Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/GameplayPrediction.cpp:686-695 · KeyRingBufferSize/FReplicatedPredictionKeyMap::FReplicatedPredictionKeyMap；702-707 · ReplicatePredictionKey |

## 1.2 四层身份模型（源码载体）

| 层 | 载体 | 生命周期 | 可序列化? |
|---|---|---|---|
| 类型 | `UGameplayEffect* Def` / `TSubclassOf<UGameplayAbility>`（类路径序列化） | 资产 | 是（类引用） |
| 应用规格 | `FGameplayEffectSpec`（Def+Level+Context+Modifiers 求值缓存+捕获） | 创建→应用或丢弃 | 部分（FGameplayEffectSpecForRPC 瘦身版随 AGE/RPC 复制） |
| 运行实例 | `FActiveGameplayEffect`（FastArray 项）/ `UGameplayAbility` 实例（UObject 子对象） | Active 期 | 是（AGE 走 FastArray；能力实例走 ReplicateSubobject） |
| 句柄 | `FGameplayAbilitySpecHandle`(int32) / `FActiveGameplayEffectHandle`(int32+weak+bool) / `FPredictionKey`(int16×2+bool) | 引用凭据 | SpecHandle 有 `operator<<`（4 字节，GameplayAbilitySpecHandle.h:44-49）；**AGE handle 明确不复制**——客户端在 PostReplicatedAdd 自铸新号（GameplayEffect.cpp:2870-2871 "Handles are not replicated, so create a new one"）；PredictionKey 有定向 NetSerialize |

## 1.3 各句柄内存布局（线协议适配性）

```
FGameplayAbilitySpecHandle  { int32 Handle }                          // 4B；UPROPERTY 可复制
FActiveGameplayEffectHandle { TWeakObjectPtr<ASC> (8B) + int32 + bool } // ~16B；但跨端比较无意义（两端各自发号）
FPredictionKey              { int16 Current + int16 Base + bool + FObjectKey(8B) }
                            // 网络形态: [1b valid][1b serverInit][int16 Current] ≈ 19 bit（详见 S8.2.2）
FGameplayAttribute          { TFieldPath<FProperty> + FString AttributeName + TObjectPtr<UStruct> Owner }
                            // 指针语义！靠 (Owner, Name) 对齐，GetTypeHash 用指针哈希且带 FIXME（AttributeSet.h:131-135）
```
- **跨端比较结论**：SpecHandle 有意义（随 ActivatableAbilities FastArray 复制，两端一致）；AGE handle 无意义（每端自铸，RPC 里传它只在"发起端自查"场景成立）；PredictionKey 只对来源客户端有意义（序列化级保证）。FGameplayAttribute 的指针哈希是**进程内标识**，跨端靠 AttributeSet 子对象路径对齐。

## 1.4 InstancingPolicy 的状态存储（社区高频坑的源码答案）

三种取值（GameplayAbilityTypes.h:36 起，`NonInstanced` 已标 UE_DEPRECATED_FORGAME 5.5，GameplayAbility.cpp:35 的 CVar `AbilitySystem.Fix.AllowNonInstancedAbilities` 默认 0 且注释"removed in UE5.5"）：

| 策略 | 状态存哪 | 关键行为 |
|---|---|---|
| InstancedPerExecution | 每次激活 `CreateNewInstanceOfAbility` 新 UObject，状态在实例成员上；`Spec->ReplicatedInstances`（复制）或 `NonReplicatedInstances`（不复制）持有（GameplayAbilitySpec.h:256-262） | 预测激活要求 ReplicationPolicy==ReplicateNo，否则 Error + 不本地激活（AbilitySystemComponent_Abilities.cpp:1947-1962，"we lack the code to predict spawning an instance and merge with the server spawned version"） |
| InstancedPerActor | 首次授些建主实例（GetPrimaryInstance），状态在主实例成员上 | 已激活时再激活：bRetriggerInstancedAbility → 先 EndAbility 再走；否则拒绝（AbilitySystemComponent_Abilities.cpp:1831-1852） |
| NonInstanced（弃用中） | 状态放在 **Spec 上**：`Spec->ActivationInfo`（已弃用字段，GameplayAbilitySpec.h:236-239）——**CDO 上的成员变量被所有激活共享**，这就是社区坑的根源：UGameplayAbility 成员在 NonInstanced 下是全局共享的 | InternalTryActivateAbility 用 PRAGMA_DISABLE 兼容旧路径（AbilitySystemComponent_Abilities.cpp:1861-1866） |

**Spec 的瞬态/复制字段一览**（GameplayAbilitySpec.h:193-272）：复制的有 Handle/Ability/Level/InputID/DynamicAbilityTags/DynamicAbilityTriggers/ReplicatedInstances；**NotReplicated 的有 ActiveCount（注释："Can't replicate until prediction accurately handles this"）、InputPressed、RemoveAfterActivation、PendingRemove、bActivateOnce、ActivationInfo（注释："needs to be overwritten locally on clients during prediction"）、GameplayEffectHandle（"valid only on Authority"）、NonReplicatedInstances**；SetByCallerTagMagnitudes 是无 UPROPERTY 的裸 TMap（不上网）。

## 1.5 意外发现

1. 5.8 把 `FActiveGameplayEffectHandle(int32)` 裸构造器标弃用（ActiveGameplayEffectHandle.h:30-35），理由自述"leaks internal implementation, leaves the Owning ASC undefined"——句柄从"裸数字"进化为"自证存活的凭据"，正是目标引擎句柄设计该走的路。
2. `GetInstantExecutedHandle()` 哨兵语义反直觉（头注释自己承认 "unintuitively returns false"，ActiveGameplayEffectHandle.h:40-48）：`IsValid()==false` 但 `WasSuccessfullyApplied()==true`。返回值编码了两比特信息。
3. ASC 查 handle → 效果是**容器线性扫描**（GameplayEffect.cpp:3384-3394 · GetActiveGameplayEffect），且注释承认数组不稳定导致无法用 map 加速（3683-3685）。
4. 预测键 `operator==`/`GetTypeHash` 忽略 Base（GameplayPrediction.h:370-390）——依赖关系不进入键身份，只影响 delegate 传播。

## 1.6 对目标环境的迁移含义

目标引擎要求「三层区分（类型 ID/实例 ID/句柄）」+「规范化字节」。GAS 的四层模型里，**Spec 层是最值得保留的发明**：它把"一次应用的全部参数与求值缓存"做成值类型，使应用成为纯函数式的快照操作（重放友好）。但 GAS 句柄的三个特性必须改掉：① 进程级全局计数器（不可快照、不可跨进程对账）→ 改为每世界/每实例命名空间 + 显式溢出策略；② 客户端自铸 handle（两端身份不一致）→ 改为复制型实例 ID；② 弱指针反查（依赖 UObject GC）→ 改为带 epoch 的代数句柄，一次比较完成存活性判定（比 WeakPtr 反查便宜且可序列化）。
