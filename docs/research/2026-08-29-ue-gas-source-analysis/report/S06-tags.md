# S6 · GameplayTag 的内部表示与网络序列化

> 结论先行
> 1. Tag 的运行时表示是**单个 FName**（`FGameplayTag::TagName`，GameplayTagContainer.h 的类定义区），层级关系不在字符串里展开，而是查 `UGameplayTagsManager` 的节点树；容器 `FGameplayTagContainer` = `TArray<FGameplayTag> GameplayTags`（**显式 tag 列表**）+ 运行时补齐的父 tag（FillParentTags）。
> 2. 网络位布局：**每个 tag = [第一段 N bit][1 bit more][第二段 MaxBits−N bit]** 的两段变长 NetIndex（SerializeTagNetIndexPacked，GameplayTagContainer.cpp:69-126）；N = `NetIndexFirstBitSegment`（ini 可配），MaxBits = `NetIndexTrueBitNum = ceil(log2(标签数+1))`；容器先写 1 bit 空标志 + 定长计数字段。
> 3. **表一致性没有运行时校验**：两端索引表按「tag 名排序 + CommonlyReplicatedTags 前置」构建（ConstructNetIndex），CRC32 全表哈希**被计算、被日志、但全引擎无消费者**（grep 仅命中定义）；表错位时的行为 = 索引越界 ensure + 静默返回 NAME_None，**索引在界内但映射错位 = 静默错认 tag，不断线**。

## 6.1 待证清单裁决表

| # | 待证项 | 裁决 | 证据 |
|---|---|---|---|
| 6.1 | 内部表示 | **FName 单值**（运行时）；网络身份是 uint16 NetIndex（FGameplayTagNetIndex） | FGameplayTag 的 TagName 字段（GameplayTagContainer.h，容器 NetSerialize 直接操作 GameplayTags 数组，GameplayTagContainer.cpp:1066-1123）；GetNetIndexFromTag/GetTagNameFromNetIndex（GameplayTagsManager.cpp:839-864） |
| 6.2 | 位布局 | 见 6.2 节；InvalidTagNetIndex = 表长+1（哨兵） | GameplayTagContainer.cpp:69-126 · SerializeTagNetIndexPacked；GameplayTagsManager.cpp:810 · InvalidTagNetIndex 赋值 |
| 6.3 | 表构建时机与排序 | **启动/标签树变更后** ConstructNetIndex：节点值数组 → **按 tag 名排序**（FCompareFGameplayTagNodeByTag）→ CommonlyReplicatedTags 交换到头部 → 逐个赋 NetIndex；注释明言"it should be the same on both client and server"（靠**同配置同字典**保证，非协议保证） | GameplayTagsManager.cpp:767-837 · ConstructNetIndex（排序 777，common 前置 782-800，"same on both"注释 802，逐项赋号+累计 CRC 820-834） |
| 6.4 | 表不一致的实际行为 | **不校验、不断线**：越界 → ensureMsgf("Tag index is out of sync on client!") + 返回 NAME_None；界内错位 → 静默错认。CRC32 哈希（NetworkGameplayTagNodeIndexHash，826）只在日志打印，`GetNetworkGameplayTagNodeIndexHash()` 全引擎 0 调用 | GameplayTagsManager.cpp:843-848 · GetTagNameFromNetIndex 的越界路径；:634 哈希 getter 无消费者（Engine/Source 全树 grep 仅 1 处=定义） |
| 6.5 | 层级匹配实现与代价 | MatchesTag = 走节点树向上比对（含父链）；容器级 HasAny/HasAll 遍历显式 tag 数组 × 父展开。`HasAll` 失败时还能报缺失集 | FQueryEvaluator 与 MatchesTag 系（GameplayTagContainer.cpp 的匹配区；DoesAbilitySatisfyTagRequirements:398-400 演示 RemoveTags(GetGameplayTagParents()) 的缺失集计算） |
| 6.6 | CountContainer 为什么是 count | `FGameplayTagCountContainer`：`TMap<FGameplayTag, FGameplayTagCountItem>`，每项显式计数 + 父聚合（ParentTags 计数）；**加减点**：AddLooseGameplayTags/RemoveLooseGameplayTags（±1，带 EGameplayTagReplicationState）、GE 授予 tag 在 AddActiveGameplayEffectGrantedTagsAndModifiers(+1)/Remove...(−1)（GameplayEffect.cpp:4663-4664/4971-4972，按 ShouldUseMinimalReplication 选 SimulatedTagOnly）、堆叠变化 NotifyTagMap_StackCountChange（3585-3588） | GameplayEffectTypes.h:1059-1101（FGameplayTagCountItem/FGameplayTagCountContainer 声明）；AbilitySystemComponent 的 UpdateTagMap_Internal（AbilitySystemComponent.h:1895-1896） |
| 6.7 | Loose vs Replicated 两套路径 | **同一容器、不同复制态**：`EGameplayTagReplicationState { None, SimulatedTagOnly, CountToOwner }`（GameplayEffectTypes.h:1049 起）。权威+Minimal/Mixed 模式 → 授予 tag 走 SimulatedTagOnly 进 `MinimalReplicationTags`（COND_SkipOwner）；ActivationOwnedTags 且 ShouldReplicateActivationOwnedTags → CountToOwner 进 `ReplicatedLooseTags`（COND_None）；两者都 UE_DEPRECATED(5.7)——5.8 的新路是**单一 GameplayTagCountContainer 复制属性**（AbilitySystemComponent.h:1858，GetLifetimeReplicatedProps 注册）+ Iris 的 `AbilitySystem.Fix.ReplicateTagCountContainerWithIris`（默认 **false**，GameplayEffectTypes.cpp:47 一带） | AbilitySystemComponent.cpp:1842-1874 · 注册；AbilitySystemComponent.h:1921-1923/1941-1943 · 弃用标注；GameplayEffect.cpp:4669-4670 注释："we do not have working Tag Count replication through GameplayTagCountContainerNetSerializer ... replicate using the legacy path" |

## 6.2 位布局细节（可照抄级）

```
FGameplayTagContainer::NetSerialize (GameplayTagContainer.cpp:1066-1123):
  [1 bit]  IsEmpty（空容器早退）
  [NumBitsForContainerSize bit] 元素个数（默认可容 2^N-1，超限 ensure+截断）
  × N: FGameplayTag::NetSerialize_Packed (1572-1630):
      [1 bit] bUseFastReplication（引擎网版本≥CustomExports 才写；否则 CVar 假设旧回放）
      fast 路径:
        SerializeTagNetIndexPacked:
          值 ≤ 2^N-1: [N bit index][1 bit more=0]
          值 > 2^N-1: [N bit 低位|more=1][MaxBits-N bit 高位]   // 共 MaxBits+1 bit
      非 fast: [1 bit] bUseDynamicReplication → 动态字典路径 / 否则整个 FName 明文
```
- 读取端 `NetIndex → GetTagNameFromNetIndex` 直接查表（1616）。
- 回放特殊路径 `NetSerialize_ForReplayUsingFastReplication`（1336 起）：InvalidTagNetIndex 不能存（远端值可能不同，1364 注释）。
- 非 Shipping 下每次发 tag 都 `NotifyTagReplicated`（1100-1102）喂频率统计——`GameplayTags.PrintReport` 的数据来源，用于人工优化 CommonlyReplicatedTags。

## 6.3 Schema 封闭引擎能否照抄索引压缩？

能，但要把 GAS 的隐含前提变成显式契约（坐标为证）：
1. **表构建是确定性的**（名字排序 + ini 列表前置，GameplayTagsManager.cpp:777/782）——两端一致靠"同 ini + 同字典"，**没有任何握手**。封闭 Schema 引擎应把表内容（或其哈希）放进版本协商：GAS 已算好 CRC32（826）却不用，属于现成半成品。
2. **表可以在运行时失效重建**（bNetworkIndexInvalidated / VerifyNetworkIndex，GameplayTagsManager.h:929）——热重载/插件加载会改表；照抄时必须冻结版本或显式迁移。
3. 位宽自适应（NetIndexTrueBitNum 随标签数取对数，811）+ 两段变长（常用 tag 前置）对「字段集封闭」的线协议是安全优化——因为索引值域 = Schema 常量，可静态求值每个 tag 的编码宽度。
4. **错位是静默数据损坏**（6.4）——目标引擎的 Release 哈希/对账诉求下，这个错误路径必须升级为硬失败。

## 6.4 意外发现

1. `GameplayTags.PrintNetIndices` / `PrintReplicationIndicies` / `PackingTest` / `PrintReplicationFrequencyReport` 一族非 Shipping 命令全部围绕这套压缩（cvar 清单见 S12）。
2. `GOldReplaysUseGameplayTagFastReplication`（GameplayTagContainer.cpp:46-47）：旧回放兼容靠 CVar 猜测编码方式——版本化不彻底的历史包袱。
3. Iris 的 `GameplayTagContainerNetSerializer` 已存在于 GameplayTags/Public（symbol-map 收录），注释自曝慢路径"skips stable sorting for now"并直接反量化比较（GameplayTagContainerNetSerializer.cpp:174-175，告解扫描发现）。

## 6.5 迁移含义

目标引擎「Schema 生成 + 字段集封闭 + 规范化字节」与这套压缩**高度兼容**，条件是把三点写死进协议：① tag 表是 Schema 的一部分（版本内冻结，索引静态分配）；② 编码宽度随表声明（可做编译期常量而非运行时 log2）；③ 表哈希进入握手与回放头（GAS 算了没用的那个 CRC 就是该放的坑位）。Loose/Replicated 的三分复制态（None/SimulatedTagOnly/CountToOwner）是 GAS 对带宽的精细妥协，ECS 权威存储下可简化为「权威全量 + 订阅投影」，不必复刻。
