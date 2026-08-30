# UE GAS 调研来源总表

> 访问日期：2026-08-29。UE 源码只尝试委托方指定的 `Go1c/UnrealEngine`；没有 clone、Download ZIP 或切换到其他 UE 源码镜像。社区 GitHub 引用使用访问时分支坐标，正文一律标为 `Reported`。

| 编号 | 类型（官方文档｜源码｜规范｜论文｜社区） | 标题 | 定位（URL 或 owner/repo@ref:路径#L行号） | 实际访问状态（读到全文｜只读到摘要｜打不开） | 支撑章节 |
|---|---|---|---|---|---|
| S000 | 规范 | 本次调研任务书与目标引擎画像 | 会话附件：Pasted markdown.md | 读到全文 | 全篇、R |
| S001 | 源码 | Go1c/UnrealEngine 仓库根页 | https://github.com/Go1c/UnrealEngine | 打不开：抓取 Cache miss | 可达性、全篇置信度 |
| S002 | 源码 | Go1c/UnrealEngine GameplayAbilities 目录尝试 | https://github.com/Go1c/UnrealEngine/tree/5.6/Engine/Plugins/Runtime/GameplayAbilities | 打不开：ref/目录均无法确认 | 可达性、全篇置信度 |
| S003 | 源码 | Go1c/UnrealEngine 仓内搜索 UAbilitySystemComponent | https://github.com/search?q=repo%3AGo1c%2FUnrealEngine+UAbilitySystemComponent&type=code | 打不开：抓取 Cache miss | 可达性、全篇置信度 |
| S004 | 官方文档 | Gameplay Ability System（UE 5.6 主题入口） | https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-ability-system-for-unreal-engine?application_version=5.6 | 读到全文 | A、B、M、O |
| S005 | 官方文档 | Understanding the Unreal Engine Gameplay Ability System（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/understanding-the-unreal-engine-gameplay-ability-system?application_version=5.6 | 读到全文 | A、B、D、E、H、I、J、Q、R |
| S006 | 官方文档 | Using Gameplay Abilities in Unreal Engine（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/using-gameplay-abilities-in-unreal-engine?application_version=5.6 | 读到全文 | C、D、H、I、K |
| S007 | 官方文档 | Gameplay Attributes and Attribute Sets（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-attributes-and-attribute-sets-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6 | 读到全文 | E、F、H |
| S008 | 官方文档 | Gameplay Effects for the Gameplay Ability System（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-effects-for-the-gameplay-ability-system-in-unreal-engine?application_version=5.6 | 读到全文 | E、F、J、M |
| S009 | 官方文档 | Using Gameplay Tags in Unreal Engine（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/using-gameplay-tags-in-unreal-engine?application_version=5.6 | 读到全文 | D、E、G、M |
| S010 | 官方样例文档 | Abilities in Lyra（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/abilities-in-lyra-in-unreal-engine?application_version=5.6 | 读到全文 | A、B、D、H、J、O、R |
| S011 | 官方样例文档 | Lyra Sample Game（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/lyra-sample-game-in-unreal-engine?application_version=5.6 | 读到全文；工程本体未取得 | A、O、P |
| S012 | 官方样例文档 | Upgrading the Lyra Starter Game to the Latest Engine Release | https://dev.epicgames.com/documentation/en-us/unreal-engine/upgrading-the-lyra-starter-game-to-the-latest-engine-release-in-unreal-engine | 读到全文 | H、O、P |
| S013 | 官方文档 | Unreal Engine 5.5 Release Notes—Gameplay Ability System | https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-5-5-release-notes | 读到全文 | C、D、H、M、P |
| S014 | 官方样例文档 | Gameplay Abilities in Action RPG（Legacy） | https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-abilities-in-action-rpg?application_version=4.27 | 只读到摘要；工程本体未取得 | A、D、O、P |
| S015 | 官方 API | GameplayAbilities Plugin Index（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/PluginIndex/GameplayAbilities?application_version=5.6 | 读到索引 | A、B、P |
| S016 | 官方 API | UAbilitySystemComponent（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/UAbilitySystemComponent?application_version=5.6 | 读到成员索引 | B、C、D、H、I |
| S017 | 官方 API | FGameplayAbilitySpec（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayAbilitySpec?application_version=5.6 | 读到成员索引 | B、C、D、H |
| S018 | 官方 API | FGameplayAbilitySpecHandle（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayAbilitySpecHandle?application_version=5.6 | 读到成员索引 | C |
| S019 | 官方 API | FActiveGameplayEffectHandle（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FActiveGameplayEffectHandle?application_version=5.6 | 读到成员索引 | C、E、H |
| S020 | 官方 API | FGameplayEffectSpec（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayEffectSpec?application_version=5.6 | 读到成员索引 | B、C、E、H、L |
| S021 | 官方 API | FGameplayEffectContextHandle（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayEffectContextHandle?application_version=5.6 | 读到成员索引 | B、C、E、H |
| S022 | 官方 API | FGameplayEffectAttributeCaptureDefinition（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayEffectAttributeCaptureDefinition?application_version=5.6 | 读到页面/摘要 | E、L |
| S023 | 官方 API | FGameplayEffectCustomExecutionParameters（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayEffectCustomExecutionParameters?application_version=5.6 | 读到成员索引 | E、I |
| S024 | 官方 API | FAggregator（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FAggregator?application_version=5.6 | 读到成员索引 | E、F、L、N |
| S025 | 官方 API | FActiveGameplayEffectsContainer（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FActiveGameplayEffectsContainer?application_version=5.6 | 读到成员索引 | E、F、H、I、L、N |
| S026 | 官方 API | FActiveGameplayEffect（UE 5.6，含 inhibited 状态字段） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FActiveGameplayEffect?application_version=5.6 | 读到成员索引 | E、H、L、R |
| S027 | 官方 API | FPredictionKey（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FPredictionKey?application_version=5.6 | 读到成员索引 | C、H、I |
| S028 | 官方 API | FPredictionKeyDelegates（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FPredictionKeyDelegates?application_version=5.6 | 读到成员索引 | I |
| S029 | 官方 API | FReplicatedPredictionKeyMap（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FReplicatedPredictionKeyMap?application_version=5.6 | 读到成员索引 | H、I |
| S030 | 官方 API | FReplicatedPredictionKeyMap::KeyRingBufferSize（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FReplicatedPredictionKeyMap/KeyRingBufferSize?application_version=5.6 | 读到全文 | C、I |
| S031 | 官方 API | FGameplayAbilityTargetDataHandle（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/Abilities/FGameplayAbilityTargetDataHandle?application_version=5.6 | 读到成员索引 | B、H、K |
| S032 | 官方 API | FGameplayAbilityRepAnimMontage::NetSerialize（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/Abilities/FGameplayAbilityRepAnimMontage/NetSerialize?application_version=5.6 | 读到页面 | B、H、I、J、K |
| S033 | 官方 API | UAbilityTask_PlayMontageAndWait（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/Abilities/Tasks/UAbilityTask_PlayMontageAndWait?application_version=5.6 | 读到成员索引 | D、I、K |
| S034 | 官方 API | UGameplayTask::EndTask（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/GameplayTasks/UGameplayTask/EndTask?application_version=5.6 | 读到页面 | K、L |
| S035 | 官方 API | FFastArraySerializer（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/NetCore/FFastArraySerializer?application_version=5.6 | 读到成员索引 | H |
| S036 | 官方 API | FFastArraySerializerItem（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/NetCore/FFastArraySerializerItem?application_version=5.6 | 读到成员索引 | H |
| S037 | 官方 API | FFastArraySerializer::FastArrayDeltaSerialize（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/NetCore/FFastArraySerializer/FastArrayDeltaSerialize?application_version=5.6 | 读到页面 | H |
| S038 | 官方 API | Iris FastArray replication fragment helper | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/IrisCore/FFastArrayReplicationFragmentHelper | 读到摘要 | H、P |
| S039 | 官方 API | FGameplayTag（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/GameplayTags/FGameplayTag?application_version=5.6 | 读到成员索引 | G、H |
| S040 | 官方 API | FGameplayTagQuery（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/GameplayTags/FGameplayTagQuery?application_version=5.6 | 读到全文 | G |
| S041 | 官方 API | UGameplayTagsManager（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/GameplayTags/UGameplayTagsManager?application_version=5.6 | 读到成员索引 | G、H、P |
| S042 | 官方 API | FOnGameplayEffectTagCountChanged（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FOnGameplayEffectTagCountChanged?application_version=5.6 | 读到页面 | G、H |
| S043 | 官方文档 | Understanding Networked Movement in CharacterMovementComponent | https://dev.epicgames.com/documentation/en-us/unreal-engine/understanding-networked-movement-in-the-character-movement-component-for-unreal-engine | 读到全文 | I、Q |
| S044 | 官方 API | Network Prediction model definition（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/NetworkPrediction/FNetworkPredictionModelDef?application_version=5.6 | 读到成员索引 | I、Q |
| S045 | 官方 API | FNetworkPredictionDriverBase::FinalizeFrame（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/NetworkPrediction/FNetworkPredictionDriverBase/FinalizeFrame?application_version=5.6 | 读到页面 | I |
| S046 | 官方 API | PhysicsReplicationResimulationSettings（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/PhysicsReplicationResimulationSettings?application_version=5.6 | 读到全文 | I、L、Q |
| S047 | 官方文档 | Overview of Mass Gameplay | https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-gameplay-in-unreal-engine | 读到全文；未发现一等 GAS 权威存储桥 | F、N、Q、R |
| S048 | 官方文档 | Enhanced Input in Unreal Engine（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine?application_version=5.6 | 读到全文 | B、D、O、P |
| S049 | 官方文档 | Gameplay Debugger in Unreal Engine（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/using-the-gameplay-debugger-in-unreal-engine?application_version=5.6 | 读到全文 | M |
| S050 | 官方文档 | Replay System in Unreal Engine（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/replay-system-in-unreal-engine?application_version=5.6 | 读到主题页/摘要 | L、P |
| S051 | 社区 | tranek/GASDocumentation：定位、商业项目与版本声明 | tranek/GASDocumentation@master:README.md#L170-L195（https://github.com/tranek/GASDocumentation/blob/master/README.md#L170-L195） | 读到全文；自述主线约 UE 5.3 | A、B、I、O、Q |
| S052 | 社区 | tranek/GASDocumentation：ASC Owner/Avatar 与复制模式 | tranek/GASDocumentation@master:README.md#L279-L320（https://github.com/tranek/GASDocumentation/blob/master/README.md#L279-L320） | 读到全文 | B、H、O |
| S053 | 社区 | tranek/GASDocumentation：Base/Current 与 Meta Attribute | tranek/GASDocumentation@master:README.md#L455-L500（https://github.com/tranek/GASDocumentation/blob/master/README.md#L455-L500） | 读到全文 | E、F、R |
| S054 | 社区 | tranek/GASDocumentation：Modifier 聚合公式与顺序 | tranek/GASDocumentation@master:README.md#L790-L830（https://github.com/tranek/GASDocumentation/blob/master/README.md#L790-L830） | 读到全文 | E、L、R |
| S055 | 社区 | tranek/GASDocumentation：Stacking | tranek/GASDocumentation@master:README.md#L920-L955（https://github.com/tranek/GASDocumentation/blob/master/README.md#L920-L955） | 读到全文 | E、R |
| S056 | 社区 | tranek/GASDocumentation：AbilityTask 生命周期 | tranek/GASDocumentation@master:README.md#L1955-L1995（https://github.com/tranek/GASDocumentation/blob/master/README.md#L1955-L1995） | 读到全文 | K、L、N |
| S057 | 社区 | tranek/GASDocumentation：GameplayCue 可靠性与 late join | tranek/GASDocumentation@master:README.md#L2185-L2220（https://github.com/tranek/GASDocumentation/blob/master/README.md#L2185-L2220） | 读到全文 | H、J |
| S058 | 社区 | tranek/GASDocumentation：不预测清单与 cooldown 限制 | tranek/GASDocumentation@master:README.md#L2225-L2265（https://github.com/tranek/GASDocumentation/blob/master/README.md#L2225-L2265） | 读到全文 | I、O、Q、R |
| S059 | 社区 | tranek/GASDocumentation：Prediction Key 与 Scoped Prediction Window | tranek/GASDocumentation@master:README.md#L2260-L2310（https://github.com/tranek/GASDocumentation/blob/master/README.md#L2260-L2310） | 读到全文 | H、I、R |
| S060 | 社区 | tranek/GASDocumentation：RPC batching 与 replication proxy | tranek/GASDocumentation@master:README.md#L2685-L2735（https://github.com/tranek/GASDocumentation/blob/master/README.md#L2685-L2735） | 读到全文 | H、M、N、O |
| S061 | 社区 | tranek/GASShooter 示例 | tranek/GASShooter@master:README.md（https://github.com/tranek/GASShooter/blob/master/README.md） | 读到全文；明确非生产就绪且版本较旧 | D、H、I、O、Q |
| S062 | 社区/演讲 | People Can Fly：How We Used GAS in Outriders: Worldslayer（Unreal Fest 2024） | https://dev.epicgames.com/community/learning/talks-and-demos/EPyd/unreal-engine-how-we-used-the-gameplay-ability-system-in-ue-for-outriders-worldslayer-unreal-fest-2024 | 读到摘要 | A、M、N、O、Q |
| S063 | 社区/演讲 | Bokeh Game Studio：Slitterhead 中 GAS 的动作实现（Unreal Fest 2024） | https://www.docswell.com/s/EpicGamesJapan/59VRN7-UE_UF24T_Bokeh | 读到公开演示材料 | A、M、N、O、Q |
| S064 | 社区报道 | GameMakers：Slitterhead GAS 实践报道 | https://gamemakers.jp/article/2025_01_31_91555/ | 读到全文 | M、N、O、Q |
| S065 | 官方文档 | Mover：rollback networking（横向对照） | https://dev.epicgames.com/documentation/en-us/unreal-engine/mover-in-unreal-engine | 读到全文；非 GAS 基线 | I、Q |
| S066 | 官方 API | UGameplayCueNotify_Static（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/UGameplayCueNotify_Static?application_version=5.6 | 读到成员索引 | J |
| S067 | 官方 API | AGameplayCueNotify_Actor（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/AGameplayCueNotify_Actor?application_version=5.6 | 读到成员索引 | J |
| S068 | 官方 API | FGameplayTargetDataFilterHandle（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/Abilities/FGameplayTargetDataFilterHandle?application_version=5.6 | 读到页面 | H、K |
| S069 | 官方 API | FNetSerializeScriptStructCache（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FNetSerializeScriptStructCache?application_version=5.6 | 读到成员索引 | H、K、L |
| S070 | 官方 API | FActiveGameplayEffectsContainer::HasApplicationImmunityToSpec（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FActiveGameplayEffectsContainer/HasApplicationImmunityToSpec?application_version=5.6 | 读到页面 | E |
| S071 | 官方 API | FGameplayEffectSpec::GetPeriod（UE 5.6） | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/GameplayAbilities/FGameplayEffectSpec/GetPeriod?application_version=5.6 | 读到页面 | E、L |

## 计数与证据约束

- 总条目：72。
- 官方文档 / 官方 API / 官方样例：54。
- 指定 UE 源码镜像尝试：3 条，全部不可达；因此没有任何基于 `Go1c/UnrealEngine` permalink 的源码级 `Verified` 断言。
- 社区 / 演讲 / 报道：14；全部与官方资料区分并按版本降级。
- 论文：0。GAS 的关键机制主要由引擎文档、API 参考、样例、演讲与工程实践承载；本次未找到直接解释 GAS 内核的同行评审论文。

## 使用规则

1. 正文 `[Sxxx]` 回指本表。`Verified` 可以来自官方文档/API 明文；但涉及函数体控制流、字段更新先后、容器稳定排序等源码实现时，因指定镜像不可达，统一降为 `Reported` 或 `Estimated`。
2. 搜索摘要只用于定位；未展开的内容不提升为 `Verified`。
3. 社区资料可补官方文档空白，但不与官方结论混写；版本不明或早于 5.6 的行为会显式标版本。
4. 源码访问现象与重试记录见 `appendix/source-access-log.md`。