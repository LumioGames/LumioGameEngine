Generated V1.4 contract artifacts. Do not hand-edit package sources.
Regenerate with `python tools/lumio_contract.py generate --out packages`.
`abi/` is the ADR-040 Root ABI bundle: `lumio_core.h`, the layout Golden
record `root-abi-bundle.json`, and the digests of the Rust and C# bindings.
`binary/` is the ADR-047 LumioBinV1 profile: the primitive byte layout for
public payload bytes, with self-verifying Golden and rejection vectors.
Per ADR-048 the C# projects multi-target netstandard2.1 and net8.0, the
ContractTypes artifact carries generated type bodies for the eight closed
contracts in schema declaration order, and the ProtocolPermissionValidator
carries the executable ADR-022 gate rather than a list of field names.
