import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const definitionPath = resolve(root, 'engine/abi/native-abi.json');
const definitionBytes = await readFile(definitionPath);
const definition = JSON.parse(definitionBytes);
const definitionHash = createHash('sha256').update(definitionBytes).digest('hex');
const wirePath = resolve(root, 'engine/wire/voxel-world-v1.json');
const wireBytes = await readFile(wirePath);
const wireHash = createHash('sha256').update(wireBytes).digest('hex');
const wireContract = JSON.parse(wireBytes);
const abiVersion = Number(definition.abiVersion);
const entrySymbol = String(definition.entrySymbol);

if (!Number.isInteger(abiVersion) || abiVersion < 1) {
  throw new Error('native-abi.json abiVersion must be a positive integer');
}
if (!/^lumio_[a-z0-9_]+_v\d+$/.test(entrySymbol)) {
  throw new Error(`invalid entrySymbol: ${entrySymbol}`);
}

const rustPath = resolve(root, 'engine/native/modules/sdk-native/src/abi_generated.rs');
const csharpPath = resolve(root, 'engine/managed/Lumio.Engine.NativeLoader/AbiConstants.g.cs');
const headerPath = resolve(root, 'engine/native/modules/sdk-native/include/lumio_engine.h');
await mkdir(dirname(rustPath), { recursive: true });
await mkdir(dirname(csharpPath), { recursive: true });
await mkdir(dirname(headerPath), { recursive: true });

const requiredVoxelSlots = [
  'block_read_cell',
  'block_read_box',
  'block_read_column',
  'block_write_prepare',
  'block_write_commit',
  'block_write_abort',
  'section_revision_query',
  'residency_pin_declare',
  'residency_pin_release',
  'residency_pin_status',
];
const rootFields = definition.root?.fields ?? [];
const rootFieldNames = new Set(rootFields.map((field) => field.name));
for (const slot of requiredVoxelSlots) {
  if (!rootFieldNames.has(slot)) throw new Error(`native-abi.json is missing voxel root slot: ${slot}`);
}
const voxel = definition.voxel;
if (!voxel?.types || !voxel.enums?.presence || !voxel.constants?.cellOffset) {
  throw new Error('native-abi.json is missing the generated voxel ABI metadata');
}
if (voxel.constants.blockId?.type !== 'u32') throw new Error('voxel BlockId must be u32');
if (wireHash !== voxel.sourceSha256) throw new Error(`voxel ABI source hash mismatch: expected ${voxel.sourceSha256}, got ${wireHash}`);
if (wireContract.contractId !== voxel.contractId
  || wireContract.errorCodes?.length !== voxel.sourceCounts.errors
  || wireContract.rules?.length !== voxel.sourceCounts.rules
  || (wireContract.testCases?.length ?? 0) + (wireContract.invalidCases?.length ?? 0) !== voxel.sourceCounts.cases) {
  throw new Error('voxel ABI source counts or contract identity do not match the frozen wire contract');
}
if (voxel.constants.cellOffset.strides?.y !== 256
  || voxel.constants.cellOffset.strides?.z !== 16
  || voxel.constants.cellOffset.strides?.x !== 1) {
  throw new Error('voxel cellOffset strides must be y=256, z=16, x=1');
}
if (voxel.errorStatusBase !== 1000 || !Array.isArray(voxel.errorCodes) || voxel.errorCodes.length !== 51) {
  throw new Error('voxel ABI must declare all 51 stable contract error statuses from base 1000');
}
if (voxel.layout?.pointerWidth !== 64
  || voxel.layout?.endianness !== 'little'
  || voxel.layout?.rootStructSize !== 280
  || voxel.layout?.voxelSlotsStartOffset !== 200) {
  throw new Error('voxel ABI layout must preserve the x64 append-only root offsets');
}
const directAbiTests = definition.directAbiTests ?? [];
for (const name of ['block_catalog_row_incomplete', 'write_batch_too_large', 'pin_region_not_ready']) {
  if (!directAbiTests.some((testCase) => testCase.name === name)) {
    throw new Error(`native-abi.json is missing direct ABI test: ${name}`);
  }
}

const rustTypes = {
  i32: 'i32',
  u8: 'u8',
  u16: 'u16',
  u32: 'u32',
  u64: 'u64',
  bytes2: '[u8; 2]',
  bytes3: '[u8; 3]',
  bytes4: '[u8; 4]',
  bytes7: '[u8; 7]',
  pointer: '*mut core::ffi::c_void',
  presence: 'VoxelPresence',
  section_key: 'VoxelSectionKey',
  world_coordinate: 'VoxelWorldCoordinate',
  world_coordinate_const: 'VoxelWorldCoordinate',
  box_request: 'VoxelBoxRequest',
  column_request: 'VoxelColumnRequest',
};
const rustType = (type) => rustTypes[type] ?? 'u32';

const csharpTypes = {
  i32: 'int',
  u8: 'byte',
  u16: 'ushort',
  u32: 'uint',
  u64: 'ulong',
  bytes2: 'byte[]',
  bytes3: 'byte[]',
  bytes4: 'byte[]',
  bytes7: 'byte[]',
  pointer: 'nint',
  presence: 'VoxelPresence',
  section_key: 'VoxelSectionKey',
  world_coordinate: 'VoxelWorldCoordinate',
  box_request: 'VoxelBoxRequest',
  column_request: 'VoxelColumnRequest',
};
const csharpType = (type) => csharpTypes[type] ?? 'uint';

const pascal = (value) => value.split('_').filter(Boolean).map((part) => part[0].toUpperCase() + part.slice(1)).join('');
const rustStructs = Object.entries(voxel.types)
  .filter(([, type]) => type.kind === 'struct')
  .map(([name, type]) => {
    const fields = type.fields.map((field) => `    pub ${field.name}: ${rustType(field.type)},`).join('\n');
    return `#[repr(C)]\n#[derive(Clone, Copy, Debug)]\npub struct Voxel${pascal(name)} {\n${fields}\n}`;
  }).join('\n\n');
const csharpStructs = Object.entries(voxel.types)
  .filter(([, type]) => type.kind === 'struct')
  .map(([name, type]) => {
    const fields = type.fields.map((field) => {
      const typeName = csharpType(field.type);
      const attr = typeName === 'byte[]' ? `    [MarshalAs(UnmanagedType.ByValArray, SizeConst = ${Number(field.type.slice(5)) || 1})]\n` : '';
      return `${attr}    public ${typeName} ${pascal(field.name)};`;
    }).join('\n');
    return `[StructLayout(LayoutKind.Sequential)]\ninternal struct Voxel${pascal(name)}\n{\n${fields}\n}`;
  }).join('\n\n') + `

[StructLayout(LayoutKind.Sequential)]
internal struct VoxelRootSlots
{
    public nint BlockReadCell;
    public nint BlockReadBox;
    public nint BlockReadColumn;
    public nint BlockWritePrepare;
    public nint BlockWriteCommit;
    public nint BlockWriteAbort;
    public nint SectionRevisionQuery;
    public nint ResidencyPinDeclare;
    public nint ResidencyPinRelease;
    public nint ResidencyPinStatus;
}` + `

internal static class VoxelErrorCodes
{
${voxel.errorCodes.map((code, index) => `    public const int ${pascal(code)} = ${voxel.errorStatusBase + index};`).join('\n')}
}`;

const rustFunctionTypes = `
pub type VoxelStatus = i32;
pub type BlockReadCellFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelWorldCoordinate, *mut VoxelBlockReadCellResult) -> VoxelStatus;
pub type BlockReadBatchFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, *mut VoxelBlockReadResult, u32, *mut u32, *mut VoxelSectionSegment, u32, *mut u32, *mut u8) -> VoxelStatus;
pub type BlockWritePrepareFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelBlockWriteEntry, u32, *mut *mut core::ffi::c_void) -> VoxelStatus;
pub type BlockWriteCommitFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut VoxelWriteReceipt, u32, *mut u32) -> VoxelStatus;
pub type BlockWriteAbortFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> VoxelStatus;
pub type SectionRevisionQueryFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelSectionKey, *mut VoxelSectionRevisionResult) -> VoxelStatus;
pub type ResidencyPinDeclareFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelSectionKey, u32, u32, *mut *mut core::ffi::c_void) -> VoxelStatus;
pub type ResidencyPinReleaseFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> VoxelStatus;
pub type ResidencyPinStatusFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut VoxelPinStatus) -> VoxelStatus;
`;

const rootSlotComments = requiredVoxelSlots.map((slot) => `    // generated root slot: ${slot}`).join('\n');
const errorConstantName = (code) => `VOXEL_ERROR_${code.toUpperCase().replaceAll('-', '_')}`;
const rustErrorConstants = voxel.errorCodes.map((code, index) =>
  `pub const ${errorConstantName(code)}: i32 = ${voxel.errorStatusBase + index};`).join('\n');
const rustRootSlots = `
#[repr(C)]
pub struct VoxelRootSlots {
    pub block_read_cell: Option<BlockReadCellFn>,
    pub block_read_box: Option<BlockReadBatchFn>,
    pub block_read_column: Option<BlockReadBatchFn>,
    pub block_write_prepare: Option<BlockWritePrepareFn>,
    pub block_write_commit: Option<BlockWriteCommitFn>,
    pub block_write_abort: Option<BlockWriteAbortFn>,
    pub section_revision_query: Option<SectionRevisionQueryFn>,
    pub residency_pin_declare: Option<ResidencyPinDeclareFn>,
    pub residency_pin_release: Option<ResidencyPinReleaseFn>,
    pub residency_pin_status: Option<ResidencyPinStatusFn>,
}
`;
await writeFile(rustPath, `// Generated by eng/generate-abi.mjs. Do not edit.\n\npub const ABI_VERSION: u32 = ${abiVersion};\npub const ENTRY_SYMBOL: &str = "${entrySymbol}";\npub const DEFINITION_SHA256: &str = "${definitionHash}";\npub const VOXEL_MAX_CELLS_PER_READ_REQUEST: u32 = ${voxel.constants.maxCellsPerReadRequest};\npub const VOXEL_MAX_ENTRIES_PER_WRITE_BATCH: u32 = ${voxel.constants.maxEntriesPerWriteBatch};\npub const VOXEL_CELL_OFFSET_Y_STRIDE: u32 = ${voxel.constants.cellOffset.strides.y};\npub const VOXEL_CELL_OFFSET_Z_STRIDE: u32 = ${voxel.constants.cellOffset.strides.z};\npub const VOXEL_CELL_OFFSET_X_STRIDE: u32 = ${voxel.constants.cellOffset.strides.x};\n${rustErrorConstants}\n\n#[repr(u32)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum VoxelPresence {\n    Ready = ${voxel.enums.presence.values.Ready},\n    Unchanged = ${voxel.enums.presence.values.Unchanged},\n    Pending = ${voxel.enums.presence.values.Pending},\n    Unavailable = ${voxel.enums.presence.values.Unavailable},\n}\n\n${rustStructs}\n${rustFunctionTypes}\n${rustRootSlots}\n// Root slots are appended after the existing ping/CLR/timer slots.\n${rootSlotComments}\n`, 'utf8');
await writeFile(csharpPath, `// <auto-generated />\nusing System;\nusing System.Runtime.InteropServices;\n\nnamespace Lumio.Engine.NativeLoader;\n\ninternal static class AbiConstants\n{\n    public const uint AbiVersion = ${abiVersion};\n    public const string EntrySymbol = "${entrySymbol}";\n    public const string DefinitionSha256 = "${definitionHash}";\n    public const uint VoxelMaxCellsPerReadRequest = ${voxel.constants.maxCellsPerReadRequest};\n    public const uint VoxelMaxEntriesPerWriteBatch = ${voxel.constants.maxEntriesPerWriteBatch};\n    public const uint VoxelCellOffsetYStride = ${voxel.constants.cellOffset.strides.y};\n    public const uint VoxelCellOffsetZStride = ${voxel.constants.cellOffset.strides.z};\n    public const uint VoxelCellOffsetXStride = ${voxel.constants.cellOffset.strides.x};\n}\n\ninternal enum VoxelPresence : uint\n{\n    Ready = ${voxel.enums.presence.values.Ready},\n    Unchanged = ${voxel.enums.presence.values.Unchanged},\n    Pending = ${voxel.enums.presence.values.Pending},\n    Unavailable = ${voxel.enums.presence.values.Unavailable},\n}\n\n${csharpStructs}\n`, 'utf8');

const headerStructs = `${voxel.errorCodes.map((code, index) => `#define ${errorConstantName(code)} ${voxel.errorStatusBase + index}`).join('\n')}\n\n` + Object.entries(voxel.types)
  .filter(([, type]) => type.kind === 'struct')
  .map(([name, type]) => {
    const fields = type.fields.map((field) => {
      const cType = ({ i32: 'int32_t', u8: 'uint8_t', u16: 'uint16_t', u32: 'uint32_t', u64: 'uint64_t', bytes2: 'uint8_t[2]', bytes3: 'uint8_t[3]', bytes4: 'uint8_t[4]', bytes7: 'uint8_t[7]', presence: 'lumio_voxel_presence_t', section_key: 'lumio_voxel_section_key_t', world_coordinate: 'lumio_voxel_world_coordinate_t', box_request: 'lumio_voxel_box_request_t', column_request: 'lumio_voxel_column_request_t' })[field.type] ?? 'uint32_t';
      if (cType.startsWith('uint8_t[')) {
        return `    uint8_t ${field.name}[${cType.slice(8, -1)}];`;
      }
      return `    ${cType} ${field.name};`;
    }).join('\n');
    return `typedef struct lumio_voxel_${name}_t {\n${fields}\n} lumio_voxel_${name}_t;`;
  }).join('\n\n');
await writeFile(headerPath, `/* Generated by eng/generate-abi.mjs. Do not edit. */\n#ifndef LUMIO_ENGINE_H\n#define LUMIO_ENGINE_H\n\n#include <stdint.h>\n#include <stddef.h>\n\n#ifdef __cplusplus\nextern "C" {\n#endif\n\ntypedef int32_t lumio_status_t;\ntypedef enum lumio_voxel_presence_t {\n    LUMIO_VOXEL_READY = ${voxel.enums.presence.values.Ready},\n    LUMIO_VOXEL_UNCHANGED = ${voxel.enums.presence.values.Unchanged},\n    LUMIO_VOXEL_PENDING = ${voxel.enums.presence.values.Pending},\n    LUMIO_VOXEL_UNAVAILABLE = ${voxel.enums.presence.values.Unavailable}\n} lumio_voxel_presence_t;\n\n#define LUMIO_VOXEL_MAX_CELLS_PER_READ_REQUEST ${voxel.constants.maxCellsPerReadRequest}u\n#define LUMIO_VOXEL_MAX_ENTRIES_PER_WRITE_BATCH ${voxel.constants.maxEntriesPerWriteBatch}u\n#define LUMIO_VOXEL_CELL_OFFSET_Y_STRIDE ${voxel.constants.cellOffset.strides.y}u\n#define LUMIO_VOXEL_CELL_OFFSET_Z_STRIDE ${voxel.constants.cellOffset.strides.z}u\n#define LUMIO_VOXEL_CELL_OFFSET_X_STRIDE ${voxel.constants.cellOffset.strides.x}u\n\n${headerStructs}\n\n/* The opaque world/token/pin handles are caller-managed; Native allocates no result buffers. */\ntypedef lumio_status_t (*lumio_block_read_cell_fn)(void*, const lumio_voxel_world_coordinate_t*, lumio_voxel_block_read_cell_result_t*);\ntypedef lumio_status_t (*lumio_block_read_batch_fn)(void*, const void*, lumio_voxel_block_read_result_t*, uint32_t, lumio_voxel_section_segment_t*, uint32_t, uint32_t*, uint8_t*);\ntypedef lumio_status_t (*lumio_block_write_prepare_fn)(void*, const lumio_voxel_block_write_entry_t*, uint32_t, void**);\ntypedef lumio_status_t (*lumio_block_write_commit_fn)(void*, void*, lumio_voxel_write_receipt_t*, uint32_t, uint32_t*);\ntypedef lumio_status_t (*lumio_block_write_abort_fn)(void*, void*);\ntypedef lumio_status_t (*lumio_section_revision_query_fn)(void*, const lumio_voxel_section_key_t*, uint64_t*);\ntypedef lumio_status_t (*lumio_residency_pin_declare_fn)(void*, const lumio_voxel_section_key_t*, uint32_t, uint32_t, void**);\ntypedef lumio_status_t (*lumio_residency_pin_release_fn)(void*, void*);\ntypedef lumio_status_t (*lumio_residency_pin_status_fn)(void*, void*, lumio_voxel_pin_status_t*);\n\n/* ABI source declares the append-only root slots; implementations may leave them unavailable until their provider card lands. */\ntypedef struct lumio_engine_root_api_v1_t {\n    uint32_t abi_version;\n    uint32_t struct_size;\n    uint8_t abi_hash[32];\n    uint8_t build_id[16];\n    void* ping;\n    void* create_clr_host;\n    void* clr_host_call;\n    void* destroy_clr_host;\n    void* timer_create_manager;\n    void* timer_destroy_manager;\n    void* timer_register_dispatch;\n    void* timer_register_scope;\n    void* timer_teardown_scope;\n    void* timer_create_slot;\n    void* timer_bind_slot;\n    void* timer_close_slot;\n    void* timer_schedule_one_shot;\n    void* timer_schedule_repeating;\n    void* timer_cancel;\n    void* timer_advance;\n    void* timer_pump;\n    void* timer_drain;\n    lumio_block_read_cell_fn block_read_cell;\n    lumio_block_read_batch_fn block_read_box;\n    lumio_block_read_batch_fn block_read_column;\n    lumio_block_write_prepare_fn block_write_prepare;\n    lumio_block_write_commit_fn block_write_commit;\n    lumio_block_write_abort_fn block_write_abort;\n    lumio_section_revision_query_fn section_revision_query;\n    lumio_residency_pin_declare_fn residency_pin_declare;\n    lumio_residency_pin_release_fn residency_pin_release;\n    lumio_residency_pin_status_fn residency_pin_status;\n} lumio_engine_root_api_v1_t;\n\n#ifdef __cplusplus\n}\n#endif\n#endif /* LUMIO_ENGINE_H */\n`, 'utf8');

const generatedHeader = await readFile(headerPath, 'utf8');
await writeFile(
  headerPath,
  generatedHeader.replace(
    'typedef lumio_status_t (*lumio_section_revision_query_fn)(void*, const lumio_voxel_section_key_t*, uint64_t*);',
    'typedef lumio_status_t (*lumio_section_revision_query_fn)(void*, const lumio_voxel_section_key_t*, lumio_voxel_section_revision_result_t*);',
  ).replace(
    'typedef lumio_status_t (*lumio_block_read_batch_fn)(void*, const void*, lumio_voxel_block_read_result_t*, uint32_t, lumio_voxel_section_segment_t*, uint32_t, uint32_t*, uint8_t*);',
    'typedef lumio_status_t (*lumio_block_read_batch_fn)(void*, const void*, lumio_voxel_block_read_result_t*, uint32_t, uint32_t*, lumio_voxel_section_segment_t*, uint32_t, uint32_t*, uint8_t*);',
  ),
  'utf8',
);

console.log(`ABI_VERSION=${abiVersion}`);
console.log(`ENTRY_SYMBOL=${entrySymbol}`);
console.log(`DEFINITION_SHA256=${definitionHash}`);
