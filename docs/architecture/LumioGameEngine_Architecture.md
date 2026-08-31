# LumioGameEngine 架构与开发说明

> 本文是预上线开发期的 Living Architecture。它描述当前仓库边界和可运行的接口，不是发布基线。

## 1. 产品拓扑

`LumioGame` 是最终产品组合根，包含 Server、Client 和玩法内容。Server 与 Client 都消费同一个 `LumioEngineSDK`；SDK 由本仓组装，吸收原 `LumioCoreEngine` 的聚合职责。

```text
LumioGame
├── LumioServer ──┐
├── LumioClient ──┴──> LumioEngineSDK
└── Gameplay ────────> LumioEngineSDK
                       ├── LumioGameRuntime ──┐
                       └── LumioVoxelEngine ──┴──> LumioNativeCore
```

箭头表示编译或运行时消费方向；产品包含关系由 `LumioGame` 维护，Server/Client 不进入 SDK 内部实现。

## 2. 仓库职责

| 仓库 | 职责 | 不负责 |
| --- | --- | --- |
| `LumioGameEngine`（本仓） | SDK 组装、Native 聚合、ABI/Binding、共享 Loader、开发启动器、集成验证和 SDK 产物 | 具体玩法、Server/Client Host 业务、Voxel 领域算法 |
| `LumioNativeCore` | 领域无关 Rust Kernel、Handle、Error、Capability、内存、Job 和空间基础 | Voxel、ECS、Gameplay、网络和 Host |
| `LumioVoxelEngine` | VoxelWorld、Chunk、Revision、Mutation、Streaming、Snapshot 和 Voxel Migration | Gameplay 权限、Socket、Session 和 Host 生命周期 |
| `LumioGameRuntime` | ECS、Tick、Coordinator、Replication、GAS、Persistence、Config 和 Determinism | 进程、Socket、玩法内容和 Voxel 内部 |
| `LumioServer` | Server Host、网络、Session、WorldSlot、CoreCLR Hosting、维护和升级编排 | Runtime 语义、Native 聚合和玩法规则 |
| `LumioClient` | Client Connection、Replica、Prediction、Unity/HybridCLR Adapter 和 Headless Bot | Server 权威、Native 聚合和玩法内容 |
| `LumioGame` | Gameplay、Mapping、配置、内容、Scenario、Migration，以及 Server/Client 组合 | 通用 ABI、Runtime/Host 生命周期和 Voxel 内部 |

`LumioCoreEngine` 不再是独立职责或依赖节点；其实现迁入本仓 `engine/native/`。

## 3. API 与 ABI

### 3.1 API

API 是源码级接口。Server、Client、Game 和 Runtime 使用托管 API；NativeCore、VoxelEngine 和 SDK Native 聚合内部使用 Rust API。源码 API 变化通过正常编译传播，不要求登记 Baseline。

### 3.2 ABI

ABI 只存在于托管代码与 Native 动态库之间，定义调用约定、符号、固定宽度类型、结构布局和函数表。Rust ABI 本身不稳定，不能直接暴露 Rust 容器、字符串、异常或对象引用。

SDK Native 库只导出一个根符号：

```c
lumio_status_t lumio_engine_get_api_v1(
    uint32_t requested_version,
    const lumio_engine_root_api_v1** out_api);
```

Root 表使用 `#[repr(C)]`，携带 `abi_version`、`struct_size`、`abi_hash[32]` 和 `build_id[16]`，下层 API 的 Handle 输出参数一律使用指针。所有 Header、Rust Binding 和 C# Binding 从 `engine/abi/native-abi.json` 生成。

MS-00002 起，Root 表 v1 另外暴露三个 CLR 装载函数（定义与参数语义见 `engine/abi/native-abi.json` 的字段 doc）：`create_clr_host`（hostfxr → runtimeconfig → 指定托管程序集的 UnmanagedCallersOnly 入口，创建期 fail-fast、失败完整回滚）、`clr_host_call`（单次字节协议调用，输入/输出均由调用方缓冲）、`destroy_clr_host`。状态码相应扩展 `ClrInitFailed/ClrEntryFailed/BufferTooSmall`。两个实测约束已钉进定义文档：入口描述第二段必须是**托管方法名**（不是 UnmanagedCallersOnly 的 EntryPoint 别名）；CoreCLR 每进程只能成功初始化一次（二次 `create_clr_host` 在 initialize 步失败），宿主应在进程生命周期内至多创建一次。

NativeCore 和 VoxelEngine 不导出自己的根符号；SDK 聚合层负责把它们组合成一个 Native 库。首期不支持 Native 回调；需要回调时必须新增版本化 C 函数表。

### 3.3 Wire 协议

Browser/Bot 与 Server 之间的 WebSocket 消息是 wire 协议，不是 ABI。开发态里程碑（MS-00002 Hello World）的最小 wire 契约唯一真值是 [`engine/wire/hello-wire-v1.json`](../engine/wire/hello-wire-v1.json)：消息形状、字段语义（sender/sequence/revision/payloadSha256/latency）、失败错误码、进程 readiness/shutdown 边界与审计事件词表。消费方（Rust Server、C# Runtime、Browser、Bot、集成验收）不得另写一份协议真值；校验入口 `node eng/verify-hello-wire.mjs`。

## 4. 开发期构建与最新代码证明

预上线默认流程只有“编译、加载、证明”三步，不执行 Baseline、镜像同步、全量 Fixture 或正式发布门。

```text
修改任意引擎源码
  -> SDK Native 增量构建
  -> 计算源码 BuildId，复制到 .run/<BuildId>/<platform>/
  -> Server 与 Client 用同一绝对路径启动
  -> Loader 校验 ABI Hash、BuildId 和文件 SHA-256
  -> 两端日志打印实际路径、BuildId、ABI Hash、Binary SHA-256
```

源码 BuildId 覆盖已跟踪和未跟踪的有效源码文件、工具链和构建参数；二进制 SHA-256 在构建后计算并写入 `build-info.json`（BOM-less UTF-8）。每次运行使用唯一目录，避免旧进程映射或锁定新产物。

首个双端验证使用 WSL2 Ubuntu 中的 `Lumio.Server.MvpHost.App` 和 `Lumio.Client.Bot.Host`，Windows DLL 与 Unity Adapter 使用同一 ABI 后续验证。

### 4.1 MS-00002 Hello World 集成验证入口

Hello World 里程碑的 Windows 端到端验收（Rust Server + SDK Native DLL + CoreCLR/C# Runtime 权威 Tick + 真实 Chromium 浏览器 + 独立 Headless Bot，双向消息均经 InputCommand → authoritative Tick → Delta，两轮一致）由 LumioGame 仓的集成启动器执行：

```text
LumioGame/integration/hello/launcher.mjs
  消费: eng/dev-build.ps1 产出的 .run/<BuildId>/win-x64（DLL + build-info.json）
        + LumioGameRuntime modules/hello/entry 构建输出（dll + runtimeconfig.json）
        + LumioServer target 下的 lumio-server.exe
        + LumioClient modules/web/hello（静态页）与 modules/hello/host（Bot dll）
        + engine/wire/hello-wire-v1.json（复制为页面旁 contract.json）
  产出: evidence/<轮>/ 完整证据包（audit/bot trace/browser result/截图/Playwright trace）
```

2026-08-31 验收证据归档：`LumioGame/integration/hello/evidence-run1/`（结论 SUCCESS；BUILD_ID `ab12bf280961a39632022f7c6f3be78f`，ABI Hash `1dfc86da…`，双方向延迟 1–12ms，两轮方向/sender/revision/payloadSha256/tickId 一致，全部进程退出码 0、无残留）。各仓交付 commit 与验证证据索引见 Workflow RM-00010 各需求（R-00335~R-00343）的证据评论。

## 5. SDK 组成

`LumioEngineSDK` 是一个逻辑 SDK，不限定为单个文件，包含：

- SDK Native 聚合库（按平台生成 DLL/SO/静态库）。
- `LumioGameRuntime` 托管程序集和公开 Gameplay/Host API。
- Native Loader、ABI Binding、BuildInfo 和开发启动工具。
- SDK 目录清单与本次构建证明。

Server 和 Client 是 SDK 的消费者和产品 Host，不作为 SDK 内部实现打包；`LumioGame` 负责将它们与自己的 Gameplay 组合。

## 6. 预上线质量边界

- Native 或托管内部实现改动：目标仓单元测试 + SDK 构建 + 双端加载证明。
- ABI/API 改动：修改唯一 ABI 定义，重新生成 Binding，并重编直接消费者；预上线允许破坏式变化。
- 只有进入正式硬化阶段，才启用完整契约、失败矩阵、供应链证据、兼容策略和发布审查。
- 旧 Baseline、Schema、Fixture、生成物和镜像不属于当前主线开发入口；迁移前 tag 与 Git 历史是唯一留档。

## 7. 迁移完成条件

1. 本仓完成 `LumioCoreEngine` 实现迁移并能构建 SDK Native 库。
2. Server 与 Client 共享 Loader，能拒绝路径、ABI Hash 或 BuildId 不匹配。
3. Game 只通过 SDK API 编译 Gameplay，并包含 Server/Client 产物。
4. 活动源码和 CI 不再依赖 `LumioCoreEngine`、Baselines 或 contract mirror。
5. WSL Linux 与 Windows 至少各有一条真实 Host 加载验证。
