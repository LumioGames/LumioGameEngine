using System;

namespace Lumio.Gen.CanonicalSerializer
{
    // ADR-047 LumioBinV1: the binary canonical form for public payload bytes.
    public static class LumioBinForm
    {
        public const string FormId = "LumioBinV1";
        public const string ByteOrder = "LittleEndian";
        public const string StringEncoding = "Utf8";
        public const string StringLengthPrefix = "u32";
        public const string BytesLengthPrefix = "u32";
        public const string ArrayCountPrefix = "u32";
        public const string FieldOrder = "SchemaDeclarationOrder";
        public const string Padding = "None";
        public const string Floats = "None";
        public const string DigestFraming = "None";
    }

    public readonly struct LumioBinIntegerWidth
    {
        public LumioBinIntegerWidth(string kind, uint bytes, bool signed)
        {
            Kind = kind; Bytes = bytes; Signed = signed;
        }
        public string Kind { get; }
        public uint Bytes { get; }
        public bool Signed { get; }
    }
    public static class LumioBinIntegerWidths
    {
        public static readonly LumioBinIntegerWidth[] All =
        {
            new LumioBinIntegerWidth("u8", 1, false),
            new LumioBinIntegerWidth("u16", 2, false),
            new LumioBinIntegerWidth("u32", 4, false),
            new LumioBinIntegerWidth("u64", 8, false),
            new LumioBinIntegerWidth("i32", 4, true),
            new LumioBinIntegerWidth("i64", 8, true),
        };
    }

    public readonly struct LumioBinGolden
    {
        public LumioBinGolden(string id, string @case, string sha256)
        {
            Id = id; Case = @case; Sha256 = sha256;
        }
        public string Id { get; }
        public string Case { get; }
        public string Sha256 { get; }
    }
    public static class LumioBinGoldens
    {
        public static readonly LumioBinGolden[] All =
        {
            new LumioBinGolden("integer-widths", "IntegerWidthsLittleEndian", "e4c15e2b8347986315e042c3b009ac9d9fc4833ffdfa984671c804d48c53af72"),
            new LumioBinGolden("string-utf8", "StringUtf8ByteLength", "a2969994674a03c90bdf3a04fc1e872e57dfb5c69b20c02a6ec58a8fcdecc77f"),
            new LumioBinGolden("bytes-prefixed", "BytesLengthPrefix", "0099fed1a7eb2bd476767cc61c24fd219eb85f12a771097b6ed1f8f9c0a191fc"),
            new LumioBinGolden("array-count", "ArrayCountPrefix", "a39723192d4a221f9eb82ffb339d1ca9306ed7cd3c9ebff18d66b3f3094d3080"),
            new LumioBinGolden("struct-declaration-order", "DeclarationOrderNoPadding", "906a52a6e0337a092c17b65dbc4d35ceeede618307bb6178e8661f6ef9e43f95"),
            new LumioBinGolden("nested-composition", "NestedComposition", "109299fca81e33863a42d186eae66c8f3528b1b960deb067b53060d1c9438ad7"),
        };
    }

    public readonly struct LumioBinRejection
    {
        public LumioBinRejection(string id, string @case, string error)
        {
            Id = id; Case = @case; Error = error;
        }
        public string Id { get; }
        public string Case { get; }
        public string Error { get; }
    }
    public static class LumioBinRejections
    {
        public static readonly LumioBinRejection[] All =
        {
            new LumioBinRejection("u8-above-range", "IntegerRangeOverflow", "IntegerRangeOverflow"),
            new LumioBinRejection("u32-negative", "UnsignedNegative", "IntegerRangeOverflow"),
            new LumioBinRejection("u32-fractional", "NonIntegerNumber", "NonIntegerNumber"),
            new LumioBinRejection("u32-integral-float", "IntegralFloat", "NonIntegerNumber"),
            new LumioBinRejection("u32-string", "TypeMismatch", "TypeMismatch"),
            new LumioBinRejection("u32-boolean", "BooleanForInteger", "TypeMismatch"),
            new LumioBinRejection("bytes-odd-length", "MalformedHexBytes", "TypeMismatch"),
            new LumioBinRejection("bytes-upper-case", "MalformedHexBytes", "TypeMismatch"),
            new LumioBinRejection("bytes-non-hex", "MalformedHexBytes", "TypeMismatch"),
            new LumioBinRejection("f32-layout", "UnknownLayoutKind", "UnknownLayoutKind"),
            new LumioBinRejection("struct-missing-field", "MissingField", "MissingField"),
            new LumioBinRejection("struct-unknown-field", "UnknownField", "UnknownField"),
        };
    }
}
