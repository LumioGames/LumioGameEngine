//! Source 漂移拒绝（规格 §7.5 负向 Fixture：错误 commit、错误 tree、dirty checkout）。
//!
//! 验收项 2「source/toolchain/feature 任一漂移明确失败」的 source 部分。
//! SourceInspector 只验证本地 checkout 与锁，不实现 VCS、不 clone、不联网。

mod common;

use common::TempWorkspace;
use lumio_core_composition::{compose, CompositionErrorKind};

#[test]
fn wrong_expected_commit_is_rejected() {
    let ws = TempWorkspace::create("source-commit");
    let mut request = ws.request("wrong-commit");
    request.sources[0].expected_commit = "0".repeat(40);

    let error = compose(request).expect_err("commit 漂移必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::SourceCommitMismatch);
}

#[test]
fn wrong_expected_tree_id_is_rejected() {
    let ws = TempWorkspace::create("source-tree");
    let mut request = ws.request("wrong-tree");
    request.sources[1].expected_tree_id = "1".repeat(40);

    let error = compose(request).expect_err("tree 漂移必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::SourceTreeDigestMismatch);
}

#[test]
fn dirty_checkout_is_rejected() {
    let ws = TempWorkspace::create("source-dirty");
    // 改已跟踪文件：commit 没变、tree_id 也没变，只有工作区脏了——
    // 不查这一项就会拿「已提交内容」的摘要给「实际会被编译的内容」背书。
    std::fs::write(ws.native.checkout_root.join("Cargo.toml"), "dirtied\n").expect("弄脏 checkout");

    let error = compose(ws.request("dirty")).expect_err("dirty checkout 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::DirtySourceTree);
}

#[test]
fn untracked_files_alone_do_not_count_as_dirty() {
    let ws = TempWorkspace::create("source-untracked");
    // 未跟踪文件不进 tree，也不会被 cargo 编译进产物（.DS_Store 之类），
    // 把它算作漂移会让开发机上永远 compose 不出计划。
    std::fs::write(ws.voxel.checkout_root.join("scratch.log"), "noise\n")
        .expect("放一个未跟踪文件");

    compose(ws.request("untracked")).expect("仅有未跟踪文件时仍应成功");
}

#[test]
fn missing_checkout_is_configuration_error() {
    let ws = TempWorkspace::create("source-missing");
    let mut request = ws.request("missing");
    request.sources[0].checkout_root = ws.root.join("build/sources/does-not-exist");

    let error = compose(request).expect_err("checkout 不存在必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn checkout_outside_workspace_is_configuration_error() {
    let ws = TempWorkspace::create("source-outside");
    let mut request = ws.request("outside");
    // ADR-0006 第 4 条：计划内一切路径是 WorkspaceRelativePath，无法相对化即拒绝。
    request.sources[0].checkout_root = ws.root.parent().expect("有父目录").to_path_buf();

    let error = compose(request).expect_err("workspace 外的 checkout 必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::InvalidConfiguration);
}

#[test]
fn source_lock_records_both_components_in_declaration_order() {
    let ws = TempWorkspace::create("source-order");
    let frozen = compose(ws.request("order")).expect("冻结成功");

    let repositories = &frozen.plan.source_lock.repositories;
    assert_eq!(repositories[0].commit, ws.native.commit);
    assert_eq!(repositories[1].commit, ws.voxel.commit);
    assert_eq!(repositories[0].checkout_root, ws.native.relative);
    assert_eq!(repositories[1].checkout_root, ws.voxel.relative);
    assert_eq!(
        frozen.plan.source_lock.source_tree_digest.len(),
        64,
        "source_tree_digest 是 64 位小写十六进制"
    );
}
