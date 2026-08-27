# LumioCoreEngine 模块

> LumioCoreEngine 的模块导航与边界索引。

## 文档范围

本目录描述当前架构设计，不代表对应模块已经完成实现。公共 ABI、Manifest、Capability、ID 和错误语义仍以 `LumioGameEngineArchitecture` 的 `LGE-V1.0-2026-08-27` 为唯一来源；本仓的架构文件是只读镜像。

## 模块索引

| 模块 | 定位 | 优先级 | 类型 |
| --- | --- | --- | --- |
| [`composition`](composition/README.md) | 锁定 Native 组合与可复现构建输入 | P0 | 构建组合 |
| [`root-abi`](root-abi/README.md) | 发布唯一 Root C ABI 和生成绑定 | P0 | 公共边界 |
| [`loader`](loader/README.md) | 在进程内加载唯一相容 Native 包 | P0 | 运行时加载 |
| [`manifest`](manifest/README.md) | 生成规范化 CoreEngineManifest | P0 | 产物描述 |
| [`signing`](signing/README.md) | 签名、供应链证明和 SBOM | P1 | 供应链 |
| [`platform`](platform/README.md) | 平台目录、链接方式和兼容矩阵 | P1 | 平台适配 |
| [`smoke`](smoke/README.md) | ABI、包和 NativeHeadless 验证 | P0 | 验证支撑 |
| [`diagnostics`](diagnostics/README.md) | CoreEngine 事件与 Failure Bundle | P1 | 观测支撑 |

“首批模块”表示拓扑中的模块存在；`P0/P1` 表示交付优先级，两者不是同一个概念。

## 依赖方向

```text
Native/Voxel Source Schema
        |
        v
composition -----> root-abi -----> platform
      |                 |              |
      +-----------------+--------------+
                        v
                     manifest
                        |
                        v
                     signing
                        |
                        v
                      loader

所有生产模块 -----> smoke（验证）
loader / manifest / signing / platform -----> diagnostics（观测适配）
```

模块不得循环依赖，不得读取其他模块内部实现；只消费已发布 Schema、Artifact 或生成契约。

## README 约定

每个模块 README 必须说明：负责什么、不负责什么、输入与输出、依赖关系、生命周期与失败行为、验收范围和相关文档。README 只描述当前设计；架构决策写入 ADR，公共契约变更先回到架构源。

## 相关文档

- [仓库根 README](../README.md)
- [仓库边界与架构契约](../.spec/knowledge/standards/repository-architecture.md)
- [架构基线镜像](../docs/architecture/LumioGameEngine_Architecture_v1.0.md)
