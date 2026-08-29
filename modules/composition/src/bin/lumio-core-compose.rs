//! `lumio-core-compose`——composition CLI（子命令 compose / verify / print，规格 §7.4）。
//!
//! 退出码：0 成功；2 配置；3 Source/Feature/Toolchain 漂移；4 冻结失败；
//! 5 Architecture Gate。它们是仓内工具退出码，不是公共 ErrorCode。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lumio_core_composition::{compose, verify_frozen_plan, ComposeRequest, CompositionError};

const USAGE: &str = "\
用法：
  lumio-core-compose compose --config <*.compose.toml> --out <计划目录>/build-plan.json
  lumio-core-compose verify  --plan <build-plan.json> --digest <build-plan.sha256>
  lumio-core-compose print   --plan <build-plan.json>

--workspace-root <目录> 可选，缺省取当前工作目录。计划内路径一律相对该根。";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(message)) => {
            eprintln!("lumio-core-compose: {message}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Composition(error)) => {
            eprintln!("lumio-core-compose: {error}");
            ExitCode::from(error.kind().exit_code())
        }
    }
}

enum Failure {
    Usage(String),
    Composition(CompositionError),
}

impl From<CompositionError> for Failure {
    fn from(error: CompositionError) -> Self {
        Failure::Composition(error)
    }
}

/// 取 `--name <value>`。重复给出即报错，不静默取最后一个。
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

fn known_flags(args: &[String], allowed: &[&str]) -> Result<(), Failure> {
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if !token.starts_with("--") {
            return Err(Failure::Usage(format!("多余的位置参数：{token}")));
        }
        if !allowed.contains(&token.as_str()) {
            return Err(Failure::Usage(format!("未知选项：{token}")));
        }
        index += 2;
    }
    Ok(())
}

fn run(args: &[String]) -> Result<(), Failure> {
    let (command, rest) = args
        .split_first()
        .ok_or_else(|| Failure::Usage("缺少子命令".to_string()))?;

    match command.as_str() {
        "compose" => {
            known_flags(rest, &["--config", "--out", "--workspace-root"])?;
            let config = required(rest, "--config")?;
            // `--out` 指的是计划文件本身（规格 §7.4 的命令行、justfile 与下游
            // `build-platform --plan …/build-plan.json` 一致）；发布单元是它所在的目录。
            let out = plan_directory(&required(rest, "--out")?)?;
            let workspace_root = workspace_root(rest)?;
            let request = ComposeRequest::from_config_file(&config, &workspace_root, &out)?;
            let frozen = compose(request)?;
            println!("{}", frozen.plan_digest);
            eprintln!(
                "冻结完成：{}\n  计划     {}\n  摘要     {}\n  来源记录 {}",
                out.display(),
                frozen.plan_path.display(),
                frozen.plan_digest_path.display(),
                frozen.provenance_path.display()
            );
            Ok(())
        }
        "verify" => {
            known_flags(rest, &["--plan", "--digest"])?;
            let plan = required(rest, "--plan")?;
            let digest = required(rest, "--digest")?;
            let frozen = verify_frozen_plan(&plan, &digest)?;
            println!("{}", frozen.plan_digest);
            Ok(())
        }
        "print" => {
            known_flags(rest, &["--plan", "--digest"])?;
            let plan = required(rest, "--plan")?;
            // 未显式给 sidecar 时按约定取同目录的那份：print 也必须先验后读，
            // 不提供「不校验直接看」的入口。
            let digest = option(rest, "--digest")?.unwrap_or_else(|| sibling_digest(&plan));
            let frozen = verify_frozen_plan(&plan, &digest)?;
            println!("{:#?}", frozen.plan);
            Ok(())
        }
        other => Err(Failure::Usage(format!("未知子命令：{other}"))),
    }
}

/// `--out …/build-plan.json` -> 计划目录 `…/`。
///
/// 文件名是钉死的：三份产物的名字被 ADR-0006 第 6 条与下游各 CLI 的参数固定，
/// 允许改名会让 `build-platform --plan` 找不到东西，而且要到构建期才暴露。
fn plan_directory(out: &Path) -> Result<PathBuf, Failure> {
    match out.file_name().and_then(|name| name.to_str()) {
        Some("build-plan.json") => {}
        _ => {
            return Err(Failure::Usage(format!(
                "--out 必须以 build-plan.json 结尾（发布单元是它所在的目录）：{}",
                out.display()
            )))
        }
    }
    Ok(out
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf())
}

fn sibling_digest(plan: &Path) -> PathBuf {
    plan.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("build-plan.sha256")
}

fn workspace_root(args: &[String]) -> Result<PathBuf, Failure> {
    match option(args, "--workspace-root")? {
        Some(path) => Ok(path),
        None => {
            std::env::current_dir().map_err(|e| Failure::Usage(format!("取当前工作目录失败：{e}")))
        }
    }
}
