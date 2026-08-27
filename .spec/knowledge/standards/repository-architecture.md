---
name: repository-architecture
description: 架构源治理——Baseline、ADR、Schema/ID/Fixture 所有权与七仓同步门槛;改公共语义或契约前查
metadata:
  type: doc
  status: 已交付
---

# 架构源治理

## 唯一事实源

- 本仓是 LumioGameEngine V3 架构、状态机、公共契约、依赖图和变更规则的唯一来源；当前基线为 `LGE-V1.1-2026-08-27`。
- 规范正文是 [`LumioGameEngine_Architecture_v1.1.md`](../../../docs/architecture/LumioGameEngine_Architecture_v1.1.md)，公共决策唯一入口是 [`.spec/decisions/`](../../decisions/README.md)。
- [`schemas/`](../../../schemas/README.md)、[`ids/`](../../../ids/README.md)、[`fixtures/`](../../../fixtures/README.md) 与 [`tools/lumio_contract.py`](../../../tools/README.md) 共同组成可执行 Architecture Gate。
- 七个实现仓可以保留同 Baseline/Hash 的只读镜像，但不得独立修改共享公共语义。

## 变更顺序

任何改变公共状态、字段、错误、时序、ID、版本或依赖方向的变更必须按顺序完成：

1. 在 `.spec/decisions/` 新建或修订 `Draft` ADR，写清背景、决策、替代方案、接口/Schema、失败语义、兼容影响、迁移和验证 Fixture。
2. 更新 Schema、ID Registry 与索引；为变更加入至少一份正向 Fixture 和一份失败 Fixture。
3. 运行完整 Contract validate，更新架构正文、README、BaselineId/Hash 与 Release/Capability 信息。
4. 同步七个实现仓库的只读镜像和受影响边界规范；生成物记录 Compiler/Input/Output Hash。

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
