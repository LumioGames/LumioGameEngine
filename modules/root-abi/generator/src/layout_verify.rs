//! Layout 检查与 layout report（规格 §8.4）。
//!
//! 布局常量**全部来自上游 layoutProfile**，本仓一个都不写死——写死了上游改布局时本仓
//! 会静默不一致（`tests/no_private_schema.rs` 有源码级断言盯着这条）。
//! 这里做的是「上游 bundle 声明的 profile」与「compiler 运行时用的 profile」是否一致，
//! 以及三种语言的产物是否都按同一 profile 生成。

use crate::error::{err, AbiGenerationError, AbiGenerationErrorKind};
use crate::input_set::RootAbiBundle;

/// 三种语言产物的布局一致性判定结果。
pub(crate) struct LayoutChecks {
    pub(crate) c_valid: bool,
    pub(crate) rust_valid: bool,
    pub(crate) csharp_valid: bool,
    pub(crate) report: serde_json::Value,
}

/// `role` 是上游给每份产物标的语言角色；三份齐备且 profile 一致才算通过。
pub(crate) fn check(
    bundle: &RootAbiBundle,
    compiler_layout_profile: &serde_json::Value,
) -> Result<LayoutChecks, AbiGenerationError> {
    if &bundle.layout_profile != compiler_layout_profile {
        return Err(err(
            AbiGenerationErrorKind::OutputHashMismatch,
            "上游 bundle 声明的 layoutProfile 与锁定 compiler 运行时使用的不一致：\
             两者必须同源，否则产物按 A 生成却按 B 校验"
                .to_string(),
        ));
    }

    let mut roles: Vec<&str> = bundle
        .output_files
        .iter()
        .map(|file| file.role.as_str())
        .collect();
    roles.sort_unstable();

    let has = |role: &str| roles.binary_search(&role).is_ok();
    let checks = LayoutChecks {
        c_valid: has("CHeader"),
        rust_valid: has("RustBinding"),
        csharp_valid: has("CSharpBinding"),
        report: serde_json::json!({
            "kind": "root-abi-layout-report",
            "baselineId": bundle.baseline_id,
            "bundleId": bundle.bundle_id,
            "schemaEpoch": bundle.schema_epoch,
            "layoutProfile": bundle.layout_profile,
            "roles": roles,
        }),
    };

    if !(checks.c_valid && checks.rust_valid && checks.csharp_valid) {
        return Err(err(
            AbiGenerationErrorKind::BlockedOnArchitectureGate,
            format!("上游 bundle 未同时声明 CHeader / RustBinding / CSharpBinding：实际 {roles:?}"),
        ));
    }
    Ok(checks)
}
