//! 跨字段不变量与 ArchitectureBaselineId / TargetProfile / ABI 输入检查（规格 §7.2）。

use std::path::{Component, Path};

use serde::Deserialize;

use crate::encode::sha256_hex;
use crate::error::{err, invalid, CompositionError, CompositionErrorKind};
use crate::model::{
    ArchitectureDocumentPaths, ArchitectureDocumentRef, ArchitectureInputLock, BuildInvocation,
    FeatureSet, PackageLayout, SourceLock, ENVIRONMENT_WHITELIST,
};

pub(crate) fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

// ── architecture.lock.json（上游拥有，只读消费）──────────────────────────────

/// 只声明本 crate 需要的字段；lock 由 LCE-P0-002 拥有，字段会增长，故不 deny_unknown。
#[derive(Debug, Deserialize)]
struct ArchitectureLockDocument {
    repository: String,
    commit: String,
    #[serde(rename = "architectureBaselineId")]
    architecture_baseline_id: String,
    #[serde(rename = "requiredPathSha256")]
    required_path_sha256: std::collections::BTreeMap<String, String>,
}

fn read_lock(path: &Path) -> Result<ArchitectureLockDocument, CompositionError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| invalid(format!("读取 {} 失败：{e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| invalid(format!("解析 {} 失败：{e}", path.display())))
}

/// 供 `ComposeRequest::from_config_file` 在解析配置阶段确定镜像目录名。
pub(crate) fn read_baseline_id(path: &Path) -> Result<String, CompositionError> {
    Ok(read_lock(path)?.architecture_baseline_id)
}

/// 架构源树路径 -> 本仓只读镜像路径（与 tools/sync-architecture.sh 的投影规则同口径：
/// schemas/ ids/ fixtures/ packages/ 保持前缀，.spec/decisions/ 折叠为 decisions/）。
pub(crate) fn mirror_path(
    baseline_id: &str,
    source_path: &str,
) -> Result<String, CompositionError> {
    let projected = if let Some(rest) = source_path.strip_prefix(".spec/decisions/") {
        format!("decisions/{rest}")
    } else if source_path.starts_with("schemas/")
        || source_path.starts_with("ids/")
        || source_path.starts_with("fixtures/")
        || source_path.starts_with("packages/")
    {
        source_path.to_string()
    } else {
        return Err(err(
            CompositionErrorKind::ArchitectureLockMismatch,
            format!(
                "架构源路径 {source_path} 不可投影到本仓镜像\
                 （仅允许 schemas/ ids/ fixtures/ packages/ .spec/decisions/ 前缀）"
            ),
        ));
    };
    Ok(format!("generated/architecture/{baseline_id}/{projected}"))
}

/// 解析一份架构文档引用：摘要取自 lock 的登记值，并与本仓镜像的实际字节对账。
/// 两侧任一不符即拒绝——只信 lock 而不读镜像，等于对镜像漂移视而不见。
fn resolve_document(
    lock: &ArchitectureLockDocument,
    workspace_root: &Path,
    declaration: &ArchitectureDocumentPaths,
    missing_kind: CompositionErrorKind,
) -> Result<ArchitectureDocumentRef, CompositionError> {
    let source_path = &declaration.source_path;
    let registered = lock.required_path_sha256.get(source_path).ok_or_else(|| {
        err(
            missing_kind,
            format!("architecture.lock.json 的 requiredPathSha256 没有 {source_path}"),
        )
    })?;
    if !is_sha256_hex(registered) {
        return Err(err(
            CompositionErrorKind::ArchitectureLockMismatch,
            format!("lock 登记的 {source_path} 摘要不是 64 位小写十六进制"),
        ));
    }

    let mirror = workspace_root.join(mirror_path(&lock.architecture_baseline_id, source_path)?);
    let bytes = std::fs::read(&mirror).map_err(|e| {
        err(
            missing_kind,
            format!("读取只读镜像 {} 失败：{e}", mirror.display()),
        )
    })?;
    let actual = sha256_hex(&bytes);
    if &actual != registered {
        return Err(err(
            CompositionErrorKind::ArchitectureLockMismatch,
            format!(
                "只读镜像 {} 与 lock 登记摘要不符（镜像 {actual}，登记 {registered}）",
                mirror.display()
            ),
        ));
    }

    Ok(ArchitectureDocumentRef {
        source_path: source_path.clone(),
        source_sha256: registered.clone(),
    })
}

pub(crate) struct ArchitectureInputs {
    pub(crate) lock: ArchitectureInputLock,
    pub(crate) target_profile_document: ArchitectureDocumentRef,
    pub(crate) abi_schema: ArchitectureDocumentRef,
    pub(crate) generated_artifact_descriptor: ArchitectureDocumentRef,
    pub(crate) target_profile: TargetProfileDocument,
}

// ── TargetProfile 文档（架构源拥有）──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct TargetProfileToolchain {
    pub(crate) compiler: String,
    pub(crate) version: String,
    #[allow(dead_code)]
    pub(crate) sdk: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TargetProfileDocument {
    #[serde(rename = "targetProfileId")]
    pub(crate) target_profile_id: String,
    pub(crate) os: String,
    pub(crate) arch: String,
    #[serde(rename = "abiRuntime")]
    pub(crate) abi_runtime: String,
    pub(crate) toolchain: TargetProfileToolchain,
    #[serde(rename = "loadBackend")]
    pub(crate) load_backend: String,
}

pub(crate) fn resolve_architecture_inputs(
    workspace_root: &Path,
    architecture_lock_path: &Path,
    target_profile_document_path: &Path,
    declarations: &crate::model::PlanDeclarations,
) -> Result<ArchitectureInputs, CompositionError> {
    let lock = read_lock(architecture_lock_path)?;
    let lock_bytes = std::fs::read(architecture_lock_path).map_err(|e| {
        invalid(format!(
            "读取 {} 失败：{e}",
            architecture_lock_path.display()
        ))
    })?;

    let target_profile_document = resolve_document(
        &lock,
        workspace_root,
        &declarations.target_profile_document,
        CompositionErrorKind::TargetProfileReferenceMismatch,
    )?;
    let abi_schema = resolve_document(
        &lock,
        workspace_root,
        &declarations.root_abi_abi_schema,
        CompositionErrorKind::RootAbiContractUnavailable,
    )?;
    let generated_artifact_descriptor = resolve_document(
        &lock,
        workspace_root,
        &declarations.root_abi_generated_artifact_descriptor,
        CompositionErrorKind::RootAbiContractUnavailable,
    )?;

    // CLI 传入的 TargetProfile 路径必须就是投影出来的那份镜像，否则两条路径会各自漂移。
    let expected_mirror = workspace_root.join(mirror_path(
        &lock.architecture_baseline_id,
        &target_profile_document.source_path,
    )?);
    if !same_file(&expected_mirror, target_profile_document_path) {
        return Err(err(
            CompositionErrorKind::TargetProfileReferenceMismatch,
            format!(
                "TargetProfile 文档路径 {} 与投影得到的镜像 {} 不是同一份",
                target_profile_document_path.display(),
                expected_mirror.display()
            ),
        ));
    }
    let profile_text = std::fs::read_to_string(&expected_mirror).map_err(|e| {
        err(
            CompositionErrorKind::TargetProfileReferenceMismatch,
            format!("读取 TargetProfile {} 失败：{e}", expected_mirror.display()),
        )
    })?;
    let target_profile: TargetProfileDocument =
        serde_json::from_str(&profile_text).map_err(|e| {
            err(
                CompositionErrorKind::TargetProfileReferenceMismatch,
                format!("解析 TargetProfile {} 失败：{e}", expected_mirror.display()),
            )
        })?;
    if target_profile.load_backend.is_empty() {
        return Err(err(
            CompositionErrorKind::TargetProfileReferenceMismatch,
            "TargetProfile 缺少 loadBackend".to_string(),
        ));
    }

    Ok(ArchitectureInputs {
        lock: ArchitectureInputLock {
            architecture_baseline_id: lock.architecture_baseline_id.clone(),
            architecture_source_repository: lock.repository.clone(),
            architecture_source_commit: lock.commit.clone(),
            lock_file: to_workspace_relative(workspace_root, architecture_lock_path)?,
            lock_file_digest: sha256_hex(&lock_bytes),
        },
        target_profile_document,
        abi_schema,
        generated_artifact_descriptor,
        target_profile,
    })
}

fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => left == right,
    }
}

// ── WorkspaceRelativePath ──────────────────────────────────────────────────

/// 绝对路径 -> WorkspaceRelativePath（ADR-0006 第 4 条）。workspace 之外即拒绝。
pub(crate) fn to_workspace_relative(
    workspace_root: &Path,
    path: &Path,
) -> Result<String, CompositionError> {
    // 先按原始路径相对化：checkout 常以符号链接落在 workspace 内（操作者把同级仓库
    // 链进 build/sources/），此时原始路径就在 workspace 下，而 canonicalize 会把它解析
    // 到 workspace 之外，得出假的「不在 workspace 内」。
    // 原始路径相对化不了才回退到规范化比较——那是为了处理 workspace 根自身经符号链接
    // 给出的情形（如 macOS 的 /tmp -> /private/tmp）。
    let relative_owned;
    let relative = match path.strip_prefix(workspace_root) {
        Ok(relative) => relative,
        Err(_) => {
            let root = workspace_root
                .canonicalize()
                .unwrap_or_else(|_| workspace_root.to_path_buf());
            let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            relative_owned = target
                .strip_prefix(&root)
                .map_err(|_| {
                    invalid(format!(
                        "{} 不在 workspace {} 内，无法相对化（计划内路径必须是 WorkspaceRelativePath）",
                        path.display(),
                        workspace_root.display()
                    ))
                })?
                .to_path_buf();
            relative_owned.as_path()
        }
    };
    if relative.as_os_str().is_empty() {
        return Err(invalid(
            "路径就是 workspace 根，不能作为 WorkspaceRelativePath".to_string(),
        ));
    }

    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => {
                let text = part
                    .to_str()
                    .ok_or_else(|| invalid(format!("路径 {} 含非 UTF-8 分量", path.display())))?;
                parts.push(text);
            }
            _ => {
                return Err(invalid(format!(
                    "路径 {} 含 . / .. / 盘符等非法分量",
                    path.display()
                )))
            }
        }
    }
    Ok(parts.join("/"))
}

/// 计划内**声明式**路径（配置直接给出的字符串）的不变量检查。
pub(crate) fn check_declared_path(value: &str, what: &str) -> Result<(), CompositionError> {
    if value.is_empty() {
        return Err(invalid(format!("{what} 不得为空")));
    }
    if value.starts_with('/') || value.contains('\\') || value.contains(':') {
        return Err(invalid(format!(
            "{what} 必须是相对 workspace 根的正斜杠路径：{value}"
        )));
    }
    if value.contains("//") {
        return Err(invalid(format!("{what} 含重复分隔符：{value}")));
    }
    if value
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(invalid(format!("{what} 含空分量或 . / ..：{value}")));
    }
    if value.contains('\0') {
        return Err(invalid(format!("{what} 含 NUL")));
    }
    Ok(())
}

// ── 跨字段不变量 ───────────────────────────────────────────────────────────

pub(crate) fn check_package_layout(layout: &PackageLayout) -> Result<(), CompositionError> {
    let entries = [
        ("package_layout.staging_root", &layout.staging_root),
        ("package_layout.native_root", &layout.native_root),
        ("package_layout.include_root", &layout.include_root),
        ("package_layout.managed_root", &layout.managed_root),
        ("package_layout.metadata_root", &layout.metadata_root),
        ("package_layout.evidence_root", &layout.evidence_root),
        ("package_layout.symbols_root", &layout.symbols_root),
    ];
    for (what, value) in entries {
        check_declared_path(value, what)?;
    }
    for (what, value) in entries.iter().skip(1) {
        if !value.starts_with(&format!("{}/", layout.staging_root)) {
            return Err(invalid(format!(
                "{what} 必须位于 staging_root {} 之下：{value}",
                layout.staging_root
            )));
        }
    }
    Ok(())
}

pub(crate) fn check_build_profile(
    profile: &crate::model::BuildProfile,
) -> Result<(), CompositionError> {
    if profile.cargo_profile.is_empty() {
        return Err(invalid("build_profile.cargo_profile 不得为空".to_string()));
    }
    if !matches!(profile.panic_strategy.as_str(), "abort" | "unwind") {
        return Err(invalid(format!(
            "build_profile.panic_strategy 只能是 abort / unwind：{}",
            profile.panic_strategy
        )));
    }
    if profile.codegen_units == 0 {
        return Err(invalid(
            "build_profile.codegen_units 必须大于 0".to_string(),
        ));
    }
    Ok(())
}

/// rustflags 的键（`-Copt-level=3` -> `-Copt-level`）。同键不同值即顺序敏感冲突。
fn rustflag_key(flag: &str) -> &str {
    flag.split_once('=').map_or(flag, |(key, _)| key)
}

pub(crate) fn normalize_invocations(
    invocations: &[BuildInvocation],
    features: &FeatureSet,
    sources: &SourceLock,
) -> Result<Vec<BuildInvocation>, CompositionError> {
    if invocations.is_empty() {
        return Err(invalid("build_invocations 不得为空".to_string()));
    }

    let mut normalized = Vec::with_capacity(invocations.len());
    for invocation in invocations {
        if invocation.package.is_empty() {
            return Err(invalid("build_invocations 含空 package".to_string()));
        }
        check_declared_path(&invocation.manifest_path, "build_invocations.manifest_path")?;

        let owner = sources
            .repositories
            .iter()
            .find(|repository| repository.component == invocation.source_component)
            .ok_or_else(|| {
                invalid(format!(
                    "build_invocation 引用了 source_lock 之外的组件 {}",
                    invocation.source_component.as_str()
                ))
            })?;
        if !invocation
            .manifest_path
            .starts_with(&format!("{}/", owner.checkout_root))
        {
            return Err(invalid(format!(
                "{} 的 manifest_path {} 不在其 checkout {} 之下",
                invocation.source_component.as_str(),
                invocation.manifest_path,
                owner.checkout_root
            )));
        }

        let mut sorted_features: Vec<String> = invocation.features.clone();
        sorted_features.sort();
        sorted_features.dedup();
        for feature in &sorted_features {
            if !features.enabled.contains(feature) {
                return Err(invalid(format!(
                    "build_invocation {} 引用了未启用的 feature {feature}",
                    invocation.package
                )));
            }
        }

        let mut sorted_flags: Vec<String> = invocation.rustflags.clone();
        sorted_flags.sort();
        sorted_flags.dedup();
        for pair in sorted_flags.windows(2) {
            if rustflag_key(&pair[0]) == rustflag_key(&pair[1]) {
                return Err(invalid(format!(
                    "rustflags 同键不同值，顺序敏感冲突不得进入计划：{} 与 {}",
                    pair[0], pair[1]
                )));
            }
        }

        for (key, value) in &invocation.environment {
            if !ENVIRONMENT_WHITELIST.contains(&key.as_str()) {
                return Err(invalid(format!(
                    "环境变量 {key} 不在 V1 封闭白名单 {ENVIRONMENT_WHITELIST:?} 内"
                )));
            }
            if key == "CARGO_NET_OFFLINE" && !matches!(value.as_str(), "true" | "false") {
                return Err(invalid(format!(
                    "CARGO_NET_OFFLINE 只能是 \"true\" / \"false\"：{value}"
                )));
            }
        }

        normalized.push(BuildInvocation {
            source_component: invocation.source_component,
            manifest_path: invocation.manifest_path.clone(),
            package: invocation.package.clone(),
            target: invocation.target.clone(),
            profile: invocation.profile.clone(),
            features: sorted_features,
            no_default_features: invocation.no_default_features,
            rustflags: sorted_flags,
            environment: invocation.environment.clone(),
        });
    }

    // (source_component, package) 固定排序（ADR-0006 第 2 条）。
    normalized
        .sort_by(|a, b| (a.source_component, &a.package).cmp(&(b.source_component, &b.package)));
    for pair in normalized.windows(2) {
        if pair[0].source_component == pair[1].source_component
            && pair[0].package == pair[1].package
        {
            return Err(invalid(format!(
                "build_invocations 里 {} / {} 重复",
                pair[0].source_component.as_str(),
                pair[0].package
            )));
        }
    }
    Ok(normalized)
}
