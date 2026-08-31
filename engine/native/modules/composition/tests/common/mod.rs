//! 集成测试共用的临时 workspace 夹具（LCE-P0-004）。
//!
//! 四个测试文件都需要同一套「可写的最小 workspace」：真实 `architecture.lock.json`
//! 与只读镜像文件（保证摘要口径与仓库一致）+ 两个一次性 git checkout。夹具建在
//! 进程私有临时目录里，不触碰仓库工作区，也不依赖网络。

// 本文件会被**每个**测试二进制各编译一份，而每份只用其中一部分，没用到的项在别的
// 二进制里就成了 dead_code。这是共享夹具的固有形态，不是真的死代码。
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use lumio_core_composition::{
    ArchitectureDocumentPaths, BuildInvocation, BuildProfile, ComposeRequest, FeatureCatalog,
    PackageLayout, PlanDeclarations, SourceCheckoutRequest, SourceComponent, ToolReference,
    ToolchainLock,
};

/// 仓库根（tests 相对 `modules/composition`，上溯两级）。
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("modules/composition 上溯两级即仓库根")
        .to_path_buf()
}

/// 本仓当前 architecture lock 所 pin 的基线目录名，直接从 lock 读，避免测试写死。
pub fn baseline_id() -> String {
    let lock = std::fs::read_to_string(repo_root().join("architecture.lock.json"))
        .expect("读取 architecture.lock.json");
    let key = "\"architectureBaselineId\": \"";
    let start = lock.find(key).expect("lock 含 architectureBaselineId") + key.len();
    let end = start + lock[start..].find('"').expect("基线值以引号结束");
    lock[start..end].to_string()
}

/// 一次性 workspace 夹具。`TempWorkspace` 析构即递归删除。
pub struct TempWorkspace {
    pub root: PathBuf,
    pub native: SourceFixture,
    pub voxel: SourceFixture,
}

pub struct SourceFixture {
    pub checkout_root: PathBuf,
    pub relative: String,
    pub commit: String,
    pub tree_id: String,
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn unique_dir(tag: &str) -> PathBuf {
    // 不用随机数：进程 id + 单调计数即可保证同一次 nextest 运行内唯一。
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEQ: AtomicU32 = AtomicU32::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "lce-composition-{}-{}-{}",
        tag,
        std::process::id(),
        seq
    ))
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} 启动失败：{e}"));
    assert!(
        out.status.success(),
        "git {args:?} 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("git 输出是 UTF-8")
}

fn init_source(root: &Path, relative: &str, marker: &str) -> SourceFixture {
    let checkout_root = root.join(relative);
    std::fs::create_dir_all(&checkout_root).expect("建 checkout 目录");
    std::fs::write(checkout_root.join("Cargo.toml"), marker).expect("写 checkout 内容");
    git(&checkout_root, &["init", "--quiet"]);
    git(&checkout_root, &["add", "."]);
    git(
        &checkout_root,
        &[
            "-c",
            "user.name=lce-test",
            "-c",
            "user.email=lce-test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
    let commit = git(&checkout_root, &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    let tree_id = git(&checkout_root, &["rev-parse", "HEAD^{tree}"])
        .trim()
        .to_string();
    SourceFixture {
        checkout_root,
        relative: relative.to_string(),
        commit,
        tree_id,
    }
}

fn copy_from_repo(repo: &Path, dest_root: &Path, relative: &str) {
    let src = repo.join(relative);
    let dest = dest_root.join(relative);
    std::fs::create_dir_all(dest.parent().expect("有父目录")).expect("建镜像目录");
    std::fs::copy(&src, &dest).unwrap_or_else(|e| panic!("复制 {relative} 失败：{e}"));
}

impl TempWorkspace {
    /// 建一个含真实 lock、真实镜像文件与两个 git checkout 的 workspace。
    pub fn create(tag: &str) -> Self {
        let root = unique_dir(tag);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("建临时 workspace");

        let repo = repo_root();
        let baseline = baseline_id();
        copy_from_repo(&repo, &root, "architecture.lock.json");
        for relative in mirror_documents(&baseline) {
            copy_from_repo(&repo, &root, &relative);
        }
        copy_from_repo(&repo, &root, "tools/tools.lock.toml");
        copy_from_repo(&repo, &root, "tools/checksums.sha256");

        let native = init_source(&root, "build/sources/lumio-native-core", "native fixture\n");
        let voxel = init_source(&root, "build/sources/lumio-voxel-engine", "voxel fixture\n");
        TempWorkspace {
            root,
            native,
            voxel,
        }
    }

    pub fn out_dir(&self, name: &str) -> PathBuf {
        self.root.join("build/plans").join(name)
    }

    /// 一份全字段合法的 ComposeRequest；各测试按需改单个字段制造漂移。
    pub fn request(&self, out_name: &str) -> ComposeRequest {
        let baseline = baseline_id();
        ComposeRequest {
            workspace_root: self.root.clone(),
            architecture_lock_path: self.root.join("architecture.lock.json"),
            sources: [
                SourceCheckoutRequest {
                    component: SourceComponent::LumioNativeCore,
                    repository: "https://github.com/LumioGames/LumioNativeCore".to_string(),
                    expected_commit: self.native.commit.clone(),
                    checkout_root: self.native.checkout_root.clone(),
                    expected_tree_id: self.native.tree_id.clone(),
                },
                SourceCheckoutRequest {
                    component: SourceComponent::LumioVoxelEngine,
                    repository: "https://github.com/LumioGames/LumioVoxelEngine".to_string(),
                    expected_commit: self.voxel.commit.clone(),
                    checkout_root: self.voxel.checkout_root.clone(),
                    expected_tree_id: self.voxel.tree_id.clone(),
                },
            ],
            requested_features: BTreeSet::from([
                "voxel-streaming".to_string(),
                "server-authority".to_string(),
            ]),
            target_profile_document_path: self.root.join(format!(
                "generated/architecture/{baseline}/fixtures/valid/target-profile-linux-server.json"
            )),
            tools_lock_path: self.root.join("tools/tools.lock.toml"),
            output_plan_path: self.out_dir(out_name),
            declarations: declarations(&self.native.relative, &self.voxel.relative),
        }
    }
}

/// 夹具引用的三个只读镜像文档（相对仓库根）。
pub fn mirror_documents(baseline: &str) -> Vec<String> {
    vec![
        format!(
            "generated/architecture/{baseline}/fixtures/valid/target-profile-linux-server.json"
        ),
        format!("generated/architecture/{baseline}/schemas/root-abi-bundle.schema.json"),
        format!("generated/architecture/{baseline}/packages/abi/root-abi-bundle.json"),
    ]
}

/// 与 `config/p0/linux-server-x86_64-glibc.compose.toml` 同构的声明集，
/// 但工具摘要用测试专用占位（测试只验证格式与漂移语义，不冒充真实工具制品）。
pub fn declarations(native_relative: &str, voxel_relative: &str) -> PlanDeclarations {
    PlanDeclarations {
        feature_catalog: FeatureCatalog {
            known: vec![
                "server-authority".to_string(),
                "voxel-streaming".to_string(),
                "voxel-persistence".to_string(),
                "client-prediction".to_string(),
            ],
            conflicts: vec![[
                "client-prediction".to_string(),
                "server-authority".to_string(),
            ]],
        },
        // 摘要不是编的：取自 tools/checksums.sha256 的 linux-x86_64-p0-build 登记
        // （R-00265 在 CI 目标环境实测）。测试夹具与真实 P0 配置消费同一份真值。
        toolchain: ToolchainLock {
            rustc: tool(
                "rustc",
                "1.89.0",
                "c6ac0142c05a60b0f6c15e1a118bd5ac3bd04924cd6dc0c328f501c5f86e4142",
            ),
            cargo: tool(
                "cargo",
                "1.89.0",
                "5b32bd53b8a08d8e206daed51f3fce40cdc0729038ff96bd0ca845596a2c1019",
            ),
            linker: tool(
                "cc",
                "11.4.0",
                "821af3c74506283c179ca413bb33e6b528805a4dd8a5c09df125e5ad560a9e89",
            ),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            // TargetProfile 的 sdk=glibc-2.35 是平台约束，不是可执行工具，不为它编造摘要。
            sdk: None,
        },
        build_profile: BuildProfile {
            cargo_profile: "release".to_string(),
            panic_strategy: "abort".to_string(),
            lto: true,
            codegen_units: 1,
            debug_symbols: true,
        },
        build_invocations: vec![
            invocation(
                SourceComponent::LumioNativeCore,
                &format!("{native_relative}/Cargo.toml"),
                "lumio-native-core",
            ),
            invocation(
                SourceComponent::LumioVoxelEngine,
                &format!("{voxel_relative}/Cargo.toml"),
                "lumio-voxel-engine",
            ),
        ],
        package_layout: PackageLayout {
            staging_root: "build/platform/linux-server-x86_64-glibc/staging".to_string(),
            native_root: "build/platform/linux-server-x86_64-glibc/staging/native".to_string(),
            include_root: "build/platform/linux-server-x86_64-glibc/staging/include".to_string(),
            managed_root: "build/platform/linux-server-x86_64-glibc/staging/managed".to_string(),
            metadata_root: "build/platform/linux-server-x86_64-glibc/staging/metadata".to_string(),
            evidence_root: "build/platform/linux-server-x86_64-glibc/staging/evidence".to_string(),
            symbols_root: "build/platform/linux-server-x86_64-glibc/staging/symbols".to_string(),
        },
        target_profile_document: ArchitectureDocumentPaths {
            source_path: "fixtures/valid/target-profile-linux-server.json".to_string(),
        },
        root_abi_abi_schema: ArchitectureDocumentPaths {
            source_path: "schemas/root-abi-bundle.schema.json".to_string(),
        },
        root_abi_generated_artifact_descriptor: ArchitectureDocumentPaths {
            source_path: "packages/abi/root-abi-bundle.json".to_string(),
        },
    }
}

fn tool(tool_id: &str, version: &str, executable_sha256: &str) -> ToolReference {
    ToolReference {
        tool_id: tool_id.to_string(),
        version: version.to_string(),
        executable_sha256: executable_sha256.to_string(),
    }
}

fn invocation(
    source_component: SourceComponent,
    manifest_path: &str,
    package: &str,
) -> BuildInvocation {
    BuildInvocation {
        source_component,
        manifest_path: manifest_path.to_string(),
        package: package.to_string(),
        target: "x86_64-unknown-linux-gnu".to_string(),
        profile: "release".to_string(),
        features: vec![
            "server-authority".to_string(),
            "voxel-streaming".to_string(),
        ],
        no_default_features: true,
        // 单 token 写法：ADR-0006 第 2 条要求 rustflags 入计划前按字节序排序，
        // `-C` 与其值分成两个元素会被排序拆散。
        rustflags: vec![
            "-Csymbol-mangling-version=v0".to_string(),
            "-Cforce-frame-pointers=yes".to_string(),
        ],
        environment: BTreeMap::from([("CARGO_NET_OFFLINE".to_string(), "true".to_string())]),
    }
}
