//! Feature 解析与工具链漂移（规格 §7.5 负向 Fixture：未知 / 冲突 / 重复 Feature、rustc 漂移）。
//!
//! 验收项 2「source/toolchain/feature 任一漂移明确失败」的 feature + toolchain 部分，
//! 以及验收项 3「平台参数全部已在计划中」。

mod common;

use common::TempWorkspace;
use lumio_core_composition::{compose, CompositionErrorKind};

#[test]
fn unknown_requested_feature_is_rejected() {
    let ws = TempWorkspace::create("feature-unknown");
    let mut request = ws.request("unknown");
    request
        .requested_features
        .insert("not-a-declared-feature".to_string());

    let error = compose(request).expect_err("未知 feature 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::UnknownFeature);
}

#[test]
fn conflicting_features_are_rejected() {
    let ws = TempWorkspace::create("feature-conflict");
    let mut request = ws.request("conflict");
    // catalog 声明 client-prediction 与 server-authority 互斥，同时请求即冲突。
    request
        .requested_features
        .insert("client-prediction".to_string());

    let error = compose(request).expect_err("冲突 feature 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::FeatureConflict);
}

#[test]
fn duplicate_entries_in_feature_catalog_are_rejected() {
    let ws = TempWorkspace::create("feature-duplicate");
    let mut request = ws.request("duplicate");
    request
        .declarations
        .feature_catalog
        .known
        .push("voxel-streaming".to_string());

    let error = compose(request).expect_err("catalog 重复声明必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn resolved_feature_set_partitions_catalog_and_is_sorted() {
    let ws = TempWorkspace::create("feature-partition");
    let frozen = compose(ws.request("partition")).expect("冻结成功");

    let features = &frozen.plan.feature_set;
    assert_eq!(
        features.enabled,
        vec![
            "server-authority".to_string(),
            "voxel-streaming".to_string()
        ]
    );
    assert_eq!(
        features.disabled,
        vec![
            "client-prediction".to_string(),
            "voxel-persistence".to_string()
        ],
        "未请求的已知 feature 必须显式记为 disabled——计划要自描述，不留由下游猜的空白"
    );
}

#[test]
fn build_invocation_feature_outside_enabled_set_is_rejected() {
    let ws = TempWorkspace::create("feature-invocation");
    let mut request = ws.request("invocation");
    request.declarations.build_invocations[0]
        .features
        .push("voxel-persistence".to_string());

    let error = compose(request).expect_err("调用引用未启用 feature 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn rustc_version_drifting_from_target_profile_is_rejected() {
    let ws = TempWorkspace::create("toolchain-rustc");
    let mut request = ws.request("rustc-drift");
    // TargetProfile 文档钉定 toolchain.version = 1.89.0。
    request.declarations.toolchain.rustc.version = "1.90.0".to_string();

    let error = compose(request).expect_err("rustc 漂移必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::ToolchainMismatch);
}

#[test]
fn malformed_tool_digest_is_rejected() {
    let ws = TempWorkspace::create("toolchain-digest");
    let mut request = ws.request("bad-digest");
    request.declarations.toolchain.linker.executable_sha256 = "NOT-A-DIGEST".to_string();

    let error = compose(request).expect_err("非 64 位小写十六进制摘要必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::ToolchainMismatch);
}

#[test]
fn target_triple_inconsistent_with_target_profile_is_rejected() {
    let ws = TempWorkspace::create("toolchain-triple");
    let mut request = ws.request("bad-triple");
    request.declarations.toolchain.target_triple = "aarch64-apple-darwin".to_string();

    let error = compose(request).expect_err("三元组与 TargetProfile 不符必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::TargetNotApplicable);
}

#[test]
fn environment_key_outside_whitelist_is_rejected() {
    let ws = TempWorkspace::create("feature-env");
    let mut request = ws.request("env");
    // ADR-0006 第 5 条：V1 白名单只有 CARGO_NET_OFFLINE。
    request.declarations.build_invocations[0]
        .environment
        .insert("RUSTFLAGS".to_string(), "-Cdebuginfo=0".to_string());

    let error = compose(request).expect_err("白名单外环境变量必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn order_sensitive_rustflag_conflict_is_rejected() {
    let ws = TempWorkspace::create("feature-rustflags");
    let mut request = ws.request("rustflags");
    // 同键不同值：排序去重后语义取决于顺序，ADR-0006 第 2 条要求 compose 期拒绝，
    // 不允许顺序语义潜入计划。
    request.declarations.build_invocations[0]
        .rustflags
        .push("-Cforce-frame-pointers=no".to_string());

    let error = compose(request).expect_err("同键不同值 rustflag 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn plan_carries_every_platform_parameter_needed_downstream() {
    let ws = TempWorkspace::create("feature-platform-params");
    let frozen = compose(ws.request("platform-params")).expect("冻结成功");
    let plan = &frozen.plan;

    // 验收项 3：platform 专属参数缺失时必须重新 compose，platform 不得补写——
    // 因此这些字段必须在计划里齐备。
    assert_eq!(plan.toolchain.target_triple, "x86_64-unknown-linux-gnu");
    assert_eq!(plan.build_profile.cargo_profile, "release");
    assert_eq!(plan.build_profile.panic_strategy, "abort");
    assert_eq!(plan.build_invocations.len(), 2);
    assert!(plan
        .package_layout
        .native_root
        .starts_with(&plan.package_layout.staging_root));
    assert!(!plan.package_layout.symbols_root.is_empty());
    assert_eq!(plan.target_profile_document.source_sha256.len(), 64);
    assert_eq!(plan.root_abi_contract.abi_schema.source_sha256.len(), 64);
    assert_eq!(plan.architecture.lock_file_digest.len(), 64);
    assert_eq!(plan.inputs_digest.len(), 64);
}
