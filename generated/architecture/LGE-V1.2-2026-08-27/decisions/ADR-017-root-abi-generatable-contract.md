# ADR-017: Root ABI Generatable Contract Granularity

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioCoreEngine` (`root-abi`)
- **Baseline**: `LGE-V1.1-2026-08-27`
- **Relation**: Refines the contract granularity of [ADR-006](ADR-006-native-managed-abi.md); does not supersede its fault-domain or composition decisions.

## Context

ADR-006 fixed the ABI principles (versioned Root API Table, POD-only crossings, stable Error Codes) but the public schema recorded only coarse fields. Headers and C# bindings cannot be generated from principles: they need per-slot function signatures, a declared calling convention, explicit handle and buffer lifecycles and an error-detail lifetime. Leaving these to implementation would create a private de-facto ABI that the architecture no longer governs.

## Decision

`native-managed-abi.schema.json` becomes the single generatable source of the Root ABI:

- Each API table declares `name`, `version`, `structSize`, `reservedSlots`, `functionCount` and an ordered `slots` array; each slot declares `slotIndex`, symbol `name`, typed `params`, a `returns` type and `since`. `functionCount` must equal the slot count and slot indexes are contiguous from 0 — appending is the only additive evolution.
- Types come from a closed `typeRef` grammar: fixed-width scalars, `status`, `handle:<kind>`, `buffer:in|out|inout`, `struct:<name>:v<N>` and `ptr:const|mut:<name>`. No raw pointers without qualification, no language containers.
- `callingConvention` (`C`), `entrySymbol` (`lumio_core_get_api_v<N>`) and `symbolPrefix` (`lumio_`) are explicit required fields.
- Handle lifecycle is declared by `handleModel`: Index+Generation+Context encoding, invalidation by generation bump, double-destroy returns a stable error.
- Buffer contract is declared by `bufferModel`: Ptr+Len+Capacity layout; a too-small buffer returns the required size without partial writes.
- Error detail is declared by `errorDetail`: retrieval (`PerCallOutParam` or `ThreadLocalLastError`) and lifetime (`CallerOwned` or `UntilNextCallSameThread`).
- `panicBoundary`, `exceptionBoundary`, `threading` and `loadPolicy` move from optional to required; `loadPolicy` is fixed to `OnePackagePerProcess` in V1.

## Contract

`schemas/native-managed-abi.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules: `functionCount` equals slot count, slot indexes contiguous from 0, slot names unique. Generated headers and bindings must embed layout assertions and the generator input hash per ADR-006.

## Failure semantics

A package whose ABI document fails structural or semantic validation is rejected before load (`NativeAbiMismatch` family). A slot-count or index gap is a build-time failure in `root-abi` generation, never a runtime discovery.

## Alternatives

Keeping signatures in private generator config was rejected: the public contract would no longer be sufficient to reproduce bindings. A free-form type string was rejected in favor of a closed grammar so generators cannot diverge on parsing.

## Compatibility and migration

Existing V1.0 ABI fixtures gain the new required fields; there is no deployed binary consumer yet, so the change lands in `LGE-V1.1-2026-08-27` without a migration window. Future slot additions bump `functionCount` and `structSize` and are validated by `since`.

## Verification

Fixtures `abi/compatible` (positive), `abi/pointer-width` (structural failure) and `abi/slot-count` (semantic failure: `functionCount` does not match slots).
