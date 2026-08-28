# TransportProfile — WebSocket 档 Capability 登记与覆盖度结论

- **BaselineId**：`LGE-V1.4-2026-08-27`（本文不改基线）
- **登记 id**：`LGE-V1.4-TRANSPORT-WS-2026-08-28`
- **日期**：2026-08-28
- **来源卡**：Workflow `R-00258`（MVP A1 前置探明）
- **依据**：架构正文 §7.3「Wire 与 Transport」、§10「Host Profile」、[`DECISIONS_PENDING.md`](DECISIONS_PENDING.md) D-004；MVP 立项见 [`../plans/mvp-browser-voxel-multiplayer.md`](../plans/mvp-browser-voxel-multiplayer.md) §4 轨道 A1

## 0. 结论（一句话）

**WebSocket 档不需要任何公共契约变更。** 现有 Envelope、`transportPolicy`、MessageType 集合、ErrorCode 命名空间与 Host Capability 面已经完整覆盖 MVP A1（`LocalSplitProcess`，WSS）所需的公共语义；本次只做「登记」——落一份 Host Capability 记录 + 一条补齐既有规则覆盖的失败 Fixture，并列出 A1 可依赖的字段与错误码清单。

**D-009（RPC/Message dispatch）与 D-011（Auth wire）未被触碰**，仍是「Not frozen / 有意阻塞」，见 §5。

## 1. 为什么 WebSocket 是「换传输」而不是「改契约」

架构正文 §7.3 的规则是「**可换传输，不可绕业务协议**」：Envelope、Serializer、权限校验、大小限制和有界队列必须复用同一套，传输层只允许换掉 Socket/TLS/OS 网络栈。`DECISIONS_PENDING` D-004（Transport/Codec/压缩选型）把这件事写死为：

> Adapter-only choice does not change baseline; envelope/codec changes do.

WSS 提供的是「全可靠、有序、带帧」的字节通道 —— 它落在 `reliability: "Reliable"` 这一档，不引入新的可靠性语义、不引入新的消息类型、不改变 Envelope 任何字段的含义。因此它是 D-004 意义上的 adapter 级选择。

**本文不确认 D-004。** 选哪个具体的 WebSocket 实现库（含许可证、AOT、WASM 可用性证据）仍是 D-004 未决项，归 `LumioServer` / `LumioClient` 在 MVP A1 落地时按选型门评审。本文只确认「WebSocket 这一档能用现有公共面表达」。

## 2. 覆盖度逐项核对

| # | WebSocket 档需要什么 | 现有公共面 | 结论 |
| --- | --- | --- | --- |
| 1 | 可靠有序投递语义 | `replication-envelope.reliability` 枚举 `Reliable`\|`Unreliable`；语义规则「FullSnapshot must use Reliable delivery」 | ✅ 已覆盖，WSS 全程用 `Reliable` |
| 2 | 单条消息大小上限 | `transportPolicy.maxMessageBytes`（1..1048576） | ✅ 已覆盖 |
| 3 | 分片上限 | `transportPolicy.maxFragmentBytes`（1..65536）——对应 WS 帧分片 | ✅ 已覆盖 |
| 4 | 连接级反重放窗口 | `transportPolicy.antiReplayWindow`；归属由 `protocol-permission-gate.antiReplay.connectionScopeOwner = ConnectionLayer` 冻结 | ✅ 已覆盖 |
| 5 | 认证绑定方式 | `transportPolicy.authBinding` 枚举 `SessionAdmission`\|`ConnectionGeneration`；MVP auth 存根在 Handshake 授予准入 → `SessionAdmission` | ✅ 已覆盖 |
| 6 | 错误分级 | `transportPolicy.errorClass` 枚举 `Retryable`\|`Rejectable`\|`Fatal`；`Error` body 同名字段 | ✅ 已覆盖 |
| 7 | 完整性校验 | `integrity.algorithm` 枚举 `None`\|`CRC32C`\|`SHA256`\|`AEAD`，各自带值格式约束 | ✅ 已覆盖（WSS 下 `None` 亦合法，TLS 已保完整性） |
| 8 | 消息类型集合 | 8 个 MessageType，三方一致（Schema 枚举 / ID Registry / Fixture 实际使用集合），由 `message_type_consistency_errors` 机械保证 | ✅ 已覆盖，无需新增类型 |
| 9 | 断线重连 | §7.3：新连接代次必须重做通道认证 + 完整 Handshake；连内 Resync 不重握手（D-012 明确 V1 无 Resume Token） | ✅ 已覆盖，无需新增字段 |
| 10 | 权限门字段集 | ADR-022 冻结的 6 项（SessionId / Product+Release / MessageId / Role / Claims / ConnectionGeneration） | ✅ 已覆盖 |
| 11 | Host 侧能力声明 | `host-capability.schema.json`：`capabilities` / `requiredCapabilities` 是自由 `id` 数组（沿用既有 `InMemoryTransport` 命名法） | ✅ 已覆盖，登记 `WebSocketTransport` 即可，无 Schema 改动 |
| 12 | 超限消息的专属错误码 | ID Registry 无 `MessageTooLarge` 一类的码 | ⚠️ 见 §4 观察项（**不阻塞 A1**） |

**Schema / ID Registry / Fixture 三者都不需要改动**（§4 的观察项除外，且它当前不构成 A1 阻塞）。

## 3. A1 可依赖的公共字段与错误码清单

以下是 `LumioServer`（transport / auth 存根 / session / world-slot）与 `LumioClient`（connection / handshake / bot）在 A1 联调中**可以直接依赖**的公共面。清单外的一切都不是公共契约。

### 3.1 Envelope 必填字段（`replication-envelope.schema.json`）

`protocolVersion`、`length`、`sequence`、`sessionId`、`productId`、`gameReleaseId`、`messageType`、`reliability`、`integrity{algorithm,value}`、`traceId`、`transportPolicy`、`body`。Envelope `additionalProperties: false` —— 传输适配器**不得**往 Envelope 里塞 WebSocket 专属字段。

### 3.2 `transportPolicy` 必填五项

| 字段 | A1 取值口径 |
| --- | --- |
| `maxMessageBytes` | 部署侧配置，上限 1048576 |
| `maxFragmentBytes` | 部署侧配置，上限 65536 |
| `antiReplayWindow` | ≥ 1，连接层拥有 |
| `authBinding` | `SessionAdmission`（MVP auth 存根） |
| `errorClass` | `Retryable` / `Rejectable` / `Fatal` |

### 3.3 MessageType 与 body 必填字段

| MessageType | numeric | body 必填 |
| --- | --- | --- |
| `Handshake` | 1 | `role` |
| `FullSnapshot` | 2 | `snapshotId`、`tickId`、完整 `sessionRevisionVector`、`schemaEpoch`、`mappingSetHash`（且必须 `Reliable`） |
| `Delta` | 3 | `baseSnapshotId`、`fromRevision`、`toRevision`、`mappingSetHash`、`confirmationSequence`、`tombstones` |
| `ResyncRequest` | 4 | `resyncReason` |
| `MaintenanceKick` | 5 | `reasonCode` |
| `BaselineAck` | 6 | `snapshotId`、`confirmedRevision` |
| `DeltaAck` | 7 | `confirmationSequence`、`toRevision` |
| `Error` | 8 | `errorClass`、`reasonCode` |

### 3.4 A1 会用到的稳定 ErrorCode（`ids/index.json` · ErrorCode 命名空间）

| id | numeric | A1 触发场景 |
| --- | --- | --- |
| `MaintenanceKick` | 1002 | 维护踢出 |
| `ReleaseMismatch` | 1003 | Product/Release 不一致（D-007：精确匹配，无兼容窗口） |
| `MessagePermissionDenied` | 1031 | 权限门拒绝 |
| `StaleConnectionGeneration` | 1032 | 旧连接代次的消息 |
| `BudgetExceeded` | 1035 | 预算类拒绝 |
| `QueueFull` | 1036 | 有界队列背压 |
| `SessionMismatch` | 1040 | SessionId 不一致 |
| `RoleMismatch` | 1041 | Role 不符 |
| `ClaimNotGranted` | 1042 | Claim 未在准入时授予 |
| `SessionAntiReplay` | 1043 | 会话级反重放命中 |

其中 1040–1043 与 1003 / 1031 / 1032 同时是 `protocol-permission-gate.rejectReason` 的合法取值，且由 `vocabulary_consistency_errors` 机械保证 ⊆ ErrorCode 命名空间。

### 3.5 Host Capability 登记

新增正向 Fixture `host/local-split-process`（`fixtures/valid/host-capability-local-split-process.json`）：`preset = LocalSplitProcess`、`roomMode = Online`、`roles = [Client]`、`capabilities` 含 `WebSocketTransport`、`requiredCapabilities = [WebSocketTransport]`。

命名沿用既有 `InMemoryTransport` 的 `<Kind>Transport` 约定。传输类 Capability **不进 ID Registry 的 `Capability` 命名空间**——该命名空间当前登记的是 CoreEngine 包能力（`Native`、`HybridCLR`、`ReferenceVoxel`、`Voxel*`），而 `host-capability` / `voxel-world-port` / `core-engine-manifest` 的 capability 数组在 Schema 上是自由 `id`，既有 `InMemoryTransport`、`DeterministicClock`、`Renderer` 都不在该命名空间内。本次沿用现状，不擅自扩大 ID Registry 的语义。

## 4. 观察项（不阻塞 A1，交架构所有者判断）

**超出 `maxMessageBytes` / `maxFragmentBytes` 时没有专属稳定错误码。** `transportPolicy` 声明了上限，但 ErrorCode 命名空间里没有 `MessageTooLarge` 一类的码。

- **A1 的临时口径**：连接层按 `errorClass = Rejectable` 拒绝，`Error.reasonCode` 用 `BudgetExceeded`（1035）；这与 VOX-D-003 已确认的「满载动作 = 生成的 `QueueFull`/`BudgetExceeded`」口径一致。
- **不在本卡新增错误码**：A1 是否真的需要把「超长」与其他 Rejectable 区分开，要等联调数据；提前占号会把一个未验证的需求写进公共 ID Registry。若 A1 证明需要区分，按 ADR → ID Registry → 正反 Fixture 顺序补，属新卡范围。

## 5. 冻结面未触碰声明

- **D-009（RPC/Message dispatch contract）**：未触碰，仍 Not frozen。本文没有引入任何 MessageId 命名空间、RPC Envelope 或生成式 dispatch；A1 的上行输入继续走既有 replication envelope 的 MessageType 面。`packages/index.json` 的 `blocked` 列表保持 `D-009: protocol-dispatch not frozen`。
- **D-011（Auth wire credential/ticket schema）**：未触碰，仍 Not frozen。本文只引用已冻结的**行为**契约（每次 Handshake 在 Session 准入前通过反重放校验）与已有的 `authBinding` 枚举值，没有定义任何凭据/票据线格式。MVP 用 auth 存根，`blocked` 列表保持 `D-011: Auth wire not frozen`。
- **D-012（Session Resume Token）**：未触碰，V1 仍不提供。
- **架构正文 v1.4 未改动**，`docs/architecture/.baseline.sha256` 校验通过。

## 6. 验证

- `fixtures/valid/host-capability-local-split-process.json`（`host/local-split-process`）—— WebSocket 档的 Host Capability 登记记录。
- `fixtures/invalid/replication-unreliable-full-snapshot.json`（`replication/unreliable-full-snapshot`）—— WSS「全可靠有序」这一属性的失败面：`FullSnapshot` 声明 `Unreliable` 必须被拒。该规则（`FullSnapshot must use Reliable delivery`）此前**没有任何 Fixture 覆盖**，本次补上。
- 未新增失败 Fixture 的理由：WebSocket 档没有引入任何新的拒绝规则，为「有正反例」而造一个不对应真实规则的失败样例，等同假 Fixture，本仓明确禁止。
- 收口门槛：`spec-lint` + `spec-lint.test.mjs` + `py_compile` + `lumio_contract.py validate` 全绿；生成物随 `fixtures/valid` 变化重新发布。
