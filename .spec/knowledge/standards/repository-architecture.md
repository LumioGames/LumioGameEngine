---
name: repository-architecture
description: 引擎 SDK 组装、API/ABI 边界与开发态构建证明；改跨仓接口或发布边界前查
metadata:
  type: doc
  status: 已交付
---

# 引擎 SDK 组装与接口边界

## 唯一事实源

- 本仓是 `LumioEngineSDK` 的组装根，吸收原 `LumioCoreEngine`，并维护当前 Living 架构说明。
- 规范正文是 [`LumioGameEngine_Architecture.md`](../../../docs/architecture/LumioGameEngine_Architecture.md)，公共决策入口是 [`.spec/decisions/`](../../decisions/README.md)。
- `engine/abi/native-abi.json`、`eng/generate-abi.mjs`、`engine/native/modules/sdk-native` 与共享 C# Loader 组成开发态接口闭环。
- NativeCore、VoxelEngine 和 GameRuntime 是 SDK provider；Server、Client 和 Game 是 SDK consumer/Host。

## 变更顺序

开发态跨边界变更按以下最短顺序完成：

1. 修改唯一 API 或 ABI 定义（只有托管/Native 二进制边界需要 ABI）。
2. 运行 `node eng/generate-abi.mjs` 并重编 SDK Native 与直接消费者。
3. 运行 `eng/dev-run.ps1`，确认 Server/Client 的实际路径、BuildId、ABI Hash 和 Binary SHA-256 一致。
4. 进入正式硬化时，再启用历史 Schema/Fixture/Baseline 资产并单独建立发布决策。

不得跳步用实现代码、README 镜像或生成物反向定义公共契约。

## 所有权与非目标

- 本仓定义 Tick/Determinism、Entity、Replication、CrossWorldTxn、ABI、Manifest、Persistence、Logging、Config、Release 和安全/供应链的公共语义。
- 本仓维护 ID Namespace、Schema 版本、兼容判定、Failure Bundle 与统一测试结果格式。
- 本仓不实现 ECS、Voxel、网络、CoreCLR、Renderer 或具体 Gameplay，不替代实现仓单元测试、Benchmark 或产品数据。
- P2 表示实现可后置，不表示架构可删除；P2 能力必须保留所有者、接口位置与兼容策略。

## ADR 生命周期

- `Draft` 表示方向已写明、正在由 Schema/Fixture/实现验证，可在被接受前修订。
- `Accepted` 后正文不可改写；改变结论必须新增下一编号 ADR 并标明取代关系。
- `Reserved` 只占用明确的未来扩展边界，不能被实现当作已批准能力。
- 待选实现与运维默认值集中在 [`DECISIONS_PENDING.md`](../../../docs/architecture/DECISIONS_PENDING.md)，确认前不得提升为稳定公共语义。
