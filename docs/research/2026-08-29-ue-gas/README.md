# UE GAS 深度调研交付包

- **包名**：`ue-gas-research-2026-08-29`
- **调研对象**：Unreal Engine Gameplay Ability System（GAS）
- **基线**：UE 5.6 官方公开文档/API；UE 5.5 与旧样例仅用于版本对照
- **重点**：H 数据同步、I 客户端预测与权威收敛、R 可迁移性结论
- **目标环境**：Rust + C#、ECS 唯一权威状态、确定性 Tick、单提交点、整帧回滚与状态哈希

## 本包是什么

本包是一次纯外部、证据驱动的 GAS 架构调研。它不访问委托方代码库，不改写目标引擎设计；只回答 UE/GAS 如何做、设计背后的坑、跨品类实践、网络复制与预测边界，以及这些机制在目标 ECS/确定性引擎中哪些可继承、哪些必须重写。任务范围与验收要求来自会话附件 `Pasted markdown.md`。

## 建议阅读顺序

1. 先读本 README 的执行摘要。
2. 直接读主报告 **H** 与 **I**，掌握同步和“不是回滚重演”的关键结论。
3. 读 **E/F/G**，理解 Effect、Attribute、Tag 的设计逻辑。
4. 读 **O/P/Q**，判断品类、版本和限制。
5. 最后读 **R**，用于目标架构取舍。

## 文件清单

- `report/ue-gas-research-2026-08-29.md`：A–R 十八章主报告。
- `sources.md`：来源总表，逐条记录类型、定位、访问状态和支撑章节。
- `appendix/state-machine-crosswalk.csv`：UE 隐式/显式状态与目标状态机映射。
- `appendix/replication-matrix.csv`：同步对象、方向、受众、粒度和目标映射。
- `appendix/source-access-log.md`：指定源码镜像、官方文档与样例可达性。
- `appendix/terminology.md`：术语对照。
- `appendix/mermaid-diagrams.md`：关键图表源码。
- `appendix/research-self-check.md`：交付前机械检查与人工验收结果。

## 完整章节索引

- **A**：起源、野心与设计哲学
- **B**：整体架构与对象图
- **C**：标识、句柄与实例模型
- **D**：Ability 生命周期
- **E**：GameplayEffect：数值系统的核心
- **F**：Attribute 系统
- **G**：GameplayTag 系统
- **H**：数据同步 / 网络复制
- **I**：客户端预测与状态回滚 / 收敛
- **J**：GameplayCue：表现层解耦
- **K**：AbilityTask 与异步执行
- **L**：确定性、快照与存档
- **M**：数据驱动、工具链与工程化
- **N**：性能与规模
- **O**：跨品类的实际应用形态
- **P**：版本演进
- **Q**：批评、限制与替代方案
- **R**：精髓提炼与可迁移性评估

## 执行摘要全文

1. **GAS 不是“技能库”，而是以 ASC 为中心的局部 Gameplay 状态与副作用协调器。** [Verified] `UAbilitySystemComponent` 统一承载已授予 Ability、活跃 GameplayEffect、Attribute、GameplayTag、GameplayCue、蒙太奇复制与 PredictionKey 协议。它的野心是让“规则、持续状态、表现、网络”共享同一套语义，而不是让每个技能自己发 RPC、扣资源、挂 Timer、管理 Buff。[S004][S005][S016]
2. **最值得继承的设计世界观是“定义、实例、上下文、聚合、表现分离”。** [Verified] `GameplayEffect` 定义与 `GameplayEffectSpec` 运行时规格分开，Effect 与 Context 分开，Attribute 拆 Base/Current，Owner 与 Avatar 分开，逻辑状态与 Cue 分开。每层都对应一个具体坑：共享资产被运行时污染、来源/目标混淆、Buff 移除后无法重算、复活换 Pawn 后状态丢失、逻辑依赖特效对象等。[S005][S007][S008]
3. **GAS 的网络同步是“耐久状态 + 瞬时事件 + 本地推导”的混合模型。** [Verified/Reported] Attribute、ActiveEffect、AbilitySpec 等走属性/FastArray 状态复制；激活、TargetData、通用事件和结束走 RPC；当前数值、Cue 与部分显示由本地 Effect/Tag 状态重建或预测。Full/Mixed/Minimal 不是三种算法，而是同一权威状态向不同观察者投影的三种带宽策略。[S005][S035]–[S038][S052]
4. **FastArray 的核心价值是“数组项级脏标记与增删改回调”，不是自动字段级补丁。** [Verified] `FFastArraySerializer` / `Item` 提供 item identity、replication key、add/change/remove 回调与增量发送；某项被标脏后，该项的 NetSerialize 负载仍可能整体重发。迁移到 ECS 时应继承“稳定元素 ID + 版本 + tombstone/变更回调”，不应照搬 UObject 容器。[S035]–[S038]
5. **GAS Prediction 不是确定性 rollback/resimulation。** [Verified/Reported] 客户端在 Prediction Window 内乐观激活并注册可撤销副作用；服务器接收同一个 PredictionKey 后接受、catch-up 或拒绝。系统不保留完整世界帧，也不把历史输入重放到当前帧，因此它属于“乐观应用 + 权威覆盖收敛”，不是 CharacterMovement/Network Prediction/Chaos 意义上的回滚重演。[S027]–[S030][S043]–[S046][S059]
6. **GAS 语境中的“rollback”是选择性撤销，不是事务回滚。** [Reported] 可绑定到 PredictionKey 的预测 Effect、Tag/Attribute 变化、Cue 等可以在拒绝时移除或补偿；已经触发的外部世界写入、任意 Blueprint 副作用、一次性音画、非 GAS 连锁逻辑，通常没有统一逆操作。目标引擎要求 Ability/ECS/体素覆盖层整帧同进退，必须另建帧日志、快照、确定性重演与提交协议。[S028][S059]
7. **Prediction Window 是逻辑作用域，不是 N 毫秒或 N 帧历史。** [Verified/Reported] Key 在能力激活/RPC 作用域产生，并由服务器复制回客户端触发 catch-up；依赖 Key 表达子动作因果。`KeyRingBufferSize = 32` 是复制确认环形槽容量，不等于“能回滚 32 帧”。[S027]–[S030][S059]
8. **Lyra 的附加层精确暴露了裸 GAS 的产品化缺口。** [Verified] AbilitySet 解决批量授予与撤销；PlayerState ASC 初始化封装解决 Owner/Avatar 时序；InputTag 解决 Enhanced Input 到 Ability 的映射；Tag Relationship Mapping 解决分散互斥规则；ActivationPolicy/Group 解决被动、自发与并发组；AdditionalCost 解决多重成本；GamePhase Ability 解决全局阶段生命周期。[S010]–[S012]
9. **对 ECS + 确定性 Tick，引进概念而不是对象图。** [Estimated] 应保留 Definition/Spec/Instance、Effect 句柄、Tag 计数、捕获规则、聚合通道、受众投影、PredictionKey 因果标识和 Cue 边界；应删除 UObject/GC、Actor Owner 链、Blueprint latent task、WorldTimer 与“ASC 再存一份权威状态”。所有 Effect/Attribute/Tag 实例状态必须投影到 ECS，并由一个提交序列产生复制、快照和哈希。
10. **最关键的不可迁移部分，是 GAS 对“非确定性但可收敛”的接受。** [Estimated] 浮点聚合、Timer/World Time、回调驱动重算、资产类对象与网络到达顺序，适合 UE 的服务器权威实时游戏，却天然不提供 canonical state hash。目标引擎可以继承它的因果和可逆副作用标注，但必须自己定义稳定排序、定点/受控浮点、帧号时间、全域事务日志与 resimulation。

## 已知缺口

- **指定 UE 源码镜像不可读**：无法给出任何 `Go1c/UnrealEngine@commit:path#Lx-Ly` permalink，源码级断言全部降级。
- **历史来源链证据不足**：GAS 与 UE3/Gears、Paragon、Fortnite 的口述演进可见于社区与演讲，但未找到一份 Epic 官方、逐版本可核的完整谱系。
- **Fortnite 内部实现不可见**：只能引用 Epic 对 GAS 定位、Lyra 公开实践与社区对 Fortnite batching/replication proxy 的记述，不能声称掌握生产分支细节。
- **官方样例工程本体未取得**：Lyra/ActionRPG 以公开文档为证；具体项目路径只写官方文档明示或社区明确列出的部分。
- **缺少公开可复现实测**：没有可信、同硬件同场景的“每 ASC/每 ActiveEffect 字节数”“N 玩家+M AI TPS/带宽”基准，不编造数字。
- **完整存档/快照缺口**：未找到 Epic 官方定义的“活跃 AbilityTask + Timer + PredictionKey + ActiveEffect 全量可恢复快照格式”。
- **Mass ↔ GAS 一等集成缺口**：找到 Mass Gameplay 的 ECS 定位，但未找到 Epic 官方把 Mass Fragment 设为 GAS Attribute/Effect 唯一权威存储的方案。

## 证据阅读说明

报告内 `[Sxxx]` 回指 `sources.md`。`Verified` 只用于亲自读到的 Epic 官方文档/API 明文或实际访问现象；指定源码镜像不可达，因此涉及源码函数体的控制流、时序和位布局没有标为源码级 Verified。`Reported` 是明确标注版本的社区/演讲实践，`Estimated` 是架构推断。

## 关键结论导航

- 数据同步：主报告 H.1–H.14；附录 `replication-matrix.csv`。
- 预测/回滚：主报告 I.1–I.15；重点看 I.6、I.7、I.11、I.13。
- 状态机：主报告 D.6、R.4；附录 `state-machine-crosswalk.csv`。
- ECS 迁移：主报告 F.6、Q.2、R.2–R.7。
- 不可照搬：主报告 B.4、R.5。