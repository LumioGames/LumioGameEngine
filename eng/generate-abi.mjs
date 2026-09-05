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

// The prefix, A-1 tail, and A-4 physics tail are frozen ABI slots. Validation
// happens before any output is written, and C members come from this list.
const expectedRootFields = [
  ['abi_version', 'u32'],
  ['struct_size', 'u32'],
  ['abi_hash', 'bytes32'],
  ['build_id', 'bytes16'],
  ['ping', 'fn(pointer) -> status'],
  ['create_clr_host', 'fn(cstring, cstring, cstring, cstring, pointer) -> status'],
  ['clr_host_call', 'fn(pointer, pointer, u32, pointer, u32, pointer) -> status'],
  ['destroy_clr_host', 'fn(pointer) -> status'],
  ['timer_create_manager', 'fn(u32, pointer) -> status'],
  ['timer_destroy_manager', 'fn(pointer) -> status'],
  ['timer_register_dispatch', 'fn(pointer, u32) -> status'],
  ['timer_register_scope', 'fn(pointer, u64, u32, pointer) -> status'],
  ['timer_teardown_scope', 'fn(pointer, u64) -> status'],
  ['timer_create_slot', 'fn(pointer, pointer) -> status'],
  ['timer_bind_slot', 'fn(pointer, pointer, u32) -> status'],
  ['timer_close_slot', 'fn(pointer, pointer) -> status'],
  ['timer_schedule_one_shot', 'fn(pointer, u64, u32, u32, u64, pointer, pointer) -> status'],
  ['timer_schedule_repeating', 'fn(pointer, u64, u32, u32, u64, u64, pointer, pointer) -> status'],
  ['timer_cancel', 'fn(pointer, pointer) -> status'],
  ['timer_advance', 'fn(pointer, u64) -> status'],
  ['timer_pump', 'fn(pointer, u64) -> status'],
  ['timer_drain', 'fn(pointer, pointer, u32, pointer) -> status'],
  ['block_read_cell', 'fn(pointer, pointer, pointer) -> status'],
  ['block_read_box', 'fn(pointer, pointer, pointer, u32, pointer, pointer, u32, pointer, pointer) -> status'],
  ['block_read_column', 'fn(pointer, pointer, pointer, u32, pointer, pointer, u32, pointer, pointer) -> status'],
  ['block_write_prepare', 'fn(pointer, u64, pointer, u32, pointer) -> status'],
  ['block_write_commit', 'fn(pointer, pointer, pointer, u32, pointer) -> status'],
  ['block_write_abort', 'fn(pointer, pointer) -> status'],
  ['section_revision_query', 'fn(pointer, pointer, pointer) -> status'],
  ['residency_pin_declare', 'fn(pointer, pointer, u32, u32, pointer) -> status'],
  ['residency_pin_release', 'fn(pointer, pointer) -> status'],
  ['residency_pin_status', 'fn(pointer, pointer, pointer) -> status'],
  ['raycast', 'fn(pointer, pointer, pointer) -> status'],
  ['sweep', 'fn(pointer, pointer, pointer) -> status'],
  ['overlap', 'fn(pointer, pointer, pointer, u32, pointer) -> status'],
].map(([name, type]) => ({ name, type }));
const rootFields = definition.root?.fields;
if (!Array.isArray(rootFields) || rootFields.length !== expectedRootFields.length) {
  throw new Error(`root fields must exactly match the frozen 22-field prefix, 10-slot A-1 tail, and 3-slot A-4 tail (expected ${expectedRootFields.length})`);
}
const rootNames = new Set();
for (const [index, expected] of expectedRootFields.entries()) {
  const actual = rootFields[index];
  if (!actual || typeof actual.name !== 'string' || rootNames.has(actual.name)) {
    throw new Error(`root fields must exactly match frozen order and uniqueness at index ${index}`);
  }
  rootNames.add(actual.name);
  if (actual.name !== expected.name || actual.type !== expected.type) {
    throw new Error(`root fields must exactly match frozen name/type signature at index ${index}: expected ${expected.name}/${expected.type}`);
  }
}

const requiredVoxelSlots = expectedRootFields.slice(22).map((field) => field.name);
const physicsSlots = expectedRootFields.slice(32).map((field) => field.name);
const voxel = definition.voxel;
if (!voxel?.types || !voxel.enums?.presence || !voxel.constants?.cellOffset) {
  throw new Error('native-abi.json is missing the generated voxel ABI metadata');
}
const blockWritePrepare = rootFields.find((field) => field.name === 'block_write_prepare');
if (blockWritePrepare.type !== 'fn(pointer, u64, pointer, u32, pointer) -> status') {
  throw new Error('block_write_prepare must carry a caller-supplied u64 transaction_id');
}
if (voxel.constants.blockId?.type !== 'u32') throw new Error('voxel BlockId must be u32');
if (wireHash !== voxel.sourceSha256) throw new Error(`voxel ABI source hash mismatch: expected ${voxel.sourceSha256}, got ${wireHash}`);

const wireErrorCodes = wireContract.errorCodes;
if (wireContract.contractId !== voxel.contractId
  || wireErrorCodes?.length !== voxel.sourceCounts.errors
  || wireContract.rules?.length !== voxel.sourceCounts.rules
  || (wireContract.testCases?.length ?? 0) + (wireContract.invalidCases?.length ?? 0) !== voxel.sourceCounts.cases) {
  throw new Error('voxel ABI source counts or contract identity do not match the frozen wire contract');
}
if (!Array.isArray(wireErrorCodes)
  || !Array.isArray(voxel.errorCodes)
  || wireErrorCodes.length !== 51
  || voxel.errorCodes.length !== 51
  || wireErrorCodes.some((code, index) => code !== voxel.errorCodes[index])
  || new Set(wireErrorCodes).size !== wireErrorCodes.length
  || new Set(voxel.errorCodes).size !== voxel.errorCodes.length) {
  throw new Error('voxel ABI error codes must match the frozen wire names, order, and uniqueness');
}

const wireCellOffset = wireContract.identity?.cellOffset;
const cellOffset = voxel.constants.cellOffset;
const wireFormula = typeof wireCellOffset?.formula === 'string' && wireCellOffset.formula.startsWith('cellOffset = ')
  ? wireCellOffset.formula.slice('cellOffset = '.length)
  : wireCellOffset?.formula;
const canonicalCellOffset = {
  formula: wireFormula,
  strides: wireCellOffset?.strides,
  min: 0,
  max: 4095,
  inverse: wireCellOffset?.inverse,
};
if (JSON.stringify(cellOffset) !== JSON.stringify(canonicalCellOffset)) {
  throw new Error('voxel cellOffset formula, strides, range, and inverse must exactly match the canonical wire contract');
}
if (typeof wireCellOffset?.range !== 'string' || !wireCellOffset.range.startsWith('0 ~ 4095')) {
  throw new Error('canonical wire cellOffset range must be 0 ~ 4095');
}
if (voxel.errorStatusBase !== 1000 || voxel.errorCodes.length !== 51) {
  throw new Error('voxel ABI must declare all 51 stable contract error statuses from base 1000');
}
if (voxel.enums.presence.type !== 'u32') {
  throw new Error('voxel presence must use fixed-width u32 storage');
}
if (JSON.stringify(voxel.enums.query_resolution) !== JSON.stringify({
  type: 'u32',
  values: { Hit: 0, Miss: 1, Unresolved: 2 },
  unresolvedSectionField: 'unresolved_section',
  unresolvedIsNormalNotAnError: true,
})) {
  throw new Error('voxel query resolution must expose Hit/Miss/Unresolved as a fixed-width result field');
}
const requiredPhysicsFields = {
  raycast_result: [['resolution', 'query_resolution'], ['unresolved_section', 'section_key']],
  sweep_result: [['resolution', 'query_resolution'], ['unresolved_section', 'section_key']],
  overlap_result: [['resolution', 'query_resolution'], ['unresolved_section', 'section_key'], ['actual_count', 'u32'], ['truncated', 'u8']],
};
for (const [typeName, fields] of Object.entries(requiredPhysicsFields)) {
  const declared = voxel.types[typeName]?.fields;
  if (!Array.isArray(declared) || fields.some(([name, type]) => !declared.some((field) => field.name === name && field.type === type))) {
    throw new Error(`voxel ${typeName} must preserve its explicit query resolution and output fields`);
  }
}
if (voxel.layout?.pointerWidth !== 64
  || voxel.layout?.endianness !== 'little'
  || voxel.layout?.rootStructSize !== 304
  || voxel.layout?.voxelSlotsStartOffset !== 200
  || voxel.layout?.physicsSlotsStartOffset !== 280
  || JSON.stringify(voxel.layout?.rootSlotOffsets) !== JSON.stringify({
    block_read_cell: 200,
    block_read_box: 208,
    block_read_column: 216,
    block_write_prepare: 224,
    block_write_commit: 232,
    block_write_abort: 240,
    section_revision_query: 248,
    residency_pin_declare: 256,
    residency_pin_release: 264,
    residency_pin_status: 272,
    raycast: 280,
    sweep: 288,
    overlap: 296,
  })
  || JSON.stringify(voxel.layout?.physicsSlotOffsets) !== JSON.stringify({ raycast: 280, sweep: 288, overlap: 296 })) {
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
  f32: 'f32',
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
  query_resolution: 'VoxelQueryResolution',
  section_key: 'VoxelSectionKey',
  world_coordinate: 'VoxelWorldCoordinate',
  box_request: 'VoxelBoxRequest',
  column_request: 'VoxelColumnRequest',
  world_point: 'VoxelWorldPoint',
};
const csharpTypes = {
  i32: 'int',
  f32: 'float',
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
  query_resolution: 'VoxelQueryResolution',
  section_key: 'VoxelSectionKey',
  world_coordinate: 'VoxelWorldCoordinate',
  box_request: 'VoxelBoxRequest',
  column_request: 'VoxelColumnRequest',
  world_point: 'VoxelWorldPoint',
};
const cTypes = {
  i32: 'int32_t',
  f32: 'float',
  u8: 'uint8_t',
  u16: 'uint16_t',
  u32: 'uint32_t',
  u64: 'uint64_t',
  bytes2: 'uint8_t[2]',
  bytes3: 'uint8_t[3]',
  bytes4: 'uint8_t[4]',
  bytes7: 'uint8_t[7]',
  pointer: 'void*',
  presence: 'lumio_voxel_presence_t',
  query_resolution: 'lumio_voxel_query_resolution_t',
  section_key: 'lumio_voxel_section_key_t',
  world_coordinate: 'lumio_voxel_world_coordinate_t',
  box_request: 'lumio_voxel_box_request_t',
  column_request: 'lumio_voxel_column_request_t',
  world_point: 'lumio_voxel_world_point_t',
};
const unsupportedType = (type) => {
  throw new Error(`unsupported ABI field type: ${type}`);
};
const rustType = (type) => rustTypes[type] ?? unsupportedType(type);
const csharpType = (type) => csharpTypes[type] ?? unsupportedType(type);
const cType = (type) => cTypes[type] ?? unsupportedType(type);
const pascal = (value) => value.split('_').filter(Boolean).map((part) => part[0].toUpperCase() + part.slice(1)).join('');
const errorConstantName = (code) => `VOXEL_ERROR_${code.toUpperCase().split('-').join('_')}`;

const structEntries = Object.entries(voxel.types).filter(([, type]) => type.kind === 'struct');
for (const [name, type] of structEntries) {
  if (!Array.isArray(type.fields)) throw new Error(`voxel type ${name} must declare fields`);
  for (const field of type.fields) {
    rustType(field.type);
    csharpType(field.type);
    cType(field.type);
  }
}
const rustQueryResolution = `#[repr(u32)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum VoxelQueryResolution {\n    Hit = ${voxel.enums.query_resolution.values.Hit},\n    Miss = ${voxel.enums.query_resolution.values.Miss},\n    Unresolved = ${voxel.enums.query_resolution.values.Unresolved},\n}`;
const rustStructs = rustQueryResolution + '\n\n' + structEntries.map(([name, type]) => {
  const fields = type.fields.map((field) => `    pub ${field.name}: ${rustType(field.type)},`).join('\n');
  return `#[repr(C)]\n#[derive(Clone, Copy, Debug)]\npub struct Voxel${pascal(name)} {\n${fields}\n}`;
}).join('\n\n');
const csharpQueryResolution = `internal enum VoxelQueryResolution : uint\n{\n    Hit = ${voxel.enums.query_resolution.values.Hit},\n    Miss = ${voxel.enums.query_resolution.values.Miss},\n    Unresolved = ${voxel.enums.query_resolution.values.Unresolved},\n}\n`;
const csharpStructs = csharpQueryResolution + '\n' + structEntries.map(([name, type]) => {
  const fields = type.fields.map((field) => {
    const typeName = csharpType(field.type);
    const arraySize = typeName === 'byte[]' ? Number(field.type.slice(5)) : 0;
    const attr = arraySize ? `    [MarshalAs(UnmanagedType.ByValArray, SizeConst = ${arraySize})]\n` : '';
    return `${attr}    public ${typeName} ${pascal(field.name)};`;
  }).join('\n');
  return `[StructLayout(LayoutKind.Sequential)]\ninternal struct Voxel${pascal(name)}\n{\n${fields}\n}`;
}).join('\n\n');

const voxelCArguments = {
  block_read_cell: ['void*', 'const lumio_voxel_world_coordinate_t*', 'lumio_voxel_block_read_cell_result_t*'],
  block_read_box: ['void*', 'const void*', 'lumio_voxel_block_read_result_t*', 'uint32_t', 'uint32_t*', 'lumio_voxel_section_segment_t*', 'uint32_t', 'uint32_t*', 'uint8_t*'],
  block_read_column: ['void*', 'const void*', 'lumio_voxel_block_read_result_t*', 'uint32_t', 'uint32_t*', 'lumio_voxel_section_segment_t*', 'uint32_t', 'uint32_t*', 'uint8_t*'],
  block_write_prepare: ['void*', 'uint64_t', 'const lumio_voxel_block_write_entry_t*', 'uint32_t', 'void**'],
  block_write_commit: ['void*', 'void*', 'lumio_voxel_write_receipt_t*', 'uint32_t', 'uint32_t*'],
  block_write_abort: ['void*', 'void*'],
  section_revision_query: ['void*', 'const lumio_voxel_section_key_t*', 'lumio_voxel_section_revision_result_t*'],
  residency_pin_declare: ['void*', 'const lumio_voxel_section_key_t*', 'uint32_t', 'uint32_t', 'void**'],
  residency_pin_release: ['void*', 'void*'],
  residency_pin_status: ['void*', 'void*', 'lumio_voxel_pin_status_t*'],
  raycast: ['void*', 'const lumio_voxel_raycast_request_t*', 'lumio_voxel_raycast_result_t*'],
  sweep: ['void*', 'const lumio_voxel_sweep_request_t*', 'lumio_voxel_sweep_result_t*'],
  overlap: ['void*', 'const lumio_voxel_overlap_request_t*', 'lumio_voxel_overlap_hit_t*', 'uint32_t', 'lumio_voxel_overlap_result_t*'],
};
const rootScalarCType = { u32: 'uint32_t', bytes32: 'uint8_t[32]', bytes16: 'uint8_t[16]' };
const rootGenericCType = { pointer: 'void*', cstring: 'const char*', u32: 'uint32_t', u64: 'uint64_t' };
const parseFunctionType = (type) => {
  const match = /^fn\((.*)\) -> (.+)$/.exec(type);
  if (!match || match[2] !== 'status') throw new Error(`unsupported root ABI function type: ${type}`);
  return match[1] ? match[1].split(', ').map((arg) => rootGenericCType[arg] ?? unsupportedType(arg)) : [];
};
const cFunctionTypeName = (field) => field.name === 'block_read_box' || field.name === 'block_read_column'
  ? 'lumio_block_read_batch_fn'
  : `lumio_${field.name}_fn`;
const typedVoxelFields = rootFields.filter((field) => voxelCArguments[field.name]);
const functionTypedefs = typedVoxelFields.filter((field, index, fields) =>
  fields.findIndex((candidate) => cFunctionTypeName(candidate) === cFunctionTypeName(field)) === index).map((field) => {
  const args = voxelCArguments[field.name];
  return `typedef lumio_status_t (*${cFunctionTypeName(field)})(${args.join(', ')});`;
}).join('\n');
const rootMembers = rootFields.map((field) => {
  if (field.type.startsWith('fn(')) {
    return voxelCArguments[field.name]
      ? `    ${cFunctionTypeName(field)} ${field.name};`
      : '    void* ' + field.name + ';';
  }
  const scalarType = rootScalarCType[field.type];
  if (!scalarType) return unsupportedType(field.type);
  if (scalarType.includes('[')) return `    uint8_t ${field.name}[${scalarType.match(/\[(\d+)\]/)[1]}];`;
  return `    ${scalarType} ${field.name};`;
}).join('\n');

const rustErrorConstants = voxel.errorCodes.map((code, index) =>
  `pub const ${errorConstantName(code)}: i32 = ${voxel.errorStatusBase + index};`).join('\n');
const rustFunctionTypes = `
pub type VoxelStatus = i32;
pub type BlockReadCellFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelWorldCoordinate, *mut VoxelBlockReadCellResult) -> VoxelStatus;
pub type BlockReadBatchFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const core::ffi::c_void, *mut VoxelBlockReadResult, u32, *mut u32, *mut VoxelSectionSegment, u32, *mut u32, *mut u8) -> VoxelStatus;
pub type BlockWritePrepareFn = unsafe extern "C" fn(*mut core::ffi::c_void, u64, *const VoxelBlockWriteEntry, u32, *mut *mut core::ffi::c_void) -> VoxelStatus;
pub type BlockWriteCommitFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut VoxelWriteReceipt, u32, *mut u32) -> VoxelStatus;
pub type BlockWriteAbortFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> VoxelStatus;
pub type SectionRevisionQueryFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelSectionKey, *mut VoxelSectionRevisionResult) -> VoxelStatus;
pub type ResidencyPinDeclareFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelSectionKey, u32, u32, *mut *mut core::ffi::c_void) -> VoxelStatus;
pub type ResidencyPinReleaseFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> VoxelStatus;
pub type ResidencyPinStatusFn = unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, *mut VoxelPinStatus) -> VoxelStatus;
pub type RaycastFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelRaycastRequest, *mut VoxelRaycastResult) -> VoxelStatus;
pub type SweepFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelSweepRequest, *mut VoxelSweepResult) -> VoxelStatus;
pub type OverlapFn = unsafe extern "C" fn(*mut core::ffi::c_void, *const VoxelOverlapRequest, *mut VoxelOverlapHit, u32, *mut VoxelOverlapResult) -> VoxelStatus;
`;
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
    pub raycast: Option<RaycastFn>,
    pub sweep: Option<SweepFn>,
    pub overlap: Option<OverlapFn>,
}
`;

const csharpFunctionTypes = `

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int BlockWritePrepareFn(nint world, ulong transactionId, nint entries, uint entryCount, out nint token);
[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int RaycastFn(nint world, nint request, out VoxelRaycastResult result);
[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int SweepFn(nint world, nint request, out VoxelSweepResult result);
[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int OverlapFn(nint world, nint request, nint hits, uint capacity, out VoxelOverlapResult result);`;
const csharpRootSlots = `

[StructLayout(LayoutKind.Sequential)]
internal struct VoxelRootSlots
{
${requiredVoxelSlots.map((slot) => `    public nint ${pascal(slot)};`).join('\n')}
}

internal static class VoxelErrorCodes
{
${voxel.errorCodes.map((code, index) => `    public const int ${pascal(code)} = ${voxel.errorStatusBase + index};`).join('\n')}
}`;

const headerQueryResolution = `typedef uint32_t lumio_voxel_query_resolution_t;\n#define LUMIO_VOXEL_QUERY_HIT ((lumio_voxel_query_resolution_t)${voxel.enums.query_resolution.values.Hit}u)\n#define LUMIO_VOXEL_QUERY_MISS ((lumio_voxel_query_resolution_t)${voxel.enums.query_resolution.values.Miss}u)\n#define LUMIO_VOXEL_QUERY_UNRESOLVED ((lumio_voxel_query_resolution_t)${voxel.enums.query_resolution.values.Unresolved}u)`;
const headerStructs = `${voxel.errorCodes.map((code, index) => `#define ${errorConstantName(code)} ${voxel.errorStatusBase + index}`).join('\n')}\n\n${headerQueryResolution}\n\n` + structEntries.map(([name, type]) => {
  const fields = type.fields.map((field) => {
    const mapped = cType(field.type);
    if (mapped.includes('[')) return `    uint8_t ${field.name}[${mapped.match(/\[(\d+)\]/)[1]}];`;
    return `    ${mapped} ${field.name};`;
  }).join('\n');
  return `typedef struct lumio_voxel_${name}_t {\n${fields}\n} lumio_voxel_${name}_t;`;
}).join('\n\n');
const entryDeclaration = `lumio_status_t ${entrySymbol}(uint32_t requested_version, const lumio_engine_root_api_v1_t** out_api);`;

await writeFile(rustPath, `// Generated by eng/generate-abi.mjs. Do not edit.\n\npub const ABI_VERSION: u32 = ${abiVersion};\npub const ENTRY_SYMBOL: &str = "${entrySymbol}";\npub const DEFINITION_SHA256: &str = "${definitionHash}";\npub const VOXEL_MAX_CELLS_PER_READ_REQUEST: u32 = ${voxel.constants.maxCellsPerReadRequest};\npub const VOXEL_MAX_ENTRIES_PER_WRITE_BATCH: u32 = ${voxel.constants.maxEntriesPerWriteBatch};\npub const VOXEL_CELL_OFFSET_Y_STRIDE: u32 = ${cellOffset.strides.y};\npub const VOXEL_CELL_OFFSET_Z_STRIDE: u32 = ${cellOffset.strides.z};\npub const VOXEL_CELL_OFFSET_X_STRIDE: u32 = ${cellOffset.strides.x};\n${rustErrorConstants}\n\n#[repr(u32)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum VoxelPresence {\n    Ready = ${voxel.enums.presence.values.Ready},\n    Unchanged = ${voxel.enums.presence.values.Unchanged},\n    Pending = ${voxel.enums.presence.values.Pending},\n    Unavailable = ${voxel.enums.presence.values.Unavailable},\n}\n\n${rustStructs}\n${rustFunctionTypes}\n${rustRootSlots}` , 'utf8');
await writeFile(csharpPath, `// <auto-generated />\nusing System;\nusing System.Runtime.InteropServices;\n\nnamespace Lumio.Engine.NativeLoader;\n\ninternal static class AbiConstants\n{\n    public const uint AbiVersion = ${abiVersion};\n    public const string EntrySymbol = "${entrySymbol}";\n    public const string DefinitionSha256 = "${definitionHash}";\n    public const uint VoxelMaxCellsPerReadRequest = ${voxel.constants.maxCellsPerReadRequest};\n    public const uint VoxelMaxEntriesPerWriteBatch = ${voxel.constants.maxEntriesPerWriteBatch};\n    public const uint VoxelCellOffsetYStride = ${cellOffset.strides.y};\n    public const uint VoxelCellOffsetZStride = ${cellOffset.strides.z};\n    public const uint VoxelCellOffsetXStride = ${cellOffset.strides.x};\n}\n\ninternal enum VoxelPresence : uint\n{\n    Ready = ${voxel.enums.presence.values.Ready},\n    Unchanged = ${voxel.enums.presence.values.Unchanged},\n    Pending = ${voxel.enums.presence.values.Pending},\n    Unavailable = ${voxel.enums.presence.values.Unavailable},\n}\n\n${csharpStructs}\n${csharpRootSlots}\n${csharpFunctionTypes}\n`, 'utf8');
await writeFile(headerPath, `/* Generated by eng/generate-abi.mjs. Do not edit. */\n#ifndef LUMIO_ENGINE_H\n#define LUMIO_ENGINE_H\n\n#include <stdint.h>\n#include <stddef.h>\n\n#ifdef __cplusplus\nextern "C" {\n#endif\n\ntypedef int32_t lumio_status_t;\ntypedef uint32_t lumio_voxel_presence_t;\n#define LUMIO_VOXEL_READY ((lumio_voxel_presence_t)${voxel.enums.presence.values.Ready}u)\n#define LUMIO_VOXEL_UNCHANGED ((lumio_voxel_presence_t)${voxel.enums.presence.values.Unchanged}u)\n#define LUMIO_VOXEL_PENDING ((lumio_voxel_presence_t)${voxel.enums.presence.values.Pending}u)\n#define LUMIO_VOXEL_UNAVAILABLE ((lumio_voxel_presence_t)${voxel.enums.presence.values.Unavailable}u)\n\n#define LUMIO_VOXEL_MAX_CELLS_PER_READ_REQUEST ${voxel.constants.maxCellsPerReadRequest}u\n#define LUMIO_VOXEL_MAX_ENTRIES_PER_WRITE_BATCH ${voxel.constants.maxEntriesPerWriteBatch}u\n#define LUMIO_VOXEL_CELL_OFFSET_Y_STRIDE ${cellOffset.strides.y}u\n#define LUMIO_VOXEL_CELL_OFFSET_Z_STRIDE ${cellOffset.strides.z}u\n#define LUMIO_VOXEL_CELL_OFFSET_X_STRIDE ${cellOffset.strides.x}u\n\n${headerStructs}\n\n${functionTypedefs}\n\n/* The root declaration is emitted from the validated native-abi.json root field model. */\ntypedef struct lumio_engine_root_api_v1_t {\n${rootMembers}\n} lumio_engine_root_api_v1_t;\n\n${entryDeclaration}\n\n#ifdef __cplusplus\n}\n#endif\n#endif /* LUMIO_ENGINE_H */\n`, 'utf8');

console.log(`ABI_VERSION=${abiVersion}`);
console.log(`ENTRY_SYMBOL=${entrySymbol}`);
console.log(`DEFINITION_SHA256=${definitionHash}`);
