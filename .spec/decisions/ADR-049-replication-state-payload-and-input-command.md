# ADR-049: Replication State Payload and InputCommand Carriage

- **Status**: Accepted (2026-09-01; D-009 unblocked for the RM-00011 slice by the Room Review Rulings 2026-09-01 — `../reviews/2026-09-01-ecs-formal-entity-chat-decision-log.md`. Delivered as a pre-launch living-architecture wire contract, not a baseline event; see §Compatibility.)
- **Owner**: `LumioGameEngineArchitecture` (contract truth), `LumioGameRuntime` (replication semantics and mapping consumption), `LumioServer` / `LumioClient` (wire adapters), `LumioGame` (domain mapping declarations)
- **Relation**: fills the two holes [ADR-045](ADR-045-replication-body-closure.md) §4 deliberately left open ("no world-state payload is frozen here"; "client-to-server gameplay command carriage is likewise not decided"). Encodes payload bytes under [ADR-047](ADR-047-lumio-bin-canonical-profile.md)'s `LumioBinV1`, which ADR-047 §Compatibility already named as this payload's decided encoding. Refines [ADR-028](ADR-028-replication-typed-bodies.md) and, through it, [ADR-005](ADR-005-replication-prediction.md); the Accepted text of both is unchanged. Digest framing follows [ADR-041](ADR-041-canonical-digest-profiles.md) §2. [ADR-052](ADR-052-ms00002-hello-wire-and-clr-host-abi.md) recorded that this ADR's original V1.5 baseline-batch delivery route was not adopted; §Migration records how this Accepted text supersedes that route without reopening the decision.

## 背景（Context）

ADR-028 separated the replication envelope from a typed `body`. ADR-045 closed that body per MessageType and made explicit what had previously been reachable by accident: **there is nowhere to put world state, and nowhere to put a client input.** Two consequences were measured, not predicted: `A1-β` was unmeetable because a conforming `FullSnapshot`/`Delta` carried no world state (`LumioServer` hit the wall and correctly refused a private body member), and client-to-server gameplay commands had no carriage at all after ADR-045 §1 closed the `Ack`-smuggling route on purpose.

The direction was adjudicated on 2026-08-29 (2026-08-29 契约面裁决（旧制度产物，已随 docs/ 删除，见 git 历史） §裁决三): *what* to build was settled there. What happened since:

- **2026-08-31** — ADR-052 delivered MS-00002 on a minimal dev-state contract (`engine/wire/hello-wire-v1.json`) and explicitly recorded that "ADR-049 的 V1.5 基线化路线未被采纳执行": the mainline became a pre-launch living architecture, the baseline/Schema/Fixture/mirror governance system was removed, and re-baselining the envelope through a V1.5 batch had no vehicle. That decision concerned the **delivery route only**; the envelope semantics this ADR freezes were never rejected.
- **2026-09-01** — The RM-00011 room review ruled D-009 unblocked for the ECS entity/chat slice: this ADR is to be finalized as the generic gameplay command envelope, with `ChatInput` as the first tenant, paper-validated against a second tenant (voxel-dig) before freezing. The same ruling froze the delivery shape: contracts land as self-contained dev-state wire contract JSON under `engine/wire/` (hello-wire precedent, Owner adjudication 2026-09-01).

This ADR is therefore the first place the envelope's **field-level and failure semantics** are written down in their final, host-agnostic form.

## 决策（Decision）

### 1. 下行状态以声明块进入既有 typed bodies

`FullSnapshot` gains a required `stateBlocks`; `Delta` gains a required `changedBlocks`. Both are arrays of the same block shape. **No parallel downstream envelope is introduced** — sequencing, transport policy, length bound, integrity and session identity remain properties of the messages that already exist for this purpose in the carrying contract.

Each block carries:

| Member | Type | Meaning |
| --- | --- | --- |
| `mappingId` | registered mapping id | which registered mapping this block's bytes belong to |
| `payload` | lowercase hex | the block's bytes, encoded under `LumioBinV1` per the mapping's declared `fieldOrder` |
| `payloadSha256` | sha256-hex | prefix-free SHA-256 of the payload bytes (ADR-047 §2 construction, no domain tag, no length framing) |

`stateBlocks` and `changedBlocks` are **required, and MAY be empty**. An empty array is the defined encoding of "this snapshot/delta carries no state for any mapping" — the same reasoning ADR-045 §2 used to refuse a sentinel for the empty mapping set: an empty array runs the same rules and yields a defined value, whereas an omitted member reopens the "missing means what?" ambiguity ADR-028 closed.

### 2. 块序即映射注册表声明序，且机器可查

The blocks in one array appear in **code-point-ascending `mappingId` order**. Ascending also forbids a repeated `mappingId`, which would otherwise make "which block wins" an implementation choice. Two conforming encoders that produce different bytes for the same state are a fatal contract violation, not a tolerable variance — the position ADR-035 fixed for `chunkOrder` and ADR-047 fixed for struct declaration order, applied to block arrays.

### 3. `payloadSha256` 绑定字节，且校验器重算

A published digest a gate does not recompute rots into a lie. `eng/verify-wire.mjs` **decodes `payload` and recomputes the digest**, and rejects **before** the payload is interpreted, so a divergent encoder fails at admission rather than corrupting state. `payloadSha256` covers the payload bytes only; message-level integrity, where the carrying transport defines one, stays in the transport. The two do not overlap and neither substitutes for the other.

### 4. 上行输入是 InputCommand 信封，不带 gameplay 序号/帧/tick，也不带发送者

Client-to-server gameplay input travels as an `InputCommand` message carrying a `commands` array of `CommandBlock`s under the same §1–§3 discipline. Two properties are **final rulings, not omissions**:

- **No client gameplay sequence, frame or tick.** The Room Review decision log (2026-09-01): "`ChatInput` carries message text only at the gameplay level. The client does not supply a gameplay input sequence or frame number. Transport/session sequencing and duplicate handling remain protocol-layer concerns." The Draft of this ADR carried `commandSequence`, `tickId` and `predictionKey`; the finalization **removes** them. A future tenant that needs client prediction forces its own ADR; none is designed here (no prediction/rollback design is a boundary of the freeze card).
- **No sender field.** The sender is resolved server-side from the connection's common binding (`AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration`, frozen by the binding-and-query contract). A client cannot choose an arbitrary sender entity, so the envelope carries nothing to choose.

### 5. Chat 映射三件与有界输入

The first tenant mapping set, frozen in `engine/wire/gameplay-command-envelope-v1.json`:

- **`chat.input`** (kind `command`, c2s): field `text` only, `maxUtf8Bytes = 512`.
- **`chat.event`** (kind `event`, s2c, delivery `delta-live-only`): `messageId`, `roomSequence` (strictly increasing per Room), `senderNetEntityId`, `text`, `appliedTick` — all server-stamped. Live notification only: it appears exclusively in `Delta.changedBlocks`, never in `FullSnapshot.stateBlocks` (reconnect does not replay chat events; the client chat window is rebuilt empty), and the server does not persist chat history.
- **`chat.component`** (kind `componentState`, direction `none`): `lastMessageText`, `lastMessageTick`, dimensions `persist-only / replication:none / visibility:server-only`. It participates in the existing ECS field-attribute persistence flow (component-level snapshot/restore) and never appears in any wire block array — it is not a client property-sync stream.

Bounded input is frozen to a **single behavior: reject** — `chat_text_too_long` for the text cap, `chat_rate_exceeded` for more than one `chat.input` per sender per authoritative tick. Reject was chosen over drop because explicit rejection keeps the 100-Bot acceptance deterministic: a sender can always distinguish "rejected" from "accepted", whereas silent drops would make cadence verification ambiguous. The rate rule is receiver-enforced (it needs tick state the validator does not have); the text cap is validator-enforced on the decoded bytes.

### 6. 通用性纸面套验：voxel-dig 第二租户

Before freezing, the envelope was paper-validated against a voxel-dig command — the worked example, in full:

A `voxel.dig` tenant mapping would be declared as `fieldOrder: ["x","y","z"]`, `fields: { x: u32, y: u32, z: u32 }`, kind `command`. The command block for digging block (10, 64, 3) is exactly 12 bytes — `0a000000 40000000 03000000` — under the same §1–§3 rules (registered id, ascending order alongside other commands, digest recomputed over those 12 bytes). The downstream acknowledgement would be a kind `state` mapping `voxel.chunk-delta` with its own declared layout; it rides `changedBlocks` with zero envelope change.

What the example proves: adding a tenant requires **one mapping declaration** and no envelope, block, digest or ordering change; fixed-width integer-only tenants need no new wire types; the block-kind rules (§5's delta-only/no-replay partition) generalize by kind, not by tenant name. What it does not prove, and does not claim: float/variable-layout tenants (an ADR-047 refused kind) and prediction-bearing tenants (§4) remain outside until their own ADRs.

## 接口与 Schema（Contract）

The single truth is `engine/wire/gameplay-command-envelope-v1.json` (`lumio.gameplay-envelope.v1`): messages (`InputCommand`, `FullSnapshot`, `Delta`, `Error`), shared block types, the `mappings` registry (which **is** the mapping-id namespace for this contract — the role the ID registry played under the removed governance system), per-mapping `dimensions` (persistence / replication / visibility), `boundedInput`, `errorCodes`, `limits`, the `rules` table with `enforcedBy: validator|receiver`, and embedded `testCases` / `invalidCases`.

The gate is `node eng/verify-wire.mjs`: it auto-discovers every `engine/wire/*.json`, enforces structural grammar, reference integrity (case codes and rule ids resolve), block semantics (digest recompute, LumioBinV1 decode/re-encode equality, per-field caps, block-kind rules, ascending-unique order), and executes the embedded cases — validator-checkable cases must fail with their declared code; receiver-enforced cases are verified for declaration completeness. `hello-wire-v1.json` passes the structural layer unchanged; `eng/verify-hello-wire.mjs` remains the deep validator for that contract and is untouched.

Downstream consumption (Owner adjudication 2026-09-01): implementation repos hand-write their typed surface against the JSON as field truth and carry contract-conformance tests; no generation pipeline is rebuilt.

## 失败语义（Failure semantics）

A `FullSnapshot` without `stateBlocks`, or a `Delta` without `changedBlocks`, is invalid — the member is required, and an empty array is how "nothing to send" is spelled. A block whose `payloadSha256` does not recompute from its `payload` is rejected **before** the payload is interpreted (`bad_payload_hash`). A block list that is not strictly ascending by `mappingId`, or that repeats a `mappingId`, is invalid (`block_order_violation`). A payload that does not decode as canonical `LumioBinV1` for its mapping yields no state and no partial application (`undecodable_payload`): an undecodable byte string yields no value — never a truncated, padded or reordered read. An `InputCommand` block whose `mappingId` is unregistered or not kind `command` is `unknown_command_type`; a `chat.event` block in `FullSnapshot.stateBlocks`, or any block of the wrong kind for its array, is `state_block_kind_mismatch`. Oversized chat text is `chat_text_too_long` at admission; a second `chat.input` from the same sender within one authoritative tick is `chat_rate_exceeded` at the commit face. `chat.event.roomSequence` regressions (duplicate or decrease within a Room) are `bad_envelope` at the receiver.

None of these by itself requests a resync: a malformed message proves nothing about baseline continuity. Gap and resync semantics are unchanged by this ADR.

## 替代方案（Alternatives）

- **A single opaque `payload` blob per message, no per-mapping blocks** — rejected. The gate could check that bytes exist but never that they mean the same thing on both ends, and a per-mapping digest would be impossible (ADR-028's free-form-payload defect one level down).
- **A separate downstream state envelope (a `StateSnapshot` message)** — rejected. Sequencing, transport policy, length bound and session identity would be duplicated, and a duplicated property is one that can disagree. Extending the existing messages is smaller and more checkable.
- **Carrying gameplay sequence / frame / prediction fields on the input envelope** — rejected by ruling (§4): protocol-layer sequencing stays protocol-layer; no prediction design exists in this slice to hang them on.
- **Drop instead of reject for bounded input** — rejected (§5): silent drops make the 100-Bot cadence acceptance unverifiable from the sender side.
- **JSON-native bodies without LumioBinV1** — rejected. ADR-047 already decided this payload's encoding; canonical-JSON key ordering would be a second, weaker byte discipline.
- **V1.5 baseline-batch delivery** (the Draft's route) — superseded by the living-architecture mainline (ADR-052's record); this Accepted text delivers the same semantics through the dev-state wire-contract vehicle. See §Migration.

## 兼容影响（Compatibility）

Pre-launch living architecture: breaking changes carry no compatibility window, and there is no deployed consumer of the formal replication path to migrate (the position ADR-028 recorded when it broke the envelope shape in V1.3). `hello-wire-v1` (MS-00002) is untouched and remains independently valid; this contract neither extends it nor reuses its message set — the two coexist until the hello milestone retires. On the JSON wire, `u64` is carried as a number bounded to 2⁵³−1; hosts using true u64 internally must bound-check at their adapter.

## 迁移方案（Migration）

From the Draft: the semantics of §1–§3 are unchanged from the Draft's decision; §4 is narrowed by the 2026-09-01 ruling (no client sequence/tick/prediction members) and §5–§6 are new (chat tenant freeze, bounded input, paper validation). The Draft's `MessageType`-registry registration, `schemas/*` files and `tools/lumio_contract.py` rules are obsolete with the removed governance system; their function lives in the contract's `mappings` registry and `eng/verify-wire.mjs`. ADR-052's record that "the V1.5 route was not adopted" stands; this text is the adopted replacement route, so no new ADR is needed to supersede it. If the project later re-hardens into a baseline system, this contract upgrades per the governance decision of that time; the frozen semantics (block shape, ordering, digest rule, kind partition, bounded input) carry over unchanged.

## 验证（Verification）

Embedded in the contract and executed by the gate (`node eng/verify-wire.mjs`):

- Positive cases: `input/chat-single-command` (one well-formed `chat.input` block), `snapshot/empty-state-blocks` (empty array pins the no-sentinel rule), `delta/chat-event` (one well-formed `chat.event`), `error/rate-exceeded` (rejection receipt carries the mapping id).
- Negative cases, one per rule clause, each rejected with the declared code: `input/text-too-long` (513-byte text), `input/digest-mismatch`, `input/unknown-command-type` (`voxel.dig` — registered nowhere, deliberately), `input/duplicate-mapping`, `input/undecodable-payload` (length prefix runs past the bytes), `snapshot/event-replay` (delta-only event inside a snapshot), `delta/component-state-on-wire` (persist-only mapping on the wire).
- Receiver-side probes (declaration-checked, runtime-enforced): `runtime/chat-rate-second-per-tick`, `runtime/chat-room-sequence-regression`.

Acceptance bar (对照组探针纪律, carried from the Draft): a deliberately broken probe contract placed in `engine/wire/` must turn the gate red with clean rejections, and its removal must restore green — only a probe that goes red and then green proves the guard works. The executing session's evidence: probe run rejected with exit 1 (duplicate error codes, malformed contractId, bad dir, unresolvable case codes), removal restored `verify-wire: all contracts green` with `hello-wire-v1.json` passing unchanged and `eng/verify-hello-wire.mjs` 9/9. `node eng/generate-abi.mjs` zero-diff; `node .spec/tools/spec-lint.mjs` OK.

## 修订记录（2026-09-02，ADR-056 §5–§6）

本段为 Accepted 正文的附录，不改写上方决策原文。ADR-056 §5–§6 将 Room 路径快照、事件广播验收与顶号通知修订如下，契约真值仍是 `engine/wire/gameplay-command-envelope-v1.json`（C-1′）：

- `FullSnapshot.stateBlocks` 是 Room 路径唯一快照载体。ADR-045 五字段闭合体（`snapshotId` / `tickId` / `sessionRevisionVector` / `schemaEpoch` / `mappingSetHash`）不是本契约的 FullSnapshot；缺 `stateBlocks` 或呈 ADR-045 形状以 `bad_envelope` 拒绝。`stateBlocks` 必须含该 Room 每个活体实体的已复制状态块；空数组仍只表示「本 Room 无可复制状态」。
- 新增 s2c `ConnectionSuperseded`：`messageType=ConnectionSuperseded`、`reasonCode=connection_superseded`、`netEntityId:u64`、`newConnectionGeneration:u64`。语义：旧连接必须先收到本消息，服务器随后再关闭；先关后发以 `session_closed` 拒绝。错误码词表不因此扩展。
- `mappings.*.dimensions` 改为生成物：`source: "generated-from-field-annotations"`，钉 N-04 声明表拷贝与 sha256 `a47e92d663ba8f9726cf8defdacf2f56ebbaf1b93a8be9b7435430fad48bddc0`。`chat.component` 三维必须与 `ChatComponent` 字段标注生成结果一致（`persistent` / `not-replicated` / `server-only`）。
- `chat.event` 验收以客户端实际收到的 `Delta.changedBlocks` 为准；harness 不得由发送计数合成 `eventOrder` / `appliedTicks` / `restoredWindow`。

## 修订记录（2026-09-03，ADR-058）

本段为 Accepted 正文的附录，不改写上方决策原文与前两条修订记录。ADR-058 将 C-1 修订如下，契约真值仍是 `engine/wire/gameplay-command-envelope-v1.json`：

- `roomSequence` 语义改为世界内严格递增序号（字段名不变）；同 Tick 内按发送者 NetEntityId 排序后分配。
- `entity.identity` 普查块升为创建记录（EntityType + NetEntityId + 该观察者可见 Sync 字段当前值）；FullSnapshot 与 Delta 用同一种记录，创建优先。
- 新增 InputCommand 种类 `field.write`（`Authority.Owner` 字段上行；正用例 `input/field-write-owner-name`，写别人实体或 Server 权威字段 → `unauthorized`）。
- `chat.event` 的 `senderNetEntityId` 在 LumioBinV1 上编码为 `senderNetEntityIdInstanceId` + `senderNetEntityIdCounter`（两个 u64 LE，16 字节）；与 C-2 32-hex 是同一 128 位值。不新增 ADR-047 `u128` 原语。
- `chat.input` / `chat.event` 分别是 `ChatComponent.SendMessage` ServerRpc 与 `OnChatMessage` ClientRpc 的线上形态；事件 delta-live-only，服务器不保留历史。

## 修订记录（2026-09-02，R-00368 r2 / C-1′ entity.identity）

本段为 Accepted 正文的附录，不改写上方决策原文与前一条修订记录。Room 路径 FullSnapshot 的活体身份普查租户是 `entity.identity`（kind=`state`，direction=`s2c`）：payload 为 LumioBinV1 数组（`u32` 元素个数 + 文档序记录），记录字段 `netEntityId` / `entityType`（仅 `player`|`bot`）/`unmappedMark`，按 `netEntityId` 严格升序；`EntityIdentity.claimedMark` 不上本块。空 `stateBlocks: []` 仍只表示本 Room 无房间可见可复制状态；有活体时必须出现本块，零活体时省略。不新增错误码、不增加第二条 kind=state 映射。
