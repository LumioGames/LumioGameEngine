//! lumio-core-root-abi——Root API 运行时视图与绑定（`AbiExpectation`、`SymbolResolver`、
//! `RootApiTableView`、`bind_root_api`，规格 §8.3）。
//!
//! slot、布局与 Handle 语义一律来自架构源，本仓不补写:
//! [`generated`] 是上游 compiler 产出的 Rust 绑定**原文**（LCE-P0-005 发布到
//! `modules/root-abi/generated/<baseline>/rust/contracts.rs`），经 `#[path]` 只读引入，
//! 不复制、不改写；本 crate 的每一个布局常量都由它反推。
//!
//! # 与 §8.3 的两处偏差（均为「不自造公共语义」，见交付说明）
//!
//! 1. **不提供 `RootApiTableView::supports(CapabilityId)`。** ADR-040
//!    「What this bundle deliberately does not freeze」明写：V1 既未冻结
//!    `capability_bits` 是 bitmask 还是计数，也未冻结任何位位置；ID Registry 的
//!    `Capability` numeric 是枚举序数而非位位置，且「a consumer must not derive a
//!    capability key from either source」。按 `CapabilityId` 判定必然要本仓自造位
//!    映射，因此保持缺位（seam，不建同名临时方法），只经
//!    [`RootApiTableView::capability_bits`] 暴露不透明原值，绑定期做**精确相等**校验。
//!    上游确认位语义后按独立需求卡补齐。
//! 2. **不校验单张 API table 的 `version` 期望值。** 上游把 per-table `version`
//!    发布在 `metadata/native-managed-abi.json` 与 bundle JSON 里，没有任何
//!    Rust 可消费的常量；在运行时闭包内为此引入 JSON 解析依赖不成比例。绑定期
//!    读出并经 [`ApiTableView::version`] 如实公开，但不与任何期望值比较。
//!
//! 两处都属「上游未发布可消费真值」，不以本地临时格式、alias 或假 Golden 填补。

mod bind;
mod error;
mod expectation;
mod handle_guard;
mod symbol;
mod table_view;

/// 架构源生成的 Root ABI Rust 绑定（原文只读引入，禁止手改）。
///
/// 发布路径 `modules/root-abi/generated/LGE-V1.4-2026-08-27/rust/contracts.rs`，
/// 产出与摘要链见同目录 `generated-contract-artifact.json`（LCE-P0-005）。
/// 安全消费方应使用 [`RootApiTableView`]，它不交出任何裸指针。
#[rustfmt::skip]
#[path = "../../generated/LGE-V1.4-2026-08-27/rust/contracts.rs"]
pub mod generated;

pub use bind::bind_root_api;
pub use error::{RootAbiError, RootAbiErrorKind};
pub use expectation::{AbiExpectation, Endianness, ENTRY_SYMBOL};
pub use handle_guard::HandleGuard;
pub use symbol::{SymbolLookupError, SymbolResolver};
pub use table_view::{ApiTableView, GeneratedApiTablesView, RootApiTableView};
