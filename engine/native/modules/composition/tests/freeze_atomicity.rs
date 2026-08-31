//! 冻结协议（ADR-0006 第 7 条：temp / fsync / no-replace rename，已发布不可覆盖）。
//!
//! 规格 §7.4「失败只留下不可发现的临时目录；不得发布 build-plan.json」的判据。

mod common;

use common::TempWorkspace;
use lumio_core_composition::{compose, CompositionErrorKind};

/// 计划目录的父目录里除了已发布目录之外还剩什么——临时物泄漏检测用。
fn siblings(ws: &TempWorkspace) -> Vec<String> {
    let plans_root = ws.root.join("build/plans");
    match std::fs::read_dir(&plans_root) {
        Ok(entries) => {
            let mut names: Vec<String> = entries
                .map(|entry| {
                    entry
                        .expect("读目录项")
                        .file_name()
                        .to_string_lossy()
                        .into()
                })
                .collect();
            names.sort();
            names
        }
        Err(_) => Vec::new(),
    }
}

#[test]
fn freeze_publishes_exactly_three_files() {
    let ws = TempWorkspace::create("freeze-files");
    let frozen = compose(ws.request("published")).expect("冻结成功");

    let mut names: Vec<String> = std::fs::read_dir(ws.out_dir("published"))
        .expect("读计划目录")
        .map(|entry| {
            entry
                .expect("读目录项")
                .file_name()
                .to_string_lossy()
                .into()
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "build-plan.json".to_string(),
            "build-plan.sha256".to_string(),
            "provenance.json".to_string()
        ],
        "发布单元是整个计划目录，三文件全有或全无"
    );
    assert!(frozen.plan_path.ends_with("build-plan.json"));
    assert!(frozen.plan_digest_path.ends_with("build-plan.sha256"));
    assert!(frozen.provenance_path.ends_with("provenance.json"));
}

#[test]
fn republishing_to_an_existing_directory_is_refused_without_touching_it() {
    let ws = TempWorkspace::create("freeze-exists");
    let first = compose(ws.request("once")).expect("首次冻结成功");
    let before = std::fs::read(&first.plan_path).expect("读回首份计划");

    let error = compose(ws.request("once")).expect_err("重复冻结到已存在目录必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::OutputAlreadyExists);

    let after = std::fs::read(&first.plan_path).expect("再读回首份计划");
    assert_eq!(
        before, after,
        "已发布计划在任何情况下不得被覆盖（ADR 0001 不可变性的字节级判据）"
    );
    assert_eq!(siblings(&ws), vec!["once".to_string()]);
}

#[test]
fn a_file_occupying_the_output_path_is_also_refused() {
    let ws = TempWorkspace::create("freeze-file-collision");
    let out = ws.out_dir("collision");
    std::fs::create_dir_all(out.parent().expect("有父目录")).expect("建 plans 目录");
    std::fs::write(&out, "occupied\n").expect("用普通文件占住输出路径");

    let error = compose(ws.request("collision")).expect_err("路径被占用必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::OutputAlreadyExists);
    assert_eq!(
        std::fs::read_to_string(&out).expect("读回占位文件"),
        "occupied\n"
    );
}

#[test]
fn a_rejected_compose_leaves_no_discoverable_artifact() {
    let ws = TempWorkspace::create("freeze-failed");
    let mut request = ws.request("never-published");
    request.sources[0].expected_commit = "0".repeat(40);

    compose(request).expect_err("source 漂移必须失败");

    assert!(
        !ws.out_dir("never-published").exists(),
        "失败不得发布计划目录"
    );
    assert!(
        siblings(&ws).is_empty(),
        "失败也不得留下可发现的临时目录，实际残留：{:?}",
        siblings(&ws)
    );
}

#[test]
fn validation_failure_after_encoding_still_leaves_nothing_behind() {
    let ws = TempWorkspace::create("freeze-late-failure");
    // 先占住计划目录，让失败发生在编码与摘要都算完之后的发布阶段。
    let out = ws.out_dir("late");
    std::fs::create_dir_all(&out).expect("预建计划目录");

    compose(ws.request("late")).expect_err("目标已存在必须失败");

    assert!(
        std::fs::read_dir(&out)
            .expect("读计划目录")
            .next()
            .is_none(),
        "被拒的发布不得往已存在目录里写入任何字节"
    );
    assert_eq!(siblings(&ws), vec!["late".to_string()]);
}
