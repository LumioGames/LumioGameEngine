//! crate API 行为测试（LCE-P0-003）：Schema registry 的摘要索引访问为主，兼覆
//! 各嵌入面与只读镜像的字节同一性、ErrorCode/Capability 的派生表自洽。
//! 文件级零差异与 descriptor Hash 取证在 tests/generated_integrity.rs（本卡
//! 授权文件集只含这两个测试文件，crate 级断言集中于此）。

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use lumio_core_contracts::generated::{contracts, error_codes, schema_registry};
use lumio_core_contracts::{CapabilityId, ErrorCode, IdStatus};

fn mirror_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../generated/architecture/LGE-V1.4-2026-08-27")
        .canonicalize()
        .expect("定位只读镜像失败")
}

fn mirror_bytes(rel: &str) -> Vec<u8> {
    fs::read(mirror_root().join(rel)).unwrap_or_else(|e| panic!("读镜像 {rel} 失败：{e}"))
}

#[test]
fn schema_entries_match_mirror_bytes() {
    let index = mirror_bytes("schemas/index.json");
    assert_eq!(schema_registry::SCHEMAS_INDEX_JSON, &index[..]);
    // 注册表条目数以镜像注册表为准（每条恰有一个 "file" 键）。
    let entry_count = String::from_utf8(index)
        .unwrap()
        .matches("\"file\"")
        .count();
    assert_eq!(schema_registry::SCHEMAS.len(), entry_count);
    assert_eq!(
        schema_registry::COMMON_SCHEMA_DEFS,
        &mirror_bytes("schemas/common.schema.json")[..]
    );
    for entry in &schema_registry::SCHEMAS {
        assert_eq!(
            entry.bytes,
            &mirror_bytes(&format!("schemas/{}", entry.file))[..],
            "schema {} 嵌入字节与镜像不一致",
            entry.file
        );
    }
}

#[test]
fn schema_digest_index_lookups() {
    let mut digests = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for entry in &schema_registry::SCHEMAS {
        assert_eq!(entry.sha256_hex.len(), 64, "{} 摘要长度非 64", entry.file);
        assert!(
            entry
                .sha256_hex
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
            "{} 摘要非小写十六进制",
            entry.file
        );
        assert!(
            digests.insert(entry.sha256_hex),
            "摘要重复：{}",
            entry.sha256_hex
        );
        assert!(ids.insert(entry.id), "contract id 重复：{}", entry.id);
        let by_digest = schema_registry::schema_by_digest(entry.sha256_hex)
            .unwrap_or_else(|| panic!("按摘要找不到 {}", entry.file));
        assert_eq!(by_digest.file, entry.file);
        assert_eq!(by_digest.bytes, entry.bytes);
        let by_id = schema_registry::schema_by_id(entry.id)
            .unwrap_or_else(|| panic!("按 id 找不到 {}", entry.id));
        assert_eq!(by_id.file, entry.file);
    }
    assert!(schema_registry::schema_by_digest(&"0".repeat(64)).is_none());
    assert!(schema_registry::schema_by_id("no-such-contract").is_none());
}

#[test]
fn contracts_and_ids_embeds_match_mirror_bytes() {
    assert_eq!(
        contracts::ROOT_ABI_BUNDLE_JSON,
        &mirror_bytes("packages/abi/root-abi-bundle.json")[..]
    );
    assert_eq!(
        contracts::LUMIO_CORE_H,
        &mirror_bytes("packages/abi/lumio_core.h")[..]
    );
    assert_eq!(
        error_codes::IDS_INDEX_JSON,
        &mirror_bytes("ids/index.json")[..]
    );
}

#[test]
fn error_code_and_capability_tables_are_consistent() {
    let mut numerics = BTreeSet::new();
    let mut names = BTreeSet::new();
    for code in ErrorCode::ALL {
        assert!(
            numerics.insert(code.numeric()),
            "ErrorCode 数值重复：{}",
            code.numeric()
        );
        assert!(names.insert(code.id()), "ErrorCode id 重复：{}", code.id());
        assert_eq!(ErrorCode::from_numeric(code.numeric()), Some(code));
        assert!(!code.since().is_empty());
        let _ = code.status();
    }
    assert_eq!(ErrorCode::from_numeric(0), None);
    let mut capability_numerics = BTreeSet::new();
    for capability in CapabilityId::ALL {
        assert!(capability_numerics.insert(capability.numeric()));
        assert_eq!(
            CapabilityId::from_numeric(capability.numeric()),
            Some(capability)
        );
    }
    assert_eq!(CapabilityId::from_numeric(0), None);
    // 上游已发布的语义抽查：HybridCLR 在 registry 中登记为 Reserved。
    assert_eq!(CapabilityId::HybridCLR.status(), IdStatus::Reserved);
}

#[test]
fn crate_surface_records_baseline_pin() {
    assert_eq!(
        lumio_core_contracts::ARCHITECTURE_BASELINE_ID,
        "LGE-V1.4-2026-08-27"
    );
    assert_eq!(
        lumio_core_contracts::ARCHITECTURE_COMMIT,
        "1f2ead332b3dfc3042e1495bfbe6febb8699df7e"
    );
}
