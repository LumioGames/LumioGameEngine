# ADR-045: Replication Body Closure, MappingSetHash Domain and Length Bound

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (envelope publisher), `LumioServer` / `LumioClient` / `LumioGameRuntime` (wire consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (no existing required field, enum or ID changes; adds one digest domain and closes an already-typed body)
- **Relation**: Makes [ADR-028](ADR-028-replication-typed-bodies.md)'s own stated rationale machine-enforceable. Reuses the canonical form and digest framing frozen by [ADR-041](ADR-041-canonical-digest-profiles.md), adding a sixth domain in the same shape as its `CapabilitySetV1`.

## Context

ADR-028 separated the envelope from a typed `body` and rejected a free-form payload, in its own words, "because two implementations can pass the gate and disagree on Snapshot identity."

That rationale was never enforced. Measured on `origin/main` at 2026-08-28, five mutations of the published fixtures all passed the full 179-fixture gate with zero failures and zero warnings:

| Mutation | Gate |
| --- | --- |
| `FullSnapshot.body` carrying a private world-state payload member | passed |
| `mappingSetHash` set to the integer `42` | passed |
| `mappingSetHash` set to `null` | passed |
| `length` set to `999999999` against `maxMessageBytes: 65536` | passed |
| `BaselineAck.body` carrying a client gameplay command | passed |

Three independent causes:

1. **`replication-envelope.schema.json` declared `body` as a bare `{"type": "object"}`** with no `additionalProperties: false`, and `replication_body_errors` checked only for *missing* members, never for extra ones. The typed body was open, so the exact state ADR-028 rejected — two conforming-looking implementations carrying divergent private payloads — was reachable through the gate.
2. **`mappingSetHash` had no type anywhere.** It appeared only as a member name in a Python tuple, in ADR-028 prose, and in a registration document. No schema constrained it, so any JSON value satisfied it, and the value for a session with no registered mappings was undefined.
3. **`length` was constrained only as `integer >= 0`** and read by nothing. All eight positive fixtures declare `256` while their envelopes serialize to 482–789 bytes, so the published examples match neither an envelope-length nor a body-length reading.

`LumioServer` reached the same wall from the other side while designing its MVP host, correctly refused to add a private body member, and escalated instead of working around it. Its self-imposed "outbound body member set is exact" assertion was the right call, but it is one repository's discipline with no public counterpart.

## Decision

### 1. The typed body is closed, per MessageType

`replication-envelope.schema.json` now carries one `if`/`then` clause per registered MessageType. Each clause fixes that type's **complete legal member set** and sets `additionalProperties: false` on `body`:

| MessageType | Required | Also legal |
| --- | --- | --- |
| `Handshake` | `role` | — |
| `FullSnapshot` | `snapshotId`, `tickId`, `sessionRevisionVector`, `schemaEpoch`, `mappingSetHash` | — |
| `BaselineAck` | `snapshotId`, `confirmedRevision` | — |
| `Delta` | `baseSnapshotId`, `fromRevision`, `toRevision`, `mappingSetHash`, `confirmationSequence`, `tombstones` | `gapDetected`, `resyncReason` |
| `DeltaAck` | `confirmationSequence`, `toRevision` | — |
| `ResyncRequest` | `resyncReason` | — |
| `MaintenanceKick` | `reasonCode` | — |
| `Error` | `errorClass`, `reasonCode` | — |

Closure lives in the **Schema**, not only in `lumio_contract.py`, because downstream repositories consume published schemas; a rule that exists only in this repository's Python gate is not a public rule. `Delta` keeps `gapDetected` and `resyncReason` because ADR-028's own gap semantics require them — closure means a fixed member set, not the required set.

**This is the decision that carries the load.** Until the state payload is frozen (§4), a conforming implementation has nowhere to put world state, and can no longer smuggle it into a typed body while passing the gate.

### 2. `mappingSetHash` is the `ReplicationMappingSetV1` digest

`mappingSetHash` is `hash256` (64 lower-case hex characters), and its value is the ADR-041 digest of a sixth domain, declared in the published profile in the same shape as `CapabilitySetV1`:

```
domainTag     ReplicationMappingSetV1
input         the registered mappingId list, wrapped as {digestDomain,mappings}
normalization [{"path":"mappings","op":"sortAscending","by":"$self","collation":"codePoint"}]
```

The empty mapping set needs **no sentinel constant**: an empty `mappings` array runs the same rule and yields a defined value. Both are published as self-verifying Goldens:

```
EmptyMappingSet          {"digestDomain":"ReplicationMappingSetV1","mappings":[]}
                         a805f7c841f708981cc82a93047d7b0c8e6bf923f3dba18e179036741a6d2ea7
MappingOrderPermutation  input given unsorted; canonical bytes come out sorted
                         4120cf666fec14f6bcaf703a5d10706d755f36fb0e354dfdec6e6d5bddc40e23
```

This answers the MVP question directly: a `LumioGame` session with no registered mappings sends the `EmptyMappingSet` digest, not an empty string, not zeroes, not an omitted member.

### 3. `length` is a bound, not a byte-count claim

`length` MUST NOT exceed the envelope's own `transportPolicy.maxMessageBytes`. The gate enforces exactly that and nothing more.

This ADR deliberately does **not** define `length` as a byte count of anything, because the envelope's wire byte encoding is not frozen — that waits on §4. Freezing a byte-count meaning now would either contradict whatever encoding is chosen later, or silently bless CanonicalJsonV1 text as the wire form, which no one has decided. A bound is what a transport actually needs at admission time, and it is checkable today.

The eight positive fixtures keep `256`, which is now meaningful as "declared to fit the 65536-byte limit" rather than as a false byte count.

### 4. What this ADR does not decide

**No world-state payload is frozen here.** `FullSnapshot` and `Delta` still carry no member able to hold world state, so the MVP acceptance criterion "another client sees the block get mined" (`A1-β`) stays blocked. That decision is deliberately deferred so it can be taken together with the binary canonical primitive layout that `ADR-010:20` currently points at and that does not exist — freezing a payload member without its byte layout would reproduce exactly that dangling reference.

Client-to-server gameplay command carriage is likewise not decided; `D-009` stays frozen. §1 makes the previously-available workaround — smuggling commands through an `Ack` body — fail the gate.

## Contract

`replication-envelope.schema.json`; `canonical/canonical-digest-profile.json` domain `ReplicationMappingSetV1` and its two Goldens.

## Failure semantics

A `body` carrying any member outside its MessageType's set is invalid. A `mappingSetHash` that is not 64 lower-case hex characters is invalid. A `length` greater than the envelope's `maxMessageBytes` is invalid.

## Alternatives

**Leaving closure in `lumio_contract.py` alone was rejected**: downstream consumes schemas, so a Python-only rule leaves every other implementation unconstrained — the same asymmetry that let ADR-028 go unenforced for two baselines.

**A sentinel constant for the empty mapping set (`""`, all-zeroes, or an omitted member) was rejected**: each needs its own special case in every implementation, and an omitted member would reopen the "missing means what?" ambiguity that ADR-028 closed for the other body members.

**Defining `length` as the CanonicalJsonV1 byte count was rejected**: it would make a text encoding normative for the wire by side effect, pre-empting §4.

**Freezing the state payload in this ADR was rejected**: its line encoding is the same primitive byte layout that `ADR-010:20` references and that no profile provides, so the two must be decided together or the payload member inherits the dangling reference.

## Compatibility and migration

Additive and enforcing. No published positive fixture changes; no required member, enum value or registered ID changes. An implementation that already sends exactly the documented members is unaffected — `LumioServer`'s outbound exact-set assertion is now the public rule rather than one repository's discipline.

`tools/lumio_generate.py` changed, so `compilerHash` changes and every generated descriptor is reissued. This is the churn `D-5` is about: downstream needs a tag or artifact digest to pin, not a branch name.

## Verification

Fixtures `replication/body-extra-member`, `replication/mapping-set-hash-type`, `replication/length-exceeds-max`, `replication/ack-smuggled-command` — one negative per decision, each reproducing one of the five measured mutations. Goldens `replication-mapping-set-empty` and `replication-mapping-set-permutation` are re-derived from their inputs at every `validate`.
