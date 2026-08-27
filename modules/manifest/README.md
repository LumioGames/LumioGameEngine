# manifest

> 生成规范化、可复现的 `CoreEngineManifest`。优先级：P0；状态：设计中。

## 负责什么

- 描述 Source Commit、Compiler、Feature、Platform、Architecture 和依赖。
- 记录 ABI、Capability、Artifact Hash、生成工具版本和平台链接方式。
- 生成稳定序列化结果、Manifest Hash 和兼容检查结果。

## 明确不负责什么

- 不生成上层产品的 `ReleaseManifest`、Gameplay 或内容语义。
- 不管理密钥、不定义信任根，也不修改已生成产物。

## 输入与输出

- 输入：`composition`、`root-abi`、`platform` 的产物元数据。
- 输出：Canonical Manifest、Hash、依赖描述和供 `signing`/`loader` 使用的验证输入。

## 生命周期与失败行为

`Collect -> Normalize -> Validate -> Hash -> Publish`。必填字段缺失、排序不稳定、Hash 不一致或平台声明冲突必须失败。生成时间等非确定字段不得进入 Artifact Hash。

## 验收范围

覆盖字段完整性、稳定排序、Round-trip、篡改检测、未知字段策略和 Manifest 与包内容的一致性。

## 相关文档

- [模块索引](../README.md)
- [Signing](../signing/README.md)
