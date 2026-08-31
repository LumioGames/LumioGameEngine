# 术语对照表

| 英文 | 本报告用语 | 精确定义/注意 |
|---|---|---|
| Ability System Component (ASC) | 能力系统组件 | UE GAS 的状态宿主与网络协调中心；迁移时不等于必须有同名对象 |
| Gameplay Ability | Ability / 能力 | 产生、等待、提交和结束 Gameplay 操作的定义/实例 |
| Gameplay Effect (GE) | GameplayEffect / Effect | 声明持续期、Modifier、Tag、Stack、Cue 等的定义 |
| Gameplay Effect Spec | Effect Spec / 规格 | 定义在某次应用中的具体化：等级、Context、捕获、动态数值等 |
| Active Gameplay Effect | 活跃 Effect 实例 | 目标 ASC 上长期存在的 Duration/Infinite 实例 |
| Gameplay Effect Context | Effect Context / 上下文 | Instigator、causer、source object、hit 等可选来源信息 |
| AttributeSet | 属性集 | UE UObject 字段集合；目标引擎映射为 ECS 属性组件/表 |
| BaseValue | 基础值 | 不含临时 Modifier 的长期基线 |
| CurrentValue | 当前值 | Base 经合格活跃 Modifier 聚合后的值 |
| Meta Attribute | 中转属性 | Damage/Healing 等短期计算意图，不是长期资源 |
| Aggregator | 聚合器 | 维护 Base、Modifier、qualifier/channel 与重算依赖的结构 |
| GameplayTag | GameplayTag / 状态标签 | 集中注册、可层级匹配的 schema 标识，不是任意字符串 |
| Tag Count | Tag 计数 | 多来源贡献的引用计数；0/非0决定是否拥有 Tag |
| GameplayCue | 表现 Cue | 逻辑状态/事件到音画表现的解耦标识与参数 |
| AbilityTask | Ability 异步任务 | UE UObject latent 节点；目标迁移为可序列化 WaitRecord |
| PredictionKey | 预测键 | 连接内短期因果、去重、catch-up/reject 标识；不是全局实例 ID |
| Reconciliation | 权威收敛 | 乐观本地状态被权威状态确认、覆盖或选择性撤销 |
| Rollback / Resimulation | 回滚重演 | 回到历史快照并重放未确认输入；GAS 默认并不提供全域模型 |
| FastArray | 增量结构数组复制 | 以稳定项标识/replication key发送增删改；不是自动字段级数据库补丁 |
| Full / Mixed / Minimal | GAS 复制模式 | ActiveEffect 细节向 owner/其他观察者的投影政策 |
| Owner Actor | 状态所有者 Actor | 网络所有权和 ASC 持久宿主 |
| Avatar Actor | 执行载体 Actor | 当前身体/动画/位置实体，可随重生或附身改变 |
| Inhibited | 被抑制 | ActiveEffect 仍存在但 ongoing requirements 不满足，贡献暂时撤下 |
| Commit | 提交 | Ability 中确认并应用 cost/cooldown 的业务步骤；不等于全世界 ACID 事务 |
| TargetData | 目标数据 | 客户端/服务器传递目标选择的多态数据；不是自动反作弊证明 |
| Audience Projection | 受众投影 | 从同一权威状态为 owner/private/public/minimal 生成不同网络视图 |
