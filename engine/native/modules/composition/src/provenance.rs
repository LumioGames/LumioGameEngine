//! 来源、recipe、plan 摘要链（规格 §7.2、ADR-0006 第 6 条）。
//!
//! `build_recipe_digest` 的投影输入集是 composition 私有细节，但必须满足与 BuildPlan
//! 同一套确定性规则——否则同一份配方两次 compose 会得到不同的 provenance。编码与摘要
//! 一律复用 `encode` 模块，不在这里另写一份：两处各自演化正是编码漂移的入口。

use serde::Serialize;

use crate::encode::{encode, sha256_hex};
use crate::error::CompositionError;
use crate::model::{
    BuildInvocation, BuildPlan, BuildProfile, FeatureSet, PackageLayout, ProvenanceRecord,
    ToolchainLock,
};

/// 「配方」= 决定构建**怎么做**的那部分计划（不含来源与架构输入，它们各自单列在
/// ProvenanceRecord 里）。
///
/// 这是一个**投影**，不是 BuildPlan 的子集复制：字段增删或顺序调整都会改变
/// `build_recipe_digest`，属于摘要口径变更。本文件末尾的 Golden 字节常量锁住它——
/// 没有 Golden 的投影结构可以被静默改写而测试全绿。
#[derive(Serialize)]
struct BuildRecipe<'a> {
    feature_set: &'a FeatureSet,
    toolchain: &'a ToolchainLock,
    build_profile: &'a BuildProfile,
    build_invocations: &'a [BuildInvocation],
    package_layout: &'a PackageLayout,
}

impl<'a> BuildRecipe<'a> {
    fn of(plan: &'a BuildPlan) -> Self {
        BuildRecipe {
            feature_set: &plan.feature_set,
            toolchain: &plan.toolchain,
            build_profile: &plan.build_profile,
            build_invocations: &plan.build_invocations,
            package_layout: &plan.package_layout,
        }
    }
}

fn recipe_digest(plan: &BuildPlan) -> Result<String, CompositionError> {
    let bytes = encode(&BuildRecipe::of(plan), "BuildRecipe")?;
    Ok(sha256_hex(&bytes))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// BuildRecipe 投影的 Golden 字节。**改动即 `build_recipe_digest` 口径变更**——
    /// 调字段顺序、增删字段都会让本测试红，那是版本事件，不是「更新期望值」。
    const GOLDEN_RECIPE: &str = concat!(
        r#"{"feature_set":{"enabled":["中文-feature","ctrl\u0001flag","unit\u001fsep"],"#,
        r#""disabled":[]},"#,
        r#""toolchain":{"rustc":{"tool_id":"rustc","version":"1.89.0","#,
        r#""executable_sha256":"2222222222222222222222222222222222222222222222222222222222222222"},"#,
        r#""cargo":{"tool_id":"cargo","version":"1.89.0","#,
        r#""executable_sha256":"3333333333333333333333333333333333333333333333333333333333333333"},"#,
        r#""linker":{"tool_id":"cc","version":"11.4.0","#,
        r#""executable_sha256":"4444444444444444444444444444444444444444444444444444444444444444"},"#,
        r#""target_triple":"x86_64-unknown-linux-gnu","sdk":null},"#,
        r#""build_profile":{"cargo_profile":"release","panic_strategy":"abort","lto":true,"#,
        r#""codegen_units":4294967295,"debug_symbols":false},"#,
        r#""build_invocations":[{"source_component":"LumioNativeCore","#,
        r#""manifest_path":"build/sources/native/Cargo.toml","package":"lumio-native-core","#,
        r#""target":"x86_64-unknown-linux-gnu","profile":"release","features":["中文-feature"],"#,
        r#""no_default_features":true,"rustflags":["-Cforce-frame-pointers=yes"],"#,
        r#""environment":{"CARGO_NET_OFFLINE":"true"}}],"#,
        r#""package_layout":{"staging_root":"build/platform/p0/staging","#,
        r#""native_root":"build/platform/p0/staging/native","#,
        r#""include_root":"build/platform/p0/staging/include","#,
        r#""managed_root":"build/platform/p0/staging/managed","#,
        r#""metadata_root":"build/platform/p0/staging/metadata","#,
        r#""evidence_root":"build/platform/p0/staging/evidence","#,
        r#""symbols_root":"build/platform/p0/staging/symbols"}}"#,
        "\n"
    );

    #[test]
    fn recipe_projection_matches_golden_bytes() {
        let plan = crate::encode::tests::golden_plan();
        let bytes = encode(&BuildRecipe::of(&plan), "BuildRecipe").expect("编码成功");
        assert_eq!(String::from_utf8(bytes).expect("UTF-8"), GOLDEN_RECIPE);
    }

    #[test]
    fn recipe_digest_ignores_source_and_architecture_but_tracks_build_inputs() {
        let mut plan = crate::encode::tests::golden_plan();
        let baseline = recipe_digest(&plan).expect("算摘要");

        // 来源与架构输入不属于「怎么构建」，单列在 ProvenanceRecord 的其他字段里。
        plan.architecture.architecture_baseline_id = "LGE-V9.9-1999-01-01".to_string();
        plan.source_lock.repositories[0].commit = "9".repeat(40);
        assert_eq!(baseline, recipe_digest(&plan).expect("再算"));

        // 构建输入变了就必须变。
        plan.build_profile.codegen_units = 1;
        assert_ne!(baseline, recipe_digest(&plan).expect("三算"));
    }
}
