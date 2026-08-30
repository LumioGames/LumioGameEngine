> **来源说明（非本仓调研产出）**：本文件是 Owner 提供的历史项目参考资料，原件来自本机 `~/Downloads/GAS迭代概要设计.md`，内容出自 Owner 此前参与的另一个项目（飞书/Lark 文档导出，文内图片与部分链接指向该项目内部飞书空间，本会话不可达，仅保留原文供人工参考）。
>
> **用途**：[`2026-08-30-gas-architecture-battle-prompt.md`](2026-08-30-gas-architecture-battle-prompt.md) 的输入之一——上一个项目在 UE GAS 之上做过一轮实战迭代（GAS 2.0）后沉淀下来的概要设计，记录了真实上线后的痛点与对应决策，不是纸上谈兵的调研。Owner 明确表示整体倾向这份设计。**它跑在 UE 之上**（有 UObject、非确定性 Tick、非 ECS 权威存储、非固定步长），今天的定稿会要判断它的每条决策是「与引擎无关的实战智慧」还是「假设了 UE 地基才成立的选择」——不因为是 Owner 偏好就自动通过，也不因为它不是外部调研就被轻视。
>
> **正文原样保留**（含原文的删除线/高亮标注，代表原作者自己的迭代决策痕迹），仅补了这个说明头。

---

> TLDR: GAS 的迭代相关内容，包括现状、问题和迭代方案。迭代方案主要包括对 Effect、Ability、Ability Task、Cue、预测等方面的改进和优化。
>

# 现状
[GAS2.0 概要设计]（原文飞书链接，本会话不可达）

<!-- 原文此处两张架构图，本会话未取得图片，仅保留下方文字描述 -->

+ **AbilityComponent** 作为技能承载的容器，管理技能。提供接口：**添加删除技能、释放技能、添加删除BUff**
+ **AttributeComponent** 负责属性存储、修改和同步，包括HP、MP、攻击力等等
+ **TagComponent** 提供Tag标记功能，为其他系统提供Tag添加查询等基础功能
    - 比如实现AB技能互斥功能：设置B技能释放条件为没有TagA，然后在A技能释放时为对象添加TagA即可实现
+ **FxComponent** 负责表现相关功能。双端都会挂载，并且通过**FxParams**来管理和同步表现需要的数据。
    - 服务端只有数据部分，通过数据增删管理生命周期
    - 客户端则根据**fx_key**来创建对应的**FxController**，然后根据FxParams来调用底层提供的特效、音效等模块，来实现逻辑和表现分离

<!-- 原文此处一张技能释放方式示意图，本会话未取得图片，仅保留下方文字描述 -->

框架将提供三种技能释放方式：

1. C/S 模式。该模式不做任何预测先行，完全由服务端单边运行
2. 客户端预先表现技能释放。该模式只在释放技能时触发主端先行表现。技能流程内的表现不做预测
3. 全预测。主端运行整个技能流程，包括cAbility和sAbility，同时服务端也运行sAbiltiy。该模式对代码有要求，调用的全部接口都需要支持防重入(Redo问题)。

# 问题
（原文附「GAS痛点反馈」飞书文档链接，本会话不可达）

## 没有组合
不同的效果不能组合，Effect没有人用。各个系统实际上很多是拿着 Effect 级别的操作（播放动画，特效）当成 Ability 用。原因有：

1. 需要同时发送很多<u>Cue</u>（RPC）；可以考虑将其合成一个RPC, 减少RPC数量
2. 配置没支持Effect

## 断线重连没有支持
支持，应该是bug

## 预表现不好用
重构一下使用的接口，start ability要先add

## 与表现相关的部分没有做缓存机制
导致模型加载前的ability无法正常表现。但逻辑已经跑过去。

在Cue层支持（决定哪个支持）

## 持续性的ability比较少，不熟悉使用方法
新增AbilityTask，并提供一下常用Task：

+ 延迟的Task
+ 播放动画的Task
+ 响应`Attribute`变化的Task
+ 响应`Effect`变化的Task
+ 响应玩家输入的Task

## 文档不全
持续更新

## 没有阻塞性机制
例如 A_ablility 被B_alility 打断，B播放之后再回退到A继续

# 迭代方案
参考：https://github.com/tranek/GASDocumentation

https://dev.epicgames.com/documentation/en-us/unreal-engine/gameplay-ability-system-for-unreal-engine

**概念不变的部分**

AbilityComponent

Tag

Attribute

Targeting

**主要迭代**

Ability

Effect

Fx改成Cue

新增Task

移除Buff(下沉到Effect)

预测

## **Effect & Cue**
Effect是逻辑封装最基础的节点，主要是修改Attribute，赋予Tag

`Cue`是执行非游戏逻辑相关的表现层功能, 像动画,音效, 粒子效果, 镜头抖动等等

**这里把两者合并到一个概念，统一用Effect**

允许在`Ability`之外应用`Effect` P1

### 类型
做成基类

逻辑类：Effect只同步其所属客户端，Tag和Attribute等同步到所有客户端

| 类型 | **Cue执行** | **修改时机** |
| --- | --- | --- |
| 即刻(Instant) | Execute | 对Attribute立即进行的永久性修改. |
| 持续(Duration) | Add & Remove | 对Attribute中CurrentValue的临时修改和赋予持续的Tag. 持续时间读表 |
| 永久(Infinite) | Add & Remove | 对Attribute中CurrentValue的临时修改和赋予持续的Tag. 该类型自身永不过期且必须由某个Ability或手动移除. |


表现类：EffectCue同步到所有客户端

| 类型 | 事件 | Effect类型 | 描述 |
| --- | --- | --- | --- |
| Static | Execute | Instant | Static Cue直接操作static class(没有实例)适用于一次性的音效和粒子效果 |
| Non-Static | Add&Remove | Duration或Infinite | Non-Static Cue会在添加(Added)时生成一个新的实例, 因为其是实例化的, 所以可以随时间推移执行操作直到被移除(Removed). 适合循环的声音和粒子效果, 其会在持续(Duration)或无限(Infinite)Effect被移除或手动调用移除时移除. |


### **EffectModifier（P1）**
`Modifier`可以修改`Attribute`并且是唯一可以<u>预测性</u>修改`Attribute`的方法. 一个`Effect`可以有0个或多个`Modifier`, 每个`Modifier`通过某个指定的操作只能修改一个`Attribute`

<!-- 原文此处一张 Modifier 结构图，本会话未取得图片 -->

Note：修改值暂时只支持读表，其他如当前值百分比、自定义函数等TODO支持

### **Effect堆栈（P2）**
BUff的代替，堆栈数=等级，影响Modifier

新的`Effect`实例不会添加到堆栈中, 而是修改当前已经存在的`Effect`堆栈数. 堆栈只适用于`持续(Duration)`和`无限(Infinite)Effect`

**P1**功能：堆栈上限、监听堆栈数变化时的回调

### ~~**Effect Tag**~~
**放在ability上**

| 分类 | 描述 |
| --- | --- |
| Effect Tags | Effect拥有的Tag, 它们自身没有任何功能且只用于描述Effect. |
| Granted Tags | 应用Effect时Target将获得的Tag. 当Effect移除时它们也会从Target中移除. 只作用于持续(Duration)和无限(Infinite)Effect. |
| Ongoing Tag Requirements | 这些Tag将决定Effect是开启还是关闭. Effect可以是关闭但仍然是应用的. 如果某个Effect由于不符合Ongoing Tag Requirements而关闭, 但是之后又满足需求了, 那么该Effect会重新打开并重新应用它的Modifier. 该Tag只作用于持续(Duration)和无限(Infinite)Effect. |
| Application Tag Requirements | 决定某个Effect是否可以应用到该Target的Tag, 如果不满足这些需求, 那么Effect就不可应用. |
| Remove Effects with Tags | 当Effect成功应用后, 如果位于Target上的任意Effect在其Tags或Granted Tags中有任意一个本Tag的话, 其就会自Target上移除. |


**阻塞机制**：如果`持续(Duration)`和`无限(Infinite)Effect`的Ongoing Tag Requirements未满足/满足的话, 那么Effect就可以被暂时的关闭和打开, 关闭`Effect`会移除其`Modifier`和已应用`Tag`效果, 但是不会移除该`Effect`, 重新打开`Effect`会重新应用其`Modifier`和`Tag`.

### **花费(Cost)Effect**
`Cost Effect`是一个带有一个或多个`Modifier`的`即刻(Instant)Effect`

默认情况下, `Cost Effect`是用于预测的, 建议使用该功能

一般不用为每个有花费的Ability都设置一个独一无二的`Cost Effect`, 而是复用一个`Cost Effect`, 只需在Ability中指定花费值

### **冷却(Cooldown)Effect**
`Cooldown Effect`是一个不带有`Modifier`的`持续(Duration)Effect`

默认情况下, `Cooldown Effect`是用于预测的, 建议使用该功能

### 配置
根据策划的反馈，仍用Excel作为GAS编辑器，便于高效拷贝构造Ability

Effect的配置会有一个UI窗口，方便操作

### RPC优化
**RPC合批（P1）**

每次Effect触发都是一次RPC. 在同一时刻触发多个Effect的情况下, 可以将它们合成一个RPC（帧末）

**客户端Cue**

用于触发Cue的函数默认是同步的. 每个Cue事件都是一个RPC. 这会导致大量RPC.可以通过使用客户端Cue来避免这个问题. 客户端Cue只能在单独的客户端上Execute, Add或Remove.

可以使用客户端Cue的场景:

+ 伤害飘字
+ 动画触发的Cue

### **EffectContext（P2）**
<u>EffectContext</u>存有关于`Effect`创建者和<u>TargetData</u>的信息

可以通过使用一个`EffectContext`来使得在`Cue`中访问`TargetData`, 比如用在可以伤害多个敌人的霰弹枪

## **Ability**
第三方客户端不会再运行Ability，由Effect进行同步或者是由Task进行RPC

`Ability`使用<u>AbilityTask</u>实现随时间推移而发生的行为, 例如等待某个事件, 等待某个Attribute改变, 等待玩家选择一个目标

<!-- 原文此处一张 Ability 状态图，本会话未取得图片 -->

can_trigger：CanActivateAbility

on_start：ActivateAbility

on_end：EndAbility

我打算把UE的_CommitAbility_加进来，因为实际开发中是有很多类似判定帧的需求，需要等到那一帧再判定一次花费、目标等

### **激活Ability**
提供了三种激活`Ability`的方法: 通过`Tag`, `Ability`授予即激活, 和Event

对于**客户端预测**`Ability`的激活顺序:

1. **所属(Owner)客户端**调用`Try`
2. 调用`CanActivateAbility()`并返回是否满足`Tag`需求, 是否满足花费, `Ability`是否不在冷却期和当前是否没有其他实例被激活
3. 调用`CallServer RPC`并传入其生成的`Prediction Key`
4. 调用`ActivateAbility()`最终激活Ability

**服务端**接收到`CallServer RPC`

1. 调用`CanActivateAbility()`并返回是否满足`Tag`需求, 是否满足花费, `Ability`是否不在冷却期和当前是否没有其他实例被激活
2. 如果成功则调用`ClientActivateAbilitySucceed()`告知客户端更新它的`AbilityInfo`(即该激活已由服务端确认)并触发`OnConfirmDelegate`
3. 调用`ActivateAbility()`最终激活Ability

如果服务端在任意时刻激活失败, 就会调用`ClientActivateAbilityFailed()`, 立即终止客户端的`Ability`并撤销所有预测的修改.

### **实例化策略**
缓存池

| 实例化策略 | 描述 | 何时使用的例子 |
| --- | --- | --- |
| 按Entity实例化(Instanced Per Entity) | 每个Entity只能有一个在激活之间复用的Ability实例. | 推荐的实例化策略. 可以对任一Ability使用并在激活之间提供持久化. 使用时需要在激活之间手动重置变量. |
| 按操作实例化(Instanced Per Execution) | 每有一个Ability激活, 就有一个新的Ability实例创建. | 这些Ability的好处是每次激活时变量都会重置, 其性能要比Instanced Per Entity差, 因为每次激活时都会生成新的Ability. |
| 非实例化(Non-Instanced) | 没有实例创建. | 它是三种方式中性能最好的, 但是使用它是最受限制的. 非实例化(Non-Instanced)Ability不能存储状态, 这意味着没有动态变量和不能绑定到AbilityTask委托. 使用它的最佳场景就是需要频繁使用的简单Ability, 像MOBA或RTS游戏中小兵的基础攻击、跳跃 |


修改了一下默认实例化策略：按操作实例化——>按Entity实例化

### **Net Execution Policy**
| 执行策略 | 描述 |
| --- | --- |
| Local Predicted | Local Predicted Ability首先在所属(Owner)客户端激活, 之后在服务端激活. 服务端版本会纠正客户端预测的所有不正确的地方. 参见Prediction. |
| Server Only | Ability只运行在服务端. 被动Ability一般是Server Only |


决定某个`Ability`是否是客户端可<u>预测</u>的, 同时影响`Cost`和`Cooldown Effect`

移除ACTIVE_PREDICT_ONLY（只预测激活）概念

### **Ability Tag**
部分Tag：

| Tags | 描述 |
| --- | --- |
| Ability Tags | ability自身Tag |
| Interrupt Abilities with Tag | 打断其他ability的Tag Ability被触发，打断所有拥有任意该集合中Tag的其他Ability |
| Block Abilities with Tag | 禁止其他ability的Tag 当Ability被触发，拥有任意该集合中Tag的其他Ability将无法被触发 |
| Provide Tags | Ability激活后，赋予使用者Tag |
| Source Required Tags | 使用者拥有任一Tag时Ability能被激活、持续生效 使用者Tag变更，且此字段下所有Tag都被删除，Ability会被中断 |
| Target Required Tags | 目标拥有任一Tag时Ability能被激活、持续生效 使用者Tag变更，且此字段下所有Tag都被删除，Ability会被中断 |


### **传递数据到Ability**
通过Even、使用WaitEventAbilityTask、保存数据到Entity等三个方法可以传递数据到ability

### RPC优化
如果`Ability`在一帧同一原子(Atomic)操作中执行完了Start——>End, 我们就可以优化该工作流, 将所有RPC整合为1个RPC

## **Ability Task**
非一帧完成的Ability默认带Task

为了实现随时间推移而触发或响应一段时间后触发的委托操作, 推荐使用`AbilityTask`

提供以下Task：

+ 延迟的Task
+ 播放动画的Task
+ 响应`Attribute`变化的Task
+ 响应`Effect`变化的Task
+ 响应玩家输入的Task

和Ability一样，`AbilityTask`只运行在Owner客户端或服务端

> UE的建议：然而, 可以通过设置`bSimulatedTask = true`使`AbilityTask`运行在第三方Client上,并将所有成员变量设置为同步的, 这只在极少的情况下有用, 比如在移动`AbilityTask`中, 不想同步每次移动变化, 但是又需要模拟整个移动`AbilityTask`, 所有的`RootMotionSource AbilityTask`都是这样做的
>

### **Task & Prediction Window**
TODO：将Task和PredictionWindow一起实现，减少概念

预测写法（示例，原文为伪代码）

```plain
class AbilityTask_WaitEvent:

    def on_event_callback(self):
        if self.ability_comp.is_valid:
            self.ability_comp.consume_replicated_event("Event", self, self.get_activation_prediction_key())
            if self.should_broadcast_ability_task_delegates():
                self.on_event_broadcast()
            self.end_task()

    def on_local_event_callback(self):
        with ScopedPredictionWindow(self.ability_comp):
            if self.ability_comp.is_valid and self.is_predicting_client():
                self.ability_comp.server_set_replicated_event("Event", self, self.get_activation_prediction_key(), self.ability_comp.scoped_prediction_key)
            self.on_event_callback()

    def activate(self):
        if self.ability_comp.is_valid:
            if self.ability.is_locally_controlled():
                # 等待事件callback
                self.ability_comp.local_callbacks.add(self.on_local_event_callback)
                self.registered_callbacks = True
            else:
                if self.add_replicated_delegate("Event", self.on_event_callback):
                    return

    def on_destroy(self):
        if self.registered_callbacks and self.ability_comp.is_valid:
            self.ability_comp.local_callbacks.remove(self.on_local_event_callback)
        super().on_destroy()
```

不预测写法：原文附两张图（阮嘉伟 / Lalo 标注），本会话未取得图片。

## **预测(Prediction)**
客户端预测的意思是客户端无需等待服务端的许可而激活`Ability`和应用`Effect`. 它可以"预测"它将应用`Effect`的目标. 服务端在客户端激活之后运行`Ability`(网络延迟)并告知客户端它的预测是否正确, 如果客户端的预测出错, 那么它就会"回滚"其"错误预测"的修改以匹配服务端.

**什么是可预测的:**

+ Ability激活
+ 触发事件
+ Effect应用
    - Attribute修改
    - Tag修改
+ Cue
+ 动画

**什么是不可预测的:**

+ Effect移除

#### **Prediction Key**
预测建立在`Prediction Key`的概念上, 其是一个由客户端激活`Ability`时生成的整型标识符.

+ 客户端激活`Ability`时生成`Prediction Key`, 这是`Activation Prediction Key`.
+ 客户端`CallServer RPC`将该`Prediction Key`发送到服务端.
+ 客户端在`Prediction Key`有效时将其添加到应用的所有`Effect`.
+ 客户端的`Prediction Key`出域. 之后该`Ability`中的预测Effect需要一个新的<u>Scoped Prediction Window</u>.
+ 服务端从客户端接收`Prediction Key`.
+ 服务端将`Prediction Key`添加到其应用的所有`Effect`.
+ 服务端同步该`Prediction Key`回客户端.
+ 客户端使用`Prediction Key`从服务端接收同步的`Effect`, 该`Prediction Key`用于应用`Effect`. 如果任何同步的`Effect`与客户端使用相同`Prediction Key`应用的`Effect`相匹配, 那么其就是正确预测的. 目标上暂时会有`Effect`的两份拷贝直到客户端移除它预测的那一个.
+ 客户端从服务端上接收回`Prediction Key`, 这是同步的`Prediction Key`, 该`Prediction Key`现在被标记为陈旧(Stale).
+ 客户端移除所有由同步的陈旧(Stale)`Prediction Key`创建的`Effect`. 由服务端同步的`Effect`会持续存在. 任何客户端添加的和没有从服务端接收到一个匹配的同步版本的都被视为错误预测.

#### Task & **Prediction Window**
为了在`AbilityTask`的回调函数中预测更多的行为, 需要使用新的`Scoped Prediction Key`创建`Scoped Prediction Window`, 有时这被视为客户端和服务端间的同步点(Sync Point).

## 其他
+ Effect可以赋予Ability
+ 运行时创建动态`Effect`
+ 安全策略：客户端是否有权限自由地发起/终止Ability

