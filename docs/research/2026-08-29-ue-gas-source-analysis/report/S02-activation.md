# S2 · Ability 激活调用链（逐函数 + 失败出口穷举）

> 结论先行
> 1. 激活链是 `TryActivateAbility → InternalTryActivateAbility → {服务器: InternalServerTryActivateAbility → InternalTryActivateAbility(重入)；客户端预测: FScopedPredictionWindow → ServerTryActivateAbility RPC → CallActivateAbility}`，两条路**复用同一个 InternalTryActivateAbility**——服务器把客户端请求当本地激活再跑一遍全部检查。
> 2. `CanActivateAbility` 的检查顺序是**固定的 9 步：角色/安全 → ASC → Spec → 用户输入抑制 → 冷却 → 消耗 → Tag 满足性 → 输入绑定阻塞 → 蓝图覆盖**（GameplayAbility.cpp:457-575）——冷却排在消耗前面，Tag 排第三；失败原因靠 OptionalRelevantTags 的 tag 顺序可观察。
> 3. Commit 三段式：`CommitCheck`（重查冷却+消耗，**不查 Tag/输入**）→ `CommitExecute`（**先 ApplyCooldown 后 ApplyCost**）→ K2_CommitExecute → NotifyAbilityCommit；消耗/冷却与激活解耦的原因写在 CommitCheck 的注释里："激活中途状态可能已变，且此时 CanActivateAbility 会因自身激活带来的 tag/输入阻塞而误拒"。

## 2.1 失败出口穷举表（本章硬指标）

### TryActivateAbility（AbilitySystemComponent_Abilities.cpp:1604-1683）

| # | 条件 | 返回 | 通知 |
|---|---|---|---|
| 1 | Handle 无效 | false | Warning 日志 |
| 2 | `Spec->PendingRemove \|\| RemoveAfterActivation` | false | 无（静默） |
| 3 | Ability 无效 | false | Warning |
| 4 | ActorInfo/Owner/Avatar 无效 | false | 无 |
| 5 | NetMode==SimulatedProxy | false | 无 |
| 6 | 非本地 + LocalOnly/LocalPredicted | bAllowRemoteActivation ? ClientTryActivateAbility(客户端 RPC)+true : false | 日志 |
| 7 | 非权威 + ServerOnly/ServerInitiated | bAllowRemoteActivation ? 本地先跑 CanActivateAbility（带日志使能）→过则 CallServerTryActivateAbility(**不带预测键**，服务器将发服务器键)+true / 不过则 NotifyAbilityFailed+false : false | 日志+FailureTags |

### InternalTryActivateAbility（同文件 1704-1994）

| # | 条件 | 返回 | 通知 |
|---|---|---|---|
| 8 | Handle 无效 / Spec 不存在 | false | Warning |
| 9 | ActorInfo 无效 | false | 无 |
| 10 | NetMode 解析（PC 优先，avatar torn-off 边界案例注释）后 == SimulatedProxy | false | 无 |
| 11 | 非本地 + (LocalOnly \|\| (LocalPredicted && 无有效入参键)) | false | NetworkFailTag + NotifyAbilityFailed |
| 12 | 非权威 + ServerOnly/ServerInitiated | false | NetworkFailTag + Notify |
| 13 | TriggerEventData && !ShouldAbilityRespondToEvent | false | Notify（**tags 为空**） |
| 14 | CanActivateAbility 失败 | false | 默认失败 tag（`UGameplayAbilitiesDeveloperSettings::ActivateFailCanActivateAbilityTag`）补位 + Notify |
| 15 | InstancedPerActor 且 IsActive：bRetriggerInstancedAbility ? 先 End 再继续 : false | false/继续 | Verbose 日志 |
| 16 | InstancedPerActor 但主实例缺失 | false | Warning |

之后按 NetExecutionPolicy 分叉（见 2.3），成功尾部：MarkAbilitySpecDirty、AbilityLastActivatedTime=World time、日志带 PredictionKey。

## 2.2 CanActivateAbility 的顺序（钉死）

坐标：Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/Abilities/GameplayAbility.cpp:457-575

```
1  AvatarActor 有效 && ShouldActivateAbility(LocalRole)   // 非模拟代理 && (权威 || 安全策略允许)
2  ASC 有效
3  FindAbilitySpecFromHandle 命中
4  !GetUserAbilityActivationInhibited()                    // UI/系统级输入抑制
5  !ShouldIgnoreCooldowns() && CheckCooldown()             // 失败→ ActivateFailCooldownTag + 命中的冷却 tag
6  !ShouldIgnoreCosts() && CheckCost()                     // 失败→ ActivateFailCostTag
7  DoesAbilitySatisfyTagRequirements()                     // 失败→ ActivateFailTagsBlockedTag/MissingTag
8  !IsAbilityInputBlocked(Spec->InputID)                   // 输入绑定级阻塞
9  bHasBlueprintCanUse → K2_CanActivateAbility             // 失败→ 默认 tag + K2 失败 tags
```

Tag 满足性内部顺序（DoesAbilitySatisfyTagRequirements，349-443）：**先查全部阻塞**（资产tags×容器阻塞表 → 拥有tags×ActivationBlockedTags → Source → Target）**再查全部必需**（ActivationRequired → Source → Target），OptionalRelevantTags 的注释明说这个顺序是刻意的（"so OptionalRelevantTags will contain blocked tags first"）。
- 冷却的本质：`CheckCooldown` = ASC 当前拥有 tags 是否命中冷却 GE 授予的 tags（1064-1104）；冷却 GE 无 tag 时 Warning（`AbilitySystem.WarnCooldownEffectWithoutTags`，默认 1）。
- 消耗的本质：`CanApplyAttributeModifiers`，**只对 Additive op 检查 `CurrentValue + Cost < 0`**（GameplayEffect.cpp:5497-5528）。

## 2.3 NetExecutionPolicy 四值的实际网络流

坐标分叉点：AbilitySystemComponent_Abilities.cpp:1872-1967

| 策略 | 服务器/本地路径 | 客户端路径 | RPC |
|---|---|---|---|
| LocalOnly | 直接激活（1872-1924 分支） | 非本地收到请求 → ClientTryActivateAbility 转给拥有客户端（1652） | 客户端 RPC |
| LocalPredicted | （被服务器执行时走权威分支，用客户端键或新服务器键） | PrePredictionActivation(FlushServerMoves) → FScopedPredictionWindow 生成键 → ServerTryActivateAbility[WithEventData] 带键 → NewCaughtUpDelegate 挂 OnClientActivateAbilityCaughtUp → CallActivateAbility（1925-1967） | 服务器 RPC（可靠）+ ClientActivateAbilitySucceed/Failed（客户端 RPC） |
| ServerOnly | 权威创建服务器键激活；客户端只能请求（1660-1680：本地先 CanActivate，过则无键上行） | 不本地执行 | 服务器 RPC |
| ServerInitiated | 同 ServerOnly，但键为服务器发起（1875-1881 bCreateNewServerKey 条件含两种策略） | 同上 | 同上 |

服务器应答（InternalServerTryActivateAbility，2054-2125）：`#if WITH_SERVER_CODE` 包裹；测试后门 DenyClientActivation（非 Shipping）；Spec 缺失/ensure 失败/安全策略违规（ServerOnlyExecution/ServerOnly）→ ClientActivateAbilityFailed；**ConsumeAllReplicatedData 清旧执行残档**；FScopedPredictionWindow 接客户端键；再 InternalTryActivateAbility；失败 → Failed RPC + InputPressed=false + MarkDirty。

客户端收场：
- **拒绝**（2279-2333）：`BroadcastRejectedDelegate(PredictionKey)`（撤销总开关）→ Spec/实例 SetActivationRejected → `K2_EndAbility()`（蓝图侧被强杀）。
- **确认**（2362-2471）：找本地同键实例 → ConfirmActivateSucceed；找不到仅 Verbose 日志（不重建）；非预测能力（服务器发起）→ 客户端创建实例并 CallActivateAbility。**Epic 的 Fixme（2399-2400）**："We need a better way to link up/reconcile predictive replicated abilities"。
- **CaughtUp 仍 Predicting**（2335-2360）：只打日志不杀（注释：可靠 RPC 丢但属性 bunch 先到）。

## 2.4 NetSecurityPolicy 的实际作用点

1. `ShouldActivateAbility`（GameplayAbility.cpp:445-449）：非权威侧 ServerOnly/ServerOnlyExecution 直接 false（进 CanActivateAbility 第 1 步）。
2. 服务器入口拒绝客户端激活：InternalServerTryActivateAbility:2087-2093（ServerOnlyExecution/ServerOnly → Failed RPC）。
3. 终止方向：ReplicateEndOrCancelAbility:2147（客户端上行受 ServerOnlyTermination/ServerOnly 拦截）；ServerEndAbility_Implementation:2226-2229 / ServerCancelAbility_Implementation:2252-2255 同样拦截。

## 2.5 Tag 驱动的互斥/阻塞/取消：求值顺序与应用点

- **应用点 = PreActivate 尾部**（GameplayAbility.cpp:999）：`ApplyAbilityBlockAndCancelTags(AssetTags, this, bEnableBlock=true, BlockAbilitiesWithTag, bExecuteCancel=true, CancelAbilitiesWithTag)`——**先 block 后 cancel**（ASC 实现 1431-1446：先 BlockAbilitiesWithTags（BlockedAbilityTags 计数 +1）再 CancelAbilities）。
- **关键顺序约束**（1001 行注释）：Spec->ActiveCount 的自增**必须在 block/cancel 之后**，"否则能力可能在完全激活前误取消自己"。
- **取消链**：CancelAbilities → CancelAbilitySpec → ForceCancelAbilityDueToReplication（服务器说取消则强制 SetCanBeCanceled(true)，2213-2220 注释："We do not support 'server says ability was cancelled but client disagrees'"）。
- **「已授予但被阻塞」的表达**：Spec 留在 ActivatableAbilities.Items 里，`BlockedAbilityTags`（FGameplayTagCountContainer 计数）+ `IsAbilityInputBlocked(InputID)`（BlockedAbilityBindings 的 uint8 计数）在 CanActivateAbility 第 7/8 步拒绝——**没有"Blocked"状态位，只有检查时失败**。
- 激活授予的 tag：PreActivate 里 `AddLooseGameplayTags(ActivationOwnedTags)`（990，按 ShouldReplicateActivationOwnedTags 决定 CountToOwner 复制态）；EndAbility 对称移除（870）。

## 2.6 Commit 语义与失败回退

- `CommitAbility`（592-609）= CommitCheck + CommitExecute + K2_CommitExecute + NotifyAbilityCommit。失败（CommitCheck false）只返回 false——**没有自动 EndAbility**；模板代码在 ActivateAbility 的注释里（925-936）：失败后应由能力自己 `EndAbility(replicate=true, cancelled=true)`。
- `CommitCheck`（648-682）：有效性三查 → 冷却 → 消耗。注释（650-656）解释为何不复用 CanActivateAbility：input inhibition 会误拒（自己激活可能恰好带来阻塞自己的 tag）。
- `CommitExecute`（684-689）：**ApplyCooldown → ApplyCost 顺序固定**。冷却/消耗都是 GE（ApplyGameplayEffectToOwner，1106-1145），因此进入 S3 的全部应用路径。
- 蓝图侧 K2_CommitAbilityCooldown(ForceCooldown)/K2_CommitAbilityCost 可拆开提交（1408-1439 一带）。

## 2.7 RPC 批处理（FServerAbilityRPCBatch）

- 打包**恰好三个调用**：ServerTryActivateAbility + ServerSetReplicatedTargetData + （Ended 时）ServerEndAbility（ServerAbilityRPCBatch_Internal，4194-4208；FakeActivationInfo 注释自认"bogus for the general case"）。
- 客户端入队：CallServerTryActivateAbility/CallServerSetReplicatedTargetData/CallServerEndAbility 在批处理窗口内改写 LocalServerAbilityRPCBatchData（4254-4334）；窗口外直接发 RPC。乱序防御：批窗口内没见过 TryActivate 却来了 TargetData/End → 放行为独立 RPC（4287-4294、4319-4326）。
- 服务器展开顺序 = 激活 → 目标数据 → 结束，全部共享同一个 PredictionKey。

## 2.8 意外发现

1. `static FGameplayTagContainer DummyContainer`（GameplayAbility.cpp:469）——函数级 static 的哑容器残留。
2. `InternalTryActivateAbilityFailureTags` 是 ASC 成员而非局部变量——失败原因跨层传递靠成员状态。
3. `AbilitySystem.SetActivationInfoMultipleTimes` CVar（默认 false，AbilitySystemComponent_Abilities.cpp:50 一带注册）守着一段"cautiously remove this code"的兼容逻辑（1976-1980）。
4. `CVarDenyClientActivation`（1482-1489，非 Shipping）是官方的误预测测试工具——服务器拒绝接下来 N 次客户端激活。

## 2.9 对目标环境的迁移含义

目标状态机 `Requested → Activated → Executing → Completed(+Rejected/Cancelled/Expired/RolledBack)` 与 GAS 的 ActivationMode（Authority/NonAuthority/Predicting/Confirmed/Rejected，GameplayAbilitySpec.h:25-45）是**同构但不同粒度**：GAS 的五态只描述"激活的预测状态"，不描述执行进度（执行进度 = 实例存活 + ActiveCount + task 挂起态）。迁移时：把 ActivationMode 映射为 Rejected/Confirmed 的判定位，把 Executing 拆出来自己建（GAS 没有这个概念可借）。CanActivateAbility 的**固定 9 步顺序与失败 tag 顺序**是现成的可照搬资产——顺序即失败语义，这点对错误提示与重试逻辑同样关键；Commit 的"重查但只查冷却+消耗"的解耦设计也值得保留。
