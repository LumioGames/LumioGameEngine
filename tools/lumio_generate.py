"""Deterministic V1.4 generated-contract artifact publisher."""

from __future__ import annotations

import hashlib
import json
import shutil
from pathlib import Path
from typing import Any, Dict, List, Tuple

BASELINE = "LGE-V1.4-2026-08-27"
PUBLISHER = "LumioGameEngineArchitecture"
KINDS = [
    "ProtocolPermissionValidator",
    "MappingTable",
    "CanonicalSerializer",
    "LanguageBinding",
    "ContractTypes",
    "ContractRuntime",
]
FORBIDDEN = ["LumioClient", "LumioGame"]
CONSUMERS = ["LumioClient", "LumioGame", "LumioGameRuntime", "LumioServer"]
RUST_CRATES = {
    "ProtocolPermissionValidator": "lumio-gen-protocol-permission-validator",
    "MappingTable": "lumio-gen-mapping-table",
    "CanonicalSerializer": "lumio-gen-canonical-serializer",
    "LanguageBinding": "lumio-gen-language-binding",
    "ContractTypes": "lumio-gen-contract-types",
    "ContractRuntime": "lumio-gen-contract-runtime",
}
CS_PROJ = {
    "ProtocolPermissionValidator": "Lumio.Gen.ProtocolPermissionValidator",
    "MappingTable": "Lumio.Gen.MappingTable",
    "CanonicalSerializer": "Lumio.Gen.CanonicalSerializer",
    "LanguageBinding": "Lumio.Gen.LanguageBinding",
    "ContractTypes": "Lumio.Gen.ContractTypes",
    "ContractRuntime": "Lumio.Gen.ContractRuntime",
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def input_hash(root: Path) -> str:
    items: List[Tuple[str, bytes]] = []
    for rel in (
        "schemas",
        "ids/index.json",
        "fixtures/valid",
    ):
        target = root / rel
        if target.is_file():
            items.append((rel.replace("\\", "/"), target.read_bytes()))
            continue
        for path in target.rglob("*"):
            if path.is_file() and path.suffix == ".json":
                key = path.relative_to(root).as_posix()
                items.append((key, path.read_bytes()))
    items.sort(key=lambda item: item[0])
    return sha256_bytes(b"\n".join(key.encode() + b"\0" + blob for key, blob in items))


def compiler_hash(root: Path) -> str:
    files = [
        root / "tools" / "lumio_contract.py",
        root / "tools" / "lumio_generate.py",
    ]
    return sha256_bytes(b"".join(p.read_bytes() for p in files))


def dir_output_hash(directory: Path) -> str:
    lines = []
    files = [path for path in directory.rglob("*") if path.is_file()]
    files.sort(key=lambda path: path.relative_to(directory).as_posix())
    for path in files:
        rel = path.relative_to(directory).as_posix()
        if rel.endswith(".descriptor.json"):
            continue
        lines.append("{}={}".format(rel, sha256_file(path)))
    return sha256_bytes("\n".join(lines).encode("utf-8"))


def load_state_machines(root: Path) -> List[Dict[str, Any]]:
    machines = []
    paths = list((root / "fixtures" / "valid").glob("state-machine-*.json"))
    paths.sort(key=lambda path: path.name)
    for path in paths:
        machines.append(json.loads(path.read_text(encoding="utf-8")))
    return machines


def load_schema_ids(root: Path) -> List[str]:
    index = json.loads((root / "schemas" / "index.json").read_text(encoding="utf-8"))
    return [item["id"] for item in index.get("schemas", [])]


def load_error_ids(root: Path) -> List[str]:
    registry = json.loads((root / "ids" / "index.json").read_text(encoding="utf-8"))
    for ns in registry.get("namespaces", []):
        if ns.get("namespace") == "ErrorCode":
            return [v["id"] for v in ns.get("values", [])]
    return []


def rust_cargo(name: str) -> str:
    return (
        "[package]\n"
        "name = \"{}\"\n"
        "version = \"0.0.0\"\n"
        "edition = \"2021\"\n"
        "publish = false\n\n"
        "[dependencies]\n"
    ).format(name)


def csproj(name: str) -> str:
    return (
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n"
        "  <PropertyGroup>\n"
        "    <TargetFramework>net8.0</TargetFramework>\n"
        "    <ImplicitUsings>disable</ImplicitUsings>\n"
        "    <Nullable>enable</Nullable>\n"
        "    <AllowUnsafeBlocks>false</AllowUnsafeBlocks>\n"
        "    <DisableImplicitNuGetFallbackFolder>true</DisableImplicitNuGetFallbackFolder>\n"
        "  </PropertyGroup>\n"
        "  <!-- Pure managed; no Native / PInvoke / implementation-project PackageReference. -->\n"
        "</Project>\n"
    )


def kebab_kind(kind: str) -> str:
    chars = []
    for i, char in enumerate(kind):
        if char.isupper() and i:
            chars.append("-")
        chars.append(char.lower())
    return "".join(chars)


def rust_lib_header(kind: str) -> str:
    return (
        "//! Generated {} artifact. Do not hand-edit.\n"
        "//! Publisher: LumioGameEngineArchitecture / {}.\n\n"
        "#![forbid(unsafe_code)]\n\n"
    ).format(kind, BASELINE)


SHA256_RS = r'''
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
    0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
    0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
    0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6eabbdc, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
    0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
    0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut buf = data.to_vec();
    buf.push(0x80);
    while buf.len() % 64 != 56 {
        buf.push(0);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in buf.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut a = h;
        for i in 0..64 {
            let s1 = a[4].rotate_right(6) ^ a[4].rotate_right(11) ^ a[4].rotate_right(25);
            let ch = (a[4] & a[5]) ^ ((!a[4]) & a[6]);
            let t1 = a[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a[0].rotate_right(2) ^ a[0].rotate_right(13) ^ a[0].rotate_right(22);
            let maj = (a[0] & a[1]) ^ (a[0] & a[2]) ^ (a[1] & a[2]);
            let t2 = s0.wrapping_add(maj);
            a[7] = a[6];
            a[6] = a[5];
            a[5] = a[4];
            a[4] = a[3].wrapping_add(t1);
            a[3] = a[2];
            a[2] = a[1];
            a[1] = a[0];
            a[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(a[i]);
        }
    }
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data).iter().map(|b| format!("{b:02x}")).collect()
}
'''


def emit_protocol_permission(rust_dir: Path, cs_dir: Path) -> None:
    fields = [
        "sessionId",
        "productId",
        "gameReleaseId",
        "messageId",
        "role",
        "claims",
        "connectionGeneration",
        "antiReplay",
        "admittedSessionId",
        "admittedProductId",
        "admittedGameReleaseId",
        "admittedRole",
        "admittedClaims",
        "admittedConnectionGeneration",
        "verdict",
    ]
    rust = rust_lib_header("ProtocolPermissionValidator")
    rust += "pub const ACTIVE_PERMISSION_FIELDS: &[&str] = &[\n"
    for f in fields:
        rust += "    \"{}\",\n".format(f)
    rust += "];\n\n"
    rust += "pub fn is_active_field(name: &str) -> bool {\n    ACTIVE_PERMISSION_FIELDS.contains(&name)\n}\n"
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["ProtocolPermissionValidator"]))
    cs = (
        "namespace Lumio.Gen.ProtocolPermissionValidator;\n\n"
        "public static class ActivePermissionFields\n{\n"
        "    public static readonly string[] Names = new[]\n    {\n"
        + "".join("        \"{}\",\n".format(f) for f in fields)
        + "    };\n}\n"
    )
    write_text(cs_dir / "ActivePermissionFields.cs", cs)
    write_text(cs_dir / (CS_PROJ["ProtocolPermissionValidator"] + ".csproj"), csproj(CS_PROJ["ProtocolPermissionValidator"]))


def emit_mapping(rust_dir: Path, cs_dir: Path) -> None:
    rust = rust_lib_header("MappingTable")
    rust += (
        "pub const MAPPING_ROLES: &[&str] = &[\"ServerToClient\", \"ClientToServer\", \"SharedProjection\"];\n"
        "pub const MAPPING_REQUIRED: &[&str] = &[\"mappingId\", \"schemaVersion\", \"source\", \"target\", \"role\", \"owner\", \"visibility\", \"delivery\", \"lifecycle\", \"prediction\"];\n"
    )
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["MappingTable"]))
    write_text(
        cs_dir / "MappingTable.cs",
        "namespace Lumio.Gen.MappingTable;\n\npublic static class MappingContract\n{\n"
        "    public static readonly string[] Roles = { \"ServerToClient\", \"ClientToServer\", \"SharedProjection\" };\n}\n",
    )
    write_text(cs_dir / (CS_PROJ["MappingTable"] + ".csproj"), csproj(CS_PROJ["MappingTable"]))


def emit_canonical(rust_dir: Path, cs_dir: Path, profile: Dict[str, Any]) -> None:
    rust = rust_lib_header("CanonicalSerializer")
    rust += (
        "/// snapshot-header.checksum covers SHA-256 of the canonical JSON of the header\n"
        "/// object with `checksum` and `hash` omitted (UTF-8, sorted keys, no extra whitespace).\n"
        "pub const SNAPSHOT_CHECKSUM_OMIT: &[&str] = &[\"checksum\", \"hash\"];\n"
        "pub const SNAPSHOT_MAGIC: &str = \"LUMIOSNP1\";\n\n"
        "pub fn checksum_domain_doc() -> &'static str {\n"
        "    \"SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields\"\n"
        "}\n"
    )
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["CanonicalSerializer"]))
    rust += "\n"
    rust += "/// ADR-041 CanonicalJsonV1: the canonical form is defined by the architecture source,\n"
    rust += "/// never inherited from a generic JCS library's defaults.\n"
    form = profile["canonicalForm"]
    rust += "pub const CANONICAL_FORM_ID: &str = \"%s\";\n" % form["formId"]
    rust += "pub const CANONICAL_ENCODING: &str = \"%s\";\n" % form["encoding"]
    rust += "pub const CANONICAL_MEMBER_ORDER: &str = \"%s\";\n" % form["memberOrder"]
    rust += "pub const CANONICAL_ARRAY_ORDER: &str = \"%s\";\n" % form["arrayOrder"]
    rust += "pub const CANONICAL_ITEM_SEPARATOR: char = ',';\n"
    rust += "pub const CANONICAL_KEY_VALUE_SEPARATOR: char = ':';\n"
    rust += "pub const CANONICAL_NUMBERS: &str = \"%s\";\n" % form["numbers"]
    rust += "pub const CANONICAL_UNKNOWN_MEMBERS: &str = \"%s\";\n" % form["unknownMembers"]
    rust += "pub const CANONICAL_DUPLICATE_MEMBERS: &str = \"%s\";\n" % form["duplicateMembers"]
    rust += "pub const DIGEST_ALGORITHM: &str = \"%s\";\n" % profile["digestAlgorithm"]["name"]
    rust += "pub const DIGEST_FRAMING: &str = \"%s\";\n\n" % profile["digestAlgorithm"]["framing"]
    rust += (
        "#[derive(Clone, Copy, Debug)]\n"
        "pub struct DigestDomain {\n"
        "    pub digest: &'static str,\n"
        "    pub domain_tag: &'static str,\n"
        "    pub sort_rule: &'static str,\n"
        "    pub omit_members: &'static [&'static str],\n"
        "}\n\n"
    )
    rust += "pub const DIGEST_DOMAINS: &[DigestDomain] = &[\n"
    for domain in profile["digestDomains"]:
        omit = ", ".join("\"%s\"" % name for name in domain.get("omitMembers", []))
        rust += (
            "    DigestDomain { digest: \"%s\", domain_tag: \"%s\", sort_rule: \"%s\", omit_members: &[%s] },\n"
            % (domain["digest"], domain["domainTag"], domain["sortRule"], omit)
        )
    rust += "];\n\n"
    rust += "/// Golden vectors: `(id, case, sha256)`. Full inputs and canonical bytes are in\n"
    rust += "/// the published `canonical/canonical-digest-profile.json`.\n"
    rust += "pub const CANONICAL_GOLDENS: &[(&str, &str, &str)] = &[\n"
    for golden in profile["goldens"]:
        rust += "    (\"%s\", \"%s\", \"%s\"),\n" % (golden["id"], golden["case"], golden["sha256"])
    rust += "];\n"
    write_text(rust_dir / "CHECKSUM_DOMAIN.md", "checksum = SHA-256(canonical_json(header without checksum,hash))\n")
    write_text(
        cs_dir / "CanonicalSerializer.cs",
        "namespace Lumio.Gen.CanonicalSerializer;\n\npublic static class SnapshotChecksum\n{\n"
        "    public const string Domain = \"SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields\";\n"
        "    public const string Magic = \"LUMIOSNP1\";\n}\n",
    )
    cs_extra = ["\n"]
    cs_extra.append("public static class CanonicalForm\n{\n")
    cs_extra.append("    public const string FormId = \"%s\";\n" % form["formId"])
    cs_extra.append("    public const string Encoding = \"%s\";\n" % form["encoding"])
    cs_extra.append("    public const string MemberOrder = \"%s\";\n" % form["memberOrder"])
    cs_extra.append("    public const string ArrayOrder = \"%s\";\n" % form["arrayOrder"])
    cs_extra.append("    public const char ItemSeparator = ',';\n")
    cs_extra.append("    public const char KeyValueSeparator = ':';\n")
    cs_extra.append("    public const string Numbers = \"%s\";\n" % form["numbers"])
    cs_extra.append("    public const string UnknownMembers = \"%s\";\n" % form["unknownMembers"])
    cs_extra.append("    public const string DuplicateMembers = \"%s\";\n" % form["duplicateMembers"])
    cs_extra.append("    public const string DigestAlgorithm = \"%s\";\n" % profile["digestAlgorithm"]["name"])
    cs_extra.append("    public const string DigestFraming = \"%s\";\n}\n\n" % profile["digestAlgorithm"]["framing"])
    cs_extra.append("public readonly record struct DigestDomain(string Digest, string DomainTag, string SortRule, string[] OmitMembers);\n")
    cs_extra.append("public static class DigestDomains\n{\n    public static readonly DigestDomain[] All =\n    {\n")
    for domain in profile["digestDomains"]:
        omit = ", ".join("\"%s\"" % name for name in domain.get("omitMembers", []))
        cs_extra.append(
            "        new DigestDomain(\"%s\", \"%s\", \"%s\", new[] { %s }),\n"
            % (domain["digest"], domain["domainTag"], domain["sortRule"], omit)
            if omit
            else "        new DigestDomain(\"%s\", \"%s\", \"%s\", System.Array.Empty<string>()),\n"
            % (domain["digest"], domain["domainTag"], domain["sortRule"])
        )
    cs_extra.append("    };\n}\n\n")
    cs_extra.append("public readonly record struct CanonicalGolden(string Id, string Case, string Sha256);\n")
    cs_extra.append("public static class CanonicalGoldens\n{\n    public static readonly CanonicalGolden[] All =\n    {\n")
    for golden in profile["goldens"]:
        cs_extra.append(
            "        new CanonicalGolden(\"%s\", \"%s\", \"%s\"),\n"
            % (golden["id"], golden["case"], golden["sha256"])
        )
    cs_extra.append("    };\n}\n")
    write_text(cs_dir / "CanonicalProfile.cs", "using System;\n\nnamespace Lumio.Gen.CanonicalSerializer;\n" + "".join(cs_extra))
    write_text(cs_dir / (CS_PROJ["CanonicalSerializer"] + ".csproj"), csproj(CS_PROJ["CanonicalSerializer"]))


def emit_language_binding(rust_dir: Path, cs_dir: Path, schema_ids: List[str]) -> None:
    rust = rust_lib_header("LanguageBinding")
    rust += "pub mod root_abi;\n\n"
    rust += (
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n"
        "pub struct Binding {\n"
        "    pub schema_id: &'static str,\n"
        "    pub rust_type: &'static str,\n"
        "    pub csharp_type: &'static str,\n"
        "}\n\n"
    )
    rust += "pub const BINDINGS: &[Binding] = &[\n"
    for sid in schema_ids:
        pascal = "".join(p.title() for p in sid.replace("_", "-").split("-"))
        rust += "    Binding { schema_id: \"%s\", rust_type: \"%s\", csharp_type: \"%s\" },\n" % (
            sid,
            pascal,
            pascal,
        )
    rust += "];\n"
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["LanguageBinding"]))
    cs_lines = [
        "namespace Lumio.Gen.LanguageBinding;\n",
        "public readonly record struct Binding(string SchemaId, string RustType, string CsharpType);\n",
        "public static class Bindings\n{\n    public static readonly Binding[] All =\n    {\n",
    ]
    for sid in schema_ids:
        pascal = "".join(p.title() for p in sid.replace("_", "-").split("-"))
        cs_lines.append("        new Binding(\"%s\", \"%s\", \"%s\"),\n" % (sid, pascal, pascal))
    cs_lines.append("    };\n}\n")
    write_text(cs_dir / "Bindings.cs", "".join(cs_lines))
    write_text(cs_dir / (CS_PROJ["LanguageBinding"] + ".csproj"), csproj(CS_PROJ["LanguageBinding"]))


def emit_contract_types(
    rust_dir: Path,
    cs_dir: Path,
    machines: List[Dict[str, Any]],
    schema_ids: List[str],
    errors: List[str],
    abi: Dict[str, Any],
) -> None:
    rust = rust_lib_header("ContractTypes")
    rust += "pub const BASELINE_ID: &str = \"%s\";\n" % BASELINE
    rust += "pub const SCHEMA_IDS: &[&str] = &[\n"
    for sid in schema_ids:
        rust += "    \"%s\",\n" % sid
    rust += "];\n\n"
    rust += "pub const STABLE_ERROR_IDS: &[&str] = &[\n"
    for e in errors:
        rust += "    \"%s\",\n" % e
    rust += "];\n\n"
    rust += (
        "pub const VOXEL_WORLD_ROLES: &[&str] = &[\"Authority\", \"Replica\"];\n"
        "pub const CHUNK_PRESENCE: &[&str] = &[\"Ready\", \"NotLoaded\", \"Pending\", \"Unavailable\"];\n\n"
        "#[derive(Clone, Copy, Debug)]\n"
        "pub struct Transition {\n"
        "    pub machine: &'static str,\n"
        "    pub from: &'static str,\n"
        "    pub to: &'static str,\n"
        "    pub event: &'static str,\n"
        "}\n\n"
        "pub fn state_transition_table() -> &'static [Transition] {\n    TRANSITIONS\n}\n\n"
        "pub fn machine_ids() -> impl Iterator<Item = &'static str> {\n"
        "    MACHINE_IDS.iter().copied()\n}\n"
    )
    machine_ids: List[str] = []
    seen = set()
    for machine in machines:
        mid = str(machine.get("machineId", ""))
        if mid and mid not in seen:
            seen.add(mid)
            machine_ids.append(mid)
    machine_ids.sort()
    rust += "pub const ABI_ENTRY_SYMBOL: &str = \"%s\";\n" % abi["entrySymbol"]
    rust += "pub const ABI_SYMBOL_PREFIX: &str = \"%s\";\n" % abi["symbolPrefix"]
    rust += "pub const ABI_VERSION: u32 = %d;\n" % abi["abiVersion"]
    rust += "pub const ABI_CALLING_CONVENTION: &str = \"%s\";\n" % abi["callingConvention"]
    rust += "pub const ABI_POINTER_WIDTH: u32 = %d;\n" % abi["pointerWidth"]
    rust += "pub const ABI_ENDIANNESS: &str = \"%s\";\n\n" % abi["endianness"]
    rust += (
        "#[derive(Clone, Copy, Debug)]\n"
        "pub struct AbiTypeMapping {\n"
        "    pub type_ref: &'static str,\n"
        "    pub c: &'static str,\n"
        "    pub csharp: &'static str,\n"
        "    pub rust: &'static str,\n"
        "    pub size: usize,\n"
        "    pub align: usize,\n"
        "}\n\n"
    )
    rust += "pub const ABI_TYPE_MAPPING: &[AbiTypeMapping] = &[\n"
    for key, c_type, cs_type, rust_type, size, align in ABI_TYPE_MAPPING:
        rust += (
            "    AbiTypeMapping { type_ref: \"%s\", c: \"%s\", csharp: \"%s\", rust: \"%s\", size: %d, align: %d },\n"
            % (key, c_type, cs_type, rust_type, size, align)
        )
    rust += "];\n\n"
    rust += "pub const MACHINE_IDS: &[&str] = &[\n"
    for mid in machine_ids:
        rust += "    \"%s\",\n" % mid
    rust += "];\n\n"
    rust += "const TRANSITIONS: &[Transition] = &[\n"
    for machine in machines:
        mid = machine.get("machineId", "")
        for tr in machine.get("transitions") or []:
            rust += "    Transition { machine: \"%s\", from: \"%s\", to: \"%s\", event: \"%s\" },\n" % (
                mid,
                tr.get("from", ""),
                tr.get("to", ""),
                tr.get("event", ""),
            )
    rust += "];\n"
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["ContractTypes"]))
    # C#
    cs = ["namespace Lumio.Gen.ContractTypes;\n\n"]
    cs.append("public static class Catalog\n{\n    public const string BaselineId = \"%s\";\n" % BASELINE)
    cs.append("    public static readonly string[] SchemaIds = { %s };\n" % ", ".join("\"%s\"" % s for s in schema_ids))
    cs.append("    public static readonly string[] StableErrorIds = { %s };\n" % ", ".join("\"%s\"" % e for e in errors))
    cs.append("    public static readonly string[] ChunkPresence = { \"Ready\", \"NotLoaded\", \"Pending\", \"Unavailable\" };\n")
    cs.append("    public static readonly string[] VoxelWorldRoles = { \"Authority\", \"Replica\" };\n")
    cs.append("    public const string AbiEntrySymbol = \"%s\";\n" % abi["entrySymbol"])
    cs.append("    public const string AbiSymbolPrefix = \"%s\";\n" % abi["symbolPrefix"])
    cs.append("    public const uint AbiVersion = %d;\n" % abi["abiVersion"])
    cs.append("    public const string AbiCallingConvention = \"%s\";\n" % abi["callingConvention"])
    cs.append("    public const uint AbiPointerWidth = %d;\n" % abi["pointerWidth"])
    cs.append("    public const string AbiEndianness = \"%s\";\n}\n\n" % abi["endianness"])
    cs.append("public readonly record struct AbiTypeMapping(string TypeRef, string C, string Csharp, string Rust, int Size, int Align);\n")
    cs.append("public static class AbiTypeMappings\n{\n    public static readonly AbiTypeMapping[] All =\n    {\n")
    for key, c_type, cs_type, rust_type, size, align in ABI_TYPE_MAPPING:
        cs.append(
            "        new AbiTypeMapping(\"%s\", \"%s\", \"%s\", \"%s\", %d, %d),\n"
            % (key, c_type, cs_type, rust_type, size, align)
        )
    cs.append("    };\n}\n\n")
    cs.append("public readonly record struct Transition(string Machine, string From, string To, string Event);\n")
    cs.append("public static class StateTransitionTable\n{\n    public static readonly Transition[] All =\n    {\n")
    for machine in machines:
        mid = machine.get("machineId", "")
        for tr in machine.get("transitions") or []:
            cs.append(
                "        new Transition(\"%s\", \"%s\", \"%s\", \"%s\"),\n"
                % (mid, tr.get("from", ""), tr.get("to", ""), tr.get("event", ""))
            )
    cs.append("    };\n}\n")
    write_text(cs_dir / "ContractTypes.cs", "".join(cs))
    write_text(cs_dir / (CS_PROJ["ContractTypes"] + ".csproj"), csproj(CS_PROJ["ContractTypes"]))


def emit_contract_runtime(rust_dir: Path, cs_dir: Path) -> None:
    rust = rust_lib_header("ContractRuntime")
    rust += (
        "mod sha256;\n"
        "pub use sha256::{sha256, sha256_hex};\n\n"
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n"
        "pub struct Hash256(pub [u8; 32]);\n\n"
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n"
        "pub enum ChainBreak { Truncated, Mismatch }\n\n"
        "pub fn hash_chain_append(prev: &Hash256, payload: &[u8]) -> Hash256 {\n"
        "    let mut buf = Vec::with_capacity(32 + payload.len());\n"
        "    buf.extend_from_slice(&prev.0);\n"
        "    buf.extend_from_slice(payload);\n"
        "    Hash256(sha256(&buf))\n"
        "}\n\n"
        "pub fn hash_chain_verify(prev: &Hash256, payload: &[u8], expected: &Hash256) -> Result<(), ChainBreak> {\n"
        "    let got = hash_chain_append(prev, payload);\n"
        "    if got.0 == expected.0 { Ok(()) } else { Err(ChainBreak::Mismatch) }\n"
        "}\n\n"
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n"
        "pub struct BufferFull;\n\n"
        "pub struct BoundedBuffer { inner: Vec<u8>, cap: usize }\n"
        "impl BoundedBuffer {\n"
        "    pub fn new(cap: usize) -> Self { Self { inner: Vec::new(), cap } }\n"
        "    pub fn push(&mut self, byte: u8) -> Result<(), BufferFull> {\n"
        "        if self.inner.len() >= self.cap { return Err(BufferFull); }\n"
        "        self.inner.push(byte); Ok(())\n"
        "    }\n"
        "    pub fn as_slice(&self) -> &[u8] { &self.inner }\n"
        "}\n\n"
        "pub fn canonical_object_pairs(pairs: &mut [(String, String)]) -> String {\n"
        "    pairs.sort_by(|a, b| a.0.cmp(&b.0));\n"
        "    let mut out = String::from(\"{\");\n"
        "    for (i, (k, v)) in pairs.iter().enumerate() {\n"
        "        if i > 0 { out.push(','); }\n"
        "        out.push('\"');\n"
        "        out.push_str(k);\n"
        "        out.push_str(\"\\\":\");\n"
        "        out.push_str(v);\n"
        "    }\n"
        "    out.push('}');\n"
        "    out\n"
        "}\n"
    )
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "src" / "sha256.rs", "#![allow(clippy::all)]\n" + SHA256_RS)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["ContractRuntime"]))
    write_text(
        rust_dir / "tests" / "chain.rs",
        "use lumio_gen_contract_runtime::*;\n"
        "#[test]\nfn chain_round_trip() {\n"
        "    let genesis = Hash256(sha256(b\"\"));\n"
        "    let next = hash_chain_append(&genesis, b\"rec-1\");\n"
        "    assert!(hash_chain_verify(&genesis, b\"rec-1\", &next).is_ok());\n"
        "    assert!(hash_chain_verify(&genesis, b\"rec-2\", &next).is_err());\n"
        "}\n"
        "#[test]\nfn truncated_buffer() {\n"
        "    let mut buf = BoundedBuffer::new(2);\n"
        "    assert!(buf.push(1).is_ok());\n"
        "    assert!(buf.push(2).is_ok());\n"
        "    assert!(buf.push(3).is_err());\n"
        "}\n",
    )
    write_text(
        cs_dir / "ContractRuntime.cs",
        "using System;\nusing System.Security.Cryptography;\nusing System.Text;\n\n"
        "namespace Lumio.Gen.ContractRuntime;\n\n"
        "public enum ChainBreak { Truncated, Mismatch }\n\n"
        "public static class HashChain\n{\n"
        "    public static byte[] Append(byte[] prev, byte[] payload)\n    {\n"
        "        var buf = new byte[prev.Length + payload.Length];\n"
        "        Buffer.BlockCopy(prev, 0, buf, 0, prev.Length);\n"
        "        Buffer.BlockCopy(payload, 0, buf, prev.Length, payload.Length);\n"
        "        return SHA256.HashData(buf);\n    }\n"
        "    public static bool Verify(byte[] prev, byte[] payload, byte[] expected)\n    {\n"
        "        var got = Append(prev, payload);\n"
        "        return got.AsSpan().SequenceEqual(expected);\n    }\n"
        "    public static byte[] Sha256(byte[] data) => SHA256.HashData(data);\n}\n\n"
        "public sealed class BoundedBuffer\n{\n"
        "    private readonly byte[] _data; private int _len;\n"
        "    public BoundedBuffer(int cap) { _data = new byte[cap]; }\n"
        "    public bool TryPush(byte b) { if (_len >= _data.Length) return false; _data[_len++] = b; return true; }\n"
        "    public int Length => _len;\n"
        "}\n\n"
        "public static class SelfTest\n{\n"
        "    public static void HashChainRoundTrip()\n    {\n"
        "        var genesis = HashChain.Sha256(Array.Empty<byte>());\n"
        "        var next = HashChain.Append(genesis, System.Text.Encoding.UTF8.GetBytes(\"rec-1\"));\n"
        "        if (!HashChain.Verify(genesis, System.Text.Encoding.UTF8.GetBytes(\"rec-1\"), next))\n"
        "            throw new InvalidOperationException(\"hash chain round-trip failed\");\n"
        "        if (HashChain.Verify(genesis, System.Text.Encoding.UTF8.GetBytes(\"rec-2\"), next))\n"
        "            throw new InvalidOperationException(\"hash chain must reject a mutated payload\");\n"
        "    }\n"
        "    public static void TruncatedBuffer()\n    {\n"
        "        var buf = new BoundedBuffer(2);\n"
        "        if (!buf.TryPush(1) || !buf.TryPush(2) || buf.TryPush(3))\n"
        "            throw new InvalidOperationException(\"bounded buffer did not truncate\");\n"
        "    }\n"
        "}\n",
    )
    write_text(cs_dir / (CS_PROJ["ContractRuntime"] + ".csproj"), csproj(CS_PROJ["ContractRuntime"]))


# --------------------------------------------------------------------------
# ADR-041 Canonical and Digest Profiles
# --------------------------------------------------------------------------

CANONICAL_PROFILE_ID = "canonical-digest-v1"
CANONICAL_PROFILE_FILE = "canonical/canonical-digest-profile.json"
CANONICAL_FORM = {
    "formId": "CanonicalJsonV1",
    "encoding": "AsciiEscaped",
    "memberOrder": "CodePointAscending",
    "arrayOrder": "DocumentOrder",
    "separators": {"item": ",", "keyValue": ":"},
    "numbers": "IntegerOnly",
    "unknownMembers": "Reject",
    "duplicateMembers": "Reject",
}
CANONICAL_DIGEST_ALGORITHM = {"name": "SHA-256", "framing": "PrefixFreeOverCanonicalBytes"}
CANONICAL_DIGEST_DOMAINS = [
    {
        "digest": "manifestDigest",
        "domainTag": "CoreEngineManifestBody",
        "input": "the CoreEngineManifestBody document itself (ADR-018; the one input with no digestDomain member)",
        "sortRule": "member order only; the body has no array whose order is semantic",
    },
    {
        "digest": "artifactSetDigest",
        "domainTag": "ArtifactSetV1",
        "input": "the ArtifactIndex with artifactSetDigest omitted, wrapped as {digestDomain,indexVersion,targetProfileId,entries}",
        "sortRule": "entries sorted ascending by path (code point); paths are unique within an index",
        "omitMembers": ["artifactSetDigest"],
    },
    {
        "digest": "artifactIndexDigest",
        "domainTag": "ArtifactIndexV1",
        "input": "the complete ArtifactIndex document including artifactSetDigest, wrapped as {digestDomain,index}",
        "sortRule": "index.entries sorted ascending by path (code point)",
    },
    {
        "digest": "targetProfileDigest",
        "domainTag": "TargetProfileV1",
        "input": "the complete TargetProfile document, wrapped as {digestDomain,profile}",
        "sortRule": "member order only; the profile has no array",
    },
    {
        "digest": "capabilitySetDigest",
        "domainTag": "CapabilitySetV1",
        "input": "the capability id list, wrapped as {digestDomain,capabilities}",
        "sortRule": "capabilities sorted ascending by code point; the array is uniqueItems so ties are impossible",
    },
]
CANONICAL_DOMAIN_TAGS = {item["domainTag"] for item in CANONICAL_DIGEST_DOMAINS}
CANONICAL_GOLDEN_CASES = [
    "EmptyArtifactSet",
    "SingleArtifact",
    "MultiArtifact",
    "PathOrderPermutation",
    "CapabilityOrderPermutation",
    "EscapeBoundary",
    "IntegerBoundary",
    "SchemaVersionChange",
]


class CanonicalError(RuntimeError):
    """Raised when a value cannot be put into CanonicalJsonV1 form."""


def assert_canonicalizable(value: Any, path: str = "$") -> None:
    """ADR-041 section 1: integers only, no float, no non-string member name."""
    if value is None or isinstance(value, bool) or isinstance(value, (str, int)):
        return
    if isinstance(value, float):
        raise CanonicalError("{} is a non-integer number; CanonicalJsonV1 is integer-only".format(path))
    if isinstance(value, list):
        for index, item in enumerate(value):
            assert_canonicalizable(item, "{}[{}]".format(path, index))
        return
    if isinstance(value, dict):
        for key, item in value.items():
            if not isinstance(key, str):
                raise CanonicalError("{} has a non-string member name".format(path))
            assert_canonicalizable(item, "{}.{}".format(path, key))
        return
    raise CanonicalError("{} has a type CanonicalJsonV1 does not define".format(path))


def apply_digest_domain_sort(value: Any) -> Any:
    """Apply the ADR-041 section 3 sort rule for a recognised digest domain."""
    if not isinstance(value, dict):
        return value
    tag = value.get("digestDomain")
    if tag not in CANONICAL_DOMAIN_TAGS:
        return value
    out = dict(value)
    if tag == "ArtifactSetV1" and isinstance(out.get("entries"), list):
        out["entries"] = sorted(out["entries"], key=lambda entry: str(entry.get("path", "")))
    elif tag == "ArtifactIndexV1" and isinstance(out.get("index"), dict):
        index = dict(out["index"])
        if isinstance(index.get("entries"), list):
            index["entries"] = sorted(index["entries"], key=lambda entry: str(entry.get("path", "")))
        out["index"] = index
    elif tag == "CapabilitySetV1" and isinstance(out.get("capabilities"), list):
        out["capabilities"] = sorted(out["capabilities"], key=str)
    return out


def canonical_bytes(value: Any) -> str:
    """CanonicalJsonV1 bytes of a digest input, with the domain sort applied first."""
    prepared = apply_digest_domain_sort(value)
    assert_canonicalizable(prepared)
    return canonical_json(prepared)


def canonical_digest(value: Any) -> str:
    return sha256_bytes(canonical_bytes(value).encode("ascii"))


def artifact_set_digest_input(index: Dict[str, Any]) -> Dict[str, Any]:
    return {
        "digestDomain": "ArtifactSetV1",
        "indexVersion": index.get("indexVersion"),
        "targetProfileId": index.get("targetProfileId"),
        "entries": list(index.get("entries") or []),
    }


def artifact_index_digest_input(index: Dict[str, Any]) -> Dict[str, Any]:
    return {"digestDomain": "ArtifactIndexV1", "index": dict(index)}


def target_profile_digest_input(profile: Dict[str, Any]) -> Dict[str, Any]:
    return {"digestDomain": "TargetProfileV1", "profile": dict(profile)}


def capability_set_digest_input(capabilities: List[str]) -> Dict[str, Any]:
    return {"digestDomain": "CapabilitySetV1", "capabilities": list(capabilities)}


def _golden(golden_id: str, case: str, value: Any) -> Dict[str, Any]:
    text = canonical_bytes(value)
    return {
        "id": golden_id,
        "case": case,
        "input": value,
        "canonicalBytes": text,
        "sha256": sha256_bytes(text.encode("ascii")),
    }


ESCAPE_BOUNDARY_SAMPLE = "\"\\/\b\f\n\r\t\u001f\u00e9\u4e16\U0001d11e"  # quote, backslash, solidus, shorthand controls, C0, Latin-1, BMP, astral


def derive_canonical_profile(root: Path) -> Dict[str, Any]:
    """Derive the ADR-041 profile and its self-verifying Golden vectors."""
    index = json.loads((root / "fixtures" / "valid" / "artifact-index.json").read_text(encoding="utf-8"))
    entries = list(index.get("entries") or [])
    goldens = [
        _golden(
            "artifact-set-empty",
            "EmptyArtifactSet",
            {
                "digestDomain": "ArtifactSetV1",
                "indexVersion": 1,
                "targetProfileId": index.get("targetProfileId"),
                "entries": [],
            },
        ),
        _golden(
            "artifact-set-single",
            "SingleArtifact",
            {
                "digestDomain": "ArtifactSetV1",
                "indexVersion": 1,
                "targetProfileId": index.get("targetProfileId"),
                "entries": entries[:1],
            },
        ),
        _golden("artifact-set-multi", "MultiArtifact", artifact_set_digest_input(index)),
        _golden(
            "artifact-set-path-permutation",
            "PathOrderPermutation",
            {
                "digestDomain": "ArtifactSetV1",
                "indexVersion": index.get("indexVersion"),
                "targetProfileId": index.get("targetProfileId"),
                "entries": list(reversed(entries)),
            },
        ),
        _golden(
            "capability-set-permutation",
            "CapabilityOrderPermutation",
            capability_set_digest_input(["VoxelStreaming", "Native", "ReferenceVoxel", "VoxelSnapshot"]),
        ),
        _golden(
            "escape-boundary",
            "EscapeBoundary",
            {"sample": ESCAPE_BOUNDARY_SAMPLE, "empty": ""},
        ),
        _golden(
            "integer-boundary",
            "IntegerBoundary",
            {"sample": [0, 1, -1, 9007199254740993, 18446744073709551615, -9007199254740993]},
        ),
        _golden(
            "artifact-set-schema-version",
            "SchemaVersionChange",
            {
                "digestDomain": "ArtifactSetV1",
                "indexVersion": 2,
                "targetProfileId": index.get("targetProfileId"),
                "entries": entries[:1],
            },
        ),
    ]
    return {
        "profileId": CANONICAL_PROFILE_ID,
        "baselineId": BASELINE,
        "schemaEpoch": 1,
        "canonicalForm": json.loads(json.dumps(CANONICAL_FORM)),
        "digestAlgorithm": dict(CANONICAL_DIGEST_ALGORITHM),
        "digestDomains": json.loads(json.dumps(CANONICAL_DIGEST_DOMAINS)),
        "goldens": goldens,
    }


def emit_canonical_profile(root: Path, out_dir: Path) -> Dict[str, Any]:
    profile = derive_canonical_profile(root)
    write_text(out_dir / CANONICAL_PROFILE_FILE, canonical_json(profile) + "\n")
    return profile


# --------------------------------------------------------------------------
# ADR-040 Root ABI Generated Bundle
# --------------------------------------------------------------------------

ABI_COMPILER_NAME = "lumio-abi-compiler"
ABI_COMPILER_VERSION = "1.0.0"
ABI_BUNDLE_ID = "root-abi-v1"
ABI_INPUT_SET = [
    "schemas/native-managed-abi.schema.json",
    "fixtures/valid/native-managed-abi.json",
]
ABI_DOCUMENT = "fixtures/valid/native-managed-abi.json"
LAYOUT_PROFILE = {
    "targetProfileId": "linux-x86_64-glibc",
    "os": "LinuxServer",
    "arch": "x86_64",
    "abiRuntime": "glibc",
    "pointerBytes": 8,
    "maxAlignment": 8,
    "rootHeaderBytes": 16,
    "tableHeaderBytes": 16,
}
ABI_OUTPUT_FILES = [
    ("abi/lumio_core.h", "CHeader"),
    ("rust/lumio-gen-language-binding/src/root_abi.rs", "RustBinding"),
    ("csharp/Lumio.Gen.LanguageBinding/RootAbi.cs", "CSharpBinding"),
]
ABI_BUNDLE_FILE = "abi/root-abi-bundle.json"

# typeRef production -> (C, C#, Rust, size, align). Frozen by ADR-040 section 3;
# the parametric families use their grammar spelling as the key.
ABI_TYPE_MAPPING: List[Tuple[str, str, str, str, int, int]] = [
    ("u8", "uint8_t", "byte", "u8", 1, 1),
    ("u16", "uint16_t", "ushort", "u16", 2, 2),
    ("u32", "uint32_t", "uint", "u32", 4, 4),
    ("u64", "uint64_t", "ulong", "u64", 8, 8),
    ("i8", "int8_t", "sbyte", "i8", 1, 1),
    ("i16", "int16_t", "short", "i16", 2, 2),
    ("i32", "int32_t", "int", "i32", 4, 4),
    ("i64", "int64_t", "long", "i64", 8, 8),
    ("f32", "float", "float", "f32", 4, 4),
    ("f64", "double", "double", "f64", 8, 8),
    ("bool32", "uint32_t", "uint", "u32", 4, 4),
    ("status", "lumio_status_t", "LumioStatus", "LumioStatus", 4, 4),
    ("handle:<kind>", "lumio_handle_t", "LumioHandle", "LumioHandle", 16, 8),
    ("buffer:in", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
    ("buffer:out", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
    ("buffer:inout", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
    ("struct:<name>:v<N>", "const lumio_<name>_v<N>*", "IntPtr", "*const Lumio<Name>V<N>", 8, 8),
    ("ptr:const:<name>", "const lumio_<name>*", "IntPtr", "*const Lumio<Name>", 8, 8),
    ("ptr:mut:<name>", "lumio_<name>*", "IntPtr", "*mut Lumio<Name>", 8, 8),
]
ABI_TYPE_MAPPING_KEYS = [row[0] for row in ABI_TYPE_MAPPING]

ROOT_FIELDS = [
    ("abi_version", 0, 4, "uint32_t", "uint", "u32"),
    ("struct_size", 4, 4, "uint32_t", "uint", "u32"),
    ("capability_bits", 8, 8, "uint64_t", "ulong", "u64"),
]
TABLE_FIELDS = [
    ("version", 0, 4, "uint32_t", "uint", "u32"),
    ("struct_size", 4, 4, "uint32_t", "uint", "u32"),
    ("reserved0", 8, 8, "uint64_t", "ulong", "u64"),
]


class AbiError(RuntimeError):
    """Raised when the ABI document cannot produce a bundle."""


def pascal(snake: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in snake.split("_") if part)


def abi_struct_parts(type_ref: str) -> Tuple[str, str]:
    """Split `struct:<name>:v<N>` into (name, version-suffix)."""
    _, name, version = type_ref.split(":", 2)
    return name, version


def abi_c_type(type_ref: str) -> str:
    if type_ref.startswith("handle:"):
        return "lumio_handle_t"
    if type_ref.startswith("buffer:"):
        return "lumio_buffer_t"
    if type_ref.startswith("struct:"):
        name, version = abi_struct_parts(type_ref)
        return "const struct lumio_{}_{}*".format(name, version)
    if type_ref.startswith("ptr:const:"):
        return "const struct lumio_{}*".format(type_ref.split(":", 2)[2])
    if type_ref.startswith("ptr:mut:"):
        return "struct lumio_{}*".format(type_ref.split(":", 2)[2])
    for key, c_type, _cs, _rs, _size, _align in ABI_TYPE_MAPPING:
        if key == type_ref:
            return c_type
    raise AbiError("typeRef {} is outside the ADR-017 grammar".format(type_ref))


def abi_cs_type(type_ref: str) -> str:
    if type_ref.startswith("handle:"):
        return "LumioHandle"
    if type_ref.startswith("buffer:"):
        return "LumioBuffer"
    if type_ref.startswith(("struct:", "ptr:")):
        return "IntPtr"
    for key, _c, cs_type, _rs, _size, _align in ABI_TYPE_MAPPING:
        if key == type_ref:
            return cs_type
    raise AbiError("typeRef {} is outside the ADR-017 grammar".format(type_ref))


def abi_rust_type(type_ref: str) -> str:
    if type_ref.startswith("handle:"):
        return "LumioHandle"
    if type_ref.startswith("buffer:"):
        return "LumioBuffer"
    if type_ref.startswith("struct:"):
        name, version = abi_struct_parts(type_ref)
        return "*const Lumio{}{}".format(pascal(name), version.upper())
    if type_ref.startswith("ptr:const:"):
        return "*const Lumio{}".format(pascal(type_ref.split(":", 2)[2]))
    if type_ref.startswith("ptr:mut:"):
        return "*mut Lumio{}".format(pascal(type_ref.split(":", 2)[2]))
    for key, _c, _cs, rust_type, _size, _align in ABI_TYPE_MAPPING:
        if key == type_ref:
            return rust_type
    raise AbiError("typeRef {} is outside the ADR-017 grammar".format(type_ref))


def abi_opaque_types(abi: Dict[str, Any]) -> List[Tuple[str, str, str]]:
    """Collect (c_tag, rust_name, origin) for every named struct/ptr target."""
    seen: Dict[str, Tuple[str, str, str]] = {}
    for table in abi.get("apiTable", []):
        for slot in table.get("slots", []):
            refs = [slot.get("returns", "")] + [p.get("type", "") for p in slot.get("params", [])]
            for ref in refs:
                if ref.startswith("struct:"):
                    name, version = abi_struct_parts(ref)
                    tag = "lumio_{}_{}".format(name, version)
                    seen[tag] = (tag, "Lumio{}{}".format(pascal(name), version.upper()), ref)
                elif ref.startswith("ptr:"):
                    name = ref.split(":", 2)[2]
                    tag = "lumio_{}".format(name)
                    seen[tag] = (tag, "Lumio{}".format(pascal(name)), ref)
    return [seen[key] for key in sorted(seen)]


def abi_minimum_table_size(table: Dict[str, Any], profile: Dict[str, Any]) -> int:
    slots = int(table.get("functionCount", 0)) + int(table.get("reservedSlots", 0))
    return int(profile["tableHeaderBytes"]) + slots * int(profile["pointerBytes"])


def abi_minimum_root_size(abi: Dict[str, Any], profile: Dict[str, Any]) -> int:
    tables = len(abi.get("apiTable", []))
    return int(profile["rootHeaderBytes"]) + tables * int(profile["pointerBytes"])


def abi_c_signature(slot: Dict[str, Any]) -> str:
    params = ", ".join(
        "{} {}".format(abi_c_type(p["type"]), p["name"]) for p in slot.get("params", [])
    ) or "void"
    return "{} (*{})({})".format(abi_c_type(slot["returns"]), slot["name"], params)


def abi_cs_signature(slot: Dict[str, Any]) -> str:
    params = ", ".join(
        "{} {}".format(abi_cs_type(p["type"]), p["name"]) for p in slot.get("params", [])
    )
    return "{} {}({})".format(abi_cs_type(slot["returns"]), slot["name"], params)


def abi_rust_signature(slot: Dict[str, Any]) -> str:
    params = ", ".join(
        "{}: {}".format(p["name"], abi_rust_type(p["type"])) for p in slot.get("params", [])
    )
    return "extern \"C\" fn({}) -> {}".format(params, abi_rust_type(slot["returns"]))


def abi_input_hash(root: Path) -> str:
    items = []
    for rel in ABI_INPUT_SET:
        blob = (root / rel).read_bytes()
        items.append(rel.encode() + b"\0" + blob)
    return sha256_bytes(b"\n".join(items))


def derive_bundle(
    abi: Dict[str, Any],
    compiler_digest: str,
    input_digest: str,
    output_digests: List[Tuple[str, str, str]],
) -> Dict[str, Any]:
    """Derive the ADR-040 generation record from a validated ABI document."""
    profile = dict(LAYOUT_PROFILE)
    pointer = int(profile["pointerBytes"])
    if int(abi.get("pointerWidth", 0)) != pointer * 8:
        raise AbiError(
            "ABI document pointerWidth {} does not match layout profile {}".format(
                abi.get("pointerWidth"), pointer * 8
            )
        )
    tables: List[Dict[str, Any]] = []
    for table in abi.get("apiTable", []):
        slots = []
        for slot in table.get("slots", []):
            slots.append(
                {
                    "slotIndex": slot["slotIndex"],
                    "name": slot["name"],
                    "offset": int(profile["tableHeaderBytes"]) + int(slot["slotIndex"]) * pointer,
                    "returns": slot["returns"],
                    "params": [
                        {"name": p["name"], "typeRef": p["type"]} for p in slot.get("params", [])
                    ],
                    "cSignature": abi_c_signature(slot),
                    "csharpSignature": abi_cs_signature(slot),
                    "rustSignature": abi_rust_signature(slot),
                }
            )
        tables.append(
            {
                "name": table["name"],
                "version": table["version"],
                "declaredStructSize": table["structSize"],
                "minimumStructSize": abi_minimum_table_size(table, profile),
                "reservedSlots": table["reservedSlots"],
                "functionCount": table["functionCount"],
                "fields": [
                    {"name": n, "offset": o, "size": s, "c": c, "csharp": cs, "rust": rs}
                    for n, o, s, c, cs, rs in TABLE_FIELDS
                ],
                "slots": slots,
            }
        )
    return {
        "bundleId": ABI_BUNDLE_ID,
        "baselineId": BASELINE,
        "schemaEpoch": 1,
        "compiler": {
            "name": ABI_COMPILER_NAME,
            "version": ABI_COMPILER_VERSION,
            "digest": compiler_digest,
        },
        "inputSet": list(ABI_INPUT_SET),
        "inputHash": input_digest,
        "abi": {
            "abiVersion": abi["abiVersion"],
            "entrySymbol": abi["entrySymbol"],
            "symbolPrefix": abi["symbolPrefix"],
            "callingConvention": abi["callingConvention"],
            "pointerWidth": abi["pointerWidth"],
            "endianness": abi["endianness"],
            "capabilityBits": abi["capabilityBits"],
        },
        "layoutProfile": profile,
        "typeMapping": [
            {"typeRef": k, "c": c, "csharp": cs, "rust": rs, "size": size, "align": align}
            for k, c, cs, rs, size, align in ABI_TYPE_MAPPING
        ],
        "root": {
            "declaredStructSize": abi["structSize"],
            "minimumStructSize": abi_minimum_root_size(abi, profile),
            "fields": [
                {"name": n, "offset": o, "size": s, "c": c, "csharp": cs, "rust": rs}
                for n, o, s, c, cs, rs in ROOT_FIELDS
            ],
            "tables": [
                {
                    "name": table["name"],
                    "offset": int(profile["rootHeaderBytes"]) + index * pointer,
                }
                for index, table in enumerate(abi.get("apiTable", []))
            ],
        },
        "tables": tables,
        "outputFiles": [
            {"path": path, "role": role, "digest": digest} for path, role, digest in output_digests
        ],
    }


def emit_c_header(abi: Dict[str, Any]) -> str:
    profile = LAYOUT_PROFILE
    pointer = int(profile["pointerBytes"])
    out = [
        "/* Generated Root ABI header. Do not hand-edit. */\n",
        "/* Publisher: LumioGameEngineArchitecture / {}. */\n".format(BASELINE),
        "/* Compiler: {} {}. ADR-040. */\n".format(ABI_COMPILER_NAME, ABI_COMPILER_VERSION),
        "/* Layout profile: {} (pointer {} bytes, max align {}). */\n\n".format(
            profile["targetProfileId"], pointer, profile["maxAlignment"]
        ),
        "#ifndef LUMIO_CORE_H\n#define LUMIO_CORE_H\n\n",
        "#include <stdint.h>\n#include <stddef.h>\n\n",
        "#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n",
        "#define LUMIO_ABI_VERSION {}\n".format(abi["abiVersion"]),
        "#define LUMIO_ENTRY_SYMBOL \"{}\"\n".format(abi["entrySymbol"]),
        "#define LUMIO_SYMBOL_PREFIX \"{}\"\n".format(abi["symbolPrefix"]),
        "#define LUMIO_CAPABILITY_BITS {}u\n\n".format(abi["capabilityBits"]),
        "typedef int32_t lumio_status_t;\n\n",
        "typedef struct lumio_handle_t {\n"
        "    uint32_t index;\n"
        "    uint32_t generation;\n"
        "    uint64_t context;\n"
        "} lumio_handle_t;\n\n",
        "typedef struct lumio_buffer_t {\n"
        "    void* ptr;\n"
        "    uint64_t len;\n"
        "    uint64_t capacity;\n"
        "} lumio_buffer_t;\n\n",
    ]
    opaque = abi_opaque_types(abi)
    if opaque:
        out.append("/* Opaque caller-owned payloads; bodies are guarded by their own struct_size. */\n")
        for tag, _rust_name, _origin in opaque:
            out.append("struct {};\n".format(tag))
        out.append("\n")
    for table in abi.get("apiTable", []):
        minimum = abi_minimum_table_size(table, profile)
        declared = int(table["structSize"])
        out.append("typedef struct {} {{\n".format(table["name"]))
        out.append("    uint32_t version;\n    uint32_t struct_size;\n    uint64_t reserved0;\n")
        for slot in table["slots"]:
            out.append("    {};\n".format(abi_c_signature(slot)))
        if int(table["reservedSlots"]) > 0:
            out.append("    void* reserved[{}];\n".format(table["reservedSlots"]))
        if declared > minimum:
            out.append("    unsigned char reserved_tail[{}];\n".format(declared - minimum))
        out.append("}} {};\n\n".format(table["name"]))
    root_minimum = abi_minimum_root_size(abi, profile)
    root_declared = int(abi["structSize"])
    out.append("typedef struct lumio_root_api {\n")
    out.append("    uint32_t abi_version;\n    uint32_t struct_size;\n    uint64_t capability_bits;\n")
    for table in abi.get("apiTable", []):
        out.append("    const {}* {};\n".format(table["name"], table["name"]))
    if root_declared > root_minimum:
        out.append("    unsigned char reserved_tail[{}];\n".format(root_declared - root_minimum))
    out.append("} lumio_root_api;\n\n")
    out.append(
        "lumio_status_t {}(uint32_t requested_version, const lumio_root_api** out_table);\n\n".format(
            abi["entrySymbol"]
        )
    )
    out.append("/* Layout Golden assertions: a mismatch is a build failure, never a runtime discovery. */\n")
    out.append("#define LUMIO_STATIC_ASSERT(cond, tag) typedef char lumio_assert_##tag[(cond) ? 1 : -1]\n")
    out.append("LUMIO_STATIC_ASSERT(sizeof(lumio_handle_t) == 16, handle_size);\n")
    out.append("LUMIO_STATIC_ASSERT(sizeof(lumio_buffer_t) == 24, buffer_size);\n")
    out.append("LUMIO_STATIC_ASSERT(sizeof(lumio_status_t) == 4, status_size);\n")
    out.append("LUMIO_STATIC_ASSERT(sizeof(void*) == {}, pointer_size);\n".format(pointer))
    for table in abi.get("apiTable", []):
        tag = table["name"]
        out.append(
            "LUMIO_STATIC_ASSERT(sizeof({}) == {}, {}_size);\n".format(tag, table["structSize"], tag)
        )
        for slot in table["slots"]:
            offset = int(profile["tableHeaderBytes"]) + int(slot["slotIndex"]) * pointer
            out.append(
                "LUMIO_STATIC_ASSERT(offsetof({}, {}) == {}, {}_offset);\n".format(
                    tag, slot["name"], offset, slot["name"]
                )
            )
    out.append(
        "LUMIO_STATIC_ASSERT(sizeof(lumio_root_api) == {}, root_size);\n".format(root_declared)
    )
    for index, table in enumerate(abi.get("apiTable", [])):
        offset = int(profile["rootHeaderBytes"]) + index * pointer
        out.append(
            "LUMIO_STATIC_ASSERT(offsetof(lumio_root_api, {}) == {}, root_{}_offset);\n".format(
                table["name"], offset, table["name"]
            )
        )
    out.append("\n#ifdef __cplusplus\n}\n#endif\n\n#endif /* LUMIO_CORE_H */\n")
    return "".join(out)


def emit_rust_root_abi(abi: Dict[str, Any]) -> str:
    profile = LAYOUT_PROFILE
    pointer = int(profile["pointerBytes"])
    out = [
        "//! Generated Root ABI binding. Do not hand-edit.\n",
        "//! Publisher: LumioGameEngineArchitecture / {}. ADR-040.\n".format(BASELINE),
        "//! Layout profile: {}.\n\n".format(profile["targetProfileId"]),
        "#![allow(non_camel_case_types)]\n\n",
        "pub const ABI_VERSION: u32 = {};\n".format(abi["abiVersion"]),
        "pub const ENTRY_SYMBOL: &str = \"{}\";\n".format(abi["entrySymbol"]),
        "pub const SYMBOL_PREFIX: &str = \"{}\";\n".format(abi["symbolPrefix"]),
        "pub const CALLING_CONVENTION: &str = \"{}\";\n".format(abi["callingConvention"]),
        "pub const CAPABILITY_BITS: u64 = {};\n".format(abi["capabilityBits"]),
        "pub const TARGET_PROFILE_ID: &str = \"{}\";\n".format(profile["targetProfileId"]),
        "pub const POINTER_BYTES: usize = {};\n".format(pointer),
        "pub const MAX_ALIGNMENT: usize = {};\n".format(profile["maxAlignment"]),
        "pub const ROOT_HEADER_BYTES: usize = {};\n".format(profile["rootHeaderBytes"]),
        "pub const TABLE_HEADER_BYTES: usize = {};\n\n".format(profile["tableHeaderBytes"]),
        "pub type LumioStatus = i32;\n\n",
        "#[repr(C)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub struct LumioHandle {\n"
        "    pub index: u32,\n    pub generation: u32,\n    pub context: u64,\n}\n\n",
        "#[repr(C)]\n#[derive(Clone, Copy, Debug)]\npub struct LumioBuffer {\n"
        "    pub ptr: *mut core::ffi::c_void,\n    pub len: u64,\n    pub capacity: u64,\n}\n\n",
    ]
    for _tag, rust_name, _origin in abi_opaque_types(abi):
        out.append(
            "#[repr(C)]\npub struct {} {{\n    _opaque: [u8; 0],\n}}\n\n".format(rust_name)
        )
    for table in abi.get("apiTable", []):
        minimum = abi_minimum_table_size(table, profile)
        declared = int(table["structSize"])
        out.append("#[repr(C)]\npub struct {} {{\n".format(pascal(table["name"])))
        out.append("    pub version: u32,\n    pub struct_size: u32,\n    pub reserved0: u64,\n")
        for slot in table["slots"]:
            out.append(
                "    pub {}: Option<{}>,\n".format(slot["name"], abi_rust_signature(slot))
            )
        if int(table["reservedSlots"]) > 0:
            out.append(
                "    pub reserved: [*mut core::ffi::c_void; {}],\n".format(table["reservedSlots"])
            )
        if declared > minimum:
            out.append("    pub reserved_tail: [u8; {}],\n".format(declared - minimum))
        out.append("}\n\n")
    root_minimum = abi_minimum_root_size(abi, profile)
    root_declared = int(abi["structSize"])
    out.append("#[repr(C)]\npub struct LumioRootApi {\n")
    out.append("    pub abi_version: u32,\n    pub struct_size: u32,\n    pub capability_bits: u64,\n")
    for table in abi.get("apiTable", []):
        out.append("    pub {}: *const {},\n".format(table["name"], pascal(table["name"])))
    if root_declared > root_minimum:
        out.append("    pub reserved_tail: [u8; {}],\n".format(root_declared - root_minimum))
    out.append("}\n\n")
    out.append("/// Layout Golden: `(struct, field, offset)` triples the consumer asserts.\n")
    out.append("pub const SLOT_OFFSETS: &[(&str, &str, usize)] = &[\n")
    for table in abi.get("apiTable", []):
        for slot in table["slots"]:
            out.append(
                "    (\"{}\", \"{}\", {}),\n".format(
                    table["name"],
                    slot["name"],
                    int(profile["tableHeaderBytes"]) + int(slot["slotIndex"]) * pointer,
                )
            )
    out.append("];\n\n")
    out.append("pub const STRUCT_SIZES: &[(&str, usize)] = &[\n")
    out.append("    (\"lumio_handle_t\", 16),\n    (\"lumio_buffer_t\", 24),\n")
    for table in abi.get("apiTable", []):
        out.append("    (\"{}\", {}),\n".format(table["name"], table["structSize"]))
    out.append("    (\"lumio_root_api\", {}),\n".format(root_declared))
    out.append("];\n\n")
    out.append("const _: () = {\n")
    out.append("    assert!(core::mem::size_of::<LumioHandle>() == 16);\n")
    out.append("    assert!(core::mem::size_of::<LumioBuffer>() == 24);\n")
    for table in abi.get("apiTable", []):
        out.append(
            "    assert!(core::mem::size_of::<{}>() == {});\n".format(
                pascal(table["name"]), table["structSize"]
            )
        )
    out.append(
        "    assert!(core::mem::size_of::<LumioRootApi>() == {});\n".format(root_declared)
    )
    for table in abi.get("apiTable", []):
        for slot in table["slots"]:
            out.append(
                "    assert!(core::mem::offset_of!({}, {}) == {});\n".format(
                    pascal(table["name"]),
                    slot["name"],
                    int(profile["tableHeaderBytes"]) + int(slot["slotIndex"]) * pointer,
                )
            )
    for index, table in enumerate(abi.get("apiTable", [])):
        out.append(
            "    assert!(core::mem::offset_of!(LumioRootApi, {}) == {});\n".format(
                table["name"], int(profile["rootHeaderBytes"]) + index * pointer
            )
        )
    out.append("};\n")
    return "".join(out)


def emit_csharp_root_abi(abi: Dict[str, Any]) -> str:
    profile = LAYOUT_PROFILE
    pointer = int(profile["pointerBytes"])
    out = [
        "// Generated Root ABI binding. Do not hand-edit.\n",
        "// Publisher: LumioGameEngineArchitecture / {}. ADR-040.\n".format(BASELINE),
        "// Pure managed layout description; the consumer binds the entry symbol itself.\n",
        "using System;\nusing System.Runtime.InteropServices;\n\n",
        "namespace Lumio.Gen.LanguageBinding;\n\n",
        "public static class RootAbi\n{\n",
        "    public const uint AbiVersion = {};\n".format(abi["abiVersion"]),
        "    public const string EntrySymbol = \"{}\";\n".format(abi["entrySymbol"]),
        "    public const string SymbolPrefix = \"{}\";\n".format(abi["symbolPrefix"]),
        "    public const string CallingConvention = \"{}\";\n".format(abi["callingConvention"]),
        "    public const ulong CapabilityBits = {};\n".format(abi["capabilityBits"]),
        "    public const string TargetProfileId = \"{}\";\n".format(profile["targetProfileId"]),
        "    public const int PointerBytes = {};\n".format(pointer),
        "    public const int MaxAlignment = {};\n".format(profile["maxAlignment"]),
        "    public const int RootHeaderBytes = {};\n".format(profile["rootHeaderBytes"]),
        "    public const int TableHeaderBytes = {};\n".format(profile["tableHeaderBytes"]),
        "}\n\n",
        "public enum LumioStatus : int { Ok = 0 }\n\n",
        "[StructLayout(LayoutKind.Sequential)]\npublic struct LumioHandle\n{\n"
        "    public uint Index;\n    public uint Generation;\n    public ulong Context;\n}\n\n",
        "[StructLayout(LayoutKind.Sequential)]\npublic struct LumioBuffer\n{\n"
        "    public IntPtr Ptr;\n    public ulong Len;\n    public ulong Capacity;\n}\n\n",
    ]
    for table in abi.get("apiTable", []):
        minimum = abi_minimum_table_size(table, profile)
        declared = int(table["structSize"])
        out.append("[StructLayout(LayoutKind.Sequential)]\npublic struct {}\n{{\n".format(pascal(table["name"])))
        out.append("    public uint Version;\n    public uint StructSize;\n    public ulong Reserved0;\n")
        for slot in table["slots"]:
            out.append("    // {}\n".format(abi_cs_signature(slot)))
            out.append("    public IntPtr {};\n".format(pascal(slot["name"])))
        if int(table["reservedSlots"]) > 0:
            out.append(
                "    [MarshalAs(UnmanagedType.ByValArray, SizeConst = {})]\n"
                "    public IntPtr[] Reserved;\n".format(table["reservedSlots"])
            )
        if declared > minimum:
            out.append(
                "    [MarshalAs(UnmanagedType.ByValArray, SizeConst = {})]\n"
                "    public byte[] ReservedTail;\n".format(declared - minimum)
            )
        out.append("}\n\n")
    root_minimum = abi_minimum_root_size(abi, profile)
    root_declared = int(abi["structSize"])
    out.append("[StructLayout(LayoutKind.Sequential)]\npublic struct LumioRootApi\n{\n")
    out.append("    public uint AbiVersion;\n    public uint StructSize;\n    public ulong CapabilityBits;\n")
    for table in abi.get("apiTable", []):
        out.append("    public IntPtr {};\n".format(pascal(table["name"])))
    if root_declared > root_minimum:
        out.append(
            "    [MarshalAs(UnmanagedType.ByValArray, SizeConst = {})]\n"
            "    public byte[] ReservedTail;\n".format(root_declared - root_minimum)
        )
    out.append("}\n\n")
    out.append("public readonly record struct SlotOffset(string Table, string Slot, int Offset);\n")
    out.append("public static class RootAbiLayout\n{\n    public static readonly SlotOffset[] SlotOffsets =\n    {\n")
    for table in abi.get("apiTable", []):
        for slot in table["slots"]:
            out.append(
                "        new SlotOffset(\"{}\", \"{}\", {}),\n".format(
                    table["name"],
                    slot["name"],
                    int(profile["tableHeaderBytes"]) + int(slot["slotIndex"]) * pointer,
                )
            )
    out.append("    };\n\n    public static readonly (string Name, int Size)[] StructSizes =\n    {\n")
    out.append("        (\"lumio_handle_t\", 16),\n        (\"lumio_buffer_t\", 24),\n")
    for table in abi.get("apiTable", []):
        out.append("        (\"{}\", {}),\n".format(table["name"], table["structSize"]))
    out.append("        (\"lumio_root_api\", {}),\n".format(root_declared))
    out.append("    };\n}\n")
    return "".join(out)


def validate_abi_document(root: Path, abi: Dict[str, Any]) -> None:
    """ADR-040: reject an invalid ABI document before a single output byte is written."""
    import lumio_contract  # local import: lumio_contract imports this module at load time

    schema_file = root / "schemas" / "native-managed-abi.schema.json"
    schema = json.loads(schema_file.read_text(encoding="utf-8"))
    errors = lumio_contract.structural_errors(
        abi, schema, schema_file, lumio_contract.SchemaResolver()
    )
    errors.extend(lumio_contract.semantic_errors("native-managed-abi", abi))
    if errors:
        raise AbiError(
            "ABI document {} is invalid; no bundle is emitted: {}".format(
                ABI_DOCUMENT, "; ".join(errors)
            )
        )


def emit_root_abi(root: Path, out_dir: Path, compiler_digest: str) -> Dict[str, Any]:
    """Emit the ADR-040 bundle: C header, Rust and C# bindings, generation record."""
    abi = json.loads((root / ABI_DOCUMENT).read_text(encoding="utf-8"))
    validate_abi_document(root, abi)
    contents = {
        "abi/lumio_core.h": emit_c_header(abi),
        "rust/lumio-gen-language-binding/src/root_abi.rs": emit_rust_root_abi(abi),
        "csharp/Lumio.Gen.LanguageBinding/RootAbi.cs": emit_csharp_root_abi(abi),
    }
    for rel, text in contents.items():
        write_text(out_dir / rel, text)
    digests = [
        (rel, role, sha256_file(out_dir / rel)) for rel, role in ABI_OUTPUT_FILES
    ]
    bundle = derive_bundle(abi, compiler_digest, abi_input_hash(root), digests)
    write_text(out_dir / ABI_BUNDLE_FILE, canonical_json(bundle) + "\n")
    return bundle


def emit_workspace(rust_root: Path) -> None:
    members = ",\n    ".join("\"{}\"".format(name) for name in RUST_CRATES.values())
    write_text(
        rust_root / "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\n    %s\n]\n" % members,
    )


def descriptor(
    kind: str,
    language: str,
    compiler: str,
    in_hash: str,
    out_hash: str,
) -> Dict[str, Any]:
    artifact_id = "{}-{}".format(kebab_kind(kind), language)
    return {
        "artifactId": artifact_id,
        "artifactKind": kind,
        "publisher": PUBLISHER,
        "baselineId": BASELINE,
        "schemaEpoch": 1,
        "compilerHash": compiler,
        "inputHash": in_hash,
        "outputHash": out_hash,
        "implementationDependencies": [],
        "forbiddenDependents": list(FORBIDDEN),
        "consumers": list(CONSUMERS),
        "language": language,
        "packagePath": "rust/{}/".format(RUST_CRATES[kind])
        if language == "rust"
        else "csharp/{}/".format(CS_PROJ[kind]),
    }


def generate(root: Path, out_dir: Path) -> Dict[str, Any]:
    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True)
    machines = load_state_machines(root)
    if len(machines) != 12:
        raise RuntimeError("expected 12 state-machine fixtures, got {}".format(len(machines)))
    schema_ids = load_schema_ids(root)
    errors = load_error_ids(root)
    abi = json.loads((root / ABI_DOCUMENT).read_text(encoding="utf-8"))
    comp = compiler_hash(root)
    inh = input_hash(root)
    rust_root = out_dir / "rust"
    cs_root = out_dir / "csharp"
    desc_dir = out_dir / "descriptors"

    emit_protocol_permission(
        rust_root / RUST_CRATES["ProtocolPermissionValidator"],
        cs_root / CS_PROJ["ProtocolPermissionValidator"],
    )
    emit_mapping(rust_root / RUST_CRATES["MappingTable"], cs_root / CS_PROJ["MappingTable"])
    canonical_profile = derive_canonical_profile(root)
    emit_canonical(
        rust_root / RUST_CRATES["CanonicalSerializer"],
        cs_root / CS_PROJ["CanonicalSerializer"],
        canonical_profile,
    )
    emit_language_binding(
        rust_root / RUST_CRATES["LanguageBinding"],
        cs_root / CS_PROJ["LanguageBinding"],
        schema_ids,
    )
    emit_contract_types(
        rust_root / RUST_CRATES["ContractTypes"],
        cs_root / CS_PROJ["ContractTypes"],
        machines,
        schema_ids,
        errors,
        abi,
    )
    emit_contract_runtime(
        rust_root / RUST_CRATES["ContractRuntime"],
        cs_root / CS_PROJ["ContractRuntime"],
    )
    emit_workspace(rust_root)
    write_text(out_dir / ".gitignore", "rust/target/\ncsharp/**/bin/\ncsharp/**/obj/\n")
    bundle = emit_root_abi(root, out_dir, comp)
    write_text(out_dir / CANONICAL_PROFILE_FILE, canonical_json(canonical_profile) + "\n")

    inventory = []
    for kind in KINDS:
        rust_pkg = rust_root / RUST_CRATES[kind]
        cs_pkg = cs_root / CS_PROJ[kind]
        rust_hash = dir_output_hash(rust_pkg)
        cs_hash = dir_output_hash(cs_pkg)
        for language, digest, pkg in (
            ("rust", rust_hash, rust_pkg),
            ("csharp", cs_hash, cs_pkg),
        ):
            desc = descriptor(kind, language, comp, inh, digest)
            # language/packagePath extra fields are inventory-only; descriptors stay schema-closed.
            slim = {k: v for k, v in desc.items() if k not in ("language", "packagePath")}
            name = "{}.json".format(slim["artifactId"])
            write_text(desc_dir / name, canonical_json(slim) + "\n")
            write_text(pkg / "artifact.descriptor.json", canonical_json(slim) + "\n")
            inventory.append(desc)
    index = {
        "baselineId": BASELINE,
        "schemaEpoch": 1,
        "compilerHash": comp,
        "inputHash": inh,
        "artifacts": inventory,
        "blocked": [
            {"id": "D-009", "reason": "protocol-dispatch not frozen"},
            {"id": "D-011", "reason": "Auth wire not frozen"},
        ],
        "stateMachineCount": len(machines),
        "stateMachineIds": [m.get("machineId") for m in machines],
        "canonicalDigest": {
            "profileId": canonical_profile["profileId"],
            "profilePath": CANONICAL_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / CANONICAL_PROFILE_FILE),
            "formId": canonical_profile["canonicalForm"]["formId"],
            "digestAlgorithm": dict(canonical_profile["digestAlgorithm"]),
            "goldenCount": len(canonical_profile["goldens"]),
        },
        "rootAbi": {
            "bundleId": bundle["bundleId"],
            "bundlePath": ABI_BUNDLE_FILE,
            "bundleDigest": sha256_file(out_dir / ABI_BUNDLE_FILE),
            "compiler": dict(bundle["compiler"]),
            "inputHash": bundle["inputHash"],
            "layoutProfileId": bundle["layoutProfile"]["targetProfileId"],
            "outputFiles": [dict(item) for item in bundle["outputFiles"]],
        },
    }
    write_text(out_dir / "index.json", canonical_json(index) + "\n")
    write_text(
        out_dir / "README.md",
        "Generated V1.4 contract artifacts. Do not hand-edit package sources.\n"
        "Regenerate with `python tools/lumio_contract.py generate --out packages`.\n"
        "`abi/` is the ADR-040 Root ABI bundle: `lumio_core.h`, the layout Golden\n"
        "record `root-abi-bundle.json`, and the digests of the Rust and C# bindings.\n",
    )
    return index


def command_generate(root: Path, out: str) -> int:
    out_dir = Path(out)
    if not out_dir.is_absolute():
        out_dir = root / out
    index = generate(root, out_dir)
    print("generated {} artifacts under {}".format(len(index["artifacts"]), out_dir))
    print("compilerHash {}".format(index["compilerHash"]))
    print("inputHash {}".format(index["inputHash"]))
    print("stateMachines {}".format(",".join(index["stateMachineIds"])))
    return 0
