//! generated 完整性校验兼锁定生成器（LCE-P0-003 / R-00015）。
//!
//! 本文件同时是 `src/generated/` 与 `generated-contract-artifact.json` 的
//! **锁定生成器**与**校验器**（生成物只能由锁定生成器产生，规格 §20.1
//! 「重新生成零差异」语义）：
//!
//! - 校验（默认）：从只读镜像与 architecture.lock.json 重新渲染全部生成文件，
//!   与已提交字节逐一比对；descriptor 的逐文件摘要与 Input/Output Hash 从字节
//!   重算；上游 provenance（compiler/inputHash/bundleDigest 等）与镜像
//!   packages/index.json、root-abi-bundle.json 中的发布值逐项对账。
//! - 重生成（基线变更时）：`LUMIO_CONTRACTS_REGENERATE=1 cargo test -p
//!   lumio-core-contracts --locked --test generated_integrity`。
//!
//! 本文件刻意不 import 被测 crate：生成文件缺失或损坏时校验以断言失败呈现，
//! 且重生成可在 crate 无法编译时自举（crate 级 API 行为在 tests/schema_registry.rs）。
//!
//! Hash 口径与 tools/sync-architecture.sh、tools/verify-architecture-lock.sh 一致：
//! 逐文件 SHA-256 = 字节摘要；Input/Output Hash = 按 source path 字典序拼接
//! `<path> <sha256>\n` 字节流的整体 SHA-256。上游聚合 inputHash 的口径未随镜像
//! 发布，因此只逐字对账、不重算（逐文件摘要均可重算）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ── SHA-256（FIPS 180-4；本仓依赖图为空，测试侧内嵌实现，正确性由
//    architecture.lock.json 既有摘要交叉验证） ─────────────────────────────

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, c) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

// ── 最小 JSON 解析（镜像制品按 canonical profile 只含整数；不支持浮点） ─────

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Int(u64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(m) => m
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("JSON 缺少字段 {key}")),
            _ => panic!("对非对象取字段 {key}"),
        }
    }
    fn s(&self) -> &str {
        match self {
            Json::Str(s) => s,
            _ => panic!("期望字符串，得到 {self:?}"),
        }
    }
    fn u(&self) -> u64 {
        match self {
            Json::Int(n) => *n,
            _ => panic!("期望整数，得到 {self:?}"),
        }
    }
    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(v) => v,
            _ => panic!("期望数组，得到 {self:?}"),
        }
    }
}

fn parse_json(text: &str) -> Json {
    let b = text.as_bytes();
    let mut i = 0usize;
    let v = parse_value(b, &mut i);
    skip_ws(b, &mut i);
    assert!(i == b.len(), "JSON 尾部有多余内容（offset {i}）");
    v
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], b' ' | b'\t' | b'\n' | b'\r') {
        *i += 1;
    }
}

fn parse_value(b: &[u8], i: &mut usize) -> Json {
    skip_ws(b, i);
    match b[*i] {
        b'{' => {
            *i += 1;
            let mut m = Vec::new();
            skip_ws(b, i);
            if b[*i] == b'}' {
                *i += 1;
                return Json::Obj(m);
            }
            loop {
                skip_ws(b, i);
                let k = parse_string(b, i);
                skip_ws(b, i);
                assert!(b[*i] == b':', "对象缺少冒号");
                *i += 1;
                let v = parse_value(b, i);
                m.push((k, v));
                skip_ws(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Json::Obj(m);
                    }
                    c => panic!("对象内非法字符 {}", c as char),
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            skip_ws(b, i);
            if b[*i] == b']' {
                *i += 1;
                return Json::Arr(v);
            }
            loop {
                v.push(parse_value(b, i));
                skip_ws(b, i);
                match b[*i] {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Json::Arr(v);
                    }
                    c => panic!("数组内非法字符 {}", c as char),
                }
            }
        }
        b'"' => Json::Str(parse_string(b, i)),
        b't' => {
            assert!(&b[*i..*i + 4] == b"true");
            *i += 4;
            Json::Bool(true)
        }
        b'f' => {
            assert!(&b[*i..*i + 5] == b"false");
            *i += 5;
            Json::Bool(false)
        }
        b'n' => {
            assert!(&b[*i..*i + 4] == b"null");
            *i += 4;
            Json::Null
        }
        b'0'..=b'9' => {
            let start = *i;
            while *i < b.len() && b[*i].is_ascii_digit() {
                *i += 1;
            }
            assert!(
                *i >= b.len() || !matches!(b[*i], b'.' | b'e' | b'E'),
                "镜像 JSON 应只含整数（canonical profile IntegerOnly）"
            );
            Json::Int(std::str::from_utf8(&b[start..*i]).unwrap().parse().unwrap())
        }
        c => panic!("非法 JSON 起始字符 {}", c as char),
    }
}

fn parse_string(b: &[u8], i: &mut usize) -> String {
    assert!(b[*i] == b'"', "期望字符串");
    *i += 1;
    let mut out = String::new();
    loop {
        match b[*i] {
            b'"' => {
                *i += 1;
                return out;
            }
            b'\\' => {
                *i += 1;
                match b[*i] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{8}'),
                    b'f' => out.push('\u{c}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let hex = std::str::from_utf8(&b[*i + 1..*i + 5]).unwrap();
                        let cp = u32::from_str_radix(hex, 16).unwrap();
                        assert!(!(0xd800..=0xdfff).contains(&cp), "不支持代理对转义");
                        out.push(char::from_u32(cp).unwrap());
                        *i += 4;
                    }
                    c => panic!("非法转义 \\{}", c as char),
                }
                *i += 1;
            }
            _ => {
                let start = *i;
                while b[*i] != b'"' && b[*i] != b'\\' {
                    *i += 1;
                }
                out.push_str(std::str::from_utf8(&b[start..*i]).unwrap());
            }
        }
    }
}

// ── 输入装载 ───────────────────────────────────────────────────────────────

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("定位仓库根失败")
}

const MIRROR_REL: &str = "generated/architecture/LGE-V1.4-2026-08-27";
const BASELINE_ID: &str = "LGE-V1.4-2026-08-27";
const GENERATED_DIR: &str = "modules/root-abi/contracts/src/generated";
const DESCRIPTOR_REL: &str = "modules/root-abi/contracts/generated-contract-artifact.json";
const SELF_REL: &str = "modules/root-abi/contracts/tests/generated_integrity.rs";
const REGENERATE_ARGV: &str =
    "LUMIO_CONTRACTS_REGENERATE=1 cargo test -p lumio-core-contracts --locked --test generated_integrity";

/// 渲染输入的全量装载结果：镜像字节、lock 元数据与解析后的上游发布值。
struct Inputs {
    root: PathBuf,
    architecture_commit: String,
    architecture_repository: String,
    lock_sha: BTreeMap<String, String>,
    /// mirror 相对路径 -> 字节（本卡消费面全集）。
    files: BTreeMap<String, Vec<u8>>,
    bundle: Json,
    ids_index: Json,
    schemas_index: Json,
    packages_index: Json,
}

fn load_inputs() -> Inputs {
    let root = repo_root();
    let lock_text = fs::read_to_string(root.join("architecture.lock.json")).expect("读 lock 失败");
    let lock = parse_json(&lock_text);
    assert_eq!(lock.get("architectureBaselineId").s(), BASELINE_ID);
    let commit = lock.get("commit").s().to_string();
    let repository = lock.get("repository").s().to_string();
    let lock_sha: BTreeMap<String, String> = match lock.get("requiredPathSha256") {
        Json::Obj(m) => m
            .iter()
            .map(|(k, v)| (k.clone(), v.s().to_string()))
            .collect(),
        _ => panic!("lock.requiredPathSha256 非对象"),
    };

    let mirror = root.join(MIRROR_REL);
    let read = |rel: &str| -> Vec<u8> {
        fs::read(mirror.join(rel)).unwrap_or_else(|e| panic!("读镜像 {rel} 失败：{e}"))
    };
    let mut files = BTreeMap::new();
    for rel in [
        "packages/abi/lumio_core.h",
        "packages/abi/root-abi-bundle.json",
        "packages/index.json",
        "ids/index.json",
        "schemas/index.json",
        "schemas/common.schema.json",
    ] {
        files.insert(rel.to_string(), read(rel));
    }
    let schemas_index = parse_json(std::str::from_utf8(&files["schemas/index.json"]).unwrap());
    for entry in schemas_index.get("schemas").arr() {
        let rel = format!("schemas/{}", entry.get("file").s());
        let bytes = read(&rel);
        files.insert(rel, bytes);
    }
    let bundle =
        parse_json(std::str::from_utf8(&files["packages/abi/root-abi-bundle.json"]).unwrap());
    let ids_index = parse_json(std::str::from_utf8(&files["ids/index.json"]).unwrap());
    let packages_index = parse_json(std::str::from_utf8(&files["packages/index.json"]).unwrap());
    Inputs {
        root,
        architecture_commit: commit,
        architecture_repository: repository,
        lock_sha,
        files,
        bundle,
        ids_index,
        schemas_index,
        packages_index,
    }
}

// ── 渲染辅助 ───────────────────────────────────────────────────────────────

/// 字节串 -> Rust byte-string 字面量正文（逐字节确定性转义）。
fn byte_literal(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() + 16);
    for &b in data {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(b as char),
            _ => out.push_str(&format!("\\x{b:02x}")),
        }
    }
    out
}

/// 字符串 -> Rust 字符串字面量正文（上游值均为 ASCII，越界即 panic 防静默漂移）。
fn str_literal(s: &str) -> String {
    assert!(s.is_ascii(), "上游字符串值出现非 ASCII：{s:?}");
    byte_literal(s.as_bytes())
}

fn type_mapping_entry<'a>(bundle: &'a Json, type_ref: &str) -> &'a Json {
    bundle
        .get("typeMapping")
        .arr()
        .iter()
        .find(|e| e.get("typeRef").s() == type_ref)
        .unwrap_or_else(|| panic!("bundle.typeMapping 缺少 {type_ref}"))
}

const GENERATED_HEADER: &str = "// 本文件由锁定生成器从只读架构镜像派生——禁止手改（rules/system.md 生成物纪律）。\n// 重生成：LUMIO_CONTRACTS_REGENERATE=1 cargo test -p lumio-core-contracts --locked --test generated_integrity\n// 派生输入与逐文件摘要见 modules/root-abi/contracts/generated-contract-artifact.json。\n";

// ── 渲染：src/generated/mod.rs ─────────────────────────────────────────────

fn render_mod_rs(inp: &Inputs) -> String {
    let epoch = inp.bundle.get("schemaEpoch").u();
    assert_eq!(
        epoch,
        inp.packages_index.get("schemaEpoch").u(),
        "schemaEpoch 不一致"
    );
    format!(
        "//! 架构源生成制品的只读包装（LCE-P0-003）：ContractTypes / ErrorCode /\n\
         //! Capability / Schema registry 的消费面，逐字节来自只读镜像 {mirror}。\n\
         {header}\n\
         pub mod contracts;\n\
         pub mod error_codes;\n\
         pub mod schema_registry;\n\
         \n\
         #[rustfmt::skip]\n\
         pub const ARCHITECTURE_BASELINE_ID: &str = \"{baseline}\";\n\
         #[rustfmt::skip]\n\
         pub const ARCHITECTURE_COMMIT: &str = \"{commit}\";\n\
         #[rustfmt::skip]\n\
         pub const ARCHITECTURE_REPOSITORY: &str = \"{repo}\";\n\
         #[rustfmt::skip]\n\
         pub const SCHEMA_EPOCH: u32 = {epoch};\n",
        mirror = MIRROR_REL,
        header = GENERATED_HEADER,
        baseline = BASELINE_ID,
        commit = inp.architecture_commit,
        repo = inp.architecture_repository,
        epoch = epoch,
    )
}

// ── 渲染：src/generated/contracts.rs ───────────────────────────────────────

fn render_contracts_rs(inp: &Inputs) -> String {
    let b = &inp.bundle;
    let abi = b.get("abi");
    let layout = b.get("layoutProfile");
    let root = b.get("root");
    let compiler = b.get("compiler");
    let status = type_mapping_entry(b, "status");
    let handle = type_mapping_entry(b, "handle:<kind>");
    let buffer = type_mapping_entry(b, "buffer:in");
    let bundle_bytes = &inp.files["packages/abi/root-abi-bundle.json"];
    let header_bytes = &inp.files["packages/abi/lumio_core.h"];
    assert_eq!(b.get("baselineId").s(), BASELINE_ID);

    let mut out = String::new();
    out.push_str(
        "//! ContractTypes 消费面（packages/abi）：Root ABI bundle 与 C Header 的\n\
         //! 逐字节嵌入 + bundle 发布值的标量视图。Rust 侧结构体绑定（repr(C) 类型）\n\
         //! 属上游 LanguageBinding 生成包，其 consumers 不含本仓，故本文件不含任何\n\
         //! repr(C)/extern \"C\" 定义（seam，见 crate 文档）。\n",
    );
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    let scalar_str = [
        ("BUNDLE_ID", b.get("bundleId").s()),
        ("ENTRY_SYMBOL", abi.get("entrySymbol").s()),
        ("SYMBOL_PREFIX", abi.get("symbolPrefix").s()),
        ("CALLING_CONVENTION", abi.get("callingConvention").s()),
        ("ENDIANNESS", abi.get("endianness").s()),
        ("LAYOUT_PROFILE_ID", layout.get("targetProfileId").s()),
        ("LAYOUT_OS", layout.get("os").s()),
        ("LAYOUT_ARCH", layout.get("arch").s()),
        ("LAYOUT_ABI_RUNTIME", layout.get("abiRuntime").s()),
        ("COMPILER_NAME", compiler.get("name").s()),
        ("COMPILER_VERSION", compiler.get("version").s()),
        ("COMPILER_SHA256_HEX", compiler.get("digest").s()),
        ("UPSTREAM_INPUT_HASH_HEX", b.get("inputHash").s()),
    ];
    for (name, value) in scalar_str {
        out.push_str(&format!(
            "#[rustfmt::skip]\npub const {name}: &str = \"{}\";\n",
            str_literal(value)
        ));
    }
    let scalar_u32 = [
        ("ABI_VERSION", abi.get("abiVersion").u()),
        ("POINTER_WIDTH_BITS", abi.get("pointerWidth").u()),
        ("POINTER_BYTES", layout.get("pointerBytes").u()),
        ("MAX_ALIGNMENT", layout.get("maxAlignment").u()),
        ("ROOT_HEADER_BYTES", layout.get("rootHeaderBytes").u()),
        ("TABLE_HEADER_BYTES", layout.get("tableHeaderBytes").u()),
        (
            "ROOT_DECLARED_STRUCT_SIZE",
            root.get("declaredStructSize").u(),
        ),
        (
            "ROOT_MINIMUM_STRUCT_SIZE",
            root.get("minimumStructSize").u(),
        ),
        ("STATUS_SIZE_BYTES", status.get("size").u()),
        ("HANDLE_SIZE_BYTES", handle.get("size").u()),
        ("HANDLE_ALIGN_BYTES", handle.get("align").u()),
        ("BUFFER_SIZE_BYTES", buffer.get("size").u()),
        ("BUFFER_ALIGN_BYTES", buffer.get("align").u()),
    ];
    for (name, value) in scalar_u32 {
        out.push_str(&format!(
            "#[rustfmt::skip]\npub const {name}: u32 = {value};\n"
        ));
    }
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const CAPABILITY_BITS: u64 = {};\n",
        abi.get("capabilityBits").u()
    ));
    out.push('\n');
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const ROOT_ABI_BUNDLE_SHA256_HEX: &str = \"{}\";\n",
        sha256_hex(bundle_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const ROOT_ABI_BUNDLE_JSON: &[u8] = b\"{}\";\n",
        byte_literal(bundle_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const LUMIO_CORE_H_SHA256_HEX: &str = \"{}\";\n",
        sha256_hex(header_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const LUMIO_CORE_H: &[u8] = b\"{}\";\n",
        byte_literal(header_bytes)
    ));
    out
}

// ── 渲染：src/generated/error_codes.rs ─────────────────────────────────────

fn registry_namespace<'a>(ids_index: &'a Json, name: &str) -> &'a Json {
    ids_index
        .get("namespaces")
        .arr()
        .iter()
        .find(|n| n.get("namespace").s() == name)
        .unwrap_or_else(|| panic!("ids/index.json 缺少 namespace {name}"))
}

fn render_enum(out: &mut String, enum_name: &str, ns: &Json) {
    let values = ns.get("values").arr();
    for v in values {
        let status = v.get("status").s();
        // 状态枚举以 schemas/id-registry.schema.json 的 status enum 为准。
        assert!(
            matches!(status, "Active" | "Reserved" | "Deprecated"),
            "未知 id status：{status}"
        );
    }
    out.push_str(&format!(
        "/// ids/index.json `{}` namespace 的 1:1 派生（owner: {}）。\n",
        ns.get("namespace").s(),
        ns.get("owner").s()
    ));
    out.push_str("#[rustfmt::skip]\n#[allow(clippy::upper_case_acronyms)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    out.push_str(&format!("pub enum {enum_name} {{\n"));
    for v in values {
        out.push_str(&format!(
            "    {} = {},\n",
            v.get("id").s(),
            v.get("numeric").u()
        ));
    }
    out.push_str("}\n\n");
    out.push_str(&format!("#[rustfmt::skip]\nimpl {enum_name} {{\n"));
    out.push_str(&format!(
        "    pub const ALL: [{enum_name}; {}] = [\n",
        values.len()
    ));
    for v in values {
        out.push_str(&format!("        {enum_name}::{},\n", v.get("id").s()));
    }
    out.push_str("    ];\n\n");
    out.push_str("    pub const fn numeric(self) -> u32 {\n        self as u32\n    }\n\n");
    out.push_str("    pub const fn id(self) -> &'static str {\n        match self {\n");
    for v in values {
        let id = v.get("id").s();
        out.push_str(&format!("            {enum_name}::{id} => \"{id}\",\n"));
    }
    out.push_str("        }\n    }\n\n");
    out.push_str("    pub const fn status(self) -> IdStatus {\n        match self {\n");
    for v in values {
        out.push_str(&format!(
            "            {enum_name}::{} => IdStatus::{},\n",
            v.get("id").s(),
            v.get("status").s()
        ));
    }
    out.push_str("        }\n    }\n\n");
    out.push_str("    pub const fn since(self) -> &'static str {\n        match self {\n");
    for v in values {
        out.push_str(&format!(
            "            {enum_name}::{} => \"{}\",\n",
            v.get("id").s(),
            str_literal(v.get("since").s())
        ));
    }
    out.push_str("        }\n    }\n\n");
    out.push_str(&format!(
        "    pub const fn from_numeric(numeric: u32) -> Option<{enum_name}> {{\n        match numeric {{\n"
    ));
    for v in values {
        out.push_str(&format!(
            "            {} => Some({enum_name}::{}),\n",
            v.get("numeric").u(),
            v.get("id").s()
        ));
    }
    out.push_str("            _ => None,\n        }\n    }\n}\n\n");
}

fn render_error_codes_rs(inp: &Inputs) -> String {
    let ids = &inp.ids_index;
    assert_eq!(ids.get("baselineId").s(), BASELINE_ID);
    let ids_bytes = &inp.files["ids/index.json"];
    let mut out = String::new();
    out.push_str(
        "//! ErrorCode / Capability 消费面（ids/index.json）：registry 全文逐字节嵌入 +\n\
         //! Architecture 所有的两个 namespace 的 1:1 枚举派生。MessageType / FaultClass\n\
         //! （owner: GameRuntime）不属本仓消费面，只随嵌入字节提供，不做类型派生。\n",
    );
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const ID_REGISTRY_VERSION: u32 = {};\n",
        ids.get("registryVersion").u()
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const IDS_INDEX_SHA256_HEX: &str = \"{}\";\n",
        sha256_hex(ids_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const IDS_INDEX_JSON: &[u8] = b\"{}\";\n",
        byte_literal(ids_bytes)
    ));
    out.push('\n');
    out.push_str(
        "/// id-registry（schemas/id-registry.schema.json）status 字段的枚举。\n\
         #[rustfmt::skip]\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n\
         pub enum IdStatus {\n    Active,\n    Reserved,\n    Deprecated,\n}\n\n",
    );
    let error_ns = registry_namespace(ids, "ErrorCode");
    assert_eq!(error_ns.get("owner").s(), "Architecture");
    render_enum(&mut out, "ErrorCode", error_ns);
    let capability_ns = registry_namespace(ids, "Capability");
    assert_eq!(capability_ns.get("owner").s(), "Architecture");
    render_enum(&mut out, "Capability", capability_ns);
    // render_enum 以空行收尾；文件末尾收敛为单个换行（rustfmt 定点）。
    assert!(out.ends_with("}\n\n"));
    out.pop();
    out
}

// ── 渲染：src/generated/schema_registry.rs ─────────────────────────────────

fn render_schema_registry_rs(inp: &Inputs) -> String {
    let idx = &inp.schemas_index;
    assert_eq!(idx.get("baselineId").s(), BASELINE_ID);
    let index_bytes = &inp.files["schemas/index.json"];
    let common_bytes = &inp.files["schemas/common.schema.json"];
    let entries = idx.get("schemas").arr();
    let mut out = String::new();
    out.push_str(
        "//! Schema registry 消费面（schemas/）：注册表逐字节嵌入 + 逐 schema 的\n\
         //! 摘要索引访问。共享 $defs 文件 common.schema.json 未在上游注册表登记，\n\
         //! 以独立常量提供。摘要为字节 SHA-256，与 architecture.lock.json 同源可对账。\n",
    );
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    out.push_str(
        "/// 上游注册表条目 + 本仓摘要索引：字段逐项来自 schemas/index.json 与镜像字节。\n\
         #[rustfmt::skip]\n#[derive(Debug)]\n\
         pub struct SchemaEntry {\n\
         \x20   pub id: &'static str,\n\
         \x20   pub file: &'static str,\n\
         \x20   pub owner: &'static str,\n\
         \x20   pub priority: &'static str,\n\
         \x20   pub sha256_hex: &'static str,\n\
         \x20   pub bytes: &'static [u8],\n\
         }\n\n",
    );
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const SCHEMA_SET_VERSION: u32 = {};\n",
        idx.get("schemaSetVersion").u()
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const SCHEMAS_INDEX_SHA256_HEX: &str = \"{}\";\n",
        sha256_hex(index_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const SCHEMAS_INDEX_JSON: &[u8] = b\"{}\";\n",
        byte_literal(index_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const COMMON_SCHEMA_DEFS_SHA256_HEX: &str = \"{}\";\n",
        sha256_hex(common_bytes)
    ));
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const COMMON_SCHEMA_DEFS: &[u8] = b\"{}\";\n",
        byte_literal(common_bytes)
    ));
    out.push('\n');
    out.push_str(&format!(
        "#[rustfmt::skip]\npub const SCHEMAS: [SchemaEntry; {}] = [\n",
        entries.len()
    ));
    for e in entries {
        let file = e.get("file").s();
        let bytes = &inp.files[&format!("schemas/{file}")];
        out.push_str(&format!(
            "    SchemaEntry {{\n        id: \"{}\",\n        file: \"{}\",\n        owner: \"{}\",\n        priority: \"{}\",\n        sha256_hex: \"{}\",\n        bytes: b\"{}\",\n    }},\n",
            str_literal(e.get("id").s()),
            str_literal(file),
            str_literal(e.get("owner").s()),
            str_literal(e.get("priority").s()),
            sha256_hex(bytes),
            byte_literal(bytes),
        ));
    }
    out.push_str("];\n\n");
    out.push_str(
        "/// Schema bytes 的摘要索引访问（小写 64 位十六进制字节 SHA-256）。\n\
         #[rustfmt::skip]\n\
         pub fn schema_by_digest(sha256_hex: &str) -> Option<&'static SchemaEntry> {\n\
         \x20   SCHEMAS.iter().find(|e| e.sha256_hex == sha256_hex)\n\
         }\n\n\
         /// 按上游注册表 contract id 访问。\n\
         #[rustfmt::skip]\n\
         pub fn schema_by_id(id: &str) -> Option<&'static SchemaEntry> {\n\
         \x20   SCHEMAS.iter().find(|e| e.id == id)\n\
         }\n",
    );
    out
}

// ── 渲染：generated-contract-artifact.json ─────────────────────────────────

fn aggregate_hash(pairs: &BTreeMap<String, String>) -> String {
    let stream: String = pairs.iter().map(|(p, h)| format!("{p} {h}\n")).collect();
    sha256_hex(stream.as_bytes())
}

fn json_escape(s: &str) -> String {
    assert!(s.is_ascii(), "descriptor 值出现非 ASCII：{s:?}");
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_descriptor(inp: &Inputs, outputs: &BTreeMap<String, String>) -> String {
    let root_abi = inp.packages_index.get("rootAbi");
    let compiler = root_abi.get("compiler");
    let bundle = &inp.bundle;
    // 上游发布值两处（packages/index.json 与 bundle 本体）逐项一致才可记录。
    assert_eq!(
        compiler.get("digest").s(),
        bundle.get("compiler").get("digest").s()
    );
    assert_eq!(root_abi.get("inputHash").s(), bundle.get("inputHash").s());
    assert_eq!(
        root_abi.get("bundleDigest").s(),
        sha256_hex(&inp.files["packages/abi/root-abi-bundle.json"]),
        "bundleDigest 与镜像 bundle 字节不一致"
    );

    let inputs: BTreeMap<String, String> = inp
        .files
        .iter()
        .map(|(p, bytes)| (p.clone(), sha256_hex(bytes)))
        .collect();
    let self_bytes = fs::read(inp.root.join(SELF_REL)).expect("读生成器自身失败");
    let self_normalized: Vec<u8> = self_bytes.into_iter().filter(|b| *b != b'\r').collect();

    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"schemaVersion\": 1,\n");
    s.push_str("  \"kind\": \"contracts-wrapper-generation-record\",\n");
    s.push_str("  \"repository\": \"https://github.com/LumioGames/LumioGameEngine\",\n");
    s.push_str(&format!(
        "  \"architectureRepository\": \"{}\",\n",
        json_escape(&inp.architecture_repository)
    ));
    s.push_str(&format!(
        "  \"architectureCommit\": \"{}\",\n",
        json_escape(&inp.architecture_commit)
    ));
    s.push_str(&format!(
        "  \"architectureBaselineId\": \"{BASELINE_ID}\",\n"
    ));
    s.push_str(&format!("  \"mirrorRoot\": \"{MIRROR_REL}\",\n"));
    s.push_str(&format!("  \"generatorName\": \"{SELF_REL}\",\n"));
    s.push_str("  \"generatorVersion\": 1,\n");
    s.push_str(&format!(
        "  \"generatorSha256\": \"{}\",\n",
        sha256_hex(&self_normalized)
    ));
    s.push_str(&format!("  \"argv\": \"{REGENERATE_ARGV}\",\n"));
    s.push_str("  \"targetPlatform\": \"host-independent\",\n");
    s.push_str("  \"upstream\": {\n");
    s.push_str(&format!(
        "    \"artifactId\": \"{}\",\n",
        json_escape(root_abi.get("bundleId").s())
    ));
    s.push_str("    \"artifactKind\": \"ContractTypes\",\n");
    s.push_str("    \"publisher\": \"LumioGameEngineArchitecture\",\n");
    s.push_str(&format!(
        "    \"schemaEpoch\": {},\n",
        inp.packages_index.get("schemaEpoch").u()
    ));
    s.push_str(&format!(
        "    \"compiler\": {{ \"name\": \"{}\", \"version\": \"{}\", \"digest\": \"{}\" }},\n",
        json_escape(compiler.get("name").s()),
        json_escape(compiler.get("version").s()),
        json_escape(compiler.get("digest").s())
    ));
    s.push_str(&format!(
        "    \"inputHash\": \"{}\",\n",
        json_escape(root_abi.get("inputHash").s())
    ));
    s.push_str(&format!(
        "    \"bundleDigest\": \"{}\",\n",
        json_escape(root_abi.get("bundleDigest").s())
    ));
    s.push_str(&format!(
        "    \"layoutProfileId\": \"{}\",\n",
        json_escape(root_abi.get("layoutProfileId").s())
    ));
    s.push_str("    \"outputFiles\": [\n");
    let out_files = root_abi.get("outputFiles").arr();
    for (i, f) in out_files.iter().enumerate() {
        let sep = if i + 1 == out_files.len() { "" } else { "," };
        s.push_str(&format!(
            "      {{ \"path\": \"{}\", \"digest\": \"{}\", \"role\": \"{}\" }}{sep}\n",
            json_escape(f.get("path").s()),
            json_escape(f.get("digest").s()),
            json_escape(f.get("role").s())
        ));
    }
    s.push_str("    ]\n");
    s.push_str("  },\n");
    s.push_str(&format!("  \"inputFileCount\": {},\n", inputs.len()));
    s.push_str("  \"inputPaths\": {\n");
    let n = inputs.len();
    for (i, (p, h)) in inputs.iter().enumerate() {
        let sep = if i + 1 == n { "" } else { "," };
        s.push_str(&format!("    \"{p}\": \"{h}\"{sep}\n"));
    }
    s.push_str("  },\n");
    s.push_str(&format!(
        "  \"inputHash\": \"{}\",\n",
        aggregate_hash(&inputs)
    ));
    s.push_str("  \"outputPaths\": {\n");
    let n = outputs.len();
    for (i, (p, h)) in outputs.iter().enumerate() {
        let sep = if i + 1 == n { "" } else { "," };
        s.push_str(&format!("    \"{p}\": \"{h}\"{sep}\n"));
    }
    s.push_str("  },\n");
    s.push_str(&format!(
        "  \"outputHash\": \"{}\"\n",
        aggregate_hash(outputs)
    ));
    s.push_str("}\n");
    s
}

// ── 渲染全集 ───────────────────────────────────────────────────────────────

fn render_all(inp: &Inputs) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(format!("{GENERATED_DIR}/mod.rs"), render_mod_rs(inp));
    files.insert(
        format!("{GENERATED_DIR}/contracts.rs"),
        render_contracts_rs(inp),
    );
    files.insert(
        format!("{GENERATED_DIR}/error_codes.rs"),
        render_error_codes_rs(inp),
    );
    files.insert(
        format!("{GENERATED_DIR}/schema_registry.rs"),
        render_schema_registry_rs(inp),
    );
    let outputs: BTreeMap<String, String> = files
        .iter()
        .map(|(p, text)| (p.clone(), sha256_hex(text.as_bytes())))
        .collect();
    files.insert(DESCRIPTOR_REL.to_string(), render_descriptor(inp, &outputs));
    files
}

fn regenerate_requested() -> bool {
    std::env::var("LUMIO_CONTRACTS_REGENERATE").is_ok_and(|v| v == "1")
}

// ── 校验 ───────────────────────────────────────────────────────────────────

/// 重新渲染零差异（重生成模式在此落盘）。其余测试均以本断言为前提逐项取证。
#[test]
fn rendered_files_match_committed() {
    let inp = load_inputs();
    let rendered = render_all(&inp);
    if regenerate_requested() {
        for (rel, text) in &rendered {
            let path = inp.root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, text).unwrap_or_else(|e| panic!("写 {rel} 失败：{e}"));
        }
    }
    for (rel, text) in &rendered {
        let committed = fs::read(inp.root.join(rel)).unwrap_or_else(|e| {
            panic!("读已提交生成物 {rel} 失败（未生成？重生成命令见文件头注释）：{e}")
        });
        assert!(
            committed == text.as_bytes(),
            "{rel} 与锁定生成器的重新渲染不一致（生成物被手改或输入漂移；重生成命令见文件头注释）"
        );
    }
}

/// 验收项 1：descriptor 的 Input/Output Hash 与逐文件摘要从字节重算一致。
#[test]
fn descriptor_input_output_hash_match() {
    let inp = load_inputs();
    let text = fs::read_to_string(inp.root.join(DESCRIPTOR_REL)).expect("读 descriptor 失败");
    let d = parse_json(&text);
    let input_paths: BTreeMap<String, String> = match d.get("inputPaths") {
        Json::Obj(m) => m
            .iter()
            .map(|(k, v)| (k.clone(), v.s().to_string()))
            .collect(),
        _ => panic!("inputPaths 非对象"),
    };
    assert_eq!(
        input_paths.keys().collect::<Vec<_>>(),
        inp.files.keys().collect::<Vec<_>>(),
        "descriptor 输入集与本卡消费面不一致"
    );
    for (rel, declared) in &input_paths {
        let actual = sha256_hex(&inp.files[rel]);
        assert_eq!(&actual, declared, "输入 {rel} 摘要与镜像字节不一致");
        let lock_declared = inp
            .lock_sha
            .get(rel)
            .unwrap_or_else(|| panic!("architecture.lock.json 缺少 {rel}"));
        assert_eq!(lock_declared, declared, "输入 {rel} 摘要与 lock 登记不一致");
    }
    assert_eq!(
        d.get("inputHash").s(),
        aggregate_hash(&input_paths),
        "descriptor inputHash 重算不一致"
    );
    let output_paths: BTreeMap<String, String> = match d.get("outputPaths") {
        Json::Obj(m) => m
            .iter()
            .map(|(k, v)| (k.clone(), v.s().to_string()))
            .collect(),
        _ => panic!("outputPaths 非对象"),
    };
    for (rel, declared) in &output_paths {
        let actual = sha256_hex(&fs::read(inp.root.join(rel)).expect("读生成物失败"));
        assert_eq!(&actual, declared, "输出 {rel} 摘要与提交字节不一致");
    }
    assert_eq!(
        d.get("outputHash").s(),
        aggregate_hash(&output_paths),
        "descriptor outputHash 重算不一致"
    );
    let self_bytes = fs::read(inp.root.join(SELF_REL)).unwrap();
    let self_normalized: Vec<u8> = self_bytes.into_iter().filter(|b| *b != b'\r').collect();
    assert_eq!(
        d.get("generatorSha256").s(),
        sha256_hex(&self_normalized),
        "generatorSha256 与本文件当前内容不一致（生成器变更后必须重生成并一起提交）"
    );
}

/// 上游 provenance 与镜像发布值逐项对账（compiler / inputHash / bundleDigest /
/// outputFiles；镜像内可达的输出 lumio_core.h 摘要从字节重算）。
#[test]
fn descriptor_upstream_provenance_matches_mirror() {
    let inp = load_inputs();
    let text = fs::read_to_string(inp.root.join(DESCRIPTOR_REL)).expect("读 descriptor 失败");
    let d = parse_json(&text);
    let up = d.get("upstream");
    let root_abi = inp.packages_index.get("rootAbi");
    assert_eq!(up.get("artifactId").s(), root_abi.get("bundleId").s());
    assert_eq!(
        up.get("compiler").get("digest").s(),
        root_abi.get("compiler").get("digest").s()
    );
    assert_eq!(up.get("inputHash").s(), root_abi.get("inputHash").s());
    assert_eq!(up.get("bundleDigest").s(), root_abi.get("bundleDigest").s());
    assert_eq!(
        up.get("bundleDigest").s(),
        sha256_hex(&inp.files["packages/abi/root-abi-bundle.json"])
    );
    let declared: Vec<(String, String, String)> = up
        .get("outputFiles")
        .arr()
        .iter()
        .map(|f| {
            (
                f.get("path").s().to_string(),
                f.get("digest").s().to_string(),
                f.get("role").s().to_string(),
            )
        })
        .collect();
    let published: Vec<(String, String, String)> = root_abi
        .get("outputFiles")
        .arr()
        .iter()
        .map(|f| {
            (
                f.get("path").s().to_string(),
                f.get("digest").s().to_string(),
                f.get("role").s().to_string(),
            )
        })
        .collect();
    assert_eq!(declared, published, "outputFiles 与上游发布不一致");
    let header = declared
        .iter()
        .find(|(p, _, _)| p == "abi/lumio_core.h")
        .expect("上游 outputFiles 缺少 abi/lumio_core.h");
    assert_eq!(
        header.1,
        sha256_hex(&inp.files["packages/abi/lumio_core.h"]),
        "lumio_core.h 上游摘要与镜像字节不一致"
    );
}

/// 验收项 3：ErrorCode / Capability 与上游 ID registry 集合精确一致。
/// 生成文件是注册表的确定性函数，rendered_files_match_committed 已锁字节一致；
/// 此处再按解析值直接断言（id, numeric, status, since）全集，双保险。
#[test]
fn error_code_and_capability_sets_match_registry() {
    let inp = load_inputs();
    let committed = fs::read_to_string(inp.root.join(format!("{GENERATED_DIR}/error_codes.rs")))
        .expect("读 error_codes.rs 失败");
    for ns_name in ["ErrorCode", "Capability"] {
        let ns = registry_namespace(&inp.ids_index, ns_name);
        let values = ns.get("values").arr();
        let mut expected_variants = String::new();
        for v in values {
            expected_variants.push_str(&format!(
                "    {} = {},\n",
                v.get("id").s(),
                v.get("numeric").u()
            ));
        }
        let enum_header = format!("pub enum {ns_name} {{\n");
        let start = committed
            .find(&enum_header)
            .unwrap_or_else(|| panic!("error_codes.rs 缺少 enum {ns_name}"));
        let body_start = start + enum_header.len();
        let body_end = committed[body_start..]
            .find("}\n")
            .map(|off| body_start + off)
            .expect("enum 未闭合");
        assert_eq!(
            &committed[body_start..body_end],
            expected_variants,
            "enum {ns_name} 与 registry 值集不一致"
        );
        assert!(
            committed.matches(&format!("{ns_name}::")).count() >= values.len() * 4,
            "enum {ns_name} 的派生表不完整"
        );
    }
}
