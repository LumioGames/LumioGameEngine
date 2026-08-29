using System.Collections.Generic;

namespace Lumio.Gen.ContractTypes
{
    // ADR-048 (D-3): closed contract type bodies, generated from schemas/.
    // Field order is the schema declaration order. Do not hand-edit.

    /// <summary>An object the architecture source deliberately leaves open (a
    /// replication body, a WAL inner, a config row's values). Carried verbatim as
    /// its canonical JSON text; no shape is invented for what no ADR has closed.</summary>
    public sealed class OpaqueJson
    {
        public OpaqueJson(string json) { Json = json; }
        public string Json { get; }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ConfigTableColumnsItemType
    {
        /// <summary>Wire value <c>bool</c>.</summary>
        Bool = 0,
        /// <summary>Wire value <c>i32</c>.</summary>
        I32 = 1,
        /// <summary>Wire value <c>i64</c>.</summary>
        I64 = 2,
        /// <summary>Wire value <c>u32</c>.</summary>
        U32 = 3,
        /// <summary>Wire value <c>u64</c>.</summary>
        U64 = 4,
        /// <summary>Wire value <c>f32</c>.</summary>
        F32 = 5,
        /// <summary>Wire value <c>f64</c>.</summary>
        F64 = 6,
        /// <summary>Wire value <c>string</c>.</summary>
        String = 7,
        /// <summary>Wire value <c>enum</c>.</summary>
        Enum = 8,
        /// <summary>Wire value <c>ref</c>.</summary>
        Ref = 9,
    }

    public static class ConfigTableColumnsItemTypeWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ConfigTableColumnsItemType value)
        {
            switch (value)
            {
                case ConfigTableColumnsItemType.Bool: return "bool";
                case ConfigTableColumnsItemType.I32: return "i32";
                case ConfigTableColumnsItemType.I64: return "i64";
                case ConfigTableColumnsItemType.U32: return "u32";
                case ConfigTableColumnsItemType.U64: return "u64";
                case ConfigTableColumnsItemType.F32: return "f32";
                case ConfigTableColumnsItemType.F64: return "f64";
                case ConfigTableColumnsItemType.String: return "string";
                case ConfigTableColumnsItemType.Enum: return "enum";
                case ConfigTableColumnsItemType.Ref: return "ref";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ConfigTableActivation
    {
        DevelopmentHotLoad = 0,
        ProductionSignedSwitch = 1,
    }

    public static class ConfigTableActivationWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ConfigTableActivation value)
        {
            switch (value)
            {
                case ConfigTableActivation.DevelopmentHotLoad: return "DevelopmentHotLoad";
                case ConfigTableActivation.ProductionSignedSwitch: return "ProductionSignedSwitch";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ProcessorDescriptorRole
    {
        Server = 0,
        Client = 1,
        Shared = 2,
        Replay = 3,
    }

    public static class ProcessorDescriptorRoleWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ProcessorDescriptorRole value)
        {
            switch (value)
            {
                case ProcessorDescriptorRole.Server: return "Server";
                case ProcessorDescriptorRole.Client: return "Client";
                case ProcessorDescriptorRole.Shared: return "Shared";
                case ProcessorDescriptorRole.Replay: return "Replay";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ProcessorDescriptorPhase
    {
        IngressCapture = 0,
        DecodeAndCanonicalize = 1,
        ApplyInputs = 2,
        ProcessorPlan = 3,
        CrossWorldPrepare = 4,
        NativeJobBarrier = 5,
        CommitDecision = 6,
        VoxelCommit = 7,
        EcsCommandBufferCommit = 8,
        GasAndEventFinalize = 9,
        ReplicationProjection = 10,
        SnapshotHashMetrics = 11,
        EgressPublish = 12,
    }

    public static class ProcessorDescriptorPhaseWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ProcessorDescriptorPhase value)
        {
            switch (value)
            {
                case ProcessorDescriptorPhase.IngressCapture: return "IngressCapture";
                case ProcessorDescriptorPhase.DecodeAndCanonicalize: return "DecodeAndCanonicalize";
                case ProcessorDescriptorPhase.ApplyInputs: return "ApplyInputs";
                case ProcessorDescriptorPhase.ProcessorPlan: return "ProcessorPlan";
                case ProcessorDescriptorPhase.CrossWorldPrepare: return "CrossWorldPrepare";
                case ProcessorDescriptorPhase.NativeJobBarrier: return "NativeJobBarrier";
                case ProcessorDescriptorPhase.CommitDecision: return "CommitDecision";
                case ProcessorDescriptorPhase.VoxelCommit: return "VoxelCommit";
                case ProcessorDescriptorPhase.EcsCommandBufferCommit: return "EcsCommandBufferCommit";
                case ProcessorDescriptorPhase.GasAndEventFinalize: return "GasAndEventFinalize";
                case ProcessorDescriptorPhase.ReplicationProjection: return "ReplicationProjection";
                case ProcessorDescriptorPhase.SnapshotHashMetrics: return "SnapshotHashMetrics";
                case ProcessorDescriptorPhase.EgressPublish: return "EgressPublish";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ProcessorDescriptorDeterminismClass
    {
        Stable = 0,
        StableWithOrderedMerge = 1,
        PlatformSemantic = 2,
        NonAuthoritative = 3,
    }

    public static class ProcessorDescriptorDeterminismClassWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ProcessorDescriptorDeterminismClass value)
        {
            switch (value)
            {
                case ProcessorDescriptorDeterminismClass.Stable: return "Stable";
                case ProcessorDescriptorDeterminismClass.StableWithOrderedMerge: return "StableWithOrderedMerge";
                case ProcessorDescriptorDeterminismClass.PlatformSemantic: return "PlatformSemantic";
                case ProcessorDescriptorDeterminismClass.NonAuthoritative: return "NonAuthoritative";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum TxnJournalRecordCommitState
    {
        Pending = 0,
        Committed = 1,
        Aborted = 2,
    }

    public static class TxnJournalRecordCommitStateWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(TxnJournalRecordCommitState value)
        {
            switch (value)
            {
                case TxnJournalRecordCommitState.Pending: return "Pending";
                case TxnJournalRecordCommitState.Committed: return "Committed";
                case TxnJournalRecordCommitState.Aborted: return "Aborted";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum TxnJournalRecordDurabilityState
    {
        Buffered = 0,
        Durable = 1,
    }

    public static class TxnJournalRecordDurabilityStateWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(TxnJournalRecordDurabilityState value)
        {
            switch (value)
            {
                case TxnJournalRecordDurabilityState.Buffered: return "Buffered";
                case TxnJournalRecordDurabilityState.Durable: return "Durable";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum TxnJournalRecordRecordKind
    {
        Prepare = 0,
        CommitIntent = 1,
        ParticipantMarker = 2,
        Committed = 3,
        Aborted = 4,
    }

    public static class TxnJournalRecordRecordKindWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(TxnJournalRecordRecordKind value)
        {
            switch (value)
            {
                case TxnJournalRecordRecordKind.Prepare: return "Prepare";
                case TxnJournalRecordRecordKind.CommitIntent: return "CommitIntent";
                case TxnJournalRecordRecordKind.ParticipantMarker: return "ParticipantMarker";
                case TxnJournalRecordRecordKind.Committed: return "Committed";
                case TxnJournalRecordRecordKind.Aborted: return "Aborted";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum CommandLogRecordCommitState
    {
        Pending = 0,
        Committed = 1,
        Aborted = 2,
    }

    public static class CommandLogRecordCommitStateWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(CommandLogRecordCommitState value)
        {
            switch (value)
            {
                case CommandLogRecordCommitState.Pending: return "Pending";
                case CommandLogRecordCommitState.Committed: return "Committed";
                case CommandLogRecordCommitState.Aborted: return "Aborted";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum CommandLogRecordDurabilityState
    {
        Buffered = 0,
        Durable = 1,
    }

    public static class CommandLogRecordDurabilityStateWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(CommandLogRecordDurabilityState value)
        {
            switch (value)
            {
                case CommandLogRecordDurabilityState.Buffered: return "Buffered";
                case CommandLogRecordDurabilityState.Durable: return "Durable";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum CommandLogRecordRecordKind
    {
        Append = 0,
        Confirmed = 1,
        Rejected = 2,
    }

    public static class CommandLogRecordRecordKindWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(CommandLogRecordRecordKind value)
        {
            switch (value)
            {
                case CommandLogRecordRecordKind.Append: return "Append";
                case CommandLogRecordRecordKind.Confirmed: return "Confirmed";
                case CommandLogRecordRecordKind.Rejected: return "Rejected";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum WalRecordEnvelopeInnerKind
    {
        TxnJournal = 0,
        CommandLog = 1,
    }

    public static class WalRecordEnvelopeInnerKindWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(WalRecordEnvelopeInnerKind value)
        {
            switch (value)
            {
                case WalRecordEnvelopeInnerKind.TxnJournal: return "TxnJournal";
                case WalRecordEnvelopeInnerKind.CommandLog: return "CommandLog";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum EntityIdentityNamespace
    {
        Authoritative = 0,
        Provisional = 1,
        Replay = 2,
    }

    public static class EntityIdentityNamespaceWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(EntityIdentityNamespace value)
        {
            switch (value)
            {
                case EntityIdentityNamespace.Authoritative: return "Authoritative";
                case EntityIdentityNamespace.Provisional: return "Provisional";
                case EntityIdentityNamespace.Replay: return "Replay";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum EntityIdentityLifecycle
    {
        Reserved = 0,
        Alive = 1,
        Tombstoned = 2,
        Destroyed = 3,
    }

    public static class EntityIdentityLifecycleWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(EntityIdentityLifecycle value)
        {
            switch (value)
            {
                case EntityIdentityLifecycle.Reserved: return "Reserved";
                case EntityIdentityLifecycle.Alive: return "Alive";
                case EntityIdentityLifecycle.Tombstoned: return "Tombstoned";
                case EntityIdentityLifecycle.Destroyed: return "Destroyed";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: ids/index.json:MessageType.</summary>
    public enum ReplicationEnvelopeMessageType
    {
        Handshake = 1,
        FullSnapshot = 2,
        BaselineAck = 6,
        Delta = 3,
        DeltaAck = 7,
        ResyncRequest = 4,
        MaintenanceKick = 5,
        Error = 8,
    }

    public static class ReplicationEnvelopeMessageTypeWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ReplicationEnvelopeMessageType value)
        {
            switch (value)
            {
                case ReplicationEnvelopeMessageType.Handshake: return "Handshake";
                case ReplicationEnvelopeMessageType.FullSnapshot: return "FullSnapshot";
                case ReplicationEnvelopeMessageType.BaselineAck: return "BaselineAck";
                case ReplicationEnvelopeMessageType.Delta: return "Delta";
                case ReplicationEnvelopeMessageType.DeltaAck: return "DeltaAck";
                case ReplicationEnvelopeMessageType.ResyncRequest: return "ResyncRequest";
                case ReplicationEnvelopeMessageType.MaintenanceKick: return "MaintenanceKick";
                case ReplicationEnvelopeMessageType.Error: return "Error";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ReplicationEnvelopeReliability
    {
        Reliable = 0,
        Unreliable = 1,
    }

    public static class ReplicationEnvelopeReliabilityWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ReplicationEnvelopeReliability value)
        {
            switch (value)
            {
                case ReplicationEnvelopeReliability.Reliable: return "Reliable";
                case ReplicationEnvelopeReliability.Unreliable: return "Unreliable";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ReplicationEnvelopeIntegrityAlgorithm
    {
        None = 0,
        CRC32C = 1,
        SHA256 = 2,
        AEAD = 3,
    }

    public static class ReplicationEnvelopeIntegrityAlgorithmWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ReplicationEnvelopeIntegrityAlgorithm value)
        {
            switch (value)
            {
                case ReplicationEnvelopeIntegrityAlgorithm.None: return "None";
                case ReplicationEnvelopeIntegrityAlgorithm.CRC32C: return "CRC32C";
                case ReplicationEnvelopeIntegrityAlgorithm.SHA256: return "SHA256";
                case ReplicationEnvelopeIntegrityAlgorithm.AEAD: return "AEAD";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ReplicationEnvelopeTransportPolicyAuthBinding
    {
        SessionAdmission = 0,
        ConnectionGeneration = 1,
    }

    public static class ReplicationEnvelopeTransportPolicyAuthBindingWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ReplicationEnvelopeTransportPolicyAuthBinding value)
        {
            switch (value)
            {
                case ReplicationEnvelopeTransportPolicyAuthBinding.SessionAdmission: return "SessionAdmission";
                case ReplicationEnvelopeTransportPolicyAuthBinding.ConnectionGeneration: return "ConnectionGeneration";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Ordinal authority: schema declaration order.</summary>
    public enum ReplicationEnvelopeTransportPolicyErrorClass
    {
        Retryable = 0,
        Rejectable = 1,
        Fatal = 2,
    }

    public static class ReplicationEnvelopeTransportPolicyErrorClassWire
    {
        /// <summary>The value that crosses the wire, which the identifier may not equal.</summary>
        public static string Value(ReplicationEnvelopeTransportPolicyErrorClass value)
        {
            switch (value)
            {
                case ReplicationEnvelopeTransportPolicyErrorClass.Retryable: return "Retryable";
                case ReplicationEnvelopeTransportPolicyErrorClass.Rejectable: return "Rejectable";
                case ReplicationEnvelopeTransportPolicyErrorClass.Fatal: return "Fatal";
                default: return string.Empty;
            }
        }
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ConfigTableColumnsItem
    {
        public ConfigTableColumnsItem(string name, ConfigTableColumnsItemType type, bool required, double? minimum, double? maximum, IReadOnlyList<string>? enumValues, string? refTarget, OpaqueJson? defaultValue)
        {
            Name = name;
            Type = type;
            Required = required;
            Minimum = minimum;
            Maximum = maximum;
            EnumValues = enumValues;
            RefTarget = refTarget;
            DefaultValue = defaultValue;
        }

        public string Name { get; }
        public ConfigTableColumnsItemType Type { get; }
        public bool Required { get; }
        /// <summary>Optional in the schema.</summary>
        public double? Minimum { get; }
        /// <summary>Optional in the schema.</summary>
        public double? Maximum { get; }
        /// <summary>Optional in the schema.</summary>
        public IReadOnlyList<string>? EnumValues { get; }
        /// <summary>Optional in the schema.</summary>
        public string? RefTarget { get; }
        /// <summary>Optional in the schema.</summary>
        public OpaqueJson? DefaultValue { get; }

        public static readonly string[] FieldOrder =
        {
            "name",
            "type",
            "required",
            "minimum",
            "maximum",
            "enumValues",
            "refTarget",
            "defaultValue",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ConfigTableRowsItem
    {
        public ConfigTableRowsItem(string key, OpaqueJson values)
        {
            Key = key;
            Values = values;
        }

        public string Key { get; }
        public OpaqueJson Values { get; }

        public static readonly string[] FieldOrder =
        {
            "key",
            "values",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ConfigTable
    {
        public ConfigTable(string tableId, ulong schemaVersion, ulong configRevision, string generatedAt, string sourceHash, IReadOnlyList<ConfigTableColumnsItem> columns, IReadOnlyList<ConfigTableRowsItem> rows, ConfigTableActivation activation, string? signature)
        {
            TableId = tableId;
            SchemaVersion = schemaVersion;
            ConfigRevision = configRevision;
            GeneratedAt = generatedAt;
            SourceHash = sourceHash;
            Columns = columns;
            Rows = rows;
            Activation = activation;
            Signature = signature;
        }

        public string TableId { get; }
        public ulong SchemaVersion { get; }
        public ulong ConfigRevision { get; }
        public string GeneratedAt { get; }
        public string SourceHash { get; }
        public IReadOnlyList<ConfigTableColumnsItem> Columns { get; }
        public IReadOnlyList<ConfigTableRowsItem> Rows { get; }
        public ConfigTableActivation Activation { get; }
        /// <summary>Optional in the schema.</summary>
        public string? Signature { get; }

        public static readonly string[] FieldOrder =
        {
            "tableId",
            "schemaVersion",
            "configRevision",
            "generatedAt",
            "sourceHash",
            "columns",
            "rows",
            "activation",
            "signature",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ProcessorDescriptorBudget
    {
        public ProcessorDescriptorBudget(ulong maxMicros, ulong maxCommands)
        {
            MaxMicros = maxMicros;
            MaxCommands = maxCommands;
        }

        public ulong MaxMicros { get; }
        public ulong MaxCommands { get; }

        public static readonly string[] FieldOrder =
        {
            "maxMicros",
            "maxCommands",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ProcessorDescriptor
    {
        public ProcessorDescriptor(string processorId, ProcessorDescriptorRole role, ProcessorDescriptorPhase phase, string query, IReadOnlyList<string> readSet, IReadOnlyList<string> writeSet, bool mayEmitStructuralCommands, IReadOnlyList<string>? before, IReadOnlyList<string>? after, ProcessorDescriptorDeterminismClass determinismClass, ProcessorDescriptorBudget budget, string diagnosticName)
        {
            ProcessorId = processorId;
            Role = role;
            Phase = phase;
            Query = query;
            ReadSet = readSet;
            WriteSet = writeSet;
            MayEmitStructuralCommands = mayEmitStructuralCommands;
            Before = before;
            After = after;
            DeterminismClass = determinismClass;
            Budget = budget;
            DiagnosticName = diagnosticName;
        }

        public string ProcessorId { get; }
        public ProcessorDescriptorRole Role { get; }
        public ProcessorDescriptorPhase Phase { get; }
        public string Query { get; }
        public IReadOnlyList<string> ReadSet { get; }
        public IReadOnlyList<string> WriteSet { get; }
        public bool MayEmitStructuralCommands { get; }
        /// <summary>Optional in the schema.</summary>
        public IReadOnlyList<string>? Before { get; }
        /// <summary>Optional in the schema.</summary>
        public IReadOnlyList<string>? After { get; }
        public ProcessorDescriptorDeterminismClass DeterminismClass { get; }
        public ProcessorDescriptorBudget Budget { get; }
        public string DiagnosticName { get; }

        public static readonly string[] FieldOrder =
        {
            "processorId",
            "role",
            "phase",
            "query",
            "readSet",
            "writeSet",
            "mayEmitStructuralCommands",
            "before",
            "after",
            "determinismClass",
            "budget",
            "diagnosticName",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class TxnJournalRecord
    {
        public TxnJournalRecord(ulong recordVersion, ulong recordSeq, string previousHash, string payloadHash, ulong length, string checksum, TxnJournalRecordCommitState commitState, TxnJournalRecordDurabilityState durabilityState, string sessionId, string gameReleaseId, ulong tickId, string txnId, string? commandId, TxnJournalRecordRecordKind recordKind, string idempotencyKey)
        {
            RecordVersion = recordVersion;
            RecordSeq = recordSeq;
            PreviousHash = previousHash;
            PayloadHash = payloadHash;
            Length = length;
            Checksum = checksum;
            CommitState = commitState;
            DurabilityState = durabilityState;
            SessionId = sessionId;
            GameReleaseId = gameReleaseId;
            TickId = tickId;
            TxnId = txnId;
            CommandId = commandId;
            RecordKind = recordKind;
            IdempotencyKey = idempotencyKey;
        }

        public ulong RecordVersion { get; }
        public ulong RecordSeq { get; }
        public string PreviousHash { get; }
        public string PayloadHash { get; }
        public ulong Length { get; }
        public string Checksum { get; }
        public TxnJournalRecordCommitState CommitState { get; }
        public TxnJournalRecordDurabilityState DurabilityState { get; }
        public string SessionId { get; }
        public string GameReleaseId { get; }
        public ulong TickId { get; }
        public string TxnId { get; }
        /// <summary>Optional in the schema.</summary>
        public string? CommandId { get; }
        public TxnJournalRecordRecordKind RecordKind { get; }
        public string IdempotencyKey { get; }

        public static readonly string[] FieldOrder =
        {
            "recordVersion",
            "recordSeq",
            "previousHash",
            "payloadHash",
            "length",
            "checksum",
            "commitState",
            "durabilityState",
            "sessionId",
            "gameReleaseId",
            "tickId",
            "txnId",
            "commandId",
            "recordKind",
            "idempotencyKey",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class CommandLogRecord
    {
        public CommandLogRecord(ulong recordVersion, ulong recordSeq, string previousHash, string payloadHash, ulong length, string checksum, CommandLogRecordCommitState commitState, CommandLogRecordDurabilityState durabilityState, string sessionId, string gameReleaseId, ulong tickId, string? txnId, string commandId, CommandLogRecordRecordKind recordKind, string idempotencyKey)
        {
            RecordVersion = recordVersion;
            RecordSeq = recordSeq;
            PreviousHash = previousHash;
            PayloadHash = payloadHash;
            Length = length;
            Checksum = checksum;
            CommitState = commitState;
            DurabilityState = durabilityState;
            SessionId = sessionId;
            GameReleaseId = gameReleaseId;
            TickId = tickId;
            TxnId = txnId;
            CommandId = commandId;
            RecordKind = recordKind;
            IdempotencyKey = idempotencyKey;
        }

        public ulong RecordVersion { get; }
        public ulong RecordSeq { get; }
        public string PreviousHash { get; }
        public string PayloadHash { get; }
        public ulong Length { get; }
        public string Checksum { get; }
        public CommandLogRecordCommitState CommitState { get; }
        public CommandLogRecordDurabilityState DurabilityState { get; }
        public string SessionId { get; }
        public string GameReleaseId { get; }
        public ulong TickId { get; }
        /// <summary>Optional in the schema.</summary>
        public string? TxnId { get; }
        public string CommandId { get; }
        public CommandLogRecordRecordKind RecordKind { get; }
        public string IdempotencyKey { get; }

        public static readonly string[] FieldOrder =
        {
            "recordVersion",
            "recordSeq",
            "previousHash",
            "payloadHash",
            "length",
            "checksum",
            "commitState",
            "durabilityState",
            "sessionId",
            "gameReleaseId",
            "tickId",
            "txnId",
            "commandId",
            "recordKind",
            "idempotencyKey",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class WalRecordEnvelope
    {
        public WalRecordEnvelope(ulong recordVersion, ulong recordSeq, string previousHash, string payloadHash, ulong length, string checksum, WalRecordEnvelopeInnerKind innerKind, OpaqueJson inner)
        {
            RecordVersion = recordVersion;
            RecordSeq = recordSeq;
            PreviousHash = previousHash;
            PayloadHash = payloadHash;
            Length = length;
            Checksum = checksum;
            InnerKind = innerKind;
            Inner = inner;
        }

        public ulong RecordVersion { get; }
        public ulong RecordSeq { get; }
        public string PreviousHash { get; }
        public string PayloadHash { get; }
        public ulong Length { get; }
        public string Checksum { get; }
        public WalRecordEnvelopeInnerKind InnerKind { get; }
        public OpaqueJson Inner { get; }

        public static readonly string[] FieldOrder =
        {
            "recordVersion",
            "recordSeq",
            "previousHash",
            "payloadHash",
            "length",
            "checksum",
            "innerKind",
            "inner",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class EntityIdentity
    {
        public EntityIdentity(string netEntityId, string authorityDomain, ulong worldEpoch, ulong sequence, ulong generation, string? localEntityId, EntityIdentityNamespace @namespace, EntityIdentityLifecycle lifecycle, ulong? tombstoneUntilRevision, string? remappedFrom, ulong? sourceRevision, string? sourceReleaseId)
        {
            NetEntityId = netEntityId;
            AuthorityDomain = authorityDomain;
            WorldEpoch = worldEpoch;
            Sequence = sequence;
            Generation = generation;
            LocalEntityId = localEntityId;
            Namespace = @namespace;
            Lifecycle = lifecycle;
            TombstoneUntilRevision = tombstoneUntilRevision;
            RemappedFrom = remappedFrom;
            SourceRevision = sourceRevision;
            SourceReleaseId = sourceReleaseId;
        }

        public string NetEntityId { get; }
        public string AuthorityDomain { get; }
        public ulong WorldEpoch { get; }
        public ulong Sequence { get; }
        public ulong Generation { get; }
        /// <summary>Optional in the schema.</summary>
        public string? LocalEntityId { get; }
        public EntityIdentityNamespace Namespace { get; }
        public EntityIdentityLifecycle Lifecycle { get; }
        /// <summary>Optional in the schema.</summary>
        public ulong? TombstoneUntilRevision { get; }
        /// <summary>Optional in the schema.</summary>
        public string? RemappedFrom { get; }
        /// <summary>Optional in the schema.</summary>
        public ulong? SourceRevision { get; }
        /// <summary>Optional in the schema.</summary>
        public string? SourceReleaseId { get; }

        public static readonly string[] FieldOrder =
        {
            "netEntityId",
            "authorityDomain",
            "worldEpoch",
            "sequence",
            "generation",
            "localEntityId",
            "namespace",
            "lifecycle",
            "tombstoneUntilRevision",
            "remappedFrom",
            "sourceRevision",
            "sourceReleaseId",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ReplicationEnvelopeIntegrity
    {
        public ReplicationEnvelopeIntegrity(ReplicationEnvelopeIntegrityAlgorithm algorithm, string value)
        {
            Algorithm = algorithm;
            Value = value;
        }

        public ReplicationEnvelopeIntegrityAlgorithm Algorithm { get; }
        public string Value { get; }

        public static readonly string[] FieldOrder =
        {
            "algorithm",
            "value",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ReplicationEnvelopeTransportPolicy
    {
        public ReplicationEnvelopeTransportPolicy(ulong maxMessageBytes, ulong maxFragmentBytes, ulong antiReplayWindow, ReplicationEnvelopeTransportPolicyAuthBinding authBinding, ReplicationEnvelopeTransportPolicyErrorClass errorClass)
        {
            MaxMessageBytes = maxMessageBytes;
            MaxFragmentBytes = maxFragmentBytes;
            AntiReplayWindow = antiReplayWindow;
            AuthBinding = authBinding;
            ErrorClass = errorClass;
        }

        public ulong MaxMessageBytes { get; }
        public ulong MaxFragmentBytes { get; }
        public ulong AntiReplayWindow { get; }
        public ReplicationEnvelopeTransportPolicyAuthBinding AuthBinding { get; }
        public ReplicationEnvelopeTransportPolicyErrorClass ErrorClass { get; }

        public static readonly string[] FieldOrder =
        {
            "maxMessageBytes",
            "maxFragmentBytes",
            "antiReplayWindow",
            "authBinding",
            "errorClass",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class ReplicationEnvelope
    {
        public ReplicationEnvelope(string sessionId, string productId, string gameReleaseId, ulong protocolVersion, ulong length, ulong sequence, ReplicationEnvelopeMessageType messageType, ReplicationEnvelopeReliability reliability, ReplicationEnvelopeIntegrity integrity, string traceId, ReplicationEnvelopeTransportPolicy transportPolicy, OpaqueJson body)
        {
            SessionId = sessionId;
            ProductId = productId;
            GameReleaseId = gameReleaseId;
            ProtocolVersion = protocolVersion;
            Length = length;
            Sequence = sequence;
            MessageType = messageType;
            Reliability = reliability;
            Integrity = integrity;
            TraceId = traceId;
            TransportPolicy = transportPolicy;
            Body = body;
        }

        public string SessionId { get; }
        public string ProductId { get; }
        public string GameReleaseId { get; }
        public ulong ProtocolVersion { get; }
        public ulong Length { get; }
        public ulong Sequence { get; }
        public ReplicationEnvelopeMessageType MessageType { get; }
        public ReplicationEnvelopeReliability Reliability { get; }
        public ReplicationEnvelopeIntegrity Integrity { get; }
        public string TraceId { get; }
        public ReplicationEnvelopeTransportPolicy TransportPolicy { get; }
        public OpaqueJson Body { get; }

        public static readonly string[] FieldOrder =
        {
            "sessionId",
            "productId",
            "gameReleaseId",
            "protocolVersion",
            "length",
            "sequence",
            "messageType",
            "reliability",
            "integrity",
            "traceId",
            "transportPolicy",
            "body",
        };
    }

    /// <summary>Fields in schema declaration order.</summary>
    public sealed class SessionRevisionVector
    {
        public SessionRevisionVector(ulong tickId, ulong gameRevision, ulong voxelWorldRevision, IReadOnlyDictionary<string, ulong> chunkRevisionSet, ulong replicationRevision, ulong configRevision, ulong schemaEpoch)
        {
            TickId = tickId;
            GameRevision = gameRevision;
            VoxelWorldRevision = voxelWorldRevision;
            ChunkRevisionSet = chunkRevisionSet;
            ReplicationRevision = replicationRevision;
            ConfigRevision = configRevision;
            SchemaEpoch = schemaEpoch;
        }

        public ulong TickId { get; }
        public ulong GameRevision { get; }
        public ulong VoxelWorldRevision { get; }
        public IReadOnlyDictionary<string, ulong> ChunkRevisionSet { get; }
        public ulong ReplicationRevision { get; }
        public ulong ConfigRevision { get; }
        public ulong SchemaEpoch { get; }

        public static readonly string[] FieldOrder =
        {
            "tickId",
            "gameRevision",
            "voxelWorldRevision",
            "chunkRevisionSet",
            "replicationRevision",
            "configRevision",
            "schemaEpoch",
        };
    }
}
