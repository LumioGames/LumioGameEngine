//! 架构源生成制品的只读包装（LCE-P0-003）：ContractTypes / ErrorCode /
//! Capability / Schema registry 的消费面，逐字节来自只读镜像 generated/architecture/LGE-V1.4-2026-08-27。
// 本文件由锁定生成器从只读架构镜像派生——禁止手改（rules/system.md 生成物纪律）。
// 重生成：LUMIO_CONTRACTS_REGENERATE=1 cargo test -p lumio-core-contracts --locked --test generated_integrity
// 派生输入与逐文件摘要见 modules/root-abi/contracts/generated-contract-artifact.json。

pub mod contracts;
pub mod error_codes;
pub mod schema_registry;

#[rustfmt::skip]
pub const ARCHITECTURE_BASELINE_ID: &str = "LGE-V1.4-2026-08-27";
#[rustfmt::skip]
pub const ARCHITECTURE_COMMIT: &str = "1f2ead332b3dfc3042e1495bfbe6febb8699df7e";
#[rustfmt::skip]
pub const ARCHITECTURE_REPOSITORY: &str = "https://github.com/LumioGames/LumioGameEngineArchitecture";
#[rustfmt::skip]
pub const SCHEMA_EPOCH: u32 = 1;
