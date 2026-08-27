# signing

> CoreEngine 供应链聚合模块：证据生成、离线签名、运行时验证与信任策略四个安全域的内部分拆。优先级：P0 最小子集（测试 Signer + 运行时 Verifier）/ P1 完整（生产 Key Management/Rotation）；状态：设计中。

## 负责什么

内部按四个安全域分拆，不得合并部署：

- **`evidence-generator`**（构建期）：生成 SBOM、License 清单和 Provenance 证据，输出 EvidenceSet（各证据的 Digest 与媒体类型）。
- **`signer-tool`**（离线/CI 私钥域）：对 Canonical CoreEngineManifestBody 的精确字节生成 **Detached SignatureEnvelope**；私钥只存在于受控外部系统。
- **`runtime-verifier`**（运行时只读验证域）：对实际打开的文件句柄验证 Hash、签名与信任策略，输出 **VerifiedPackageDescriptor** 供 Loader 消费。
- **`trust-policy`**（信任策略域）：维护 Trust Root、Key Rotation 与 Revocation 元数据，以只读形式发布；包内数据不得自举为信任根。

签名载荷规则：签名只覆盖 Canonical ManifestBody 字节；SBOM/License/Provenance 以 Digest 绑定进 ManifestBody（EvidenceSet），签名后不可替换；SignatureEnvelope 与 ManifestBody 分离，时间戳与证书链只存在于 Envelope。

## 明确不负责什么

- 运行时发布包只包含 `runtime-verifier` 与只读 trust metadata；**不得包含 `signer-tool` 或任何私钥访问代码**。
- 不把生产密钥写入仓库、Prompt 或日志；密钥托管由受控外部系统负责。
- 不定义 Canonical Serialization 算法（公共契约，架构源所有）。
- 不实现用户认证、业务权限或产品 Release 路由。
- 不修改 ABI、ManifestBody 或平台二进制内容。

## 输入与输出

- 输入：`manifest` 的 Canonical ManifestBody 与 Manifest Digest、`platform` 的 ArtifactIndex 与包产物、依赖清单。
- 输出：EvidenceSet、Detached SignatureEnvelope、`runtime-verifier` 接口与 VerifiedPackageDescriptor、只读 Trust/Key Rotation 元数据。

## 依赖关系

- 消费：`manifest::CanonicalManifestBody`、`platform::ArtifactIndex`、依赖清单。
- 被消费：`loader`（仅 `runtime-verifier` 接口与 VerifiedPackageDescriptor）、`smoke`（签名与篡改 Fixture）、Release 工具（SignatureEnvelope 引用）。
- Loader 不得编译依赖 `signer-tool` 或 `evidence-generator` 的内部实现。

## 生命周期与失败行为

`Generate Evidence -> Bind Digests -> Freeze Payload -> Sign（离线） -> Verify（独立） -> Publish`。每阶段记录上一阶段 Digest。内容篡改、签名无效、信任根未知、Evidence 被替换、SBOM 缺失或许可证检查失败必须明确拒绝并给出稳定 ErrorCode；Verifier 失败必须可与 Signer 失败区分。

## 验收范围

覆盖有效/无效签名、载荷篡改、错误 Key、过期与撤销、Evidence 替换检测、Key Rotation 元数据、SBOM 完整性、许可证审计；测试密钥与生产密钥严格隔离；`runtime-verifier` 可独立于 `signer-tool` 测试。

## 相关文档

- [模块索引](../README.md)
- [Manifest](../manifest/README.md)
- [Loader](../loader/README.md)
