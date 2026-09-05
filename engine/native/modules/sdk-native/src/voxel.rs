use crate::abi_generated::{
    VoxelBlockReadCellResult, VoxelBlockReadResult, VoxelBlockWriteEntry, VoxelBoxRequest,
    VoxelColumnRequest, VoxelPresence, VoxelSectionKey, VoxelSectionRevisionResult,
    VoxelSectionSegment, VoxelWorldCoordinate, VoxelWriteReceipt,
};
use crate::LumioStatus;
use lumio_voxel_contracts::{sha256, BASELINE_ID, SCHEMA_EPOCH};
use lumio_voxel_domain::block::{BlockId, CellOffset};
use lumio_voxel_domain::config_snapshot::{
    DecisionEvidence, GateSourceHashes, GeneratedHostCapability, GeneratedVoxelConfig,
    VoxelConfigSnapshot, P0_DECISION_GATES,
};
use lumio_voxel_domain::publication::PublishedStateRoot;
use lumio_voxel_domain::revision::{GeneratedRevisionStamp, REVISION_STAMP_SCHEMA};
use lumio_voxel_domain::section::{
    SectionDeltaBuilder, SectionPage, SectionPayload, SectionPayloadEnvelope, SectionSlot,
    SectionStorage,
};
use lumio_voxel_ops::async_support::{OriginEnvelope, OriginToken};
use lumio_voxel_ops::mutation::{MutationEntry, MutationRequest, PreparedMutation};
use lumio_voxel_ops::query::{BlockReadSection, BlockReadWorld, GeneratedVoxelQueryRequest};
use lumio_voxel_world::port::GeneratedVoxelWorldPortAdapter;
use lumio_voxel_world::world::{
    PinBudget, PinId, RegionPinManager, VoxelWorld, WorldCommand, WorldConfigAdapter,
    WorldDescriptor,
};
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SectionKey(i32, u8, i32);

impl SectionKey {
    fn from_abi(key: VoxelSectionKey) -> Result<Self, &'static str> {
        if key.y > 15 {
            return Err("section_y_out_of_range");
        }
        Ok(Self(key.x, key.y, key.z))
    }

    fn to_abi(self) -> VoxelSectionKey {
        VoxelSectionKey::new(self.0, self.1, self.2)
    }

    fn id(self) -> String {
        format!("s:{}:{}:{}", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug)]
struct SectionState {
    presence: VoxelPresence,
    revision: u64,
    storage: Option<SectionStorage>,
}

#[derive(Clone, Copy, Debug)]
struct WriteEntry {
    section: SectionKey,
    offset: u16,
    block_id: u32,
}

struct PreparedToken {
    transaction_id: u64,
    request: MutationRequest,
    entries: Vec<WriteEntry>,
    mutation: Option<PreparedMutation>,
    receipts: Vec<VoxelWriteReceipt>,
    terminal_error: Option<&'static str>,
}

struct PinToken {
    id: PinId,
}

static NEXT_PROVIDER_ID: AtomicU64 = AtomicU64::new(1);

/// Native-owned integration adapter over the paired VoxelEngine public APIs.
///
/// `VoxelWorld` owns lifecycle, publication, query routing, and mutation routing. The
/// immutable `BlockReadWorld` is rebuilt from the same Section state after publication so the
/// ABI's caller-buffer shape can use the paired block-read API without leaking ownership.
pub struct NativeVoxelProvider {
    world: VoxelWorld,
    block_world: BlockReadWorld,
    sections: BTreeMap<SectionKey, SectionState>,
    prepared: BTreeMap<usize, Box<PreparedToken>>,
    transactions: BTreeMap<u64, usize>,
    pin_tokens: BTreeMap<usize, Box<PinToken>>,
    next_query_id: u64,
}

impl Default for NativeVoxelProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeVoxelProvider {
    pub fn new() -> Self {
        let provider_id = NEXT_PROVIDER_ID.fetch_add(1, Ordering::Relaxed);
        let world_id = format!("native-voxel-world-{provider_id}");
        let context_id = format!("native-voxel-context-{provider_id}");
        let snapshot = approved_snapshot("lumio-engine-native");
        let mut world = VoxelWorld::create(
            WorldDescriptor {
                role: "Authority".to_string(),
                world_context_id: context_id,
                capabilities: vec!["Native".to_string(), "ReferenceVoxel".to_string()],
                config: WorldConfigAdapter { world_id },
            },
            snapshot,
        )
        .expect("approved paired VoxelEngine snapshot must create a world");
        world.set_region_pin_manager(RegionPinManager::from_budget(PinBudget::new(
            usize::MAX,
            usize::MAX,
        )));
        drive_lifecycle(&mut world);
        Self {
            world,
            block_world: BlockReadWorld::new(),
            sections: BTreeMap::new(),
            prepared: BTreeMap::new(),
            transactions: BTreeMap::new(),
            pin_tokens: BTreeMap::new(),
            next_query_id: 1,
        }
    }

    pub fn world_state(&self) -> lumio_voxel_world::world::WorldStateView {
        self.world.state_view()
    }

    pub fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    pub fn seed_ready_section(&mut self, key: VoxelSectionKey, revision: u64, block_id: u32) {
        self.seed(
            key,
            SectionState {
                presence: VoxelPresence::Ready,
                revision,
                storage: Some(SectionStorage::uniform(BlockId::from_raw(block_id))),
            },
        );
    }

    pub fn seed_unchanged_section(&mut self, key: VoxelSectionKey, revision: u64, block_id: u32) {
        self.seed(
            key,
            SectionState {
                presence: VoxelPresence::Unchanged,
                revision,
                storage: Some(SectionStorage::uniform(BlockId::from_raw(block_id))),
            },
        );
    }

    pub fn seed_pending_section(&mut self, key: VoxelSectionKey, revision: u64) {
        self.seed_missing(key, revision, VoxelPresence::Pending);
    }

    pub fn seed_unavailable_section(&mut self, key: VoxelSectionKey, revision: u64) {
        self.seed_missing(key, revision, VoxelPresence::Unavailable);
    }

    fn seed_missing(&mut self, key: VoxelSectionKey, revision: u64, presence: VoxelPresence) {
        self.seed(
            key,
            SectionState {
                presence,
                revision,
                storage: None,
            },
        );
    }

    fn seed(&mut self, key: VoxelSectionKey, state: SectionState) {
        let key = SectionKey::from_abi(key).expect("seed key must be canonical");
        self.sections.insert(key, state);
        self.rebuild_block_world();
        // The seed helpers also model a residency update that may be rejected by the
        // world publication transition (for example Ready -> Pending after a pin is
        // ready). Keep the caller-buffer source at the attempted state so the ABI
        // guard proves that such a stale/missing result is rejected rather than leaked.
        if let Err(error) = self.publish_section(key) {
            let attempted_missing = self.sections.get(&key).is_some_and(|state| {
                matches!(
                    state.presence,
                    VoxelPresence::Pending | VoxelPresence::Unavailable
                )
            }) && self
                .world
                .region_pin_manager()
                .is_some_and(|manager| manager.validate_presence(&key.id(), "Pending").is_err());
            if !attempted_missing {
                panic!("seed publication through paired VoxelEngine must succeed: {error}");
            }
        }
    }

    fn rebuild_block_world(&mut self) {
        let entries = self.sections.iter().map(|(key, state)| {
            let section = BlockReadSection::from_parts(
                presence_name(state.presence),
                Some(state.revision),
                state.storage.clone(),
            )
            .expect("ABI presence and SectionStorage must satisfy BlockReadWorld");
            (key.id(), section)
        });
        self.block_world = BlockReadWorld::from_sections(entries)
            .expect("canonical adapter Section ids must build BlockReadWorld");
    }

    fn publish_section(&mut self, changed: SectionKey) -> Result<(), &'static str> {
        let view = self.world.publication_authority().capture();
        let mut directory = lumio_voxel_domain::section::SectionDirectoryBuilder::new();
        let mut revisions = BTreeMap::new();
        for (key, state) in &self.sections {
            directory
                .insert(&key.id(), section_slot(*key, state)?)
                .map_err(|e| e.error_id())?;
            revisions.insert(key.id(), state.revision);
        }
        let stamp = GeneratedRevisionStamp {
            schema_id: REVISION_STAMP_SCHEMA,
            world_id: view.stamp().world_id.clone(),
            context_id: view.stamp().context_id.clone(),
            generation: view.stamp().generation,
            world_revision: view.stamp().world_revision,
            section_revision_set: revisions,
        };
        let root =
            PublishedStateRoot::new(stamp, directory.freeze(), view.dirty_frontier().clone());
        let state = self.sections.get(&changed).ok_or("unknown_section_key")?;
        let mut delta = SectionDeltaBuilder::new(view.directory());
        delta
            .stage((changed.id(), section_slot(changed, state)?))
            .map_err(|e| e.error_id())?;
        let replacement = delta.freeze().map_err(|e| e.error_id())?;
        let mut publication = self
            .world
            .publication_authority()
            .prepare(
                world_revision(view.stamp().world_revision)?,
                root,
                replacement,
            )
            .map_err(|e| e.error_id())?;
        let token = publication.seal().map_err(|e| e.error_id())?;
        self.world
            .publication_authority()
            .publish_once(token)
            .map_err(|e| e.error_id())?;
        Ok(())
    }

    fn read_cell(
        &mut self,
        coordinate: VoxelWorldCoordinate,
    ) -> Result<VoxelBlockReadResult, &'static str> {
        let section = coordinate_section(coordinate)?;
        self.ensure_world_query(&[section])?;
        self.read_cell_cached(coordinate)
    }

    fn read_cell_cached(
        &mut self,
        coordinate: VoxelWorldCoordinate,
    ) -> Result<VoxelBlockReadResult, &'static str> {
        let mut block_id = None;
        let result = if let Some(manager) = self.world.region_pin_manager() {
            self.block_world.read_cell_into_with_presence_guard(
                coordinate.x,
                i64::from(coordinate.y),
                coordinate.z,
                &mut block_id,
                manager,
            )
        } else {
            self.block_world.read_cell_into(
                coordinate.x,
                i64::from(coordinate.y),
                coordinate.z,
                &mut block_id,
            )
        }
        .map_err(|error| error.error_id())?;
        let presence = parse_presence(result.presence())?;
        Ok(VoxelBlockReadResult {
            presence,
            has_block_id: u8::from(block_id.is_some()),
            _reserved: [0; 3],
            block_id: block_id.map_or(0, BlockId::raw),
            section_revision: result.section_revision(),
        })
    }

    fn ensure_world_query(&mut self, sections: &[SectionKey]) -> Result<(), &'static str> {
        let state = self.world.state_view();
        let mut ids = sections.iter().map(|key| key.id()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        for chunk in ids.chunks(16) {
            let query_id = format!("native-query-{}", self.next_query_id);
            self.next_query_id = self.next_query_id.saturating_add(1);
            let origin = OriginToken::try_new(
                state.world_context_id().to_string(),
                state.instance_generation(),
                query_id.clone(),
                0,
                BTreeMap::new(),
                "VoxelCommit",
            )
            .map_err(|error| canonical_error_id(error.error_id()))?;
            let config_hash = self.world.config_hash().to_string();
            GeneratedVoxelWorldPortAdapter::new(&mut self.world)
                .query(OriginEnvelope {
                    origin,
                    config_hash,
                    payload: GeneratedVoxelQueryRequest {
                        query_id,
                        world_id: state.world_id().to_string(),
                        context: state.world_context_id().to_string(),
                        section_ids: chunk.to_vec(),
                        cancel: false,
                    },
                })
                .map(|_| ())
                .map_err(|error| canonical_error_id(error.error_id()))?;
        }
        Ok(())
    }

    fn ensure_world_query_range(
        &mut self,
        min: VoxelWorldCoordinate,
        max: VoxelWorldCoordinate,
    ) -> Result<(), &'static str> {
        let mut batch = [SectionKey(0, 0, 0); 16];
        let mut used = 0_usize;
        for y in (min.y / 16)..=(max.y / 16) {
            for z in min.z.div_euclid(16)..=max.z.div_euclid(16) {
                for x in min.x.div_euclid(16)..=max.x.div_euclid(16) {
                    batch[used] = SectionKey(x, y, z);
                    used += 1;
                    if used == batch.len() {
                        self.ensure_world_query(&batch)?;
                        used = 0;
                    }
                }
            }
        }
        if used > 0 {
            self.ensure_world_query(&batch[..used])?;
        }
        Ok(())
    }
}

fn approved_snapshot(label: &str) -> Arc<VoxelConfigSnapshot> {
    let source = GateSourceHashes {
        architecture_baseline_id: BASELINE_ID.to_string(),
        voxel_head: "b2f0d8a3763a02f805e29cbd101560ba7fdca77b".to_string(),
        architecture_mirror_sha256:
            "f1d36acf33a1f5e8326a9e58d609fcf7d9fa85177f9b5b60bb3f4742c1afebd0".to_string(),
        v13_decision_gates_sha256:
            "4850057dd8926c11c8c3beebe109d18dffdb7e84cd451426d7d635860be5ede2".to_string(),
        blueprint_sha256: "32e76066eb298aad20f4149760abbeddacb6d6c43e096945f1cf0ea75b2471aa"
            .to_string(),
    };
    let digests = P0_DECISION_GATES
        .iter()
        .map(|gate| {
            (
                (*gate).to_string(),
                hex32(&sha256(format!("approved-{gate}").as_bytes())),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let evidence = P0_DECISION_GATES
        .iter()
        .map(|gate| DecisionEvidence {
            gate_id: (*gate).to_string(),
            approval_status: "approved".to_string(),
            source_hashes: source.clone(),
            evidence_digest: digests[*gate].clone(),
        })
        .collect::<Vec<_>>();
    let config = GeneratedVoxelConfig {
        schema_id: "config-table",
        host_capability_schema_id: "host-capability",
        schema_epoch: SCHEMA_EPOCH,
        config_hash: hex32(&sha256(label.as_bytes())),
        gate_source_hashes: digests,
        host_capability: GeneratedHostCapability::from_names(["Native", "ReferenceVoxel"]),
        start_capabilities: vec!["Native".to_string(), "ReferenceVoxel".to_string()],
        key_material: None,
    };
    VoxelConfigSnapshot::from_generated(&config, &evidence)
        .expect("paired VoxelEngine config snapshot must be approved")
}

fn drive_lifecycle(world: &mut VoxelWorld) {
    for (event, to) in [
        ("Initialize", "Initialized"),
        ("Prime", "Ready"),
        ("Start", "Running"),
    ] {
        let state = world.state_view();
        let origin = OriginToken::try_new(
            state.world_context_id().to_string(),
            state.instance_generation(),
            format!("native-lifecycle-{event}"),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        )
        .expect("lifecycle origin");
        world
            .endpoint()
            .admit(WorldCommand::Lifecycle { event, to, origin })
            .expect("paired VoxelEngine lifecycle must admit");
    }
}

fn world_revision(n: u64) -> Result<lumio_voxel_domain::revision::WorldRevision, &'static str> {
    Ok(lumio_voxel_domain::revision::WorldRevision::from_raw(n))
}

fn section_slot(key: SectionKey, state: &SectionState) -> Result<SectionSlot, &'static str> {
    match state.presence {
        VoxelPresence::Ready => {
            let storage = state.storage.as_ref().ok_or("section_unavailable")?;
            let id = lumio_voxel_domain::key::SectionId::parse(&key.id())
                .map_err(|_| "unknown_section_key")?;
            let envelope = SectionPayloadEnvelope::encode_full(id, state.revision, storage);
            let payload = SectionPayload::from_pages_with_storage(
                [SectionPage::new(
                    "Dense",
                    "None",
                    envelope.payload().to_vec(),
                    sha256(envelope.payload()),
                )],
                Some(storage.clone()),
            )
            .map_err(|error| error.error_id())?;
            Ok(SectionSlot::ready(payload))
        }
        VoxelPresence::Unchanged => Ok(SectionSlot::unchanged()),
        VoxelPresence::Pending => Ok(SectionSlot::pending()),
        VoxelPresence::Unavailable => Ok(SectionSlot::unavailable()),
    }
}

fn presence_name(presence: VoxelPresence) -> &'static str {
    match presence {
        VoxelPresence::Ready => "Ready",
        VoxelPresence::Unchanged => "Unchanged",
        VoxelPresence::Pending => "Pending",
        VoxelPresence::Unavailable => "Unavailable",
    }
}

fn parse_presence(name: &str) -> Result<VoxelPresence, &'static str> {
    match name {
        "Ready" => Ok(VoxelPresence::Ready),
        "Unchanged" => Ok(VoxelPresence::Unchanged),
        "Pending" => Ok(VoxelPresence::Pending),
        "Unavailable" => Ok(VoxelPresence::Unavailable),
        _ => Err("cell_read_missing_presence"),
    }
}

fn coordinate_section(coordinate: VoxelWorldCoordinate) -> Result<SectionKey, &'static str> {
    Ok(SectionKey(
        coordinate.x.div_euclid(16),
        coordinate.y / 16,
        coordinate.z.div_euclid(16),
    ))
}

fn provider<'a>(world: *mut c_void) -> Result<&'a mut NativeVoxelProvider, i32> {
    if world.is_null() {
        return Err(LumioStatus::InvalidArgument as i32);
    }
    // SAFETY: the opaque pointer is created by NativeVoxelProvider::as_opaque_ptr.
    Ok(unsafe { &mut *world.cast::<NativeVoxelProvider>() })
}

fn canonical_error_id(error: &str) -> &'static str {
    match error {
        "unknown_section_key" => "unknown_section_key",
        "unknown_chunk_key" => "unknown_chunk_key",
        "section_y_out_of_range" => "section_y_out_of_range",
        "coordinate_out_of_bounds" => "coordinate_out_of_bounds",
        "section_unavailable" => "section_unavailable",
        "stale_section_revision" => "stale_section_revision",
        "read_budget_exceeded" => "read_budget_exceeded",
        "read_result_missing_revision" => "read_result_missing_revision",
        "write_batch_too_large" => "write_batch_too_large",
        "unstructured_mutation_entry" => "unstructured_mutation_entry",
        "cell_offset_out_of_range" => "cell_offset_out_of_range",
        "residency_pin_exceeds_budget" => "residency_pin_exceeds_budget",
        "pin_region_not_ready" => "pin_region_not_ready",
        "pinned_section_evicted" => "pinned_section_evicted",
        "pinned_read_returned_pending" => "pinned_read_returned_pending",
        "world_y_out_of_range" => "world_y_out_of_range",
        "cell_read_missing_presence" => "cell_read_missing_presence",
        _ => "InvalidHandle",
    }
}

fn status_for_error(error: &str) -> i32 {
    match error {
        "unknown_section_key" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_SECTION_KEY,
        "unknown_chunk_key" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_CHUNK_KEY,
        "section_y_out_of_range" => crate::abi_generated::VOXEL_ERROR_SECTION_Y_OUT_OF_RANGE,
        "coordinate_out_of_bounds" => crate::abi_generated::VOXEL_ERROR_COORDINATE_OUT_OF_BOUNDS,
        "section_unavailable" => crate::abi_generated::VOXEL_ERROR_SECTION_UNAVAILABLE,
        "stale_section_revision" => crate::abi_generated::VOXEL_ERROR_STALE_SECTION_REVISION,
        "palette_overflow" => crate::abi_generated::VOXEL_ERROR_PALETTE_OVERFLOW,
        "section_encoding_mismatch" => crate::abi_generated::VOXEL_ERROR_SECTION_ENCODING_MISMATCH,
        "section_digest_mismatch" => crate::abi_generated::VOXEL_ERROR_SECTION_DIGEST_MISMATCH,
        "dirty_section_not_durable" => crate::abi_generated::VOXEL_ERROR_DIRTY_SECTION_NOT_DURABLE,
        "lighting_in_payload" => crate::abi_generated::VOXEL_ERROR_LIGHTING_IN_PAYLOAD,
        "chunk_carries_data" => crate::abi_generated::VOXEL_ERROR_CHUNK_CARRIES_DATA,
        "unknown_material_class" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_MATERIAL_CLASS,
        "material_class_not_a_cell_lane" => {
            crate::abi_generated::VOXEL_ERROR_MATERIAL_CLASS_NOT_A_CELL_LANE
        }
        "liquid_auto_propagation_unsupported" => {
            crate::abi_generated::VOXEL_ERROR_LIQUID_AUTO_PROPAGATION_UNSUPPORTED
        }
        "cross_material_face_merge" => crate::abi_generated::VOXEL_ERROR_CROSS_MATERIAL_FACE_MERGE,
        "entity_binding_missing" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_MISSING,
        "entity_binding_orphan" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_ORPHAN,
        "entity_binding_type_mismatch" => {
            crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_TYPE_MISMATCH
        }
        "entity_binding_not_sparse" => crate::abi_generated::VOXEL_ERROR_ENTITY_BINDING_NOT_SPARSE,
        "business_data_in_payload" => crate::abi_generated::VOXEL_ERROR_BUSINESS_DATA_IN_PAYLOAD,
        "binding_commit_split" => crate::abi_generated::VOXEL_ERROR_BINDING_COMMIT_SPLIT,
        "block_type_scope_violation" => {
            crate::abi_generated::VOXEL_ERROR_BLOCK_TYPE_SCOPE_VIOLATION
        }
        "system_reserved_type_misuse" => {
            crate::abi_generated::VOXEL_ERROR_SYSTEM_RESERVED_TYPE_MISUSE
        }
        "room_local_type_without_mapping" => {
            crate::abi_generated::VOXEL_ERROR_ROOM_LOCAL_TYPE_WITHOUT_MAPPING
        }
        "player_type_declares_behavior" => {
            crate::abi_generated::VOXEL_ERROR_PLAYER_TYPE_DECLARES_BEHAVIOR
        }
        "palette_reclaim_before_escalation" => {
            crate::abi_generated::VOXEL_ERROR_PALETTE_RECLAIM_BEFORE_ESCALATION
        }
        "dead_palette_entry_in_payload" => {
            crate::abi_generated::VOXEL_ERROR_DEAD_PALETTE_ENTRY_IN_PAYLOAD
        }
        "delta_base_revision_mismatch" => {
            crate::abi_generated::VOXEL_ERROR_DELTA_BASE_REVISION_MISMATCH
        }
        "delta_used_for_first_delivery" => {
            crate::abi_generated::VOXEL_ERROR_DELTA_USED_FOR_FIRST_DELIVERY
        }
        "unresolved_hit_treated_as_air" => {
            crate::abi_generated::VOXEL_ERROR_UNRESOLVED_HIT_TREATED_AS_AIR
        }
        "unresolved_hit_treated_as_solid" => {
            crate::abi_generated::VOXEL_ERROR_UNRESOLVED_HIT_TREATED_AS_SOLID
        }
        "query_buffer_overflow" => crate::abi_generated::VOXEL_ERROR_QUERY_BUFFER_OVERFLOW,
        "query_result_divergence" => crate::abi_generated::VOXEL_ERROR_QUERY_RESULT_DIVERGENCE,
        "collision_behavior_not_from_material_table" => {
            crate::abi_generated::VOXEL_ERROR_COLLISION_BEHAVIOR_NOT_FROM_MATERIAL_TABLE
        }
        "query_mutates_world" => crate::abi_generated::VOXEL_ERROR_QUERY_MUTATES_WORLD,
        "world_y_out_of_range" => crate::abi_generated::VOXEL_ERROR_WORLD_Y_OUT_OF_RANGE,
        "block_catalog_not_dense" => crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_NOT_DENSE,
        "block_catalog_name_reused" => crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_NAME_REUSED,
        "block_catalog_row_incomplete" => {
            crate::abi_generated::VOXEL_ERROR_BLOCK_CATALOG_ROW_INCOMPLETE
        }
        "read_budget_exceeded" => crate::abi_generated::VOXEL_ERROR_READ_BUDGET_EXCEEDED,
        "read_result_missing_revision" => {
            crate::abi_generated::VOXEL_ERROR_READ_RESULT_MISSING_REVISION
        }
        "write_batch_too_large" => crate::abi_generated::VOXEL_ERROR_WRITE_BATCH_TOO_LARGE,
        "unstructured_mutation_entry" => {
            crate::abi_generated::VOXEL_ERROR_UNSTRUCTURED_MUTATION_ENTRY
        }
        "cell_offset_out_of_range" => crate::abi_generated::VOXEL_ERROR_CELL_OFFSET_OUT_OF_RANGE,
        "residency_pin_exceeds_budget" => {
            crate::abi_generated::VOXEL_ERROR_RESIDENCY_PIN_EXCEEDS_BUDGET
        }
        "pin_region_not_ready" => crate::abi_generated::VOXEL_ERROR_PIN_REGION_NOT_READY,
        "pinned_section_evicted" => crate::abi_generated::VOXEL_ERROR_PINNED_SECTION_EVICTED,
        "pinned_read_returned_pending" => {
            crate::abi_generated::VOXEL_ERROR_PINNED_READ_RETURNED_PENDING
        }
        "unknown_behavior_template" => crate::abi_generated::VOXEL_ERROR_UNKNOWN_BEHAVIOR_TEMPLATE,
        "cell_read_missing_presence" => {
            crate::abi_generated::VOXEL_ERROR_CELL_READ_MISSING_PRESENCE
        }
        "unregistered_block_type" => {
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE
        }
        _ => LumioStatus::InvalidArgument as i32,
    }
}

fn ffi(call: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(call)).unwrap_or(LumioStatus::InvalidArgument as i32)
}

pub unsafe extern "C" fn block_read_cell(
    world: *mut c_void,
    coordinate: *const VoxelWorldCoordinate,
    out: *mut VoxelBlockReadCellResult,
) -> i32 {
    ffi(|| {
        if coordinate.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let result = match provider.read_cell(unsafe { *coordinate }) {
            Ok(result) => result,
            Err(error) => return status_for_error(error),
        };
        unsafe {
            (*out).presence = result.presence;
            (*out).has_block_id = result.has_block_id;
            (*out).section_revision = result.section_revision;
            if result.has_block_id != 0 {
                (*out).block_id = result.block_id;
            }
        }
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_read_box(
    world: *mut c_void,
    request: *const c_void,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    batch_read(
        world,
        request,
        BatchKind::Box,
        out_cells,
        cell_capacity,
        out_cell_count,
        out_segments,
        segment_capacity,
        out_segment_count,
        out_truncated,
    )
}

pub unsafe extern "C" fn block_read_column(
    world: *mut c_void,
    request: *const c_void,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    batch_read(
        world,
        request,
        BatchKind::Column,
        out_cells,
        cell_capacity,
        out_cell_count,
        out_segments,
        segment_capacity,
        out_segment_count,
        out_truncated,
    )
}

#[derive(Clone, Copy)]
enum BatchKind {
    Box,
    Column,
}

#[allow(clippy::too_many_arguments)]
unsafe fn batch_read(
    world: *mut c_void,
    request: *const c_void,
    kind: BatchKind,
    out_cells: *mut VoxelBlockReadResult,
    cell_capacity: u32,
    out_cell_count: *mut u32,
    out_segments: *mut VoxelSectionSegment,
    segment_capacity: u32,
    out_segment_count: *mut u32,
    out_truncated: *mut u8,
) -> i32 {
    ffi(|| {
        if request.is_null()
            || out_cell_count.is_null()
            || out_segment_count.is_null()
            || out_truncated.is_null()
        {
            return LumioStatus::InvalidArgument as i32;
        }
        if (cell_capacity > 0 && out_cells.is_null())
            || (segment_capacity > 0 && out_segments.is_null())
        {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let (min, max) = match kind {
            BatchKind::Box => {
                let req = unsafe { &*request.cast::<VoxelBoxRequest>() };
                if req.min.x > req.max.x || req.min.y > req.max.y || req.min.z > req.max.z {
                    return LumioStatus::InvalidArgument as i32;
                }
                let count = (i64::from(req.max.x) - i64::from(req.min.x) + 1)
                    .checked_mul(i64::from(req.max.y) - i64::from(req.min.y) + 1)
                    .and_then(|n| n.checked_mul(i64::from(req.max.z) - i64::from(req.min.z) + 1));
                if !matches!(
                    count,
                    Some(n)
                        if n >= 0
                            && n <= i64::from(
                                crate::abi_generated::VOXEL_MAX_CELLS_PER_READ_REQUEST
                            )
                ) {
                    return status_for_error("read_budget_exceeded");
                }
                (req.min, req.max)
            }
            BatchKind::Column => {
                let req = unsafe { &*request.cast::<VoxelColumnRequest>() };
                if req.min_y > req.max_y {
                    return LumioStatus::InvalidArgument as i32;
                }
                let count = usize::from(req.max_y - req.min_y) + 1;
                if count > crate::abi_generated::VOXEL_MAX_CELLS_PER_READ_REQUEST as usize {
                    return status_for_error("read_budget_exceeded");
                }
                (
                    VoxelWorldCoordinate::new(req.x, req.min_y, req.z),
                    VoxelWorldCoordinate::new(req.x, req.max_y, req.z),
                )
            }
        };
        if let Err(error) = provider.ensure_world_query_range(min, max) {
            return status_for_error(error);
        }
        // Validate the complete immutable source before invoking callbacks so a rejected
        // ready-pin read cannot expose a partial caller-buffer result.
        if let Some(manager) = provider.world.region_pin_manager() {
            let guarded = provider.block_world.read_box_with_presence_guard(
                (min.x, i64::from(min.y), min.z),
                (max.x, i64::from(max.y), max.z),
                manager,
            );
            if let Err(error) = guarded {
                return status_for_error(error.error_id());
            }
        }

        let cell_limit = cell_capacity as usize;
        let segment_limit = segment_capacity as usize;
        let visit = match kind {
            BatchKind::Box => provider.block_world.visit_box(
                (min.x, i64::from(min.y), min.z),
                (max.x, i64::from(max.y), max.z),
                |index, cell| {
                    if index < cell_limit {
                        let presence = parse_presence(cell.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_cells.add(index).write(VoxelBlockReadResult {
                                presence,
                                has_block_id: u8::from(cell.block_id().is_some()),
                                _reserved: [0; 3],
                                block_id: cell.block_id().map_or(0, BlockId::raw),
                                section_revision: cell.section_revision(),
                            });
                        }
                    }
                },
                |index, segment| {
                    if index < segment_limit {
                        let id = segment.section_id();
                        let presence = parse_presence(segment.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_segments.add(index).write(VoxelSectionSegment {
                                section_key: VoxelSectionKey::new(id.x(), id.y(), id.z()),
                                presence,
                                section_revision: segment.section_revision(),
                                first_result: segment.first_cell() as u32,
                                result_count: segment.cell_count() as u32,
                            });
                        }
                    }
                },
            ),
            BatchKind::Column => provider.block_world.visit_column(
                min.x,
                min.z,
                i64::from(min.y)..=i64::from(max.y),
                |index, cell| {
                    if index < cell_limit {
                        let presence = parse_presence(cell.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_cells.add(index).write(VoxelBlockReadResult {
                                presence,
                                has_block_id: u8::from(cell.block_id().is_some()),
                                _reserved: [0; 3],
                                block_id: cell.block_id().map_or(0, BlockId::raw),
                                section_revision: cell.section_revision(),
                            });
                        }
                    }
                },
                |index, segment| {
                    if index < segment_limit {
                        let id = segment.section_id();
                        let presence = parse_presence(segment.presence())
                            .expect("BlockReadWorld only stores contract presences");
                        unsafe {
                            out_segments.add(index).write(VoxelSectionSegment {
                                section_key: VoxelSectionKey::new(id.x(), id.y(), id.z()),
                                presence,
                                section_revision: segment.section_revision(),
                                first_result: segment.first_cell() as u32,
                                result_count: segment.cell_count() as u32,
                            });
                        }
                    }
                },
            ),
        };
        let summary = match visit {
            Ok(summary) => summary,
            Err(error) => return status_for_error(error.error_id()),
        };
        unsafe {
            out_cell_count.write(summary.cell_count() as u32);
            out_segment_count.write(summary.segment_count() as u32);
            out_truncated.write(u8::from(
                summary.cell_count() > cell_limit || summary.segment_count() > segment_limit,
            ));
        }
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_write_prepare(
    world: *mut c_void,
    transaction_id: u64,
    entries: *const VoxelBlockWriteEntry,
    entry_count: u32,
    out_token: *mut *mut c_void,
) -> i32 {
    ffi(|| {
        if out_token.is_null() || (entry_count > 0 && entries.is_null()) {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        if entry_count > crate::abi_generated::VOXEL_MAX_ENTRIES_PER_WRITE_BATCH {
            return status_for_error("write_batch_too_large");
        }
        let state = provider.world.state_view();
        let input = if entry_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(entries, entry_count as usize) }
        };
        let mut mutation_entries = Vec::with_capacity(input.len());
        let mut owned_entries = Vec::with_capacity(input.len());
        for entry in input {
            let section = match SectionKey::from_abi(entry.section_key) {
                Ok(section) => section,
                Err(error) => return status_for_error(error),
            };
            let offset = match CellOffset::new(entry.cell_offset) {
                Ok(offset) => offset,
                Err(_) => return status_for_error("cell_offset_out_of_range"),
            };
            mutation_entries.push(MutationEntry::new(
                section.id(),
                offset,
                BlockId::from_raw(entry.block_id),
                entry.expected_section_revision,
            ));
            owned_entries.push(WriteEntry {
                section,
                offset: entry.cell_offset,
                block_id: entry.block_id,
            });
        }
        let request = MutationRequest::new(
            transaction_id.to_string(),
            state.world_id(),
            state.instance_generation(),
            mutation_entries,
        );
        if let Some(address) = provider.transactions.get(&transaction_id).copied() {
            if provider
                .prepared
                .get(&address)
                .is_some_and(|token| token.request == request)
            {
                unsafe { out_token.write(address as *mut c_void) };
                return LumioStatus::Success as i32;
            }
        }
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            transaction_id.to_string(),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        let mutation = match GeneratedVoxelWorldPortAdapter::new(&mut provider.world)
            .prepare_mutation(OriginEnvelope {
                origin,
                config_hash,
                payload: request.clone(),
            }) {
            Ok(envelope) => envelope.payload,
            Err(error) => return status_for_error(error.error_id()),
        };
        let mut token = Box::new(PreparedToken {
            transaction_id,
            request,
            entries: owned_entries,
            mutation: Some(mutation),
            receipts: Vec::new(),
            terminal_error: None,
        });
        let address = (&mut *token) as *mut PreparedToken as usize;
        provider.transactions.insert(transaction_id, address);
        provider.prepared.insert(address, token);
        unsafe { out_token.write(address as *mut c_void) };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn block_write_commit(
    world: *mut c_void,
    token: *mut c_void,
    out_receipts: *mut VoxelWriteReceipt,
    receipt_capacity: u32,
    out_receipt_count: *mut u32,
) -> i32 {
    ffi(|| {
        if token.is_null()
            || out_receipt_count.is_null()
            || (receipt_capacity > 0 && out_receipts.is_null())
        {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = token as usize;
        let required = match provider.prepared.get(&address) {
            Some(token) => token
                .entries
                .iter()
                .map(|entry| entry.section)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        if required > receipt_capacity as usize {
            unsafe { out_receipt_count.write(required as u32) };
            return LumioStatus::BufferTooSmall as i32;
        }
        let prepared = match provider.prepared.get_mut(&address) {
            Some(token) => token.mutation.take(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        let Some(prepared) = prepared else {
            if let Some(error) = provider.prepared[&address].terminal_error {
                return status_for_error(error);
            }
            let receipts = &provider.prepared[&address].receipts;
            return write_receipts(receipts, out_receipts, receipt_capacity, out_receipt_count);
        };
        let transaction_id = provider.prepared[&address].transaction_id;
        let state = provider.world.state_view();
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            format!("commit-{transaction_id}"),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        let receipt =
            match GeneratedVoxelWorldPortAdapter::new(&mut provider.world).commit(OriginEnvelope {
                origin,
                config_hash,
                payload: prepared,
            }) {
                Ok(receipt) => receipt.payload,
                Err(error) => {
                    let error_id = canonical_error_id(error.error_id());
                    provider
                        .prepared
                        .get_mut(&address)
                        .expect("token remains owned")
                        .terminal_error = Some(error_id);
                    return status_for_error(error_id);
                }
            };
        let _receipt_bytes = receipt.receipt;
        let view = provider.world.publication_authority().capture();
        let entries = provider.prepared[&address].entries.clone();
        let mut storages = BTreeMap::<SectionKey, SectionStorage>::new();
        for entry in &entries {
            if let std::collections::btree_map::Entry::Vacant(slot) = storages.entry(entry.section)
            {
                let storage = provider
                    .sections
                    .get(&entry.section)
                    .and_then(|state| state.storage.clone())
                    .unwrap_or_else(|| SectionStorage::uniform(BlockId::from_raw(0)));
                slot.insert(storage);
            }
            storages
                .get_mut(&entry.section)
                .expect("storage inserted above")
                .write(
                    CellOffset::new(entry.offset).expect("validated offset"),
                    BlockId::from_raw(entry.block_id),
                );
        }
        for (section, storage) in storages {
            let revision = view
                .stamp()
                .section_revision_set
                .get(&section.id())
                .copied()
                .unwrap_or(view.stamp().world_revision);
            provider.sections.insert(
                section,
                SectionState {
                    presence: VoxelPresence::Ready,
                    revision,
                    storage: Some(storage),
                },
            );
        }
        provider.rebuild_block_world();
        let receipts = entries
            .iter()
            .map(|entry| entry.section)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|section| VoxelWriteReceipt {
                section_key: section.to_abi(),
                up_to_section_revision: view
                    .stamp()
                    .section_revision_set
                    .get(&section.id())
                    .copied()
                    .unwrap_or(view.stamp().world_revision),
                world_revision: view.stamp().world_revision,
            })
            .collect::<Vec<_>>();
        provider
            .prepared
            .get_mut(&address)
            .expect("token remains owned")
            .receipts = receipts;
        write_receipts(
            &provider.prepared[&address].receipts,
            out_receipts,
            receipt_capacity,
            out_receipt_count,
        )
    })
}

fn write_receipts(
    receipts: &[VoxelWriteReceipt],
    out: *mut VoxelWriteReceipt,
    capacity: u32,
    count: *mut u32,
) -> i32 {
    if receipts.len() > capacity as usize {
        unsafe { count.write(receipts.len() as u32) };
        return LumioStatus::BufferTooSmall as i32;
    }
    unsafe {
        count.write(receipts.len() as u32);
        for (index, receipt) in receipts.iter().enumerate() {
            out.add(index).write(*receipt);
        }
    }
    LumioStatus::Success as i32
}

pub unsafe extern "C" fn block_write_abort(world: *mut c_void, token: *mut c_void) -> i32 {
    ffi(|| {
        if token.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = token as usize;
        let request = match provider.prepared.get(&address) {
            Some(token) => token.request.clone(),
            None => return LumioStatus::InvalidArgument as i32,
        };
        let state = provider.world.state_view();
        let origin = match OriginToken::try_new(
            state.world_context_id(),
            state.instance_generation(),
            format!("abort-{}", request.txn_id),
            0,
            BTreeMap::new(),
            "VoxelCommit",
        ) {
            Ok(origin) => origin,
            Err(error) => return status_for_error(error.error_id()),
        };
        let config_hash = provider.world.config_hash().to_string();
        match GeneratedVoxelWorldPortAdapter::new(&mut provider.world).abort(OriginEnvelope {
            origin,
            config_hash,
            payload: request.clone(),
        }) {
            Ok(_) => {
                provider.prepared.remove(&address);
                provider
                    .transactions
                    .remove(&request.txn_id.parse::<u64>().unwrap_or_default());
                LumioStatus::Success as i32
            }
            Err(error) => status_for_error(error.error_id()),
        }
    })
}

pub unsafe extern "C" fn section_revision_query(
    world: *mut c_void,
    key: *const VoxelSectionKey,
    out: *mut VoxelSectionRevisionResult,
) -> i32 {
    ffi(|| {
        if key.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let key = match SectionKey::from_abi(unsafe { *key }) {
            Ok(key) => key,
            Err(error) => return status_for_error(error),
        };
        if let Err(error) = provider.ensure_world_query(&[key]) {
            return status_for_error(error);
        }
        let id = key.id();
        let view = provider.world.publication_authority().capture();
        let slot = match view.directory().lookup(&id) {
            Ok(Some(slot)) => slot,
            Ok(None) => return status_for_error("unknown_section_key"),
            Err(error) => return status_for_error(error.error_id()),
        };
        let presence = match parse_presence(slot.presence()) {
            Ok(presence) => presence,
            Err(error) => return status_for_error(error),
        };
        let revision = view
            .stamp()
            .section_revision_set
            .get(&id)
            .copied()
            .unwrap_or(view.stamp().world_revision);
        unsafe {
            out.write(VoxelSectionRevisionResult {
                presence,
                _reserved: [0; 4],
                section_revision: revision,
            })
        };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn residency_pin_declare(
    world: *mut c_void,
    keys: *const VoxelSectionKey,
    section_count: u32,
    budget: u32,
    out_pin: *mut *mut c_void,
) -> i32 {
    ffi(|| {
        if out_pin.is_null() || (section_count > 0 && keys.is_null()) {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let input = if section_count == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(keys, section_count as usize) }
        };
        let ids = input
            .iter()
            .map(|key| SectionKey::from_abi(*key).map(|key| key.id()))
            .collect::<Result<Vec<_>, _>>();
        let ids = match ids {
            Ok(ids) => ids,
            Err(error) => return status_for_error(error),
        };
        let ready = {
            let view = provider.world.publication_authority().capture();
            ids.iter().all(|section_id| {
                view.directory()
                    .lookup(section_id)
                    .ok()
                    .flatten()
                    .is_some_and(|slot| slot.presence() == "Ready")
            })
        };
        let manager = match provider.world.region_pin_manager_mut() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let id = match manager
            .declare_pin_with_budget(ids, PinBudget::new(budget as usize, budget as usize))
        {
            Ok(id) => id,
            Err(error) => return status_for_error(error.error_id()),
        };
        if ready {
            let _ = manager.mark_ready(id);
        }
        let mut token = Box::new(PinToken { id });
        let address = (&mut *token) as *mut PinToken as usize;
        provider.pin_tokens.insert(address, token);
        unsafe { out_pin.write(address as *mut c_void) };
        LumioStatus::Success as i32
    })
}

pub unsafe extern "C" fn residency_pin_release(world: *mut c_void, pin: *mut c_void) -> i32 {
    ffi(|| {
        if pin.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let address = pin as usize;
        let id = match provider.pin_tokens.get(&address) {
            Some(token) => token.id,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let manager = match provider.world.region_pin_manager_mut() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        match manager.release_pin(id) {
            Ok(()) => {
                provider.pin_tokens.remove(&address);
                LumioStatus::Success as i32
            }
            Err(error) => status_for_error(error.error_id()),
        }
    })
}

pub unsafe extern "C" fn residency_pin_status(
    world: *mut c_void,
    pin: *mut c_void,
    out: *mut crate::abi_generated::VoxelPinStatus,
) -> i32 {
    ffi(|| {
        if pin.is_null() || out.is_null() {
            return LumioStatus::InvalidArgument as i32;
        }
        let provider = match provider(world) {
            Ok(provider) => provider,
            Err(status) => return status,
        };
        let id = match provider.pin_tokens.get(&(pin as usize)) {
            Some(token) => token.id,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let manager = match provider.world.region_pin_manager() {
            Some(manager) => manager,
            None => return LumioStatus::InvalidArgument as i32,
        };
        let status = match manager.status(id) {
            Ok(status) => status,
            Err(error) => return status_for_error(error.error_id()),
        };
        let ready_count = manager
            .sections(id)
            .map(|sections| {
                sections
                    .iter()
                    .filter(|section_id| {
                        provider.sections.iter().any(|(key, state)| {
                            key.id() == **section_id && state.presence == VoxelPresence::Ready
                        })
                    })
                    .count()
            })
            .unwrap_or(0);
        unsafe {
            out.write(crate::abi_generated::VoxelPinStatus {
                ready: u8::from(status.is_ready()),
                _reserved: [0; 7],
                section_count: status.section_count() as u32,
                ready_section_count: ready_count as u32,
            })
        };
        LumioStatus::Success as i32
    })
}

fn hex32(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 15) as usize] as char);
    }
    out
}

impl VoxelSectionKey {
    pub const fn new(x: i32, y: u8, z: i32) -> Self {
        Self {
            x,
            y,
            _reserved: [0; 3],
            z,
        }
    }
}
impl VoxelWorldCoordinate {
    pub const fn new(x: i32, y: u8, z: i32) -> Self {
        Self { x, y, z }
    }
}
impl VoxelBoxRequest {
    pub const fn new(min: VoxelWorldCoordinate, max: VoxelWorldCoordinate) -> Self {
        Self { min, max }
    }
}
impl VoxelBlockWriteEntry {
    pub const fn new(
        section_key: VoxelSectionKey,
        cell_offset: u16,
        block_id: u32,
        expected_section_revision: u64,
    ) -> Self {
        Self {
            section_key,
            cell_offset,
            _reserved: [0; 2],
            block_id,
            expected_section_revision,
        }
    }
}
impl Default for VoxelBlockReadCellResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            has_block_id: 0,
            _reserved: [0; 3],
            block_id: 0,
            section_revision: 0,
        }
    }
}
impl Default for VoxelBlockReadResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            has_block_id: 0,
            _reserved: [0; 3],
            block_id: 0,
            section_revision: 0,
        }
    }
}
impl Default for VoxelSectionRevisionResult {
    fn default() -> Self {
        Self {
            presence: VoxelPresence::Unavailable,
            _reserved: [0; 4],
            section_revision: 0,
        }
    }
}
impl Default for VoxelSectionSegment {
    fn default() -> Self {
        Self {
            section_key: VoxelSectionKey::new(0, 0, 0),
            presence: VoxelPresence::Unavailable,
            section_revision: 0,
            first_result: 0,
            result_count: 0,
        }
    }
}
impl Default for VoxelWriteReceipt {
    fn default() -> Self {
        Self {
            section_key: VoxelSectionKey::new(0, 0, 0),
            up_to_section_revision: 0,
            world_revision: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_block_type_maps_to_stable_contract_status() {
        assert_eq!(
            status_for_error("unregistered_block_type"),
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE
        );
        assert_eq!(
            crate::abi_generated::VOXEL_ERROR_UNREGISTERED_BLOCK_TYPE,
            1051
        );
    }
}
