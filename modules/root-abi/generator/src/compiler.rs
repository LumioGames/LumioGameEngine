//! 锁定上游 compiler 的身份校验与调用（规格 §4「root-abi/generator 只调用锁定上游
//! compiler 并验 hash」、§8.4）。
//!
//! 本仓**不实现** cbindgen / ClangSharp / 任何模板（卡面非目标）。这里做的全部事情是：
//! 1. 复算 compiler 身份摘要，与上游 bundle 声明的 `compiler.digest` 比对；
//! 2. 把只读镜像喂给它，收回它产出的文本。
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
spec = importlib.util.spec_from_file_location("lumio_generate", generate_path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
from pathlib import Path
mirror = Path(mirror_root)
abi = json.loads((mirror / module.ABI_DOCUMENT).read_text(encoding="utf-8"))
emitters = {
    "abi/lumio_core.h": module.emit_c_header,
    "rust/lumio-gen-language-binding/src/root_abi.rs": module.emit_rust_root_abi,
    "csharp/Lumio.Gen.LanguageBinding/RootAbi.cs": module.emit_csharp_root_abi,
}
json.dump(
    {
        "outputs": {path: emit(abi) for path, emit in emitters.items()},
        "abiDocument": (mirror / module.ABI_DOCUMENT).read_text(encoding="utf-8"),
        "layoutProfile": module.LAYOUT_PROFILE,
        "compilerName": module.ABI_COMPILER_NAME,
        "compilerVersion": module.ABI_COMPILER_VERSION,
        "bundleId": module.ABI_BUNDLE_ID,
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
}

/// 以只读镜像为输入运行锁定 compiler。
///
/// 身份校验必须在此之前完成——先跑再验等于已经执行了未经核对的代码。
pub(crate) fn run(
    compiler_directory: &Path,
    mirror_root: &Path,
) -> Result<CompilerOutput, AbiGenerationError> {
    let generate = compiler_directory.join("lumio_generate.py");
    let output = Command::new("python3")
        .arg("-c")
        .arg(DRIVER)
        .arg(&generate)
        .arg(mirror_root)
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
