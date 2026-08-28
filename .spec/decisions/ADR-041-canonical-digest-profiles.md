# ADR-041: Canonical and Digest Profiles

- **Status**: Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (profile publisher), `LumioCoreEngine` (`manifest`, `platform`, `runtime-verifier` consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; no existing required field, enum or ID changes)
- **Relation**: Completes the canonicalization clause of [ADR-018](ADR-018-coreengine-manifest-canonicalization.md) without rewriting it; publishes through the artifact rules of [ADR-023](ADR-023-generated-contract-artifact.md) / [ADR-039](ADR-039-contract-runtime-artifact.md).

## Context

ADR-018 fixed the Manifest Digest as "SHA-256 of the canonical JSON bytes of the ManifestBody (sorted keys, ASCII, minimal separators)" and said the Artifact Set Digest is "the digest over the canonicalized index". Four things were left underdetermined, and every one of them is a place where two independent implementations silently disagree:

1. **`artifactSetDigest` self-reference.** It is stored *inside* `ArtifactIndex`. Digesting "the canonicalized index" with the field present is impossible; with the field absent, nothing said so.
2. **`artifactIndexDigest` vs `artifactSetDigest`.** The ManifestBody requires both. Nothing distinguishes them, so a consumer could reasonably compute the same bytes twice — or invent its own split.
3. **`targetProfileDigest` and `capabilitySetDigest`.** Both are `hash256` in the schema and neither has a defined input. A capability set is an unordered concept stored as an array, so two builds of the same package can disagree purely on member order.
4. **The canonical form itself was a prose sentence.** "Sorted keys, ASCII, minimal separators" does not pin escaping, number handling, array order or unknown-member behavior, and there is no published Golden anywhere to check an implementation against. ADR-018's wording also invites treating a generic JCS library's defaults as the contract, which would make the architecture source no longer the authority.

## Decision

### 1. `CanonicalJsonV1` is the canonical form, defined here, not by reference

- **Output is always ASCII.** Every code point above `U+007E` is escaped as `\uXXXX` (astral code points as a UTF-16 surrogate pair, both escaped). The canonical byte string is therefore its own JSON string literal, which is what makes the Goldens in §4 publishable inline.
- **Object members are sorted ascending by the Unicode code points of the member name.** Every member name in a canonicalizable document matches `^[A-Za-z][A-Za-z0-9]*$`, so code-point order, UTF-8 byte order and UTF-16 code-unit order coincide; the rule is stated in code-point terms so the coincidence is not load-bearing.
- **Separators are exactly `,` and `:`** with no whitespace anywhere, including none after `[` or before `]`.
- **Arrays keep document order.** Where a digest domain needs an order-independent value, §3 sorts the array *before* canonicalization; canonicalization itself never reorders an array.
- **Numbers must be integers**, serialized with no sign for non-negative values, no leading zero, no exponent and no fraction. A non-integer number in a canonicalizable document is a canonicalization error, not a rounding decision. This removes floating-point formatting from the contract entirely.
- **Escaping inside strings** is exactly: `"` → `\"`, `\` → `\\`, `U+0008` → `\b`, `U+000C` → `\f`, `U+000A` → `\n`, `U+000D` → `\r`, `U+0009` → `\t`, every other code point below `U+0020` → `\u00XX` with lower-case hex digits, and every code point above `U+007E` → `\uXXXX` with lower-case hex digits. `/` is **not** escaped.
- **Duplicate member names and unknown members are rejected.** Every canonicalizable document schema is `additionalProperties: false`; canonicalization does not silently drop or reorder an unexpected member.

A generic JCS implementation may happen to agree on today's documents. That agreement is **not** the contract: an implementation is conformant only if it reproduces the Golden vectors in §4.

### 2. Digests are prefix-free SHA-256 over canonical bytes; domains separate structurally

Every frozen digest is `SHA-256(CanonicalJsonV1(<digest input>))` with **no prefix, salt or length framing** — this is what ADR-018 already fixed for the Manifest Digest and this ADR does not fork it.

Domain separation is structural: each digest input defined below is a JSON **object carrying a mandatory `digestDomain` member** naming the domain and its version. The single exception is `manifestDigest`, whose input is the ManifestBody itself, frozen by ADR-018 before this ADR; the ManifestBody is `additionalProperties: false` and has no `digestDomain` member, so it cannot collide with any domain object defined here.

### 3. The four digest inputs

| Digest | Input value |
| --- | --- |
| `manifestDigest` | The `CoreEngineManifestBody` document itself. Unchanged from ADR-018. |
| `artifactSetDigest` | `{"digestDomain":"ArtifactSetV1","indexVersion":<n>,"targetProfileId":<id>,"entries":[<the index entries, sorted ascending by `path`>]}` — the ArtifactIndex **with its own `artifactSetDigest` member omitted**, wrapped in the domain object. This is the self-reference rule. |
| `artifactIndexDigest` | `{"digestDomain":"ArtifactIndexV1","index":<the complete ArtifactIndex document, `artifactSetDigest` included, `entries` sorted ascending by `path`>}` — the digest of the published index *file*, which is what makes it distinct from the set digest above. |
| `targetProfileDigest` | `{"digestDomain":"TargetProfileV1","profile":<the complete TargetProfile document>}` |
| `capabilitySetDigest` | `{"digestDomain":"CapabilitySetV1","capabilities":[<the capability ids, sorted ascending by code point>]}` |

Sorting rules used above, and nowhere else:

- `entries` sort ascending by the `path` member. `path` matches `^[A-Za-z0-9][A-Za-z0-9._/-]{0,255}$`, so it is ASCII and code-point order is byte order. Ties are impossible: paths are unique within an index (ADR-018 semantic rule).
- `capabilities` sort ascending by code point. The array is `uniqueItems`, so ties are impossible. **A permutation of the capability set therefore produces an identical `capabilitySetDigest`** — the property a build system needs and could not previously rely on.

The empty artifact set is defined (`entries: []` canonicalizes to `[]` and has a stable digest) even though `artifact-index.entries` keeps `minItems: 1`: a shipped package always has files, but the serializer must still agree on the empty case, so it is frozen as a Golden rather than by loosening a published constraint.

### 4. Each domain publishes its normalization as data, not as prose

The sort rules of §3 are published in `digestDomains[].normalization` as an ordered list of executable steps (`path`, `op`, `by`, `collation`) — the same declaration the generator and the gate execute. `sortRule` remains as the human-readable gloss of that declaration, never as its authority.

This is not decoration. Normalization runs **before** canonicalization, and `CanonicalJsonV1` itself never reorders an array (`arrayOrder = DocumentOrder`), so an implementation that reads the form parameters but misses the sort produces different bytes **and raises nothing**. Three downstream repositories independently implemented from the form block alone and each reproduced exactly 6 of 8 Goldens, failing exactly the two permutation vectors — the other six inputs are already sorted, so the omission does not show. Publishing the rule as prose while publishing `omitMembers` as data made that outcome the predictable one: a consumer correctly executes the omit and silently skips the sort.

It also closes a contradiction in §5 below: conformance is defined as reproducing the Goldens, so a published profile from which the Goldens cannot be reproduced makes the conformance criterion unreachable.

### 5. Golden vectors are published, and they are self-verifying

`schemas/canonical-digest-profile.schema.json` describes a published record carrying the §1 form parameters, the §3 domain table with its §4 normalization, and a list of Golden vectors. Each vector is a triple: the input value, the exact canonical byte string, and its SHA-256. The architecture gate **recomputes** both the bytes and the digest from the input value, so a Golden cannot rot into a lie.

The vector set covers, at minimum: the empty / single / multi artifact set; two entry orderings that must collapse to one digest; two capability orderings that must collapse to one digest; the escaping boundary (quote, backslash, the five shorthand controls, another C0 control, non-ASCII BMP and an astral code point); an integer boundary; and a schema-version change that must change the digest.

### 5. `artifact-index.artifactSetDigest` is now enforced, not documented

`tools/lumio_contract.py` recomputes `artifactSetDigest` from the index's own entries and rejects a mismatch. The ambiguity is closed by a rule a fixture can fail, not by a paragraph.

## Contract

`schemas/canonical-digest-profile.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules:

- **Profile**: the canonical form parameters equal the §1 freeze; the domain table equals the §3 set exactly and each domain publishes the §4 normalization; every Golden's `canonicalBytes` equals the re-canonicalization of its `input`, and its `sha256` equals the digest of those bytes; the required coverage cases of §4 are all present.
- **ArtifactIndex**: `artifactSetDigest` equals the §3 recomputation (existing unique-path rule unchanged).

## Failure semantics

A canonicalization error (non-integer number, duplicate member, unknown member) produces no bytes and therefore no digest; it is a build-time failure. A digest mismatch anywhere in the chain keeps the ADR-018 mapping to `ManifestDigestMismatch` / `ArtifactDigestMismatch` / `EvidenceDigestMismatch` / `SignatureInvalid` and the package never reaches the Loader.

## Alternatives

Prefixing each digest with domain bytes was rejected: ADR-018 already froze `manifestDigest` as a prefix-free SHA-256 over canonical bytes, and forking the construction for the other four would leave two digest algorithms in one chain. The structural `digestDomain` member achieves the same separation inside the form ADR-018 already fixed.

Defining `artifactSetDigest` over the entries array alone was rejected: ADR-018 says the digest is over the canonicalized *index*, and dropping `indexVersion` / `targetProfileId` would let two indexes for different targets share a set digest.

Making `artifactIndexDigest` an alias of `artifactSetDigest` was rejected: the ManifestBody requires both, and a consumer that finds two names for one value will eventually pick one and drop the other. The index digest covers the published file including its self-declared set digest; the set digest covers the artifact set.

Relaxing `artifact-index.entries` to `minItems: 0` so the empty case could be a fixture was rejected: it would broaden a published constraint to serve a serializer edge case that a Golden covers just as well.

Deferring to a JCS library's defaults was rejected outright — it is the failure this card exists to prevent, and it would move the authority for the canonical form out of the architecture source.

## Compatibility and migration

Additive only. No existing schema, required field, enum, ID or state changes meaning, so the `LGE-V1.4-2026-08-27` baseline id and every repository mirror stay valid. Two published fixtures gain real digests in place of placeholders (`artifact/index` and the matching `artifactSetDigest` in `cemanifest/linux-server`), which is a fixture correction, not a contract change. The `CanonicalSerializer` artifact gains the profile and therefore a new `outputHash`; consumers that pin an `outputHash` re-pin. `LumioCoreEngine` deletes any local canonicalization or digest-domain decision and consumes the published profile.

## Verification

Fixtures `canonical/profile` (positive), `canonical/missing-normalization` (a domain table stripped of its machine-readable normalization — the exact defect three repositories hit), `canonical/golden-digest-mismatch` (a Golden whose `sha256` does not match its bytes), `canonical/golden-bytes-mismatch` (a Golden whose `canonicalBytes` is not the canonicalization of its input), `canonical/missing-domain` (the domain table missing a frozen domain), and `artifact/set-digest-mismatch` (an ArtifactIndex whose `artifactSetDigest` disagrees with its entries), alongside the existing `artifact/index` and `artifact/duplicate-path`.
