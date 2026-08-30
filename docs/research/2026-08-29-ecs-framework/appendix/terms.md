# 术语表

| 术语 | 本报告定义 |
|---|---|
| Active Component | 持有状态并可执行局部行为/生命周期方法的组件对象；不同于只承载数据的 POD component。 |
| Archetype | 由组件类型集合定义的实体存储组；相同集合实体通常按 chunk/column 紧凑存放。 |
| Baseline | 客户端解码后续 Delta 所依赖的完整或已确认状态版本。 |
| Canonical order | 与物理内存排列无关、跨平台固定的实体/组件/字段/命令顺序。 |
| Component revision | 某组件已提交状态版本，用于筛选、去重、回调和诊断。 |
| Dirty mask | 标识自某基线后哪些字段可能变化的位集合。 |
| Dormancy | 保留实体/副本但暂停或降低复制/模拟的状态；不是 Destroy。 |
| EntityTypeDescriptor | 预编译的实体类型元数据：组件集合、默认值、依赖、Schema、生命周期派发表。 |
| Interest/AOI | 某观察者是否有权/需要接收某实体或组件状态的动态关系。 |
| InterestEpoch | 同一观察者与实体每次新的兴趣生命期编号，用于隔离旧包。 |
| LocalEntityHandle | 仅在一个 World 内有效的 Index + Generation 句柄。 |
| NetworkEntityId | 跨端协议身份；不得等同于本地数组下标。 |
| Presentation Residency | 模型、动画、粒子、音频等客户端表现资源是否加载/显示。 |
| Replica Residency | 客户端是否保留网络逻辑副本；与表现驻留分开。 |
| Replication Transaction | 一组实体/组件字段先在 staging 完整应用、校验与解析，再原子发布并回调的事务。 |
| Schema manifest | 所有稳定 type/field ID、版本、量化、可见性和 serializer 的可比较清单/hash。 |
| Soft leave | 失去网络兴趣后暂存副本，等待短期重进或 TTL 淘汰。 |
| Sparse Set | 每组件类型维护 sparse index 与紧凑 dense arrays 的存储结构。 |
| Structural change | 创建/销毁实体、增删组件、改变影响存储/查询结构的操作。 |
| Tombstone | 已销毁身份的记录，用于拒绝迟到包、旧引用和 ID 复用。 |
| World | 实体所有权、句柄有效性、系统调度、时间线和状态事务的边界。 |
