//! 平台装载实现：kernel32 动态符号解析 + hostfxr 原型（hostfxr.h / coreclr_delegates.h）。
//!
//! MS-00002 Wave 2 只实现 Windows（hostfxr 的 `char_t` 在 Windows 上是 UTF-16，路径与
//! 名称入参一律在 Rust 侧转宽字符）；Unix 的 dlopen 路径未接入，见交付 known gaps。
//! 本模块内的全部 FFI 调用都位于 `unsafe fn` 体内（edition 2021 隐式 unsafe 上下文），
//! 逐调用写行内 SAFETY 注释。

/// 装载失败（ABI 映射为 ClrInitFailed；细因见 `load_clr` 各失败分支的注释）。
pub(super) enum LoadError {
    InitFailed,
}

/// `hostfxr_delegate_type::hdt_load_assembly_and_get_function_pointer` 的枚举值。
///
/// 官方 hostfxr.h（dotnet/runtime，main 与 release/10.0 同序）按声明顺序：
/// com_activation=0、load_in_memory_assembly=1、winrt_activation=2、com_register=3、
/// com_unregister=4、**load_assembly_and_get_function_pointer=5**、
/// get_function_pointer=6、load_assembly=7、load_assembly_bytes=8。
/// 注意：既不是 3 也不是 2——以本常量为准，并由真实 fixture 的端到端测试验证。
const HDT_LOAD_ASSEMBLY_AND_GET_FUNCTION_POINTER: u32 = 5;

#[cfg(windows)]
mod windows_host {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use super::LoadError;
    use crate::{HostFxrCloseFn, HostFxrHandle, LoadedClr, ManagedEntryFn, ModuleHandle};

    /// coreclr_delegates.h：`#define UNMANAGEDCALLERSONLY_METHOD ((const char_t*)-1)`。
    /// 请求 `[UnmanagedCallersOnly]` 方法的原始入口指针时传该哨兵（而非字符串）；
    /// 传 null 会得到二参默认封送签名，与 SDK 的五参固定签名不符。
    const UNMANAGEDCALLERSONLY_METHOD: *const u16 = usize::MAX as *const u16;

    /// hostfxr.h：`const struct hostfxr_initialize_parameters*`（可选，本 crate 恒传 null）。
    /// 该参数是 .NET 5+ 新增——按两参原型声明会把出参错位成结构体指针，
    /// 实测在 hostfxr_initialize_for_runtime_config 内部直接 AV（见交付记录）。
    type HostFxrInitializeParameters = *const c_void;

    /// `hostfxr_initialize_for_runtime_config`（hostfxr.h；HOSTFXR_CALLTYPE 在 Win32 上是
    /// `__cdecl`，因此用 extern "C" 而不是 "system"）。
    type InitializeForRuntimeConfigFn = unsafe extern "C" fn(
        *const u16, // runtime_config_path
        HostFxrInitializeParameters,
        *mut HostFxrHandle, // out host_context_handle
    ) -> i32;
    /// `hostfxr_get_runtime_delegate`（hostfxr.h；同为 __cdecl）。
    type GetRuntimeDelegateFn = unsafe extern "C" fn(HostFxrHandle, u32, *mut *mut c_void) -> i32;
    /// `load_assembly_and_get_function_pointer`（coreclr_delegates.h；
    /// CORECLR_DELEGATE_CALLTYPE 在 Win32 上是 __stdcall，x64 下与 "C" 同构）。
    type LoadAssemblyAndGetFunctionPointerFn = unsafe extern "C" fn(
        *const u16,       // assembly_path
        *const u16,       // type_name（程序集限定名）
        *const u16,       // method_name（托管方法名）
        *const u16,       // delegate_type_name：null 或 UNMANAGEDCALLERSONLY_METHOD 哨兵
        *mut c_void,      // reserved：头文件要求必须为 0
        *mut *mut c_void, // out delegate
    ) -> i32;

    #[allow(unsafe_code)]
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }

    /// 已装载的 hostfxr 模块与其必需入口。
    struct HostFxrApi {
        module: ModuleHandle,
        initialize_for_runtime_config: InitializeForRuntimeConfigFn,
        get_runtime_delegate: GetRuntimeDelegateFn,
        close: HostFxrCloseFn,
    }

    /// 把路径转为 NUL 结尾 UTF-16（hostfxr 在 Windows 上的 `char_t`）。
    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// 把 `&str` 转为 NUL 结尾 UTF-16。
    fn wide_string(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    impl HostFxrApi {
        /// 装载 hostfxr 并解析三个必需入口；任一步失败即卸载模块并返回 None
        /// （hostfxr 路径不存在 / 不是有效 PE / 缺任一必需导出）。
        ///
        /// # Safety
        ///
        /// 返回的函数指针只在模块存活期间有效；模块所有权随返回值移交调用方，
        /// 调用方负责最终经 [`free_module`] 释放。
        #[allow(unsafe_code)]
        unsafe fn load(path: &Path) -> Option<HostFxrApi> {
            let path_wide = wide(path);
            // SAFETY: path_wide 是本栈上合法的 NUL 结尾 UTF-16，LoadLibraryW 只读它。
            let module = LoadLibraryW(path_wide.as_ptr());
            if module.is_null() {
                return None;
            }

            // SAFETY: module 刚由 LoadLibraryW 返回且未释放；名字是编译期 NUL 结尾字面量。
            let initialize = GetProcAddress(
                module,
                c"hostfxr_initialize_for_runtime_config"
                    .as_ptr()
                    .cast::<u8>(),
            );
            // SAFETY: 同上。
            let delegate = GetProcAddress(
                module,
                c"hostfxr_get_runtime_delegate".as_ptr().cast::<u8>(),
            );
            // SAFETY: 同上。
            let close = GetProcAddress(module, c"hostfxr_close".as_ptr().cast::<u8>());

            if initialize.is_null() || delegate.is_null() || close.is_null() {
                // SAFETY: module 由本次装载持有且其入口尚未分发使用。
                FreeLibrary(module);
                return None;
            }

            Some(HostFxrApi {
                module,
                // SAFETY: 指针取自当前模块导出表且已判空；原型与 hostfxr.h 一致。
                initialize_for_runtime_config: std::mem::transmute::<
                    *mut c_void,
                    InitializeForRuntimeConfigFn,
                >(initialize),
                // SAFETY: 同上。
                get_runtime_delegate: std::mem::transmute::<*mut c_void, GetRuntimeDelegateFn>(
                    delegate,
                ),
                // SAFETY: 同上。
                close: std::mem::transmute::<*mut c_void, HostFxrCloseFn>(close),
            })
        }
    }

    /// 完整装载链：hostfxr → runtime config → delegate → UnmanagedCallersOnly 入口。
    /// 任何一步失败都完整回滚（先 hostfxr_close 再 FreeLibrary），不留下半初始化状态。
    ///
    /// # Safety
    ///
    /// 路径与字符串只在调用期间读取；成功时全部资源所有权随返回的 [`LoadedClr`]
    /// 移交（其 Drop 负责释放），失败时本函数内部已全部回滚。
    #[allow(unsafe_code)]
    pub(crate) unsafe fn load_clr(
        hostfxr_path: &Path,
        runtime_config_path: &Path,
        assembly_path: &Path,
        type_name: &str,
        method_name: &str,
    ) -> Result<LoadedClr, LoadError> {
        // SAFETY: hostfxr_path 在调用期间有效；失败时 load 内部已回滚。
        let Some(hostfxr) = HostFxrApi::load(hostfxr_path) else {
            return Err(LoadError::InitFailed);
        };

        let runtime_config_wide = wide(runtime_config_path);
        let mut context: HostFxrHandle = std::ptr::null_mut();
        // SAFETY: runtime_config_wide 是本栈上 NUL 结尾 UTF-16；parameters 按头文件可选
        // 传 null；context 为有效出参。
        let status = (hostfxr.initialize_for_runtime_config)(
            runtime_config_wide.as_ptr(),
            std::ptr::null(),
            &mut context,
        );
        if status != 0 || context.is_null() {
            // SAFETY: initialize 失败时契约不产生需调用方关闭的上下文；模块仅本函数持有。
            // 细因：runtimeconfig 不存在 / 非法 JSON / 所需运行时不可用。
            free_module(hostfxr.module);
            return Err(LoadError::InitFailed);
        }

        let mut delegate: *mut c_void = std::ptr::null_mut();
        // SAFETY: context 来自成功的 initialize；delegate 为有效出参；枚举值 5 的依据见常量注释。
        let status = (hostfxr.get_runtime_delegate)(
            context,
            super::HDT_LOAD_ASSEMBLY_AND_GET_FUNCTION_POINTER,
            &mut delegate,
        );
        if status != 0 || delegate.is_null() {
            // SAFETY: context 与模块均由本路径持有；先关上下文再卸载模块，完整回滚。
            (hostfxr.close)(context);
            free_module(hostfxr.module);
            return Err(LoadError::InitFailed);
        }

        // SAFETY: delegate 来自 hdt_load_assembly_and_get_function_pointer；原型与
        // coreclr_delegates.h 一致。
        let load_assembly: LoadAssemblyAndGetFunctionPointerFn = std::mem::transmute(delegate);

        let assembly_wide = wide(assembly_path);
        let type_wide = wide_string(type_name);
        let method_wide = wide_string(method_name);
        let mut entry: *mut c_void = std::ptr::null_mut();
        // SAFETY: 三个宽字符串均 NUL 结尾；delegate_type_name 传 UNMANAGEDCALLERSONLY_METHOD
        // 哨兵（coreclr_delegates.h）；reserved 按头文件要求为 0；entry 为有效出参。
        let status = load_assembly(
            assembly_wide.as_ptr(),
            type_wide.as_ptr(),
            method_wide.as_ptr(),
            UNMANAGEDCALLERSONLY_METHOD,
            std::ptr::null_mut(),
            &mut entry,
        );
        if status != 0 || entry.is_null() {
            // SAFETY: 同 delegate 失败分支——完整回滚，不留半初始化。
            // 细因：类型/方法名与程序集不符，或方法未标注 UnmanagedCallersOnly。
            (hostfxr.close)(context);
            free_module(hostfxr.module);
            return Err(LoadError::InitFailed);
        }

        // SAFETY: entry 是 UnmanagedCallersOnly 方法的原始入口，签名与 ManagedEntryFn
        // 约定一致（五参 int32 返回）。
        let entry_fn: ManagedEntryFn = std::mem::transmute(entry);

        Ok(LoadedClr {
            entry: entry_fn,
            close: hostfxr.close,
            context,
            module: hostfxr.module,
        })
    }

    /// 释放 hostfxr 模块句柄。
    ///
    /// # Safety
    ///
    /// 句柄必须来自 `LoadLibraryW`、尚未释放，且调用后不再使用其任何函数指针。
    #[allow(unsafe_code)]
    pub(crate) unsafe fn free_module(module: ModuleHandle) {
        // SAFETY: 前置条件由调用方（失败回滚路径或 LoadedClr::drop）保证。
        FreeLibrary(module);
    }
}

#[cfg(windows)]
pub(super) use windows_host::{free_module, load_clr};

#[cfg(not(windows))]
mod unsupported_host {
    use std::path::Path;

    use super::LoadError;
    use crate::{LoadedClr, ModuleHandle};

    /// Unix 装载链未接入（MS-00002 Wave 2 known gap）；保留同签名以便上层无 cfg 分支。
    ///
    /// # Safety
    ///
    /// 与 Windows 版语义一致；当前实现无副作用，恒定返回 InitFailed。
    #[allow(unsafe_code)]
    pub(crate) unsafe fn load_clr(
        _hostfxr_path: &Path,
        _runtime_config_path: &Path,
        _assembly_path: &Path,
        _type_name: &str,
        _method_name: &str,
    ) -> Result<LoadedClr, LoadError> {
        Err(LoadError::InitFailed)
    }

    /// Unix 侧占位：LoadedClr 在该平台无法构造，本函数实际不可达。
    #[allow(unsafe_code)]
    #[allow(dead_code)]
    pub(crate) unsafe fn free_module(_module: ModuleHandle) {}
}

#[cfg(not(windows))]
pub(super) use unsupported_host::{free_module, load_clr};
