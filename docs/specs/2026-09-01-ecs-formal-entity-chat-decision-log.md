# ECS Formal Entity and Chat Decision Log

> Status: living-draft
>
> This document records confirmed decisions for the formal ECS entity/chat slice. It is intentionally concise: discussion context and implementation narration stay outside this log.

## Confirmed Decisions

### Topology and Identity

- There is one central `Account Server` and one `Game Server` domain with multiple `RoomId` values. Cross-server transfer is out of scope.
- `AccountEntity` belongs to the Account Server. It is loaded or created by stable `AccountId` and survives login/logout as a persistent account identity.
- `AccountEntity` is a long-lived account object: login loads or creates it, logout ends only the session, and Game Server integrations carry `AccountId` rather than a cross-World `AccountEntity` object reference.
- An Account Server is mandatory for this slice; it is not replaced by a Game Server bypass or a fake account map. It must provide an account registry, username/password login, AccountEntity creation/load, and a Game Server admission credential. This Hello World profile uses the configured default test password `123456`; production credential policy is a separate security decision.
- A successful game admission creates either `PlayerEntity` or `BotEntity` in the selected Room. The Game Server classifies an authenticated account whose login name matches the `Bot` plus decimal digits pattern as a bot and creates `BotEntity`; Browser and other normal accounts create `PlayerEntity`. Clients do not submit an arbitrary EntityType field.
- One account may be active in only one Room at a time.
- `AccountId` is the persistent business identity. `NetEntityId` is the opaque runtime/network identity and is never reused after destruction. `LocalEntityId` and World-internal generation are implementation-level handles.
- `PlayerEntity` and `BotEntity` may carry the stable `AccountId` value as an identity attribute, but never a live `AccountEntity` object reference. When runtime Entity A expires and a later login creates B, the AccountEntity/AccountId remains the same while the Game binding changes from A to B.
- A disconnect retains the server entity A for five minutes while the server continues normal Room simulation. Reconnect performs a fresh login/handshake, rebinds A during that window, and rebuilds only the client `ReplicaWorld` from a full snapshot. The server does not roll back or rebuild the Room. The client clears its local chat window. After expiry, A is destroyed and a later login creates B.
- The five-minute window uses the process-local monotonic clock. It does not cross a process restart; recovered clients must log in again.
- A disconnected entity remains room-visible with an explicit disconnected state until expiry. Only that account's input is rejected while disconnected/reconnecting; the server and other Room entities continue normally.

### Room and Bot Slice

- The Room model supports multiple isolated Room Worlds. The main acceptance scenario places 100 independent Bot accounts and one Browser account in one Room; the server creates 100 `BotEntity` instances plus one `PlayerEntity`, for 101 Game ECS Entities. Smaller isolation smoke tests cover additional Rooms.
- The 100 Bot clients authenticate through the same Account Server path as normal clients. The Bot launcher generates the login names; Game Server performs the `Bot` plus decimal digits classification. The ordinary Browser client uses a non-Bot account and receives `PlayerEntity`.
- The Bot launcher creates login names `Bot01` through `Bot100` by looping over a counter and submits each username with the default test password `123456`. Accounts are not pre-provisioned: Account Server performs idempotent login-or-register, creating a missing account and its `AccountEntity`, or authenticating the existing account and returning the same stable `AccountId`.
- Login-or-register never overwrites an existing account: an existing username with a wrong password is rejected, while concurrent first requests for the same username converge on one AccountEntity and one AccountId. Password material is stored/verified through the Account Server's credential mechanism and is never returned to Game Server or clients.
- The `Bot` plus decimal digits login-name namespace is restricted to the Bot tool registration context. Normal Browser/client registration cannot create or claim a Bot-numbered account; Game admission accepts a Bot-numbered account as `BotEntity` only with the corresponding authenticated Bot-tool context.
- A human-readable login name such as `Bot01` is not itself the universal Entity identity. Game admission uses the returned `AccountId`; Game Server derives the entity type from the authenticated login-name rule and creates the corresponding `BotEntity` or `PlayerEntity`.
- The login flow is `Client -> Account Server login-or-register -> AccountId/AccountEntity plus opaque Game admission credential -> Game Server Room admission -> PlayerEntity or BotEntity`. Game Server does not create an account or accept a username/password as a substitute for Account Server admission.

### Common Entity Binding and Attribute Query

- Entity-to-connection binding is a shared runtime capability, not Chat-specific logic. After admission, the server maintains the binding `AccountId + RoomId + NetEntityId + EntityType + ConnectionGeneration` for each active connection.
- A client can resolve its own bound `NetEntityId`; the server can resolve an admitted connection or `NetEntityId` to the authoritative Entity in its Room. `AccountEntity` objects never cross into a Game World as object references.
- Server and client use a controlled ECS Attribute Query surface to read entity properties. The query is not a SQL/database API and does not permit direct storage access.
- Server queries are authoritative, Simulation-Owner-Thread reads scoped to the requested Room/World and server permission. Client queries are limited to the client's `ReplicaWorld`, synchronized fields, visibility/AOI and client permission.
- `NetEntityId` is the runtime reference exposed for entity interaction and mapping. `AccountId` remains the persistent business identity and is not automatically disclosed as a public client attribute.
- Entity resolution and Attribute Query return explicit existence, visibility, permission and stale-generation outcomes; unknown, destroyed or tombstoned entities are never resurrected.

### Entity Interaction Contract

- `NetEntityId` is the single game-layer reference used when a client or server system addresses an Entity. It is not an array index, database primary-key API, or client-generated temporary identifier.
- A client command may reference a target `NetEntityId` when the gameplay contract needs a target. The server validates that the target belongs to the same permitted Room/World, is alive for the current revision, and is visible/authorized for the connection before applying the command.
- Server gameplay systems query authoritative attributes by `NetEntityId` through the ECS query surface on the Simulation Owner Thread. They do not reach into ECS storage from network threads.
- Client gameplay and presentation query the local `ReplicaWorld` by `NetEntityId`. The local result is limited to attributes delivered by replication and permitted by visibility/claims; a client-side query does not fetch server-only or persist-only data.
- Query results carry the Entity reference and the observed revision/Tick so consumers can detect stale reads. Destroyed or tombstoned references return an explicit non-existent/stale result and never resolve to a replacement Entity.
- Attribute queries address generated, stable `AttributeId` values declared by the component schema; arbitrary property-name lookup, SQL expressions and direct database/storage reads are out of scope.
- Each attribute declaration independently states its persistence, replication and visibility dimensions. A client may query only attributes that are replicated into its `ReplicaWorld` and visible under the current Room/AOI/claims; the server may query authoritative attributes under server policy.

Example interaction flow:

1. `AccountId=acct-07` authenticates at the central Account Server and is represented by its `AccountEntity`.
2. The connection enters `RoomId=room-01`; Game Server creates `PlayerEntity` with `NetEntityId=N1` and records the connection binding to `N1`.
3. The client sends only `ChatInput(text)`. The server resolves the sender as `N1` through the common binding, applies the command on the next authoritative Tick, and emits the event with `senderNetEntityId=N1`.
4. Another client receives `N1` in its ReplicaWorld mapping and may query synchronized/public attributes for `N1`; server-only or persist-only fields remain unavailable to that client.

### Chat

- Chat uses the formal typed-mapping path: client input is carried by `InputCommand`; server state is carried by `FullSnapshot.stateBlocks` and `Delta.changedBlocks`. The old Hello wire contract is not extended.
- `ChatInput` carries message text only at the gameplay level. The client does not supply a gameplay input sequence or frame number. Transport/session sequencing and duplicate handling remain protocol-layer concerns.
- `ChatComponent.SetMessage` runs on the Simulation Owner Thread, updates the component's last-message state, and emits a `ChatMessageEvent` in the same authoritative Tick.
- The first channel is the current Room public channel. Chat obtains the sender through the common connection-to-Entity binding; the client supplies message text only and cannot choose an arbitrary sender entity.
- Chat delivery is reliable and ordered within a Room. Each event has a server `MessageId`, a strictly increasing Room chat sequence, and the authoritative `appliedTick` at which `SetMessage` committed; clients do not provide the tick.
- `ChatMessageEvent` is a live notification. The server does not persist chat history. Clients append received events to their own chat window; this presentation state is independent of server persistence.
- `ChatComponent` participates in the existing ECS field-attribute persistence flow. Its persisted state is the last message text and the last message logical Tick/Frame, marked persist-only on the authoritative server; it is not a client property-sync stream. The Game Server does not create a parallel Chat persistence path.
- On reconnect, the client does not replay old Chat events or repopulate the cleared chat window from the last-message component state.

### Timer and Persistence Boundaries

- The shared Native Timer Manager first supports fixed Tick/Frame timers, one-shot and repeating timers, cancellation, scope/generation checks, and controlled `CallbackSlot` callbacks. Full multi-time-domain support is deferred.
- The reconnect retention deadline is Host-owned monotonic time, separate from gameplay Tick/Frame timers.
- General ECS persistence continues to use the existing Snapshot/Restore and WAL/Command Log architecture. Chat history is explicitly excluded; only the declared `ChatComponent` state is eligible for ECS persistence.

### Workflow Scope

- This work creates one new Requirement Room and does not create a new milestone.
- Existing `RM-00010 Hello World` remains archived and untouched. Existing module-room requirements remain in their original rooms and are referenced as prerequisites rather than moved.

### Room Review Rulings (2026-09-01)

Owner rulings from the RM-00011 room review (evidence and per-card change list: `docs/reviews/2026-09-01-rm-00011-room-review.md`).

- **Contracts first, then parallel (MVP delivery-efficiency principle).** All public protocols, APIs and constraints this slice consumes are frozen in a Wave 0 contract set before implementation cards start. Repositories then develop in parallel against frozen contracts and fixtures without waiting on one another; integration happens last. Every requirement card states this principle explicitly.
- **Mainline attachment.** RM-00011 is on the MS-00001 mainline; all of its requirements attach to milestone MS-00001. No new milestone is created.
- **Host track: C# first, relay to Rust, short overlap.** The slice is delivered on the C# MVP host to protect the MS-00001 target date. A slice-scoped minimal Rust host starts construction immediately in parallel (this amends the adoption-time deferral of the Rust host mainline). After the C# slice passes acceptance, the Rust host re-runs the identical acceptance suite; on pass, the C# MVP host is frozen and retired to reference status. Public contracts are host-agnostic. Rust-host requirement references on the cards are architecture references, not start prerequisites.
- **Account Server is a formal standalone service.** An independent C# process in the LumioServer repository under `account-server/`, held to the formal standard: real login-or-register protocol; hashed credential storage (no plaintext, even for the `123456` test profile); signed, expiring, opaque admission credentials; a durable account store (the same AccountId survives service restarts); a real Bot-tool credential claim. The third-service topology is recorded by a new ADR.
- **AccountEntity is an ECS entity.** The Account Server hosts a low-frequency ECS World; account data is modeled as dedicated components. Credential material never lives in ordinary components — it stays in the credential store or behind a persist-only, never-replicated declaration.
- **D-009 is unblocked for this slice.** ADR-049 is to be finalized as the generic gameplay command envelope (InputCommand upstream, state payload downstream). ChatInput is the first tenant; the envelope's generality must be paper-validated against a voxel-dig command before freezing. Delivery follows ADR -> Schema/ID -> Fixture -> Baseline -> seven-repo mirrors.
- **Duplicate admission is takeover.** A new authenticated admission for an account with a live connection kicks the old connection with an explicit termination notice and rebinds the same retained entity (same NetEntityId) through the reconnect rebind path. The acceptance is single-behavior; the former reject-or-idempotent disjunction is removed.
- **Two-layer timer, both first-class, with real slice consumers.** The Host timer service (monotonic time, typed command delivery) owns the five-minute reconnect deadline. The Native Tick/Frame Timer Manager (NativeCore core, Server/Client adapter distribution, CallbackSlot) gets real consumers in this slice: Bot chat cadence runs on the Client Timer Manager and at least one server-side periodic task runs on the Server Timer Manager. The Timer ABI freeze card records the layering and the end-state unification direction toward the native core scheduler.
- **No dependency on the replication send scheduler.** Chat replication uses minimal reliable ordered Room broadcast; `R-00295` is not a prerequisite, and the scheduler may take over the broadcast path later without contract change.
- **Persistence scope narrowed to component level.** The slice verifies only the ChatComponent last-message fields' snapshot/restore roundtrip; full-Room process-restart recovery is a non-goal of this room and stays with the persistence mainline.

## Open Decisions

- Freeze the exact `ChatInput`, `ChatMessageEvent`, `ChatComponent`, and mapping schemas against the formal generated contract surface.
- Freeze the common connection-to-Entity binding and ECS Attribute Query API, including visibility, permission, revision and stale-entity failure semantics.
- Specify the Native Timer Manager ABI, `TimerHandle`, callback-slot failure semantics, and lifecycle fixtures.
- Specify Account Auth/Profile Port fields and the deterministic reference implementation boundary.
- Freeze the exact Bot-tool registration/admission claim and its failure codes in the Account Auth/Profile contract.
- Specify Room snapshot keys, recovery ordering, and the measurable Snapshot/WAL cadence for the ECS slice.
- Produce the final Workflow Requirement Room blueprint, dependency DAG, waves, and acceptance-item set after the public contracts are frozen.

## Change Log

| Date | Change |
| --- | --- |
| 2026-09-01 | Initial living decision log created from the confirmed ECS entity, Room, reconnect, Chat, Timer, persistence, and Workflow discussion. |
| 2026-09-01 | Clarified that the last-message fields are server ECS persistence only; client chat UI consumes live events independently. |
| 2026-09-01 | Clarified reconnect scope: the server continues normal Room simulation; only the client connection and `ReplicaWorld` are rebuilt. |
| 2026-09-01 | Confirmed `AccountEntity` is long-lived and keyed by stable `AccountId`; logout ends the session without replacing the account object. |
| 2026-09-01 | Removed client gameplay input sequence/frame from ChatInput; the server stamps the authoritative applied Tick and protocol sequencing remains below the Chat payload. |
| 2026-09-01 | Corrected the main count and type rule: 100 Bot clients create 100 `BotEntity` instances; one Browser client creates one `PlayerEntity`, for 101 Game ECS Entities. |
| 2026-09-01 | Clarified that Game Entities retain the stable `AccountId` value, not an AccountEntity reference; runtime rebinding A -> B does not change account identity. |
| 2026-09-01 | Corrected entity-type source: the Bot launcher only generates/logs in Bot-numbered accounts; Game Server classifies the authenticated login name and creates `BotEntity`, while normal Browser accounts create `PlayerEntity`. |
| 2026-09-01 | Confirmed the `Bot` plus decimal digits namespace is restricted to Bot-tool registration/admission; ordinary clients cannot register or claim Bot accounts. |
| 2026-09-01 | Workflow record created as `RM-00011` with 11 Requirements and 39 acceptance items; the 23 planned Requirement reference edges remain in the resumable bundle after the platform reference endpoint returned HTTP 500. |
| 2026-09-01 | Room review rulings recorded: contracts-first parallel delivery, MS-00001 attachment, C# host with relay to a slice-scoped minimal Rust host, formal standalone Account Server with an ECS AccountEntity World, D-009 unblocked via ADR-049 finalization, takeover on duplicate admission, two-layer timer with real slice consumers, no R-00295 dependency, component-level persistence scope. Review report: `docs/reviews/2026-09-01-rm-00011-room-review.md`. |
