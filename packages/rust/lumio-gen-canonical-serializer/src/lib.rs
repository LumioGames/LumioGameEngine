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
