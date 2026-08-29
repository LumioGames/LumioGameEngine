//! ADR-0006 的仓内确定性编码，仅限 BuildPlan 与 ProvenanceRecord。
//!
//! 规则（ADR-0006 第 2 条，全部由本文件末尾的 Golden 单测锁定）：
//! UTF-8 无 BOM；紧凑 JSON；文件恰以一个 LF 结尾；只有无符号整数与布尔，禁浮点；
//! 控制字符转 `\u00XX` 小写十六进制，不转义 `/`，非 ASCII 原样输出；
//! 对象键按结构体字段声明序发出；解码端 typed + `deny_unknown_fields`。
//!
//! 这不是架构源的 CanonicalSerializer：那管公共载荷的互操作，BuildPlan 是仓内输入，
//! 两者不共享实现与 Golden。

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{err, CompositionError, CompositionErrorKind};
use crate::model::{
    ArchitectureDocumentRef, ArchitectureInputLock, BuildInvocation, BuildPlan, BuildProfile,
    FeatureSet, PackageLayout, ProvenanceRecord, RootAbiContractRef, SourceLock, ToolchainLock,
};

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn encode<T: Serialize>(value: &T, what: &str) -> Result<Vec<u8>, CompositionError> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| {
        err(
            CompositionErrorKind::NonDeterministicPlan,
            format!("{what} 规范编码失败：{e}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// `BuildPlan` 去掉 `inputs_digest` 后的投影：`inputs_digest` 是对这份字节的 SHA-256，
/// 自排除避免自引用（ADR-0006 第 6 条）。字段顺序必须与 `BuildPlan` 完全一致。
#[derive(Serialize)]
struct BuildPlanCore<'a> {
    plan_format_version: u32,
    architecture: &'a ArchitectureInputLock,
    source_lock: &'a SourceLock,
    feature_set: &'a FeatureSet,
    target_profile_document: &'a ArchitectureDocumentRef,
    toolchain: &'a ToolchainLock,
    build_profile: &'a BuildProfile,
    root_abi_contract: &'a RootAbiContractRef,
    build_invocations: &'a [BuildInvocation],
    package_layout: &'a PackageLayout,
}

impl<'a> BuildPlanCore<'a> {
    fn of(plan: &'a BuildPlan) -> Self {
        BuildPlanCore {
            plan_format_version: plan.plan_format_version,
            architecture: &plan.architecture,
            source_lock: &plan.source_lock,
            feature_set: &plan.feature_set,
            target_profile_document: &plan.target_profile_document,
            toolchain: &plan.toolchain,
            build_profile: &plan.build_profile,
            root_abi_contract: &plan.root_abi_contract,
            build_invocations: &plan.build_invocations,
            package_layout: &plan.package_layout,
        }
    }
}

/// `inputs_digest` 的取值：对「省略 inputs_digest 字段后的 BuildPlan 规范编码字节」
/// 取 SHA-256。注意投影字节同样以一个 LF 结尾——摘要口径的一部分，不可省。
pub(crate) fn inputs_digest(plan: &BuildPlan) -> Result<String, CompositionError> {
    let bytes = encode(&BuildPlanCore::of(plan), "BuildPlan(core)")?;
    Ok(sha256_hex(&bytes))
}

pub(crate) fn encode_plan(plan: &BuildPlan) -> Result<Vec<u8>, CompositionError> {
    encode(plan, "BuildPlan")
}

pub(crate) fn encode_provenance(
    provenance: &ProvenanceRecord,
) -> Result<Vec<u8>, CompositionError> {
    encode(provenance, "ProvenanceRecord")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::model::{SourceComponent, SourceRepository, ToolReference};

    /// 覆盖 ADR-0006 第 2 条每一条字节规则的合成计划。数值刻意用边界值，字符串刻意
    /// 含非 ASCII、控制字符与正斜杠——这些正是最容易在换实现时悄悄漂移的地方。
    fn golden_plan() -> BuildPlan {
        let document = |path: &str| ArchitectureDocumentRef {
            source_path: path.to_string(),
            source_sha256: "0".repeat(64),
        };
        BuildPlan {
            plan_format_version: 1,
            architecture: ArchitectureInputLock {
                architecture_baseline_id: "LGE-V1.4-2026-08-27".to_string(),
                architecture_source_repository: "https://example.invalid/a/b".to_string(),
                architecture_source_commit: "a".repeat(40),
                lock_file: "architecture.lock.json".to_string(),
                lock_file_digest: "b".repeat(64),
            },
            source_lock: SourceLock {
                repositories: [
                    SourceRepository {
                        component: SourceComponent::LumioNativeCore,
                        repository: "https://example.invalid/native".to_string(),
                        checkout_root: "build/sources/native".to_string(),
                        commit: "c".repeat(40),
                        tree_id: "d".repeat(40),
                    },
                    SourceRepository {
                        component: SourceComponent::LumioVoxelEngine,
                        repository: "https://example.invalid/voxel".to_string(),
                        checkout_root: "build/sources/voxel".to_string(),
                        commit: "e".repeat(40),
                        tree_id: "f".repeat(40),
                    },
                ],
                source_tree_digest: "1".repeat(64),
            },
            feature_set: FeatureSet {
                // 非 ASCII 与控制字符：转义规则的直接判据。
                enabled: vec!["中文-feature".to_string(), "ctrl\u{1}flag".to_string()],
                disabled: vec![],
            },
            target_profile_document: document("fixtures/valid/target-profile-linux-server.json"),
            toolchain: ToolchainLock {
                rustc: ToolReference {
                    tool_id: "rustc".to_string(),
                    version: "1.89.0".to_string(),
                    executable_sha256: "2".repeat(64),
                },
                cargo: ToolReference {
                    tool_id: "cargo".to_string(),
                    version: "1.89.0".to_string(),
                    executable_sha256: "3".repeat(64),
                },
                linker: ToolReference {
                    tool_id: "cc".to_string(),
                    version: "11.4.0".to_string(),
                    executable_sha256: "4".repeat(64),
                },
                target_triple: "x86_64-unknown-linux-gnu".to_string(),
                sdk: None,
            },
            build_profile: BuildProfile {
                cargo_profile: "release".to_string(),
                panic_strategy: "abort".to_string(),
                lto: true,
                codegen_units: u32::MAX,
                debug_symbols: false,
            },
            root_abi_contract: RootAbiContractRef {
                abi_schema: document("schemas/root-abi-bundle.schema.json"),
                generated_artifact_descriptor: document("packages/abi/root-abi-bundle.json"),
            },
            build_invocations: vec![BuildInvocation {
                source_component: SourceComponent::LumioNativeCore,
                manifest_path: "build/sources/native/Cargo.toml".to_string(),
                package: "lumio-native-core".to_string(),
                target: "x86_64-unknown-linux-gnu".to_string(),
                profile: "release".to_string(),
                features: vec!["中文-feature".to_string()],
                no_default_features: true,
                rustflags: vec!["-Cforce-frame-pointers=yes".to_string()],
                environment: BTreeMap::from([(
                    "CARGO_NET_OFFLINE".to_string(),
                    "true".to_string(),
                )]),
            }],
            package_layout: PackageLayout {
                staging_root: "build/platform/p0/staging".to_string(),
                native_root: "build/platform/p0/staging/native".to_string(),
                include_root: "build/platform/p0/staging/include".to_string(),
                managed_root: "build/platform/p0/staging/managed".to_string(),
                metadata_root: "build/platform/p0/staging/metadata".to_string(),
                evidence_root: "build/platform/p0/staging/evidence".to_string(),
                symbols_root: "build/platform/p0/staging/symbols".to_string(),
            },
            inputs_digest: "5".repeat(64),
        }
    }

    /// Golden 字节。**任何一处不同即编码漂移，是必须升 plan_format_version 的版本事件**
    /// （ADR-0006 第 2、10 条），不是「更新一下期望值」。
    const GOLDEN: &str = concat!(
        r#"{"plan_format_version":1,"#,
        r#""architecture":{"architecture_baseline_id":"LGE-V1.4-2026-08-27","#,
        r#""architecture_source_repository":"https://example.invalid/a/b","#,
        r#""architecture_source_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","#,
        r#""lock_file":"architecture.lock.json","#,
        r#""lock_file_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},"#,
        r#""source_lock":{"repositories":[{"component":"LumioNativeCore","#,
        r#""repository":"https://example.invalid/native","checkout_root":"build/sources/native","#,
        r#""commit":"cccccccccccccccccccccccccccccccccccccccc","#,
        r#""tree_id":"dddddddddddddddddddddddddddddddddddddddd"},"#,
        r#"{"component":"LumioVoxelEngine","repository":"https://example.invalid/voxel","#,
        r#""checkout_root":"build/sources/voxel","#,
        r#""commit":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","#,
        r#""tree_id":"ffffffffffffffffffffffffffffffffffffffff"}],"#,
        r#""source_tree_digest":"1111111111111111111111111111111111111111111111111111111111111111"},"#,
        r#""feature_set":{"enabled":["中文-feature","ctrl\u0001flag"],"disabled":[]},"#,
        r#""target_profile_document":{"source_path":"fixtures/valid/target-profile-linux-server.json","#,
        r#""source_sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"#,
        r#""toolchain":{"rustc":{"tool_id":"rustc","version":"1.89.0","#,
        r#""executable_sha256":"2222222222222222222222222222222222222222222222222222222222222222"},"#,
        r#""cargo":{"tool_id":"cargo","version":"1.89.0","#,
        r#""executable_sha256":"3333333333333333333333333333333333333333333333333333333333333333"},"#,
        r#""linker":{"tool_id":"cc","version":"11.4.0","#,
        r#""executable_sha256":"4444444444444444444444444444444444444444444444444444444444444444"},"#,
        r#""target_triple":"x86_64-unknown-linux-gnu","sdk":null},"#,
        r#""build_profile":{"cargo_profile":"release","panic_strategy":"abort","lto":true,"#,
        r#""codegen_units":4294967295,"debug_symbols":false},"#,
        r#""root_abi_contract":{"abi_schema":{"source_path":"schemas/root-abi-bundle.schema.json","#,
        r#""source_sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"#,
        r#""generated_artifact_descriptor":{"source_path":"packages/abi/root-abi-bundle.json","#,
        r#""source_sha256":"0000000000000000000000000000000000000000000000000000000000000000"}},"#,
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
        r#""symbols_root":"build/platform/p0/staging/symbols"},"#,
        r#""inputs_digest":"5555555555555555555555555555555555555555555555555555555555555555"}"#,
        "\n"
    );

    #[test]
    fn encoding_matches_golden_bytes() {
        let encoded = encode_plan(&golden_plan()).expect("编码成功");
        assert_eq!(String::from_utf8(encoded).expect("UTF-8"), GOLDEN);
    }

    #[test]
    fn golden_locks_every_byte_level_rule() {
        let bytes = encode_plan(&golden_plan()).expect("编码成功");
        let text = std::str::from_utf8(&bytes).expect("UTF-8");

        assert!(!text.starts_with('\u{feff}'), "无 BOM");
        assert_eq!(text.matches('\n').count(), 1, "恰一个 LF，且在结尾");
        assert!(text.ends_with('\n'));
        assert!(!text.contains(": "), "紧凑：键值之间无空格");
        assert!(!text.contains(", "), "紧凑：元素之间无空格");
        assert!(text.contains("中文-feature"), "非 ASCII 原样输出，不转 \\u");
        assert!(text.contains(r"ctrl\u0001flag"), "控制字符转小写 \\u00XX");
        assert!(text.contains("https://example.invalid/a/b"), "不转义正斜杠");
        assert!(text.contains("\"sdk\":null"), "缺省字段发 null，键集恒定");
        assert!(!text.contains('.') || !text.contains("e+"), "无浮点");
    }

    #[test]
    fn inputs_digest_is_taken_over_the_plan_without_its_own_field() {
        let mut plan = golden_plan();
        let first = inputs_digest(&plan).expect("算摘要");
        // 改 inputs_digest 本身不影响该摘要（自排除）。
        plan.inputs_digest = "9".repeat(64);
        assert_eq!(first, inputs_digest(&plan).expect("再算"));
        // 改其他任何字段都影响。
        plan.build_profile.codegen_units = 1;
        assert_ne!(first, inputs_digest(&plan).expect("三算"));
    }

    #[test]
    fn decoding_rejects_unknown_fields() {
        let bytes = encode_plan(&golden_plan()).expect("编码成功");
        let text = String::from_utf8(bytes).expect("UTF-8");
        let tampered = text.replace(
            r#"{"plan_format_version":1,"#,
            r#"{"plan_format_version":1,"extra":1,"#,
        );
        assert!(
            serde_json::from_str::<BuildPlan>(&tampered).is_err(),
            "未知键必须被拒绝：Schema 演进只走版本号，不做前向兼容"
        );
    }
}
