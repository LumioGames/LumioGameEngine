//! 输入集合解析与 Input Hash（规格 §8.4、§3.6）。
//!
//! 输入**只有两个来源**：本仓 `architecture.lock.json` 与它 pin 的只读镜像。
//! 绝不读架构源仓工作区——那是不受 lock 约束的可变输入，一旦读了，「同输入重建零差异」
//! 就失去意义（`tests/no_private_schema.rs` 对此有源码级断言）。

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{err, invalid, AbiGenerationError, AbiGenerationErrorKind};

/// 只声明本 crate 需要的字段；lock 由 LCE-P0-002 拥有，字段会增长。
#[derive(Debug, Deserialize)]
pub(crate) struct ArchitectureLock {
    pub(crate) commit: String,
    #[serde(rename = "architectureBaselineId")]
    pub(crate) architecture_baseline_id: String,
    pub(crate) repository: String,
}

/// 上游 Root ABI bundle（镜像内 `packages/abi/root-abi-bundle.json`）。
///
/// 它是本卡的**声明真值**：compiler 身份、输入集合、每份产物的期望摘要都取自这里，
/// 本仓不另行定义。
#[derive(Debug, Deserialize)]
pub(crate) struct RootAbiBundle {
    #[serde(rename = "baselineId")]
    pub(crate) baseline_id: String,
    #[serde(rename = "bundleId")]
    pub(crate) bundle_id: String,
    pub(crate) compiler: BundleCompiler,
    #[serde(rename = "inputHash")]
    pub(crate) input_hash: String,
    #[serde(rename = "inputSet")]
    pub(crate) input_set: Vec<String>,
    #[serde(rename = "layoutProfile")]
    pub(crate) layout_profile: serde_json::Value,
    #[serde(rename = "outputFiles")]
    pub(crate) output_files: Vec<BundleOutputFile>,
    #[serde(rename = "schemaEpoch")]
    pub(crate) schema_epoch: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleCompiler {
    pub(crate) digest: String,
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleOutputFile {
    pub(crate) digest: String,
    pub(crate) path: String,
    pub(crate) role: String,
}

pub(crate) fn read_lock(path: &Path) -> Result<ArchitectureLock, AbiGenerationError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| invalid(format!("读取 {} 失败：{e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| invalid(format!("解析 {} 失败：{e}", path.display())))
}

/// 只读镜像根目录：`generated/architecture/<baselineId>/`。
pub(crate) fn mirror_root(workspace_root: &Path, lock: &ArchitectureLock) -> PathBuf {
    workspace_root
        .join("generated/architecture")
        .join(&lock.architecture_baseline_id)
}

pub(crate) fn read_bundle(mirror: &Path) -> Result<RootAbiBundle, AbiGenerationError> {
    let path = mirror.join("packages/abi/root-abi-bundle.json");
    // bundle 不在镜像里 = 上游还没把本仓列为 Root ABI 的 consumer = AG-001 对本仓未关闭。
    // 这时不得回退到本仓模板（卡面 blocked 行为）。
    let text = std::fs::read_to_string(&path).map_err(|e| {
        err(
            AbiGenerationErrorKind::BlockedOnArchitectureGate,
            format!(
                "上游 Root ABI bundle 不可用（{}：{e}）；AG-001 对本仓未关闭，\
                 不得回退本仓模板",
                path.display()
            ),
        )
    })?;
    serde_json::from_str(&text).map_err(|e| invalid(format!("解析 {} 失败：{e}", path.display())))
}

/// 复算 Input Hash，口径与上游 `abi_input_hash` 完全一致：
/// 按 `inputSet` 声明顺序，逐项 `路径字节 || NUL || 文件字节`，以单个 LF 连接后取 SHA-256。
///
/// 这里刻意重算而不是照抄 bundle 的值：抄下来只能证明「我读到了这个数」，
/// 重算才能证明「镜像里的输入确实就是产生这个数的那份」。
pub(crate) fn compute_input_hash(
    mirror: &Path,
    bundle: &RootAbiBundle,
) -> Result<String, AbiGenerationError> {
    let mut parts: Vec<Vec<u8>> = Vec::with_capacity(bundle.input_set.len());
    for relative in &bundle.input_set {
        let path = mirror.join(relative);
        let blob = std::fs::read(&path).map_err(|e| {
            err(
                AbiGenerationErrorKind::BlockedOnArchitectureGate,
                format!("输入 {} 不在只读镜像内：{e}", path.display()),
            )
        })?;
        let mut item = relative.as_bytes().to_vec();
        item.push(0);
        item.extend_from_slice(&blob);
        parts.push(item);
    }
    Ok(crate::sha256_hex(&parts.join(&b'\n')))
}
