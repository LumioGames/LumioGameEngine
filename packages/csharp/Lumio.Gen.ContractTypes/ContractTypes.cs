namespace Lumio.Gen.ContractTypes
{
    public static class Catalog
    {
        public const string BaselineId = "LGE-V1.4-2026-08-27";
        public static readonly string[] SchemaIds = { "session-revision-vector", "cross-world-txn", "replication-envelope", "client-authority-update", "protocol-permission-gate", "generated-contract-artifact", "entity-identity", "native-managed-abi", "root-abi-bundle", "canonical-digest-profile", "lumio-bin-profile", "trust-profile", "loader-profile", "evidence-profile", "release-manifest", "maintenance-command", "snapshot-header", "config-table", "logging-event", "processor-descriptor", "failure-bundle", "release-catalog", "replication-mapping", "host-capability", "id-registry", "target-profile", "artifact-index", "core-engine-manifest", "signature-envelope", "verified-package-descriptor", "voxel-world-port", "voxel-chunk-page", "voxel-revision-stamp", "voxel-query", "voxel-mutation-receipt", "tick-phase-contract", "gas-lifecycle", "txn-journal-record", "command-log-record", "wal-record-envelope", "gameplay-scope-activation", "state-machine-descriptor", "voxel-snapshot-payload", "voxel-durability-ack", "migration-manifest", "contract-result", "mod-manifest" };
        public static readonly string[] StableErrorIds = { "RevisionConflict", "MaintenanceKick", "ReleaseMismatch", "NativeAbiMismatch", "StaleEpoch", "FencingTokenStale", "ManifestMalformed", "ManifestUnsupportedVersion", "ManifestDigestMismatch", "ArtifactMissing", "ArtifactDigestMismatch", "SignatureMissing", "SignatureInvalid", "TrustRootUnknown", "TrustPolicyRejected", "KeyRevoked", "EvidenceMissing", "EvidenceDigestMismatch", "TargetProfileMismatch", "CapabilityMissing", "SymbolMissing", "SymbolCollision", "PackageIdentityConflict", "WorkerPoolDuplicate", "LoaderTimeout", "LoaderCancelled", "LoaderOutOfMemory", "PartialLoadRolledBack", "InvalidHandle", "HandleDoubleRelease", "MessagePermissionDenied", "StaleConnectionGeneration", "ChunkUnavailable", "TargetRevisionUnavailable", "BudgetExceeded", "QueueFull", "CoordinateOutOfBounds", "DirtyChunkNotDurable", "SnapshotBaseMismatch", "SessionMismatch", "RoleMismatch", "ClaimNotGranted", "SessionAntiReplay", "InvalidArgument", "WrongContext", "BufferTooSmall", "CapacityExceeded", "Cancelled", "TimedOut", "ContextClosing", "ContextDestroyed", "PanicBoundary", "InternalInvariant" };
        public static readonly string[] ChunkPresence = { "Ready", "NotLoaded", "Pending", "Unavailable" };
        public static readonly string[] VoxelWorldRoles = { "Authority", "Replica" };
        public const string AbiEntrySymbol = "lumio_core_get_api_v1";
        public const string AbiSymbolPrefix = "lumio_";
        public const uint AbiVersion = 1;
        public const string AbiCallingConvention = "C";
        public const uint AbiPointerWidth = 64;
        public const string AbiEndianness = "Little";
    }

    public readonly struct AbiTypeMapping
    {
        public AbiTypeMapping(string typeRef, string c, string csharp, string rust, int size, int align)
        {
            TypeRef = typeRef; C = c; Csharp = csharp; Rust = rust; Size = size; Align = align;
        }
        public string TypeRef { get; }
        public string C { get; }
        public string Csharp { get; }
        public string Rust { get; }
        public int Size { get; }
        public int Align { get; }
    }
    public static class AbiTypeMappings
    {
        public static readonly AbiTypeMapping[] All =
        {
            new AbiTypeMapping("u8", "uint8_t", "byte", "u8", 1, 1),
            new AbiTypeMapping("u16", "uint16_t", "ushort", "u16", 2, 2),
            new AbiTypeMapping("u32", "uint32_t", "uint", "u32", 4, 4),
            new AbiTypeMapping("u64", "uint64_t", "ulong", "u64", 8, 8),
            new AbiTypeMapping("i8", "int8_t", "sbyte", "i8", 1, 1),
            new AbiTypeMapping("i16", "int16_t", "short", "i16", 2, 2),
            new AbiTypeMapping("i32", "int32_t", "int", "i32", 4, 4),
            new AbiTypeMapping("i64", "int64_t", "long", "i64", 8, 8),
            new AbiTypeMapping("f32", "float", "float", "f32", 4, 4),
            new AbiTypeMapping("f64", "double", "double", "f64", 8, 8),
            new AbiTypeMapping("bool32", "uint32_t", "uint", "u32", 4, 4),
            new AbiTypeMapping("status", "lumio_status_t", "LumioStatus", "LumioStatus", 4, 4),
            new AbiTypeMapping("handle:<kind>", "lumio_handle_t", "LumioHandle", "LumioHandle", 16, 8),
            new AbiTypeMapping("buffer:in", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
            new AbiTypeMapping("buffer:out", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
            new AbiTypeMapping("buffer:inout", "lumio_buffer_t", "LumioBuffer", "LumioBuffer", 24, 8),
            new AbiTypeMapping("struct:<name>:v<N>", "const lumio_<name>_v<N>*", "IntPtr", "*const Lumio<Name>V<N>", 8, 8),
            new AbiTypeMapping("ptr:const:<name>", "const lumio_<name>*", "IntPtr", "*const Lumio<Name>", 8, 8),
            new AbiTypeMapping("ptr:mut:<name>", "lumio_<name>*", "IntPtr", "*mut Lumio<Name>", 8, 8),
        };
    }

    public readonly struct Transition
    {
        public Transition(string machine, string from, string to, string @event)
        {
            Machine = machine; From = from; To = to; Event = @event;
        }
        public string Machine { get; }
        public string From { get; }
        public string To { get; }
        public string Event { get; }
    }
    public static class StateTransitionTable
    {
        public static readonly Transition[] All =
        {
            new Transition("ClientReplicaSession", "Disconnected", "Connecting", "Connect"),
            new Transition("ClientReplicaSession", "Connecting", "Negotiating", "ChannelEstablished"),
            new Transition("ClientReplicaSession", "Negotiating", "Synchronizing", "HandshakeAccepted"),
            new Transition("ClientReplicaSession", "Synchronizing", "Active", "BaselineApplied"),
            new Transition("ClientReplicaSession", "Active", "Resyncing", "ResyncRequested"),
            new Transition("ClientReplicaSession", "Resyncing", "Active", "ResyncComplete"),
            new Transition("ClientReplicaSession", "Active", "Reconnecting", "ConnectionLost"),
            new Transition("ClientReplicaSession", "Resyncing", "Reconnecting", "ConnectionLost"),
            new Transition("ClientReplicaSession", "Reconnecting", "Synchronizing", "HandshakeAccepted"),
            new Transition("CoreEngineLoader", "Uninitialized", "Preflighting", "BeginPreflight"),
            new Transition("CoreEngineLoader", "Preflighting", "Verified", "TrustVerified"),
            new Transition("CoreEngineLoader", "Verified", "Binding", "BindSymbols"),
            new Transition("CoreEngineLoader", "Binding", "ApiReady", "ApiTableBound"),
            new Transition("CoreEngineLoader", "ApiReady", "Leased", "Lease"),
            new Transition("CoreEngineLoader", "Leased", "Quiescing", "Quiesce"),
            new Transition("CoreEngineLoader", "Quiescing", "Released", "Release"),
            new Transition("CoreEngineLoader", "Preflighting", "FailedRolledBack", "PreLeaseFailure"),
            new Transition("CoreEngineLoader", "Verified", "FailedRolledBack", "PreLeaseFailure"),
            new Transition("CoreEngineLoader", "Binding", "FailedRolledBack", "PreLeaseFailure"),
            new Transition("CoreEngineLoader", "ApiReady", "FailedRolledBack", "PreLeaseFailure"),
            new Transition("CrossWorldTxn", "Created", "Prepared", "ParticipantsPrepared"),
            new Transition("CrossWorldTxn", "Created", "Aborted", "Reject"),
            new Transition("CrossWorldTxn", "Created", "Expired", "DeadlineExceeded"),
            new Transition("CrossWorldTxn", "Prepared", "CommitIntent", "PersistCommitIntent"),
            new Transition("CrossWorldTxn", "Prepared", "Aborted", "Reject"),
            new Transition("CrossWorldTxn", "Prepared", "Expired", "DeadlineExceeded"),
            new Transition("CrossWorldTxn", "CommitIntent", "Committed", "MarkersApplied"),
            new Transition("CrossWorldTxn", "CommitIntent", "Indeterminate", "CrashWindow"),
            new Transition("CrossWorldTxn", "Indeterminate", "Committed", "IntentReplayed"),
            new Transition("EcsCommandBuffer", "Open", "Sealed", "Seal"),
            new Transition("EcsCommandBuffer", "Sealed", "Merged", "Merge"),
            new Transition("EcsCommandBuffer", "Merged", "Prepared", "Prepare"),
            new Transition("EcsCommandBuffer", "Prepared", "Applied", "Apply"),
            new Transition("GameplayScopeActivation", "OldActiveNewStaging", "NewValidated", "ValidationPassed"),
            new Transition("GameplayScopeActivation", "NewValidated", "BarrierSwitch", "CommitSwitch"),
            new Transition("GameplayScopeActivation", "NewValidated", "OldActiveNewStaging", "NewScopeDiscarded"),
            new Transition("GameplayScopeActivation", "BarrierSwitch", "OldQuiescing", "SwitchApplied"),
            new Transition("GameplayScopeActivation", "OldQuiescing", "OldUnloaded", "OldScopeUnloaded"),
            new Transition("GasAbility", "Requested", "Activated", "Activate"),
            new Transition("GasAbility", "Requested", "Rejected", "Reject"),
            new Transition("GasAbility", "Requested", "Cancelled", "Cancel"),
            new Transition("GasAbility", "Requested", "RolledBack", "RollBack"),
            new Transition("GasAbility", "Activated", "Executing", "Execute"),
            new Transition("GasAbility", "Activated", "Rejected", "Reject"),
            new Transition("GasAbility", "Activated", "Cancelled", "Cancel"),
            new Transition("GasAbility", "Activated", "RolledBack", "RollBack"),
            new Transition("GasAbility", "Executing", "Completed", "Finish"),
            new Transition("GasAbility", "Executing", "Expired", "Expire"),
            new Transition("GasAbility", "Executing", "Cancelled", "Cancel"),
            new Transition("GasAbility", "Executing", "RolledBack", "RollBack"),
            new Transition("GasEffect", "Pending", "Active", "Apply"),
            new Transition("GasEffect", "Pending", "Rejected", "Reject"),
            new Transition("GasEffect", "Pending", "RolledBack", "RollBack"),
            new Transition("GasEffect", "Active", "Expired", "Expire"),
            new Transition("GasEffect", "Active", "Removed", "Remove"),
            new Transition("GasEffect", "Active", "RolledBack", "RollBack"),
            new Transition("ReleasePool", "Published", "Verified", "VerifyManifest"),
            new Transition("ReleasePool", "Verified", "Warmup", "Warm"),
            new Transition("ReleasePool", "Warmup", "Serving", "Serve"),
            new Transition("ReleasePool", "Serving", "Draining", "Drain"),
            new Transition("ReleasePool", "Draining", "Empty", "LastSessionClosed"),
            new Transition("ReleasePool", "Empty", "Retired", "Retire"),
            new Transition("SimulationSession", "Created", "Initialized", "Initialize"),
            new Transition("SimulationSession", "Initialized", "Ready", "Prime"),
            new Transition("SimulationSession", "Ready", "Running", "Start"),
            new Transition("SimulationSession", "Running", "Paused", "Pause"),
            new Transition("SimulationSession", "Paused", "Running", "Resume"),
            new Transition("SimulationSession", "Running", "Draining", "Drain"),
            new Transition("SimulationSession", "Paused", "Draining", "Drain"),
            new Transition("SimulationSession", "Draining", "Snapshotted", "FinalSnapshotTaken"),
            new Transition("SimulationSession", "Snapshotted", "Disposed", "Dispose"),
            new Transition("VoxelChunkResidency", "Unallocated", "Loading", "RequestLoad"),
            new Transition("VoxelChunkResidency", "Loading", "Ready", "PagesLoaded"),
            new Transition("VoxelChunkResidency", "Loading", "Failed", "LoadFailed"),
            new Transition("VoxelChunkResidency", "Ready", "Dirty", "AuthoritativeWrite"),
            new Transition("VoxelChunkResidency", "Dirty", "Ready", "DurabilityAckCovers"),
            new Transition("VoxelChunkResidency", "Ready", "Evicting", "EvictApproved"),
            new Transition("VoxelChunkResidency", "Evicting", "Unloaded", "EvictionComplete"),
            new Transition("VoxelChunkResidency", "Unloaded", "Loading", "RequestLoad"),
            new Transition("VoxelSnapshotCapture", "Requested", "Cutting", "BeginCut"),
            new Transition("VoxelSnapshotCapture", "Cutting", "Pinned", "PinEstablished"),
            new Transition("VoxelSnapshotCapture", "Pinned", "Encoding", "BeginEncode"),
            new Transition("VoxelSnapshotCapture", "Encoding", "Verified", "PayloadVerified"),
            new Transition("VoxelSnapshotCapture", "Verified", "Ready", "Publish"),
            new Transition("VoxelSnapshotCapture", "Ready", "Released", "ReleasePin"),
            new Transition("WorldSlotHost", "Allocated", "Bootstrapping", "BeginBootstrap"),
            new Transition("WorldSlotHost", "Bootstrapping", "NativeReady", "NativeLoaded"),
            new Transition("WorldSlotHost", "NativeReady", "ManagedReady", "ManagedLoaded"),
            new Transition("WorldSlotHost", "ManagedReady", "LoadingSession", "LoadSession"),
            new Transition("WorldSlotHost", "LoadingSession", "Running", "SessionLoaded"),
            new Transition("WorldSlotHost", "Running", "Quiescing", "Quiesce"),
            new Transition("WorldSlotHost", "Quiescing", "Running", "Resume"),
            new Transition("WorldSlotHost", "Quiescing", "Snapshotting", "BeginSnapshot"),
            new Transition("WorldSlotHost", "Quiescing", "Reloading", "BeginReload"),
            new Transition("WorldSlotHost", "Quiescing", "Migrating", "BeginMigrate"),
            new Transition("WorldSlotHost", "Snapshotting", "Quiescing", "SnapshotComplete"),
            new Transition("WorldSlotHost", "Reloading", "Quiescing", "ReloadComplete"),
            new Transition("WorldSlotHost", "Migrating", "Stopping", "MigrationHandedOff"),
            new Transition("WorldSlotHost", "Quiescing", "Stopping", "Stop"),
            new Transition("WorldSlotHost", "Stopping", "Destroyed", "TeardownComplete"),
        };
    }
}
