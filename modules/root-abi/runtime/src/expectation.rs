//! 从已验证契约构造加载期望（规格 §8.2 `expectation.rs`、§8.3）。
//!
//! 期望值**一个都不在本仓手写**：全部来自 `lumio-core-contracts` 发布的标量视图与
//! 架构源生成的 Rust 绑定。两条派生路径互为外部锚点——生成绑定来自上游 compiler 的
//! `rust/contracts.rs`，标量视图来自上游 `packages/abi/root-abi-bundle.json` 的嵌入
//! 字节；本文件的编译期断言要求两者逐项一致，任一侧漂移都在编译期失败。

use std::ffi::CStr;

use lumio_core_contracts::contracts as published;

use crate::generated;

/// 字节序；取值来自已发布的 `endianness` 字段，本仓不新增取值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    /// 已发布契约里的拼写。
    pub fn as_str(self) -> &'static str {
        match self {
            Endianness::Little => "Little",
            Endianness::Big => "Big",
        }
    }

    /// 当前编译目标的字节序。
    pub fn host() -> Self {
        if cfg!(target_endian = "little") {
            Endianness::Little
        } else {
            Endianness::Big
        }
    }

    const fn from_published(published: &str) -> Option<Self> {
        // `match` 不能直接匹配 &str 常量的 const fn 形式，改用逐字节比较。
        if bytes_eq(published.as_bytes(), b"Little") {
            Some(Endianness::Little)
        } else if bytes_eq(published.as_bytes(), b"Big") {
            Some(Endianness::Big)
        } else {
            None
        }
    }
}

const fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// 已发布 entry symbol 的 NUL 结尾形式。
///
/// 字面量本身不是真值来源：下方 `const` 断言要求它与 `lumio-core-contracts` 和生成
/// 绑定两侧发布的 `entrySymbol` 逐字节相等，上游改名即编译期失败。
const ENTRY_SYMBOL_WITH_NUL: &[u8] = b"lumio_core_get_api_v1\0";

const _: () = {
    let literal = ENTRY_SYMBOL_WITH_NUL;
    let from_bundle = published::ENTRY_SYMBOL.as_bytes();
    let from_binding = generated::ENTRY_SYMBOL.as_bytes();

    assert!(
        literal.len() == from_bundle.len() + 1,
        "entry symbol 相对已发布 bundle 漂移"
    );
    assert!(
        literal[from_bundle.len()] == 0,
        "entry symbol 必须以单个 NUL 结尾"
    );
    assert!(
        bytes_eq(from_bundle, from_binding),
        "生成绑定与 bundle 的 entry symbol 不一致"
    );

    let mut i = 0;
    while i < from_bundle.len() {
        assert!(
            literal[i] == from_bundle[i],
            "entry symbol 相对已发布 bundle 漂移"
        );
        i += 1;
    }
};

// 生成绑定与 bundle 标量视图必须逐项一致（两条独立派生路径互锚）。
const _: () = {
    assert!(generated::ABI_VERSION == published::ABI_VERSION);
    assert!(generated::CAPABILITY_BITS == published::CAPABILITY_BITS);
    assert!(generated::POINTER_BYTES == published::POINTER_BYTES as usize);
    assert!(generated::MAX_ALIGNMENT == published::MAX_ALIGNMENT as usize);
    assert!(generated::ROOT_HEADER_BYTES == published::ROOT_HEADER_BYTES as usize);
    assert!(generated::TABLE_HEADER_BYTES == published::TABLE_HEADER_BYTES as usize);
    assert!(published::POINTER_WIDTH_BITS == published::POINTER_BYTES * 8);
};

/// 唯一 entry symbol。
pub const ENTRY_SYMBOL: &CStr = match CStr::from_bytes_with_nul(ENTRY_SYMBOL_WITH_NUL) {
    Ok(symbol) => symbol,
    Err(_) => panic!("entry symbol 常量不是合法 C 字符串"),
};

/// 加载期望：绑定时用它逐项核对镜像发布的 Root API Table（规格 §8.3）。
#[derive(Debug, Clone)]
pub struct AbiExpectation {
    /// 仓内可读标识（`bundleId@baselineId`），只用于诊断，不是公共契约。
    pub abi_identity: String,
    pub abi_version: u32,
    pub minimum_struct_size: usize,
    /// 已发布的 `capability_bits` **原值**。
    ///
    /// ADR-040「What this bundle deliberately does not freeze」：V1 既未冻结它是
    /// bitmask 还是计数，也未冻结任何位位置。因此这里只做**精确相等**比较，
    /// 绝不做子集/按位判定——那需要本仓自造位语义。
    pub required_capability_bits: u64,
    pub pointer_width: u8,
    pub endianness: Endianness,
    pub entry_symbol: &'static CStr,
}

impl AbiExpectation {
    /// 从已发布契约构造期望；不接受调用方覆盖任何一项。
    pub fn from_generated_contract() -> Self {
        Self {
            abi_identity: format!(
                "{}@{}",
                published::BUNDLE_ID,
                lumio_core_contracts::ARCHITECTURE_BASELINE_ID
            ),
            abi_version: published::ABI_VERSION,
            minimum_struct_size: published::ROOT_MINIMUM_STRUCT_SIZE as usize,
            required_capability_bits: published::CAPABILITY_BITS,
            pointer_width: u8::try_from(published::POINTER_WIDTH_BITS)
                .expect("已发布指针宽度必须能用 u8 表示"),
            endianness: Endianness::from_published(published::ENDIANNESS)
                .expect("已发布 endianness 必须是 Little 或 Big"),
            entry_symbol: ENTRY_SYMBOL,
        }
    }
}
