# ADR-014: Unity, HybridCLR and Platform Capability

- **Status**: Accepted (Implementation Baseline `LGE-V1.1-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioClient` (platform adapter), `LumioGameRuntime` (capability contract)
- **Baseline**: `LGE-V1.1-2026-08-27`

## Context

Desktop, iOS and Android clients have different memory, AOT and startup constraints. A boolean `IsMobile`/`IsOffline` check in Gameplay would multiply behavior and make headless verification unreliable.

## Decision

Host declares orthogonal capabilities: Role, RoomMode, ProcessTopology, Transport, Native, Render, Clock, Fault and Platform. Presets (`PureHeadless`, `NativeHeadless`, `LocalEmbedded`, `LocalSplitProcess`, `RemoteDS`, `MobileLocal`) are named bundles, not new Gameplay branches. Unity Client may use HybridCLR when the signed module, Hash, ABI, Schema and resource budget pass; Server defaults to CoreCLR and does not require HybridCLR in V1.

## Contract

Scenarios declare `RequiredCapabilities`; Hosts declare `ProvidedCapabilities`. Capability negotiation is part of ReleaseManifest/Handshake. Unsupported combinations fail before session activation with a stable reason.

## Failure semantics

Missing capability, AOT type, memory or startup budget rejects the preset or falls back only when the Scenario explicitly permits it. A failed hot-reload unloads the module and preserves the last valid active module; it cannot alter stable ABI or persistence format.

## Alternatives

Gameplay branching on platform/mode was rejected. Treating MobileLocal as a single merged World was rejected because it bypasses server/client authority tests. Making Server HybridCLR a prerequisite was rejected until a measured spike exists.

## Compatibility and migration

Adding a capability is additive; changing its meaning or a preset's required set needs a new manifest/schema epoch. HybridCLR module upgrades use normal Release validation for any breaking contract.

## Verification

Run capability matching in Pure/Native Headless and MobileLocal fixtures, Unity AOT/HybridCLR smoke, memory/startup budget tests and failed module rollback tests.
