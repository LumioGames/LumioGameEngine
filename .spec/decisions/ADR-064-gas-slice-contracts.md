# ADR-064：炸弹人切片的 GAS 契约——技能八态准入、瞬时效果与两本账、预测键 = 输入序号、预测世界形态与表现连续性、表现缓冲走 ClientRpc 记录；取代 ADR-050 / 051

状态：Draft（2026-09-05，Owner 定炸弹人切片的 GAS 面；随 Runtime 卡 RT-4 / RT-5 落地、切片场景 1 / 3 跑通后转 Accepted）
取代：[ADR-050](ADR-050-gas-a1-contracts.md) / [ADR-051](ADR-051-gas-a2-contracts.md)（旧制度 Draft：其 `schemas/` / `fixtures/` / `tools/lumio_contract.py` 引用随制度废止已失效，「回退一帧含体素」与 ADR-063 第 7 条相抵；仍成立的技术结论在本 ADR 第 2 / 5 / 6 条逐条沿用，两份正文不改写）
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioGameRuntime`（GAS / ECS / 生成器唯一实现）、`LumioClient` / `LumioGame`（消费方）

## 治理原则

- 沿用 ADR-056：**第一性原理——如无必要，勿增实体。**
- 沿用 ADR-058：**AI Agent 友好**——同一件事只在一处维护，每件事只有一种写法。
- 沿用 ADR-060：**彻底清理，不留兼容。**
- 沿用 ADR-063：世界模型强约束（GAS 只能是实体上的组件）、玩法归玩法引擎归引擎、ROI 原则（大白话 + 编号步骤的游戏例子）。

## 背景

[`reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md`](../reviews/2026-09-05-dual-transform-bomber-research-gap-audit.md) 对照外部调研核出三条设计证明缺口：D01「预测世界」是独立 World 实例还是受限状态集没写清；D02 `fx_key + 参数` 在「校正改格」「重放」下怎么保持连续没证明；D03 切片到底验多大的 GAS 面没定。同时 Runtime `modules/gas` 现状只有句柄索引与生命周期状态机，`activate / apply_effect / tick_effects` 仍是「候选接口」，`modules/gas/README.md` 与 ADR-050 / 051 都还挂着旧制度的 Baseline / Schema 引用。

主会话 2026-09-05 向 Owner 提了三个问题，Owner 裁决：① 引擎验收里的「按键先动、服务器纠正」先在 C# 客户端（Bot.Host / 同进程双端）跑通，浏览器怎么跑预测**现在就调研 WASM**（LumioClient 调研卡 CL-1）；② **技能状态机进切片**（移动 / 放弹走完整八态准入）；③ **扣血也走 Effect**（瞬时效果 + 最小求值 + 属性两本账进切片）。②③ 与主会话「先最小面」的推荐相反，按 Owner 决定执行；代价是 GAS 先立契约（本 ADR）、Runtime 多两张卡。

一句话讲这张 ADR 在定什么：

1. 玩家按放弹，技能先过五道检查（有没有技能、冷却到没到、手上有没有炸弹、有没有被沉默、这格能不能放）才算放出去。
2. 放出去 = 在世界里建一颗炸弹实体；客户端在自己的「预测世界」里先建一颗，画面立刻有。
3. 引信到点，爆炸系统对每个被火打到的人下一张「伤害 -2」的效果单；效果单在提交相结算，改的是血量的「基础账」，「当前账」随之算出来。
4. 让血量从正数变成 0 的那张单就是击杀单，谁下的单谁算击杀；对已经 0 血的人再下的单直接拒。
5. 服务器的包到了，客户端把预测世界扔掉、从确认世界重新复制一份、把服务器还没处理的按键重放一遍；画面上的炸弹按「表现键」（炸弹 + 格子 + 主人）认，同键就不闪。

## 决策（Owner 裁决 2026-09-05；数据结构与流程由主会话按裁决填实）

1. **切片 GAS 面。** 进切片：M2 Ability 生命周期（八态 + 准入 5 步 + Commit 判定 + 执行时限）、M3 Effect 生命周期的**瞬时子集**（六态集不变，切片只走 Pending → Active → Expired 同帧）、M4 求值（冻结公式三操作符）、M5 Attribute 两本账、M7 预测（档位 + 预测世界重建）。不进切片：M6 Tag（准入第 ④ 步用空表通过，Tag 表握手另立卡）、M9 帧调度器（瞬时效果没有到期；炸弹引信是炸弹实体上的字段，由玩法系统扫描——引擎归引擎）、配表 TypeId（切片内 TypeId 用代码常量经生成注册；LumioConfig 接入另立卡）、存档三档（切片无存档）、挂起点与打断三积木（接口保留、切片不验收）。

2. **四组件字段声明（组件类是唯一真源，全部按 [`ecs.md`](../knowledge/features/ecs.md) M4 ① 的 `Sync` 写法；取代 ADR-051 的七对矩阵）。**
   - `AbilityComponent`：`SyncList<技能条目>(Scope.Owner)`——条目 = TypeId、状态、激活输入序号、激活帧、时限到期帧、应用快照字段组。终态即出表；瞬时技能同帧进出，按 ecs M4 ③「加了又删 = 抵消」折叠成零字节。
   - `EffectComponent`：`SyncList<效果条目>(Scope.Owner)`——条目 = TypeId、状态、来源 `NetEntityId`、应用快照、`fx_key`、剩余帧 / 层数。Effect 明细仅自己可见（沿用 ADR-051）。
   - `AttributeComponent`：属性由玩法在其 partial 里**一处声明**（`Attribute 血量 = new(初值: 6)`，依赖列表写在声明上），生成器展开成两本账两个字段：`[Persist] Sync<long> 血量基础 = new(Scope.Owner)`（存档；**只发给绑定者自己**——预测世界要在客户端跑同一段准入 ③ 与扣减，没有基础账就跑不了；无绑定者的实体（炸弹、怪）`Scope.Owner` 自然零字节，旁人永远收不到）+ `Sync<long> 血量当前 = new(Scope.Aoi)`（上网、不存档、永为推导值）。「修订号」= 包级 revision（C-1″ `WorldChange.tick`），**不另设每属性修订字段**。
   - `TagComponent`：`SyncDict<uint, ushort>(Scope.Aoi)` 计数容器；切片声明、不验收。
   - **表现缓冲不是组件字段**：它是 `EffectComponent` 上引擎生成的 `[ClientRpc(Scope.Aoi)] OnFx(fxKey, 参数)` 记录，在 `GasAndEventFinalize` 产出、走 C-1″ `WorldChange.rpcs`，**不加第五种记录**（事件 = ClientRpc 是 ecs M4 ⑦ 唯一写法；ADR-051 的 `presentationBuffer` 字段作废）。
   - 沿用：没有 `FxComponent`；Modifier 账本没有独立存在（效果条目的推导视图，永不上网）；句柄 = ECS 下标 + 世代号，终态即失效。

3. **技能激活只有一种写法。** 技能类型 = 玩法声明的类：`[AbilityType(TypeId, Prediction = 档位, 消耗 = nameof(属性))] public sealed partial class 放弹技能 : AbilityType { public struct 输入 { … } public override bool 可以激活吗(in 输入) { … } public override void 执行(in 输入) { … } }`；注册表随 EntityType / 组件同一次「生成三件」产出，不反射、不手写。激活入口 `Get<AbilityComponent>().Activate<放弹技能>(in 输入)`：客户端调用 = 上行（引擎生成的 `[ServerRpc]`，进 C-1″ `InputCommand.commands[]` 的生成 ServerRpc 种类，信封带本连接递增的 `sequence`）；服务器 `ApplyInputs` 相执行准入五步 → Activated → Executing（同帧连转）→ `执行(输入)` 跑内容代码（写 `LogicTransform`、下 Effect 单、下结构单）→ 内容层 `End()` 或同帧返回即 Completed。准入五步在切片的落点：① 句柄权限 = 调用者是该实体的绑定者且实体类型声明了该技能；② 冷却 = 条目上的下次可用帧（切片为 0）；③ 消耗 = `消耗 = nameof(…)` 指名属性的基础账扣减（放弹 = `手上炸弹数基础 -1`；基础账 `Scope.Owner`，所以客户端预测世界读的是同一本账），**Commit 判定复查后才真扣**；④ Tag = 空表通过；⑤ 内容层自定义 = `可以激活吗`（这格能不能放）。移动技能同一写法，档位「逻辑预测」，`执行` 写 `LogicTransform`；转角缓冲等工作状态是技能类型上的共享普通字段（ADR-063 第 5 条）。

4. **Effect 应用单。** 业务相 `Effects.Apply<伤害>(目标, in 参数, 来源)` 只下单（`来源` = 下单者实体号，进效果条目、进 `OnFx`）；`EcsCommandBufferCommit` 相尾按单序（系统序 + 下单序）**在结算中的基础账上**结算：校验（目标不存在 / 基础账已 ≤ 0 / 免疫 → Rejected）→ 入表 Active → **瞬时效果的 Modifier 直接改基础账**（`血量基础 -= 2`，下一张单读到的就是改后的值）→ 同帧 Expired 出表；持续效果的 Modifier 只进当前账（切片不验收）→ 全部单结算完，标脏属性按静态拓扑序重算当前账**一次**并写回（当前账不参与结算期判定）→ `GasAndEventFinalize` 产出 `OnFx`。**击杀 = 跨零，生死看基础账**：让 `血量基础` 从 > 0 变 ≤ 0 的那张单是击杀单，其来源即击杀者，`OnFx` 参数带跨零标记；对基础账已 ≤ 0 目标的后续伤害单一律 Rejected——同帧两道火只记一次击杀靠的是这个，取代 ADR-063 第 4 条「击杀去重靠血量当场生效」的措辞（那是伤害直接写字段时的机制，Owner ③ 之后不再成立）。**能被瞬时效果直接改的属性（血量这类）不得再挂持续修饰**——要护盾另开属性——所以它的基础 = 当前恒成立，客户端读当前账画血条即可。死亡态 = `血量基础 ≤ 0`（客户端等价读当前账），不另设字段；死亡后的结构动作（销毁、掉落、重生）由玩法的死亡系统在**下一帧**业务相读到 `≤ 0` 后下单——钩子里不下结构单是 ecs §6 红线，伤害本身不因此延后。

5. **求值（沿用 ADR-050 公式，数值改整数）。** `当前 = (基础 + Σ加法) × (1000 + Σ千分比) / 1000`，向零取整；覆盖按配表显式优先级、同级后写赢；加法项与千分比项按（应用序, 条目下标）定序求和。属性值与修饰量一律 `long`，百分比用千分比整数——不用浮点，也不用 ADR-050 的 34 位十进制（那是跨语言 JSON 对账方案，随旧制度作废；整数在两台机器上逐位相同且零依赖）。重算唯一时机 = `EcsCommandBufferCommit` 相尾，按编译期算死的拓扑序一次；公式声明成环 = 生成报错。切片验：血量 = 基础账（无修饰）；火力 / 移速 = 基础 + 加法（帽子的数值归 LumioGame）。

6. **同帧顺序与状态集（沿用 ADR-050）。** 八态 / 六态状态集不变；效果同帧事件序 命中 → 溢出 → 快照替换 / 层数 → 时长 → 周期 → 移除垫后；同帧应用又移除 = 抵消；抑制 = Active 内摘除事件，不加状态；Rejected 只属准入段、Commit 复查失败走 Cancelled，两者都不扣消耗。切片只走 命中 → 移除。

7. **预测键 = 输入序号。** 客户端每条 `InputCommand` 带本连接单调 +1 的 `sequence`（从 1 起）；服务器每处理一条（接受或拒绝）都把该观察者的 `appliedInputSequence` 推进到该号，随下一包 `WorldChange` 下发；序号不连续或回退 = 连接协议违约，拒绝该连接。[`gas.md`](../knowledge/features/gas.md) M7 ①「预测键 = 帧号」收敛为「预测键 = 输入序号」；ADR-051 的 `AbilityComponent.inputFrame` 客户端权威字段作废（预测键在信封上，不在组件上）。

8. **预测世界的具体形态（回答审计 D01）。**
   1) 客户端一个 `WorldManager` 持两个 `World`：**确认世界**（只被 `DecodePack` 写）+ **预测世界**（派生物）。这是 ecs §6「第二个世界」红线在客户端的唯一例外（ADR-063 第 7 条）；服务器仍只有一个世界。
   2) 每包权威状态提交进确认世界后立即重建：预测世界 = 确认世界的 ECS **整体克隆**（全部实体、组件的全部 `Sync` 字段与共享普通字段；体素不克隆，预测世界读体素时只读确认世界的体素）+ 按序重放 `sequence > appliedInputSequence` 的本地输入——每条输入在预测世界跑一遍 `ApplyInputs`（同一段 Activate / 准入 / 执行）→ 预测档系统（[`tick.md`](../knowledge/features/tick.md) §4）→ 本地结构提交（`EcsCommandBufferCommit` 语义，不做跨域）。克隆走 ADR-060 第 11 条的模板池（按类型克隆，池热后零分配）。
   3) 预测世界里新建的实体拿本地临时号（World 本地句柄），不上网、不进哈希、不存档，随下一次重建作废。
   4) 不可预测清单（Effect 移除 / 周期跳 / 出模拟域动作）在预测世界不执行；预测世界不产出 outbox。
   5) 表现层（`ModelTransform` 插帧、Local 实体、fx 控制器）**只读预测世界**；UI 读确认世界。
   6) 规模信号：整体克隆每包超预算 → 收窄克隆域到「Self + 被预测输入可触及的实体」，模型不变（gas.md §5 kill criteria 2）。

9. **表现连续性（回答审计 D02）。** 表现层不认实体句柄，认**表现键** = (EntityType, `fx_key`, 稳定业务参数)——炸弹的表现键 = (炸弹实体类型, 炸弹 `fx_key`, 格子, 主人 `NetEntityId`)。每次重建后表现层对「重建前后预测世界里的表现键集合」做差：同键 → 控制器继续（画面不变）；键消失 → 控制器结束；新键 → 控制器开始。三种情况：
   - **同格通过**：预测炸弹键 = (炸弹, fx, (4,5), 我) → 正式炸弹进确认世界 → 重建后预测世界里同键仍在 → 画面不变、零闪断、控制器对象同一个。
   - **改格**（服务器把我按停在 (3,5) 才放弹）：旧键消失、新键出现 → 画面上炸弹换格——这是**正确的纠正**不是缺陷；炸弹静止，换格直接换位不插帧。
   - **被拒**：旧键消失 → 炸弹消失。
   一次性**预测**表现（放弹起手音效等——逻辑预测 / 表现先行档技能在预测世界执行时挂的 Local 实体或本地 fx 触发；**不是** `OnFx`，预测世界不跑第 10 相、不产 outbox）按（输入序号, `fx_key`）去重——输入序号跨重建稳定，重放不重播；服务器的 `OnFx` 记录本来只到达一次，不在此列。不做认领键、不改号、不搬特效（ADR-063 第 15 条维持）。

10. **哈希对账四元组（回答审计 D17）。** 双端对账哈希只在四个条件同时成立时比较：同一 tick（包的 `tick`）、同一可见集（该观察者视野表投影出的实体集）、同一字段集（同步域 `Sync` 字段，排除 `Scope.None` / 预测 / 表现）、确认世界对**该观察者的服务器投影**（不是服务器全世界）。预测误差是另一个仪表——客户端在重建前记录预测世界的 `LogicTransform` 与确认世界同 tick 值之差，只做诊断、不进哈希。服务器全量快照哈希与对账哈希是两回事（gas.md M8 ④）。

11. **执行时限。** 技能类型声明默认时限（帧），超时框架置 Expired 并清场；切片技能同帧完成，验收只用一个故意不 `End` 的测试技能证明时限生效。

12. **不采纳。** 扣血直接写字段、Effect 不进切片（主会话推荐）；技能状态机不进切片（主会话推荐）；表现缓冲作为 `EffectComponent` 字段；34 位十进制求值；每属性修订号；死亡回调里下结构单；预测世界只克隆 Self；认领键 / 改号 / 搬特效；属性基础账 `Scope.None`（本 ADR 初稿）；跨零判定读当前账（本 ADR 初稿）。理由见「替代方案」。

## 替代方案

- **扣血直接写字段、Effect / Attribute 另有切片**（主会话推荐）：Owner 否——GAS 要在真实战斗里成立，只验状态机不验效果等于没验。
- **技能状态机不进切片，`移动技能` 就是带 `[ServerRpc]` 的组件 + 档位标注**（主会话推荐）：Owner 否——同上。
- **表现缓冲作为 `EffectComponent.presentationBuffer` 字段**（ADR-051）：否——事件不是字段（ecs M4 ⑦「字段 = 最后状态，事件 = 一次性通知」），ClientRpc 记录已经是事件的唯一写法，多一种载体多一份 codec。
- **34 位十进制 `ROUND_HALF_EVEN` 求值**（ADR-050）：否——那是 Rust / C# / Python 三方 JSON 对账方案；现在只有 C# 一份实现（阶段 2 才下沉 Rust），整数 + 千分比逐位相同且零依赖。
- **每属性修订号**：否——包级 revision 已经能回答「客户端拿到的是哪一版」，预测世界重建不需要按属性对号。
- **死亡回调里下结构单（销毁 / 掉落同帧）**：否——ecs §6 红线「钩子里下结构单 永不」；下一帧业务相下单，20 Hz 下晚 50 ms 不可感知。
- **预测世界只克隆 Self 与其可触及实体**：暂否——先整体克隆一处维护，超预算再按 kill criterion 收窄，模型不变。
- **认领键 / 改号 / 搬特效**：维持 ADR-063 第 15 条；表现键做差已覆盖三种情况。
- **预测键 = 帧号**（gas.md M7 ① 原措辞）：收敛为输入序号——客户端要知道的是「哪些按键还没被处理」，帧号答不了这个问题，输入序号答得了且 ADR-063 已选它上网线。
- **属性基础账 `Scope.None`**（本 ADR 初稿，reviewer P1-1）：否——逻辑预测要求客户端在预测世界跑同一段准入 ③ 与扣减，基础账不到客户端就跑不了（要么永远预测不出放弹，要么每次重建都重扣）；改 `Scope.Owner` 只多给绑定者自己几个字节，旁人仍收不到，记账仍一本。
- **跨零 / 已死判定读当前账**（本 ADR 初稿，reviewer P1-2）：否——当前账只在相尾重算一次，同帧多单会读到旧值，四张 -2 单打 6 点血会全部通过；结算期判定改读结算中的基础账，并禁止血量这类属性挂持续修饰，基础 = 当前恒成立。

## 接口 / Schema

- **C-1″ `engine/wire/gameplay-command-envelope-v1.json`**（随 R5-01 一并落，本 ADR 追加语义）：`InputCommand.sequence`（u64，本连接单调 +1，从 1 起）；`commands[].mappingId` 的「生成 ServerRpc 种类」覆盖 `AbilityComponent.Activate`（不加新消息）；`WorldChange.appliedInputSequence` = 该观察者已处理（接受或拒绝）到的最大序号；`WorldChange.rpcs` 记录承载 `EffectComponent.OnFx(fxKey: u32, 参数: LumioBinV1 按 fx 声明)`。正反用例：序号不连续 → 拒绝；`appliedInputSequence` 在拒绝后照样推进；`OnFx` 记录 `componentId` = `EffectComponent`。
- **生成器**：`[AbilityType(TypeId, Prediction = 纯权威 | 表现先行 | 逻辑预测, 消耗 = nameof(属性))]`（`消耗` 可省 = 无消耗）、`[EffectType(TypeId, 瞬时 | 持续)]`、`Attribute` 声明展开为两本账两个 `Sync` 字段、`AbilityComponent.Activate<T>` / `Effects.Apply<T>` 桩、拓扑序表；GAS 类型注册表并入「生成三件」；技能标非业务相 / 公式成环 / `[Persist]` 打在当前账 / 出现 `FxComponent` → 生成报错。
- **Runtime 公开 API 形状**（签名由 RT-4 / RT-5 卡填实）：`AbilityComponent.Activate<T>(in T.输入)`；`AbilityType.可以激活吗 / 执行 / End`；`Effects.Apply<T>(NetEntityId 目标, in 参数, NetEntityId 来源)`；生成的 `X基础`（`Scope.Owner`）/ `X当前`（`Scope.Aoi`）；`EffectComponent.OnFx`；准入失败结果带步序号。
- **预测世界**（客户端 Runtime 模块，RT-3）：`WorldManager.ConfirmedWorld` / `WorldManager.PredictedWorld`；重建触发 = 每次 `DecodePack` 提交后；`sequence` 由 R5-02 的 codec 在 `EncodeInput` 按连接分配（+1 从 1 起，两端同一程序集，客户端不自行分配），RT-3 只加输入历史；表现层订阅「重建后表现键差集」。

## 失败语义

- 准入任一步失败 → Rejected 并带步序号；Commit 复查失败 → Cancelled；两者都不扣消耗、不进 Executing。
- 技能类型未注册、技能标非业务相、`sequence` 不连续或回退 → 拒绝（序号问题 = 断开该连接）。
- Effect 校验失败（目标不存在 / 基础账已 ≤ 0 / 免疫）→ Rejected，单不生效、不产 `OnFx`。
- 生成期：公式声明成环、`[Persist]` 打在 `X当前`、声明 `FxComponent`、每属性修订字段、表现缓冲写成组件字段 → 生成失败。
- 收口审查退回：客户端源码出现认领 / 搬特效；预测世界出现体素快照或上行了本地临时号；死亡回调里下结构单；预测世界执行了不可预测清单里的动作。

## 兼容影响

- ADR-050 / ADR-051：状态改 Superseded（由本 ADR 取代），正文不改写。
- ADR-063（Draft）：第 4 条击杀去重口径、第 7 条预测键与预测世界形态、第 13 条切片 GAS 面、第 14 条追加引擎缺口 ⑥ 双 Transform 落地 ⑦ 预测世界重建 ⑧ GAS M2–M5——追加「修订记录（2026-09-05，ADR-064）」段。
- `knowledge/features/gas.md`（§2 组件、§3.1 图、§3.4、M2 / M3 / M4 / M5 / M7 / M8 / M10、§5、黑话表）、`knowledge/features/ecs.md`（M4 ⑧、M10 ①②④、§6 一行）、`knowledge/features/tick.md`（§1 TLDR、§2 第 3 相一栏、§3 表与规则 3、§8）、`knowledge/features/bomber-slice.md`（§1–§7）已按本 ADR 改写。
- `plans/2026-09-04-rm-00011-r5-cards.md`：R5-01 范围追加 `InputCommand.sequence` 与 `appliedInputSequence` 语义用例；R5-02 追加 ADR-063 第 5 / 6 / 14② 生成器项；R5-03 追加子仓 `.spec` 入口文档同步。
- 新计划 `plans/2026-09-05-bomber-engine-runtime-cards.md`：RT-1 ~ RT-5 与 CL-1。

## 迁移方案

第一阶段（本 ADR 同批）：本 ADR + 上列文档改写 + r5 卡范围追加 + Runtime / Client 卡计划。第二阶段：R5-01 落 `sequence` / `appliedInputSequence` / `OnFx` 记录用例；R5-02 落生成器基础（共享普通字段 lint、`Scope.None`、`Sync<NetEntityId>`）。第三阶段（R5-02 合入后）：RT-1（Tick 统一）→ RT-4（M2）→ RT-5（M3 / M4 / M5）与 RT-3（预测世界重建）；CL-1 与 wave 1 并行。验收 = `bomber-slice.md` §5 场景 1 / 3 在 C# 客户端跑通。

## 验证 Fixture

1. **准入五步**：手上炸弹数为 0 时放弹 → Rejected，步序号 = 3；这格已有炸弹 → 步序号 = 5；两者都不扣炸弹数。
2. **Commit 判定**：准入通过后、Commit 前基础账被另一张单扣到 0 → Cancelled 且不扣。
3. **瞬时 Effect 与击杀跨零**：两颗炸弹同帧对同一玩家各下一张 -2 单，按单序在结算中的基础账上扣 6 → 2，两条 `OnFx` 无击杀标记；第三张单让基础账跨零，`OnFx` 带击杀标记且来源 = 那颗炸弹的主人；第四张对基础账已 ≤ 0 目标 → Rejected、无 `OnFx`；四张单同帧，当前账相尾重算计数仍 = 1、结果 = 基础账。
4. **求值**：基础 100、加法 +20、千分比 +100 → 当前 132；两台机器逐位相同；公式声明成环 → 生成失败。
5. **重算一次**：同帧三张单标脏同一属性，重算计数 = 1。
6. **预测键与表现键**：150 ms 延迟下，A 手上两颗，在格 X 按放弹（序号 17）、走到格 Y 再按（18）；17 到服务器前 B 已在 X 放了弹 → 服务器对 17 准入 ⑤ 拒、收 18；客户端预测时 X 还空，两颗预测炸弹都建了；收到 `appliedInputSequence ≥ 18` 的包后重建：17 那颗消失、18 那颗与正式炸弹表现键相同、fx 控制器对象同一个（「手上只有一颗」构造不出这个结果——17 先到先过、被拒的会是 18）；改格用例：服务器把玩家按停在上一格 → 旧键结束、新键开始各恰好一次。
7. **一次性预测表现去重**：同一未确认输入重放十次，预测世界里挂的放弹起手表现（本地 fx 触发）计数 = 1；服务器 `OnFx` 到达一次。
8. **预测世界边界**：预测世界无 Section 快照；改预测世界的属性，确认世界不变；预测世界建的炸弹本地号不出现在任何上行包与哈希。
9. **执行时限**：故意不 `End` 的测试技能在时限帧后 Expired，句柄失效。
10. **对账四元组**：客户端 AOI 半图时对账哈希与服务器对该观察者投影的哈希一致；人为改客户端一个同步字段 → 下一 tick 报漂移；改预测世界位置 → 对账哈希不变。
11. **生成器负例**：`[Persist]` 打在 `X当前` / 声明 `FxComponent` / 技能标非业务相 / 每属性修订字段 → 各自生成失败。

## 修订记录（2026-09-05，Owner 暂定；本段为附录，不改写上文）

盘点 LumioNativeCore 时核对 CL-1 WASM 调研（LumioClient PR #18）的事实：浏览器客户端用 .NET browser-wasm 跑 Runtime，**不装 Native 库**。由此 Owner 暂定：

1. **进预测世界的东西必须能在浏览器里跑**；今天满足这一条的只有 Runtime C#。因此 gas.md **M9「到期与唤醒」归 Runtime 域**（到期帧是条目上的权威字段，排期索引是派生缓存不进快照，每帧一次批量取件，计帧不计秒），改 2026-08-30 GAS 裁决板 11a「帧调度器落 NativeCore」。预测世界里的到期不是定时器，是「到期帧 ≤ 当前帧」的字段比较，服务器与浏览器跑同一份 C#。
2. **Native 定时内核（`lumio-timer`，ADR-056 §7）只管宿主节拍**：服务器推帧、Bot 节奏、断线保留窗；不进预测世界。仍是全项目唯一的定时内核。
3. **否决「C# 一套 + Rust 一套、宿主自选、Rust 不可用回退 C#」**：两套实现同一份确定性逻辑只在写出来那天一致，且违反如无必要勿增实体、不留兼容层、ADR-056 单一内核与「同输入同输出」。
4. **「参考 + 优化」双实现**只允许用于有性能需求的重计算（视野粗筛、编解码），用哪个在编译期定死、以逐字节差分测试守护；重计算内核将来进网页走「同一份 Rust 编成 WASM」，仍是一套。
5. Rust 下沉（求值 / Tag / 堆叠）仍按本 ADR 上文为阶段 2；RM-00002 的 R-00302 / R-00308 / R-00309 因前提失效作废，阶段 2 到来时按本修订重开。
