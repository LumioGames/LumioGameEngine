//! `test-support` 的可用性与隔离（规格 §9.3、卡面验收项 4）。
//!
//! 该 feature 只供 runtime-verifier 的 **dev-dependency** 构造架构 Fixture bytes，
//! 默认关闭，且必须由 runtime dependency gate 证明未进入发布闭包。本文件从**能用**
//! 与**不该被谁用**两侧各钉一半：
//! - 本测试自身以 `--features test-support` 运行，证明 builder 确实可用且产出可读集合；
//! - 「不进 normal 依赖」由 justfile 的 `runtime-deps` recipe 在仓级断言（已接入
//!   `just check`）——单个测试进程看不到别的 crate 的 feature 解析结果，在这里写
//!   `cargo tree` 等于把门禁伪装成单测。

#![cfg(feature = "test-support")]

use std::sync::Arc;

use lumio_core_platform_contracts::{
    ControlFileKind, OpenedArtifactSetFixtureBuilder, PackagePath, TestFixtureBuildError,
};

fn control_bytes(marker: &str) -> Arc<[u8]> {
    Arc::from(marker.as_bytes())
}

fn full_builder() -> OpenedArtifactSetFixtureBuilder {
    OpenedArtifactSetFixtureBuilder::new()
        .control(ControlFileKind::ManifestBody, control_bytes("manifest"))
        .control(ControlFileKind::ArtifactIndex, control_bytes("index"))
        .control(
            ControlFileKind::SignatureEnvelope,
            control_bytes("signature"),
        )
}

#[test]
fn builder_produces_a_readable_immutable_set() {
    let native = PackagePath::parse("native/liblumio_core.so").expect("合法路径");
    let opened = full_builder()
        .artifact(native.clone(), Arc::from(&b"\x7fELF-ish"[..]))
        .build()
        .expect("三个控制文件齐备即可构造");

    let manifest = opened.control(ControlFileKind::ManifestBody);
    assert_eq!(manifest.len(), "manifest".len() as u64);

    let mut buffer = [0u8; 4];
    let read = manifest.read_at(4, &mut buffer).expect("read_at 成功");
    assert_eq!(&buffer[..read], b"fest", "read_at 按偏移读，不共享游标");

    // 同一 offset 重复读必须得到同样的字节：没有可变游标才谈得上并发安全。
    let mut again = [0u8; 4];
    let read_again = manifest.read_at(4, &mut again).expect("再读");
    assert_eq!(&again[..read_again], b"fest");

    assert_eq!(opened.artifact_paths().len(), 1);
    assert!(opened.artifact(&native).is_some());
    let absent = PackagePath::parse("native/absent.so").expect("合法路径");
    assert!(opened.artifact(&absent).is_none());
}

#[test]
fn read_at_past_the_end_yields_zero_bytes() {
    let opened = full_builder().build().expect("构造成功");
    let index = opened.control(ControlFileKind::ArtifactIndex);
    let mut buffer = [0u8; 8];
    assert_eq!(index.read_at(index.len(), &mut buffer).expect("越界读"), 0);
    assert_eq!(
        index
            .read_at(index.len() + 1024, &mut buffer)
            .expect("远越界读"),
        0
    );
}

#[test]
fn missing_control_file_is_refused() {
    // `control()` 返回 `&dyn ArtifactBytes` 而非 Option——三个控制文件齐备是类型层
    // 不变量，缺一个就不该造得出集合，否则不变量只能靠调用方自觉。
    let error = OpenedArtifactSetFixtureBuilder::new()
        .control(ControlFileKind::ManifestBody, control_bytes("manifest"))
        .build()
        .expect_err("缺控制文件必须失败");
    assert!(matches!(
        error,
        TestFixtureBuildError::MissingControlFile(ControlFileKind::ArtifactIndex)
            | TestFixtureBuildError::MissingControlFile(ControlFileKind::SignatureEnvelope)
    ));
}

#[test]
fn duplicate_artifact_path_is_refused() {
    let path = PackagePath::parse("evidence/sbom.cdx.json").expect("合法路径");
    let error = full_builder()
        .artifact(path.clone(), Arc::from(&b"a"[..]))
        .artifact(path.clone(), Arc::from(&b"b"[..]))
        .build()
        .expect_err("同一路径登记两次必须失败");
    assert_eq!(error, TestFixtureBuildError::DuplicateArtifact(path));
}

#[test]
fn fixture_identities_are_distinct_per_entry() {
    let first = PackagePath::parse("evidence/a.json").expect("合法");
    let second = PackagePath::parse("evidence/b.json").expect("合法");
    let opened = full_builder()
        .artifact(first.clone(), Arc::from(&b"a"[..]))
        .artifact(second.clone(), Arc::from(&b"b"[..]))
        .build()
        .expect("构造成功");

    // platform_identity 是「同一对象」判据的载体；Fixture 也必须让不同条目可区分，
    // 否则用它写的 same-object 测试会假通过。
    let a = opened.artifact(&first).expect("有").platform_identity();
    let b = opened.artifact(&second).expect("有").platform_identity();
    assert_ne!(a, b);
    assert_eq!(a, opened.artifact(&first).expect("有").platform_identity());
}
