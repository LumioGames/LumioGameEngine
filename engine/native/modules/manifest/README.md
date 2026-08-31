# manifest

> 生成规范化、可复现、无自引用的 `CoreEngineManifestBody`。优先级：P0；状态：设计中。

## 负责什么

- 描述 Source Commit、Compiler、Feature、TargetProfile 和依赖。
- 记录 ABI Identity、Capability Set、ArtifactIndex 引用、Artifact Set Digest、EvidenceSet（SBOM/License/Provenance 的 Digest）和生成工具版本。
- 按公共 Canonical Serialization 契约生成稳定序列化结果、**Manifest Digest** 和兼容检查结果。
- 维护 Digest Chain：ManifestBody 记录上游各阶段（Source、BuildPlan、ArtifactIndex、EvidenceSet）的 Digest。

## 明确不负责什么

- **Signature 不进入 ManifestBody**：签名位于分离的 SignatureEnvelope（`signing` 所有），消除自引用。
- 不定义 Canonical Serialization 算法（公共契约，架构源所有）。
- 不生成上层产品的 `ReleaseManifest`、Gameplay 或内容语义。
- 不管理密钥、不定义信任根，也不修改已生成产物。

## 输入与输出

- 输入：`composition` 的 ProvenanceRecord、`root-abi` 的 ABI 描述、`platform` 的 ArtifactIndex。
- 输出：Canonical CoreEngineManifestBody、Manifest Digest、依赖描述，供 `signing`（签名载荷）与 `loader`（包 Schema 校验）使用。

## 依赖关系

- 消费：`composition::ProvenanceRecord`、`root-abi::ABI 描述`、`platform::ArtifactIndex`。
- 被消费：`signing`（Canonical ManifestBody 为签名载荷）、`loader`（包 Schema）、`smoke`（一致性校验）、上层 Release 工具（通过 PackageIdentity 精确引用）。

## 生命周期与失败行为

`Collect -> Normalize -> Validate -> Digest -> Publish`。必填字段缺失、排序不稳定、Digest 不一致、ArtifactIndex 路径重复或平台声明冲突必须失败。生成时间等非确定字段不得进入 ManifestBody（只能存在于 SignatureEnvelope 或外部元数据）。

## 验收范围

覆盖字段完整性、稳定排序、Round-trip、篡改检测、未知字段策略、Digest Chain 完整性和 ManifestBody 与包内容的一致性。

## 相关文档

- [模块索引](../README.md)
- [Signing](../signing/README.md)
- [Loader](../loader/README.md)
