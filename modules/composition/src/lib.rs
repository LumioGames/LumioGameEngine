//! composition——把锁定的 Source、Feature、TargetProfile 与工具链约束解析为不可变
//! BuildPlan 与 ProvenanceRecord；不 clone、不编译、不链接（ADR 0001、规格 §7）。
//!
//! 公开面只有模型、[`compose`]、[`verify_frozen_plan`] 与错误（规格 §7.2）。
//! **不存在接受可变 `BuildPlan` 的执行 API**：下游只能拿到 [`FrozenBuildPlan`]
//! （规格 §7.5 完成条件、ADR-0006 第 8 条）。
//!
//! 生命周期（规格 §7.4）：
//! `ResolveSources -> ResolveFeatures -> ValidateToolchain -> ValidateInputs
//!  -> EncodeDeterministically -> FreezeBuildPlan -> RecordProvenance -> Publish`

mod encode;
mod error;
mod features;
mod freeze;
mod model;
mod provenance;
mod source;
mod toolchain;
mod validate;

pub use error::{CompositionError, CompositionErrorKind};
pub use model::{
    ArchitectureDocumentPaths, ArchitectureDocumentRef, ArchitectureInputLock, BuildInvocation,
    BuildPlan, BuildProfile, ComposeRequest, FeatureCatalog, FeatureSet, FrozenBuildPlan,
    GitObjectId, PackageLayout, PlanDeclarations, ProvenanceRecord, RootAbiContractRef,
    Sha256Digest, SourceCheckoutRequest, SourceComponent, SourceLock, SourceRepository,
    ToolReference, ToolchainLock, WorkspaceRelativePath, ENVIRONMENT_WHITELIST,
    PLAN_FORMAT_VERSION,
};

use std::path::Path;
use std::sync::Arc;

/// 从一份 compose 输入产出唯一的冻结计划。
///
/// 全部校验、编码与摘要在写盘之前完成；发布是整目录一次性 rename，目标已存在即拒绝。
pub fn compose(request: ComposeRequest) -> Result<FrozenBuildPlan, CompositionError> {
    let ComposeRequest {
        workspace_root,
        architecture_lock_path,
        sources,
        requested_features,
        target_profile_document_path,
        tools_lock_path,
        output_plan_path,
        declarations,
    } = request;

    // ResolveSources
    let source_lock = source::resolve(&sources, &workspace_root)?;

    // ResolveFeatures
    let feature_set = features::resolve(&declarations.feature_catalog, &requested_features)?;

    // ValidateInputs（架构锁、只读镜像、TargetProfile、Root ABI 引用）
    let architecture = validate::resolve_architecture_inputs(
        &workspace_root,
        &architecture_lock_path,
        &target_profile_document_path,
        &declarations,
    )?;

    // ValidateToolchain
    toolchain::validate(
        &declarations.toolchain,
        &architecture.target_profile,
        &tools_lock_path,
    )?;

    // ValidateInputs（跨字段不变量）
    validate::check_build_profile(&declarations.build_profile)?;
    validate::check_package_layout(&declarations.package_layout)?;
    let build_invocations = validate::normalize_invocations(
        &declarations.build_invocations,
        &feature_set,
        &source_lock,
    )?;

    let mut plan = BuildPlan {
        plan_format_version: PLAN_FORMAT_VERSION,
        architecture: architecture.lock,
        source_lock,
        feature_set,
        target_profile_document: architecture.target_profile_document,
        toolchain: declarations.toolchain,
        build_profile: declarations.build_profile,
        root_abi_contract: RootAbiContractRef {
            abi_schema: architecture.abi_schema,
            generated_artifact_descriptor: architecture.generated_artifact_descriptor,
        },
        build_invocations,
        package_layout: declarations.package_layout,
        // 先占位，下一步用「省略本字段的编码」算出真值。
        inputs_digest: String::new(),
    };

    // EncodeDeterministically
    plan.inputs_digest = encode::inputs_digest(&plan)?;
    let plan_bytes = encode::encode_plan(&plan)?;
    let plan_digest = encode::sha256_hex(&plan_bytes);

    // RecordProvenance
    let provenance = provenance::record(&plan, &plan_digest)?;
    let provenance_bytes = encode::encode_provenance(&provenance)?;

    // Publish
    let digest_bytes = format!("{plan_digest}\n").into_bytes();
    let output = freeze::publish(
        &output_plan_path,
        &plan_bytes,
        &digest_bytes,
        &provenance_bytes,
    )?;

    Ok(FrozenBuildPlan {
        plan: Arc::new(plan),
        plan_path: output.plan_path,
        plan_digest_path: output.plan_digest_path,
        plan_digest,
        provenance_path: output.provenance_path,
    })
}

/// 只读回读一份已冻结的计划。
///
/// 消费者（root-abi generator、platform-build、manifest、evidence-generator）**必须**
/// 经此取得 `FrozenBuildPlan` 再消费，且验证与使用针对同一份已验证字节
/// （ADR-0006 第 6、8 条）。
pub fn verify_frozen_plan(plan: &Path, digest: &Path) -> Result<FrozenBuildPlan, CompositionError> {
    let bytes = std::fs::read(plan).map_err(|e| {
        CompositionError::new(
            CompositionErrorKind::InvalidConfiguration,
            format!("读取 {} 失败：{e}", plan.display()),
        )
    })?;
    let sidecar = std::fs::read_to_string(digest).map_err(|e| {
        CompositionError::new(
            CompositionErrorKind::InvalidConfiguration,
            format!("读取 {} 失败：{e}", digest.display()),
        )
    })?;

    let expected = sidecar.strip_suffix('\n').unwrap_or(&sidecar);
    let actual = encode::sha256_hex(&bytes);
    if expected != actual {
        return Err(CompositionError::new(
            CompositionErrorKind::NonDeterministicPlan,
            format!("sidecar 摘要与计划字节不符（sidecar {expected}，实际 {actual}）"),
        ));
    }

    // 版本号先于一切其他解析（ADR-0006 第 3 条 fail-closed）。
    let probe: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        CompositionError::new(
            CompositionErrorKind::NonDeterministicPlan,
            format!("计划不是合法 JSON：{e}"),
        )
    })?;
    match probe.get("plan_format_version").and_then(|v| v.as_u64()) {
        Some(version) if version == u64::from(PLAN_FORMAT_VERSION) => {}
        other => {
            return Err(CompositionError::new(
                CompositionErrorKind::InvalidConfiguration,
                format!(
                    "plan_format_version={other:?}，本仓只接受 {PLAN_FORMAT_VERSION}；\
                     跨版本不做兼容，重新 compose"
                ),
            ))
        }
    }

    let decoded: BuildPlan = serde_json::from_slice(&bytes).map_err(|e| {
        CompositionError::new(
            CompositionErrorKind::NonDeterministicPlan,
            format!("计划解码失败（未知键或类型不符）：{e}"),
        )
    })?;

    // 重编码零差异：sidecar 只能证明「字节没被改」，证明不了「这些字节是规范编码」。
    // 攻击者能改文件就能重算 sidecar，此时只剩这一步与 inputs_digest 还能发现。
    let reencoded = encode::encode_plan(&decoded)?;
    if reencoded != bytes {
        return Err(CompositionError::new(
            CompositionErrorKind::NonDeterministicPlan,
            format!("计划 {} 不是规范编码（重编码后字节不同）", plan.display()),
        ));
    }
    let recomputed_inputs = encode::inputs_digest(&decoded)?;
    if recomputed_inputs != decoded.inputs_digest {
        return Err(CompositionError::new(
            CompositionErrorKind::NonDeterministicPlan,
            format!(
                "inputs_digest 失配（计划内 {}，重算 {recomputed_inputs}）",
                decoded.inputs_digest
            ),
        ));
    }

    let plan_dir = plan.parent().ok_or_else(|| {
        CompositionError::new(
            CompositionErrorKind::InvalidConfiguration,
            "计划文件没有父目录".to_string(),
        )
    })?;
    Ok(FrozenBuildPlan {
        plan: Arc::new(decoded),
        plan_path: plan.to_path_buf(),
        plan_digest_path: digest.to_path_buf(),
        plan_digest: actual,
        provenance_path: plan_dir.join("provenance.json"),
    })
}
