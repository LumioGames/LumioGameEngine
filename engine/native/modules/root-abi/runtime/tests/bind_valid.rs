//! 有效 entry/table 绑定（规格 §8.2 `tests/bind_valid.rs`）。
//!
//! 不加载动态库（本卡非目标）：用进程内 `extern "C"` entry + 泄漏的合规 table
//! 充当「已映射镜像」，只验证 `bind_root_api` 的契约行为。

use std::cell::Cell;
use std::ffi::{c_void, CStr};
use std::ptr::NonNull;
use std::sync::Arc;

use lumio_core_root_abi::generated::{
    LumioBuffer, LumioCoreApi, LumioCoreConfigV1, LumioHandle, LumioRootApi, LumioStatus,
    LumioVoxelApi, LumioVoxelWorldDescV1,
};
use lumio_core_root_abi::{bind_root_api, AbiExpectation, SymbolLookupError, SymbolResolver};

// ── 进程内假包（不加载动态库）────────────────────────────────────────────

extern "C" fn core_init(_config: *const LumioCoreConfigV1, _out: LumioHandle) -> LumioStatus {
    0
}
extern "C" fn core_shutdown(_context: LumioHandle) -> LumioStatus {
    0
}
extern "C" fn core_last_error_detail(_context: LumioHandle, _out: LumioBuffer) -> LumioStatus {
    0
}
extern "C" fn voxel_create(
    _context: LumioHandle,
    _desc: *const LumioVoxelWorldDescV1,
    _out: LumioHandle,
) -> LumioStatus {
    0
}
extern "C" fn voxel_destroy(_world: LumioHandle) -> LumioStatus {
    0
}

fn conforming_root() -> *const LumioRootApi {
    let core = Box::leak(Box::new(LumioCoreApi {
        version: 1,
        struct_size: 48,
        reserved0: 0,
        lumio_core_init: Some(core_init),
        lumio_core_shutdown: Some(core_shutdown),
        lumio_core_last_error_detail: Some(core_last_error_detail),
        reserved: [std::ptr::null_mut()],
    }));
    let voxel = Box::leak(Box::new(LumioVoxelApi {
        version: 1,
        struct_size: 32,
        reserved0: 0,
        lumio_voxel_world_create: Some(voxel_create),
        lumio_voxel_world_destroy: Some(voxel_destroy),
    }));
    Box::leak(Box::new(LumioRootApi {
        abi_version: 1,
        struct_size: 64,
        capability_bits: 7,
        lumio_core_api: core,
        lumio_voxel_api: voxel,
        reserved_tail: [0u8; 32],
    }))
}

thread_local! {
    static TABLE: Cell<*const LumioRootApi> = const { Cell::new(std::ptr::null()) };
    static REQUESTED: Cell<u32> = const { Cell::new(u32::MAX) };
}

extern "C" fn entry(requested_version: u32, out_table: *mut *const LumioRootApi) -> LumioStatus {
    REQUESTED.with(|r| r.set(requested_version));
    // SAFETY: 调用方（bind_root_api）传入本地变量地址，非空且对齐。
    unsafe { *out_table = TABLE.with(|t| t.get()) };
    0
}

struct EntryResolver;

impl SymbolResolver for EntryResolver {
    unsafe fn resolve(&self, symbol: &CStr) -> Result<NonNull<c_void>, SymbolLookupError> {
        assert_eq!(
            symbol,
            AbiExpectation::from_generated_contract().entry_symbol
        );
        Ok(NonNull::new(entry as *const () as *mut c_void).expect("fn 地址非空"))
    }
}

fn bind_conforming() -> lumio_core_root_abi::RootApiTableView {
    TABLE.with(|t| t.set(conforming_root()));
    let expected = AbiExpectation::from_generated_contract();
    // SAFETY: resolver 返回的是本进程内 `entry` 的地址，签名与 entry 契约一致，
    // 且 table 在测试进程存活期内被泄漏保持。
    unsafe { bind_root_api(Arc::new(EntryResolver), &expected) }.expect("合规 table 必须绑定成功")
}

// ── 断言 ────────────────────────────────────────────────────────────────

#[test]
fn binds_conforming_table_and_publishes_header_values() {
    let view = bind_conforming();

    assert_eq!(view.abi_version(), 1);
    assert_eq!(view.struct_size(), 64);
    assert_eq!(view.capability_bits(), 7);
}

#[test]
fn passes_expected_abi_version_as_requested_version() {
    let expected = AbiExpectation::from_generated_contract();
    let _view = bind_conforming();

    assert_eq!(REQUESTED.with(|r| r.get()), expected.abi_version);
}

#[test]
fn exposes_generated_tables_with_upstream_header_values() {
    let view = bind_conforming();
    let tables = view.generated_tables();

    assert_eq!(tables.len(), 2);
    assert_eq!(tables.core_api().name(), "lumio_core_api");
    assert_eq!(tables.core_api().version(), 1);
    assert_eq!(tables.core_api().struct_size(), 48);
    assert_eq!(tables.core_api().slot_count(), 3);
    assert_eq!(tables.voxel_api().name(), "lumio_voxel_api");
    assert_eq!(tables.voxel_api().version(), 1);
    assert_eq!(tables.voxel_api().struct_size(), 32);
    assert_eq!(tables.voxel_api().slot_count(), 2);
}

#[test]
fn slot_offsets_come_from_the_upstream_golden() {
    let view = bind_conforming();
    let tables = view.generated_tables();

    assert_eq!(
        tables.core_api().slot_offsets().collect::<Vec<_>>(),
        vec![
            ("lumio_core_init", 16usize),
            ("lumio_core_shutdown", 24),
            ("lumio_core_last_error_detail", 32),
        ]
    );
    assert_eq!(
        tables.voxel_api().slot_offsets().collect::<Vec<_>>(),
        vec![
            ("lumio_voxel_world_create", 16usize),
            ("lumio_voxel_world_destroy", 24)
        ]
    );
}

/// ADR-040 §4：声明的 `structSize` 只要 **不小于** 派生最小值即合规，
/// 尾部保留是下界而不是等式——最小合规 table 必须能绑定。
#[test]
fn accepts_minimum_declared_root_struct_size() {
    let root = conforming_root() as *mut LumioRootApi;
    // SAFETY: `root` 由 Box::leak 产生，仍然存活且独占。
    unsafe { (*root).struct_size = 32 };
    TABLE.with(|t| t.set(root));

    let expected = AbiExpectation::from_generated_contract();
    // SAFETY: 同 `bind_conforming`。
    let view = unsafe { bind_root_api(Arc::new(EntryResolver), &expected) }
        .expect("struct_size 等于派生最小值时必须接受");

    assert_eq!(view.struct_size(), 32);
}

/// §8.3：View 的私有 `Arc<dyn SymbolResolver>` 把 API 表寿命绑定到常驻映像——
/// 绑定成功后 resolver 的强引用必须仍被 View 持有。
#[test]
fn view_keeps_the_image_guard_alive() {
    TABLE.with(|t| t.set(conforming_root()));
    let resolver: Arc<dyn SymbolResolver> = Arc::new(EntryResolver);
    let expected = AbiExpectation::from_generated_contract();

    // SAFETY: 同 `bind_conforming`。
    let view = unsafe { bind_root_api(Arc::clone(&resolver), &expected) }.expect("绑定成功");

    assert_eq!(
        Arc::strong_count(&resolver),
        2,
        "View 必须持有 resolver 的强引用"
    );
    drop(view);
    assert_eq!(Arc::strong_count(&resolver), 1, "View 释放后强引用必须归还");
}

/// 期望值只能来自已发布契约，不得在本仓手写。
#[test]
fn expectation_is_derived_from_published_contract() {
    let expected = AbiExpectation::from_generated_contract();

    assert_eq!(
        expected.abi_version,
        lumio_core_contracts::contracts::ABI_VERSION
    );
    assert_eq!(
        expected.required_capability_bits,
        lumio_core_contracts::contracts::CAPABILITY_BITS
    );
    assert_eq!(
        expected.minimum_struct_size,
        lumio_core_contracts::contracts::ROOT_MINIMUM_STRUCT_SIZE as usize
    );
    assert_eq!(
        u32::from(expected.pointer_width),
        lumio_core_contracts::contracts::POINTER_WIDTH_BITS
    );
    assert_eq!(
        expected
            .entry_symbol
            .to_str()
            .expect("entry symbol 是 UTF-8"),
        lumio_core_contracts::contracts::ENTRY_SYMBOL
    );
    assert_eq!(
        expected.endianness.as_str(),
        lumio_core_contracts::contracts::ENDIANNESS
    );
}

/// 本仓测量出的布局必须等于上游 Golden（三语言一致性的 Rust 一侧）。
///
/// 同时按与 C / C# 探针**完全相同**的规范格式打印测量值——
/// `cargo test ... -- --nocapture` 的输出可与另两份逐行 diff。
#[test]
fn rust_measured_layout_equals_upstream_golden() {
    use lumio_core_root_abi::generated as g;

    println!("abi_version={}", g::ABI_VERSION);
    println!("capability_bits={}", g::CAPABILITY_BITS);
    println!("entry_symbol={}", g::ENTRY_SYMBOL);
    println!("symbol_prefix={}", g::SYMBOL_PREFIX);
    println!("pointer_bytes={}", std::mem::size_of::<*const c_void>());

    for (name, size) in g::STRUCT_SIZES {
        let measured = match *name {
            "lumio_handle_t" => std::mem::size_of::<LumioHandle>(),
            "lumio_buffer_t" => std::mem::size_of::<LumioBuffer>(),
            "lumio_core_api" => std::mem::size_of::<LumioCoreApi>(),
            "lumio_voxel_api" => std::mem::size_of::<LumioVoxelApi>(),
            "lumio_root_api" => std::mem::size_of::<LumioRootApi>(),
            other => panic!("Golden 出现未知结构 {other}"),
        };
        println!("size.{name}={measured}");
        assert_eq!(measured, *size, "{name} 的 size 与上游 Golden 不一致");
    }

    for (name, _) in g::STRUCT_SIZES {
        let measured = match *name {
            "lumio_handle_t" => std::mem::align_of::<LumioHandle>(),
            "lumio_buffer_t" => std::mem::align_of::<LumioBuffer>(),
            "lumio_core_api" => std::mem::align_of::<LumioCoreApi>(),
            "lumio_voxel_api" => std::mem::align_of::<LumioVoxelApi>(),
            "lumio_root_api" => std::mem::align_of::<LumioRootApi>(),
            other => panic!("Golden 出现未知结构 {other}"),
        };
        println!("align.{name}={measured}");
        assert_eq!(
            measured,
            g::MAX_ALIGNMENT,
            "{name} 的 align 与 layout profile 不一致"
        );
    }

    for (table, slot, offset) in g::SLOT_OFFSETS {
        let measured = match (*table, *slot) {
            ("lumio_core_api", "lumio_core_init") => {
                std::mem::offset_of!(LumioCoreApi, lumio_core_init)
            }
            ("lumio_core_api", "lumio_core_shutdown") => {
                std::mem::offset_of!(LumioCoreApi, lumio_core_shutdown)
            }
            ("lumio_core_api", "lumio_core_last_error_detail") => {
                std::mem::offset_of!(LumioCoreApi, lumio_core_last_error_detail)
            }
            ("lumio_voxel_api", "lumio_voxel_world_create") => {
                std::mem::offset_of!(LumioVoxelApi, lumio_voxel_world_create)
            }
            ("lumio_voxel_api", "lumio_voxel_world_destroy") => {
                std::mem::offset_of!(LumioVoxelApi, lumio_voxel_world_destroy)
            }
            other => panic!("Golden 出现未知 slot {other:?}"),
        };
        println!("offset.{table}.{slot}={measured}");
        assert_eq!(
            measured, *offset,
            "{table}.{slot} 的 offset 与上游 Golden 不一致"
        );
    }

    // Root Table 的 table 指针槽位：ADR-040 §4 的 `16 + i * pointerBytes`。
    for (index, (table, slot)) in [
        ("lumio_root_api", "lumio_core_api"),
        ("lumio_root_api", "lumio_voxel_api"),
    ]
    .into_iter()
    .enumerate()
    {
        let measured = match slot {
            "lumio_core_api" => std::mem::offset_of!(LumioRootApi, lumio_core_api),
            _ => std::mem::offset_of!(LumioRootApi, lumio_voxel_api),
        };
        println!("offset.{table}.{slot}={measured}");
        assert_eq!(measured, g::ROOT_HEADER_BYTES + index * g::POINTER_BYTES);
    }

    assert_eq!(std::mem::size_of::<*const c_void>(), g::POINTER_BYTES);
}
