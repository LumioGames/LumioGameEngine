# T9 · 时间、Tick 与步长

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **不存在 `net.MaxTickRate` 这类 CVar——预研的「名称待核」钉死为：服务器 tick 率是每 NetDriver 的 Config 属性 `NetServerMaxTickRate`**（NetDriver.h:876-877，ini 默认 30，BaseEngine.ini:1867 `[/Script/OnlineSubsystemUtils.IpNetDriver]`），经 `UGameEngine::GetMaxTickRate`（GameEngine.cpp:1719-1765，:1740-1746）对 DS 生效；`t.MaxFPS`（CVar，UnrealEngine.cpp:12136-12138，默认 0=不限）若设置则最终覆盖。DS 的帧率控制就是**睡眠等待**（UnrealEngine.cpp:3094-3097 `SleepNoStats`，注释「服务器不在乎墙钟精度」）。
2. **UE 服务器是非固定步长的，而 Epic 在需要的子系统里逐个补固定步长**：物理 substepping（Chaos Solver 的 while 循环切步）、NetworkPrediction 的 Fixed 策略（`ENetworkPredictionTickingPolicy::Fixed` 注释原文 "Everyone ticks at same fixed rate. **Supports group rollback**."）、引擎级 `bUseFixedFrameRate/FixedFrameRate`（Engine.h:1674-1678）。这些补丁彼此独立、没有一个统一的世界级固定步长提交点——这是与目标引擎画像最尖锐的对照。
3. **时间同步在 AGameStateBase 而非 APlayerState**（对常见说法的纠正）：服务器 10Hz 定时把世界时间写进 `ReplicatedWorldTimeSecondsDouble`（GameStateBase.cpp:54-69、155-162，无任何 RTT 补偿），客户端用「≤250 样本滚动均值 + 50% 阻尼」渐近偏移（:164-194）——**引擎实现不估计 RTT、不做半 RTT 校正**，偏移量内含整段下行延迟。

## 9.1 tick 率控制链（准确名称全表）

```
FEngineLoop::Tick (LaunchEngineLoop.cpp:5700-5701)
 └─ UEngine::UpdateTimeAndHandleMaxTickRate (UnrealEngine.cpp:2980)
     ├─ :3058 GivenMaxTickRate = GetMaxTickRate(DeltaRealTime)
     ├─ :3059 MaxTickRate = bUseFixedFrameRate ? FixedFrameRate : Given   # 固定帧率模式优先
     ├─ :3068-3077 IMaxTickRateHandlerModule 模块化接管机会
     └─ :3094-3097 [DS] FPlatformProcess::SleepNoStats(WaitTime)          # 服务器直接睡
UGameEngine::GetMaxTickRate (GameEngine.cpp:1719-1765)
 ├─ :1740-1746 NetDriver && (NM_DedicatedServer || (NM_ListenServer && bClampListenServerTickRate))
 │            → MaxTickRate = Clamp(NetDriver->GetNetServerMaxTickRate(), 1, 1000)
 └─ :1758-1762 Super（UEngine::GetMaxTickRate, UnrealEngine.cpp:12193-12240）非 0 时覆盖
              # Super 只读 SmoothedFrameRateRange 与 t.MaxFPS（:12234-12236）
```

- `t.MaxFPS`：CVar 注册 UnrealEngine.cpp:12136-12138，默认 0；`t.UnsteadyFPS` 调试 :12140-12150。
- `NetServerMaxTickRate`：NetDriver.h:876-877（UPROPERTY(Config)）；ini 值 BaseEngine.ini:1867（=30，同段 :1868 `MaxNetTickRate=120`）；setter NetDriver.cpp:8563-8574（同步 DDoS 上限并广播 OnNetServerMaxTickRateChanged）。
- **网络层还有第二重节流**：`UNetConnection::Tick`（NetConnection.cpp:4781）按 `DesiredTickRate = Clamp(EngineTickRate, 0, MaxNetTickRate=120)` 决定本帧是否跳过网络收发（:4808-4823）。
- 复制与 tick 的解耦点：TickFlush 消费 `GEngine->GetMaxTickRate`（NetDriver.cpp:1184），对象级 NetUpdateFrequency 独立生效（T5.11）；`bCPUSaturated = dt > 1.2×(1/MaxTickRate)`（NetDriver.cpp:6349）把「帧超时」变成复制调度输入。

## 9.2 TickGroup 与顺序控制

ETickingGroup 全表（EngineBaseTypes.h:83-110）：`TG_PrePhysics`(:86) → `TG_StartPhysics`(:89,Hidden) → `TG_DuringPhysics`(:92,可与物理并行) → `TG_EndPhysics`(:95,Hidden) → `TG_PostPhysics`(:98) → `TG_PostUpdateWork`(:101) → `TG_LastDemotable`(:104,Hidden) → `TG_NewlySpawned`(:107,反复重跑至无新对象) → `TG_MAX`。

FTickFunction 的依赖表达（EngineBaseTypes.h:183）：`TickGroup`(:195)/`EndTickGroup`(:203)（组区间）、`bHighPriority`(:227，组内先跑，执行 TickTaskManager.cpp:243-653)、`Prerequisites`(:262，显式前置 FTickPrerequisite :113-173)、`TickInterval`(:256，**按间隔节流的 tick**——UE 里「固定步长组件」的通用实现位)、`bAllowTickOnDedicatedServer`(:221)。**表达能力 = 组区间 + 显式前置 + 优先级 + 间隔**，但没有「帧内唯一提交点」语义——各 TickFunction 直接改状态，无提交/回滚边界。

## 9.3 Epic 补固定步长的清单（对照画像的核心证据）

| 子系统 | 固定步长实现 | 坐标 |
|---|---|---|
| 物理同步 substep | `UPhysicsSettings::bSubstepping`（默认 false）+ `MaxSubstepDeltaTime`（默认 1/60）+ `MaxSubsteps`（默认 6，Clamp 1-16）；Chaos Solver `AdvanceOneTimeStepTask::DoWork` 的 `while (StepsRemaining>0 && TimeRemaining>MinDeltaTime)` 切步循环 | PhysicsSettings.h:323-345；PhysicsSettings.cpp:21-27；PBDRigidsSolver.cpp:474-593（:568-572 循环、:591 AdvanceOneTimeStep） |
| 物理异步固定步长 | `bTickPhysicsAsync` + `AsyncFixedTimeStepSize`（默认 1/30），Solver 侧外部驱动 | PhysicsSettings.h:331-337 |
| NetworkPrediction Fixed | `ENetworkPredictionTickingPolicy::Fixed`（"Supports group rollback"）；累加器主循环 `while (UnspentTimeMS >= FixedStep)` 每步 ProduceInput→Tick 全部 Fixed 服务 | NetworkPredictionConfig.h:9-17；NetworkPredictionWorldManager.cpp:244-318（:272 累加、:274-317 循环） |
| 引擎级固定帧率 | `bUseFixedFrameRate` + `FixedFrameRate`（Engine.h:1674-1678），在 UpdateTimeAndHandleMaxTickRate :3059 优先于 GetMaxTickRate | UnrealEngine.cpp:3059 |
| 移动组件 | CMC 内部按 `MaxMoveDeltaTime`（GameNetworkManager，默认 0.125）切分移动 tick（见 T10） | GameNetworkManager.h:145-146 |

**裁决**：UE 的固定步长是**每子系统一个补丁**，不是一个架构属性。物理、网络预测、移动各自带着自己的 dt 切分与上限常量，且互不一致（1/60、1/30、0.125、FixedTickFrameRate=60）。Epic 在 NetworkPrediction 里承认了正确答案（Fixed + group rollback），但它是 Beta 插件且默认禁用——引擎主干仍是变步长的。

## 9.4 时间同步（纠正：在 AGameStateBase）

- 字段：`ReplicatedWorldTimeSecondsDouble`（GameStateBase.h:149-151，double，ReplicatedUsing）、`ServerWorldTimeSecondsDelta`(:153-155)、`ServerWorldTimeSecondsUpdateFrequency`(:157-159)。
- 服务器：仅 Authority 游戏世界启动 0.1s 循环定时器（GameStateBase.cpp:54-69），`UpdateServerWorldTimeSeconds` 直接写 `World->GetTimeSeconds()`（:155-162，**无 RTT 补偿**）；无条件复制（:250-263，DOREPLIFETIME :257）。
- 客户端（:164-194）：`offset_raw = 下发值 − 本地时钟`；滚动均值窗口 ≤250 样本（超 250 折叠，:176-180）；`delta += (target − delta) × 0.5`（:185-191）；`GetServerWorldTimeSeconds() = 本地时间 + delta`（:144-153）。
- **公式与缺陷**：不做 RTT/2 校正 → 系统性低估服务器时间约「下行单程延迟」；异常偏移（重连、卡顿尖峰）只被均值稀释，无 outlier 丢弃。引擎内 PlayerState 的 Ping（ExactPing 体系）不参与本公式。
- 回放快进后世界时间修正：`FinalizeFastForward` 手动重算偏移（DemoNetDriver.cpp:3026-3037）。
- 下发频率：`ServerWorldTimeSecondsUpdateFrequency` 是类默认属性（0.1s/10Hz），非 CVar（GameStateBase.cpp:36 注释 "Default to every 100 ms"）。

## 9.5 可变 delta time 的来源与后果

DeltaSeconds 由 FEngineLoop 按墙钟测量（无帧锁定），随后被三处驯化：带宽 token bucket 的 clamp（NetConnection.cpp:5116-5120，防 hitch 突发）、复制频率门的自适应插值（NetDriver.cpp:5391-5409）、物理 substep 切步。未被驯化的地方（游戏逻辑 Tick 本身）就是变步长不确定性的根源——这正是 NetworkPrediction 用 Fixed 策略绕开的东西（见 T10）。

## 意外发现

1. DS 的帧率控制就是**无精度睡眠**（UnrealEngine.cpp:3094-3097 注释自述），没有忙等或定时器——tick 率抖动直接进 DeltaSeconds。
2. `MaxNetTickRate=120`（ini）与 `NetServerMaxTickRate=30` 并存：网络层自我节流的上限独立于引擎 tick 上限（NetConnection.cpp:4808-4823）。
3. 时间同步的均值折叠（:176-180）是「250 样本上限的内存小抄」式实现——把 sum/num 折叠成均值再继续累积，等价于无限窗口均值（对持续漂移不敏感，对阶跃响应慢）。

## 对目标环境的迁移含义

目标引擎「固定步长 + 分相位 + 唯一提交点」在 UE 中找不到任何单一对应物——UE 的答案是把固定性下沉到需要的子系统（物理/NP/移动）并接受主干变步长。这反过来说明：**一旦提交点存在于架构层，T4 的 shadow state、T10 的 saved-move 重放、T13 的检查点、时间同步的偏移估计全部统一到「帧号 + 帧状态」一个坐标系上**；UE 每一处都要自造一套「上次状态/上次时间」的局部账本（changelist、SavedMove、checkpoint、ServerWorldTimeDelta），且彼此不通约。时间同步的教训：**至少把 RTT/2 校正和 outlier 丢弃做进协议**（UE 两样都没有，10Hz 均值 + 0.5 阻尼是带宽最优解而非精度最优解）。
