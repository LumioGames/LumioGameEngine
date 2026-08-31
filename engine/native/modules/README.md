# LumioCoreEngine 模块

> LumioCoreEngine 的模块导航与边界索引。

## 文档范围

本目录描述当前架构设计，不代表对应模块已经完成实现。公共 ABI、Manifest、Capability、ID 和错误语义仍以 `LumioGameEngineArchitecture` 的 `LGE-V1.4-2026-08-27` 为唯一来源；本仓的架构文件是只读镜像。

文中语句分三类：**Current Fact**（当前事实）、**Design Requirement**（设计要求，默认类别）、**Pending Decision**（待决策，须 ADR 后生效）。

## 模块索引

| 模块 | 定位 | 类型 | 优先级 |
| --- | --- | --- | --- |
| [`composition`](composition/README.md) | 锁定 Native 组合并产出不可变 BuildPlan | 生产 · 构建计划 | P0 |
| [`root-abi`](root-abi/README.md) | 发布唯一 Root C ABI 和生成绑定 | 生产 · 公共边界 | P0 |
| [`loader`](loader/README.md) | 进程内锁定并加载唯一 PackageIdentity | 生产 · 运行时加载 | P0 |
| [`manifest`](manifest/README.md) | 生成规范化 CoreEngineManifestBody | 生产 · 产物描述 | P0 |
| [`signing`](signing/README.md) | 供应链证据、离线签名与运行时验证 | 生产 · 供应链 | P0 最小子集 / P1 完整 |
| [`platform`](platform/README.md) | TargetProfile 规范化与唯一构建执行入口 | 生产 · 平台构建 | P0 最小子集 / P1 完整矩阵 |
| [`smoke`](smoke/README.md) | ABI、包和 NativeHeadless 验证 | 验证平面 · 非生产 | 验证门，随各阶段交付 |
| [`diagnostics`](diagnostics/README.md) | CoreEngine 事件契约与 Failure Evidence 适配 | 观测适配平面 · 非生产 | P1 |

优先级语义（对应根 README「当前阶段与开发节奏」）：

- **P0 最小垂直切片**：单一 TargetProfile（Linux Server、x86_64、glibc、DynamicLibrary）+ 测试密钥 Signer/运行时 Verifier + Loader 状态机 + 基础事件契约，贯穿一条端到端 Slice 并由 `smoke` 验证。
- **P1 扩展**：完整平台矩阵、生产 Key Management/Rotation、远程签名、完整 SBOM/License 自动化。
- `smoke` 是每个阶段的验证门，不参与生产模块的 P0/P1 依赖；模块拓扑的存在性以架构源第 16 章模块地图为准，P0/P1 只表示交付优先级。

## 依赖与产物视图

一张图只表达一种关系；箭头语义在每张图前显式声明。

### 源码 / Schema 依赖图

箭头语义与架构基线 2.2 一致：`A -> B` 表示 A 消费 B 的公开 Schema、Artifact 或契约（编译期/读取期依赖），**不是**产物流向，也不是运行时调用方向。

```text
composition  -> NativeCore/VoxelEngine 已发布源描述与 Feature 约束
root-abi     -> 架构源 ABI Schema + NativeCore/VoxelEngine 源 Schema + composition::BuildPlan
platform     -> composition::BuildPlan + root-abi::生成产物 + 平台 SDK/工具链
manifest     -> composition::ProvenanceRecord + root-abi::ABI 描述 + platform::ArtifactIndex
signing      -> manifest::CanonicalManifestBody + platform::ArtifactIndex + 依赖清单
loader       -> manifest::包 Schema + signing::RuntimeVerifier 接口
                + root-abi::RootApiTable 契约 + platform::LoadBackend 契约
smoke        -> 以上全部公开契约与产物（验证平面）
diagnostics  -> 架构源 LoggingEvent/FailureBundle Schema + 各模块公开事件契约（观测适配平面）
```

约束：

- 生产模块不得依赖 `smoke` 或 `diagnostics` 的实现。
- `loader` 只消费公开 Manifest Schema、Verifier 接口和 LoadBackend 契约，不编译依赖 Manifest 生成器或 Signer 工具的内部实现。
- `diagnostics` 不读取其他模块私有状态，只接收公开事件和状态快照。
- 模块不得循环依赖，不得读取其他模块内部实现。

### 构建产物流

箭头语义：生产者到消费者的产物交接顺序。该顺序消除签名自引用：Evidence 以 Digest 绑定进 ManifestBody，签名在 Body 定稿之后发生，Loader 只消费运行时验证输出。

```text
Source Lock + 公共 ABI Schema
        |
        v
BuildPlan + 生成 Header/Binding
        |
        v
PlatformArtifactSet + ArtifactIndex
        |
        v
SBOM / License / Provenance Digest（EvidenceSet）
        |
        v
Canonical CoreEngineManifestBody
        |
        v
Detached SignatureEnvelope
        |
        v
VerifiedPackageDescriptor（运行时验证输出）
        |
        v
LoaderLease + RootApiTableView
```

### 验证与观测事件流

箭头语义：报告与事件的流向。

```text
全部生产模块产物 --被验证--> smoke --> 测试报告 / Fixture 引用 --> CI 与 Release Gate

composition / root-abi / platform / manifest / signing / loader / smoke
  --结构化事件 + Failure Evidence Fragment--> diagnostics 适配 --> Host 注入的 EventSink
```

## 术语

| 术语 | 语义 |
| --- | --- |
| Artifact Hash | 单个产物文件字节的 SHA-256，登记在 ArtifactIndex 条目中 |
| ArtifactIndex | 逐文件清单：规范路径、类型、大小、Digest；由 `platform` 唯一生产 |
| Artifact Set Digest | 对规范化 ArtifactIndex 的整体 Digest |
| Manifest Digest | 对 Canonical CoreEngineManifestBody 字节的 Digest |
| EvidenceSet | SBOM、License、Provenance 的 Digest 与媒体类型集合，以 Digest 绑定进 ManifestBody |
| SignatureEnvelope | 与 ManifestBody 分离的签名载荷；时间戳、证书链等非确定字段只存在于 Envelope |
| PackageIdentity | Manifest Digest + Artifact Set Digest + ABI Identity + TargetProfile（+ Capability Set Digest）的组合身份 |
| VerifiedPackageDescriptor | 运行时 Verifier 的输出；Loader 只消费该结果，不消费离线验证结论 |
| LoaderLease | Loader 输出的引用计数租约，连同 RootApiTableView 一起交给 Host |

禁则：不得使用 `Manifest Hash`、`Package Hash` 这类混用术语；必须使用上表术语（spec-lint 机器校验）。

## README 约定

每个模块 README 必须包含以下章节（spec-lint 机器校验）：**负责什么、明确不负责什么、输入与输出、依赖关系、生命周期与失败行为、验收范围、相关文档**。

README 只描述当前设计；架构决策写入 [ADR](../.spec/decisions/README.md)，公共契约变更先回到架构源。

## 相关文档

- [仓库根 README](../README.md)
- [仓库边界与架构契约](../.spec/knowledge/standards/repository-architecture.md)
- [架构基线镜像](../docs/architecture/LumioGameEngine_Architecture_v1.2.md)
