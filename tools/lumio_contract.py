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
import math
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

_LUMIO_BIN_FORM = dict(_abi.LUMIO_BIN_FORM)
_LUMIO_BIN_DIGEST_ALGORITHM = dict(_abi.LUMIO_BIN_DIGEST_ALGORITHM)
_LUMIO_BIN_VALUE_ENCODING = dict(_abi.LUMIO_BIN_VALUE_ENCODING)
_LUMIO_BIN_VECTOR_SEMANTICS = dict(_abi.LUMIO_BIN_VECTOR_SEMANTICS)
_LUMIO_BIN_GOLDEN_CASES = set(_abi.LUMIO_BIN_GOLDEN_CASES)
_LUMIO_BIN_REJECTION_CASES = set(_abi.LUMIO_BIN_REJECTION_CASES)
_LUMIO_BIN_PROFILE_FILE = _abi.LUMIO_BIN_PROFILE_FILE

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


def _transition_event_table(path: Path) -> Dict[Tuple[str, str], set]:
    """Derive the explicit event vocabulary for each state transition."""
    document = load_json(path)
    table: Dict[Tuple[str, str], set] = {}
    for item in document.get("transitions") or []:
        key = (str(item.get("from")), str(item.get("to")))
        table.setdefault(key, set()).add(item.get("event"))
    return table


def _terminal_states(path: Path) -> set:
    return set(load_json(path).get("terminalStates") or [])


_ABILITY_FIXTURE = FIXTURE_DIR / "valid" / "state-machine-gas-ability.json"
_EFFECT_FIXTURE = FIXTURE_DIR / "valid" / "state-machine-gas-effect.json"
_ABILITY_TRANSITIONS = _transition_table(_ABILITY_FIXTURE)
_ABILITY_TRANSITION_EVENTS = _transition_event_table(_ABILITY_FIXTURE)
_ABILITY_TERMINAL = _terminal_states(_ABILITY_FIXTURE)
_EFFECT_TRANSITIONS = _transition_table(_EFFECT_FIXTURE)
_EFFECT_TRANSITION_EVENTS = _transition_event_table(_EFFECT_FIXTURE)
_EFFECT_TERMINAL = _terminal_states(_EFFECT_FIXTURE)
_GAS_ADMISSION_ORDER = ("HandlePermission", "Cooldown", "Cost", "Tag", "GameCustom")
_GAS_COMMIT_ORDER = ("Cooldown", "Cost")
_GAS_EFFECT_EVENT_RANK = {
    "Apply": 0,
    "Hit": 1,
    "Overflow": 2,
    "SnapshotReplacement": 3,
    "Stack": 4,
    "Refresh": 4,
    "Suppress": 4,
    "Duration": 5,
    "Period": 6,
    "Removal": 7,
    "Expire": 7,
}
_GAS_COMPONENT_NAMES = (
    "AbilityComponent",
    "EffectComponent",
    "AttributeComponent",
    "TagComponent",
)
_GAS_TAG_QUERY_MODES = ("Exact", "Parent", "Child")
_GAS_NON_PREDICTABLE_ACTIONS = (
    "EffectRemoval",
    "EffectPeriod",
    "OutOfSimulation",
)
_GAS_PREDICTION_ROLLBACK_STEPS = (
    "RestoreConfirmedFrame",
    "ApplyAuthoritativeEcsGasVoxel",
    "ReplayUnconfirmedInputs",
)
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
    "GasAbility": ("gas-lifecycle", ("$defs", "abilityState", "enum")),
    "GasEffect": ("gas-lifecycle", ("$defs", "effectState", "enum")),
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
        triples = {(item.get("from"), item.get("to"), item.get("event")) for item in ability.get("transitions") or []}
        expected_pairs = {
            (source, dest)
            for source, dests in _ABILITY_TRANSITIONS.items()
            for dest in dests
        }
        if pairs != expected_pairs:
            errors.append("GasAbility descriptor transitions must equal the ADR-031 table")
        expected_triples = {
            (source, dest, event)
            for (source, dest), events in _ABILITY_TRANSITION_EVENTS.items()
            for event in events
        }
        if triples != expected_triples:
            errors.append("GasAbility descriptor transition events must equal the ADR-031 table")
        if set(ability.get("terminalStates") or []) != _ABILITY_TERMINAL:
            errors.append("GasAbility descriptor terminal states must equal the ADR-031 set")
    effect = descriptors.get("GasEffect")
    if effect is not None:
        pairs = {(item.get("from"), item.get("to")) for item in effect.get("transitions") or []}
        triples = {(item.get("from"), item.get("to"), item.get("event")) for item in effect.get("transitions") or []}
        expected_pairs = {
            (source, dest)
            for source, dests in _EFFECT_TRANSITIONS.items()
            for dest in dests
        }
        if pairs != expected_pairs:
            errors.append("GasEffect descriptor transitions must equal the ADR-031 table")
        expected_triples = {
            (source, dest, event)
            for (source, dest), events in _EFFECT_TRANSITION_EVENTS.items()
            for event in events
        }
        if triples != expected_triples:
            errors.append("GasEffect descriptor transition events must equal the ADR-031 table")
        if set(effect.get("terminalStates") or []) != _EFFECT_TERMINAL:
            errors.append("GasEffect descriptor terminal states must equal the ADR-031 set")
        self_events = {(item.get("state"), item.get("event")) for item in effect.get("selfEvents") or []}
        if self_events != {
            ("Active", "Stack"),
            ("Active", "Duration"),
            ("Active", "Refresh"),
            ("Active", "Suppress"),
        }:
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


def published_lumio_bin_profile_errors() -> List[str]:
    """ADR-047: the published binary profile must be re-derivable from the tool."""
    published = PACKAGE_DIR / _LUMIO_BIN_PROFILE_FILE
    if not published.is_file():
        return []
    document = load_json(published)
    try:
        derived = _abi.derive_lumio_bin_profile()
    except _abi.LumioBinError as exc:
        return ["published LumioBinV1 profile cannot be derived: {}".format(exc)]
    if canonical_json(derived) != canonical_json(document):
        return [
            "published {} does not match the profile derived from the frozen vectors;"
            " regenerate with `python3 tools/lumio_contract.py generate --out packages`".format(
                _LUMIO_BIN_PROFILE_FILE
            )
        ]
    return []


def published_canonical_surface_errors() -> List[str]:
    """ADR-041: both published language surfaces must carry the whole profile.

    The profile is only consumable if it can be read from the artifact a
    repository actually depends on. A surface published in one language and not
    the other is not a smaller surface, it is an asymmetry the missing side
    cannot work around, so Rust and C# are asserted together here rather than
    each being trusted to the reader of a generator diff.
    """
    rust_lib = (
        PACKAGE_DIR / "rust" / "lumio-gen-canonical-serializer" / "src" / "lib.rs"
    )
    cs_profile = (
        PACKAGE_DIR / "csharp" / "Lumio.Gen.CanonicalSerializer" / "CanonicalProfile.cs"
    )
    if not (rust_lib.is_file() and cs_profile.is_file()):
        return []
    try:
        profile = _abi.derive_canonical_profile(ROOT)
    except _abi.CanonicalError as exc:
        return ["canonical profile cannot be derived: {}".format(exc)]
    rust_text = rust_lib.read_text(encoding="utf-8")
    cs_text = cs_profile.read_text(encoding="utf-8")
    form = profile["canonicalForm"]
    errors: List[str] = []
    for rust_name, cs_name, value in (
        ("CANONICAL_FORM_ID", "FormId", form["formId"]),
        ("CANONICAL_ENCODING", "Encoding", form["encoding"]),
        ("CANONICAL_MEMBER_ORDER", "MemberOrder", form["memberOrder"]),
        ("CANONICAL_ARRAY_ORDER", "ArrayOrder", form["arrayOrder"]),
        ("CANONICAL_NUMBERS", "Numbers", form["numbers"]),
        ("CANONICAL_UNKNOWN_MEMBERS", "UnknownMembers", form["unknownMembers"]),
        ("CANONICAL_DUPLICATE_MEMBERS", "DuplicateMembers", form["duplicateMembers"]),
        ("DIGEST_ALGORITHM", "DigestAlgorithm", profile["digestAlgorithm"]["name"]),
        ("DIGEST_FRAMING", "DigestFraming", profile["digestAlgorithm"]["framing"]),
    ):
        expected_rust = 'pub const {}: &str = "{}";'.format(rust_name, value)
        if expected_rust not in rust_text:
            errors.append(
                "canonical-serializer lib.rs is missing or disagrees with `{}`".format(
                    expected_rust
                )
            )
        expected_cs = 'public const string {} = "{}";'.format(cs_name, value)
        if expected_cs not in cs_text:
            errors.append(
                "CanonicalProfile.cs is missing or disagrees with `{}`".format(expected_cs)
            )
    for domain in profile["digestDomains"]:
        tag = str(domain["domainTag"])
        pair = '"{}", "{}"'.format(domain["digest"], tag)
        if 'digest: "{}", domain_tag: "{}"'.format(domain["digest"], tag) not in rust_text:
            errors.append(
                "canonical-serializer lib.rs DIGEST_DOMAINS is missing {}".format(tag)
            )
        if "new DigestDomain({}".format(pair) not in cs_text:
            errors.append("CanonicalProfile.cs DigestDomains is missing {}".format(tag))
    for golden in profile["goldens"]:
        if str(golden["sha256"]) not in rust_text:
            errors.append(
                "canonical-serializer lib.rs CANONICAL_GOLDENS is missing {}".format(
                    golden["id"]
                )
            )
        if str(golden["sha256"]) not in cs_text:
            errors.append(
                "CanonicalProfile.cs CanonicalGoldens is missing {}".format(golden["id"])
            )
    return errors[:6]


def _message_type_ids() -> List[str]:
    """Registered `MessageType` ids — the gate's registered set, from the registry."""
    return _abi.load_message_ids(ROOT)


def published_capability_constant_errors() -> List[str]:
    """ADR-040 section 7 (D-015): the three emitted forms must agree with the registry.

    The registry is the authority and the generator is the only emitter, so the
    failure this guards is a stale *published* artifact: a `Capability` value
    added to `ids/index.json` without regenerating leaves three language surfaces
    disagreeing with the authority and with each other, silently.
    """
    header = PACKAGE_DIR / "abi" / "lumio_core.h"
    rust = PACKAGE_DIR / "rust" / "lumio-gen-language-binding" / "src" / "root_abi.rs"
    csharp = PACKAGE_DIR / "csharp" / "Lumio.Gen.LanguageBinding" / "RootAbi.cs"
    if not (header.is_file() and rust.is_file() and csharp.is_file()):
        return []
    capabilities = _abi.load_capabilities(ROOT)
    if not capabilities:
        return ["ids/index.json publishes no Capability namespace"]
    errors: List[str] = []
    header_text = header.read_text(encoding="utf-8")
    rust_text = rust.read_text(encoding="utf-8")
    csharp_text = csharp.read_text(encoding="utf-8")
    for name, numeric, status in capabilities:
        expected_c = "#define LUMIO_CAPABILITY_{} {}u".format(_abi.c_screaming(name), numeric)
        if expected_c not in header_text:
            errors.append("lumio_core.h is missing or disagrees with `{}`".format(expected_c))
        expected_rust = '("{}", {}, "{}")'.format(name, numeric, status)
        if expected_rust not in rust_text:
            errors.append("root_abi.rs capability table is missing {}".format(expected_rust))
        expected_cs = "public const uint {} = {}u;".format(name, numeric)
        if expected_cs not in csharp_text:
            errors.append("RootAbi.cs is missing or disagrees with `{}`".format(expected_cs))
    expected_count = "#define LUMIO_CAPABILITY_COUNT {}u".format(len(capabilities))
    if expected_count not in header_text:
        errors.append(
            "lumio_core.h must publish {} for the {} registered capabilities".format(
                expected_count, len(capabilities)
            )
        )
    return errors[:6]


def published_contract_body_errors() -> List[str]:
    """ADR-048 (D-3): the published type bodies must carry the schema's field order.

    Declaration order is the one property a JSON-shaped input cannot carry and a
    consumer cannot infer, so it is asserted here rather than left to the reader
    of a diff: each published type republishes its order, and that order must
    equal the order the schema declares today.
    """
    rust_body = (
        PACKAGE_DIR / "rust" / "lumio-gen-contract-types" / "src" / "bodies.rs"
    )
    cs_body = PACKAGE_DIR / "csharp" / "Lumio.Gen.ContractTypes" / "ContractBodies.cs"
    if not (rust_body.is_file() and cs_body.is_file()):
        return []
    try:
        projector = _abi.TypeProjector(ROOT)
        projector.project()
    except _abi.SchemaTypeError as exc:
        return ["closed contract types cannot be projected from the schemas: {}".format(exc)]
    rust_text = rust_body.read_text(encoding="utf-8")
    cs_text = cs_body.read_text(encoding="utf-8")
    by_name = {name: members for name, members in projector.structs}
    errors: List[str] = []
    for _schema_id, type_name in _abi.CLOSED_CONTRACT_TYPES:
        members = by_name.get(type_name)
        if members is None:
            errors.append("closed contract type {} was not projected".format(type_name))
            continue
        order = [field for field, _r, _c, _q, _k in members]
        expected_rust = "pub const FIELD_ORDER: &'static [&'static str] = &[{}];".format(
            ", ".join('"{}"'.format(f) for f in order)
        )
        if expected_rust not in rust_text:
            errors.append(
                "{} in bodies.rs does not publish the schema declaration order {}".format(
                    type_name, order
                )
            )
        if "public sealed class {}\n".format(type_name) not in cs_text:
            errors.append("ContractBodies.cs is missing the type {}".format(type_name))
    return errors[:6]


def published_cargo_lock_errors() -> List[str]:
    """The workspace lockfile is a generated artifact and must be published.

    `command_generate` rmtree's `packages/` and copies the freshly generated
    tree over it, so any tracked file the generator does not emit is staged as
    a deletion. `packages/rust/Cargo.lock` was exactly that file, and the
    deletion shipped twice (b8f8c50, f9c446b) before anyone caught it: the six
    crates have no dependencies, so any later `cargo` invocation recreates the
    lockfile byte-for-byte and `git status` goes quiet again. Only a
    regenerate-then-commit with no Cargo run in between leaves the deletion
    visible, and CI cannot see it either -- its `cargo check` runs after
    checkout and would just recreate the file there too.

    The workspace manifest is the guard's precondition rather than the lockfile
    itself: keying on the lockfile would make the missing-file case, which is
    the failure being guarded, silently skip the check.
    """
    manifest = PACKAGE_DIR / "rust" / "Cargo.toml"
    if not manifest.is_file():
        return []
    published = PACKAGE_DIR / "rust" / "Cargo.lock"
    hint = (
        "regenerate with `python3 tools/lumio_contract.py generate --out packages`"
    )
    if not published.is_file():
        return [
            "published packages/rust/Cargo.lock is missing while the workspace"
            " manifest is published; {}".format(hint)
        ]
    if published.read_text(encoding="utf-8") != _abi.workspace_lock():
        return [
            "published packages/rust/Cargo.lock does not match the lockfile"
            " derived from the workspace members; {}".format(hint)
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


def _gas_kind(value: Dict[str, Any]) -> str:
    """Return the lifecycle record kind, retaining the original transition form."""
    kind = value.get("kind")
    if kind:
        return str(kind)
    if "admissionOrder" in value:
        return "Admission"
    if "recheckOrder" in value:
        return "Commit"
    return "Transition"


def _gas_record_kind_errors(value: Dict[str, Any], kind: str) -> List[str]:
    """Keep record-kind conditionals enforced when the fallback validator is used."""
    errors: List[str] = []
    declared = value.get("kind")
    if "kind" in value and declared not in ("Transition", "Admission", "Commit"):
        errors.append("GAS lifecycle kind must be Transition, Admission or Commit")
    has_admission = "admissionOrder" in value
    has_commit = "recheckOrder" in value
    if kind == "Transition" and (has_admission or has_commit):
        errors.append("Transition records cannot carry Admission or Commit fields")
    elif kind == "Admission" and has_commit:
        errors.append("Admission records cannot carry Commit fields")
    elif kind == "Commit" and has_admission:
        errors.append("Commit records cannot carry Admission fields")
    return errors


def _gas_check_name(item: Any) -> Optional[str]:
    if not isinstance(item, dict):
        return None
    name = item.get("name")
    return None if name is None else str(name)


def _gas_check_passed(item: Any) -> Optional[bool]:
    if not isinstance(item, dict):
        return None
    if item.get("result") in ("Pass", "Fail"):
        return item.get("result") == "Pass"
    return None


def _gas_check_order_errors(checks: List[Any], label: str) -> List[str]:
    """Ensure declared check ordinals agree with their deterministic list order."""
    errors: List[str] = []
    for index, item in enumerate(checks, start=1):
        if not isinstance(item, dict):
            continue
        order = item.get("order")
        if isinstance(order, bool) or not isinstance(order, int):
            errors.append("{} check order must be an integer".format(label))
        elif order != index:
            errors.append("{} check order must match its list position".format(label))
    return errors


def _gas_finite_number(value: Any) -> bool:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return False
    try:
        return math.isfinite(float(value))
    except (OverflowError, ValueError):
        return False


def _gas_transition_errors(value: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    machine = value.get("machine")
    source = value.get("fromState")
    dest = value.get("toState")
    event = value.get("event")
    if machine == "Ability":
        allowed = _ABILITY_TRANSITIONS.get(source, set()) if isinstance(source, str) else set()
        if not isinstance(dest, str) or dest not in allowed:
            errors.append("illegal Ability transition {} -> {}".format(source, dest))
        elif not isinstance(event, str) or event not in _ABILITY_TRANSITION_EVENTS.get((source, dest), set()):
            errors.append(
                "event {} is not valid for Ability transition {} -> {}".format(event, source, dest)
            )
        if isinstance(dest, str) and dest in _ABILITY_TERMINAL and value.get("handleValid") is not False:
            errors.append("a terminal Ability state invalidates the Handle")
        if isinstance(dest, str) and dest not in _ABILITY_TERMINAL and "handleValid" in value and value.get("handleValid") is not True:
            errors.append("a non-terminal Ability state keeps the Handle valid")
        if dest == "RolledBack" and value.get("predicted") is not True:
            errors.append("Ability RolledBack is only valid for predicted instances")
    elif machine == "Effect":
        internal = {"Stack", "Duration", "Refresh", "Suppress"}
        if isinstance(event, str) and event in internal:
            if source != "Active" or dest != "Active":
                errors.append("{} is an Active-internal event, not a state transition".format(event))
            if "handleValid" in value and value.get("handleValid") is not True:
                errors.append("an Active-internal Effect event keeps the Handle valid")
        else:
            allowed = _EFFECT_TRANSITIONS.get(source, set()) if isinstance(source, str) else set()
            if not isinstance(dest, str) or dest not in allowed:
                errors.append("illegal Effect transition {} -> {}".format(source, dest))
            elif not isinstance(event, str) or event not in _EFFECT_TRANSITION_EVENTS.get((source, dest), set()):
                errors.append("event {} is not valid for Effect transition {} -> {}".format(event, source, dest))
            if isinstance(dest, str) and dest in _EFFECT_TERMINAL and value.get("handleValid") is not False:
                errors.append("a terminal Effect state invalidates the Handle")
            if isinstance(dest, str) and dest not in _EFFECT_TERMINAL and "handleValid" in value and value.get("handleValid") is not True:
                errors.append("a non-terminal Effect state keeps the Handle valid")
            if dest == "RolledBack" and value.get("predicted") is not True:
                errors.append("Effect RolledBack is only valid for predicted instances")
    else:
        errors.append("machine must be Ability or Effect")
    return errors


def _gas_admission_errors(value: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    if value.get("kind") != "Admission":
        errors.append("Admission records must declare kind=Admission")
    if value.get("machine") != "Ability":
        errors.append("Admission records apply only to Ability")
    if value.get("fromState") != "Requested":
        errors.append("Admission must start in Requested")
    if value.get("event") != "Activate":
        errors.append("Admission event must be Activate")
    order = value.get("admissionOrder")
    if not isinstance(order, list) or tuple(order) != _GAS_ADMISSION_ORDER:
        errors.append("admission order must be HandlePermission, Cooldown, Cost, Tag, GameCustom")
    checks_value = value.get("checks")
    checks = checks_value if isinstance(checks_value, list) else []
    if checks_value is not None and not isinstance(checks_value, list):
        errors.append("admission checks must be an array")
    names = [_gas_check_name(item) for item in checks]
    errors.extend(_gas_check_order_errors(checks, "admission"))
    if tuple(names) != _GAS_ADMISSION_ORDER[:len(names)]:
        errors.append("admission checks must follow the five-step order")
    passed = [_gas_check_passed(item) for item in checks]
    if any(item is None for item in passed):
        errors.append("every admission check must declare Pass or Fail")
    first_failure = next((index for index, item in enumerate(passed) if item is False), None)
    if first_failure is None and len(checks) != len(_GAS_ADMISSION_ORDER):
        errors.append("a successful admission must pass all five checks")
    if first_failure is not None and first_failure != len(checks) - 1:
        errors.append("admission stops at the first failed check")
    expected_outcome = "Rejected" if first_failure is not None else "Activated"
    if value.get("outcome") != expected_outcome or value.get("toState") != expected_outcome:
        errors.append("admission outcome and toState must follow the first failed check")
    failure_step = value.get("failureStep")
    if first_failure is None and failure_step is not None:
        errors.append("a successful admission cannot declare a failure step")
    if first_failure is None and value.get("failureReason") is not None:
        errors.append("a successful admission cannot declare a failure reason")
    elif first_failure is not None and failure_step != _GAS_ADMISSION_ORDER[first_failure]:
        errors.append("failureStep must name the first failed admission check")
    if first_failure is not None and not isinstance(value.get("failureReason"), str):
        errors.append("a rejected admission must declare a failure reason")
    expected_handle = expected_outcome == "Activated"
    if value.get("handleValid") is not expected_handle:
        errors.append("Rejected admission invalidates the Handle and Activated keeps it valid")
    if value.get("chargeCount", 0) != 0 or value.get("charged") is True:
        errors.append("admission does not charge cost; charging occurs at Commit")
    if value.get("phase") not in (None, "Admission"):
        errors.append("Admission phase must be Admission")
    if value.get("outcome") not in ("Activated", "Rejected"):
        errors.append("Admission outcome must be Activated or Rejected")
    return errors


def _gas_commit_errors(value: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    if value.get("kind") != "Commit":
        errors.append("Commit records must declare kind=Commit")
    if value.get("machine") != "Ability":
        errors.append("Commit records apply only to Ability")
    if value.get("fromState") != "Executing":
        errors.append("Commit decision must occur from Executing")
    if value.get("event") != "Commit":
        errors.append("Commit event must be Commit")
    order = value.get("recheckOrder")
    if not isinstance(order, list) or tuple(order) != _GAS_COMMIT_ORDER:
        errors.append("Commit rechecks only Cooldown then Cost")
    checks_value = value.get("checks")
    checks = checks_value if isinstance(checks_value, list) else []
    if checks_value is not None and not isinstance(checks_value, list):
        errors.append("Commit checks must be an array")
    names = [_gas_check_name(item) for item in checks]
    errors.extend(_gas_check_order_errors(checks, "Commit"))
    if tuple(names) != _GAS_COMMIT_ORDER[:len(names)]:
        errors.append("Commit checks must cover only Cooldown then Cost")
    passed = [_gas_check_passed(item) for item in checks]
    if any(item is None for item in passed):
        errors.append("every Commit check must declare Pass or Fail")
    failed = next((index for index, item in enumerate(passed) if item is False), None)
    if failed is None and len(checks) != len(_GAS_COMMIT_ORDER):
        errors.append("a successful Commit must pass both rechecks")
    if failed is not None and failed != len(checks) - 1:
        errors.append("Commit stops at the first failed recheck")
    expected_outcome = "Cancelled" if failed is not None else "Executing"
    if value.get("outcome") != expected_outcome or value.get("toState") != expected_outcome:
        errors.append("Commit failure is Cancelled; a successful Commit remains Executing")
    if failed is None and value.get("failureStep") is not None:
        errors.append("a successful Commit cannot declare a failure step")
    elif failed is not None and value.get("failureStep") != _GAS_COMMIT_ORDER[failed]:
        errors.append("failureStep must name the failed Commit check")
    if value.get("handleValid") is not (expected_outcome == "Executing"):
        errors.append("Cancelled Commit invalidates the Handle")
    charge_count = value.get("chargeCount", 0)
    if isinstance(charge_count, bool) or not isinstance(charge_count, int):
        if charge_count is not None:
            errors.append("chargeCount must be an integer")
        charge_count = 0
    if charge_count > 1:
        errors.append("Commit cannot charge more than once")
    if charge_count < 0:
        errors.append("chargeCount cannot be negative")
    if isinstance(value.get("charged"), bool) and value.get("charged") != (charge_count == 1):
        errors.append("charged must agree with chargeCount")
    if failed is not None and (charge_count != 0 or value.get("charged") is True):
        errors.append("a failed Commit cannot charge or retain a partial cost write")
    prepared = value.get("prepared") is True or value.get("phase") in ("Prepared", "CommitIntent")
    commit_intent = value.get("commitIntent") is True or value.get("phase") == "CommitIntent"
    phase = value.get("phase")
    if phase not in (None, "Prepared", "CommitDecision", "Commit", "CommitIntent"):
        errors.append("Commit phase is not recognized")
    if phase == "Prepared" and value.get("commitIntent") is True:
        errors.append("Prepared phase cannot already carry CommitIntent")
    if phase == "CommitIntent" and value.get("commitIntent") is not True:
        errors.append("CommitIntent phase requires commitIntent=true")
    if phase in ("Prepared", "CommitIntent") and value.get("prepared") is not True:
        errors.append("Prepared and CommitIntent phases require prepared=true")
    if not prepared:
        errors.append("Commit decision requires a Prepared record")
    if commit_intent and not prepared:
        errors.append("CommitIntent requires Prepared")
    if prepared or commit_intent:
        if value.get("businessRejected") is True:
            errors.append("Prepared/CommitIntent forbids later business rejection")
    return errors


def _gas_evaluation_errors(value: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    formula = "(Base + SigmaAdd) * (1 + SigmaPercent)"
    if value.get("formula") != formula:
        errors.append("evaluation formula must be (Base + SigmaAdd) * (1 + SigmaPercent)")
    if value.get("operators") != ["Add", "Percent", "Override"]:
        errors.append("V1 evaluation operators must be Add, Percent and Override")
    if value.get("percentageAggregation") != "Additive":
        errors.append("percentage modifiers aggregate additively")
    if value.get("overrideTieBreak") != "PriorityDescendingThenSequenceDescending":
        errors.append("override tie-break must be explicit priority then sequence descending")
    ids: set = set()
    sequences: set = set()
    add = 0.0
    percent = 0.0
    candidates: List[Tuple[int, int, float, str]] = []
    modifiers_value = value.get("modifiers")
    modifiers = modifiers_value if isinstance(modifiers_value, list) else []
    if modifiers_value is not None and not isinstance(modifiers_value, list):
        errors.append("modifiers must be an array")
    for item in modifiers:
        if not isinstance(item, dict):
            errors.append("modifier must be an object")
            continue
        ident = item.get("id")
        ident_key = ident if isinstance(ident, (str, int, float, bool, type(None))) else canonical_json(ident)
        if ident_key in ids:
            errors.append("modifier ids must be unique")
        ids.add(ident_key)
        op = item.get("operator")
        number = item.get("value")
        priority = item.get("priority")
        sequence = item.get("sequence")
        if not _gas_finite_number(number):
            errors.append("modifier values must be finite numbers")
            continue
        if isinstance(sequence, int) and not isinstance(sequence, bool):
            if sequence in sequences:
                errors.append("modifier sequence values must be unique")
            sequences.add(sequence)
        if op == "Add":
            add += float(number)
        elif op == "Percent":
            percent += float(number)
        elif op == "Override":
            if isinstance(priority, int) and isinstance(sequence, int):
                candidates.append((priority, sequence, float(number), str(ident)))
        else:
            errors.append("unsupported evaluation operator {}".format(op))
    overrides_value = value.get("overrides")
    overrides = overrides_value if isinstance(overrides_value, list) else []
    if overrides_value is not None and not isinstance(overrides_value, list):
        errors.append("overrides must be an array")
    for item in overrides:
        if not isinstance(item, dict):
            errors.append("override must be an object")
            continue
        ident = item.get("id")
        ident_key = ident if isinstance(ident, (str, int, float, bool, type(None))) else canonical_json(ident)
        if ident_key in ids:
            errors.append("modifier and override ids must be unique")
        ids.add(ident_key)
        number = item.get("value")
        priority = item.get("priority")
        sequence = item.get("sequence")
        if not _gas_finite_number(number):
            errors.append("override values must be finite numbers")
            continue
        if isinstance(priority, int) and isinstance(sequence, int):
            candidates.append((priority, sequence, float(number), str(ident)))
    keys = [(item[0], item[1]) for item in candidates]
    if len(keys) != len(set(keys)):
        errors.append("override priority and sequence must identify one deterministic winner")
    base = value.get("base")
    if not _gas_finite_number(base):
        errors.append("base must be a finite number")
        base = 0.0
    computed = (float(base) + add) * (1.0 + percent)
    if candidates:
        computed = max(candidates, key=lambda item: (item[0], item[1]))[2]
    if not math.isfinite(computed):
        errors.append("evaluation result must remain finite")
    result = value.get("result")
    if not _gas_finite_number(result):
        errors.append("result must be a finite number")
    elif float(result) != computed:
        errors.append("result does not match the frozen evaluation formula")
    return errors


def _gas_effect_event_errors(value: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    tick = value.get("tickId")
    events_value = value.get("events")
    events = events_value if isinstance(events_value, list) else []
    if events_value is not None and not isinstance(events_value, list):
        errors.append("Effect events must be an array")
    names: List[str] = []
    orders: List[int] = []
    current_state = value.get("initialState")
    for item in events:
        if not isinstance(item, dict):
            errors.append("Effect events must be objects")
            continue
        name = item.get("event")
        name = name if isinstance(name, str) else str(name)
        names.append(name)
        order = item.get("order")
        orders.append(order)
        if item.get("tickId") != tick:
            errors.append("every Effect event must use the enclosing Tick")
        if not isinstance(order, int) or isinstance(order, bool):
            errors.append("Effect event order must be an integer")
        if name not in _GAS_EFFECT_EVENT_RANK:
            errors.append("unknown Effect event {}".format(name))
        if name == "Suppress":
            if item.get("fromState") != "Active" or item.get("toState") != "Active":
                errors.append("Suppress is an Active-internal event and cannot change state")
        elif name in {"Stack", "Refresh", "Duration", "Period", "Hit", "Overflow", "SnapshotReplacement"}:
            if item.get("fromState") != "Active" or item.get("toState") != "Active":
                errors.append("{} is an Active-internal event".format(name))
        elif name == "Apply":
            if item.get("fromState") != "Pending" or item.get("toState") != "Active":
                errors.append("Apply must transition Pending to Active")
        elif name == "Removal":
            if item.get("fromState") != "Active" or item.get("toState") != "Removed":
                errors.append("Removal must transition Active to Removed")
        elif name == "Expire":
            if item.get("fromState") != "Active" or item.get("toState") != "Expired":
                errors.append("Expire must transition Active to Expired")
        if isinstance(current_state, str) and current_state in _EFFECT_TERMINAL:
            errors.append("no Effect event may follow a terminal state")
        elif name == "Apply":
            if current_state != "Pending":
                errors.append("Apply requires the current Effect state to be Pending")
            current_state = "Active"
        elif name in {"Hit", "Overflow", "SnapshotReplacement", "Stack", "Refresh", "Suppress", "Duration", "Period"}:
            if current_state != "Active":
                errors.append("{} requires the current Effect state to be Active".format(name))
        elif name == "Removal":
            if current_state != "Active":
                errors.append("Removal requires the current Effect state to be Active")
            current_state = "Removed"
        elif name == "Expire":
            if current_state != "Active":
                errors.append("Expire requires the current Effect state to be Active")
            current_state = "Expired"
    if orders != list(range(1, len(events) + 1)):
        errors.append("Effect event order must be contiguous and deterministic")
    if value.get("eventOrder") is not None and value.get("eventOrder") != names:
        errors.append("eventOrder must equal the ordered event entries")
    non_apply = [name for name in names if name != "Apply"]
    ranks = [_GAS_EFFECT_EVENT_RANK.get(name, 99) for name in non_apply]
    if any(left > right for left, right in zip(ranks, ranks[1:])):
        errors.append("Effect events must follow Hit -> Overflow -> SnapshotReplacement/Stack -> Duration -> Period -> Removal")
    if "Apply" in names:
        if value.get("initialState") != "Pending" or names[0] != "Apply":
            errors.append("Apply must be first and begin from Pending")
    elif value.get("initialState") != "Active":
        errors.append("an Effect tick without Apply must begin Active")
    terminals = [name for name in names if name in ("Removal", "Expire")]
    if terminals and names[-1] not in ("Removal", "Expire"):
        errors.append("Removal/Expire must be the final same-tick event")
    final = value.get("finalState")
    expected_final = "Active"
    if names and names[-1] == "Removal":
        expected_final = "Removed"
    elif names and names[-1] == "Expire":
        expected_final = "Expired"
    if final != expected_final:
        errors.append("finalState must match the terminal Effect event")
    if isinstance(current_state, str) and final != current_state:
        errors.append("finalState must match the Effect state after applying events")
    terminal = isinstance(final, str) and final in _EFFECT_TERMINAL
    if terminal and value.get("handleValid") is not False:
        errors.append("terminal Effect states invalidate the Handle")
    elif not terminal and "handleValid" in value and value.get("handleValid") is not True:
        errors.append("an Active Effect keeps the Handle valid")
    has_apply = "Apply" in names
    has_remove = "Removal" in names
    if has_apply and has_remove:
        if value.get("sameTickOutcome") != "Cancelled":
            errors.append("application plus removal in one Tick must be Cancelled")
    elif value.get("sameTickOutcome") == "Cancelled":
        errors.append("Cancelled same-tick outcome requires Apply and Removal")
    elif names and names[-1] == "Removal" and value.get("sameTickOutcome") not in (None, "Removed"):
        errors.append("a terminal Removal event must report Removed outcome")
    elif names and names[-1] == "Expire" and value.get("sameTickOutcome") not in (None, "Expired"):
        errors.append("a terminal Expire event must report Expired outcome")
    for key in value:
        lowered = str(key).lower()
        if any(token in lowered for token in ("second", "millisecond", "wallclock", "timestamp", "datetime")) or lowered == "time":
            errors.append("Effect timing fields must be Tick/frame numbers, not wall-clock values")
    if isinstance(tick, int) and not isinstance(tick, bool) and isinstance(value.get("durationTicks"), int) and isinstance(value.get("expiresAtTick"), int):
        if value.get("expiresAtTick") < tick + value.get("durationTicks"):
            errors.append("expiresAtTick must not precede durationTicks from the current Tick")
    return errors


def _gas_registry_values(namespace: str) -> List[Dict[str, Any]]:
    """Read one permanent namespace from the architecture ID Registry."""
    try:
        registry = load_json(ID_REGISTRY_FILE)
    except ContractError:
        return []
    for item in registry.get("namespaces", []) if isinstance(registry, dict) else []:
        if isinstance(item, dict) and item.get("namespace") == namespace:
            values = item.get("values")
            return values if isinstance(values, list) else []
    return []


def _gas_component_entries(value: Dict[str, Any]) -> Tuple[List[Dict[str, Any]], Dict[str, List[Dict[str, Any]]]]:
    """Flatten component rows while retaining their declared container."""
    all_entries: List[Dict[str, Any]] = []
    by_component: Dict[str, List[Dict[str, Any]]] = {}
    components = value.get("components") if isinstance(value, dict) else None
    if not isinstance(components, list):
        return all_entries, by_component
    for container in components:
        if not isinstance(container, dict):
            continue
        name = str(container.get("component", ""))
        rows = container.get("entries")
        if not isinstance(rows, list):
            continue
        bucket = by_component.setdefault(name, [])
        for row in rows:
            if isinstance(row, dict):
                row_copy = dict(row)
                row_copy["_component"] = name
                bucket.append(row_copy)
                all_entries.append(row_copy)
    return all_entries, by_component


def _gas_components_errors(value: Dict[str, Any]) -> List[str]:
    """Validate the four ECS containers and world/index/generation identity."""
    errors: List[str] = []
    if not isinstance(value, dict):
        return ["GAS component record must be an object"]
    if value.get("kind") != "Components":
        errors.append("GAS component record kind must be Components")
    components = value.get("components")
    names = [item.get("component") for item in components if isinstance(item, dict)] if isinstance(components, list) else []
    normalized_names = [str(name) for name in names]
    if tuple(sorted(normalized_names)) != tuple(sorted(_GAS_COMPONENT_NAMES)):
        errors.append("GAS exposes exactly AbilityComponent, EffectComponent, AttributeComponent and TagComponent")
    if len(normalized_names) != len(set(normalized_names)):
        errors.append("GAS component containers must be unique")
    unknown = sorted(set(normalized_names) - set(_GAS_COMPONENT_NAMES))
    if unknown:
        errors.append("unknown GAS component container(s): {}".format(", ".join(unknown)))

    # FxComponent is intentionally not a tolerated extension, even when a
    # structural validator reports the oneOf failure first. `fx_key` is
    # permitted for one entry level only, never on the container itself.
    def scan(node: Any, context: str = "other") -> None:
        if isinstance(node, dict):
            if node.get("component") == "FxComponent" or "FxComponent" in node:
                errors.append("FxComponent is forbidden; use EffectComponent.fx_key")
            if "fx_key" in node and context != "effect_entry":
                errors.append("fx_key is allowed only inside an EffectComponent entry")
            if any(
                token in str(key).lower()
                for key in node
                for token in ("wallclock", "timestamp", "datetime", "millisecond", "second")
            ):
                errors.append("GAS component timing fields must use Tick numbers, not wall-clock values")
            if node.get("component") == "EffectComponent":
                for key, child in node.items():
                    scan(child, "effect_entries" if key == "entries" else "other")
            elif context == "effect_entries":
                for child in node.values():
                    scan(child, "effect_entry")
            else:
                for child in node.values():
                    scan(child, "other")
        elif isinstance(node, list):
            child_context = "effect_entry" if context == "effect_entries" else context
            for child in node:
                scan(child, child_context)

    scan(value)
    # Keep one stable occurrence for the common invalid-fixture case.
    errors = list(dict.fromkeys(errors))

    entries, _by_component = _gas_component_entries(value)
    world_id = value.get("worldId")
    # Components attached to one ECS entity may share its index+generation
    # Handle. Conflicting generations or validity at one index are the error.
    handles: Dict[int, Tuple[Any, Any]] = {}
    row_slots: set = set()
    instance_ids: set = set()
    for entry in entries:
        component = entry.get("_component")
        instance_id = str(entry.get("instanceId"))
        if instance_id in instance_ids:
            errors.append("GAS instanceId values must be unique across component rows")
        instance_ids.add(instance_id)
        row = entry.get("row")
        handle = entry.get("handle")
        if not isinstance(row, dict) or not isinstance(handle, dict):
            continue
        row_index = row.get("index")
        handle_index = handle.get("index")
        if isinstance(row_index, int) and not isinstance(row_index, bool):
            slot = (component, row_index)
            if slot in row_slots:
                errors.append("an ECS component cannot contain duplicate row indexes")
            row_slots.add(slot)
        if row_index != handle_index:
            errors.append("Handle index must equal its ECS row index")
        if handle.get("worldId") != world_id:
            errors.append("Handle worldId must equal the enclosing worldId")
        if "handleValid" in entry and entry.get("handleValid") != handle.get("valid"):
            errors.append("handleValid must agree with Handle.valid")
        if isinstance(handle_index, int) and not isinstance(handle_index, bool) and isinstance(handle.get("generation"), int) and not isinstance(handle.get("generation"), bool):
            key = handle_index
            identity = (handle.get("generation"), handle.get("valid"))
            if key in handles and handles[key] != identity:
                errors.append("world-bound Handle index must have one current generation and validity")
            handles[key] = identity
        state = entry.get("state")
        if component == "AbilityComponent" and isinstance(state, str):
            if state in _ABILITY_TERMINAL and handle.get("valid") is not False:
                errors.append("terminal Ability rows invalidate their Handle")
            elif state not in _ABILITY_TERMINAL and handle.get("valid") is not True:
                errors.append("non-terminal Ability rows keep their Handle valid")
        if component == "EffectComponent" and isinstance(state, str):
            if state in _EFFECT_TERMINAL and handle.get("valid") is not False:
                errors.append("terminal Effect rows invalidate their Handle")
            elif state not in _EFFECT_TERMINAL and handle.get("valid") is not True:
                errors.append("non-terminal Effect rows keep their Handle valid")

    probes = value.get("handleProbes")
    seen_reasons: set = set()
    if isinstance(probes, list):
        for probe in probes:
            if not isinstance(probe, dict):
                continue
            handle = probe.get("handle")
            if not isinstance(handle, dict):
                continue
            actual = "Accepted"
            reason: Optional[str] = None
            if handle.get("worldId") != world_id:
                actual, reason = "Rejected", "CrossWorld"
            else:
                index = handle.get("index")
                generation = handle.get("generation")
                current_identity = handles.get(index) if isinstance(index, int) and not isinstance(index, bool) else None
                if current_identity is None:
                    actual, reason = "Rejected", "MissingRow"
                else:
                    current_generation, current_valid = current_identity
                    if generation != current_generation:
                        actual, reason = "Rejected", "StaleGeneration"
                    elif current_valid is not True:
                        actual, reason = "Rejected", "Terminal"
            if probe.get("expected") != actual:
                errors.append("Handle probe expected {} but resolves as {}".format(probe.get("expected"), actual))
            if actual == "Rejected" and probe.get("reason") != reason:
                errors.append("Handle rejection reason must be {}".format(reason))
            if actual == "Accepted" and "reason" in probe:
                errors.append("an accepted Handle probe cannot carry a rejection reason")
            if probe.get("expected") == "Accepted" and handle.get("valid") is not True:
                errors.append("an accepted Handle probe must declare valid=true")
            if probe.get("expected") == "Rejected" and handle.get("valid") is not False:
                errors.append("a rejected Handle probe must declare valid=false")
            if reason:
                seen_reasons.add(reason)
    required_probe_reasons = {"CrossWorld", "StaleGeneration"}
    if isinstance(probes, list) and not required_probe_reasons <= seen_reasons:
        errors.append("Handle probes must cover stale-generation and cross-world rejection")
    return list(dict.fromkeys(errors))


def _gas_tag_table_hash(entries: List[Dict[str, Any]]) -> str:
    table = {
        "namespace": "Tag",
        "entries": [
            {
                "tagId": item.get("tagId") if isinstance(item, dict) else None,
                "numeric": item.get("numeric") if isinstance(item, dict) else None,
                "status": item.get("status") if isinstance(item, dict) else None,
                "since": item.get("since") if isinstance(item, dict) else None,
            }
            for item in entries
        ],
    }
    return hashlib.sha256(canonical_json(table).encode("ascii")).hexdigest()


def _gas_tag_schema_hash() -> str:
    try:
        schema = load_json(SCHEMA_DIR / "gas-tag.schema.json")
    except ContractError:
        return ""
    return hashlib.sha256(canonical_json(schema).encode("ascii")).hexdigest()


def _gas_tag_is_descendant(candidate: str, ancestor: str) -> bool:
    return candidate.startswith(ancestor + ".")


def _gas_tag_errors(value: Dict[str, Any]) -> List[str]:
    """Validate counted Tag state, hierarchy queries and the pre-world handshake."""
    errors: List[str] = []
    if not isinstance(value, dict):
        return ["GAS Tag record must be an object"]
    registry = value.get("registry") if isinstance(value.get("registry"), dict) else {}
    entries = registry.get("entries") if isinstance(registry.get("entries"), list) else []
    registry_ids = [str(item.get("tagId")) for item in entries if isinstance(item, dict)]
    registry_numerics = [canonical_json(item.get("numeric")) for item in entries if isinstance(item, dict)]
    if len(registry_ids) != len(set(registry_ids)):
        errors.append("Tag registry identifiers must be unique")
    if len(registry_numerics) != len(set(registry_numerics)):
        errors.append("Tag registry numeric identifiers must be unique")
    permanent = _gas_registry_values("Tag")
    expected_permanent = [
        {
            "tagId": item.get("id"),
            "numeric": item.get("numeric"),
            "status": item.get("status"),
            "since": item.get("since"),
        }
        for item in permanent
        if isinstance(item, dict)
    ]
    declared = [
        {
            "tagId": item.get("tagId"),
            "numeric": item.get("numeric"),
            "status": item.get("status"),
            "since": item.get("since"),
        }
        for item in entries
        if isinstance(item, dict)
    ]
    if declared != expected_permanent:
        errors.append("Tag registry must equal the complete permanent ids/index.json#Tag table")
    table_hash = _gas_tag_table_hash(entries)
    schema_hash = _gas_tag_schema_hash()
    if registry.get("tableHash") != table_hash:
        errors.append("Tag tableHash does not cover the complete canonical table")
    if registry.get("schemaHash") != schema_hash:
        errors.append("Tag schemaHash does not match the gas-tag schema handshake descriptor")

    counts_value = value.get("counts")
    counts = counts_value if isinstance(counts_value, list) else []
    count_ids = [str(item.get("tagId")) for item in counts if isinstance(item, dict)]
    if len(count_ids) != len(set(count_ids)):
        errors.append("Tag counts must contain each tag at most once")
    known_ids = set(registry_ids)
    status_by_id = {}
    for item in entries:
        if isinstance(item, dict):
            status_by_id[str(item.get("tagId"))] = item.get("status")
    active_counts = set()
    for item in counts:
        if not isinstance(item, dict):
            continue
        tag_id = str(item.get("tagId"))
        count = item.get("count")
        if tag_id not in known_ids:
            errors.append("Tag count references an unregistered identifier")
        if status_by_id.get(tag_id) != "Active":
            errors.append("Tag counts may reference only Active identifiers")
        if isinstance(count, int) and not isinstance(count, bool) and count > 0:
            active_counts.add(tag_id)

    query_modes: set = set()
    queries_value = value.get("queries")
    queries = queries_value if isinstance(queries_value, list) else []
    for query in queries:
        if not isinstance(query, dict):
            continue
        mode = query.get("mode")
        tag_id = str(query.get("tagId"))
        query_modes.add(str(mode))
        if tag_id not in known_ids:
            errors.append("Tag query references an unregistered identifier")
            continue
        if mode == "Exact":
            matches = sorted(item for item in active_counts if item == tag_id)
        elif mode == "Parent":
            matches = sorted(item for item in active_counts if _gas_tag_is_descendant(item, str(tag_id)))
        elif mode == "Child":
            matches = sorted(item for item in active_counts if _gas_tag_is_descendant(str(tag_id), item))
        else:
            matches = []
        if query.get("matches") != matches:
            errors.append("Tag {} query matches are not deterministic".format(mode))
        if query.get("expected") is not bool(matches):
            errors.append("Tag {} query expected must match its counted result".format(mode))
    if query_modes != set(_GAS_TAG_QUERY_MODES):
        errors.append("Tag fixtures must exercise Exact, Parent and Child matching")

    handshake = value.get("handshake") if isinstance(value.get("handshake"), dict) else {}
    table_mismatch = not (
        handshake.get("localTableHash") == table_hash
        and handshake.get("peerTableHash") == table_hash
        and handshake.get("localTableHash") == handshake.get("peerTableHash")
    )
    schema_mismatch = not (
        handshake.get("localSchemaHash") == schema_hash
        and handshake.get("peerSchemaHash") == schema_hash
        and handshake.get("localSchemaHash") == handshake.get("peerSchemaHash")
    )
    mismatch = table_mismatch or schema_mismatch
    expected_accepted = not mismatch
    if handshake.get("accepted") is not expected_accepted:
        errors.append("Tag handshake accepted must equal full-table/schema hash agreement")
    if mismatch:
        expected_reason = "TagTableHashMismatch" if table_mismatch else "TagSchemaHashMismatch"
        if handshake.get("failureReason") != expected_reason:
            errors.append("Tag handshake failureReason must be {}".format(expected_reason))
        if handshake.get("phase") in ("WorldReady", "Running"):
            errors.append("Tag hash mismatch must hard-fail before WorldReady or Running")
    elif "failureReason" in handshake:
        errors.append("an accepted Tag handshake cannot carry failureReason")
    return list(dict.fromkeys(errors))


def _gas_replication_hash(domain: str, fields: List[Dict[str, Any]], included: List[str]) -> str:
    by_id = {str(item.get("fieldId")): item for item in fields if isinstance(item, dict)}
    payload = {
        "domain": domain,
        "fields": [{"fieldId": field_id, "value": by_id[field_id].get("value")} for field_id in included if field_id in by_id],
    }
    return hashlib.sha256(canonical_json(payload).encode("ascii")).hexdigest()


def _gas_replication_errors(value: Dict[str, Any]) -> List[str]:
    """Validate field visibility declarations and the two hash projections."""
    errors: List[str] = []
    if not isinstance(value, dict):
        return ["GAS replication record must be an object"]
    if value.get("kind") != "ReplicationContract":
        errors.append("GAS replication record kind must be ReplicationContract")
    fields_value = value.get("fields")
    fields = fields_value if isinstance(fields_value, list) else []
    field_ids = [str(item.get("fieldId")) for item in fields if isinstance(item, dict)]
    if len(field_ids) != len(set(field_ids)):
        errors.append("replication fieldId values must be unique")
    declared_components = {
        str(item.get("component")) for item in fields if isinstance(item, dict)
    }
    if declared_components != set(_GAS_COMPONENT_NAMES):
        errors.append("replication declarations must cover all four GAS components")
    field_by_id = {str(item.get("fieldId")): item for item in fields if isinstance(item, dict)}
    all_ids = [str(item.get("fieldId")) for item in fields if isinstance(item, dict)]
    authoritative_ids: List[str] = []
    sync_ids: List[str] = []
    for item in fields:
        if not isinstance(item, dict):
            continue
        field_id = str(item.get("fieldId"))
        component = item.get("component")
        field_name = item.get("field")
        if field_id != "{}.{}".format(component, field_name):
            errors.append("fieldId must be component.field for deterministic projections")
        normalized_field = re.sub(r"[_-]", "", str(field_name)).lower()
        normalized_field_id = re.sub(r"[_-]", "", field_id).lower()
        if normalized_field == "modifierledger" or normalized_field_id == "effectcomponent.modifierledger":
            errors.append("Modifier ledger is a derived Effect view and cannot be a replicated or persisted field")
        hidden = item.get("hidden") is True
        public = item.get("thirdPartyPublic") is True
        sync = item.get("sync") is True
        predicted = item.get("predicted") is True
        presentation = item.get("presentation") is True
        persisted = item.get("persisted") is True
        if hidden and (public or sync):
            errors.append("hidden replication fields cannot be public or synchronized")
        if public and (hidden or not sync):
            errors.append("third-party-public fields must be visible and synchronized")
        if public and item.get("owner") != "ThirdParty":
            errors.append("third-party-public fields must declare owner=ThirdParty")
        if hidden and item.get("owner") not in ("Server", "None"):
            errors.append("hidden fields must be owned by Server or None")
        if presentation and persisted:
            errors.append("presentation fields are transient and cannot be persisted")
        if presentation and item.get("authority") == "Server":
            errors.append("presentation fields cannot claim Server authority")
        if predicted and item.get("authority") == "Server":
            errors.append("predicted values cannot claim Server authority")
        if item.get("authority") == "Client" and sync and not predicted:
            errors.append("non-predicted Client-authority fields cannot enter the confirmation sync domain")
        if item.get("authority") == "Server" and not presentation:
            authoritative_ids.append(field_id)
        if sync and not predicted and not presentation and not hidden:
            sync_ids.append(field_id)
    server = value.get("serverSnapshot") if isinstance(value.get("serverSnapshot"), dict) else {}
    client = value.get("clientConfirmation") if isinstance(value.get("clientConfirmation"), dict) else {}
    if server.get("domain") != "ServerSnapshot":
        errors.append("serverSnapshot domain must be ServerSnapshot")
    if client.get("domain") != "ClientConfirmation":
        errors.append("clientConfirmation domain must be ClientConfirmation")
    server_in = server.get("includedFields") if isinstance(server.get("includedFields"), list) else []
    server_out = server.get("excludedFields") if isinstance(server.get("excludedFields"), list) else []
    client_in = client.get("includedFields") if isinstance(client.get("includedFields"), list) else []
    client_out = client.get("excludedFields") if isinstance(client.get("excludedFields"), list) else []
    server_in_ids = [str(item) for item in server_in]
    server_out_ids = [str(item) for item in server_out]
    client_in_ids = [str(item) for item in client_in]
    client_out_ids = [str(item) for item in client_out]
    if server_in_ids != authoritative_ids:
        errors.append("server snapshot hash must include every authoritative field in declaration order")
    if client_in_ids != sync_ids:
        errors.append("client confirmation hash must include only the non-predicted sync domain")
    expected_out = [field_id for field_id in all_ids if field_id not in set(authoritative_ids)]
    expected_client_out = [field_id for field_id in all_ids if field_id not in set(sync_ids)]
    if server_out_ids != expected_out:
        errors.append("server snapshot excludedFields must be the exact complement of authoritative fields")
    if client_out_ids != expected_client_out:
        errors.append("client confirmation excludedFields must be the exact complement of the sync domain")
    if set(server_in_ids) & set(server_out_ids) or set(client_in_ids) & set(client_out_ids):
        errors.append("hash inclusion and exclusion sets must be disjoint")
    if server.get("hash") != _gas_replication_hash("ServerSnapshot", fields, server_in_ids):
        errors.append("server snapshot hash does not match its authoritative field preimage")
    if client.get("hash") != _gas_replication_hash("ClientConfirmation", fields, client_in_ids):
        errors.append("client confirmation hash does not match its sync-domain preimage")
    # A field excluded from both domains is intentional only for prediction or
    # presentation; this closes accidental silent omission of authority data.
    for field_id in all_ids:
        item = field_by_id[field_id]
        if field_id in set(server_out_ids) and item.get("authority") == "Server" and item.get("presentation") is not True:
            errors.append("authoritative field {} cannot be omitted from server snapshot hash".format(field_id))
    return list(dict.fromkeys(errors))


def _gas_prediction_errors(value: Dict[str, Any]) -> List[str]:
    """Validate frame-keyed prediction rejection and deterministic replay."""
    errors: List[str] = []
    if not isinstance(value, dict):
        return ["GAS prediction record must be an object"]
    if value.get("kind") != "PredictionRollback":
        errors.append("GAS prediction record kind must be PredictionRollback")
    if value.get("inputFrame") != value.get("predictionKey"):
        errors.append("predictionKey must equal the input frame")
    if value.get("windowBoundary") != "GasAndEventFinalize":
        errors.append("prediction window boundary must be GasAndEventFinalize")
    actions_value = value.get("nonPredictableActions")
    actions = actions_value if isinstance(actions_value, list) else []
    names = [str(item.get("action")) for item in actions if isinstance(item, dict)]
    if tuple(sorted(names)) != tuple(sorted(_GAS_NON_PREDICTABLE_ACTIONS)):
        errors.append("non-predictable actions must be EffectRemoval, EffectPeriod and OutOfSimulation")
    for item in actions:
        if isinstance(item, dict) and item.get("predicted") is not False:
            errors.append("non-predictable actions must declare predicted=false")
    rollback = value.get("rollback") if isinstance(value.get("rollback"), dict) else {}
    if rollback.get("unit") != "EcsGasVoxelFrame" or rollback.get("frameCount") != 1:
        errors.append("prediction rejection rolls back exactly one ECS/GAS/Voxel frame")
    if rollback.get("clientRolledBack") is not True:
        errors.append("client prediction rejection must roll back the client frame")
    if rollback.get("serverRolledBack") is not False:
        errors.append("server prediction authority must never roll back")
    if tuple(rollback.get("steps") or ()) != _GAS_PREDICTION_ROLLBACK_STEPS:
        errors.append("rollback steps must restore, apply authority, then replay inputs")
    confirmed = rollback.get("confirmedFrame")
    input_frame = value.get("inputFrame")
    if isinstance(confirmed, int) and isinstance(input_frame, int) and confirmed >= input_frame:
        errors.append("confirmed frame must precede the rejected input frame")
    replay = rollback.get("replayInputFrames")
    if isinstance(replay, list):
        valid_replay_numbers = all(isinstance(frame, int) and not isinstance(frame, bool) for frame in replay)
        if valid_replay_numbers and (replay != sorted(replay) or len(replay) != len(set(replay))):
            errors.append("replay input frames must be strictly ascending and deterministic")
        if not valid_replay_numbers:
            errors.append("replay input frames must be integers")
        if isinstance(confirmed, int) and any(isinstance(frame, int) and not isinstance(frame, bool) and frame <= confirmed for frame in replay):
            errors.append("replay input frames must be after the confirmed frame")
        if isinstance(input_frame, int) and not isinstance(input_frame, bool) and any(
            isinstance(frame, int) and not isinstance(frame, bool) and frame <= input_frame for frame in replay
        ):
            errors.append("replay input frames must be strictly greater than the rejected input frame")
        if input_frame in replay:
            errors.append("the rejected input frame cannot be replayed as an accepted command")
    if rollback.get("deterministicReplay") is not True:
        errors.append("prediction replay must declare deterministicReplay=true")

    # Guard the closed conceptual vocabulary even when a future schema edit
    # accidentally makes an open object member available.
    forbidden_tokens = ("task", "predictionwindow", "wallclock", "timestamp", "datetime", "millisecond", "second")
    def scan(node: Any) -> None:
        if isinstance(node, dict):
            for key, child in node.items():
                lowered = str(key).lower()
                if any(token in lowered for token in forbidden_tokens):
                    errors.append("prediction contract must not introduce Task, PredictionWindow or wall-clock fields")
                scan(child)
        elif isinstance(node, list):
            for child in node:
                scan(child)
    scan(value)
    return list(dict.fromkeys(errors))


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
        # ADR-047 section 4: `checksum` covers the header itself, in the B-profile
        # domain. Documented as one sentence before, so nothing could fail it.
        expected_checksum = _abi.snapshot_checksum(value)
        if value.get("checksum") != expected_checksum:
            errors.append(
                "snapshot checksum does not match the SnapshotHeaderV1 digest of the header"
                " (expected {})".format(expected_checksum)
            )
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

    elif schema_id == "lumio-bin-profile":
        if value.get("binaryForm") != _LUMIO_BIN_FORM:
            errors.append("binaryForm must equal the ADR-047 frozen LumioBinV1 parameters")
        if value.get("digestAlgorithm") != _LUMIO_BIN_DIGEST_ALGORITHM:
            errors.append("digestAlgorithm must equal the ADR-047 frozen construction")
        if value.get("valueEncoding") != _LUMIO_BIN_VALUE_ENCODING:
            errors.append("valueEncoding must equal the ADR-047 frozen vector spelling")
        if value.get("vectorSemantics") != _LUMIO_BIN_VECTOR_SEMANTICS:
            errors.append("vectorSemantics must declare `error` normative and `case` a label")
        goldens = value.get("goldens", [])
        golden_ids = [item.get("id") for item in goldens]
        if len(golden_ids) != len(set(golden_ids)):
            errors.append("golden ids must be unique")
        covered = {item.get("case") for item in goldens}
        if covered != _LUMIO_BIN_GOLDEN_CASES:
            errors.append(
                "goldens must cover every frozen case (missing: {}, unregistered: {})".format(
                    sorted(_LUMIO_BIN_GOLDEN_CASES - covered), sorted(covered - _LUMIO_BIN_GOLDEN_CASES)
                )
            )
        for golden in goldens:
            try:
                payload = _abi.lumio_bin_encode(golden.get("layout"), golden.get("value"))
            except _abi.LumioBinError as exc:
                errors.append("golden {} does not encode: {}".format(golden.get("id"), exc))
                continue
            if golden.get("bytesHex") != payload.hex():
                errors.append(
                    "golden {} bytesHex does not re-encode from its layout and value".format(
                        golden.get("id")
                    )
                )
            digest = hashlib.sha256(payload).hexdigest()
            if golden.get("sha256") != digest:
                errors.append("golden {} sha256 must equal {}".format(golden.get("id"), digest))
        rejections = value.get("rejections", [])
        rejection_ids = [item.get("id") for item in rejections]
        if len(rejection_ids) != len(set(rejection_ids)):
            errors.append("rejection ids must be unique")
        rejection_cases = {item.get("case") for item in rejections}
        if not rejection_cases.issubset(_LUMIO_BIN_REJECTION_CASES):
            errors.append(
                "rejections carry unregistered cases: {}".format(
                    sorted(rejection_cases - _LUMIO_BIN_REJECTION_CASES)
                )
            )
        for rejection in rejections:
            # A rejection vector earns its place only by actually being refused,
            # and for the published reason: "fails somehow" is not a contract.
            try:
                _abi.lumio_bin_encode(rejection.get("layout"), rejection.get("value"))
            except _abi.LumioBinError as exc:
                if exc.code != rejection.get("error"):
                    errors.append(
                        "rejection {} declares error {} but the encoder raised {}".format(
                            rejection.get("id"), rejection.get("error"), exc.code
                        )
                    )
                continue
            errors.append("rejection {} was accepted by the encoder".format(rejection.get("id")))

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
        # ADR-048: the gate is executed, not asserted about. The published
        # validator computes the verdict from the record and the record must
        # agree with it — which is what makes ADR-022's "generated validator"
        # a thing a fixture can fail rather than a catalog of field names.
        verdict, reason = _abi.evaluate_protocol_gate(value, _message_type_ids())
        declared_verdict = value.get("verdict")
        declared_reason = value.get("rejectReason")
        if declared_reason in _abi.GATE_DECLARED_ONLY_REASONS:
            # Session-scope anti-replay is owned by `ClientReplicaSession`, so the
            # gate cannot see it. Such a reject is legitimate only when every
            # check the gate *can* run passes — otherwise the record is hiding a
            # derivable failure behind a reason nothing can verify.
            if declared_verdict != "Reject":
                errors.append("{} is a rejection reason and requires a Reject verdict".format(declared_reason))
            elif verdict != "Accept":
                errors.append(
                    "this record fails the generated gate with {}; it cannot be reported as {}".format(
                        reason, declared_reason
                    )
                )
        elif declared_verdict != verdict:
            errors.append(
                "the generated gate evaluates this record as {}{}, not {}".format(
                    verdict, " ({})".format(reason) if reason else "", declared_verdict
                )
            )
        elif verdict == "Reject" and declared_reason != reason:
            errors.append(
                "the generated gate rejects this record with {}, not {}".format(reason, declared_reason)
            )
        if declared_verdict == "Accept" and declared_reason:
            errors.append("Accept cannot carry a rejectReason")
        if declared_verdict == "Reject" and not declared_reason:
            errors.append("Reject requires a rejectReason")

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
        if not isinstance(value, dict):
            errors.append("GAS lifecycle record must be an object")
        else:
            kind = _gas_kind(value)
            errors.extend(_gas_record_kind_errors(value, kind))
            if kind == "Transition":
                errors.extend(_gas_transition_errors(value))
            elif kind == "Admission":
                errors.extend(_gas_admission_errors(value))
            elif kind == "Commit":
                errors.extend(_gas_commit_errors(value))
            else:
                errors.append("unknown GAS lifecycle record kind {}".format(kind))

    elif schema_id == "gas-evaluation":
        if not isinstance(value, dict):
            errors.append("GAS evaluation record must be an object")
        else:
            errors.extend(_gas_evaluation_errors(value))

    elif schema_id == "gas-effect-events":
        if not isinstance(value, dict):
            errors.append("GAS Effect event record must be an object")
        else:
            errors.extend(_gas_effect_event_errors(value))

    elif schema_id == "gas-components":
        errors.extend(_gas_components_errors(value))

    elif schema_id == "gas-tag":
        errors.extend(_gas_tag_errors(value))

    elif schema_id == "gas-replication":
        errors.extend(_gas_replication_errors(value))

    elif schema_id == "gas-prediction":
        errors.extend(_gas_prediction_errors(value))

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
    consistency.extend(published_canonical_surface_errors())
    consistency.extend(published_lumio_bin_profile_errors())
    consistency.extend(published_capability_constant_errors())
    consistency.extend(published_contract_body_errors())
    consistency.extend(published_cargo_lock_errors())
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
