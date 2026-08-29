//! BuildPlan、SourceLock、FeatureSet、ToolchainLock、BuildInvocation 与 compose 输入模型
//! （规格 §7.3）。
//!
//! 字段**声明序即规范编码键序**（ADR-0006 第 2 条）：调整任何结构体的字段顺序都会改变
//! 已发布计划的字节，属于必须升 `plan_format_version` 的变更。
//!
//! 摘要类型用仓内名 `Sha256Digest`：规格 §7.3 在字段类型上写作 `Digest256`，但该公共
//! 类型尚未经 `lumio-core-contracts` 发布（其 lib.rs 明确列为 seam 并禁止建同名临时
//! 类型）。上游发布后按独立需求卡替换，届时是纯类型替换，不动字节。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{invalid, CompositionError};

/// 64 位小写十六进制 SHA-256。仓内类型，**不是**公共 `Digest256`。
pub type Sha256Digest = String;

/// 完整 Git object id；V1 内部编码固定为 40 位小写十六进制 SHA-1
/// （升级 object format 须经 ADR-0006 迁移 `plan_format_version`）。
pub type GitObjectId = String;

/// UTF-8、正斜杠、无 `.`/`..`、相对 workspace root（ADR-0006 第 4 条）。
pub type WorkspaceRelativePath = String;

pub const PLAN_FORMAT_VERSION: u32 = 1;

/// `BuildInvocation.environment` 的封闭白名单（ADR-0006 第 5 条，V1 只有一项）。
pub const ENVIRONMENT_WHITELIST: [&str; 1] = ["CARGO_NET_OFFLINE"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceComponent {
    LumioNativeCore,
    LumioVoxelEngine,
}

impl SourceComponent {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceComponent::LumioNativeCore => "LumioNativeCore",
            SourceComponent::LumioVoxelEngine => "LumioVoxelEngine",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputLock {
    pub architecture_baseline_id: String,
    pub architecture_source_repository: String,
    pub architecture_source_commit: GitObjectId,
    pub lock_file: WorkspaceRelativePath,
    pub lock_file_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRepository {
    pub component: SourceComponent,
    pub repository: String,
    pub checkout_root: WorkspaceRelativePath,
    pub commit: GitObjectId,
    pub tree_id: GitObjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLock {
    pub repositories: [SourceRepository; 2],
    pub source_tree_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureSet {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolReference {
    pub tool_id: String,
    pub version: String,
    pub executable_sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainLock {
    pub rustc: ToolReference,
    pub cargo: ToolReference,
    pub linker: ToolReference,
    pub target_triple: String,
    /// SDK 不是可执行工具时为 null：P0 的 `sdk = glibc-2.35` 是 TargetProfile 上的
    /// 平台约束，由 platform 执行期按 minimumOs 校验，本仓不为它编造可执行摘要。
    pub sdk: Option<ToolReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildProfile {
    pub cargo_profile: String,
    pub panic_strategy: String,
    pub lto: bool,
    pub codegen_units: u32,
    pub debug_symbols: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInvocation {
    pub source_component: SourceComponent,
    pub manifest_path: WorkspaceRelativePath,
    pub package: String,
    pub target: String,
    pub profile: String,
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub rustflags: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDocumentRef {
    /// 相对**架构源提交树**的路径（由 `architecture_source_commit` 锁定），
    /// 不是本 workspace 路径，两者不得混用（ADR-0006 第 4 条）。
    pub source_path: String,
    pub source_sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootAbiContractRef {
    pub abi_schema: ArchitectureDocumentRef,
    pub generated_artifact_descriptor: ArchitectureDocumentRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageLayout {
    pub staging_root: WorkspaceRelativePath,
    pub native_root: WorkspaceRelativePath,
    pub include_root: WorkspaceRelativePath,
    pub managed_root: WorkspaceRelativePath,
    pub metadata_root: WorkspaceRelativePath,
    pub evidence_root: WorkspaceRelativePath,
    pub symbols_root: WorkspaceRelativePath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildPlan {
    pub plan_format_version: u32,
    pub architecture: ArchitectureInputLock,
    pub source_lock: SourceLock,
    pub feature_set: FeatureSet,
    pub target_profile_document: ArchitectureDocumentRef,
    pub toolchain: ToolchainLock,
    pub build_profile: BuildProfile,
    pub root_abi_contract: RootAbiContractRef,
    pub build_invocations: Vec<BuildInvocation>,
    pub package_layout: PackageLayout,
    pub inputs_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceRecord {
    pub architecture_baseline_id: String,
    pub architecture_source_commit: GitObjectId,
    pub source_tree_ids: [GitObjectId; 2],
    pub source_tree_digest: Sha256Digest,
    pub build_recipe_digest: Sha256Digest,
    pub build_plan_digest: Sha256Digest,
}

// ── compose 输入 ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCheckoutRequest {
    pub component: SourceComponent,
    pub repository: String,
    pub expected_commit: GitObjectId,
    pub checkout_root: PathBuf,
    pub expected_tree_id: GitObjectId,
}

/// 架构源文档在**源提交树**里的路径。本仓镜像路径由基线 id 投影得出，不重复声明，
/// 避免两处路径各自漂移。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDocumentPaths {
    pub source_path: String,
}

/// Feature 全集与互斥声明。compose 不猜 feature：没在 `known` 里的一律未知。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureCatalog {
    pub known: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<[String; 2]>,
}

/// 规格 §7.3 的 `ComposeRequest` 只列了输入路径与请求 feature，但 BuildPlan 还含
/// toolchain / build_profile / build_invocations / package_layout / 文档引用。
/// compose 不探测环境、不调用 cargo/rustc（卡面硬要求），这些只能来自配置声明；
/// 本结构就是那份声明集，由 `*.compose.toml` 解析而来。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDeclarations {
    pub feature_catalog: FeatureCatalog,
    pub toolchain: ToolchainLock,
    pub build_profile: BuildProfile,
    pub build_invocations: Vec<BuildInvocation>,
    pub package_layout: PackageLayout,
    pub target_profile_document: ArchitectureDocumentPaths,
    pub root_abi_abi_schema: ArchitectureDocumentPaths,
    pub root_abi_generated_artifact_descriptor: ArchitectureDocumentPaths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeRequest {
    pub workspace_root: PathBuf,
    pub architecture_lock_path: PathBuf,
    pub sources: [SourceCheckoutRequest; 2],
    pub requested_features: BTreeSet<String>,
    pub target_profile_document_path: PathBuf,
    pub tools_lock_path: PathBuf,
    pub output_plan_path: PathBuf,
    pub declarations: PlanDeclarations,
}

/// 冻结成功的唯一凭据。消费者只能拿到它，不存在接收可变 `BuildPlan` 的执行 API
/// （规格 §7.5 完成条件、ADR-0006 第 8 条）。
#[derive(Debug, Clone)]
pub struct FrozenBuildPlan {
    pub plan: std::sync::Arc<BuildPlan>,
    pub plan_path: PathBuf,
    pub plan_digest_path: PathBuf,
    pub plan_digest: Sha256Digest,
    pub provenance_path: PathBuf,
}

// ── 配置文件（本地 Fixture，明确非公共 Schema，规格 §7.2）────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComposeConfig {
    plan_format_version: u32,
    architecture_lock: String,
    tools_lock: String,
    sources: Vec<ConfigSource>,
    features: ConfigFeatures,
    documents: ConfigDocuments,
    toolchain: ToolchainLock,
    build_profile: BuildProfile,
    build_invocations: Vec<BuildInvocation>,
    package_layout: PackageLayout,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigSource {
    component: SourceComponent,
    repository: String,
    checkout_root: String,
    expected_commit: String,
    expected_tree_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFeatures {
    known: Vec<String>,
    requested: Vec<String>,
    #[serde(default)]
    conflicts: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigDocuments {
    target_profile: String,
    root_abi_schema: String,
    root_abi_generated_artifact_descriptor: String,
}

impl ComposeRequest {
    /// 从 `*.compose.toml` 读入一份 compose 输入。
    ///
    /// `workspace_root` 由调用方显式传入（计划不自带 workspace 根，ADR-0006 第 4 条）；
    /// 配置里的路径一律相对该根解析。TargetProfile 文档的**本仓镜像路径**由
    /// architecture lock 的基线 id 投影得出，配置只声明源树路径。
    pub fn from_config_file(
        config_path: &Path,
        workspace_root: &Path,
        output_plan_path: &Path,
    ) -> Result<Self, CompositionError> {
        let text = std::fs::read_to_string(config_path).map_err(|e| {
            invalid(format!(
                "读取 compose 配置失败 {}：{e}",
                config_path.display()
            ))
        })?;
        let config: ComposeConfig = toml::from_str(&text).map_err(|e| {
            invalid(format!(
                "解析 compose 配置失败 {}：{e}",
                config_path.display()
            ))
        })?;

        if config.plan_format_version != PLAN_FORMAT_VERSION {
            return Err(invalid(format!(
                "配置声明 plan_format_version={}，当前唯一合法值是 {PLAN_FORMAT_VERSION}",
                config.plan_format_version
            )));
        }
        if config.sources.len() != 2 {
            return Err(invalid(format!(
                "sources 必须恰有 2 项（NativeCore + VoxelEngine），实际 {}",
                config.sources.len()
            )));
        }
        let mut requested = BTreeSet::new();
        for feature in &config.features.requested {
            if !requested.insert(feature.clone()) {
                return Err(invalid(format!("features.requested 重复声明 {feature}")));
            }
        }

        let architecture_lock_path = workspace_root.join(&config.architecture_lock);
        let baseline = crate::validate::read_baseline_id(&architecture_lock_path)?;
        let target_profile_document_path = workspace_root.join(crate::validate::mirror_path(
            &baseline,
            &config.documents.target_profile,
        )?);

        let mut sources = config.sources.into_iter();
        let build = |source: ConfigSource| SourceCheckoutRequest {
            component: source.component,
            repository: source.repository,
            expected_commit: source.expected_commit,
            checkout_root: workspace_root.join(&source.checkout_root),
            expected_tree_id: source.expected_tree_id,
        };
        let first = build(sources.next().expect("已校验有 2 项"));
        let second = build(sources.next().expect("已校验有 2 项"));

        Ok(ComposeRequest {
            workspace_root: workspace_root.to_path_buf(),
            architecture_lock_path,
            sources: [first, second],
            requested_features: requested,
            target_profile_document_path,
            tools_lock_path: workspace_root.join(&config.tools_lock),
            output_plan_path: output_plan_path.to_path_buf(),
            declarations: PlanDeclarations {
                feature_catalog: FeatureCatalog {
                    known: config.features.known,
                    conflicts: config.features.conflicts,
                },
                toolchain: config.toolchain,
                build_profile: config.build_profile,
                build_invocations: config.build_invocations,
                package_layout: config.package_layout,
                target_profile_document: ArchitectureDocumentPaths {
                    source_path: config.documents.target_profile,
                },
                root_abi_abi_schema: ArchitectureDocumentPaths {
                    source_path: config.documents.root_abi_schema,
                },
                root_abi_generated_artifact_descriptor: ArchitectureDocumentPaths {
                    source_path: config.documents.root_abi_generated_artifact_descriptor,
                },
            },
        })
    }
}
