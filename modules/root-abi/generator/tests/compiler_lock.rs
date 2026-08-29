//! 锁定 compiler 的身份校验与生成物摘要链（规格 §8.4、卡面验收项 1/2/3）。
//!
//! 本 crate 是**薄适配器**：它不含模板、slot 表、type map，只做四件事——校验锁定上游
//! compiler 的 SHA-256、以只读镜像为输入调用它、把每份产出与上游 bundle 声明的摘要
//! 逐份对账、原子只读发布。这些测试钉的就是这四件事，不钉生成内容本身（内容是上游的）。

use std::path::{Path, PathBuf};

use lumio_core_root_abi_generator::{
    generate, verify_generated, AbiGenerationErrorKind, GenerateAbiRequest,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("modules/root-abi/generator 上溯三级即仓库根")
        .to_path_buf()
}

/// 锁定 compiler 的落点：`just fetch-architecture-tools` 按 lock 的 pin 提交取到这里。
fn compiler_directory() -> PathBuf {
    let lock = repo_root().join("architecture.lock.json");
    let text = std::fs::read_to_string(&lock).expect("读 architecture.lock.json");
    let key = "\"commit\": \"";
    let start = text.find(key).expect("lock 含 commit") + key.len();
    let end = start + text[start..].find('"').expect("commit 以引号结束");
    repo_root()
        .join("build/architecture-tools")
        .join(&text[start..end])
        .join("tools")
}

fn available() -> bool {
    compiler_directory().join("lumio_generate.py").is_file()
}

/// 没取过工具链时跳过，并把原因说清楚——测试静默跳过与通过长得一样，那正是本仓
/// B-00002 要消灭的形态。
macro_rules! require_compiler {
    () => {
        if !available() {
            eprintln!(
                "SKIP: 未找到锁定 compiler（{}）。\
                 先跑 `LUMIO_ARCHITECTURE_REPO=<架构源仓> just fetch-architecture-tools`。",
                compiler_directory().display()
            );
            return;
        }
    };
}

fn request(output_directory: PathBuf) -> GenerateAbiRequest {
    GenerateAbiRequest {
        // 计划核对由 CLI 路径覆盖（just generate-abi 总是传 --plan）；
        // 这些测试钉的是 compiler 身份与摘要链，不重复造一份冻结计划。
        frozen_plan_path: None,
        architecture_lock_path: repo_root().join("architecture.lock.json"),
        mirror_root: None,
        compiler_directory: compiler_directory(),
        output_directory,
    }
}

fn temp_out(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEQ: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "lce-abi-{}-{}-{}",
        tag,
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn compiler_digest_and_input_hash_come_from_the_locked_bundle() {
    require_compiler!();
    let out = temp_out("hashes");
    let artifacts = generate(request(out.clone())).expect("生成成功");

    // 验收项 1：Compiler / Input / Output Hash 完整。三者都必须是 64 位小写十六进制，
    // 且 compiler / input 与上游 bundle 的声明值相等（相等性由实现在生成期强制，
    // 这里再读回一次，防止「算了但没用」）。
    for digest in [
        &artifacts.compiler_digest,
        &artifacts.input_hash,
        &artifacts.output_hash,
    ] {
        assert_eq!(digest.len(), 64, "{digest}");
        assert!(digest
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    }

    let report =
        verify_generated(&out, &repo_root().join("architecture.lock.json")).expect("回读校验成功");
    assert!(report.input_hash_matches);
    assert!(report.output_hash_matches);
    assert!(report.schema_valid);
    assert!(report.c_layout_valid && report.rust_layout_valid && report.csharp_layout_valid);

    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn wrong_compiler_directory_is_rejected_before_any_output_is_written() {
    require_compiler!();
    let out = temp_out("bad-compiler");
    let mut bad = request(out.clone());
    bad.compiler_directory = repo_root().join("tools"); // 本仓 tools/，不是锁定 compiler

    let error = generate(bad).expect_err("compiler 身份不符必须失败");
    assert_eq!(error.kind(), AbiGenerationErrorKind::CompilerDigestMismatch);
    assert!(
        !out.exists(),
        "compiler 校验失败时不得留下输出目录（卡面 blocked 行为：输出目录不存在）"
    );
}

#[test]
fn regenerating_into_a_fresh_directory_is_byte_identical() {
    require_compiler!();
    // 验收项 3：同输入重建零差异。
    let first = temp_out("repro-a");
    let second = temp_out("repro-b");
    let a = generate(request(first.clone())).expect("首次生成");
    let b = generate(request(second.clone())).expect("再次生成");

    assert_eq!(a.output_hash, b.output_hash);
    for (left, right) in [
        (&a.header_path, &b.header_path),
        (&a.csharp_binding_path, &b.csharp_binding_path),
        (&a.rust_contracts_path, &b.rust_contracts_path),
        (&a.abi_document_path, &b.abi_document_path),
        (&a.layout_report_path, &b.layout_report_path),
        (
            &a.generated_artifact_descriptor_path,
            &b.generated_artifact_descriptor_path,
        ),
    ] {
        assert_eq!(
            std::fs::read(left).expect("读左"),
            std::fs::read(right).expect("读右"),
            "{} 与 {} 必须逐字节相同",
            left.display(),
            right.display()
        );
    }

    let _ = std::fs::remove_dir_all(&first);
    let _ = std::fs::remove_dir_all(&second);
}

#[test]
fn publishing_over_an_existing_directory_is_refused() {
    require_compiler!();
    let out = temp_out("exists");
    generate(request(out.clone())).expect("首次生成");
    let error = generate(request(out.clone())).expect_err("已发布目录不得覆盖");
    assert_eq!(error.kind(), AbiGenerationErrorKind::OutputAlreadyExists);
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn hand_editing_any_generated_byte_makes_verification_fail() {
    require_compiler!();
    // 验收项 2：手改稳定失败。逐份产物各改一个字节，每次都必须被发现。
    let lock = repo_root().join("architecture.lock.json");
    for which in 0..6usize {
        let out = temp_out(&format!("tamper-{which}"));
        let artifacts = generate(request(out.clone())).expect("生成成功");
        let target = [
            &artifacts.header_path,
            &artifacts.csharp_binding_path,
            &artifacts.rust_contracts_path,
            &artifacts.abi_document_path,
            &artifacts.layout_report_path,
            &artifacts.generated_artifact_descriptor_path,
        ][which]
            .clone();

        // 发布是只读的，改之前先恢复写权限——这一步本身也证明了「只读发布」成立。
        let mut permissions = std::fs::metadata(&target).expect("取权限").permissions();
        assert!(permissions.readonly(), "{} 必须是只读", target.display());
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&target, permissions).expect("恢复写权限");

        let mut bytes = std::fs::read(&target).expect("读产物");
        bytes.push(b' ');
        std::fs::write(&target, &bytes).expect("写回被篡改的产物");

        let error = verify_generated(&out, &lock).expect_err("手改必须被发现");
        assert!(
            matches!(
                error.kind(),
                AbiGenerationErrorKind::OutputHashMismatch
                    | AbiGenerationErrorKind::UnregisteredFile
            ),
            "{} 被改后得到 {:?}",
            target.display(),
            error.kind()
        );
        let _ = std::fs::remove_dir_all(&out);
    }
}

#[test]
fn an_unregistered_file_in_the_output_directory_is_rejected() {
    require_compiler!();
    // 验收项 4：生成目录没有未登记文件。
    let out = temp_out("stray");
    generate(request(out.clone())).expect("生成成功");
    std::fs::write(out.join("stray.txt"), b"not generated\n").expect("塞一个未登记文件");

    let error = verify_generated(&out, &repo_root().join("architecture.lock.json"))
        .expect_err("未登记文件必须被发现");
    assert_eq!(error.kind(), AbiGenerationErrorKind::UnregisteredFile);
    let _ = std::fs::remove_dir_all(&out);
}
