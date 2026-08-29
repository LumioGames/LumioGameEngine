# ADR-048: Generated Surface — Closed Contract Bodies, Executable Gate, Dual Target

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (generator and published artifacts); `LumioGameRuntime`, `LumioClient`, `LumioCoreEngine`, `LumioServer`, `LumioGame` (consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; packaging and generated content only — no schema, required field, enum, ID or published Golden changes meaning)
- **Relation**: makes [ADR-023](ADR-023-generated-contract-artifact.md) / [ADR-039](ADR-039-contract-runtime-artifact.md) artifacts consumable rather than catalog-only, and delivers the "generated validator" that [ADR-022](ADR-022-protocol-permission-gate.md) specified but never shipped. Projects the closed bodies [ADR-045](ADR-045-replication-body-closure.md) froze; does not extend them. Records the D-015 ruling whose text lands in [ADR-040](ADR-040-root-abi-generated-bundle.md) §7.

## Context

The generated artifacts published id *strings* and field *names*. Three repositories independently reported, from implementation rather than from review, that this is not consumable:

- `LumioGameRuntime` — eight acceptance items whose "definitions found" count was **0**: there was nothing to count.
- `LumioClient` — a compile-time wall. `netstandard2.1` is the Unity face, every generated project targeted `net8.0` alone, so the packages could not even be referenced.
- `LumioCoreEngine` — S3 reported "no consumable Rust ContractTypes artifact" and fell back to embedding bytes it derived itself (R-00015, BLOCKED).

Two published rules make this unresolvable downstream: a repository must not invent a public contract, and it must use the generated validator. Under a catalog-only surface those two rules have no joint solution — the only repository that can break the deadlock is the publisher. That is what this ADR does.

## Decision

### 1. Eight closed contracts get generated type bodies in C# and Rust

`ConfigTable`, `ProcessorDescriptor`, `TxnJournalRecord`, `CommandLogRecord`, `WalRecordEnvelope`, `EntityIdentity`, `ReplicationEnvelope`, `SessionRevisionVector` are emitted as real types, generated from `schemas/` — never hand-written and never transcribed.

- **Field order is the schema declaration order.** `allOf` `$ref` members (the session/release triple, the ADR-032 recovery record chain) come first in their own declaration order, then the schema's own `properties` in file order. This is the one property a JSON-shaped input cannot carry and a consumer cannot infer, so `tools/lumio_contract.py` asserts that every published type republishes the order its schema declares **today** — a reordered schema fails the gate rather than silently producing differently-ordered types.
- **Ordinal authority is `ids/index.json` first, declaration order second, and the type says which.** Where a registry owns the names — `ReplicationEnvelope.messageType` against the `MessageType` namespace — the ordinals are the registry numerics (`Handshake` = 1, `BaselineAck` = 6), not the enum's position in the schema. Where no registry owns them, the ordinal is the declaration index. Every emitted enum records its authority in a doc comment, because an ordinal whose source is ambiguous is worse than no ordinal.
- **Wire spelling survives identifier sanitization.** A schema enum carries values like `bool` and `i32` that are not legal C# member names. The identifier is a projection for the consumer's compiler; `wire_value()` / `Wire.Value()` keeps the string that actually crosses, so no consumer reconstructs it from the identifier.
- **Open objects stay open.** A replication `body`, a WAL `inner` and a config row's `values` are `{"type": "object"}` with nothing closing them. They are emitted as `OpaqueJson`, carried verbatim. Giving them an invented shape would be exactly the "do not invent a public contract" violation this ADR exists to remove.

**This ADR publishes no new payload.** The `ReplicationEnvelope` body variants are the ones ADR-045 already froze; `FullSnapshot` and `Delta` gain no state-carrying member here. The D-1 world-state payload remains a V1.5 baseline event.

### 2. The Protocol/Permission validator becomes executable

ADR-022 specified a *generated validator* and what shipped was a list of field names. The published artifact now contains the decision itself: given a message and the context it was admitted under, it returns a verdict and, on rejection, a reason — in Rust, in C#, and in the gate that validates the fixtures.

The rejection precedence is published as data, because a validator that fails two checks at once must give one answer and three implementations must give the *same* one:

`StaleConnectionGeneration` → `SessionMismatch` → `ReleaseMismatch` → `MessagePermissionDenied` → `RoleMismatch` → `ClaimNotGranted`

Generation leads because ADR-022's failure semantics already require that code whenever the generation differs; the rest follow ADR-022's own clause order. `SessionAntiReplay` is **not** in the precedence: it is owned by `ClientReplicaSession` and is invisible to the gate. A record may declare it, and the gate then requires every check it *can* run to pass — otherwise a derivable failure could hide behind a reason nothing is able to verify.

The `messageId` clause is enforced exactly as far as this repository publishes it: **the id must be a registered `MessageType`**. ADR-022 also says "permitted for the admitted Role", and no role-to-message permission table exists anywhere in the architecture source. Deriving one here would invent a public contract; it belongs with the D-009 dispatch surface that remains blocked. The gate therefore checks registration and stops, and says so rather than appearing to check more than it does.

### 3. Every generated C# package multi-targets `netstandard2.1;net8.0`

Unity consumes `netstandard2.1`, the .NET Host consumes `net8.0`, and one published package must serve both. Consequences, all packaging-only:

- Generated C# uses **block-scoped** namespaces. Under `netstandard2.1` the default language version is C# 8, which has no file-scoped namespace. Pinning `LangVersion` forward would move the requirement onto Unity's compiler instead of solving it; block scope compiles on every version either consumer can offer.
- `record` / `init` are not used: they need `IsExternalInit`, which `netstandard2.1` does not ship. Positional records become readonly structs with a constructor and get-only properties — the same shape, no dependency.
- `SHA256.HashData` (net5.0+) becomes `SHA256.Create()` + `ComputeHash`.

**No contract byte changes.** This is the packaging form, not the payload.

### 4. Capability keys are emitted by the generator (D-015)

Recorded in full in ADR-040 §7. In short: the ID Registry `Capability` namespace remains the sole authority for the numerics, and the architecture generator becomes their sole *emitter*, publishing them in three forms — Rust (`CAPABILITY_KEYS`), C# (`CapabilityKeys`) and C (`LUMIO_CAPABILITY_*` in `lumio_core.h`). Downstream consumes the generated constants and keeps the no-hand-writing rule. `tools/lumio_contract.py` cross-checks all three published forms against the registry, so a `Capability` value added without regenerating fails the gate instead of leaving three language surfaces quietly disagreeing.

## Contract

Generated content only; no schema file changes. Published surfaces:

- `packages/rust/lumio-gen-contract-types/src/bodies.rs` and `packages/csharp/Lumio.Gen.ContractTypes/ContractBodies.cs` — the §1 bodies.
- `packages/rust/lumio-gen-protocol-permission-validator/src/lib.rs` and `packages/csharp/Lumio.Gen.ProtocolPermissionValidator/ProtocolGate.cs` — the §2 gate.
- Every `packages/csharp/*/*.csproj` — `<TargetFrameworks>netstandard2.1;net8.0</TargetFrameworks>`.
- `packages/abi/lumio_core.h`, `.../root_abi.rs`, `.../RootAbi.cs` — the §4 capability keys.

Semantic rules in `tools/lumio_contract.py`: published field order equals the schema declaration order for all eight types; the three capability forms agree with `ids/index.json`; every `protocol-permission-gate` fixture's declared verdict and reason equal what the gate computes.

## Failure semantics

A schema construct with no defined projection is a build-time failure of the generator, never a guessed type. A published type whose field order disagrees with its schema, a capability constant that disagrees with the registry, and a gate fixture whose declared verdict disagrees with the computed one are all `validate` failures. The gate never derives `SessionAntiReplay` and never claims to check role-to-message permission.

## Alternatives

**Hand-written types in each repository** was rejected — it is the drift ADR-022 and ADR-023 exist to prevent, and three repositories writing the same eight types is three chances to disagree.

**Extending the artifact-kind enum for the new content** was rejected: an enum change to a published schema is a baseline event under the `schemas/README.md` change rule. The bodies ship inside the existing `ContractTypes` and `ProtocolPermissionValidator` kinds, which is what those kinds are for.

**Pinning `LangVersion` to 10 so file-scoped namespaces survive** was rejected: it buys generator-side tidiness and spends it out of the Unity consumer's budget, which is the exact constraint D-4 exists to satisfy.

**Emitting a typed union for the open `body` / `inner` objects** was rejected: nothing in the architecture source closes them, so any shape would be invented — and inventing one inside the artifact that exists to stop invention is the worst possible place for it.

**Deriving a role-to-message permission table** was rejected for the same reason, and additionally because it would pre-empt D-009.

**Freezing capability bit positions** was rejected as out of scope: D-015 rules on the *key* space, and `capability_bits` bitmask-vs-count semantics stay unfrozen in ADR-040 §7.

## Compatibility and migration

Additive. No schema, required field, enum, ID, state or published Golden changes meaning; `LGE-V1.4-2026-08-27` and every repository mirror stay valid; no new BaselineId.

`compilerHash` moves for all twelve artifacts and `outputHash` moves for every artifact that gained content, so consumers pinning a hash re-pin — the one migration action this ADR requires. Existing consumers that read only the previously published id lists and field-name arrays are unaffected: nothing was removed or renamed.

Unblocks, concretely: `LumioGameRuntime`'s eight zero-definition acceptance items now have definitions to count; `LumioClient` can reference the packages from Unity; `LumioCoreEngine` R-00015 can consume a real Rust `ContractTypes` artifact instead of self-deriving bytes; `LumioNativeCore` R-00083 can build `StaticCapabilities` on the generated capability keys.

## Verification

- `gate/accept`, `gate/reject-claim`, `gate/anti-replay` (positive) and `gate/stale-generation`, `gate/wrong-reject-reason`, `gate/unregistered-message`, `gate/hidden-failure` (negative) — every fixture is run **through the gate**, not asserted about: the declared verdict and reason must equal the computed ones. `gate/hidden-failure` is the vector for §2's last rule, a stale generation reported as `SessionAntiReplay`.
- Published field order equals the schema declaration order for all eight types; proven by a control-group probe — permuting two members of `ReplicationEnvelope`'s published order makes `validate` fail with that type named, and restoring it makes it pass.
- The three capability forms equal `ids/index.json`; proven the same way — changing one numeric in the published `lumio_core.h` fails the gate naming that constant.
- `cargo check` and `cargo clippy --all-targets` are clean on the generated crates; every generated C# project builds for **both** `netstandard2.1` and `net8.0`.
