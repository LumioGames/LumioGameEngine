#!/usr/bin/env python3
"""Validate Lumio architecture schemas and their positive/negative fixtures.

The runner uses the mature ``jsonschema`` package when it is available.  A
small deterministic Draft-2020-12 subset is kept as a bootstrap fallback so
the architecture repository can be checked before the implementation build
environment has installed its dependencies.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple
from urllib.parse import unquote, urlparse


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "schemas"
FIXTURE_DIR = ROOT / "fixtures"
ID_REGISTRY_FILE = ROOT / "ids" / "index.json"
SCHEMA_INDEX = SCHEMA_DIR / "index.json"
FIXTURE_INDEX = FIXTURE_DIR / "index.json"
PACKAGE_DIR = ROOT / "packages"

# ADR-040 Root ABI bundle freezes: the generator in ``lumio_generate`` is the single
# authority for these values; the gate imports them so a drift is impossible.
if str(Path(__file__).resolve().parent) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
import lumio_generate as _abi  # noqa: E402

_ABI_LAYOUT_PROFILE = dict(_abi.LAYOUT_PROFILE)
_ABI_ROOT_HEADER_BYTES = int(_abi.LAYOUT_PROFILE["rootHeaderBytes"])
_ABI_TABLE_HEADER_BYTES = int(_abi.LAYOUT_PROFILE["tableHeaderBytes"])
_ABI_TYPE_MAPPING_KEYS = set(_abi.ABI_TYPE_MAPPING_KEYS)
_ABI_OUTPUT_PATHS = [path for path, _role in _abi.ABI_OUTPUT_FILES]
_ABI_OUTPUT_ROLES = [role for _path, role in _abi.ABI_OUTPUT_FILES]
_ABI_BUNDLE_FILE = _abi.ABI_BUNDLE_FILE

_CANONICAL_FORM = dict(_abi.CANONICAL_FORM)
_CANONICAL_DIGEST_ALGORITHM = dict(_abi.CANONICAL_DIGEST_ALGORITHM)
_CANONICAL_DIGEST_KEYS = [item["digest"] for item in _abi.CANONICAL_DIGEST_DOMAINS]
_CANONICAL_DOMAIN_TAGS = {item["digest"]: item["domainTag"] for item in _abi.CANONICAL_DIGEST_DOMAINS}
_CANONICAL_NORMALIZATION = {
    item["digest"]: item.get("normalization") or [] for item in _abi.CANONICAL_DIGEST_DOMAINS
}
_CANONICAL_GOLDEN_CASES = set(_abi.CANONICAL_GOLDEN_CASES)
_CANONICAL_PROFILE_FILE = _abi.CANONICAL_PROFILE_FILE

_LOADER_REENTRY = {
    "afterFailedRolledBack": "NewInstanceFromUninitialized",
    "afterReleased": "NewInstanceFromUninitialized",
    "latchScope": "Process",
    "latchClearedOnRelease": False,
    "sameIdentityAcquire": "ReturnExistingLease",
    "differentIdentityAcquire": "PackageIdentityConflict",
}
# ADR-043 section 3: lower rank wins. PartialLoadRolledBack is a floor, not a winner.
_LOADER_ERROR_PRIORITY = [
    "PackageIdentityConflict",
    "NativeAbiMismatch",
    "SymbolMissing",
    "SymbolCollision",
    "CapabilityMissing",
    "TargetProfileMismatch",
    "LoaderOutOfMemory",
    "LoaderTimeout",
    "LoaderCancelled",
    "PartialLoadRolledBack",
]
_LOADER_ACQUIRE_CASES = {
    "FirstAcquire",
    "ConcurrentSameIdentity",
    "ConcurrentDifferentIdentity",
    "AfterReleasedSameIdentity",
    "AfterReleasedDifferentIdentity",
    "RetryAfterFailedRolledBack",
}
# ADR-044 section 1: the closed format set and what each spelling implies.
_EVIDENCE_PROFILES = {
    "sbom": ("Sbom", "CycloneDX", "1.6", "application/vnd.cyclonedx+json"),
    "license": ("License", "SPDX", "2.3", "application/spdx+json"),
    "provenance": ("Provenance", "SLSA-v1", "1.0", "application/vnd.in-toto+json"),
}
_EVIDENCE_FORMATS = {item[1] for item in _EVIDENCE_PROFILES.values()}
_EVIDENCE_VECTOR_CASES = {
    "Valid",
    "MissingKind",
    "DigestMismatch",
    "UnknownFormat",
    "IndexEntryWithoutManifestReference",
    "ManifestReferenceWithoutIndexEntry",
}


def evaluate_acquire(vector: Dict[str, Any]) -> Tuple[str, Optional[str]]:
    """ADR-043 section 2: the latch is by identity, not by time."""
    latched = vector.get("latchedIdentity")
    requested = vector.get("requestedIdentity")
    if latched is None:
        return "NewLease", None
    if latched == requested:
        return "ExistingLease", None
    return "Rejected", "PackageIdentityConflict"


def evaluate_loader_failure(causes: List[str]) -> Optional[str]:
    """ADR-043 section 3: the reported code is the highest-ranked cause."""
    ranked = [c for c in causes if c in _LOADER_ERROR_PRIORITY]
    if not ranked:
        return None
    return min(ranked, key=_LOADER_ERROR_PRIORITY.index)


def evaluate_evidence(vector: Dict[str, Any]) -> Tuple[str, Optional[str]]:
    """ADR-044 sections 3-4: DigestOnly plus exact bidirectional coverage."""
    evidence_set = vector.get("evidenceSet") or {}
    entries = vector.get("indexEntries") or []
    by_kind: Dict[str, List[Dict[str, Any]]] = {}
    for entry in entries:
        by_kind.setdefault(str(entry.get("kind")), []).append(entry)
    for kind, (artifact_kind, fmt, _version, _media) in _EVIDENCE_PROFILES.items():
        ref = evidence_set.get(kind)
        if ref is None:
            return "Rejected", "EvidenceMissing"
        if ref.get("format") != fmt:
            return "Rejected", "EvidenceMissing"
        matches = by_kind.get(artifact_kind) or []
        if len(matches) != 1:
            return "Rejected", "EvidenceMissing"
        if matches[0].get("sha256") != ref.get("digest"):
            return "Rejected", "EvidenceDigestMismatch"
    extra = set(by_kind) - {item[0] for item in _EVIDENCE_PROFILES.values()}
    if extra:
        return "Rejected", "EvidenceMissing"
    return "Accepted", None


_TRUST_PREIMAGE_FIELDS = ["domainSeparator", "trustDomain", "payloadType", "payloadDigest"]
_TRUST_REJECT_PRIORITY = [
    (1, "SignatureMissing"),
    (2, "TrustRootUnknown"),
    (3, "KeyRevoked"),
    (4, "SignatureInvalid"),
    (5, "TrustPolicyRejected"),
]
_TRUST_VECTOR_CASES = {
    "Accept",
    "TamperedSignature",
    "TamperedPayloadDigest",
    "WrongTrustDomain",
    "UnknownKey",
    "RevokedKey",
    "BeforeNotBefore",
    "AfterNotAfter",
}


class ContractError(Exception):
    """Raised for malformed registry data or an unreadable contract file."""


def load_json(path: Path) -> Any:
    try:
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError) as exc:
        raise ContractError("cannot read JSON {}: {}".format(path, exc)) from exc


def canonical_json(value: Any) -> str:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


class SchemaResolver:
    """Resolve local and sibling-file references used by the contract set."""

    def __init__(self) -> None:
        self.cache: Dict[Path, Any] = {}

    def load(self, path: Path) -> Any:
        path = path.resolve()
        if path not in self.cache:
            self.cache[path] = load_json(path)
        return self.cache[path]

    @staticmethod
    def pointer(document: Any, fragment: str) -> Any:
        if not fragment or fragment == "#":
            return document
        if fragment.startswith("#"):
            fragment = fragment[1:]
        if not fragment.startswith("/"):
            raise ContractError("unsupported JSON pointer fragment #{}".format(fragment))
        current = document
        for token in fragment[1:].split("/"):
            token = unquote(token.replace("~1", "/").replace("~0", "~"))
            if isinstance(current, list):
                current = current[int(token)]
            else:
                current = current[token]
        return current

    def resolve(self, reference: str, current_file: Path, current_schema: Any) -> Tuple[Any, Path]:
        if reference.startswith("#"):
            return self.pointer(current_schema, reference), current_file

        parsed = urlparse(reference)
        fragment = "#" + parsed.fragment if parsed.fragment else ""
        if parsed.scheme in ("http", "https"):
            # The registry is intentionally offline.  References to the
            # published schema URL resolve by filename within this repository.
            filename = Path(parsed.path).name
            target = SCHEMA_DIR / filename
        elif parsed.scheme == "file":
            target = Path(unquote(parsed.path))
        else:
            target = (current_file.parent / parsed.path).resolve()
        document = self.load(target)
        return self.pointer(document, fragment), target


def _type_matches(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    return True


def _path(path: str, token: Any) -> str:
    if isinstance(token, int):
        return "{}[{}]".format(path, token)
    return "{}.{}".format(path, token)


def fallback_validate(
    value: Any,
    schema: Any,
    resolver: SchemaResolver,
    current_file: Path,
    current_schema: Any,
    path: str = "$",
) -> List[str]:
    """Validate the keywords used by this repository's Draft 2020-12 files."""

    if not isinstance(schema, dict):
        return ["{}: schema must be an object".format(path)]

    errors: List[str] = []
    if "$ref" in schema:
        try:
            target, target_file = resolver.resolve(str(schema["$ref"]), current_file, current_schema)
            # Local references inside the resolved target must resolve against
            # the target *document*, not against the resolved fragment.
            target_root = resolver.load(target_file) if target_file.is_file() else current_schema
            errors.extend(fallback_validate(value, target, resolver, target_file, target_root, path))
        except (ContractError, KeyError, IndexError, ValueError) as exc:
            errors.append("{}: unresolved $ref {} ({})".format(path, schema["$ref"], exc))

    if "const" in schema and value != schema["const"]:
        errors.append("{}: expected const {!r}".format(path, schema["const"]))
    if "enum" in schema and value not in schema["enum"]:
        errors.append("{}: value {!r} is not in enum".format(path, value))

    expected_type = schema.get("type")
    if expected_type is not None:
        expected_types = expected_type if isinstance(expected_type, list) else [expected_type]
        if not any(_type_matches(value, str(item)) for item in expected_types):
            errors.append("{}: expected type {}, got {}".format(path, expected_types, type(value).__name__))
            return errors

    if "allOf" in schema:
        for item in schema["allOf"]:
            errors.extend(fallback_validate(value, item, resolver, current_file, current_schema, path))
    if "anyOf" in schema:
        alternatives = [fallback_validate(value, item, resolver, current_file, current_schema, path) for item in schema["anyOf"]]
        if all(alternatives):
            errors.append("{}: no anyOf alternative matched".format(path))
    if "oneOf" in schema:
        alternatives = [not fallback_validate(value, item, resolver, current_file, current_schema, path) for item in schema["oneOf"]]
        if sum(1 for matched in alternatives if matched) != 1:
            errors.append("{}: oneOf matched {} alternatives".format(path, sum(1 for matched in alternatives if matched)))
    if "not" in schema and not fallback_validate(value, schema["not"], resolver, current_file, current_schema, path):
        errors.append("{}: not constraint matched".format(path))

    if isinstance(value, str):
        if "minLength" in schema and len(value) < int(schema["minLength"]):
            errors.append("{}: shorter than minLength".format(path))
        if "maxLength" in schema and len(value) > int(schema["maxLength"]):
            errors.append("{}: longer than maxLength".format(path))
        if "pattern" in schema:
            try:
                if re.search(str(schema["pattern"]), value) is None:
                    errors.append("{}: does not match pattern".format(path))
            except re.error as exc:
                errors.append("{}: invalid schema pattern: {}".format(path, exc))

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            errors.append("{}: below minimum".format(path))
        if "maximum" in schema and value > schema["maximum"]:
            errors.append("{}: above maximum".format(path))

    if isinstance(value, list):
        if "minItems" in schema and len(value) < int(schema["minItems"]):
            errors.append("{}: fewer than minItems".format(path))
        if "maxItems" in schema and len(value) > int(schema["maxItems"]):
            errors.append("{}: more than maxItems".format(path))
        if schema.get("uniqueItems"):
            seen = set()
            for index, item in enumerate(value):
                marker = canonical_json(item)
                if marker in seen:
                    errors.append("{}: duplicate item at index {}".format(path, index))
                seen.add(marker)
        item_schema = schema.get("items")
        if item_schema is not None:
            for index, item in enumerate(value):
                errors.extend(fallback_validate(item, item_schema, resolver, current_file, current_schema, _path(path, index)))

    if isinstance(value, dict):
        required = schema.get("required", [])
        for key in required:
            if key not in value:
                errors.append("{}: missing required property {!r}".format(path, key))
        properties = schema.get("properties", {})
        pattern_properties = schema.get("patternProperties", {})
        additional = schema.get("additionalProperties", True)
        for key, item in value.items():
            matched = key in properties
            if matched:
                errors.extend(fallback_validate(item, properties[key], resolver, current_file, current_schema, _path(path, key)))
            for key_pattern, key_schema in pattern_properties.items():
                try:
                    if re.search(str(key_pattern), key) is not None:
                        matched = True
                        errors.extend(fallback_validate(item, key_schema, resolver, current_file, current_schema, _path(path, key)))
                except re.error as exc:
                    errors.append("{}: invalid schema key pattern: {}".format(path, exc))
            if matched:
                continue
            if additional is False:
                errors.append("{}: unexpected property {!r}".format(path, key))
            elif isinstance(additional, dict):
                errors.extend(fallback_validate(item, additional, resolver, current_file, current_schema, _path(path, key)))

    return errors


def structural_errors(value: Any, schema: Any, schema_file: Path, resolver: SchemaResolver) -> List[str]:
    """Use upstream jsonschema if installed, otherwise the bootstrap validator."""
    try:
        from jsonschema import Draft202012Validator  # type: ignore
        try:
            # jsonschema >= 4.18 uses the standalone referencing package and
            # avoids the deprecated RefResolver API.
            from referencing import Registry, Resource  # type: ignore

            resources = []
            for candidate in SCHEMA_DIR.glob("*.schema.json"):
                document = resolver.load(candidate)
                resource = Resource.from_contents(document)
                resources.append((candidate.as_uri(), resource))
                if isinstance(document, dict) and document.get("$id"):
                    resources.append((str(document["$id"]), resource))
            registry = Registry().with_resources(resources)
            validator = Draft202012Validator(schema, registry=registry)
        except ImportError:
            # Older supported jsonschema versions still expose RefResolver.
            from jsonschema import RefResolver  # type: ignore

            store: Dict[str, Any] = {}
            for candidate in SCHEMA_DIR.glob("*.schema.json"):
                document = resolver.load(candidate)
                store[candidate.as_uri()] = document
                if isinstance(document, dict) and document.get("$id"):
                    store[str(document["$id"])] = document
            validator = Draft202012Validator(schema, resolver=RefResolver(schema_file.as_uri(), schema, store=store))
        return ["{}: {}".format("$" if error.absolute_path == () else ".".join(str(p) for p in error.absolute_path), error.message) for error in validator.iter_errors(value)]
    except ImportError:
        return fallback_validate(value, schema, resolver, schema_file, schema)


_SCOPE_REQUIRED_IDS = {
    "Process": (),
    "Release": ("releasePoolId",),
    "Session": ("sessionId",),
    "World": ("worldId", "tickId"),
    "Txn": ("txnId", "worldId", "tickId"),
}

_SCOPE_FORBIDDEN_IDS = {
    "Process": ("sessionId", "worldId", "tickId", "txnId"),
    "Release": ("sessionId", "worldId", "tickId", "txnId"),
    "Session": ("worldId", "tickId", "txnId"),
    "World": ("txnId",),
    "Txn": (),
}


def correlation_scope_errors(correlation: Any) -> List[str]:
    """Enforce ADR-011 scope tiers: IDs may not be fabricated below their scope."""
    errors: List[str] = []
    if not isinstance(correlation, dict):
        return errors
    scope = correlation.get("scope")
    if scope not in _SCOPE_REQUIRED_IDS:
        return errors
    for field in _SCOPE_REQUIRED_IDS[scope]:
        if field not in correlation:
            errors.append("{} scope requires correlation field {!r}".format(scope, field))
    for field in _SCOPE_FORBIDDEN_IDS[scope]:
        if field in correlation:
            errors.append("{} scope must not fabricate correlation field {!r}".format(scope, field))
    return errors


_CURRENT_BASELINE = "LGE-V1.4-2026-08-27"
# ADR-040 section 3 freezes lumio_status_t as int32 carrying the ErrorCode
# numeric, with 0 reserved for success.  ADR-046 makes the resulting range a
# gate: a registered ErrorCode above int32 max cannot cross the Root ABI.
_STATUS_NUMERIC_MAX = 2147483647
_GENESIS_HASH = "0" * 64
_TICK_PHASES = [
    "IngressCapture",
    "DecodeAndCanonicalize",
    "ApplyInputs",
    "ProcessorPlan",
    "CrossWorldPrepare",
    "NativeJobBarrier",
    "CommitDecision",
    "VoxelCommit",
    "EcsCommandBufferCommit",
    "GasAndEventFinalize",
    "ReplicationProjection",
    "SnapshotHashMetrics",
    "EgressPublish",
]
_BUSINESS_PROCESSOR_PHASES = {
    "ApplyInputs",
    "ProcessorPlan",
    "CrossWorldPrepare",
    "CommitDecision",
    "GasAndEventFinalize",
}
_BUSINESS_ABORT_REASONS = {
    "RevisionConflict",
    "PermissionDenied",
    "InsufficientResource",
    "ValidationFailed",
}
_REPLICATION_MESSAGE_TYPES = [
    "Handshake",
    "FullSnapshot",
    "BaselineAck",
    "Delta",
    "DeltaAck",
    "ResyncRequest",
    "MaintenanceKick",
    "Error",
]
_REPLICATION_BODY_REQUIRED = {
    "Handshake": ("role",),
    "FullSnapshot": ("snapshotId", "tickId", "sessionRevisionVector", "schemaEpoch", "mappingSetHash"),
    "BaselineAck": ("snapshotId", "confirmedRevision"),
    "Delta": ("baseSnapshotId", "fromRevision", "toRevision", "mappingSetHash", "confirmationSequence", "tombstones"),
    "DeltaAck": ("confirmationSequence", "toRevision"),
    "ResyncRequest": ("resyncReason",),
    "MaintenanceKick": ("reasonCode",),
    "Error": ("errorClass", "reasonCode"),
}
_SESSION_REVISION_FIELDS = (
    "tickId",
    "gameRevision",
    "voxelWorldRevision",
    "chunkRevisionSet",
    "replicationRevision",
    "configRevision",
    "schemaEpoch",
)


def _transition_table(path: Path) -> Dict[str, set]:
    """Derive a from→{to} map from an ADR-038 descriptor fixture (single source)."""
    document = load_json(path)
    table: Dict[str, set] = {}
    for item in document.get("transitions") or []:
        table.setdefault(str(item.get("from")), set()).add(item.get("to"))
    return table


def _terminal_states(path: Path) -> set:
    return set(load_json(path).get("terminalStates") or [])


_ABILITY_FIXTURE = FIXTURE_DIR / "valid" / "state-machine-gas-ability.json"
_EFFECT_FIXTURE = FIXTURE_DIR / "valid" / "state-machine-gas-effect.json"
_ABILITY_TRANSITIONS = _transition_table(_ABILITY_FIXTURE)
_ABILITY_TERMINAL = _terminal_states(_ABILITY_FIXTURE)
_EFFECT_TRANSITIONS = _transition_table(_EFFECT_FIXTURE)
_EFFECT_TERMINAL = _terminal_states(_EFFECT_FIXTURE)
_CONFIG_INT_BOUNDS = {
    "i32": (-2147483648, 2147483647),
    "i64": (-9223372036854775808, 9223372036854775807),
    "u32": (0, 4294967295),
    "u64": (0, 18446744073709551615),
}
_CHUNK_KEY = re.compile(r"^c:(0|-?[1-9][0-9]{0,9}):(0|-?[1-9][0-9]{0,9}):(0|-?[1-9][0-9]{0,9})$")
_INTEGRITY_VALUE_RULES = {
    "None": re.compile(r"^none$"),
    "CRC32C": re.compile(r"^[0-9a-f]{8}$"),
    "SHA256": re.compile(r"^[0-9a-f]{64}$"),
    "AEAD": re.compile(r"^[A-Za-z0-9+/=_-]{24,256}$"),
}
# ADR-037 freezes the abort reason vocabulary split between the transaction
# and the voxel participant: the intersection and both domain-only remainders
# may only change through a new ADR.
_SHARED_ABORT_REASONS = {
    "RevisionConflict",
    "ChunkUnloaded",
    "ValidationFailed",
    "DeadlineExceeded",
    "Cancelled",
    "InsufficientResource",
}
_TXN_ONLY_ABORT_REASONS = {"PermissionDenied"}
_VOXEL_ONLY_ABORT_REASONS = {"LeaseExpired"}
# ADR-038 freezes the descriptor registry: every machine listed here must have
# exactly one valid descriptor fixture. Where a domain schema also carries the
# state enum, the pointer names the schema and the path to that enum so the
# descriptor and the schema cannot drift apart.
_STATE_MACHINE_SOURCES: Dict[str, Any] = {
    "WorldSlotHost": None,
    "SimulationSession": None,
    "ClientReplicaSession": None,
    "EcsCommandBuffer": ("cross-world-txn", ("properties", "commandBufferState", "enum")),
    "CrossWorldTxn": ("cross-world-txn", ("properties", "state", "enum")),
    "CoreEngineLoader": ("failure-bundle", ("properties", "coreEngine", "properties", "loaderState", "enum")),
    "GasAbility": None,
    "GasEffect": None,
    "ReleasePool": ("release-catalog", ("properties", "entries", "items", "properties", "state", "enum")),
    "GameplayScopeActivation": ("gameplay-scope-activation", ("properties", "stage", "enum")),
    "VoxelSnapshotCapture": ("voxel-snapshot-payload", ("$defs", "voxelCaptureState", "enum")),
    "VoxelChunkResidency": ("voxel-chunk-page", ("$defs", "voxelChunkState", "enum")),
}


def chunk_revision_set_errors(candidate: Any, context: str) -> List[str]:
    errors: List[str] = []
    if isinstance(candidate, dict):
        for key in candidate:
            if _CHUNK_KEY.match(str(key)) is None:
                errors.append("{} chunk key {!r} must use the canonical ChunkId format".format(context, key))
    return errors


def recovery_record_checksum(value: Any) -> str:
    material = "{}:{}:{}:{}:{}".format(
        value.get("recordVersion"),
        value.get("recordSeq"),
        value.get("previousHash"),
        value.get("payloadHash"),
        value.get("length"),
    )
    return hashlib.sha256(material.encode("utf-8")).hexdigest()


def recovery_record_errors(value: Any) -> List[str]:
    errors: List[str] = []
    if not isinstance(value, dict):
        return errors
    previous = value.get("previousHash")
    seq = value.get("recordSeq")
    if seq == 0 and previous != _GENESIS_HASH:
        errors.append("the first recovery record must use a genesis previousHash")
    if isinstance(seq, int) and seq > 0 and previous == _GENESIS_HASH:
        errors.append("a non-genesis recovery record cannot continue a broken previousHash chain")
    if value.get("checksum") != recovery_record_checksum(value):
        errors.append("recovery record checksum does not match the hash chain")
    return errors


def config_cell_errors(column: Any, raw: Any, row_key: Any, known_keys: set) -> List[str]:
    errors: List[str] = []
    name = column.get("name")
    declared = column.get("type")
    value = raw
    if value is None and "defaultValue" in column and not column.get("required"):
        value = column.get("defaultValue")
    if value is None:
        return errors

    def out_of_range(number: Any) -> bool:
        if "minimum" in column and number < column["minimum"]:
            return True
        if "maximum" in column and number > column["maximum"]:
            return True
        return False

    if declared == "bool":
        if not isinstance(value, bool):
            errors.append("row {} column {} must be bool".format(row_key, name))
    elif declared in _CONFIG_INT_BOUNDS:
        if not isinstance(value, int) or isinstance(value, bool):
            errors.append("row {} column {} must be {}".format(row_key, name, declared))
        else:
            low, high = _CONFIG_INT_BOUNDS[declared]
            if value < low or value > high or out_of_range(value):
                errors.append("row {} column {} is out of range".format(row_key, name))
    elif declared in ("f32", "f64"):
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            errors.append("row {} column {} must be {}".format(row_key, name, declared))
        elif out_of_range(value):
            errors.append("row {} column {} is out of range".format(row_key, name))
    elif declared == "string":
        if not isinstance(value, str):
            errors.append("row {} column {} must be string".format(row_key, name))
    elif declared == "enum":
        allowed = column.get("enumValues") or []
        if value not in allowed:
            errors.append("row {} column {} is not in enumValues".format(row_key, name))
    elif declared == "ref":
        if not isinstance(value, str):
            errors.append("row {} column {} must be a ref id".format(row_key, name))
        elif value not in known_keys:
            errors.append("row {} column {} missing ref {}".format(row_key, name, value))
        if not column.get("refTarget"):
            errors.append("ref column {} requires refTarget".format(name))
    return errors


def replication_length_errors(value: Any) -> List[str]:
    """ADR-045 section 3: `length` is a declared bound, not a byte-count claim.

    The envelope's wire byte encoding is not frozen yet (it waits on the state
    payload decision), so the gate cannot check `length` against real bytes. What
    it can check, and what a transport actually needs, is that the declared length
    fits the negotiated `transportPolicy.maxMessageBytes`.
    """
    errors: List[str] = []
    if not isinstance(value, dict):
        return errors
    length = value.get("length")
    policy = value.get("transportPolicy")
    if not isinstance(length, int) or isinstance(length, bool):
        return errors
    if isinstance(policy, dict):
        maximum = policy.get("maxMessageBytes")
        if isinstance(maximum, int) and not isinstance(maximum, bool) and length > maximum:
            errors.append("declared length exceeds the negotiated maxMessageBytes")
    return errors


def replication_body_errors(message_type: Any, body: Any) -> List[str]:
    errors: List[str] = []
    if not isinstance(body, dict):
        return ["typed replication body is required"]
    required = _REPLICATION_BODY_REQUIRED.get(message_type, ())
    missing = [field for field in required if field not in body]
    if missing:
        errors.append("{} body requires {}".format(message_type, ", ".join(missing)))
    if message_type == "FullSnapshot":
        vector = body.get("sessionRevisionVector")
        if not isinstance(vector, dict) or any(field not in vector for field in _SESSION_REVISION_FIELDS):
            errors.append("FullSnapshot requires a complete SessionRevisionVector")
        if isinstance(vector, dict):
            errors.extend(chunk_revision_set_errors(vector.get("chunkRevisionSet"), "FullSnapshot"))
    if message_type == "Delta":
        if "fromRevision" in body and "toRevision" in body and body.get("toRevision", 0) < body.get("fromRevision", 0):
            errors.append("Delta ToRevision cannot precede FromRevision")
        if body.get("gapDetected") and not body.get("resyncReason"):
            errors.append("a detected Delta gap must carry a Resync reason")
    if message_type == "ResyncRequest" and not body.get("resyncReason"):
        errors.append("ResyncRequest requires a reason")
    if message_type == "Error" and body.get("errorClass") not in ("Retryable", "Rejectable", "Fatal"):
        errors.append("Error body requires one of Retryable, Rejectable or Fatal")
    return errors


def message_type_consistency_errors(
    schemas: Dict[str, Dict[str, Any]],
    fixtures: Dict[str, Dict[str, Any]],
    id_registry: Any,
) -> List[str]:
    schema = schemas.get("replication-envelope", {}).get("document", {})
    schema_types = set(((schema.get("properties") or {}).get("messageType") or {}).get("enum") or [])
    registry_types = set()
    for namespace in id_registry.get("namespaces", []):
        if namespace.get("namespace") == "MessageType":
            registry_types = {item.get("id") for item in namespace.get("values", [])}
    used_registered = set()
    for fixture in fixtures.values():
        if fixture["meta"].get("schema") != "replication-envelope":
            continue
        message_type = fixture["document"].get("messageType")
        if isinstance(message_type, str) and message_type in registry_types:
            used_registered.add(message_type)
    errors: List[str] = []
    if schema_types != set(_REPLICATION_MESSAGE_TYPES):
        errors.append("replication-envelope messageType enum must match the frozen V1 set")
    if schema_types != registry_types:
        errors.append("MessageType Schema enum and ID Registry must be identical")
    missing = registry_types - used_registered
    if missing:
        errors.append("registered MessageType values are unused by fixtures: {}".format(sorted(missing)))
    return errors


def vocabulary_consistency_errors(schemas: Dict[str, Dict[str, Any]], id_registry: Any) -> List[str]:
    """Enforce the ADR-037 shared-vocabulary freezes at registry level."""
    errors: List[str] = []
    error_codes = set()
    for namespace in id_registry.get("namespaces", []):
        if namespace.get("namespace") == "ErrorCode":
            error_codes = {item.get("id") for item in namespace.get("values", [])}
    gate = schemas.get("protocol-permission-gate", {}).get("document", {})
    gate_reasons = set((((gate.get("properties") or {}).get("rejectReason") or {}).get("enum")) or [])
    unregistered = gate_reasons - error_codes
    if unregistered:
        errors.append("gate reject reasons must be registered ErrorCodes: {}".format(sorted(unregistered)))
    txn = schemas.get("cross-world-txn", {}).get("document", {})
    txn_reasons = set((((txn.get("properties") or {}).get("abortReason") or {}).get("enum")) or [])
    voxel = schemas.get("voxel-mutation-receipt", {}).get("document", {})
    voxel_reasons = set((((voxel.get("$defs") or {}).get("voxelMutationAbortReason") or {}).get("enum")) or [])
    if txn_reasons and voxel_reasons:
        if txn_reasons & voxel_reasons != _SHARED_ABORT_REASONS:
            errors.append("the shared abort reason intersection is frozen by ADR-037")
        if txn_reasons - voxel_reasons != _TXN_ONLY_ABORT_REASONS:
            errors.append("transaction-only abort reasons are frozen by ADR-037")
        if voxel_reasons - txn_reasons != _VOXEL_ONLY_ABORT_REASONS:
            errors.append("voxel-only abort reasons are frozen by ADR-037")
    release = schemas.get("release-manifest", {}).get("document", {})
    core_package = ((release.get("properties") or {}).get("coreEnginePackage")) or {}
    core_required = set(core_package.get("required") or [])
    common = load_json(SCHEMA_DIR / "common.schema.json")
    identity_required = set((((common.get("$defs") or {}).get("packageIdentity") or {}).get("required")) or [])
    if identity_required and not identity_required <= core_required:
        errors.append("coreEnginePackage must require every packageIdentity member")
    return errors


def state_machine_consistency_errors(
    schemas: Dict[str, Dict[str, Any]],
    fixtures: Dict[str, Dict[str, Any]],
) -> List[str]:
    """Enforce the ADR-038 descriptor registry against the frozen machine set."""
    errors: List[str] = []
    if "state-machine-descriptor" not in schemas:
        return errors
    descriptors: Dict[str, Any] = {}
    for fixture in fixtures.values():
        meta = fixture["meta"]
        if meta.get("schema") != "state-machine-descriptor" or meta.get("expected") != "valid":
            continue
        document = fixture["document"]
        machine_id = document.get("machineId")
        if machine_id in descriptors:
            errors.append("duplicate state machine descriptor {}".format(machine_id))
        descriptors[machine_id] = document
    if set(descriptors) != set(_STATE_MACHINE_SOURCES):
        missing = sorted(set(_STATE_MACHINE_SOURCES) - set(descriptors))
        extra = sorted(set(descriptors) - set(_STATE_MACHINE_SOURCES))
        errors.append(
            "descriptor registry must cover the frozen ADR-038 machine set"
            " (missing: {}, unregistered: {})".format(missing, extra)
        )
    for machine_id, source in _STATE_MACHINE_SOURCES.items():
        descriptor = descriptors.get(machine_id)
        if descriptor is None or source is None:
            continue
        schema_id, enum_path = source
        node: Any = schemas.get(schema_id, {}).get("document", {})
        for key in enum_path:
            node = node.get(key) if isinstance(node, dict) else None
            if node is None:
                break
        schema_states = set(node or [])
        if schema_states and set(descriptor.get("states") or []) != schema_states:
            errors.append(
                "descriptor {} states must equal the {} state enum".format(machine_id, schema_id)
            )
    ability = descriptors.get("GasAbility")
    if ability is not None:
        pairs = {(item.get("from"), item.get("to")) for item in ability.get("transitions") or []}
        expected_pairs = {
            (source, dest)
            for source, dests in _ABILITY_TRANSITIONS.items()
            for dest in dests
        }
        if pairs != expected_pairs:
            errors.append("GasAbility descriptor transitions must equal the ADR-031 table")
        if set(ability.get("terminalStates") or []) != _ABILITY_TERMINAL:
            errors.append("GasAbility descriptor terminal states must equal the ADR-031 set")
    effect = descriptors.get("GasEffect")
    if effect is not None:
        pairs = {(item.get("from"), item.get("to")) for item in effect.get("transitions") or []}
        expected_pairs = {
            (source, dest)
            for source, dests in _EFFECT_TRANSITIONS.items()
            for dest in dests
        }
        if pairs != expected_pairs:
            errors.append("GasEffect descriptor transitions must equal the ADR-031 table")
        if set(effect.get("terminalStates") or []) != _EFFECT_TERMINAL:
            errors.append("GasEffect descriptor terminal states must equal the ADR-031 set")
        self_events = {(item.get("state"), item.get("event")) for item in effect.get("selfEvents") or []}
        if self_events != {("Active", "Stack"), ("Active", "Duration"), ("Active", "Refresh")}:
            errors.append("GasEffect descriptor self events must equal the ADR-031 Active-internal set")
    return errors


# --------------------------------------------------------------------------
# ADR-042 Ed25519 (RFC 8032 PureEdDSA) verification.
#
# A verifier, never a signer: the gate only checks published vectors, so no
# private key is needed and none is committed. Self-tested against the RFC 8032
# section 7.1 vectors on every run, so a defect here cannot silently pass a bad
# vector -- the SHA-256 K[28] incident is the reason that self-test is not
# optional.
# --------------------------------------------------------------------------

_ED_P = 2 ** 255 - 19
_ED_L = 2 ** 252 + 27742317777372353535851937790883648493
_ED_D = -121665 * pow(121666, _ED_P - 2, _ED_P) % _ED_P
_ED_I = pow(2, (_ED_P - 1) // 4, _ED_P)


def _ed_recover_x(y: int, sign: int) -> Optional[int]:
    if y >= _ED_P:
        return None
    xx = (y * y - 1) * pow(_ED_D * y * y + 1, _ED_P - 2, _ED_P) % _ED_P
    x = pow(xx, (_ED_P + 3) // 8, _ED_P)
    if (x * x - xx) % _ED_P != 0:
        x = x * _ED_I % _ED_P
    if (x * x - xx) % _ED_P != 0:
        return None
    if x % 2 != sign:
        x = _ED_P - x
    return x


def _ed_add(point: Tuple[int, int, int, int], other: Tuple[int, int, int, int]) -> Tuple[int, int, int, int]:
    x1, y1, z1, t1 = point
    x2, y2, z2, t2 = other
    a = (y1 - x1) * (y2 - x2) % _ED_P
    b = (y1 + x1) * (y2 + x2) % _ED_P
    c = 2 * t1 * t2 * _ED_D % _ED_P
    d = 2 * z1 * z2 % _ED_P
    e, f, g, h = b - a, d - c, d + c, b + a
    return (e * f % _ED_P, g * h % _ED_P, f * g % _ED_P, e * h % _ED_P)


def _ed_mul(point: Tuple[int, int, int, int], scalar: int) -> Tuple[int, int, int, int]:
    result = (0, 1, 1, 0)
    while scalar > 0:
        if scalar & 1:
            result = _ed_add(result, point)
        point = _ed_add(point, point)
        scalar >>= 1
    return result


_ED_GY = 4 * pow(5, _ED_P - 2, _ED_P) % _ED_P
_ED_GX = _ed_recover_x(_ED_GY, 0)
_ED_G = (_ED_GX, _ED_GY, 1, _ED_GX * _ED_GY % _ED_P)


def _ed_decompress(data: bytes) -> Optional[Tuple[int, int, int, int]]:
    if len(data) != 32:
        return None
    y = int.from_bytes(data, "little")
    sign = y >> 255
    y &= (1 << 255) - 1
    x = _ed_recover_x(y, sign)
    return None if x is None else (x, y, 1, x * y % _ED_P)


def _ed_equal(point: Tuple[int, int, int, int], other: Tuple[int, int, int, int]) -> bool:
    x1, y1, z1, _ = point
    x2, y2, z2, _ = other
    return (x1 * z2 - x2 * z1) % _ED_P == 0 and (y1 * z2 - y2 * z1) % _ED_P == 0


def ed25519_verify(public_key: bytes, message: bytes, signature: bytes) -> bool:
    """RFC 8032 PureEdDSA verification. Returns False for any malformed input."""
    if len(public_key) != 32 or len(signature) != 64:
        return False
    point = _ed_decompress(public_key)
    if point is None:
        return False
    r = _ed_decompress(signature[:32])
    if r is None:
        return False
    scalar = int.from_bytes(signature[32:], "little")
    if scalar >= _ED_L:
        return False
    digest = hashlib.sha512(signature[:32] + public_key + message).digest()
    h = int.from_bytes(digest, "little") % _ED_L
    return _ed_equal(_ed_mul(_ED_G, scalar), _ed_add(r, _ed_mul(point, h)))


# RFC 8032 section 7.1, tests 1-3: (public key, message, signature), all hex.
_RFC8032_VECTORS = (
    (
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        "",
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
    ),
    (
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
        "72",
        "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
    ),
    (
        "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
        "af82",
        "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a",
    ),
)


def ed25519_self_test_errors() -> List[str]:
    """The verifier must reproduce RFC 8032 and reject a one-bit mutation."""
    errors: List[str] = []
    for index, (public_hex, message_hex, signature_hex) in enumerate(_RFC8032_VECTORS, start=1):
        public = bytes.fromhex(public_hex)
        message = bytes.fromhex(message_hex)
        signature = bytes.fromhex(signature_hex)
        if not ed25519_verify(public, message, signature):
            errors.append("Ed25519 verifier rejects RFC 8032 test {}".format(index))
        mutated = bytearray(signature)
        mutated[-1] ^= 0x01
        if ed25519_verify(public, message, bytes(mutated)):
            errors.append("Ed25519 verifier accepts a mutated RFC 8032 test {}".format(index))
    return errors


def trust_key_id(trust_domain: str, public_key: bytes) -> str:
    """ADR-042 section 3: keyId is a function of the key, never a chosen name."""
    return "{}-{}".format(trust_domain.lower(), hashlib.sha256(public_key).hexdigest()[:16])


def trust_preimage(envelope: Dict[str, Any]) -> bytes:
    """ADR-042 section 2: the domain-separated bytes that are actually signed."""
    return b"\x00".join(
        [
            b"LumioSignatureV1",
            str(envelope.get("trustDomain", "")).encode("utf-8"),
            str(envelope.get("payloadType", "")).encode("utf-8"),
            str(envelope.get("payloadDigest", "")).encode("utf-8"),
        ]
    )


def evaluate_trust(envelope: Dict[str, Any], policy: Dict[str, Any]) -> Tuple[str, Optional[str]]:
    """Run the ADR-042 section 4 rejection order and report the first failure."""
    keys = {key.get("keyId"): key for key in policy.get("keys", [])}
    if str(policy.get("trustDomain")) != str(envelope.get("trustDomain")):
        return "Rejected", "TrustRootUnknown"
    if not envelope.get("signature"):
        return "Rejected", "SignatureMissing"
    key = keys.get(envelope.get("keyId"))
    if key is None:
        return "Rejected", "TrustRootUnknown"
    if key.get("status") == "Revoked":
        return "Rejected", "KeyRevoked"
    try:
        public = bytes.fromhex(str(key.get("publicKey", "")))
        signature = bytes.fromhex(str(envelope.get("signature", "")))
    except ValueError:
        return "Rejected", "SignatureInvalid"
    if not ed25519_verify(public, trust_preimage(envelope), signature):
        return "Rejected", "SignatureInvalid"
    signed_at = str(envelope.get("signedAt", ""))
    if signed_at < str(key.get("notBefore", "")) or signed_at > str(key.get("notAfter", "")):
        return "Rejected", "TrustPolicyRejected"
    return "Trusted", None


def published_canonical_profile_errors() -> List[str]:
    """ADR-041: the published profile must be re-derivable from the fixture inputs."""
    published = PACKAGE_DIR / _CANONICAL_PROFILE_FILE
    if not published.is_file():
        return []
    document = load_json(published)
    try:
        derived = _abi.derive_canonical_profile(ROOT)
    except _abi.CanonicalError as exc:
        return ["published canonical/digest profile cannot be derived: {}".format(exc)]
    if canonical_json(derived) != canonical_json(document):
        return [
            "published {} does not match the profile derived from the registered inputs;"
            " regenerate with `python3 tools/lumio_contract.py generate --out packages`".format(
                _CANONICAL_PROFILE_FILE
            )
        ]
    return []


def published_root_abi_bundle_errors() -> List[str]:
    """ADR-040: the published bundle must be re-derivable from the ABI document."""
    published = PACKAGE_DIR / _ABI_BUNDLE_FILE
    if not published.is_file():
        return []
    document = load_json(published)
    abi = load_json(ROOT / _abi.ABI_DOCUMENT)
    expected_compiler = _abi.compiler_hash(ROOT)
    if document.get("compiler", {}).get("digest") != expected_compiler:
        return [
            "published Root ABI bundle records compiler digest {} but the locked compiler hashes to {};"
            " regenerate with `python3 tools/lumio_contract.py generate --out packages`".format(
                document.get("compiler", {}).get("digest"), expected_compiler
            )
        ]
    expected_input = _abi.abi_input_hash(ROOT)
    if document.get("inputHash") != expected_input:
        return [
            "published Root ABI bundle records inputHash {} but the frozen input set hashes to {};"
            " regenerate with `python3 tools/lumio_contract.py generate --out packages`".format(
                document.get("inputHash"), expected_input
            )
        ]
    try:
        derived = _abi.derive_bundle(
            abi,
            document.get("compiler", {}).get("digest", ""),
            document.get("inputHash", ""),
            [
                (item.get("path"), item.get("role"), item.get("digest"))
                for item in document.get("outputFiles", [])
            ],
        )
    except _abi.AbiError as exc:
        return ["published Root ABI bundle cannot be derived: {}".format(exc)]
    if canonical_json(derived) != canonical_json(document):
        return [
            "published {} does not match the bundle derived from {};"
            " regenerate with `python3 tools/lumio_contract.py generate --out packages`".format(
                _ABI_BUNDLE_FILE, _abi.ABI_DOCUMENT
            )
        ]
    for item in document.get("outputFiles", []):
        target = PACKAGE_DIR / str(item.get("path", ""))
        if not target.is_file():
            return ["published Root ABI output file is missing: {}".format(item.get("path"))]
        digest = hashlib.sha256(target.read_bytes()).hexdigest()
        if digest != item.get("digest"):
            return [
                "published Root ABI output {} digest {} does not match the bundle".format(
                    item.get("path"), digest
                )
            ]
    return []


def semantic_errors(schema_id: str, value: Any) -> List[str]:
    errors: List[str] = []

    if schema_id == "cross-world-txn":
        if value.get("commitOrder") != ["VoxelCommit", "EcsCommandBufferCommit"]:
            errors.append("commit order must be VoxelCommit then EcsCommandBufferCommit")
        state = value.get("state")
        markers = value.get("participantMarkers", {})
        voxel_marker = markers.get("voxelCommit")
        ecs_marker = markers.get("ecsCommandBufferCommit")
        intent = value.get("commitIntentPersisted")
        buffer_state = value.get("commandBufferState")
        apply_result = value.get("ecsApplyResult")
        abort_reason = value.get("abortReason")
        if intent is True and abort_reason in _BUSINESS_ABORT_REASONS:
            errors.append("CommitIntent persistence forbids a later business-level reject")
        if state == "Committed":
            if intent is not True:
                errors.append("Committed transaction requires persisted CommitIntent")
            if voxel_marker != "Applied" or ecs_marker != "Applied":
                errors.append("Committed transaction requires both participant markers Applied")
            if buffer_state != "Applied":
                errors.append("Committed transaction requires CommandBuffer Applied")
            if apply_result not in ("Applied", "AlreadyApplied"):
                errors.append("Committed transaction requires ECS Apply Applied or AlreadyApplied")
            if value.get("resultRevisionVector") is None:
                errors.append("Committed transaction requires ResultRevisionVector")
        elif state == "Aborted":
            if intent is True:
                errors.append("Aborted transaction cannot persist CommitIntent")
            if voxel_marker not in (None, "NotStarted") or ecs_marker not in (None, "NotStarted"):
                errors.append("Aborted transaction cannot report a started participant")
            if buffer_state == "Applied":
                errors.append("Aborted transaction cannot report CommandBuffer Applied")
            if abort_reason == "RevisionConflict":
                if value.get("observedGameRevision") == value.get("expectedGameRevision") and value.get("observedVoxelRevision") == value.get("expectedVoxelRevision"):
                    errors.append("RevisionConflict requires an observed revision mismatch")
        elif state == "Expired":
            if intent is True:
                errors.append("Expired transaction cannot persist CommitIntent")
            if abort_reason != "DeadlineExceeded":
                errors.append("Expired transaction requires DeadlineExceeded")
        elif state == "CommitIntent":
            if intent is not True:
                errors.append("CommitIntent state requires persisted CommitIntent")
            if buffer_state != "Prepared":
                errors.append("CommitIntent requires CommandBuffer Prepared")
        elif state == "Indeterminate":
            if intent is not True:
                errors.append("Indeterminate transaction requires persisted CommitIntent")
            if apply_result not in ("Indeterminate", "Faulted", "Applied", "AlreadyApplied"):
                errors.append("Indeterminate Apply must be Applied, AlreadyApplied, Indeterminate or Faulted")
            if voxel_marker == "Applied" and ecs_marker == "Applied":
                errors.append("Indeterminate transaction cannot have both markers Applied")
            if voxel_marker not in ("Unknown", "Applied", "Failed", "NotStarted") or ecs_marker not in ("Unknown", "Applied", "Failed", "NotStarted"):
                errors.append("Indeterminate transaction requires enum participant markers")
        if value.get("tickId", 0) > value.get("deadlineTick", 0):
            errors.append("transaction TickId cannot exceed DeadlineTick")

    elif schema_id == "replication-envelope":
        message_type = value.get("messageType")
        if message_type not in _REPLICATION_MESSAGE_TYPES:
            errors.append("messageType {} is not registered".format(message_type))
        if message_type == "FullSnapshot" and value.get("reliability") != "Reliable":
            errors.append("FullSnapshot must use Reliable delivery")
        integrity = value.get("integrity")
        if isinstance(integrity, dict):
            rule = _INTEGRITY_VALUE_RULES.get(str(integrity.get("algorithm")))
            if rule is not None and rule.match(str(integrity.get("value", ""))) is None:
                errors.append("integrity value does not match the declared algorithm")
        errors.extend(replication_length_errors(value))
        errors.extend(replication_body_errors(message_type, value.get("body")))

    elif schema_id == "entity-identity":
        lifecycle = value.get("lifecycle")
        namespace = value.get("namespace")
        domain = str(value.get("authorityDomain", ""))
        if lifecycle == "Alive" and "tombstoneUntilRevision" in value:
            errors.append("Alive entity cannot retain a tombstone horizon")
        if lifecycle == "Tombstoned" and "tombstoneUntilRevision" not in value:
            errors.append("Tombstoned entity requires a tombstone horizon")
        if namespace == "Provisional":
            if not domain.startswith("client-"):
                errors.append("Provisional entity must use the client provisional authority domain")
            if value.get("remappedFrom") == value.get("netEntityId"):
                errors.append("provisional remapping must change the NetEntityId")
        elif namespace == "Authoritative" and domain.startswith("client-"):
            errors.append("Authoritative entity cannot use a provisional authority domain")
        elif namespace == "Replay":
            if "sourceRevision" not in value or "sourceReleaseId" not in value:
                errors.append("Replay entity that retains the original id requires sourceRevision and sourceReleaseId")

    elif schema_id == "release-manifest":
        if value.get("compatibilityPolicy") == "ExactRelease" and value.get("serverReleaseId") != value.get("clientReleaseId"):
            errors.append("ExactRelease requires identical server and client release ids")
        product = value.get("productId")
        release = str(value.get("gameReleaseId", ""))
        if product and release and not release.startswith(product + "-"):
            errors.append("GameReleaseId must be namespaced by ProductId")
        core_engine = value.get("coreEnginePackage")
        if isinstance(core_engine, dict) and core_engine.get("abiIdentity") != value.get("coreEngineAbi"):
            errors.append("coreEnginePackage.abiIdentity must equal coreEngineAbi")

    elif schema_id == "maintenance-command":
        mode = value.get("mode")
        action = value.get("action")
        if mode == "Forced" and action != "StopInputAndKick":
            errors.append("Forced maintenance requires StopInputAndKick")
        if mode == "Graceful" and action != "DrainAndKick":
            errors.append("Graceful maintenance requires DrainAndKick")
        if value.get("broadcastCode") != "MaintenanceKick":
            errors.append("maintenance must broadcast MaintenanceKick")
        grace = value.get("graceDeadlineSeconds")
        if mode == "Forced" and grace != 0:
            errors.append("Forced maintenance requires graceDeadlineSeconds 0")
        if mode == "Graceful" and isinstance(grace, int) and grace < 1:
            errors.append("Graceful maintenance requires a positive grace window")

    elif schema_id == "snapshot-header":
        if value.get("compression") == "None" and "payload" in value:
            payload = str(value.get("payload", "")).encode("utf-8")
            if value.get("payloadLength") != len(payload):
                errors.append("uncompressed payloadLength does not match payload bytes")
            digest = hashlib.sha256(payload).hexdigest()
            if value.get("hash") != digest:
                errors.append("snapshot hash does not match uncompressed payload")
        if value.get("activationState") == "Active" and value.get("encryption") is None:
            errors.append("Active snapshot must declare encryption metadata")

    elif schema_id == "config-table":
        keys = [row.get("key") for row in value.get("rows", [])]
        if len(keys) != len(set(keys)):
            errors.append("config table row keys must be unique")
        column_defs = value.get("columns", [])
        columns = [column.get("name") for column in column_defs]
        if len(columns) != len(set(columns)):
            errors.append("config table column names must be unique")
        known_columns = {column.get("name"): column for column in column_defs}
        required_columns = {column.get("name") for column in column_defs if column.get("required")}
        known_keys = {key for key in keys if isinstance(key, str)}
        for row in value.get("rows", []):
            values = row.get("values", {})
            missing = required_columns.difference(values.keys())
            if missing:
                errors.append("row {} is missing required columns {}".format(row.get("key"), sorted(missing)))
            unknown = set(values.keys()) - set(known_columns)
            if unknown:
                errors.append("row {} has unknown columns {}".format(row.get("key"), sorted(unknown)))
            for name, cell in values.items():
                column = known_columns.get(name)
                if column:
                    errors.extend(config_cell_errors(column, cell, row.get("key"), known_keys))
        if value.get("activation") == "ProductionSignedSwitch" and not value.get("signature"):
            errors.append("production config activation requires a signature")

    elif schema_id == "logging-event":
        category = value.get("category")
        durability = value.get("durability")
        if category in ("Audit", "TxnJournal", "CommandLog") and durability not in ("Durable", "EmergencySync"):
            errors.append("{} events cannot use BestEffort durability".format(category))
        if durability == "EmergencySync" and value.get("severity") not in ("Error", "Fatal"):
            errors.append("EmergencySync durability is reserved for Error/Fatal severity")
        if category == "FailureBundle" and not value.get("correlation", {}).get("snapshotId"):
            errors.append("FailureBundle event requires SnapshotId correlation")
        errors.extend(correlation_scope_errors(value.get("correlation")))

    elif schema_id == "processor-descriptor":
        if value.get("mayEmitStructuralCommands") and value.get("phase") not in _BUSINESS_PROCESSOR_PHASES:
            errors.append("mayEmitStructuralCommands is only legal on a business phase")

    elif schema_id == "failure-bundle":
        names = [artifact.get("name") for artifact in value.get("artifacts", [])]
        if len(names) != len(set(names)):
            errors.append("FailureBundle artifact names must be unique")
        incident_kind = value.get("incidentKind")
        if incident_kind == "Simulation":
            has_snapshot = bool(value.get("snapshotId"))
            has_pre_snapshot = bool(value.get("noSnapshotReason") and value.get("bootstrapPhase") and (value.get("lastKnownRevision") or value.get("lastKnownManifest")))
            if not has_snapshot and not has_pre_snapshot:
                errors.append("a Simulation incident must reference a snapshot or a pre-snapshot bootstrap attestation")
        if incident_kind in ("CoreEngineLoad", "SupplyChain") and "coreEngine" not in value:
            errors.append("a {} incident requires the coreEngine block".format(incident_kind))
        errors.extend(correlation_scope_errors(value.get("correlation")))

    elif schema_id == "native-managed-abi":
        # ADR-040 freezes the linux-x86_64-glibc layout profile: a 16-byte root and
        # api-table header, then one pointer-sized word per declared and reserved slot.
        pointer_bytes = int(value.get("pointerWidth", 64)) // 8
        table_names = [table.get("name") for table in value.get("apiTable", [])]
        if len(table_names) != len(set(table_names)):
            errors.append("apiTable names must be unique")
        for table in value.get("apiTable", []):
            slots = table.get("slots", [])
            if table.get("functionCount") != len(slots):
                errors.append("api table {} functionCount must equal the number of slots".format(table.get("name")))
            indexes = [slot.get("slotIndex") for slot in slots]
            if indexes != list(range(len(slots))):
                errors.append("api table {} slot indexes must be contiguous from 0".format(table.get("name")))
            slot_names = [slot.get("name") for slot in slots]
            if len(slot_names) != len(set(slot_names)):
                errors.append("api table {} slot names must be unique".format(table.get("name")))
            declared = int(table.get("structSize", 0))
            minimum = _ABI_TABLE_HEADER_BYTES + (
                int(table.get("functionCount", 0)) + int(table.get("reservedSlots", 0))
            ) * pointer_bytes
            if declared < minimum:
                errors.append(
                    "api table {} structSize {} is below the derived minimum {}".format(
                        table.get("name"), declared, minimum
                    )
                )
            if pointer_bytes and declared % pointer_bytes != 0:
                errors.append(
                    "api table {} structSize must be a multiple of the pointer alignment".format(
                        table.get("name")
                    )
                )
        root_declared = int(value.get("structSize", 0))
        root_minimum = _ABI_ROOT_HEADER_BYTES + len(value.get("apiTable", [])) * pointer_bytes
        if root_declared < root_minimum:
            errors.append(
                "root structSize {} is below the derived minimum {}".format(root_declared, root_minimum)
            )
        if pointer_bytes and root_declared % pointer_bytes != 0:
            errors.append("root structSize must be a multiple of the pointer alignment")

    elif schema_id == "root-abi-bundle":
        profile = value.get("layoutProfile", {})
        if profile != _ABI_LAYOUT_PROFILE:
            errors.append("layoutProfile must equal the ADR-040 frozen profile")
        pointer_bytes = int(profile.get("pointerBytes") or 0)
        table_header = int(profile.get("tableHeaderBytes") or 0)
        root_header = int(profile.get("rootHeaderBytes") or 0)
        mapped = [row.get("typeRef") for row in value.get("typeMapping", [])]
        if len(mapped) != len(set(mapped)):
            errors.append("typeMapping entries must be unique")
        if set(mapped) != _ABI_TYPE_MAPPING_KEYS:
            missing = sorted(_ABI_TYPE_MAPPING_KEYS - set(mapped))
            extra = sorted(set(mapped) - _ABI_TYPE_MAPPING_KEYS)
            errors.append(
                "typeMapping must cover the frozen typeRef grammar"
                " (missing: {}, unregistered: {})".format(missing, extra)
            )
        root = value.get("root", {})
        if root.get("minimumStructSize", 0) > root.get("declaredStructSize", 0):
            errors.append("root minimumStructSize cannot exceed declaredStructSize")
        expected_root_minimum = root_header + len(root.get("tables", [])) * pointer_bytes
        if root.get("minimumStructSize") != expected_root_minimum:
            errors.append(
                "root minimumStructSize must equal {}".format(expected_root_minimum)
            )
        for index, entry in enumerate(root.get("tables", [])):
            if entry.get("offset") != root_header + index * pointer_bytes:
                errors.append(
                    "root table {} offset must follow the frozen root header".format(entry.get("name"))
                )
        declared_tables = [entry.get("name") for entry in root.get("tables", [])]
        if declared_tables != [table.get("name") for table in value.get("tables", [])]:
            errors.append("root table pointers must match the api table list in document order")
        for table in value.get("tables", []):
            expected_minimum = table_header + (
                int(table.get("functionCount", 0)) + int(table.get("reservedSlots", 0))
            ) * pointer_bytes
            if table.get("minimumStructSize") != expected_minimum:
                errors.append(
                    "table {} minimumStructSize must equal {}".format(table.get("name"), expected_minimum)
                )
            if int(table.get("minimumStructSize", 0)) > int(table.get("declaredStructSize", 0)):
                errors.append(
                    "table {} minimumStructSize cannot exceed declaredStructSize".format(table.get("name"))
                )
            slots = table.get("slots", [])
            if table.get("functionCount") != len(slots):
                errors.append("table {} functionCount must equal the number of slots".format(table.get("name")))
            for slot in slots:
                expected_offset = table_header + int(slot.get("slotIndex", 0)) * pointer_bytes
                if slot.get("offset") != expected_offset:
                    errors.append(
                        "table {} slot {} offset must equal {}".format(
                            table.get("name"), slot.get("name"), expected_offset
                        )
                    )
        roles = [item.get("role") for item in value.get("outputFiles", [])]
        if sorted(roles) != sorted(_ABI_OUTPUT_ROLES):
            errors.append("outputFiles must publish exactly the frozen ADR-040 role set")
        paths = [item.get("path") for item in value.get("outputFiles", [])]
        if paths != _ABI_OUTPUT_PATHS:
            errors.append("outputFiles must publish exactly the frozen ADR-040 path list")

    elif schema_id == "artifact-index":
        paths = [entry.get("path") for entry in value.get("entries", [])]
        if len(paths) != len(set(paths)):
            errors.append("artifact paths must be unique within an index")
        # ADR-041: artifactSetDigest is the digest of this index with its own
        # artifactSetDigest member omitted, so the self-reference is defined.
        try:
            expected = _abi.canonical_digest(_abi.artifact_set_digest_input(value))
        except _abi.CanonicalError as exc:
            errors.append("artifact index is not canonicalizable: {}".format(exc))
        else:
            if value.get("artifactSetDigest") != expected:
                errors.append(
                    "artifactSetDigest must equal the ADR-041 recomputation {}".format(expected)
                )

    elif schema_id == "loader-profile":
        if value.get("reentry") != _LOADER_REENTRY:
            errors.append("reentry must equal the ADR-043 freeze")
        order = [item.get("errorCode") for item in value.get("errorPriority", [])]
        if order != _LOADER_ERROR_PRIORITY:
            errors.append("errorPriority must equal the ADR-043 frozen order {}".format(_LOADER_ERROR_PRIORITY))
        ranks = [item.get("rank") for item in value.get("errorPriority", [])]
        if ranks != list(range(1, len(ranks) + 1)):
            errors.append("errorPriority ranks must be contiguous from 1")
        seen = set()
        for vector in value.get("acquireVectors", []):
            seen.add(vector.get("case"))
            decision, reason = evaluate_acquire(vector)
            if decision != vector.get("expected"):
                errors.append(
                    "acquire vector {} expects {} but the latch rule yields {}".format(
                        vector.get("vectorId"), vector.get("expected"), decision
                    )
                )
            elif reason != vector.get("rejectReason"):
                errors.append(
                    "acquire vector {} declares rejectReason {} but the latch rule yields {}".format(
                        vector.get("vectorId"), vector.get("rejectReason"), reason
                    )
                )
        if seen != _LOADER_ACQUIRE_CASES:
            errors.append(
                "acquire vectors must cover every frozen case (missing: {})".format(
                    sorted(_LOADER_ACQUIRE_CASES - seen)
                )
            )
        for vector in value.get("failureVectors", []):
            expected = evaluate_loader_failure(list(vector.get("causes") or []))
            if vector.get("reported") != expected:
                errors.append(
                    "failure vector {} reports {} but the frozen priority yields {}".format(
                        vector.get("vectorId"), vector.get("reported"), expected
                    )
                )

    elif schema_id == "evidence-profile":
        declared = {
            item.get("kind"): (
                item.get("artifactKind"), item.get("format"), item.get("specVersion"), item.get("mediaType")
            )
            for item in value.get("profiles", [])
        }
        if declared != _EVIDENCE_PROFILES:
            errors.append("profiles must equal the ADR-044 frozen format/version/media-type set")
        for item in value.get("profiles", []):
            if item.get("digestObject") != "RawBytes":
                errors.append("evidence digests are over raw published bytes, never a canonicalization")
        seen = set()
        for vector in value.get("vectors", []):
            seen.add(vector.get("case"))
            decision, reason = evaluate_evidence(vector)
            if decision != vector.get("expected"):
                errors.append(
                    "evidence vector {} expects {} but the coverage rules yield {}".format(
                        vector.get("vectorId"), vector.get("expected"), decision
                    )
                )
            elif reason != vector.get("rejectReason"):
                errors.append(
                    "evidence vector {} declares rejectReason {} but the coverage rules yield {}".format(
                        vector.get("vectorId"), vector.get("rejectReason"), reason
                    )
                )
        if seen != _EVIDENCE_VECTOR_CASES:
            errors.append(
                "evidence vectors must cover every frozen case (missing: {})".format(
                    sorted(_EVIDENCE_VECTOR_CASES - seen)
                )
            )

    elif schema_id == "trust-profile":
        signature_profile = value.get("signatureProfile", {})
        if signature_profile.get("preimageFields") != _TRUST_PREIMAGE_FIELDS:
            errors.append("preimageFields must equal the ADR-042 frozen order {}".format(_TRUST_PREIMAGE_FIELDS))
        order = [(item.get("order"), item.get("rejectReason")) for item in value.get("rejectPriority", [])]
        if order != _TRUST_REJECT_PRIORITY:
            errors.append("rejectPriority must equal the ADR-042 frozen order {}".format(_TRUST_REJECT_PRIORITY))
        policy = value.get("trustPolicy", {})
        domain = str(policy.get("trustDomain", ""))
        for key in policy.get("keys", []):
            try:
                public = bytes.fromhex(str(key.get("publicKey", "")))
            except ValueError:
                errors.append("policy key {} publicKey is not hex".format(key.get("keyId")))
                continue
            expected = trust_key_id(domain, public)
            if key.get("keyId") != expected:
                errors.append(
                    "policy key {} keyId must be the ADR-042 derivation {}".format(key.get("keyId"), expected)
                )
            if key.get("notBefore", "") >= key.get("notAfter", ""):
                errors.append("policy key {} validity window is empty".format(key.get("keyId")))
            if domain == "Production" and str(key.get("keyId", "")).startswith("test-"):
                errors.append("a Production trust policy cannot carry a test key")
        seen_cases = set()
        for vector in value.get("vectors", []):
            seen_cases.add(vector.get("case"))
            decision, reason = evaluate_trust(vector.get("envelope", {}), policy)
            if decision != vector.get("expected"):
                errors.append(
                    "vector {} expects {} but evaluating the frozen order yields {}".format(
                        vector.get("vectorId"), vector.get("expected"), decision
                    )
                )
            elif reason != vector.get("rejectReason"):
                errors.append(
                    "vector {} declares rejectReason {} but the frozen order yields {}".format(
                        vector.get("vectorId"), vector.get("rejectReason"), reason
                    )
                )
        if seen_cases != _TRUST_VECTOR_CASES:
            missing = sorted(_TRUST_VECTOR_CASES - seen_cases)
            extra = sorted(seen_cases - _TRUST_VECTOR_CASES)
            errors.append(
                "vectors must cover every frozen case (missing: {}, unregistered: {})".format(missing, extra)
            )

    elif schema_id == "canonical-digest-profile":
        if value.get("canonicalForm") != _CANONICAL_FORM:
            errors.append("canonicalForm must equal the ADR-041 frozen CanonicalJsonV1 parameters")
        if value.get("digestAlgorithm") != _CANONICAL_DIGEST_ALGORITHM:
            errors.append("digestAlgorithm must equal the ADR-041 frozen construction")
        declared = [item.get("digest") for item in value.get("digestDomains", [])]
        if declared != _CANONICAL_DIGEST_KEYS:
            missing = [key for key in _CANONICAL_DIGEST_KEYS if key not in declared]
            extra = [key for key in declared if key not in _CANONICAL_DIGEST_KEYS]
            errors.append(
                "digestDomains must cover the frozen digest set in order"
                " (missing: {}, unregistered: {})".format(missing, extra)
            )
        for item in value.get("digestDomains", []):
            expected_tag = _CANONICAL_DOMAIN_TAGS.get(item.get("digest"))
            if expected_tag is not None and item.get("domainTag") != expected_tag:
                errors.append(
                    "digest {} must use domain tag {}".format(item.get("digest"), expected_tag)
                )
            expected_norm = _CANONICAL_NORMALIZATION.get(item.get("digest"))
            if expected_norm is not None and (item.get("normalization") or []) != expected_norm:
                errors.append(
                    "digest {} must publish the frozen machine-readable normalization {}".format(
                        item.get("digest"), expected_norm
                    )
                )
        goldens = value.get("goldens", [])
        ids = [item.get("id") for item in goldens]
        if len(ids) != len(set(ids)):
            errors.append("golden ids must be unique")
        covered = {item.get("case") for item in goldens}
        if covered != _CANONICAL_GOLDEN_CASES:
            missing = sorted(_CANONICAL_GOLDEN_CASES - covered)
            extra = sorted(covered - _CANONICAL_GOLDEN_CASES)
            errors.append(
                "goldens must cover every frozen case"
                " (missing: {}, unregistered: {})".format(missing, extra)
            )
        by_case = {}
        for golden in goldens:
            try:
                text = _abi.canonical_bytes(golden.get("input"))
            except _abi.CanonicalError as exc:
                errors.append("golden {} input is not canonicalizable: {}".format(golden.get("id"), exc))
                continue
            if golden.get("canonicalBytes") != text:
                errors.append(
                    "golden {} canonicalBytes does not re-canonicalize from its input".format(golden.get("id"))
                )
            digest = hashlib.sha256(text.encode("ascii")).hexdigest()
            if golden.get("sha256") != digest:
                errors.append("golden {} sha256 must equal {}".format(golden.get("id"), digest))
            by_case.setdefault(golden.get("case"), []).append(golden)
        multi = by_case.get("MultiArtifact") or []
        permutation = by_case.get("PathOrderPermutation") or []
        if multi and permutation and multi[0].get("sha256") != permutation[0].get("sha256"):
            errors.append("a path permutation must collapse to the MultiArtifact digest")
        single = by_case.get("SingleArtifact") or []
        versioned = by_case.get("SchemaVersionChange") or []
        if single and versioned and single[0].get("sha256") == versioned[0].get("sha256"):
            errors.append("a schema version change must change the digest")

    elif schema_id == "core-engine-manifest":
        if "Native" not in value.get("capabilitySet", []):
            errors.append("a CoreEngine NativeLibrary package must declare the Native capability")
        # ADR-044: format is a closed set; a tool default or a free string is not a profile.
        for kind, ref in (value.get("evidenceSet") or {}).items():
            fmt = (ref or {}).get("format")
            expected = _EVIDENCE_PROFILES.get(kind)
            if expected is not None and fmt != expected[1]:
                errors.append(
                    "evidenceSet.{} format {} is outside the ADR-044 frozen set (expected {})".format(
                        kind, fmt, expected[1]
                    )
                )

    elif schema_id == "signature-envelope":
        if value.get("trustDomain") == "Production" and str(value.get("keyId", "")).startswith("test-"):
            errors.append("a Production trust domain cannot use a test key")

    elif schema_id == "verified-package-descriptor":
        checks = value.get("checks", {})
        check_names = ("manifestDigestVerified", "artifactDigestsVerified", "signatureVerified", "trustPolicyVerified")
        if value.get("trustDecision") == "Trusted":
            if not all(checks.get(name) is True for name in check_names):
                errors.append("a Trusted decision requires every verification check to pass")
            if value.get("rejectReason"):
                errors.append("a Trusted decision cannot carry a reject reason")
        if value.get("trustDecision") == "Rejected" and not value.get("rejectReason"):
            errors.append("a Rejected decision must carry a reject reason")

    elif schema_id == "id-registry":
        namespace_names = [namespace.get("namespace") for namespace in value.get("namespaces", [])]
        if len(namespace_names) != len(set(namespace_names)):
            errors.append("ID Registry namespace names must be unique")
        for namespace in value.get("namespaces", []):
            ids = [item.get("id") for item in namespace.get("values", [])]
            numerics = [item.get("numeric") for item in namespace.get("values", [])]
            if len(ids) != len(set(ids)):
                errors.append("ID Registry ids must be unique in {}".format(namespace.get("namespace")))
            if len(numerics) != len(set(numerics)):
                errors.append("ID Registry numeric values must be unique in {}".format(namespace.get("namespace")))
            if namespace.get("namespace") == "ErrorCode":
                out_of_range = sorted(
                    numeric for numeric in numerics
                    if isinstance(numeric, int) and numeric > _STATUS_NUMERIC_MAX
                )
                if out_of_range:
                    errors.append(
                        "ErrorCode numerics must fit lumio_status_t (int32): {}".format(out_of_range)
                    )

    elif schema_id == "contract-result":
        failures = value.get("failures", 0)
        if value.get("passed") is not (failures == 0):
            errors.append("contract result passed must agree with failure count")
        if value.get("validated", 0) < failures:
            errors.append("contract result failures cannot exceed validated count")

    elif schema_id == "release-catalog":
        routes = [(entry.get("productId"), entry.get("gameReleaseId")) for entry in value.get("entries", [])]
        if len(routes) != len(set(routes)):
            errors.append("ReleaseCatalog ProductId + GameReleaseId routes must be unique")
        for entry in value.get("entries", []):
            if entry.get("state") == "Serving" and entry.get("healthy") is False:
                errors.append("a Serving ReleasePool must report healthy")

    elif schema_id == "replication-mapping":
        source = value.get("source", {})
        target = value.get("target", {})
        if source == target:
            errors.append("replication mapping source and target must be explicit projections")
        if value.get("role") == "ClientToServer" and value.get("owner") == "AllClients":
            errors.append("ClientToServer mappings cannot be owned by AllClients")

    elif schema_id == "migration-manifest":
        nodes = value.get("nodes", [])
        node_ids = [node.get("nodeId") for node in nodes]
        if len(node_ids) != len(set(node_ids)):
            errors.append("migration node ids must be unique")
        known = set(node_ids)
        graph = {node.get("nodeId"): list(node.get("dependsOn", [])) for node in nodes}
        for node_id, dependencies in graph.items():
            missing = [dependency for dependency in dependencies if dependency not in known]
            if missing:
                errors.append("migration node {} references missing dependency {}".format(node_id, missing[0]))
        visiting = set()
        visited = set()

        def visit(node_id: str) -> None:
            if node_id in visiting:
                errors.append("migration dependency graph contains a cycle")
                return
            if node_id in visited or node_id not in graph:
                return
            visiting.add(node_id)
            for dependency in graph[node_id]:
                visit(dependency)
            visiting.remove(node_id)
            visited.add(node_id)

        for node_id in graph:
            visit(node_id)
        if value.get("targetSchemaEpoch", 0) <= value.get("sourceSchemaEpoch", 0):
            errors.append("target SchemaEpoch must be newer than source SchemaEpoch")

    elif schema_id == "mod-manifest":
        if value.get("nativeLibraries"):
            errors.append("V1 Mod boundary cannot load native libraries")
        if value.get("lifecycle") not in ("Reserved",):
            errors.append("third-party Mods remain Reserved in V1")

    elif schema_id == "client-authority-update":
        canonical_order = [
            "ValidateBaselineAndRevision",
            "RestoreConfirmedPredictionFrame",
            "ApplyAuthoritativeEcsGasVoxel",
            "DropConfirmedCommands",
            "ReplayUnconfirmedInOrder",
            "EmitPresentationDiff",
        ]
        if value.get("stepOrder") != canonical_order:
            errors.append("authority update stepOrder must match Architecture §7.2")
        results = value.get("stepResults", {})
        all_steps = all(results.get(key) is True for key in (
            "validateBaselineAndRevision",
            "restoreConfirmedPredictionFrame",
            "applyAuthoritativeEcsGasVoxel",
            "dropConfirmedCommands",
            "replayUnconfirmedInOrder",
            "emitPresentationDiff",
        ))
        visibility = (
            value.get("visibleSideEffects"),
            value.get("ackAllowed"),
            value.get("baselineAdvanced"),
            value.get("confirmedPointAdvanced"),
        )
        state = value.get("state")
        if state == "Committed":
            if not all_steps:
                errors.append("Committed authority update requires every step to succeed")
            if visibility != (True, True, True, True):
                errors.append("Committed authority update is the only state that may expose side effects, Ack, Baseline or Confirmed Point")
            if value.get("resultRevisionVector") is None:
                errors.append("Committed authority update requires ResultRevisionVector")
            if value.get("faultClass"):
                errors.append("Committed authority update cannot carry a FaultClass")
        else:
            if any(visibility):
                errors.append("non-Committed authority update must have zero visible side effects and must not Ack or advance Baseline/Confirmed Point")
            if not value.get("faultClass"):
                errors.append("non-Committed authority update requires a FaultClass attestation")
            if state == "Aborted" and value.get("faultClass") != "SessionLocalProven":
                errors.append("Aborted authority update must attest SessionLocalProven")
            if state == "Indeterminate" and value.get("faultClass") not in ("SlotStateUnproven", "ProcessFault"):
                errors.append("Indeterminate authority update must attest SlotStateUnproven or ProcessFault")

    elif schema_id == "protocol-permission-gate":
        admitted_claims = set(value.get("admittedClaims") or [])
        extra_claims = [claim for claim in value.get("claims") or [] if claim not in admitted_claims]
        matched = (
            value.get("sessionId") == value.get("admittedSessionId")
            and value.get("productId") == value.get("admittedProductId")
            and value.get("gameReleaseId") == value.get("admittedGameReleaseId")
            and value.get("role") == value.get("admittedRole")
            and not extra_claims
            and value.get("connectionGeneration") == value.get("admittedConnectionGeneration")
        )
        if value.get("verdict") == "Accept":
            if not matched:
                errors.append("Accept requires Session, Release, Role, Claims and Connection Generation to match admission")
            if extra_claims:
                errors.append("Accept cannot include claims outside admission")
            if value.get("rejectReason"):
                errors.append("Accept cannot carry a rejectReason")
        elif value.get("verdict") == "Reject":
            if not value.get("rejectReason"):
                errors.append("Reject requires a rejectReason")
            if value.get("connectionGeneration") != value.get("admittedConnectionGeneration"):
                if value.get("rejectReason") != "StaleConnectionGeneration":
                    errors.append("a Connection Generation mismatch must use StaleConnectionGeneration")

    elif schema_id == "generated-contract-artifact":
        if value.get("implementationDependencies"):
            errors.append("generated contract artifacts must not depend on implementation projects")
        forbidden = set(value.get("forbiddenDependents") or [])
        if forbidden != {"LumioClient", "LumioGame"}:
            errors.append("generated artifacts must forbid LumioClient and LumioGame implementation dependents")

    elif schema_id == "tick-phase-contract":
        if value.get("tickModel") != "FailStop":
            errors.append("V1 Tick model must be FailStop")
        if value.get("commitPoint") != "GasAndEventFinalize":
            errors.append("the unique Tick Commit Point is GasAndEventFinalize")
        phases = value.get("phases") or []
        names = [phase.get("phase") for phase in phases]
        if names != _TICK_PHASES:
            errors.append("the Tick phase matrix must list the 13 phases in order")
        commit_flags = [phase for phase in phases if phase.get("isAuthoritativeCommitPoint")]
        if len(commit_flags) != 1 or commit_flags[0].get("phase") != "GasAndEventFinalize":
            errors.append("exactly one phase may be the authoritative Commit Point")
        for phase in phases:
            if phase.get("phase") == "GasAndEventFinalize":
                if phase.get("visibleToLaterPhases") != "AfterCommit":
                    errors.append("the Commit Point must publish AfterCommit visibility")
            elif phase.get("isAuthoritativeCommitPoint"):
                errors.append("only GasAndEventFinalize may set the Commit Point flag")
            elif phase.get("phase") in _TICK_PHASES and _TICK_PHASES.index(phase.get("phase")) < _TICK_PHASES.index("GasAndEventFinalize"):
                if phase.get("visibleToLaterPhases") != "WithinTickPrivate":
                    errors.append("{} cannot be visible after commit".format(phase.get("phase")))

    elif schema_id == "gas-lifecycle":
        machine = value.get("machine")
        source = value.get("fromState")
        dest = value.get("toState")
        if machine == "Ability":
            allowed = _ABILITY_TRANSITIONS.get(source, set())
            if dest not in allowed:
                errors.append("illegal Ability transition {} -> {}".format(source, dest))
            if dest in _ABILITY_TERMINAL and value.get("handleValid") is not False:
                errors.append("a terminal Ability state invalidates the Handle")
        elif machine == "Effect":
            event = value.get("event")
            if event in ("Stack", "Duration", "Refresh"):
                if source != "Active" or dest != "Active":
                    errors.append("Stack/Duration/Refresh are Active-internal events, not states")
            else:
                allowed = _EFFECT_TRANSITIONS.get(source, set())
                if dest not in allowed:
                    errors.append("illegal Effect transition {} -> {}".format(source, dest))
            if dest in _EFFECT_TERMINAL and value.get("handleValid") is not False:
                errors.append("a terminal Effect state invalidates the Handle")

    elif schema_id == "state-machine-descriptor":
        states = value.get("states") or []
        state_set = set(states)
        terminal = set(value.get("terminalStates") or [])
        any_active = set(value.get("anyActiveTo") or [])
        initial = value.get("initialState")
        if initial not in state_set:
            errors.append("initialState must be a declared state")
        for field_name, members in (("terminalStates", terminal), ("anyActiveTo", any_active)):
            undeclared = members - state_set
            if undeclared:
                errors.append("{} must reference declared states: {}".format(field_name, sorted(undeclared)))
        outgoing: Dict[str, set] = {}
        seen_events = set()
        for transition in value.get("transitions") or []:
            source = transition.get("from")
            dest = transition.get("to")
            event = transition.get("event")
            if source not in state_set or dest not in state_set:
                errors.append("transition {} -> {} must reference declared states".format(source, dest))
                continue
            if source in terminal:
                errors.append("terminal state {} cannot own outgoing transitions".format(source))
            if (source, event) in seen_events:
                errors.append("state {} reuses event {} for two transitions".format(source, event))
            seen_events.add((source, event))
            outgoing.setdefault(source, set()).add(dest)
        for entry in value.get("selfEvents") or []:
            state = entry.get("state")
            if state not in state_set:
                errors.append("selfEvents must reference declared states")
            elif state in terminal:
                errors.append("terminal state {} cannot own internal events".format(state))
        if initial in state_set:
            reachable = {initial}
            frontier = [initial]
            while frontier:
                current = frontier.pop()
                targets = set(outgoing.get(current, set()))
                if current not in terminal:
                    targets |= any_active
                for target in targets:
                    if target in state_set and target not in reachable:
                        reachable.add(target)
                        frontier.append(target)
            unreachable = state_set - reachable
            if unreachable:
                errors.append("states must be reachable from initialState: {}".format(sorted(unreachable)))
        if not any_active:
            for state in sorted(state_set - terminal):
                if not outgoing.get(state):
                    errors.append("non-terminal state {} needs an outgoing transition".format(state))

    elif schema_id in ("txn-journal-record", "command-log-record", "wal-record-envelope"):
        errors.extend(recovery_record_errors(value))
        if schema_id == "wal-record-envelope":
            inner = value.get("inner")
            if not isinstance(inner, dict):
                errors.append("WAL envelope requires an inner recovery record")
            elif value.get("innerKind") == "TxnJournal" and not inner.get("txnId"):
                errors.append("TxnJournal WAL inner requires txnId")
            elif value.get("innerKind") == "CommandLog" and not inner.get("commandId"):
                errors.append("CommandLog WAL inner requires commandId")

    elif schema_id == "gameplay-scope-activation":
        if value.get("reactivatedOldScope") is True:
            errors.append("a quiesced or unloaded old Scope cannot be reactivated")
        if value.get("failurePhase") == "AfterSwitch" and value.get("recovery") != "SessionFaultedFromSnapshot":
            errors.append("post-BarrierSwitch failure must Fault the Session from Snapshot")
        if value.get("failurePhase") == "BeforeSwitch" and value.get("recovery") != "KeepOldActive":
            errors.append("pre-BarrierSwitch failure must keep OldActive and discard NewStaging")
        if value.get("stage") in ("OldQuiescing", "OldUnloaded") and value.get("switchCommitted") is not True:
            errors.append("OldQuiescing/OldUnloaded require a committed BarrierSwitch")

    elif schema_id == "voxel-snapshot-payload":
        kind = value.get("kind")
        if kind == "DiffPayload":
            base = value.get("base") or {}
            advance = value.get("worldRevisionAdvance")
            cut = value.get("cutProjection") or {}
            base_revision = base.get("baseWorldRevision")
            target_revision = cut.get("worldRevision")
            if all(isinstance(item, int) for item in (advance, base_revision, target_revision)):
                if target_revision != base_revision + advance:
                    errors.append("diff target revision must equal baseWorldRevision plus worldRevisionAdvance")
        if kind in ("SnapshotPayload", "DiffPayload"):
            chunks = value.get("chunks") or []
            coordinates = []
            for entry in chunks:
                match = _CHUNK_KEY.match(str(entry.get("chunkId", "")))
                if match is not None:
                    coordinates.append(tuple(int(group) for group in match.groups()))
            if coordinates != sorted(coordinates):
                errors.append("payload chunk entries must be in canonical CoordXYZAscending order")
            offset = 0
            contiguous = True
            for entry in chunks:
                if entry.get("byteOffset") != offset:
                    errors.append("payload chunk byte ranges must be contiguous and ascending")
                    contiguous = False
                    break
                length = entry.get("byteLength")
                offset += length if isinstance(length, int) else 0
            if contiguous and chunks and isinstance(value.get("payloadLength"), int) and offset != value.get("payloadLength"):
                errors.append("payload chunk byte ranges must sum to payloadLength")

    elif schema_id == "voxel-durability-ack":
        if value.get("kind") == "DurabilityAck":
            covered = [chunk.get("chunkId") for chunk in value.get("coveredChunks") or []]
            if len(covered) != len(set(covered)):
                errors.append("a durability acknowledgment cannot cover the same chunk twice")

    return errors


def registry() -> Tuple[Dict[str, Dict[str, Any]], Dict[str, Dict[str, Any]]]:
    schema_index = load_json(SCHEMA_INDEX)
    fixture_index = load_json(FIXTURE_INDEX)
    if schema_index.get("baselineId") != _CURRENT_BASELINE or fixture_index.get("baselineId") != _CURRENT_BASELINE:
        raise ContractError("schema and fixture registries must use baseline {}".format(_CURRENT_BASELINE))
    if schema_index.get("schemaSetVersion") != 1 or fixture_index.get("fixtureSetVersion") != 1:
        raise ContractError("unsupported schema or fixture registry version")
    if not ID_REGISTRY_FILE.is_file():
        raise ContractError("ID Registry is missing: {}".format(ID_REGISTRY_FILE))
    id_registry = load_json(ID_REGISTRY_FILE)
    if id_registry.get("baselineId") != _CURRENT_BASELINE:
        raise ContractError("ID Registry must use baseline {}".format(_CURRENT_BASELINE))
    resolver = SchemaResolver()
    registry_schema_path = SCHEMA_DIR / "schemas-index.json"
    registry_schema = load_json(registry_schema_path)
    registry_errors = structural_errors(schema_index, registry_schema, registry_schema_path, resolver)
    if registry_errors:
        raise ContractError("invalid schema registry: {}".format("; ".join(registry_errors[:3])))
    schemas: Dict[str, Dict[str, Any]] = {}
    for entry in schema_index.get("schemas", []):
        schema_id = entry.get("id")
        if not schema_id or schema_id in schemas:
            raise ContractError("duplicate or empty schema id: {}".format(schema_id))
        path = SCHEMA_DIR / str(entry.get("file", ""))
        if not path.is_file():
            raise ContractError("registered schema is missing: {}".format(path))
        schemas[schema_id] = {"meta": entry, "path": path, "document": load_json(path)}
    id_schema = schemas["id-registry"]
    id_errors = structural_errors(id_registry, id_schema["document"], id_schema["path"], resolver)
    if id_errors:
        raise ContractError("invalid ID Registry: {}".format("; ".join(id_errors[:3])))
    canonical_id_fixture = FIXTURE_DIR / "valid" / "id-registry.json"
    if canonical_id_fixture.is_file() and canonical_json(id_registry) != canonical_json(load_json(canonical_id_fixture)):
        raise ContractError("ids/index.json and its positive fixture differ")
    registered_schema_files = {str(item["path"].relative_to(SCHEMA_DIR)) for item in schemas.values()}
    for path in SCHEMA_DIR.glob("*.schema.json"):
        # common.schema.json contains shared definitions and is intentionally
        # not a standalone fixture contract.
        if path.name != "common.schema.json" and path.name not in registered_schema_files:
            raise ContractError("schema file is not registered: {}".format(path))
    fixtures: Dict[str, Dict[str, Any]] = {}
    for entry in fixture_index.get("fixtures", []):
        fixture_id = entry.get("id")
        if not fixture_id or fixture_id in fixtures:
            raise ContractError("duplicate or empty fixture id: {}".format(fixture_id))
        schema_id = entry.get("schema")
        if schema_id not in schemas:
            raise ContractError("fixture {} references unknown schema {}".format(fixture_id, schema_id))
        path = FIXTURE_DIR / str(entry.get("file", ""))
        if not path.is_file():
            raise ContractError("registered fixture is missing: {}".format(path))
        if entry.get("expected") not in ("valid", "invalid"):
            raise ContractError("fixture {} has invalid expected result".format(fixture_id))
        fixtures[fixture_id] = {"meta": entry, "path": path, "document": load_json(path)}
    registered_fixture_files = {str(item["path"].relative_to(FIXTURE_DIR)) for item in fixtures.values()}
    for directory in (FIXTURE_DIR / "valid", FIXTURE_DIR / "invalid"):
        for path in directory.glob("*.json"):
            if str(path.relative_to(FIXTURE_DIR)) not in registered_fixture_files:
                raise ContractError("fixture file is not registered: {}".format(path))

    for schema_id, schema in schemas.items():
        if schema["meta"].get("priority") != "P0":
            continue
        covered = [fixture["meta"] for fixture in fixtures.values() if fixture["meta"].get("schema") == schema_id]
        if not any(item.get("expected") == "valid" for item in covered):
            raise ContractError("P0 schema {} has no positive fixture".format(schema_id))
        if not any(item.get("expected") == "invalid" for item in covered):
            raise ContractError("P0 schema {} has no failure fixture".format(schema_id))
    consistency = message_type_consistency_errors(schemas, fixtures, id_registry)
    consistency.extend(vocabulary_consistency_errors(schemas, id_registry))
    consistency.extend(state_machine_consistency_errors(schemas, fixtures))
    consistency.extend(published_root_abi_bundle_errors())
    consistency.extend(published_canonical_profile_errors())
    consistency.extend(ed25519_self_test_errors())
    if consistency:
        raise ContractError("; ".join(consistency))
    return schemas, fixtures


def validate_fixture(fixture_id: str, fixture: Dict[str, Any], schemas: Dict[str, Dict[str, Any]], resolver: SchemaResolver) -> Tuple[bool, List[str]]:
    meta = fixture["meta"]
    schema_id = str(meta["schema"])
    schema = schemas[schema_id]
    structural = structural_errors(fixture["document"], schema["document"], schema["path"], resolver)
    semantic = semantic_errors(schema_id, fixture["document"])
    errors = structural + semantic
    expected = meta.get("expected")
    passed = (expected == "valid" and not errors) or (expected == "invalid" and bool(errors))
    return passed, errors


def command_validate(selected: Optional[str], json_output: bool = False) -> int:
    schemas, fixtures = registry()
    if selected:
        if selected not in fixtures:
            print("unknown fixture: {}".format(selected), file=sys.stderr)
            return 2
        targets = [(selected, fixtures[selected])]
    else:
        targets = sorted(fixtures.items())
    resolver = SchemaResolver()
    failures = 0
    result_items = []
    for fixture_id, fixture in targets:
        passed, errors = validate_fixture(fixture_id, fixture, schemas, resolver)
        result_items.append({
            "id": fixture_id,
            "expected": fixture["meta"].get("expected"),
            "passed": passed,
            "errors": errors,
        })
        if passed and not json_output:
            print("PASS {} ({})".format(fixture_id, fixture["meta"].get("expected")))
        elif not passed:
            failures += 1
            if not json_output:
                print("FAIL {}".format(fixture_id))
                for error in errors[:12]:
                    print("  - {}".format(error))
                if len(errors) > 12:
                    print("  - ... {} more".format(len(errors) - 12))
    if json_output:
        print(json.dumps({
            "resultVersion": 1,
            "baselineId": _CURRENT_BASELINE,
            "command": "validate",
            "passed": failures == 0,
            "validated": len(targets),
            "failures": failures,
            "fixtureResults": result_items,
        }, ensure_ascii=True, sort_keys=True))
    else:
        print("Validated {} fixture(s), {} failure(s).".format(len(targets), failures))
    return 1 if failures else 0


def command_canonical(path_text: str) -> int:
    path = Path(path_text).resolve()
    value = load_json(path)
    print(canonical_json(value))
    return 0


def command_hash(path_text: str) -> int:
    path = Path(path_text).resolve()
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    print("{}  {}".format(digest, path))
    return 0


def command_generate(out: str) -> int:
    """Publish six kinds × Rust/C# and prove two-run outputHash stability."""
    import shutil
    import tempfile

    tools_dir = str(Path(__file__).resolve().parent)
    if tools_dir not in sys.path:
        sys.path.insert(0, tools_dir)
    from lumio_generate import generate  # type: ignore

    out_dir = Path(out)
    if not out_dir.is_absolute():
        out_dir = ROOT / out

    tmp_root = Path(tempfile.mkdtemp(prefix="lumio-gen-"))
    try:
        first = generate(ROOT, tmp_root / "a")
        second = generate(ROOT, tmp_root / "b")
        by_a = {item["artifactId"]: item for item in first["artifacts"]}
        by_b = {item["artifactId"]: item for item in second["artifacts"]}
        if set(by_a) != set(by_b):
            print("error: artifact set drifted between generate runs", file=sys.stderr)
            return 1
        mismatches = []
        for artifact_id, item in sorted(by_a.items()):
            other = by_b[artifact_id]
            for field in ("outputHash", "compilerHash", "inputHash", "baselineId", "schemaEpoch"):
                if item.get(field) != other.get(field):
                    mismatches.append("{}:{}".format(artifact_id, field))
        if mismatches:
            print("error: unstable generate {}".format(mismatches), file=sys.stderr)
            return 1
        if canonical_json(first.get("rootAbi")) != canonical_json(second.get("rootAbi")):
            print("error: unstable Root ABI bundle between generate runs", file=sys.stderr)
            return 1
        if out_dir.exists():
            shutil.rmtree(out_dir)
        shutil.copytree(tmp_root / "a", out_dir)
        index = first
    except RuntimeError as exc:
        print("error: {}".format(exc), file=sys.stderr)
        return 1
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)

    schema_file = SCHEMA_DIR / "generated-contract-artifact.schema.json"
    schema = load_json(schema_file)
    resolver = SchemaResolver()
    failures = 0
    for path in sorted((out_dir / "descriptors").glob("*.json")):
        document = load_json(path)
        errors = structural_errors(document, schema, schema_file, resolver)
        errors.extend(semantic_errors("generated-contract-artifact", document))
        if errors:
            failures += 1
            print("FAIL {}: {}".format(path.name, "; ".join(errors)))
        else:
            print("PASS descriptor {}".format(path.name))
    if failures:
        return 1
    if len(index["artifacts"]) != 12:
        print("error: expected 12 artifacts, got {}".format(len(index["artifacts"])), file=sys.stderr)
        return 1
    print("generated {} artifacts under {}".format(len(index["artifacts"]), out_dir))
    print("compilerHash {}".format(index["compilerHash"]))
    print("inputHash {}".format(index["inputHash"]))
    print("stateMachines {}".format(",".join(index["stateMachineIds"])))
    root_abi = index.get("rootAbi") or {}
    print("rootAbi bundle {} digest {}".format(root_abi.get("bundlePath"), root_abi.get("bundleDigest")))
    print("rootAbi compiler {} {}".format(
        root_abi.get("compiler", {}).get("name"), root_abi.get("compiler", {}).get("version")))
    print("stable outputHash: yes")
    return 0


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    validate_parser = subparsers.add_parser("validate", help="validate registered fixtures")
    validate_parser.add_argument("--fixture", help="validate one fixture id")
    validate_parser.add_argument("--json", action="store_true", help="emit the versioned machine-readable result")
    canonical_parser = subparsers.add_parser("canonical", help="print canonical JSON")
    canonical_parser.add_argument("file")
    hash_parser = subparsers.add_parser("hash", help="print SHA-256 for a file")
    hash_parser.add_argument("file")
    generate_parser = subparsers.add_parser("generate", help="publish V1.4 generated contract artifacts")
    generate_parser.add_argument("--out", default="packages", help="output directory (default: packages)")
    args = parser.parse_args(argv)
    try:
        if args.command == "validate":
            return command_validate(args.fixture, args.json)
        if args.command == "canonical":
            return command_canonical(args.file)
        if args.command == "hash":
            return command_hash(args.file)
        if args.command == "generate":
            return command_generate(args.out)
    except (ContractError, OSError) as exc:
        print("error: {}".format(exc), file=sys.stderr)
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
