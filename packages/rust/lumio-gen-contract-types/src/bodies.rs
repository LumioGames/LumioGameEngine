//! ADR-048 (D-3): closed contract type bodies, generated from `schemas/`.
//! Field order is the schema declaration order. Do not hand-edit.

use std::collections::BTreeMap;

/// An object the architecture source deliberately leaves open (a
/// replication `body`, a WAL `inner`, a config row's `values`). Carried
/// verbatim as its canonical JSON text; this crate does not invent a shape
/// for something no ADR has closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueJson(pub String);

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigTableColumnsItemType {
    /// Wire value `bool`.
    Bool,
    /// Wire value `i32`.
    I32,
    /// Wire value `i64`.
    I64,
    /// Wire value `u32`.
    U32,
    /// Wire value `u64`.
    U64,
    /// Wire value `f32`.
    F32,
    /// Wire value `f64`.
    F64,
    /// Wire value `string`.
    String,
    /// Wire value `enum`.
    Enum,
    /// Wire value `ref`.
    Ref,
}

impl ConfigTableColumnsItemType {
    pub const fn ordinal(self) -> u32 {
        match self {
            ConfigTableColumnsItemType::Bool => 0,
            ConfigTableColumnsItemType::I32 => 1,
            ConfigTableColumnsItemType::I64 => 2,
            ConfigTableColumnsItemType::U32 => 3,
            ConfigTableColumnsItemType::U64 => 4,
            ConfigTableColumnsItemType::F32 => 5,
            ConfigTableColumnsItemType::F64 => 6,
            ConfigTableColumnsItemType::String => 7,
            ConfigTableColumnsItemType::Enum => 8,
            ConfigTableColumnsItemType::Ref => 9,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ConfigTableColumnsItemType::Bool => "bool",
            ConfigTableColumnsItemType::I32 => "i32",
            ConfigTableColumnsItemType::I64 => "i64",
            ConfigTableColumnsItemType::U32 => "u32",
            ConfigTableColumnsItemType::U64 => "u64",
            ConfigTableColumnsItemType::F32 => "f32",
            ConfigTableColumnsItemType::F64 => "f64",
            ConfigTableColumnsItemType::String => "string",
            ConfigTableColumnsItemType::Enum => "enum",
            ConfigTableColumnsItemType::Ref => "ref",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigTableActivation {
    DevelopmentHotLoad,
    ProductionSignedSwitch,
}

impl ConfigTableActivation {
    pub const fn ordinal(self) -> u32 {
        match self {
            ConfigTableActivation::DevelopmentHotLoad => 0,
            ConfigTableActivation::ProductionSignedSwitch => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ConfigTableActivation::DevelopmentHotLoad => "DevelopmentHotLoad",
            ConfigTableActivation::ProductionSignedSwitch => "ProductionSignedSwitch",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessorDescriptorRole {
    Server,
    Client,
    Shared,
    Replay,
}

impl ProcessorDescriptorRole {
    pub const fn ordinal(self) -> u32 {
        match self {
            ProcessorDescriptorRole::Server => 0,
            ProcessorDescriptorRole::Client => 1,
            ProcessorDescriptorRole::Shared => 2,
            ProcessorDescriptorRole::Replay => 3,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ProcessorDescriptorRole::Server => "Server",
            ProcessorDescriptorRole::Client => "Client",
            ProcessorDescriptorRole::Shared => "Shared",
            ProcessorDescriptorRole::Replay => "Replay",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessorDescriptorPhase {
    IngressCapture,
    DecodeAndCanonicalize,
    ApplyInputs,
    ProcessorPlan,
    CrossWorldPrepare,
    NativeJobBarrier,
    CommitDecision,
    VoxelCommit,
    EcsCommandBufferCommit,
    GasAndEventFinalize,
    ReplicationProjection,
    SnapshotHashMetrics,
    EgressPublish,
}

impl ProcessorDescriptorPhase {
    pub const fn ordinal(self) -> u32 {
        match self {
            ProcessorDescriptorPhase::IngressCapture => 0,
            ProcessorDescriptorPhase::DecodeAndCanonicalize => 1,
            ProcessorDescriptorPhase::ApplyInputs => 2,
            ProcessorDescriptorPhase::ProcessorPlan => 3,
            ProcessorDescriptorPhase::CrossWorldPrepare => 4,
            ProcessorDescriptorPhase::NativeJobBarrier => 5,
            ProcessorDescriptorPhase::CommitDecision => 6,
            ProcessorDescriptorPhase::VoxelCommit => 7,
            ProcessorDescriptorPhase::EcsCommandBufferCommit => 8,
            ProcessorDescriptorPhase::GasAndEventFinalize => 9,
            ProcessorDescriptorPhase::ReplicationProjection => 10,
            ProcessorDescriptorPhase::SnapshotHashMetrics => 11,
            ProcessorDescriptorPhase::EgressPublish => 12,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ProcessorDescriptorPhase::IngressCapture => "IngressCapture",
            ProcessorDescriptorPhase::DecodeAndCanonicalize => "DecodeAndCanonicalize",
            ProcessorDescriptorPhase::ApplyInputs => "ApplyInputs",
            ProcessorDescriptorPhase::ProcessorPlan => "ProcessorPlan",
            ProcessorDescriptorPhase::CrossWorldPrepare => "CrossWorldPrepare",
            ProcessorDescriptorPhase::NativeJobBarrier => "NativeJobBarrier",
            ProcessorDescriptorPhase::CommitDecision => "CommitDecision",
            ProcessorDescriptorPhase::VoxelCommit => "VoxelCommit",
            ProcessorDescriptorPhase::EcsCommandBufferCommit => "EcsCommandBufferCommit",
            ProcessorDescriptorPhase::GasAndEventFinalize => "GasAndEventFinalize",
            ProcessorDescriptorPhase::ReplicationProjection => "ReplicationProjection",
            ProcessorDescriptorPhase::SnapshotHashMetrics => "SnapshotHashMetrics",
            ProcessorDescriptorPhase::EgressPublish => "EgressPublish",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessorDescriptorDeterminismClass {
    Stable,
    StableWithOrderedMerge,
    PlatformSemantic,
    NonAuthoritative,
}

impl ProcessorDescriptorDeterminismClass {
    pub const fn ordinal(self) -> u32 {
        match self {
            ProcessorDescriptorDeterminismClass::Stable => 0,
            ProcessorDescriptorDeterminismClass::StableWithOrderedMerge => 1,
            ProcessorDescriptorDeterminismClass::PlatformSemantic => 2,
            ProcessorDescriptorDeterminismClass::NonAuthoritative => 3,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ProcessorDescriptorDeterminismClass::Stable => "Stable",
            ProcessorDescriptorDeterminismClass::StableWithOrderedMerge => "StableWithOrderedMerge",
            ProcessorDescriptorDeterminismClass::PlatformSemantic => "PlatformSemantic",
            ProcessorDescriptorDeterminismClass::NonAuthoritative => "NonAuthoritative",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxnJournalRecordCommitState {
    Pending,
    Committed,
    Aborted,
}

impl TxnJournalRecordCommitState {
    pub const fn ordinal(self) -> u32 {
        match self {
            TxnJournalRecordCommitState::Pending => 0,
            TxnJournalRecordCommitState::Committed => 1,
            TxnJournalRecordCommitState::Aborted => 2,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            TxnJournalRecordCommitState::Pending => "Pending",
            TxnJournalRecordCommitState::Committed => "Committed",
            TxnJournalRecordCommitState::Aborted => "Aborted",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxnJournalRecordDurabilityState {
    Buffered,
    Durable,
}

impl TxnJournalRecordDurabilityState {
    pub const fn ordinal(self) -> u32 {
        match self {
            TxnJournalRecordDurabilityState::Buffered => 0,
            TxnJournalRecordDurabilityState::Durable => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            TxnJournalRecordDurabilityState::Buffered => "Buffered",
            TxnJournalRecordDurabilityState::Durable => "Durable",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxnJournalRecordRecordKind {
    Prepare,
    CommitIntent,
    ParticipantMarker,
    Committed,
    Aborted,
}

impl TxnJournalRecordRecordKind {
    pub const fn ordinal(self) -> u32 {
        match self {
            TxnJournalRecordRecordKind::Prepare => 0,
            TxnJournalRecordRecordKind::CommitIntent => 1,
            TxnJournalRecordRecordKind::ParticipantMarker => 2,
            TxnJournalRecordRecordKind::Committed => 3,
            TxnJournalRecordRecordKind::Aborted => 4,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            TxnJournalRecordRecordKind::Prepare => "Prepare",
            TxnJournalRecordRecordKind::CommitIntent => "CommitIntent",
            TxnJournalRecordRecordKind::ParticipantMarker => "ParticipantMarker",
            TxnJournalRecordRecordKind::Committed => "Committed",
            TxnJournalRecordRecordKind::Aborted => "Aborted",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandLogRecordCommitState {
    Pending,
    Committed,
    Aborted,
}

impl CommandLogRecordCommitState {
    pub const fn ordinal(self) -> u32 {
        match self {
            CommandLogRecordCommitState::Pending => 0,
            CommandLogRecordCommitState::Committed => 1,
            CommandLogRecordCommitState::Aborted => 2,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            CommandLogRecordCommitState::Pending => "Pending",
            CommandLogRecordCommitState::Committed => "Committed",
            CommandLogRecordCommitState::Aborted => "Aborted",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandLogRecordDurabilityState {
    Buffered,
    Durable,
}

impl CommandLogRecordDurabilityState {
    pub const fn ordinal(self) -> u32 {
        match self {
            CommandLogRecordDurabilityState::Buffered => 0,
            CommandLogRecordDurabilityState::Durable => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            CommandLogRecordDurabilityState::Buffered => "Buffered",
            CommandLogRecordDurabilityState::Durable => "Durable",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandLogRecordRecordKind {
    Append,
    Confirmed,
    Rejected,
}

impl CommandLogRecordRecordKind {
    pub const fn ordinal(self) -> u32 {
        match self {
            CommandLogRecordRecordKind::Append => 0,
            CommandLogRecordRecordKind::Confirmed => 1,
            CommandLogRecordRecordKind::Rejected => 2,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            CommandLogRecordRecordKind::Append => "Append",
            CommandLogRecordRecordKind::Confirmed => "Confirmed",
            CommandLogRecordRecordKind::Rejected => "Rejected",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalRecordEnvelopeInnerKind {
    TxnJournal,
    CommandLog,
}

impl WalRecordEnvelopeInnerKind {
    pub const fn ordinal(self) -> u32 {
        match self {
            WalRecordEnvelopeInnerKind::TxnJournal => 0,
            WalRecordEnvelopeInnerKind::CommandLog => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            WalRecordEnvelopeInnerKind::TxnJournal => "TxnJournal",
            WalRecordEnvelopeInnerKind::CommandLog => "CommandLog",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityIdentityNamespace {
    Authoritative,
    Provisional,
    Replay,
}

impl EntityIdentityNamespace {
    pub const fn ordinal(self) -> u32 {
        match self {
            EntityIdentityNamespace::Authoritative => 0,
            EntityIdentityNamespace::Provisional => 1,
            EntityIdentityNamespace::Replay => 2,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            EntityIdentityNamespace::Authoritative => "Authoritative",
            EntityIdentityNamespace::Provisional => "Provisional",
            EntityIdentityNamespace::Replay => "Replay",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityIdentityLifecycle {
    Reserved,
    Alive,
    Tombstoned,
    Destroyed,
}

impl EntityIdentityLifecycle {
    pub const fn ordinal(self) -> u32 {
        match self {
            EntityIdentityLifecycle::Reserved => 0,
            EntityIdentityLifecycle::Alive => 1,
            EntityIdentityLifecycle::Tombstoned => 2,
            EntityIdentityLifecycle::Destroyed => 3,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            EntityIdentityLifecycle::Reserved => "Reserved",
            EntityIdentityLifecycle::Alive => "Alive",
            EntityIdentityLifecycle::Tombstoned => "Tombstoned",
            EntityIdentityLifecycle::Destroyed => "Destroyed",
        }
    }
}

/// Ordinal authority: ids/index.json:MessageType.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicationEnvelopeMessageType {
    Handshake,
    FullSnapshot,
    BaselineAck,
    Delta,
    DeltaAck,
    ResyncRequest,
    MaintenanceKick,
    Error,
}

impl ReplicationEnvelopeMessageType {
    pub const fn ordinal(self) -> u32 {
        match self {
            ReplicationEnvelopeMessageType::Handshake => 1,
            ReplicationEnvelopeMessageType::FullSnapshot => 2,
            ReplicationEnvelopeMessageType::BaselineAck => 6,
            ReplicationEnvelopeMessageType::Delta => 3,
            ReplicationEnvelopeMessageType::DeltaAck => 7,
            ReplicationEnvelopeMessageType::ResyncRequest => 4,
            ReplicationEnvelopeMessageType::MaintenanceKick => 5,
            ReplicationEnvelopeMessageType::Error => 8,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ReplicationEnvelopeMessageType::Handshake => "Handshake",
            ReplicationEnvelopeMessageType::FullSnapshot => "FullSnapshot",
            ReplicationEnvelopeMessageType::BaselineAck => "BaselineAck",
            ReplicationEnvelopeMessageType::Delta => "Delta",
            ReplicationEnvelopeMessageType::DeltaAck => "DeltaAck",
            ReplicationEnvelopeMessageType::ResyncRequest => "ResyncRequest",
            ReplicationEnvelopeMessageType::MaintenanceKick => "MaintenanceKick",
            ReplicationEnvelopeMessageType::Error => "Error",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicationEnvelopeReliability {
    Reliable,
    Unreliable,
}

impl ReplicationEnvelopeReliability {
    pub const fn ordinal(self) -> u32 {
        match self {
            ReplicationEnvelopeReliability::Reliable => 0,
            ReplicationEnvelopeReliability::Unreliable => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ReplicationEnvelopeReliability::Reliable => "Reliable",
            ReplicationEnvelopeReliability::Unreliable => "Unreliable",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicationEnvelopeIntegrityAlgorithm {
    None,
    CRC32C,
    SHA256,
    AEAD,
}

impl ReplicationEnvelopeIntegrityAlgorithm {
    pub const fn ordinal(self) -> u32 {
        match self {
            ReplicationEnvelopeIntegrityAlgorithm::None => 0,
            ReplicationEnvelopeIntegrityAlgorithm::CRC32C => 1,
            ReplicationEnvelopeIntegrityAlgorithm::SHA256 => 2,
            ReplicationEnvelopeIntegrityAlgorithm::AEAD => 3,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ReplicationEnvelopeIntegrityAlgorithm::None => "None",
            ReplicationEnvelopeIntegrityAlgorithm::CRC32C => "CRC32C",
            ReplicationEnvelopeIntegrityAlgorithm::SHA256 => "SHA256",
            ReplicationEnvelopeIntegrityAlgorithm::AEAD => "AEAD",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicationEnvelopeTransportPolicyAuthBinding {
    SessionAdmission,
    ConnectionGeneration,
}

impl ReplicationEnvelopeTransportPolicyAuthBinding {
    pub const fn ordinal(self) -> u32 {
        match self {
            ReplicationEnvelopeTransportPolicyAuthBinding::SessionAdmission => 0,
            ReplicationEnvelopeTransportPolicyAuthBinding::ConnectionGeneration => 1,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ReplicationEnvelopeTransportPolicyAuthBinding::SessionAdmission => "SessionAdmission",
            ReplicationEnvelopeTransportPolicyAuthBinding::ConnectionGeneration => "ConnectionGeneration",
        }
    }
}

/// Ordinal authority: schema declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicationEnvelopeTransportPolicyErrorClass {
    Retryable,
    Rejectable,
    Fatal,
}

impl ReplicationEnvelopeTransportPolicyErrorClass {
    pub const fn ordinal(self) -> u32 {
        match self {
            ReplicationEnvelopeTransportPolicyErrorClass::Retryable => 0,
            ReplicationEnvelopeTransportPolicyErrorClass::Rejectable => 1,
            ReplicationEnvelopeTransportPolicyErrorClass::Fatal => 2,
        }
    }

    /// The value that crosses the wire, which the identifier may not equal.
    pub const fn wire_value(self) -> &'static str {
        match self {
            ReplicationEnvelopeTransportPolicyErrorClass::Retryable => "Retryable",
            ReplicationEnvelopeTransportPolicyErrorClass::Rejectable => "Rejectable",
            ReplicationEnvelopeTransportPolicyErrorClass::Fatal => "Fatal",
        }
    }
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigTableColumnsItem {
    pub name: String,
    pub r#type: ConfigTableColumnsItemType,
    pub required: bool,
    /// Optional in the schema.
    pub minimum: Option<f64>,
    /// Optional in the schema.
    pub maximum: Option<f64>,
    /// Optional in the schema.
    pub enum_values: Option<Vec<String>>,
    /// Optional in the schema.
    pub ref_target: Option<String>,
    /// Optional in the schema.
    pub default_value: Option<OpaqueJson>,
}

impl ConfigTableColumnsItem {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["name", "type", "required", "minimum", "maximum", "enumValues", "refTarget", "defaultValue"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigTableRowsItem {
    pub key: String,
    pub values: OpaqueJson,
}

impl ConfigTableRowsItem {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["key", "values"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigTable {
    pub table_id: String,
    pub schema_version: u64,
    pub config_revision: u64,
    pub generated_at: String,
    pub source_hash: String,
    pub columns: Vec<ConfigTableColumnsItem>,
    pub rows: Vec<ConfigTableRowsItem>,
    pub activation: ConfigTableActivation,
    /// Optional in the schema.
    pub signature: Option<String>,
}

impl ConfigTable {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["tableId", "schemaVersion", "configRevision", "generatedAt", "sourceHash", "columns", "rows", "activation", "signature"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessorDescriptorBudget {
    pub max_micros: u64,
    pub max_commands: u64,
}

impl ProcessorDescriptorBudget {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["maxMicros", "maxCommands"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessorDescriptor {
    pub processor_id: String,
    pub role: ProcessorDescriptorRole,
    pub phase: ProcessorDescriptorPhase,
    pub query: String,
    pub read_set: Vec<String>,
    pub write_set: Vec<String>,
    pub may_emit_structural_commands: bool,
    /// Optional in the schema.
    pub before: Option<Vec<String>>,
    /// Optional in the schema.
    pub after: Option<Vec<String>>,
    pub determinism_class: ProcessorDescriptorDeterminismClass,
    pub budget: ProcessorDescriptorBudget,
    pub diagnostic_name: String,
}

impl ProcessorDescriptor {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["processorId", "role", "phase", "query", "readSet", "writeSet", "mayEmitStructuralCommands", "before", "after", "determinismClass", "budget", "diagnosticName"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct TxnJournalRecord {
    pub record_version: u64,
    pub record_seq: u64,
    pub previous_hash: String,
    pub payload_hash: String,
    pub length: u64,
    pub checksum: String,
    pub commit_state: TxnJournalRecordCommitState,
    pub durability_state: TxnJournalRecordDurabilityState,
    pub session_id: String,
    pub game_release_id: String,
    pub tick_id: u64,
    pub txn_id: String,
    /// Optional in the schema.
    pub command_id: Option<String>,
    pub record_kind: TxnJournalRecordRecordKind,
    pub idempotency_key: String,
}

impl TxnJournalRecord {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["recordVersion", "recordSeq", "previousHash", "payloadHash", "length", "checksum", "commitState", "durabilityState", "sessionId", "gameReleaseId", "tickId", "txnId", "commandId", "recordKind", "idempotencyKey"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandLogRecord {
    pub record_version: u64,
    pub record_seq: u64,
    pub previous_hash: String,
    pub payload_hash: String,
    pub length: u64,
    pub checksum: String,
    pub commit_state: CommandLogRecordCommitState,
    pub durability_state: CommandLogRecordDurabilityState,
    pub session_id: String,
    pub game_release_id: String,
    pub tick_id: u64,
    /// Optional in the schema.
    pub txn_id: Option<String>,
    pub command_id: String,
    pub record_kind: CommandLogRecordRecordKind,
    pub idempotency_key: String,
}

impl CommandLogRecord {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["recordVersion", "recordSeq", "previousHash", "payloadHash", "length", "checksum", "commitState", "durabilityState", "sessionId", "gameReleaseId", "tickId", "txnId", "commandId", "recordKind", "idempotencyKey"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct WalRecordEnvelope {
    pub record_version: u64,
    pub record_seq: u64,
    pub previous_hash: String,
    pub payload_hash: String,
    pub length: u64,
    pub checksum: String,
    pub inner_kind: WalRecordEnvelopeInnerKind,
    pub inner: OpaqueJson,
}

impl WalRecordEnvelope {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["recordVersion", "recordSeq", "previousHash", "payloadHash", "length", "checksum", "innerKind", "inner"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityIdentity {
    pub net_entity_id: String,
    pub authority_domain: String,
    pub world_epoch: u64,
    pub sequence: u64,
    pub generation: u64,
    /// Optional in the schema.
    pub local_entity_id: Option<String>,
    pub namespace: EntityIdentityNamespace,
    pub lifecycle: EntityIdentityLifecycle,
    /// Optional in the schema.
    pub tombstone_until_revision: Option<u64>,
    /// Optional in the schema.
    pub remapped_from: Option<String>,
    /// Optional in the schema.
    pub source_revision: Option<u64>,
    /// Optional in the schema.
    pub source_release_id: Option<String>,
}

impl EntityIdentity {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["netEntityId", "authorityDomain", "worldEpoch", "sequence", "generation", "localEntityId", "namespace", "lifecycle", "tombstoneUntilRevision", "remappedFrom", "sourceRevision", "sourceReleaseId"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplicationEnvelopeIntegrity {
    pub algorithm: ReplicationEnvelopeIntegrityAlgorithm,
    pub value: String,
}

impl ReplicationEnvelopeIntegrity {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["algorithm", "value"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplicationEnvelopeTransportPolicy {
    pub max_message_bytes: u64,
    pub max_fragment_bytes: u64,
    pub anti_replay_window: u64,
    pub auth_binding: ReplicationEnvelopeTransportPolicyAuthBinding,
    pub error_class: ReplicationEnvelopeTransportPolicyErrorClass,
}

impl ReplicationEnvelopeTransportPolicy {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["maxMessageBytes", "maxFragmentBytes", "antiReplayWindow", "authBinding", "errorClass"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplicationEnvelope {
    pub session_id: String,
    pub product_id: String,
    pub game_release_id: String,
    pub protocol_version: u64,
    pub length: u64,
    pub sequence: u64,
    pub message_type: ReplicationEnvelopeMessageType,
    pub reliability: ReplicationEnvelopeReliability,
    pub integrity: ReplicationEnvelopeIntegrity,
    pub trace_id: String,
    pub transport_policy: ReplicationEnvelopeTransportPolicy,
    pub body: OpaqueJson,
}

impl ReplicationEnvelope {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["sessionId", "productId", "gameReleaseId", "protocolVersion", "length", "sequence", "messageType", "reliability", "integrity", "traceId", "transportPolicy", "body"];
}

/// Fields in schema declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionRevisionVector {
    pub tick_id: u64,
    pub game_revision: u64,
    pub voxel_world_revision: u64,
    pub chunk_revision_set: BTreeMap<String, u64>,
    pub replication_revision: u64,
    pub config_revision: u64,
    pub schema_epoch: u64,
}

impl SessionRevisionVector {
    /// The schema declaration order this type was generated from.
    pub const FIELD_ORDER: &'static [&'static str] = &["tickId", "gameRevision", "voxelWorldRevision", "chunkRevisionSet", "replicationRevision", "configRevision", "schemaEpoch"];
}

