# D-005 Consumer Track Report: R-00141 Canonical Binary Codec

## Result

**BLOCKED (authoritative card revision).** The latest R-00141 TD revision says that the architecture publishes only the LumioBinV1 profile declarations and golden vectors; it does not publish an executable encoder/decoder, and this consumer track is explicitly forbidden from developing a local substitute. The assigned LumioGameRuntime worktree contains no persistence project after cleanup, no implementation commit was made, and no substitute production or test files remain.

## Blocking authority

The complete card read-back was read before any work:

C:\Users\g923\AppData\Local\Temp\d005-cards\R-00141.md

The dispatch brief was read in full:

C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-canonical-brief.md

The decisive latest TD card text (R-00141.md, revision dated 2026-08-29) states:

- S06 MessagePack wrapping is obsolete because MessagePack is a rejected public surface; the card must follow LumioBinV1.
- The new baseline publishes only the LumioBinV1 declaration and golden vectors, not an executable codec.
- Before an upstream codec is published, the whole card remains blocked and a local encoder/decoder must not be developed as a replacement.
- The later reconciliation records that the generated surface still has no executable canonical encoder/decoder and that S05 must align to LumioBinV1, not a locally invented format.

This is a hard stop under the card's own blocker discipline. No implicit MessagePack adapter, local schema, fake generated contract, or durability tier was selected.

## Architecture authority read-back

Authority inputs named by the brief:

    Architecture repository: C:\Work\LumioGames\LumioGameEngineArchitecture
    Primary commit:         c14df420ac05b0d23f1fb674977b9a4c957edac5
    Also present in:        f71cac137733b7f1609ae8235676d44c9f324858
    Baseline:               LGE-V1.4-2026-08-27

The raw committed blobs were read with git cat-file and binary redirected before SHA-256 hashing. The resulting SHA-256 values exactly match the brief:

    docs/specs/lumio-save-design-overview.md
    d69c69374ef960b1968f0e8b2fdd4195d1abd52ed5ab34fd00b406fa85f141f1

    docs/specs/2026-08-30-save-load-architecture-decisions.md
    82ed79a72ced56913c79ffa0bfb6d3763221ff2312c13c4a4d34f56e89b56f7c

Verification commands and key results:

    git -C C:\Work\LumioGames\LumioGameEngineArchitecture cat-file -e c14df420ac05b0d23f1fb674977b9a4c957edac5:docs/specs/lumio-save-design-overview.md
    exit 0

    git -C C:\Work\LumioGames\LumioGameEngineArchitecture cat-file -e c14df420ac05b0d23f1fb674977b9a4c957edac5:docs/specs/2026-08-30-save-load-architecture-decisions.md
    exit 0

    git -C C:\Work\LumioGames\LumioGameEngineArchitecture cat-file -e c14df420ac05b0d23f1fb674977b9a4c957edac5:packages/binary/lumio-bin-profile.json
    exit 0

The published package tree at that commit contains:

    docs/adr/ADR-047-lumio-bin-canonical-profile.md
    packages/binary/lumio-bin-profile.json
    packages/csharp/Lumio.Gen.CanonicalSerializer/CanonicalProfile.cs
    packages/csharp/Lumio.Gen.CanonicalSerializer/CanonicalSerializer.cs
    packages/csharp/Lumio.Gen.CanonicalSerializer/Lumio.Gen.CanonicalSerializer.csproj
    packages/csharp/Lumio.Gen.CanonicalSerializer/LumioBinProfile.cs
    packages/csharp/Lumio.Gen.CanonicalSerializer/artifact.descriptor.json

The profile declaration read-back is:

    FormId=LumioBinV1
    ByteOrder=LittleEndian
    StringEncoding=Utf8
    StringLengthPrefix=u32
    BytesLengthPrefix=u32
    ArrayCountPrefix=u32
    FieldOrder=SchemaDeclarationOrder
    Floats=None
    DigestFraming=None

The published JSON reports profileId=lumio-bin-v1, schemaEpoch=1, six goldens, and twelve rejection vectors. The six generated golden identifiers and published SHA-256 values are:

    integer-widths             e4c15e2b8347986315e042c3b009ac9d9fc4833ffdfa984671c804d48c53af72
    string-utf8                a2969994674a03c90bdf3a04fc1e872e57dfb5c69b20c02a6ec58a8fcdecc77f
    bytes-prefixed             0099fed1a7eb2bd476767cc61c24fd219eb85f12a771097b6ed1f8f9c0a191fc
    array-count                a39723192d4a221f9eb82ffb339d1ca9306ed7cd3c9ebff18d66b3f3094d3080
    struct-declaration-order   906a52a6e0337a092c17b65dbc4d35ceeede618307bb6178e8661f6ef9e43f95
    nested-composition         109299fca81e33863a42d186eae66c8f3528b1b960deb067b53060d1c9438ad7

Additional raw publication hashes captured for provenance:

    packages/binary/lumio-bin-profile.json
    03b3fab181d1ebbe73b2c853d569c0819f08309cf339848809f92100368d458e

    packages/csharp/Lumio.Gen.CanonicalSerializer/LumioBinProfile.cs
    a6491b1319da26c9bf71f7a4e0070fcfab8ee0d7e6866ab70ed87dcca4bc383

An architecture package symbol scan found no executable codec surface:

    git -C C:\Work\LumioGames\LumioGameEngineArchitecture grep -n -E "class .*Codec|Encode<|Decode<|CanonicalRecordWriter|CanonicalPrimitive" c14df420ac05b0d23f1fb674977b9a4c957edac5 -- packages/csharp packages/rust
    no executable codec symbol hits in published packages

Therefore LumioBinForm, LumioBinGoldens, and the JSON profile are read-only inputs, not an implementation dependency that can be completed in this consumer repository.

## Runtime preflight

Assigned worktree:

    Repository: C:\Users\g923\orca\workspaces\LumioGameRuntime\d005-canonical
    Branch:     Go1c/d005-canonical
    HEAD:       ef822a76cd5586513ea6e52b3ea4f5497917bdc8

The initial required project probe was run before scaffolding:

    dotnet test modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/Lumio.GameRuntime.Persistence.Tests.csproj
    exit 1
    MSBUILD : error MSB1009: project file does not exist
    switch: modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/Lumio.GameRuntime.Persistence.Tests.csproj

This is the expected missing-project RED evidence. The environment itself is available, but that does not remove the architecture blocker:

    dotnet --info
    SDK 10.0.111
    Host/runtime 10.0.11
    RID win-x64

After cleanup, the persistence source and test project directories are absent, and the final Git read-back is clean:

    git status --short --untracked-files=all
    (no output)

    Test-Path modules/persistence/src/Lumio.GameRuntime.Persistence
    False

    Test-Path modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests
    False

## Files touched during this dispatch

The following files were provisionally created while the original dispatch brief was being evaluated, then removed immediately after the authoritative TD block was surfaced. None is present in the final worktree and none was committed:

    modules/persistence/src/Lumio.GameRuntime.Persistence/Lumio.GameRuntime.Persistence.csproj
    modules/persistence/src/Lumio.GameRuntime.Persistence/packages.lock.json
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/ICanonicalCodec.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/CanonicalRecordWriter.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/CanonicalRecordReader.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/CanonicalPrimitiveWriter.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/CanonicalPrimitiveReader.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/MessagePackCanonicalCodecAdapter.cs
    modules/persistence/src/Lumio.GameRuntime.Persistence/Canonical/CanonicalBudgetExceededException.cs
    modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/Lumio.GameRuntime.Persistence.Tests.csproj
    modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/packages.lock.json
    modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/CanonicalRoundTripGoldenTests.cs
    modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/CanonicalPropertyTests.cs
    modules/persistence/tests/Lumio.GameRuntime.Persistence.Tests/CanonicalBudgetTests.cs

Build-generated bin/ and obj/ directories under those provisional project directories were also removed. No existing README, generated contract source, shared manifest, architecture source, or Workflow object was modified.

## TDD and verification disposition

No GREEN result is claimed. The only card-valid RED evidence is the missing project probe above; any provisional local codec tests are deliberately not accepted as card evidence because the latest card forbids the implementation they exercised, and all such files were removed.

All R-00141 acceptance items remain unverified and must stay not_started:

1. T20.S01 primitive golden execution by a runtime codec.
2. T20.S02 generated-record/property round trips and strict rejection.
3. T20.S03 checked encode/decode/depth budgets and pre-allocation rejection.
4. T20.S04 focused module test execution.
5. T20.S05 executable canonical layer.
6. T20.S06 replacement of the obsolete MessagePack wording with an upstream executable LumioBinV1 consumer.
7. T20.S07 focused tests and package-surface scan.
8. T20.S08 implementation commit.

The required next prerequisite is an upstream architecture publication of the executable LumioBinV1 codec, or an explicit card/authority revision that authorizes a consumer implementation. Until then, no conflict-free work in this card's file set can produce a valid deliverable.

## Delivery

    Status: BLOCKED
    Implementation commit: none
    Runtime source HEAD: ef822a76cd5586513ea6e52b3ea4f5497917bdc8
    Target-worktree files remaining: none
    Architecture source changes: none
    Workflow changes: none
    Report: C:\Work\LumioGames\LumioGameEngineArchitecture\.sdd\d005-canonical-report.md

Known gaps are the blocked executable codec publication and all unverified R-00141 acceptance items. No durability tier, schema, error identity, or third-party codec was inferred by this report.

