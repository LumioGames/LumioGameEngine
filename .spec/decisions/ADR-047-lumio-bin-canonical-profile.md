# ADR-047: LumioBinV1 Binary Canonical Profile

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (profile publisher); `LumioGameRuntime` (persistence format consumer), `LumioVoxelEngine` (payload producer/consumer), `LumioServer` (host durability), `LumioGame` (domain payload schemas)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; no existing schema, required field, enum, ID or Golden changes meaning)
- **Relation**: supplies the primitive layer that [ADR-010](ADR-010-persistence-config.md) §Contract referred to as "the same canonical codec rules" and that [ADR-035](ADR-035-voxel-snapshot-payload.md) assumed when it froze voxel payload ordering, offsets and hashing. It sits beside `CanonicalJsonV1` ([ADR-041](ADR-041-canonical-digest-profiles.md)), not above or below it: ADR-041 is the form for canonicalizable JSON documents, this is the form for binary payload bytes. Section 4 also completes the `snapshot-header.checksum` domain (the "B profile") that ADR-010 left as one undocumented sentence.

## Context

ADR-010 says domain payload schemas "reference the same canonical codec rules". Measured on `origin/main = c350ec6`, there was nothing for that sentence to point at:

```text
$ git grep -lniE 'messagepack|msgpack' -- schemas .spec/decisions packages   → zero hits
$ endianness hits are all native-ABI context (ADR-006 / ADR-020 / ADR-040); the
  persistence domain has none
```

The only canonical form the architecture source had frozen was `CanonicalJsonV1`, whose `encoding` is `AsciiEscaped` — a JSON **text** form. A binary voxel chunk payload cannot be carried by it.

Meanwhile ADR-035 froze the voxel payload very tightly: `chunkOrder` = `CoordXYZAscending`, per-entry `byteOffset`/`byteLength` contiguous, ascending and summing to `payloadLength`, `payloadHash` = the SHA-256 of the canonical bytes, `determinism` = `SameCutSameBytes`, and in its own words two encodes of one cut that differ in bytes are a fatal contract violation. It fixed **domain ordering and framing** and said nothing about the **primitive layer**: integer width, byte order, how a string or a byte array carries its length, whether a struct is padded, what order fields appear in.

Two facts follow, and together they are the defect. First, these payload bytes are **public** — ADR-035 says "every conforming encoder", so the bytes cross repository boundaries. Second, the public thing had no authority. Every implementation would have to invent a primitive encoding, which is exactly the "private de-facto contract" failure that ADR-017, ADR-023 and ADR-040 exist to prevent, and exactly the shape of the D-1 replication-payload gap. `LumioGameRuntime` could not even decide whether to keep its `MessagePack 3.1.8` dependency without this ruling.

## Decision

### 1. `LumioBinV1` is the binary canonical form, defined here

- **Byte order is little-endian**, everywhere, with no per-field override.
- **Integers are fixed width**: `u8`, `u16`, `u32`, `u64` unsigned, and `i32`, `i64` two's complement. A value outside its declared width's closed range is a build-time refusal, never a truncation or a wrap.
- **Strings are UTF-8 with a `u32` byte-length prefix.** The prefix counts *bytes*, not code points and not UTF-16 code units. There is no terminator and no BOM.
- **Byte strings carry a `u32` byte-length prefix** and then the raw bytes.
- **Arrays carry a `u32` element count prefix**, then the elements in document order. Canonical ordering *within* a domain (ADR-035's `CoordXYZAscending`, ADR-041's normalization steps) is the domain's business and runs before encoding; `LumioBinV1` never reorders anything.
- **Structs are the concatenation of their fields in schema declaration order, with no padding**, no alignment, no field tags and no length prefix. Declaration order — not member-name order — is the rule, because a JSON-shaped input cannot carry it and a consumer cannot infer it.
- **Missing and unknown struct fields are rejected.** The field set is closed, matching the `additionalProperties: false` discipline the schema set already uses.
- **There are no floating-point types.** A domain that needs one declares its own rule in its own ADR; this profile refuses `f32`/`f64` as an unknown layout kind rather than freezing an IEEE-754 formatting rule nothing yet needs.

### 2. Digests over `LumioBinV1` bytes are prefix-free SHA-256

`SHA-256` over the encoded bytes, with **no prefix, salt, length framing or domain string** — the same construction ADR-041 §2 froze for `CanonicalJsonV1`, so the two profiles do not fork the digest algorithm.

This profile deliberately does **not** reuse ADR-041's `PrefixFreeOverEncodedBytes` spelling for `framing`. Two independent clean-room implementations read that name as an instruction to *add* a prefix — a length prefix being the standard way to make a byte string prefix-free — and the first, working from an earlier draft, prepended one: it reproduced **6/6 Golden byte strings and 0/6 digests**, silently. That is the D-8 failure mode exactly: the rule lives in a name while the consumer executes data. So `framing` is published as `None`, which admits no such reading, and `digestInput` (`EncodedBytesOnly`) states the input positively. That the encoding is itself prefix-free — which is what makes the unframed digest sound — is a **property of the form argued here**, not an operation the digest performs, so it is not published as one.

For the same reason the profile publishes `vectorSemantics`: `error` is `Normative` and `case` is a `HumanLabel`. Several cases share one error (both malformed-hex spellings are a `TypeMismatch`, and `UnsignedNegative` is an `IntegerRangeOverflow`), so a consumer keying conformance on the more descriptive-looking `case` invents error names that do not exist in the profile. A clean-room reader had to guess which field was normative; now it does not.

### 3. The profile publishes Goldens *and* rejections, both self-verifying

`schemas/lumio-bin-profile.schema.json` describes the published record: the §1 form parameters, the digest construction, the spelling each layout kind uses inside a vector's `value`, and two vector lists. The architecture gate **re-encodes every Golden from its `layout` and `value`** and recomputes its digest, and it **re-runs every rejection vector and requires the encoder to refuse it with the declared `error`**. A vector cannot rot into a lie, and "the encoder failed somehow" does not count as conformance.

Goldens cover: every integer width including both signed extremes; a UTF-8 string whose byte length differs from its code-point count (10 code points, 16 bytes, including an astral character); a length-prefixed byte string; an array count prefix; a struct whose declaration order is deliberately not its member-name order and whose widths do not align (an encoder that sorts members, or pads to natural alignment, produces different bytes and reproduces no other vector); and a nested composition of struct-in-array-in-struct including empty string and empty byte-string edges.

Rejections cover: integer range overflow; a negative value against an unsigned width; a fractional number; **an integral number spelled as a float** (`1.0`); a boolean against an integer; a string against an integer; **malformed hex in a byte string** (odd length, upper case, and non-hex characters); an unknown layout kind; a missing struct field; an unknown struct field.

Several of those exist because the clean-room checks found them missing. `1.5` is refused under both a spelling-based and a value-based reading of the integer rule and therefore discriminates nothing; `1.0` is the case that separates them. Malformed hex had no published error name at all, so a clean-room implementation invented one — two conforming encoders would have disagreed silently.

Two `valueEncoding` members exist for the same reason. `integers` is spelled `JsonIntegerLiteralsNoFractionOrExponent` rather than the ambiguous `JsonIntegerNumbers`, because the rule is about the **literal spelling, not the value**: `1.0` and `1e2` are integral in value and refused as literals, and a reader who assumed the value-based reading would have passed every published vector while accepting `1e2` — the exponent spelling being one a JSON serializer cannot emit, so no Golden can pin it. `integerPrecision` (`ExactArbitraryPrecision`) is published because a double-backed JSON reader — any stock JavaScript `JSON.parse`, the most likely second implementation of a profile whose vectors are spelled in JSON — rounds the `u64` Golden's `18446744073709551615` to 2⁶⁴ and then rejects a *valid* vector. That Golden makes the requirement fail loudly instead of silently.

### 4. The snapshot-header checksum domain (the B profile) is completed

`snapshot-header` requires both `checksum` and `hash`, and the only published authority for the pair was a single generated line with no domain tag, no worked example and no statement of how the two differ. It is now a generated document with all three:

- **`hash` covers the payload bytes**: `SHA-256(payload)`, where the payload is the uncompressed domain bytes — encoded under `LumioBinV1` when those bytes are binary. It says nothing about the header.
- **`checksum` covers the header** with both digest members removed, in a structurally separated domain: `SHA-256(CanonicalJsonV1({"digestDomain":"SnapshotHeaderV1","header":<header minus checksum and hash>}))`. Omitting `checksum` is what makes the value computable at all; omitting `hash` means a payload re-hash does not force a header rewrite.

The domain tag is a member of the digest input, exactly as in ADR-041 §2, so a B-profile digest cannot collide with an A-profile one. `tools/lumio_contract.py` now **recomputes `checksum`** for every `snapshot-header` fixture; the rule is enforced by something a fixture can fail rather than by a sentence.

### 5. `ADR-010` now points at something

ADR-010's Contract clause is amended by reference (its own text is `Accepted` and unchanged): "the same canonical codec rules" means `CanonicalJsonV1` for canonicalizable JSON documents and `LumioBinV1` for binary payload bytes.

## Contract

`schemas/lumio-bin-profile.schema.json`, registered in `schemas/index.json` as P0 `Architecture`; published as `packages/binary/lumio-bin-profile.json` with `consumers` registered in `packages/index.json`. Semantic rules in `tools/lumio_contract.py`:

- the published profile is re-derivable from the generator's frozen vectors (a hand-edited publication is rejected);
- `binaryForm`, `digestAlgorithm`, `valueEncoding` and `vectorSemantics` equal the §1–§2 freeze;
- `goldens` cover every frozen case, and each one's `bytesHex` and `sha256` are recomputed from its `layout` and `value`;
- every `rejections` entry is actually refused by the encoder, with the declared `error`;
- `snapshot-header.checksum` equals the §4 recomputation.

The `CanonicalSerializer` artifact publishes the form parameters, the integer width table and the vector identifiers with their digests to Rust and C#; the vectors themselves stay in the published JSON, which is the single place a conformance test reads them from.

## Failure semantics

An unencodable value produces no bytes and therefore no digest: it is a build-time failure, never a truncated, wrapped, padded or reordered encoding. Two conforming encoders that disagree on one byte for the same layout and value are a fatal contract violation, not a tolerable variance — this is ADR-035's `SameCutSameBytes` rule reaching down to the primitive layer that previously had no rule at all. A snapshot header whose `checksum` does not recompute is rejected before the payload is examined.

## Alternatives

**MessagePack (or CBOR, or any third-party codec)** was rejected. It brings a dependency to audit in seven repositories and an AOT cost on Unity/HybridCLR, and — decisively — it does not actually deliver byte determinism: the format admits multiple valid encodings of the same value (integer width selection, str/bin choice, map ordering), so "canonical MessagePack" would itself have to be specified here. Having specified it, the third-party dependency buys nothing. `LumioGameRuntime`'s `MessagePack 3.1.8` dependency is consequently ruled removable.

**"Each domain defines its own primitive encoding"** was rejected outright. The payload bytes are public by ADR-035's own wording; letting each domain invent a primitive layer reproduces the underlying D-1 defect — a public artifact with no authority — and guarantees the divergence this ADR exists to prevent.

**Varint / LEB128 length and integer encoding** was rejected. It saves bytes on small values and costs determinism and implementation simplicity: a varint admits non-minimal encodings, so a canonical profile must then forbid them and test for them. Fixed width has one spelling per value by construction.

**Tagged or self-describing fields** (field ids, type tags, TLV) were rejected: the schema already names and orders every field, so tags would be a second, redundant source of truth that could disagree with the schema. Closed field sets in declaration order keep the schema the single authority.

**Freezing a float format now** was rejected as premature. No frozen contract currently needs one, and an IEEE-754 rule adopted speculatively (NaN payload bits, signed zero, subnormal handling) is the kind of detail that is wrong until a real consumer constrains it. `floats = None` makes the absence explicit and machine-checkable rather than implicit.

**Adding the snapshot checksum as a seventh `CanonicalJsonV1` digest domain** was rejected: it would extend the frozen `digest` enum of `canonical-digest-profile.schema.json`, and an enum change to a published schema is a baseline event under the `schemas/README.md` change rule. The B profile is documented in its own generated authority instead, with the same structural domain separation and no change to an existing schema.

## Compatibility and migration

Additive. No existing schema, required field, enum, ID, state or published Golden changes meaning, so `LGE-V1.4-2026-08-27` and every repository mirror stay valid, and no new BaselineId is required.

Two consequences downstream repositories must absorb:

- The `CanonicalSerializer` artifact gains the `LumioBinV1` surface and the completed `CHECKSUM_DOMAIN.md`, so its `outputHash` moves; `compilerHash` moves for every artifact because the generator and validator sources changed. Consumers that pin an `outputHash` re-pin. No published byte vector of any *existing* profile changes.
- `fixtures/valid/snapshot-active.json` gains a real `checksum` in place of its `4b4b…` placeholder. This is a fixture correction of the same kind ADR-041 made when two placeholder digests became real, not a contract change: the field, its type and its position are unchanged, and the value was never a published Golden.

`LumioGameRuntime` may now remove `MessagePack 3.1.8` and encode persistence and snapshot payload bytes under this profile. `LumioVoxelEngine` gains the primitive layer its ADR-035 payload always assumed. The D-1 replication state payload (ReplicationEnvelope typed bodies) is *not* implemented here — its required-field extension and `MessageType` addition are baseline events and ride the V1.5 batch — but its encoding is now decided: `LumioBinV1`.

## Verification

- `lumiobin/profile` (positive, `fixtures/valid/lumio-bin-profile.json`) — every Golden re-encodes byte-for-byte from its layout and value, every digest recomputes, and every rejection vector is refused with its declared error.
- `lumiobin/golden-bytes-mismatch` (negative) — a Golden whose `bytesHex` swaps the two bytes of the `u16` member, the classic little-endian mistake, is rejected.
- `lumiobin/golden-digest-mismatch` (negative) — a Golden whose `sha256` is not the digest of its bytes is rejected.
- `lumiobin/rejection-not-rejected` (negative) — a "rejection" vector the encoder actually accepts is rejected.
- `snapshot/checksum-mismatch` (negative, `fixtures/invalid/snapshot-header-checksum-mismatch.json`) — a header whose `checksum` disagrees with the recomputed B-profile digest is rejected; `snapshot/active` proves the positive case.
- **Clean-room reproduction** (the D-8 acceptance bar): an implementation built from `packages/binary/lumio-bin-profile.json` alone, with no access to this ADR, the generator or the fixtures, reproduces every Golden byte-for-byte and every digest, and refuses every rejection vector for the declared reason. Two independent runs were performed. The first reproduced 6/6 bytes, 6/6 digests and 7/7 rejections; its report on what remained guessable produced `digestInput`, `integerPrecision` and four added rejection vectors. The second, against the revised profile, reproduced 6/6 bytes, 6/6 digests and 11/11 rejections — and reported that it had still needed two *unforced judgments*: ranking `digestInput` above the then-`PrefixFree…` spelling of `framing`, and treating `error` rather than `case` as normative. Both were reader guesses the data did not settle, so `framing` became `None`, `vectorSemantics` was published, `integers` was respelled, and a non-hex rejection vector was added. Conformance is defined as reproducing these vectors from the published file, so a profile that a competent reader can only reproduce by guessing correctly does not meet the bar.
