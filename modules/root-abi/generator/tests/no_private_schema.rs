//! 本仓不得私有化 ABI 语义（规格 §8.1 非职责、卡面「非目标」）。
//!
//! generator 是薄适配器：模板、slot 表、type map 全部属于**上游 compiler**，本 crate
//! 只负责校验身份、喂输入、对账摘要、只读发布。这条边界一旦被越过，本仓就成了第二个
//! ABI 定义处，而两处定义迟早会分叉——那正是规格 §4「私有模板会制造第二 ABI」要防的。
//!
//! 这些断言是**源码级**的：它们不跑生成，只看本 crate 的代码里有没有出现不该有的东西。

use std::path::{Path, PathBuf};

fn source_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        for entry in std::fs::read_dir(dir).expect("读源码目录") {
            let path = entry.expect("目录项").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let text = std::fs::read_to_string(&path).expect("读源码");
                out.push((path, text));
            }
        }
    }
    let mut out = Vec::new();
    walk(&source_directory(), &mut out);
    assert!(!out.is_empty(), "至少应有源码文件");
    out
}

/// 只看代码，不看注释与文档——注释里出现这些词是**解释边界**，恰恰是应该有的。
fn code_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//") && !line.starts_with("*") && !line.is_empty())
}

#[test]
fn generator_contains_no_c_or_csharp_or_rust_type_mapping_table() {
    // 上游 ABI_TYPE_MAPPING 的形态：把 typeRef 映射到 C / C# / Rust 拼写。
    // 本仓出现任何一组这类字面量，就等于开了第二份 type map。
    let markers = [
        "uint8_t",
        "uint16_t",
        "uint32_t",
        "uint64_t",
        "int8_t",
        "int16_t",
        "int32_t",
        "int64_t",
        "lumio_status_t",
        "lumio_handle_t",
        "LumioStatus",
        "LumioHandle",
    ];
    for (path, text) in rust_sources() {
        for line in code_lines(&text) {
            for marker in markers {
                assert!(
                    !line.contains(marker),
                    "{} 的代码里出现了 C/C# 类型拼写 {marker}：type map 属上游 compiler，\
                     本仓不得持有第二份\n    {line}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn generator_emits_no_header_or_binding_text() {
    // 生成文本的特征：`#include`、`#pragma`、`extern "C"`、`namespace`、`#[repr(C)]`
    // 之类只可能出现在被生成的内容里。适配器不产生这些字节，只搬运上游产物。
    let markers = [
        "#include",
        "#pragma",
        "extern \\\"C\\\"",
        "namespace Lumio",
        "#[repr(C)]",
        "public static class",
        "typedef struct",
    ];
    for (path, text) in rust_sources() {
        for line in code_lines(&text) {
            for marker in markers {
                assert!(
                    !line.contains(marker),
                    "{} 的代码里出现了生成文本片段 {marker}：本仓不实现模板\n    {line}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn generator_declares_no_slot_indices_or_layout_constants() {
    // slot 编号与布局常量（pointerBytes / maxAlignment / rootHeaderBytes …）都由上游
    // layoutProfile 提供；本仓写死任何一个，就会在上游改布局时静默不一致。
    let markers = [
        "pointerBytes",
        "maxAlignment",
        "rootHeaderBytes",
        "tableHeaderBytes",
        "slot_index",
        "slotIndex",
    ];
    for (path, text) in rust_sources() {
        for line in code_lines(&text) {
            for marker in markers {
                // 允许以字符串键的形式**读取**上游字段，不允许把值写死成常量。
                let is_constant_definition = line.contains("const ") || line.contains("static ");
                assert!(
                    !(line.contains(marker) && is_constant_definition),
                    "{} 把上游布局常量 {marker} 写死了：它必须每次从上游 layoutProfile 读\n    {line}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn generator_does_not_read_the_architecture_source_working_tree() {
    // 输入只能是本仓的 architecture.lock.json 与只读镜像。直接去读架构源仓工作区
    // （或 docs/architecture/）会让产物依赖一个不受 lock 约束的可变输入。
    let forbidden = ["LumioGameEngineArchitecture", "docs/architecture"];
    for (path, text) in rust_sources() {
        for line in code_lines(&text) {
            for marker in forbidden {
                assert!(
                    !line.contains(marker),
                    "{} 的代码里引用了架构源工作区 {marker}：输入只能是 lock 与只读镜像\n    {line}",
                    path.display()
                );
            }
        }
    }
}
