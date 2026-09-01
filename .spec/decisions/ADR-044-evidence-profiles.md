# ADR-044: Evidence Profiles

- **Status**: Historical · Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (profile publisher), `LumioCoreEngine` (`evidence-generator`, `runtime-verifier` consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; no existing required field, enum or ID changes)
- **Relation**: Gives `core-engine-manifest.evidenceSet` (frozen in shape by [ADR-018](ADR-018-coreengine-manifest-canonicalization.md)) its byte-level meaning; digests follow [ADR-041](ADR-041-canonical-digest-profiles.md)'s framing but **not** its canonicalization, for the reason in §2.

## Context

ADR-018 made `evidenceSet` three digest-bound references — `sbom`, `license`, `provenance`, each `{format, digest}` — and deliberately did not embed the evidence itself. `format` is schema-typed as `^[A-Za-z][A-Za-z0-9.-]{1,31}$`: a free string. So a `CycloneDX` in one repository and a `cyclonedx` in another both validate, neither carries a spec version, and nothing says what the digest is even over.

Four things are undetermined, and each one lets a package pass one verifier and fail another:

1. **Which format at which version, and under which media type.** "CycloneDX" is not a format; CycloneDX 1.4 and 1.6 are different documents.
2. **What the digest covers.** The raw file bytes, or some canonicalization of the JSON? ADR-041 just froze a canonical form for our own documents, which makes the wrong answer here very tempting.
3. **Where evidence files may live, and whether the ArtifactIndex and the ManifestBody must agree about them.**
4. **How far a verifier validates**, and where a licence decision stops being an evidence check and becomes a trust decision.

The card also forbids `LumioCoreEngine` from resolving any of this from a tool's default version, a file extension, or a free string.

## Decision

### 1. Three profiles, each pinned to a spec version and a media type

| `evidenceSet` member | `format` | `specVersion` | `mediaType` |
| --- | --- | --- | --- |
| `sbom` | `CycloneDX` | `1.6` | `application/vnd.cyclonedx+json` |
| `license` | `SPDX` | `2.3` | `application/spdx+json` |
| `provenance` | `SLSA-v1` | `1.0` | `application/vnd.in-toto+json` |

`format` is now a closed set of exactly these three spellings, case-sensitive. The `format` string **implies** the `specVersion` and `mediaType`; they are not carried in the manifest, so `evidenceSet` keeps its published shape and no required field changes.

This is what makes the card's acceptance work: swap Syft for another SBOM generator, cargo-about for another licence tool, or one attestor for another, and as long as the output is CycloneDX 1.6 / SPDX 2.3 / in-toto SLSA 1.0 JSON, the ManifestBody and the verifier produce the same result. The tool is not the contract; the profile is.

### 2. The digest is over the raw file bytes — never a canonicalization

`evidenceSet.<kind>.digest` is the SHA-256 of the evidence file's **bytes as published**, unmodified.

This is deliberately *not* ADR-041's `CanonicalJsonV1`. Evidence documents are third-party tool output; re-serializing them to compute a digest would mean every verifier must reproduce our canonicalization of someone else's JSON, and any tool that emits a member order or a number spelling our canonicalizer normalizes would produce a digest that disagrees with the file anyone can hash. Raw bytes have exactly one reading and can be checked with `sha256sum`.

ADR-041 governs documents **we** define. This one governs documents we merely reference. The framing is the same (prefix-free SHA-256); the input is not.

### 3. Evidence lives in the ArtifactIndex, and the two must agree

Every evidence file is an `ArtifactIndex` entry whose `kind` is `Sbom`, `License` or `Provenance` — kinds the index schema already carries — with a `path` under `evidence/`.

**Coverage is bidirectional and exact:**

- every `evidenceSet` member's digest must equal the `sha256` of the index entry of the matching kind; and
- the index must contain **exactly one** entry of each of the three kinds.

An index carrying an evidence file the manifest does not reference, or a manifest referencing evidence the index does not carry, is **incomplete coverage** and is rejected. Half-covered evidence is worse than none: it looks audited.

### 4. The Loader validates digests; it does not read the evidence

At the load boundary, verification is **`DigestOnly`**: the file's bytes hash to the declared digest, and coverage per §3 holds. A verifier does **not** parse the SBOM, walk the dependency graph, or evaluate licence text at load time.

- Missing evidence, or an index missing a required kind → `EvidenceMissing` (1017).
- A digest that does not match the file's bytes → `EvidenceDigestMismatch` (1018).
- A `format` outside §1's closed set → `EvidenceMissing` (1017): a profile the verifier does not recognise is evidence it does not have.

**Where a licence decision belongs.** Rejecting a package because a dependency's licence is unacceptable is a **trust policy** decision, not an evidence check: it maps to `TrustPolicyRejected` (1015) and it happens in the trust layer of ADR-042, against a policy, at the operator's discretion. It is not an `Evidence*` code and it is not the Loader's judgement. Two organisations may accept different licences from the same, perfectly valid, byte-identical SBOM — so licence acceptability cannot live in a contract that must produce one answer.

Semantic validation of evidence content (does the SBOM actually list every linked library?) is a supply-chain pipeline concern, out of scope here, and explicitly **not** a load-time gate.

## Contract

`schemas/evidence-profile.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules:

- The three profiles equal §1 exactly — kind, `format`, `specVersion`, `mediaType`, `digestObject = RawBytes`, `validation = DigestOnly`, `pathPrefix = evidence/`.
- `core-engine-manifest.evidenceSet.<kind>.format` is one of §1's three spellings.
- Every published vector is re-evaluated by the gate against §3 and §4: a vector's declared outcome and `rejectReason` must be what the rules produce.

## Failure semantics

`EvidenceMissing` (1017) and `EvidenceDigestMismatch` (1018) as in §4, both already registered; the package never reaches the Loader. Licence rejection is `TrustPolicyRejected` (1015) and belongs to the trust layer. No new `ErrorCode` is introduced.

## Alternatives

Canonicalizing evidence JSON before digesting was rejected — see §2; it would make our canonicalizer a dependency of every third-party tool's output.

Carrying `specVersion` and `mediaType` in `evidenceSet` was rejected: it would add required fields to a published schema, which the `schemas/` change rule prices at a new baseline id, to express something a closed `format` set already determines.

Accepting a range of spec versions (for example "CycloneDX 1.4 or later") was rejected: a range means two verifiers can accept different documents, which is the failure this card exists to prevent. Widening the range later is a new ADR and a cheap one.

Letting the verifier evaluate licences at load time was rejected: licence acceptability is an operator policy that legitimately differs between organisations, so a contract that must yield one answer cannot contain it.

Allowing evidence outside `evidence/`, or evidence absent from the ArtifactIndex, was rejected: evidence that is not in the artifact set is not covered by the Artifact Set Digest, so it could be swapped without changing PackageIdentity.

## Compatibility and migration

Additive. No existing schema, required field, enum, ID or state changes meaning, so the `LGE-V1.4-2026-08-27` baseline id and every repository mirror stay valid. The published fixtures already use `CycloneDX`, `SPDX` and `SLSA-v1`, so §1 codifies the existing spellings rather than renaming them; `cemanifest/linux-server` and `artifact/index` gain agreeing digests under §3. `LumioCoreEngine` deletes any interpretation of evidence drawn from a tool's default version, a file extension or a free string.

## Verification

Fixtures `evidence/profile` (positive), `evidence/profile-frozen-set` (a profile whose format/version/media-type triple is not §1), `evidence/vector-outcome-mismatch` (a vector whose declared outcome is not what §3–§4 produce), plus vectors covering: valid; a missing kind; a digest that disagrees with the index entry; an unrecognised `format`; and incomplete coverage in each direction (index entry with no manifest reference, manifest reference with no index entry).
