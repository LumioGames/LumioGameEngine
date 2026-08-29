//! symbol、版本、大小、能力、布局负向矩阵（规格 §8.2 `tests/bind_invalid.rs`）。
//!
//! 每条负例都断言**稳定公共 ErrorCode**（规格 §6.2 / §8.3 错误映射），
//! 不断言消息文本——文本是仓内诊断，码值才是契约。

use std::cell::Cell;
use std::ffi::{c_void, CStr};
use std::ptr::NonNull;
use std::sync::Arc;

use lumio_core_contracts::ErrorCode;
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

fn leak_core() -> *const LumioCoreApi {
    Box::leak(Box::new(LumioCoreApi {
        version: 1,
        struct_size: 48,
        reserved0: 0,
        lumio_core_init: Some(core_init),
        lumio_core_shutdown: Some(core_shutdown),
        lumio_core_last_error_detail: Some(core_last_error_detail),
        reserved: [std::ptr::null_mut()],
    }))
}

fn leak_voxel() -> *const LumioVoxelApi {
    Box::leak(Box::new(LumioVoxelApi {
        version: 1,
        struct_size: 32,
        reserved0: 0,
        lumio_voxel_world_create: Some(voxel_create),
        lumio_voxel_world_destroy: Some(voxel_destroy),
    }))
}

fn leak_root() -> *mut LumioRootApi {
    Box::leak(Box::new(LumioRootApi {
        abi_version: 1,
        struct_size: 64,
        capability_bits: 7,
        lumio_core_api: leak_core(),
        lumio_voxel_api: leak_voxel(),
        reserved_tail: [0u8; 32],
    }))
}

thread_local! {
    static TABLE: Cell<*const LumioRootApi> = const { Cell::new(std::ptr::null()) };
    static STATUS: Cell<LumioStatus> = const { Cell::new(0) };
}

extern "C" fn entry(_requested_version: u32, out_table: *mut *const LumioRootApi) -> LumioStatus {
    let status = STATUS.with(|s| s.get());
    if status != 0 {
        return status;
    }
    // SAFETY: 调用方（bind_root_api）传入本地变量地址，非空且对齐。
    unsafe { *out_table = TABLE.with(|t| t.get()) };
    0
}

enum Resolve {
    Entry,
    Missing,
    Collision,
}

struct FakeResolver(Resolve);

impl SymbolResolver for FakeResolver {
    unsafe fn resolve(&self, _symbol: &CStr) -> Result<NonNull<c_void>, SymbolLookupError> {
        match self.0 {
            Resolve::Entry => Ok(NonNull::new(entry as *const () as *mut c_void).expect("非空")),
            Resolve::Missing => Err(SymbolLookupError::NotFound),
            Resolve::Collision => Err(SymbolLookupError::Collision),
        }
    }
}

/// 用给定的 table 与 resolve 行为跑一次绑定，返回失败时的稳定 ErrorCode。
fn bind_expecting_failure(
    resolve: Resolve,
    table: *const LumioRootApi,
    status: LumioStatus,
) -> ErrorCode {
    TABLE.with(|t| t.set(table));
    STATUS.with(|s| s.set(status));
    let expected = AbiExpectation::from_generated_contract();
    // SAFETY: resolver 要么报错，要么返回本进程内 `entry` 的地址。
    let result = unsafe { bind_root_api(Arc::new(FakeResolver(resolve)), &expected) };
    result.expect_err("本例必须失败").code()
}

fn bind_table_expecting_failure(table: *const LumioRootApi) -> ErrorCode {
    bind_expecting_failure(Resolve::Entry, table, 0)
}

// ── symbol 面 ───────────────────────────────────────────────────────────

#[test]
fn missing_entry_symbol_maps_to_symbol_missing_1021() {
    let code = bind_expecting_failure(Resolve::Missing, leak_root(), 0);

    assert_eq!(code, ErrorCode::SymbolMissing);
    assert_eq!(code as i32, 1021);
}

#[test]
fn colliding_entry_symbol_maps_to_symbol_collision_1022() {
    let code = bind_expecting_failure(Resolve::Collision, leak_root(), 0);

    assert_eq!(code, ErrorCode::SymbolCollision);
    assert_eq!(code as i32, 1022);
}

// ── entry 调用面 ────────────────────────────────────────────────────────

#[test]
fn entry_returning_registered_error_code_is_passed_through() {
    let code = bind_expecting_failure(
        Resolve::Entry,
        leak_root(),
        ErrorCode::CapabilityMissing as i32,
    );

    assert_eq!(code, ErrorCode::CapabilityMissing);
    assert_eq!(code as i32, 1020);
}

#[test]
fn entry_returning_unregistered_status_maps_to_native_abi_mismatch_1004() {
    let code = bind_expecting_failure(Resolve::Entry, leak_root(), 4242);

    assert_eq!(code, ErrorCode::NativeAbiMismatch);
    assert_eq!(code as i32, 1004);
}

#[test]
fn null_out_table_maps_to_native_abi_mismatch_1004() {
    let code = bind_table_expecting_failure(std::ptr::null());

    assert_eq!(code, ErrorCode::NativeAbiMismatch);
}

#[test]
fn misaligned_root_pointer_maps_to_native_abi_mismatch_1004() {
    // 故意制造未对齐地址；bind 必须在任何读取之前就拒绝，不得触发 UB。
    let backing: &'static mut [u8; 128] = Box::leak(Box::new([0u8; 128]));
    let misaligned = unsafe { backing.as_mut_ptr().add(1) } as *const LumioRootApi;

    let code = bind_table_expecting_failure(misaligned);

    assert_eq!(code, ErrorCode::NativeAbiMismatch);
}

// ── root header 面 ──────────────────────────────────────────────────────

#[test]
fn root_abi_version_mismatch_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: `root` 由 Box::leak 产生，仍然存活且独占。
    unsafe { (*root).abi_version = 2 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn root_struct_size_below_minimum_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。24 < 派生最小值 32。
    unsafe { (*root).struct_size = 24 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn root_struct_size_not_aligned_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。36 不是 maxAlignment=8 的整数倍。
    unsafe { (*root).struct_size = 36 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn capability_bits_mismatch_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。capability_bits 是不透明 u64（ADR-040：位语义未冻结），
    // 只做精确相等比较，不做子集判定。
    unsafe { (*root).capability_bits = 3 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

// ── API table 面 ────────────────────────────────────────────────────────

#[test]
fn null_core_table_pointer_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。
    unsafe { (*root).lumio_core_api = std::ptr::null() };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn null_voxel_table_pointer_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。
    unsafe { (*root).lumio_voxel_api = std::ptr::null() };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn core_table_struct_size_below_minimum_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。40 < lumio_core_api 的派生最小值 48（含 1 个保留 slot）。
    unsafe { (*((*root).lumio_core_api as *mut LumioCoreApi)).struct_size = 40 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

/// **已知缺口的守卫测试**（见 crate 文档「与 §8.3 的两处偏差」第 2 条）。
///
/// per-table `version` 的期望值只发布在 `metadata/native-managed-abi.json` 与
/// bundle JSON 里，没有任何 Rust 可消费常量，因此绑定期**不做**版本比较——
/// 读出即如实公开。这条测试把「不比较」钉成显式行为：将来若要开始比较，
/// 必须先有可消费的上游真值并改这条测试，不能顺手加个字面量 `1` 就算数。
#[test]
fn table_version_is_surfaced_verbatim_and_not_asserted() {
    let root = leak_root();
    // SAFETY: 同上；`lumio_core_api` 指向本测试泄漏的独占 table。
    unsafe { (*((*root).lumio_core_api as *mut LumioCoreApi)).version = 7 };

    TABLE.with(|t| t.set(root));
    STATUS.with(|s| s.set(0));
    let expected = AbiExpectation::from_generated_contract();
    // SAFETY: resolver 返回本进程内 `entry` 的地址。
    let view = unsafe { bind_root_api(Arc::new(FakeResolver(Resolve::Entry)), &expected) }
        .expect("per-table version 当前不参与绑定判定");

    assert_eq!(view.generated_tables().core_api().version(), 7);
}

#[test]
fn null_core_slot_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。
    unsafe { (*((*root).lumio_core_api as *mut LumioCoreApi)).lumio_core_shutdown = None };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

#[test]
fn null_voxel_slot_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。
    unsafe { (*((*root).lumio_voxel_api as *mut LumioVoxelApi)).lumio_voxel_world_destroy = None };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}

/// root 声明的 `struct_size` 必须覆盖它自己声明的 table 指针槽位；
/// 否则读取 offset 16/24 就是越界读。
#[test]
fn root_struct_size_not_covering_table_pointers_maps_to_native_abi_mismatch_1004() {
    let root = leak_root();
    // SAFETY: 同上。16 是合法的 8 的倍数，但放不下两个 table 指针（需要 32）。
    unsafe { (*root).struct_size = 16 };

    assert_eq!(
        bind_table_expecting_failure(root),
        ErrorCode::NativeAbiMismatch
    );
}
