# ADR-020: TargetProfile / PackagingProfile / LoadBackend Orthogonalization

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioCoreEngine` (`platform`), consumed by `composition`, `manifest`, `loader`
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

"Platform" conflated three independent axes: what machine the binary runs on, how the artifact set is packaged for a host, and how the host loads native code. Flat platform names (e.g. "iOS", "Linux Server") cannot express that iOS forbids `dlopen` while sharing the same ABI, or that one OS/arch pair ships in several packaging shapes.

## Decision

Split the axes and make each explicit in the public contract:

- **TargetProfile** (`target-profile.schema.json`): `os`, `arch`, `abiRuntime` (e.g. glibc, musl, bionic, darwin), `minOsVersion`, `toolchainTriple`, `pointerWidth`, `endianness` — the compilation/runtime environment. Identified by `targetProfileId` and pinned by `targetProfileDigest`.
- **LoadBackend** (required field of TargetProfile): `DynamicLibrary`, `StaticLink` or `NoNative`. iOS-class targets declare `StaticLink`; PureHeadless hosts declare `NoNative` and take the no-Loader path — it is a distinct backend, not a Loader mode.
- **PackagingProfile** (field of TargetProfile): how artifacts are laid out for the host (`LooseFiles`, `Archive`, `EmbeddedInApp`).
- One TargetProfile is referenced by `BuildPlan`, `ArtifactIndex`, `CoreEngineManifestBody` and `PackageIdentity`; the Loader refuses a package whose TargetProfile digest does not match the host (`TargetProfileMismatch`).

## Contract

`target-profile.schema.json` with `loadBackend` required; `core-engine-manifest.schema.json` references `targetProfileId` + `targetProfileDigest`; ErrorCode `TargetProfileMismatch` (1019).

## Failure semantics

A missing or mismatched TargetProfile fails at manifest validation or Loader preflight, never at symbol-bind time. `NoNative` hosts reject any package containing a `NativeLibrary` artifact.

## Alternatives

A flat platform enum was rejected: every new packaging shape would mint a fake "platform" and the Loader could not reason about load capability separately from CPU/OS. Free-form host strings were rejected as unverifiable.

## Compatibility and migration

Additive in `LGE-V1.1-2026-08-27`. The P0 slice pins exactly one profile (`linux-server-x86_64-glibc`, `DynamicLibrary`, `LooseFiles`); adding a profile is data, not schema change.

## Verification

Fixtures `target/linux-server` (positive) and `target/missing-load-backend` (failure). Manifest fixtures reference the profile by id and digest.
