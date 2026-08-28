# ADR-040: Root ABI Generated Bundle

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (bundle publisher), `LumioCoreEngine` and `LumioNativeCore` (`root-abi` consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; no existing required field, enum or ID changes)
- **Relation**: Implements the generatable half of [ADR-017](ADR-017-root-abi-generatable-contract.md) and the "generated headers and bindings include layout assertions and compiler/input hashes" clause of [ADR-006](ADR-006-native-managed-abi.md); publishes through the artifact rules of [ADR-023](ADR-023-generated-contract-artifact.md) / [ADR-039](ADR-039-contract-runtime-artifact.md).

## Context

ADR-017 made `native-managed-abi.schema.json` the single generatable source of the Root ABI, but nothing in the architecture source consumed it. The published V1.4 artifacts (`packages/`) derive `LanguageBinding` from the schema id list and `ContractTypes` from the state-machine descriptors; not one byte comes from `apiTable`, `slots` or the `typeRef` grammar. There is no C header, no interop-layout binding and no layout Golden, so `LumioCoreEngine` cannot build `lumio_core.h` or its C/C#/Rust layout tests from published artifacts alone — exactly the private de-facto ABI ADR-017 set out to prevent.

Four things are missing before a downstream repository can generate nothing by hand: the identity of the compiler that produced the bundle, the exact input set it consumed, the mapping from the closed `typeRef` grammar to concrete C/C#/Rust types, and a byte-level layout Golden for one named target.

## Decision

The architecture source publishes a **Root ABI Generated Bundle**: a language-neutral generation record plus the generated header and per-language bindings, all derived from the validated ABI document and reproducible by re-running the locked compiler.

### 1. Compiler identity is frozen

`lumio-abi-compiler`, version `1.0.0`. Its digest is the SHA-256 of the concatenated locked generator sources (`tools/lumio_contract.py`, `tools/lumio_generate.py`) — the same value already published as `compilerHash`. Name and version are now recorded next to the digest so a consumer can state *which* compiler it verified against, not only *that* the bytes matched.

### 2. The input set is frozen

The bundle consumes exactly two files: `schemas/native-managed-abi.schema.json` (the contract) and `fixtures/valid/native-managed-abi.json` (the ABI document instance, structurally and semantically validated before any output is emitted). The bundle records both paths and their combined `inputHash`. A document that fails validation produces no output; the generator fails instead of emitting a partial bundle.

### 3. The `typeRef` mapping is frozen

Every production of the ADR-017 grammar maps to one C, one C# and one Rust spelling with a fixed size and alignment on the layout profile of §4:

| `typeRef` | C | C# | Rust | size | align |
| --- | --- | --- | --- | --- | --- |
| `u8` / `u16` / `u32` / `u64` | `uint8_t` … `uint64_t` | `byte` / `ushort` / `uint` / `ulong` | `u8` … `u64` | 1 / 2 / 4 / 8 | = size |
| `i8` / `i16` / `i32` / `i64` | `int8_t` … `int64_t` | `sbyte` / `short` / `int` / `long` | `i8` … `i64` | 1 / 2 / 4 / 8 | = size |
| `f32` / `f64` | `float` / `double` | `float` / `double` | `f32` / `f64` | 4 / 8 | = size |
| `bool32` | `uint32_t` | `uint` | `u32` | 4 | 4 |
| `status` | `lumio_status_t` | `LumioStatus` | `LumioStatus` | 4 | 4 |
| `handle:<kind>` | `lumio_handle_t` | `LumioHandle` | `LumioHandle` | 16 | 8 |
| `buffer:in\|out\|inout` | `lumio_buffer_t` | `LumioBuffer` | `LumioBuffer` | 24 | 8 |
| `struct:<name>:v<N>` | `const lumio_<name>_v<N>*` | `IntPtr` | `*const Lumio<Name>V<N>` | 8 | 8 |
| `ptr:const:<name>` / `ptr:mut:<name>` | `const lumio_<name>*` / `lumio_<name>*` | `IntPtr` | `*const` / `*mut Lumio<Name>` | 8 | 8 |

Three shared POD types carry the models ADR-006/ADR-017 already fixed, now with bytes:

- `lumio_status_t` = `int32_t`. It carries the ID Registry `ErrorCode` numeric; `0` is success and no other value is reused.
- `lumio_handle_t` = `{ uint32_t index; uint32_t generation; uint64_t context; }` — the Index+Generation+Context encoding of ADR-006, 16 bytes, align 8.
- `lumio_buffer_t` = `{ void* ptr; uint64_t len; uint64_t capacity; }` — the Ptr+Len+Capacity layout of ADR-017, 24 bytes, align 8. `len` and `capacity` are fixed-width `uint64_t`, never `size_t`, so the layout does not follow the host toolchain.

`struct:<name>:v<N>` crosses the boundary **by pointer to a caller-owned POD**; the struct body is not part of the Root ABI at this granularity and stays guarded by its own leading `struct_size` per ADR-006. Parameter direction is deliberately *not* frozen: it is a per-slot design choice of the ABI document's owner, not a property of the grammar.

### 4. Struct layout is frozen for one named profile

The layout profile is `linux-x86_64-glibc` (SysV AMD64: `os=LinuxServer`, `arch=x86_64`, `abiRuntime=glibc`, pointer 8 bytes, maximum alignment 8):

- **Root table header** (16 bytes): `uint32_t abi_version` @0, `uint32_t struct_size` @4, `uint64_t capability_bits` @8. Then one function-table pointer per `apiTable` entry, in document order, at `16 + i*8`.
- **API table header** (16 bytes): `uint32_t version` @0, `uint32_t struct_size` @4, `uint64_t reserved0` @8. Then one function pointer per slot, in `slotIndex` order, at `16 + slotIndex*8`; `reservedSlots` further pointer-sized words follow the declared slots.
- **Minimum struct size**: `16 + (functionCount + reservedSlots) * pointerBytes` for an API table, `16 + tableCount * pointerBytes` for the root table. A declared `structSize` must be **at least** the minimum and a multiple of the alignment. Declaring more reserves tail space, which is the ADR-006 forward-compatibility guard; declaring less is a build-time failure, never a runtime discovery.

Slot function-pointer offsets are exact and are the Golden a consumer's layout test asserts. Tail reserve is intentionally a lower bound, not an equality, so the root table can grow without a new ABI major.

`callingConvention` `C` means the SysV AMD64 C calling convention on this profile; no other convention is generatable in V1.

### 5. Output file names are frozen

| Path (under the published package root) | Role |
| --- | --- |
| `abi/lumio_core.h` | C header: shared POD types, per-table struct, static layout assertions |
| `abi/root-abi-bundle.json` | Generation record + layout Golden (this ADR's record) |
| `rust/lumio-gen-language-binding/src/root_abi.rs` | Rust `repr(C)` structs and layout constants |
| `csharp/Lumio.Gen.LanguageBinding/RootAbi.cs` | C# sequential-layout structs and layout constants |

`abi/root-abi-bundle.json` records the digest of every other output file. The C# binding stays pure managed — it publishes layouts and signatures, never a native import — so the `packages/csharp` no-native policy of ADR-023/ADR-039 is unchanged: `LumioCoreEngine` binds the single `entrySymbol` itself and asserts the published layout.

The `rootAbi` entry in `packages/index.json` (the language-neutral directory of §"Alternatives") carries its own `consumers` list, distinct from the six Rust/C# `artifactKind`s' `consumers`: `["LumioCoreEngine", "LumioNativeCore"]`. Both are native-toolchain repositories that bind `lumio_core.h` directly; neither consumes the Rust/C# generated crates, so they are deliberately absent from the six kinds' `CONSUMERS` list and present only here. A repository not in this list has no standing to depend on `packages/abi/`.

### 6. The generation record has a schema

`schemas/root-abi-bundle.schema.json` (P0, owner `Architecture`) is the generation record contract: compiler identity, input set and `inputHash`, ABI identity (`abiVersion`, `entrySymbol`, `symbolPrefix`, `callingConvention`, `pointerWidth`, `endianness`), layout profile, the frozen type mapping, the derived root and per-table layout, and the output file digests. It is registered in `schemas/index.json` and exercised by positive and negative fixtures.

## Contract

`schemas/root-abi-bundle.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules:

- **ABI document** (`native-managed-abi`, extending the ADR-017 rules): `apiTable` names are unique; every declared `structSize` is at least the §4 minimum and a multiple of the alignment; the root `structSize` is at least `16 + tableCount * pointerBytes`.
- **Bundle**: the layout profile matches §4; every `typeRef` production of the grammar appears exactly once in `typeMapping` with the §3 sizes; root fields and table slot offsets are contiguous from the frozen headers; `minimumStructSize` never exceeds `declaredStructSize`; output paths are exactly the §5 set.
- **Published bundle**: when `packages/abi/root-abi-bundle.json` exists it must equal the bundle derived from the current ABI document — a hand-edited or stale bundle fails the gate, per the "generated artifacts are never hand-edited" rule.

## Failure semantics

An ABI document that fails structural or semantic validation is rejected before generation and maps to the `NativeAbiMismatch` family, as in ADR-017. A declared `structSize` below the derived minimum, a duplicate table name and a non-contiguous `slotIndex` are all build-time failures of the bundle generator. A published bundle that disagrees with the ABI document fails `tools/lumio_contract.py validate`; the architecture gate never publishes a bundle it cannot re-derive.

## Alternatives

Adding a seventh `artifactKind` for the bundle was rejected: extending the `generated-contract-artifact.schema.json` enum is an enum change, and the `schemas/` change rule requires a new baseline id for that — disproportionate for an additive publication that no existing consumer reads. The bundle is therefore published as a language-neutral directory whose digests are recorded in the package inventory.

Emitting a native import in the published C# artifact was rejected: it would break the pure-managed policy that ADR-023/ADR-039 enforce and that CI checks. Publishing layouts and letting the consumer bind one entry symbol keeps the policy intact and still leaves nothing hand-written but the single bind call.

Deriving the tail reserve as an equality (`structSize == minimum`) was rejected: it would forbid the reserved tail that ADR-006 relies on for additive evolution, and the V1.4 ABI document already declares a root reserve.

Freezing parameter direction (for example "output handles must be `ptr:mut:`") was rejected as outside the mapping this card freezes; it would retroactively invalidate a valid V1.4 ABI document without a contract reason.

## Compatibility and migration

Additive only. No existing schema, required field, enum, ID or fixture changes meaning, so the `LGE-V1.4-2026-08-27` baseline id and every repository mirror stay valid. The published `LanguageBinding` and `ContractTypes` artifacts gain content and therefore new `outputHash` values; consumers that pin an `outputHash` re-pin, and consumers that only read the existing schema-id binding table are unaffected. `LumioCoreEngine` deletes any hand-written `lumio_core.h`, interop struct or template and consumes `abi/` instead.

## Verification

Fixtures `abi/entry-symbol` (structural failure: `entrySymbol` outside the ADR-017 pattern), `abi/duplicate-table` (semantic failure: duplicate `apiTable` name), `abi/slot-index-gap` (semantic failure: non-contiguous `slotIndex`), `abi/struct-size` (semantic failure: declared `structSize` below the derived minimum), alongside the existing `abi/compatible`, `abi/pointer-width` and `abi/slot-count`. Bundle fixtures `rootabi/bundle` (positive), `rootabi/short-struct-size` (`minimumStructSize` exceeds `declaredStructSize`) and `rootabi/incomplete-type-mapping` (a `typeRef` production missing from the mapping). `tools/lumio_contract.py validate` additionally re-derives the published bundle and compares it byte-for-byte with `packages/abi/root-abi-bundle.json`.
