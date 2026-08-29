//! `PackagePath` 的路径安全不变量（规格 §6.3、§9.3）。
//!
//! 这是包内路径进入运行时的唯一闸门：所有包内路径先转为 `PackagePath`，其不变量为
//! UTF-8、相对、正斜杠、非空、无盘符 / NUL / `.` / `..` / 重复分隔符，规范化后仍在
//! package root 内。
//!
//! 分工要说清楚：本类型**不碰文件系统**（卡面非目标：不打开文件）。它拒掉的是
//! **词法上**可能逃逸的一切写法——由于 `..` 一律拒绝，攻击者无法用「symlink + `..`」
//! 组合出逃逸路径；真正跟随符号链接的防御是打开时的
//! `openat2(RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS)`，归 platform-runtime（LCE-P0-014）。
//! 两层缺一不可，本文件只证第一层。

use lumio_core_platform_contracts::{PackagePath, PackagePathError};
use proptest::prelude::*;

fn rejected(value: &str) -> PackagePathError {
    match PackagePath::parse(value) {
        Ok(path) => panic!("{value:?} 必须被拒绝，却解析成了 {:?}", path.as_str()),
        Err(error) => error,
    }
}

#[test]
fn absolute_paths_are_rejected() {
    for value in ["/native/liblumio_core.so", "/", "//server/share/x"] {
        assert_eq!(rejected(value), PackagePathError::NotRelative, "{value:?}");
    }
}

#[test]
fn drive_letters_and_backslashes_are_rejected() {
    // 盘符与反斜杠在 Windows 上都是「另一种绝对路径 / 另一种分隔符」，
    // 只按正斜杠切分会把 `a\..\..\etc` 看成单个合法分量。
    for value in [
        r"C:/native/x.so",
        r"c:x",
        r"native\liblumio_core.so",
        r"..\..\etc\passwd",
        r"\\server\share",
    ] {
        let error = rejected(value);
        assert!(
            matches!(
                error,
                PackagePathError::DriveLetter | PackagePathError::BackslashSeparator
            ),
            "{value:?} 得到 {error:?}"
        );
    }
}

#[test]
fn nul_bytes_are_rejected() {
    for value in ["native/lib\0.so", "\0", "metadata/a\0b/c"] {
        assert_eq!(rejected(value), PackagePathError::ContainsNul, "{value:?}");
    }
}

#[test]
fn traversal_components_are_rejected() {
    for value in [
        "../etc/passwd",
        "native/../../etc/passwd",
        "native/..",
        "..",
        "native/./x.so",
        ".",
        "./x",
    ] {
        let error = rejected(value);
        assert!(
            matches!(
                error,
                PackagePathError::ParentTraversal | PackagePathError::CurrentDirectory
            ),
            "{value:?} 得到 {error:?}"
        );
    }
}

#[test]
fn symlink_escape_shapes_are_rejected_lexically() {
    // 「symlink escape」在本层的形态：先进一个（可能是符号链接的）目录，再用 `..`
    // 跳出去。`..` 被无条件拒绝，这条路在词法层就断了。
    for value in [
        "evidence/link/../../etc/passwd",
        "metadata/../../../root/.ssh/id_rsa",
        "native/link/..",
    ] {
        assert_eq!(
            rejected(value),
            PackagePathError::ParentTraversal,
            "{value:?}"
        );
    }
}

#[test]
fn empty_and_duplicate_separators_are_rejected() {
    assert_eq!(rejected(""), PackagePathError::Empty);
    for value in ["native//x.so", "native/", "a//b//c", "a/b//"] {
        let error = rejected(value);
        assert!(
            matches!(
                error,
                PackagePathError::EmptyComponent | PackagePathError::TrailingSeparator
            ),
            "{value:?} 得到 {error:?}"
        );
    }
}

#[test]
fn control_characters_are_rejected() {
    // 控制字符在 Linux 上是合法文件名字符，但路径会原样进错误消息（Display），
    // 恶意包可借此往日志里注入换行 / 回车 / ANSI 转义。包内制品没有正当理由用它们。
    for value in [
        "native/a\nb.so",
        "native/a\rb.so",
        "metadata/\u{1b}[31mred.json",
        "evidence/tab\there.json",
    ] {
        assert_eq!(
            rejected(value),
            PackagePathError::ControlCharacter,
            "{value:?}"
        );
    }
}

#[test]
fn windows_trailing_dot_or_space_is_rejected() {
    // Win32 解析时静默剥离尾随点与空格：`…manifest.json.` 与控制文件本身会解析到
    // 同一个文件，于是两个不同的 PackagePath 值指向同一实体，「路径即身份」被破坏。
    for value in [
        "metadata/core-engine-manifest.json.",
        "metadata/core-engine-manifest.json ",
        "native /liblumio_core.so",
        "native./x.so",
    ] {
        assert_eq!(
            rejected(value),
            PackagePathError::TrailingDotOrSpace,
            "{value:?}"
        );
    }
}

#[test]
fn windows_reserved_device_names_are_rejected() {
    // 这些名字在 Win32 上打开的是设备而不是文件，带扩展名同样命中。
    for value in [
        "CON",
        "native/NUL",
        "native/nul.so",
        "COM1.so",
        "evidence/LPT9.json",
        "aux/x.json",
    ] {
        assert_eq!(
            rejected(value),
            PackagePathError::ReservedDeviceName,
            "{value:?}"
        );
    }
}

#[test]
fn names_that_merely_resemble_reserved_devices_are_accepted() {
    // 拒绝得过宽会让合法制品进不了包：只有分量的 stem 恰好等于保留名才拒绝。
    for value in [
        "console.json",
        "native/nullable.so",
        "COM10.so",
        "LPT0.json",
    ] {
        PackagePath::parse(value)
            .unwrap_or_else(|error| panic!("{value:?} 应被接受，却报 {error:?}"));
    }
}

#[test]
fn canonical_relative_paths_round_trip() {
    for value in [
        "native/liblumio_core.so",
        "metadata/core-engine-manifest.json",
        "evidence/sbom.cdx.json",
        "a",
        "a/b/c/d/e",
        "含中文的-文件名.json",
        "with space.txt",
        "dot.in.name/x.tar.zst",
        // 前缀是 `..` 但不是 `..` 本身的分量必须放行，否则会误伤合法文件名。
        "..leading-dots.json",
        "native/...three",
    ] {
        let path = PackagePath::parse(value)
            .unwrap_or_else(|error| panic!("{value:?} 应被接受，却报 {error:?}"));
        assert_eq!(path.as_str(), value, "round-trip 必须逐字节相同");
    }
}

#[test]
fn components_iterate_in_order() {
    let path = PackagePath::parse("metadata/core-engine-manifest.json").expect("合法路径");
    assert_eq!(
        path.components().collect::<Vec<_>>(),
        vec!["metadata", "core-engine-manifest.json"]
    );
}

#[test]
fn equal_paths_compare_and_hash_equal() {
    let a = PackagePath::parse("native/liblumio_core.so").expect("合法");
    let b = PackagePath::parse("native/liblumio_core.so").expect("合法");
    assert_eq!(a, b);
    let mut set = std::collections::BTreeSet::new();
    set.insert(a);
    assert!(!set.insert(b), "同值路径在集合里必须去重");
}

proptest! {
    // 验收命令要求对随机路径执行 10 万 case。
    // failure_persistence 关掉：本 crate 的测试目录下没有 lib.rs/main.rs，
    // proptest 会为找不到落盘位置而在每次运行里刷一行噪音；反例靠失败输出里的
    // 最小化结果复现即可，不需要落盘。
    #![proptest_config(ProptestConfig {
        cases: 100_000,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// 任意字符串输入下都不 panic，且**接受即满足全部不变量**——
    /// 这条是核心：解析器可以拒绝得过宽，但绝不能放进一个违反不变量的值。
    #[test]
    fn accepted_paths_always_satisfy_every_invariant(value in ".*") {
        if let Ok(path) = PackagePath::parse(&value) {
            let text = path.as_str();
            prop_assert_eq!(text, &value, "接受时必须原样保留");
            prop_assert!(!text.is_empty());
            prop_assert!(!text.starts_with('/'));
            prop_assert!(!text.contains('\\'));
            prop_assert!(!text.contains('\0'));
            prop_assert!(!text.contains("//"));
            prop_assert!(!text.ends_with('/'));
            for component in text.split('/') {
                prop_assert!(!component.is_empty());
                prop_assert_ne!(component, ".");
                prop_assert_ne!(component, "..");
            }
            // 不在这里复述实现的盘符判据（抄实现的断言会跟着实现一起错）；
            // 改为断言一条独立性质：给任何被接受的路径加上盘符前缀后必被拒绝。
            let with_drive = format!("c:{}", text);
            prop_assert!(PackagePath::parse(&with_drive).is_err());
        }
    }

    /// 由合法分量拼出来的路径必须被接受——拒绝得过宽会让合法制品进不了包。
    #[test]
    fn paths_built_from_safe_components_are_accepted(
        // 首尾字符都不取 `.`/空格：分量因此不可能等于 `.`/`..`，也不会以点或空格结尾。
        // 用生成器排除而不是生成后 prop_assume 丢弃——后者会把大量样本浪费在被丢弃的
        // 分支上（实测 10 万 case 里丢弃过千，proptest 直接判 too many global rejects）。
        // 以 `.` 开头、含内部点的合法文件名由 canonical_relative_paths_round_trip 覆盖。
        components in prop::collection::vec(
            "[a-zA-Z0-9_-]([a-zA-Z0-9._-]{0,10}[a-zA-Z0-9_-])?",
            1..6,
        )
    ) {
        // 保留设备名极少被随机命中（62^n 分之几），这里用 assume 丢弃是安全的，
        // 不会触发 global reject 上限；它们的拒绝由定向负例测试覆盖。
        prop_assume!(!components.iter().any(|component| {
            let stem = component.split('.').next().unwrap_or(component);
            ["CON", "PRN", "AUX", "NUL"].iter().any(|n| stem.eq_ignore_ascii_case(n))
                || (stem.len() == 4
                    && (stem[..3].eq_ignore_ascii_case("COM") || stem[..3].eq_ignore_ascii_case("LPT"))
                    && stem.as_bytes()[3].is_ascii_digit()
                    && stem.as_bytes()[3] != b'0')
        }));
        let joined = components.join("/");
        let path = PackagePath::parse(&joined);
        prop_assert!(path.is_ok(), "{:?} 应被接受，实际 {:?}", joined, path.err());
    }
}
