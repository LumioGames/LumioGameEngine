---
name: bomber-slice
description: 炸弹人战斗切片的引擎验收需求真值——世界模型套用、引擎能力组合、第二样板与五组验收场景;排引擎卡或改引擎验收标准前查
metadata:
  type: doc
  status: 设计中
---

# 炸弹人战斗切片 · 引擎验收需求真值

> 引擎的验收主线从聊天切片（[`ecs-entity-chat.md`](ecs-entity-chat.md)）改为战斗切片（[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) 第 13 条）：一局炸弹人就能撞出帧内结算、身份、预测三组根问题，聊天撞不出来。
> **分工**：玩法规则、数值、帽子经济、地图生成归 `LumioGame`（其 `docs/specs/bomber/design.md` 与 ADR 0014–0018）；本文只定**引擎要证明的能力**与**验收场景**。文中数字（61×61、2.1 秒引信 = LumioGame kernel contract `fuseMs = 2100`、火力 2、3.5 格/秒、6 个半心）取自 LumioGame 现行口径，只作例子，不是引擎契约。

---

## 1. 目标

```text
玩家按键立刻动（预测）→ 放弹（预测建实体）→ 引信到点连锁爆炸（同帧帧内结算）
  → 木箱变空气（体素批量写）→ 伤害 / 死亡 / 掉落（字段当场生效 + 结构单）
  → 所有客户端同帧看到（复制）→ 死者重新入场（身份三态）→ 两轮跑出同一份哈希（一致性）
```

跑通这一条 = ECS / GAS / 体素 / Tick / DS 五份概要的语义在一款真实游戏里成立。

## 2. 世界模型套用（[`rules/system.md`](../../rules/system.md) 四问）

| 东西 | 要不要服务器逻辑 | 动不动 | 判定 | 落点 |
|---|---|---|---|---|
| 地面、硬砖、软砖（木箱）、水 | 不要（它们本身没逻辑；挡路 / 透光由材质类表定） | 不动 | **体素** | 官方全局段 8 种 Solid + 水 Liquid（[`voxel.md`](voxel.md) M1a；LumioGame 答复 §①） |
| 玩家 | 要 | 动 | **CS 实体** | `PlayerEntity`：`LogicTransform` + 血量 / 火力 / 炸弹数（LumioGame `BomberPlayerState`）+ 移动 / 放弹两个 Ability |
| 炸弹 | 要（引信、连锁、挡路） | 引信到点变火焰 | **CS 实体** | `炸弹`：格子、主人、到期帧、火力、四臂到达长度；引信 → 爆炸态 → 留火 → 销毁（LumioGame ADR 0017） |
| 帽子堆、掉落物 | 要（能被捡） | 不动但有生命周期 | **CS 实体** | 结构单生成 / 销毁 |
| 火焰画面、爆炸特效、客户端预测用的炸弹 | 不要 | — | **Local 实体** | 由炸弹实体的四臂长度 / `fx_key` 长出来；预测世界里的预测炸弹随重建出现或消失 |
| 移动、放弹 | — | — | **GAS Ability**（实体上的组件） | 两端跑同一段代码；预测归 [`gas.md`](gas.md) M7 |

没有第三种东西。「炸弹做成地图上的列表记录」「爆炸格每格一个实体」都被否（前者违反一切皆实体，后者 100 人下每秒百级实体生灭——LumioGame ADR 0017）。

## 3. 引擎能力组合（本切片要求引擎证明的，按依赖顺序）

| # | 能力 | 引擎落点 | 现状 / 缺口 |
|---|---|---|---|
| 1 | 游戏系统注册进 Tick 的第 3 / 4 相 | [`tick.md`](tick.md) §4 | **缺口**：Runtime 无公开接口，派 Runtime 卡（ADR-063 第 14 条 ①） |
| 2 | 移动预测：同一段 Ability 代码两端跑，转角缓冲等工作状态放共享文件 | [`gas.md`](gas.md) M7、[`ecs.md`](ecs.md) M4 ①（共享文件普通字段 + lint）、M7（`LogicTransform` / `ModelTransform`） | lint 口径已定（ADR-063 第 5 条），Runtime 生成器随 R5-02 落 |
| 3 | 放弹预测：客户端预测世界里 `Commands.Create<炸弹>()`，包级 `appliedInputSequence` 驱动重建 | [`gas.md`](gas.md) M7 ①②、[`ecs.md`](ecs.md) M10 | C-1″ 字段随 R5-01；预测世界重建归 Runtime 客户端模块 |
| 4 | 帧初批量读整图、帧末一批写、同帧多批合一 | [`tick.md`](tick.md) §3、[`voxel.md`](voxel.md) M6 ①c / M7a / M8 ③a（pin） | 体素卡 I-4 / I-5 / I-6（`plans/2026-09-05-voxel-impl-dispatcher-prompt.md`） |
| 5 | 改动层按 Section 派发，`Delta` 增量 | [`voxel.md`](voxel.md) M5 | 体素卡 I-3 / I-10 |
| 6 | 伤害当场生效、死亡销毁记录带 `terminated`、击杀走 `[ClientRpc]` 事件 | [`ecs.md`](ecs.md) M4 ⑦、M5 ③、M6 | 销毁记录 `reason` 随 R5-01 |
| 7 | 重新入场：5 分钟内 rebind 同一实体，超时新实体、旧号答墓碑（服务器）/ 未知（客户端） | [`ecs.md`](ecs.md) M10 ③、M5 ③ | 已在 RM-00011 验收面 |
| 8 | `Sync<NetEntityId>`（炸弹的主人、击杀者） | [`ecs.md`](ecs.md) M4 ② | **缺口**：派 Runtime 卡（ADR-063 第 14 条 ②） |
| 9 | 每帧轻量哈希对账 | [`ecs.md`](ecs.md) M10 ④、[`ds-server.md`](ds-server.md) M11 | 已在 RM-00011 验收面 |
| 10 | tick 频率作为 `WorldEntity` 配置 | [`tick.md`](tick.md) §5 | 字段名随 Runtime 卡定 |

**不在本切片**：GAS 的 Ability 八态 / Effect 六态全集、Attribute 拓扑重算、存档与崩溃恢复、AOI 分帧与休眠、多房间。它们各有自己的切片，不阻塞这条主线。

## 4. 第二样板（以后所有战斗类 ECS / GAS 代码与讨论以此为标准；与 §4.5 聊天样板并列）

**① 移动是一个 Ability，代码两端跑；工作状态放共享文件。**

```csharp
// Abilities/移动/移动技能.cs —— 共享文件，两端都编：服务器算权威位置，客户端在预测世界里先算一遍
[EcsComponent]
public sealed partial class 移动技能 : Component
{
    public int 转角缓冲剩余帧;                 // 普通字段：两端各算各的，不上网、不存档；lint 允许（两端都赋值）

    [ServerRpc] public partial void 按输入移动(输入 输入);   // 客户端调用即上行；服务器 ApplyInputs 相执行；客户端预测档同帧在预测世界里本地执行同一体

    public partial void 按输入移动(输入 输入)   // 方法体只写一份
    {
        var 位置 = Get<LogicTransform>();
        if (输入.按了转弯) 转角缓冲剩余帧 = 6;                     // 提前按了转弯，记住 6 帧
        if (到了路口(位置) && 转角缓冲剩余帧 > 0) 转向(位置, 输入);   // 到路口自动转，手感靠这个
        位置.逻辑位置.Value = 前进一格的几分之一(位置, 输入);        // 写 Sync 字段 = 记账；服务器权威、客户端预测
        转角缓冲剩余帧--;
    }
}
// 服务器只有 LogicTransform；客户端另有 ModelTransform（.Client.cs），每个渲染帧往逻辑位置滑，永不写回
```

**② 放弹：预测世界里建实体，通过不管、没通过随重建消失。**

```csharp
// Abilities/放弹/放弹技能.cs —— 共享文件
[EcsComponent]
public sealed partial class 放弹技能 : Component
{
    [ServerRpc] public partial void 放弹();
    public partial void 放弹()
    {
        var 我 = Get<玩家状态>();
        if (我.手上炸弹数 <= 0 || 这格已有炸弹(我.所在格)) return;   // 客户端预测时同样的拒绝条件
        var 单 = World.Commands.Create<炸弹>();                       // 服务器：提交相发号；客户端预测世界：本地建预测炸弹
        单.Get<炸弹状态>().格子.Value = 我.所在格;
        单.Get<炸弹状态>().主人.Value = Self;                          // Sync<NetEntityId>
        单.Get<炸弹状态>().到期帧.Value = World.Tick + 毫秒换帧(2100);   // 换算率 = WorldEntity 的 tick 频率；2100 取自 LumioGame fuseMs
        单.Get<炸弹状态>().火力.Value = 我.火力;
        我.手上炸弹数.Value--;
    }
}
// 客户端：按键 → 预测世界里跑上面这段 → 画面上炸弹立刻出现。服务器包到达（带 appliedInputSequence）→ 预测世界从确认世界重建：
// 通过 = 重建后那个位置站着的就是正式炸弹（同一颗，画面不变）；被拒 = 重建后没有它，画面消失。没有对号、没有换特效。
```

**③ 爆炸系统：帧初读整图、本帧全在数组上算、连锁同帧、帧末一批交。**

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
            弹.已爆.Value = true;
            foreach (var 方向 in 四个方向)
            {
                int 臂长 = 0;
                for (int 步 = 1; 步 <= 弹.火力; 步++)
                {
                    var 格 = 弹.格子 + 方向 * 步;
                    var 块 = 图[格];                                  // 读帧初照片，不读本帧改动
                    if (块 == 硬砖) break;
                    if (块 == 木箱) { if (!批.已含(格)) 批.写(格, 空气); break; }   // 去重靠批本身：同一格只下一次单
                    foreach (var 人 in 站在(格)) 扣血(人, 弹);        // 血量当场生效：第二道火看到血已是 0，不再算击杀
                    foreach (var 另一颗 in 炸弹在(格)) if (!另一颗.已爆) 到期的.Enqueue(另一颗);   // 连锁进队尾
                    臂长 = 步;
                }
                弹.臂长[方向].Value = 臂长;                             // 客户端据四臂长度直接画火焰，中途加入者也看得到
            }
            弹.爆炸态开始帧.Value = World.Tick;
        }
        体素.提交(批);                                // ④ 帧末一批交，VoxelCommit 相发布；1200 格连锁 = 重发 ≤16 条载荷，不分帧
    }

    void 扣血(玩家状态 人, 炸弹状态 弹)
    {
        if (人.血量 <= 0) return;                     // 已死，不重复击杀
        人.血量.Value -= 2;                           // 一颗炸弹 = 一颗心 = 2 个半心点（数值归 LumioGame）
        if (人.血量 <= 0) { 击杀事件(弹.主人, 人.Self); World.Commands.Destroy(人.Self); 掉落系统.下单(人.所在格); }   // 销毁记录的 reason 由引擎盖：玩法下的销毁单 = terminated，出视野 = left_aoi
    }
}
```

**④ 客户端怎么画**：火焰是 Local 实体，从炸弹实体的四臂长度长出来；死亡是 `terminated` 销毁记录 + 击杀 `[ClientRpc]` 事件；预测世界只含实体，地形不预测——木箱消失靠体素 `Delta` 到达后重画。

## 5. 五组验收场景（引擎是否成立的主要证据；证据 = 结构化日志 + 抓包 + 两轮哈希）

| # | 场景 | 要证明的事 | 判据 |
|---|---|---|---|
| 1 | **同帧连锁爆炸** | 多颗炸弹、同一木箱、多次伤害按确定规则结算；不重复掉落、不重复奖励 | A 引爆 B 在同一 tick；体素收到的批里该木箱只出现一次，掉落只生成一次；被两道火打的玩家只有一次击杀记录；两轮日志逐位一致 |
| 2 | **视野与权限切换** | 未见过 / 暂不可见 / 已终结不混淆；新授权收旧值；撤权后本地正确失效 | 把视野收窄到半图：离开视野的实体销毁记录 `reason = left_aoi`，客户端答「未知」；死亡的 `terminated`，答「已终结」；把观察者加进 `Claim` 名单后下一包收到当前值，移出后收到失效 |
| 3 | **连续快速输入与延迟** | 多次放弹与多个产物正确对应本地预测，没有重复炸弹、错认、闪断 | 人为 150 ms 延迟下连按两下放弹：预测炸弹与正式炸弹无重复、无闪断；服务器拒掉其中一颗时它随 `appliedInputSequence` 到达后的重建消失；客户端源码无认领 / 搬特效代码 |
| 4 | **实体高频生灭与持续地形破坏** | 内存不随历史创建量无界增长；网格不因持续更新饿死 | 一小时内生灭十万级炸弹 / 掉落，服务器常驻内存与活体数相关、与历史创建数无关（网络 ID → 句柄结构由实现仓选）；持续爆炸下客户端显示的 Section revision 单调递增且滞后不超预算 |
| 5 | **恢复与重新连接** | 旧 ID 不指向新实体；晚加入客户端拿到正确当前状态 | 断线 4 分 59 秒重连拿回同一实体；5 分 01 秒重连拿到新实体且旧号服务器答墓碑、客户端答未知；中途加入者看到进行中的爆炸（四臂长度）与当前地形（首包全量 + 短票）；发号在预留落盘后杀进程，重启后新号全部大于已占段 |

规模：目标人数（LumioGame 档位 8–100）+ 压力余量；没有实测前，不把「某模块测试很多」换算成「游戏已支持目标规模」。

## 6. 非目标

- 不定玩法数值、帽子经济、地图生成、拾取与减益——归 LumioGame。
- 不做存档 / 崩溃恢复验收（每局独立地图，无跨局存档；恢复场景只验发号与身份）。
- 不做 AOI 分帧、休眠 / LOD、多房间、GAS 完整状态机。
- 不为炸弹人往引擎加任何专属原语（爆炸传播、连锁、火焰）——全部由玩法用第 3 / 4 相的系统 + 体素批量读写拼出来。

## 7. 相关

- [`tick.md`](tick.md)、[`ecs.md`](ecs.md)、[`gas.md`](gas.md)、[`voxel.md`](voxel.md)、[`ds-server.md`](ds-server.md)；[`ecs-entity-chat.md`](ecs-entity-chat.md)（第一切片，身份与重连验收面沿用）。
- 决策：[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md)、[ADR-062](../../decisions/ADR-062-voxel-world-public-contract.md)；LumioGame ADR 0015 / 0017 / 0018（炸弹实体与爆炸态模型）；体素需求答复 [`reviews/2026-09-04-bomber-voxel-asks-reply.md`](../../reviews/2026-09-04-bomber-voxel-asks-reply.md)。
- 派活：体素 15 卡 `plans/2026-09-05-voxel-impl-dispatcher-prompt.md`；Runtime 缺口两张卡（系统注册进 Tick、`Sync<NetEntityId>`）待立。
