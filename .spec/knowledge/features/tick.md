---
name: tick
description: 一帧 13 相的活文档——每相能改什么、能看到什么、唯一提交点、帧内读写规则、游戏系统注册与 tick 频率归属;写系统或跨体素/ECS 提交前查
metadata:
  type: doc
  status: 设计中
---

# Lumio Tick 设计概要（一帧 13 相）

> 相名与顺序以 Runtime `modules/simulation/src/Lumio.GameRuntime.Simulation/Phases/TickPhase.cs` 的枚举为准（本文是它的活文档，冲突时改本文或开 ADR，不在实现里绕）。旧制度的 [ADR-002](../../decisions/ADR-002-tick-determinism.md) / [ADR-027](../../decisions/ADR-027-tick-fail-stop.md) / [ADR-030](../../decisions/ADR-030-processor-structural-commands.md) 的技术结论迁入本文；帧内读写规则与游戏系统注册方式由 [ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) 裁决。
> 配套：[ECS](ecs.md)（结构事务落在哪一相）、[GAS](gas.md)（重算与取样落在哪一相）、[体素](voxel.md)（`VoxelCommit`）、[DS](ds-server.md)（配额读取与打包相）、[存档](save-load.md)（取样切点）。

---

## 1. TLDR

**一帧走 13 步，只有第 10 步对外算数。** 业务只在第 3 / 4 步跑，只做三件事：改已有实体的字段（当场生效，后面的系统立刻看到）、给「生东西 / 删东西 / 加减组件」下单（第 9 步统一办）、给地形下单（第 8 步统一办）。第 10 步 `GasAndEventFinalize` 是唯一提交点：到这一步之前出任何错，整帧作废、从上一帧重来，没有「改了一半」；过了这一步，结果才对复制、快照、发送可见。

三条底线：

1. **一个提交点。** 多一个提交点，复制 / 快照 / 回放 / 预测四件事要各造一台机器。
2. **业务读地形只读帧初，改地形帧末一次提交。** 帧内不提供「读到本帧已下单改动」的读法；连锁这类「本帧已改了哪些格」由玩法在自己的帧内工作集里算。
3. **服务器永不回退。** 崩溃恢复靠快照 + 流水；客户端猜错靠「预测世界从确认世界重建」（[GAS M7](gas.md)），都不在这 13 步里做撤销。

---

## 2. 13 相表

`可写域` 与 `失败类` 沿用 Runtime 相契约表的名字；「能看到什么」一列是大白话。

| # | 相 | 干什么 | 可写域 | 业务代码能做什么 | 失败类 | 可取消 | 对后续相可见 |
|---|---|---|---|---|---|---|---|
| 1 | `IngressCapture` | 把网络线程塞进 inbox 的消息收进本帧 | IngressQueue | 不跑业务 | ProcessFault | 提交前 | 帧内私有 |
| 2 | `DecodeAndCanonicalize` | 解包、校验信封、按（发送者 NetEntityId, 序号）排死 | CanonicalCommandSet | 不跑业务 | BusinessReject（坏包只拒本条） | 提交前 | 帧内私有 |
| 3 | `ApplyInputs` | 执行 `[ServerRpc]` 方法体与 Owner 字段上行（`OnClientWrite` 校验） | InputApplySet | **改已有字段（当场生效）**；下结构单；下地形单；读地形帧初值 | BusinessReject | 提交前 | 帧内私有 |
| 4 | `ProcessorPlan` | 跑本帧注册的游戏系统（移动、AI、爆炸、GAS 应用单） | ProcessorPlan | 同第 3 相 | SessionFault | 提交前 | 帧内私有 |
| 5 | `CrossWorldPrepare` | 把本帧地形单合成一批，做 prepare（全部可失败校验在这） | PreparedGameDelta | 不跑业务 | BusinessReject（整批拒） | 提交前 | 帧内私有 |
| 6 | `NativeJobBarrier` | 收回 Native 作业结果（空间粗筛的候选进 / 出对清单等） | NativeCompletions | 不跑业务 | ProcessFault | 提交前 | 帧内私有 |
| 7 | `CommitDecision` | 决定本帧提交还是整帧作废 | CommitIntent | 不跑业务 | SessionFault | 提交前 | 帧内私有 |
| 8 | `VoxelCommit` | 地形批一次性不可失败发布（Section 载荷、revision、脏标记同批） | VoxelWorld | 不跑业务 | ProcessFault | **不可取消** | 帧内私有 |
| 9 | `EcsCommandBufferCommit` | 结构单一口气生效：亮相 → 全体 Awake → 全体 Start；GAS 账本重算写回 | GameWorld | 只有九回调（[ECS M3](ecs.md)），钩子里不下结构单 | ProcessFault | 不可取消 | 帧内私有 |
| 10 | `GasAndEventFinalize` | **唯一提交点**：状态取样、产出表现缓冲与 ClientRpc 事件 | GasEvents | 不跑业务 | ProcessFault | 不可取消 | **对外可见** |
| 11 | `ReplicationProjection` | 从脏账取变更集，按每个观察者的视野 × Scope × 书签打包 | ReplicationView | 不跑业务 | ProcessFault | 不可取消 | 对外可见 |
| 12 | `SnapshotHashMetrics` | 每帧轻量哈希（位置 + Attribute）、指标 | SnapshotHash | 不跑业务 | ProcessFault | 不可取消 | 对外可见 |
| 13 | `EgressPublish` | 把包交给连接层；outbox 里攒的不可回滚动作（发消息、写文件、播表现）在这之后才执行 | EgressQueue | 不跑业务 | ProcessFault | 不可取消 | 对外可见 |

冻结的几条：**恰好 13 相、序号连续**；**第 10 相是唯一提交点**；**第 8 相起不可取消**；同一帧同一份规范输入重跑一遍，结果逐字节相同（幂等）；超预算一律 fail-stop，不做「部分提交」。跨 World 次序固定 **`VoxelCommit` → `EcsCommandBufferCommit`**：体素是参与者，不是协调者。

---

## 3. 帧内读写规则（三层，谁看到什么）

| 改的是什么 | 什么时候生效 | 同一帧后面的系统看到什么 | 例子 |
|---|---|---|---|
| **已有实体的字段** | 当场（第 3 / 4 相写 `.Value` 即生效） | 新值 | 血量 100 → 70，后面的系统读到 70 |
| **生东西 / 删东西 / 加减组件** | 第 9 相统一办 | 看不到（下一帧才查得到） | 放一颗炸弹，本帧后面的系统查不到它 |
| **地形** | 第 8 相统一办 | 看不到（业务读地形永远是帧初值） | 炸掉一格木箱，本帧后面的系统读到的还是木箱 |

规则：

1. **业务读地形只读帧初。** 有炸弹要爆的那一帧，玩法先向体素批量读一次（小地图直接整图，[体素 M7a](voxel.md)），拿到「帧初照片」放进本地数组；本帧全部计算在这个数组上做。
2. **改地形只下单，帧末一次提交。** 各系统下的地形单在第 5 相合成一批，`expectedSectionRevision` = 帧初 revision；同一格多批写入按系统序生效、最后一条为准；第 8 相一次发布。
3. **连锁在自己的工作集里算，同一帧算完。** 炸弹人：把要爆的炸弹排成队 → 拿一颗出来，火焰往四个方向走，每走一格查本地数组（铁皮停、木箱下单后停、另一颗炸弹塞进队尾继续、玩家直接改血量）→ 队空了把攒下的单交给体素 → 帧末提交。**去重靠批次本身**（下单前看批里有没有这格），**击杀去重靠血量当场生效**（第二道火看到血已经是 0，不再算击杀）。「B 的火穿不穿 A 刚炸掉的木箱」是玩法规则：帧初照片里木箱还在，火默认被挡；想让它穿，玩法在批里查一下，一行代码。
4. **不提供第二条读路径。** 体素不做「本帧工作副本」读法；ECS 不做字段级撤销。这两条是红线。

---

## 4. 游戏系统怎么注册进 13 相

**现状（引擎缺口，[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md) 第 14 条 ①）**：Runtime 没有公开的 System / Processor 接口，`WorldManager.Tick()` 内部序列固定，游戏只能在 `Tick()` 前后手动调自己的函数（LumioGame ADR 0015 的实测结论）。这是 Runtime 待派的卡，不是游戏设计错。

**设计意图（Runtime 卡按此落地）**：

- 游戏在玩法程序集里声明系统类，标注它跑在哪一相：`[System(Phase.ProcessorPlan)] public sealed partial class 爆炸系统 : System { … }`。只允许标第 3 / 4 相（业务相）；其他相由引擎独占，标了生成报错。
- 注册表由生成器产出（与 EntityType、组件注册表同一次「生成三件」），世界只收生成的注册表；不手写注册、不反射发现。
- 同一相内多个系统的执行顺序 = 声明序 + 可选依赖声明，编译期算死、成环报错；系统只声明读写哪些组件，读写集互不重叠的系统才允许并行（实现仓自由度），语义顺序不变。
- 系统里能做的与第 3 / 4 相一样：改字段、下结构单、下地形单；不许读墙钟、不许自建随机（[ECS 三义务](ecs.md)）。
- 客户端同一套注册表、同一段代码：预测档的系统在客户端预测世界里跑，其余只在服务器跑（按 `.Server.cs` / 共享文件归属自然区分）。

---

## 5. tick 频率与时间

- **tick 频率是游戏配置，不是引擎常量。** 落在 `WorldEntity` 的 Tick 配置字段上（[ECS M1a ③](ecs.md)「Tick 配置是它组件上的字段」），随快照入档；一局之内不变。
- **框架里只有帧数没有秒。** GAS「策划配秒、管线换帧」的换算率就是这个字段；断线保留 5 分钟这类真实时间归宿主单调钟（[DS](ds-server.md) Timer），不进 13 相。
- **Tick 号 u64、单调递增**，与 revision 一起盖在每个包和快照上；客户端声称的 tick 只作参考，结算一律按服务器自己的 tick。

---

## 6. 失败与回滚

- **服务器**：第 10 相之前任何一步抛错、被取消、超预算 → 本帧作废，从上一帧快照 + 日志重建（fail-stop）；不做字段级 undo；没有「部分提交」。钩子里的不可回滚动作一律记 outbox，第 13 相之后才执行。
- **客户端**：猜错了不是回退，是「预测世界从确认世界整体重建 + 重放未确认输入」（[GAS M7](gas.md)）；预测世界只含被预测的域，体素不进预测世界。
- **崩溃恢复**：`CreateFromSnapshot` 建新世界，只跑 `OnHydrate`；发号从快照记录的已占段之后继续（[ECS M1 ⑤](ecs.md)）。

---

## 7. 明确不做

| 不做 | 级别 | 什么情况回头 |
|---|---|---|
| 第二个提交点 | **红线** | 永不 |
| 体素「本帧工作副本」读法 | **红线** | 永不——玩法在帧初照片上算 |
| 字段级 undo | **红线** | 永不——整帧作废 + 日志 / 预测世界重建 |
| 游戏专属相位 / 为某款游戏加原语 | **红线** | 永不——引擎归引擎，玩法用第 3 / 4 相拼 |
| 分帧提交一批地形单 | 不做 | 跨 tick 半提交状态比它省的贵；炸弹人 1200 格连锁 = 重发 ≤16 条载荷 |
| 帧内并行跑业务系统 | 推迟 | 实现仓自由度；语义顺序由声明序定，并行不得改变可观测结果 |

---

## 8. 相关

- [`ecs.md`](ecs.md) M3（四步落在哪一相）、M1a（WorldEntity 的 Tick 配置）、M10；[`gas.md`](gas.md) §2「一帧里 GAS 在哪几格干活」、M7；[`voxel.md`](voxel.md) M6 / M7a；[`ds-server.md`](ds-server.md) M4（配额在第 4 相前读、第 11 相消费）；[`save-load.md`](save-load.md) M2（取样在第 10 相之后）。
- 决策：[ADR-063](../../decisions/ADR-063-architecture-review-owner-rulings-identity-persist-prediction.md)；历史结论：ADR-002 / 027 / 030（Historical）。
