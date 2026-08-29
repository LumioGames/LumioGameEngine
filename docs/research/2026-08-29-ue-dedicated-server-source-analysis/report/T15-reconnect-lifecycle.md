# T15 · 断线重连、会话恢复与版本共存：「没有」的证明

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **断线即销毁，无等待窗口**：第 N 帧 TickFlush/TickDispatch 里 `Close(ConnectionTimeout)`；第 N+1 帧 `TickDispatch` 的残留连接清扫调 `UNetConnection::CleanUp`，**同一帧内联完成** PC 销毁 → `Logout` → PlayerState `OnDeactivated()`(默认 `Destroy()`)。全链坐标见下。引擎留下的唯一「重连档案」是 `AGameMode::InactivePlayerArray`——一份限时的 PlayerState 快照（按 UniqueId 或地址+名字匹配，非加密凭据）。
2. **会话恢复原语为零**：`ReconnectToken` / `ResumeToken` / `ReconnectSessionId` / `bReconnecting` 在 Engine/Source 全树 0 命中（搜索记录见下表）；控制台 `reconnect` 命令只是拿 `LastRemoteURL` 重新 Browse——全新连接、全新 PlayerController。OnlineSubsystem 的 `Reservation` 是派对/会话**预约**（beacon 名额预留），与断线重连无关。
3. **引擎唯一一处「连接跨越世界切换存活」是 seamless travel**：NetDriver 整体搬到新 World（`CopyWorldData`），新 PlayerController 直接**过继**旧连接（`NewPC->NetConnection = OldPC->NetConnection`）。Browse（硬切）则一律 `ShutdownWorldNetDriver`。graceful drain 的全局钩子不存在；最接近的三件局部能力是：每连接 `GracefulClose`（等可靠数据 ack，上限 2s）、travel 期间 `NotifyAcceptingConnection` 返回 Ignore、`AGameSession::AtCapacity`/`PreLogin` 的可覆写拒绝点。

## 15.1 断链销毁链（服务器侧，全坐标）

```
第 N 帧  UNetConnection::Tick (NetConnection.cpp:4781, 判定 :4921-4926)
          └─ (Now - LastReceiveRealtime) > GetTimeoutValue()   # 值来自 NetDriver 的 Config 属性
             └─ HandleConnectionTimeout (NetConnection.cpp:5198)
                 ├─ BroadcastNetworkFailure(ConnectionTimeout)  # :5202 → UEngine::HandleNetworkFailure (UnrealEngine.cpp:15230)
                 │    └─ 服务器分支不 travel（UnrealEngine.cpp:15275-15278 "Hosts don't travel"）
                 └─ Close(ENetCloseResult::ConnectionTimeout)   # :5205 → Close :1107（SendCloseReason :1142、关 Channels[0] :1146、USOCK_Closed :1149）
第 N+1 帧 UNetDriver::TickDispatch 残留连接清扫（NetDriver.cpp:2878-2893）
          └─ UNetConnection::CleanUp (NetConnection.cpp:1399)
              ├─ 子连接递归 :1402-1405；Close(Cleanup) :1412
              ├─ Driver->RemoveClientConnection :1425（NetDriver.cpp:7118-7131；RecentlyDisconnectedClients 仅做地址条目回收）
              ├─ 杀掉全部通道 :1437-1453
              ├─ DestroyOwningActor() :1480 → UNetConnection::DestroyOwningActor :1506 → PC->OnNetCleanup(this) :1514
              └─ MarkAsGarbage() :1490（连接对象等 GC）
APlayerController::OnNetCleanup (PlayerController.cpp:1467)
  ├─ World->DestroySwappedPC :1473（World.cpp:7228）
  ├─ Player->PlayerController = nullptr :1481-1484；NetConnection = nullptr :1486
  └─ Destroy(true) :1487 → UWorld::DestroyActor (LevelActor.cpp:839) → Destroyed() :926（同帧）
AController::Destroyed (Controller.cpp:595)
  ├─ [Authority] GameMode->Logout(this) :603 → AGameModeBase::Logout (GameModeBase.cpp:1051)
  │     └─ GameSession->NotifyLogout :1061（GameSession.cpp:350 → 在线会话注销）
  │         └─ AGameMode::Logout (GameMode.cpp:120)（仅 AGameMode 子类！）
  │             └─ AddInactivePlayer :126 → (GameMode.cpp:601) PlayerState->Duplicate() :608 存入 InactivePlayerArray :663
  │                寿命 InactivePlayerStateLifeSpan :618 / 上限 MaxInactivePlayers :666-681
  ├─ [Authority] CleanupPlayerState() :606 → (PlayerController.cpp:1392) PlayerState->OnDeactivated() :1397
  │     └─ APlayerState::OnDeactivated (PlayerState.cpp:128) → 默认 Destroy() :131   ← PlayerState 就地销毁
  └─ UnPossess / RemoveController :609-610
```

客户端侧断链：`BroadcastNetworkFailure` → NM_Client 分支 `bShouldTravel=true`（UnrealEngine.cpp:15275-15278）→ `HandleDisconnect`（:15512）→ `SetClientTravel("?closed")` :15527 → `UEngine::Browse` → `ShutdownWorldNetDriver`（:14656，注释 14660-14663 "completely disconnecting... Destroys the net driver"）。

## 15.2 重连痕迹搜索记录（「没有」的证据）

| 关键字 | 范围 | 命中 | 判定 |
|---|---|---|---|
| ReconnectToken / ResumeToken / ReconnectSessionId / bReconnecting | Engine/Source 全树 | **各 0** | 无 token 原语 |
| SessionResume（子串） | 同上 | 4，全为编辑器 PIE 的 PlaySessionResumed（PlayLevel.cpp:1079 等） | 与网络无关 |
| Reservation | Runtime/Engine + Runtime/Net | 网络/会话语义 0（其余为资产注册/Verse/Iris baseline） | 非重连 |
| Reservation | Plugins/Online/* | PartyBeacon*/OnlinePartyInterface.h:1215,1299 | 派对/会话**预约** |
| Reconnect | Runtime/Engine | 6 处：GameMode.h:136 注释；GameMode.cpp:109/114（InactivePlayer）；PlayerState.h:261/282（OnReactivated）；StatelessConnectHandlerComponent.cpp:183-185（ClientID 区分同地址新旧连接，注释自述 "This is not a complete solution"） | 仅快照匹配 |
| reconnect（命令） | UnrealEngine.cpp:15565 | `SetClientTravel(LastRemoteURL)` :15570 | 全新连接 |
| bShutdown / bIsLocalReplay / NotifyConnectionClosed | Runtime/Engine+Net | 网络 0 / 0 / 0 | 无 drain 开关、无该 API |

**引擎仅有的两个「重连」机制**：① `AGameMode::FindInactivePlayer`（GameMode.cpp:687）：`PostLogin` 记 `SavedNetworkAddress`（:112）→ 按 UniqueId（:712-715，桌面退化为地址+名字 :719-721）匹配 → `OverridePlayerState → APlayerState::DispatchOverrideWith`（PlayerState.cpp:95/107-113，**只恢复 spectator 标志/UniqueId/名字**）→ `OnReactivated`（默认空，PlayerState.cpp:134-137）；bIsTearingDown 时不留档（GameMode.cpp:606）。② StatelessConnectHandler 的 SessionID/ClientID（见 T3）。**没有跨地图、没有加密凭据、没有增量恢复。**

## 15.3 Seamless travel：连接存活的机制与「能否借用」裁决

分岔点：`AGameModeBase::ProcessServerTravel`（GameModeBase.cpp:477，bSeamless 判定 :488）与 `APlayerController::ClientTravelInternal_Implementation`（PlayerController.cpp:5735，bSeamless && TRAVEL_Relative → `World->SeamlessTravel` :5742-5745）；两边汇入 `UWorld::SeamlessTravel`（World.cpp:9135）→ `FSeamlessTravelHandler`（StartTravel :8274 / Tick :8614）。

连接保持的直接证据：NetDriver 搬 World（`CopyWorldData`，World.cpp:8526-8552：`LoadedWorld->SetNetDriver(NetDriver)` :8536、`NetDriver->SetWorld(LoadedWorld)` :8551-8552）；**`AGameModeBase::SwapPlayerControllers`（GameModeBase.cpp:561）：`NewPC->NetConnection = OldPC->NetConnection;` :568、`NewPC->SetPlayer(Player)` :570**——连接（UPlayer 身份）从旧 PC 过继给新 PC；旧 PC 挂 `PendingSwapConnection`（:582）等 `DestroySwappedPC`（World.cpp:7228）。保留名单 `GetSeamlessTravelActorList`（GameModeBase.cpp:539-546，永远保留全部 PlayerArray；客户端侧 PlayerController.cpp:3635）；客户端加载完成后 `ServerNotifyLoadedWorld`（PlayerController.cpp:697-724）才触发换 PC；PlayerState 数据经 `SeamlessTravelFrom/To`（PlayerController.cpp:3652-3662；PlayerState.cpp:413-418，引擎默认只搬 Score/Ping/PlayerId/UniqueId/PlayerName/StartTime/SavedNetworkAddress，CopyProperties 本体 PlayerState.cpp:116-126）。

**借用裁决**：seamless travel 证明「连接与 GameWorld 解耦、身份可过继」在 UE 对象模型内可行，但它依赖 NetDriver 单世界所有权 + Travel 状态机（TickWorldTravel 掳获 NextURL 期间 `NotifyAcceptingConnection` 直接 Ignore，World.cpp:7089-7093）。**做真正的断线重连（token + 增量恢复），可复用的挂载点是 `SwapPlayerControllers` 的「连接过继」手法 + `AGameMode::InactivePlayerArray` 的快照思路，但 token 签发/校验、副本状态保鲜（尤其离开玩家的实体状态）、超时回收都必须项目自建**——引擎没有提供其中任何一件。

## 15.4 优雅关闭 / 排空 / 多版本共存

- **每连接排空（最接近 drain）**：`UNetConnection::GracefulClose`（NetConnection.cpp:1176-1224；声明 NetConnection.h:1182）——`bPendingDestroy` → USOCK_Closing → 等所有通道可靠数据 ack（`TryClosePendingGracefulClose` :1206、`HasAcknowledgedAllReliableData` :1214）→ 真正 Close；上限 `GracefulCloseConnectionTimeout`（NetDriver.h:950-951，默认 2.0s），超时由 Tick 兜底（NetConnection.cpp:4924-4936）。开关 `net.GracefulCloseEnabled`（默认 true，NetConnection.cpp:248）。入口 `APlayerController::DestroyNetworkActorHandled`（PlayerController.cpp:292-298）。
- **拒绝新连接**：只有 travel 状态绑定的 `UWorld::NotifyAcceptingConnection`（World.cpp:7080，NextURL 非空 → Ignore :7089-7093）；**没有可主动进入的 drain 生命周期**。
- **登录层容量拒绝**：`AGameSession::AtCapacity`（GameSession.cpp:322，:338-340）→ `ApproveLogin` 返回 "Server full."（:220-234）→ `PreLogin` 两处调用（GameModeBase.cpp:690/715）。**项目可在此实现「只出不进」**。
- **服务器关停**：`UNetDriver::Shutdown`（NetDriver.cpp:2552）：尽力通知（NMT_Failure "Host closed the connection." + SendCloseReason + FlushNet，:2604-2609）→ 销毁各 PC 的 Pawn（:2613-2619）→ 逐连接 CleanUp（:2623）。是「广播后关」，不是排空。
- **多版本共存 / 滚动更新**：引擎内无任何钩子（上述搜索全 0）。版本不匹配的连接在握手层被拒（T3 的 FNetworkVersion 检查）——**UE 假设的运维形态是「一个服务器版本 + 客户端精确匹配」，滚动更新需要外部网关/会话层把旧客户端导流到旧版本进程**，这不在引擎词汇表内。

## 15.5 生产项目补那层的挂载点（可操作结论）

1. **会话 token**：`AGameModeBase::PreLogin` 的 Options 字符串（Login URL 参数）是引擎内置的凭据传递通道（T3 登录序列）；校验逻辑挂 `PreLogin` 返回错误串即拒——无需改引擎。
2. **状态保鲜**：复用 InactivePlayerArray 模式扩容（或项目自表）：断链时 `Logout` 里转存玩家实体状态（GameMode.cpp:120-126 是模板），重连时在 `PostLogin` 匹配恢复。
3. **排空**：`AtCapacity` + 自建 drain 状态（«拒绝 PreLogin 但不断旧连接»），配合每连接 GracefulClose 的 2s 可靠排空窗口。
4. **半开检测**：T14 的 ConnectionTimeout（默认 60s）+ 应用层心跳；1.5s 的静默停发窗口（T5）说明引擎对「假活连接」的唯一缓解就是超时。

## 对目标环境的迁移含义

目标引擎诉求「断线重连 + 恢复令牌 + 会话排空」在 UE 中全部属于 R9 第三类（源码里根本没有）或勉强第一类（GracefulClose 这种局部件）。可吸收的设计只有两个反面教训与一个正面件：反面①「断链即全毁 + PlayerState 快照匹配」证明**重连语义必须建在连接层之上**（token 属于会话，不属于 socket）；反面②「身份过继」（SwapPlayerControllers）证明连接与逻辑身份分离是对的，但 UE 只在 travel 这一条路径上做了它。正面件：GracefulClose 的「等可靠数据 ack 再关、带 2s 上限」是会话排空的最小正确语义，目标引擎的 drain 应以此为每连接原语，再叠加进程级「拒新保旧」。
