"""Deterministic V1.4 generated-contract artifact publisher."""

from __future__ import annotations

import hashlib
import json
import re
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
# The Root ABI bundle (abi/) is a C-level FFI surface, not a Rust/C# generated
# package: its consumers are the native-toolchain repositories named by
# ADR-040's Owner line, not the six kinds above.
ROOT_ABI_CONSUMERS = ["LumioCoreEngine", "LumioNativeCore"]
# The canonical/digest and trust profiles name LumioCoreEngine as their consumer
# in the ADR-041 and ADR-042 Owner lines. Without this a downstream that
# converges its mirror on the consumers relation cannot judge these two files.
CANONICAL_PROFILE_CONSUMERS = ["LumioCoreEngine"]
TRUST_PROFILE_CONSUMERS = ["LumioCoreEngine"]
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


def load_message_ids(root: Path) -> List[str]:
    """The `MessageType` namespace, in registry order; the gate's registered set."""
    registry = json.loads((root / "ids" / "index.json").read_text(encoding="utf-8"))
    for ns in registry.get("namespaces", []):
        if ns.get("namespace") == "MessageType":
            return [v["id"] for v in ns.get("values", [])]
    return []


def load_error_ids(root: Path) -> List[str]:
    registry = json.loads((root / "ids" / "index.json").read_text(encoding="utf-8"))
    for ns in registry.get("namespaces", []):
        if ns.get("namespace") == "ErrorCode":
            return [v["id"] for v in ns.get("values", [])]
    return []


def load_capabilities(root: Path) -> List[Tuple[str, int, str]]:
    """ADR-040 section 7 (D-015): the `Capability` namespace, in registry order.

    `ids/index.json` stays the authority for the numerics; the generator is the
    only thing allowed to project them into a language, which is what lets the
    three native repositories share one key space instead of inventing three.
    """
    registry = json.loads((root / "ids" / "index.json").read_text(encoding="utf-8"))
    for ns in registry.get("namespaces", []):
        if ns.get("namespace") == "Capability":
            return [
                (str(v["id"]), int(v["numeric"]), str(v.get("status", "")))
                for v in ns.get("values", [])
            ]
    return []


def capability_rust(capabilities: List[Tuple[str, int, str]]) -> str:
    out = [
        "/// ADR-040 section 7 (D-015): capability keys projected from the ID Registry.\n",
        "/// `ids/index.json` is the authority for these numerics; this table is its only\n",
        "/// published projection. A consumer reads it instead of inventing a private key.\n",
        "/// These are enumeration keys, not bit positions: `capability_bits` semantics\n",
        "/// stay unfrozen (ADR-040 section 7).\n",
        "pub const CAPABILITY_KEYS: &[(&str, u32, &str)] = &[\n",
    ]
    for name, numeric, status in capabilities:
        out.append("    (\"{}\", {}, \"{}\"),\n".format(name, numeric, status))
    out.append("];\n\n")
    out.append("pub fn capability_key(name: &str) -> Option<u32> {\n")
    out.append("    let mut i = 0;\n    while i < CAPABILITY_KEYS.len() {\n")
    out.append("        if CAPABILITY_KEYS[i].0.as_bytes() == name.as_bytes() {\n")
    out.append("            return Some(CAPABILITY_KEYS[i].1);\n        }\n        i += 1;\n    }\n")
    out.append("    None\n}\n")
    return "".join(out)


def capability_csharp(capabilities: List[Tuple[str, int, str]]) -> str:
    out = [
        "// ADR-040 section 7 (D-015): capability keys projected from the ID Registry.\n",
        "// ids/index.json is the authority for these numerics; this is its only published\n",
        "// projection. Enumeration keys, not bit positions.\n",
        cs_value_struct(
            "CapabilityKey", [("string", "Name"), ("uint", "Numeric"), ("string", "Status")]
        ),
        "\npublic static class CapabilityKeys\n{\n    public static readonly CapabilityKey[] All =\n    {\n",
    ]
    for name, numeric, status in capabilities:
        out.append(
            "        new CapabilityKey(\"{}\", {}u, \"{}\"),\n".format(name, numeric, status)
        )
    out.append("    };\n\n")
    for name, numeric, _status in capabilities:
        out.append("    public const uint {} = {}u;\n".format(name, numeric))
    out.append("}\n")
    return "".join(out)


def capability_c(capabilities: List[Tuple[str, int, str]]) -> str:
    out = [
        "/* ADR-040 section 7 (D-015): capability keys projected from the ID Registry. */\n",
        "/* ids/index.json is the authority; these are enumeration keys, NOT bit\n"
        "   positions -- LUMIO_CAPABILITY_BITS semantics stay unfrozen. */\n",
    ]
    for name, numeric, _status in capabilities:
        out.append("#define LUMIO_CAPABILITY_{} {}u\n".format(c_screaming(name), numeric))
    out.append("#define LUMIO_CAPABILITY_COUNT {}u\n\n".format(len(capabilities)))
    return "".join(out)


def c_screaming(name: str) -> str:
    """`VoxelMeshCollision` -> `VOXEL_MESH_COLLISION`."""
    chars: List[str] = []
    for index, char in enumerate(name):
        if char.isupper() and index and not name[index - 1].isupper():
            chars.append("_")
        chars.append(char.upper())
    return "".join(chars)


def rust_cargo(name: str) -> str:
    return (
        "[package]\n"
        "name = \"{}\"\n"
        "version = \"0.0.0\"\n"
        "edition = \"2021\"\n"
        "publish = false\n\n"
        "[dependencies]\n"
    ).format(name)


# ADR-048 (D-4): Unity consumes `netstandard2.1`, the .NET Host consumes `net8.0`.
# One package must serve both, so every generated project multi-targets.
CS_TARGET_FRAMEWORKS = "netstandard2.1;net8.0"


def csproj(name: str) -> str:
    return (
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n"
        "  <PropertyGroup>\n"
        "    <TargetFrameworks>{}</TargetFrameworks>\n"
        "    <ImplicitUsings>disable</ImplicitUsings>\n"
        "    <Nullable>enable</Nullable>\n"
        "    <AllowUnsafeBlocks>false</AllowUnsafeBlocks>\n"
        "    <DisableImplicitNuGetFallbackFolder>true</DisableImplicitNuGetFallbackFolder>\n"
        "  </PropertyGroup>\n"
        "  <!-- Pure managed; no Native / PInvoke / implementation-project PackageReference. -->\n"
        "</Project>\n".format(CS_TARGET_FRAMEWORKS)
    )


# Reserved words a generated camelCase parameter can collide with. A contract
# member named `case`, `event` or `namespace` is legal JSON and legal as a C#
# *property*; only the constructor parameter needs the verbatim prefix.
CS_KEYWORDS = {
    "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
    "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
    "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
    "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock",
    "long", "namespace", "new", "null", "object", "operator", "out", "override", "params",
    "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed", "short",
    "sizeof", "stackalloc", "static", "string", "struct", "switch", "this", "throw", "true",
    "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual",
    "void", "volatile", "while",
}


def cs_param(name: str) -> str:
    """camelCase a member name for use as a constructor parameter."""
    lowered = name[0].lower() + name[1:]
    return "@" + lowered if lowered in CS_KEYWORDS else lowered


def cs_value_struct(name: str, members: List[Tuple[str, str]]) -> str:
    """Emit a positional-record-shaped readonly struct without using `record`.

    `record` and `init` accessors require `IsExternalInit`, which netstandard2.1
    does not ship; a constructor plus get-only properties is the same shape and
    compiles on every target either consumer offers.
    """
    params = ", ".join("{} {}".format(t, cs_param(n)) for t, n in members)
    assigns = " ".join("{} = {};".format(n, cs_param(n)) for _t, n in members)
    props = "".join("    public {} {} {{ get; }}\n".format(t, n) for t, n in members)
    return (
        "public readonly struct {name}\n{{\n"
        "    public {name}({params})\n    {{\n        {assigns}\n    }}\n"
        "{props}}}\n".format(name=name, params=params, assigns=assigns, props=props)
    )


def cs_namespace(name: str, body: str, usings: str = "") -> str:
    """Wrap generated C# in a block-scoped namespace.

    Block scope, not `namespace X;`: under `netstandard2.1` the default language
    version is C# 8, which has no file-scoped namespace, and pinning LangVersion
    forward would push the requirement onto Unity's compiler instead. Block scope
    compiles on every version either consumer can offer.
    """
    indented = "".join(
        ("    " + line if line.strip() else line) + "\n" for line in body.rstrip("\n").split("\n")
    )
    return "{}namespace {}\n{{\n{}}}\n".format(usings, name, indented)


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


# Known-answer vectors for the generated SHA-256, from FIPS 180-4. The third is
# 56 bytes so that its length padding spills into a second compression block.
# tools/lumio_kat.py imports this list to drive the Rust / C# / hashlib
# three-way comparison, so the vectors are defined here once and only here.
KAT_VECTORS = [
    (b"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
    (b"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
    (
        b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    ),
]

KAT_COMMENTS = [
    "FIPS 180-4 reference digest of the empty message.",
    "FIPS 180-4 B.1: single-block message.",
    "FIPS 180-4 B.2: 448-bit message, 56 bytes, spans two compression blocks.",
]


def kat_test_rs() -> str:
    """Emit the known-answer test for the generated Rust SHA-256.

    Every other test in the generated chain.rs only asserts the hasher agrees
    with itself, so a corrupted round constant stays invisible to them. These
    are the only assertions there compared against values fixed outside this
    codebase.
    """
    lines = [
        "#[test]",
        "fn sha256_known_answer_vectors() {",
    ]
    for (data, digest), comment in zip(KAT_VECTORS, KAT_COMMENTS):
        lines.append("    // {}".format(comment))
        lines.append("    assert_eq!(")
        lines.append('        sha256_hex(b"{}"),'.format(data.decode("ascii")))
        lines.append('        "{}",'.format(digest))
        lines.append("    );")
    lines.append("}")
    return "\n".join(lines) + "\n"


SHA256_RS = r'''
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
    0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
    0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
    0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
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


# ADR-048 section 2: the order the generated gate reports a rejection in when a
# record fails more than one check. `StaleConnectionGeneration` leads because
# ADR-022's failure semantics already require that code whenever the generation
# differs; the rest follow ADR-022's own clause order. Published as data so the
# three implementations cannot drift into three different first answers.
GATE_REJECT_PRECEDENCE = [
    "StaleConnectionGeneration",
    "SessionMismatch",
    "ReleaseMismatch",
    "MessagePermissionDenied",
    "RoleMismatch",
    "ClaimNotGranted",
]
# Owned by `ClientReplicaSession` (ADR-022), not computable from the record: the
# gate accepts it as a declared verdict but never derives it.
GATE_DECLARED_ONLY_REASONS = ["SessionAntiReplay"]


def evaluate_protocol_gate(record: Dict[str, Any], registered_message_ids: List[str]) -> Tuple[str, Any]:
    """Run the ADR-022 gate over one record: returns (verdict, rejectReason|None).

    This is the executable form of ADR-022. The `messageId` clause is enforced as
    far as the architecture source actually publishes it — the id must be a
    registered `MessageType` — and no further: no role-to-message permission
    table exists in this repository, so deriving one here would be inventing a
    public contract rather than executing one.
    """
    if record.get("connectionGeneration") != record.get("admittedConnectionGeneration"):
        return "Reject", "StaleConnectionGeneration"
    if record.get("sessionId") != record.get("admittedSessionId"):
        return "Reject", "SessionMismatch"
    if record.get("productId") != record.get("admittedProductId") or record.get(
        "gameReleaseId"
    ) != record.get("admittedGameReleaseId"):
        return "Reject", "ReleaseMismatch"
    if record.get("messageId") not in registered_message_ids:
        return "Reject", "MessagePermissionDenied"
    if record.get("role") != record.get("admittedRole"):
        return "Reject", "RoleMismatch"
    admitted = set(record.get("admittedClaims") or [])
    if [claim for claim in record.get("claims") or [] if claim not in admitted]:
        return "Reject", "ClaimNotGranted"
    return "Accept", None


def emit_protocol_permission(
    rust_dir: Path, cs_dir: Path, message_ids: List[str]
) -> None:
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
    rust += "pub fn is_active_field(name: &str) -> bool {\n    ACTIVE_PERMISSION_FIELDS.contains(&name)\n}\n\n"
    rust += gate_rust(message_ids)
    write_text(rust_dir / "src" / "lib.rs", rust)
    write_text(rust_dir / "Cargo.toml", rust_cargo(RUST_CRATES["ProtocolPermissionValidator"]))
    cs = cs_namespace(
        "Lumio.Gen.ProtocolPermissionValidator",
        "public static class ActivePermissionFields\n{\n"
        "    public static readonly string[] Names = new[]\n    {\n"
        + "".join("        \"{}\",\n".format(f) for f in fields)
        + "    };\n}\n",
    )
    write_text(cs_dir / "ActivePermissionFields.cs", cs)
    write_text(cs_dir / "ProtocolGate.cs", gate_csharp(message_ids))
    write_text(cs_dir / (CS_PROJ["ProtocolPermissionValidator"] + ".csproj"), csproj(CS_PROJ["ProtocolPermissionValidator"]))


def gate_rust(message_ids: List[str]) -> str:
    """Emit the executable ADR-022 gate: an admission decision, not a field list."""
    out = [
        "/// Registered `MessageType` ids (ids/index.json, registry order).\n",
        "pub const REGISTERED_MESSAGE_IDS: &[&str] = &[\n",
    ]
    for mid in message_ids:
        out.append("    \"{}\",\n".format(mid))
    out.append("];\n\n")
    out.append("/// Rejection precedence when a record fails more than one check (ADR-048).\n")
    out.append("pub const REJECT_PRECEDENCE: &[&str] = &[\n")
    for reason in GATE_REJECT_PRECEDENCE:
        out.append("    \"{}\",\n".format(reason))
    out.append("];\n\n")
    out.append(
        "/// Reasons the session owner declares and the gate never derives (ADR-022).\n"
        "pub const DECLARED_ONLY_REASONS: &[&str] = &[{}];\n\n".format(
            ", ".join('"{}"'.format(r) for r in GATE_DECLARED_ONLY_REASONS)
        )
    )
    out.append(
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n"
        "pub enum Verdict {\n    Accept,\n    Reject,\n}\n\n"
        "/// One Active-session message and the context it was admitted under.\n"
        "#[derive(Clone, Copy, Debug)]\n"
        "pub struct GateInput<'a> {\n"
        "    pub session_id: &'a str,\n"
        "    pub product_id: &'a str,\n"
        "    pub game_release_id: &'a str,\n"
        "    pub message_id: &'a str,\n"
        "    pub role: &'a str,\n"
        "    pub claims: &'a [&'a str],\n"
        "    pub connection_generation: u64,\n"
        "    pub admitted_session_id: &'a str,\n"
        "    pub admitted_product_id: &'a str,\n"
        "    pub admitted_game_release_id: &'a str,\n"
        "    pub admitted_role: &'a str,\n"
        "    pub admitted_claims: &'a [&'a str],\n"
        "    pub admitted_connection_generation: u64,\n"
        "}\n\n"
        "/// The ADR-022 gate. `None` reason means Accept.\n"
        "///\n"
        "/// The `messageId` clause is enforced only as far as this repository\n"
        "/// publishes it -- the id must be registered. No role-to-message\n"
        "/// permission table exists, so the gate does not invent one.\n"
        "pub fn evaluate(input: &GateInput) -> (Verdict, Option<&'static str>) {\n"
        "    if input.connection_generation != input.admitted_connection_generation {\n"
        "        return (Verdict::Reject, Some(\"StaleConnectionGeneration\"));\n    }\n"
        "    if input.session_id != input.admitted_session_id {\n"
        "        return (Verdict::Reject, Some(\"SessionMismatch\"));\n    }\n"
        "    if input.product_id != input.admitted_product_id\n"
        "        || input.game_release_id != input.admitted_game_release_id\n    {\n"
        "        return (Verdict::Reject, Some(\"ReleaseMismatch\"));\n    }\n"
        "    if !REGISTERED_MESSAGE_IDS.contains(&input.message_id) {\n"
        "        return (Verdict::Reject, Some(\"MessagePermissionDenied\"));\n    }\n"
        "    if input.role != input.admitted_role {\n"
        "        return (Verdict::Reject, Some(\"RoleMismatch\"));\n    }\n"
        "    let mut i = 0;\n"
        "    while i < input.claims.len() {\n"
        "        if !input.admitted_claims.contains(&input.claims[i]) {\n"
        "            return (Verdict::Reject, Some(\"ClaimNotGranted\"));\n        }\n"
        "        i += 1;\n    }\n"
        "    (Verdict::Accept, None)\n}\n"
    )
    return "".join(out)


def gate_csharp(message_ids: List[str]) -> str:
    body = [
        "// The executable ADR-022 Protocol/Permission gate.\n",
        "public enum Verdict { Accept, Reject }\n\n",
        cs_value_struct(
            "GateInput",
            [
                ("string", "SessionId"),
                ("string", "ProductId"),
                ("string", "GameReleaseId"),
                ("string", "MessageId"),
                ("string", "Role"),
                ("string[]", "Claims"),
                ("ulong", "ConnectionGeneration"),
                ("string", "AdmittedSessionId"),
                ("string", "AdmittedProductId"),
                ("string", "AdmittedGameReleaseId"),
                ("string", "AdmittedRole"),
                ("string[]", "AdmittedClaims"),
                ("ulong", "AdmittedConnectionGeneration"),
            ],
        ),
        "\npublic static class ProtocolGate\n{\n",
        "    public static readonly string[] RegisteredMessageIds =\n    {\n",
    ]
    for mid in message_ids:
        body.append("        \"{}\",\n".format(mid))
    body.append("    };\n\n")
    body.append("    /// <summary>Rejection precedence when more than one check fails (ADR-048).</summary>\n")
    body.append("    public static readonly string[] RejectPrecedence =\n    {\n")
    for reason in GATE_REJECT_PRECEDENCE:
        body.append("        \"{}\",\n".format(reason))
    body.append("    };\n\n")
    body.append(
        "    /// <summary>Reasons the session owner declares and the gate never derives.</summary>\n"
        "    public static readonly string[] DeclaredOnlyReasons = { %s };\n\n"
        % ", ".join('"{}"'.format(r) for r in GATE_DECLARED_ONLY_REASONS)
    )
    body.append(
        "    /// <summary>Runs the gate. A null reason means Accept. The messageId clause\n"
        "    /// is enforced only as far as the architecture source publishes it: the id\n"
        "    /// must be registered. No role-to-message table exists, so none is invented.</summary>\n"
        "    public static Verdict Evaluate(GateInput input, out string? rejectReason)\n    {\n"
        "        if (input.ConnectionGeneration != input.AdmittedConnectionGeneration)\n"
        "        { rejectReason = \"StaleConnectionGeneration\"; return Verdict.Reject; }\n"
        "        if (input.SessionId != input.AdmittedSessionId)\n"
        "        { rejectReason = \"SessionMismatch\"; return Verdict.Reject; }\n"
        "        if (input.ProductId != input.AdmittedProductId || input.GameReleaseId != input.AdmittedGameReleaseId)\n"
        "        { rejectReason = \"ReleaseMismatch\"; return Verdict.Reject; }\n"
        "        if (System.Array.IndexOf(RegisteredMessageIds, input.MessageId) < 0)\n"
        "        { rejectReason = \"MessagePermissionDenied\"; return Verdict.Reject; }\n"
        "        if (input.Role != input.AdmittedRole)\n"
        "        { rejectReason = \"RoleMismatch\"; return Verdict.Reject; }\n"
        "        foreach (var claim in input.Claims)\n        {\n"
        "            if (System.Array.IndexOf(input.AdmittedClaims, claim) < 0)\n"
        "            { rejectReason = \"ClaimNotGranted\"; return Verdict.Reject; }\n        }\n"
        "        rejectReason = null;\n        return Verdict.Accept;\n    }\n}\n"
    )
    return cs_namespace("Lumio.Gen.ProtocolPermissionValidator", "".join(body))


# --------------------------------------------------------------------------
# ADR-048 (D-3): closed contract type bodies, generated from the schema
# --------------------------------------------------------------------------

# The eight closed contracts three repositories independently reported they
# could not consume from a catalog of names. Order is the published order.
CLOSED_CONTRACT_TYPES = [
    ("config-table", "ConfigTable"),
    ("processor-descriptor", "ProcessorDescriptor"),
    ("txn-journal-record", "TxnJournalRecord"),
    ("command-log-record", "CommandLogRecord"),
    ("wal-record-envelope", "WalRecordEnvelope"),
    ("entity-identity", "EntityIdentity"),
    ("replication-envelope", "ReplicationEnvelope"),
    ("session-revision-vector", "SessionRevisionVector"),
]

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "async", "await", "box", "final", "macro", "override", "priv", "try",
    "typeof", "unsized", "virtual", "yield",
}


class SchemaTypeError(RuntimeError):
    """Raised when a schema construct has no defined projection into a type."""


def snake(name: str) -> str:
    chars: List[str] = []
    for index, char in enumerate(name):
        if char.isupper() and index and not name[index - 1].isupper():
            chars.append("_")
        chars.append(char.lower())
    out = "".join(chars)
    return "r#" + out if out in RUST_KEYWORDS else out


def pascal_member(name: str) -> str:
    return name[0].upper() + name[1:]


class TypeProjector:
    """Project the closed-contract schemas onto Rust and C# type bodies.

    Field order is the schema's declaration order, never member-name order, and
    never the order a JSON parser happens to hand back. Ordinals come from
    `ids/index.json` where a registry owns the name, and from declaration order
    otherwise — those two are the only authorities, and both are recorded on the
    emitted type so a consumer never has to guess which one it is reading.
    """

    def __init__(self, root: Path) -> None:
        self.root = root
        self.common = json.loads(
            (root / "schemas" / "common.schema.json").read_text(encoding="utf-8")
        )
        self.structs: List[Tuple[str, List[Tuple[str, str, str, bool, str]]]] = []
        self.enums: List[Tuple[str, List[Tuple[str, int, str]], str]] = []
        self._emitted: Dict[str, str] = {}
        self.registry_ordinals: Dict[str, List[Tuple[str, int]]] = {}
        for namespace in ("MessageType",):
            values = json.loads((root / "ids" / "index.json").read_text(encoding="utf-8"))
            for ns in values.get("namespaces", []):
                if ns.get("namespace") == namespace:
                    self.registry_ordinals[namespace] = [
                        (str(v["id"]), int(v["numeric"])) for v in ns.get("values", [])
                    ]

    def resolve(self, node: Dict[str, Any]) -> Dict[str, Any]:
        """Follow a local `$ref` into common.schema.json; other refs are an error."""
        seen = 0
        while isinstance(node, dict) and "$ref" in node:
            seen += 1
            if seen > 16:
                raise SchemaTypeError("cyclic $ref chain")
            ref = str(node["$ref"])
            if "#/$defs/" not in ref:
                raise SchemaTypeError("unsupported $ref {}".format(ref))
            name = ref.split("#/$defs/")[-1]
            target = self.common.get("$defs", {}).get(name)
            if target is None:
                raise SchemaTypeError("unresolved $ref {}".format(ref))
            node = target
        return node

    def declaration_order(self, schema: Dict[str, Any]) -> List[Tuple[str, Dict[str, Any], bool]]:
        """Fields in declaration order: `allOf` refs first, then own properties.

        `allOf` contributes real members (the session/release triple, the recovery
        record chain), so a type built from `properties` alone is missing fields
        the contract requires. Injected members come first and keep their own
        declaration order; ties resolve to the first declaration seen.
        """
        fields: List[Tuple[str, Dict[str, Any], bool]] = []
        seen = set()
        required = set(schema.get("required") or [])

        def take(source: Dict[str, Any]) -> None:
            for name, sub in (source.get("properties") or {}).items():
                if name in seen:
                    continue
                seen.add(name)
                fields.append((name, sub, name in required or name in set(source.get("required") or [])))

        for item in schema.get("allOf") or []:
            if "$ref" in item:
                take(self.resolve(item))
        take(schema)
        return fields

    @staticmethod
    def variant(value: str) -> str:
        """A language-safe identifier for an enum value; the wire string stays authoritative.

        Schema enums carry wire spellings like `bool` and `i32` that are not legal
        member names in C#. The identifier is a projection for the consumer's
        compiler; `name()`/`WireValue` keeps the value that actually crosses.
        """
        parts = re.split(r"[^A-Za-z0-9]+", value)
        ident = "".join(p[0].upper() + p[1:] if p else "" for p in parts)
        if not ident:
            raise SchemaTypeError("enum value {!r} has no identifier form".format(value))
        return "V" + ident if ident[0].isdigit() else ident

    def enum_type(self, name: str, values: List[str], authority: str) -> str:
        if name in self._emitted:
            return self._emitted[name]
        registry = self.registry_ordinals.get(authority)
        if registry is not None:
            by_name = dict(registry)
            missing = [v for v in values if v not in by_name]
            if missing:
                raise SchemaTypeError(
                    "{} values {} are not registered in {}".format(name, missing, authority)
                )
            members = [(self.variant(v), by_name[v], v) for v in values]
            source = "ids/index.json:" + authority
        else:
            members = [(self.variant(v), i, v) for i, v in enumerate(values)]
            source = "schema declaration order"
        self.enums.append((name, members, source))
        self._emitted[name] = name
        return name

    def type_of(
        self, node: Dict[str, Any], hint: str, required: bool
    ) -> Tuple[str, str]:
        """Return (rust_type, csharp_type) for one schema node."""
        node = self.resolve(node)
        kind = node.get("type")

        if "enum" in node and kind == "string":
            authority = "MessageType" if hint.endswith("MessageType") else ""
            name = self.enum_type(hint, [str(v) for v in node["enum"]], authority)
            rust, cs = name, name
        elif kind == "string":
            rust, cs = "String", "string"
        elif kind == "boolean":
            rust, cs = "bool", "bool"
        elif kind == "integer":
            unsigned = node.get("minimum") is not None and node["minimum"] >= 0
            rust, cs = ("u64", "ulong") if unsigned else ("i64", "long")
        elif kind == "number":
            # Only config-table column bounds are `number`; they are schema
            # metadata, not payload bytes, so LumioBinV1's no-float rule is
            # untouched by carrying them as a double here.
            rust, cs = "f64", "double"
        elif kind == "array":
            item_rust, item_cs = self.type_of(
                node.get("items") or {"type": "string"}, hint + "Item", True
            )
            rust, cs = "Vec<{}>".format(item_rust), "IReadOnlyList<{}>".format(item_cs)
        elif kind == "object":
            if node.get("patternProperties"):
                value_node = list(node["patternProperties"].values())[0]
                v_rust, v_cs = self.type_of(value_node, hint + "Value", True)
                rust = "BTreeMap<String, {}>".format(v_rust)
                cs = "IReadOnlyDictionary<string, {}>".format(v_cs)
            elif node.get("properties"):
                name = self.struct_type(hint, node)
                rust, cs = name, name
            else:
                # A deliberately open object (`body`, `inner`, config `values`).
                # Nothing in the architecture source closes it, so it is carried
                # verbatim rather than given an invented shape.
                rust, cs = "OpaqueJson", "OpaqueJson"
        elif kind is None and not node:
            # An untyped member (`defaultValue`): the column's own `type` decides
            # it at validation time, so there is no single static projection.
            rust, cs = "OpaqueJson", "OpaqueJson"
        else:
            raise SchemaTypeError("no type projection for {} ({})".format(hint, kind))

        if not required:
            rust = "Option<{}>".format(rust)
            cs = cs + "?"
        return rust, cs

    def struct_type(self, name: str, schema: Dict[str, Any]) -> str:
        if name in self._emitted:
            return self._emitted[name]
        self._emitted[name] = name
        members: List[Tuple[str, str, str, bool, str]] = []
        for field, node, required in self.declaration_order(schema):
            rust, cs = self.type_of(node, name + pascal_member(field), required)
            members.append((field, rust, cs, required, self.resolve(node).get("type", "")))
        self.structs.append((name, members))
        return name

    def project(self) -> None:
        for schema_id, type_name in CLOSED_CONTRACT_TYPES:
            schema = json.loads(
                (self.root / "schemas" / "{}.schema.json".format(schema_id)).read_text(
                    encoding="utf-8"
                )
            )
            self.struct_type(type_name, self.resolve(schema))


def contract_bodies_rust(projector: TypeProjector) -> str:
    out = [
        "//! ADR-048 (D-3): closed contract type bodies, generated from `schemas/`.\n",
        "//! Field order is the schema declaration order. Do not hand-edit.\n\n",
        "use std::collections::BTreeMap;\n\n",
        "/// An object the architecture source deliberately leaves open (a\n"
        "/// replication `body`, a WAL `inner`, a config row's `values`). Carried\n"
        "/// verbatim as its canonical JSON text; this crate does not invent a shape\n"
        "/// for something no ADR has closed.\n"
        "#[derive(Clone, Debug, PartialEq, Eq)]\n"
        "pub struct OpaqueJson(pub String);\n\n",
    ]
    for name, members, source in projector.enums:
        out.append("/// Ordinal authority: {}.\n".format(source))
        out.append("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum {} {{\n".format(name))
        for member, _ordinal, wire in members:
            if member != wire:
                out.append("    /// Wire value `{}`.\n".format(wire))
            out.append("    {},\n".format(member))
        out.append("}\n\n")
        out.append("impl {} {{\n".format(name))
        out.append("    pub const fn ordinal(self) -> u32 {\n        match self {\n")
        for member, ordinal, _wire in members:
            out.append("            {}::{} => {},\n".format(name, member, ordinal))
        out.append("        }\n    }\n\n")
        out.append("    /// The value that crosses the wire, which the identifier may not equal.\n")
        out.append("    pub const fn wire_value(self) -> &'static str {\n        match self {\n")
        for member, _ordinal, wire in members:
            out.append("            {}::{} => \"{}\",\n".format(name, member, wire))
        out.append("        }\n    }\n}\n\n")
    for name, members in projector.structs:
        out.append("/// Fields in schema declaration order.\n")
        out.append("#[derive(Clone, Debug, PartialEq)]\npub struct {} {{\n".format(name))
        for field, rust, _cs, required, _kind in members:
            if not required:
                out.append("    /// Optional in the schema.\n")
            out.append("    pub {}: {},\n".format(snake(field), rust))
        out.append("}\n\n")
        out.append(
            "impl {name} {{\n    /// The schema declaration order this type was generated from.\n"
            "    pub const FIELD_ORDER: &'static [&'static str] = &[{fields}];\n}}\n\n".format(
                name=name, fields=", ".join('"{}"'.format(f) for f, _r, _c, _q, _k in members)
            )
        )
    return "".join(out)


def contract_bodies_csharp(projector: TypeProjector) -> str:
    body = [
        "// ADR-048 (D-3): closed contract type bodies, generated from schemas/.\n",
        "// Field order is the schema declaration order. Do not hand-edit.\n\n",
        "/// <summary>An object the architecture source deliberately leaves open (a\n"
        "/// replication body, a WAL inner, a config row's values). Carried verbatim as\n"
        "/// its canonical JSON text; no shape is invented for what no ADR has closed.</summary>\n"
        "public sealed class OpaqueJson\n{\n"
        "    public OpaqueJson(string json) { Json = json; }\n"
        "    public string Json { get; }\n}\n\n",
    ]
    for name, members, source in projector.enums:
        body.append("/// <summary>Ordinal authority: {}.</summary>\n".format(source))
        body.append("public enum {}\n{{\n".format(name))
        for member, ordinal, wire in members:
            if member != wire:
                body.append("    /// <summary>Wire value <c>{}</c>.</summary>\n".format(wire))
            body.append("    {} = {},\n".format(member, ordinal))
        body.append("}\n\n")
        body.append("public static class {}Wire\n{{\n".format(name))
        body.append("    /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>\n")
        body.append("    public static string Value({} value)\n    {{\n        switch (value)\n        {{\n".format(name))
        for member, _ordinal, wire in members:
            body.append("            case {}.{}: return \"{}\";\n".format(name, member, wire))
        body.append("            default: return string.Empty;\n        }\n    }\n}\n\n")
    for name, members in projector.structs:
        params = ", ".join("{} {}".format(cs, cs_param(pascal_member(f))) for f, _r, cs, _q, _k in members)
        body.append("/// <summary>Fields in schema declaration order.</summary>\n")
        body.append("public sealed class {}\n{{\n".format(name))
        body.append("    public {}({})\n    {{\n".format(name, params))
        for field, _rust, _cs, _required, _kind in members:
            member = pascal_member(field)
            body.append("        {} = {};\n".format(member, cs_param(member)))
        body.append("    }\n\n")
        for field, _rust, cs, required, _kind in members:
            if not required:
                body.append("    /// <summary>Optional in the schema.</summary>\n")
            body.append("    public {} {} {{ get; }}\n".format(cs, pascal_member(field)))
        body.append("\n    public static readonly string[] FieldOrder =\n    {\n")
        for field, _rust, _cs, _required, _kind in members:
            body.append("        \"{}\",\n".format(field))
        body.append("    };\n}\n\n")
    return cs_namespace(
        "Lumio.Gen.ContractTypes",
        "".join(body),
        usings="using System.Collections.Generic;\n\n",
    )


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
        cs_namespace(
            "Lumio.Gen.MappingTable",
            "public static class MappingContract\n{\n"
            "    public static readonly string[] Roles = { \"ServerToClient\", \"ClientToServer\", \"SharedProjection\" };\n}\n",
        ),
    )
    write_text(cs_dir / (CS_PROJ["MappingTable"] + ".csproj"), csproj(CS_PROJ["MappingTable"]))


def emit_canonical(
    rust_dir: Path,
    cs_dir: Path,
    profile: Dict[str, Any],
    bin_profile: Dict[str, Any],
    checksum_doc: str,
) -> None:
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
    write_text(rust_dir / "src" / "lib.rs", rust + "\n" + lumio_bin_rust(bin_profile))
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
        "pub struct NormalizationStep {\n"
        "    pub path: &'static str,\n"
        "    pub op: &'static str,\n"
        "    pub by: &'static str,\n"
        "    pub collation: &'static str,\n"
        "}\n\n"
        "#[derive(Clone, Copy, Debug)]\n"
        "pub struct DigestDomain {\n"
        "    pub digest: &'static str,\n"
        "    pub domain_tag: &'static str,\n"
        "    pub sort_rule: &'static str,\n"
        "    pub omit_members: &'static [&'static str],\n"
        "    /// Executed in declared order, before canonicalization.\n"
        "    pub normalization: &'static [NormalizationStep],\n"
        "}\n\n"
    )
    rust += "pub const DIGEST_DOMAINS: &[DigestDomain] = &[\n"
    for domain in profile["digestDomains"]:
        omit = ", ".join("\"%s\"" % name for name in domain.get("omitMembers", []))
        steps = ", ".join(
            "NormalizationStep { path: \"%s\", op: \"%s\", by: \"%s\", collation: \"%s\" }"
            % (step["path"], step["op"], step["by"], step["collation"])
            for step in domain.get("normalization", [])
        )
        rust += (
            "    DigestDomain { digest: \"%s\", domain_tag: \"%s\", sort_rule: \"%s\", omit_members: &[%s], normalization: &[%s] },\n"
            % (domain["digest"], domain["domainTag"], domain["sortRule"], omit, steps)
        )
    rust += "];\n\n"
    rust += "/// Golden vectors: `(id, case, sha256)`. Full inputs and canonical bytes are in\n"
    rust += "/// the published `canonical/canonical-digest-profile.json`.\n"
    rust += "pub const CANONICAL_GOLDENS: &[(&str, &str, &str)] = &[\n"
    for golden in profile["goldens"]:
        rust += "    (\"%s\", \"%s\", \"%s\"),\n" % (golden["id"], golden["case"], golden["sha256"])
    rust += "];\n"
    write_text(rust_dir / "CHECKSUM_DOMAIN.md", checksum_doc)
    write_text(
        cs_dir / "CanonicalSerializer.cs",
        cs_namespace(
            "Lumio.Gen.CanonicalSerializer",
            "public static class SnapshotChecksum\n{\n"
            "    public const string Domain = \"SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields\";\n"
            "    public const string Magic = \"LUMIOSNP1\";\n}\n",
        ),
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
    cs_extra.append(cs_value_struct("NormalizationStep", [("string", "Path"), ("string", "Op"), ("string", "By"), ("string", "Collation")]))
    cs_extra.append(cs_value_struct("DigestDomain", [("string", "Digest"), ("string", "DomainTag"), ("string", "SortRule"), ("string[]", "OmitMembers"), ("NormalizationStep[]", "Normalization")]))
    cs_extra.append("public static class DigestDomains\n{\n    public static readonly DigestDomain[] All =\n    {\n")
    for domain in profile["digestDomains"]:
        omit = ", ".join("\"%s\"" % name for name in domain.get("omitMembers", []))
        cs_extra.append(
            "        new DigestDomain(\"%s\", \"%s\", \"%s\", %s, %s),\n"
            % (
                domain["digest"],
                domain["domainTag"],
                domain["sortRule"],
                ("new[] { %s }" % omit) if omit else "System.Array.Empty<string>()",
                (
                    "new[] { %s }"
                    % ", ".join(
                        "new NormalizationStep(\"%s\", \"%s\", \"%s\", \"%s\")"
                        % (step["path"], step["op"], step["by"], step["collation"])
                        for step in domain.get("normalization", [])
                    )
                )
                if domain.get("normalization")
                else "System.Array.Empty<NormalizationStep>()",
            )
        )
    cs_extra.append("    };\n}\n\n")
    cs_extra.append(cs_value_struct("CanonicalGolden", [("string", "Id"), ("string", "Case"), ("string", "Sha256")]))
    cs_extra.append("public static class CanonicalGoldens\n{\n    public static readonly CanonicalGolden[] All =\n    {\n")
    for golden in profile["goldens"]:
        cs_extra.append(
            "        new CanonicalGolden(\"%s\", \"%s\", \"%s\"),\n"
            % (golden["id"], golden["case"], golden["sha256"])
        )
    cs_extra.append("    };\n}\n")
    write_text(
        cs_dir / "CanonicalProfile.cs",
        cs_namespace("Lumio.Gen.CanonicalSerializer", "".join(cs_extra), usings="using System;\n\n"),
    )
    write_text(cs_dir / "LumioBinProfile.cs", lumio_bin_csharp(bin_profile))
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
        # A plain readonly struct, not a positional record: `record` needs
        # `IsExternalInit`, which netstandard2.1 does not ship.
        "public readonly struct Binding\n{\n"
        "    public Binding(string schemaId, string rustType, string csharpType)\n    {\n"
        "        SchemaId = schemaId; RustType = rustType; CsharpType = csharpType;\n    }\n"
        "    public string SchemaId { get; }\n"
        "    public string RustType { get; }\n"
        "    public string CsharpType { get; }\n}\n\n",
        "public static class Bindings\n{\n    public static readonly Binding[] All =\n    {\n",
    ]
    for sid in schema_ids:
        pascal = "".join(p.title() for p in sid.replace("_", "-").split("-"))
        cs_lines.append("        new Binding(\"%s\", \"%s\", \"%s\"),\n" % (sid, pascal, pascal))
    cs_lines.append("    };\n}\n")
    write_text(cs_dir / "Bindings.cs", cs_namespace("Lumio.Gen.LanguageBinding", "".join(cs_lines)))
    write_text(cs_dir / (CS_PROJ["LanguageBinding"] + ".csproj"), csproj(CS_PROJ["LanguageBinding"]))


def emit_contract_types(
    rust_dir: Path,
    cs_dir: Path,
    machines: List[Dict[str, Any]],
    schema_ids: List[str],
    errors: List[str],
    abi: Dict[str, Any],
    projector: "TypeProjector",
) -> None:
    rust = rust_lib_header("ContractTypes")
    rust += "pub mod bodies;\n\n"
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
    cs = []
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
    cs.append(cs_value_struct("AbiTypeMapping", [("string", "TypeRef"), ("string", "C"), ("string", "Csharp"), ("string", "Rust"), ("int", "Size"), ("int", "Align")]))
    cs.append("public static class AbiTypeMappings\n{\n    public static readonly AbiTypeMapping[] All =\n    {\n")
    for key, c_type, cs_type, rust_type, size, align in ABI_TYPE_MAPPING:
        cs.append(
            "        new AbiTypeMapping(\"%s\", \"%s\", \"%s\", \"%s\", %d, %d),\n"
            % (key, c_type, cs_type, rust_type, size, align)
        )
    cs.append("    };\n}\n\n")
    cs.append(cs_value_struct("Transition", [("string", "Machine"), ("string", "From"), ("string", "To"), ("string", "Event")]))
    cs.append("public static class StateTransitionTable\n{\n    public static readonly Transition[] All =\n    {\n")
    for machine in machines:
        mid = machine.get("machineId", "")
        for tr in machine.get("transitions") or []:
            cs.append(
                "        new Transition(\"%s\", \"%s\", \"%s\", \"%s\"),\n"
                % (mid, tr.get("from", ""), tr.get("to", ""), tr.get("event", ""))
            )
    cs.append("    };\n}\n")
    write_text(cs_dir / "ContractTypes.cs", cs_namespace("Lumio.Gen.ContractTypes", "".join(cs)))
    write_text(rust_dir / "src" / "bodies.rs", contract_bodies_rust(projector))
    write_text(cs_dir / "ContractBodies.cs", contract_bodies_csharp(projector))
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
        "}\n" + kat_test_rs(),
    )
    write_text(
        cs_dir / "ContractRuntime.cs",
        cs_namespace(
            "Lumio.Gen.ContractRuntime",
        "public enum ChainBreak { Truncated, Mismatch }\n\n"
        "public static class HashChain\n{\n"
        "    public static byte[] Append(byte[] prev, byte[] payload)\n    {\n"
        "        var buf = new byte[prev.Length + payload.Length];\n"
        "        Buffer.BlockCopy(prev, 0, buf, 0, prev.Length);\n"
        "        Buffer.BlockCopy(payload, 0, buf, prev.Length, payload.Length);\n"
        "        return Sha256(buf);\n    }\n"
        "    public static bool Verify(byte[] prev, byte[] payload, byte[] expected)\n    {\n"
        "        var got = Append(prev, payload);\n"
        "        if (got.Length != expected.Length) return false;\n"
        "        for (var i = 0; i < got.Length; i++) { if (got[i] != expected[i]) return false; }\n"
        "        return true;\n    }\n"
        # SHA256.Create()/ComputeHash, not SHA256.HashData: the static one-shot
        # is net5.0+ and this package also targets netstandard2.1.
        "    public static byte[] Sha256(byte[] data)\n    {\n"
        "        using (var sha = SHA256.Create()) { return sha.ComputeHash(data); }\n    }\n}\n\n"
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
            usings="using System;\nusing System.Security.Cryptography;\nusing System.Text;\n\n",
        ),
    )
    write_text(cs_dir / (CS_PROJ["ContractRuntime"] + ".csproj"), csproj(CS_PROJ["ContractRuntime"]))


# --------------------------------------------------------------------------
# ADR-041 Canonical and Digest Profiles
# --------------------------------------------------------------------------

CANONICAL_PROFILE_ID = "canonical-digest-v1"
CANONICAL_PROFILE_FILE = "canonical/canonical-digest-profile.json"
TRUST_PROFILE_FILE = "trust/trust-profile.json"
LOADER_PROFILE_FILE = "loader/loader-profile.json"
EVIDENCE_PROFILE_FILE = "evidence/evidence-profile.json"
LOADER_PROFILE_CONSUMERS = ["LumioCoreEngine"]
EVIDENCE_PROFILE_CONSUMERS = ["LumioCoreEngine"]
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
# `normalization` is the machine-readable half: it is what the generator and the
# gate actually execute, and what a downstream must execute to reproduce the
# Goldens. `sortRule` stays as the human-readable gloss of the same rule.
CANONICAL_DIGEST_DOMAINS = [
    {
        "digest": "manifestDigest",
        "domainTag": "CoreEngineManifestBody",
        "input": "the CoreEngineManifestBody document itself (ADR-018; the one input with no digestDomain member)",
        "sortRule": "member order only; the body has no array whose order is semantic",
        "normalization": [],
    },
    {
        "digest": "artifactSetDigest",
        "domainTag": "ArtifactSetV1",
        "input": "the ArtifactIndex with artifactSetDigest omitted, wrapped as {digestDomain,indexVersion,targetProfileId,entries}",
        "sortRule": "entries sorted ascending by path (code point); paths are unique within an index",
        "omitMembers": ["artifactSetDigest"],
        "normalization": [
            {"path": "entries", "op": "sortAscending", "by": "path", "collation": "codePoint"}
        ],
    },
    {
        "digest": "artifactIndexDigest",
        "domainTag": "ArtifactIndexV1",
        "input": "the complete ArtifactIndex document including artifactSetDigest, wrapped as {digestDomain,index}",
        "sortRule": "index.entries sorted ascending by path (code point)",
        "normalization": [
            {"path": "index.entries", "op": "sortAscending", "by": "path", "collation": "codePoint"}
        ],
    },
    {
        "digest": "targetProfileDigest",
        "domainTag": "TargetProfileV1",
        "input": "the complete TargetProfile document, wrapped as {digestDomain,profile}",
        "sortRule": "member order only; the profile has no array",
        "normalization": [],
    },
    {
        "digest": "capabilitySetDigest",
        "domainTag": "CapabilitySetV1",
        "input": "the capability id list, wrapped as {digestDomain,capabilities}",
        "sortRule": "capabilities sorted ascending by code point; the array is uniqueItems so ties are impossible",
        "normalization": [
            {"path": "capabilities", "op": "sortAscending", "by": "$self", "collation": "codePoint"}
        ],
    },
    {
        "digest": "mappingSetHash",
        "domainTag": "ReplicationMappingSetV1",
        "input": "the registered mappingId list, wrapped as {digestDomain,mappings}",
        "sortRule": "mappings sorted ascending by code point; mappingId is unique within a set so ties are impossible",
        "normalization": [
            {"path": "mappings", "op": "sortAscending", "by": "$self", "collation": "codePoint"}
        ],
    },
]
CANONICAL_NORMALIZATION_BY_TAG = {
    item["domainTag"]: item.get("normalization") or [] for item in CANONICAL_DIGEST_DOMAINS
}
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
    "EmptyMappingSet",
    "MappingOrderPermutation",
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


def _sort_key(item: Any, by: str) -> str:
    """`$self` sorts the element itself; anything else sorts by that member."""
    if by == "$self":
        return str(item)
    return str(item.get(by, "")) if isinstance(item, dict) else ""


def _apply_normalization_step(value: Any, step: Dict[str, Any]) -> Any:
    """Execute one published normalization step against a dotted member path."""
    if step.get("op") != "sortAscending" or step.get("collation") != "codePoint":
        raise CanonicalError("unsupported normalization step {}".format(step))
    parts = str(step.get("path", "")).split(".")
    out = dict(value)
    cursor = out
    for name in parts[:-1]:
        child = cursor.get(name)
        if not isinstance(child, dict):
            return out
        child = dict(child)
        cursor[name] = child
        cursor = child
    leaf = parts[-1]
    target = cursor.get(leaf)
    if isinstance(target, list):
        cursor[leaf] = sorted(target, key=lambda item: _sort_key(item, str(step.get("by", "$self"))))
    return out


def apply_digest_domain_sort(value: Any) -> Any:
    """Execute the domain's published `normalization` steps, in declared order.

    The generator, the gate and any downstream run the same declaration, so a
    consumer that reads only the published profile reproduces the Goldens.
    """
    if not isinstance(value, dict):
        return value
    tag = value.get("digestDomain")
    if tag not in CANONICAL_DOMAIN_TAGS:
        return value
    out = value
    for step in CANONICAL_NORMALIZATION_BY_TAG.get(tag, []):
        out = _apply_normalization_step(out, step)
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


def replication_mapping_set_digest_input(mappings: List[str]) -> Dict[str, Any]:
    """ADR-045 section 2: the FullSnapshot/Delta `mappingSetHash` preimage.

    The empty mapping set is not a special case and needs no sentinel constant:
    an empty `mappings` array runs the same rule and yields a defined digest.
    """
    return {"digestDomain": "ReplicationMappingSetV1", "mappings": list(mappings)}


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
        _golden(
            "replication-mapping-set-empty",
            "EmptyMappingSet",
            replication_mapping_set_digest_input([]),
        ),
        _golden(
            "replication-mapping-set-permutation",
            "MappingOrderPermutation",
            replication_mapping_set_digest_input(
                ["mapping-voxel-chunk", "mapping-actor-transform", "mapping-actor-health"]
            ),
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


# --------------------------------------------------------------------------
# ADR-047 LumioBinV1 binary canonical profile
# --------------------------------------------------------------------------

LUMIO_BIN_PROFILE_ID = "lumio-bin-v1"
LUMIO_BIN_PROFILE_FILE = "binary/lumio-bin-profile.json"
# ADR-010 names GameRuntime the format owner and Voxel/Game the domain payload
# owners; ADR-035 makes the voxel payload bytes public to every conforming
# encoder, and Server persists them. CoreEngine consumes CanonicalJsonV1 only.
LUMIO_BIN_PROFILE_CONSUMERS = ["LumioGame", "LumioGameRuntime", "LumioServer", "LumioVoxelEngine"]
LUMIO_BIN_FORM = {
    "formId": "LumioBinV1",
    "byteOrder": "LittleEndian",
    "integers": {
        "u8": {"bytes": 1, "signed": False},
        "u16": {"bytes": 2, "signed": False},
        "u32": {"bytes": 4, "signed": False},
        "u64": {"bytes": 8, "signed": False},
        "i32": {"bytes": 4, "signed": True, "encoding": "TwosComplement"},
        "i64": {"bytes": 8, "signed": True, "encoding": "TwosComplement"},
    },
    "strings": {"encoding": "Utf8", "lengthPrefix": "u32", "lengthUnit": "Bytes"},
    "bytes": {"lengthPrefix": "u32", "lengthUnit": "Bytes"},
    "arrays": {"countPrefix": "u32", "order": "DocumentOrder"},
    "structs": {
        "fieldOrder": "SchemaDeclarationOrder",
        "padding": "None",
        "missingFields": "Reject",
        "unknownFields": "Reject",
    },
    "floats": "None",
}
# `framing` deliberately does NOT reuse ADR-041's `PrefixFreeOverEncodedBytes`
# spelling. Two independent clean-room implementations read that name as an
# instruction to *add* a prefix; one prepended a length and matched every Golden's
# bytes while missing every digest, silently. `None` cannot be read that way, and
# `digestInput` states the input positively. That the encoding is itself
# prefix-free is a property of the form, argued in ADR-047 section 2 — it is not
# an operation the digest performs, so it is not published as one.
LUMIO_BIN_DIGEST_ALGORITHM = {
    "name": "SHA-256",
    "framing": "None",
    "digestInput": "EncodedBytesOnly",
}
# `case` is a human-readable label and `error` is the contract: several rejection
# cases share one error (both malformed-hex spellings are a `TypeMismatch`), so a
# consumer that keys conformance on `case` invents error names that do not exist.
# Published as data because a clean-room reader otherwise has to guess which field
# is normative.
LUMIO_BIN_VECTOR_SEMANTICS = {"case": "HumanLabel", "error": "Normative"}
# How a Golden's `value` member carries each layout kind inside the published
# JSON. Without this a consumer cannot tell how to read the vectors it must
# reproduce, and byte arrays in particular have no natural JSON spelling.
LUMIO_BIN_VALUE_ENCODING = {
    # Named for the literal spelling, not the value: `1.0` and `1e2` are integral
    # in value and non-integer as literals, and LumioBinV1 refuses both. The older
    # `JsonIntegerNumbers` spelling left a clean-room reader to guess which reading
    # applied, and only one published vector discriminated them.
    "integers": "JsonIntegerLiteralsNoFractionOrExponent",
    # A double-backed JSON reader (any stock JavaScript `JSON.parse`) rounds the
    # u64 Golden's 18446744073709551615 to 2**64 and then rejects a valid vector.
    # The requirement is therefore published, not assumed.
    "integerPrecision": "ExactArbitraryPrecision",
    "strings": "JsonStrings",
    "bytes": "LowercaseHexJsonStrings",
    "arrays": "JsonArraysDocumentOrder",
    "structs": "JsonObjectsEveryDeclaredFieldNoExtras",
}
LUMIO_BIN_GOLDEN_CASES = [
    "IntegerWidthsLittleEndian",
    "StringUtf8ByteLength",
    "BytesLengthPrefix",
    "ArrayCountPrefix",
    "DeclarationOrderNoPadding",
    "NestedComposition",
]
LUMIO_BIN_REJECTION_CASES = [
    "IntegerRangeOverflow",
    "UnsignedNegative",
    "NonIntegerNumber",
    "IntegralFloat",
    "BooleanForInteger",
    "TypeMismatch",
    "MalformedHexBytes",
    "UnknownLayoutKind",
    "MissingField",
    "UnknownField",
]
# width in bytes, and the closed inclusive range the width admits.
LUMIO_BIN_INTEGERS: Dict[str, Tuple[int, bool, int, int]] = {
    "u8": (1, False, 0, 2 ** 8 - 1),
    "u16": (2, False, 0, 2 ** 16 - 1),
    "u32": (4, False, 0, 2 ** 32 - 1),
    "u64": (8, False, 0, 2 ** 64 - 1),
    "i32": (4, True, -(2 ** 31), 2 ** 31 - 1),
    "i64": (8, True, -(2 ** 63), 2 ** 63 - 1),
}
_LUMIO_BIN_HEX = re.compile(r"^([0-9a-f]{2})*$")


class LumioBinError(RuntimeError):
    """Raised when a value cannot be encoded under LumioBinV1.

    `code` is the published rejection reason: a downstream that reproduces the
    profile's rejection vectors must refuse the same input for the same reason,
    not merely fail somehow.
    """

    def __init__(self, code: str, message: str) -> None:
        super().__init__("{}: {}".format(code, message))
        self.code = code


def lumio_bin_encode(layout: Any, value: Any, path: str = "$") -> bytes:
    """Encode one value under the LumioBinV1 layout. Little-endian, no padding."""
    if not isinstance(layout, dict):
        raise LumioBinError("UnknownLayoutKind", "{} has no layout object".format(path))
    kind = layout.get("kind")

    if kind in LUMIO_BIN_INTEGERS:
        width, signed, low, high = LUMIO_BIN_INTEGERS[str(kind)]
        if isinstance(value, float):
            raise LumioBinError("NonIntegerNumber", "{} is not an integer".format(path))
        if isinstance(value, bool) or not isinstance(value, int):
            raise LumioBinError("TypeMismatch", "{} must be an integer for {}".format(path, kind))
        if value < low or value > high:
            raise LumioBinError(
                "IntegerRangeOverflow", "{} = {} is outside the {} range".format(path, value, kind)
            )
        return value.to_bytes(width, "little", signed=signed)

    if kind == "string":
        if not isinstance(value, str):
            raise LumioBinError("TypeMismatch", "{} must be a string".format(path))
        payload = value.encode("utf-8")
        return lumio_bin_encode({"kind": "u32"}, len(payload), path + ".length") + payload

    if kind == "bytes":
        if not isinstance(value, str) or _LUMIO_BIN_HEX.match(value) is None:
            raise LumioBinError(
                "TypeMismatch", "{} must be a lower-case hex string of whole bytes".format(path)
            )
        payload = bytes.fromhex(value)
        return lumio_bin_encode({"kind": "u32"}, len(payload), path + ".length") + payload

    if kind == "array":
        if not isinstance(value, list):
            raise LumioBinError("TypeMismatch", "{} must be an array".format(path))
        out = lumio_bin_encode({"kind": "u32"}, len(value), path + ".count")
        for index, item in enumerate(value):
            out += lumio_bin_encode(layout.get("items"), item, "{}[{}]".format(path, index))
        return out

    if kind == "struct":
        if not isinstance(value, dict):
            raise LumioBinError("TypeMismatch", "{} must be an object".format(path))
        fields = layout.get("fields") or []
        declared = [str(field.get("name")) for field in fields]
        for name in declared:
            if name not in value:
                raise LumioBinError("MissingField", "{} is missing field {}".format(path, name))
        for name in value:
            if name not in declared:
                raise LumioBinError("UnknownField", "{} carries unknown field {}".format(path, name))
        out = b""
        # Declaration order, never member-name order: this is the one rule that
        # a JSON-shaped input cannot carry and a consumer cannot infer.
        for field in fields:
            out += lumio_bin_encode(
                field.get("layout"), value[str(field.get("name"))], "{}.{}".format(path, field.get("name"))
            )
        return out

    raise LumioBinError("UnknownLayoutKind", "{} has unknown layout kind {!r}".format(path, kind))


def _lumio_bin_golden(golden_id: str, case: str, layout: Any, value: Any) -> Dict[str, Any]:
    payload = lumio_bin_encode(layout, value)
    return {
        "id": golden_id,
        "case": case,
        "layout": layout,
        "value": value,
        "bytesHex": payload.hex(),
        "sha256": sha256_bytes(payload),
    }


def _lumio_bin_rejection(rejection_id: str, case: str, layout: Any, value: Any, error: str) -> Dict[str, Any]:
    """Record a vector the encoder must refuse — and prove it refuses it here."""
    try:
        lumio_bin_encode(layout, value)
    except LumioBinError as exc:
        if exc.code != error:
            raise LumioBinError(
                exc.code, "rejection {} expected {} but the encoder raised {}".format(rejection_id, error, exc.code)
            )
        return {"id": rejection_id, "case": case, "layout": layout, "value": value, "error": error}
    raise LumioBinError(error, "rejection {} was accepted by the encoder".format(rejection_id))


# A struct whose declaration order is deliberately not the member-name order and
# whose widths do not align: an encoder that sorts members, or pads to natural
# alignment, produces different bytes for it and reproduces no other vector.
LUMIO_BIN_UNALIGNED_STRUCT = {
    "kind": "struct",
    "fields": [
        {"name": "zeta", "layout": {"kind": "u8"}},
        {"name": "alpha", "layout": {"kind": "u64"}},
        {"name": "mid", "layout": {"kind": "u8"}},
        {"name": "beta", "layout": {"kind": "u32"}},
    ],
}
LUMIO_BIN_NESTED_STRUCT = {
    "kind": "struct",
    "fields": [
        {"name": "chunkRevision", "layout": {"kind": "u64"}},
        {
            "name": "pages",
            "layout": {
                "kind": "array",
                "items": {
                    "kind": "struct",
                    "fields": [
                        {"name": "pageIndex", "layout": {"kind": "u32"}},
                        {"name": "label", "layout": {"kind": "string"}},
                        {"name": "digest", "layout": {"kind": "bytes"}},
                    ],
                },
            },
        },
        {"name": "trailer", "layout": {"kind": "i32"}},
    ],
}


def derive_lumio_bin_profile() -> Dict[str, Any]:
    """Derive the ADR-047 LumioBinV1 profile with its self-verifying vectors.

    Every Golden's bytes and digest, and every rejection, are recomputed from
    the declared layout and value here, so a published vector cannot rot into a
    lie the way a hand-copied byte string can.
    """
    goldens = [
        _lumio_bin_golden(
            "integer-widths",
            "IntegerWidthsLittleEndian",
            {
                "kind": "struct",
                "fields": [
                    {"name": "u8Value", "layout": {"kind": "u8"}},
                    {"name": "u16Value", "layout": {"kind": "u16"}},
                    {"name": "u32Value", "layout": {"kind": "u32"}},
                    {"name": "u64Value", "layout": {"kind": "u64"}},
                    {"name": "i32Value", "layout": {"kind": "i32"}},
                    {"name": "i64Value", "layout": {"kind": "i64"}},
                ],
            },
            {
                "u8Value": 255,
                "u16Value": 258,
                "u32Value": 16909060,
                "u64Value": 18446744073709551615,
                "i32Value": -2,
                "i64Value": -9223372036854775808,
            },
        ),
        # 10 code points, 16 UTF-8 bytes: an encoder that prefixes the character
        # count, or that escapes to ASCII the way CanonicalJsonV1 does, misses.
        _lumio_bin_golden(
            "string-utf8",
            "StringUtf8ByteLength",
            {"kind": "string"},
            "aé世\U0001d11e chunk",
        ),
        _lumio_bin_golden("bytes-prefixed", "BytesLengthPrefix", {"kind": "bytes"}, "00ff10a0"),
        _lumio_bin_golden(
            "array-count",
            "ArrayCountPrefix",
            {"kind": "array", "items": {"kind": "u32"}},
            [1, 256, 65536],
        ),
        _lumio_bin_golden(
            "struct-declaration-order",
            "DeclarationOrderNoPadding",
            LUMIO_BIN_UNALIGNED_STRUCT,
            {"zeta": 1, "alpha": 2, "mid": 3, "beta": 4},
        ),
        _lumio_bin_golden(
            "nested-composition",
            "NestedComposition",
            LUMIO_BIN_NESTED_STRUCT,
            {
                "chunkRevision": 24,
                "pages": [
                    {"pageIndex": 0, "label": "page-0", "digest": "0a0b"},
                    {"pageIndex": 1, "label": "", "digest": ""},
                ],
                "trailer": -1,
            },
        ),
    ]
    rejections = [
        _lumio_bin_rejection(
            "u8-above-range", "IntegerRangeOverflow", {"kind": "u8"}, 256, "IntegerRangeOverflow"
        ),
        _lumio_bin_rejection(
            "u32-negative", "UnsignedNegative", {"kind": "u32"}, -1, "IntegerRangeOverflow"
        ),
        _lumio_bin_rejection(
            "u32-fractional", "NonIntegerNumber", {"kind": "u32"}, 1.5, "NonIntegerNumber"
        ),
        # `1.5` is refused by both a spelling-based and a value-based reading, so
        # it does not discriminate. `1.0` is the case that does: it is an integer
        # by value and a non-integer by JSON spelling, and LumioBinV1 reads the
        # spelling. Without this vector two conforming encoders disagree.
        _lumio_bin_rejection(
            "u32-integral-float", "IntegralFloat", {"kind": "u32"}, 1.0, "NonIntegerNumber"
        ),
        _lumio_bin_rejection("u32-string", "TypeMismatch", {"kind": "u32"}, "7", "TypeMismatch"),
        _lumio_bin_rejection(
            "u32-boolean", "BooleanForInteger", {"kind": "u32"}, True, "TypeMismatch"
        ),
        # `bytes` values are lower-case hex; an odd-length, upper-case or non-hex
        # string is a TypeMismatch, not a private error a downstream has to invent.
        _lumio_bin_rejection(
            "bytes-odd-length", "MalformedHexBytes", {"kind": "bytes"}, "0a0", "TypeMismatch"
        ),
        _lumio_bin_rejection(
            "bytes-upper-case", "MalformedHexBytes", {"kind": "bytes"}, "0A0B", "TypeMismatch"
        ),
        _lumio_bin_rejection(
            "bytes-non-hex", "MalformedHexBytes", {"kind": "bytes"}, "zz", "TypeMismatch"
        ),
        _lumio_bin_rejection("f32-layout", "UnknownLayoutKind", {"kind": "f32"}, 1, "UnknownLayoutKind"),
        _lumio_bin_rejection(
            "struct-missing-field",
            "MissingField",
            LUMIO_BIN_UNALIGNED_STRUCT,
            {"zeta": 1, "alpha": 2, "mid": 3},
            "MissingField",
        ),
        _lumio_bin_rejection(
            "struct-unknown-field",
            "UnknownField",
            LUMIO_BIN_UNALIGNED_STRUCT,
            {"zeta": 1, "alpha": 2, "mid": 3, "beta": 4, "omega": 5},
            "UnknownField",
        ),
    ]
    return {
        "profileId": LUMIO_BIN_PROFILE_ID,
        "baselineId": BASELINE,
        "schemaEpoch": 1,
        "binaryForm": json.loads(json.dumps(LUMIO_BIN_FORM)),
        "digestAlgorithm": dict(LUMIO_BIN_DIGEST_ALGORITHM),
        "valueEncoding": dict(LUMIO_BIN_VALUE_ENCODING),
        "vectorSemantics": dict(LUMIO_BIN_VECTOR_SEMANTICS),
        "goldens": goldens,
        "rejections": rejections,
    }


# --------------------------------------------------------------------------
# ADR-047 snapshot-header checksum domain (the B profile)
# --------------------------------------------------------------------------

SNAPSHOT_CHECKSUM_DOMAIN_TAG = "SnapshotHeaderV1"
SNAPSHOT_CHECKSUM_OMIT = ["checksum", "hash"]


def snapshot_checksum_input(header: Dict[str, Any]) -> Dict[str, Any]:
    """The B-profile digest input: the header minus its two digest members,
    wrapped in the same structural domain object ADR-041 section 2 uses."""
    body = {key: value for key, value in header.items() if key not in SNAPSHOT_CHECKSUM_OMIT}
    return {"digestDomain": SNAPSHOT_CHECKSUM_DOMAIN_TAG, "header": body}


def snapshot_checksum(header: Dict[str, Any]) -> str:
    return sha256_bytes(canonical_json(snapshot_checksum_input(header)).encode("ascii"))


def checksum_domain_md(root: Path) -> str:
    """Emit the B profile as a document with a worked Golden, not one line.

    The A profile (canonical/digest) publishes form parameters and Goldens; the
    B profile published a single sentence with no domain tag and no vector, so a
    downstream could not tell whether its digest agreed with anyone else's.
    """
    header = json.loads(
        (root / "fixtures" / "valid" / "snapshot-active.json").read_text(encoding="utf-8")
    )
    digest_input = snapshot_checksum_input(header)
    bytes_text = canonical_json(digest_input)
    return (
        "# Snapshot Header Checksum Domain ({tag}, the B profile)\n"
        "\n"
        "Generated with the CanonicalSerializer artifact. Do not hand-edit.\n"
        "Authority: ADR-047 section 4; form: `CanonicalJsonV1` (ADR-041).\n"
        "\n"
        "## The two digests are not the same digest\n"
        "\n"
        "- `hash` covers the **payload bytes** the header describes: `SHA-256(payload)`,\n"
        "  where the payload is the uncompressed domain bytes (ADR-047: encoded under\n"
        "  `LumioBinV1` when the payload is binary). It says nothing about the header.\n"
        "- `checksum` covers the **header** with the {omit} members removed:\n"
        "  `SHA-256(CanonicalJsonV1({{\"digestDomain\":\"{tag}\",\"header\":<header minus those two>}}))`.\n"
        "  Omitting both is what makes the value computable at all — `checksum` cannot\n"
        "  cover itself, and `hash` is omitted so a re-hash of the payload does not\n"
        "  force a header rewrite.\n"
        "\n"
        "Domain tag: `{tag}`. The tag is a member of the digest input, exactly as in\n"
        "ADR-041 section 2, so a B-profile digest can never collide with an A-profile one.\n"
        "\n"
        "## Golden\n"
        "\n"
        "Input (`fixtures/valid/snapshot-active.json`, the registered positive fixture):\n"
        "\n"
        "```json\n"
        "{bytes_text}\n"
        "```\n"
        "\n"
        "```text\n"
        "checksum = {digest}\n"
        "```\n"
        "\n"
        "The architecture gate recomputes this value from the fixture on every run, so\n"
        "the Golden cannot drift away from the rule it documents.\n"
    ).format(
        tag=SNAPSHOT_CHECKSUM_DOMAIN_TAG,
        omit=", ".join("`{}`".format(name) for name in SNAPSHOT_CHECKSUM_OMIT),
        bytes_text=bytes_text,
        digest=snapshot_checksum(header),
    )


def lumio_bin_rust(profile: Dict[str, Any]) -> str:
    """Publish the LumioBinV1 form parameters and Golden digests to Rust.

    Only the identifiers and digests are emitted; the layouts, values and bytes
    live in the published `binary/lumio-bin-profile.json`, which is the single
    place a conformance test reads them from.
    """
    form = profile["binaryForm"]
    out = [
        "/// ADR-047 LumioBinV1: the binary canonical form for public payload bytes.\n",
        "/// `CanonicalJsonV1` stays the form for canonicalizable JSON documents; this is\n",
        "/// the primitive layer ADR-010 referred to and ADR-035 assumed.\n",
        "pub const LUMIO_BIN_FORM_ID: &str = \"%s\";\n" % form["formId"],
        "pub const LUMIO_BIN_BYTE_ORDER: &str = \"%s\";\n" % form["byteOrder"],
        "pub const LUMIO_BIN_STRING_ENCODING: &str = \"%s\";\n" % form["strings"]["encoding"],
        "pub const LUMIO_BIN_STRING_LENGTH_PREFIX: &str = \"%s\";\n" % form["strings"]["lengthPrefix"],
        "pub const LUMIO_BIN_BYTES_LENGTH_PREFIX: &str = \"%s\";\n" % form["bytes"]["lengthPrefix"],
        "pub const LUMIO_BIN_ARRAY_COUNT_PREFIX: &str = \"%s\";\n" % form["arrays"]["countPrefix"],
        "pub const LUMIO_BIN_FIELD_ORDER: &str = \"%s\";\n" % form["structs"]["fieldOrder"],
        "pub const LUMIO_BIN_PADDING: &str = \"%s\";\n" % form["structs"]["padding"],
        "pub const LUMIO_BIN_FLOATS: &str = \"%s\";\n" % form["floats"],
        "pub const LUMIO_BIN_DIGEST_FRAMING: &str = \"%s\";\n\n"
        % profile["digestAlgorithm"]["framing"],
        "/// Integer widths, as `(kind, bytes, signed)`. Little-endian, no padding.\n",
        "pub const LUMIO_BIN_INTEGER_WIDTHS: &[(&str, u32, bool)] = &[\n",
    ]
    for name in ("u8", "u16", "u32", "u64", "i32", "i64"):
        spec = form["integers"][name]
        out.append(
            "    (\"%s\", %d, %s),\n" % (name, spec["bytes"], "true" if spec["signed"] else "false")
        )
    out.append("];\n\n")
    out.append("/// Golden vectors: `(id, case, sha256)`. Layouts, values and bytes are in\n")
    out.append("/// the published `binary/lumio-bin-profile.json`.\n")
    out.append("pub const LUMIO_BIN_GOLDENS: &[(&str, &str, &str)] = &[\n")
    for golden in profile["goldens"]:
        out.append("    (\"%s\", \"%s\", \"%s\"),\n" % (golden["id"], golden["case"], golden["sha256"]))
    out.append("];\n\n")
    out.append("/// Inputs a conforming encoder must refuse: `(id, case, error)`.\n")
    out.append("pub const LUMIO_BIN_REJECTIONS: &[(&str, &str, &str)] = &[\n")
    for rejection in profile["rejections"]:
        out.append(
            "    (\"%s\", \"%s\", \"%s\"),\n"
            % (rejection["id"], rejection["case"], rejection["error"])
        )
    out.append("];\n")
    return "".join(out)


def lumio_bin_csharp(profile: Dict[str, Any]) -> str:
    form = profile["binaryForm"]
    out = [
        "// ADR-047 LumioBinV1: the binary canonical form for public payload bytes.\n",
        "public static class LumioBinForm\n{\n",
        "    public const string FormId = \"%s\";\n" % form["formId"],
        "    public const string ByteOrder = \"%s\";\n" % form["byteOrder"],
        "    public const string StringEncoding = \"%s\";\n" % form["strings"]["encoding"],
        "    public const string StringLengthPrefix = \"%s\";\n" % form["strings"]["lengthPrefix"],
        "    public const string BytesLengthPrefix = \"%s\";\n" % form["bytes"]["lengthPrefix"],
        "    public const string ArrayCountPrefix = \"%s\";\n" % form["arrays"]["countPrefix"],
        "    public const string FieldOrder = \"%s\";\n" % form["structs"]["fieldOrder"],
        "    public const string Padding = \"%s\";\n" % form["structs"]["padding"],
        "    public const string Floats = \"%s\";\n" % form["floats"],
        "    public const string DigestFraming = \"%s\";\n}\n\n" % profile["digestAlgorithm"]["framing"],
        cs_value_struct("LumioBinIntegerWidth", [("string", "Kind"), ("uint", "Bytes"), ("bool", "Signed")]),
        "public static class LumioBinIntegerWidths\n{\n    public static readonly LumioBinIntegerWidth[] All =\n    {\n",
    ]
    for name in ("u8", "u16", "u32", "u64", "i32", "i64"):
        spec = form["integers"][name]
        out.append(
            "        new LumioBinIntegerWidth(\"%s\", %d, %s),\n"
            % (name, spec["bytes"], "true" if spec["signed"] else "false")
        )
    out.append("    };\n}\n\n")
    out.append(cs_value_struct("LumioBinGolden", [("string", "Id"), ("string", "Case"), ("string", "Sha256")]))
    out.append("public static class LumioBinGoldens\n{\n    public static readonly LumioBinGolden[] All =\n    {\n")
    for golden in profile["goldens"]:
        out.append(
            "        new LumioBinGolden(\"%s\", \"%s\", \"%s\"),\n"
            % (golden["id"], golden["case"], golden["sha256"])
        )
    out.append("    };\n}\n\n")
    out.append(cs_value_struct("LumioBinRejection", [("string", "Id"), ("string", "Case"), ("string", "Error")]))
    out.append("public static class LumioBinRejections\n{\n    public static readonly LumioBinRejection[] All =\n    {\n")
    for rejection in profile["rejections"]:
        out.append(
            "        new LumioBinRejection(\"%s\", \"%s\", \"%s\"),\n"
            % (rejection["id"], rejection["case"], rejection["error"])
        )
    out.append("    };\n}\n")
    return cs_namespace("Lumio.Gen.CanonicalSerializer", "".join(out), usings="using System;\n\n")


def emit_lumio_bin_profile(out_dir: Path) -> Dict[str, Any]:
    profile = derive_lumio_bin_profile()
    write_text(out_dir / LUMIO_BIN_PROFILE_FILE, canonical_json(profile) + "\n")
    return profile


def emit_validated_profile(root: Path, out_dir: Path, fixture: str, target: str) -> Dict[str, Any]:
    """Publish a gate-validated profile fixture into the consumable package tree."""
    profile = json.loads((root / "fixtures" / "valid" / fixture).read_text(encoding="utf-8"))
    write_text(out_dir / target, canonical_json(profile) + "\n")
    return profile


def emit_trust_profile(root: Path, out_dir: Path) -> Dict[str, Any]:
    """Publish the ADR-042 trust profile. The gate validates it; this only copies
    the validated record into the consumable package tree."""
    profile = json.loads(
        (root / "fixtures" / "valid" / "trust-profile.json").read_text(encoding="utf-8")
    )
    write_text(out_dir / TRUST_PROFILE_FILE, canonical_json(profile) + "\n")
    return profile


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


def emit_c_header(abi: Dict[str, Any], capabilities: List[Tuple[str, int, str]]) -> str:
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
        capability_c(capabilities),
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


def emit_rust_root_abi(abi: Dict[str, Any], capabilities: List[Tuple[str, int, str]]) -> str:
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
    out.append("};\n\n")
    out.append(capability_rust(capabilities))
    return "".join(out)


def emit_csharp_root_abi(abi: Dict[str, Any], capabilities: List[Tuple[str, int, str]]) -> str:
    profile = LAYOUT_PROFILE
    pointer = int(profile["pointerBytes"])
    out = [
        "// Generated Root ABI binding. Do not hand-edit.\n",
        "// Publisher: LumioGameEngineArchitecture / {}. ADR-040.\n".format(BASELINE),
        "// Pure managed layout description; the consumer binds the entry symbol itself.\n",

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
    out.append(cs_value_struct("SlotOffset", [("string", "Table"), ("string", "Slot"), ("int", "Offset")]))
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
    out.append("    };\n\n    public static readonly StructSize[] StructSizes =\n    {\n")
    out.append("        new StructSize(\"lumio_handle_t\", 16),\n        new StructSize(\"lumio_buffer_t\", 24),\n")
    for table in abi.get("apiTable", []):
        out.append("        new StructSize(\"{}\", {}),\n".format(table["name"], table["structSize"]))
    out.append("        new StructSize(\"lumio_root_api\", {}),\n".format(root_declared))
    out.append("    };\n}\n")
    header = out[:3]
    body = out[3:]
    return "".join(header) + cs_namespace(
        "Lumio.Gen.LanguageBinding",
        cs_value_struct("StructSize", [("string", "Name"), ("int", "Size")])
        + "\n"
        + "".join(body)
        + "\n"
        + capability_csharp(capabilities),
        usings="using System;\nusing System.Runtime.InteropServices;\n\n",
    )


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
    capabilities = load_capabilities(root)
    contents = {
        "abi/lumio_core.h": emit_c_header(abi, capabilities),
        "rust/lumio-gen-language-binding/src/root_abi.rs": emit_rust_root_abi(abi, capabilities),
        "csharp/Lumio.Gen.LanguageBinding/RootAbi.cs": emit_csharp_root_abi(abi, capabilities),
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
        load_message_ids(root),
    )
    emit_mapping(rust_root / RUST_CRATES["MappingTable"], cs_root / CS_PROJ["MappingTable"])
    canonical_profile = derive_canonical_profile(root)
    lumio_bin_profile = derive_lumio_bin_profile()
    emit_canonical(
        rust_root / RUST_CRATES["CanonicalSerializer"],
        cs_root / CS_PROJ["CanonicalSerializer"],
        canonical_profile,
        lumio_bin_profile,
        checksum_domain_md(root),
    )
    emit_language_binding(
        rust_root / RUST_CRATES["LanguageBinding"],
        cs_root / CS_PROJ["LanguageBinding"],
        schema_ids,
    )
    projector = TypeProjector(root)
    projector.project()
    emit_contract_types(
        rust_root / RUST_CRATES["ContractTypes"],
        cs_root / CS_PROJ["ContractTypes"],
        machines,
        schema_ids,
        errors,
        abi,
        projector,
    )
    emit_contract_runtime(
        rust_root / RUST_CRATES["ContractRuntime"],
        cs_root / CS_PROJ["ContractRuntime"],
    )
    emit_workspace(rust_root)
    write_text(out_dir / ".gitignore", "rust/target/\ncsharp/**/bin/\ncsharp/**/obj/\n")
    bundle = emit_root_abi(root, out_dir, comp)
    write_text(out_dir / CANONICAL_PROFILE_FILE, canonical_json(canonical_profile) + "\n")
    emit_lumio_bin_profile(out_dir)
    trust_profile = emit_trust_profile(root, out_dir)
    loader_profile = emit_validated_profile(root, out_dir, "loader-profile.json", LOADER_PROFILE_FILE)
    evidence_profile = emit_validated_profile(root, out_dir, "evidence-profile.json", EVIDENCE_PROFILE_FILE)

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
        "loader": {
            "profileId": loader_profile["profileId"],
            "profilePath": LOADER_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / LOADER_PROFILE_FILE),
            "errorPriorityLength": len(loader_profile["errorPriority"]),
            "consumers": list(LOADER_PROFILE_CONSUMERS),
        },
        "evidence": {
            "profileId": evidence_profile["profileId"],
            "profilePath": EVIDENCE_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / EVIDENCE_PROFILE_FILE),
            "profileCount": len(evidence_profile["profiles"]),
            "consumers": list(EVIDENCE_PROFILE_CONSUMERS),
        },
        "trust": {
            "profileId": trust_profile["profileId"],
            "profilePath": TRUST_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / TRUST_PROFILE_FILE),
            "signatureProfileId": trust_profile["signatureProfile"]["profileId"],
            "trustDomain": trust_profile["trustPolicy"]["trustDomain"],
            "vectorCount": len(trust_profile["vectors"]),
            "consumers": list(TRUST_PROFILE_CONSUMERS),
        },
        "canonicalDigest": {
            "profileId": canonical_profile["profileId"],
            "profilePath": CANONICAL_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / CANONICAL_PROFILE_FILE),
            "formId": canonical_profile["canonicalForm"]["formId"],
            "digestAlgorithm": dict(canonical_profile["digestAlgorithm"]),
            "goldenCount": len(canonical_profile["goldens"]),
            "consumers": list(CANONICAL_PROFILE_CONSUMERS),
        },
        "binary": {
            "profileId": lumio_bin_profile["profileId"],
            "profilePath": LUMIO_BIN_PROFILE_FILE,
            "profileDigest": sha256_file(out_dir / LUMIO_BIN_PROFILE_FILE),
            "formId": lumio_bin_profile["binaryForm"]["formId"],
            "digestAlgorithm": dict(lumio_bin_profile["digestAlgorithm"]),
            "goldenCount": len(lumio_bin_profile["goldens"]),
            "rejectionCount": len(lumio_bin_profile["rejections"]),
            "consumers": list(LUMIO_BIN_PROFILE_CONSUMERS),
        },
        "rootAbi": {
            "bundleId": bundle["bundleId"],
            "bundlePath": ABI_BUNDLE_FILE,
            "bundleDigest": sha256_file(out_dir / ABI_BUNDLE_FILE),
            "compiler": dict(bundle["compiler"]),
            "inputHash": bundle["inputHash"],
            "layoutProfileId": bundle["layoutProfile"]["targetProfileId"],
            "outputFiles": [dict(item) for item in bundle["outputFiles"]],
            "consumers": list(ROOT_ABI_CONSUMERS),
        },
    }
    write_text(out_dir / "index.json", canonical_json(index) + "\n")
    write_text(
        out_dir / "README.md",
        "Generated V1.4 contract artifacts. Do not hand-edit package sources.\n"
        "Regenerate with `python tools/lumio_contract.py generate --out packages`.\n"
        "`abi/` is the ADR-040 Root ABI bundle: `lumio_core.h`, the layout Golden\n"
        "record `root-abi-bundle.json`, and the digests of the Rust and C# bindings.\n"
        "`binary/` is the ADR-047 LumioBinV1 profile: the primitive byte layout for\n"
        "public payload bytes, with self-verifying Golden and rejection vectors.\n"
        "Per ADR-048 the C# projects multi-target netstandard2.1 and net8.0, the\n"
        "ContractTypes artifact carries generated type bodies for the eight closed\n"
        "contracts in schema declaration order, and the ProtocolPermissionValidator\n"
        "carries the executable ADR-022 gate rather than a list of field names.\n",
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
