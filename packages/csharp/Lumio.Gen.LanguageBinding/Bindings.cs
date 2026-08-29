namespace Lumio.Gen.LanguageBinding
{
    public readonly struct Binding
    {
        public Binding(string schemaId, string rustType, string csharpType)
        {
            SchemaId = schemaId; RustType = rustType; CsharpType = csharpType;
        }
        public string SchemaId { get; }
        public string RustType { get; }
        public string CsharpType { get; }
    }

    public static class Bindings
    {
        public static readonly Binding[] All =
        {
            new Binding("session-revision-vector", "SessionRevisionVector", "SessionRevisionVector"),
            new Binding("cross-world-txn", "CrossWorldTxn", "CrossWorldTxn"),
            new Binding("replication-envelope", "ReplicationEnvelope", "ReplicationEnvelope"),
            new Binding("client-authority-update", "ClientAuthorityUpdate", "ClientAuthorityUpdate"),
            new Binding("protocol-permission-gate", "ProtocolPermissionGate", "ProtocolPermissionGate"),
            new Binding("generated-contract-artifact", "GeneratedContractArtifact", "GeneratedContractArtifact"),
            new Binding("entity-identity", "EntityIdentity", "EntityIdentity"),
            new Binding("native-managed-abi", "NativeManagedAbi", "NativeManagedAbi"),
            new Binding("root-abi-bundle", "RootAbiBundle", "RootAbiBundle"),
            new Binding("canonical-digest-profile", "CanonicalDigestProfile", "CanonicalDigestProfile"),
            new Binding("lumio-bin-profile", "LumioBinProfile", "LumioBinProfile"),
            new Binding("trust-profile", "TrustProfile", "TrustProfile"),
            new Binding("loader-profile", "LoaderProfile", "LoaderProfile"),
            new Binding("evidence-profile", "EvidenceProfile", "EvidenceProfile"),
            new Binding("release-manifest", "ReleaseManifest", "ReleaseManifest"),
            new Binding("maintenance-command", "MaintenanceCommand", "MaintenanceCommand"),
            new Binding("snapshot-header", "SnapshotHeader", "SnapshotHeader"),
            new Binding("config-table", "ConfigTable", "ConfigTable"),
            new Binding("logging-event", "LoggingEvent", "LoggingEvent"),
            new Binding("processor-descriptor", "ProcessorDescriptor", "ProcessorDescriptor"),
            new Binding("failure-bundle", "FailureBundle", "FailureBundle"),
            new Binding("release-catalog", "ReleaseCatalog", "ReleaseCatalog"),
            new Binding("replication-mapping", "ReplicationMapping", "ReplicationMapping"),
            new Binding("host-capability", "HostCapability", "HostCapability"),
            new Binding("id-registry", "IdRegistry", "IdRegistry"),
            new Binding("target-profile", "TargetProfile", "TargetProfile"),
            new Binding("artifact-index", "ArtifactIndex", "ArtifactIndex"),
            new Binding("core-engine-manifest", "CoreEngineManifest", "CoreEngineManifest"),
            new Binding("signature-envelope", "SignatureEnvelope", "SignatureEnvelope"),
            new Binding("verified-package-descriptor", "VerifiedPackageDescriptor", "VerifiedPackageDescriptor"),
            new Binding("voxel-world-port", "VoxelWorldPort", "VoxelWorldPort"),
            new Binding("voxel-chunk-page", "VoxelChunkPage", "VoxelChunkPage"),
            new Binding("voxel-revision-stamp", "VoxelRevisionStamp", "VoxelRevisionStamp"),
            new Binding("voxel-query", "VoxelQuery", "VoxelQuery"),
            new Binding("voxel-mutation-receipt", "VoxelMutationReceipt", "VoxelMutationReceipt"),
            new Binding("tick-phase-contract", "TickPhaseContract", "TickPhaseContract"),
            new Binding("gas-lifecycle", "GasLifecycle", "GasLifecycle"),
            new Binding("txn-journal-record", "TxnJournalRecord", "TxnJournalRecord"),
            new Binding("command-log-record", "CommandLogRecord", "CommandLogRecord"),
            new Binding("wal-record-envelope", "WalRecordEnvelope", "WalRecordEnvelope"),
            new Binding("gameplay-scope-activation", "GameplayScopeActivation", "GameplayScopeActivation"),
            new Binding("state-machine-descriptor", "StateMachineDescriptor", "StateMachineDescriptor"),
            new Binding("voxel-snapshot-payload", "VoxelSnapshotPayload", "VoxelSnapshotPayload"),
            new Binding("voxel-durability-ack", "VoxelDurabilityAck", "VoxelDurabilityAck"),
            new Binding("migration-manifest", "MigrationManifest", "MigrationManifest"),
            new Binding("contract-result", "ContractResult", "ContractResult"),
            new Binding("mod-manifest", "ModManifest", "ModManifest"),
        };
    }
}
