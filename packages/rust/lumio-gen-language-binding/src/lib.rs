//! Generated LanguageBinding artifact. Do not hand-edit.
//! Publisher: LumioGameEngineArchitecture / LGE-V1.4-2026-08-27.

#![forbid(unsafe_code)]

pub mod root_abi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub schema_id: &'static str,
    pub rust_type: &'static str,
    pub csharp_type: &'static str,
}

pub const BINDINGS: &[Binding] = &[
    Binding { schema_id: "session-revision-vector", rust_type: "SessionRevisionVector", csharp_type: "SessionRevisionVector" },
    Binding { schema_id: "cross-world-txn", rust_type: "CrossWorldTxn", csharp_type: "CrossWorldTxn" },
    Binding { schema_id: "replication-envelope", rust_type: "ReplicationEnvelope", csharp_type: "ReplicationEnvelope" },
    Binding { schema_id: "client-authority-update", rust_type: "ClientAuthorityUpdate", csharp_type: "ClientAuthorityUpdate" },
    Binding { schema_id: "protocol-permission-gate", rust_type: "ProtocolPermissionGate", csharp_type: "ProtocolPermissionGate" },
    Binding { schema_id: "generated-contract-artifact", rust_type: "GeneratedContractArtifact", csharp_type: "GeneratedContractArtifact" },
    Binding { schema_id: "entity-identity", rust_type: "EntityIdentity", csharp_type: "EntityIdentity" },
    Binding { schema_id: "native-managed-abi", rust_type: "NativeManagedAbi", csharp_type: "NativeManagedAbi" },
    Binding { schema_id: "root-abi-bundle", rust_type: "RootAbiBundle", csharp_type: "RootAbiBundle" },
    Binding { schema_id: "canonical-digest-profile", rust_type: "CanonicalDigestProfile", csharp_type: "CanonicalDigestProfile" },
    Binding { schema_id: "release-manifest", rust_type: "ReleaseManifest", csharp_type: "ReleaseManifest" },
    Binding { schema_id: "maintenance-command", rust_type: "MaintenanceCommand", csharp_type: "MaintenanceCommand" },
    Binding { schema_id: "snapshot-header", rust_type: "SnapshotHeader", csharp_type: "SnapshotHeader" },
    Binding { schema_id: "config-table", rust_type: "ConfigTable", csharp_type: "ConfigTable" },
    Binding { schema_id: "logging-event", rust_type: "LoggingEvent", csharp_type: "LoggingEvent" },
    Binding { schema_id: "processor-descriptor", rust_type: "ProcessorDescriptor", csharp_type: "ProcessorDescriptor" },
    Binding { schema_id: "failure-bundle", rust_type: "FailureBundle", csharp_type: "FailureBundle" },
    Binding { schema_id: "release-catalog", rust_type: "ReleaseCatalog", csharp_type: "ReleaseCatalog" },
    Binding { schema_id: "replication-mapping", rust_type: "ReplicationMapping", csharp_type: "ReplicationMapping" },
    Binding { schema_id: "host-capability", rust_type: "HostCapability", csharp_type: "HostCapability" },
    Binding { schema_id: "id-registry", rust_type: "IdRegistry", csharp_type: "IdRegistry" },
    Binding { schema_id: "target-profile", rust_type: "TargetProfile", csharp_type: "TargetProfile" },
    Binding { schema_id: "artifact-index", rust_type: "ArtifactIndex", csharp_type: "ArtifactIndex" },
    Binding { schema_id: "core-engine-manifest", rust_type: "CoreEngineManifest", csharp_type: "CoreEngineManifest" },
    Binding { schema_id: "signature-envelope", rust_type: "SignatureEnvelope", csharp_type: "SignatureEnvelope" },
    Binding { schema_id: "verified-package-descriptor", rust_type: "VerifiedPackageDescriptor", csharp_type: "VerifiedPackageDescriptor" },
    Binding { schema_id: "voxel-world-port", rust_type: "VoxelWorldPort", csharp_type: "VoxelWorldPort" },
    Binding { schema_id: "voxel-chunk-page", rust_type: "VoxelChunkPage", csharp_type: "VoxelChunkPage" },
    Binding { schema_id: "voxel-revision-stamp", rust_type: "VoxelRevisionStamp", csharp_type: "VoxelRevisionStamp" },
    Binding { schema_id: "voxel-query", rust_type: "VoxelQuery", csharp_type: "VoxelQuery" },
    Binding { schema_id: "voxel-mutation-receipt", rust_type: "VoxelMutationReceipt", csharp_type: "VoxelMutationReceipt" },
    Binding { schema_id: "tick-phase-contract", rust_type: "TickPhaseContract", csharp_type: "TickPhaseContract" },
    Binding { schema_id: "gas-lifecycle", rust_type: "GasLifecycle", csharp_type: "GasLifecycle" },
    Binding { schema_id: "txn-journal-record", rust_type: "TxnJournalRecord", csharp_type: "TxnJournalRecord" },
    Binding { schema_id: "command-log-record", rust_type: "CommandLogRecord", csharp_type: "CommandLogRecord" },
    Binding { schema_id: "wal-record-envelope", rust_type: "WalRecordEnvelope", csharp_type: "WalRecordEnvelope" },
    Binding { schema_id: "gameplay-scope-activation", rust_type: "GameplayScopeActivation", csharp_type: "GameplayScopeActivation" },
    Binding { schema_id: "state-machine-descriptor", rust_type: "StateMachineDescriptor", csharp_type: "StateMachineDescriptor" },
    Binding { schema_id: "voxel-snapshot-payload", rust_type: "VoxelSnapshotPayload", csharp_type: "VoxelSnapshotPayload" },
    Binding { schema_id: "voxel-durability-ack", rust_type: "VoxelDurabilityAck", csharp_type: "VoxelDurabilityAck" },
    Binding { schema_id: "migration-manifest", rust_type: "MigrationManifest", csharp_type: "MigrationManifest" },
    Binding { schema_id: "contract-result", rust_type: "ContractResult", csharp_type: "ContractResult" },
    Binding { schema_id: "mod-manifest", rust_type: "ModManifest", csharp_type: "ModManifest" },
];
