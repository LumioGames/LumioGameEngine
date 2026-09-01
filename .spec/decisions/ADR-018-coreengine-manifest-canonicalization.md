# ADR-018: CoreEngineManifestBody Canonicalization and Detached SignatureEnvelope

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioCoreEngine` (`manifest`, `signing`), consumed by `Server`/`Client` release tooling
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

The CoreEngine package needs a manifest that is reproducible (same inputs, same bytes), verifiable (digest-addressed) and signable. Embedding the signature or timestamps inside the manifest creates a self-reference cycle: the signed bytes would change once the signature is written into them. The release layer additionally needs a precise, digest-level reference to the exact CoreEngine package a game release was validated against.

## Decision

- **CoreEngineManifestBody** (`core-engine-manifest.schema.json`) is a canonical, self-reference-free document: package identity fields, `sourceLock` (NativeCore/VoxelEngine commits + source tree digest), `buildPlanDigest`, `toolchain`, `featureSet`, `targetProfileId`/`targetProfileDigest`, `abiIdentity`, `capabilitySet`, `artifactIndexDigest`, `artifactSetDigest`, an `evidenceSet` (SBOM, License, Provenance — bound by digest and media format, never embedded) and `generator`. It contains no signature, no timestamp and no field derived from its own bytes.
- **Canonicalization**: the Manifest Digest is the SHA-256 of the canonical JSON bytes of the ManifestBody (sorted keys, ASCII, minimal separators — the same canonical form `tools/lumio_contract.py canonical` emits). Any signer or verifier must reproduce these bytes exactly.
- **SignatureEnvelope** (`signature-envelope.schema.json`) is a detached document: `payloadType` (`CoreEngineManifestBody`), `payloadDigest` (the Manifest Digest), `algorithm`, `keyId`, `trustDomain` (`Production`/`Staging`/`Test`), `signature`, `signedAt` and optional certificate chain and transparency-log reference. All non-deterministic fields live only here.
- **ArtifactIndex** (`artifact-index.schema.json`) is the per-file inventory (canonical path, kind, size, SHA-256), produced exclusively by `platform`; the Artifact Set Digest is the digest over the canonicalized index and is what ManifestBody references.
- **ReleaseManifest** gains a required `coreEnginePackage` block: `packageId`, `manifestDigest`, `artifactSetDigest`, `abiIdentity`, `targetProfileDigest` (+ optional capability-set and envelope digests). `coreEnginePackage.abiIdentity` must equal the release's `coreEngineAbi`.

## Contract

Schemas `core-engine-manifest`, `signature-envelope`, `artifact-index` and the extended `release-manifest`; semantic rules: unique artifact paths, `Native` capability required for a NativeLibrary package, Production trust domain rejects test keys, release/CoreEngine ABI identity equality.

## Failure semantics

Digest mismatch anywhere in the chain (artifact → index → set digest → manifest digest → envelope payload digest) is a stable verification error (`ManifestDigestMismatch`, `ArtifactDigestMismatch`, `EvidenceDigestMismatch`, `SignatureInvalid`) and the package never reaches the Loader.

## Alternatives

Embedding signatures in the manifest (like the game ReleaseManifest's inline `signature`) was rejected for the self-reference cycle; the game manifest predates this decision and its inline signature covers a different, non-canonicalized flow. Embedding SBOM bytes was rejected: evidence is digest-bound so the body stays small and reproducible.

## Compatibility and migration

New schemas are additive in `LGE-V1.1-2026-08-27`. `release-manifest` adds a required block — both game fixtures were updated in the same baseline; downstream release tooling must populate the block from the CoreEngine package registry before adopting V1.1.

## Verification

Fixtures `cemanifest/linux-server`, `cemanifest/missing-native`, `envelope/ed25519`, `envelope/production-test-key`, `artifact/index`, `artifact/duplicate-path`, `release/a-1.1`, `release/boe-2.1`, `release/coreengine-abi-mismatch`.
