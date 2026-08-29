# T1 · 进程形态、权威判定与构建裁剪

> UE 5.8.2（git ff8421f2b）。除标注外 Verified-Src。

## 结论先行

1. **权威判定是两级纯查表**：`HasAuthority() = (GetLocalRole() == ROLE_Authority)`（Actor.h:4967-4970，inline），而 LocalRole/RemoteRole 是每 actor 一对 Role 字段（Actor.h:864/748）——没有任何运行期协商，Role 由生成路径与连接方向决定。`ENetMode` 四值（EngineBaseTypes.h:978-996），注释明示约定「NetMode < NM_Client 即某种服务器」；DS 上 `GetNetMode()` 有编译期短路（Actor.h:4989-4994：`IsRunningDedicatedServer()` 编译期判定 + GameNetDriver 特判）。
2. **编译期开关三件套**：`UE_SERVER`（定义于 TargetRules 生成的编译环境，DS target 全模块生效）、`WITH_SERVER_CODE`（Engine 模块内门控服务器逻辑，如 ServerReplicateActors 整体 `#if WITH_SERVER_CODE`，NetDriver.cpp:6281/5197）、`DEDICATED_SERVER`/`IS_MONOLITHIC` 类由 BuildConfiguration 与平台决定。渗透量级：`WITH_SERVER_CODE` 在 Engine/Private 下数百处（Estimated，grep 观察量级）；语义要点是**客户端构建真的不编译服务器代码段**——这决定了「打包客户端里剩多少服务器信息」的答案（T14）。
3. **单进程多世界/多房间：有骨架、无工程化**。多 NetDriver 并存是一等概念（FWorldContext::ActiveNetDrivers，Actor.cpp:3074 遍历），但共享 static（NetDriver.cpp:5207 的 DeltaTimeOverflow FIXME 自认多驱动互踩）、`GEngine->GetMaxTickRate` 只问一个 NetDriver（GameEngine.cpp:1740-1746 取 GameNetDriver 家族判断）、Iris 反而原生支持多 ReplicationSystem（ReplicationSystem.h:22-24 默认 8 个内联实例）——多世界支持是「不阻止但也没做完」。

## 1.1 权威判定的实现

| 调用 | 实现 | 判了什么 |
|---|---|---|
| `AActor::HasAuthority()` | `GetLocalRole() == ROLE_Authority`（Actor.h:4967-4970） | 本端对该 actor 是否权威——纯字段 |
| `AActor::GetLocalRole()` | `return Role;`（Actor.h:776） | |
| `AActor::GetRemoteRole()` | `return RemoteRole;`（Actor.h:4984-4987） | 对端角色（复制方向由此定） |
| `AActor::GetNetMode()` | Actor.h:4989+：DS 编译期短路；否则 `GetNetDriver()->GetNetMode()` | 进程形态，不是 per-actor 语义 |

ENetRole 全表（EngineTypes.h:3582-3593）：ROLE_None < ROLE_SimulatedProxy < ROLE_AutonomousProxy < ROLE_Authority < ROLE_MAX。ENetMode 全表（EngineBaseTypes.h:978-996）：NM_Standalone / NM_DedicatedServer / NM_ListenServer / NM_Client（注释：小于 NM_Client 的都是服务器变体）。

## 1.2 构建裁剪三态

- **编译期剥离**：`WITH_SERVER_CODE=0`（客户端/编辑器构建）时服务器分支整体消失（NetDriver.cpp:6281、5197 的函数体级 `#if`）；`UE_SERVER` 由 server target 类型定义（作用域是整个编译单元集合，渗透量级 Estimated：Engine/Runtime 内千处级出现）。
- **运行期跳过**：`GetNetMode()` 分支（渲染、音频、本地玩家逻辑在 NM_DedicatedServer 下早退）；`bAllowTickOnDedicatedServer`（EngineBaseTypes.h:221）控制组件 tick 是否在 DS 跑。
- **必须手工标**：复制体本身没有「服务器专属属性」标记——server-only 状态靠「不进 Replicated 声明」实现；cook 阶段的 server-only 资产剥离不在引擎网络层（属打包管线，本次未展开）。**漏标后果**：把服务器内部状态标成 Replicated = 泄漏给客户端（无任何告警）；把客户端需要的标漏 = 静默不同步。

## 1.3 单进程多世界

- 支持：多 World 各持 GameNetDriver（FWorldContext::ActiveNetDrivers；SetNetDormancy 遍历全部驱动，Actor.cpp:3074-3081）。
- 勉强处：PrepConnections 的 static DeltaTimeOverflow（NetDriver.cpp:5207 FIXME）；GetMaxTickRate 只认 GameNetDriver 语义（GameEngine.cpp:1740-1746）；GEngine 单例如 GlobalNetObjectCount、stat 累加器全是进程级。
- 未做处：没有 per-world 的会话/入口隔离；Iris 的多 ReplicationSystem（默认内联 8 个）是唯一显式多实例化的复制层。

## 对目标环境的迁移含义

目标引擎「Authority/Replica 两个世界角色 + 发布 id 定进程」可直接采用 UE 的两点：①Role 是**数据的属性而非进程的属性**（per-entity role 让同一代码跑三种形态）；②服务器代码的编译期剥离边界清晰（WITH_SERVER_CODE 的函数体级门控优于文件级）。应抛弃：进程级单例假设（GEngine/全局 stat）——目标引擎一进程多世界是明确诉求，UE 的 static 陷阱清单（NetDriver.cpp:5207 等）就是反面清单。
