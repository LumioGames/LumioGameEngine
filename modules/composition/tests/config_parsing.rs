//! `ComposeRequest::from_config_file` 的解析口径（规格 §7.2 的本地 Fixture）。
//!
//! 这层此前只被 CLI 间接覆盖：其余集成测试都在代码里直接构造 `ComposeRequest`，
//! 于是「配置读错」这类故障没有任何测试能发现。ADR 0007 §2 把「合法 TOML 被静默
//! 误读」列为选定解析器的首要理由，那条理由需要有测试兜着。

mod common;

use std::path::Path;

use common::TempWorkspace;
use lumio_core_composition::{ComposeRequest, CompositionErrorKind, SourceComponent};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/local")
        .join(name)
}

/// 夹具只提供一个可写的 workspace 根与真实的 architecture lock（解析期要读它取基线 id）。
fn workspace() -> TempWorkspace {
    TempWorkspace::create("config-parsing")
}

#[test]
fn fixture_config_parses_into_a_complete_request() {
    let ws = workspace();
    let request = ComposeRequest::from_config_file(
        &fixture("p0-compose.toml"),
        &ws.root,
        &ws.out_dir("parsed"),
    )
    .expect("本地 Fixture 必须能被解析");

    assert_eq!(
        request.sources[0].component,
        SourceComponent::LumioNativeCore
    );
    assert_eq!(
        request.sources[1].component,
        SourceComponent::LumioVoxelEngine
    );
    assert_eq!(
        request.sources[0].checkout_root,
        ws.root.join("build/sources/lumio-native-core"),
        "配置里的相对路径必须相对调用方给的 workspace 根解析"
    );
    assert_eq!(
        request.requested_features.iter().collect::<Vec<_>>(),
        vec!["experimental-codec"]
    );

    // 多行数组必须被正确读出——手写逐行扫描会在这里读到空集合，
    // 然后报「登记缺失」把人指向错误方向（ADR 0007 §2）。
    assert_eq!(
        request.declarations.feature_catalog.known,
        vec![
            "experimental-codec".to_string(),
            "experimental-diagnostics".to_string()
        ]
    );
    assert_eq!(request.declarations.feature_catalog.conflicts.len(), 1);

    // TargetProfile 的本仓镜像路径由基线 id 投影得出，不在配置里重复声明。
    let baseline = common::baseline_id();
    assert_eq!(
        request.target_profile_document_path,
        ws.root.join(format!(
            "generated/architecture/{baseline}/fixtures/valid/target-profile-linux-server.json"
        ))
    );

    assert_eq!(request.declarations.toolchain.rustc.version, "1.89.0");
    assert!(request.declarations.toolchain.sdk.is_none());
    assert_eq!(request.declarations.build_invocations.len(), 1);
    assert_eq!(request.declarations.build_profile.codegen_units, 1);
}

#[test]
fn unknown_key_in_config_is_rejected() {
    let ws = workspace();
    let text = std::fs::read_to_string(fixture("p0-compose.toml")).expect("读夹具");
    let path = ws.root.join("unknown-key.compose.toml");
    // 配置里多一个键就是配置写错了；静默忽略会让人以为改生效了。
    std::fs::write(
        &path,
        text.replace("[features]", "typo_key = 1\n\n[features]"),
    )
    .expect("写");

    let error = ComposeRequest::from_config_file(&path, &ws.root, &ws.out_dir("unknown"))
        .expect_err("未知键必须被拒绝");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn config_declaring_another_plan_format_version_is_rejected() {
    let ws = workspace();
    let text = std::fs::read_to_string(fixture("p0-compose.toml")).expect("读夹具");
    let path = ws.root.join("v2.compose.toml");
    std::fs::write(
        &path,
        text.replace("plan_format_version = 1", "plan_format_version = 2"),
    )
    .expect("写");

    // ADR-0006 第 3 条：版本比对先于一切其他解析，≠1 一律整体拒绝。
    let error = ComposeRequest::from_config_file(&path, &ws.root, &ws.out_dir("v2"))
        .expect_err("非 1 的 plan_format_version 必须被拒绝");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn duplicate_requested_feature_in_config_is_rejected() {
    let ws = workspace();
    let text = std::fs::read_to_string(fixture("p0-compose.toml")).expect("读夹具");
    let path = ws.root.join("dup.compose.toml");
    std::fs::write(
        &path,
        text.replace(
            r#"requested = ["experimental-codec"]"#,
            r#"requested = ["experimental-codec", "experimental-codec"]"#,
        ),
    )
    .expect("写");

    // BTreeSet 会静默吞掉重复；配置里写重复是笔误，应当报出来。
    let error = ComposeRequest::from_config_file(&path, &ws.root, &ws.out_dir("dup"))
        .expect_err("重复 requested feature 必须被拒绝");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn config_with_only_one_source_is_rejected() {
    let ws = workspace();
    let text = std::fs::read_to_string(fixture("p0-compose.toml")).expect("读夹具");
    let path = ws.root.join("one-source.compose.toml");
    let cut = text
        .find("[[sources]]")
        .and_then(|first| text[first + 1..].find("[[sources]]").map(|n| first + 1 + n))
        .expect("夹具有两个 sources");
    let second_end = text.find("[features]").expect("夹具有 features 段");
    std::fs::write(&path, format!("{}{}", &text[..cut], &text[second_end..])).expect("写");

    let error = ComposeRequest::from_config_file(&path, &ws.root, &ws.out_dir("one"))
        .expect_err("sources 必须恰有两项");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}
