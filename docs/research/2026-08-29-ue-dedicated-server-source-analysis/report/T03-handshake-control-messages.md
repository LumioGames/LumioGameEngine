# T3 · 握手、控制消息与版本校验

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。NMT 全表见 appendix/control-messages.csv。

## 结论先行

1. **一次连接 = 三条并行协议栈串联**：①PacketHandler 层的无状态握手（StatelessConnectHandlerComponent，HMAC cookie 验证、不占服务器内存、防 DRDoS）；②控制通道上的登录状态机（Hello → Challenge → Login → Welcome → Join，每一步都校验「上一步是否完成」，由 `ExpectedClientLoginMsgType` 强制顺序）；③版本与内容校验（`FNetworkVersion` 校验和 + `EEngineNetworkRuntimeFeatures` 位集，**两者都是精确相等判等，无兼容窗口**）。
2. **UE 的网络版本号是一个 CRC32**，组成 = 项目名 + 项目版本串 + NetCL（生成头 `ENGINE_NET_VERSION`）+ 全部网络自定义版本（`FGameNetworkCustomVersion` 等），`IsNetworkCompatible` 就是 `Local == Remote`（NetworkVersion.cpp:263-271），另有项目可绑定的 override 委托（:230、:265）。**对目标引擎「Release 精确匹配」诉求：UE 的答案是「单版本共存、不匹配即拒」，与多版本共存完全对立——这不是实现缺口，是设计立场。**
3. **5.8 的 NMT_* 不是具名枚举而是宏生成的匿名常量**（`DEFINE_CONTROL_CHANNEL_MESSAGE(Name, Index, ...)`，DataChannel.h:154-158），序号显式指定且**存在空洞**（7、8、11、14、33 未用）。控制消息全部可靠（`FControlChannelOutBunch` 强制 `bReliable=true`，DataBunch.cpp:209-215）、未 ack 超 1 秒重发（DataChannel.cpp:2236-2247）。

## 3.1 无状态握手：cookie、时间窗、防了什么

设计文档就在实现文件的头注释（Engine/Source/Runtime/Engine/Private/PacketHandlers/StatelessConnectHandlerComponent.cpp:30-160）：

```
Client Initial Connect    →  Server Stateless Challenge（下发 Timestamp + Cookie）
Client Challenge Response →  Server: 校验通过则创建 UNetConnection，再发 Ack
（Restart Handshake 变体：服务器收到未知 IP 的游戏包 → 1 字节响应触发重启 → 客户端带 OriginalCookie 重应答 → 连接可恢复）
```

- **Cookie 公式**（:146）：`Cookie = HMAC(HandshakeSecret, Timestamp + ClientIP + ClientPort)`。服务器持有 2 个大随机 HandshakeSecret（:142-143），**不保存任何每连接状态**——收到应答时用 SecretIdBit 指示的当前/上一把 Secret 重新计算并比对（:148-150）。
- **密钥轮换**：每 15 + rand(0,5) 秒换新 Secret，旧值同窗口内仍接受（:156）。
- **时间窗**：应答携带挑战时的服务器 Timestamp，校验其新鲜度（:159 注释 "Checks on the handshake Timestamp... limiting replay attacks"）——轮换窗 + 时间窗共同限制重放。
- **DRDoS 缓解**：初始包定长填充（PacketSizeFiller，:121-125），服务器忽略尺寸不符的初始包；Restart 响应最小化到 1 字节（:74-75）。
- **防住了**：纯伪造源的连接洪泛（无内存分配即被 HMAC 拒）、反射放大。
- **防不住**（源码内注释与结构可见的边界）：**不认证「人」**——cookie 只绑定 IP:Port，拿到 cookie 的中间人/被劫持客户端可完成握手；无抗重放的会话绑定（连接建立后的鉴权完全交给 Login 层的游戏凭据）；Timestamp 是服务器自己的时钟，防的是离线重放 cookie，不是防 MITM。
- 相关 CVar（注册点同文件）：`net.MagicHeader`(:236)、`net.HandshakeResendInterval`(默认 1s,:246)、`net.MinHandshakeVersion`(:265)、`net.CurrentHandshakeVersion`(:270)、`net.HandshakeEnforceNetworkCLVersion`(:282)、`net.VerifyNetSessionID`(:289)、`net.VerifyNetClientID`(:296)、`net.VerifyMagicHeader`(:303)。SessionID/ClientID 的用途（:105-106）：SessionID 每次**非 seamless** travel 递增、ClientID 每连接递增——区分同地址新旧连接；注释自评 "This is not a complete solution"（:183-185，见 T15）。
- **Restart Handshake 能恢复连接**（:72-100 的协议图 + "Connection restored"）——这是 UE 里唯一一处「游戏包来自未知 IP → 让对端重走握手 → 恢复既有 UNetConnection」的机制，**但触发条件是 NAT 重绑定这类地址变化，不是断线重连**（连接对象从未被销毁）。T15 的结论不受影响，但此机制证明引擎在包层有恢复原语。

## 3.2 登录状态机与登录钩子链

- 分发：`UControlChannel::ReceivedBunch`（DataChannel.cpp:1817）→ `Driver->Notify->NotifyControlMessage(...)`（:2032/2038）。**`UNetDriver::NotifyControlMessage` 不存在**——`FNetworkNotify` 是接口（NetworkDelegates.h:85），实现只有 `UWorld::NotifyControlMessage`（World.cpp:7243，登录期与常规期）与 `UPendingNetGame::NotifyControlMessage`（PendingNetGame.cpp:243，客户端握手期），外加 beacon 双侧（OnlineBeaconHost.cpp:140-155 / OnlineBeaconClient.cpp:395）。
- **顺序强制**：服务器侧入口先过 `IsClientMsgTypeValid`（World.cpp:7339；实现 NetConnection.cpp:5846-5870）——`ExpectedClientLoginMsgType` 由 IpConnection 初始为 `NMT_Hello`（OnlineSubsystemUtils 插件 IpConnection.cpp:178），发 Challenge 时改为 `NMT_Login`（NetConnection.cpp:6192-6193）。乱序消息直接拒。**这就是「确切序列」的执行器**：
  `NMT_Hello(客户端) → [版本检查：失败发 NMT_Upgrade 或 NMT_Failure] → NMT_Challenge(服务器) → NMT_Netspeed + NMT_Login(客户端) → [服务器 PreLogin/Login] → NMT_Welcome(服务器) → NMT_Join(客户端) → 通道全面打开`。
- **登录钩子链的实际调用点**：`AGameSession::ApproveLogin`（GameSession.cpp:220-234，容量拒绝 "Server full."）在 `AGameModeBase::PreLogin` 内被调（GameModeBase.cpp:690/715 两处）；`NMT_Login` 处理（World.cpp:7485 起）最终走 GameMode 的 Login/PostLogin；`Logout` 的调用点在 `AController::Destroyed`（Controller.cpp:603，全链见 T15）。**准入判定能挂在**：StatelessConnect 层（无）、`PreLogin`（引擎推荐位，返回错误串 → `NMT_Failure` World.cpp:7434/7454）、`AGameSession::AtCapacity`、`FGameModeEvents` 委托族。
- 加密握手：`NMT_EncryptionAck`（NetConnection.cpp:903 SendClientEncryptionAck）；失败路径 `SendChallengeControlMessage` → `Close(EncryptionFailure)`（NetConnection.cpp:6214-6219）；开关 `net.AllowEncryption`（NetDriver.cpp:529，0/1/2=禁/允/强制）。
- 客户端侧失败显示：`NMT_Failure` 的 `ErrorMsg` 直接进 `UEngine::BroadcastNetworkFailure` → 游戏层 UI（Travel 失败图）；版本不匹配时客户端收到 `NMT_Upgrade`（PNG.cpp:254-260）触发补丁/升级流程（引擎内只置 bUpgradeHandlingInstalled 位，具体 UI 归平台层——[游戏侧行为，未核实]）。

## 3.3 版本校验的确切组成（对应「Release 精确匹配」）

`FNetworkVersion::GetLocalNetworkVersion`（Engine/Source/Runtime/Core/Private/Misc/NetworkVersion.cpp:223-261）：

```
VersionString = "{FApp::GetProjectName()} {GetProjectVersion()}, NetCL: {GetNetworkCompatibleChangelist()}"
              + ", {每个网络自定义版本名}: {版本}"        # FCustomVersionContainer
LocalNetworkVersion = CRC32(小写(VersionString))
```

- `GetProjectVersion`：static 默认 **"1.0.0"**（:88-92），由游戏启动时 `SetProjectVersion` 写入（:100-111，来源为项目 .uproject/配置的版本串）。
- `GetNetworkCompatibleChangelist`：返回生成头 `ENGINE_NET_VERSION`（:135-137），可被 `networkversionoverride` CVar/命令行覆盖（:141-150）。本源码树的引擎侧数值即 Build.version 的 CompatibleChangelist=55116800 体系（项目打补丁时由此推进 NetCL）。
- 自定义版本：`FGameNetworkCustomVersion`（SetGameNetworkProtocolVersion :113-121）+ UE::Net 私有容器（Iris 等子系统往里加）——**引擎在版本串里塞的是「序列化语义的版本」，不是内容版本**。
- `IsNetworkCompatible`：**精确相等**（:263-271），或项目 override 委托。`AreNetworkRuntimeFeaturesCompatible` 亦精确相等（:295-298，Iris 开关位不同即不兼容，:302-309）。
- 校验发生点：握手层（StatelessConnect 的 NetworkVersion 字段，:114）+ `NMT_Hello` 处理（World.cpp:7369 起，不匹配 → `NMT_Upgrade` World.cpp:7397 / `NMT_Failure`）+ `UNetConnection::HandleReceiveNetUpgrade` → `Close(OutdatedClient)`（NetConnection.cpp:1367-1372）。
- **对目标引擎的直接结论**：UE 把「线协议兼容性」压成一个 32 位校验和并要求全等——没有协商、没有能力降级、没有版本区间。目标引擎若要「服务端与客户端 Release 精确匹配」，UE 的做法可原样平移（Schema digest 的 CRC）；若要滚动更新，就必须在握手层引入「版本区间/能力协商」——UE 源码里没有任何可参考的实现，只有反例（Iris 开关都进精确匹配位集）。

## 3.4 连接建立时序图（硬指标图 1）

见 appendix/diagrams.md（每框标函数与行号）。要点：握手（PacketHandler 层，无控制通道）→ 控制通道建立（`UControlChannel::OpenToken` 序）→ NMT 状态机 → Welcome 携带地图/RedirectURL → 客户端 `UPendingNetGame` 转正式 NetConnection（PNG.cpp:369-596 区间）→ Join → `APlayerController` 通道打开。

## 意外发现

1. 控制通道有 **RELIABLE_BUFFER 之外的二级队列**：`UControlChannel::QueuedMessages`（上限 `MAX_QUEUED_CONTROL_MESSAGES=32768`，ControlChannel.h:61-64），超限**绕过 Close() 直接置 USOCK_Closed**（DataChannel.cpp:2170-2176）——引擎作者也知道关连接的正规路径有时赶不上队列爆掉的速度。
2. `NMT_Login` 读载荷前把 `Bunch.ArMaxSerializeSize += 16MB`（World.cpp:7492-7498）——超大 join URL 是官方预期内的输入。
3. 未注册的未知控制消息在 dev 构建直接 `check` 崩溃、其余构建断连（DataChannel.cpp:2134-2142）——「向前兼容」在控制通道上不存在。
4. `NMT_Skip`/`NMT_Abort` 声明了载荷（FGuid）但引擎内**零发送点**（DataChannel.h:182-183 声明，World.cpp:7477/7481 空 case）——历史遗物，序号 12/13 占位。

## 对目标环境的迁移含义

握手层值得整体平移的是**无状态 cookie 的三件套**（HMAC(IP:Port+时间戳) + 双 Secret 轮换 + 定长防放大），它把「分配连接状态」推迟到密码学验证之后——对浏览器客户端（WebSocket 握手本身已过 TLS，但 WSS 升级请求仍可被伪造）同样成立。登录状态机的 `ExpectedClientLoginMsgType` 单步期待值是极简而正确的顺序防御，比「全套状态机」便宜得多。版本校验采纳「单 CRC、精确匹配」但必须在同一层留出「能力位集」的扩展位（UE 的 EEngineNetworkRuntimeFeatures 就是这么把 Iris 开关带进匹配的）——目标引擎的 Schema digest + Release id 应做成两个字段：前者精确匹配，后者用于未来的定向降级。
