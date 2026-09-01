# ADR-042: Signature and Trust Profile

- **Status**: Historical · Draft (targets the next Implementation Baseline; additive within `LGE-V1.4-2026-08-27`)
- **Owner**: `LumioGameEngineArchitecture` (profile publisher), `LumioCoreEngine` (`signing`, `runtime-verifier` consumers)
- **Baseline**: `LGE-V1.4-2026-08-27` (additive; no existing required field, enum or ID changes)
- **Relation**: Completes the detached-envelope decision of [ADR-018](ADR-018-coreengine-manifest-canonicalization.md); signs the digests frozen by [ADR-041](ADR-041-canonical-digest-profiles.md); rejection codes come from the ID Registry family fixed by [ADR-019](ADR-019-loader-state-machine-package-identity.md).

## Context

ADR-018 made `SignatureEnvelope` a detached document carrying `payloadType`, `payloadDigest`, `algorithm`, `keyId`, `trustDomain`, `signature` and `signedAt`. Every one of those is a shape; none of them is a byte. A Test Signer and a `runtime-verifier` written from that schema cannot interoperate, because five things are undetermined:

1. **What is actually signed.** `payloadDigest` is a 32-byte digest. Signing it bare would make a signature reusable across payload types and across trust domains — a `Test` signature would verify as `Production` if the digest happened to match.
2. **How the signature and public key are encoded.** Raw? DER? PEM? JWK? base64 or hex? The schema says `minLength: 32, maxLength: 4096`, which admits all of them.
3. **Which Ed25519 it is.** RFC 8032 defines PureEdDSA, Ed25519ph (prehashed) and Ed25519ctx (with context). They produce different signatures over the same message.
4. **Where `keyId` comes from.** The existing fixture uses a hand-written string. Two implementations will not agree on a hand-written string.
5. **Which rejection wins.** A package can fail several checks at once. Without a frozen order, two verifiers report different `rejectReason` for the same input and neither is wrong.

The card that forces this also forbids `LumioCoreEngine` from deciding any of it locally. So it is decided here.

## Decision

### 1. `LumioSignatureV1`: pure Ed25519, raw, lower-case hex

- **Algorithm**: Ed25519 as defined by RFC 8032, **PureEdDSA** variant. Ed25519ph and Ed25519ctx are not part of V1 and are not accepted; there is no application prehash — the verifier passes the preimage of §2 to Ed25519 unmodified.
- **Signature encoding**: the 64 raw signature bytes as 128 lower-case hex characters. Not DER, not base64, not PEM.
- **Public key encoding**: the 32 raw public key bytes as 64 lower-case hex characters. Not PEM, not JWK, not DER, not a certificate.

Hex rather than base64 so that a byte-for-byte diff of two published vectors is readable, and so that there is exactly one spelling of every value (base64 admits padding and alphabet variants).

### 2. The signed preimage is domain separated

The signature is **never** over the bare digest. It is over this byte string, built with ASCII bytes and single `0x00` separators, in exactly this order:

```
LumioSignatureV1 <0x00> <trustDomain> <0x00> <payloadType> <0x00> <payloadDigest>
```

`trustDomain` and `payloadType` are the envelope's own values, spelled exactly as their enums. `payloadDigest` is the 64 lower-case hex characters, **not** the 32 decoded bytes — the preimage is entirely printable except the separators, which makes a mismatch diagnosable by eye.

Consequences, and the reason the preimage exists:

- A signature minted for `Test` **cannot** verify against the same payload presented as `Staging` or `Production`. The trust domain is inside the signed bytes, not merely alongside them.
- A signature over a `CoreEngineManifestBody` cannot be replayed as some future payload type that happens to share a digest.
- A bare-digest signature from any other protocol cannot be imported.

### 3. `keyId` is derived, not chosen

```
keyId = <trustDomain lower-cased> "-" <first 16 lower-case hex chars of SHA-256(raw 32-byte public key)>
```

The Test domain key of §5 is therefore `test-929ebec145050e2a`. A `keyId` is a **function of the key**, so two implementations always agree on it, a trust policy cannot silently rebind a name to a different key, and the existing rule that a `Production` domain must not use a `test-` key becomes derivable rather than a prefix convention.

### 4. Rejection priority is total and frozen

A verifier evaluates in this order and reports the **first** failure. The order is a contract: two verifiers given the same broken package must produce the same `rejectReason`.

| Order | Condition | `rejectReason` |
| --- | --- | --- |
| 1 | No `SignatureEnvelope` accompanies the package | `SignatureMissing` (1012) |
| 2 | `keyId` is not present in the trust policy for the envelope's `trustDomain` | `TrustRootUnknown` (1014) |
| 3 | The key is present but `status` is `Revoked` | `KeyRevoked` (1016) |
| 4 | Ed25519 verification of §2's preimage fails | `SignatureInvalid` (1013) |
| 5 | `signedAt` falls outside the key's `notBefore` / `notAfter` window, or any other policy constraint fails | `TrustPolicyRejected` (1015) |

The order is not arbitrary. You cannot check a signature you do not have (1 before 4), nor one whose key you cannot name (2 before 4). A revoked key is refused **before** the mathematics runs (3 before 4): revocation is a decision about the key, and a verifier must not report "the signature is fine, but…" about a key its operator withdrew. Time-window and policy checks come last (5) because they are the only ones that can legitimately differ between two policies over the same cryptographically valid signature.

### 5. The Test trust domain is one key, offline, and carries no private half

P0 scope is exactly: trust domain `Test`, a single key, offline verification. Production key management, rotation, remote signers and transparency logs are **not** decided here and stay open in `DECISIONS_PENDING` — a `Test` profile must not become a de-facto production KMS decision by being the only thing written down.

The published Test key is:

```
publicKey  026d0a4e76097c8da2e9797d7908d3bd42f441971d32b7c997395e999f97f121
keyId      test-929ebec145050e2a
```

**The architecture source publishes no private key.** Committing one would put a signing credential in the repository, and it is not needed: a downstream verifier is proved by accepting the published vectors, and a downstream *signer* is proved by having its own signature over §2's preimage accepted by a verifier built to this profile. Neither direction requires our private half.

### 6. The profile is published with self-verifying vectors

`schemas/trust-profile.schema.json` describes a published record carrying the §1 encoding, the §2 preimage rule, the §3 `keyId` rule, the §4 rejection order, the §5 trust policy, and a vector set. Each vector is a complete envelope plus its expected outcome. The architecture gate **re-derives every `keyId`, rebuilds every preimage, and runs Ed25519 over every vector**, so a vector cannot rot into a lie and the published policy cannot drift from the published vectors.

The gate's Ed25519 verifier is itself gated: it self-tests against the RFC 8032 §7.1 vectors on every run, and rejects a single-bit-flipped signature for each. The vectors committed here were produced by an independent Ed25519 implementation (the `cryptography` package, author-time only) and are verified in CI by the gate's own implementation — two independent implementations agreeing over the same bytes, which is the interoperability property this card asks for.

## Contract

`schemas/trust-profile.schema.json` (structural) plus `tools/lumio_contract.py` semantic rules:

- The signature profile, `keyId` rule and rejection order equal the §1–§4 freeze.
- Every policy key's `keyId` equals the §3 derivation from its own `publicKey`; `publicKey` is 64 lower-case hex; a `Production` domain carries no `test-` key.
- Every vector's `signature` is 128 lower-case hex; the vector's expected outcome is reproduced by actually evaluating §4 against §5's policy — an `Accept` vector must verify, and a `Reject` vector must fail at exactly the declared `rejectReason` and no earlier check.
- The gate's Ed25519 implementation reproduces the RFC 8032 §7.1 vectors and rejects each of them under a one-bit mutation.

`signature-envelope.schema.json` gains no required field; its `signature` and `keyId` values in fixtures are brought onto the profile.

## Failure semantics

Every rejection maps to the ID Registry code named in §4; the Loader consumes only a `VerifiedPackageDescriptor` and never re-derives trust itself (ADR-018/ADR-019 unchanged). A malformed envelope — wrong hex length, undecodable key — is `SignatureInvalid`, not a parse crash: a verifier must reach a stable error code for every input, including hostile ones.

## Alternatives

Signing the bare `payloadDigest` was rejected: it makes a signature reusable across trust domains and payload types, which is precisely the confusion a detached envelope invites.

Binary framing of the preimage (length-prefixed fields) was rejected in favour of `0x00`-separated ASCII: every field in the preimage is drawn from a closed enum or a hex digest, none of which can contain `0x00`, so the framing is already unambiguous, and a printable preimage is diagnosable from a log line.

DER, PEM and JWK encodings were rejected for both key and signature — they carry optional structure, multiple valid spellings of one value, and a parser surface that a stable ABI does not need. Raw bytes in lower-case hex have exactly one spelling.

Ed25519ph was rejected: an application prehash adds a second digest domain to reason about, and the payload is already a digest.

A hand-written `keyId` was rejected: it is the one field where two implementations can disagree while both believe they are correct.

Making `SignatureInvalid` outrank `KeyRevoked` was rejected: it would have a verifier run cryptography on behalf of a key its operator has withdrawn, and report a mathematical verdict about a key that should never have been considered.

Publishing the Test private key was rejected: it is a signing credential, the repository rules forbid committing one, and neither interoperability direction needs it.

## Compatibility and migration

Additive. No existing schema, required field, enum, ID or state changes meaning, so the `LGE-V1.4-2026-08-27` baseline id and every repository mirror stay valid. Two fixtures gain profile-conformant values in place of placeholders (`envelope/ed25519` and `envelope/production-test-key`), which is a fixture correction, not a contract change. `LumioCoreEngine` deletes any local decision about raw/DER, prehash, PEM/JWK or domain-separation bytes and consumes the published profile.

## Verification

Fixtures `trust/profile` (positive), `trust/keyid-mismatch` (a policy key whose `keyId` is not the derivation of its own `publicKey`), `trust/reject-order` (a rejection order that is not the §4 freeze) and `trust/vector-outcome-mismatch` (a vector whose declared outcome is not what evaluating §4 produces), alongside the existing `envelope/ed25519` and `envelope/production-test-key`. The vector set itself covers: accept; tampered signature; tampered payload digest; a signature presented under a different trust domain; an unknown key; a revoked key; `signedAt` before `notBefore`; and `signedAt` after `notAfter`.
