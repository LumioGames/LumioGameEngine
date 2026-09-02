# LumioGameEngine

[简体中文](README.md) | [English](README.en.md)

> SDK 组装根 · Native 聚合根 · 跨 Host 开发入口

---

<!-- lumio-community:start -->
<div align="center">
<table>
<tr>
<td align="center" width="50%" valign="top">
<a href="https://qm.qq.com/q/PGkXh4tCyQ"><img src="https://raw.githubusercontent.com/LumioGames/.github/main/profile/assets/qr-qq.svg" width="170" alt="QQ 交流群 972220164"></a><br>
<a href="https://qm.qq.com/q/PGkXh4tCyQ"><img src="https://img.shields.io/badge/QQ%20%E4%BA%A4%E6%B5%81%E7%BE%A4-972220164-6171F0?style=for-the-badge&logo=tencentqq&logoColor=white" alt="QQ 交流群 972220164"></a><br>
<sub>什么都能聊</sub>
</td>
<td align="center" width="50%" valign="top">
<a href="https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=fffn1ae7-fd83-4315-96ac-6fa3aba3968e"><img src="https://raw.githubusercontent.com/LumioGames/.github/main/profile/assets/qr-engine.svg" width="170" alt="LumioEngine 开发者社区"></a><br>
<a href="https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=fffn1ae7-fd83-4315-96ac-6fa3aba3968e"><img src="https://img.shields.io/badge/%E9%A3%9E%E4%B9%A6%E7%BE%A4-LumioEngine%20%E5%BC%80%E5%8F%91%E8%80%85%E7%A4%BE%E5%8C%BA-5DE2C6?style=for-the-badge&logoColor=1E2A3A" alt="LumioEngine 开发者社区"></a><br>
<sub>飞书话题群 · Rust / C# 引擎层</sub>
</td>
</tr>
</table>
<sub>先进群再看代码。其它群和整体介绍见 <a href="https://github.com/LumioGames">LumioGames 主页</a>。</sub>
</div>
<!-- lumio-community:end -->

`LumioCoreEngine` 独立仓库已标记为 **Deprecated**，仅保留历史审计与回滚用途；新开发不得继续向该仓库添加功能或依赖，其实现已迁入本仓 `engine/native/`。

## 这是什么

`LumioGameEngine` 是 Lumio 游戏引擎的 **SDK 组装根**：它把领域无关的 Native Kernel、Voxel 引擎和托管 Runtime 组合成一个可发布的 `LumioEngineSDK`，定义并生成 Server 与 Client 唯一遵守的 API/ABI 边界，同时是这套跨仓架构的唯一文档来源。

工程上遵循几条不打折扣的原则：

- **单一事实源** — 全部架构决策、接口定义与验收标准收敛在本仓 [`.spec/`](.spec/AGENTS.md)，任何实现代码或生成物都不得反向定义契约。
- **证据先于声称** — 每一次构建都会计算源码 `BuildId`、ABI Hash 与二进制 SHA-256，Host 启动日志逐项打印并校验，"跑起来了"以哈希核对为准，不靠口头确认。
- **预上线务实** — 当前处于 Living Architecture 阶段，只要求证明"运行中的 Host 加载了刚构建的代码"，不强制 Baseline、契约镜像或全量 Fixture；ABI/API 破坏式变更被允许，代价是同步更新唯一定义并重编消费方。

## 产品拓扑

```text
LumioGame
├── LumioServer ──┐
├── LumioClient ──┴──> LumioEngineSDK
└── Gameplay ────────> LumioEngineSDK
                       ├── LumioGameRuntime ──┐
                       └── LumioVoxelEngine ──┴──> LumioNativeCore
```

| 仓库 | 职责 | 不负责 |
| --- | --- | --- |
| **`LumioGameEngine`（本仓）** | SDK 组装、Native 聚合、ABI/Binding、共享 Loader、开发启动器与集成验证 | 具体玩法、Server/Client Host 业务、Voxel 领域算法 |
| `LumioNativeCore` | 领域无关 Rust Kernel、Handle、Error、Capability、内存与 Job | Voxel、ECS、Gameplay、网络与 Host |
| `LumioVoxelEngine` | VoxelWorld、Chunk、Revision、Mutation、Streaming、Snapshot | Gameplay 权限、Socket、Session 与 Host 生命周期 |
| `LumioGameRuntime` | ECS、Tick、Coordinator、Replication、GAS、Persistence、Config | 进程、Socket、玩法内容与 Voxel 内部 |
| `LumioServer` | Server Host、网络、Session、WorldSlot、CoreCLR Hosting | Runtime 语义、Native 聚合与玩法规则 |
| `LumioClient` | Client Connection、Replica、Prediction、Unity/HybridCLR Adapter、Headless Bot | Server 权威、Native 聚合与玩法内容 |
| `LumioGame` | Gameplay、Mapping、配置、内容、Scenario、Server/Client 组合 | 通用 ABI、Runtime/Host 生命周期与 Voxel 内部 |

完整拓扑、职责边界与决策记录见 [`.spec/knowledge/features/architecture.md`](.spec/knowledge/features/architecture.md) 与 [`.spec/decisions/`](.spec/decisions/README.md)。

## 接口边界

| 层次 | 定义 | 唯一真值 |
| --- | --- | --- |
| API | 源码级接口，随正常编译传播 | 各仓源码 |
| ABI | 托管代码与 Native 动态库之间的调用约定、结构布局与函数表 | [`engine/abi/native-abi.json`](engine/abi/native-abi.json) |
| Wire | Browser/Bot 与 Server 之间的 WebSocket 消息协议，不是 ABI | [`engine/wire/hello-wire-v1.json`](engine/wire/hello-wire-v1.json) |

SDK Native 库只导出一个根符号，其余能力全部经由版本化函数表暴露：

```c
lumio_status_t lumio_engine_get_api_v1(
    uint32_t requested_version,
    const lumio_engine_root_api_v1** out_api);
```

Header、Rust Binding 与 C# Binding 全部由 `engine/abi/native-abi.json` 生成；任何跨边界变更都必须先改这份唯一定义，再跑生成与校验，禁止手写第二套布局或用实现代码反推契约。

## 开发工作流

```text
修改源码 -> SDK Native 增量构建 -> 生成 BuildId -> 复制到 .run/<BuildId>
          -> Server 与 Client 用同一路径启动 -> 日志打印并核对 BuildId / ABI Hash / SHA-256
```

```bash
node eng/generate-abi.mjs        # 重新生成 ABI Binding
./eng/dev-run.sh                 # 增量构建、启动双端、校验哈希（WSL2/Linux）
# Windows: powershell -NoProfile -ExecutionPolicy Bypass -File eng/dev-run.ps1
```

需要检查 SDK Rust 或共享 Loader 时，再分别运行：

```bash
cargo test -p lumio-engine-native
dotnet test engine/managed/Lumio.Engine.NativeLoader.Tests/Lumio.Engine.NativeLoader.Tests.csproj
```

MS-00002 Hello World 里程碑已在 Windows 上完成真实端到端验收（Rust Server + SDK Native DLL + CoreCLR 权威 Tick + 真实浏览器 + 独立 Headless Bot，双向消息全部经哈希核对一致）；证据归档于 `LumioGame/integration/hello/evidence-run1/`。

## 目录结构

- `engine/native/` — 吸收后的 Native 聚合、Root ABI、Loader 与平台构建模块。
- `engine/abi/` — 托管与 Native 边界的唯一 ABI 定义及生成输出。
- `engine/managed/` — 共享 C# Loader、BuildInfo 与 SDK API。
- `engine/wire/` — Host 间 WebSocket 协议的唯一契约定义。
- `eng/` — 跨仓开发构建、运行与 BuildId 校验脚本。
- `.spec/` — 全仓唯一文档根：规则、知识、计划、决策与审查记录。

## 首次环境搭建

Windows 开发环境使用 WSL2 Ubuntu 24.04 构建和运行 `.so`；Windows `.dll` 使用同一 ABI 单独构建。Server 入口为 `Lumio.Server.MvpHost.App`，Client 首条验证入口为 `Lumio.Client.Bot.Host`。

```powershell
wsl --install -d Ubuntu-24.04
```

安装 Rust、.NET 10、clang 与 build-essential 后，从仓库根目录执行 `./eng/dev-run.sh`；脚本会启动 Server 和 Headless Client，并在任一 Hash/BuildId 不匹配时返回非零退出码。

## 文档地图

- 项目介绍与 Agent 调度：[`.spec/AGENTS.md`](.spec/AGENTS.md)
- 知识导航：[`.spec/knowledge/README.md`](.spec/knowledge/README.md)
- 系统硬规则：[`.spec/rules/system.md`](.spec/rules/system.md)
- 决策记录：[`.spec/decisions/README.md`](.spec/decisions/README.md)

## License

Apache License 2.0 — see [LICENSE](LICENSE)。
