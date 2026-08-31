//! 真实 CoreCLR 装载链测试（仅 Windows；需要本机 dotnet SDK 构建夹具程序集）。
//!
//! 不 mock delegate：经 `dotnet build` 产出真实组件，用本机 hostfxr 走完整
//! create → call → destroy 链路，是 clr-host 契约的端到端证据。

#![cfg(windows)]

use std::ffi::{c_void, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use lumio_clr_host::{clr_host_call, create_clr_host, destroy_clr_host, ClrHostStatus};

/// 夹具入口：类型名 + ';' + **托管方法名**。方法名（LumioFixtureEntry）刻意不同于
/// UnmanagedCallersOnly 的 EntryPoint 别名（lumio_fixture_entry）：实测 hostfxr 的
/// load_assembly_and_get_function_pointer 按托管方法名解析，传别名返回 0x80131513
/// （MissingMethod）→ ClrInitFailed。双名夹具把该语义钉进本测试。
const FIXTURE_ENTRY_SPEC: &str =
    "LumioClrHostFixture.FixtureEntry, LumioClrHostFixture;LumioFixtureEntry";

/// 反例探针：按 EntryPoint 别名请求必须失败（MissingMethod → ClrInitFailed，不产句柄）。
const FIXTURE_ALIAS_ONLY_SPEC: &str =
    "LumioClrHostFixture.FixtureEntry, LumioClrHostFixture;lumio_fixture_entry";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_project() -> PathBuf {
    manifest_dir().join("testfixture/LumioClrHostFixture/LumioClrHostFixture.csproj")
}

/// 构建夹具（进程内只构建一次）并返回其 bin 目录。
fn build_fixture() -> PathBuf {
    static FIXTURE_DIR: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE_DIR
        .get_or_init(|| {
            let output = Command::new("dotnet")
                .arg("build")
                .arg(fixture_project())
                .args(["-v", "quiet", "-nologo"])
                .output()
                .expect("无法启动 dotnet；本测试需要本机 dotnet SDK（10.0.111）");
            if !output.status.success() {
                panic!(
                    "dotnet build 夹具失败（{}）：\n{}{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
            }

            let dir = fixture_project()
                .parent()
                .unwrap()
                .join("bin/Debug/net10.0");
            let assembly = dir.join("LumioClrHostFixture.dll");
            let runtime_config = dir.join("LumioClrHostFixture.runtimeconfig.json");
            assert!(
                assembly.is_file(),
                "夹具程序集缺失：{}（EnableDynamicLoading/GenerateRuntimeConfigurationFiles 未生效？）",
                assembly.display()
            );
            assert!(
                runtime_config.is_file(),
                "夹具 runtimeconfig 缺失：{}",
                runtime_config.display()
            );
            dir
        })
        .clone()
}

/// 定位本机 hostfxr.dll：优先 LUMIO_CLRHOST_TEST_HOSTFXR，其次 DOTNET_ROOT 与
/// %USERPROFILE%\.dotnet 下的 host/fxr/<version>/hostfxr.dll（取最高版本）。
fn locate_hostfxr() -> PathBuf {
    if let Ok(path) = std::env::var("LUMIO_CLRHOST_TEST_HOSTFXR") {
        let path = PathBuf::from(path);
        assert!(
            path.is_file(),
            "LUMIO_CLRHOST_TEST_HOSTFXR 指向的文件不存在：{path:?}"
        );
        return path;
    }

    let mut roots = Vec::new();
    if let Ok(dotnet_root) = std::env::var("DOTNET_ROOT") {
        roots.push(PathBuf::from(dotnet_root).join("host").join("fxr"));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        roots.push(
            PathBuf::from(profile)
                .join(".dotnet")
                .join("host")
                .join("fxr"),
        );
    }

    let mut best: Option<(Vec<u64>, PathBuf)> = None;
    for fxr_root in roots {
        let Ok(entries) = fs::read_dir(&fxr_root) else {
            continue;
        };
        for entry in entries.flatten() {
            let candidate = entry.path().join("hostfxr.dll");
            if !candidate.is_file() {
                continue;
            }
            let Some(version) = entry.file_name().to_str().and_then(parse_version) else {
                continue;
            };
            if best.as_ref().is_none_or(|(current, _)| *current < version) {
                best = Some((version, candidate));
            }
        }
    }

    best.map(|(_, path)| path)
        .unwrap_or_else(|| panic!("找不到 hostfxr.dll：设置 LUMIO_CLRHOST_TEST_HOSTFXR 或把 dotnet 装到 %USERPROFILE%\\.dotnet"))
}

/// 把 "10.0.11" 之类目录名解析成可比较的数值段（后缀非数字即忽略）。
fn parse_version(value: &str) -> Option<Vec<u64>> {
    let mut parts = Vec::new();
    for segment in value.split('.') {
        match segment.parse::<u64>() {
            Ok(number) => parts.push(number),
            Err(_) => return (parts.len() >= 2).then_some(parts),
        }
    }
    (parts.len() >= 2).then_some(parts)
}

fn text(path: &Path) -> CString {
    CString::new(path.as_os_str().as_encoded_bytes()).expect("测试路径含 NUL 字节")
}

/// 经 ABI 建一次 host，返回句柄与状态。
fn create(
    hostfxr: &Path,
    runtime_config: &Path,
    assembly: &Path,
    entry_spec: &str,
) -> (i32, *mut c_void) {
    let hostfxr = text(hostfxr);
    let runtime_config = text(runtime_config);
    let assembly = text(assembly);
    let entry_spec = CString::new(entry_spec).unwrap();
    let mut handle = std::ptr::null_mut();
    // SAFETY: 均为合法 CString；handle 是栈上可写出参。
    let status = unsafe {
        create_clr_host(
            hostfxr.as_ptr(),
            runtime_config.as_ptr(),
            assembly.as_ptr(),
            entry_spec.as_ptr(),
            &mut handle,
        )
    };
    (status, handle)
}

#[test]
fn missing_hostfxr_fails_cleanly_and_is_repeatable() {
    let hostfxr = manifest_dir().join("testfixture/does-not-exist/hostfxr.dll");
    let config = manifest_dir().join("testfixture/does-not-exist/runtimeconfig.json");
    let assembly = manifest_dir().join("testfixture/does-not-exist/assembly.dll");

    for _ in 0..3 {
        let (status, handle) = create(&hostfxr, &config, &assembly, FIXTURE_ENTRY_SPEC);
        assert_eq!(status, ClrHostStatus::ClrInitFailed as i32);
        assert!(handle.is_null(), "失败路径不得产出句柄");
    }
}

#[test]
fn invalid_runtime_config_fails_cleanly() {
    let hostfxr = locate_hostfxr();
    let scratch = std::env::temp_dir().join(format!("lumio-clrhost-test-{}", std::process::id()));
    fs::create_dir_all(&scratch).expect("无法创建测试临时目录");
    let config = scratch.join("bad.runtimeconfig.json");
    fs::write(&config, "{ this is not a runtime config").expect("无法写入坏 runtimeconfig");
    let assembly = scratch.join("missing-assembly.dll");

    // 两种「坏」各测两遍：内容非法 JSON、文件不存在；都必须干净失败且可重复。
    for _ in 0..2 {
        let (status, handle) = create(&hostfxr, &config, &assembly, FIXTURE_ENTRY_SPEC);
        assert_eq!(status, ClrHostStatus::ClrInitFailed as i32);
        assert!(handle.is_null());
    }
    fs::remove_file(&config).ok();

    let missing = scratch.join("does-not-exist.runtimeconfig.json");
    for _ in 0..2 {
        let (status, handle) = create(&hostfxr, &missing, &assembly, FIXTURE_ENTRY_SPEC);
        assert_eq!(status, ClrHostStatus::ClrInitFailed as i32);
        assert!(handle.is_null());
    }
}

#[test]
fn managed_entry_roundtrip_lowercases_input() {
    let fixture = build_fixture();
    let hostfxr = locate_hostfxr();
    let runtime_config = fixture.join("LumioClrHostFixture.runtimeconfig.json");
    let assembly = fixture.join("LumioClrHostFixture.dll");

    let (status, handle) = create(&hostfxr, &runtime_config, &assembly, FIXTURE_ENTRY_SPEC);
    assert_eq!(
        status,
        ClrHostStatus::Success as i32,
        "真实装载链必须一次到位（hostfxr={}）",
        hostfxr.display()
    );
    assert!(!handle.is_null());

    let input = b"HeLLo WoRld 42!";
    let mut output = [0_u8; 64];
    let mut written: u32 = 0;
    // SAFETY: handle 来自成功的 create；缓冲在栈上存活；written 为有效出参。
    let status = unsafe {
        clr_host_call(
            handle,
            input.as_ptr(),
            input.len() as u32,
            output.as_mut_ptr(),
            output.len() as u32,
            &mut written,
        )
    };
    assert_eq!(status, ClrHostStatus::Success as i32);
    assert_eq!(written as usize, input.len());
    assert_eq!(&output[..written as usize], b"hello world 42!");

    // 容量不足：托管入口返回 2 → BufferTooSmall，且 written 携带所需长度。
    let mut small = [0_u8; 4];
    let mut required: u32 = 0;
    // SAFETY: 同上。
    let status = unsafe {
        clr_host_call(
            handle,
            input.as_ptr(),
            input.len() as u32,
            small.as_mut_ptr(),
            small.len() as u32,
            &mut required,
        )
    };
    assert_eq!(status, ClrHostStatus::BufferTooSmall as i32);
    assert_eq!(required as usize, input.len());

    // SAFETY: 此前没有并发调用；销毁后不再触碰句柄。
    assert_eq!(
        unsafe { destroy_clr_host(handle) },
        ClrHostStatus::Success as i32
    );

    // 反例（MS-00002 集成实测钉进测试，置于本测试末尾是刻意的）：load_assembly_and_get_
    // function_pointer 按**托管方法名**解析；把 UnmanagedCallersOnly 的 EntryPoint 别名当
    // 方法名传入会得到 0x80131513（MissingMethod）→ ClrInitFailed，且不得产出句柄。
    // 注：CoreCLR 每进程只能成功 initialize 一次（hostfxr_close 不会卸载已启动的运行时，
    // 二次 initialize_for_runtime_config 返回 0x80008081）——因此本反例必须在一次成功
    // create/destroy 之后执行：届时它会在 initialize 或 load_assembly 任一步失败，两种
    // 失败都映射 ClrInitFailed + 空句柄，断言对执行顺序不敏感。
    let (status, handle) = create(
        &hostfxr,
        &runtime_config,
        &assembly,
        FIXTURE_ALIAS_ONLY_SPEC,
    );
    assert_eq!(status, ClrHostStatus::ClrInitFailed as i32);
    assert!(handle.is_null(), "MissingMethod 路径不得产出句柄");
}
