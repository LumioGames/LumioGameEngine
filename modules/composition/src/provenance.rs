//! 来源、recipe、plan 摘要链（规格 §7.2、ADR-0006 第 6 条）。
//!
//! `build_recipe_digest` 的投影输入集是 composition 私有细节，但必须满足与 BuildPlan
//! 同一套确定性规则——否则同一份配方两次 compose 会得到不同的 provenance。

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::CompositionError;
use crate::model::{
    BuildInvocation, BuildPlan, BuildProfile, FeatureSet, PackageLayout, ProvenanceRecord,
    ToolchainLock,
};

/// 「配方」= 决定构建**怎么做**的那部分计划（不含来源与架构输入，它们各自单列在
/// ProvenanceRecord 里）。字段顺序同样是编码键序的一部分。
#[derive(Serialize)]
struct BuildRecipe<'a> {
    feature_set: &'a FeatureSet,
    toolchain: &'a ToolchainLock,
    build_profile: &'a BuildProfile,
    build_invocations: &'a [BuildInvocation],
    package_layout: &'a PackageLayout,
}

fn recipe_digest(plan: &BuildPlan) -> Result<String, CompositionError> {
    let recipe = BuildRecipe {
        feature_set: &plan.feature_set,
        toolchain: &plan.toolchain,
        build_profile: &plan.build_profile,
        build_invocations: &plan.build_invocations,
        package_layout: &plan.package_layout,
    };
    let mut bytes = serde_json::to_vec(&recipe).map_err(|e| {
        crate::error::err(
            crate::error::CompositionErrorKind::NonDeterministicPlan,
            format!("BuildRecipe 规范编码失败：{e}"),
        )
    })?;
    bytes.push(b'\n');
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut out = String::with_capacity(64);
    for byte in hasher.finalize() {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    Ok(out)
}

pub(crate) fn record(
    plan: &BuildPlan,
    build_plan_digest: &str,
) -> Result<ProvenanceRecord, CompositionError> {
    let [native, voxel] = &plan.source_lock.repositories;
    Ok(ProvenanceRecord {
        architecture_baseline_id: plan.architecture.architecture_baseline_id.clone(),
        architecture_source_commit: plan.architecture.architecture_source_commit.clone(),
        source_tree_ids: [native.tree_id.clone(), voxel.tree_id.clone()],
        source_tree_digest: plan.source_lock.source_tree_digest.clone(),
        build_recipe_digest: recipe_digest(plan)?,
        build_plan_digest: build_plan_digest.to_string(),
    })
}
