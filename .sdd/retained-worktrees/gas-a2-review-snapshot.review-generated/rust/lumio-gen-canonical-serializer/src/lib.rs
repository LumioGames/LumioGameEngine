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

/// ADR-041 CanonicalJsonV1: the canonical form is defined by the architecture source,
/// never inherited from a generic JCS library's defaults.
pub const CANONICAL_FORM_ID: &str = "CanonicalJsonV1";
pub const CANONICAL_ENCODING: &str = "AsciiEscaped";
pub const CANONICAL_MEMBER_ORDER: &str = "CodePointAscending";
pub const CANONICAL_ARRAY_ORDER: &str = "DocumentOrder";
pub const CANONICAL_ITEM_SEPARATOR: char = ',';
pub const CANONICAL_KEY_VALUE_SEPARATOR: char = ':';
pub const CANONICAL_NUMBERS: &str = "IntegerOnly";
pub const CANONICAL_UNKNOWN_MEMBERS: &str = "Reject";
pub const CANONICAL_DUPLICATE_MEMBERS: &str = "Reject";
pub const DIGEST_ALGORITHM: &str = "SHA-256";
pub const DIGEST_FRAMING: &str = "PrefixFreeOverCanonicalBytes";

#[derive(Clone, Copy, Debug)]
pub struct NormalizationStep {
    pub path: &'static str,
    pub op: &'static str,
    pub by: &'static str,
    pub collation: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct DigestDomain {
    pub digest: &'static str,
    pub domain_tag: &'static str,
    pub sort_rule: &'static str,
    pub omit_members: &'static [&'static str],
    /// Executed in declared order, before canonicalization.
    pub normalization: &'static [NormalizationStep],
}

pub const DIGEST_DOMAINS: &[DigestDomain] = &[
    DigestDomain { digest: "manifestDigest", domain_tag: "CoreEngineManifestBody", sort_rule: "member order only; the body has no array whose order is semantic", omit_members: &[], normalization: &[] },
    DigestDomain { digest: "artifactSetDigest", domain_tag: "ArtifactSetV1", sort_rule: "entries sorted ascending by path (code point); paths are unique within an index", omit_members: &["artifactSetDigest"], normalization: &[NormalizationStep { path: "entries", op: "sortAscending", by: "path", collation: "codePoint" }] },
    DigestDomain { digest: "artifactIndexDigest", domain_tag: "ArtifactIndexV1", sort_rule: "index.entries sorted ascending by path (code point)", omit_members: &[], normalization: &[NormalizationStep { path: "index.entries", op: "sortAscending", by: "path", collation: "codePoint" }] },
    DigestDomain { digest: "targetProfileDigest", domain_tag: "TargetProfileV1", sort_rule: "member order only; the profile has no array", omit_members: &[], normalization: &[] },
    DigestDomain { digest: "capabilitySetDigest", domain_tag: "CapabilitySetV1", sort_rule: "capabilities sorted ascending by code point; the array is uniqueItems so ties are impossible", omit_members: &[], normalization: &[NormalizationStep { path: "capabilities", op: "sortAscending", by: "$self", collation: "codePoint" }] },
    DigestDomain { digest: "mappingSetHash", domain_tag: "ReplicationMappingSetV1", sort_rule: "mappings sorted ascending by code point; mappingId is unique within a set so ties are impossible", omit_members: &[], normalization: &[NormalizationStep { path: "mappings", op: "sortAscending", by: "$self", collation: "codePoint" }] },
];

/// Golden vectors: `(id, case, sha256)`. Full inputs and canonical bytes are in
/// the published `canonical/canonical-digest-profile.json`.
pub const CANONICAL_GOLDENS: &[(&str, &str, &str)] = &[
    ("artifact-set-empty", "EmptyArtifactSet", "7a92ee35f0ae0644282f438a675d7624800a8aeac5125c85d7796d844831ce69"),
    ("artifact-set-single", "SingleArtifact", "b5102a58a8338bff3200949d01c341abaa66527628ea91cdaa7a928987e9a7e9"),
    ("artifact-set-multi", "MultiArtifact", "7d851d441b751469da4c5e935736fe7684b18db9387d195a3dcbd48684d9a365"),
    ("artifact-set-path-permutation", "PathOrderPermutation", "7d851d441b751469da4c5e935736fe7684b18db9387d195a3dcbd48684d9a365"),
    ("capability-set-permutation", "CapabilityOrderPermutation", "6c0859b0a2c6624a7725bcbdb2d71b1e2a65bbf413cac6874875dc69fc54d19a"),
    ("escape-boundary", "EscapeBoundary", "b7fb9362f56ca50b68969d61dd73ee73f31a23a64d4f15a898045c1f6c4cb5eb"),
    ("integer-boundary", "IntegerBoundary", "a6135f19ccbcd596fb0bd7bc81fcd9770f49609c483812fc776bf71c28c2de73"),
    ("artifact-set-schema-version", "SchemaVersionChange", "e0fd1f58d4435bec2365841d68280a6bf03aa158e99de07c3e970f7bc836968e"),
    ("replication-mapping-set-empty", "EmptyMappingSet", "a805f7c841f708981cc82a93047d7b0c8e6bf923f3dba18e179036741a6d2ea7"),
    ("replication-mapping-set-permutation", "MappingOrderPermutation", "4120cf666fec14f6bcaf703a5d10706d755f36fb0e354dfdec6e6d5bddc40e23"),
];

/// ADR-047 LumioBinV1: the binary canonical form for public payload bytes.
/// `CanonicalJsonV1` stays the form for canonicalizable JSON documents; this is
/// the primitive layer ADR-010 referred to and ADR-035 assumed.
pub const LUMIO_BIN_FORM_ID: &str = "LumioBinV1";
pub const LUMIO_BIN_BYTE_ORDER: &str = "LittleEndian";
pub const LUMIO_BIN_STRING_ENCODING: &str = "Utf8";
pub const LUMIO_BIN_STRING_LENGTH_PREFIX: &str = "u32";
pub const LUMIO_BIN_BYTES_LENGTH_PREFIX: &str = "u32";
pub const LUMIO_BIN_ARRAY_COUNT_PREFIX: &str = "u32";
pub const LUMIO_BIN_FIELD_ORDER: &str = "SchemaDeclarationOrder";
pub const LUMIO_BIN_PADDING: &str = "None";
pub const LUMIO_BIN_FLOATS: &str = "None";
pub const LUMIO_BIN_DIGEST_FRAMING: &str = "None";

/// Integer widths, as `(kind, bytes, signed)`. Little-endian, no padding.
pub const LUMIO_BIN_INTEGER_WIDTHS: &[(&str, u32, bool)] = &[
    ("u8", 1, false),
    ("u16", 2, false),
    ("u32", 4, false),
    ("u64", 8, false),
    ("i32", 4, true),
    ("i64", 8, true),
];

/// Golden vectors: `(id, case, sha256)`. Layouts, values and bytes are in
/// the published `binary/lumio-bin-profile.json`.
pub const LUMIO_BIN_GOLDENS: &[(&str, &str, &str)] = &[
    ("integer-widths", "IntegerWidthsLittleEndian", "e4c15e2b8347986315e042c3b009ac9d9fc4833ffdfa984671c804d48c53af72"),
    ("string-utf8", "StringUtf8ByteLength", "a2969994674a03c90bdf3a04fc1e872e57dfb5c69b20c02a6ec58a8fcdecc77f"),
    ("bytes-prefixed", "BytesLengthPrefix", "0099fed1a7eb2bd476767cc61c24fd219eb85f12a771097b6ed1f8f9c0a191fc"),
    ("array-count", "ArrayCountPrefix", "a39723192d4a221f9eb82ffb339d1ca9306ed7cd3c9ebff18d66b3f3094d3080"),
    ("struct-declaration-order", "DeclarationOrderNoPadding", "906a52a6e0337a092c17b65dbc4d35ceeede618307bb6178e8661f6ef9e43f95"),
    ("nested-composition", "NestedComposition", "109299fca81e33863a42d186eae66c8f3528b1b960deb067b53060d1c9438ad7"),
];

/// Inputs a conforming encoder must refuse: `(id, case, error)`.
pub const LUMIO_BIN_REJECTIONS: &[(&str, &str, &str)] = &[
    ("u8-above-range", "IntegerRangeOverflow", "IntegerRangeOverflow"),
    ("u32-negative", "UnsignedNegative", "IntegerRangeOverflow"),
    ("u32-fractional", "NonIntegerNumber", "NonIntegerNumber"),
    ("u32-integral-float", "IntegralFloat", "NonIntegerNumber"),
    ("u32-string", "TypeMismatch", "TypeMismatch"),
    ("u32-boolean", "BooleanForInteger", "TypeMismatch"),
    ("bytes-odd-length", "MalformedHexBytes", "TypeMismatch"),
    ("bytes-upper-case", "MalformedHexBytes", "TypeMismatch"),
    ("bytes-non-hex", "MalformedHexBytes", "TypeMismatch"),
    ("f32-layout", "UnknownLayoutKind", "UnknownLayoutKind"),
    ("struct-missing-field", "MissingField", "MissingField"),
    ("struct-unknown-field", "UnknownField", "UnknownField"),
];
