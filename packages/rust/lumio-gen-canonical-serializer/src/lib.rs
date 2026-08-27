//! Generated CanonicalSerializer artifact. Do not hand-edit.
//! Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27.

#![forbid(unsafe_code)]

/// snapshot-header.checksum covers SHA-256 of the canonical JSON of the header
/// object with `checksum` and `hash` omitted (UTF-8, sorted keys, no extra whitespace).
pub const SNAPSHOT_CHECKSUM_OMIT: &[&str] = &["checksum", "hash"];
pub const SNAPSHOT_MAGIC: &str = "LUMIOSNP1";

pub fn checksum_domain_doc() -> &'static str {
    "SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields"
}
