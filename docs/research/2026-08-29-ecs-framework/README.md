# ECS Framework Research 2026-08-29

这是围绕“前后端共用、以对象组合为核心的 ECS / Entity-Component 框架”的纯外部技术调研包。目标不是选用现成库，而是用公开框架、源码、论文和事故证据校验既有自研画像的完整性。

## 信息源可达性

- 可联网并检索官方文档、公开 GitHub 仓库、Release、Issue；没有 clone 或打包第三方源码。
- Scott Bilas GDC 2002 PDF 读到全文；经典 AOI 论文多数只读到摘要/元数据，报告已降级说明。
- Mirror 的关键同步/AOI路径读到带 ref/行号源码；商业引擎与Photon只作官方文档级验证。
- 中文社区只作版本敏感的补充证据。

## 包内文件

- `report/ecs-framework-research-2026-08-29.md`：A–P完整报告。
- `sources.md`：110条来源及实际访问状态。
- `appendix/framework-matrix.csv`：框架总对照。
- `appendix/lifecycle-matrix.csv`：生命周期语义对照。
- `appendix/sync-matrix.csv`：声明同步对照。
- `appendix/aoi-matrix.csv`：AOI/兴趣管理对照。
- `appendix/gap-register.csv`：52 项完整性缺口登记。
- `appendix/terms.md`：术语表。
- `appendix/methodology.md`：方法与证据分级。
- `appendix/acceptance-checklist.md`：逐项验收自检。
- `appendix/validation.json`：章节、引用、置信度和缺口数量的自动校验结果。
- `appendix/manifest.csv`：包内文件尺寸与 SHA-256 清单（不含自身）。

## 推荐阅读顺序

1. 先读本页执行摘要；
2. 直接阅读报告 G（属性同步）、H（AOI）与 P（目标建议）；
3. 存储选型读 J + P.6；
4. 生命周期争议读 F + O；
5. 需要查证时按 `[Sxxx]` 去 `sources.md`。

## 执行摘要

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

## Known gaps

- **跨框架统一 benchmark 不存在**：公开 C# ECS 基准明确声明“不代表真实负载”，而且大多没有覆盖组件钩子、网络 Baseline、回滚快照、状态哈希、跨 managed/native 边界。J/P 章因此只给定性倾向与必须补做的实测矩阵。
- **商业源码不可达**：Photon Fusion 与 Unreal/Unity 的核心复制实现不是公开源码；相关内部算法只按官方文档陈述，不作源码级推断。
- **KBEngine 版本口径不统一**：主仓、插件、旧 API 镜像和社区文章跨多年；能够确认 EntityDef/回调思想，但无法将所有细节归到单一现代 commit。
- **ET 分支演进快**：主仓 README 可确认前后端 C#、模块与组件方向；旧社区文章的生命周期类名只作为历史/Reported，不据此给目标 API 定名。
- **AOI 规模数字不可比**：没有找到公开、可复现且同时报告 N 实体、M 观察者、移动率、兴趣半径、硬件、Tick 和带宽的主流框架横向数据。
- **“放弃同一路线”的完整复盘稀缺**：可找到具体 bug、迁移与设计批评，但没有找到一个公开项目完整披露“因 Active Component + 前后端共用而整体改写”的可核复盘；O 章明确保留空白。
- **论文全文限制**：三篇 AOI 经典文献主要依据摘要和元数据；不使用未读正文中的复杂度或实验数字。
