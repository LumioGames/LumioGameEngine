//! lumio-core-root-abi-generator——只调用锁定上游 compiler 产出并校验 Root ABI 制品
//! （规格 §8、§4「只消费架构源生成制品」）。
//!
//! **本 crate 是薄适配器。** 模板、slot 表、type map、布局常量全部属于上游 compiler；
//! 这里只做四件事：
//! 1. 复算锁定 compiler 的身份摘要，与上游 bundle 声明的 `compiler.digest` 比对；
//! 2. 以**只读镜像**为唯一输入运行它（绝不读架构源仓工作区）；
//! 3. 把每份产出与上游 bundle 声明的摘要逐份对账；
//! 4. 临时目录 → 全量验证 → 只读 → 原子发布（规格 §3.6）。
//!
//! 在本仓自己实现模板会制造第二个 ABI 定义处，两处定义迟早分叉——
//! `tests/no_private_schema.rs` 用源码级断言盯着这条边界。

mod compiler;
mod error;
mod input_set;
mod layout_verify;
mod output_set;
mod publish;

pub use error::{AbiGenerationError, AbiGenerationErrorKind};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use error::{err, invalid};

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let mut out = String::with_capacity(64);
    for byte in hasher.finalize() {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// 生成请求（规格 §8.3）。
///
/// 与 §8.3 的差异及理由：`compiler_path` + `compiler_digest` 合并为
/// `compiler_directory` —— compiler 身份由**两个文件**共同决定（上游 `compiler_hash()`
/// 的口径），单个路径表达不了；而期望摘要必须来自上游 bundle 而非调用方传入，
/// 调用方能传摘要就等于能绕过对账。`build_plan` 暂不接入：本卡的输入集合由上游
/// bundle 的 `inputSet` 声明，全部在只读镜像内，与 BuildPlan 无交集
/// （LCE-P0-008 消费计划时再按其卡面接入）。
#[derive(Debug, Clone)]
pub struct GenerateAbiRequest {
    /// 已冻结的 BuildPlan（`…/build-plan.json`）。
    ///
    /// 规格 §8.3 写的是 `build_plan: FrozenBuildPlan`，这里是可选路径，理由：
    /// Root ABI 的输入集合**全部**由上游 bundle 的 `inputSet` 声明且都在只读镜像内，
    /// 与 BuildPlan 无交集；计划在这里的作用是**交叉核对**——它记的 architecture
    /// 基线与提交必须与本仓 lock 一致，否则「按 A 计划构建、按 B 基线生成 ABI」
    /// 会一路无声地走到运行时。给了就强制核对（CLI 总是给），不给则跳过该核对。
    pub frozen_plan_path: Option<PathBuf>,
    pub architecture_lock_path: PathBuf,
    /// 只读镜像根；`None` 表示按 lock 的基线 id 从 workspace 推导。
    pub mirror_root: Option<PathBuf>,
    /// `just fetch-architecture-tools` 取到的锁定 compiler 目录。
    pub compiler_directory: PathBuf,
    pub output_directory: PathBuf,
}

/// 生成结果（规格 §8.3）。
#[derive(Debug, Clone)]
pub struct GeneratedAbiArtifacts {
    pub header_path: PathBuf,
    pub csharp_binding_path: PathBuf,
    pub rust_contracts_path: PathBuf,
    pub abi_document_path: PathBuf,
    pub layout_report_path: PathBuf,
    pub generated_artifact_descriptor_path: PathBuf,
    pub compiler_digest: String,
    pub input_hash: String,
    pub output_hash: String,
}

/// 回读校验结果（规格 §8.3）。
#[derive(Debug, Clone)]
pub struct AbiCompatibilityReport {
    pub abi_identity: String,
    pub schema_valid: bool,
    pub semantic_rules_valid: bool,
    pub c_layout_valid: bool,
    pub rust_layout_valid: bool,
    pub csharp_layout_valid: bool,
    pub symbols_valid: bool,
    pub input_hash_matches: bool,
    pub output_hash_matches: bool,
}

fn workspace_root_of(lock_path: &Path) -> Result<PathBuf, AbiGenerationError> {
    lock_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| invalid("architecture.lock.json 路径没有父目录".to_string()))
}

/// 组装全部待发布内容，并完成所有对账。**不写盘**——写盘只发生在 `publish` 里。
/// 待发布内容 + 三个摘要（compiler / input / output）。
struct BuiltArtifacts {
    files: BTreeMap<String, Vec<u8>>,
    compiler_digest: String,
    input_hash: String,
    output_hash: String,
}

fn build_files(request: &GenerateAbiRequest) -> Result<BuiltArtifacts, AbiGenerationError> {
    let lock = input_set::read_lock(&request.architecture_lock_path)?;
    let workspace_root = workspace_root_of(&request.architecture_lock_path)?;
    let mirror = match &request.mirror_root {
        Some(path) => path.clone(),
        None => input_set::mirror_root(&workspace_root, &lock),
    };
    let bundle = input_set::read_bundle(&mirror)?;

    // 计划与 lock 的基线/提交必须一致。计划经 composition 的只读入口取得——
    // 那是唯一合法的读法（ADR 0006 第 8 条：消费者不得自建第二套解析器）。
    if let Some(plan_path) = &request.frozen_plan_path {
        let digest_path = plan_path
            .parent()
            .ok_or_else(|| invalid("计划路径没有父目录".to_string()))?
            .join("build-plan.sha256");
        let frozen = lumio_core_composition::verify_frozen_plan(plan_path, &digest_path)
            .map_err(|e| invalid(format!("已冻结计划不可消费：{e}")))?;
        let planned = &frozen.plan.architecture;
        if planned.architecture_baseline_id != lock.architecture_baseline_id
            || planned.architecture_source_commit != lock.commit
        {
            return Err(err(
                AbiGenerationErrorKind::InputHashMismatch,
                format!(
                    "计划与 lock 的架构输入不一致：计划 {}@{}，lock {}@{}",
                    planned.architecture_baseline_id,
                    planned.architecture_source_commit,
                    lock.architecture_baseline_id,
                    lock.commit
                ),
            ));
        }
    }

    if bundle.baseline_id != lock.architecture_baseline_id {
        return Err(err(
            AbiGenerationErrorKind::InputHashMismatch,
            format!(
                "上游 bundle 基线 {} 与 lock 基线 {} 不符",
                bundle.baseline_id, lock.architecture_baseline_id
            ),
        ));
    }

    // 1. compiler 身份先于一切——先跑再验等于已经执行了未经核对的代码。
    let compiler_digest = compiler::digest(&request.compiler_directory)?;
    if compiler_digest != bundle.compiler.digest {
        return Err(err(
            AbiGenerationErrorKind::CompilerDigestMismatch,
            format!(
                "锁定 compiler 身份不符：实测 {compiler_digest}，上游 bundle 声明 {}",
                bundle.compiler.digest
            ),
        ));
    }

    // 2. 输入集合摘要：重算而不是照抄，才能证明镜像里的输入就是产生该值的那份。
    let input_hash = input_set::compute_input_hash(&mirror, &bundle)?;
    if input_hash != bundle.input_hash {
        return Err(err(
            AbiGenerationErrorKind::InputHashMismatch,
            format!(
                "输入集合摘要不符：实测 {input_hash}，上游 bundle 声明 {}",
                bundle.input_hash
            ),
        ));
    }

    // 3. 跑锁定 compiler。
    let produced = compiler::run(&request.compiler_directory, &mirror)?;
    if produced.bundle_id != bundle.bundle_id {
        return Err(err(
            AbiGenerationErrorKind::CompilerDigestMismatch,
            format!(
                "compiler 自报 bundleId {} 与 bundle 声明 {} 不符",
                produced.bundle_id, bundle.bundle_id
            ),
        ));
    }
    if produced.compiler_name != bundle.compiler.name
        || produced.compiler_version != bundle.compiler.version
    {
        return Err(err(
            AbiGenerationErrorKind::CompilerDigestMismatch,
            format!(
                "compiler 自报 {} {} 与 bundle 声明 {} {} 不符",
                produced.compiler_name,
                produced.compiler_version,
                bundle.compiler.name,
                bundle.compiler.version
            ),
        ));
    }

    // 4. layout 检查（常量全部来自上游 profile）。
    let layout = layout_verify::check(&bundle, &produced.layout_profile)?;

    // 5. 逐份产物与上游声明摘要对账。
    let declared = output_set::declared_digests(&bundle)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (upstream, local) in output_set::UPSTREAM_TO_LOCAL {
        let text = produced.outputs.get(upstream).ok_or_else(|| {
            err(
                AbiGenerationErrorKind::CompilerInvocationFailed,
                format!("锁定 compiler 未产出 {upstream}"),
            )
        })?;
        let bytes = text.as_bytes().to_vec();
        let actual = sha256_hex(&bytes);
        let expected = declared
            .get(local)
            .expect("declared_digests 已覆盖全部本仓路径");
        if &actual != expected {
            return Err(err(
                AbiGenerationErrorKind::OutputHashMismatch,
                format!("{local} 摘要不符：实测 {actual}，上游 bundle 声明 {expected}"),
            ));
        }
        files.insert(local.to_string(), bytes);
    }

    // 6. 本仓自产的三份登记文件。
    files.insert(
        "metadata/native-managed-abi.json".to_string(),
        produced.abi_document.into_bytes(),
    );
    files.insert(
        "reports/layout-report.json".to_string(),
        canonical_json(&layout.report)?,
    );

    // descriptor 最后写：它覆盖前面所有文件的摘要，自己不进自己的 outputHash。
    let output_hash = output_set::compute_output_hash(&files);
    let descriptor = build_descriptor(&bundle, &lock, &input_hash, &output_hash, &files);
    files.insert(
        "generated-contract-artifact.json".to_string(),
        canonical_json(&descriptor)?,
    );

    Ok(BuiltArtifacts {
        files,
        compiler_digest: bundle.compiler.digest.clone(),
        input_hash,
        output_hash,
    })
}

/// descriptor 的**唯一**构造处，generate 与 `verify_generated` 共用。
///
/// 共用是必须的：verify 若不能按同一规则重建 descriptor，就无从判断 descriptor 自身
/// 有没有被改——只能校验「它记的别人」，校验不了「它自己」。（首版正是这么写的，
/// 于是往 descriptor 末尾加一个空格可以完全不被发现；`hand_editing_…` 测试抓到了。）
///
/// `compiler.digest` 取 bundle 的声明值而非实测值：verify 不要求锁定 compiler 在场，
/// 而生成期已断言过两者相等。
fn build_descriptor(
    bundle: &input_set::RootAbiBundle,
    lock: &input_set::ArchitectureLock,
    input_hash: &str,
    output_hash: &str,
    files: &BTreeMap<String, Vec<u8>>,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "root-abi-generated-contract-artifact",
        "baselineId": bundle.baseline_id,
        "bundleId": bundle.bundle_id,
        "schemaEpoch": bundle.schema_epoch,
        "architectureRepository": lock.repository,
        "architectureCommit": lock.commit,
        "compiler": {
            "name": bundle.compiler.name,
            "version": bundle.compiler.version,
            "digest": bundle.compiler.digest,
        },
        "inputHash": input_hash,
        "outputHash": output_hash,
        "registeredFiles": output_set::registered_files(),
        "fileDigests": files
            .iter()
            .map(|(path, bytes)| (path.clone(), sha256_hex(bytes)))
            .collect::<BTreeMap<String, String>>(),
    })
}

/// 与 ADR 0006 同一确定性口径：紧凑、无多余空白、恰一个结尾 LF。
/// 生成物必须可复现，缩进与键序的任何随意都会让「同输入重建零差异」失效。
fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, AbiGenerationError> {
    let mut bytes =
        serde_json::to_vec(value).map_err(|e| invalid(format!("生成登记文件失败：{e}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// 生成并只读发布 Root ABI 制品。
pub fn generate(request: GenerateAbiRequest) -> Result<GeneratedAbiArtifacts, AbiGenerationError> {
    let built = build_files(&request)?;
    publish::publish(&request.output_directory, &built.files)?;

    let at = |relative: &str| request.output_directory.join(relative);
    Ok(GeneratedAbiArtifacts {
        header_path: at("include/lumio_core.h"),
        csharp_binding_path: at("csharp/Lumio.CoreEngine.Native.g.cs"),
        rust_contracts_path: at("rust/contracts.rs"),
        abi_document_path: at("metadata/native-managed-abi.json"),
        layout_report_path: at("reports/layout-report.json"),
        generated_artifact_descriptor_path: at("generated-contract-artifact.json"),
        compiler_digest: built.compiler_digest,
        input_hash: built.input_hash,
        output_hash: built.output_hash,
    })
}

/// 回读校验一份已发布的生成目录。
///
/// 判据全部来自目录内的 descriptor 与上游 bundle，不依赖生成时的内存状态——
/// 否则「手改后失败」只能在生成的同一个进程里成立。
pub fn verify_generated(
    root: &Path,
    lock_path: &Path,
) -> Result<AbiCompatibilityReport, AbiGenerationError> {
    let lock = input_set::read_lock(lock_path)?;
    let workspace_root = workspace_root_of(lock_path)?;
    let mirror = input_set::mirror_root(&workspace_root, &lock);
    let bundle = input_set::read_bundle(&mirror)?;

    let descriptor_path = root.join("generated-contract-artifact.json");
    let descriptor: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&descriptor_path)
            .map_err(|e| invalid(format!("读取 {} 失败：{e}", descriptor_path.display())))?,
    )
    .map_err(|e| invalid(format!("解析 {} 失败：{e}", descriptor_path.display())))?;

    // 目录内文件集合必须与登记表**完全一致**：多一个是未登记，少一个是缺失。
    let registered = output_set::registered_files();
    let mut present: Vec<String> = Vec::new();
    collect_files(root, root, &mut present)?;
    present.sort();
    if present != registered {
        return Err(err(
            AbiGenerationErrorKind::UnregisteredFile,
            format!("生成目录文件集合与登记表不符：实际 {present:?}，登记 {registered:?}"),
        ));
    }

    // 逐份对账：先与 descriptor 记的摘要比，再把三份 compiler 产物与上游 bundle 比。
    let recorded = descriptor
        .get("fileDigests")
        .and_then(|value| value.as_object())
        .ok_or_else(|| invalid("descriptor 缺少 fileDigests".to_string()))?;
    let declared = output_set::declared_digests(&bundle)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for relative in &registered {
        let bytes = std::fs::read(root.join(relative))
            .map_err(|e| invalid(format!("读取 {relative} 失败：{e}")))?;
        let actual = sha256_hex(&bytes);
        if relative != "generated-contract-artifact.json" {
            let expected = recorded
                .get(relative)
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    err(
                        AbiGenerationErrorKind::UnregisteredFile,
                        format!("descriptor 未登记 {relative} 的摘要"),
                    )
                })?;
            if actual != expected {
                return Err(err(
                    AbiGenerationErrorKind::OutputHashMismatch,
                    format!("{relative} 已被改动：实测 {actual}，登记 {expected}"),
                ));
            }
            if let Some(upstream) = declared.get(relative) {
                if &actual != upstream {
                    return Err(err(
                        AbiGenerationErrorKind::OutputHashMismatch,
                        format!("{relative} 与上游 bundle 声明摘要不符：{actual} != {upstream}"),
                    ));
                }
            }
            files.insert(relative.clone(), bytes);
        }
    }

    let input_hash = input_set::compute_input_hash(&mirror, &bundle)?;
    let output_hash = output_set::compute_output_hash(&files);

    // descriptor 自身也必须被校验。按同一规则重建后逐字节比对——
    // 「它记的每一条都对得上」证明不了「它自己没被改」：往末尾加一个空格，
    // 前一种检查全绿。
    let rebuilt = canonical_json(&build_descriptor(
        &bundle,
        &lock,
        &input_hash,
        &output_hash,
        &files,
    ))?;
    let actual_descriptor = std::fs::read(&descriptor_path)
        .map_err(|e| invalid(format!("读取 {} 失败：{e}", descriptor_path.display())))?;
    if rebuilt != actual_descriptor {
        return Err(err(
            AbiGenerationErrorKind::OutputHashMismatch,
            format!(
                "{} 已被改动：按同一规则重建后字节不同",
                descriptor_path.display()
            ),
        ));
    }

    let recorded_input_hash = descriptor
        .get("inputHash")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    let layout = layout_verify::check(&bundle, &bundle.layout_profile)?;
    Ok(AbiCompatibilityReport {
        abi_identity: format!("{}/{}", bundle.baseline_id, bundle.bundle_id),
        schema_valid: true,
        semantic_rules_valid: true,
        c_layout_valid: layout.c_valid,
        rust_layout_valid: layout.rust_valid,
        csharp_layout_valid: layout.csharp_valid,
        symbols_valid: true,
        input_hash_matches: input_hash == recorded_input_hash,
        output_hash_matches: true,
    })
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), AbiGenerationError> {
    for entry in
        std::fs::read_dir(dir).map_err(|e| invalid(format!("读取 {} 失败：{e}", dir.display())))?
    {
        let path = entry
            .map_err(|e| invalid(format!("读取目录项失败：{e}")))?
            .path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| invalid("生成目录内路径无法相对化".to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            out.push(relative);
        }
    }
    Ok(())
}
