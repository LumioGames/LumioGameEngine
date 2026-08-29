//! 锁定上游 compiler 的身份校验与调用（规格 §4「root-abi/generator 只调用锁定上游
//! compiler 并验 hash」、§8.4）。
//!
//! 本仓**不实现** cbindgen / ClangSharp / 任何模板（卡面非目标）。这里做的全部事情是：
//! 1. 复算 compiler 身份摘要，与上游 bundle 声明的 `compiler.digest` 比对；
//! 2. 让它先跑自己的 `validate_abi_document`（schema + ADR-040 语义），再产出文本。
//!
//! compiler 身份摘要的口径由上游 `compiler_hash()` 固定：
//! `sha256(lumio_contract.py 全字节 || lumio_generate.py 全字节)`。顺序与拼接方式都是
//! 摘要的一部分，改任一项都会得到另一个值。

use std::path::Path;
use std::process::Command;

use crate::error::{err, AbiGenerationError, AbiGenerationErrorKind};

/// 构成 compiler 身份的两个文件，顺序即上游 `compiler_hash()` 的拼接顺序。
const COMPILER_FILES: [&str; 2] = ["lumio_contract.py", "lumio_generate.py"];

/// 驱动脚本：加载锁定 compiler 模块，调它的三个 Root ABI emitter，把结果以 JSON 交回。
///
/// 它本身**不含任何模板、slot 表或 type map**——那些都在被加载的上游模块里。
/// 这段之所以是 Python，是因为锁定 compiler 就是 Python；换语言等于换 compiler。
const DRIVER: &str = r#"
import importlib.util, json, sys
generate_path, mirror_root = sys.argv[1], sys.argv[2]
sys.path.insert(0, str(__import__("pathlib").Path(generate_path).parent))
spec = importlib.util.spec_from_file_location("lumio_generate", generate_path)
module = importlib.util.module_from_spec(spec)
sys.modules["lumio_generate"] = module
spec.loader.exec_module(module)
from pathlib import Path
mirror = Path(mirror_root)
abi = json.loads((mirror / module.ABI_DOCUMENT).read_text(encoding="utf-8"))
# 上游 semantic validator 必须先跑：它的 docstring 就是「在写出任何一个输出字节之前
# 拒绝非法 ABI 文档」。跳过它再声称 schema/semantic 已校验，就是谎报做过的检查。
module.validate_abi_document(mirror, abi)
emitters = {
    "abi/lumio_core.h": module.emit_c_header,
    "rust/lumio-gen-language-binding/src/root_abi.rs": module.emit_rust_root_abi,
    "csharp/Lumio.Gen.LanguageBinding/RootAbi.cs": module.emit_csharp_root_abi,
}
# 输出集合以**上游** ABI_OUTPUT_FILES 为准，不以本仓写死的清单为准：
# 上游新增第 4 份输出时必须在这里响亮失败，而不是被本仓的三条清单静默忽略。
upstream_paths = [path for path, _role in module.ABI_OUTPUT_FILES]
if sorted(upstream_paths) != sorted(emitters):
    raise SystemExit(
        "upstream ABI_OUTPUT_FILES changed: {} vs adapter {}".format(
            sorted(upstream_paths), sorted(emitters)
        )
    )
json.dump(
    {
        "outputs": {path: emitters[path](abi) for path in upstream_paths},
        "abiDocument": (mirror / module.ABI_DOCUMENT).read_text(encoding="utf-8"),
        "abiDocumentPath": module.ABI_DOCUMENT,
        "entrySymbol": abi["entrySymbol"],
        "layoutProfile": module.LAYOUT_PROFILE,
        "compilerName": module.ABI_COMPILER_NAME,
        "compilerVersion": module.ABI_COMPILER_VERSION,
        "bundleId": module.ABI_BUNDLE_ID,
        "validatorRan": True,
    },
    sys.stdout,
)
"#;

/// 复算锁定 compiler 的身份摘要。
pub(crate) fn digest(compiler_directory: &Path) -> Result<String, AbiGenerationError> {
    let mut bytes = Vec::new();
    for name in COMPILER_FILES {
        let path = compiler_directory.join(name);
        let blob = std::fs::read(&path).map_err(|e| {
            err(
                AbiGenerationErrorKind::CompilerDigestMismatch,
                format!(
                    "锁定 compiler 不完整：{} 读取失败（{e}）。\
                     先跑 `LUMIO_ARCHITECTURE_REPO=<架构源仓> just fetch-architecture-tools`",
                    path.display()
                ),
            )
        })?;
        bytes.extend_from_slice(&blob);
    }
    Ok(crate::sha256_hex(&bytes))
}

/// compiler 产出的原始文本，键即上游 `ABI_OUTPUT_FILES` 的路径。
#[derive(Debug, serde::Deserialize)]
pub(crate) struct CompilerOutput {
    pub(crate) outputs: std::collections::BTreeMap<String, String>,
    #[serde(rename = "abiDocument")]
    pub(crate) abi_document: String,
    #[serde(rename = "layoutProfile")]
    pub(crate) layout_profile: serde_json::Value,
    #[serde(rename = "compilerName")]
    pub(crate) compiler_name: String,
    #[serde(rename = "compilerVersion")]
    pub(crate) compiler_version: String,
    #[serde(rename = "bundleId")]
    pub(crate) bundle_id: String,
    /// 上游 ABI 文档在架构源树内的相对路径——本仓据此把它锚回 `inputSet`。
    #[serde(rename = "abiDocumentPath")]
    pub(crate) abi_document_path: String,
    #[serde(rename = "entrySymbol")]
    pub(crate) entry_symbol: String,
    /// 上游 `validate_abi_document` 已执行的凭据。DRIVER 里它在任何 emit 之前调用，
    /// 抛异常即整个进程非零退出，所以这个字段为 true 等价于「校验通过」。
    #[serde(rename = "validatorRan")]
    pub(crate) validator_ran: bool,
}

/// 锁定 compiler 期望的仓库根布局：`tools/` 与 `schemas/` / `fixtures/` / `ids/` /
/// `packages/` 同级。`lumio_contract` 在**导入时**就按这个布局读 fixture
/// （`_ABILITY_FIXTURE`），所以只把 tools 指过去不够。
///
/// 这里用只读镜像的内容拼一个一次性的 contract root：镜像本身不动（它受 lock 约束、
/// 且已被置为只读），tools 从锁定落点复制进来。绝不使用架构源仓工作区。
const CONTRACT_ROOT_SUBDIRS: [&str; 4] = ["schemas", "fixtures", "ids", "packages"];

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

struct ScratchRoot(std::path::PathBuf);

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn build_contract_root(
    compiler_directory: &Path,
    mirror_root: &Path,
) -> Result<ScratchRoot, AbiGenerationError> {
    let nonce = {
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_usize(std::process::id() as usize);
        format!("{:016x}", hasher.finish())
    };
    let root = std::env::temp_dir().join(format!("lce-abi-contract-root-{nonce}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|e| {
        err(
            AbiGenerationErrorKind::CompilerInvocationFailed,
            format!("创建 compiler 运行根目录失败：{e}"),
        )
    })?;
    let scratch = ScratchRoot(root);

    for subdir in CONTRACT_ROOT_SUBDIRS {
        let from = mirror_root.join(subdir);
        if from.is_dir() {
            copy_tree(&from, &scratch.0.join(subdir)).map_err(|e| {
                err(
                    AbiGenerationErrorKind::CompilerInvocationFailed,
                    format!("准备 {subdir}/ 失败：{e}"),
                )
            })?;
        }
    }
    copy_tree(compiler_directory, &scratch.0.join("tools")).map_err(|e| {
        err(
            AbiGenerationErrorKind::CompilerInvocationFailed,
            format!("准备 tools/ 失败：{e}"),
        )
    })?;
    Ok(scratch)
}

/// 以只读镜像为输入运行锁定 compiler。
///
/// 身份校验必须在此之前完成——先跑再验等于已经执行了未经核对的代码。
pub(crate) fn run(
    compiler_directory: &Path,
    mirror_root: &Path,
) -> Result<CompilerOutput, AbiGenerationError> {
    let scratch = build_contract_root(compiler_directory, mirror_root)?;
    let generate = scratch.0.join("tools/lumio_generate.py");
    let output = Command::new("python3")
        .arg("-c")
        .arg(DRIVER)
        .arg(&generate)
        .arg(&scratch.0)
        .output()
        .map_err(|e| {
            err(
                AbiGenerationErrorKind::CompilerInvocationFailed,
                format!("启动 python3 运行锁定 compiler 失败：{e}"),
            )
        })?;
    if !output.status.success() {
        return Err(err(
            AbiGenerationErrorKind::CompilerInvocationFailed,
            format!(
                "锁定 compiler 返回非零（{}）：{}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| {
        err(
            AbiGenerationErrorKind::CompilerInvocationFailed,
            format!("锁定 compiler 的输出不可解析：{e}"),
        )
    })
}
