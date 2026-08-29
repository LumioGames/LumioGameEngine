#!/usr/bin/env python3
"""Cross-language known-answer check for the generated SHA-256 implementations.

The Rust and C# ContractRuntime artifacts are two generated implementations of
one hash-chain contract, and nothing used to compare them. A wrong round
constant shipped in the Rust K table produced a wrong digest for every input
and survived to release, because the generated Rust tests only asserted that
the hasher agreed with itself.

This driver closes that gap from the outside: it hashes the FIPS 180-4 vectors
in ``lumio_generate.KAT_VECTORS`` through all three implementations that the
project depends on --- the generated Rust crate, the generated C# project, and
Python's ``hashlib`` --- and requires every leg to agree with the frozen
expected digest. Any disagreement exits non-zero.

Run it from the repository root::

    python3 tools/lumio_kat.py

``--skip-csharp`` / ``--skip-rust`` exist for local iteration only; CI runs the
full three-way comparison and must not pass either flag.
"""

import argparse
import hashlib
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from lumio_generate import KAT_VECTORS  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
RUST_CRATE = ROOT / "packages" / "rust" / "lumio-gen-contract-runtime"
CS_PROJECT = ROOT / "packages" / "csharp" / "Lumio.Gen.ContractRuntime"


def _ascii(data: bytes) -> str:
    """Vectors are ASCII so they can be embedded in Rust and C# source."""
    return data.decode("ascii")


def hashlib_digests() -> list:
    return [hashlib.sha256(data).hexdigest() for data, _ in KAT_VECTORS]


def rust_digests(workdir: Path) -> list:
    """Build a throwaway binary against the generated crate and read its output.

    A path dependency keeps this offline; ``[workspace]`` detaches the temp
    project from the generated packages/rust workspace.
    """
    proj = workdir / "rust-kat"
    (proj / "src").mkdir(parents=True)
    (proj / "Cargo.toml").write_text(
        "[workspace]\n"
        "[package]\n"
        'name = "lumio-kat-driver"\n'
        'version = "0.0.0"\n'
        'edition = "2021"\n'
        "publish = false\n\n"
        "[dependencies]\n"
        'lumio-gen-contract-runtime = {{ path = "{}" }}\n'.format(RUST_CRATE.as_posix()),
        encoding="utf-8",
    )
    body = "\n".join(
        '    println!("{{}}", sha256_hex(b"{}"));'.format(_ascii(data))
        for data, _ in KAT_VECTORS
    )
    (proj / "src" / "main.rs").write_text(
        "use lumio_gen_contract_runtime::sha256_hex;\n\nfn main() {\n" + body + "\n}\n",
        encoding="utf-8",
    )
    out = subprocess.run(
        ["cargo", "run", "--quiet", "--manifest-path", str(proj / "Cargo.toml")],
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        raise SystemExit("rust driver failed:\n{}".format(out.stderr))
    return out.stdout.split()


def csharp_digests(workdir: Path) -> list:
    """Same, through the generated C# runtime's HashChain.Sha256.

    The generated source is pulled in with ``Compile Include`` rather than a
    ProjectReference: a ProjectReference makes MSBuild write obj/ and bin/ into
    packages/csharp, which pollutes a published artifact directory and trips the
    repository-policy grep for PackageReference. Nothing is written outside the
    temp directory this way, and the file under test is still the generated one.
    """
    proj = workdir / "cs-kat"
    proj.mkdir(parents=True)
    csproj = "\n".join(
        [
            '<Project Sdk="Microsoft.NET.Sdk">',
            "  <PropertyGroup>",
            "    <OutputType>Exe</OutputType>",
            "    <TargetFramework>net8.0</TargetFramework>",
            "    <ImplicitUsings>disable</ImplicitUsings>",
            "    <Nullable>enable</Nullable>",
            # The generated project targets net8.0; roll the throwaway driver
            # forward so it runs on whatever shared runtime the host has.
            "    <RollForward>LatestMajor</RollForward>",
            "  </PropertyGroup>",
            "  <ItemGroup>",
            '    <Compile Include="{}" Link="ContractRuntime.cs" />'.format(
                (CS_PROJECT / "ContractRuntime.cs").as_posix()
            ),
            "  </ItemGroup>",
            "</Project>",
            "",
        ]
    )
    (proj / "kat.csproj").write_text(csproj, encoding="utf-8")
    literals = ", ".join('"{}"'.format(_ascii(data)) for data, _ in KAT_VECTORS)
    program = "\n".join(
        [
            "using System;",
            "using System.Text;",
            "using Lumio.Gen.ContractRuntime;",
            "",
            "public static class Kat",
            "{",
            "    public static void Main()",
            "    {",
            "        string[] vectors = { " + literals + " };",
            "        foreach (var v in vectors)",
            "        {",
            "            var digest = HashChain.Sha256(Encoding.ASCII.GetBytes(v));",
            "            Console.WriteLine(Convert.ToHexString(digest).ToLowerInvariant());",
            "        }",
            "    }",
            "}",
            "",
        ]
    )
    (proj / "Program.cs").write_text(program, encoding="utf-8")
    out = subprocess.run(
        ["dotnet", "run", "--project", str(proj / "kat.csproj"), "--verbosity", "quiet"],
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        raise SystemExit("c# driver failed:\n{}\n{}".format(out.stdout, out.stderr))
    return [line for line in out.stdout.split() if len(line) == 64]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-rust", action="store_true", help="local iteration only")
    parser.add_argument("--skip-csharp", action="store_true", help="local iteration only")
    args = parser.parse_args()

    expected = [digest for _, digest in KAT_VECTORS]
    legs = {"hashlib": hashlib_digests()}

    with tempfile.TemporaryDirectory() as tmp:
        workdir = Path(tmp)
        if not args.skip_rust:
            if shutil.which("cargo") is None:
                raise SystemExit("cargo not found; the Rust leg is required")
            legs["rust"] = rust_digests(workdir)
        if not args.skip_csharp:
            if shutil.which("dotnet") is None:
                raise SystemExit("dotnet not found; the C# leg is required")
            legs["csharp"] = csharp_digests(workdir)

    failures = []
    for index, (data, want) in enumerate(zip([d for d, _ in KAT_VECTORS], expected)):
        for name, got in legs.items():
            if index >= len(got):
                failures.append("{} produced no digest for vector {}".format(name, index))
            elif got[index] != want:
                failures.append(
                    "{} vector {} (len {}): got {} want {}".format(
                        name, index, len(data), got[index], want
                    )
                )

    for name, got in sorted(legs.items()):
        status = "OK" if all(g == w for g, w in zip(got, expected)) and len(got) == len(expected) else "MISMATCH"
        print("{:<8} {} ({} vectors)".format(name, status, len(got)))

    if failures:
        for line in failures:
            print("FAIL {}".format(line), file=sys.stderr)
        return 1
    print(
        "{} agree on {} FIPS 180-4 vectors".format(
            " + ".join(sorted(legs)), len(expected)
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
