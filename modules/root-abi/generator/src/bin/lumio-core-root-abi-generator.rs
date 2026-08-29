//! `lumio-core-root-abi-generator`——Root ABI 生成 CLI（规格 §8.4）。
//!
//! 退出码：0 成功；2 配置；3 身份/摘要漂移；4 发布失败；5 Architecture Gate。
//! 仓内工具退出码，不是公共 ErrorCode。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lumio_core_root_abi_generator::{generate, verify_generated, GenerateAbiRequest};

const USAGE: &str = "\
用法：
  lumio-core-root-abi-generator generate --plan <build-plan.json> \\
      --architecture-lock <architecture.lock.json> --out <生成目录> \\
      [--compiler-dir <锁定 compiler 目录>]
  lumio-core-root-abi-generator verify-generated --generated <生成目录> \\
      --architecture-lock <lock>

--compiler-dir 缺省取 build/architecture-tools/<lock.commit>/tools，
即 `just fetch-architecture-tools` 的落点。";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(message)) => {
            eprintln!("lumio-core-root-abi-generator: {message}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Generation(error)) => {
            eprintln!("lumio-core-root-abi-generator: {error}");
            ExitCode::from(error.kind().exit_code())
        }
    }
}

enum Failure {
    Usage(String),
    Generation(lumio_core_root_abi_generator::AbiGenerationError),
}

impl From<lumio_core_root_abi_generator::AbiGenerationError> for Failure {
    fn from(error: lumio_core_root_abi_generator::AbiGenerationError) -> Self {
        Failure::Generation(error)
    }
}

fn option(args: &[String], name: &str) -> Result<Option<PathBuf>, Failure> {
    let mut found = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            let value = args
                .get(index + 1)
                .ok_or_else(|| Failure::Usage(format!("{name} 缺少取值")))?;
            if found.is_some() {
                return Err(Failure::Usage(format!("{name} 重复给出")));
            }
            found = Some(PathBuf::from(value));
            index += 2;
        } else {
            index += 1;
        }
    }
    Ok(found)
}

fn required(args: &[String], name: &str) -> Result<PathBuf, Failure> {
    option(args, name)?.ok_or_else(|| Failure::Usage(format!("缺少 {name}")))
}

/// 锁定 compiler 的默认落点由 lock 的 commit 决定——写死 commit 会让工具链与 lock 脱钩。
fn default_compiler_directory(lock_path: &Path) -> Result<PathBuf, Failure> {
    let text = std::fs::read_to_string(lock_path)
        .map_err(|e| Failure::Usage(format!("读取 {} 失败：{e}", lock_path.display())))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| Failure::Usage(format!("解析 {} 失败：{e}", lock_path.display())))?;
    let commit = value
        .get("commit")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Failure::Usage("lock 缺少 commit".to_string()))?;
    let workspace_root = lock_path
        .parent()
        .ok_or_else(|| Failure::Usage("lock 路径没有父目录".to_string()))?;
    Ok(workspace_root
        .join("build/architecture-tools")
        .join(commit)
        .join("tools"))
}

fn run(args: &[String]) -> Result<(), Failure> {
    let (command, rest) = args
        .split_first()
        .ok_or_else(|| Failure::Usage("缺少子命令".to_string()))?;

    match command.as_str() {
        "generate" => {
            let lock = required(rest, "--architecture-lock")?;
            let out = required(rest, "--out")?;
            let compiler_directory = match option(rest, "--compiler-dir")? {
                Some(path) => path,
                None => default_compiler_directory(&lock)?,
            };
            let artifacts = generate(GenerateAbiRequest {
                frozen_plan_path: option(rest, "--plan")?,
                architecture_lock_path: lock,
                mirror_root: None,
                compiler_directory,
                output_directory: out.clone(),
            })?;
            println!("{}", artifacts.output_hash);
            eprintln!(
                "生成完成：{}\n  compilerDigest {}\n  inputHash      {}\n  outputHash     {}",
                out.display(),
                artifacts.compiler_digest,
                artifacts.input_hash,
                artifacts.output_hash
            );
            Ok(())
        }
        // 名字取规格 §8.4 的 `verify-generated`；`--generated` 同理。
        "verify-generated" => {
            let root = required(rest, "--generated")?;
            let lock = required(rest, "--architecture-lock")?;
            let report = verify_generated(&root, &lock)?;
            println!("{}", report.abi_identity);
            eprintln!(
                "校验通过：input_hash_matches={} output_hash_matches={} layout(c/rust/csharp)={}/{}/{}",
                report.input_hash_matches,
                report.output_hash_matches,
                report.c_layout_valid,
                report.rust_layout_valid,
                report.csharp_layout_valid
            );
            Ok(())
        }
        other => Err(Failure::Usage(format!("未知子命令：{other}"))),
    }
}
