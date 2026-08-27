# ADR-006: NativeManagedAbiV1, Loader and Fault Domain

- **Status**: Draft for Architecture Gate
- **Owner**: `LumioCoreEngine` (aggregate ABI/Loader), `LumioNativeCore`/`LumioVoxelEngine` (source contracts)
- **Baseline**: `LGE-V1.0-2026-08-27`

## Context

Rust/C# boundary failures can otherwise cross into undefined memory, stale callbacks or duplicate Native loads. CoreEngine must ship one coherent Native combination to Server and Client.

## Decision

Expose one versioned Root API Table with `abi_version`, `struct_size` and capability bits. Cross-boundary values are fixed-width POD, versioned buffers and opaque Index+Generation+Context handles. Creator-side ownership or caller-provided buffers are explicit. Rust catches panic; Managed entry points catch exceptions; both map to stable Error Codes. Native workers never call hot Managed Gameplay. The Loader rejects a second incompatible package in one process.

## Contract

`native-managed-abi.schema.json` records pointer width, endianness, API table, ownership, threading and load policy. Generated headers and bindings include layout assertions and compiler/input hashes.

## Failure semantics

ABI/version/capability mismatch fails before World creation. Buffer-too-small returns required size. Invalid or repeated handles return stable errors. Cancellation, timeout and completion after World destruction are terminal and cannot write state. OOM, stack overflow, Native UB and CoreCLR crash are process-level faults.

## Alternatives

Passing Rust/C# containers or exceptions was rejected for layout and ownership ambiguity. Per-library ad hoc P/Invoke was rejected because it permits duplicate loaders and drift. Native callbacks into Gameplay were rejected for reentrancy and unload hazards.

## Compatibility and migration

ABI major changes require a new CoreEngine package and ReleaseManifest. Minor additive API entries are accepted only when `struct_size` and capability checks prove support. Old packages remain available for rollback.

## Verification

Run compatible and pointer-width failure fixtures, layout/align tests, panic/exception conversion, stale-handle, repeated-load, cancellation and ALC unload tests.
