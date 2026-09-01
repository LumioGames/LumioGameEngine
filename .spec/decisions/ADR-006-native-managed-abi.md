# ADR-006: NativeManagedAbiV1, Loader and Fault Domain

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioCoreEngine` (aggregate ABI/Loader), `LumioNativeCore`/`LumioVoxelEngine` (source contracts)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

Rust/C# boundary failures can otherwise cross into undefined memory, stale callbacks or duplicate Native loads. CoreEngine must ship one coherent Native combination to Server and Client.

## Decision

Expose one versioned Root API Table with `abi_version`, `struct_size` and capability bits. Cross-boundary values are fixed-width POD, versioned buffers and opaque Index+Generation+Context handles. Creator-side ownership or caller-provided buffers are explicit. Rust catches panic; Managed entry points catch exceptions; both map to stable Error Codes. Native workers never call hot Managed Gameplay. The Loader rejects a second incompatible package in one process.

Catching a panic or exception transports a failure; it never classifies its blast radius. Every failure caught on the simulation path returns a stable Error Code plus a Runtime-attested `FaultClass` (registered in the ID Registry):

- `SessionLocalProven`: the Runtime proves the failing session's tick effects were not committed or were rolled back and authoritative state is unpolluted; the Host may isolate that session.
- `SlotStateUnproven`: the Runtime cannot prove authoritative state integrity; the Host must treat the owning WorldSlot as faulted and recover it from the last valid snapshot.
- `ProcessFault`: OOM, stack overflow, Native UB and CoreCLR crash remain process-level faults.

The Host never infers state consistency from catchability, and the hosting bridge never adjudicates fault scope on its own.

The single cross-repository Root API symbol (e.g. `lumio_core_get_api_v1`) is owned and exported exclusively by CoreEngine's `root-abi`/`composition` component. NativeCore and VoxelEngine ship provider API Table source contracts that CoreEngine composes; their release artifacts export no cross-repository root symbol, and a composed package's symbol table carries exactly one root entry. `capability_bits` appears only in the Root API Table and capability snapshots; other versioned exported structs are guarded by `struct_size` (plus a per-struct version field where a struct evolves independently) and do not carry capability bits.

## Contract

`native-managed-abi.schema.json` records pointer width, endianness, API table, ownership, threading and load policy. Generated headers and bindings include layout assertions and compiler/input hashes.

## Failure semantics

ABI/version/capability mismatch fails before World creation. Buffer-too-small returns required size. Invalid or repeated handles return stable errors. Cancellation, timeout and completion after World destruction are terminal and cannot write state. OOM, stack overflow, Native UB and CoreCLR crash are process-level faults. A caught failure without a `FaultClass` attestation defaults to `SlotStateUnproven`.

## Alternatives

Passing Rust/C# containers or exceptions was rejected for layout and ownership ambiguity. Per-library ad hoc P/Invoke was rejected because it permits duplicate loaders and drift. Native callbacks into Gameplay were rejected for reentrancy and unload hazards.

## Compatibility and migration

ABI major changes require a new CoreEngine package and ReleaseManifest. Minor additive API entries are accepted only when `struct_size` and capability checks prove support. Old packages remain available for rollback.

## Verification

Run compatible and pointer-width failure fixtures, layout/align tests, panic/exception conversion, stale-handle, repeated-load, cancellation and ALC unload tests, plus fault-classification tests proving a `SessionLocalProven` attestation isolates one session while `SlotStateUnproven` forces slot recovery, and a symbol-export check proving a composed package exposes exactly one cross-repository root symbol.
