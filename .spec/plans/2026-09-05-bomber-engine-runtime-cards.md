---
name: 2026-09-05-bomber-engine-runtime-cards
description: 炸弹人切片引擎侧六卡（Runtime RT-1~RT-5 + Client 调研 CL-1）的正文、接口与依赖 DAG，按 ADR-063 / 064 填实；落单或派活前查
metadata:
  type: doc
  status: 设计中
---

# 炸弹人切片 · 引擎侧卡片正文（Runtime RT-1 ~ RT-5 + LumioClient CL-1）

- 来源：[`reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md`](../reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md) 核出的引擎缺口（D04 / D05 / D07 / D08 / D11 / D18 + §5 G-02）与 Owner 2026-09-05 三条裁决（浏览器预测现在就调研 WASM；技能状态机进切片；扣血走 Effect）。
- 真值：[ADR-064](../decisions/ADR-064-gas-slice-contracts.md) > [ADR-063](../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) > ADR-060 / 058 > [`tick.md`](../knowledge/features/tick.md) / [`ecs.md`](../knowledge/features/ecs.md) / [`gas.md`](../knowledge/features/gas.md) / [`bomber-slice.md`](../knowledge/features/bomber-slice.md) > C-1″（R5-01 提交号）> 本卡正文。Workflow 上的 done / handback / closeout 报告都不是真值。
- 落单口径：**不新开需求室**——另一会话 2026-09-05 已在 Workflow 建 RM-00014「引擎九项必备能力补齐：双 Transform、Tick 与预测闭环」（来源总单 R-00459，交付单 R-00460 ~ R-00472，全部初始态、未派工）。本文件的卡按下表对齐到既有单，派活前由主 loop 用本文件正文更新对应单的正文与验收项（评论回写，不新建重复单）：RT-1 → R-00462（E2 Tick 接线）；RT-2 → R-00461（E1 双 Transform）+ R-00463（E3 受控写入）的写守卫部分；RT-3 → R-00466（E5 预测世界重建）；RT-4 / RT-5 → R-00468（E6 最小 GAS 纵链，正文按 ADR-064 扩到八态准入 + 瞬时 Effect + 两本账）；CL-1 → 作为 R-00467（C5 客户端接线）与 R-00470（C8 表现）的调研前置，RM-00014 无对应单则在该室新建一张调研单；R5-01 / R5-02 / R5-03 的追加项仍在 RM-00011 原单执行，R-00460（A0 公共契约）/ R-00463 / R-00464 / R-00465 作为其下游引用不重复实现。体素接线 R-00469、表现 R-00470、验证工具 R-00471、联合验收 R-00472 不在本文件范围。
- 共同执行规范：逐字沿用 [`2026-09-04-rm-00011-r5-cards.md`](2026-09-04-rm-00011-r5-cards.md)「共同执行规范」段（治理原则 / 硬禁令 / 工作方式 / 交回格式五段），真值优先级换成本文件上一条。
- 派活口径：照 [`2026-09-04-rm-00011-r5-dispatcher-prompt.md`](2026-09-04-rm-00011-r5-dispatcher-prompt.md) §3（领卡 → 开工评论 → 冷启动 worker 独立 worktree → 机器检查 → reviewer 独立环境 → 合入 → 回写 → 沉淀），禁词 grep 沿用并加：`Baseline`、`LGE-V1`、`候选接口`、`presentationBuffer`、`inputFrame`（ADR-051 作废字段）、`FxComponent`、`decimal`（求值路径）、任何含「认领」「搬特效」的新增注释。
- 任务卡格式：每卡含「涉及范围」（文件集，供并行判重叠）、「验收标准」、「依赖」、「接口」（Consumes / Produces），口径见 [`tasks/README.md`](../tasks/README.md)。

## 依赖 DAG 与并行边界

```text
R5-01（C-1″ sequence / appliedInputSequence / OnFx 用例）
  ↓
R5-02（生成器：共享字段 lint / Scope.None / Sync<NetEntityId> / 占段发号 / (发送者, sequence) 排序 / codec / 模板池）
  ↓
RT-1  Tick 统一 + 系统注册 + tick 频率 + Runtime 文档                     ── 碰 WorldManager.cs、simulation/**、gen-declarations（[System] / [After] / [Reads] / [Writes] 与写者冲突校验）
  ├─→ RT-2  双 Transform + owner-thread 写守卫 + 双写者负例              ── 碰 ecs 组件、SyncTypes.cs、样板（不碰 gen-declarations：[Writes] 机制归 RT-1）
  └─→ RT-4  GAS M2：AbilityComponent / 八态 / 准入五步 / Activate<T> / Attribute 声明展开   ── 碰 gas/**、gen-declarations
          ├─→ RT-5  GAS M3 / M4 / M5：Effect 单 / 整数求值 / 两本账重算 / 击杀跨零 / OnFx      ── 碰 gas/**、gen-declarations
          └─→ RT-3  预测世界重建（客户端 Runtime 模块）+ appliedInputSequence 消费 + 表现键差集   ── 碰 WorldManager.cs、ecs/Prediction/**
CL-1  LumioClient WASM 调研                                                 ── 无前置，与 wave 1 并行
```

| wave | 卡 | 仓 | 并行说明 |
|---|---|---|---|
| 1 | CL-1 | LumioClient | 与 R5-01 / 体素 wave 1 / Platform W0 异仓并行 |
| 3 | RT-1 | LumioGameRuntime | R5-02 合入后；独占 `WorldManager.cs` 与 `gen-declarations` |
| 4 | RT-2 ∥ RT-4 | LumioGameRuntime | 文件集不重叠：RT-2 = ecs 组件 + `SyncTypes.cs` + 样板；RT-4 = `gas/**` + `AbilityComponent` + `gen-declarations`。`[Reads]` / `[Writes]` 声明与写者冲突校验已归 RT-1，RT-2 不碰生成器 |
| 5 | RT-3 ∥ RT-5 | LumioGameRuntime | RT-3 = `WorldManager.cs` 客户端路径 + `ecs/Prediction/**`；RT-5 = `gas/**`；`gen-declarations` 归 RT-5 |

最长链：R5-01 → R5-02 → RT-1 → RT-4 → RT-5（5 张串行）。RT-3 依赖 RT-4 是因为预测世界里重放的输入就是 `Activate<T>`。

---

<!-- card:RT-1 -->
- title: [程序·Runtime][Wave 3] Tick 统一：`WorldManager.Tick()` 只走 `TickRunner` 13 相；游戏系统 `[System(Phase)]` 生成注册进第 3 / 4 相；tick 频率落 `WorldEntity.TickRate`；Runtime 文档改指 Living Architecture
- category: runtime / simulation
- priority: P0
- risk: high
- wave: 3
- 前置: R5-02 合入
<!-- body -->
# 执行提示词：[程序·Runtime][Wave 3] Tick 统一 + 系统注册 + tick 频率 + 文档

`workflow-plan: bomber-engine/RT-1`

## 任务元数据
- 目标仓库：`LumioGameRuntime`
- 责任角色：Runtime 模拟框架工程师（Tick / 相契约 / 生成器唯一实现方）
- 前置状态：R5-02 合入（模板池、codec、`(发送者, sequence)` 排序、生成三件在 `origin/main`；派活评论钉 SHA）

## 涉及范围（拥有的文件集）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs`、`World.cs`、`WorldSaveComponent.cs`（WorldEntity 的 `TickRate` 字段与 `毫秒换帧`）
- `modules/simulation/**`（`Tick/TickRunner.cs`、`Tick/TickExecutorComposition.cs` 公开构造、`Planning/ProcessorPlan*.cs` 接入生成注册表、`README.md`）
- `tools/gen-declarations/**`（`[System(Phase)]` / `[After]` 注册表生成，属「生成三件」；`[Reads]` / `[Writes]` 声明与「同相内同组件写者 > 1 → 生成失败并点名」的校验也归本卡——ecs M7「同帧双写者 = 启动时报错」的机制；`LogicTransform` 组件本身归 RT-2）
- `modules/ecs/samples/username/**`（样板加一个 `ProcessorPlan` 相的示范系统与测试）
- `.spec/AGENTS.md`「项目是什么」段、`.spec/knowledge/standards/repository-architecture.md`、根 `README.md`「架构基线」段、各模块 README 的「架构基线」行
- 对应测试项目

## 来源真值
- ADR-063 第 4 / 14 条 ①③；ADR-064 第 3 条（`ApplyInputs` 相执行技能，本卡只留入口）；`tick.md` §2 13 相表、§4 注册方式、§5 频率、§6 失败；`ecs.md` M1a ③、M3；差异审计 D05、§5 G-02、D18
- 现状证据：`WorldManager.cs:132-139` 服务器路径是 `ApplyInputs → CommitCreates → StampAndProject → ConsumeSave → Tick++` 私有序列，`modules/ecs/src` 对 `TickRunner` 零引用；`TickExecutorComposition.cs:127` public 类、`:134` internal 构造；`simulation/README.md:29` 提交点写「ECS 后、GAS 前」与 `PhaseContractTable.cs:51 / :111`、`TickRunner.cs:321-326`（`GasAndEventFinalize` 后 `MarkCommitted`）矛盾；`.spec/AGENTS.md:12` 仍指 `LumioGameEngineArchitecture` 与 `LGE-V1.4` 基线

## 产品背景与已锁决策
- **一条 Tick 路径**：`tick.md` 冻结「恰好 13 相、第 10 相唯一提交点、第 8 相起不可取消」；`WorldManager.Tick()` 现在自己走五步，十三相执行器只在模块测试里跑——LumioGame 只能在 `Tick()` 前后手调函数（LumioGame ADR 0015）。Owner 定：Manager 的每一步映射到相（收 inbox = 1 / 解包排序 = 2 / ServerRpc 与 Owner 字段上行 = 3 / 注册系统 = 4 / 地形单合批 = 5 / Native 收件 = 6 / 提交决定 = 7 / 体素 = 8 / 结构单 + GAS 重算 = 9 / 取样 + 事件 = 10 / 按观察者打包 = 11 / 轻量哈希 = 12 / 交连接层 = 13），私有序列删除，不留「兼容」开关。
- **注册方式**（tick.md §4）：`[System(Phase.ApplyInputs | Phase.ProcessorPlan)] public sealed partial class X : System { public override void Run() }`；只允许第 3 / 4 相，其他相生成报错；同相顺序 = 声明序 + 可选 `[After(typeof(Y))]`，编译期成环报错；系统只声明读写哪些组件（`[Reads]` / `[Writes]` 声明与写者冲突校验都归本卡）；注册表由生成器产出、世界只收生成的注册表，不反射。客户端同一套注册表：预测档系统在预测世界跑（RT-3 接），其余只在服务器跑（按 `.Server.cs` / 共享文件归属自然区分）。
- **tick 频率**：`WorldEntity` 上 `[Persist] Sync<uint> TickRate = new(Scope.Room)`（Hz）作为唯一真值，随快照入档、一局不变；引擎函数 `Ticks.FromMilliseconds(ms)`（名字可自定）读它换帧；框架里没有秒。
- **失败语义**：第 10 相前抛错 / 取消 / 超预算 → 整帧作废 + 从上一帧快照重建（现有 `FailStopController` 接线），不做字段级 undo；钩子里的不可回滚动作记 outbox，第 13 相后执行。
- **文档**：`simulation/README.md` 提交点改 `GasAndEventFinalize`；`.spec/AGENTS.md` / `repository-architecture.md` / 根 README「架构基线」改指 Living Architecture（架构仓 `LumioGameEngine/.spec/knowledge/features/architecture.md` + `engine/wire` / `engine/abi`），删 Baseline / 镜像 / Schema-Fixture 入口措辞。

## 本需求目标
`WorldManager.Tick()` 在服务器与客户端都只经 `TickRunner` 13 相；玩法程序集声明的系统被生成注册表带进第 3 / 4 相按声明序执行；tick 频率从 `WorldEntity` 读；样板七步在新路径下全绿且两轮逐字节一致；Runtime 文档零旧制度措辞。

## 详细要求（逐条照做）
1. `TickExecutorComposition` 给 Manager 一个公开装配入口；`WorldManager.Tick()` = 组装 13 相并执行；删 `CommitCreates / StampAndProject / ConsumeSave` 作为顶层步骤的写法（各自成为对应相的实现体，命名随相）。客户端路径同一 runner（第 3 / 4 相在客户端只跑预测档系统，本卡先留空钩子，RT-3 填）。
2. `System` 基类 + `[System(Phase)]` + `[After(typeof)]` + `[Reads(typeof)]` / `[Writes(typeof)]`；生成器产 `GeneratedSystemRegistry`（生成三件扩到四件或并入注册表，名字自定），标非第 3 / 4 相、成环、同相内两个系统声明写同一组件 → 生成失败并点名。
3. `WorldEntity.TickRate` + `Ticks.FromMilliseconds`；`WorldSaveComponent` 快照带它；`CreateFromSnapshot` 读回。
4. 失败：注入一个在 `ProcessorPlan` 相抛错的测试系统 → 本帧作废、世界等于上一帧、日志含相名；第 8 相后抛错 → `ProcessFault`。
5. 样板：`modules/ecs/samples/username` 加一个 `ProcessorPlan` 相示范系统（例如每帧统计在线数写 `WorldEntity` 字段），测试断言它在 `ApplyInputs` 相之后、提交之前跑。
6. 文档同步（上列文件）；`simulation/README.md` 与 `PhaseContractTable.CreateDefault()` 逐格一致。

## 验证计划与证据
`dotnet build`；`dotnet exec` 全部测试 dll（含 simulation 146 项不倒退）；样板七步；两轮同输入 13 相哈希逐字节一致；`grep -rn "CommitCreates\|StampAndProject\|ConsumeSave" modules/ecs/src` 只出现在相实现体内或零命中；`grep -rn "LGE-V1\|LumioGameEngineArchitecture\|Baseline" .spec README.md modules/*/README.md` 零命中；20 Hz 下 `Ticks.FromMilliseconds(2100) == 42` 测试。

## 接口
- Consumes：R5-02 的 `WorldManager` / 模板池 / codec / `(发送者, sequence)` 排序；`TickPhase` 枚举与 `PhaseContractTable`（不改相名）。
- Produces：`System` 基类、`[System(Phase)]`、`[After(typeof)]`、`[Reads(typeof)]` / `[Writes(typeof)]` 与写者冲突校验、`GeneratedSystemRegistry`、`WorldEntity.TickRate`、`Ticks.FromMilliseconds(ms)`、`WorldManager.Tick()` 唯一路径（RT-2 / RT-3 / RT-4 / RT-5 全部依赖）。

## 验收标准
1. `WorldManager.Tick()` 服务器 / 客户端只经 `TickRunner` 13 相；`modules/ecs/src` 无第二条 Tick 序列；样板七步全绿、两轮哈希一致
2. `[System(Phase)]` 注册进第 3 / 4 相按声明序 + `[After]` 执行，非法相 / 成环 / 同相双写者生成失败并点名；样板示范系统有测试
3. `WorldEntity.TickRate` 进快照、`Ticks.FromMilliseconds` 按它换帧；测试系统抛错 → 整帧作废
4. `simulation/README.md` 提交点 = `GasAndEventFinalize`；Runtime `.spec` 入口文档零旧制度措辞（grep 证据在交回物）

## 明确不做与禁止事项
不做双 Transform（RT-2）、预测世界（RT-3）、GAS（RT-4 / 5）；不改 `engine/wire`；不加「兼容旧路径」开关；不改相名与相数。

## 阻塞与升级
`TickRunner` 的 `IRuntimeSession` / 预算模型装不下 Manager 的 inbox 或 outbox → BLOCKED，写清哪一相的可写域不够，其余照做。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. Tick 只有 13 相一条路径，样板七步全绿、两轮哈希一致
2. 系统注册进第 3 / 4 相，非法相 / 成环生成失败
3. TickRate 进快照并驱动换帧；相内抛错整帧作废
4. simulation README 与 Runtime `.spec` 入口文档同步，零旧制度措辞
<!-- /card -->

<!-- card:RT-2 -->
- title: [程序·Runtime][Wave 4] 双 Transform：`LogicTransform`（权威、上网、存档、`Teleport`）与 `ModelTransform`（客户端 Local、插帧、不上网）落地；用 RT-1 的 `[Writes]` 机制补双写者负例；`Sync<T>` 写入 owner-thread 守卫
- category: runtime / ecs
- priority: P0
- risk: medium
- wave: 4
- 前置: RT-1 合入
<!-- body -->
# 执行提示词：[程序·Runtime][Wave 4] 双 Transform + 单写者 + 写守卫

`workflow-plan: bomber-engine/RT-2`

## 任务元数据
- 目标仓库：`LumioGameRuntime`
- 责任角色：ECS 框架工程师
- 前置状态：RT-1 合入（`System` 基类、注册表、`[Writes]` 机制）

## 涉及范围（拥有的文件集）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/Components/LogicTransform.cs`（新）、`Components/ModelTransform.Client.cs`（新；只进客户端程序集）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/Sync/SyncTypes.cs`、`World/OwnerThreadGuard.cs`（写守卫）
- 不碰 `tools/gen-declarations/**`（`[Writes]` 机制归 RT-1；本卡只在样板里用它）
- `modules/ecs/samples/username/**`（`PlayerEntity` 加 `LogicTransform`，客户端加 `ModelTransform` 示范）
- 对应测试项目

## 来源真值
- `ecs.md` M7 全条、M4 ⑥（客户端应用服务器数据不记脏）、M9 ③（权威读写只在 owner thread）；ADR-063 第 7 条（服务器只有 `LogicTransform`）、第 14 条 ⑥；差异审计 D04、D08
- 现状证据：四仓 `*.cs` 搜 `LogicTransform|ModelTransform` 零命中；`SyncTypes.cs:255-258` setter 先写 slot 再通知、无线程断言；`World.cs:265-274` `OnLocalWrite` 只记 Dirty / Hook

## 产品背景与已锁决策
- 逻辑位置和画面位置是两个组件：`LogicTransform` 服务器权威、`Sync`、`[Persist]`、固定步长推进；`ModelTransform` 只在客户端程序集（`.Client.cs`），不同步不存档，每渲染帧从 `LogicTransform` 插过去、永不写回。任何逻辑判定只看 `LogicTransform`。
- 坐标表示归实现仓，但必须整数（确定性）；建议毫格 `int` 三分量；`所在格` 是派生只读属性（负数向下取整、半开区间）；`Teleport()` 显式标记「这次别插值」，`ModelTransform` 消费后清。
- 父子记账（父指针 + 子列表、换爹不跳位、`SetParent(随父销毁)`）按 M7 ④⑤⑥⑦ 建接口与测试；网络同步局部坐标 + 父引用、父不可见兜底世界坐标——本切片只验「无父」路径，父子路径要有测试但不进炸弹人验收。
- **单写者**：系统 / 技能类型声明 `[Writes(typeof(LogicTransform))]`，RT-1 的生成期校验对同一相内同组件写者 > 1 报错并点名两者（不是运行时看运气）；本卡只补 `LogicTransform` 的负例。
- **写守卫**：`Sync<T>.Value` setter 断言当前线程 = Manager owner thread（非 owner 线程写 → 抛出，且不产生半更新）；`ApplyingRemote` 路径不变。服务器入站 Owner 校验是另一层，本卡不动。

## 详细要求（逐条照做）
1. `LogicTransform`：`逻辑位置`（`Sync<Int3>`，`[Persist]`，`Scope.Aoi`）、`朝向`、`父`（`Sync<NetEntityId>`，可空）、派生 `所在格`、`Teleport()`；父子记账在结构事务结算时统一解绑、顺序排死。
2. `ModelTransform.Client.cs`：`表现位置`、`上次逻辑位置`、`跳变标记`；`每渲染帧(alpha)` 从 `LogicTransform` 插值；`Teleport` 标记消费。
3. 用 RT-1 的 `[Writes]` 机制：样板里两个系统声明写 `LogicTransform` 的负例 → 生成 / 启动失败并指出是哪两个（机制不在本卡，只加负例与组件）。
4. 写守卫：`SyncTypes.cs` setter 加 owner-thread 断言；`OwnerThreadGuard` 复用；非 owner 线程写的测试证明「拒绝且值未变」。
5. 样板：`PlayerEntity` 加 `LogicTransform`；一个测试系统每帧前进；客户端 `ModelTransform` 插帧测试；`Teleport` 后不插值测试。

## 验证计划与证据
`dotnet build` / `dotnet exec`；随机扰动 `ModelTransform` 后服务器哈希不变；两写者启动失败输出；非 owner 线程写抛出且值不变；`ModelTransform` 不在服务器程序集（反射断言 / 结构断言）、不进快照、不进哈希；`SetParent` 前后世界坐标零漂移。

## 接口
- Consumes：RT-1 的 `System` / 注册表 / `[Writes]` 机制；R5-02 的 `Sync<T>` / 生成器。
- Produces：`LogicTransform`（`逻辑位置` / `朝向` / `父` / `所在格` / `Teleport`）、`ModelTransform`（客户端）（RT-3 表现键示例、RT-4 技能样板与 LumioGame 依赖）。

## 验收标准
1. `LogicTransform` / `ModelTransform` 落地，随机扰动 `ModelTransform` 服务器哈希不变；`ModelTransform` 不在服务器程序集、不进快照与哈希
2. 样板两个系统写 `LogicTransform` 的负例 → 生成 / 启动失败并点名；非 owner 线程写 `Sync` → 拒绝且无半更新
3. `Teleport` 后不插值；`SetParent` 零漂移；父销毁子留原地（测试存在，切片不验收）
4. 样板与测试全绿

## 明确不做与禁止事项
不做物理接管相；不做 AOI 成套进视野（M6 ⑦ 只留钩子）；不改 `engine/wire`。

## 阻塞与升级
坐标整数表示与 LumioGame kernel contract 的毫格口径冲突 → BLOCKED 上报，不本地折中。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. 双 Transform 落地，ModelTransform 不进服务器 / 快照 / 哈希
2. 双写者启动报错；非 owner 线程写被拒
3. Teleport / SetParent 语义有测试
4. 样板与测试全绿
<!-- /card -->

<!-- card:RT-3 -->
- title: [程序·Runtime][Wave 5] 预测世界重建（客户端 Runtime 模块）：一个 Manager 持确认 + 预测两个 World；每包整体克隆 + 重放 `sequence > appliedInputSequence`；本地临时号；表现键差集；预测误差仪表；对账哈希四元组
- category: runtime / prediction
- priority: P0
- risk: high
- wave: 5
- 前置: RT-1、RT-4 合入（RT-2 的 `LogicTransform` 合入后接表现键示例）
<!-- body -->
# 执行提示词：[程序·Runtime][Wave 5] 预测世界重建

`workflow-plan: bomber-engine/RT-3`

## 任务元数据
- 目标仓库：`LumioGameRuntime`
- 责任角色：ECS / 预测框架工程师
- 前置状态：RT-1（Tick 路径、客户端第 3 / 4 相钩子）、RT-4（`Activate<T>`、准入）合入

## 涉及范围（拥有的文件集）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/World/WorldManager.cs`（客户端路径：`ConfirmedWorld` / `PredictedWorld`、重建触发）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/Prediction/**`（新：克隆、输入历史、重放、表现键差集、误差仪表）
- 输入历史挂在 R5-02 codec 的 `EncodeInput`（已分配 `sequence`）/ `DecodePack`（已暴露 `appliedInputSequence`）之上——本卡不改 codec，只加历史与重建
- `modules/ecs/samples/username/**` 与 `modules/gas/samples/bomber/**`（客户端预测示范与同进程双端环回测试）
- 对应测试项目

## 来源真值
- ADR-064 第 7 / 8 / 9 / 10 条；ADR-063 第 7 条；`gas.md` M7 ①②③⑤、M10 ③；`ecs.md` M10 ①②④、§6「第二个世界」例外；`tick.md` §4（预测档系统在客户端预测世界跑）、§6；`bomber-slice.md` §5 场景 3 / 5；差异审计 D07、D17
- 现状证据：LumioClient `AuthorityUpdateOrchestrator.cs:52` 固定 `AuthorityPredictionUpdate(update, 0)`、`reconcile` 未消费——那条路径由 R5-03 删除，预测归本卡

## 产品背景与已锁决策
1. 玩家按右键，客户端立刻把人往右画——这是在**预测世界**里跑了一遍移动技能。
2. 服务器的包到了，先写进**确认世界**（只有 `DecodePack` 能写它）。
3. 然后把预测世界扔掉：从确认世界整体克隆一份（全部实体、全部 `Sync` 与共享普通字段；体素不克隆、只读确认世界的体素），再把 `sequence > appliedInputSequence` 的本地输入按序重放——每条输入 = 在预测世界跑 `ApplyInputs`（同一段 `Activate` / 准入 / 执行）→ 预测档系统 → 本地结构提交。
4. 预测世界里新建的实体拿本地临时号，不上网、不进哈希、随下一次重建作废；不可预测清单（Effect 移除 / 周期 / 出模拟域）在预测世界不执行；预测世界不产 outbox。
5. 画面只读预测世界，按**表现键**（EntityType + `fx_key` + 稳定业务参数）做差：同键继续、键消失结束、新键开始；一次性表现按（输入序号, `fx_key`）去重。
6. 克隆走模板池（ADR-060 第 11 条），池热后零分配；整体克隆超预算是回图信号，先不收窄。
7. 输入历史有界；超上限 = 停止预测直到追上，不伪造确认。误差仪表：重建前记录预测世界与确认世界同 tick 的 `LogicTransform` 差，只做诊断。
8. 对账哈希只在四元组成立时比：同 tick、同可见集、同字段集（同步域 `Sync`，排除 `Scope.None` / 预测 / 表现）、确认世界对该观察者的服务器投影。

## 详细要求（逐条照做）
1. `WorldManager`（客户端）持 `ConfirmedWorld` + `PredictedWorld`；`DecodePack` 提交进确认世界后触发 `Rebuild()`。
2. `Rebuild()`：克隆（按类型模板克隆，含共享普通字段）→ 重放输入历史中 `sequence > appliedInputSequence` 的条目 → 每条跑客户端第 3 / 4 相（RT-1 钩子）→ 本地结构提交（临时号）。
3. 输入历史：每次 `EncodeInput`（R5-02 已分配 `sequence`）后入历史；`appliedInputSequence` 到达即裁剪；上限可配，超限停止预测。
4. 表现键差集：`IPresentationDiff { Started, Continued, Ended }` 每次重建后发布；表现键由实体类型 + 组件声明的 `[PresentationKey]` 字段（或等价声明，名字自定）派生；一次性表现去重表按（输入序号, `fx_key`）。
5. 误差仪表 + 对账哈希：`SnapshotHashMetrics` 相在客户端对确认世界按四元组算哈希；预测世界不进哈希。
6. 同进程双端环回测试（Runtime 内，不依赖 Client 仓）：150 ms 人为延迟，A 手上两颗、在格 X 按放弹（17）、走到格 Y 再按（18），17 到服务器前 B 已在 X 放弹 → 服务器拒 17 收 18；改格用例；重放十次去重（预测世界里的一次性预测表现，不是 `OnFx`）；预测世界无体素；本地号不上行。

## 验证计划与证据
ADR-064 验证 Fixture 6 / 7 / 8 / 10 的测试输出；克隆分配计数（池热后零）；`grep` 客户端源码无认领 / 搬特效；预测世界无 Section 引用（结构断言）。

## 接口
- Consumes：RT-1 的客户端第 3 / 4 相钩子；RT-4 的 `Activate<T>` / 准入结果；R5-02 的 codec 与模板池；R5-01 的 `sequence` / `appliedInputSequence`。
- Produces：`WorldManager.ConfirmedWorld` / `PredictedWorld`、`IPresentationDiff`、输入历史 API、误差仪表（LumioClient 表现层与 CL-1 的 WASM 页面依赖）。

## 验收标准
1. A 两颗炸弹按 17（格 X）、18（格 Y），B 抢先占了 X → 服务器拒 17 收 18：重建后 17 消失、18 与正式炸弹表现键相同且控制器对象同一个；改格用例旧键结束新键开始各一次
2. 同一未确认输入重放十次，预测世界里挂的一次性预测表现计数 = 1（`OnFx` 不在此列）；预测世界无体素快照；本地临时号不出现在任何上行包与哈希
3. 对账哈希四元组：AOI 半图下与服务器对该观察者投影的哈希一致；改一个同步字段下一 tick 报漂移；改预测世界位置哈希不变
4. 克隆池热后零分配；输入历史超限停止预测不伪造；两轮日志逐位一致

## 明确不做与禁止事项
不做认领键 / 改号 / 搬特效；不克隆体素；不做「只克隆 Self」优化；不在 LumioClient 仓写代码（Client 表现层接入归其后续卡）。

## 阻塞与升级
整体克隆在 100 实体 × 20 包/秒下超帧预算 → 上报数据，不自行收窄克隆域。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. 两颗预测炸弹（B 抢占格 X）拒 17 收 18：消失 / 同键不闪；改格各一次
2. 重放去重、预测世界无体素、本地号不上行
3. 对账哈希四元组三个用例
4. 克隆零分配、历史超限停止预测、两轮一致
<!-- /card -->

<!-- card:RT-4 -->
- title: [程序·Runtime][Wave 4] GAS M2：`AbilityComponent` 技能条目、八态机、准入五步（③ 消耗 = 属性基础账）、Commit 判定、执行时限；`[AbilityType]` + `Activate<T>` 唯一写法；`Attribute` 一处声明展开两本账；`modules/gas` 重写与 README；炸弹人样板骨架
- category: runtime / gas
- priority: P0
- risk: high
- wave: 4
- 前置: RT-1 合入
<!-- body -->
# 执行提示词：[程序·Runtime][Wave 4] GAS M2 + Attribute 声明展开

`workflow-plan: bomber-engine/RT-4`

## 任务元数据
- 目标仓库：`LumioGameRuntime`
- 责任角色：GAS 框架工程师（`modules/gas` 与生成器 GAS 部分唯一实现方）
- 前置状态：RT-1 合入（`System`、13 相唯一路径、`Ticks.FromMilliseconds`）

## 涉及范围（拥有的文件集）
- `modules/gas/**`（`Identity/` 保留并对齐；`Lifecycle/` 改；新增 `Ability/`、`Attribute/`；`README.md` 重写；`samples/bomber/**` 新）
- `tools/gen-declarations/**`（`[AbilityType]` 注册、`AbilityType.输入` codec、`Activate<T>` 生成 ServerRpc 桩、`AbilityComponent` 条目类型、`Attribute` 声明展开为两个 `Sync` 字段；本 wave 生成器唯一所有者）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/Components/AbilityComponent.cs`、`AttributeComponent.cs`（引擎组件；新）
- 对应测试项目

## 来源真值
- ADR-064 第 1 / 2 / 3 / 6 / 11 条与「接口 / Schema」「失败语义」；`gas.md` §2 四组件、M1、M2（含 ⑧）、M5 ④、M7 ⑤；`ecs.md` M4 ①②③⑥⑦；`bomber-slice.md` §4 ①②③；差异审计 D03、D09
- 现状证据：`modules/gas/src/.../Lifecycle/GasWorldContext.cs` 只有 Framework 状态机与句柄索引（注释「Holds handle indexes only」）；`modules/gas/README.md:39` 仍标「候选接口」；`Bomber/` 在 LumioGame 只有 Contracts

## 产品背景与已锁决策
1. 玩家按放弹 → 客户端 `Self.Get<AbilityComponent>().Activate<放弹技能>(输入)`——调用即上行（生成的 ServerRpc，信封带 `sequence`）。
2. 服务器 `ApplyInputs` 相：准入五步（① 句柄权限：调用者是绑定者且实体类型声明了该技能 ② 冷却：条目上的下次可用帧 ③ 消耗：某属性基础账 ≥ 代价 ④ Tag：空表通过 ⑤ `可以激活吗`）→ 任一步败 = Rejected 带步序号、不扣 → Activated → Commit 判定只复查 ② ③ 并真扣 → Executing → `执行(输入)` 同帧连转 → 返回或 `End()` = Completed；忘了 End 到时限 = Expired 清场。
3. 技能条目住 `AbilityComponent`（`SyncList<技能条目>(Scope.Owner)`）；终态即出表；瞬时技能同帧进出折叠零字节。句柄 = ECS 下标 + 世代号。
4. 属性由玩法在 `AttributeComponent` 的 partial 里一处声明（`Attribute 血量 = new(初值: 6)`），生成器展开成 `[Persist] Sync<long> 血量基础 = new(Scope.Owner)`（只给绑定者自己——客户端预测世界要跑同一段准入 ③ 与扣减；无绑定者的实体零字节）+ `Sync<long> 血量当前 = new(Scope.Aoi)`；本卡当前账 = 基础账直拷贝（RT-5 换成求值）；「修订号」= 包级 revision，不另设字段。
5. 技能类型 = `[AbilityType(TypeId, Prediction = 档位, 消耗 = nameof(属性))] class X : AbilityType { struct 输入; 可以激活吗; 执行; }`，注册表随生成三件；TypeId 切片内用代码常量。
6. `modules/gas/README.md` 按现行口径重写（删 Baseline / 候选接口 / PredictionKey-PredictionFrame 旧词）。
7. 样板 `modules/gas/samples/bomber/`：`玩家属性`（血量 / 火力 / 移速 / 手上炸弹数）、`移动技能`、`放弹技能`、`炸弹` EntityType + `炸弹状态`、Host 环回；地形先用样板内二维数组代替体素（正式接体素归 LumioGame 切片），与 `bomber-slice.md` §4 逐文件一致。

## 详细要求（逐条照做）
1. `AbilityComponent` + 条目类型 + 八态转移表 + 准入五步 + Commit 判定 + 执行时限 + `RolledBack` 入口（只在预测世界，RT-3 接）；准入结果类型带步序号。
2. `AbilityType` 基类（`输入` struct 约束、`可以激活吗`、`执行`、`End`、`时限`）；`[AbilityType]`；生成注册表与 `Activate<T>` ServerRpc 桩（`InputCommand` 生成 ServerRpc 种类）。
3. `AttributeComponent` + `Attribute` 声明展开（两个 `Sync` 字段）+ 基础账扣减 API（供准入 ③）。
4. 服务器 `ApplyInputs` 相接线：`Activate` 记录 → 准入 → 执行；客户端本地路径只暴露入口（预测世界由 RT-3 驱动）。
5. README 重写；样板骨架与测试。

## 验证计划与证据
ADR-064 Fixture 1 / 2 / 9 / 11（技能标非业务相 → 生成失败）的测试输出；状态机转移表穷举测试；样板放弹链路：`Activate` → 准入 → 建炸弹 → 抓包里技能条目零字节；`grep -rn "候选接口\|Baseline\|PredictionKey\|PredictionFrame" modules/gas` 零命中。

## 接口
- Consumes：RT-1 的 `System` / 13 相 / `Ticks.FromMilliseconds`；R5-02 的 `SyncList` / 生成三件 / codec。
- Produces：`AbilityComponent.Activate<T>(in T.输入)`、`AbilityType`、`[AbilityType(TypeId, Prediction, 消耗)]`、准入结果（步序号）、`AttributeComponent` + `Attribute` 声明展开（`X基础` / `X当前`）、`GeneratedAbilityRegistry`（RT-3 / RT-5 / LumioGame 依赖）。

## 验收标准
1. 准入五步：手上炸弹数 0 → Rejected 步序号 3；这格已有炸弹 → 步序号 5；两者不扣；Commit 复查失败 → Cancelled 不扣
2. 八态转移表穷举测试全绿；不 `End` 的测试技能到时限 Expired、句柄失效
3. `Attribute` 一处声明展开两本账，`[Persist]` 打在当前账 → 生成失败；技能标非业务相 → 生成失败
4. 样板放弹链路跑通，技能条目在包里零字节；`modules/gas` 零旧制度措辞

## 明确不做与禁止事项
不做 Effect / 求值 / 重算（RT-5）；不做 Tag 表与握手；不做挂起点 / 打断三积木的验收（接口可留）；不做配表接入；不改 `engine/wire`。

## 阻塞与升级
`SyncList` 条目类型嵌套层数（ecs M4 ② 两层警告）装不下技能条目的应用快照 → BLOCKED，写清字段集，其余照做。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. 准入五步与 Commit 判定的拒绝 / 取消带步序号且不扣
2. 八态穷举 + 执行时限
3. Attribute 声明展开 + 两条生成负例
4. 样板放弹链路 + README 零旧词
<!-- /card -->

<!-- card:RT-5 -->
- title: [程序·Runtime][Wave 5] GAS M3 / M4 / M5：`EffectComponent` 与瞬时效果单、整数冻结公式与静态拓扑序、当前账提交相尾一次重算、击杀 = 跨零、`OnFx` ClientRpc 记录；炸弹人样板爆炸系统 + 死亡系统
- category: runtime / gas
- priority: P0
- risk: high
- wave: 5
- 前置: RT-4 合入
<!-- body -->
# 执行提示词：[程序·Runtime][Wave 5] GAS M3 / M4 / M5

`workflow-plan: bomber-engine/RT-5`

## 任务元数据
- 目标仓库：`LumioGameRuntime`
- 责任角色：GAS 框架工程师
- 前置状态：RT-4 合入

## 涉及范围（拥有的文件集）
- `modules/gas/**`（新增 `Effect/`、`Evaluation/`；`samples/bomber/**` 加伤害 / 爆炸系统 / 死亡系统；README 补 M3–M5）
- `tools/gen-declarations/**`（`[EffectType]`、`Effects.Apply<T>` 桩、拓扑序表、`OnFx` ClientRpc 生成；本 wave 生成器唯一所有者）
- `modules/ecs/src/Lumio.GameRuntime.Ecs/Components/EffectComponent.cs`（引擎组件；新）
- 对应测试项目

## 来源真值
- ADR-064 第 2 / 4 / 5 / 6 条与「接口 / Schema」「失败语义」；`gas.md` M3（含 ⑥）、M4（含 ⑦）、M5、M8 ②③④、M10 ②；`ecs.md` M4 ③⑦⑧；`tick.md` §2 第 9 / 10 相、§3 规则 3；`bomber-slice.md` §4 ③④、§5 场景 1

## 产品背景与已锁决策
1. 引信到点，爆炸系统对每个被火打到的人 `Effects.Apply<伤害>(人, {点数 = 2}, 来源: 弹.主人)`——只下单。
2. 第 9 相尾按单序（系统序 + 下单序）**在结算中的基础账上**结算：校验（目标不存在 / 基础账已 ≤ 0 / 免疫 → Rejected）→ Active → 瞬时效果的 Modifier **直接改基础账**（下一张单读到的就是改后的值）→ 同帧 Expired 出表；持续效果的 Modifier 只进当前账（本卡实现入口与测试，切片不验收）；当前账不参与结算期判定。
3. 标脏属性按编译期拓扑序重算当前账**一次**：`当前 = (基础 + Σ加法) × (1000 + Σ千分比) / 1000` 向零取整，覆盖按显式优先级、同级后写赢，定序求和，全整数 `long`；成环 = 生成报错。
4. **击杀 = 跨零，生死看基础账**：让基础账从 > 0 变 ≤ 0 的那张单是击杀单，来源即击杀者；对基础账已 ≤ 0 目标的后续单 Rejected、无 `OnFx`。能被瞬时效果直接改的属性（血量）不得挂持续修饰，基础 = 当前恒成立；死亡态 = 血量基础 ≤ 0，不另设字段。
5. 第 10 相产出 `OnFx`：`EffectComponent` 上引擎生成的 `[ClientRpc(Scope.Aoi)] OnFx(fxKey, 参数)`（参数含来源、目标、结果值、跨零标记），走 C-1″ `rpcs`（R5-01 已有用例），不加记录种类；**不是**组件字段。
6. 同帧事件序 命中 → 溢出 → 快照替换 / 层数 → 时长 → 周期 → 移除垫后，顺序表钉死并有测试；本卡只实现 命中 → 移除 路径，其余留明确未实现入口（不伪装）。
7. 样板：`伤害` EffectType、`爆炸系统`（帧初读样板数组、连锁队列、下伤害单、帧末写数组）、`死亡系统`（下一帧读当前账 ≤ 0 下结构单）——与 `bomber-slice.md` §4 ③④ 逐文件一致。

## 详细要求（逐条照做）
1. `EffectComponent`（`SyncList<效果条目>(Scope.Owner)`）+ 六态转移表 + 瞬时子集结算 + 校验拒绝。
2. `Effects.Apply<T>(目标, in 参数, 来源)` 下单 API + `[EffectType(TypeId, 瞬时)]` + `EffectType.应用` 基类 + 生成注册。
3. 求值器：整数公式、覆盖优先级、拓扑序表编译期生成、重算计数器；`X当前` 从 RT-4 的直拷贝换成求值。
4. 击杀跨零 + 已死拒单 + `OnFx` 生成与第 10 相产出。
5. 样板伤害 / 爆炸系统 / 死亡系统 + 场景 1 测试（两轮逐位一致）。

## 验证计划与证据
ADR-064 Fixture 3 / 4 / 5 / 11（`[Persist]` 打当前账、`FxComponent` → 生成失败）的测试输出；样板场景 1：同帧连锁、两道火两张单 6 → 2、第三张跨零记击杀且来源正确、第四张 Rejected 无 `OnFx`、重算计数 = 1、死亡下一帧结构提交、两轮哈希一致；抓包：旁人收到 `血量当前` 与 `OnFx`、收不到效果明细与基础账。

## 接口
- Consumes：RT-4 的 `AbilityComponent` / `AttributeComponent` / 声明展开 / 注册表；RT-1 的第 9 / 10 相；R5-01 的 `OnFx` rpcs 用例。
- Produces：`Effects.Apply<T>`、`EffectType`、`[EffectType]`、`EffectComponent`、求值器与拓扑序表、`OnFx` 记录形状（LumioGame 炸弹人接入与 RT-3 表现键去重依赖）。

## 验收标准
1. 瞬时 Effect：按单序在基础账上 6 → 2、第三张让基础跨零记击杀且来源正确、基础已 ≤ 0 目标后续单 Rejected 无 `OnFx`；四张同帧当前账重算计数 = 1
2. 求值：100 / +20 / +100‰ → 132 两台机器逐位相同；成环生成失败；`[Persist]` 打当前账 / `FxComponent` → 生成失败
3. `OnFx` 走 `rpcs` 记录、`Scope.Aoi`；旁人收不到效果明细与基础账（抓包证据）
4. 样板场景 1 两轮逐位一致；死亡系统下一帧下结构单

## 明确不做与禁止事项
不做堆叠 / 时长 / 周期 / 抑制的验收（顺序表有测试即可）；不做 Tag；不做帧调度器；不做求值下沉 Rust；不改 `engine/wire`。

## 阻塞与升级
`OnFx` 参数按 fx 声明编码超出 LumioBinV1 阶梯 → BLOCKED 上报契约缺口。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. 瞬时 Effect + 击杀跨零 + 已死拒单 + 重算一次
2. 整数求值两机一致 + 三条生成负例
3. OnFx 走 rpcs、Scope.Aoi、明细不外泄
4. 样板场景 1 两轮一致、死亡下一帧
<!-- /card -->

<!-- card:CL-1 -->
- title: [调研·Client][Wave 1] LumioClient：.NET WebAssembly 能否在浏览器里跑 Runtime 客户端模块（C-1″ codec + 确认世界 + 预测世界重建）——可行性、包体积、冷启动、每包重建耗时、JS 互操作方案；交付报告 + ADR 草案建议，不进 `modules/`
- category: research / client
- priority: P1
- risk: medium
- wave: 1
- 前置: 无（用现行 Runtime `origin/main` 程序集即可，不等 R5-02）
<!-- body -->
# 执行提示词：[调研·Client][Wave 1] 浏览器跑 Runtime 预测的 WASM 可行性

`workflow-plan: bomber-engine/CL-1`

> **正文唯一源已迁移**：Owner 2026-09-05 追加「预研分手机浏览器与 Chrome 浏览器两种」，完整提示词（两轨调研问题、现状事实表、探针工程约束、报告体例、验收）见 [`2026-09-05-cl-1-runtime-wasm-spike-prompt.md`](2026-09-05-cl-1-runtime-wasm-spike-prompt.md)。派活与 Workflow 回写以该文件为准；下文只保留元数据与在 DAG 里的位置，两处不一致以该文件为准。

## 任务元数据
- 目标仓库：`LumioClient`
- 责任角色：客户端平台工程师（调研，不交实现）
- 前置状态：无

## 涉及范围（拥有的文件集）
- `docs/spikes/2026-09-<日>-spike-runtime-wasm.md`（新；沿用 `docs/spikes/2026-08-28-spike-hybridclr-63.md` 的体例）
- `spikes/runtime-wasm/**`（新；一次性探针工程，**不进 `modules/`**，不进 `eng/project-reference-allowlist.json`）
- 不改任何现有模块源码与 CI

## 来源真值
- Owner 2026-09-05：浏览器预测路径「现在就调研 WASM」；LumioGame ADR 0013（客户端暂定浏览器、首发不接游戏引擎）；架构仓 `architecture.md` §1 / §2（LumioClient 职责）；ADR-064 第 8 条（预测世界 = 确认世界克隆 + 重放，表现层只读预测世界）；差异审计 D11
- 现状证据：`modules/web/` 只有 Hello / Chat 纯静态页（无构建、无框架）；`modules/unity-adapter/src/` 无实现源；Runtime 程序集 `net10.0;netstandard2.1` 双目标（`global.json` SDK 10.0.100）

## 调研问题（逐条给出实测数据，不给推断）
1. **能不能跑**：.NET 10 `browser-wasm` 目标（`Microsoft.NET.Sdk.WebAssembly` 或 `wasm-experimental`，以官方文档为准）能否加载 `Lumio.GameRuntime.Ecs`（含 `WorldManager.Create` 客户端路径）与 C-1″ codec，经浏览器 `WebSocket` 连上现行 Rust 宿主并解出一条真实 `WorldChange` 包。给出复现步骤与截图 / 控制台输出。
2. **多大多慢**：wasm + dll（压缩后）总体积；冷启动到 `WorldManager` 可用的时间；一个 100 实体 × 每实体 3 组件的世界，「整体克隆 + 重放 5 条输入」一次耗时（模拟 ADR-064 第 8 条的重建）；20 包/秒下每帧预算占比。AOT（`RunAOTCompilation`）开与不开各测一组。
3. **怎么接画面**：C# 侧只吐「表现键差集」（RT-3 的 `IPresentationDiff` 形状），JS / Canvas 负责画——`[JSExport]` / `[JSImport]` 互操作的调用开销与每帧数据量；是否需要 SharedArrayBuffer / 线程。
4. **三条路线的成本对照**（表格）：A .NET WASM 跑 Runtime 客户端模块（一处维护）；B 浏览器只画不预测（预测只在 C# 客户端 Bot.Host / 桌面壳）；C 用生成的 JS / TS 第二份预测实现——C 违反「一处维护」，只作对照，不推荐。
5. **对照组**：现有纯 JS 聊天页的体积与冷启动，作为基线。

## 交付
- 报告：数据表（命令 + 关键输出逐条附）、可行 / 不可行结论、三条路线成本对照、推荐路线一句话 + 理由、ADR 草案建议（标题与决策条目，不落 ADR 文件——ADR 归架构仓）。
- 探针工程留在 `spikes/`，可复现（README 写清 SDK 版本与命令）。
- known gaps：没测到的（移动端浏览器、Safari、断网重连）逐条列出。

## 验证计划与证据
- 至少一个 WASM 页面加载 Runtime 程序集并解出真实 C-1″ 包的证据（控制台输出 + 抓包）；
- 体积 / 启动 / 重建耗时的原始数字与测量方法；
- `git status` 证明 `modules/**` 与 CI 零改动。

## 接口
- Consumes：Runtime `origin/main` 的 `Lumio.GameRuntime.Ecs` 程序集与 C-1″ codec（R5-02 之前的形态即可）；架构仓 `engine/wire/gameplay-command-envelope-v1.json`。
- Produces：可行性报告与 ADR 草案建议（架构仓据此开「浏览器客户端预测路径」ADR，决定 LumioClient 后续卡）。

## 验收标准
1. 报告含 1–5 每条的实测数据与复现步骤；无「应该可以」式推断
2. WASM 页面加载 Runtime 程序集并解出一条真实 `WorldChange` 包（证据在报告）
3. 三条路线成本对照表 + 推荐 + ADR 草案建议
4. `modules/**`、CI、allowlist 零改动；探针工程可复现

## 明确不做与禁止事项
不写正式客户端；不改 Runtime；不引入 CDN / 框架进 `modules/web`；不替 Owner 下结论（推荐是建议）。

## 阻塞与升级
Runtime 程序集在 `browser-wasm` 下编译失败（`netstandard2.1` 依赖或 `System.Threading.Channels` 之类）→ 记录错误原文，改用最小子集探针继续测体积 / 互操作，报告标 BLOCKED 项。

## 交回格式
按共同执行规范五段。
<!-- acceptance -->
1. 五条调研问题全部有实测数据与复现步骤
2. WASM 加载 Runtime 程序集并解出真实包
3. 三路线对照 + 推荐 + ADR 草案建议
4. modules / CI 零改动，探针可复现
<!-- /card -->
