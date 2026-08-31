//! rustc / cargo / linker / SDK 锁校验（规格 §7.2）。
//!
//! compose 期**不下载、不探测、不调用 cargo/rustc**：声明的每个 `ToolReference` 只与
//! 两个静态真值对账——TargetProfile 文档（版本与目标平台），以及 `tools.lock.toml`
//! 与 `checksums.sha256`（登记与摘要）。执行期在主机上复核实际二进制是 platform
//! 的职责（规格 §7.4）。

use std::path::Path;

use serde::Deserialize;

use crate::error::{err, invalid, CompositionError, CompositionErrorKind};
use crate::model::{ToolReference, ToolchainLock};
use crate::validate::{is_sha256_hex, TargetProfileDocument};

/// 工具链三项在 tools.lock 里登记在**钉定构建环境**的 host key 下（R-00265）：
/// 它们的摘要只在该环境内成立，开发机上的同名二进制是别的东西。BuildPlan 里的
/// ToolchainLock 说的正是「构建将在那个环境里执行」，所以查的也是这个键。
const BUILD_ENVIRONMENT_SUFFIX: &str = "-p0-build";

/// tools.lock 的 `supported_hosts` 键（与 tools/verify-tool-lock.sh 的映射表同口径）。
fn host_key(target_triple: &str) -> Result<&'static str, CompositionError> {
    match target_triple {
        "x86_64-unknown-linux-gnu" => Ok("linux-x86_64"),
        "aarch64-unknown-linux-gnu" => Ok("linux-arm64"),
        "x86_64-pc-windows-msvc" => Ok("windows-x86_64"),
        "aarch64-pc-windows-msvc" => Ok("windows-arm64"),
        "x86_64-apple-darwin" => Ok("darwin-x86_64"),
        "aarch64-apple-darwin" => Ok("darwin-arm64"),
        other => Err(err(
            CompositionErrorKind::TargetNotApplicable,
            format!("未登记的 target 三元组：{other}"),
        )),
    }
}

/// TargetProfile 的 (os, arch, abiRuntime) 唯一决定 target 三元组。
fn expected_triple(profile: &TargetProfileDocument) -> Result<&'static str, CompositionError> {
    match (
        profile.os.as_str(),
        profile.arch.as_str(),
        profile.abi_runtime.as_str(),
    ) {
        ("LinuxServer", "x86_64", "glibc") => Ok("x86_64-unknown-linux-gnu"),
        (os, arch, abi) => Err(err(
            CompositionErrorKind::TargetNotApplicable,
            format!(
                "TargetProfile ({os}, {arch}, {abi}) 在本仓尚无对应 target 三元组；\
                 P0 唯一 TargetProfile 是 LinuxServer/x86_64/glibc"
            ),
        )),
    }
}

fn check_shape(tool: &ToolReference, role: &str) -> Result<(), CompositionError> {
    if tool.tool_id.is_empty() || tool.version.is_empty() {
        return Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!("{role} 的 tool_id / version 不得为空"),
        ));
    }
    if !is_sha256_hex(&tool.executable_sha256) {
        return Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!(
                "{role} 的 executable_sha256 不是 64 位小写十六进制：{}",
                tool.executable_sha256
            ),
        ));
    }
    Ok(())
}

/// `tools/tools.lock.toml` 的最小消费视图。
///
/// 用 `toml` 解析而不是手写扫描：ADR 0007 §2 把「只支持子集的自研解析器」列为否决项，
/// 理由正是「一份**合法** TOML 被静默误读比读不出来更危险」——例如 `supported_hosts`
/// 写成多行数组时，逐行 `starts_with` 会命中 `supported_hosts = [` 而读不到任何 host，
/// 于是报「登记缺失」，把人指向错误的方向。
///
/// 不 `deny_unknown_fields`：这个文件由 R-00265 拥有，十个字段还会增长，
/// 本 crate 只声明自己要用的三个。
#[derive(Deserialize)]
struct ToolsLock {
    #[serde(default)]
    tools: Vec<LockedTool>,
}

#[derive(Deserialize)]
struct LockedTool {
    name: String,
    version: String,
    #[serde(default)]
    supported_hosts: Vec<String>,
}

fn parse_tools_lock(path: &Path) -> Result<ToolsLock, CompositionError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| invalid(format!("读取 {} 失败：{e}", path.display())))?;
    toml::from_str(&text).map_err(|e| invalid(format!("解析 {} 失败：{e}", path.display())))
}

fn locked_entry_exists(lock: &ToolsLock, name: &str, version: &str, host: &str) -> bool {
    lock.tools.iter().any(|tool| {
        tool.name == name
            && tool.version == version
            && tool.supported_hosts.iter().any(|entry| entry == host)
    })
}

fn registered_digest(checksums_text: &str, name: &str, host: &str) -> Option<String> {
    let key = format!("{name}@{host}");
    checksums_text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let entry = parts.next()?;
        (entry == key).then(|| digest.to_string())
    })
}

fn check_registration(
    tool: &ToolReference,
    role: &str,
    host: &str,
    lock: &ToolsLock,
    checksums_text: &str,
) -> Result<(), CompositionError> {
    if !locked_entry_exists(lock, &tool.tool_id, &tool.version, host) {
        return Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!(
                "{role}：tools.lock.toml 没有 {}@{} 版本 {} 的登记；\
                 未登记的工具不得进入 BuildPlan（规格 §3.5）",
                tool.tool_id, host, tool.version
            ),
        ));
    }
    match registered_digest(checksums_text, &tool.tool_id, host) {
        None => Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!("{role}：checksums.sha256 缺少 {}@{host} 登记", tool.tool_id),
        )),
        Some(registered) if registered != tool.executable_sha256 => Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!(
                "{role}：声明摘要与登记不符（声明 {}，登记 {registered}）",
                tool.executable_sha256
            ),
        )),
        Some(_) => Ok(()),
    }
}

pub(crate) fn validate(
    toolchain: &ToolchainLock,
    profile: &TargetProfileDocument,
    tools_lock_path: &Path,
) -> Result<(), CompositionError> {
    check_shape(&toolchain.rustc, "toolchain.rustc")?;
    check_shape(&toolchain.cargo, "toolchain.cargo")?;
    check_shape(&toolchain.linker, "toolchain.linker")?;
    if let Some(sdk) = &toolchain.sdk {
        check_shape(sdk, "toolchain.sdk")?;
    }

    let expected = expected_triple(profile)?;
    if toolchain.target_triple != expected {
        return Err(err(
            CompositionErrorKind::TargetNotApplicable,
            format!(
                "target 三元组 {} 与 TargetProfile {} 推出的 {expected} 不符",
                toolchain.target_triple, profile.target_profile_id
            ),
        ));
    }

    if toolchain.rustc.tool_id != profile.toolchain.compiler {
        return Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!(
                "编译器 {} 与 TargetProfile 声明的 {} 不符",
                toolchain.rustc.tool_id, profile.toolchain.compiler
            ),
        ));
    }
    if toolchain.rustc.version != profile.toolchain.version {
        return Err(err(
            CompositionErrorKind::ToolchainMismatch,
            format!(
                "rustc 版本漂移：TargetProfile 钉定 {}，声明 {}",
                profile.toolchain.version, toolchain.rustc.version
            ),
        ));
    }

    let build_host = format!(
        "{}{BUILD_ENVIRONMENT_SUFFIX}",
        host_key(&toolchain.target_triple)?
    );
    let host = build_host.as_str();
    let lock = parse_tools_lock(tools_lock_path)?;
    // checksums.sha256 与锁同目录，是 verify-tool-lock.sh 既有的约定布局。
    // 它不是 TOML（`<digest>  <name>@<host>` 逐行），故不在 ADR 0007 的解析器约束内。
    let checksums_path = tools_lock_path
        .parent()
        .ok_or_else(|| invalid("tools.lock.toml 路径没有父目录".to_string()))?
        .join("checksums.sha256");
    let checksums_text = std::fs::read_to_string(&checksums_path)
        .map_err(|e| invalid(format!("读取 {} 失败：{e}", checksums_path.display())))?;

    check_registration(
        &toolchain.rustc,
        "toolchain.rustc",
        host,
        &lock,
        &checksums_text,
    )?;
    check_registration(
        &toolchain.cargo,
        "toolchain.cargo",
        host,
        &lock,
        &checksums_text,
    )?;
    check_registration(
        &toolchain.linker,
        "toolchain.linker",
        host,
        &lock,
        &checksums_text,
    )?;
    if let Some(sdk) = &toolchain.sdk {
        check_registration(sdk, "toolchain.sdk", host, &lock, &checksums_text)?;
    }

    Ok(())
}
