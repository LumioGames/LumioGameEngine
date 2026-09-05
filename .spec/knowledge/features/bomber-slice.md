---
name: bomber-slice
description: 炸弹人战斗切片的引擎验收需求真值——世界模型套用、引擎能力组合、第二样板与五组验收场景;排引擎卡或改引擎验收标准前查
metadata:
  type: doc
  status: 设计中
---

# 炸弹人战斗切片 · 引擎验收需求真值

> 引擎的验收主线从聊天切片（[`ecs-entity-chat.md`](ecs-entity-chat.md)）改为战斗切片（[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) 第 13 条）：一局炸弹人就能撞出帧内结算、身份、预测三组根问题，聊天撞不出来。**切片的 GAS 面**按 [ADR-064](../../decisions/ADR-064-gas-slice-contracts.md)（Owner 2026-09-05：技能八态准入、瞬时 Effect、整数求值、两本账、预测都进切片）。
> **分工**：玩法规则、数值、帽子经济、地图生成归 `LumioGame`（其 `docs/specs/bomber/design.md` 与 ADR 0014–0018）；本文只定**引擎要证明的能力**与**验收场景**。文中数字（61×61、2.1 秒引信 = LumioGame kernel contract `fuseMs = 2100`、火力 2、3.5 格/秒、6 个半心）取自 LumioGame 现行口径，只作例子，不是引擎契约。

---

## 1. 目标

```text
玩家按键立刻动（移动技能，逻辑预测）→ 放弹（技能准入五步 → 预测世界里建实体）→ 引信到点连锁爆炸（同帧帧内结算）
  → 木箱变空气（体素批量写）→ 伤害 Effect 单提交相结算 / 击杀 = 跨零 / 死亡下一帧结构单（掉落、销毁）
  → 所有客户端同帧看到（复制 + OnFx）→ 死者重新入场（身份三态）→ 两轮跑出同一份哈希（一致性）
```

跑通这一条 = ECS / GAS / 体素 / Tick / DS 五份概要的语义在一款真实游戏里成立。

## 2. 世界模型套用（[`rules/system.md`](../../rules/system.md) 四问）

| 东西 | 要不要服务器逻辑 | 动不动 | 判定 | 落点 |
|---|---|---|---|---|
| 地面、硬砖、软砖（木箱）、水 | 不要（它们本身没逻辑；挡路 / 透光由材质类表定） | 不动 | **体素** | 官方全局段 8 种 Solid + 水 Liquid（[`voxel.md`](voxel.md) M1a；LumioGame 答复 §①） |
| 玩家 | 要 | 动 | **CS 实体** | `PlayerEntity`：`LogicTransform` + `玩家属性`（血量 / 火力 / 移速 / 手上炸弹数，每个两本账）+ `AbilityComponent`（移动 / 放弹两个技能条目）+ `EffectComponent`（伤害单入表、`OnFx` 挂它上） |
| 炸弹 | 要（引信、连锁、挡路） | 引信到点变火焰 | **CS 实体** | `炸弹`：格子、主人、到期帧、火力、四臂到达长度；引信 → 爆炸态 → 留火 → 销毁（LumioGame ADR 0017） |
| 帽子堆、掉落物 | 要（能被捡） | 不动但有生命周期 | **CS 实体** | 结构单生成 / 销毁 |
| 火焰画面、爆炸特效、客户端预测用的炸弹 | 不要 | — | **Local 实体** | 由炸弹实体的四臂长度 / `fx_key` 长出来；预测世界里的预测炸弹随重建出现或消失 |
| 移动、放弹 | — | — | **GAS Ability**（`AbilityComponent` 上的技能条目） | 两端跑同一段代码，档位「逻辑预测」；预测归 [`gas.md`](gas.md) M7 / ADR-064 第 8 条 |
| 伤害 | — | — | **GAS Effect**（瞬时） | 爆炸系统下单，提交相结算改血量基础账；击杀 = 跨零（ADR-064 第 4 条） |

没有第三种东西。「炸弹做成地图上的列表记录」「爆炸格每格一个实体」都被否（前者违反一切皆实体，后者 100 人下每秒百级实体生灭——LumioGame ADR 0017）。

## 3. 引擎能力组合（本切片要求引擎证明的，按依赖顺序）

| # | 能力 | 引擎落点 | 现状 / 缺口 |
|---|---|---|---|
| 1 | 游戏系统注册进 Tick 的第 3 / 4 相；`WorldManager.Tick()` 只有 13 相一条路径 | [`tick.md`](tick.md) §4 | **缺口**：Runtime 无公开接口、`WorldManager.Tick()` 走私有五步序列——Runtime 卡 RT-1（ADR-063 第 14 条 ①） |
| 2 | 双 Transform：`LogicTransform` 权威上网、`ModelTransform` 客户端插帧；同帧双写者启动报错 | [`ecs.md`](ecs.md) M7 | **缺口**：四仓源码零命中——Runtime 卡 RT-2（ADR-063 第 14 条 ⑥） |
| 3 | 移动预测：同一段技能代码两端跑，转角缓冲等工作状态放共享文件 | [`gas.md`](gas.md) M7、[`ecs.md`](ecs.md) M4 ①（共享文件普通字段 + lint） | lint 口径已定（ADR-063 第 5 条），生成器随 R5-02 落 |
| 4 | 放弹预测：客户端预测世界里 `Commands.Create<炸弹>()`，包级 `appliedInputSequence` 驱动重建；表现键做差保证连续 | [`gas.md`](gas.md) M7 ①②、[`ecs.md`](ecs.md) M10、ADR-064 第 7 / 8 / 9 条 | C-1″ `sequence` / `appliedInputSequence` 随 R5-01；预测世界重建归 Runtime 客户端模块——Runtime 卡 RT-3（ADR-063 第 14 条 ⑦） |
| 5 | 技能八态准入：放弹走五步（③ 消耗 = 手上炸弹数、⑤ 这格能不能放）、Commit 判定、执行时限；激活只有 `Activate<T>` 一种写法 | [`gas.md`](gas.md) M2、ADR-064 第 3 条 | **缺口**：Runtime `modules/gas` 只有句柄索引——Runtime 卡 RT-4（ADR-063 第 14 条 ⑧） |
| 6 | 属性两本账：一处声明、生成基础账 + 当前账；瞬时伤害 Effect 单改基础账、当前账拓扑序重算一次；击杀 = 跨零、已死拒单 | [`gas.md`](gas.md) M3 / M4 / M5、ADR-064 第 2 / 4 / 5 条 | **缺口**——Runtime 卡 RT-5 |
| 7 | 帧初批量读整图、帧末一批写、同帧多批合一 | [`tick.md`](tick.md) §3、[`voxel.md`](voxel.md) M6 ①c / M7a / M8 ③a（pin） | 体素卡 I-4 / I-5 / I-6（`plans/2026-09-05-voxel-impl-dispatcher-prompt.md`） |
| 8 | 改动层按 Section 派发，`Delta` 增量 | [`voxel.md`](voxel.md) M5 | 体素卡 I-3 / I-10 |
| 9 | 死亡销毁记录带 `terminated`、击杀走 `OnFx` 跨零标记、掉落 / 销毁由死亡系统下一帧下结构单 | [`ecs.md`](ecs.md) M4 ⑦、M5 ③、M6；ADR-064 第 2 / 4 条 | 销毁记录 `reason` 随 R5-01；`OnFx` 记录随 RT-5 |
| 10 | 重新入场：5 分钟内 rebind 同一实体，超时新实体、旧号答墓碑（服务器）/ 未知（客户端） | [`ecs.md`](ecs.md) M10 ③、M5 ③ | 已在 RM-00011 验收面 |
| 11 | `Sync<NetEntityId>`（炸弹的主人、击杀者） | [`ecs.md`](ecs.md) M4 ② | 随 R5-02（ADR-063 第 14 条 ②） |
| 12 | 每帧轻量哈希对账，比较条件按四元组（同 tick / 同可见集 / 同字段集 / 该观察者投影） | [`ecs.md`](ecs.md) M10 ④、[`ds-server.md`](ds-server.md) M11、ADR-064 第 10 条 | 服务器回放哈希已在 RM-00011 验收面；四元组对账随 RT-3 |
| 13 | tick 频率作为 `WorldEntity` 配置，`毫秒换帧` 引用它 | [`tick.md`](tick.md) §5 | 字段名随 RT-1 定 |

**不在本切片**：Tag 表与握手（准入第 ④ 步空表通过）、帧调度器（瞬时效果无到期）、配表 TypeId（切片内代码常量注册）、挂起点 / 打断三积木（接口保留不验收）、堆叠 / 持续效果、存档与崩溃恢复、AOI 分帧与休眠、多房间、浏览器预测（LumioClient 调研卡 CL-1 先出 WASM 可行性；引擎验收在 C# 客户端 Bot.Host / 同进程双端跑）。它们各有自己的切片，不阻塞这条主线。

## 4. 第二样板（以后所有战斗类 ECS / GAS 代码与讨论以此为标准；与 §4.5 聊天样板并列）

**① 移动是一个技能，代码两端跑；工作状态放共享文件。**

```csharp
// Abilities/移动/移动技能.cs —— 共享文件，两端都编：服务器算权威位置，客户端在预测世界里先算一遍
[AbilityType(TypeId = 1, Prediction = 档位.逻辑预测)]
public sealed partial class 移动技能 : AbilityType
{
    public struct 输入 { public 方向 方向; public bool 按了转弯; }
    public int 转角缓冲剩余帧;                 // 普通字段：两端各算各的，不上网、不存档；lint 允许（两端都赋值）

    public override bool 可以激活吗(in 输入 输入) => Get<玩家属性>().血量基础 > 0;   // 准入第 ⑤ 步：死人不能动（生死看基础账；血量不挂持续修饰，基础 = 当前）

    public override void 执行(in 输入 输入)   // 准入五步过了同帧连转到这里；方法体只写一份
    {
        var 位置 = Get<LogicTransform>();
        if (输入.按了转弯) 转角缓冲剩余帧 = 6;                     // 提前按了转弯，记住 6 帧
        if (到了路口(位置) && 转角缓冲剩余帧 > 0) 转向(位置, 输入);   // 到路口自动转，手感靠这个
        位置.逻辑位置.Value = 前进一格的几分之一(位置, 输入, Get<玩家属性>().移速当前);   // 写 Sync 字段 = 记账；服务器权威、客户端预测
        转角缓冲剩余帧--;
    }   // 返回即 Completed；瞬时技能条目同帧进出，折叠成零字节
}
// 客户端按键：Self.Get<AbilityComponent>().Activate<移动技能>(输入) —— 调用即上行（生成的 ServerRpc，信封带 sequence），同一帧在预测世界里本地跑同一段
// 服务器只有 LogicTransform；客户端另有 ModelTransform（.Client.cs），每个渲染帧往逻辑位置滑，永不写回
```

**② 放弹：准入五步，预测世界里建实体，通过不管、没通过随重建消失。**

```csharp
// Abilities/放弹/放弹技能.cs —— 共享文件
[AbilityType(TypeId = 2, Prediction = 档位.逻辑预测, 消耗 = nameof(玩家属性.手上炸弹数))]   // 准入第 ③ 步：手上炸弹数基础账 ≥ 1；Commit 复查后才 -1
public sealed partial class 放弹技能 : AbilityType
{
    public struct 输入 { }
    public override bool 可以激活吗(in 输入 _) => !这格已有炸弹(Get<LogicTransform>().所在格);   // 准入第 ⑤ 步：这格能不能放

    public override void 执行(in 输入 _)
    {
        var 单 = World.Commands.Create<炸弹>();                       // 服务器：提交相发号；客户端预测世界：本地临时号，随重建作废
        单.Get<炸弹状态>().格子.Value = Get<LogicTransform>().所在格;
        单.Get<炸弹状态>().主人.Value = Self;                          // Sync<NetEntityId>
        单.Get<炸弹状态>().到期帧.Value = World.Tick + 毫秒换帧(2100);   // 换算率 = WorldEntity 的 tick 频率；2100 取自 LumioGame fuseMs
        单.Get<炸弹状态>().火力.Value = Get<玩家属性>().火力当前;
    }
}
// 手上炸弹数 0 → Rejected 步序号 3；这格已有炸弹 → 步序号 5；两者都不扣。
// 客户端：按键 → 预测世界里跑上面这段 → 画面上炸弹立刻出现。服务器包到达（带 appliedInputSequence）→ 预测世界从确认世界重建：
// 表现键 = (炸弹, fx_key, 格子, 主人)：通过 = 同键仍在，画面不变；服务器把我按停在上一格才放 = 旧键结束新键开始，炸弹换位；被拒 = 键消失。没有对号、没有换特效。
```

**③ 属性一处声明、生成两本账；伤害是瞬时 Effect。**

```csharp
// Components/玩家属性/玩家属性.cs —— 共享文件；每个属性写一行，生成器展开成 X基础（[Persist] Sync<long>(Scope.Owner)，只给绑定者自己——预测世界要跑准入 ③）+ X当前（Sync<long>(Scope.Aoi)）
[EcsComponent]
public sealed partial class 玩家属性 : AttributeComponent
{
    public Attribute 血量 = new(初值: 6);          // 6 个半心点（数值归 LumioGame）
    public Attribute 火力 = new(初值: 2);
    public Attribute 移速 = new(初值: 3500);       // 千分格 / 秒
    public Attribute 手上炸弹数 = new(初值: 1);
}

// Effects/伤害.cs —— 共享文件
[EffectType(TypeId = 10, 瞬时 = true)]
public sealed partial class 伤害 : EffectType
{
    public struct 参数 { public long 点数; }
    public override void 应用(在 目标, in 参数 p) => 目标.Get<玩家属性>().血量基础 -= p.点数;   // 瞬时效果改基础账；当前账在第 9 相尾按拓扑序重算一次
}
// 击杀 = 跨零由引擎判、生死看基础账：结算按单序在基础账上进行，让血量基础从 > 0 变 ≤ 0 的那张单是击杀单，OnFx 带跨零标记、来源 = 下单时给的「来源」；基础账已 ≤ 0 的目标后续单 Rejected；当前账相尾重算一次
```

**④ 爆炸系统：帧初读整图、本帧全在数组上算、连锁同帧、伤害只下单、帧末一批交；死亡下一帧。**

```csharp
// Systems/爆炸系统.cs —— 只在服务器跑（.Server.cs 或共享文件里 #if LUMIO_SERVER）
[System(Phase.ProcessorPlan)]
public sealed partial class 爆炸系统 : System
{
    public override void Run()
    {
        var 到期的 = World.Each<炸弹状态>().Where(b => b.到期帧 <= World.Tick && !b.已爆).ToQueue();
        if (到期的.Count == 0) return;

        var 图 = 体素.批量读(全图矩形);            // ① 帧初照片：小地图 16 个 Section 一次拿回（pin 已就绪，永不「还没到」）
        var 批 = 体素.新写入批(图.Revision);        // ② 攒单；expectedSectionRevision = 帧初 revision
        while (到期的.TryDequeue(out var 弹))        // ③ 连锁：队列同帧算完
        {
            弹.已爆.Value = true;                    // 已有字段当场生效：后面的系统立刻看到
            foreach (var 方向 in 四个方向)
            {
                int 臂长 = 0;
                for (int 步 = 1; 步 <= 弹.火力; 步++)
                {
                    var 格 = 弹.格子 + 方向 * 步;
                    var 块 = 图[格];                                  // 读帧初照片，不读本帧改动
                    if (块 == 硬砖) break;
                    if (块 == 木箱) { if (!批.已含(格)) 批.写(格, 空气); break; }   // 去重靠批本身：同一格只下一次单
                    foreach (var 人 in 站在(格)) Effects.Apply<伤害>(人.Self, new 伤害.参数 { 点数 = 2 }, 来源: 弹.主人);   // 只下单；第 9 相结算，击杀 = 跨零由引擎判、已死的人后续单被拒
                    foreach (var 另一颗 in 炸弹在(格)) if (!另一颗.已爆) 到期的.Enqueue(另一颗);   // 连锁进队尾
                    臂长 = 步;
                }
                弹.臂长[方向].Value = 臂长;                             // 客户端据四臂长度直接画火焰，中途加入者也看得到
            }
            弹.爆炸态开始帧.Value = World.Tick;
        }
        体素.提交(批);                                // ④ 帧末一批交，VoxelCommit 相发布；1200 格连锁 = 重发 ≤16 条载荷，不分帧
    }
}

// Systems/死亡系统.cs —— 下一帧业务相：读到 血量基础 ≤ 0 才下结构单（生死看基础账；钩子里不下结构单是红线；20 Hz 下晚 50 ms 不可感知）
[System(Phase.ProcessorPlan)]
public sealed partial class 死亡系统 : System
{
    public override void Run()
    {
        foreach (var 人 in World.Each<玩家属性>().Where(p => p.血量基础 <= 0))
        {
            掉落系统.下单(人.Get<LogicTransform>().所在格);
            World.Commands.Destroy(人.Self);          // 销毁记录的 reason = terminated 由引擎盖；击杀事件已随上一帧的 OnFx 跨零标记发出
        }
    }
}
```

**⑤ 客户端怎么画**：表现层只读预测世界，按表现键做差（同键继续、键消失结束、新键开始）。火焰是 Local 实体，从炸弹实体的四臂长度长出来；击杀是 `OnFx` 带跨零标记的记录 + 下一帧的 `terminated` 销毁记录；预测世界只含实体，地形不预测——木箱消失靠体素 `Delta` 到达后重画。

## 5. 五组验收场景（引擎是否成立的主要证据；证据 = 结构化日志 + 抓包 + 两轮哈希）

| # | 场景 | 要证明的事 | 判据 |
|---|---|---|---|
| 1 | **同帧连锁爆炸** | 多颗炸弹、同一木箱、多次伤害按确定规则结算；不重复掉落、不重复奖励；伤害经 Effect 单 | A 引爆 B 在同一 tick；体素收到的批里该木箱只出现一次，掉落只生成一次；两道火 = 两张 -2 单，按单序在基础账上扣 6 → 2、当前账相尾重算一次 = 2，第三张单让基础跨零才记击杀且来源 = 那颗炸弹的主人，基础已 ≤ 0 目标的后续单 Rejected、无 `OnFx`；同帧多张单标脏同一属性重算计数 = 1；死亡销毁与掉落在下一帧结构提交；两轮日志逐位一致 |
| 2 | **视野与权限切换** | 未见过 / 暂不可见 / 已终结不混淆；新授权收旧值；撤权后本地正确失效 | 把视野收窄到半图：离开视野的实体销毁记录 `reason = left_aoi`，客户端答「未知」；死亡的 `terminated`，答「已终结」；把观察者加进 `Claim` 名单后下一包收到当前值，移出后收到失效；旁人收到血量当前账与 `OnFx`，收不到 Effect 明细与基础账 |
| 3 | **连续快速输入与延迟** | 多次放弹与多个产物正确对应本地预测，没有重复炸弹、错认、闪断；准入拒绝随重建自然到达 | 人为 150 ms 延迟下，A 手上两颗，在格 X 按放弹（17）、走到格 Y 再按（18）；17 到服务器前 B 已在 X 放了弹 → 服务器对 17 准入 ⑤ 拒、收 18；客户端预测时 X 还空、两颗都建；收到 `appliedInputSequence ≥ 18` 的包后重建，17 那颗消失、18 那颗与正式炸弹表现键相同且 fx 控制器对象同一个；改格用例（服务器把玩家按停在上一格）旧键结束新键开始各恰好一次；同一未确认输入重放十次，预测世界里的放弹起手表现触发计数 = 1（`OnFx` 本来只到一次）；客户端源码无认领 / 搬特效代码；预测世界无体素快照、本地临时号不出现在任何上行包 |
| 4 | **实体高频生灭与持续地形破坏** | 内存不随历史创建量无界增长；网格不因持续更新饿死；预测世界克隆不分配 | 一小时内生灭十万级炸弹 / 掉落，服务器常驻内存与活体数相关、与历史创建数无关（网络 ID → 句柄结构由实现仓选）；持续爆炸下客户端显示的 Section revision 单调递增且滞后不超预算；客户端每包重建走模板池，池热后堆分配为零 |
| 5 | **恢复与重新连接** | 旧 ID 不指向新实体；晚加入客户端拿到正确当前状态；对账哈希按四元组比 | 断线 4 分 59 秒重连拿回同一实体；5 分 01 秒重连拿到新实体且旧号服务器答墓碑、客户端答未知；中途加入者看到进行中的爆炸（四臂长度）与当前地形（首包全量 + 短票）；发号在预留落盘后杀进程，重启后新号全部大于已占段；客户端 AOI 半图时对账哈希与服务器对该观察者投影的哈希一致，人为改一个同步字段下一 tick 报漂移，改预测世界位置哈希不变 |

规模：目标人数（LumioGame 档位 8–100）+ 压力余量；没有实测前，不把「某模块测试很多」换算成「游戏已支持目标规模」。

## 6. 非目标

- 不定玩法数值、帽子经济、地图生成、拾取与减益——归 LumioGame。
- 不做存档 / 崩溃恢复验收（每局独立地图，无跨局存档；恢复场景只验发号与身份）。
- 不做 AOI 分帧、休眠 / LOD、多房间。
- GAS 只验切片面（ADR-064 第 1 条）：不做 Tag 表握手、帧调度器、堆叠 / 持续效果、挂起点 / 打断三积木验收、存档三档——各归自己的切片。
- 不为炸弹人往引擎加任何专属原语（爆炸传播、连锁、火焰）——全部由玩法用第 3 / 4 相的系统 + 技能 / 效果 + 体素批量读写拼出来。
- 浏览器预测不在本切片：引擎验收在 C# 客户端跑；浏览器路线等 LumioClient 调研卡 CL-1 的 WASM 结论再开 ADR。

## 7. 相关

- [`tick.md`](tick.md)、[`ecs.md`](ecs.md)、[`gas.md`](gas.md)、[`voxel.md`](voxel.md)、[`ds-server.md`](ds-server.md)；[`ecs-entity-chat.md`](ecs-entity-chat.md)（第一切片，身份与重连验收面沿用）。
- 决策：[ADR-064](../../decisions/ADR-064-gas-slice-contracts.md)（切片 GAS 面、预测世界形态、表现键）、[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md)、[ADR-062](../../decisions/ADR-062-voxel-world-public-contract.md)；LumioGame ADR 0015 / 0017 / 0018（炸弹实体与爆炸态模型）；体素需求答复 [`reviews/2026-09-04-bomber-voxel-asks-reply.md`](../../reviews/2026-09-04-bomber-voxel-asks-reply.md)；差异审计 [`reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md`](../../reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md)。
- 派活：体素 15 卡 `plans/2026-09-05-voxel-impl-dispatcher-prompt.md`；Runtime 五卡（RT-1 Tick 统一 / RT-2 双 Transform / RT-3 预测世界重建 / RT-4 GAS M2 / RT-5 GAS M3–M5）与 LumioClient 调研卡 CL-1 见 [`plans/2026-09-05-bomber-engine-runtime-cards.md`](../../plans/2026-09-05-bomber-engine-runtime-cards.md)。
