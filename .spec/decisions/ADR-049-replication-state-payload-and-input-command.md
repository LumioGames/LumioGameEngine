# ADR-049: Replication State Payload and InputCommand Carriage

- **Status**: Draft (targets `LGE-V1.5`; **a baseline event — not additive within `LGE-V1.4-2026-08-27`**, and not in force until the V1.5 batch lands)
- **Owner**: `LumioGameRuntime` (`MessageType` namespace owner and replication semantics), `LumioServer` / `LumioClient` (wire adapters), `LumioGame` (domain mapping schemas)
- **Baseline**: targets `LGE-V1.5`. Extends a closed body's required set and adds a `MessageType` enum value; both are baseline events under the `schemas/README.md` change rule. Rides the single V1.5 transition planned in [`docs/plans/2026-08-29-v1_5-baseline-batch-plan.md`](../../docs/plans/2026-08-29-v1_5-baseline-batch-plan.md).
- **Relation**: fills the two holes [ADR-045](ADR-045-replication-body-closure.md) §4 deliberately left open ("no world-state payload is frozen here"; "client-to-server gameplay command carriage is likewise not decided"). Encodes payload bytes under [ADR-047](ADR-047-lumio-bin-canonical-profile.md)'s `LumioBinV1`, which ADR-047 §Compatibility already named as this payload's decided encoding. Refines [ADR-028](ADR-028-replication-typed-bodies.md) and, through it, [ADR-005](ADR-005-replication-prediction.md); the Accepted text of both is unchanged. Digest framing follows [ADR-041](ADR-041-canonical-digest-profiles.md) §2.

## Context

ADR-028 separated the replication envelope from a typed `body`. ADR-045 closed that body per MessageType — every legal member of every registered type is now fixed, with `additionalProperties: false` — and in doing so made explicit what had previously been reachable by accident: **there is nowhere to put world state, and nowhere to put a client input.**

ADR-045 §4 says so in its own words, and names the reason for the deferral: freezing a payload member without its byte layout would inherit the dangling reference that `ADR-010:20` had at the time. That blocker is now gone. ADR-047 froze `LumioBinV1` — fixed-width little-endian integers, `u32`-prefixed strings/byte-strings/arrays, structs as declaration-order concatenation with no padding, closed field sets, prefix-free SHA-256 over the encoded bytes — and its own §Compatibility states the conclusion this ADR executes: the D-1 replication state payload "is *not* implemented here — its required-field extension and `MessageType` addition are baseline events and ride the V1.5 batch — but its encoding is now decided: `LumioBinV1`."

Two consequences are measured, not predicted:

- **Downstream is blocked on the downstream half.** The MVP acceptance criterion `A1-β` ("another client sees the block get mined") cannot be met, because a `FullSnapshot` or `Delta` that conforms to the published schema carries no world state. `LumioServer` reached this wall while designing its MVP host, correctly refused to add a private body member, and escalated rather than working around it.
- **The upstream half has no carriage at all.** Client-to-server gameplay commands had exactly one previously-available route — smuggling them through an `Ack` body — and ADR-045 §1 closed it on purpose (`replication/ack-smuggled-command`). Closing the workaround without opening the road is the state this ADR ends.

The direction was adjudicated on 2026-08-29 (`docs/plans/2026-08-29-contract-surface-adjudication.md` §裁决三) and is recorded here unchanged. What that adjudication settled is *what* to build; this ADR is the first place its **byte-level and failure semantics** are written down.

## Decision

### 1. Downstream state travels in the existing typed bodies, as declared blocks

`FullSnapshot.body` gains a required `stateBlocks`; `Delta.body` gains a required `changedBlocks`. Both are arrays of the same block shape. **No new envelope and no new MessageType is introduced for downstream state** — the envelope, its `transportPolicy`, its `length` bound (ADR-045 §3), its integrity member and its sequencing already apply, and duplicating them into a parallel envelope would be a second source of truth for the same properties.

Each block carries:

| Member | Type | Meaning |
| --- | --- | --- |
| `mappingId` | `common.schema.json#/$defs/id` | which registered mapping this block's bytes belong to |
| `payload` | `common.schema.json#/$defs/hexOrBase64` | the block's bytes, encoded under `LumioBinV1` per the mapping's declared field order |
| `payloadHash` | `common.schema.json#/$defs/hash256` | prefix-free `SHA-256` of the payload bytes (ADR-047 §2 construction, no domain tag, no length framing) |

`stateBlocks` and `changedBlocks` are **required, and MAY be empty**. An empty array is the defined encoding of "this snapshot/delta carries no state for any mapping" — the same reasoning ADR-045 §2 used to refuse a sentinel for the empty mapping set: an empty array runs the same rules and yields a defined value, whereas an omitted member reopens the "missing means what?" ambiguity ADR-028 closed.

### 2. Block order is the mappingSet declaration order, and it is machine-checkable

The blocks in `stateBlocks` / `changedBlocks` appear in **the order of the mapping set that produced the envelope's `mappingSetHash`** — that is, the code-point-ascending sorted `mappings` list frozen by ADR-045 §2 as the `ReplicationMappingSetV1` normalization. Blocks are a subset of that list (a Delta typically touches few mappings), and a subset preserves the order of the whole.

Order is fixed for the same reason ADR-035 fixed `chunkOrder` and ADR-047 fixed struct declaration order: **two conforming encoders that produce different bytes for the same state are a fatal contract violation, not a tolerable variance.** Sorting at the receiver would hide the divergence rather than prevent it.

This makes order a property the gate can check without knowing any domain semantics: the `mappingId` sequence must be strictly ascending under code-point collation, and every `mappingId` must appear in the mapping set the envelope's `mappingSetHash` digests. Strictly ascending also forbids a repeated `mappingId`, which would otherwise make "which block wins" an implementation choice.

### 3. `payloadHash` binds the bytes, and the gate recomputes it

`payloadHash` is not a declared value the gate merely type-checks. `tools/lumio_contract.py` **decodes `payload` and recomputes the digest**, exactly as ADR-047 §3 does for its Goldens and ADR-041 §4 does for its normalization declarations. The rule this repository has already learned twice is that a published digest a gate does not recompute rots into a lie; a `payloadHash` that is merely *shaped* like a hash256 would let two implementations disagree on the bytes and still pass.

`payloadHash` covers the **payload bytes only** — not the block, not the body, not the envelope. Envelope-level integrity stays where ADR-028 put it, in the envelope's own `integrity` member. The two do not overlap and neither substitutes for the other.

### 4. Upstream input is a new MessageType with its own envelope schema

Client-to-server gameplay input travels as `MessageType` **`InputCommand`**, registered in `ids/index.json` under the `MessageType` namespace (owner `GameRuntime`) at the next unused numeric — `9` at the time of writing; **values are never reused or backfilled**, so the executing session re-reads the registry and takes whatever the next free numeric then is.

Input carriage gets its **own schema** (`input-envelope.schema.json`), not another `if`/`then` clause on the replication envelope. Three reasons, in decreasing order of weight:

1. **Direction is not a body detail.** Every other registered MessageType is server-authored or an acknowledgement of one. Input is the only client-authored *state-bearing* message, so its admission path, its rate limits and its permission checks are not the outbound ones. Folding it into the outbound envelope would make "who may send this" a per-clause convention rather than a schema boundary.
2. **The outbound envelope's members do not fit.** `sequence`, `snapshotId`-family identity and the outbound `transportPolicy` are properties of a server→client stream; an input message needs a client-side sequence and a prediction key, which have no meaning outbound.
3. **ADR-022's permission gate can then key on the schema**, not on a string compare inside a shared document.

`InputCommand` is registered in the `MessageType` namespace nonetheless, because ADR-028's three-way consistency assertion (schema enum = ID registry = fixture-used types) is what keeps the namespace honest, and a MessageType that lives outside the registry would be exactly the private de-facto contract this repository exists to prevent.

The input envelope carries the session/release triple (`common.schema.json#/$defs/sessionReleaseTriple`), a client-monotonic `commandSequence`, the `predictionKey` ADR-005 already names in its prediction loop, the `tickId` the client believes it is acting on, and a `commands` array whose entries carry `mappingId` / `payload` / `payloadHash` under **the same §1–§3 rules** — same `LumioBinV1` encoding, same ascending-`mappingId` order, same recomputed digest. The two directions share one encoding discipline; only the framing differs.

### 5. What this ADR does not decide

- **No role→message permission table.** ADR-048 §2 states the generated validator checks registration, not role authority, because the architecture source has no such table and inventing one would front-run `D-009`. This ADR adds a message a client may send; it does **not** thereby publish who may send what. That stays `D-009`.
- **No wire byte encoding for the envelope itself.** ADR-045 §3 deliberately left `length` a bound rather than a byte count because the envelope's wire form is unfrozen. This ADR freezes the **payload** bytes and changes nothing about the envelope's own serialization, so `length` keeps its ADR-045 meaning.
- **No compression.** `common.schema.json` has a `compressionCodec` def and voxel payloads use one; a compressed block would need its digest domain settled (bytes before or after compression) and nothing yet requires it. Blocks are uncompressed; a future ADR may add a codec member with its digest rule stated explicitly.
- **No float rule.** ADR-047 refuses `f32`/`f64` as an unknown layout kind. A mapping needing a float declares its rule in its own domain ADR; this ADR does not reopen that.

## Contract

`schemas/replication-envelope.schema.json` (`messageType` enum gains `InputCommand`; `FullSnapshot.body` required set gains `stateBlocks`; `Delta.body` required set gains `changedBlocks`), a new `schemas/input-envelope.schema.json` registered in `schemas/index.json`, and `ids/index.json`'s `MessageType` namespace. Semantic rules land in `tools/lumio_contract.py`:

- every `mappingId` in a block list is in the mapping set whose digest is the envelope's `mappingSetHash`;
- block `mappingId`s are strictly ascending under code-point collation;
- every `payloadHash` is recomputed from the decoded `payload` and must match;
- every `payload` decodes as `LumioBinV1` for its mapping's declared layout, or the envelope is rejected;
- the `MessageType` schema enum, the ID Registry and the fixture-used set stay one set (the existing ADR-028 assertion, now covering `InputCommand`).

## Failure semantics

A `FullSnapshot` without `stateBlocks`, or a `Delta` without `changedBlocks`, is invalid — the member is required, and an empty array is how "nothing to send" is spelled. A block whose `payloadHash` does not recompute from its `payload` is invalid, and is rejected **before** the payload is interpreted, so a divergent encoder fails at admission rather than corrupting state. A block list that is not strictly ascending by `mappingId`, or that repeats a `mappingId`, or that names a `mappingId` outside the digested mapping set, is invalid. A payload that does not decode under `LumioBinV1` produces no state and no partial application: per ADR-047, an unencodable value yields no bytes, and by symmetry an undecodable byte string yields no value — never a truncated, padded or reordered read. An `InputCommand` sent on the replication envelope rather than the input envelope is invalid, as is an input envelope carrying any other `messageType`.

Rejection of any of the above is a `Rejectable` envelope-level error under ADR-028's three error classes; it does not by itself request a resync, because a malformed message proves nothing about baseline continuity. Gap and resync semantics are unchanged by this ADR.

## Alternatives

**A single opaque `payload` blob per body, with no per-mapping blocks**, was rejected. It reproduces the exact defect ADR-028 named when it refused a free-form payload — "two implementations can pass the gate and disagree on Snapshot identity" — one level down: the gate could check that bytes exist but never that they mean the same thing on both ends, and a per-mapping digest would be impossible.

**A separate downstream state envelope (a new `StateSnapshot` MessageType)** was rejected. The envelope properties that matter — sequencing, `transportPolicy`, `length` bound, integrity, session/release identity — would have to be duplicated, and a duplicated property is one that can disagree. The typed bodies already exist and are already closed; extending a closed body is a smaller and more checkable change than adding a parallel one.

**Folding `InputCommand` into `replication-envelope.schema.json` as a ninth `if`/`then` clause** was rejected for §4's three reasons; the decisive one is that direction and authority would become a convention inside a shared document rather than a schema boundary that ADR-022's gate can key on.

**Sorting blocks at the receiver instead of fixing sender order** was rejected: it converts a detectable divergence into a silent one. This is ADR-035's `SameCutSameBytes` position and ADR-047's declaration-order position applied to the same problem a third time.

**Letting `payloadHash` be optional when the envelope already carries `integrity`** was rejected. Envelope integrity covers the envelope as transmitted; it says nothing about whether two encoders produced the same *state* bytes, which is the property `A1-β` depends on. They are different assertions with different failure meanings.

**Landing this inside `LGE-V1.4-2026-08-27` as "additive"** was rejected as factually wrong, not merely cautious. Adding a member to a closed body's *required* set breaks every conforming producer, and `schemas/README.md` names required-field and enum changes as baseline events. Calling it additive would be the kind of self-contradicting claim `lessons.md` records twice.

## Compatibility and migration

**Breaking, by construction.** Two changes each independently require a new BaselineId:

1. `FullSnapshot.body` and `Delta.body` gain required members. A producer emitting the V1.4 shape is invalid under V1.5.
2. The `MessageType` enum gains a value. Every consumer that treats the enum as exhaustive must be recompiled against the regenerated surface.

There is no deployed wire consumer to migrate — the same position ADR-028 recorded when it broke the envelope shape in V1.3 — so no compatibility window is declared and no dual-shape acceptance period exists. `A1-β` and the `LumioGameRuntime` replication cards unblock when V1.5 lands, and not before.

Downstream absorbs, per repository:

- **GameRuntime** owns the `MessageType` namespace and confirms the new value; implements block production/consumption and the input path's server side.
- **Server** and **Client** take the new required members and the new input envelope. Server's inbound `Delta` acceptance must continue to admit `gapDetected` and `resyncReason` (ADR-045 §1 keeps them legal-but-optional); this ADR does not narrow that set.
- **Game** declares the per-mapping layouts whose declaration order §1 encodes against.
- **VoxelEngine** is affected only indirectly: its ADR-035 payload already assumed the primitive layer ADR-047 published, and this ADR does not change voxel payload framing.
- Every artifact's `compilerHash` moves with the batch, and `packages/` is reissued. Consumers pinning an `outputHash` re-pin against the V1.5 tag rather than a branch name (`D-5`).

## Verification

Positive fixtures:

- `replication/full-snapshot-state-blocks` — a `FullSnapshot` whose `stateBlocks` carries two blocks in ascending `mappingId` order, each `payloadHash` recomputing from its `LumioBinV1` payload.
- `replication/delta-changed-blocks` — a `Delta` carrying a strict subset of the mapping set, proving subset-of-order is legal.
- `replication/full-snapshot-empty-state-blocks` — the empty array is valid and is the defined "no state" encoding, pinning §1's no-sentinel rule the way ADR-045's `EmptyMappingSet` Golden pins its own.
- `input/command` — a well-formed `InputCommand` on the input envelope.

Negative fixtures, one per decision clause, each constructed so that removing the clause makes it pass:

- `replication/state-block-payload-hash-mismatch` — a block whose `payloadHash` is not the digest of its payload (§3).
- `replication/state-block-order-violation` — two blocks in descending `mappingId` order (§2).
- `replication/state-block-duplicate-mapping` — the same `mappingId` twice (§2).
- `replication/state-block-unknown-mapping` — a `mappingId` outside the set digested by `mappingSetHash` (§2).
- `replication/full-snapshot-missing-state-blocks` — the V1.4 shape, which must now fail (§1 and the Compatibility claim; this fixture is what makes "breaking" a fact rather than an assertion).
- `replication/state-block-undecodable-payload` — a payload that is not valid `LumioBinV1` for its layout (§Failure semantics).
- `input/on-replication-envelope` — an `InputCommand` presented on `replication-envelope.schema.json` (§4).
- `input/unregistered-target` — an input envelope whose `messageType` is not `InputCommand` (§4).

**Acceptance bar for the executing session** (承 `lessons.md` 的对照组探针纪律): each negative above must be shown to *actually* fail — produce the real non-zero `validate` output with the fixture in place, restore, and produce the passing output. "The gate passed" is not evidence that a guard works; only a probe that goes red and then green is. Each new rule in §Contract must be introduced in the **same commit** as the negative fixture constructed against it.
