//! lumio-clr-host —— CoreCLR 装载链（MS-00002 Wave 2）。
//!
//! 职责：Rust 宿主经 hostfxr 把 CoreCLR 运行时拉起、把托管程序集的固定入口解析成
//! 原生函数指针，并以字节协议转发调用。本 crate 是 SDK 内唯一的 CLR 装载实现：
//! 上层只经 sdk-native 根表的三个槽位转发到本 crate 的 ABI 函数，全链不引入第二套
//! Loader/ABI（`engine/abi/native-abi.json` 是唯一定义真值，含各参数语义 doc）。
//!
//! # unsafe 纪律：为什么 crate 级是 `deny(unsafe_code)` 而不是 `forbid`
//!
//! 本 crate 的存在意义就是手写 hostfxr / kernel32 FFI（仓规：cargo 依赖零新增，不经
//! bindings 生成器），`unsafe` 不可避免，`forbid(unsafe_code)` 不适用于本 crate。
//! 采用的约束是把 `#![deny(unsafe_code)]` 放在 crate 根、再把豁免点**逐个**写在
//! 承载 FFI 的项上：
//! - FFI 调用全部位于 `unsafe fn` 体内（edition 2021 的隐式 unsafe 上下文），每处
//!   外部调用 / 指针解引用都有行内 SAFETY 注释或 `# Safety` 契约段；
//! - 新增任何 unsafe 必须显式再加一个豁免点，审查可见；
//! - `clippy::undocumented_unsafe_blocks` 提到 deny——无法声明为 unsafe fn 的位置
//!   （如 `Drop` 这类 trait 方法）只能用裸 unsafe 块，必须带 SAFETY 注释；
//! - 除三个 ABI 边界函数外不暴露裸指针；托管入口只在创建期解析一次。
//!
//! # hostfxr 事实依据（与 .NET 10 实际行为核对，端到端测试为准）
//!
//! - `hostfxr_delegate_type::hdt_load_assembly_and_get_function_pointer` 的枚举值是
//!   **5**（官方 hostfxr.h 按声明顺序：com_activation=0、load_in_memory_assembly=1、
//!   winrt_activation=2、com_register=3、com_unregister=4、
//!   load_assembly_and_get_function_pointer=5、get_function_pointer=6、
//!   load_assembly=7、load_assembly_bytes=8）。
//! - `hostfxr_initialize_for_runtime_config` 是**三参**签名：中间还有一个可选的
//!   `const hostfxr_initialize_parameters*`（本 crate 恒传 null）。按旧的两参原型
//!   声明会把出参错位成结构体指针、在 hostfxr 内部直接 AV——已实测复现并修正。
//! - `HOSTFXR_CALLTYPE` 在 Win32 上是 `__cdecl`（hostfxr 原型用 `extern "C"`）。
//! - 请求 `[UnmanagedCallersOnly]` 入口的原始指针时，`delegate_type_name` 传哨兵
//!   `UNMANAGEDCALLERSONLY_METHOD == (const char_t*)-1`（coreclr_delegates.h），
//!   不是字符串；传 null 会得到二参默认封送签名，与本 crate 的五参固定签名不符。
//! - `load_assembly_and_get_function_pointer` 的 `method_name` 按托管方法名解析；
//!   约定托管方法名与 `[UnmanagedCallersOnly(EntryPoint = ...)]` 同名，两种解析
//!   口径都指向同一入口。

#![deny(unsafe_code)]
#![deny(clippy::undocumented_unsafe_blocks)]

use std::ffi::{c_char, c_void, CStr};
use std::path::Path;

mod sys;

/// ABI status 值（`engine/abi/native-abi.json` 的 status 表；与 sdk-native 的
/// `LumioStatus` 数值同源同一份定义，不得单边漂移）。
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClrHostStatus {
    Success = 0,
    InvalidArgument = 1,
    ClrInitFailed = 3,
    ClrEntryFailed = 4,
    BufferTooSmall = 5,
}

/// 托管入口的固定原生签名（与 LumioGameRuntime 侧约定，不得偏离）：
/// `int32_t (*)(const uint8_t* input, int32_t input_length, uint8_t* output,
/// int32_t output_capacity, int32_t* bytes_written)`。
type ManagedEntryFn = unsafe extern "system" fn(*const u8, i32, *mut u8, i32, *mut i32) -> i32;

/// hostfxr 上下文句柄（hostfxr.h 的 opaque hostfxr_handle）。
type HostFxrHandle = *mut c_void;
/// 已装载的 hostfxr 模块句柄（Windows HMODULE）。
type ModuleHandle = *mut c_void;
/// `hostfxr_close` 原型（HOSTFXR_CALLTYPE 在 Win32 上是 `__cdecl`）；Drop 期按创建期
/// 捕获的指针调用。
type HostFxrCloseFn = unsafe extern "C" fn(HostFxrHandle) -> i32;

/// 一个装载成功的 CLR host：入口指针 + Drop 期需要释放的资源。
///
/// 字段只在 `sys::load_clr` 的成功路径填充，`Drop` 负责完整回滚（先关上下文再卸载
/// 模块），因此不存在半初始化实例；创建失败路径在 `sys::load_clr` 内部就地回滚。
struct LoadedClr {
    entry: ManagedEntryFn,
    close: HostFxrCloseFn,
    context: HostFxrHandle,
    module: ModuleHandle,
}

#[allow(unsafe_code)]
impl Drop for LoadedClr {
    fn drop(&mut self) {
        // SAFETY: context 与 module 由本实例独占持有（失败路径不会构造出本实例）；
        // 调用方经 destroy_clr_host 保证此时没有并发 clr_host_call。
        unsafe { (self.close)(self.context) };
        // SAFETY: hostfxr 上下文已关闭、模块上的函数指针不再被调用；模块来自
        // LoadLibraryW 且只在此处释放一次。
        unsafe { sys::free_module(self.module) };
    }
}

/// `clr_host_call` 的内部结果。
enum CallOutcome {
    Written(usize),
    BufferTooSmall(usize),
    EntryFailed,
}

impl LoadedClr {
    /// 转发一次字节协议调用并归一化托管入口的返回码。
    ///
    /// # Safety
    ///
    /// `self` 必须来自成功的 `sys::load_clr`；input/output 切片在调用期间有效。
    #[allow(unsafe_code)]
    unsafe fn call(&self, input: &[u8], output: &mut [u8]) -> CallOutcome {
        debug_assert!(input.len() <= i32::MAX as usize);
        debug_assert!(output.len() <= i32::MAX as usize);

        let mut written: i32 = 0;
        // SAFETY: entry 在创建期解析并验证非空，指向进程内运行时代码，调用期间稳定；
        // input/output 切片在本调用栈内存活；托管侧契约保证只在 capacity 内写 output、
        // 只写 bytes_written 指向的一个 int32。
        let status = (self.entry)(
            input.as_ptr(),
            input.len() as i32,
            output.as_mut_ptr(),
            output.len() as i32,
            &mut written,
        );
        match status {
            0 => {
                if written < 0 || written as usize > output.len() {
                    return CallOutcome::EntryFailed;
                }
                CallOutcome::Written(written as usize)
            }
            2 => CallOutcome::BufferTooSmall(written.max(0) as usize),
            _ => CallOutcome::EntryFailed,
        }
    }
}

/// 解析 entry_type_name：`'<程序集限定类型名>;<入口方法名>'`，按最后一个 `;` 拆分。
/// 程序集限定名只含逗号/空格/点，方法名同理，都不会出现 `;`，因此该拆分无歧义。
fn split_entry_spec(spec: &str) -> Result<(&str, &str), ()> {
    match spec.rsplit_once(';') {
        Some((type_name, method_name))
            if !type_name.trim().is_empty() && !method_name.trim().is_empty() =>
        {
            Ok((type_name, method_name))
        }
        _ => Err(()),
    }
}

/// ABI `create_clr_host`：创建 CLR host，并在创建期解析托管入口（fail-fast）。
///
/// # Safety
///
/// 调用方必须保证：四个字符串参数是 NUL 结尾、可读的 UTF-8；`out_handle` 指向可写的
/// `*mut c_void`；成功返回的句柄只能传给 `clr_host_call` / `destroy_clr_host` 且只
/// 释放一次。
#[allow(unsafe_code)]
pub unsafe extern "C" fn create_clr_host(
    hostfxr_path: *const c_char,
    runtime_config_path: *const c_char,
    assembly_path: *const c_char,
    entry_type_name: *const c_char,
    out_handle: *mut *mut c_void,
) -> i32 {
    if hostfxr_path.is_null()
        || runtime_config_path.is_null()
        || assembly_path.is_null()
        || entry_type_name.is_null()
        || out_handle.is_null()
    {
        return ClrHostStatus::InvalidArgument as i32;
    }

    // SAFETY: ABI 契约保证指针指向 NUL 结尾的合法 UTF-8；非法 UTF-8 按 InvalidArgument 拒绝。
    let (hostfxr, runtime_config, assembly, entry_spec) = match (
        CStr::from_ptr(hostfxr_path).to_str(),
        CStr::from_ptr(runtime_config_path).to_str(),
        CStr::from_ptr(assembly_path).to_str(),
        CStr::from_ptr(entry_type_name).to_str(),
    ) {
        (Ok(hostfxr), Ok(runtime_config), Ok(assembly), Ok(entry_spec)) => {
            (hostfxr, runtime_config, assembly, entry_spec)
        }
        _ => return ClrHostStatus::InvalidArgument as i32,
    };

    // 入口描述先于任何装载动作解析：非法描述不得触碰 hostfxr / 运行时。
    let (type_name, method_name) = match split_entry_spec(entry_spec) {
        Ok(parts) => parts,
        Err(()) => return ClrHostStatus::InvalidArgument as i32,
    };

    // SAFETY: 全部入参已在上方校验；失败路径由 load_clr 内部完整回滚。
    match sys::load_clr(
        Path::new(hostfxr),
        Path::new(runtime_config),
        Path::new(assembly),
        type_name,
        method_name,
    ) {
        Ok(loaded) => {
            // SAFETY: out_handle 已判空且由调用方保证可写；Box 故意泄漏——句柄所有权
            // 移交调用方，destroy_clr_host 经 Box::from_raw 收回。
            *out_handle = Box::into_raw(Box::new(loaded)).cast::<c_void>();
            ClrHostStatus::Success as i32
        }
        Err(sys::LoadError::InitFailed) => ClrHostStatus::ClrInitFailed as i32,
    }
}

/// ABI `clr_host_call`：把一次字节协议调用转发进托管入口。
///
/// # Safety
///
/// 调用方必须保证：`host` 是 `create_clr_host` 返回、尚未销毁的句柄且本次调用不与
/// 销毁并发；input/output 缓冲在本调用期间有效；`bytes_written` 指向可写的 u32。
#[allow(unsafe_code)]
pub unsafe extern "C" fn clr_host_call(
    host: *mut c_void,
    input: *const u8,
    input_len: u32,
    output: *mut u8,
    output_capacity: u32,
    bytes_written: *mut u32,
) -> i32 {
    if host.is_null() || bytes_written.is_null() {
        return ClrHostStatus::InvalidArgument as i32;
    }
    if input_len > 0 && input.is_null() {
        return ClrHostStatus::InvalidArgument as i32;
    }
    if output_capacity > 0 && output.is_null() {
        return ClrHostStatus::InvalidArgument as i32;
    }
    if input_len > i32::MAX as u32 || output_capacity > i32::MAX as u32 {
        return ClrHostStatus::InvalidArgument as i32;
    }

    let input_bytes: &[u8] = if input_len == 0 {
        &[]
    } else {
        // SAFETY: input 非空（len>0 已判空）且 ABI 契约保证 input_len 字节可读。
        std::slice::from_raw_parts(input, input_len as usize)
    };
    let output_bytes: &mut [u8] = if output_capacity == 0 {
        &mut []
    } else {
        // SAFETY: output 非空（capacity>0 已判空）且调用方保证容量内可写、无别名。
        std::slice::from_raw_parts_mut(output, output_capacity as usize)
    };

    // SAFETY: host 来自 create_clr_host 的 Box::into_raw 且尚未销毁（ABI 契约）；
    // 这里只取共享引用，不接管所有权。
    let loaded = &*host.cast::<LoadedClr>();

    // SAFETY: loaded 来自成功装载；两个切片在本调用栈内存活。
    match loaded.call(input_bytes, output_bytes) {
        CallOutcome::Written(count) => {
            // SAFETY: bytes_written 已判空且由调用方保证可写。
            *bytes_written = count as u32;
            ClrHostStatus::Success as i32
        }
        CallOutcome::BufferTooSmall(required) => {
            // SAFETY: 同上；所需长度来自托管入口写入的 int32，必在 u32 范围内。
            *bytes_written = required as u32;
            ClrHostStatus::BufferTooSmall as i32
        }
        CallOutcome::EntryFailed => {
            // SAFETY: 同上；失败路径确定性写 0。
            *bytes_written = 0;
            ClrHostStatus::ClrEntryFailed as i32
        }
    }
}

/// ABI `destroy_clr_host`：销毁句柄并释放全部底层资源。
///
/// # Safety
///
/// 调用方必须保证 `host` 来自 `create_clr_host`、未被重复销毁，且销毁时没有并发
/// `clr_host_call`。
#[allow(unsafe_code)]
pub unsafe extern "C" fn destroy_clr_host(host: *mut c_void) -> i32 {
    if host.is_null() {
        return ClrHostStatus::InvalidArgument as i32;
    }
    // SAFETY: host 是 create_clr_host 泄漏的 Box 指针且只此一次回收（ABI 契约）；
    // Drop 会先 hostfxr_close 再卸载 hostfxr 模块。
    drop(Box::from_raw(host.cast::<LoadedClr>()));
    ClrHostStatus::Success as i32
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn text(value: &str) -> CString {
        CString::new(value).unwrap()
    }

    #[test]
    fn create_rejects_each_null_argument() {
        let value = text("value");
        let value_ptr = value.as_ptr();
        let mut handle = std::ptr::null_mut();
        let expected = ClrHostStatus::InvalidArgument as i32;

        // SAFETY: 测试只传 null 或合法 CString；出参是栈上可写变量。
        let status = unsafe {
            create_clr_host(
                std::ptr::null(),
                value_ptr,
                value_ptr,
                value_ptr,
                &mut handle,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: 同上，仅第二个参数为 null。
        let status = unsafe {
            create_clr_host(
                value_ptr,
                std::ptr::null(),
                value_ptr,
                value_ptr,
                &mut handle,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: 同上，仅第三个参数为 null。
        let status = unsafe {
            create_clr_host(
                value_ptr,
                value_ptr,
                std::ptr::null(),
                value_ptr,
                &mut handle,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: 同上，仅第四个参数为 null。
        let status = unsafe {
            create_clr_host(
                value_ptr,
                value_ptr,
                value_ptr,
                std::ptr::null(),
                &mut handle,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: 同上，仅出参为 null。
        let status = unsafe {
            create_clr_host(
                value_ptr,
                value_ptr,
                value_ptr,
                value_ptr,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(status, expected);
    }

    #[test]
    fn create_rejects_malformed_entry_spec_before_touching_the_host() {
        let hostfxr = text("Z:/definitely/missing/hostfxr.dll");
        let config = text("Z:/missing/runtimeconfig.json");
        let assembly = text("Z:/missing/assembly.dll");
        let mut handle = std::ptr::null_mut();

        for spec in [
            "NoSeparator",
            "",
            ";method",
            "Type, Assembly; ",
            "Type, Assembly;;",
        ] {
            let spec = text(spec);
            // SAFETY: 均为合法 CString；entry_type_name 非法必须在装载任何东西之前拒绝。
            let status = unsafe {
                create_clr_host(
                    hostfxr.as_ptr(),
                    config.as_ptr(),
                    assembly.as_ptr(),
                    spec.as_ptr(),
                    &mut handle,
                )
            };
            assert_eq!(
                status,
                ClrHostStatus::InvalidArgument as i32,
                "entry spec {spec:?} 必须被判为非法"
            );
        }
        assert!(handle.is_null());
    }

    #[test]
    fn call_rejects_null_host_and_null_out_param() {
        let input = b"abc".as_ptr();
        let mut output = [0_u8; 8];
        let mut written = 0_u32;
        let dangling = std::ptr::NonNull::<LoadedClr>::dangling()
            .as_ptr()
            .cast::<c_void>();
        let expected = ClrHostStatus::InvalidArgument as i32;

        // SAFETY: 只走参数校验分支，null host 不会被解引用。
        let status = unsafe {
            clr_host_call(
                std::ptr::null_mut(),
                input,
                3,
                output.as_mut_ptr(),
                8,
                &mut written,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: host 非 null 但 bytes_written 为 null，须在解引用 host 前拒绝。
        let status = unsafe {
            clr_host_call(
                dangling,
                input,
                3,
                output.as_mut_ptr(),
                8,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(status, expected);
    }

    #[test]
    fn call_rejects_null_buffers_with_nonzero_lengths() {
        let input = b"abc".as_ptr();
        let mut output = [0_u8; 8];
        let mut written = 0_u32;
        let dangling = std::ptr::NonNull::<LoadedClr>::dangling()
            .as_ptr()
            .cast::<c_void>();
        let expected = ClrHostStatus::InvalidArgument as i32;

        // SAFETY: 校验失败分支不会解引用 dangling host，也不会触碰缓冲指针。
        let status = unsafe {
            clr_host_call(
                dangling,
                std::ptr::null(),
                3,
                output.as_mut_ptr(),
                8,
                &mut written,
            )
        };
        assert_eq!(status, expected);
        // SAFETY: 同上。
        let status =
            unsafe { clr_host_call(dangling, input, 3, std::ptr::null_mut(), 8, &mut written) };
        assert_eq!(status, expected);
    }

    #[test]
    fn destroy_rejects_a_null_host() {
        // SAFETY: null 检查路径不触碰任何内存。
        let status = unsafe { destroy_clr_host(std::ptr::null_mut()) };
        assert_eq!(status, ClrHostStatus::InvalidArgument as i32);
    }

    #[test]
    fn entry_spec_split_rules() {
        assert_eq!(
            split_entry_spec("Type, Asm;method"),
            Ok(("Type, Asm", "method"))
        );
        assert!(split_entry_spec("NoSeparator").is_err());
        assert!(split_entry_spec("Type, Asm;").is_err());
        assert!(split_entry_spec(";method").is_err());
    }
}
