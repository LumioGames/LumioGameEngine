# LumioGameEngineArchitecture

> LumioGameEngine V3 的唯一架构源、公共契约目录和跨仓库实现基线。

## 版本与状态

- **Baseline**：`LGE-V1.4-2026-08-27`
- **状态**：Implementation Baseline
- **规范正文**：[`docs/architecture/LumioGameEngine_Architecture_v1.4.md`](docs/architecture/LumioGameEngine_Architecture_v1.4.md)
- **ADR 索引**：[`docs/architecture/ADR_INDEX.md`](docs/architecture/ADR_INDEX.md)
- **ADR 正文**：[`docs/adr/README.md`](docs/adr/README.md)
- **待确认决策**：[`docs/architecture/DECISIONS_PENDING.md`](docs/architecture/DECISIONS_PENDING.md)
- **契约 Schema**：[`schemas/README.md`](schemas/README.md)
- **ID Registry**：[`ids/README.md`](ids/README.md)
- **Fixture**：[`fixtures/README.md`](fixtures/README.md)
- **契约工具**：[`tools/README.md`](tools/README.md)
- **最终评审合并稿**：[`docs/reviews/LumioGameEngine_V3_Architecture_Review_Final_2026-08-27.md`](docs/reviews/LumioGameEngine_V3_Architecture_Review_Final_2026-08-27.md)

本仓库是架构、状态机、Schema、依赖图和变更规则的事实源，不拥有任何运行中的 World、连接或 Gameplay 状态。七个实现仓库可以保留同版本镜像，但不能独立修改共享基线；改变基线必须通过 ADR、更新 BaselineId 和同步检查。现行基线为 `LGE-V1.4-2026-08-27`；Accepted ADR 不可改写，只能由新 ADR 取代（ADR-015 保持 Reserved）。尚未裁决的实现选型见 `docs/architecture/DECISIONS_PENDING.md`。

## 负责范围

- 定义七仓库的职责、所有权、依赖和运行时加载关系。
- 定义 Tick、Determinism、Entity、Replication、Cross-World、ABI、Manifest、Persistence、Logging、Config 和 Release 的公共语义。
- 维护 ID Namespace、Schema 版本、兼容判定、Failure Bundle 和统一测试结果格式。
- 规定成熟开源方案的引入、许可证、SBOM、漏洞和适配边界。
- 发布架构镜像、变更记录和实现阶段的退出条件。

## Architecture Gate 产物

当前基线已经有可执行的公共契约：`schemas/index.json` 注册 41 个 P0 Schema（含 Tick 相矩阵、GAS 生命周期、恢复记录、Replication typed body、双 Scope 激活、状态机描述符、Root ABI Bundle 生成记录、Canonical/Digest、Signature/Trust、Loader 与 Evidence Profile）、4 个 P1 Schema（含 Voxel Snapshot/Diff 载荷与 DurabilityAck）和 1 个 P2 Mod 预留；`ids/index.json` 维护版本化 ID Namespace（含 MessageType `BaselineAck`/`DeltaAck`/`Error` 与 Gate 拒绝错误码 1040–1043）；`fixtures/index.json` 注册 190 个正向/失败样例（含 12 个状态机描述符实例）；`tools/lumio_contract.py validate` 在本地和 CI 中执行结构与关键语义校验，`common.schema.json` 承载 ADR-037 下沉的公共 `$defs`。它们是生成器、Serializer、ABI Header、Binding 和各仓库测试的输入，不是运行时库。

```text
python3 -m pip install -r requirements-dev.txt
python3 tools/lumio_contract.py validate
python3 tools/lumio_contract.py validate --fixture txn/committed
python3 tools/lumio_contract.py validate --json > contract-result.json
python3 tools/lumio_contract.py generate --out packages
```

Published V1.4 artifacts live under [`packages/`](packages/): six kinds × Rust/C#, each with a schema-valid descriptor five-tuple (`baselineId`, `schemaEpoch`, `compilerHash`, `inputHash`, `outputHash`) and `implementationDependencies=[]`. Implementation repositories consume those packages by path or hash-locked copy; they must not vendor a rewritten Schema or depend on `tools/` itself.

`packages/abi/` is the ADR-040 Root ABI bundle, derived from `schemas/native-managed-abi.schema.json` and its validated ABI document: the generated `lumio_core.h`, the generation record and layout Golden `root-abi-bundle.json` (frozen compiler identity, input set, `typeRef` → C/C#/Rust mapping, `linux-x86_64-glibc` struct sizes and slot offsets), and the digests of the Rust (`root_abi.rs`) and C# (`RootAbi.cs`) bindings published inside the `LanguageBinding` artifact. `LumioCoreEngine` builds its C/C#/Rust layout tests from this bundle alone and hand-writes no header, interop struct or template; `validate` re-derives the bundle and rejects a hand-edited or stale publication.

`packages/canonical/canonical-digest-profile.json` is the ADR-041 Canonical and Digest Profile: the `CanonicalJsonV1` form parameters (ASCII-escaped output, code-point member order, integer-only numbers, reject unknown and duplicate members), the digest input and sort rule for each of `manifestDigest`, `artifactSetDigest`, `artifactIndexDigest`, `targetProfileDigest`, `capabilitySetDigest` and the ADR-045 `mappingSetHash` — including the `artifactSetDigest` self-reference rule — and ten self-verifying Golden vectors. `validate` re-canonicalizes and re-digests every Golden from its input and re-derives the published profile, and it now recomputes `artifact-index.artifactSetDigest` instead of trusting it. A generic JCS library's defaults are explicitly not the contract.

`packages/binary/lumio-bin-profile.json` is the ADR-047 `LumioBinV1` profile — the binary counterpart of `CanonicalJsonV1`, and the primitive layer ADR-010 referred to as "the same canonical codec rules" and ADR-035 assumed when it froze voxel payload ordering and hashing: little-endian, fixed-width integers (`u8`/`u16`/`u32`/`u64`, two's-complement `i32`/`i64`), UTF-8 strings and byte strings with a `u32` **byte**-length prefix, arrays with a `u32` count prefix in document order, structs concatenated in **schema declaration order** with no padding and a closed field set, and no floating-point types. Digests are prefix-free `SHA-256` over the encoded bytes and nothing else (`digestInput = EncodedBytesOnly`). The profile publishes six self-verifying Golden vectors **and eleven rejection vectors**: `validate` re-encodes every Golden from its `layout` and `value`, recomputes its digest, and re-runs every rejection to confirm the encoder refuses it with the declared `error` — "it failed somehow" is not conformance. A clean-room implementation built from this file alone, with no access to the ADR or the generator, reproduces every Golden byte-for-byte. `packages/rust/lumio-gen-canonical-serializer/CHECKSUM_DOMAIN.md` is the matching B profile for `snapshot-header`: `hash` covers the payload bytes, `checksum` covers the header minus both digest members in the `SnapshotHeaderV1` domain, with a worked Golden the gate recomputes.

`packages/trust/trust-profile.json` is the ADR-042 Signature and Trust Profile: `LumioSignatureV1` (RFC 8032 pure Ed25519, raw lower-case hex for both signature and public key), the domain-separated preimage `LumioSignatureV1\0<trustDomain>\0<payloadType>\0<payloadDigest>` that is actually signed, the derived `keyId` rule, the total rejection order (`SignatureMissing` -> `TrustRootUnknown` -> `KeyRevoked` -> `SignatureInvalid` -> `TrustPolicyRejected`), the `Test` trust policy, and eight vectors covering accept, tampered signature, tampered digest, wrong trust domain, unknown key, revoked key and both validity-window edges. `validate` re-derives every `keyId`, rebuilds every preimage and runs Ed25519 over every vector; its verifier self-tests against the RFC 8032 section 7.1 vectors first. No private key is published - a downstream signer proves interoperability by having its own signature over the frozen preimage accepted.

`packages/loader/loader-profile.json` is the ADR-043 Loader profile: terminal states stay terminal, so a retry after `FailedRolledBack` and an Acquire after `Released` each begin a new loader instance rather than a transition; the PackageIdentity latch is by identity, not by time, so a concurrent Acquire for the same identity returns the existing Lease and a different identity is `PackageIdentityConflict`; and the reported error is the root cause, with `PartialLoadRolledBack` as a floor rather than a winner. `packages/evidence/evidence-profile.json` is the ADR-044 Evidence profile: CycloneDX 1.6, SPDX 2.3 and SLSA-v1 1.0 with their media types, digests over the **raw published bytes** (never a canonicalization of a third party's JSON), exact bidirectional coverage between `evidenceSet` and the ArtifactIndex, and `DigestOnly` validation at the load boundary — licence acceptability is an operator trust decision (`TrustPolicyRejected`), not an evidence check. `validate` re-evaluates every vector in both profiles.

`schemas/replication-envelope.schema.json` 的 typed body 按 ADR-045 **逐 MessageType 闭合**：每个已注册 MessageType 有一份 `if`/`then` 子句固定其完整合法成员集并置 `additionalProperties: false`，因此「两个实现同时通过门禁却携带各自私有载荷」——ADR-028 拒绝 free-form payload 时给出的正是这个理由——不再可达。`mappingSetHash` 是 `hash256`，取值为 ADR-041 第六个摘要域 `ReplicationMappingSetV1` 的摘要；**空映射集不需要哨兵常量**，空 `mappings` 数组走同一条规则即得确定值（Golden `EmptyMappingSet`）。`length` 只约束为不得超过本信封自己的 `transportPolicy.maxMessageBytes`——它是**上界而非字节数声明**，因为信封的线字节编码尚未冻结（见 ADR-045 §4）。**ADR-045 未冻结任何世界状态载荷**：`FullSnapshot`/`Delta` 仍无承载世界状态的成员，MVP 验收「另一个客户端看见方块被挖」仍被阻塞。

实现仓库在接入代码前，必须为自己的领域 Schema 增加至少一份正向和一份失败 Fixture，并记录 Compiler/Input/Output Hash。具体 Transport、Codec、压缩、日志 Sink、存储后端和耐久级别仍按 ADR-006/009/010/011/012 的选型门评审，不在公共 Schema 中提前绑定供应商。

## 不负责范围

- 不实现 ECS、Voxel、网络、CoreCLR、Renderer 或具体游戏内容。
- 不替代各实现仓库的单元测试、Benchmark 和产品数据。
- 不把未验证的第三方库类型写入稳定 ABI 或 Gameplay Contract。

## V1 必须考虑的能力

- 有界异步、多线程日志；Diagnostic、Audit、Txn Journal、Command Log、Metrics、Trace 和 Failure Bundle 分级处理。
- Snapshot/WAL/Command Log 的版本化编码与反序列化、校验、恢复、迁移和 Save/Load 兼容。
- Schema 校验、配表编译、typed Table Reader、不可变 Tick 配置快照和生产版本切换。
- ReleaseCatalog、多个 Product/Release 并行、滚动更新、Session 排空以及 Graceful/Forced 维护踢出。
- P2 ModManifest、Capability、权限、资源配额和存档挂接点；V1 不加载第三方 Native/Managed Mod。
- 满足需求时按 OSS → 上游/适配 → 参考实现 → 最小自研的顺序决策，并记录许可证和供应链证据。

## 仓库使用规则

1. 实现仓库只能引用已发布的 Baseline、Schema 和 Artifact，不依赖本仓库的运行时代码。「已发布 Artifact」包含 ContractRuntime 支持库（纯 Rust crate / 纯 C# assembly，按 BaselineId 版本化）；「本仓运行时代码」指 `tools/` 下的校验与生成工具本体，后者永不被实现仓引用（ADR-039）。
2. 共享文档镜像必须保持 `BaselineId` 和内容 Hash 一致。
3. 新增公共字段、状态、错误码或依赖方向必须先添加 ADR 和失败语义。
4. P2 表示“实现可以后置”，不表示“架构可以删除”；P2 能力必须有所有者、接口位置和兼容策略。
