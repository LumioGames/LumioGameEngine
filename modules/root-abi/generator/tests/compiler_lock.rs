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

/// 缺工具链时**失败**，不是跳过。
///
/// 首版写的是 `eprintln! + return`——那等于测试通过，libtest 还会把提示吞掉：
/// 在没取过工具链的机器上，这 6 个测试恒为绿且恒为空，而本卡四条通过条件的全部证据
/// 都挂在它们身上。这正是本仓 B-00002（空跑输出非绿灯信号）刚修过的同型问题，
/// 宏自己的注释还写着要消灭它。
macro_rules! require_compiler {
    () => {
        assert!(
            available(),
            "未找到锁定 compiler（{}）。\
             先跑 `LUMIO_ARCHITECTURE_REPO=<架构源仓> just fetch-architecture-tools`。\
             本测试不跳过——跳过与通过在输出里长得一样。",
            compiler_directory().display()
        );
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

/// 审查实测的两条绕过路径：改一份没有独立锚点的产物，再按同一规则重建 descriptor。
/// 首版这两条都能让校验全绿——descriptor 是从同一批盘上字节重建的，被背书者与
/// 背书者同源，恒等式证明不了任何东西。
#[test]
fn rewriting_a_local_only_artifact_and_its_descriptor_entry_is_still_caught() {
    require_compiler!();
    let lock = repo_root().join("architecture.lock.json");

    for (which, relative) in [
        "metadata/native-managed-abi.json",
        "reports/layout-report.json",
    ]
    .into_iter()
    .enumerate()
    {
        let out = temp_out(&format!("collusion-{which}"));
        generate(request(out.clone())).expect("生成成功");

        // 改产物本身。
        let target = out.join(relative);
        let mut permissions = std::fs::metadata(&target).expect("取权限").permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&target, permissions).expect("恢复写权限");
        std::fs::write(&target, b"{\"tampered\":1}\n").expect("写入伪造内容");

        // 同步把 descriptor 里对应的摘要、outputHash 一并改成新值——
        // 也就是「攻击者按同一规则重建 descriptor」。
        let descriptor_path = out.join("generated-contract-artifact.json");
        let mut permissions = std::fs::metadata(&descriptor_path)
            .expect("取权限")
            .permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&descriptor_path, permissions).expect("恢复写权限");
        rebuild_descriptor_in_place(&out);

        let error =
            verify_generated(&out, &lock).expect_err(&format!("{relative} 被改后必须仍被发现"));
        assert_eq!(error.kind(), AbiGenerationErrorKind::OutputHashMismatch);
        let _ = std::fs::remove_dir_all(&out);
    }
}

/// 按发布目录里的实际字节重算 descriptor 的 fileDigests 与 outputHash 并写回，
/// 模拟「攻击者也会重建 descriptor」。
fn rebuild_descriptor_in_place(root: &Path) {
    let script = r#"
import hashlib, json, sys
from pathlib import Path
root = Path(sys.argv[1])
descriptor_path = root / "generated-contract-artifact.json"
descriptor = json.loads(descriptor_path.read_text())
files = {}
for name in descriptor["registeredFiles"]:
    if name == "generated-contract-artifact.json":
        continue
    files[name] = (root / name).read_bytes()
descriptor["fileDigests"] = {k: hashlib.sha256(v).hexdigest() for k, v in files.items()}
parts = [k.encode() + b"\x00" + v for k, v in sorted(files.items())]
descriptor["outputHash"] = hashlib.sha256(b"\n".join(parts)).hexdigest()
descriptor_path.write_bytes(json.dumps(descriptor, ensure_ascii=False, separators=(",", ":")).encode() + b"\n")
"#;
    let status = std::process::Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(root)
        .status()
        .expect("重建 descriptor");
    assert!(status.success());
}

#[test]
fn tampering_with_the_upstream_bundle_is_caught_by_the_lock() {
    require_compiler!();
    // 摘要链的根：bundle 若不与 lock 对账，改它的 outputFiles.digest 就能整体移动锚点。
    let scratch = temp_out("bad-bundle");
    let workspace = scratch.join("ws");
    let baseline = baseline_id();
    let mirror = workspace.join(format!("generated/architecture/{baseline}"));
    std::fs::create_dir_all(mirror.join("packages/abi")).expect("建镜像目录");
    std::fs::copy(
        repo_root().join("architecture.lock.json"),
        workspace.join("architecture.lock.json"),
    )
    .expect("复制 lock");
    let bundle_source = repo_root().join(format!(
        "generated/architecture/{baseline}/packages/abi/root-abi-bundle.json"
    ));
    let mut bundle = std::fs::read_to_string(&bundle_source).expect("读 bundle");
    bundle.push(' '); // 一个字节即可
    std::fs::write(mirror.join("packages/abi/root-abi-bundle.json"), bundle)
        .expect("写伪造 bundle");

    let error = generate(GenerateAbiRequest {
        frozen_plan_path: None,
        architecture_lock_path: workspace.join("architecture.lock.json"),
        mirror_root: Some(mirror),
        compiler_directory: compiler_directory(),
        output_directory: scratch.join("out"),
    })
    .expect_err("bundle 与 lock 不符必须失败");
    assert_eq!(error.kind(), AbiGenerationErrorKind::InputHashMismatch);
    assert!(!scratch.join("out").exists(), "失败不得留下输出目录");
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn a_compiler_with_correct_files_but_altered_bytes_is_rejected() {
    require_compiler!();
    // 走的是摘要比较分支，而不是「文件不存在」分支。
    let scratch = temp_out("drifted-compiler");
    let fake = scratch.join("tools");
    std::fs::create_dir_all(&fake).expect("建目录");
    for name in ["lumio_contract.py", "lumio_generate.py"] {
        let mut bytes = std::fs::read(compiler_directory().join(name)).expect("读 compiler");
        if name == "lumio_generate.py" {
            bytes.push(b' ');
        }
        std::fs::write(fake.join(name), bytes).expect("写漂移后的 compiler");
    }

    let mut request = request(scratch.join("out"));
    request.compiler_directory = fake;
    let error = generate(request).expect_err("compiler 字节漂移必须失败");
    assert_eq!(error.kind(), AbiGenerationErrorKind::CompilerDigestMismatch);
    assert!(!scratch.join("out").exists());
    let _ = std::fs::remove_dir_all(&scratch);
}

fn baseline_id() -> String {
    let lock =
        std::fs::read_to_string(repo_root().join("architecture.lock.json")).expect("读 lock");
    let key = "\"architectureBaselineId\": \"";
    let start = lock.find(key).expect("有基线 id") + key.len();
    let end = start + lock[start..].find('"').expect("引号结束");
    lock[start..end].to_string()
}

/// descriptor 里**任何**字段被替换都必须失败，包括那些回读期无法从外部重建的。
///
/// 首次修 P1-4 时，`validatorRan` 与 `entrySymbol` 在重建时是从 descriptor 自己取回去的
/// ——逐字节比对对它们恒真，改这两个字段 `verify-generated` 直接 exit 0。这是 ADR 0009
/// 第 1 节判据（被背书者与背书者必须不同源）的违例，也正好落在
/// `hand_editing_…`（往文件**追加**一个空格）的形状之外：追加会被抓到，**替换**不会。
#[test]
fn replacing_a_self_reported_descriptor_field_is_caught() {
    require_compiler!();
    let lock = repo_root().join("architecture.lock.json");

    for (which, (key, value)) in [
        ("validatorRan", serde_json::Value::Bool(false)),
        (
            "entrySymbol",
            serde_json::Value::String("lumio".to_string()),
        ),
        (
            "entrySymbol",
            serde_json::Value::String("definitely_not_in_header".to_string()),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let out = temp_out(&format!("self-reported-{which}"));
        generate(request(out.clone())).expect("生成成功");
        let descriptor_path = out.join("generated-contract-artifact.json");

        let mut permissions = std::fs::metadata(&descriptor_path)
            .expect("取权限")
            .permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&descriptor_path, permissions).expect("恢复写权限");

        let mut descriptor: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&descriptor_path).expect("读 descriptor"))
                .expect("解析 descriptor");
        descriptor[key] = value.clone();
        let mut bytes = serde_json::to_vec(&descriptor).expect("重新序列化");
        bytes.push(b'\n');
        std::fs::write(&descriptor_path, &bytes).expect("写回");

        let error = verify_generated(&out, &lock)
            .unwrap_err_or_panic(&format!("{key} 被替换为 {value} 后必须失败"));
        assert_eq!(error.kind(), AbiGenerationErrorKind::OutputHashMismatch);
        let _ = std::fs::remove_dir_all(&out);
    }
}

/// `expect_err` 需要 Ok 侧实现 Debug；`AbiCompatibilityReport` 有，但这里想带上自定义
/// 消息说明是哪一组输入，故自己写一个。
trait UnwrapErrOrPanic<T, E> {
    fn unwrap_err_or_panic(self, message: &str) -> E;
}

impl<T, E> UnwrapErrOrPanic<T, E> for Result<T, E> {
    fn unwrap_err_or_panic(self, message: &str) -> E {
        match self {
            Ok(_) => panic!("{message}"),
            Err(error) => error,
        }
    }
}
