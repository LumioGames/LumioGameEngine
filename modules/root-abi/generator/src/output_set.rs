//! 输出集合的登记表与 Output Hash（规格 §8.4、§3.6）。
//!
//! 本仓发布目录的形状是**本仓约定**（规格 §8.2），与上游 package 路径不同名；
//! 但每份内容的**期望摘要来自上游 bundle**，本仓不另定义。两者的对应关系集中在这里，
//! 别处不得再写第二份映射。

use std::collections::BTreeMap;

use crate::error::{err, AbiGenerationError, AbiGenerationErrorKind};
use crate::input_set::RootAbiBundle;

/// 上游产物路径 -> 本仓发布路径。
///
/// 顺序即登记顺序，也是 Output Hash 的遍历顺序。
pub(crate) const UPSTREAM_TO_LOCAL: [(&str, &str); 3] = [
    ("abi/lumio_core.h", "include/lumio_core.h"),
    (
        "rust/lumio-gen-language-binding/src/root_abi.rs",
        "rust/contracts.rs",
    ),
    (
        "csharp/Lumio.Gen.LanguageBinding/RootAbi.cs",
        "csharp/Lumio.CoreEngine.Native.g.cs",
    ),
];

/// 由本仓自己产出、不来自 compiler 文本的三份登记文件。
pub(crate) const LOCAL_ONLY: [&str; 3] = [
    "metadata/native-managed-abi.json",
    "reports/layout-report.json",
    "generated-contract-artifact.json",
];

/// 发布目录里允许存在的全部文件（登记表）。多一个即 `UnregisteredFile`。
pub(crate) fn registered_files() -> Vec<String> {
    let mut files: Vec<String> = UPSTREAM_TO_LOCAL
        .iter()
        .map(|(_, local)| (*local).to_string())
        .collect();
    files.extend(LOCAL_ONLY.iter().map(|name| (*name).to_string()));
    files.sort();
    files
}

/// 把上游 bundle 声明的摘要按**本仓路径**索引。
pub(crate) fn declared_digests(
    bundle: &RootAbiBundle,
) -> Result<BTreeMap<String, String>, AbiGenerationError> {
    let mut by_upstream: BTreeMap<&str, &str> = BTreeMap::new();
    for file in &bundle.output_files {
        by_upstream.insert(file.path.as_str(), file.digest.as_str());
    }
    let mut out = BTreeMap::new();
    for (upstream, local) in UPSTREAM_TO_LOCAL {
        let digest = by_upstream.get(upstream).ok_or_else(|| {
            err(
                AbiGenerationErrorKind::BlockedOnArchitectureGate,
                format!("上游 bundle 未声明 {upstream} 的摘要，无法对账"),
            )
        })?;
        out.insert(local.to_string(), (*digest).to_string());
    }
    Ok(out)
}

/// Output Hash：按本仓相对路径字节序遍历发布目录内**全部登记文件**，
/// 逐项 `路径 || NUL || 内容`，以单个 LF 连接后取 SHA-256。
///
/// 与 Input Hash 同一构造方式，便于人工复核；它覆盖的是「本仓发布了什么」，
/// 而不是「上游生成了什么」——后者由逐份 declared digest 对账负责。
pub(crate) fn compute_output_hash(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut parts: Vec<Vec<u8>> = Vec::with_capacity(files.len());
    for (relative, bytes) in files {
        let mut item = relative.as_bytes().to_vec();
        item.push(0);
        item.extend_from_slice(bytes);
        parts.push(item);
    }
    crate::sha256_hex(&parts.join(&b'\n'))
}
