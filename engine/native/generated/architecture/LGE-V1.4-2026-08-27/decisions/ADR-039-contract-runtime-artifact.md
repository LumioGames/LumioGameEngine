# ADR-039: ContractRuntime Artifact Kind

- **Status**: Accepted (enters Implementation Baseline `LGE-V1.4-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameEngineArchitecture` (toolchain and published artifacts)
- **Baseline**: `LGE-V1.4-2026-08-27`
- **Relation**: Refines [ADR-023](ADR-023-generated-contract-artifact.md) (adds one artifact kind to its publication contract) and [ADR-007](ADR-007-contract-toolchain.md); consumes the recovery-record and canonical-shape primitives frozen by [ADR-032](ADR-032-durable-recovery-records.md) and [ADR-037](ADR-037-contract-common-primitives.md). Supersedes nothing.

## Context

ADR-023 froze five generated artifact kinds — `ProtocolPermissionValidator`, `MappingTable`, `CanonicalSerializer`, `LanguageBinding`, `ContractTypes` — all of which are *shapes*: types, tables, encoders whose bodies are fully derivable from schemas and the ID registry. The contract also needs a small amount of *behavior* that every repository would otherwise hand-write seven times: the hash-chain reader/writer for the three ADR-032 recovery records (previousHash linkage, checksum verification, truncation-at-corruption), canonical encode/decode helpers shared by the serializers, and bounded-buffer guards (length/limit enforcement per ADR-027 resource classes). These are pure functions over contract shapes — no I/O policy, no domain semantics, no native bindings — but they are code, not data, so no existing kind covers them. Without a published home, each implementation repository writes its own, and the seven copies drift in exactly the failure-semantics corners (torn tail handling, chain-break classification) where drift is most expensive. The PureHeadless profile additionally requires these helpers with **zero** native dependencies, in both languages, because §16.1's vertical slice must persist archives without any Rust library loaded.

## Decision

### 1. New artifact kind

`generated-contract-artifact.artifactKind` gains `ContractRuntime`. A ContractRuntime artifact is the support library shipped alongside the generated shape artifacts: hash-chain readers/writers for `TxnJournalRecord`/`CommandLogRecord`/`WalRecordEnvelope`, canonical encode/decode helpers used by `CanonicalSerializer` outputs, and bounded-buffer guard primitives. It ships in exactly two forms per baseline — a pure Rust crate and a pure C# assembly — with identical observable behavior, versioned by `BaselineId` like every other artifact.

### 2. Hard constraints (same gate as the other kinds, plus two)

Everything ADR-023 requires of a published artifact applies unchanged: publisher is this repository only, compiler/input/output hashes recorded, `implementationDependencies` empty, `forbiddenDependents` = {LumioClient, LumioGame}. Two constraints are specific to this kind and frozen here: **zero native dependencies** (the C# form must run on PureHeadless with no Rust library present; the Rust form links no engine crate) and **zero domain semantics** (the library may verify a chain and report the break classification; deciding what to *do* about a broken chain — fail-stop, rebuild, alert — stays in the consuming repository. If a helper needs to know what a `txnId` means, it does not belong here).

### 3. Repository usage rule 1, clarified

Root README rule 1 ("implementation repositories do not depend on this repository's runtime code") is read as follows: the ContractRuntime support library is a **published artifact** — referencing it is exactly as legitimate as referencing `ContractTypes`; "this repository's runtime code" means the checker/generator tooling under `tools/` (`lumio_contract.py` and the future generator), which is never a dependency of any implementation repository. The distinction is mechanical: published artifacts carry recorded hashes and a `BaselineId`; tooling does not ship.

## Contract

Changed: `schemas/generated-contract-artifact.schema.json` (`artifactKind` enum +`ContractRuntime`), root `README.md` rule 1 (clarified reading), this ADR. Fixtures: `gencfg/contract-runtime` (valid ContractRuntime artifact), `gencfg/contract-runtime-impl-dep` (failure: a ContractRuntime declaring an implementation dependency).

## Failure semantics

A ContractRuntime artifact with a non-empty `implementationDependencies`, a publisher other than this repository, or missing hashes is rejected before publication — the same registry rejection as every ADR-023 kind. A support library observed depending on a native engine crate or exposing domain policy fails architecture review as a rule-1 violation; the schema cannot see inside the package, so this constraint is enforced at generation time by the toolchain and at review time by the consuming repositories.

## Alternatives

An eighth repository for the support library was rejected: it would need its own baseline/mirror discipline for what is a build *output* of this repository's toolchain. Folding the helpers into `CanonicalSerializer` outputs was rejected because the hash-chain and buffer-guard code is shared across three record families and both serializer directions — it would be duplicated into every generated serializer, recreating the drift the artifact exists to remove. Hand-writing the helpers per repository (status quo ante) was rejected for the seven-copies drift argument in Context.

## Compatibility and migration

Additive: the enum grows, no existing artifact changes shape. First publication of an actual ContractRuntime package lands with the contract generator (Foundation first card); this ADR freezes the category and its constraints so the generator card has a settled target. Consuming repositories replace any interim hand-written chain/guard code when the first package ships.

## Verification

`python3 tools/lumio_contract.py validate` — the valid ContractRuntime fixture passes structural + semantic gates; the implementation-dependency fixture is rejected by the existing empty-`implementationDependencies` rule, proving the new kind inherits every ADR-023 constraint without new tool code.
