# 0002 · signing 内部按四个安全域分拆，运行时只发布 runtime-verifier

- 日期:2026-08-27
- 状态:生效

## 背景

架构 Review（CE-007）指出 signing 顶层模块混合了四种安全边界:包签名（离线/CI 私钥域）、运行时签名验证（只读验证域）、Trust Root 与 Key Rotation（信任策略域）、SBOM/License 生成（证据生成域）。混合部署会导致生产 Loader 误链接 Signer 或私钥 Provider，Verifier 与 Signer 被迫共同升级。

## 决策

- 保留 `signing` 聚合目录名，内部强制分拆为 `evidence-generator`、`signer-tool`、`runtime-verifier`、`trust-policy` 四个安全域，不得合并部署。
- **运行时发布包只包含 `runtime-verifier` 与只读 trust metadata**;`signer-tool` 与任何私钥访问代码不进入运行时产物。
- 签名载荷只覆盖 Canonical CoreEngineManifestBody 的精确字节;SBOM/License/Provenance 以 Digest 绑定进 ManifestBody（EvidenceSet），签名后不可替换;SignatureEnvelope 与 ManifestBody 分离（消除 Review CE-001 的自引用循环）。
- 签名算法、载荷 Canonicalization 与信任模型属公共契约，由架构源 ADR-018 冻结;本 ADR 只固化本仓内部代码组织与部署隔离。

## 后果

- Verifier 可独立于 Signer 升级与测试;测试密钥与生产密钥天然隔离。
- 四个子域各自需要独立的构建目标与发布清单，目录与构建脚本复杂度上升。
- Loader 对 signing 的依赖收窄为 `runtime-verifier` 接口，杜绝离线工具内部格式泄漏。
