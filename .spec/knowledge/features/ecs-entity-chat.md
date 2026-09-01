---
name: ecs-entity-chat
description: ECS 正式实体与聊天垂直切片的需求真值——行为与归属边界;实现该切片或改其验收标准前查
metadata:
  type: doc
  status: 设计中
---

# ECS Formal Entity and Chat Vertical Slice Requirements

> Status: planning source
>
> This document is the product-facing requirement source for the next formal ECS slice. It describes behavior and ownership; exact generated wire schemas and ABI layouts are separate contract deliverables.

## 1. Objective

Deliver the first formal ECS vertical slice after Hello World:

```text
username/password login
  -> central Account Server AccountEntity
  -> Game Server Room admission
  -> 100 BotEntity + 1 PlayerEntity in one Room
  -> generic Entity binding/query
  -> ChatComponent.SetMessage
  -> ordered ChatMessageEvent broadcast
  -> Browser chat window
```

The main acceptance case contains 101 Game ECS Entities: 100 Bot clients plus one Browser client.

## 2. Domain Model

### Account Server

- One central Account Server owns account identity and account profile data.
- `AccountId` is the stable persistent business identity.
- `AccountEntity` is the Account Server's long-lived ECS account object. Login loads or creates it; logout ends only the session.
- Login-or-register accepts a username and password. If the username is absent, the Account Server creates the account and its AccountEntity during the request. An existing username with a wrong password is rejected and never overwritten.
- Login-or-register permits names matching `Bot` plus decimal digits only when the request carries the authenticated Bot-tool registration context. Normal Browser/client registration cannot create or claim this namespace.
- The Hello World test profile uses default password `123456`; password material is never returned to clients or Game Server.
- Successful login returns `AccountId`, the account profile needed by the client, and an opaque Game Server admission credential.

### Game Server and Room

- Game Server hosts multiple isolated `RoomId` / GameWorld instances. The main acceptance uses one Room; additional Rooms receive an isolation smoke test.
- Game Server accepts an admission credential from Account Server, not a username/password directly.
- A connection may be active in only one Room at a time.
- The authenticated login name determines game entity kind. Names matching `Bot` followed by decimal digits create `BotEntity` when admitted with the Bot-tool context; other normal client names create `PlayerEntity`. The client does not submit an arbitrary entity-kind field.
- `PlayerEntity` and `BotEntity` may carry `AccountId` as an identity attribute, never an AccountEntity object reference.

### Entity Identity and Query

- `NetEntityId` is the opaque, never-reused Game Entity reference. It is the game-layer address for client commands, server systems and ReplicaWorld mappings.
- `LocalEntityId` and World-internal generation remain implementation handles and are not public game identities.
- After admission, the server maintains a binding containing `AccountId`, `RoomId`, `NetEntityId`, entity kind and connection generation.
- The client can resolve its own bound `NetEntityId`. The server can resolve an admitted connection or a `NetEntityId` to the authoritative entity in its Room.
- Server systems query authoritative ECS attributes by `NetEntityId` on the Simulation Owner Thread. Client systems query their local `ReplicaWorld` by `NetEntityId`.
- Attribute queries use generated stable `AttributeId` values, not SQL, arbitrary property names or direct storage access.
- Each Attribute independently declares persistence, replication and visibility. Clients can query only replicated and visible fields; server-only and persist-only fields are not exposed.
- Query results include observed revision/Tick and explicit non-existent, stale, invisible or unauthorized outcomes. Tombstoned references never resolve to a replacement entity.

## 3. Chat Behavior

### Input and Server Application

- The gameplay-level `ChatInput` payload contains message text only. It contains no client input sequence and no client frame number.
- Transport/session sequencing, connection-generation validation and duplicate handling remain protocol-layer responsibilities.
- Text received by the server enters the bounded input path and is applied on the next fixed Simulation Tick through `IngressCapture` and the ECS command/commit path.
- `ChatComponent.SetMessage` runs on the Simulation Owner Thread, updates the last-message state, and emits the authoritative event in the same committed Tick.

### Component State

- `ChatComponent` is a normal ECS component and does not perform network or file I/O.
- Its ECS field declarations mark `LastMessageText` and `LastMessageTick/Frame` as authoritative persist-only state.
- These fields use the existing ECS Snapshot/Restore path. They are not a parallel Chat persistence subsystem and are not a client property-sync stream.
- Chat history, event replay history and client chat-window contents are not server-persisted.

### Event and Presentation

- The first channel is the current Room public channel.
- The server obtains the sender through the common connection-to-Entity binding; Chat does not implement a private sender-derivation map.
- `ChatMessageEvent` is a live server event carrying a server-generated `MessageId`, strict Room chat order, `senderNetEntityId`, text and authoritative `appliedTick`. Internal audit correlation may retain AccountId; public clients are not required to receive it.
- Delivery to Room members is reliable and ordered. Clients suppress duplicate events by the server event identity/order and append accepted events to their own chat window.
- Browser renders received events in its chat window. The Browser does not query the server database to render chat.

## 4. Reconnect and Lifecycle

- On disconnect, the server keeps the Game Entity and continues normal Room simulation. Only that account's input is rejected.
- The disconnected entity remains Room-visible with an explicit disconnected state until the retention deadline.
- The retention window is five minutes measured by the process-local monotonic Host clock; the logical Tick is recorded for audit but does not define expiry.
- Reconnect is a fresh login and full handshake. During the five-minute window it rebinds the retained server entity A; it does not create a second Game Entity.
- The reconnecting client discards its old `ReplicaWorld`, receives a complete authoritative snapshot, rebuilds the new `ReplicaWorld`, clears its local chat window and then re-enables input.
- The server does not roll back or rebuild the Room, and no Chat event history is replayed to the reconnecting client.
- After expiry, A is destroyed and tombstoned according to the entity-identity contract. A later login creates a new runtime entity B while retaining the same AccountId.
- A process restart does not preserve old connection bindings or the five-minute session window. Recovered clients perform a normal new login; Room recovery follows the existing Snapshot/Restore contract.

## 5. Timer and Persistence Boundaries

- Native Timer Manager is a shared Server/Client infrastructure. The first slice supports fixed Tick/Frame timers, one-shot and repeating timers, cancellation, scope/generation checks and controlled `CallbackSlot` callbacks.
- The reconnect five-minute deadline is Host monotonic time and is separate from gameplay Timer Manager Tick/Frame scheduling.
- ECS persistence uses the existing canonical Snapshot/Restore and WAL/Command Log architecture. Chat history is excluded; only declared ChatComponent fields may be restored.

## 6. Acceptance Scenarios

1. **Account login-or-register**: submit `Bot01`/`123456` through Account Server; first request creates one AccountEntity and stable AccountId, repeated request loads the same account, and a wrong password is rejected.
2. **Bot launch**: a Bot tool loops `Bot01` through `Bot100`, logs each account in, obtains admission credentials and enters the same Room. The server creates exactly 100 BotEntity instances.
3. **Browser admission**: a normal Browser account logs in through Account Server and enters the same Room. The server creates exactly one PlayerEntity, bringing the Room total to 101 Game ECS Entities.
4. **Binding and self lookup**: every admitted connection resolves one current NetEntityId; the server can resolve each NetEntityId back to the matching authoritative entity and AccountId binding.
5. **Attribute query**: server and client query declared attributes by NetEntityId; unauthorized, invisible, stale and tombstoned references return explicit failures and never alias another entity.
6. **Chat path**: a Bot or Browser sends only text; the next authoritative Tick updates that sender's ChatComponent, emits one ordered event, and all permitted Room clients display it.
7. **Chat persistence boundary**: Snapshot/Restore retains each entity's last message text and logical Tick/Frame, while no Chat history or client chat-window contents are restored.
8. **Reconnect**: disconnect one client, reject its input while the Room continues, reconnect within five minutes, rebuild only that client's ReplicaWorld from a full snapshot, clear its chat window and rebind the original Entity A.
9. **Expiry**: let the monotonic retention deadline pass, verify A is destroyed/tombstoned, then log in again and verify a new Entity B with the same AccountId and a different NetEntityId.
10. **Isolation**: create a second Room with a small number of clients and verify Entity bindings, Chat events and queries do not cross Room boundaries.
11. **Scale and determinism**: capture evidence for 101 Game Entities, reliable ordered Chat delivery, fixed-Tick application, reconnect/expiry transitions and repeatable results across two identical runs.

## 7. Requirement Tracks for the New Room

The new Workflow Requirement Room will contain new cards for this slice only; existing module-room requirements remain in place and are referenced as prerequisites.

| Track | Deliverable |
| --- | --- |
| Source and governance | This requirement source, decision log, scope/non-goal control and change traceability |
| Account system | Account Server registry, login-or-register, AccountEntity lifecycle, default test credential profile and admission credential |
| Entity admission | Room admission, Bot-name classification, PlayerEntity/BotEntity creation, one-account/one-Room rule and lifecycle |
| Common identity | Connection binding, self lookup, NetEntityId resolution and generation-safe stale handling |
| Attribute query | Generated AttributeId query surface, revision results, visibility/permission filtering and failure semantics |
| Chat component | ECS component declaration, SetMessage command path, persist-only last-message fields and event production |
| Chat replication | Typed mapping, reliable ordered Room event delivery, duplicate suppression and Browser/Bot consumption |
| ReplicaWorld | Full snapshot admission, entity mapping, client self lookup and visible Attribute Query |
| Reconnect | Fresh login/handshake, five-minute Host deadline, client-only ReplicaWorld rebuild and expiry/tombstone behavior |
| Timer and persistence | Native Timer Manager first slice plus existing ECS Snapshot/Restore integration |
| End-to-end acceptance | 100 Bot + 1 Browser scenario, isolation smoke, failure cases, repeatability and evidence |

## 8. Explicit Non-Goals

- No Chat history, search, moderation, private channels or offline message delivery.
- No client-selected sender EntityId or client-selected EntityType.
- No cross-server transfer, account migration or multi-Room simultaneous control by one account.
- No AOI optimization in the first full-visibility Room slice beyond enforcing the existing visibility boundary.
- No production password policy, external identity provider or production Bot-tool authorization policy; the default `123456` profile and Bot-tool registration claim are confined to this controlled test slice.
- No changes to archived `RM-00010`, no new milestone, and no immediate code implementation in the planning deliverable.

## 9. Workflow Record

- The new Requirement Room is `RM-00011` (`ECS Formal Entity and Chat Vertical Slice`).
- The 11 online Requirements are `R-00344` through `R-00354`; they are all assigned to `RM-00011`.
- The 39 acceptance items were created and read back successfully using the project's active `需求验收` type and its `未提交` initial status.
- The 23 planned direct Requirement reference edges remain in the local upload checkpoint. The platform `bindRequirementReference` endpoint returned HTTP 500 during two independent attempts, so no non-contract relation API was used and the Room/card writes were kept intact.
