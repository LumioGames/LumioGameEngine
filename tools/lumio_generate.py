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


def emit_canonical(rust_dir: Path, cs_dir: Path) -> None:
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
    write_text(rust_dir / "CHECKSUM_DOMAIN.md", "checksum = SHA-256(canonical_json(header without checksum,hash))\n")
    write_text(
        cs_dir / "CanonicalSerializer.cs",
        "namespace Lumio.Gen.CanonicalSerializer;\n\npublic static class SnapshotChecksum\n{\n"
        "    public const string Domain = \"SHA-256 over canonical JSON of snapshot-header minus checksum and hash fields\";\n"
        "    public const string Magic = \"LUMIOSNP1\";\n}\n",
    )
    write_text(cs_dir / (CS_PROJ["CanonicalSerializer"] + ".csproj"), csproj(CS_PROJ["CanonicalSerializer"]))


def emit_language_binding(rust_dir: Path, cs_dir: Path, schema_ids: List[str]) -> None:
    rust = rust_lib_header("LanguageBinding")
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
    cs.append("    public static readonly string[] VoxelWorldRoles = { \"Authority\", \"Replica\" };\n}\n\n")
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
        "pub struct BoundedBuffer { inner: Vec<u8>, cap: usize }\n"
        "impl BoundedBuffer {\n"
        "    pub fn new(cap: usize) -> Self { Self { inner: Vec::new(), cap } }\n"
        "    pub fn push(&mut self, byte: u8) -> Result<(), ()> {\n"
        "        if self.inner.len() >= self.cap { return Err(()); }\n"
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
    rust_root = out_dir / "rust"
    cs_root = out_dir / "csharp"
    desc_dir = out_dir / "descriptors"

    emit_protocol_permission(
        rust_root / RUST_CRATES["ProtocolPermissionValidator"],
        cs_root / CS_PROJ["ProtocolPermissionValidator"],
    )
    emit_mapping(rust_root / RUST_CRATES["MappingTable"], cs_root / CS_PROJ["MappingTable"])
    emit_canonical(
        rust_root / RUST_CRATES["CanonicalSerializer"],
        cs_root / CS_PROJ["CanonicalSerializer"],
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
    )
    emit_contract_runtime(
        rust_root / RUST_CRATES["ContractRuntime"],
        cs_root / CS_PROJ["ContractRuntime"],
    )
    emit_workspace(rust_root)
    write_text(out_dir / ".gitignore", "rust/target/\ncsharp/**/bin/\ncsharp/**/obj/\n")

    comp = compiler_hash(root)
    inh = input_hash(root)
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
    }
    write_text(out_dir / "index.json", canonical_json(index) + "\n")
    write_text(
        out_dir / "README.md",
        "Generated V1.4 contract artifacts. Do not hand-edit package sources.\n"
        "Regenerate with `python tools/lumio_contract.py generate --out packages`.\n",
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
