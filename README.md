# LumioGameEngine

`LumioCoreEngine` 独立仓库已标记为 **Deprecated**，仅保留历史审计与回滚用途；新开发不得继续向该仓库添加功能或依赖。

> Lumio 游戏引擎的 SDK 组装根、Native 聚合根和跨 Host 开发入口。

## 当前架构

完整拓扑、职责和 API/ABI 说明见 [`docs/architecture/LumioGameEngine_Architecture.md`](docs/architecture/LumioGameEngine_Architecture.md)。

```text
LumioGame
├── LumioServer ──┐
├── LumioClient ──┴──> LumioEngineSDK
└── Gameplay ────────> LumioEngineSDK
                       ├── LumioGameRuntime ──┐
                       └── LumioVoxelEngine ──┴──> LumioNativeCore
```

本仓吸收原 `LumioCoreEngine` 的组合构建、Root ABI、Loader 和平台产物职责。`LumioCoreEngine` 不再作为独立依赖；其代码迁入 `engine/native/`。

## 开发期目标

预上线只要求证明运行中的 Host 加载了刚构建的代码：

```text
修改源码 -> SDK Native 增量构建 -> 生成 BuildId -> 复制到 .run/<BuildId>
          -> Server 与 Client 用同一路径启动 -> 日志打印并核对 BuildId/SHA-256
```

不要求日常开发升 Baseline、同步契约镜像、发布正式包或运行全量失败 Fixture。ABI/API 变化只需更新唯一 ABI 定义、生成 Binding 并重编直接消费者。

## 目录

- `engine/native/`：吸收后的 Native 聚合、Root ABI、Loader 和平台构建模块。
- `engine/abi/`：托管与 Native 边界的唯一 ABI 定义及生成输出。
- `engine/managed/`：共享 C# Loader、BuildInfo 和 SDK API。
- `eng/`：跨仓开发构建、运行和 BuildId 验证脚本。

## 首次环境

Windows 开发环境使用 WSL2 Ubuntu 24.04 构建和运行 `.so`；Windows `.dll` 使用同一 ABI 单独构建。Server 入口为 `Lumio.Server.MvpHost.App`，Client 首条验证入口为 `Lumio.Client.Bot.Host`。

```powershell
wsl --install -d Ubuntu-24.04
```

安装 Rust、.NET 10、clang 和 build-essential 后，从仓库根目录执行：

```bash
./eng/dev-run.sh
```

脚本会启动 Server 和 Headless Client，并在任一 Hash/BuildId 不匹配时返回非零退出码。
