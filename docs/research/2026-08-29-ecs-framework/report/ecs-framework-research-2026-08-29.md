# 前后端共用、对象组合式 ECS / Entity-Component 框架技术调研

- **报告日期**：2026-08-29
- **研究对象**：公开引擎、开源框架、标准、论文、官方技术文档与一线开发者记述
- **目标画像**：自研 C# Gameplay + Native Kernel；组件可带逻辑；前后端共用 Entity / Component 定义；服务器权威；声明式属性同步；AOI 驱动 `OnEnterScene` / `OnLeaveScene`；单一结构提交点；有界预测、整帧回滚、确定性哈希；Storage 未冻结
- **边界**：不展开技能/效果/Modifier 的内部求值（见 GAS 专项）；不展开网络传输栈、会话与每连接调度实现（见 DS 专项）；本报告只覆盖与 ECS、同步声明、AOI、生命周期、预测和状态一致性直接相交的接口面。

---

## 信息源可达性声明

1. **联网与官方文档**：可联网；Unity、Unreal、Photon、Mirror、FishNet、Colyseus、Flecs、EnTT、KBEngine 等公开文档可打开并检索。`[Verified]` 的文档级结论均能回指 `sources.md`。
2. **GitHub 在线检索**：可打开公开仓库、Release、Issue 与部分带行号 blob；没有 clone 或下载整仓。Mirror 的 dirty bit、hook guard、SyncObject 与 InterestManagement 关键路径读到了带 ref 与行号的源码，标为源码级 `Verified`。其余框架若只读到官方文档，则不冒充源码级验证。
3. **论文**：Scott Bilas 的 GDC PDF 读到全文并抽查页面；Benford–Fahlén、Morse 等 AOI 论文及 IEEE HLA 标准主要读到摘要/元数据或付费入口，因此其学术定义为 `Verified`（摘要级），实现细节不升格为源码级结论。
4. **中文社区**：可访问部分博客园、CSDN、GitHub Issue/Discussion；搜索结果受平台登录、反爬和旧版本影响。中文社区条目单独标为“社区”，不与官方文档混写。
5. **不可达或不充分**：Photon 商业运行时源码不可见；KBEngine 公开版本线与文档存在历史口径漂移；跨 C# ECS 的第三方 benchmark 不能代表目标工作负载；公开 AOI 实测数据通常不具备统一硬件、实体密度、移动率和带宽条件。

## 置信度图例

- **`[Verified]`**：本次实际读到官方文档、规范、论文原文/摘要，或带定位的公开源码/Issue。
- **`[Reported]`**：公开社区或厂商自报，来源可追溯，但未能用独立源码/实测交叉确认。
- **推测：`[Estimated]`**：根据多份证据作出的工程推断；只在事实章中用于明确标注的推测，架构建议集中在 P 章。

## 深挖对象与版本快照

| 对象 | 本次参考版本/日期 | 流派 | 证据层级 | 仓库体检快照 |
|---|---|---|---|---|
| Unity Entities | 1.4.x 文档 | 纯 DOD / Archetype ECS | 官方文档 | 商业引擎包；活跃维护 |
| Unity Netcode for Entities | 1.10.0 文档；局部对照 1.9/当前 API | Snapshot/Ghost 复制 + DOD ECS | 官方文档/官方论坛 | 商业包；活跃维护 |
| Mirror | v96.11.2（2026-08-22）；源码锚点 v96.11.0 `57afb71` | Unity 对象组件 + 自动复制 | 官方文档 + 源码 | 约 6.3k stars；2026-08 活跃；项目方自报大量生产项目 |
| FishNet | 4.7.2R（2026-04-17，`de19b5d`） | Unity 对象组件 + 自动复制/Observer | 官方文档/Release | 约 2.0k stars；2026 持续维护 |
| Photon Fusion | 2.1.2 Stable（2026-08-13） | Unity NetworkObject + Tick Snapshot/Prediction | 官方文档；闭源运行时 | 商业产品；源码级不可核 |
| KBEngine | 公开主仓与历史文档；Release 线长期低频 | MMO EntityDef + base/cell/client 投影 | 官方文档、仓库、Issue | 约 5.7k stars；稳定发布线偏旧；维护状态需分支级复核 |
| Colyseus | 0.18 文档（2026-08） | Authoritative Room + Schema delta | 官方文档/仓库 | 文档与生态 2026 活跃；核心仓库维护中 |
| Flecs | v4.1.5（2026-03-15，`d7d0c4f`）/v4.1 文档 | 关系型 DOD ECS | 官方文档/API | 8k+ stars；2026 活跃；项目方宣称可达百万实体 |
| EnTT | v4.0.0（2026） | Sparse-set DOD ECS | 官方 Wiki/作者文 | 万星量级；2026 活跃；README 列出生产采用者 |
| Friflo.Engine.ECS | v3.6.0 发布线/2026 文档 | 托管 C# Archetype ECS + Script | 官方文档/仓库/benchmark | 百级至千级 stars；持续维护；基准主要由项目方维护 |
| Unreal Actor/Component | UE 5.x 当前文档 | 对象组合 + Actor replication | 官方文档 | 商业引擎；用于语义对照而非选库 |
| ET | 2026-08 主仓 README；社区生命周期资料多为旧版 | C# 前后端框架，当前强调数据组件/扩展系统 | 仓库级 + 社区 | 约 9.9k stars；活跃；分支演进快，细节需锁 commit |

---

# 执行摘要

1. **目标画像不是今天通常所说的“纯 ECS”，而是 Active Component / Object-Composition ECS Hybrid。** `[Verified][S001][S002][S009]` 它保留 `World`、实体句柄、查询、结构提交与确定性顺序等 ECS 机制，却允许组件自带逻辑并彼此协作。这个定位本身没有问题，但性能、并行和可回放能力不会自动从“叫 ECS”获得；它们取决于存储、调用边界和调度是否被单独工程化。
2. **`Awake` 与 `Start` 的分界不是风格偏好，而是“局部构造完成”与“可依赖他者”的屏障。** `[Verified][S006][S007][S008][S054]` Unity 对场景对象先完成 Awake/OnEnable，再运行 Start；Photon 又提供 `Spawned` 与批次后的 `IAfterSpawned`，明确解决同批对象互相访问的顺序问题。目标画像只给钩子名，没有发布屏障、依赖解析和重入规则，仍会出现半初始化引用。
3. **AOI Enter/Leave 不能与 Enable/Disable 合并。** `[Verified][S040][S046][S053][S096]` 主流网络框架把“某连接是否观察某实体”放在 observer/relevancy/interest 集合；运行禁用、网络 dormancy、客户端对象驻留和表现可见性则是其他轴。合并会导致服务器无观察者时误停 AI、客户端暂时隐藏时销毁逻辑状态，或 Host 同时有 server/client World 时出现双重表现残留。
4. **服务器 AOI 与客户端 AOI 不是一个概念。** `[Verified][S029][S053][S073]` 前者是每连接的数据披露与带宽选择；后者是收到副本以后，客户端对逻辑副本、表现、资源和缓存的驻留策略。二者可以使用相同空间信号，却必须是不同状态机和不同事件载荷。
5. **“声明即同步”成熟实现的核心不是 Attribute，而是编译期 Schema 与运行时事务。** `[Verified][S037][S051][S072][S101]` 生成代码要稳定分配 type/field ID、插入写屏障、生成序列化器和权限裁剪；运行时要同时处理 dirty granularity、Baseline、Delta、序号、去重、引用补丁和回调批次。目标画像只有“字段变化即推送”，尚不足以避免半状态回调、字段重排不兼容和 AOI 重入 Baseline 风暴。
6. **网络状态回调应在完整 Baseline/Delta 事务应用后统一派发，而不是反序列化一个字段就立即回调。** `[Verified][S052][S057][S076]` Photon 的初始 spawn 不触发 `OnChangedRender`，并提供 per-object consistency 选项；Colyseus 曾因第 64 字段编码歧义导致客户端状态脱同步。目标框架需要明确“apply → resolve refs → validate → publish revisions → deterministic callbacks”的顺序。
7. **跨实体引用是同步与 AOI 的共同硬问题，画像尚未定义。** `[Verified][S095][S068]` 被引用实体可能尚未进入客户端视野、已经软离开、被预测 ID 重映射，或在同一批次稍后创建。必须使用网络 ID + 可空解析句柄 + generation/revision 校验 + unresolved patch table，禁止将远端引用直接落成 C# 对象引用。
8. **Transform 不能只作为普通组件看待。** `[Verified][S014][S059][S060]` 它同时参与父子拓扑、局部/世界矩阵、物理写回、渲染插值、AOI 位置、网络量化和预测纠正。目标画像需要冻结写权威、父子变更提交顺序、循环检测、脏传播、同步空间（local/world）及父对象不在 AOI 时的降级规则。
9. **Storage 的最稳妥起点是“稳定实体表 + 每类型稀疏/稠密存储 + 热数据专用列/Archetype”的混合，而不是押注单一布局。** `[Verified][S011][S085][S089]` 纯 Archetype 擅长稳定组合的多组件扫描，却把频繁增删变成跨块迁移；Sparse Set 擅长按类型增删与 O(1) 存在判断，却不能保证任意多组件查询都同块。组件带逻辑、托管对象、频繁组合变化与海量创建同时存在时，应让句柄和语义独立于物理布局，并用目标负载 benchmark 决定热点迁移。
10. **最大完整性缺口不是“少一个组件”，而是缺少三份冻结协议：生命周期状态机、复制事务协议、确定性/版本协议。** 推测：`[Estimated，依据 S006–S029、S032–S077、S101–S108]` 如果现在不补，最先出现的事故会是：AOI 抖动重复创建、属性回调看见半状态、旧客户端把字段解码错、预测实体引用丢失、快照恢复重复执行副作用，以及同一帧在不同机器上生成不同哈希。

## 已知缺口清单

- **跨框架统一 benchmark 不存在**：公开 C# ECS 基准明确声明“不代表真实负载”，而且大多没有覆盖组件钩子、网络 Baseline、回滚快照、状态哈希、跨 managed/native 边界。J/P 章因此只给定性倾向与必须补做的实测矩阵。
- **商业源码不可达**：Photon Fusion 与 Unreal/Unity 的核心复制实现不是公开源码；相关内部算法只按官方文档陈述，不作源码级推断。
- **KBEngine 版本口径不统一**：主仓、插件、旧 API 镜像和社区文章跨多年；能够确认 EntityDef/回调思想，但无法将所有细节归到单一现代 commit。
- **ET 分支演进快**：主仓 README 可确认前后端 C#、模块与组件方向；旧社区文章的生命周期类名只作为历史/Reported，不据此给目标 API 定名。
- **AOI 规模数字不可比**：没有找到公开、可复现且同时报告 N 实体、M 观察者、移动率、兴趣半径、硬件、Tick 和带宽的主流框架横向数据。
- **“放弃同一路线”的完整复盘稀缺**：可找到具体 bug、迁移与设计批评，但没有找到一个公开项目完整披露“因 Active Component + 前后端共用而整体改写”的可核复盘；O 章明确保留空白。
- **论文全文限制**：三篇 AOI 经典文献主要依据摘要和元数据；不使用未读正文中的复杂度或实验数字。

---

# A. 谱系、源流与流派分野

**结论先行**  
**一**：Entity-Component 最早流行的工程诉求是摆脱深继承树与单体 GameObject，不等于今天的 Archetype ECS。  
**二**：ECS 含义漂移的分水岭是“组件是否只承载数据、逻辑是否批处理、存储是否围绕查询布局”。  
**三**：目标画像属于中间形态：对象组合是语义核心，ECS 机制负责身份、存储、查询、生命周期、提交和复制。

## A.1 从对象组合到数据导向 ECS

`[Verified][S001]` Scott Bilas 在 2002 年 GDC 的 *A Data-Driven Game Object System* 已把对象身份、创建/销毁、消息路由和按组件管理器组织职责放在同一套 Game Object System 里；其关注点是可组合性、工具与数据驱动，而不是后来以 cache line、Archetype chunk 为中心的定义。

`[Verified][S002]` Mick West 2007 年 *Evolve Your Hierarchy* 直指深继承树的组合爆炸：飞行、受伤、渲染、AI 等横切能力无法通过单继承干净组合，于是将对象拆成组件并由拥有者协调。这里的组件通常是有行为的对象。

`[Verified][S003]` Adam Martin 同年讨论 MMO Entity System 时，将实体看成 ID、组件是按需属性/能力、系统处理组件集合；这条线开始把大规模服务端、数据库、网络和运行时组合放在同一个“Entity System”词汇下。

`[Verified][S004]` Mike Acton 的 Data-Oriented Design 强调从数据与转换出发、围绕访问模式布局数据。随着 Unity DOTS、Flecs、EnTT 等采用结构化存储与批处理，行业口语中的 “ECS” 逐渐默认指“数据组件 + 批量 System + 查询友好存储”。

## A.2 三个流派的分水岭

| 维度 | 纯数据导向 ECS | 对象组合式组件模型 | 中间形态 / Active-Component ECS Hybrid |
|---|---|---|---|
| 组件 | 倾向 POD/值数据，不持有业务行为 | 常为对象，带状态与生命周期方法 | 允许组件逻辑，但受 World/句柄/提交约束 |
| 逻辑位置 | System 批量处理查询结果 | 每对象/组件方法、事件回调 | 热循环可在 System，局部能力留在组件 |
| 存储目标 | Archetype/列/稀疏集，优化扫描 | 对象图、易理解与编辑 | 语义与物理布局解耦，热点另行优化 |
| 主要收益 | cache locality、并行、批量调度 | 组合复用、局部封装、工具直觉 | 共享实体语义，同时保留可管理的对象编程 |
| 放弃 | 自由对象引用、任意虚调用 | 极限扫描性能、自动并行 | 既不能假设对象模型免费，也不能假设 DOD 收益自动存在 |
| 性能上限 | 受带宽、分支、同步点、结构迁移决定 | 受指针追逐、虚调用、GC、事件链决定 | 受“活跃组件比例 × 调用密度 × 存储布局 × 提交开销”共同决定 |

`[Verified][S009][S010]` Unity Entities 把数据组件与执行系统分开，并围绕 World、EntityQuery、chunk 组织访问；`[Verified][S080][S081]` Flecs 则把 ECS 做成带关系、查询与 stage 的实体数据库。两者是纯 DOD 一侧。

`[Verified][S093]` Unreal Actor/ActorComponent 与 Unity MonoBehaviour 位于对象组合侧：组件有方法、事件和引擎生命周期。Mirror、FishNet、Photon 继承了这一编程体验，再叠加网络身份和声明式复制。

推测：`[Estimated]` 目标画像最接近“Active Component Model + ECS Runtime Services”：它应明确承诺的不是“每个组件都像对象随意互调”，而是“局部封装允许，但跨组件协作必须通过可验证依赖、稳定句柄、确定性事件和提交屏障”。

## A.3 “组件带逻辑”的具体技术争议

- `[Verified][S004][S009]` **缓存与批处理**：对象组件常分散在堆上，循环访问需要指针追逐；DOD 按组件列或 Archetype 成批扫描。
- `[Verified][S081]` **结构变更安全**：组件方法若在遍历中任意增删实体，会使底层数组重分配、迭代器失效；Flecs 在 `progress()` 中把世界置为只读并延迟结构命令。
- 推测：`[Estimated，依据 S002/S010/S081]` **调度可分析性**：方法内部可以触达任意组件时，框架无法可靠推导读写集，自动并行只能退化为显式分相或粗粒度锁。
- 推测：`[Estimated，依据 S086/S108]` **确定性**：直接对象引用、事件订阅顺序和哈希容器枚举会把隐式顺序带进模拟；重放需要显式排序和稳定 ID。
- `[Verified][S087][S088]` **反例/回应**：Friflo 等托管 ECS 同时提供值组件与 `Script`，说明工程界并非只接受纯 POD；代价是脚本路径与批量数据路径需要分别定义性能合同。

### A 章来源
S001–S005、S009–S010、S078–S088、S093。

---

# B. 前后端共用同一套实体定义的工程形态

**结论先行**  
**一**：所谓“前后端共用”至少有 Schema 共用、类型/部分逻辑共用、整套模拟逻辑共用三种，不应混称。  
**二**：成熟框架普遍允许端侧变体或可见性声明，不会把服务器全部组件无条件编入/同步到客户端。  
**三**：共用越深，预测与一致性越容易；作弊面、AOT、裁剪、包体和版本门槛也越高。

## B.1 三种共用层级

| 层级 | 代表 | 共用内容 | 主要收益 | 主要代价 |
|---|---|---|---|---|
| Schema/IDL 共用 | KBEngine EntityDef、Colyseus Schema、Protobuf/FlatBuffers | 字段、方法、类型 ID；两端生成不同代码 | 协议单源、端侧最小化 | 业务逻辑重复；生成器和迁移成本 |
| 类型 + 部分逻辑共用 | Mirror/FishNet Shared Assembly、ET、部分 Unity Netcode 项目 | DTO、组件定义、纯逻辑、预测代码 | 减少漂移，测试复用 | 条件编译、平台 API 隔离、客户端反编译可见 |
| 整套模拟逻辑共用 | Photon Fusion 预测对象、Unity Netcode predicted systems、确定性 lockstep/rollback 项目 | 输入到状态的核心模拟 | 预测/回滚最直接 | 必须确定性；服务端秘密不能写进共享产物；AOT/浮点/平台差异放大 |

`[Verified][S065][S066]` KBEngine 的 Entity Definition 把实体属性/方法和分布位置写在定义中，由服务器与客户端 SDK 生成/消费对应接口。它是“Schema 共用”，不是同一个 C# 组件对象在两端原样运行。

`[Verified][S072]` Colyseus 以 Schema 声明可同步状态，客户端 SDK 只接收被声明字段和增量；Room 逻辑仍在服务器。`[Verified][S051]` Photon 的 `[Networked]` 属性由 IL weaving 接到网络状态缓冲，服务器/状态权威与客户端同一 `NetworkBehaviour` 类型可运行不同回调路径。

`[Verified][S016][S021]` Unity Netcode 为 Ghost 提供 server/interpolated client/predicted client 变体与 send-to-owner 选择；同一源组件可生成不同端侧 Prefab 组成。这个“变体”能力比简单的 `#if SERVER` 更可审计。

## B.2 非对称字段、组件和实体

成熟表达可归为四类：

1. **字段可见性标记**：KBEngine flags、Mirror owner/observers、FishNet ReadPermission、Photon Interest、Colyseus StateView。`[Verified][S032][S044][S053][S073]`
2. **端侧组件变体**：Unity Ghost Variant 指定 Server、Interpolated Client、Predicted Client 的组件集合。`[Verified][S021]`
3. **端侧程序集/条件编译**：共享纯逻辑，Model/Renderer 等表现组件只进入客户端产物。推测：`[Estimated，行业通用；与 S027 的 Host 双份表现事故相符]`
4. **完全独立投影类型**：服务器实体经 Schema 映射成客户端 Replica/ViewModel；类型名可相同，但运行时类型不是同一个。`[Verified][S065][S072]`

## B.3 共用边界的真实事故面

- `[Verified][S018]` Unity Netcode 在连接时交换 Netcode 版本、Game 版本、RPC 与序列化组件集合；不兼容就拒绝，而不是让不同 schema 继续解码。这说明“共享定义”必须有运行时兼容门。
- `[Verified][S076]` Colyseus 0.18 明确修复/阻止第 64 个字段索引可能被解释为新结构字节的情况；结果不是“少同步一个值”，而是客户端流解码脱同步。
- `[Verified][S027]` Unity Netcode Host 场景中，未剔除服务器 Prefab 的 ParticleSystem 会在 server world 与 client world 各出现一份，产生表现残留。这是“同一源定义但端侧组件集未隔离”的直接事故。
- 推测：`[Estimated]` 把服务器私有算法编入客户端共享程序集不会自动把状态同步出去，但会扩大反编译与作弊分析面；因此“共用逻辑”必须有 secret-free 规则，服务器私有检测、掉落表、完整 AI 意图不进入客户端构建。

## B.4 坚决不共用的理由

`[Verified][S094]` Unreal 的权威属性复制以服务器 Actor 为源，客户端接收属性而不是要求同一套服务器服务逻辑。Colyseus 也让 Room 逻辑只在 Node 服务器，客户端只是 Schema 消费者。它们牺牲预测代码复用，换取服务器实现自由、秘密隔离和端侧包体最小化。

### B 章来源
S016–S021、S027、S030–S057、S063–S077、S091、S094、S101–S105。

---

# C. World / Scene / 实体命名空间

**结论先行**  
**一**：World 首先是所有权、句柄有效性和更新调度边界，不只是一个对象列表。  
**二**：服务器 World 与客户端 World 通常各存一份实体，靠 NetworkId/GhostId 映射，不共享本地 Entity 句柄。  
**三**：目标画像用 `Scene` 指 AOI 状态会与关卡、流式分块和 Unity Scene 混淆，应改成精确术语。

## C.1 World 的工业语义

`[Verified][S009][S015]` Unity Entities 可创建多个 World，每个 World 有自己的 EntityManager、系统与实体集合；`Entity` 只在对应 World/EntityManager 上有意义。`[Verified][S078][S080]` Flecs world 同时承载实体索引、表、关系和调度；`ChildOf` 还能表达所有权式层级。

网络框架通常把逻辑 World 与副本 World 分离：Unity Netcode 明确创建 server world 和 client world；Photon Multi-Peer 为每个 Runner 建立自己的对象/物理场景；KBEngine 将 base/cell/client 实体投影到不同进程/端。`[Verified][S016][S056][S066]`

**身份映射应是：**

```
NetworkEntityId  ──server map──> LocalEntityHandle(server world)
                 └─client map──> LocalEntityHandle(client world)
```

推测：`[Estimated]` 任何把 `(Index,Generation)` 直接发到网络上的做法都会把 World 本地分配策略泄漏进协议，并在回滚、重连或客户端不同创建顺序下失效。目标画像的“不透明网络身份 + 本地 Index/Generation”方向与工业实践一致。

## C.2 “Scene” 的五种常见含义

| 语义 | 例子 | 生命周期后果 |
|---|---|---|
| 内容关卡/编辑器场景 | Unity Scene、Unreal Level | 加载/卸载资源与对象集合 |
| 运行 World | ECS World / Photon Runner | 调度、句柄、时间轴边界 |
| 空间分区/Streaming Cell | 大世界 tile/chunk | 资源驻留与服务器分片 |
| 网络兴趣集合 | Mirror observer、Fusion interest、Ghost relevancy | 按连接 spawn/update/despawn |
| 表现可见集合 | 客户端 renderer/model residency | 隐藏、LOD、对象池，不必销毁逻辑副本 |

`[Verified][S049]` FishNet 甚至把 SceneCondition 作为 Observer 条件之一，恰好说明“场景成员”和“网络可见”是两个概念，通过条件组合而不是同义替换。

目标画像的 `OnEnterScene` 实际写的是“进入服务器 AOI 或客户端 AOI”，属于第四/第五种，而不是第一种。推测：`[Estimated]` 继续使用 Scene 命名，会让存档恢复、关卡切换、地图 streaming 和网络 relevancy 同时争用同一钩子。

## C.3 命名空间、销毁与跨 World 泄漏

- `[Verified][S085][S086]` EnTT 的 entity 标识含实体部分和 version；version 不匹配即可拒绝旧引用。
- `[Verified][S022]` Unity Netcode 用 `GhostOwner`/网络 ID 与预测 spawn 标记处理跨端身份，不把本地实体值当网络 ID。
- `[Verified][S028]` Host migration 从最近快照重建新的 server world，再恢复 Ghost 所有权；这说明 World 重建时本地句柄可以全部变化，稳定的是协议身份与快照记录。
- 推测：`[Estimated]` 目标框架应让跨 World 引用只携带 `{WorldId, LocalHandle}` 的显式 `WorldEntityRef` 或网络 ID，API 在解析时验证 World；禁止普通组件字段持有另一个 World 的裸本地句柄。

### C 章来源
S009、S015–S018、S022、S028、S049、S056、S063–S069、S078–S086、S091–S093。

---

# D. 实体的声明、原型与创建路径

**结论先行**  
**一**：“先声明实体类型、创建时绑定组件集”对应 Prefab/Prototype/EntityDef/Archetype Descriptor，成熟实现都会缓存结构元数据。  
**二**：创建成本不只是分配，还包括 ID、存储占位、查询索引、默认值、引用解析、生命周期与复制登记。  
**三**：公开“每秒 N 个实体”数据缺少统一条件；目标必须用自己的 active-component + sync + rollback 路径实测。

## D.1 声明机制对照

| 机制 | 代表 | 发生时间 | 变体/继承 | 热更 |
|---|---|---|---|---|
| Prefab/Authoring 转换 | Unity Entities/Ghost | 编辑期/Bake 期 | Prefab 变体、Ghost Variant | 内容热更容易，组件结构需协议兼容 |
| NetworkObject Prefab Table | Photon | 构建/运行注册 | 嵌套 NetworkObject；不能随意运行期增删 NetworkBehaviour | 受构建表约束 |
| EntityDef/IDL | KBEngine | 构建/启动期生成 | 父定义、分布 flags、组件定义 | 脚本可热更，schema 仍需版本门 |
| Schema class | Colyseus | 编译/加载期 | 组合/嵌套结构 | Node 逻辑易热换，客户端 schema 兼容仍必要 |
| ECS Prefab + IsA | Flecs | 运行时 | 原生继承/override/关系 | 动态但需控制确定性与迁移 |
| Entity descriptor / batch | Friflo 等 | 编译/运行时缓存 | 固定组件集合 | 可重建 descriptor |

`[Verified][S080]` Flecs `IsA` 支持从 Prefab 共享/覆盖组件，`ChildOf` 表达层级；`[Verified][S054]` Photon 网络对象必须经 `Runner.Spawn`，普通 `Instantiate` 只产生脱离网络时间线的本地对象；`IAfterSpawned` 在同批对象 `Spawned` 完成后运行。

## D.2 创建路径的成本账单

创建一次生产实体至少可能包含：

1. 分配不透明 NetworkId 或本地 predicted ID；
2. 从实体表取得 Index + Generation，写入 alive 状态；
3. 选择/创建 Archetype 或确保各组件 store 容量；
4. 写入组件默认值、构造托管 active component；
5. 加入查询、Tag、空间索引、Transform 层级、网络 Ghost/observer 索引；
6. 记录结构命令与确定性序号；
7. 提交后发布句柄；
8. 按依赖顺序执行 Awake/Start；
9. 首次进入某连接兴趣集时生成 Baseline；
10. 若是预测实体，登记 remap key 与超时回收。

因此“创建 API 微基准”与“可玩的实体创建”不是同一指标。`[Verified][S062]` Photon 默认 provider 在 spawn 时 Instantiate、despawn 时 Destroy，并明确提供 pooling 扩展点；这说明网络框架也把对象提供器与协议身份分开。

## D.3 优化手段

- 预编译 `EntityTypeDescriptor`：稳定组件 type IDs、依赖拓扑、默认 blob、复制 schema、生命周期 dispatch table。
- 对本地句柄、命令、引用补丁、observer diff 使用 arena/slab 或批量数组，避免逐实体小对象。
- 批量创建：一次预留实体表、组件 store 与查询索引；按 type 分组写入。
- 对 active component 使用对象池，但重置必须有 epoch/generation，避免旧订阅和异步回调泄漏。
- 同帧 create→destroy 在提交归一化中消去，除非协议要求生成可观察 tombstone。
- 将 `Awake`/`Start` 放在结构发布后、业务可见前的初始化阶段，批量排序执行。

## D.4 销毁的对称性与重入

`[Verified][S081]` Flecs 在只读阶段将结构操作排队，以避免遍历中重分配；`[Verified][S011]` Unity 把增删组件和销毁视为 structural changes。目标的唯一提交点方向正确，但仍要定义：

- 同帧 add/remove 同一组件谁胜出；
- `OnDestroy` 是否还能读组件；
- 销毁钩子里创建/销毁是下一提交批次还是拒绝；
- tombstone 保存哪些协议元数据、多久；
- 预测实体拒绝确认时，是否触发普通 Destroy 还是 `PredictionRejected`；
- 快照恢复时是否跳过副作用型钩子。

`[Verified][S110]` Friflo issue #108 展示了组件类型位集合上限若没有启动期校验，会从“超限”退化成“静默添加错误组件”。实体类型编译阶段必须验证组件数、field ID、bitset 范围与冲突约束，而不是让运行时越界。

## D.5 公开规模与证据限制

`[Reported][S078]` Flecs 项目方宣传“millions of entities”；`[Verified][S089]` C# benchmark 给出同机同负载微基准，但作者明确说不代表真实使用。由于目标组件含逻辑、同步与回滚，报告不引用这些数字作为容量承诺。

### D 章来源
S001、S009–S012、S054、S062、S065、S072、S080–S090、S110。

---

# E. Component 模型

**结论先行**  
**一**：组件带逻辑时，最大风险不是“面向对象慢”，而是依赖、引用、顺序和副作用变成隐式。  
**二**：Transform、Model、能力组件分别处在空间、表现、业务能力三个不同边界，不能只靠同一个基类与六个钩子解决。  
**三**：结构变更、引用失效和端侧差异必须由框架层显式建模，否则对象组合会退化为不可回放的对象图。

## E.1 组件协作的五种工业形态

| 形态 | 示例 | 耦合与测试 | 确定性/失效风险 |
|---|---|---|---|
| 直接持有组件对象引用 | MonoBehaviour/ActorComponent 常见 | 写法直接；单测需构造完整对象图 | 移除/池化后悬空；调用顺序隐藏在栈中 |
| 每次从 Entity 查询 | `entity.Get<T>()`、registry lookup | 依赖显式到类型；易注入 fake World | 每次查找成本；同帧移除语义要冻结 |
| 稳定 ComponentHandle | index+generation/type id | 可检测失效；可跨提交保存 | 需要解析层；不能把裸指针长期缓存 |
| 事件/消息 | observer、event bus、command | 发布者与消费者解耦 | 订阅顺序、重入、事件风暴、生命周期解绑 |
| System 中转/依赖注入 | DOD systems、service ports | 读写集清晰，最易批处理 | 局部行为分散；小能力需更多样板 |

`[Verified][S085]` EnTT 允许 registry 按类型取组件，实体只是标识；`[Verified][S080]` Flecs 用 relation/pair 表达依赖与关系。对象组件框架通常容许直接字段引用，但网络副本、预测重映射和组件移除会使这种引用跨越稳定性边界。

`[Verified][S068]` KBEngine issue #1070 是典型事故：客户端组件 `onAttach` 已运行，但对应 cell entity call 尚未可用；组件对象“存在”不等于它依赖的跨端能力已 ready。问题本质是依赖状态没有进入生命周期模型。

## E.2 依赖、互斥与初始化顺序

可见于工业框架的约束包括：

- **Required component**：组件 A 存在则 B 必须存在；Unity GameObject 有 RequireComponent 类似概念，Flecs 可通过关系/observer 实现约束。
- **Mutually exclusive**：例如 `Predicted` 与 `Interpolated` 模式、两种物理所有者不能同时存在。
- **One-of group**：渲染表示、控制源、移动模型只能选一种。
- **Order-only dependency**：组件都能独立存在，但 A 的 Start 必须晚于 B ready。
- **Runtime capability dependency**：依赖不是具体类型，而是端口/能力，如 `ITransformReadable`、`IAuthoritySource`。

`[Verified][S054]` Photon 将 `Spawned()` 与批次后的 `IAfterSpawned.AfterSpawned()` 分开，就是为“同一批中另一个 NetworkBehaviour 已完成初始化”提供屏障。它没有假设 Inspector 中组件排列顺序等同于可依赖顺序。

可复述的初始化事故链是：A.Awake 获取 B → B 已分配但默认值/网络 Baseline 未应用 → A 缓存错误值或订阅空事件 → B 随后初始化但不再通知 A。即使没有 null reference，系统仍进入稳定但错误的半初始化状态。

## E.3 跨组件引用失效

`[Verified][S086]` EnTT 的 sparse-set 讨论显示 swap-and-pop 会移动组件；稳定指针需要 tombstone/in-place delete 等策略，且会牺牲紧凑迭代。`[Verified][S095]` Unreal 的复制对象引用只有在对象具备可复制/稳定网络身份、且客户端已知它时才有意义。

在目标画像中至少有四类引用，不能混成 C# 引用：

1. **同实体同组件批次内引用**：可用 `ComponentSlot<T>` 或经实体查找；组件移除后 generation/type revision 失效。
2. **同 World 跨实体引用**：`EntityHandle(Index,Generation)`；解析时验证 alive、World 和 tombstone。
3. **跨端网络引用**：`NetworkEntityId`；客户端可能暂不可解析，需 unresolved 状态。
4. **表现资源引用**：asset/model handle；不进入确定性模拟哈希，也不应由服务器实例化。

推测：`[Estimated]` 若允许 active component 长期缓存另一个组件对象，框架至少需要移除通知或 handle 失效检查；否则对象池会把旧引用悄悄指向新生命期对象，是比普通 null 更难诊断的 ABA 问题。

## E.4 组件增删与结构提交

`[Verified][S011][S012]` Unity 把增删组件视为 structural change，可能把实体移动到另一 Archetype/chunk；Job 中必须通过 EntityCommandBuffer 延迟。`[Verified][S081]` Flecs 直接说明遍历中立即结构变更可导致数组重分配和崩溃，因此 stage/command queue 在同步点合并。

对 active component，延迟提交还有额外目的：

- 在 commit 前合并 `Add A → Remove A`；
- 验证 required/mutex 约束；
- 批量构造并一次发布，避免查询看到半组件集；
- 稳定生命周期派发顺序；
- 把结构变化纳入同一帧日志、快照和 state hash；
- 把组件对象池返还推迟到所有读者退出 epoch 之后。

## E.5 Transform 专节

### E.5.1 为什么 Transform 特殊

`[Verified][S014]` Unity Entities 的 Transform 体系区分局部变换、父关系与世界矩阵，并由系统传播；这不是一个独立 POD 的单次写入，而是有拓扑依赖的派生状态。

Transform 同时接触：

- **父子图**：父改变使整棵子树 dirty；reparent 要做循环检测与深度/拓扑更新；
- **物理**：kinematic/rigidbody 可能成为写权威，逻辑 Transform 不能同时覆盖；
- **渲染**：客户端按插值时间显示，不一定等于模拟 tick 的权威位置；
- **AOI**：空间索引通常使用世界 bounds/position，父对象变化会让大量子对象跨格；
- **网络**：位置/旋转/缩放/父关系需要不同量化和可靠性；
- **预测**：客户端保存 predicted state、权威 state 与 render-smoothed state 三份视图。

### E.5.2 层级表示的几种方式

| 方式 | 查询父 | 遍历子 | Reparent | 并行与确定性 |
|---|---:|---:|---:|---|
| 每实体 Parent 引用 | O(1) | 需反向索引或扫描 | 改父字段 | 需拓扑排序；并行按深度分层 |
| Parent + Child list | O(1) | O(children) | 更新两端，事务要求高 | 子列表顺序必须稳定 |
| 扁平层级 + depth/range | O(1) 附近 | 子树连续 | 重排代价大 | 扫描局部性好，排序规则需稳定 |
| ECS relationship/pair | 查询关系 | 关系索引 | 改 relation | Flecs 可原生查询；仍要定义 Transform 求值序 |

`[Verified][S080]` Flecs 的 `ChildOf` 删除父时可级联删除子实体；这种所有权语义并不适合所有挂载，例如“角色暂时拿起武器”通常不应因角色副本离开 AOI 就销毁服务器武器。Transform parent、生命周期 owner、网络 AOI owner 需要分开。

### E.5.3 网络表示

`[Verified][S059]` Photon `NetworkTRSP` 明确把 Translate/Rotation/Scale/Parent 视为一组，并指定用于 AOI 的主 TRSP；`SetAreaOfInterestOverride` 可让携带物使用父 NetworkObject 的 AOI 位置。`[Verified][S060]` `NetworkTransform` 的 AutoAOIOverride 让子对象跟随父对象兴趣集合，并把 Teleport 与普通插值分开。

网络 Transform 至少需要区分：

- **权威模拟值**：服务器 tick 的 local/world transform；
- **压缩值**：量化后 position/rotation/scale/parent ID；
- **预测值**：客户端当前 tick；
- **渲染值**：插值/纠偏后的视觉 Transform；
- **Teleport/warp**：禁止跨该边界插值；
- **Parent change**：通常可靠或随完整 Baseline；父引用不可解析时需冻结 world transform 或临时 root。

`[Verified][S024]` Unity Netcode 区分 interpolation、extrapolation 与预测；外推只是渲染时间线的数学操作，不等于重演 gameplay 模拟。

## E.6 Model / 表现组件

服务器没有模型并不意味着共享实体定义必须放弃；工业方案有四种：

| 方案 | 做法 | 代价 |
|---|---|---|
| 端侧组件集 | server prefab 无 Model，client prefab 有 Model | 需要 schema/prefab 变体工具；最清晰 |
| 共享声明 + 客户端适配器 | 共享 `PresentationDescriptorId`，客户端解析资源 | 逻辑不依赖引擎类型；需要表现映射层 |
| 条件编译 | `#if CLIENT` 包含 Model | 简单但类型布局/测试组合增多 |
| 服务器空实现 | 同类型存在但方法 no-op | 统一 API，但浪费对象/包体且容易误把表现状态进哈希 |

`[Verified][S021]` Unity Ghost Variant 能决定组件在 Server/Interpolated Client/Predicted Client Prefab 中是否存在；`[Verified][S027]` Host 粒子系统双份问题说明“空实现或忘记剔除”会把表现对象也放进服务器 World。

## E.7 能力即组件

“挂组件获得能力”有大量先例：NetworkBehaviour 获得复制回调，Physics/Transform 组件获得物理/空间能力，Flecs relation/tag 改变查询资格。`[Verified][S055][S080]` 关键边界不是能力内部算法，而是：

- 能力组件是否必需依赖 Attribute/Transform/Authority；
- 能力启用与组件存在是否分开；
- 能力的权威状态、预测状态与表现状态分别存在哪里；
- 能力组件移除时，未完成事件/定时器/订阅如何撤销；
- 复制 Schema 是否随能力动态安装，还是实体类型已预声明所有可选槽位。

能力的技能/效果/Modifier 求值顺序不在本报告展开；这里只保留组合接口与同步边界，内部见 GAS 专项。

### E 章来源
S011–S015、S021、S024、S027、S054–S060、S068、S080–S088、S095。

---

# F. 生命周期语义

**结论先行**  
**一**：生命周期不是六个方法名，而是状态、发布屏障、允许操作、错误处理和恢复语义的乘积。  
**二**：`Awake`/`Start` 分开解决同批组件依赖；网络对象常再增加 Spawned/AfterSpawned/Authority/Interest 回调。  
**三**：AOI、Enable、Dormancy、Replica Residency 与 Destroy 是独立轴；把它们线性排列会在 Host、抖动和恢复时破裂。

## F.1 主流生命周期钩子对照

| 框架 | 构造/挂载 | 首次可依赖 | 启用 | 网络出现 | 网络离开 | 销毁 | 钩子内结构变更 |
|---|---|---|---|---|---|---|---|
| Unity MonoBehaviour | Awake | Start | OnEnable/OnDisable | 无内建 AOI | 无内建 AOI | OnDestroy | 可调用，但运行期对象全局顺序不保证 |
| Unreal Actor/Component | PostLoad/PostActorCreated、OnComponentCreated、Pre/Initialize/PostInitializeComponents | BeginPlay | Activate/Deactivate、tick flag | replication/relevancy 在网络层 | dormancy/relevancy 不等同 Destroy | EndPlay/Destroyed | 受引擎阶段与 replication 约束 |
| Unity Entities | Add/set/enable | system query 可见 | enableable component | Ghost spawn | relevancy 可销毁客户端 Ghost | entity destroy | 结构变更走 ECB/同步点 |
| Flecs | OnAdd | OnSet/observer/自定义 phase | toggle/Disabled | 无网络内建 | 无网络内建 | OnRemove | readonly/stage 时 defer；merge 时触发 |
| Mirror | Awake/Start + OnStartServer/OnStartClient/OnStartLocalPlayer/OnStartAuthority | 网络 identity ready 后 | GameObject enable 另行 | observer spawn / OnStartClient | observer hide/stop client | OnStop* / Destroy | 服务器 API；observer rebuild 独立 |
| FishNet | Awake/Start + OnStartServer/OnStartClient/OnOwnership* | 网络启动回调后 | Unity enable 另行 | observer qualifies | observer removed | OnStop* | 由网络对象/管理器控制 |
| Photon Fusion | Awake 不适合网络状态；Spawned | IAfterSpawned | simulation inclusion | Spawned；InterestEnter | InterestExit；despawn 是另一动作 | Despawned | 网络对象经 Runner，批次后回调 |
| KBEngine client | 实体/组件创建、onAttach | onEnterWorld/相关 call ready | 客户端自定义 | onEnterWorld/onEnterSpace/视野事件 | onLeaveWorld/onLeaveSpace | 客户端实体销毁 | 历史 API 多阶段，版本需锁定 |
| Colyseus | Room onCreate / client Schema instantiate | 首次全量状态后 | 应用自定义 | collection OnAdd / view add | OnRemove / view remove | Room dispose | 状态 patch 批次由 Schema runtime 管理 |

## F.2 `Awake` 与 `Start` 为什么分开

`[Verified][S006][S007][S008]` Unity 的权威语义是：场景加载时先对所有对象执行 Awake/OnEnable，再在第一帧更新前执行 Start；由此 Awake 适合建立自身不变量，Start 适合依赖其他对象已 Awake 的初始化。但运行时动态 Instantiate 无法保证“所有未来对象 Awake 后才 Start”。

其背后的具体坑可复述为：

1. 同一实体类型一次创建 A、B、C 三个组件；
2. 若组件构造完立即执行单一 Initialize，A 访问 B 时 B 可能尚未完成默认值、依赖注入或网络状态安装；
3. 改变组件注册顺序会改变游戏状态，重放和测试不稳定；
4. 将局部构造与跨组件连接分成两阶段，才有可声明的屏障。

`[Verified][S054]` Photon 更进一步：`Spawned` 表示 NetworkObject 已接入 Runner、网络属性可用；`IAfterSpawned` 在一批对象全部完成 Spawned 后执行，被官方称为 Start 的对应物，明确避免跨 NetworkBehaviour 执行顺序担忧。因此两阶段不是上限；生产网络对象常需要：Allocate → Attach → BaselineApplied → LocalReady → BatchReady → InterestEnter → PresentationReady。

## F.3 Enable/Disable 到底禁用什么

`[Verified][S013]` Unity Entities enableable component 的禁用不是结构移除：组件仍在存储中，但默认查询像不存在一样，且不触发 Archetype 迁移。`[Verified][S096]` Unreal dormancy 是把 Actor 从复制工作中移出而不销毁 Actor。`[Verified][S048]` FishNet ObserverManager 还能只对 Host 客户端隐藏 renderer；逻辑存在、网络观察与视觉可见显然是不同轴。

“禁用”至少可能指：

- 不再被 gameplay update 查询；
- 不再接收输入；
- 不再参加物理；
- 不再复制；
- 不再作为 AOI 候选；
- 不再渲染；
- 保留状态但进入休眠；
- 仍可被管理/诊断查询找到。

若 API 只给一个 `OnDisable`，上述行为无法从名字推导，且各系统可能选择不同解释。

## F.4 AOI Enter/Leave 的先例

### Photon Fusion

`[Verified][S053]` `IInterestEnter.InterestEnter(PlayerRef)` 与 `IInterestExit.InterestExit(PlayerRef)` 在本地玩家获得/失去某 NetworkObject 兴趣时调用；兴趣可由 AOI 区域相交或显式 always-interested 产生。它是按 player 的网络兴趣事件，不是 GameObject Enable。

### Mirror / FishNet

`[Verified][S034][S040]` Mirror 由服务器重建 observer 集，对新增连接 spawn，对移除连接 hide；对象本身仍在服务器。源码还强制把 owner connection 加回 observer 集，以规避 teleport/proximity 下玩家看不见自己的问题（关联 issue #692）。

`[Verified][S046][S047]` FishNet 的 NetworkObserver 组合多个 ObserverCondition；Distance、Scene、OwnerOnly 等决定连接是否成为观察者。Observer 条件是网络披露策略，不等于 Unity 对象启用状态。

### Unity Netcode for Entities

`[Verified][S029]` Ghost relevancy 是服务器按连接决定某 Ghost 是否复制，可用于距离与反作弊 fog of war。`[Verified][S026]` 社区/官方讨论确认离开 relevancy 时客户端实体被销毁，重新相关时作为新实体创建；客户端本地派生表现必须处理重建，而不能假设只是 Disable/Enable。

### Unreal

`[Verified][S096]` Actor relevancy、NetCullDistance、NetDormancy、更新频率是复制层概念；Dormancy 可停止更新但保留对象。UE 没有把“这一帧不相关”映射成 ActorComponent `Deactivate`。

### KBEngine

`[Verified][S066][S069]` 客户端实体有 onEnterWorld/onLeaveWorld、onEnterSpace/onLeaveSpace 等分层回调；历史插件把客户端投影进入/离开世界与空间事件显式化。issue #1070 又说明组件 attach 与进入 world 不是同一时刻。

## F.5 三个判断题

### 判断 1：AOI Enter/Leave 与 Enable/Disable 是否合并？

**事实结论：主流实现分离。** `[Verified][S013][S040][S046][S053][S096]` 网络框架维护 per-connection observer/interest；Enable、Dormancy、simulation inclusion、renderer visibility 各有独立机制。少数框架在客户端“离开 relevance 就销毁副本”，那是 Replica Residency 策略，也不是通用 Disable。

合并会出现：服务器因为对某个连接不可见就停掉全局 AI；实体对 A 玩家离开、对 B 玩家仍相关，却只有一个全局 `OnLeaveScene`；客户端模型卸载触发逻辑组件 Disable，导致预测历史丢失；Host 同一进程内 server/client 路径各触发一次同名钩子。

### 判断 2：服务器 AOI 与客户端 AOI 是同一概念吗？

**事实结论：不是。** 服务器 AOI 是 `(connection, entity)` 的授权/优先级/复制关系；客户端 AOI 是本地实体、逻辑模拟、表现与资源是否驻留的策略。`[Verified][S053]` Fusion 的 InterestEnter 带 `PlayerRef`，说明关系是每玩家；客户端资源加载则不会天然带服务器所有观察者。

具体歧义场景：同一怪物已对客户端授权并保持低频状态同步，但高清模型尚未加载；若二者共享 Enter，业务会误以为模型可用。反向场景是模型因相机预取提前加载，但服务器尚未授权实体状态；共享 Enter 会让客户端逻辑访问不存在的权威副本。

### 判断 3：`Start → Enter → Leave → Disable → Destroy` 线性顺序成立吗？

**事实结论：不成立。** 兴趣可以多次进出；同一帧可因 teleport、分区重建或条件变化发生 enter→leave；对象可 Disabled 后仍保留网络状态，也可在未 Enter 某连接前直接 Destroy；快照恢复可以直接构造 active 状态。`[Verified][S026][S053][S096]`

## F.6 生命周期与延迟提交

Flecs/Unity 的共同事实是：结构命令在安全同步点应用，事件在实际 add/remove 时发生。`[Verified][S011][S081][S083]` 对目标的语义映射可拆成三个时间点（此处仅描述可实现模型，不在本章作取舍）：

1. **Record**：业务相记录 Create/Add/Remove/Destroy，不产生公开实体；
2. **Commit structural state**：归一化、验证、分配存储、建立索引；
3. **Dispatch lifecycle**：按稳定顺序触发 OnAdd/Awake/Start/OnRemove/Destroy；钩子中产生的新结构命令进入后续批次。

若 `Awake` 在结构 commit 之前运行，组件查询和句柄尚不稳定；若在 publish 后失败，其他系统可能已看见半初始化实体。因此需要独立的 “Constructing/Initializing” 状态，查询默认不返回，成功后一次 publish。

## F.7 钩子失败与 fail-stop

Unity/Unreal 一般记录脚本异常并继续引擎循环，Mirror 的一些网络回调源码有逐回调异常隔离；这与目标“整 Tick fail-stop”不是同一模型。推测：`[Estimated，依据 S030–S040 与目标约束]` 在 fail-stop 下，钩子异常应使本 Tick 的结构事务不可发布；已经发生的外部副作用（日志、文件、RPC、表现 Instantiate）无法靠字段回滚撤销，因此钩子必须分为：

- **纯状态阶段**：可随帧快照丢弃；
- **Commit 后副作用阶段**：只消费已提交事件，带幂等 key；
- **不可回滚外部操作**：经 outbox/确认点延迟。

## F.8 快照恢复时钩子是否重跑

`[Verified][S028]` Unity Host migration 会按快照重建 Ghost 状态；官方描述重点是恢复状态，而不是把一次性 gameplay spawn 副作用再执行。由此需要区分：

- `Construct/Awake`：重建内部容器/非序列化缓存，可重跑但必须纯；
- `Start`：若含一次性授予/发奖励则不可重跑；应拆为 `OnHydrate` 与 committed gameplay event；
- `InterestEnter`：恢复后按当前观察者集合重新计算，可产生 idempotent baseline；
- `OnDestroy`：历史已销毁实体不因回放到墓碑而再次发外部通知。

### F 章来源
S006–S008、S011–S013、S026–S029、S033–S049、S052–S060、S066–S069、S074–S075、S081–S083、S093–S096。

---

# G. 属性同步（最高优先级）

**结论先行**  
**一**：工业界的“声明即同步”不是运行时反射魔法，而是 Schema 编译、写屏障、dirty tracking、权限裁剪、Baseline/Delta 与回调事务的组合。  
**二**：字段粒度 dirty + 组件 revision + 集合操作日志是最常见的分层；仅有“值变就发”无法处理 AOI、重连和预测。  
**三**：状态必须批量应用后再通知；初始状态、权威纠正与本地预测要使用不同事件语义。

## G.1 实现形态全景

| 形态 | 声明位置 | 生成/运行机制 | AOT/裁剪 | Review 性 | 代表 |
|---|---|---|---|---|---|
| 代码 Attribute/Decorator | C#/TS 类型字段 | IL weaving/source generator/decorator metadata | 生成代码友好；纯反射不友好 | 与业务代码同处，易看但易误标 | Mirror、Photon、Colyseus |
| 独立 IDL/DSL/def | XML/schema/proto/fbs | 生成两端类型与 serializer | 最友好 | Schema diff 清晰 | KBEngine、Protobuf、FlatBuffers |
| Authoring + Bake | 编辑器 Prefab/Inspector | 构建期生成 Ghost schema/prefab variants | 友好 | 需导出文本/CI 检查 | Unity Netcode for Entities |
| 显式注册表 | 启动代码 | 注册 getter/setter/serializer | 可控 | 样板多，漏注册可检测 | 自研/部分 ECS |
| 手写打包 | 业务 serializer | 显式写字段 | 最可控 | 维护成本高 | 特殊高频协议 |

`[Verified][S037][S039]` Mirror 的 Weaver 为 SyncVar 生成 setter，setter 负责 dirty bit 与 hook guard；源码使用 64-bit dirty mask。`[Verified][S051]` Photon `[Networked]` 自动属性没有普通 backing implementation，IL 生成把 getter/setter连接到 NetworkBehaviour 的网络状态内存。`[Verified][S072]` Colyseus 只有声明进 Schema 的字段才参与同步，普通属性不会自动出现在 patch 中。

`[Verified][S021]` Unity Ghost Variant 允许不修改原组件类型而声明另一套复制 schema，并指定 server/interpolated/predicted 端变体。它展示了“运行组件模型”和“网络 schema”可以分离，不必把所有同步配置硬编码进 gameplay 类型。

## G.2 Schema 编译产物

一个可生产的 Schema 编译器至少应生成以下稳定产物；这是从上述框架共同机制抽象出的事实清单，具体 API 为本报告的中性描述：

```text
ComponentSchema
  StableComponentTypeId
  SchemaVersion / CompatibilityEpoch
  FieldDescriptors[]
    StableFieldId
    WireType
    DefaultValuePolicy
    Quantization / CompressionPolicy
    VisibilityPolicy
    ReliabilityClass
    PredictionRole
    ChangeCallbackId
  SerializeBaseline()
  SerializeDelta(dirtyMask, baselineRef)
  ApplyBaseline(stagingBuffer)
  ApplyDelta(stagingBuffer)
  Validate()
  HashCanonical()
```

`[Verified][S101–S105]` Protobuf 与 FlatBuffers 都把 field number/id 当兼容性的长期身份：删除字段要保留编号，不能随意复用；改类型有严格限制。字段在源码中的声明顺序不应被当成协议身份。

`[Verified][S018]` Unity Netcode 在连接阶段比较游戏版本、RPC 与序列化组件集合；不兼容客户端被阻止加入。这是 Schema 生成物必须包含 manifest/hash 的直接先例。

## G.3 脏标记粒度

| 粒度 | Dirty 标识 | 发送成本 | CPU/内存成本 | 典型失败 |
|---|---|---|---|---|
| 整 World/快照 | tick revision | 大 | 检测简单 | 大状态下不可扩展 |
| 整实体 | entity revision | 中到大 | 索引简单 | 一个小字段带上所有组件 |
| 整组件 | component revision/bit | 中 | 组件表易管理 | 大组件的局部变化浪费 |
| 字段 bitmask | field bit | 小 | 字段数上限/扩展 mask | 重排/超限/生成器错误 |
| 子字段/量化 lane | composite mask | 更小 | 编码复杂 | 半更新与回调组合爆炸 |
| 集合操作日志 | add/remove/set op | 与变化量相关 | 需序号、压缩、清理 | 无 observer 时日志无界、重复应用 |

`[Verified][S037]` Mirror 的每个 `NetworkBehaviour` 使用 64-bit dirty bit；因此单一 Behaviour 的 SyncVar 数量与 mask 宽度有直接关系。`[Verified][S038]` SyncObject 记录增量变更，但在没有 observers 时源码明确不记录，避免 change list 永久增长；重新可见时依赖初始全量而不是回放无限日志。

`[Verified][S076]` Colyseus 0.18 的迁移文档是字段位/索引边界的真实事故：旧编码中第 64 个字段索引可能与“新结构”控制字节混淆，客户端状态流发生 desynchronization。修复方式是在 schema 定义阶段拒绝危险布局，而不是在网络运行时猜测。

`[Verified][S019][S020]` Unity Netcode 可按字段量化并相对 Baseline 做 delta；三 Baseline 预测压缩能降低可预测数据的位数，但会让服务器即使值未变也继续发 snapshot，并增加两端 CPU。粒度越细不等于总成本越低。

### G.3.1 推荐用于比较的复合模型（事实抽象）

成熟实现往往同时维护三层：

- **Entity replication revision**：本实体自哪个确认点后是否有可见变化；
- **Component dirty bit + component revision**：快速筛选需要编码的组件；
- **Field mask / collection oplog**：编码具体变化。

这使 AOI Baseline 可以忽略历史 dirty，直接从当前组件状态构造；持续观察者才使用增量。

## G.4 变更检测方式

### G.4.1 写入即标脏

`[Verified][S037][S039][S051]` Mirror/Photon 通过生成 setter 或网络状态代理，在写入点标记变化。这种方式适合 active component，因为业务逻辑仍以属性写入；条件是所有可变路径都经过生成器。

失败模式：

- 返回 `ref` 或暴露可变集合，调用者绕过 setter；
- 修改结构体内部字段而未重新赋值外层字段；
- 反射/unsafe/native 写入绕过屏障；
- setter 在 ApplyRemoteState 时再次标脏，形成回声同步；
- hook 中再写同字段导致递归，需 guard。

Mirror 源码的 hook guard 是对此的直接防护。`[Verified][S039]`

### G.4.2 帧末 Diff

帧末比较当前值与 last-sent/last-ack baseline，不要求拦截写入。优点是业务写路径自由；缺点是 O(字段总量) 扫描、浮点/集合比较成本高，且必须保存上一版。对大部分不变字段浪费 CPU。

Photon `ChangeDetector` 可在 simulation snapshot、render snapshot 或自定义频率上比较前后 buffer；每个 detector 保存自己的前态。`[Verified][S061]` 这更接近“对已存在 snapshot 做观察”，不是服务器 dirty 生成的唯一机制。

### G.4.3 版本号/写版本

ECS 常给 chunk/component 维护 change version，系统只处理“自上次版本后可能写过”的块。它是保守变化检测：被声明写入不等于值真的变了。适合批处理，不适合直接作为网络字段 delta，因为会产生 false positives。

### G.4.4 集合写屏障

同步 List/Map/Set 需要包装容器，记录 `Add/Remove/Set/Clear`；直接暴露底层集合会绕过日志。集合操作还必须有稳定 key 与 operation order，不能依赖本地哈希桶顺序。

## G.5 通知时机：从“逐字段 hook”到复制事务

### G.5.1 已有框架的差异

- `[Verified][S032][S039]` Mirror SyncVar hook 由生成代码调用，并有 guard；初始状态与普通增量的具体 hook 时机需要按版本核对。
- `[Verified][S052]` Photon `OnChangedRender` 不会在客户端对象首次 spawn 时自动调用；初始化必须放在 `Spawned` 或手动调用。Gameplay 相关变化应在 `FixedUpdateNetwork` 的 change detection 中处理，才能与 prediction/rollback 对齐。
- `[Verified][S057]` Photon 默认 “Full Consistency” 让同一 NetworkObject 的所有 `[Networked]` 属性在同一 tick 到达；也可选择经典 eventual consistency，使同对象字段分开到达。
- `[Verified][S075]` Colyseus 客户端提供 collection `OnAdd/OnRemove` 与字段 Listen；回调是消费 patch 的应用层接口。

### G.5.2 半更新事故

假设属性 `Health=0` 与 `LifeState=Dead` 属于同一权威 tick：

1. 网络解码先写 Health；
2. 立即回调 UI/AI，看到 `Health=0, LifeState=Alive`；
3. 回调触发死亡表现或发送命令；
4. 随后 LifeState 才写 Dead；
5. 回放时字段顺序变化，副作用顺序不同。

这不是传输可靠性问题，而是应用事务边界缺失。Photon 的 per-object consistency 是工业证据，目标的“唯一提交点”可以成为更强的本地应用边界。

### G.5.3 可实现的接收状态机

```text
Receive packet
  → validate protocol/schema/epoch
  → deduplicate {connectionEpoch, snapshotSeq, entityId}
  → stage entity creates and component adds
  → decode all fields into staging values
  → stage removals/despawns
  → resolve references already present
  → register unresolved references
  → validate invariants and authority
  → atomically publish entity/component revisions
  → deterministic callback queue
  → presentation/event outbox after frame commit
```

关键原则：

- Apply remote 不触发 outbound dirty；
- 同一复制事务内 callback 看见完整状态；
- callback 顺序由 `(phase, entity stable order, component type id, field id)` 决定；
- callback 不能递归应用网络包；结构变化进入命令缓冲；
- 初始 Baseline、普通 Delta、Prediction Correction 使用不同原因码；
- 失败时 staging 全部丢弃，不把半对象发布给查询。

## G.6 同步调度的接口面

传输栈和每连接预算属于 DS 专项，但 ECS/复制层必须暴露可调度元数据，否则“声明即同步”会让任何字段都成为隐式带宽债务。

`[Verified][S058]` Photon 将 replication feature 分成 scheduling 与 scheduling+interest；超过每 tick 数据上限的对象会提升后续 tick 优先级。`[Verified][S020]` Unity 说明序列化 CPU 随被编码 Ghost 数近线性增加，Baseline 策略影响 CPU 与包大小。

Schema/组件至少要提供：

- `FrequencyClass`：every tick、N tick、on change、on spawn only；
- `PriorityClass`：critical gameplay、nearby motion、cosmetic；
- `ReliabilityClass`：state eventual、reliable transition、ordered event；
- `QuantizationProfile`；
- `MaxStalenessTicks`；
- `VisibilityPolicy`；
- `CoalescePolicy`：只保留最新状态或保留所有 op；
- `DependencyGroup`：必须同 Baseline 到达的字段/组件。

这些是复制调度器的输入，不在 ECS 字段 setter 里直接发包。

## G.7 可见性、权限与反作弊

`[Verified][S053]` Photon Object/Behaviour Interest 可限制某 PlayerRef 是否获得 NetworkObject 或 NetworkBehaviour 更新；文档明确提到减少流量及限制 team-only 信息。`[Verified][S073]` Colyseus StateView 允许按客户端加入/移除可见对象；`[Verified][S021]` Unity Ghost 可配置 SendToOwner 与端侧变体。

可见性需要至少四层：

1. **Entity visibility**：客户端是否知道实体存在；
2. **Component visibility**：知道实体但不接收 inventory/AI intent；
3. **Field visibility**：例如精确 HP 仅 owner/队友，敌方只见区间；
4. **Value transformation**：服务器发送裁剪/模糊后的派生值，而不是先发明文再让客户端隐藏。

`[Verified][S029]` Unity 官方将 Ghost relevancy 用于 server-side anti-cheat fog of war；安全裁剪必须在服务器编码前完成。客户端 UI 不显示并不构成权限。

需要警惕的声明默认值：Colyseus 旧 StateView 文档指出默认整个状态对所有客户端可见，再用 view 筛选。`[Verified][S073]` 对安全敏感框架，默认 allow 会在漏标时泄露；默认 deny 会增加声明工作但更安全。

## G.8 初始状态、Baseline 与 Delta

### G.8.1 进入视野

进入兴趣集时，客户端没有可依赖的 last ack baseline；服务器需发：

- 实体 identity/type/schema version；
- 必需组件存在集合；
- 当前完整字段值或可重建默认值 + 非默认 delta；
- 父/依赖实体引用；
- authority/owner/prediction mapping；
- baseline epoch/sequence；
- 可选的 history seed（插值/预测）。

`[Verified][S054]` Photon 明确指出远端可能因 AOI、优先级、late join 而晚于权威端 spawn；远端 `Spawned` 获得的是那个较晚 tick 的权威状态，不是最初 spawn tick 的值。这说明 Baseline 代表“当前可观察事实”，不是重放所有历史变化。

### G.8.2 持续 Delta

Delta 必须绑定已确认 Baseline。Unity 的 delta compression 可相对一个或三个 Baseline；三 Baseline 对可预测字段效果好，但需要持续发送 snapshot。`[Verified][S019][S020]`

### G.8.3 重进 AOI 与重复 Baseline

推荐分析维度：

- `InterestEpoch`：同一连接对同一实体每次 fresh enter 增加；
- `BaselineId`：本次 full state 的身份；
- `LastAppliedSnapshotSeq`：客户端去重；
- `SoftLeaveUntilTick`：客户端保留副本时，可重用 Baseline 还是必须刷新；
- `SchemaEpoch`：字段布局变化后旧缓存不可复用。

没有 epoch 时，旧网络包可能在离开后到达，并污染重新进入的新副本；只靠 NetworkEntityId 无法区分两个可见生命期。

## G.9 跨实体引用与 Baseline 依赖

复制引用可以遇到：

- 目标在同包稍后创建；
- 目标不在当前 AOI，但引用字段可见；
- 目标处于 soft-leave cache；
- 目标是 predicted ID，稍后 remap；
- 目标已经销毁，客户端只有 tombstone；
- 父 Transform 未进入，但子对象进入。

`[Verified][S095]` Unreal 的网络对象引用依赖对象具有网络支持与可识别身份。`[Verified][S060]` Photon 的 parenting/AOI override 又显示父子对象的兴趣依赖要显式处理。

一个中性数据结构是：

```text
ReplicatedEntityRef {
  NetworkEntityId id;
  ReferenceEpoch? expectedEpoch;
}
ResolvedEntityRef {
  LocalHandle handle;
  NetworkEntityId id;
  ResolutionState = Resolved | Pending | Tombstoned | Forbidden;
}
```

Pending 进入 patch table；当目标 Baseline 提交或 remap 完成时，在确定性 phase 内解析并发 `ReferenceResolved`，而不是在 getter 中静默变化。

## G.10 数值 Attribute 的同步

### G.10.1 同步结果还是输入

| 路线 | 发送内容 | 带宽 | 一致性 | 预测 | 作弊面 |
|---|---|---:|---|---|---|
| 同步权威结果 | Current/Base/Revision | 稳定，与变化率相关 | 客户端直接收敛 | 可做 overlay | 服务端不泄露完整规则 |
| 同步输入/事件并客户端重算 | damage/effect inputs | 事件多时高；可压缩 | 要求完全相同逻辑与顺序 | 友好 | 客户端获得更多规则/信息 |
| 混合 | 预测输入 + 周期性结果校正 | 中 | 可对账 | 最常见 | 仍需权限裁剪 |

`[Verified][S055]` Photon 的 Health 示例把 `[Networked] Health` 作为状态权威结果，并在输入权威/状态权威上模拟；权威 snapshot 到达时客户端回滚重演。`[Verified][S023]` Unity 区分可靠 RPC 事件与 eventual snapshot 状态：RPC 不带 snapshot tick smoothing，Ghost 状态可 delta 与平滑。

对于 Attribute，“基础值/当前值”必须有明确协议含义：

- Base 若是内容/装备决定，可较低频；
- Current 是权威可观察状态，通常需要同步；
- Modifier 内部列表属于 GAS 边界，不能因共享组件而默认全量泄露；
- 每个 Attribute 带 `AuthoritativeRevision/Tick`，让 UI、预测 overlay 和回放对齐；
- `Health=0` 与 `LifeState=Dead` 应在同一 consistency group。

### G.10.2 到达顺序问题

若 Attribute 结果先到、效果/表现事件后到，客户端可能先跳数值再播放动画；若事件先到、状态被丢包延迟，动画结束但 HP 未变。状态用于最终真相，事件用于一次性表现；事件携带 tick/event id，客户端可等待对应 state revision 或按策略超时。

## G.11 与客户端预测的耦合

`[Verified][S056]` Photon 明确指出非 State Authority 对 `[Networked]` 属性的写入会在下一权威状态到达时被覆盖；Input Authority 的超前模拟会在新服务器状态到达时 rollback/resimulate。一个字段若既存权威值又存 predicted 值，需要逻辑上分层：

```text
Authoritative<T> { value, tick, revision }
PredictedOverlay<T> { value, fromTick, inputs }
Rendered<T> = smoothing(Authoritative, PredictedOverlay)
```

将两个值直接写进同一个普通字段，会丢失“这个值来自哪个时间线”的信息，并使 OnChanged 无法区分本地预测写、权威纠正、重演写与最终确认。

预测实体确认还要 remap 所有引用、命令 owner、Transform parent 与事件 key；只改实体 ID 映射表不够。

## G.12 失败模式清单

| 失败 | 触发条件 | 典型后果 | 已有证据 |
|---|---|---|---|
| Dirty bypass | 可变引用/集合绕过 setter | 永不发送，客户端漂移 | Mirror/Photon 生成 setter 机制反证 S037/S051 |
| Hook 重入 | hook 再写同字段 | 递归、重复 dirty、顺序不稳 | Mirror hook guard S039 |
| 无观察者 oplog 无界 | 集合变化持续记录 | 内存增长、重进视野巨包 | Mirror guard S038 |
| Schema field ID 重用/越界 | 字段删改、mask 超限 | 解码错位或静默写错组件 | Colyseus S076；Friflo S110 |
| 半状态回调 | 逐字段立即通知 | invariant 暂破、重复副作用 | Photon consistency S057 |
| 初始状态不触发 change | 把 change hook 当 init | UI/缓存未初始化 | Photon S052 |
| AOI 重进旧包污染 | 无 interest epoch | 新副本收到旧 delta | 推测：`[Estimated]`，由 Baseline/observer模型推导 |
| 引用目标未出现 | per-entity AOI 独立 | null、错误 parent、永久丢引用 | UE refs S095 / Photon AOI S053 |
| 权限漏标 | default all/组件整体复制 | 地图、队伍、AI/库存泄露 | S029/S053/S073 |
| 预测与权威同字段 | 无来源/tick 标记 | 抖动、回滚覆盖错误 | S055/S056 |
| 序列化器 size bug | buffer/field layout 错 | 内存破坏或协议崩溃 | Unity changelog S025 |
| Delta 基线丢失 | 丢包/ack 过期 | 无法解码或错误值 | Unity baseline docs S019/S020 |
| Reconnect epoch 混用 | connection seq 重置不隔离 | 旧包对新会话生效 | 推测：`[Estimated]`，需协议 epoch |
| 同步风暴 | 大量 enter/字段同时 dirty | CPU/带宽峰值、后续饥饿 | Photon scheduling S058 |

## G.13 状态还是事件

`[Verified][S023]` Unity Netcode 官方对比：RPC 可靠、按接收帧处理、不做 delta；Ghost snapshot 不可靠但最终一致、带 tick、可压缩和平滑。由此可将同步哲学归纳为：

- **持续可查询、晚加入者必须拥有的事实**：同步状态；
- **只发生一次、不可由当前状态重建的动作**：同步事件；
- **既影响状态又有表现**：状态作为真相，事件携带幂等 ID/tick 作为提示；丢事件仍可由状态恢复，重复事件可去重；
- **预测输入**：命令/输入流；权威结果仍通过状态收敛。

### G 章来源
S016–S025、S029、S032、S036–S039、S044、S051–S061、S065、S072–S077、S094–S106、S110。

---

# H. AOI / 兴趣管理（最高优先级）

**结论先行**  
**一**：AOI 是每个观察者对实体集合的动态关系，不是实体自身的单一布尔状态。  
**二**：空间索引只解决候选集合；权限、队伍、遮挡、父子依赖、优先级和客户端驻留仍需后续阶段。  
**三**：生产语义必须包含 enter baseline、steady delta、soft leave、hysteresis、epoch 与引用依赖；否则边界抖动会放大为创建/销毁与带宽风暴。

## H.1 学术根与概念演化

`[Verified 摘要级][S097]` Benford 与 Fahlén 1993 年 *A Spatial Model of Interaction in Large Virtual Environments* 提出 aura、focus、nimbus 等空间交互概念：对象是否相互感知，不仅由距离，还由感知方向、投射程度与适配器决定。这是“兴趣不是单半径”的早期理论根。

`[Verified 摘要级][S099]` Macedonia 等人在 1995 年 NPSNET 相关论文 *Exploiting Reality with Multicast Groups* 讨论用 multicast groups 将大规模虚拟环境按空间/现实关系分发，体现了将世界切成兴趣频道的工程路线。

`[Verified 摘要级][S098]` Morse、Bic、Dillencourt 2000 年 *Interest Management in Large-Scale Virtual Environments* 将兴趣管理作为大规模 DVE 降低消息与处理负载的核心问题。

`[Verified 元数据级][S100]` IEEE HLA 的 Data Distribution Management 家族把“谁订阅/发布哪些数据区域”提升为分布式仿真标准能力。规范全文受限，因此本报告不引用其算法性能数字。

从学术到游戏工程，概念保持一致：AOI 是**数据分发关系**，空间只是最常用输入，不是完整语义。

## H.2 生产 AOI 管线

一个典型服务器 AOI 管线可表示为：

```text
Authoritative transforms / bounds
  → spatial index update
  → broad-phase candidates per observer
  → semantic filter (team, phase, ownership, stealth, permission)
  → dependency closure (parent, owner, referenced essentials)
  → hysteresis / dwell / soft-leave state machine
  → priority & budget queue
  → observer diff {Enter, Stay, Leave}
  → Baseline / Delta / Hide-or-Despawn instructions
```

`[Verified][S046]` FishNet NetworkObserver 允许多个 conditions；`[Verified][S053]` Photon 将 Area Of Interest、Global、Explicit interest 分开，并可对 NetworkObject 或 NetworkBehaviour 裁剪；这些都说明空间查询只是一层。

## H.3 工程实现方式全清单

| 方法 | 更新/查询特征 | 适合 | 抖动/缺点 | 证据定位 |
|---|---|---|---|---|
| 固定网格/九宫格 | 移动时更新 cell；查询周边 cell；平均近 O(候选) | 均匀大世界、半径相近 | cell 边界抖动；密集热点退化 | Mirror Spatial Hashing S035；FishNet conditions S046 |
| 多层网格/哈希网格 | 不同半径/对象尺度分层 | 尺度差异大 | 层间重复、更新复杂 | 推测：`[Estimated]` 数据结构推断 |
| Sweep/十字链表 | 按轴排序，局部移动调整 | 2D、移动连续 | teleport 和高维复杂；全局排序维护 | 推测：`[Estimated]`；未找到目标框架官方实现证据 |
| Quadtree/Octree | 空间递归划分；查询区域 | 非均匀静态/中等动态 | 大量移动导致节点迁移；边界重叠 | 推测：`[Estimated]` 标准结构特性 |
| KD-tree/BVH | 范围/最近邻查询快 | 读多写少、复杂 bounds | 动态重建/旋转成本 | 推测：`[Estimated]` |
| 预计算可见集/PVS | 运行时查表 | 室内/拓扑稳定 | 动态障碍与开放世界难 | 推测：`[Estimated]` |
| 查询订阅 | observer 订阅实体谓词/区域 | 分布式 schema、房间/频道 | 查询维护与权限复杂 | HLA、Colyseus StateView S073/S100 |
| 逐对象距离判定 | O(N×M) 直观 | 小房间/低规模 | 大规模不可扩展 | Photon AOI 是工程化替代 S053 |
| 组合条件 | 空间候选后按 team/owner/scene 等过滤 | 几乎所有生产游戏 | 条件顺序和缓存决定 CPU | FishNet S046–S049 |

复杂度不能脱离密度：网格的平均 O(k) 依赖每格实体数有界；所有实体聚集在一个格时会退化。树结构也不能绕开输出规模：若某观察者确实应看到 K 个实体，至少要处理 O(K) 的结果。

## H.4 数据模型：AOI 是关系而不是实体状态

最小关系键是：

```text
ObserverKey = ConnectionId / PlayerRef / ViewId
SubjectKey  = NetworkEntityId
InterestRelation {
  state: Absent | EnterQueued | Present | SoftLeaving | LeaveQueued
  epoch: uint
  enteredAtTick
  lastRelevantTick
  lastBaselineId
  priority
  reasonMask      // distance, owner, team, explicit, dependency, scene...
}
```

`[Verified][S053]` Fusion 的回调携带 `PlayerRef`；同一对象可对一个玩家 enter、另一个玩家 exit。任何只有 `entity.InAOI` 单布尔值的模型都无法表达这一事实。

## H.5 Enter 语义

进入时至少要回答六个问题：

1. **何时算 enter**：空间候选刚命中，还是权限/依赖/预算通过后？
2. **先有对象还是先有状态**：客户端是否可以先占位，再分帧补组件？
3. **Baseline 原子范围**：实体、组件组还是整个依赖簇？
4. **父/引用依赖**：父不在 AOI 时先发父、用 world transform，还是延迟子？
5. **回调时机**：网络副本构造后、Baseline 应用后、Model ready 后分别有什么事件？
6. **超预算**：enter 排队时服务器状态继续变化，Baseline 取排队时还是发送时快照？

`[Verified][S054]` Photon 的远端对象可能因 AOI、late join 或优先级晚于权威端 spawn，并用实际到达 tick 的状态创建。这支持“Baseline 在发送/应用时代表当前状态”的模型。

### H.5.1 推荐分析用的 Enter 序列

```text
CandidateEnter
  → Authorized
  → DependencyClosureBuilt
  → BaselineScheduled
  → ReplicaAllocated (not query-visible)
  → BaselineApplied + refs staged
  → ReplicaPublished
  → NetworkInterestEntered
  → PresentationRequested
  → PresentationReady (optional, asynchronous)
```

前六步属于网络副本状态；最后两步属于客户端表现。将它们都叫 `OnEnterScene` 会隐藏延迟和失败。

## H.6 Leave 语义：销毁、隐藏还是缓存

| 策略 | 客户端处理 | 再进入成本 | 状态风险 | 适合 |
|---|---|---|---|---|
| Hard despawn | 销毁逻辑副本和表现 | 高：新 Baseline/重建 | 简单，旧引用需 tombstone | 大量远离、内存紧 |
| Hide/disable presentation | 保留逻辑副本，停渲染 | 低 | 状态若不更新会陈旧 | 短暂遮挡/视锥外 |
| Soft leave cache | 保留压缩状态至 TTL | 中低 | 需 interest epoch、防旧包 | 边界反复进出 |
| Dormant | 保留副本，仅重大变化唤醒 | 低 | 唤醒全量/版本复杂 | 静态或低频对象 |
| LOD downgrade | 保留低精度/低频组件 | 中 | 多 schema/优先级 | 超大世界远景 |

`[Verified][S026]` Unity Netcode 的 relevancy 默认可表现为客户端实体销毁、重进时重新 spawn；`[Verified][S096]` Unreal dormancy 则停止复制但不销毁 Actor；`[Verified][S048]` FishNet 还能只隐藏 Host client renderer。工业界没有唯一 Leave 行为，必须由 Replica Residency policy 决定。

## H.7 抖动抑制

### H.7.1 Hysteresis / 双阈值

使用进入半径 `R_enter` 与离开半径 `R_exit > R_enter`：

```text
Absent  --distance <= R_enter--> Present
Present --distance >= R_exit --> SoftLeaving/Absent
```

这避免对象在同一边界因浮点、插值或小移动每 tick enter/leave。参数没有可跨游戏复用的公开权威值；它取决于最大速度、网络 RTT、tick、视野冗余与内存预算。本次没有找到主流框架官方给出的通用 AOI hysteresis 百分比，因此不编数字。

### H.7.2 Dwell time / 延迟离场

对象满足 leave 条件后等待 N ticks；期间重新相关则取消。优点是减少 Baseline 和资源抖动，缺点是多占连接带宽/客户端内存，并可能继续暴露本应立即隐藏的敌人。因此**权限型 leave**（隐身/战争迷雾）与**性能型 leave**（距离稍远）不能共享延迟策略。

### H.7.3 Soft leave + cache epoch

客户端保留实体但标记 `NotNetworkRelevant`，停止 gameplay/presentation 或降级；服务器不再发普通 delta。重新进入时必须比较 `BaselineId/InterestEpoch`，不能把缓存当永远有效。

### H.7.4 Budgeted enter/leave

`[Verified][S058]` Photon replication scheduling 在本 tick 因数据上限未发送的对象会提高后续优先级。对于大量进入，服务器可按距离、屏幕重要度、依赖根优先排队；但核心交互对象必须有 starvation 上限。

### H.7.5 批量与分帧

大传送/登录会让几千实体同时 enter。若每个都独立分配、序列化、触发生命周期与加载模型，会产生 CPU、GC、带宽和 GPU upload 峰值。分帧策略需保证：

- 先发 blockers/parents/players，再发装饰；
- 同一实体 Baseline 不跨越不可观察的一致性组；
- 客户端占位不进入 gameplay 查询，直到必要组件齐全；
- Loading UI 与网络 replica ready 分开。

## H.8 AOI 与实体规模

公开资料常给“支持 MMO/百万实体”但缺少统一条件。可用的成本模型比孤立数字更可靠：

```text
Server AOI cost / tick ≈
  spatial updates(moving subjects)
+ candidate queries(active observers)
+ semantic filters(candidates)
+ set diff(current vs previous)
+ baseline serialization(enters)
+ delta serialization(stays with dirty data)
+ leave messages(leaves)
```

真正的峰值常不是 steady-state query，而是 **enter churn × Baseline size**。一个空间索引从 O(N×M) 降到近 O(K) 后，序列化和客户端构造可能成为新瓶颈。

`[Verified][S035]` Mirror 提供 Spatial Hashing 作为 Interest Management 实现；`[Reported][S030]` 项目方称可用于小型 MMORPG，但没有给统一的 N/M/tick benchmark。本报告不把宣传当目标容量。

## H.9 大世界、分块与 AOI 不对齐

地图 chunk、服务器 shard、物理 broadphase cell、NavMesh tile、资源 bundle 和网络 AOI 可能使用不同网格。常见问题：

- AOI 圆跨多个 chunk，进入一个实体要拉取尚未加载的 chunk 数据；
- parent 在 A chunk、child 在 B chunk，分别 streaming；
- 服务器迁移实体所有权时 connection interest 不能断帧；
- 客户端相机预取比网络权限更远；
- 地图边界 teleport 造成整个 observer set 重算；
- 静态环境可用 chunk Baseline，动态实体仍需 per-entity delta。

`[Verified][S049]` FishNet SceneCondition 将 scene membership 作为 observer 条件，说明空间/内容分块可以参与 AOI，但不是 AOI 的全部。

## H.10 服务器 AOI 与客户端 AOI 的分工

### 服务器侧

- 依据权威位置/权限确定客户端可知实体/组件/字段；
- 生成 per-connection enter/stay/leave；
- 控制 Baseline、更新频率、优先级；
- 防作弊：不可见信息不编码；
- 维护引用依赖和 interest epoch。

### 客户端侧

- 接收 Replica 后决定逻辑副本是否参与本地非权威系统；
- 管理 Model、动画、音频、粒子、LOD 与资源流送；
- 对 soft-left 实体缓存或销毁；
- 在相机、设备内存与表现预算下二次裁剪；
- 不能扩大服务器授权的数据集合。

一个具体错误场景：服务器对附近但在墙后的敌人发送低精度脚步/声音组件，却不发送精确 Transform；客户端资源系统仍可预载敌人模型。若共用 `OnEnterScene`，业务无法知道“有音频兴趣”“有模型资源”“有完整网络实体”分别成立到哪一层。

## H.11 AOI 与 Transform/父子/引用

`[Verified][S059]` Fusion 以主 `NetworkTRSP` 的位置做 AOI；`SetAreaOfInterestOverride` 可指定另一个 NetworkObject 的位置。`[Verified][S060]` AutoAOIOverride 让被携带物继承父对象兴趣。这些机制处理了“子物体自身位置/父位置/网络对象边界不一致”。

目标环境还会遇到：

- 父对某连接不可见、子可见：发送 world transform 还是依赖闭包？
- 角色与装备分别是实体：装备必须随 owner interest 还是可以单独掉落？
- 大型 Boss 子部件跨多个格：以 root bounds、每部件 bounds 还是 composite bounds？
- 预测 projectile 引用尚未确认的 caster；
- 软离开父实体时子实体的缓存 TTL 是否一致。

这些不是普通 Transform 计算问题，而是网络依赖策略。

## H.12 AOI 与生命周期绑定：正反证据

### 明确绑定的先例

- Photon 提供 `IInterestEnter/IInterestExit`，作为 NetworkObject 组件回调。`[Verified][S053]`
- KBEngine 客户端提供进入/离开 World/Space 的实体回调。`[Verified][S066][S069]`
- Colyseus collection/view 提供 OnAdd/OnRemove，使可见状态变化映射到客户端对象集合生命周期。`[Verified][S073][S075]`

### 明确分离的先例

- Mirror/FishNet 在服务器维护 observer 集，增删连接，不把全局对象本身 Enable/Disable。`[Verified][S040][S046]`
- Unreal relevancy/dormancy 与 Actor component activation 分离。`[Verified][S096]`
- Unity Netcode relevancy 可以销毁客户端 Ghost，但客户端表现/派生组件需要单独重建；官方讨论暴露 change filter 与 particle 的边界问题。`[Verified][S026][S027]`

综合事实是：**可以有 AOI 生命周期回调，但它必须是“某端、某连接、某副本”的专用回调，不能冒充实体全局生命期或通用 Enable。**

## H.13 可直接实现的 AOI 状态机（中性参考模型）

```text
ABSENT
  CandidateEnter(reason)
    -> ENTER_PENDING
ENTER_PENDING
  AuthDenied -> ABSENT
  BudgetDeferred -> ENTER_PENDING
  BaselineCommitted(epoch, baselineId) -> PRESENT
PRESENT
  Relevant -> PRESENT (Delta)
  PerformanceLeave -> SOFT_LEAVE
  SecurityLeave/EntityDestroyed -> LEAVE_PENDING
SOFT_LEAVE
  RelevantBeforeTTL -> REENTER_PENDING (fresh/refresh baseline policy)
  TTLExpired -> LEAVE_PENDING
LEAVE_PENDING
  LeaveCommitted -> ABSENT or TOMBSTONED_CACHE
```

必须携带的序列信息：`ConnectionEpoch`、`InterestEpoch`、`BaselineId`、`SnapshotSeq`、`EntityLifecycleGeneration`。没有这些字段，旧 enter/delta/leave 在乱序网络上无法判定属于哪次兴趣生命期。

## H.14 AOI 测试矩阵

至少覆盖：

1. 在边界上每 tick 往返移动；
2. 高速 teleport 穿过多个 cell；
3. 同帧 enter→destroy；
4. leave 包先到、旧 delta 后到；
5. soft leave 后重进，服务器状态期间变化；
6. 父不相关、子相关；父后到；
7. 目标引用实体不在 AOI；
8. 连接重连，旧 connection epoch 包延迟到达；
9. 1000+ 同时 enter 的预算与饥饿；
10. Host server/client 双 World 的表现只创建一次；
11. 权限 leave 必须立即生效，不受 hysteresis；
12. 快照恢复后重新构造 observer set，不能重复外部副作用。

### H 章来源
S026–S029、S034–S035、S040–S049、S053–S060、S066–S075、S095–S100。

---

# I. System / 执行模型与帧结构

**结论先行**  
**一**：在组件带逻辑的模型里，System 仍负责批处理、跨实体规则、调度、索引维护和提交，而不是被组件方法取代。  
**二**：显式 phase、读写集、稳定 tie-break 与单一结构 merge 是并行和确定性的共同基础。  
**三**：fail-stop Tick 只能覆盖纯状态事务；异步任务与外部副作用必须经过确认边界。

## I.1 Active Component 模型中的 System 职责

即使组件有方法，以下工作仍天然属于 System/World：

- 对所有含 `Transform + Velocity` 的实体批量更新；
- 全局碰撞、AOI、导航、匹配、经济结算等跨实体规则；
- 统一采集组件命令和事件；
- 按 phase/依赖排序；
- 维护查询、空间、父子、网络 observer 索引；
- 结构 commit、生命周期派发、复制提交；
- 生成快照、日志和状态哈希；
- 预算、并行与诊断。

`[Verified][S010]` Unity Systems 围绕查询处理组件数据；`[Verified][S081]` Flecs Systems 在 pipeline 中运行，世界只读阶段的结构操作进入 stage。组件方法可以成为 System 调用的局部策略，但不应自己决定全局遍历与提交顺序。

## I.2 更新顺序

常见手段：

| 手段 | 优点 | 缺点 |
|---|---|---|
| 显式 phase | 语义清晰：Input→Simulation→Resolve→Commit→Replication | phase 粒度过粗会串行化 |
| before/after 依赖 DAG | 可自动拓扑排序 | 循环依赖与可视化成本 |
| update group | 模块化、可嵌套 | group 内仍需排序 |
| 数字 priority | 简单 | magic numbers、跨团队冲突 |
| 注册顺序 | 零配置 | 代码加载/反射顺序不可作为稳定合同 |

确定性排序至少需要 `(PhaseId, DeclaredDependencyOrder, StableSystemId)`；同一优先级不能依赖哈希集合或程序集扫描顺序。组件 lifecycle 事件队列也使用稳定 component type ID 和 entity creation ordinal 作 tie-break。

## I.3 并行与读写集

纯 DOD ECS 可由查询参数推导读/写冲突；active component 内部若能任意访问 World，真实读写集不可知。可见的并行层次是：

1. **纯函数批处理 lane**：明确 `Read<T>/Write<U>`，可并行；
2. **Entity-local active method**：只访问自身声明依赖，可按实体并行；
3. **Cross-entity command generation**：只写线程本地命令/事件；
4. **Global mutation**：在单一 commit/resolve phase 串行或分区；
5. **外部副作用**：commit 后异步 outbox。

`[Verified][S081]` Flecs 每线程 stage 独立记录命令，在 sync point merge；这既避免数据竞争，也避免遍历中结构数组失效。

## I.4 唯一结构提交点

一帧一个结构提交点的关键不是“把调用晚一点”，而是定义命令归一化：

| 同一实体/组件命令 | 可观察结果需定义 |
|---|---|
| Add A + Set A | 构造 A 后写值，触发一次 Add/Awake |
| Add A + Remove A | 无可见 A；是否保留审计事件需另定 |
| Remove A + Add A | 保留旧实例还是新 generation；生命周期是否两次 |
| Create E + Destroy E | 不发布；NetworkId/tombstone 是否消耗 |
| Destroy E + Set A | Set 拒绝或随销毁消去 |
| Reparent X 两次 | 最后写胜出，仍需循环检测 |
| 多线程同时 Set | 由命令序号/冲突策略确定，不能靠 merge 到达顺序 |

建议用于事实对照的命令 key：

```text
CommandOrder = {Tick, Phase, ProducerSystemId, ProducerPartition, LocalSequence}
```

Flecs 的 per-thread queue 与 Unity ECB 都证明结构变更需要稳定的合并规则；具体目标规则在 P 章给出。

## I.5 事件与消息

- **Frame-local event**：存在于当前/下一 phase，arena 回收；适合碰撞、状态变化提示。
- **Committed domain event**：带 tick、entity id、event id，写日志/复制；必须幂等。
- **Network transient event**：不由状态重建，带可靠性与去重策略。
- **Observation callback**：状态 commit 后生成，不应直接等同 domain event。

`[Verified][S023]` Unity Netcode RPC 与 Ghost 状态的可靠性、tick 语义不同，说明“消息”和“状态变化通知”不能共用一个无类型 event bus。

## I.6 fail-stop、长任务与跨帧挂起

整帧 fail-stop 只对内存状态有效。路径寻路、HTTP、数据库、文件、GPU upload 等可能跨帧完成；若其回调直接修改 World，会把完成时间变成非确定性输入。可用的工业化边界是：

1. Tick 内发出 `AsyncRequest{id, inputHash}`；
2. 外部执行；
3. 结果作为带顺序的 external input 在未来 tick 注入；
4. 回放从日志读取结果，不重新依赖墙钟完成顺序；
5. commit 后才发不可撤销 side effect。

推测：`[Estimated，依据 S108]` 确定性 lockstep 的核心不是所有工作单线程，而是同一输入序列产生同一状态序列；非确定性完成必须变成显式输入。

### I 章来源
S010–S012、S023、S054–S058、S074、S081–S083、S108。

---

# J. 性能与规模

**结论先行**  
**一**：Active Component 的主要成本来自托管对象、间接访问、调用分发和不可分析副作用；存储布局只能解决其中一部分。  
**二**：Archetype、Sparse Set、混合存储分别优化不同负载，不能用单一“每秒实体数”决定。  
**三**：目标必须同时测创建、结构变更、查询、同步、AOI、快照和哈希；只测组件循环会选错。

## J.1 组件带逻辑的性能天花板

1. **缓存局部性**：每组件一个 C# 对象会有对象头、对齐和离散堆地址；从实体到字典再到组件产生多次随机访存。
2. **虚调用/接口分发**：每实体每组件每 tick 调 virtual lifecycle/update，调用开销与分支预测失败会累积；更大的问题是阻止批处理和内联。
3. **指针追逐**：组件直接引用其他组件形成对象图，CPU 无法预取稳定列。
4. **GC 压力**：创建组件、闭包、事件订阅、临时 List、装箱与 LINQ 都可在高 churn 下生成短命对象；Stop-the-world 或增量 GC 均会把负载变成尾延迟。
5. **装箱/泛型擦除**：以 `object`、非泛型 interface 或反射存储值组件会装箱。
6. **不可分析副作用**：组件方法任意写 World，使 Job 调度器无法安全并行。
7. **managed/native crossing**：逐实体 P/Invoke/ICall 比批量数组传递昂贵；跨边界需批处理。

这些机制解释“为什么慢”，但不意味着 active component 必然不可用：低频能力组件可以对象化，Transform/AOI bounds/Attribute 热字段可进入紧凑 lane。

## J.2 Storage 表示对比

| 维度 | Archetype / Column | Sparse Set per component | Active-object map | 混合 |
|---|---|---|---|---|
| 固定组合批量创建 | 最强：一次选表、连续写 | 强：每 store 追加 | 弱：多对象分配/字典 | 强 |
| 频繁增删组件 | 跨 Archetype 搬迁，成本高 | 单 store O(1) 近似增删 | 字典/对象操作，GC 风险 | 将 churn 组件放 sparse |
| 多组件查询迭代 | 同表同列，最佳局部性 | 以最小集合驱动并 probe 其他集合 | 指针追逐 | 热查询用表/列 |
| 单组件存在/随机访问 | entity location 查表 | sparse index O(1) | 哈希/数组 | 可统一 handle table |
| 指针稳定 | 搬迁会失效 | swap-delete 也可能失效；可 tombstone | 对象引用稳定至池化 | 外部只暴露 handle |
| 内存 | 每 Archetype/chunk 有碎片 | 每类型 sparse index 成本 | 对象头/字典高 | 复杂但可按类型优化 |
| 确定性迭代 | 需固定 chunk/row 与 compaction 规则 | swap-delete 改顺序；需 canonical sort | 注册/字典顺序不稳 | 逻辑顺序独立于物理顺序 |

`[Verified][S011]` Unity 的 structural change 会移动实体；`[Verified][S085]` EnTT 每组件使用 specialized sparse set；`[Verified][S086]` pointer stability 需要删除策略/tombstone 并影响迭代；`[Verified][S088]` Friflo 是托管 C# Archetype ECS，同时支持 `Script`，展示混合数据/行为表面。

## J.3 创建与增删的成本差异

### Archetype

- 创建到已存在 Archetype：选 chunk、占 row、按列写默认值；批量效率高。
- 创建新组合：创建/查找 type set、layout、query cache；应由 entity type descriptor 预热。
- Add/Remove：复制保留组件到目标 Archetype，运行构造/析构钩子，更新 entity location。

### Sparse Set

- 创建实体：实体表分配；组件分别 emplace。
- Add/Remove：各 store 更新 sparse+dense；通常不搬其他组件。
- 多组件查询：以最小 dense set 遍历，对其他 set 做 membership test；组件数据不一定同行。

### Active Object

- 创建：从对象池取多个组件、重置、安装引用/事件；若无池则多次 GC allocation。
- Add/Remove：局部但对象关系/订阅清理复杂；查询索引仍需维护。

## J.4 C# benchmark 的可用结论与不可用结论

`[Verified][S089]` Doraku/Ecs.CSharp.Benchmark 在同一机器/负载比较多个 C# ECS，并明确警告结果不代表真实条件。其公开表能说明不同实现之间可能相差数量级、分配量差异显著，但不能直接推导服务器容量。

`[Verified][S090]` Friflo 的 common-use-cases benchmark 覆盖 create/delete、add/remove、queries、relations、command buffer 和索引；优点是维度比单迭代丰富，缺点是由框架项目方维护，具体实现优化与版本需逐项复核。

本报告不搬运单次表格数字作为选型结论，原因：

- 框架版本、.NET、JIT/AOT、CPU 不一致；
- 有的组件是 struct，有的是 class；
- 批量 API与逐实体 API口径不同；
- 不测 lifecycle、dependency、network dirty、snapshot；
- 无长时间 GC tail latency；
- 没有真实的实体类型分布和 churn。

## J.5 海量创建优化清单

- 预留 EntityTable、组件 dense pages、command buffers、reference patch buckets；
- 按 `EntityTypeId` 分组批量创建，避免每实体计算组件集合；
- 默认数据用 immutable blob / memset-friendly template，active object 只存差异；
- 生命周期 dispatch table 预编译，避免 reflection；
- 事件队列用 struct ring/arena；
- 空间索引批量 insert；
- Baseline 构造延迟到实际 observer enter，而不是 server create 时为所有潜在连接准备；
- Model 资源异步，逻辑 replica 不等待 GPU；
- 对象池重置采用 generation/epoch，统一清事件订阅；
- 批量跨 native 边界传 SoA span，而不是逐实体调用。

## J.6 目标负载必须实测的矩阵

| 场景 | 规模参数 | 指标 |
|---|---|---|
| 冷创建固定类型 | 1/1k/100k entities；8/16/32 components | p50/p95/p99、alloc bytes、cache misses |
| 热池化创建 | 同上 | reset cost、旧订阅泄漏 |
| 结构 churn | 每 tick 1%/10% add-remove | moved bytes、commit latency、query invalidation |
| 热查询 | 1/2/3/5 component joins | ns/entity、branch/cache miss |
| active callback | 0/1/4/16 active components | virtual dispatch、phase cost |
| AOI enter burst | 100/1k/10k Baselines | server encode、client apply、GC、frames to ready |
| Dirty sync | 1%/10%/100% fields change | bytes/tick、encode/decode CPU |
| Snapshot/hash | 10k/100k/1m entities | snapshot ms、memory, hash throughput |
| rollback | 2/8/32 ticks | restore+resim ms、allocation |
| managed/native | per entity vs batch | crossings, copies, pinning |

## J.7 真实量级的边界

`[Reported][S078]` Flecs 宣称可用于百万实体；`[Reported][S030]` Mirror 宣称有大量生产项目；这些不能转化为“目标可承载 N”。实体数量必须同时附带：活跃比例、组件大小、查询数、Tick、网络观察者、变化率、AI/物理和硬件。

### J 章来源
S004、S009–S015、S019–S020、S030、S035、S038、S058、S062、S078–S090、S110。

---

# K. 客户端预测与权威收敛

**结论先行**  
**一**：预测不是“客户端也改同步字段”，而是同一实体上维护有 tick 来源的预测时间线，并在权威快照到达时重放。  
**二**：预测创建需要独立 identity、匹配键、拒绝路径和全图引用重映射。  
**三**：整帧回滚能简化跨子系统一致性，但快照成本与副作用隔离必须同时设计。

## K.1 三种预测模型

| 模型 | 代表 | 预测范围 | 收敛方式 |
|---|---|---|---|
| 确定性重演 | GGPO/lockstep、Unity predicted systems、Photon input authority | 输入影响的完整或子集模拟 | 回到权威 tick，重放输入 |
| 乐观本地应用 + 权威覆盖 | 许多属性/UI/非关键交互 | 局部字段 | 到达后 snap/smooth/merge |
| 不预测，只插值 | 远端实体/低交互对象 | 无 gameplay 预测 | snapshot buffer 插值/有限外推 |

`[Verified][S055][S056]` Photon 在 Input Authority 上超前模拟，收到服务器状态时 rollback 并 resimulate；非权威写会被下一权威状态覆盖。`[Verified][S024]` Unity 强调 extrapolation 与 gameplay prediction 不同。

## K.2 预测状态与权威状态共存

至少要区分：

```text
CommittedAuthoritative(tick A)
PredictionHistory[A+1 ... P]
PendingInputs[A+1 ... P]
RenderedState(time R)
```

权威快照到达 tick `A'`：

1. 验证 snapshot/schema/interest epoch；
2. 将所有子系统恢复到 A' 的共同 checkpoint；
3. 应用权威 ECS/physics/ability/input acknowledgements；
4. 删除已确认输入；
5. 按稳定顺序重演 A'+1..current；
6. 比较状态 hash/误差；
7. 渲染层平滑视觉差异，不改模拟真相。

若只有 ECS 回滚、物理/计时器/随机数不回滚，会产生“位置恢复但碰撞事件没恢复”“Buff 时钟重复扣除”等跨系统分裂。目标画像把所有子系统放在同一确认单元，是强而正确的约束，但要求统一 checkpoint ID。

## K.3 客户端临时实体

`[Verified][S022]` Unity Netcode 有 `PredictedGhostSpawnRequest`，表示客户端预期服务器很快权威创建的 Ghost；`[Verified][S054]` Photon 网络对象有网络唯一 ID并通过网络 spawn 进入 collective state。

预测创建协议至少包含：

```text
PredictedSpawnKey = {ConnectionEpoch, ClientCommandSeq, LocalSpawnOrdinal, PrefabTypeId}
PredictedNetworkId namespace != AuthoritativeNetworkId namespace
```

确认消息需要返回 `PredictedSpawnKey → AuthoritativeNetworkId`，然后一次事务重映射：

- entity identity map；
- 所有 `ReplicatedEntityRef`；
- Transform parent/child；
- owner/authority；
- pending events 和 damage source；
- AOI relation keys；
- prediction history 中的实体集合；
- presentation对象与网络诊断标签。

确认失败：标记 `PredictionRejected`，从重放历史移除对应 spawn command；仅在最终确认的时间线上触发回收表现，避免 rollback 中重复爆炸/音效。

## K.4 整帧粒度与字段/实体粒度

- **整帧快照**：状态边界清楚，跨 ECS/physics/ability 一致；内存和 restore 成本高。
- **实体级历史**：只恢复受影响实体，性能潜力高；跨实体碰撞/引用使依赖图复杂。
- **字段级 undo**：最细但记录量、写屏障与逆操作复杂，难覆盖外部容器。

目标已经选择 fail-stop + 快照/日志重建，不做字段撤销。这个选择与整帧确认相容；关键是快照可增量、热点 state 可双缓冲/环形保存，表现与缓存不进入回滚单元。

## K.5 用户可见预测失败

| 失败 | 表现 | 规避 |
|---|---|---|
| 大位置误差 | snap/拉回 | render smoothing、限制预测窗口、服务器规则同构 |
| 预测实体被拒绝 | 子弹/技能消失 | 延迟不可逆表现、淡出而非立即销毁 |
| 属性纠正 | HP/弹药回跳 | UI 显示 pending/confirmed，事件按 tick 对齐 |
| 父子重映射失败 | 装备跳到原点 | world transform fallback + unresolved parent patch |
| 回放重复副作用 | 重复音效/奖励/RPC | side-effect journal + confirmed outbox |
| 浮点不确定 | 持续小误差与频繁 rollback | 固定顺序、量化/容差、关键算法确定化 |

### K 章来源
S005、S016–S024、S050–S061、S106–S109。

---

# L. 确定性、快照与版本演进

**结论先行**  
**一**：可计算 hash 不等于确定性；必须先冻结迭代、命令、随机、时间、浮点和 Schema 的规范顺序。  
**二**：快照需要一致切点、版本 manifest 与迁移；从内存 dump 恢复不是稳定格式。  
**三**：热更新只有在逻辑版本与状态版本共同进入 checkpoint/回放协议后才可对账。

## L.1 确定性杀手

1. Dictionary/HashSet 枚举顺序；
2. 并行任务完成顺序；
3. 同优先级命令未定义 tie-break；
4. entity ID 分配依赖线程时序；
5. swap-delete/compaction 改变迭代顺序；
6. 浮点 reduction 顺序、FMA/平台实现差异；
7. wall clock、随机种子、GUID；
8. 事件订阅注册顺序；
9. 对象地址/hash code；
10. 异步 IO 完成顺序；
11. schema 反射扫描/程序集加载顺序；
12. 客户端/服务器条件编译使共享系统顺序不同。

`[Verified][S108]` deterministic lockstep 要求同输入产生同模拟结果；低带宽的代价是对 determinism 极端敏感。`[Verified][S086]` sparse-set 删除策略会影响排列，说明物理迭代顺序不能默认是规范顺序。

## L.2 Canonical state hash

稳定哈希输入可定义为：

```text
WorldProtocolVersion
Tick
for entity in canonical order(NetworkId or CreationOrdinal):
  EntityTypeId
  LifecycleState
  for component in StableComponentTypeId order:
    SchemaVersion
    for field in StableFieldId order:
      CanonicalEncodedValue
```

排除项：Model/renderer、GPU handle、缓存、空间索引内部节点、对象池 free list、诊断时间戳、未确认预测 overlay。Transform 派生 world matrix是否进入 hash需冻结：若可由 local+parent 纯重建，通常 hash 源值和拓扑，避免缓存更新顺序影响。

浮点可采用：协议量化值、固定点、规范化 NaN/-0、或只对关键字段哈希。选择本身必须进 Schema 版本。

## L.3 快照类型

| 类型 | 成本 | 恢复 | 用途 |
|---|---|---|---|
| 全量 canonical snapshot | 大 | 简单 | 存档、检查点、迁移基线 |
| 增量 snapshot | 依变化量 | 需 base chain | 高频 rollback/checkpoint |
| Copy-on-write pages | 写放大受变化率影响 | 快 | 内存内回滚 |
| 输入/事件日志 | 小 | 需重演 | 长期审计、对账 |
| 混合 checkpoint+log | 平衡 | 从最近 checkpoint 重演 | 生产恢复常见 |

快照一致切点应在结构 commit、同步状态应用和所有纯状态 System 完成后；外部 outbox 需记录与 checkpoint 的 committed watermark。

`[Verified][S028]` Unity Host migration 从最近 snapshot 创建新 server world并部署 Ghost state；`[Verified][S106]` State Synchronization 文献说明发送状态而非只输入时可逐步收敛，但需序列、优先级和压缩。

## L.4 从快照恢复

恢复不是普通 create：

1. 校验 schema manifest/migration path；
2. 分配实体与组件，不发 gameplay spawn event；
3. 恢复源字段；
4. 重建派生索引、queries、Transform topology、AOI broadphase；
5. 运行纯 `OnHydrate/RebuildCache`；
6. 重建 network observer state并发 fresh Baseline；
7. 恢复 outbox watermark，避免重复外部副作用；
8. 计算 hash 与 checkpoint 记录比较。

## L.5 Schema 版本演进

`[Verified][S102]` Protobuf 明确禁止复用已删除 field number、建议 reserve，且改类型会产生兼容风险；`[Verified][S103]` unknown fields 可被保留。`[Verified][S104][S105]` FlatBuffers 支持加字段、deprecated 字段与显式 id，但字段 id必须连续并遵守 schema 演进规则。

组件 Schema 变更分类：

- 加 optional 字段 + stable default：通常向后兼容；
- 删除字段：保留 ID，旧数据忽略/保留 unknown；
- 改类型/量化：需要新 field ID 或 migration；
- 改默认值：旧存档缺字段时语义改变，需版本化 default；
- 拆/合组件：需要 entity-type migration；
- 改可见性：安全变更可能要求断开旧客户端；
- 改预测角色：历史回放不兼容；
- 改 lifecycle side effect：需要逻辑版本进入 replay。

`[Verified][S018]` 协议 manifest 不匹配时拒绝连接是比“尽量解码”更安全的在线策略；离线存档则可以经显式迁移链升级。

## L.6 热更新

热更新可分：

1. **只换纯逻辑**：状态 schema 不变；下一确认 tick 切版本，日志记录 LogicVersion。
2. **加兼容字段**：新逻辑能读旧默认；客户端版本门仍需决定。
3. **状态迁移**：暂停在 commit 边界，运行 deterministic migration，生成新 snapshot/hash。
4. **跨版本回放**：保留旧逻辑运行时，或在日志中记录迁移点；仅有最新 DLL 无法重放旧 tick。

KBEngine/ET 的脚本/热更能力说明运行时代码可替换，但不能替代 schema/回放版本协议。`[Reported][S063][S091]`

### L 章来源
S018、S025、S028、S076、S085–S086、S100–S109。

---

# M. 工具链与工程化

**结论先行**  
**一**：声明式框架的真实产品是编译器、Schema diff、Inspector、流量诊断和测试 Fixture，不只是 Runtime。  
**二**：生成物必须稳定、可审计且支持 AOT；运行时反射只能留在非热路径或编辑器。  
**三**：组件规模上升后，依赖图、类型 ID、查询与带宽所有权会成为组织问题。

## M.1 编写形态的工作面

| 形态 | 策划改数值 | 程序改结构 | 美术挂资源 | 风险 |
|---|---|---|---|---|
| 代码 Attribute | 需配置层辅助 | IDE 体验好 | 引擎引用易渗入共享层 | 漏标、字段重排、反射/AOT |
| 独立 Schema/IDL | 可配生成工具 | diff 清楚 | 用资源 ID 而非对象引用 | 双文件跳转、生成器复杂 |
| 可视化 Prefab/Authoring | 直观 | 结构 diff 依赖序列化文本 | 最自然 | merge 冲突、隐式默认、批量审计难 |
| 混合 | 数值/资源在内容表，结构/协议在代码/IDL | 边界清楚 | 适配器解析 | 工具链工作量最大 |

`[Verified][S021]` Unity Ghost Inspector/Variant 把端侧组件与 send policy暴露在 authoring；`[Verified][S065]` KBEngine EntityDef 把协议定义独立；`[Verified][S072]` Colyseus Schema 与代码同构。没有一种形态自动胜出，关键是生成 manifest可在 CI审查。

## M.2 代码生成边界

适合生成：

- stable type/field IDs 与 manifest；
- serializer/deserializer、quantizer、hash writer；
- dirty setter/write barrier；
- visibility predicate adapter；
- entity type descriptor、component slots、dependency DAG；
- lifecycle dispatch table；
- source-to-schema compatibility report；
- test fixture codecs。

不宜生成：业务规则、错误恢复策略、跨实体事务、AOI 权限逻辑。生成物是否入库取决于构建可复现性；无论是否入库，都必须能 `--verify-no-diff`，且输出按 stable ID排序。

## M.3 调试设施清单

生产框架至少需要：

- **Entity Inspector**：NetworkId、本地 handle/generation、type、components、lifecycle axes；
- **Query Inspector**：查询条件、匹配 Archetypes/stores、实体数、耗时；
- **Lifecycle Trace**：每实体钩子、命令、失败、重入原因；
- **Replication Inspector**：field dirty、last sent/ack、BaselineId、bits、visibility reason；
- **AOI Visualizer**：spatial cells、observer candidate/final set、enter/leave reason、hysteresis；
- **Reference Resolver View**：pending/tombstoned/forbidden refs；
- **Prediction Timeline**：authoritative tick、predicted tick、rollback count、error；
- **State Hash Diff**：首个不同 entity/component/field；
- **Snapshot Browser**：schema version、size、migration、outbox watermark；
- **Allocation/Pool Dashboard**：每实体类型 alloc、pool miss、retained subscriptions。

Flecs 有 Explorer 生态，Unity 有 Entities/Netcode 日志与 Inspector，Photon 有 Fusion Statistics/AOI gizmos；具体工具名随版本变化，本报告只对已读官方页面中的名称作 `Verified`，其余为能力清单。

## M.4 自动化测试

- EntityType compile tests：依赖、互斥、field ID、bitset上限；
- Lifecycle model-based tests：随机命令序列与状态机不变量；
- Golden schema/manifest：版本 diff必须显式批准；
- Serializer property tests：roundtrip、unknown field、乱序/截断；
- Dual-world tests：同输入 server/client 投影与 hash；
- AOI fuzz：位置、权限、乱序包、重连 epoch；
- Prediction tests：spawn confirm/reject/remap；
- Snapshot migration fixtures：每个历史版本；
- Allocation budgets：热点 API零分配或上限；
- Long soak：无 observer oplog、event subscription、tombstone 内存不增长。

`[Verified][S076][S110]` Schema field和组件 bitset的边界事故说明“启动时拒绝非法定义”必须有自动测试，而不是靠运行后发现随机错字段。

## M.5 大型项目的组织风险

几百组件/几千实体类型会出现：

- 同义组件/Tag 漂移；
- 依赖环与隐式 service locator；
- type ID/field ID 冲突；
- 泛型/生成代码膨胀与 AOT 编译时间；
- 查询组合爆炸；
- 一个“通用”组件拥有几十字段和 64-bit mask上限；
- 无人对字段带宽负责；
- 跨模块生命周期 phase争夺；
- Prefab/Schema/内容表三处定义不一致；
- debug dump 含服务器私有状态。

治理工具包括 component catalog owner、schema review CODEOWNERS、依赖 DAG 可视化、字段预算、deprecated ID registry 和构建 manifest。

### M 章来源
S018、S021、S025、S032、S044、S051、S065、S072、S076–S083、S087–S090、S101–S105、S110。

---

# N. 具体框架深挖

**结论先行**  
**一**：最接近“前后端定义 + 声明同步 + AOI 回调”的是 KBEngine、Mirror/FishNet、Photon 与 Unity Netcode，但它们都不是目标存储模型的完整答案。  
**二**：Flecs/Friflo 提供存储、关系、提交与查询答案，却不提供生产网络复制；两组能力必须解耦组合。  
**三**：没有一个候选同时满足 active component、独立 server/client World、强确定性、整帧回滚、Schema 迁移与 MMO AOI，因此不能照抄单一框架。

## N.1 选择与排除

本章选择八个样本：KBEngine、Mirror、FishNet、Photon Fusion、Unity Entities + Netcode for Entities、Colyseus、Flecs、Friflo.Engine.ECS。它们覆盖 MMO EntityDef、Unity 对象网络框架、tick snapshot/prediction、Schema delta、关系型 ECS 和托管 Archetype ECS。

未单列：

- **Unreal**：用于 Actor/component/lifecycle/relevancy事实对照，但其 UObject、Actor复制、引擎 GC 对象图与目标前提差异过大。
- **EnTT**：作为 Sparse Set核心证据纳入 J，但没有内建网络、AOI与声明同步；与 Flecs/Friflo 的深挖信息重叠。
- **ET**：非常值得关注的 C# 前后端框架，主仓活跃且影响大；但分支演进快，公开社区生命周期材料多为旧版，无法在本次在线检索下把所有符号锁到同一 commit，故只作 B/E/F 的对照而不冒充源码级深挖。
- **GGPO**：是 rollback网络模型，不是完整 Entity/Component框架。

## N.2 KBEngine

### 1. 一句话定位与流派

`[Verified][S063–S069]` 面向 MMO 的服务器 Entity framework：以 EntityDef/属性 flags 划分 base/cell/client 投影，客户端 SDK维护同一网络实体的本地代理，带进入世界/空间的生命周期回调；属于“分布式实体对象 + Schema 生成”，不是纯 DOD ECS。

### 2. Entity / Component / 存储模型

服务器实体按 base/cell职责分布，客户端有实体代理；EntityDef定义属性与远程方法。底层具体 C++存储并非本报告源码级深挖对象；公开文档能确认分布与生成接口，不能据此声称是 Archetype/Sparse Set。

### 3. 生命周期钩子

客户端文档/插件可见 onEnterWorld、onLeaveWorld、onEnterSpace、onLeaveSpace 等；组件 attach与实体进入 world并非同一时刻。`[Verified][S068]` issue #1070 显示 onAttach 早于 cellEntityCall ready，提出需要 enter/leave world通知组件。

### 4. 属性同步

EntityDef属性按 flags决定 base/cell/client/owner/all/others 等可见范围，服务端变化驱动客户端代理更新。具体 flags名称随版本需锁文档；本报告只把“定义文件 + 分布/可见性声明 + SDK生成”标为 Verified。

### 5. AOI / 兴趣管理

cell/space与客户端 world/space回调提供 MMO式实体进出视野语义。离开时客户端代理的保留/销毁细节需按插件版本核对；不将历史 API镜像当现代稳定合同。

### 6. 前后端共用程度

共用 EntityDef/接口语义，不是同一个 Gameplay类库原样在 C++/Python服务器与 Unity客户端执行。

### 7. 性能特征与规模

项目定位是 MMOG server并自报多人在线能力；缺少本次可复现、现代硬件与统一条件 benchmark。主仓约 5.7k stars，但公开 Release线长期低频，生产采用与维护状态需项目级调查。

### 8. 最值得抄的一点

**把“实体属性属于哪个执行位置/哪个客户端集合”写进 Schema，而不是散落在发包代码里。** 这为前后端非对称字段、权限与代码生成提供单源。

### 9. 最不该抄的一点

**不要把 base/cell/client历史分布模型和旧回调名称直接当目标 API。** 它与目标独立 World、C# Gameplay、整帧事务和预测 ID模型不同，且版本线不够新。

## N.3 Mirror

### 1. 一句话定位与流派

Unity `NetworkIdentity + NetworkBehaviour` 对象组件网络库，提供 Weaver生成的 SyncVar、RPC和 InterestManagement；属于对象组合/Active Component网络派。

### 2. Entity / Component / 存储模型

NetworkIdentity是网络对象根；多个 NetworkBehaviour挂在 GameObject上。存储围绕 Unity对象而非独立 ECS World，生命周期与场景/MonoBehaviour深度绑定。

### 3. 生命周期钩子

官方列出 OnStartServer、OnStopServer、OnStartClient、OnStopClient、OnStartLocalPlayer、OnStartAuthority/OnStopAuthority 等。网络启动回调与 Unity Awake/Start并存，说明本地对象就绪与网络角色就绪是两套阶段。

### 4. 属性同步

`[Verified source][S036–S039]` Weaver setter设置 dirty bit；一个 NetworkBehaviour使用 64-bit mask；SyncObject记录操作增量；hook有重入 guard；无 observers时不记录变更列表以免无界增长。

### 5. AOI / 兴趣管理

服务器重建 observers，新增连接看到 spawn，移除连接收到 hide。`[Verified source][S040][S041]` 自有连接被强制保留为 observer，源码关联 teleport/proximity issue #692。

### 6. 前后端共用程度

同一 Unity C# NetworkBehaviour类型运行在服务器/客户端/Host，不同角色通过回调和属性判断分支。纯 dedicated server仍携带 Unity对象模型。

### 7. 性能特征与已知规模

v96.11.2于 2026-08-22发布，约 6.3k stars，活跃。项目方自报 200M玩家与1000+ Steam games；这是厂商/项目方口径，不等于单实例规模。

### 8. 最值得抄的一点

**生成 setter + dirty mask + hook guard，以及“无 observer不积累集合日志、重进发 Baseline”的边界。** 这是声明同步容易漏掉的内存与重入保护。

### 9. 最不该抄的一点

**不要继承 NetworkBehaviour的 64字段/对象实例/Unity生命周期耦合。** 目标需要独立 World、Schema演进和批量存储，不能让每个同步组件等同 MonoBehaviour。

## N.4 FishNet

### 1. 一句话定位与流派

现代 Unity对象网络库，提供 SyncTypes、Prediction、NetworkObserver与可组合 ObserverCondition；同样属于 Active Component网络派。

### 2. Entity / Component / 存储模型

NetworkObject承载多个 NetworkBehaviour，Unity GameObject为物理容器。ObserverManager与场景管理器维护网络可见性。

### 3. 生命周期钩子

`[Verified][S045]` OnStartServer/Client、OnStopServer/Client与 ownership回调独立于 Awake/Start；NetworkObject文档还指出 transform detach/reattach与客户端开始/停止回调的时序，说明网络生命周期会影响层级。

### 4. 属性同步

`[Verified][S044]` SyncVar自动同步单值，可配置变更通知、同步频率等；完整行为受版本/Pro功能影响，报告不推断未公开源码。

### 5. AOI / 兴趣管理

`[Verified][S046–S049]` NetworkObserver组合 Distance、Scene、OwnerOnly、Match等条件；ObserverManager有 Host可见性选项。兴趣规则是连接资格，而不是对象全局 enable。

### 6. 前后端共用程度

同一 Unity组件代码在 server/client/host路径执行，角色分支明显；不适合直接作为无 Unity dedicated server的底层 ECS。

### 7. 性能特征与已知规模

4.7.2R于 2026-04-17发布，约 2.0k stars；Release持续修复 GC与网络平滑问题，说明维护活跃。缺少统一 MMO AOI benchmark。

### 8. 最值得抄的一点

**ObserverCondition可组合且有求值顺序。** 距离、scene、owner、team不应硬编码成一个 AOI if；先廉价条件再昂贵条件也能形成可测性能合同。

### 9. 最不该抄的一点

**不要把 Host renderer visibility当作客户端 replica lifecycle。** 表现隐藏是端侧适配，不应进入服务器权威实体的 Enable/Destroy语义。

## N.5 Photon Fusion 2

### 1. 一句话定位与流派

Unity `NetworkObject/NetworkBehaviour` 上的 tick-based snapshot、prediction、interest与IL-generated `[Networked]` 状态框架；是同步/预测证据最完整的商业样本。

### 2. Entity / Component / 存储模型

NetworkObject拥有网络唯一 ID，NetworkBehaviour状态进入网络缓冲；对象必须通过 Runner.Spawn接入 collective state。运行时源码闭源，内部内存布局只按官方文档描述。

### 3. 生命周期钩子

`Spawned`在对象接入 Runner且网络属性可用后调用；`IAfterSpawned`在批次全部 Spawned后调用；`Despawned`在网络移除前；`FixedUpdateNetwork`是模拟 tick，`Render`是表现回调。

### 4. 属性同步

`[Verified][S051][S052][S055][S061]` `[Networked]` auto-property由 IL生成连接状态缓冲；ChangeDetector可选择 SimulationState/SnapshotFrom/SnapshotTo；`OnChangedRender`不在首次 spawn自动触发。

### 5. AOI / 兴趣管理

`[Verified][S053]` 支持 Area Of Interest、Global、Explicit；Object与Behaviour级裁剪；`IInterestEnter/IInterestExit`携带 PlayerRef。`NetworkTRSP`把用于 AOI的空间位置与 transform同步统一。

### 6. 前后端共用程度

同一 NetworkBehaviour模拟代码可在 State Authority与Input Authority重演；角色和 topology决定执行。服务端秘密仍需拆出共享类。

### 7. 性能特征与已知规模

2.1.2 Stable于 2026-08-13；官方配置暴露 tick/send、scheduling、interest、max data等。没有源码和独立硬件 benchmark，不能用营销容量作结论。

### 8. 最值得抄的一点

**`Spawned`/`IAfterSpawned`、per-object consistency、ChangeDetector source与 InterestEnter/Exit的多轴分离。** 它把网络初始化、批次依赖、状态一致性、渲染通知和兴趣事件分开。

### 9. 最不该抄的一点

**不要复制 NetworkBehaviour绑定 Unity GameObject与商业运行时黑箱。** 目标需要可回放哈希、Native Kernel和自定义Storage，必须拥有Schema与状态事务实现。

## N.6 Unity Entities + Netcode for Entities

### 1. 一句话定位与流派

纯 DOD Archetype ECS与 Ghost snapshot/prediction的官方组合；是存储、结构提交、端侧变体和协议门的关键反向样本。

### 2. Entity / Component / 存储模型

Entity是 World内句柄；相同组件集合形成 Archetype并存于 chunk；查询按组件集合匹配。server/client World独立。

### 3. 生命周期钩子

没有 MonoBehaviour式每组件 Awake/Start；结构 add/remove、enableable状态和系统OnCreate/OnUpdate承担生命周期。Ghost spawn/relevancy通过实体结构出现/消失反映。

### 4. 属性同步

GhostField/GhostComponent/Variant在Bake/生成阶段定义Schema，snapshot按 Baseline delta、量化和owner/端侧策略发送；协议启动检查组件与RPC集合。

### 5. AOI / 兴趣管理

GhostRelevancy按 `(connection, ghost)`过滤；官方文档明确可用于距离和反作弊fog of war。relevancy离开可导致客户端实体销毁，社区案例显示表现重建和Host双World容易踩边界。

### 6. 前后端共用程度

共享组件/系统源码，可通过 server/interpolated/predicted prefab variants形成非对称端侧组合；预测系统在客户端重演。

### 7. 性能特征与已知规模

Archetype/chunk与Burst/Jobs面向批量性能；具体目标工作负载仍需 benchmark。官方文档明确序列化成本随 Ghost数增长，Baseline策略有CPU/带宽交换。

### 8. 最值得抄的一点

**让逻辑 Entity语义独立于server/client World，并把端侧组件集、协议manifest、预测/插值模式编译成 Ghost Schema。** 这直接对应目标的非对称组件和版本门。

### 9. 最不该抄的一点

**不要强迫所有 active component改成纯数据/System，也不要默认纯 Archetype适合频繁能力增删。** 目标编程范式不同，迁移会改变内容团队模型。

## N.7 Colyseus 0.18

### 1. 一句话定位与流派

Node.js authoritative Room + Schema二进制增量 + 多引擎客户端 SDK；是“服务器逻辑不共用、状态Schema共用”的清晰样本。

### 2. Entity / Component / 存储模型

Room维护可变 Schema state；客户端收到生成/手写 SDK对象。不是通用 ECS，实体/组件由应用Schema表达。

### 3. 生命周期钩子

服务器有 Room onCreate/onJoin/onLeave/onDispose；客户端集合有 OnAdd/OnRemove、字段 Listen。网络对象集合变化与Room生命周期分离。

### 4. 属性同步

Schema字段声明后自动change tracking和patch；Room有 patchRate；客户端普通属性不参与同步。0.18对字段索引边界作强校验。

### 5. AOI / 兴趣管理

StateView可按client加入/移除对象/字段视图；旧文档提醒大数据集成本。它更像query/view订阅，不是内建大世界空间索引。

### 6. 前后端共用程度

共用Schema协议，不共享服务器Room gameplay逻辑。客户端SDK跨Unity、Web等平台。

### 7. 性能特征与已知规模

0.18文档2026活跃；binary delta是核心卖点。公开资料没有等价于目标百万实体/高频AOI的统一实测。

### 8. 最值得抄的一点

**明确“只有Schema字段同步”，并把每客户端视图作为单独层；在定义时拒绝字段索引危险布局。** 隐式规则必须可验证。

### 9. 最不该抄的一点

**不要把Room级mutable state/patch cadence直接当ECS事务。** 目标有确定性Tick、结构提交、预测和快照，需要更强的时序与版本语义。

## N.8 Flecs 4.1

### 1. 一句话定位与流派

C/C++关系型、数据导向 ECS，把World做成实体数据库，提供Prefab/IsA/ChildOf、observer/hooks、pipeline和deferred staging。

### 2. Entity / Component / 存储模型

Archetype/table式存储加实体索引；关系以 pair表示；组件可被共享/override；不是对象组件树。

### 3. 生命周期钩子

OnAdd/OnSet/OnRemove、observer与type hooks；事件可同步 emit或enqueue到merge。具体调用时机与defer/merge绑定。

### 4. 属性同步

无内建网络属性同步；可用observer/change tracking构建，但Schema、权限、Baseline、AOI需上层实现。

### 5. AOI / 兴趣管理

无网络AOI；relations/query可表达空间索引元数据，但不是复制策略。

### 6. 前后端共用程度

库本身可在server/client分别运行同一组件Schema；不定义跨端映射。

### 7. 性能特征与已知规模

v4.1.5于2026-03-15，8k+ stars，活跃；项目方宣传million entities。具体目标负载需独立测。

### 8. 最值得抄的一点

**关系、Prefab继承、observer与deferred stage的组合。** 它展示Entity类型、父子、required-like关系和结构事件可以是World数据，而不是硬编码对象引用。

### 9. 最不该抄的一点

**不要把同步observer当网络复制。** 同步emit可能在半状态中重入；网络需要独立staging、Schema与per-connection state。

## N.9 Friflo.Engine.ECS

### 1. 一句话定位与流派

纯托管 C# Archetype ECS，强调高性能、易集成，并同时提供值组件、Tag、Script、关系、索引与序列化。

### 2. Entity / Component / 存储模型

`EntityStore`相当于World；`Entity`为struct handle；值组件进入Archetype，`Script`为可挂实体的class行为。这是目标“数据热点+行为组件”混合的近似样本。

### 3. 生命周期钩子

文档提供组件增删事件与System组织；具体Script完整钩子未在本次锁到单一版本源码，因此不列未验证名称。

### 4. 属性同步

无内建网络复制；JSON序列化与索引不等于网络Schema。

### 5. AOI / 兴趣管理

无内建复制AOI；可用索引/关系构建空间层。

### 6. 前后端共用程度

纯.NET库可在server/client使用同一类型，但端侧裁剪、AOT和协议需应用解决。

### 7. 性能特征与已知规模

v3.6.0发布线，持续维护；官方/项目方benchmark覆盖多种操作。第三方benchmark仍明确不是实际负载，结果只能做候选筛选。

### 8. 最值得抄的一点

**将值组件与Script分轨，同时保持统一EntityStore/Entity句柄。** 这证明active能力不必迫使Transform/Attribute也成为对象。

### 9. 最不该抄的一点

**不要把项目方“最快”口径直接当选型证据，也不要忽视组件类型BitSet边界。** issue #108显示上限缺少校验可能静默映射错误组件，Schema/type catalog必须启动期强校验。

### N 章来源
S016–S029、S030–S090、S093–S096、S109–S110。

---

# O. 踩坑史与反面证据

**结论先行**  
**一**：公开证据最强的事故集中在初始化时序、结构变更重入、字段编码边界、Observer churn与Host双世界。  
**二**：多数事故不是“算法错”，而是两个看似相同的状态被合并：已挂载/已就绪、不可见/已销毁、变更/初始化、字段序号/控制字节。  
**三**：没有找到完全同构路线的公开“整体放弃复盘”；因此本章不虚构宏大失败，只保留可核的小事故与明确推理。

## O.1 继承树为什么被组件化替代

`[Verified][S002]` *Evolve Your Hierarchy* 的核心反例是能力组合导致子类数量增长：既可移动又可渲染、可受伤、可联网的对象无法通过单继承横向复用，修改基类又影响所有子类。组件化用组合替代排列组合式继承。

但组件化的新债务是：原先由构造函数/虚表显式的依赖，变成运行时查找、事件和动态挂载。若不加依赖约束，继承地狱会变成“组件互找地狱”。

## O.2 Unity Awake/Start 的事故根

`[Verified][S006–S008]` Unity 明确把 Awake 与 Start分开，并保证场景对象的 Awake/OnEnable先于任何Start；动态实例化则不能保证全局。事故模式是 A在初始化中依赖B，而B尚未初始化。只提供一个Init钩子会把正确性绑定到编辑器排列或创建顺序。

Photon又增加批次后的 `IAfterSpawned`，说明网络对象在一批spawn中互相依赖时，两阶段仍可能不够。`[Verified][S054]`

## O.3 结构变更为什么延迟

`[Verified][S081]` Flecs文档直说：系统遍历数组时结构变更会让数组重分配并使指针失效，可能 crash；因此 `progress()`期间世界只读、命令排队到sync point。Unity同样要求 Job中的结构变更使用ECB。`[Verified][S011][S012]`

Active component还会叠加钩子重入：OnAdd里Remove自身、OnDestroy里创建子对象、回调中修改正在遍历的订阅表。延迟不仅保护内存，还把重入变成下一批可排序命令。

## O.4 KBEngine：Attach 不等于 EnterWorld

`[Verified][S068]` issue #1070 报告客户端 component的 `onAttach` 发生时，玩家的 `cellEntityCall`还没有建立，组件无法访问预期的服务器cell能力；提议给组件增加 onEnterWorld/onLeaveWorld。这个事故直接证明“组件对象已挂载”“网络实体已进入可用世界”“远端调用端口ready”是三个状态。

## O.5 Mirror：无观察者增量日志与自观察者丢失

`[Verified source][S038]` Mirror对 SyncObject明确在无 observers时不记录变化，源码注释解释否则 change list会持续增长；重新有观察者时走initial state。它说明增量日志的清理条件与可见性紧密相关。

`[Verified source][S040][S041]` InterestManagement重建observer时强制加入对象owner连接，关联issue #692：teleport出 proximity可能把玩家从自己的observer集合移除。事故来自“通用空间规则”覆盖了“owner必须看见自己”的语义约束。

## O.6 Photon：变更通知不是初始化

`[Verified][S052]` `OnChangedRender`不会在对象首次spawn时调用；需在Spawned中主动初始化表现。把“当前值存在”和“值从旧值变化”视为同一事件，会让初始UI/材质缺失；反向若初始Baseline也触发普通变更hook，则可能播放一次本不应发生的受伤/升级动画。

`[Verified][S057]` Photon还区分同对象Full Consistency与classic Eventual Consistency，证明回调看见的原子范围是可配置、且会改变业务语义。

## O.7 Colyseus：第 64 字段导致流脱同步

`[Verified][S076]` 0.18迁移文档说明旧编码中field index 63可能被解码器当作新结构标记；影响不是单字段丢失，而是后续状态字节错位。0.18在定义阶段阻止超过安全字段数量。教训是type/field ID与控制码空间必须形式化验证。

## O.8 Unity Netcode：relevancy、派生表现与Host双World

`[Verified][S026]` relevancy离开会让客户端实体销毁，重进创建新实体；依赖chunk change filter生成的客户端派生数据必须能重建。`[Verified][S027]` Host里粒子系统同时存在于server/client prefab路径，会造成看似隐藏实体仍有粒子或双份表现；解决要从端侧Prefab剔除表现组件。

这两个案例共同说明“Replica不相关”“ECS实体销毁”“表现对象隐藏”“server/client world实体集合”不是一个开关。

## O.9 Friflo：类型上限静默错误

`[Verified][S110]` issue #108 报告当组件类型超过BitSet支持范围时，会意外添加一个随机/错误组件；维护者计划增加异常和扩展位数。与Colyseus案例相同，声明系统的边界若不在构建期拒绝，会把可诊断配置错误变成运行时数据污染。

## O.10 没找到的证据

本次没有找到以下可核材料：

- 一个大型项目公开披露因为“组件带逻辑 + 前后端同类定义”而整体迁移到纯DOD的完整成本报告；
- 同一硬件下对Mirror/FishNet/Photon/Unity NfE进行AOI enter churn的公开横测；
- 主流框架公开给出通用hysteresis/dwell参数；
- 对“每实体逐字段回调 vs 批量事务回调”做生产事故统计的论文。

因此相关架构结论在P章标为建议或推断，不伪装成行业统一事实。

### O 章来源
S002、S006–S012、S026–S027、S038–S041、S052、S054、S057、S068、S076、S081、S110。

---

# P. 对目标框架的逐条审视、建议与风险分级

> 本章是报告唯一的规范性意见章。A–O 的事实与事故在这里转化为目标框架的决策建议。

**结论先行**  
**一**：保留“组件组合 + 组件可带逻辑”，但必须把 World、生命周期、结构事务、复制事务和确定性协议做成比普通对象模型更严格的硬边界。  
**二**：立即拆开 Alive、Committed、Enabled、Server Interest、Client Replica Residency、Presentation Ready、Dormant、Authority/Prediction 八个状态轴。  
**三**：Storage 采用可替换混合方案；在目标工作负载 benchmark 通过前，不冻结为纯 Archetype或纯对象字典。

## P.1 十条设计洞察

### 洞察 1：给这条路线一个准确名称，并把承诺写窄

目标应命名为 **Object-Composition / Active-Component ECS Hybrid**，而不是泛称“高性能 ECS”。它承诺的是稳定身份、World隔离、查询、结构事务、同步与回放；不承诺任意组件对象图也能获得DOD性能。这样可以让内容开发继续使用局部对象逻辑，同时强制热点数据和跨实体规则进入可批处理路径。

### 洞察 2：实体生命周期必须从线性链改成正交状态轴

当前六钩子把初始化、AOI、禁用和销毁压成单链，无法表达“对玩家A离开、对玩家B仍在”“逻辑副本保留但模型卸载”“Dormant但仍可查询”。框架应把状态轴独立存储，并让事件命名包含端、观察者与原因。线性钩子可以保留为便捷映射，但不能是规范真相。

### 洞察 3：`Awake` 是局部不变量，`Start` 是发布后的依赖屏障

创建时先完整分配实体类型的所有组件并建立slot，再执行Awake；Awake只能访问同实体已存在的组件槽，不得依赖其Start或外部实体。所有Awake成功、依赖图解析后才发布实体并按拓扑执行Start。任何钩子失败都使本次初始化事务整体不发布。

### 洞察 4：声明同步必须由编译器产生协议，而不是由setter直接发包

同步Attribute只声明意图；生成器产出稳定ID、serializer、dirty barrier、权限和hash，setter只标记本地写版本。帧提交后复制层按每连接Interest/Baseline/预算选择字段，绝不在业务setter里发网络消息。这样才能在同一tick合并多次写、支持回滚并阻止远端apply形成回声。

### 洞察 5：把网络接收定义为原子 Replication Transaction

每个接收批次先校验schema/epoch与去重，再在staging里构造实体/组件、应用全部字段、解析引用和验证不变量；成功后一次发布revision，再按稳定顺序发回调。初始Baseline、普通Delta、权威纠正和重连Refresh使用不同reason。任何字段解码或引用约束失败都不能让查询看到半状态。

### 洞察 6：AOI 是 `(Observer, Entity)` 关系；客户端驻留是另一层

服务器Interest负责数据是否披露、以何优先级复制；客户端Replica Residency决定保留、软离开、销毁；Presentation Residency再决定模型/粒子/音频。三层可以共享空间信号，但事件和TTL不同。安全型leave立即生效，性能型leave才能使用hysteresis/dwell。

### 洞察 7：跨实体引用必须有“未解析”作为一等状态

同步字段不存C#对象引用，也不假设目标已在AOI；它存NetworkEntityId，并解析为带generation/epoch的handle。目标稍后进入、预测ID确认或快照恢复时，由patch table在确定性phase完成解析。业务可明确选择Pending时忽略、等待、使用fallback或要求依赖闭包。

### 洞察 8：Transform 采用多视图、单权威写入

同一Transform需要权威模拟值、预测值、渲染值与派生world matrix；物理、动画和脚本不能在同phase同时成为writer。父子变更作为结构命令，commit时检测循环、排序父先子并更新空间索引。网络同步默认发送local TRSP + parent id；父不可见时由schema指定world fallback或依赖闭包。

### 洞察 9：物理存储顺序与规范执行顺序解耦

Archetype/Sparse Set为了性能会搬迁和swap-delete，不能直接决定回放顺序。实体表保存稳定CreationOrdinal或按NetworkId canonical sort；query可选择FastUnordered或DeterministicOrdered两种模式。状态hash永远按canonical schema顺序编码，绝不hash内存布局。

### 洞察 10：版本、快照、日志和热更新必须共享同一 Manifest

每个checkpoint记录SchemaManifestHash、LogicVersion、ConfigVersion、Tick、RNG state和Outbox watermark。在线连接不兼容即拒绝；离线存档通过显式migration chain升级。热更只能在commit边界切换，回放要么保留旧逻辑，要么记录确定性迁移点。

## P.2 完整性缺口登记表

分级定义（使用委托书要求的原始分级词）：

- **必须现在补**：不冻结就会污染公共 API、存档、协议或确定性边界，后补代价极高。
- **可以推迟但要预留**：Vertical Slice 可以先简化实现，但现在必须冻结可扩展接口、稳定 ID 或协议位置。
- **明确可以不做**：与本项目定位冲突，或成本收益不成立；不是“以后再看”。

| # | 画像未定义的能力/语义 | 同类框架如何处理 | 不补时爆炸场景 | 优先级 |
|---:|---|---|---|---|
| 1 | 生命周期正交状态轴 | UE dormancy/relevancy、FishNet observer、Unity enableable分离 | 对一个玩家leave触发全局Disable，服务器AI停止 | 必须现在补 |
| 2 | Entity Constructing/Published状态 | Photon Spawned/AfterSpawned，Unity结构可见边界 | 查询读到默认值半实体 | 必须现在补 |
| 3 | Awake允许访问范围 | Unity Awake/Start分层 | 组件注册顺序改变结果 | 必须现在补 |
| 4 | Start依赖DAG与循环检测 | 批次后回调、System order | A等B、B等A或隐式空值 | 必须现在补 |
| 5 | 钩子重入规则 | Flecs defer/merge | OnAdd里Remove导致迭代失效/递归 | 必须现在补 |
| 6 | 钩子失败的发布/回滚 | 目标fail-stop尚无细则 | 部分组件已注册、实体仍被查询 | 必须现在补 |
| 7 | 恢复专用OnHydrate语义 | 快照恢复通常重建状态而非重放副作用 | 重启后重复发奖励/音效/外部写 | 必须现在补 |
| 8 | 组件Required/Mutex/OneOf约束 | Prefab/relations/authoring validation | 运行时缺Transform或双物理writer | 必须现在补 |
| 9 | 稳定ComponentTypeId | Schema/ECS registry | 重排注册后存档与hash全变 | 必须现在补 |
| 10 | 稳定FieldId与保留策略 | Protobuf/FlatBuffers | 删除后复用ID，旧客户端错解 | 必须现在补 |
| 11 | Schema manifest/兼容门 | Unity protocol checks | 不兼容客户端继续运行并污染状态 | 必须现在补 |
| 12 | EntityType版本与迁移 | Prefab/Schema migration | 老存档无法升级、动态组件集合错位 | 必须现在补 |
| 13 | Replication Transaction原子范围 | Photon per-object consistency | HP与Dead分包，回调见半状态 | 必须现在补 |
| 14 | BaselineId/InterestEpoch | Snapshot/AOI框架隐含维护 | 重进AOI吃到旧delta | 必须现在补 |
| 15 | ConnectionEpoch去重 | 常见会话协议 | 重连后旧包对新实体生效 | 必须现在补 |
| 16 | Dirty层级与mask扩展 | Mirror bitmask、Unity字段schema | 字段过多或绕过setter不发 | 必须现在补 |
| 17 | 可变集合操作日志 | Mirror SyncObject/Colyseus collections | 直接List修改绕过dirty；无observer内存涨 | 必须现在补 |
| 18 | Remote apply禁止回声dirty | 生成setter/authority路径 | 客户端收到状态又回发 | 必须现在补 |
| 19 | 初始/变化/纠正回调区分 | Photon Spawned vs OnChanged | 初始加载播放受伤动画或不初始化UI | 必须现在补 |
| 20 | 字段/组件可见性默认 | Ghost variants/Interest/StateView | 漏标导致AI意图、库存、迷雾信息泄露 | 必须现在补 |
| 21 | 跨实体网络引用 | UE network refs | 目标不在AOI，null永久化或悬挂 | 必须现在补 |
| 22 | Unresolved reference patch table | 网络对象解析层 | 同包后创建/预测remap丢关联 | 必须现在补 |
| 23 | Tombstone保留期与原因 | entity version/dead cache | 迟到包复活实体或误指新对象 | 必须现在补 |
| 24 | 预测ID匹配/拒绝/重映射全图 | Predicted Ghost/NetworkId | 子弹确认后owner/parent仍指临时ID | 必须现在补 |
| 25 | 权威/预测/渲染字段分层 | Photon/Unity prediction | 一个字段被预测、网络和插值互相覆盖 | 必须现在补 |
| 26 | Canonical系统/事件/命令顺序 | DOD pipeline/lockstep | 不同线程merge产生不同hash | 必须现在补 |
| 27 | RNG/时间/外部输入协议 | rollback/lockstep | 回放调用墙钟或随机数分叉 | 必须现在补 |
| 28 | State hash规范编码 | 协议schema/canonical snapshot | hash受字典/内存地址影响 | 必须现在补 |
| 29 | Snapshot切点与Outbox watermark | checkpoint+log | 恢复重复外部副作用 | 必须现在补 |
| 30 | Transform writer ownership | Transform/physics/network frameworks | 物理与脚本同tick覆盖，回放不稳 | 必须现在补 |
| 31 | Parent变更、循环检测、不可见父策略 | Unity/Fusion TRSP | 子物体原点跳变或AOI依赖裂开 | 必须现在补 |
| 32 | Server Interest与Client Residency分离 | Observer/interest/dormancy | 安全leave被客户端TTL延迟 | 必须现在补 |
| 33 | AOI hysteresis/dwell分类 | 行业常见，框架多留自定义 | 边界每tickspawn/despawn风暴 | 必须现在补 |
| 34 | AOI dependency closure | parent/owner always interest | 装备先于角色，引用无法解析 | 必须现在补 |
| 35 | AOI enter预算与饥饿上限 | Photon scheduling | 传送后关键实体迟迟不可见 | 可以推迟但要预留（接口必须现在补） |
| 36 | Dormancy/LOD复制 | UE dormancy、NfE importance | 大量静态实体仍逐tick参与 | 可以推迟但要预留 |
| 37 | Query snapshot/迭代失效语义 | ECB/staging ECS | 遍历中提交导致重复/漏实体 | 必须现在补 |
| 38 | Fast vs Deterministic query模式 | 存储与canonical顺序分离 | 为了hash把所有查询强制排序，性能崩 | 必须现在补 |
| 39 | 组件/实体池化重置合同 | NetworkObjectProvider/pools | 旧订阅、timer和handle污染新实例 | 必须现在补 |
| 40 | managed/native批量边界 | DOD批处理 | 逐实体跨边界吞掉所有存储收益 | 必须现在补 |
| 41 | Lifecycle/Replication/AOI trace | 主流工具/统计 | 半状态只能靠日志猜，无法复现 | 可以推迟但要预留 |
| 42 | Schema diff CI与ID registry | Proto/FBS/Unity manifests | 团队重用字段ID或隐式breaking change | 必须现在补 |
| 43 | 长期migration fixture corpus | 序列化系统实践 | 发布半年后才发现旧存档不可读 | 可以推迟但要预留（格式必须现在补） |
| 44 | 组件目录Owner/带宽Owner | 大项目治理 | “通用组件”无限长，没人负责网络成本 | 可以推迟但要预留 |
| 45 | 多World引用类型 | Unity server/client Worlds | 本地handle误投到另一World | 必须现在补 |
| 46 | 跨帧异步结果注入协议 | deterministic simulation | IO完成顺序改变模拟 | 必须现在补 |
| 47 | 安全型与性能型AOI reason | fog-of-war vs distance | 隐身实体因soft leave继续泄露 | 必须现在补 |
| 48 | 客户端表现重建合同 | relevancy重进案例 | 粒子/UI/模型留残影或重复创建 | 可以推迟但要预留 |
| 49 | 热更新LogicVersion切点 | 脚本热更框架 | 同一日志跨代码版本无法重放 | 可以推迟但要预留（Manifest 必须现在补） |
| 50 | 任意运行时反射式同步 | 部分早期框架依赖反射 | AOT失败、热路径分配、难审计 | 明确可以不做；只保留编辑器反射 |
| 51 | 字段级通用undo | 复杂事务系统 | 写屏障和逆操作覆盖不完整 | 明确可以不做；沿用整帧快照/日志 |
| 52 | 跨World裸对象引用 | 对象模型常见捷径 | 世界销毁/回滚后悬挂 | 明确可以不做 |

## P.3 生命周期钩子对照表与精确定义

### P.3.1 推荐状态轴

| 轴 | 状态 | Owner | 是否进确定性 hash |
|---|---|---|---|
| Existence | Allocated / Alive / Tombstoned / Reclaimed | World | Alive/Tombstone 关键字段进入 |
| Structural | Recording / Committing / Published | World transaction | Published 结构进入 |
| Runtime Enable | Enabled / Disabled | Gameplay | 进入 |
| Authority | ServerAuthority / Predicted / Interpolated / PresentationOnly | Replication | 进入协议相关部分 |
| Server Interest | per connection Absent/Pending/Present/SoftLeave | Server AOI | 观察关系通常不进核心世界 hash，可单独 hash |
| Client Replica | Absent/Constructing/Resident/Cached/Despawning | Client replica | 权威副本字段进入客户端诊断 hash |
| Presentation | Unrequested/Loading/Ready/Hidden/Released | Client presentation | 不进入模拟 hash |
| Dormancy | Active/Dormant/Waking | Replication/simulation policy | policy 状态按需要进入 |

### P.3.2 画像六钩子与主流框架映射

| 画像钩子 | Unity MonoBehaviour | Unreal Actor/Component | Unity Entities / Flecs | Mirror / FishNet | Photon Fusion | KBEngine | 双向缺口判断 |
|---|---|---|---|---|---|---|---|
| `Awake` | `Awake`：实例加载/构造后；场景对象通常先于任何 `Start` | `PostLoad`/`PostActorCreated`、`InitializeComponent` 等多阶段 | “组件已存在/结构提交后”由系统或 `OnAdd` 观察；无统一对象式 Awake | Unity `Awake` 早于网络角色回调 | `Spawned` 是对象已进入 Runner 时间线后的网络初始化，不等同未发布构造 | entity/component attach；历史 issue 表明 attach 可早于远端 cell call ready | 画像有相似名，但缺少 Constructing/Published、允许访问范围、失败原子性 |
| `Start` | `Start`：第一次帧更新前，且通常在场景对象 `Awake` 后 | `BeginPlay`；组件/网络状态可能经历更多 ready 阶段 | System 初次匹配/自定义 phase；Flecs 可用 observer + pipeline | `OnStartServer` / `OnStartClient` / ownership callbacks 分角色 | `IAfterSpawned` 在同批对象 `Spawned` 后；另有 simulation callbacks | `onEnterWorld`、remote-call readiness 等并非单一 Start | 画像缺少依赖 DAG、网络角色 ready、authority/prediction ready |
| `OnEnterScene` | Unity Scene load/enable，不是网络兴趣 | Relevancy 是 per connection；不是 Scene/Activate | Ghost relevancy 或应用自定义关系；Flecs 无网络内建 | Observer add 后向连接 spawn；与 GameObject enable 分离 | `IInterestEnter(PlayerRef)` 明确是每玩家关系 | world/space/view enter 是最近先例，但含义随端/空间层次变化 | 画像独有的模糊合并项；必须拆为 ServerObserverEnter、ReplicaBaselineCommitted、PresentationReady |
| `OnLeaveScene` | Scene unload/disable，不是网络兴趣 | relevancy、channel close、dormancy 各有不同后果 | Ghost 默认可在客户端销毁；应用也可缓存；Flecs 无网络内建 | Observer remove 后 hide/despawn；对象本身服务器仍 alive | `IInterestExit(PlayerRef)`；本地对象/表现策略另定 | world/space/view leave；不必等同全局 destroy | 画像缺少 reason、epoch、hard/soft leave、缓存与重进语义 |
| `OnDisable` | 停止常规脚本更新；对象/组件仍存在 | `Deactivate`、tick/组件激活与网络 relevancy/dormancy 独立 | Enableable component 可从默认 query 过滤但不移除存储 | Unity enable 与网络 observer 独立 | simulation inclusion、render 与 interest 是不同面 | 由应用定义，非统一网络语义 | 画像有钩子但未定义：是否 tick、query、sync、AOI、事件、引用可见 |
| `OnDestroy` | 对象真正销毁时；与临时隐藏不同 | `EndPlay` / `Destroyed`，原因可区分 | Entity destroy / `OnRemove`，结构生效点受 ECB/defer 控制 | `OnStop*` + object destroy/despawn，多网络角色回调 | `Despawned` | entity removal/leave callbacks 需按层区分 | 画像缺少 query removal 时点、tombstone、池化 reset、快照恢复与副作用规则 |

**别人有而画像没有的钩子/阶段**：网络角色 ready（server/client/owner）、批次 Spawn 后屏障、Authority change、Interest per observer、Dormancy/Wake、Hydrate/Restore、Predicted Spawn confirm/reject、Presentation ready/release。`[Verified][S006][S029][S034][S045][S052–S055][S066][S081][S093–S096]`

**画像有而别人没有统一对应物的钩子**：同时代表“服务器观察关系”和“客户端副本/表现进入”的 `OnEnterScene` / `OnLeaveScene`。各框架把 Scene、relevancy、observer、replica destruction、dormancy 与 renderer visibility 分开，因此这两个名称只能作为端侧兼容别名，不能成为规范状态。`[Verified][S026][S029][S040][S046–S049][S053][S073][S096]`

### P.3.3 保留六钩子时的规范化

| 现有钩子 | 精确定义 | 允许读取 | 允许写入 | 不允许 |
|---|---|---|---|---|
| `Awake` | 组件已分配、同实体完整 slot 可见，但实体尚未发布给普通查询；建立局部不变量 | 自身字段、同实体组件存在性/只读默认 | 自身状态；记录后续命令 | 跨实体查找、网络发包、外部副作用、立即结构变更 |
| `Start` | 所有组件 Awake 成功、required 依赖解析、实体 Published 后；按依赖 DAG 运行一次 | 同 World 已 Published 实体与服务端口 | 状态与命令缓冲 | 假设 Model ready；不可逆外部副作用直接执行 |
| `OnEnterScene` | **废弃模糊规范名**；兼容层仅映射客户端 `OnReplicaInterestEntered`，且在完整 Baseline 原子提交后触发 | 已完整 Baseline 的副本 | 本地缓存/命令 | 同时代表 server AOI 与 client 表现 |
| `OnLeaveScene` | **废弃模糊规范名**；兼容层映射带 Reason/Epoch 的 replica residency 事件 | 离场前快照，按 policy | 清本地非权威状态 | 默认 Destroy、默认 Disable 全局逻辑 |
| `OnDisable` | Runtime Enable 从 Enabled→Disabled；与网络兴趣无关 | 组件状态 | 清 tick 订阅/生成命令 | 删除组件、释放 network identity、改变其他连接可见性 |
| `OnDestroy` | 实体已从普通查询移除、组件仍只读可访问；一次性最终释放 | 自身/同实体只读终态、tombstone 元数据 | 仅清非状态资源、写幂等 outbox | 创建可见状态、直接发不可去重副作用 |

### P.3.4 新增专用钩子

- `OnHydrate(HydrationContext)`：从快照/存档恢复非序列化缓存；可重跑、无外部副作用。
- `OnAuthorityChanged(old,new,tick)`：权威/预测切换。
- `OnServerObserverEnter(ConnectionId, InterestEpoch)` / `Exit`：仅服务器，每连接。
- `OnReplicaEnter(InterestEpoch, BaselineId)` / `Exit(reason)`：仅客户端逻辑副本。
- `OnPresentationReady(ResourceRevision)` / `Released`：仅客户端表现。
- `OnPredictionConfirmed/Rejected(PredictedSpawnKey)`：预测实体专用。
- `OnReferenceResolved(FieldId, NetworkEntityId)`：可选，由 patch phase 批量发。

## P.4 属性同步能力对照表与决策矩阵

### P.4.1 对照画像的当前状态

| 能力维度 | 画像状态 | 画像已给出的内容 | 仍需冻结的决定 |
|---|---|---|---|
| 声明形态 | 已有（原则）/需要决策（机制） | “声明即同步”，业务不手写发送 | Attribute/IDL/source generator 何者为规范源；稳定 Component/Field ID |
| 脏标记粒度 | 未提 | 无 | 字段 mask、component revision、超 64 字段扩展、可变集合 oplog |
| 变更检测 | 未提 | 权威侧变化会推送 | 写屏障、生成 setter、帧末 diff 或 revision；如何检测绕过写屏障 |
| 通知时机 | 未提 | 无 | 逐字段立即还是事务提交后；初始/变化/纠正/重放 reason；稳定回调顺序 |
| 可见性与权限 | 未提 | 仅说明服务器→本地客户端 | 默认 deny；owner/team/observer/服务器私有；实体/组件/字段三层裁剪 |
| 调度元数据 | 未提（传输预算属 DS） | 无 | 频率、优先级、可靠性、LOD、依赖组；ECS 向复制调度器暴露什么 |
| Baseline/Delta | 未提 | “变化即推送” | 首次 AOI、晚加入、重连 Baseline；acknowledged baseline；interest epoch |
| 失败处理 | 已有（Tick fail-stop）/需要决策（复制） | 整帧作废、快照+日志恢复 | 解码失败、schema 不兼容、旧包/重复包、半实体、引用未解析如何拒绝/重试 |
| 集合增量 | 未提 | 无 | 稳定 key、operation sequence、压缩/重置、观察者为零时是否保留日志 |
| 跨实体引用 | 未提 | Network identity 存在 | NetworkId→local handle 解析、pending patch、generation/epoch、预测重映射 |
| 预测字段 | 已有（原则）/需要决策（表达） | 客户端有界预测、整单元回滚 | 权威值与预测 overlay 是否分 lane；纠正、插值、渲染值的所有权 |
| Schema 演进 | 未提 | 无 | Manifest、兼容门、reserve ID、离线 migration、旧客户端/旧存档策略 |

### P.4.2 推荐决策矩阵

| 主题 | 推荐默认 | 可选变体 | 禁止/警告 |
|---|---|---|---|
| 声明 | C# source generator + 独立manifest | 外部IDL生成partial types | 热路径运行时反射 |
| Component ID | 显式稳定32-bit/64-bit ID registry | 哈希+碰撞锁定清单 | 按加载顺序编号 |
| Field ID | 显式/生成后冻结，删除reserve | schema-specific compact index映射 | 复用旧ID、仅按声明序 |
| Dirty | 字段mask + component revision | 大组件分lane；集合oplog | setter直接send；只做World diff |
| 写屏障 | 生成setter/API；debug检测绕过 | unsafe批量writer需显式MarkDirty | 暴露可变List/ref后不追踪 |
| Apply | staging + entity/component原子publish | consistency group跨组件 | 逐字段写入即回调 |
| 回调 | commit后稳定排序，携带reason/tick/revision | 汇总component changed | 初始Baseline伪装成普通change |
| Baseline | enter/late join/reconnect发当前完整状态 | 缓存复用需epoch/schema一致 | 回放全部历史dirty代替Baseline |
| Delta | 对acknowledged baseline编码 | 1或3 baseline按字段特性 | 无baseline强行解码 |
| 权限 | 默认deny，实体/组件/字段三层 | 派生/模糊值 | 客户端收到后再隐藏 |
| 引用 | NetworkId + unresolved patch | dependency closure | 裸C#引用/本地Index上网 |
| 集合 | stable key + op seq + compaction | 小集合整值替换 | 哈希枚举顺序上网 |
| Attribute | 同步权威Current+revision；Base低频 | 预测overlay、本地平滑 | 默认同步Modifier内部全表 |
| 事件 | 独立event id/tick/幂等；状态最终兜底 | reliable transition | 用短命bool字段模拟一次性事件 |
| Schema兼容 | connect manifest gate + offline migration | negotiated optional features | “尽量继续”解码未知breaking schema |

### P.4.3 建议的最低协议元数据

```text
ReplicationEnvelope {
  ProtocolVersion
  SchemaManifestHash
  ConnectionEpoch
  ServerTick
  SnapshotSeq
  BaselineRefs[]
  EntityTransactions[]
}
EntityTransaction {
  NetworkEntityId
  EntityLifecycleGeneration
  InterestEpoch
  EntityTypeId
  Operation = Baseline | Delta | Despawn | Refresh
  ComponentRecords[]
}
ComponentRecord {
  StableComponentTypeId
  SchemaVersion
  ComponentRevision
  FieldMask / CollectionOps
}
```

这不是传输封包选型，而是ECS复制层交给DS网络栈的稳定语义。

## P.5 AOI 能力对照表与决策矩阵

### P.5.1 对照画像的当前状态

| 能力维度 | 画像状态 | 画像已给出的内容 | 仍需冻结的决定 |
|---|---|---|---|
| Server Interest 关系 | 已有（概念）/需要决策（键） | 服务器 AOI 驱动 Enter/Leave | 必须是 per connection/observer relation；关系 ID、reason、epoch |
| Client Replica Residency | 已有（概念）但与 Server AOI 混合 | “客户端 AOI” | Resident/Cached/Absent、TTL、内存压力驱逐、重进是否复用副本 |
| Presentation Residency | 未提 | `Model` 负责表现资源 | Loading/Ready/Hidden/Released 与副本 AOI 分离；异步完成回调 |
| 候选空间索引 | 未提 | Transform 为核心 | Grid/quadtree/BVH/PVS 适配口；teleport、跨 cell、大对象处理 |
| 安全与语义过滤 | 未提 | 无 | security→owner/team→scene→dependency→priority 的顺序与默认 deny |
| Enter 语义 | 未提 | 仅有 `OnEnterScene` | 候选命中不是 enter；Baseline 原子提交后才 publish/回调；分帧预算 |
| Leave 语义 | 未提 | 仅有 `OnLeaveScene` | hard security leave、soft performance leave、dormancy、destroy/hide/cache |
| 抖动抑制 | 未提 | 无 | 双阈值、dwell、cell padding、soft cache；安全 leave 不得延迟 |
| Interest Epoch/去重 | 未提 | Network identity/tombstone 原则 | fresh interest lifecycle 编号、旧 delta/leave 拒绝、重连 epoch |
| 依赖闭包 | 未提 | Transform/Model/能力组件存在 | parent/owner/equipment/target 的 closure、fallback 或 unresolved reference |
| 优先级与预算 | 未提（DS 负责带宽预算） | 无 | ECS/AOI 输出优先级、baseline byte estimate、queue age、starvation bound |
| 大世界/分块 | 未提 | 有 World/Scene/AOI 概念 | AOI 与地图 chunk 生命周期、跨边界 ghost、迁移/预取接口 |
| 诊断 | 未提 | 无 | relation reason、cell、epoch、queue、bytes、churn、baseline storm 指标 |

### P.5.2 推荐决策矩阵

| 主题 | 推荐默认 | 可选 | 禁止/警告 |
|---|---|---|---|
| 关系键 | `(ConnectionEpoch, ObserverId, NetworkEntityId)` | 多camera/view per connection | `Entity.InAOI`单布尔 |
| Broadphase | 分区哈希网格，按世界密度可替换 | quadtree/BVH/预计算PVS | 先逐observer扫全World |
| 过滤 | 空间候选后：security→owner/team→scene→dependency→priority | 自定义condition DAG | 客户端决定安全可见性 |
| Enter | 通过权限和预算后生成Baseline | placeholder + later publish | 候选命中立刻发业务Enter |
| Leave | reason分类：security hard、performance soft | dormancy/LOD | 所有leave都Destroy或都TTL |
| 抖动 | 双阈值 + dwell；参数按速度/RTT实测 | cell padding、predictive prefetch | 写死通用百分比 |
| Epoch | 每fresh interest lifecycle递增 | soft reenter可refresh | 仅NetworkId去重 |
| 依赖 | owner/parent/required refs closure或fallback | schema逐字段策略 | 假设引用目标总可见 |
| 大量进入 | priority queue + per-tick baseline budget + starvation bound | chunk baseline | 无上限同帧创建/回调/加载 |
| 客户端副本 | Resident/Cached/Absent独立状态机 | 内存压力强制evict | 与server observer集合共用一状态 |
| 表现 | PresentationReady单独异步 | 预取/LOD | Model加载完成触发网络Enter |
| 诊断 | reason mask、cell、epoch、baseline bytes、queue age | 可视化 | 只记录“进入/离开”无原因 |

### P.5.3 服务器与客户端事件命名

- Server: `ObserverBecameInterested`, `ObserverLostInterest`
- Client replica: `ReplicaBaselineCommitted`, `ReplicaBecameIrrelevant`, `ReplicaEvicted`
- Presentation: `PresentationRequested`, `PresentationReady`, `PresentationHidden`

不要继续使用无端侧、无observer、无reason的 `OnEnterScene` 作为唯一语义。为兼容现有设计，可将其保留为客户端 `ReplicaBaselineCommitted` 后的简化回调，并在文档中明确它从不在服务器表示“实体进入全局场景”。

## P.6 Storage 建议

### P.6.1 推荐布局

采用四层混合：

1. **Stable Entity Table**：`Index + Generation + Alive + EntityTypeId + Location + CreationOrdinal + NetworkIdRef`；句柄只指表，不暴露组件地址。
2. **Managed Active Component Stores**：每组件类型 paged sparse→dense slot，实例来自池；适合低频能力、AI状态机、脚本；移除不搬其他组件。
3. **Hot Data Lanes**：Transform local/world、AOI bounds、Velocity、常用Attribute用struct SoA/Archetype column；System批量处理。
4. **Entity Type Prototype**：预计算组件集合、store reservations、默认blob、依赖DAG、schema、lifecycle dispatch；批量instantiate。

核心API对存储保持中立：`Has<T>`, `TryGet<T>`, `Query`, `CommandBuffer`不能泄漏chunk地址。后续可以把某组件从managed store迁到column，而不改变内容层语义。

### P.6.2 为什么不是纯 Archetype

目标允许active component、组件协作与动态能力，若所有组件都决定Archetype，频繁增删会搬迁大量托管引用/热数据并使组件地址不稳定。纯Archetype适合组件集合相对稳定、数据值化、查询批量的区域；可作为Hot Data Lane和固定实体类型的优化，而不应成为全部行为组件的唯一容器。

### P.6.3 为什么不是纯 Sparse Set/对象字典

Transform/AOI/Attribute是高频多组件联合扫描，纯Sparse Set需membership probe且数据跨store，纯对象字典又有指针追逐/GC。对这些热组合，Archetype/SoA列能显著提高局部性和批量native调用效率。

### P.6.4 冻结前必须通过的 benchmark Gate

候选A：纯Archetype；B：每类型Sparse Set；C：上述Hybrid。所有候选必须共享同一EntityHandle、命令、生命周期和Schema实现，以免比较不同语义。

**Gate场景：**

- 100万server实体的entity table；10万活跃Transform/AOI；1万active gameplay；
- 8/16/32组件的固定类型批量创建10万；
- 每tick 1%与10%实体增删2个能力组件；
- 2/3/5组件查询，Fast与Deterministic两种；
- 每tick 1%/10%同步字段变化；
- 1000 observer，每人100/500/2000 interest entities的set diff（可缩放分批跑）；
- 同时1k/10k enter Baseline；
- 保存32 tick rollback history并回滚8 tick；
- canonical hash；
- C# JIT、目标AOT/IL2CPP可行版本、Linux server；
- managed/native批量边界。

**验收指标：** p50/p95/p99 commit与tick时间、allocated bytes/tick、GC pause、RSS、cache miss、moved bytes、encode/decode bytes与CPU、snapshot memory、restore/resim时间。任何只给平均吞吐不报告尾延迟和分配的结果不通过。

## P.7 风险分级

### 一定会踩：架构级，必须在首个公开协议/存档前关闭

1. AOI与Enable/Scene合并；
2. 无stable component/field/entity type ID和schema manifest；
3. 无Replication Transaction、Baseline/Epoch/去重；
4. 跨实体引用无unresolved/remap；
5. 生命周期发布屏障、依赖、重入、失败、恢复未定义；
6. Transform多writer/parent/AOI语义未定义；
7. canonical order、RNG/time、state hash与snapshot切点未定义；
8. 预测实体只改ID、不重映射全图；
9. server/client端侧组件和安全可见性没有构建期裁剪；
10. Storage API泄漏具体地址/布局，导致未来无法迁移。

### 大概率会踩：规模和运维风险，Vertical Slice 前留口，规模测试前实现

1. AOI hysteresis/dwell/soft cache与enter预算；
2. dormancy/LOD/importance；
3. managed component pool reset；
4. schema migration fixture；
5. lifecycle/replication/AOI trace工具；
6. long soak内存增长检测；
7. async result注入/outbox；
8. hot lane自动选择与跨native批处理；
9. query plan/索引诊断；
10. client presentation重建和资源预算。

### 视规模而定：可后做的增强

1. 可视化entity graph编辑器；
2. 自动query optimizer；
3. 多种空间树动态切换；
4. live schema migration无停机；
5. 高级分布式World迁移；
6. 基于屏幕面积/遮挡的复杂priority；
7. 通用用户脚本热装卸；
8. 全量历史时间旅行Inspector。

## P.8 如果从零重来

我会先冻结五份小而硬的规范，而不是先写 `Entity`/`Component` 基类：

1. **Identity & World Spec**：NetworkId、本地handle/generation、tombstone、predicted namespace、cross-world规则。
2. **Lifecycle & Structural Transaction Spec**：状态轴、record/commit/publish、依赖DAG、钩子限制、失败与hydrate。
3. **Schema & Replication Transaction Spec**：stable IDs、visibility、dirty、baseline/delta、callback、reference patch、compatibility。
4. **AOI & Replica Residency Spec**：per-connection interest、reason、epoch、hysteresis、leave策略、client presentation分层。
5. **Determinism & Recovery Spec**：phase/order/RNG/time、snapshot/hash/log/outbox、logic/schema版本。

随后用三种Storage原型实现完全相同的语义测试与目标benchmark。只有当数据证明某热点值得时，才把Transform/AOI/Attribute迁入native/SoA lane；其余能力保持可测试的active component。首个垂直切片不是“角色能移动”，而是：服务器创建→客户端AOI enter Baseline→预测移动→权威纠正→AOI边界抖动→soft leave/reenter→快照恢复→两端hash/trace可解释。

## P.9 最终判断

目标画像的方向可以成立，并且比直接套用Unity GameObject或纯DOTS更贴合“C# Gameplay + Native Kernel + 前后端共用”的约束。它距离生产级框架的差距主要不在Entity/Component/Query API，而在**状态机与协议完整性**：生命周期、复制、AOI、引用、预测和恢复必须共用同一tick/epoch/revision语义。

按本章“必须现在补”清单冻结语义后，Storage 可以继续实验；在这些语义未冻结前先选 Archetype 或 Sparse Set，只会把未定义语义固化进内存布局，未来迁移成本更高。

### P 章来源
本章综合 S001–S110；具体事实来源见 A–O 及 `sources.md`，建议项均为本报告基于目标画像的架构判断。
