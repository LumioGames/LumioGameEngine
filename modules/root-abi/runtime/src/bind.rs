//! 调用 entry、检查 null/version/size/capability/layout（规格 §8.2 `bind.rs`、§8.3）。
//!
//! **本 crate 的全部 `unsafe` 集中在这里。** 读取顺序严格是「先检查 header 前缀，
//! 再读后续 slot」：ADR-040 §4 冻结了 Root Table 与 API Table 的 16 字节头部，
//! 头部之后的每一次读取都必须先由**已校验的** `struct_size` 证明落在对象内。

use std::ffi::c_void;
use std::mem;
use std::ptr::NonNull;
use std::sync::Arc;

use lumio_core_contracts::ErrorCode;

use crate::error::RootAbiError;
use crate::expectation::{AbiExpectation, Endianness};
use crate::generated::{self, LumioCoreApi, LumioStatus, LumioVoxelApi};
use crate::symbol::{SymbolLookupError, SymbolResolver};
use crate::table_view::{
    GeneratedRootApiTable, RootApiTableView, RootSnapshot, TableSnapshot, TABLE_COUNT,
};

/// entry 的签名，取自生成 header 的 `lumio_core_get_api_v1` 声明。
type EntryFn = unsafe extern "C" fn(u32, *mut *const GeneratedRootApiTable) -> LumioStatus;

// 头部字段偏移一律由生成结构体反推，本仓不写任何布局字面量。
const OFF_ABI_VERSION: usize = mem::offset_of!(GeneratedRootApiTable, abi_version);
const OFF_STRUCT_SIZE: usize = mem::offset_of!(GeneratedRootApiTable, struct_size);
const OFF_CAPABILITY_BITS: usize = mem::offset_of!(GeneratedRootApiTable, capability_bits);
const OFF_TABLE_VERSION: usize = mem::offset_of!(LumioCoreApi, version);
const OFF_TABLE_STRUCT_SIZE: usize = mem::offset_of!(LumioCoreApi, struct_size);

const _: () = {
    // Root header 三个字段必须全部落在 ADR-040 §4 冻结的 16 字节头部内。
    assert!(OFF_ABI_VERSION + mem::size_of::<u32>() <= generated::ROOT_HEADER_BYTES);
    assert!(OFF_STRUCT_SIZE + mem::size_of::<u32>() <= generated::ROOT_HEADER_BYTES);
    assert!(OFF_CAPABILITY_BITS + mem::size_of::<u64>() <= generated::ROOT_HEADER_BYTES);
    // 两张生成 table 的头部布局必须一致（ADR-040 §4 的 API table header）。
    assert!(OFF_TABLE_VERSION == mem::offset_of!(LumioVoxelApi, version));
    assert!(OFF_TABLE_STRUCT_SIZE == mem::offset_of!(LumioVoxelApi, struct_size));
    assert!(OFF_TABLE_STRUCT_SIZE + mem::size_of::<u32>() <= generated::TABLE_HEADER_BYTES);
};

/// 一张生成 API table 的绑定期规格，逐项从生成物派生。
struct TableSpec {
    /// 上游 Golden 里的 table 名。
    name: &'static str,
    /// 该 table 指针在 Root Table 中的偏移。
    root_offset: usize,
    /// 镜像必须至少提供的字节数：生成绑定里这张 table 的完整大小。
    ///
    /// 取 `size_of` 而不是「最高 slot 偏移 + 一个指针」，是因为生成结构体里
    /// **包含**上游声明的保留 slot（`lumio_core_api.reserved`），因此它恰好等于
    /// ADR-040 §4 的派生最小值 `16 + (functionCount + reservedSlots) * pointerBytes`；
    /// 而 `reservedSlots` 本身没有任何 Rust 可消费的常量。下方 const 断言保证它
    /// 始终覆盖全部 slot 槽位。
    required_bytes: usize,
}

const TABLE_SPECS: [TableSpec; TABLE_COUNT] = [
    TableSpec {
        name: "lumio_core_api",
        root_offset: mem::offset_of!(GeneratedRootApiTable, lumio_core_api),
        required_bytes: mem::size_of::<LumioCoreApi>(),
    },
    TableSpec {
        name: "lumio_voxel_api",
        root_offset: mem::offset_of!(GeneratedRootApiTable, lumio_voxel_api),
        required_bytes: mem::size_of::<LumioVoxelApi>(),
    },
];

const _: () = {
    assert!(mem::size_of::<LumioCoreApi>() >= slot_span("lumio_core_api"));
    assert!(mem::size_of::<LumioVoxelApi>() >= slot_span("lumio_voxel_api"));
};

/// Root Table 自身必须覆盖的字节数：header 加上全部 table 指针槽位。
const ROOT_REQUIRED_BYTES: usize = {
    let mut required = generated::ROOT_HEADER_BYTES;
    let mut i = 0;
    while i < TABLE_COUNT {
        let end = TABLE_SPECS[i].root_offset + generated::POINTER_BYTES;
        if end > required {
            required = end;
        }
        i += 1;
    }
    required
};

/// 从生成 Golden 推出某张 table 上「最高 slot 偏移 + 一个指针」。
const fn slot_span(table: &str) -> usize {
    let mut required = generated::TABLE_HEADER_BYTES;
    let mut i = 0;
    while i < generated::SLOT_OFFSETS.len() {
        let (name, _slot, offset) = generated::SLOT_OFFSETS[i];
        if const_str_eq(name, table) {
            let end = offset + generated::POINTER_BYTES;
            if end > required {
                required = end;
            }
        }
        i += 1;
    }
    required
}

const fn const_str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
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

/// 解析唯一 entry symbol、调用它，并逐项校验镜像发布的 Root API Table。
///
/// # Safety
///
/// 调用方保证 `resolver` 满足 [`SymbolResolver`] 的寿命义务：它返回的地址在
/// `resolver` 存活期间有效，且 `expected.entry_symbol` 在目标镜像里确实是
/// 生成 header 声明的那个 `lumio_core_get_api_v1` 签名。签名不符即是 UB，
/// 本函数无法检测——这是「唯一 entry symbol」由生成契约固定的原因。
///
/// 成功返回的 [`RootApiTableView`] 私有持有 `resolver` 的强引用，因此表的寿命
/// 不短于视图本身。
pub unsafe fn bind_root_api(
    resolver: Arc<dyn SymbolResolver>,
    expected: &AbiExpectation,
) -> Result<RootApiTableView, RootAbiError> {
    // ① 宿主与期望的基本 ABI 属性——在碰镜像之前就能判定的部分先判定。
    let host_pointer_bits = mem::size_of::<*const c_void>() * 8;
    if usize::from(expected.pointer_width) != host_pointer_bits {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：期望指针宽度 {} 位，宿主是 {host_pointer_bits} 位",
            expected.abi_identity, expected.pointer_width
        )));
    }
    if Endianness::host() != expected.endianness {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：期望 endianness {}，宿主是 {}",
            expected.abi_identity,
            expected.endianness.as_str(),
            Endianness::host().as_str()
        )));
    }

    // ② 解析唯一 entry symbol。
    // SAFETY: 由本函数的 Safety 契约转嫁给 resolver 实现。
    let entry_address = unsafe { resolver.resolve(expected.entry_symbol) }.map_err(|error| {
        let symbol = expected.entry_symbol.to_string_lossy();
        match error {
            SymbolLookupError::NotFound => {
                RootAbiError::entry_symbol_missing(format!("镜像里没有 entry symbol {symbol}"))
            }
            SymbolLookupError::Collision => RootAbiError::entry_symbol_collision(format!(
                "entry symbol {symbol} 解析到多个候选"
            )),
        }
    })?;

    // ③ 调用 entry 取表。
    // SAFETY: 签名由生成 header 固定，符合本函数 Safety 契约的前提；
    // `out_table` 是本地变量地址，非空且对齐。
    let status = unsafe {
        let entry = mem::transmute::<*mut c_void, EntryFn>(entry_address.as_ptr());
        let mut out_table: *const GeneratedRootApiTable = std::ptr::null();
        let status = entry(expected.abi_version, &mut out_table);
        if status == 0 {
            Ok(out_table)
        } else {
            Err(status)
        }
    };

    let raw = match status {
        Ok(raw) => raw,
        Err(status) => return Err(entry_status_to_error(expected, status)),
    };

    // ④ null 与对齐——任何读取之前必须先过这两关。
    let raw = NonNull::new(raw.cast_mut()).ok_or_else(|| {
        RootAbiError::abi_mismatch(format!(
            "{}：entry 返回成功但表指针为空",
            expected.abi_identity
        ))
    })?;
    let address = raw.as_ptr() as usize;
    if address % generated::MAX_ALIGNMENT != 0 {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：表地址 {address:#x} 未按 {} 字节对齐",
            expected.abi_identity,
            generated::MAX_ALIGNMENT
        )));
    }

    let base = raw.as_ptr().cast::<u8>();

    // ⑤ header 前缀：ADR-040 §4 冻结 Root header 为 16 字节，先读它、先判它。
    // SAFETY: `base` 非空且对齐；ADR-040 §4 保证任何合规 Root Table 至少有
    // 16 字节头部，三个字段的偏移已在编译期断言落在头部内。
    let (abi_version, declared_struct_size, capability_bits) = unsafe {
        (
            base.add(OFF_ABI_VERSION).cast::<u32>().read(),
            base.add(OFF_STRUCT_SIZE).cast::<u32>().read(),
            base.add(OFF_CAPABILITY_BITS).cast::<u64>().read(),
        )
    };

    if abi_version != expected.abi_version {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：期望 abi_version {}，表声明 {abi_version}",
            expected.abi_identity, expected.abi_version
        )));
    }

    let struct_size = declared_struct_size as usize;
    if struct_size % generated::MAX_ALIGNMENT != 0 {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：root struct_size {struct_size} 不是 {} 的整数倍",
            expected.abi_identity,
            generated::MAX_ALIGNMENT
        )));
    }
    let root_floor = ROOT_REQUIRED_BYTES.max(expected.minimum_struct_size);
    if struct_size < root_floor {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：root struct_size {struct_size} 小于最小值 {root_floor}",
            expected.abi_identity
        )));
    }

    // capability_bits 只做精确相等：ADR-040 未冻结位语义，子集判定要靠本仓自造位位置。
    if capability_bits != expected.required_capability_bits {
        return Err(RootAbiError::abi_mismatch(format!(
            "{}：期望 capability_bits {}，表声明 {capability_bits}",
            expected.abi_identity, expected.required_capability_bits
        )));
    }

    // ⑥ header 已过关，现在才允许读后续 table 指针与各 slot。
    let mut tables = [TableSnapshot {
        name: "",
        version: 0,
        struct_size: 0,
    }; TABLE_COUNT];
    for (index, spec) in TABLE_SPECS.iter().enumerate() {
        // SAFETY: 已校验 struct_size >= ROOT_REQUIRED_BYTES，覆盖每个 table 指针槽位。
        let table_ptr = unsafe { base.add(spec.root_offset).cast::<*const u8>().read() };
        let table_ptr = NonNull::new(table_ptr.cast_mut()).ok_or_else(|| {
            RootAbiError::abi_mismatch(format!(
                "{}：table {} 的指针为空",
                expected.abi_identity, spec.name
            ))
        })?;
        // SAFETY: 非空已判；对齐与 16 字节头部同 Root Table，由 ADR-040 §4 保证。
        let table_address = table_ptr.as_ptr() as usize;
        if table_address % generated::MAX_ALIGNMENT != 0 {
            return Err(RootAbiError::abi_mismatch(format!(
                "{}：table {} 地址 {table_address:#x} 未按 {} 字节对齐",
                expected.abi_identity,
                spec.name,
                generated::MAX_ALIGNMENT
            )));
        }

        let table_base = table_ptr.as_ptr();
        // SAFETY: ADR-040 §4 保证合规 API table 至少有 16 字节头部。
        let (version, declared) = unsafe {
            (
                table_base.add(OFF_TABLE_VERSION).cast::<u32>().read(),
                table_base.add(OFF_TABLE_STRUCT_SIZE).cast::<u32>().read(),
            )
        };

        let table_struct_size = declared as usize;
        if table_struct_size % generated::MAX_ALIGNMENT != 0 {
            return Err(RootAbiError::abi_mismatch(format!(
                "{}：table {} 的 struct_size {table_struct_size} 不是 {} 的整数倍",
                expected.abi_identity,
                spec.name,
                generated::MAX_ALIGNMENT
            )));
        }
        if table_struct_size < spec.required_bytes {
            return Err(RootAbiError::abi_mismatch(format!(
                "{}：table {} 的 struct_size {table_struct_size} 小于最小值 {}",
                expected.abi_identity, spec.name, spec.required_bytes
            )));
        }

        // slot 指针：`struct_size` 已证明每个 slot 偏移都落在对象内。
        for (_, slot, offset) in generated::SLOT_OFFSETS
            .iter()
            .filter(|(t, _, _)| *t == spec.name)
        {
            // SAFETY: `offset + POINTER_BYTES <= required_bytes <= table_struct_size`。
            let slot_ptr = unsafe { table_base.add(*offset).cast::<*const c_void>().read() };
            if slot_ptr.is_null() {
                return Err(RootAbiError::abi_mismatch(format!(
                    "{}：table {} 的 slot {slot}（偏移 {offset}）为空",
                    expected.abi_identity, spec.name
                )));
            }
        }

        tables[index] = TableSnapshot {
            name: spec.name,
            version,
            struct_size: table_struct_size,
        };
    }

    Ok(RootApiTableView::new(
        raw,
        Arc::new(expected.clone()),
        resolver,
        RootSnapshot {
            abi_version,
            struct_size,
            capability_bits,
            tables,
        },
    ))
}

/// entry 返回的非零 status：已登记码原样透传，未登记码是 status 契约违反。
fn entry_status_to_error(expected: &AbiExpectation, status: LumioStatus) -> RootAbiError {
    match u32::try_from(status).ok().and_then(ErrorCode::from_numeric) {
        Some(code) => RootAbiError::entry_rejected(
            code,
            format!("{}：entry 返回 {status}", expected.abi_identity),
        ),
        None => RootAbiError::abi_mismatch(format!(
            "{}：entry 返回未登记 status {status}",
            expected.abi_identity
        )),
    }
}
