//! lumio-core-manifest——CoreEngineManifestBody 生成/校验（Canonical bytes、Manifest
//! Digest、校验报告，规格 §10）；不签名、不持有 key、不被 Loader 编译依赖。
//!
//! 脚手架状态（LCE-P0-001）：CanonicalSerializer 制品与各 Digest Golden（AG-005）、
//! capabilitySetDigest 投影（AG-006）、ABI 生成记录（AG-001）与 ArtifactIndex 投影
//! （AG-002）均未发布。不得把通用 JSON 库默认输出提升为公共语义，因此本 crate
//! 刻意不含任何模块与公共项。
