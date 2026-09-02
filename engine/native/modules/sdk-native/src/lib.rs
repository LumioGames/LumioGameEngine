use std::ffi::{c_char, c_void};

mod abi_generated;
mod timer;

pub const SDK_ENTRY_SYMBOL: &str = abi_generated::ENTRY_SYMBOL;
pub const SDK_ABI_DEFINITION_SHA256: &str = abi_generated::DEFINITION_SHA256;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LumioStatus {
    Success = 0,
    InvalidArgument = 1,
    UnsupportedVersion = 2,
    ClrInitFailed = 3,
    ClrEntryFailed = 4,
    BufferTooSmall = 5,
    TimerStaleHandle = 6,
    TimerScopeInvalid = 7,
    TimerScopeGenerationMismatch = 8,
    TimerInvalidDueTick = 9,
    TimerInvalidInterval = 10,
    TimerScheduleBudgetExceeded = 11,
    TimerSlotClosed = 12,
    TimerSlotUnbound = 13,
    TimerSlotDispatchMismatch = 14,
    TimerSlotQueueFull = 15,
    TimerLateCompletion = 16,
    TimerManagerShutdown = 17,
}

/// Compile-time composition marker for the SDK's two provider domains.
pub fn composed_provider_names() -> (&'static str, &'static str) {
    let _ = std::any::TypeId::of::<lumio_kernel::handle::HandleKey>();
    let _ = std::any::TypeId::of::<lumio_voxel_world::world::VoxelWorld>();
    let _ = std::any::TypeId::of::<lumio_timer::TimerManager>();
    ("LumioNativeCore", "LumioVoxelEngine")
}

/// 根 API 表（engine/abi/native-abi.json 的 root.fields 是唯一真值）。
///
/// CLR host 三槽直接转发到 `lumio-clr-host` 的 ABI 实现——SDK 内唯一的 CLR 装载链，
/// 不引入第二套 Loader/ABI。timer_* 槽转发到 NativeCore `lumio-timer`（C-4 唯一内核）。
/// 槽位签名与 ABI 定义逐参一致；扩展槽位只追加不插入，struct_size 随 `size_of`
/// 自动更新，消费方按「≥ 自身布局」协商。
#[repr(C)]
pub struct LumioEngineRootApiV1 {
    pub abi_version: u32,
    pub struct_size: u32,
    pub abi_hash: [u8; 32],
    pub build_id: [u8; 16],
    pub ping: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub create_clr_host: Option<
        unsafe extern "C" fn(
            *const c_char,    // hostfxr_path（UTF-8 NUL 结尾）
            *const c_char,    // runtime_config_path
            *const c_char,    // assembly_path
            *const c_char,    // entry_type_name：'<程序集限定类型名>;<入口方法名>'
            *mut *mut c_void, // out opaque handle
        ) -> i32,
    >,
    pub clr_host_call: Option<
        unsafe extern "C" fn(
            *mut c_void, // host
            *const u8,   // input
            u32,         // input_len
            *mut u8,     // output
            u32,         // output_capacity
            *mut u32,    // out bytes_written
        ) -> i32,
    >,
    pub destroy_clr_host: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub timer_create_manager: Option<unsafe extern "C" fn(u32, *mut *mut c_void) -> i32>,
    pub timer_destroy_manager: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub timer_register_dispatch: Option<unsafe extern "C" fn(*mut c_void, u32) -> i32>,
    pub timer_register_scope: Option<unsafe extern "C" fn(*mut c_void, u64, u32, *mut u32) -> i32>,
    pub timer_teardown_scope: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
    pub timer_create_slot: Option<unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32>,
    pub timer_bind_slot: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> i32>,
    pub timer_close_slot: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32>,
    pub timer_schedule_one_shot: Option<
        unsafe extern "C" fn(*mut c_void, u64, u32, u32, u64, *mut c_void, *mut c_void) -> i32,
    >,
    pub timer_schedule_repeating: Option<
        unsafe extern "C" fn(*mut c_void, u64, u32, u32, u64, u64, *mut c_void, *mut c_void) -> i32,
    >,
    pub timer_cancel: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32>,
    pub timer_advance: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
    pub timer_pump: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
    pub timer_drain: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *mut u32) -> i32>,
}

const _: () = assert!(std::mem::size_of::<LumioEngineRootApiV1>() >= 200);

const DEFAULT_ABI_HASH: &str = abi_generated::DEFINITION_SHA256;
const DEFAULT_BUILD_ID: &str = "2222222222222222222222222222222222222222222222222222222222222222";

const ABI_HASH: &str = match option_env!("LUMIO_ABI_HASH") {
    Some(value) => value,
    None => DEFAULT_ABI_HASH,
};
const BUILD_ID: &str = match option_env!("LUMIO_BUILD_ID") {
    Some(value) => value,
    None => DEFAULT_BUILD_ID,
};

const fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}

const fn decode_hex<const N: usize>(value: &str) -> [u8; N] {
    let bytes = value.as_bytes();
    let mut output = [0; N];
    let mut index = 0;
    while index < N {
        let offset = index * 2;
        output[index] = (hex_nibble(bytes[offset]) << 4) | hex_nibble(bytes[offset + 1]);
        index += 1;
    }
    output
}

/// 根表 ping 槽：向 `marker` 写入 1 以证明原生侧可被调用。
///
/// # Safety
///
/// 调用方必须保证 `marker` 指向可写的 u32（或传 null 走拒绝路径）。
unsafe extern "C" fn ping(marker: *mut c_void) -> i32 {
    if marker.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }

    // SAFETY: the caller owns the marker pointer and the API only writes one u32.
    unsafe { marker.cast::<u32>().write(1) };
    LumioStatus::Success as i32
}

static ROOT_API: LumioEngineRootApiV1 = LumioEngineRootApiV1 {
    abi_version: abi_generated::ABI_VERSION,
    struct_size: std::mem::size_of::<LumioEngineRootApiV1>() as u32,
    abi_hash: decode_hex(ABI_HASH),
    build_id: decode_hex(BUILD_ID),
    ping: Some(ping),
    // 转发即接线：三个槽位直接指向 lumio-clr-host 的 ABI 实现（MS-00002 Wave 2）。
    create_clr_host: Some(lumio_clr_host::create_clr_host),
    clr_host_call: Some(lumio_clr_host::clr_host_call),
    destroy_clr_host: Some(lumio_clr_host::destroy_clr_host),
    // C-4 timer_*：转发 NativeCore lumio-timer，不另建内核。
    timer_create_manager: Some(timer::timer_create_manager),
    timer_destroy_manager: Some(timer::timer_destroy_manager),
    timer_register_dispatch: Some(timer::timer_register_dispatch),
    timer_register_scope: Some(timer::timer_register_scope),
    timer_teardown_scope: Some(timer::timer_teardown_scope),
    timer_create_slot: Some(timer::timer_create_slot),
    timer_bind_slot: Some(timer::timer_bind_slot),
    timer_close_slot: Some(timer::timer_close_slot),
    timer_schedule_one_shot: Some(timer::timer_schedule_one_shot),
    timer_schedule_repeating: Some(timer::timer_schedule_repeating),
    timer_cancel: Some(timer::timer_cancel),
    timer_advance: Some(timer::timer_advance),
    timer_pump: Some(timer::timer_pump),
    timer_drain: Some(timer::timer_drain),
};

/// The only Native SDK symbol. All other functions are reached through this table.
///
/// # Safety
///
/// 调用方必须保证 `out_api` 指向可写的 `*const LumioEngineRootApiV1`；返回的表指针
/// 具有静态存储期，与 DLL 卸载同生命周期。
#[no_mangle]
pub unsafe extern "C" fn lumio_engine_get_api_v1(
    requested_version: u32,
    out_api: *mut *const LumioEngineRootApiV1,
) -> i32 {
    if out_api.is_null() {
        return LumioStatus::InvalidArgument as i32;
    }
    if requested_version != abi_generated::ABI_VERSION {
        // SAFETY: null is checked above and the caller supplied writable storage.
        unsafe { out_api.write(std::ptr::null()) };
        return LumioStatus::UnsupportedVersion as i32;
    }

    // SAFETY: null is checked above and ROOT_API has static storage duration.
    unsafe { out_api.write(&ROOT_API) };
    LumioStatus::Success as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_ping_is_rejected_without_writing_memory() {
        assert_eq!(
            unsafe { ping(std::ptr::null_mut()) },
            LumioStatus::InvalidArgument as i32
        );
    }
}
