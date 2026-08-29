# T10 · 预测与和解：CharacterMovement 与 NetworkPrediction（重章）

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **CMC 的预测链是一个完整闭环，且每一环都落在源码坐标上**：客户端 `ReplicateMoveToServer`（CharacterMovementComponent.cpp:8907）保存 FSavedMove → 服务器 `ServerMove_PerformMovement`（:9967）**不信任客户端 dt、按时间戳差重算步长**并权威模拟 → 超差时 `ServerMoveHandleClientError`（:10188）下发 ClientAdjustPosition → 客户端按时间戳 ack（:11223-11295）并 `ClientUpdatePositionAfterServerUpdate`（:8606）**重放全部未 ack 的 SavedMove**。纠错由复制主循环每包至多一次送出（NetDriver.cpp:5977-5990，见 T5）。
2. **它成立的三个硬前提**：①移动是「输入(ts,dt,flags,accel) → 状态」的**可重放纯函数**（两端共用 `MoveAutonomous`，:10053 与 :8686 是同一个函数）；②时间戳定点量化、步长钳制（`MaxMoveDeltaTime`，GameNetworkManager.h:145-146）保证两端步长一致；③状态差量小且可量化（`FVector_NetQuantize10/100`，:10088-10090）。**通用 Actor 一样都没有**——这就是「为什么只有移动能预测」的结构性答案，不是「Epic 没写」。
3. **NetworkPrediction 插件是 UE 内最接近目标引擎「整帧确认/回滚单元」的实现**：Fixed ticking 策略的注释原文 "Everyone ticks at same fixed rate. **Supports group rollback**."（NetworkPredictionConfig.h:13），group rollback 有真实现（`ReconcileSimulationsPostNetworkUpdate` 对所有服务先 `PreStepRollback` 再 `StepRollback`，NetworkPredictionWorldManager.cpp:161-228）——但插件 IsBetaVersion=true、默认禁用（T0 表）。

## 10.1 客户端链路（逐步骤 + 数据结构）

`UCharacterMovementComponent::ReplicateMoveToServer`（Engine/Source/Runtime/Engine/Private/Components/CharacterMovementComponent.cpp:8907-9096）：

| 步骤 | 行号 | 说明 |
|---|---|---|
| 前置门 | :8912-8917 | `PC->AcknowledgedPawn != CharacterOwner` 则不发——注释明言否则 "flood the reliable buffer" |
| 时间戳/步长量化 | :8933 | `ClientData->UpdateTimeStampAndDeltaTime` |
| OldMove 选择 | :8938-8951 | 最旧的「重要」未 ack move（IsImportantMove 阈值判定） |
| 新 SavedMove | :8954-8961 | `CreateSavedMove` + `SetMoveFor`（记录输入与起始状态） |
| **move 合并** | :8966-9015 | PendingMove 可合并则**回退到起始位置**（带碰撞重叠测试 :8975）后 CombineWith——省 RPC 的带宽优化 |
| 本地执行 | :9024 | `PerformMovement(NewMove->DeltaTime)`（预测） |
| 记录终态 | :9026 | `PostUpdate(PostUpdate_Record)` |
| 入队 | :9032 | `SavedMoves.Push` |
| **发送节流** | :9034-9048 | `GetClientNetSendDeltaTime` clamp [1/120, 1/5]（:9039），配置项 `ClientNetSendMoveDeltaTime*`（GameNetworkManager.h:156-165） |
| 上行 | :9084-9091 | packed RPC（`CallServerMovePacked` :9137）或经典 `CallServerMove` :9192 |

数据结构：`FSavedMove_Character`（声明 Engine/Source/Runtime/Engine/Classes/GameFramework/CharacterMovementComponent.h，字段 TimeStamp/DeltaTime/Acceleration/Start/End 变换/CompressedFlags/MovementBase）；`FNetworkPredictionData_Client_Character`（SavedMoves 队列、PendingMove、LastAckedMove、ClientUpdateRealTime）。

## 10.2 服务器链路与信任边界

`ServerMove_PerformMovement`（:9967-10066）：

- **时间戳验证**：`VerifyClientTimeStamp`（:9995）失败即丢弃（只告警）——防重放/乱序的第一道闸。
- **准入门**：`PC->NotifyServerReceivedClientData`（:10014）返回 false → 加速度清零（移动仍执行）——项目可在此做「本帧输入禁用」类惩罚。
- **dt 由服务器重算**：`ServerData->GetServerMoveDeltaTime(ClientTimeStamp, TimeDilation)`（:10022）——**客户端的 DeltaTime 只用于本地预测，权威步长取客户端时间戳差**，两端步长一致性的锚点。
- 权威模拟：`MoveAutonomous(ClientTimeStamp, DeltaTime, MoveFlags, Accel)`（:10053）。
- **纠错判定只对 NewMove**（:10061-10065）→ `ServerMoveHandleClientError`（:10188）：
  - 纠错率限制：`AGameNetworkManager::WithinUpdateDelayBounds`（:10204-10211）；
  - **信任阈值**：落地/换 base 时「server trusts the client (within a threshold)」（:10241-10257，`ClientAuthorityThresholdOnBaseChange`、`MaxFallingCorrectionLeash`）——纠正风暴的工程抑制；
  - `ServerCheckClientError`（:10517）比较位置误差，超差 → `ClientAdjustPosition`/`ClientVeryShortAdjustPosition` 下发。
- `_Validate` 钩子：`ServerMove_Validate`（:10618）——见 T7 的机制与局限。
- **客户端权威的显式开关**：`AGameNetworkManager::ClientAuthorativePosition`（GameNetworkManager.h:188-189，默认 **false**）与 CMC 的 `bClientIgnoreMovementCorrections`（:11248-11263，**收到纠正只 ack 不应用**——客户端单方面拒绝服务器的信任点）。引擎默认信任面：移动「客户端说了算、服务器复核」是唯一大宗；其余（如 firing）信任与否全部是项目逻辑。

## 10.3 纠正与重放（客户端侧）

`ClientAdjustPosition_Implementation`（:11223-11330+）：按 TimeStamp 找 SavedMove（:11285）→ `AckMove`（ack 并裁剪队列，:11295）→ 应用服务器位置/速度/base/移动模式（相对 base 坐标变换 :11299-11315）→ 触发重放。
`ClientUpdatePositionAfterServerUpdate`（:8606-8690+）：对每个未 ack SavedMove：`PrepMoveFor`（恢复输入）→ **`MoveAutonomous`（与服务器同一函数）**（:8686）→ `PostUpdate(PostUpdate_Replay)`；root motion 与跳跃/蹲伏状态在重放前显式备份恢复（:8648-8657）。

## 10.4 NetworkPrediction 插件（group rollback 的真实现）

- 成熟度：IsBetaVersion=true、EnabledByDefault=false（T0 表）。
- `ENetworkPredictionTickingPolicy`（NetworkPredictionConfig.h:9-17）：Independent(客户端本地帧率/服务器按输入速率) | **Fixed**（:13，注释 "Everyone ticks at same fixed rate. Supports group rollback."）。默认设置 `PreferredTickingPolicy = Fixed`、`FixedTickFrameRate = 60`（NetworkPredictionSettings.h:18-29）。
- Fixed 累加器：`BeginNewSimulationFrame_Internal`（NetworkPredictionWorldManager.cpp:254-318）——`UnspentTimeMS += 帧毫秒`，`while (UnspentTimeMS >= FixedStepMS)` 内完成 ProduceInput → 全部 Fixed 服务 Tick。
- **group rollback**：`ReconcileSimulationsPostNetworkUpdate`（:111-120 → :122）在客户端网络更新后：向所有 `IFixedRollbackService::QueryRollback` 取最小回滚帧（:161-169）→ **for Frame = RollbackFrame..PendingFrame：所有服务先 `PreStepRollback` 再 `StepRollback`**（:176-213，注释 :196-197 "Everyone must apply corrections and flush as necessary before anyone runs the next sim tick"）→ 恢复 PendingFrame。服务接口与回滚决定逻辑：NetworkPredictionService_Rollback.inl:20-29、67-130（`ShouldReconcile` 比较 Sync/Aux 状态）。
- 与 CMC 的关系：并列的两套系统（CMC 是主干默认，NP 是下一代实验），NP 不依赖 SavedMove 的「单组件状态差」，而是**整组服务共同回滚**——这正是目标引擎「ECS/Ability/体素层同一确认回滚单元」在 UE 里的唯一原型。

## 10.5 服务器端延迟补偿（回溯命中）

引擎**没有**通用的 hit-rewind/命中回溯服务。存在的最接近物：Chaos 物理的 `RewindData`（PBDRigidsSolver.cpp:530-537 的 resim 缓存注入）——那是物理重模拟缓存，不是按客户端 RTT 回溯世界状态的 API。经典射击项目的「回溯到开枪瞬间」全部自建（历史环形缓冲）。裁决：**「引擎提供」不存在，属 R9 第三类**；可挂载点：Server RPC 处理时按 `PlayerState->ExactPing` 自存状态历史。

## 10.6 为什么通用 Actor 做不到（结构性原因）

对照 CMC 的三个前提（结论 2）：通用 Actor 属性复制是「最新值获胜」的最终一致（T7），(1) 无输入抽象（没有可上行的「本帧输入」概念）、(2) 无可重放的状态转移函数（Tick 副作用任意）、(3) 无时间戳/步长契约（属性变化无序号）。因此「预测」必须由组件自带协议（SavedMove）实现，引擎不提供通用的 per-actor 预测框架——NetworkPrediction 的服务模型（模型定义声明 SupportedTickingPolicies，NetworkPredictionConfig.h:58-70）就是 Epic 对这个缺口的正式回答，但仍是 Beta。

## 意外发现

1. 服务器把「客户端位置」用于误差比较时，对「客户端省带宽不发 base」的合法情形做了专门容忍（:10226-10239）——协议省略与纠错的交互比文档承认的更精细。
2. 重放循环对跳跃/蹲伏/root motion 做了显式的「真实值备份-恢复」（:8648-8657）——**重放不是纯函数**，引擎靠备份恢复压平副作用；确定性模型里这就是必须由提交点消灭的那类补丁。
3. `bClientIgnoreMovementCorrections` 存在且默认可开（:11248）——客户端可以「合法地」无视纠正，反作弊视角的信任边界默认洞开。
4. Deprecated 的经典 `ServerMove` 与 packed 路径并存（:10069-10135），同一逻辑三份实现（PerformMovement/ServerMove_Implementation/Deprecated ServerMove）——迁移期债。

## 对目标环境的迁移含义

目标引擎的「预测窗口 + 整帧确认/回滚」应把 CMC 的**协议形状**与 NP 的**回滚粒度**合并、并去掉两者的历史包袱：继承①「时间戳 ack + 服务器重算 dt」（两端步长锚点）、②「纠错率限制 + 信任阈值」（纠正风暴抑制，:10204-10257）、③「每包至多一次纠正」；替换④ CMC 的备份-恢复式重放——确定性 tick + 状态哈希下重放即「从纠正点重跑固定步长帧」，无需 SavedMove 的字段级保存；拒绝⑤「客户端可无视纠正」的默认洞（改为签名纠正或断连计数）。体素世界的挖掘/放置天然是离散输入事件（可重放、可哈希），比连续移动更适合整帧确认模型——**移动预测反而是目标引擎里风险最低的一块**。
