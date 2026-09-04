# LumioGameEngine

[简体中文](README.md) | [English](README.en.md)

> SDK assembly root · native aggregation root · cross-host development entry point

---

<!-- lumio-community:start -->
<div align="center">
<table>
<tr>
<td align="center" width="50%" valign="top">
<a href="https://qm.qq.com/q/PGkXh4tCyQ"><img src="https://raw.githubusercontent.com/LumioGames/.github/main/profile/assets/qr-qq.svg" width="170" alt="QQ group 972220164"></a><br>
<a href="https://qm.qq.com/q/PGkXh4tCyQ"><img src="https://img.shields.io/badge/QQ%20group-972220164-6171F0?style=for-the-badge&logo=tencentqq&logoColor=white" alt="QQ group 972220164"></a><br>
<sub>QQ group · anything goes</sub>
</td>
<td align="center" width="50%" valign="top">
<a href="https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=fffn1ae7-fd83-4315-96ac-6fa3aba3968e"><img src="https://raw.githubusercontent.com/LumioGames/.github/main/profile/assets/qr-engine.svg" width="170" alt="LumioEngine community"></a><br>
<a href="https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=fffn1ae7-fd83-4315-96ac-6fa3aba3968e"><img src="https://img.shields.io/badge/Feishu-LumioEngine%20community-5DE2C6?style=for-the-badge&logoColor=1E2A3A" alt="LumioEngine community"></a><br>
<sub>Feishu topic group · Rust / C# engine layer</sub>
</td>
</tr>
</table>
<sub>Join the chat before you read the code. Other groups and the big picture are on the <a href="https://github.com/LumioGames">LumioGames profile</a>.</sub>
</div>
<!-- lumio-community:end -->

The standalone `LumioCoreEngine` repository was **deleted** on 2026-09-04, after being marked Deprecated on 2026-08-31. Its implementation now lives under `engine/native/` in this repository. Entries in older ADRs that name it as Owner redirect through [`ADR-059`](.spec/decisions/ADR-059-lumiocoreengine-repository-retirement.md).

## What this is

`LumioGameEngine` is the **SDK assembly root** of the Lumio game engine. It composes a domain-agnostic native kernel, a voxel engine, and a managed runtime into a single publishable `LumioEngineSDK`, defines and generates the only API/ABI boundary that Server and Client are allowed to depend on, and serves as the single source of documentation for this cross-repository architecture.

Three engineering principles hold without exception:

- **Single source of truth** — every architectural decision, interface definition, and acceptance criterion converges in [`.spec/`](.spec/AGENTS.md) in this repository; no implementation code or generated artifact is allowed to define the contract in reverse.
- **Evidence before claims** — every build computes a source `BuildId`, an ABI hash, and a binary SHA-256; host startup logs print and verify all three. "It runs" is settled by hash comparison, not by verbal confirmation.
- **Pragmatic pre-launch posture** — the project is currently in its Living Architecture phase. The only requirement is proving that a running host loaded the code that was just built; there is no mandatory baseline, contract mirror, or full fixture suite. Breaking ABI/API changes are allowed, at the cost of updating the single definition and rebuilding direct consumers.

## Product topology

```text
LumioGame
├── LumioServer ──┐
├── LumioClient ──┴──> LumioEngineSDK
└── Gameplay ────────> LumioEngineSDK
                       ├── LumioGameRuntime ──┐
                       └── LumioVoxelEngine ──┴──> LumioNativeCore
```

| Repository | Owns | Does not own |
| --- | --- | --- |
| **`LumioGameEngine` (this repo)** | SDK assembly, native aggregation, ABI/bindings, shared loader, dev launcher, integration verification | Gameplay content, Server/Client host business logic, voxel domain algorithms |
| `LumioNativeCore` | Domain-agnostic Rust kernel, handles, errors, capabilities, memory, jobs | Voxel, ECS, gameplay, networking, hosts |
| `LumioVoxelEngine` | VoxelWorld, chunks, revisions, mutations, streaming, snapshots | Gameplay permissions, sockets, session and host lifecycle |
| `LumioGameRuntime` | ECS, tick, coordinator, replication, GAS, persistence, config | Processes, sockets, gameplay content, voxel internals |
| `LumioServer` | Server host, networking, session, world slots, CoreCLR hosting | Runtime semantics, native aggregation, gameplay rules |
| `LumioClient` | Client connection, replica, prediction, Unity/HybridCLR adapter, headless bot | Server authority, native aggregation, gameplay content |
| `LumioGame` | Gameplay, mapping, config, content, scenarios, Server/Client composition | Generic ABI, runtime/host lifecycle, voxel internals |

Full topology, ownership boundaries, and decision records live in [`.spec/knowledge/features/architecture.md`](.spec/knowledge/features/architecture.md) and [`.spec/decisions/`](.spec/decisions/README.md).

## Interface boundaries

| Layer | Definition | Single source of truth |
| --- | --- | --- |
| API | Source-level interfaces, propagated through normal compilation | Each repository's source |
| ABI | Calling convention, struct layout, and function tables between managed code and native dynamic libraries | [`engine/abi/native-abi.json`](engine/abi/native-abi.json) |
| Wire | WebSocket message protocol between browser/bot and server — not an ABI | [`engine/wire/hello-wire-v1.json`](engine/wire/hello-wire-v1.json) |

The SDK native library exports exactly one root symbol; every other capability is exposed through a versioned function table reached from it:

```c
lumio_status_t lumio_engine_get_api_v1(
    uint32_t requested_version,
    const lumio_engine_root_api_v1** out_api);
```

Headers, Rust bindings, and C# bindings are all generated from `engine/abi/native-abi.json`. Any cross-boundary change must update this single definition first, then regenerate and verify — hand-written second layouts or contracts reverse-engineered from implementation code are not permitted.

## Development workflow

```text
edit source -> incremental SDK native build -> compute BuildId -> copy to .run/<BuildId>
            -> Server and Client launch from the same path -> logs print and verify BuildId / ABI hash / SHA-256
```

```bash
node eng/generate-abi.mjs        # regenerate ABI bindings
./eng/dev-run.sh                 # incremental build, launch both hosts, verify hashes (WSL2/Linux)
# Windows: powershell -NoProfile -ExecutionPolicy Bypass -File eng/dev-run.ps1
```

When SDK Rust or the shared loader need targeted checks:

```bash
cargo test -p lumio-engine-native
dotnet test engine/managed/Lumio.Engine.NativeLoader.Tests/Lumio.Engine.NativeLoader.Tests.csproj
```

The MS-00002 Hello World milestone has already passed a real end-to-end acceptance run on Windows — Rust server, SDK native DLL, CoreCLR authoritative tick, a real browser, and an independent headless bot, with every bidirectional message hash-verified consistent. Evidence is archived under `LumioGame/integration/hello/evidence-run1/`.

## Directory layout

- `engine/native/` — absorbed native aggregation, root ABI, loader, and platform build modules.
- `engine/abi/` — the single ABI definition and generated bindings across the managed/native boundary.
- `engine/managed/` — shared C# loader, build info, and SDK API.
- `engine/wire/` — the single contract definition for inter-host WebSocket protocols.
- `eng/` — cross-repo dev build, run, and BuildId verification scripts.
- `.spec/` — the single documentation root: rules, knowledge, plans, decisions, and reviews.

## First-time setup

Windows development uses WSL2 Ubuntu 24.04 to build and run `.so` binaries; the Windows `.dll` is built separately from the same ABI. The server entry point is `Lumio.Server.MvpHost.App`; the first client verification entry point is `Lumio.Client.Bot.Host`.

```powershell
wsl --install -d Ubuntu-24.04
```

After installing Rust, .NET 10, clang, and build-essential, run `./eng/dev-run.sh` from the repository root. It starts the server and a headless client, and exits non-zero on any hash/BuildId mismatch.

## Documentation map

- Project overview and agent dispatch: [`.spec/AGENTS.md`](.spec/AGENTS.md)
- Knowledge navigation: [`.spec/knowledge/README.md`](.spec/knowledge/README.md)
- System-level hard rules: [`.spec/rules/system.md`](.spec/rules/system.md)
- Decision records: [`.spec/decisions/README.md`](.spec/decisions/README.md)

## License

Apache License 2.0 — see [LICENSE](LICENSE).
