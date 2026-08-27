---
name: repository-architecture
description: 仓库边界与架构契约——Native 聚合、ABI/Loader 所有权和 Architecture Gate;改平台包或公共 ABI 前查
metadata:
  type: doc
  status: 已交付
---

# 仓库边界与架构契约

## 规范来源与优先级

- Agent 的开发流程、测试政策和交付规则以 `.spec/` 为权威。
- 模块边界以根 [`README.md`](../../../README.md) 为本仓入口；共享架构以 `LumioGameEngineArchitecture` 的 `LGE-V1.1-2026-08-27` 为唯一来源，本仓 [`架构镜像`](../../../docs/architecture/LumioGameEngine_Architecture_v1.1.md) 只读。
- 冲突时不得在聚合层自行改写公共 ABI/Manifest；先在架构源完成 ADR、Schema、Fixture 和新 Baseline。

## 所有权边界

- 本仓把锁定版本的 NativeCore 与 VoxelEngine 组合为一个平台包，拥有 Root ABI、Loader、ArtifactIndex、签名、SBOM、Capability 与兼容检查。
- 本仓不拥有 ECS/Tick、VoxelWorld 状态、Gameplay、网络、Session 或 Host 生命周期，也不定义新的领域状态机。
- Runtime/Host 只消费生成的 Managed Contract 与单一 CoreEngine package；不得直接加载第二套 NativeCore/VoxelEngine 或手写 P/Invoke。
- 第三方平台 SDK 和工具经 Adapter/构建边界隔离，不能把供应商类型泄漏进稳定 ABI。
- 运行时发布包只包含 `runtime-verifier` 与只读信任元数据，不包含 Signer 工具或私钥访问代码（ADR 0002）。

## Architecture Gate

- Root ABI、Capability、Manifest、ID 与错误语义的唯一来源是架构源；Loader/平台包变化必须消费已发布 Schema/Artifact。
- ABI/符号/平台变化必须同步 Header、Binding、Manifest、Compatibility 与 Smoke Test；Loader 错误包含稳定 Code、缺失能力、Artifact 和 Trace 信息。
- 发布前从干净来源重建并比对 Hash；Header、Binding、Manifest 与平台产物不可手改。
- 聚合层只做组合、适配、发布和校验，不把 Voxel、Gameplay、Runtime 或 Host 领域逻辑下沉。
- 首次成功加载后进程锁定唯一 PackageIdentity；同一身份重复加载幂等，任何不同身份一律稳定拒绝（`PackageIdentityConflict`）并可诊断，不做「看起来兼容」判断。
