//! lumio-core-contracts——架构源生成 ContractTypes 的唯一 re-export 面（规格 §6.1）。
//!
//! 本文件只做 re-export（LCE-P0-003 实现要求）；全部内容来自 `src/generated/`，
//! 后者由锁定生成器从只读架构镜像逐字节派生（基线与提交见下方常量与
//! `generated-contract-artifact.json`），无 `build.rs`，禁止手改。
//!
//! # 尚不可 re-export 的 §6.1 名字（seam，不建同名临时类型）
//!
//! 架构源 `packages/index.json` 的 12 个语言生成包（rust/csharp/descriptors）
//! `consumers` 均不含 LumioCoreEngine——本仓消费面只有 `packages/abi/`、`ids/`、
//! `schemas/`。因此以下名字的底层 Rust 生成类型尚无本仓可消费的上游制品，
//! 按「Gate 前不得创建同名临时 struct」保持缺位，待上游把本仓加入对应
//! ContractTypes 制品的 consumers 后按独立需求卡补齐：
//! `Digest256`、`TargetProfile`、`CoreEngineManifestBody`、`ArtifactIndex`、
//! `EvidenceSet`、`SignatureEnvelope`、`PackageIdentity`、
//! `VerifiedPackageDescriptor`、`LoggingEvent`、`LoggingCorrelation`、
//! `TrustDomain`。
//! 同理，`ids/index.json` 未发布 OperationId namespace，本 crate 不提供其派生。

pub mod generated;

pub use generated::contracts;
pub use generated::error_codes;
pub use generated::schema_registry;

pub use generated::error_codes::{ErrorCode, IdStatus};
pub use generated::schema_registry::{schema_by_digest, schema_by_id, SchemaEntry};
pub use generated::{
    ARCHITECTURE_BASELINE_ID, ARCHITECTURE_COMMIT, ARCHITECTURE_REPOSITORY, SCHEMA_EPOCH,
};

/// §6.1 的本地稳定名是 `CapabilityId`；registry namespace 名为 `Capability`，
/// 生成层保持 registry 拼写，crate 面按规格名 re-export。
pub use generated::error_codes::Capability as CapabilityId;
