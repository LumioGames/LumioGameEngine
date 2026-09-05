---
name: 2026-09-05-dual-transform-bomber-research-gap-audit
description: 双 Transform 与炸弹人调研文档对照审计，核对主仓工作区设计及八个子仓的已推送实现、差异与验收缺口
metadata:
  type: doc
  status: 已交付
---

# 双 Transform / ECS / GAS / 炸弹人调研对照审计

日期：2026-09-05。审计对象：用户提供的 `Lumio_ECS_GAS_DualTransform_Bomberman_Design_2026-09-05.md`，对照主仓与八个 sibling 实现仓。

## 1. 核心结论

1. **当前不是一份方案的不同完成度，而是三层口径并存。** 调研主要基于修订前 ECS M10 的“双槽 + Local 认领”；主仓现行 ADR-063 改成“确认世界 + 预测世界重建、不认领”；子仓实际仍以聊天链路、预测编排骨架和炸弹人 Stage 0a 契约为主。不能把任何一层直接当作另外两层已经落实。
2. **最大的执行缺口在 Runtime 与 Client。** 双 Transform 尚无同名 C# 实现，Game 接入路径未进入十三相执行器，Client 的确认序号与 reconcile 计划没有接入实际共同提交路径。完成注册、创建、快照或测试桩事务，不等于完成预测移动、放弹与伤害闭环。
3. **一部分差异是主动取舍，需要对齐设计而非直接修代码。** 预测世界与认领机制、移动必须经 GAS、浏览器首发、八向移动、持续火焰、连锁与存档验收范围，均与调研的最小样例不同。调研不能直接替代现行 Owner 裁决；现行裁决也不能代替尚缺的实现证明。

下文优先级表示对“按调研贯通切片”的影响，不将所有差异都定性为软件缺陷。分类：**方向冲突**、**实现缺口**、**范围差异**、**文档漂移**。

## 2. 核验基线与边界

调研文件位于 `/Users/cui/Downloads/Lumio_ECS_GAS_DualTransform_Bomberman_Design_2026-09-05.md`，SHA-256：

`12b689755d87c6b1957730f4d1f9aabab6e697cb50f499c3ebd4adeddaaab4c6`

已对九仓执行 `git status --short --branch`、`git log -1`、`git ls-remote origin refs/heads/main`。八个实现仓 HEAD 与本次远端 main 查询一致。开始采集时，主仓本地领先远端 3 个提交，另有未提交架构修订。为保留审计现场，本审计没有 pull、merge、修改子仓源码或改写原调研文件。

**并发变更复核**：收口期间其他工作形成提交 `d9b0615`（父提交 `012e792`），随后经 PR #82 合入 `main`，最终本地与远端 main 均为 `d969cf6`。本审计执行 fetch 核验，确认 `d9b0615` 与 `d969cf6` 文件树无差异。本文所称“工作区设计”现已是该 main 的内容，ADR-063 文档状态仍标 Draft。已复核最终内容：双世界、不认领、移动经 GAS、Tick 接入缺口等主要结论不变；主仓引信示例已从2.5秒纠正为2.1秒，本文不再将其列为现存差异。本审计没有执行上述切分支、提交、推送或合入动作。

| 仓库 | 本地 HEAD | 本次远端 main | 与本次调研直接相关的状态 |
|---|---|---|---|
| LumioGameEngine | 采集 `012e792`；收口 `d969cf6` | 采集 `ce5ed03`；收口 `d969cf6` | ADR-063 修订已合入 main；ADR 文档状态仍为 Draft |
| LumioGameRuntime | `010ae46` | `010ae46` | 干净；ECS / simulation / GAS / 生成器重点核验 |
| LumioClient | `18020a1` | `18020a1` | 干净；prediction / replica / session / Bot Host / Web / Unity Adapter 重点核验 |
| LumioGame | `5bc5afc` | `5bc5afc` | 干净；炸弹人设计、ADR 0013–0019、组件与探针重点核验 |
| LumioServer | `4c7688b` | `4c7688b` | 干净；现行 Host 接入、聊天通道与 Timer 核验 |
| LumioVoxelEngine | `5d30e6e` | `5d30e6e` | 干净；体素契约、Rust World Port、跨域消费边界核验 |
| LumioNativeCore | `70b9834` | `70b9834` | 只有两处未跟踪 `.DS_Store`；Tick Timer 核验 |
| LumioConfig | `58aa9fc` | `58aa9fc` | 干净；秒转 Tick 的配置来源核验 |
| LumioPlatform | `9271bf1` | `9271bf1` | 干净；账号 / 启动与游戏模拟的边界核验 |

**证据口径**：代码结论针对上述检出的 main；不外推未合入分支、另一台机器或线上服务。主仓工作区结论显式带“工作区”标记。本文不是 Workflow 状态盘点，没有查询或写入线上单据；本地计划中的卡号与状态只用于定位，不能作为线上状态证据。没有运行网络、Renderer、目标设备性能或完整炸弹人对局测试。

## 3. 关键差异

### D01 / P1 / 方向冲突：双槽与双世界已不是同一预测模型

- **调研**：§2.1、§2.4、§17 要求同一客户端实体的确认槽 A / 预测槽 P，不增加预测 World；普通字段不自动双份。
- **当前设计**：工作区 [ecs.md M10](../knowledge/features/ecs.md) L380、[gas.md M7](../knowledge/features/gas.md) L230、[ADR-063 第 7 条](../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) L31 均要求确认世界与预测世界；每包从确认世界整体重建预测域并重放输入，第一版不包含体素。
- **版本事实**：远端 `ce5ed03` 的 ECS M10 仍是双槽与 Local 认领；所以调研所引旧 M10 有真实出处。远端 GAS M7 又写 ECS + GAS + 体素整帧回滚，说明旧公共文档自身也未完全一致。
- **影响**：恢复对象、普通工作字段归属、引用失效、实体生命周期与内存预算不同。调研 T-02 / T-05 不能按原文直接实现进当前工作区架构。
- **处理建议**：先统一要实施的预测模型，并用具体数据结构说明“世界”是独立 World 实例还是受限预测状态集。保留 A/P/M 的语义区分，但不要把它误当作双世界已经满足双槽 API。

### D02 / P1 / 方向冲突：预测炸弹与表现认领采用相反方案

- **调研**：§9.3–9.6、§10.5 要求只建 Local 预表现，正式炸弹正常复制；使用会话代次 + 命令序号 + 动作序号 + 产物序号关联并转移资源。第一版不预测炸弹逻辑存在、碰撞与最终伤害。
- **当前设计**：工作区 [bomber-slice.md](../knowledge/features/bomber-slice.md) L19、L92 与 ECS M10 要求客户端执行同一创建代码生成预测炸弹；ADR-063 L51、L72 明确拒绝认领键与“假实体换真实体”搬特效代码，表现按 `fx_key + 参数` 保持连续。
- **子仓分歧**：[Game design.md](../../../LumioGame/docs/specs/bomber/design.md) L209 仍写“客户端只预测自身移动与放弹表现”，与调研相近，却与主仓工作区的预测建实体口径不同。
- **影响**：两套方案不能混合验收。调研的 Adopted / Expired 状态机与 T-06 会被当前 ADR 的审查规则拒绝。
- **尚缺的论证**：`fx_key + 参数` 尚未定义实例唯一性、代次、校正改格后的连续性及重放副作用去重。例如两次同参数效果不能仅凭相同外观判断为同一次发生；位置变化时也不能假定键仍稳定。这是设计证明缺口，尚无实测证据证明当前方案必然重复或闪断。

### D03 / P1 / 方向冲突：移动与伤害接入 GAS 的强度不同

- **调研**：§4.2、§6.2、§17 由 Game MovementProcessor 汇总运动请求、单写者更新逻辑位姿；放弹与伤害分别通过 Ability / Effect。这个划分允许组件方法，不要求纯数据 ECS。
- **当前设计**：[系统规则](../rules/system.md) L15 与工作区 [gas.md](../knowledge/features/gas.md) L62 要求所有预测经 GAS，且“移动也是一个 Ability”，技能代码写 LogicTransform。炸弹人第二样板又在 [bomber-slice.md](../knowledge/features/bomber-slice.md) L141 的 `扣血` 中直接扣 `Sync` 血量，未展示 Damage Effect 求值；该文 L56 将 GAS 完整状态机与 Attribute 拓扑排除出切片。
- **影响**：若把调研的 MovementProcessor 接成 GAS 外的独立预测机制，会违反当前硬规则；若按当前样板直接扣血，又不能声称验过调研要求的 Effect 路径。
- **处理建议**：明确 GAS 激活入口与唯一运动求解者如何组合，以及本切片到底验证最小 Ability/Effect 集成还是只验证 ECS 战斗。无需为此要求 GAS 全集提前完成。

### D04 / P1 / 实现缺口：双 Transform 未落代码，Game 已有第二套位置字段

- **调研**：§2、§5.2、§6.4 要求唯一逻辑坐标源 LogicTransform，Cell 由它派生；ModelTransform 本地、单向、不同步、不存档。
- **源码证据**：对 Runtime、Client、Game、Server 全部可检索 `*.cs` 搜索 `LogicTransform|ModelTransform`，零命中。Game [BomberPlayerEntity.cs](../../../LumioGame/modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber/Contracts/EntityTypes/BomberPlayerEntity.cs) L7 只挂 BomberPlayerState；[BomberPlayerState.cs](../../../LumioGame/modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber/Contracts/Components/BomberPlayerState.cs) L28 同时复制和存档 `CellX/Y/Z` 与 `PosMilliX/Y/Z`。炸弹仅有 Cell 字段，未挂逻辑 Transform。
- **影响**：当前字段合同不满足“位置一处维护”。没有看到强制两组位置一致的约束；Teleport、逻辑父子变换、单写者检查、A/P 访问绑定也不能视为已交付。这里确认的是缺失与重复可写状态，未声称已经运行出坐标漂移。

### D05 / P1 / 实现缺口：Game 的实际 Tick 入口未接十三相

- **调研**：§6.2–6.8、§7 将移动、放弹、到期与伤害安排在 ECS 提交前；最后统一 Finalize，异常按 Fail-stop 处理。
- **源码证据**：Runtime [WorldManager.Tick](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs) L121 的服务器路径实际是 `ApplyInputs -> CommitCreates -> StampAndProject -> ConsumeSave -> Tick++`，没有调用 simulation 的十三相执行器。客户端路径是 ApplyClientBatch。另一路 [TickExecutorComposition](../../../LumioGameRuntime/modules/simulation/src/Lumio.GameRuntime.Simulation/Tick/TickExecutorComposition.cs) 类型已经 public，但构造函数 L134 仍 internal，不能误报成“整个类都是 internal”。
- **执行方案证据**：[Game stage0-kernel-contract.md](../../../LumioGame/docs/specs/bomber/stage0-kernel-contract.md) L21、L28 与 ADR 0015 明确采用普通 C# 函数，在 WorldManager.Tick 前后由 Scenario 手动编排。主仓 [tick.md §4](../knowledge/features/tick.md) 已承认此缺口并提出生成系统注册表。
- **影响**：十三相模块测试通过，不证明 Game 入口具备同样的结构提交、跨域事务、失败隔离和阶段读写保证。当前 Manager.Tick 内也没有故障后隔离 / 重建的完整路径，不能据文档承诺 V-26。

### D06 / P1 / 实现缺口：命令顺序与输入确认链未形成公共闭环

- **调研**：§6.1、§6.3、§7.2、§10.1 要求同 Tick 先移动后放弹、稳定命令排序、区分执行确认与业务成功，只重放尚未确认输入。
- **契约证据**：主仓 [gameplay-command-envelope-v1.json](../../engine/wire/gameplay-command-envelope-v1.json) L179 起仍是 InputCommand / FullSnapshot / Delta，尚无 `WorldChange.appliedInputSequence`。工作区 ADR-063 L60 明说该字段要随 R5-01 落地。
- **源码证据**：Runtime [InputCommandMessage](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldMessages.cs) L21 只有 mappingId、Sender、Payload、Connection，没有类型化命令序号；[WorldManager.ApplyInputs](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs) L244 对输入只按 Sender 比较排序，同一发送者没有移动/放弹的次级排序规则。Game MoveIntent / PlaceBombIntent 目前只是离线 DTO。
- **影响**：调研的“同 Tick 移动跨线后在新格放弹”、同 Tick 多请求预留、重放只保留未确认输入，均不能从当前通用入口保证。给下行加一个 Ack 数字还不够，上行序号来源、服务器连续处理边界、拒绝是否推进和同帧顺序也需落实。

### D07 / P1 / 实现缺口：Client 的共同权威事务目前只有编排形状

- **调研**：§10.1 要求 replica 与 prediction 在 Runtime 共同事务内恢复、应用、重放；成功后才推进元数据与发布表现。
- **源码证据**：[AuthorityUpdateOrchestrator.cs](../../../LumioClient/modules/session/src/Internal/Orchestration/AuthorityUpdateOrchestrator.cs) L52 固定构造 `AuthorityPredictionUpdate(update, 0)`；L55 得到 `reconcile`，L66 却仅把 `replicaPlan` 交给 Runtime，L100 也把 replicaPlan 当表现输出。`reconcile` 没有进入该提交调用。
- **进一步证据**：[ClientPrediction.cs](../../../LumioClient/modules/prediction/src/Internal/ClientPrediction.cs) 根据 ConfirmedThroughSeq 计算重放数量与清理历史；固定 0 不能正常清除从 1 起分配的输入。[ClientReplica.cs](../../../LumioClient/modules/replica/src/Internal/ClientReplica.cs) L116 先推进 metadata 再 `_world.ApplyCommitted`，ECS 应用仍发生在观察“Runtime 已提交”之后。[FoundationHostCommand.cs](../../../LumioClient/modules/bot/host/FoundationHostCommand.cs) L473 的 HostRuntime 忽略 request，并直接返回 committed=true。
- **影响**：这条 foundation/session 路径不能证明真实回放或 ECS/GAS 原子校正；即使 fake Runtime 返回成功也不能补足此证据。本结论限定到检出的调用路径，不把全部聊天功能判成不可用。

### D08 / P1 / 实现缺口：受控字段写入尚未在写入口强制执行

- **调研**：§2.2、§2.3、§4.4、V-05 要求确认值不被预测覆盖，非 Owner Thread / 非法写者被拒绝。
- **源码证据**：Runtime [SyncTypes.cs](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/Sync/SyncTypes.cs) L237 的 `.Value` setter 直接写 `_slot.Value`，然后才通知 host；[World.cs](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/World/World.cs) L265 的 OnLocalWrite 只记 Dirty / Hook，Owner 字段才上行，没有线程、角色或预测事务校验。WorldManager.Tick 的 EnsureOwner 只保护 Tick 入口，不能保护已经持有的组件引用。
- **影响**：持有组件引用的调用方可以越过线程边界修改已绑定字段；客户端写 Authority.Server 字段也不是自动进入独立预测槽。服务器入站 Owner 校验是另一层，不能替代本地写入保护。
- **证据限制**：本次是源码路径确认，未新增线程故障注入测试，也未将其扩大描述为远程权限绕过。

### D09 / P1 / 实现缺口：GAS 与炸弹人规则尚未完成所需链路

- **调研**：§5.4、§6、T-03 要求 PlaceBomb Ability、独立炸弹引信、Damage Effect 与创建/完成结果接线。
- **源码证据**：Runtime [GasWorldContext.cs](../../../LumioGameRuntime/modules/gas/src/Lumio.GameRuntime.Gas/Lifecycle/GasWorldContext.cs) 主要提供 Framework 生命周期、类型注册、Handle 发放/失效与 ECS 投影；[gas/README.md](../../../LumioGameRuntime/modules/gas/README.md) 的 activate / apply_effect / tick_effects / restore 仍是“候选接口”。Game 的 `Bomber/` 目录当前是 Components / EntityTypes / Commands / Events / Ports / generated，未找到对应移动、放弹、引信、爆炸与伤害求解实现。
- **测试证据**：[RuntimeIntegrationProbeTests.cs](../../../LumioGame/modules/server-gameplay/tests/Lumio.Game.ServerGameplay.Tests/Bomber/RuntimeIntegrationProbeTests.cs) 三个测试只做组件注册、实体创建和快照比较；`RunFixedScenario` 手动 SetSilent 扣血，未经过 Damage Effect。
- **影响**：当前可核验的是 Stage 0 契约接入，不能宣称调研的“阶段 A：权威能玩”已完成。框架文档中的能力清单不能作为实现清单。

### D10 / P1 / 实现缺口：共享预测工作状态仍被旧生成器规则拒绝

- **调研**：§2.4、§8.3 要求两端共享求解规则，并保存必要的预测内部状态。
- **当前设计**：工作区 ADR-063 第 5 条允许共享普通字段，例如转角缓冲剩余帧，并要求 lint 检查其是否只被单端赋值。
- **源码证据**：Runtime [SourceModel.cs](../../../LumioGameRuntime/tools/gen-declarations/SourceModel.cs) L162–171 仍禁止共享文件中的普通 string / bool / ulong / int / uint 字段，要求移至 `.Server.cs / .Client.cs`。
- **影响**：照工作区炸弹人第二样板添加共享 `int` 转角缓冲会被现行生成器拒绝。端别生成已存在，不应误报成“完全没有端别能力”；缺的是新规则和具体双 Transform 接入。

### D11 / P2 / 范围差异兼实现缺口：首发宿主是浏览器，表现链路尚未建成

- **调研**：§4.1、§4.4、§8、T-04 主要按 Unity Adapter / Game Presentation Binding 描述采样、主线程交接、远端 RenderTick 插值、本地校正偏移与 Teleport。
- **当前执行方案**：[Game ADR 0013](../../../LumioGame/.spec/decisions/0013-logic-first-browser-client-no-engine.md) L17 起明确逻辑先行、后续客户端暂定浏览器、首发不接 Unity / HybridCLR。
- **源码证据**：Client `modules/unity-adapter/src/` 仅有项目文件与 lock，没有实现 C# 源；[Web 模块](../../../LumioClient/modules/web/README.md) 及 `modules/web/hello` / `chat` 当前提供 Hello / Chat 页面，没有炸弹人位姿采样与渲染。ReplicaWorld 使用 Runtime WorldManager，确实没有另造一套 ECS 存储，但 [ReplicaWorld.cs](../../../LumioClient/modules/replica/src/Public/ReplicaWorld.cs) L35 仍从 Username ClientBootstrap 启动，实体类型过滤也限定 player / bot。
- **影响**：需要浏览器宿主如何复用或消费 Runtime 预测结果的可执行方案；不能把现有 JS 聊天页视为 C# 预测运行时已进入浏览器。远端采样时间、事件显示时间、销毁终态缓存、迟到资源回收均缺实际炸弹人落点。Unity 只是平台取舍，逻辑/表现隔离原则仍适用。

### D12 / P2 / 范围差异：炸弹爆炸后是否继续作为实体存在

- **调研**：§5.1、§6.6–6.8 到期产生一次即时伤害、销毁炸弹，爆炸只保留表现事实；持续火焰与连锁延后。
- **当前设计与字段**：[Game design.md](../../../LumioGame/docs/specs/bomber/design.md) L246、[BomberBombState.cs](../../../LumioGame/modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber/Contracts/Components/BomberBombState.cs) 要求同一炸弹经历引信、危险火焰、可选留火再销毁，复制 `ExplodedAtTick / DangerUntilTick / BurnUntilTick / Reach*`；火焰窗口内后来进入覆盖格的玩家也受伤。
- **影响**：调研 V-13 的到期销毁/容量释放时间、V-24 的纯事件驱动终态，以及晚加入只看当前状态的具体实现都必须改写。不能把持续危险窗口实现成“粒子多播一会儿”。这是玩法模型差异，不是炸弹独立生命周期原则被破坏。
- **新增待明确项**：炸弹处于火焰阶段时是否继续占放置容量，不能单凭“实体仍活着”自动决定；调研的“到期下一 Tick 才能再放”不应直接搬用。

### D13 / P2 / 范围差异：移动、落点与验收数值不是同一款最小样例

来源：调研 §5、§6；Game [design.md §6–7](../../../LumioGame/docs/specs/bomber/design.md) 与 [stage0-kernel-contract.md](../../../LumioGame/docs/specs/bomber/stage0-kernel-contract.md) §1–5。

| 维度 | 调研样例 | Game 当前口径 | 意义 |
|---|---|---|---|
| 移动输入 | 四方向，不斜向 | 自由八向、转角吸附 | 运动求解与测试不同 |
| 坐标平面 | 引擎 XZ，Y 固定 | Game XY、Z=0；地形适配映射到引擎 x=X,z=Y,y=Z+1 | 属可兼容坐标约定，适配层必须唯一 |
| 放置格 | floor 得到脚下所属格，格不可放就拒绝 | 最近合法格中心 + 125ms 输入缓冲 | 拒绝时机、边界格和执行 Tick 不同 |
| 基础移速 | 6 格/秒 | 3.5 格/秒 | 不用样例数字作实际验收值 |
| 逻辑频率 | 演示 60Hz | Stage 0a `tickRateHz=20`，待验证 | 3 秒=180 Tick 不能照搬 |
| 引信 | 3 秒 | 2.1 秒 | 收口时主仓 bomber-slice 示例也已修正为2.1秒，与 Game 对齐；仍与调研的演示数值不同 |
| HP / 伤害 | HP=1，伤害=1 | 6 个半心点，每弹扣 2 点 | 归因、死亡与测试结果不同 |
| 地图 | 静态地图、不做可破坏墙 | Stage 0a 19×19，完整目标含更大地图；可破坏砖、水与掉落 | 跨域事务成为实际范围 |
| 连锁 / 持续危险 | 延后 | 同 Tick 连锁 + 400ms 危险窗口 | 引擎验证范围明显更大 |
| 离弹许可 | 所有放置时已重叠角色可脱离 | 明确只写放弹者离格穿透 | 其他重叠者的行为尚不能从 Game 规则确认 |
| 角色互撞 | 明确暂不阻挡 | 本次所查 Game 条目未给同等明确结论 | 不能自行假定一致 |

Game 当前命令 DTO 也没有同时表示移动与放弹的统一动作样本。需要显式固定“同 Tick 两种动作”的依赖顺序、负坐标取整、最近合法格的平局规则与 Tick-local 预留，而不是让调用顺序偶然形成玩法。

### D14 / P2 / 实现缺口：事件尚不满足独立显示与稳定去重

- **调研**：§9.4、§10.5、§11.4 要求事件带发生 Tick、空间锚点/覆盖格、稳定身份；原炸弹销毁后仍能显示，重复收到不重播。
- **源码证据**：[BomberEvents.cs](../../../LumioGame/modules/server-gameplay/src/Lumio.Game.ServerGameplay/Bomber/Contracts/Events/BomberEvents.cs) L7 的 BombPlaced 没有命令/动作关联，也没有正式炸弹 ID；L9 的 BombExploded 只有 ChainId、Owner、CellCount、Tick，没有来源炸弹 ID、锚点或四臂。DamageApplied 已补来源炸弹 ID，不能把该历史缺口继续算在 DamageApplied 上。
- **影响**：如果采用调研的“炸弹已销毁、事件独立播”方案，当前 BombExploded 无法独立还原空间；同一链多颗炸弹也不能仅靠 ChainId 认作一个唯一爆炸事件。当前 Game 用活炸弹的 Reach* 表现可以覆盖另一种生命周期，不能据此声称满足调研的终态事件方案。上述 DTO 目前标注为内部遥测事件，公共网络映射尚待完成。

### D15 / P2 / 实现缺口：炸弹归属仍使用裸 u64，缺少完整实体身份契约

- **调研**：§6.4、§13.1 要求稳定 Owner 实体身份，炸弹不依赖仍存活的技能句柄或玩家对象。
- **源码证据**：Game BomberBombState 使用 `Sync<ulong> OwnerNetEntityIdRaw`；而 Runtime [NetEntityId.cs](../../../LumioGameRuntime/modules/ecs/src/Lumio.GameRuntime.Ecs/NetEntityId.cs) L6 已是 world InstanceId + Counter 两段 u64。Game kernel-contract L61 明确将 Raw 作为当前绕行；主仓 ADR-063 第 14 条登记 `Sync<NetEntityId>` 缺口。
- **影响**：一段 Raw 无法独立表达当前完整 128 位身份。若它只代表 Counter，就必须依赖明确的 World 上下文，不能不加说明地跨世界、进事件或存档后独立解析。这里未观察到实际跨世界误认，缺口是正式身份表达与消费路径未闭合。

### D16 / P2 / 范围差异：恢复验收被排除，不能承诺调研 V-28

- **调研**：§13.4、V-28 要求在 Tick200 保存、到期280，恢复后剩80 Tick；不重播历史爆炸或重复奖励。
- **当前设计**：工作区 [bomber-slice.md](../knowledge/features/bomber-slice.md) §3、§6 明确不做该游戏的存档/崩溃恢复验收。Game 当前有 Persist 字段和 CaptureSnapshot 探针，但没有保存、恢复、继续推进引信的炸弹人测试。
- **进一步差异**：Game design.md L265 将已命中集合定义为不持久的服务端临时结构。对于当前持续火焰模型，如果未来允许半途恢复，必须补齐命中记忆恢复策略，否则不能保证同弹同人只受伤一次；这是恢复进入范围后的条件风险。
- **处理建议**：将该项标为明确延期，或将恢复纳入一条真实炸弹生命周期测试。已有快照字节不等于玩法恢复语义完整。

### D17 / P2 / 实现缺口：Hash 证明仍未覆盖调研要求的比较切面

- **调研**：§13.5、V-30 分别定义同 Tick / 同投影复制对账与校正前预测误差，禁止比较服务器全世界和客户端 AOI 子集。
- **当前设计**：ECS M10 / GAS M8 已区分全量权威 Hash、同步域 Hash，并排除预测值及表现；与调研方向一致，但未把每次比较所需的同 Tick、可见集、字段集与校正前历史绑定完整写成执行合同。
- **现有证据**：Game 探针比较两个同 Seed 的服务器 World 快照，不是 Server/Client、不是 AOI 子集，也不是预测误差。Game stage0-kernel-contract §6 的 StateHash 还计划拼入全图地形；它是服务器回放 oracle，不能直接作客户端 AOI 对账 Hash。
- **影响**：缺少范围绑定会产生误报；只比较校正后的状态又会漏掉原预测错误。当前测试通过不证明 V-30。

### D18 / P2 / 文档与交付漂移：旧入口及计划不能证明新能力已交付

- **调研**：§1.2、G-01 使用当前 Living Architecture；T-01–T-08 按权威闭环、确认表现、预测、故障收尾推进。
- **仓库证据**：Runtime / Game 的 `.spec/AGENTS.md` 与 `knowledge/standards/repository-architecture.md` 仍将旧 `LumioGameEngineArchitecture`、冻结 Baseline、Schema/Fixture/镜像发布作为入口；Client prediction / unity-adapter README 仍引用旧基线。主仓现行 [repository-architecture.md](../knowledge/standards/repository-architecture.md) 要求活 API/ABI/wire 先落地。
- **本地派活证据**：[R5 卡草稿](../plans/2026-09-04-rm-00011-r5-cards.md) L127 已追加输入确认字段，但现行 wire 尚未变化；系统注册与 Sync<NetEntityId> 在 ADR-063 中仍是待派缺口。工作区 bomber-slice 则要求预测移动/建炸弹，同时说明 GAS 完整能力不在当前切片。最小 GAS、Transform、重建运行时与浏览器预测如何落到有前置、有验收的交付项，仍需贯通。
- **影响**：任务执行者可能读到互相排斥的约束。只同步文档或只完成 R5 旧聊天清理，不等于调研八个工作单元全部完成。本文未读取 Workflow，不判定线上已建卡与否。

## 4. 其余子仓与跨仓支撑面

| 仓库 | 已核验事实 | 对调研的判断 |
|---|---|---|
| LumioServer | [LiveElevenHost.cs](../../../LumioServer/mvp-host/src/Lumio.Server.MvpHost.App/LiveElevenHost.cs) L204 / L236 与 [ChatRoomWorldAdapter.cs](../../../LumioServer/mvp-host/src/Lumio.Server.MvpHost.App/ChatRoomWorldAdapter.cs) 接的是 ChatInput / AdmitChat；已有认证连接与宿主 Tick/Timer 基础 | 没有看到炸弹人输入、状态与事件接入的端到端 Host 路径。保持认证会话解析操作者的方向一致，但聊天接通不能证明炸弹人网络闭环 |
| LumioVoxelEngine | [Rust World Port](../../../LumioVoxelEngine/crates/lumio-voxel-world/src/port/adapter.rs) 提供 query / prepare_mutation / commit；已有 Section 契约与测试。Runtime [TxnPrepareCoordinator.cs](../../../LumioGameRuntime/modules/coordination/src/Lumio.GameRuntime.Coordination/Prepare/TxnPrepareCoordinator.cs) L73 的 IVoxelWorldPort 仍 internal，默认参与者 FailClosed | 体素不是“完全没实现”。缺的是 Game 可用的真实跨域接线。调研 MVP 不改地图，当前炸弹人却要求破坏与同帧连锁，因此实际依赖比调研更大 |
| LumioGame 地形消费 | [kernel-contract](../../../LumioGame/docs/specs/bomber/stage0-kernel-contract.md) L28–43 已改为 ITerrainStore / InMemoryChunkStore，BlockId 与坐标映射依 ADR 0019 对齐 | 不再把 ADR 0015 旧文中的“地形存在 ECS 组件”当成现行方案。工作区 ADR-063 L84 仍提该旧临时方案，属于主仓引用漂移；当前内存地形方案也不等于已接 Voxel 真后端 |
| LumioNativeCore | [TimerManager.advance](../../../LumioNativeCore/crates/lumio-timer/src/manager.rs) L526 按调用方提供的 Tick 推进；[timer README](../../../LumioNativeCore/modules/timer/README.md) 明确逻辑 Timer 不读墙钟，宿主断线窗口另管 | 与调研“引信由逻辑 Tick 决定”的底层原则一致。没有证据表明炸弹必须另造墙钟定时器，也没有证据表明 Game 引信已经接入此模块 |
| LumioConfig | [validate.py](../../../LumioConfig/src/lumio_config/validate.py) L31 从 repository.yaml 读 tickRate，L90 将 seconds 乘 tickRate；本仓默认是60。Game Stage 0a 计划20Hz | 60与20本身不是错误，调研60也是样例。但当前“配置编译率”和主仓要求的 WorldEntity 运行率缺少本切片一致性证明。必须避免把按60编译的帧数用于20Hz；尚未看到错误配置实际进入炸弹人 |
| LumioPlatform | [README](../../../LumioPlatform/README.md) 明确只负责账号权威、大厅、准入与启动，不实现 ECS / Voxel / Gameplay | 在本次范围内未发现与调研冲突；不应为双 Transform / GAS 往 Platform 加游戏模拟职责 |

## 5. 已一致或已关闭的事项

不能把调研的待对齐项机械重报为当前缺陷：

- **G-02 提交点**：Runtime [PhaseContractTable.cs](../../../LumioGameRuntime/modules/simulation/src/Lumio.GameRuntime.Simulation/Phases/PhaseContractTable.cs) L51、[TickRunner.cs](../../../LumioGameRuntime/modules/simulation/src/Lumio.GameRuntime.Simulation/Tick/TickRunner.cs) L322 已在 GasAndEventFinalize 完成后 MarkCommitted。仍错误的是 [simulation README](../../../LumioGameRuntime/modules/simulation/README.md) L29 的“ECS 后、GAS 前”。需要修正文档与接线，不是把 TickRunner 再改一个提交点。
- **G-03 同帧字段可见性**：工作区 tick.md §3 已明确已有字段当场生效；结构与地形延后。Sync setter 也直接生效。仍需补的是“移动先于放弹”的真实玩法测试与失败保护，不是这个原则仍无答案。
- **G-08 Finalize 追加当 Tick 结构单**：工作区 tick.md 已限定 Finalize 不跑业务，只写 GasEvents；gas.md 区分提交前求值与最终取样。实现接线未完，但文档方向已经关闭该歧义。
- **同一 ECS、双端独立 World**：Client ReplicaWorld 已包装 Runtime WorldManager，确实不是新的 ECS 存储实现。问题在 prediction/事务与具体 Game 注册的接线，见 D07 / D11。
- **逻辑与表现单向、服务器不为预测回退、静态地图归体素、炸弹独立实体**：调研与当前主仓原则一致。没有理由为对齐调研引入第三类世界对象或把爆炸规则下沉到 NativeCore / Voxel。
- **DamageApplied 来源炸弹 ID**：Game ADR 0018 与现行 DTO 已补齐；D14 指向的是 BombExploded / BombPlaced 的不同需求。
- **主仓引信示例**：采集初期发现的2.5秒旧数值已在并发提交 `d9b0615` 中改为2.1秒，已从现存漂移中撤下。

## 6. 验收覆盖与实跑证据

本次实际执行：

| 检查 | 命令或方式 | 结果与能证明的范围 |
|---|---|---|
| 九仓远端复核 | 各仓 `git ls-remote origin refs/heads/main`，比对本地 HEAD | 八个实现仓与远端一致；主仓收口复核为 `d969cf6`，并发变化见 §2 |
| 调研输入锁定 | `shasum -a 256` | 摘要见 §2 |
| 双 Transform 搜索 | `rg` 在四个 C# 仓的 `*.cs` 中查找 LogicTransform 或 ModelTransform | 零命中；退出码1表示无匹配，不是命令执行失败 |
| Game 现有测试 | Game 仓 `dotnet test --project modules/server-gameplay/tests/Lumio.Game.ServerGameplay.Tests/Lumio.Game.ServerGameplay.Tests.csproj` | **26 passed，0 failed，0 skipped**；其中炸弹人只有3个接入探针 |
| Runtime 模拟测试 | Runtime 仓 `dotnet test --project modules/simulation/tests/Lumio.GameRuntime.Simulation.Tests/Lumio.GameRuntime.Simulation.Tests.csproj` | **146 passed，0 failed，0 skipped**；含十三相/提交点断言，不等于 Game 已调用该执行器 |
| 报告结构 | 主仓 `node .spec/tools/spec-lint.mjs` | **spec-lint: OK** |
| 现行 wire 合同 | 主仓 `node eng/verify-wire.mjs` | **7个合同全部通过，附带测试41 passed / 0 failed**；验证的是现行合同，不能证明计划中的 C-1 新字段已实现 |

调研验收项的当前证据判断：

| 调研验收 | 当前判断 |
|---|---|
| V-01–06：双 Transform、预测隔离、帧率、单写者、线程、Teleport | 双 Transform / 表现路径尚缺；线程写保护存在 D08；没有端到端通过证据 |
| V-07–16：移动、放弹、抢格、容量、引信、伤害、独立生命周期 | 当前只有实体与字段探针；Game 玩法规则有差异，先修订验收样例再实现 |
| V-17–23：重放、认领、乱序、拒绝、历史不足 | D01/D02 的方案冲突未统一，D06/D07 的确认与事务未接齐；无真实炸弹人通过证据 |
| V-24–27：显示时间、AOI、故障、Renderer隔离 | 设计原则部分已有；没有炸弹人 Renderer/网络故障实跑 |
| V-28：存档后继续引信 | 当前切片明确排除，不能算已覆盖 |
| V-29：LocalEmbedded 双端完整消息路径 | 同 ECS 双端基础存在；本次炸弹人探针是两个服务器快照比较，并非该用例 |
| V-30：同 Tick 同 AOI 投影对账 | 只有服务器回放 Hash 基础；没有匹配此条件的炸弹人测试证据 |

这些判断表示“本次可核验证据尚不足”，不将未执行的验收写成测试失败。本次未运行全 SDK Windows Host 门槛、所有子仓全量测试、网络故障注入、Renderer 或性能压测。

## 7. 建议的对齐顺序

1. **先定唯一设计方向**：D01/D02 预测状态模型与表现连续性；D03 GAS 最小职责；D11 浏览器消费预测结果的路径。用“连续两次放弹、其中一次拒绝、一次校正跨格”证明方案，不以术语替代数据流。
2. **补 Runtime 可执行基础**：系统注册进入固定 Tick、Transform 与受控字段写入、共享状态生成、完整身份类型。将 D05 的 Game 入口接到具备既定提交与失败语义的运行路径。
3. **补协议与客户端共同事务**：真实输入序号、服务器处理边界、权威包解码、应用与重放；原子成功后才发布表现。把 D07 的计划丢失和固定0清除。
4. **按 Game 的真实规则验权威闭环**：八向移动、合法格放置、预留、2.1秒引信、持续火焰、连锁、伤害与体素修改。不要用调研的60Hz/一次伤害样例掩盖实际20Hz和持续危险窗口。
5. **最后验预测与表现故障**：RenderTick、重放幂等、稳定实例身份或等价连续性机制、代次清理、晚到事件、Hash 切面；存档明确延期或另给完整恢复验收。

以上是依赖排序建议，未创建任务、未派活、未修改线上卡。无需等待所有 GAS、存档、AOI 高级能力完成，必须先有本切片所需的最小真实路径。

## 8. 本次改动与边界

仅新增本审计报告。没有修改既有设计、原调研文档、子仓业务源码或用户未提交改动；没有提交、推送、发布或 Workflow 写入。知识沉淀落点为本次 `.spec/reviews/` 记录，未将尚待统一的建议写入活文档真值。

纯文档报告采用快速模式豁免，不派实现或审查子 Agent。结构校验结果见 §6；报告中的源码结论与设计取舍已作本地复核，未冒充独立 reviewer 审查。
