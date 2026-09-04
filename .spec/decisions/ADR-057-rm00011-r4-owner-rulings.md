# ADR-057：RM-00011 r4 Owner 裁决——ADR-056 六项 Fixture 未成立的补救范围

状态：Draft（2026-09-03，Owner 逐条裁决；依据 `reviews/2026-09-03-rm-00011-r3-owner-review.md` 退回结论，P0=0 / P1=9）
取代：无（补充 ADR-056；ADR-056 Accepted 正文不改写，本 ADR 记录其「验证 Fixture」在 r3 交付上未成立的事实与 r4 补救范围）
Owner：`LumioGameEngine`（裁决与契约真值）、`LumioGameRuntime`（ECS / 绑定 / 查询 / 持久化唯一实现）、`LumioNativeCore`（定时内核）、`LumioServer` / `LumioClient` / `LumioGame`（消费方）

## 治理原则

沿用 ADR-056：**第一性原理——如无必要，勿增实体。** 本 ADR 新增一条执行层原则：**验收尺子不由实现方修改。** oracle / Fixture 的定义只能由 Owner 经 ADR 改动；worker 在验收失败时改尺子使其通过，视同「把 not-ok 写成 SUCCESS」。

## 背景

ADR-056 在 2026-09-03 由 R-00377（N-13）转为 Accepted，依据是六项「验证 Fixture」全部 pass。Owner 侧独立深审（六仓 `origin/main` 只读快照 + 三名分仓 reviewer）实测：

| Fixture | N-13 | 实测 | 关键事实 |
|---|---|---|---|
| 1 依赖方向 / 宿主无绑定表 | pass | 不通过 | Client 对 Runtime 零引用；Server `host.rs:154 account_sessions` 第二份按账号索引表并自行裁决顶号/跨房；架构测试只匹配旧字符串名 |
| 2 标注生成 | pass | 部分 | 生成表与契约 sha 一致；但组件注册表仍手写（标注 3 / 注册 2 / 测试 4 字段不一致）；Client 手写声明表与 C-2 不一致 |
| 3 广播 / 两轮一致 | pass | 部分 | Playwright 窗口 101 条有序成立；「两轮一致」被 worker 改为多重集比较后才 pass；Rust 严格 oracle 从未运行；证据包不在任何仓 |
| 4 快照 | pass | 不通过 | 进程 B 只回报 restore 状态码；逐实体比对在进程 A 内存往返且只比一个实体 |
| 5 定时 | pass | 不通过 | 全仓唯一 `pump_wall_clock` 调用点是 harness；生产时钟带可注入偏移后门；Bot 节奏由 Server 仓启动钩子冒充 Client 进程一次 `Advance(15)` 产生 |
| 6 顶号 | pass | 部分 | 服务端先通知后关闭成立；Client 会话层不识别 `ConnectionSuperseded` 并自动重连 |

另发现 ADR-056 §1「单一 ECS」在 Runtime 内部未连通：绑定查询建了两个空 `EcsWorld` 当线程牌子并用私有字典 `_values` 返回 seed 常量；聊天走第三个 `ChatIngressWorld`；三者无数据通道。

## 决策（Owner 逐条裁决，2026-09-03）

1. **ADR-056 保持 Accepted，本 ADR 记录偏差与补救。** Server ADR 0008「冻结条件已满足」的前提随本 ADR 失效，由 Server 仓在 0008 追加「修订记录」指向本 ADR。
2. **「两轮一致」= 顺序一致。** 保持 ADR-056 Fixture 原文。Runtime 同 Tick 内按发送者 `NetEntityId` 排序后分配 `roomSequence`；验收 harness 在同一 Tick 投递全部 100 条 Bot 输入；Game `verify-evidence.mjs` `compareRuns` 改回逐位比较（`eventOrder` 与 `appliedTicks` 逐值）；删除 Server 仓 `tests/verify_rust_evidence.mjs`；Rust acceptance 与 Game oracle 收敛为同一口径的一把尺。
3. **证据 = 日志。** 不再有独立「证据包」。服务器进程与客户端进程各自输出结构化日志，由统一日志系统管理；验收 oracle 只读服务器日志 + 客户端日志，不读 harness 合成的中间文件。日志随每次收口保存到任何人可拉取的位置（默认 LumioGame `integration/entity-chat/logs/<日期-SHA>/` 入库，照 hello 场景既有模式）；oracle 自校验 sha 以行尾归一化后的字节计算。
4. **Bot 客户端由 Client 仓真实实现。** `Lumio.Client.Bot.Host` 生产路径构造 `ClientTimerManager`，`INativeTimerAbi` 的生产实现经 `Lumio.Engine.NativeLoader` 取根表槽（不自写 `LoadLibraryW` / 槽偏移）；Bot 进程常驻、按 Tick 逐条提交 `chat.input`。LumioServer 删除 `modules/process/src/entity_chat/bot_startup_hook/` 与 `bots.rs` 的 `DOTNET_STARTUP_HOOKS` 注入逻辑；验收只拉起 Bot.Host 并读其日志。
5. **在线名单只在 Runtime。** C-2′ 增加「账号已在线 → 可顶号」结局（与 `invalid_binding_shape` 区分）；Runtime `EntityBindingQuery.Admit` 按此返回；Server 删除 `host.rs` `account_sessions` 与顶号 / 跨房裁决、删除三处 `ListBindings` 兜底扫描；宿主只保留「连接 ↔ Runtime 绑定句柄」的会话表；架构测试改为结构断言（宿主 crate 内不允许以账号为键的映射）。
6. **服务器自驱主循环。** `lumio-entity-chat-replay` owner loop 按固定频率 `advance_tick_frame` 并周期 `pump_wall_clock`（真单调钟）；删除 `SystemMonotonicClock::advance_ms` 生产后门（测试用注入 Fake clock）；`schedule_room_tick` 真按内核节拍；S7 进程 B 读回后逐实体比对 101 个 `ChatComponent.lastMessageText`；`expired / historyCountMax / staleARejected / browserWindow / snapshotEntities` 改为日志观测值或删除。
7. **ECS 单一世界（原则已定，结构待 ECS 架构会话定稿）。** 只有一个世界；实体字段只能是带标注的组件字段；绑定 / 查询 / 聊天 / 存档只持有世界句柄，不自建世界。已拍板三条：**存档触发** = 单例 `WorldEntity` 挂 `WorldSaveComponent` 等组件，存档是给 WorldEntity 下一条命令，提交相里的存档系统消费；**读权限** = 同步时按 Visibility 裁剪，客户端本地读不再判定；**身份表入档** = `NetEntityId` 表是世界快照的一部分，恢复后 id 不变、永不复用。Runtime / Client 的重做卡在 ECS 定稿后写。
8. **顶号 = 退到登录界面。** 客户端收到 `ConnectionSuperseded` 后停止输入、断开、回到登录界面，不自动重连；用户再次登录才会顶掉对方。会话层增加该消息种类并置「被顶」态，`ClientSession.HandleDisconnect → StartGeneration` 在被顶态不触发；非 JSON / 非 C-1 形状的 FullSnapshot 一律 `bad_envelope` 拒绝。
9. **timer FFI 只留架构仓。** NativeCore 删除 `crates/lumio-native-ffi/src/timer.rs`（含 `provider_engine_root_api`）与 `lumio-timer/src/adapter.rs` 中旧 `ClientTimerManager / ServerTimerManager` 及其测试；NativeCore 只保留纯 Rust 内核；架构仓 `engine/native/modules/sdk-native/src/timer.rs` 是唯一 C ABI 插头。NativeCore ADR 0008 追加修订记录。
10. **合入闸与流程规则敏捷期暂停。** 分支保护必过项、同卡三次返工阈值、审与盖章分离——本轮不改，记入 `knowledge/lessons.md` 待正式期处理。
11. **P2 / P3 合成一张清理卡。** Server `cfg(windows)` 与 dead-code 修正（Mac / Linux 可编译）；广播错误记录、去 `from_utf8_lossy`、背压不阻塞 owner 线程；旧仓名 `LumioGameEngineArchitecture` 引用改名；Game `scenarios.mjs` 删 mvp-host 路径；架构仓 `eng/dev-run.*` 改为启动 Rust 宿主。

## 替代方案

- **ADR-056 回退 Draft**：被 Owner 否决——Accepted 后只新增取代 / 补充，不改写；本 ADR 承担「此章有误」的说明职责。
- **「两轮一致」改为多重集**：被否决——Tick 内确定性是 ecs.md 的核心承诺，放宽等于放弃验证；且尺子不能由 worker 改。
- **接受 Server 仓启动钩子并迁回 Client**：被否决——「每 N Tick 触发一次」仍不成立，Bot 仍非常驻客户端。
- **允许宿主保留账号表作缓存**：被否决——缓存与真值分叉已实测导致账号永久卡死。

## 接口 / Schema

- C-2′：`engine/wire/entity-binding-and-query-v1.json` Admit 结局 `account_already_online`（`errorCodes.admitOutcomeCodes`），正用例 `admit_second_connection_account_already_online`（第二次准入返回该结局并带现有 `netEntityId`），反用例 `admit_shape_error_is_not_account_already_online`（形状错误仍为 `invalid_binding_shape`）。`roomId` = 宿主路由键；绑定五元组由 `IdentityComponent` 字段 + 宿主会话表拼出；`NetEntityId` 128 位、32-hex 小写。
- ECS 结构性接口由 [ADR-058](ADR-058-ecs-world-manager-and-annotation-registry.md) 定稿（World Manager、WorldEntity、标注 → 注册桥），不在本 ADR 展开。

## 失败语义

- 验收 oracle 若与 Fixture 原文不一致，收口审查直接退回，不看其余。
- 验收证据若不能从仓库拉取并在 Mac / Linux 上复验通过，视同无证据。
- 进程名与实际执行代码归属不一致（如 Server 仓代码顶着 `Lumio.Client.Bot.Host` 进程名）视同假冒，退回。

## 兼容影响

- ADR-056 Accepted 状态保留，但其「验证 Fixture」在 r3 交付上未成立；r4 全部 P1 关闭并由独立深审逐项复核后，本 ADR 转 Accepted，ADR-056 的 Fixture 才视为落地。
- Server ADR 0008 冻结条件失效；C# `mvp-host/` 仍为冻结对照。
- Workflow R-00365…R-00377 的 done 状态不再代表 ADR-056 已落地；r4 卡见 `plans/2026-09-03-rm-00011-r4-blueprint.md`。

## 迁移方案

按 `plans/2026-09-03-rm-00011-r4-blueprint.md`：契约卡先行 → Server / NativeCore / 清理卡并行 → ECS 架构会话定稿 → Runtime 单一世界卡 → Client / Game 卡 → 集成 → 独立深审。

## 验证 Fixture

沿用 ADR-056 六项原文，追加：

- 日志可复核：收口日志目录在仓库内，`node verify-evidence.mjs --dir <logs>` 在 Windows 与 macOS 上均 exit 0。
- 一把尺：Rust acceptance 与 Game oracle 对同一份日志给出相同结论；Server 仓无第二份 oracle。
- Bot 归属：`Lumio.Client.Bot.Host` 进程内执行的定时与发言代码全部来自 LumioClient 仓；Server 仓无 `DOTNET_STARTUP_HOOKS` 注入。
- 自驱：不带任何 harness 的 `lumio-entity-chat-replay` 进程，断线后五分钟内内核回调触发 expire（日志有内核事件）。
## 修订记录（2026-09-04，ADR-060）

The R5-01 contract records the owner ruling: admit returns accepted or rejection only, with Runtime-issued identity and generation delivered by Welcome. Attribute visibility follows generated declarations, TypeOf-derived entityType, counter-derived tombstoned, and claimBy named-list credentials.
