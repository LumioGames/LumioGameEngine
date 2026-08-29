//! 同语义输入 -> 同字节计划（规格 §7.5、ADR-0006 第 2/6 条）。
//!
//! 验收项 1「同输入精确字节相同」与验收项 4「输出有 sidecar Digest」的判据在这里。

mod common;

use common::TempWorkspace;
use lumio_core_composition::{compose, verify_frozen_plan, CompositionErrorKind};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn identical_inputs_freeze_to_byte_identical_plans_in_separate_directories() {
    let ws = TempWorkspace::create("reproducible-a");

    let first = compose(ws.request("first")).expect("首次冻结成功");
    let second = compose(ws.request("second")).expect("再次冻结到另一空目录成功");

    let first_plan = std::fs::read(&first.plan_path).expect("读回首份计划");
    let second_plan = std::fs::read(&second.plan_path).expect("读回次份计划");
    assert_eq!(
        first_plan, second_plan,
        "同输入两次 compose 的 build-plan.json 必须逐字节相同"
    );

    let first_provenance = std::fs::read(&first.provenance_path).expect("读回首份来源记录");
    let second_provenance = std::fs::read(&second.provenance_path).expect("读回次份来源记录");
    assert_eq!(
        first_provenance, second_provenance,
        "provenance.json 同样不得含时间戳等非确定字段"
    );

    assert_eq!(first.plan_digest, second.plan_digest);
}

#[test]
fn permuting_set_valued_inputs_does_not_change_output_bytes() {
    let ws = TempWorkspace::create("reproducible-permuted");

    let straight = compose(ws.request("straight")).expect("原序冻结成功");

    // feature / rustflags / build_invocations 都是语义集合：置换输入顺序后
    // ADR-0006 第 2 条要求编码前排序，字节必须不变。
    let mut permuted_request = ws.request("permuted");
    permuted_request
        .declarations
        .feature_catalog
        .known
        .reverse();
    permuted_request.declarations.build_invocations.reverse();
    for invocation in &mut permuted_request.declarations.build_invocations {
        invocation.rustflags.reverse();
        invocation.features.reverse();
    }
    let permuted = compose(permuted_request).expect("置换序冻结成功");

    assert_eq!(
        std::fs::read(&straight.plan_path).expect("读回原序计划"),
        std::fs::read(&permuted.plan_path).expect("读回置换序计划"),
        "map/set/feature 输入顺序置换后计划字节必须不变"
    );
}

#[test]
fn frozen_plan_is_compact_json_with_exactly_one_trailing_newline() {
    let ws = TempWorkspace::create("reproducible-encoding");
    let frozen = compose(ws.request("encoding")).expect("冻结成功");

    let bytes = std::fs::read(&frozen.plan_path).expect("读回计划");
    assert_eq!(bytes.last(), Some(&b'\n'), "文件必须以 LF 结尾");
    let body = &bytes[..bytes.len() - 1];
    assert!(
        !body.contains(&b'\n'),
        "紧凑编码：正文内不得出现换行（无缩进、无多余空白），结尾也只有一个 LF"
    );
    assert!(
        body.starts_with(b"{\"plan_format_version\":1,"),
        "键按结构体字段声明序发出，首键是 plan_format_version 且值恒为 1"
    );
}

#[test]
fn sidecar_digest_equals_sha256_of_plan_bytes_and_verify_round_trips() {
    let ws = TempWorkspace::create("reproducible-sidecar");
    let frozen = compose(ws.request("sidecar")).expect("冻结成功");

    let plan_bytes = std::fs::read(&frozen.plan_path).expect("读回计划");
    let sidecar = std::fs::read_to_string(&frozen.plan_digest_path).expect("读回 sidecar");
    assert_eq!(
        sidecar.len(),
        65,
        "sidecar 只含 64 位十六进制 + 恰一个 LF，无文件名、无其他字节"
    );
    let hex = sidecar.strip_suffix('\n').expect("以 LF 结尾");
    assert!(hex
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    assert_eq!(hex, sha256_hex(&plan_bytes));
    assert_eq!(hex, frozen.plan_digest);

    let reread = verify_frozen_plan(&frozen.plan_path, &frozen.plan_digest_path)
        .expect("已冻结计划可被只读回读");
    assert_eq!(reread.plan_digest, frozen.plan_digest);
    assert_eq!(reread.plan.plan_format_version, 1);
}

#[test]
fn single_byte_tampering_of_plan_is_rejected_by_sidecar() {
    let ws = TempWorkspace::create("reproducible-tamper");
    let frozen = compose(ws.request("tamper")).expect("冻结成功");

    let mut bytes = std::fs::read(&frozen.plan_path).expect("读回计划");
    // 改包名里的一个字符：JSON 仍合法，只有摘要能发现。
    let needle = b"lumio-native-core";
    let position = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("计划里含 native 包名");
    bytes[position] = b'L';
    std::fs::write(&frozen.plan_path, &bytes).expect("写回被篡改的计划");

    let error = verify_frozen_plan(&frozen.plan_path, &frozen.plan_digest_path)
        .expect_err("篡改后必须失败");
    assert_eq!(error.kind(), CompositionErrorKind::NonDeterministicPlan);
}

#[test]
fn self_excluding_inputs_digest_catches_tampering_that_sidecar_was_recomputed_for() {
    let ws = TempWorkspace::create("reproducible-inputs-digest");
    let frozen = compose(ws.request("inputs-digest")).expect("冻结成功");

    let text = std::fs::read_to_string(&frozen.plan_path).expect("读回计划");
    let tampered = text.replace("\"codegen_units\":1", "\"codegen_units\":2");
    assert_ne!(text, tampered, "计划里应含 codegen_units");
    std::fs::write(&frozen.plan_path, &tampered).expect("写回被篡改的计划");
    // 连 sidecar 一起按篡改后的字节重算——攻击者能改文件就能改 sidecar。
    // 此时只剩自排除的 inputs_digest 还能发现内容被动过（ADR-0006 第 6 条）。
    std::fs::write(
        &frozen.plan_digest_path,
        format!("{}\n", sha256_hex(tampered.as_bytes())),
    )
    .expect("写回重算的 sidecar");

    let error = verify_frozen_plan(&frozen.plan_path, &frozen.plan_digest_path)
        .expect_err("inputs_digest 失配必须被发现");
    assert_eq!(error.kind(), CompositionErrorKind::NonDeterministicPlan);
}
