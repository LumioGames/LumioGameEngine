import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';

const definition = JSON.parse(
  await readFile(new URL('../engine/abi/native-abi.json', import.meta.url)),
);
const wireContract = JSON.parse(
  await readFile(new URL('../engine/wire/voxel-world-v1.json', import.meta.url)),
);
const generator = await readFile(new URL('./generate-abi.mjs', import.meta.url), 'utf8');
const header = await readFile(new URL('../engine/native/modules/sdk-native/include/lumio_engine.h', import.meta.url), 'utf8');
const rust = await readFile(new URL('../engine/native/modules/sdk-native/src/abi_generated.rs', import.meta.url), 'utf8');
const csharp = await readFile(new URL('../engine/managed/Lumio.Engine.NativeLoader/AbiConstants.g.cs', import.meta.url), 'utf8');

test('native ABI declares the voxel read/write/revision surface', () => {
  const fields = new Set(definition.root.fields.map((field) => field.name));
  for (const name of [
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
  ]) {
    assert(fields.has(name), `missing root ABI slot: ${name}`);
  }
});

test('block write prepare carries a stable caller transaction ID in every binding', () => {
  const field = definition.root.fields.find((candidate) => candidate.name === 'block_write_prepare');
  assert.equal(field?.type, 'fn(pointer, u64, pointer, u32, pointer) -> status');
  assert.match(field?.doc ?? '', /transaction_id/);
  assert.match(header, /typedef lumio_status_t \(\*lumio_block_write_prepare_fn\)\(void\*, uint64_t, const lumio_voxel_block_write_entry_t\*, uint32_t, void\*\*\);/);
  assert.match(rust, /pub type BlockWritePrepareFn = unsafe extern "C" fn\(\*mut core::ffi::c_void, u64, \*const VoxelBlockWriteEntry, u32, \*mut \*mut core::ffi::c_void\) -> VoxelStatus;/);
  assert.match(csharp, /internal delegate int BlockWritePrepareFn\(nint world, ulong transactionId, nint entries, uint entryCount, out nint token\);/);
});

test('the C entry declaration is generated from native ABI entrySymbol', () => {
  const expected = `lumio_status_t ${definition.entrySymbol}(uint32_t requested_version, const lumio_engine_root_api_v1_t** out_api);`;
  assert.equal(header.split(expected).length - 1, 1);
  assert.match(generator, /entrySymbol.*lumio_engine_root_api_v1_t/);
});

test('all 51 voxel error names are unique, ordered, and mapped identically in C, Rust, and C#', () => {
  const wireCodes = wireContract.errorCodes;
  const abiCodes = definition.voxel.errorCodes;
  assert.equal(wireCodes.length, 51);
  assert.deepEqual(abiCodes, wireCodes);
  assert.equal(new Set(wireCodes).size, wireCodes.length);

  const pascal = (value) => value.split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join('');
  const cCodes = [...header.matchAll(/^#define VOXEL_ERROR_([A-Z0-9_]+) (\d+)$/gm)]
    .map((match) => [match[1].toLowerCase(), Number(match[2])]);
  const rustCodes = [...rust.matchAll(/^pub const VOXEL_ERROR_([A-Z0-9_]+): i32 = (\d+);$/gm)]
    .map((match) => [match[1].toLowerCase(), Number(match[2])]);
  const csharpCodes = [...csharp.matchAll(/^    public const int ([A-Za-z0-9]+) = (\d+);$/gm)]
    .map((match) => [match[1], Number(match[2])])
    .filter(([name]) => name !== 'AbiVersion');
  const expected = wireCodes.map((code, index) => [code, definition.voxel.errorStatusBase + index]);
  assert.deepEqual(cCodes, expected.map(([code, value]) => [code, value]));
  assert.deepEqual(rustCodes, expected.map(([code, value]) => [code, value]));
  assert.deepEqual(csharpCodes, expected.map(([code, value]) => [pascal(code), value]));
  for (const mapping of [cCodes, rustCodes, csharpCodes]) {
    assert.equal(new Set(mapping.map(([name]) => name)).size, mapping.length);
    assert.equal(new Set(mapping.map(([, value]) => value)).size, mapping.length);
  }
  assert.match(generator, /wireContract\.errorCodes/);
  assert.match(generator, /new Set\(voxel\.errorCodes\)/);
});

test('C presence is an explicitly fixed-width uint32-compatible type', () => {
  assert.match(header, /typedef uint32_t lumio_voxel_presence_t;/);
  assert.doesNotMatch(header, /typedef enum lumio_voxel_presence_t/);
  assert.match(rust, /#\[repr\(u32\)\]\n#\[derive\(Clone, Copy, Debug, PartialEq, Eq\)\]\npub enum VoxelPresence/);
  assert.match(csharp, /internal enum VoxelPresence : uint/);
});

test('native ABI makes voxel states, coordinates, offsets, and budgets explicit', () => {
  assert.equal(definition.voxel.constants.blockId.type, 'u32');
  assert.deepEqual(definition.voxel.constants.cellOffset.strides, { y: 256, z: 16, x: 1 });
  assert.equal(definition.voxel.constants.maxCellsPerReadRequest, 262144);
  assert.equal(definition.voxel.constants.maxEntriesPerWriteBatch, 65536);
  assert.deepEqual(definition.voxel.enums.presence.values, {
    Ready: 0,
    Unchanged: 1,
    Pending: 2,
    Unavailable: 3,
  });
  assert.equal(definition.voxel.types.world_coordinate.fields[1].type, 'u8');
  assert.equal(definition.voxel.types.world_coordinate.fields[0].type, 'i32');
  assert.equal(definition.voxel.types.world_coordinate.fields[2].type, 'i32');
  assert.equal(definition.voxel.types.block_read_cell_result.required[0], 'presence');
  assert.equal(definition.voxel.types.block_read_cell_result.optional[0], 'block_id');
  assert.deepEqual(definition.voxel.types.section_revision_result.required, ['presence', 'section_revision']);
  assert.equal(definition.voxel.layout.rootStructSize, 280);
  assert.equal(definition.voxel.layout.voxelSlotsStartOffset, 200);
  assert.equal(definition.voxel.layout.rootSlotOffsets.residency_pin_status, 272);
  assert.equal(definition.voxel.layout.types.block_write_entry.size, 32);
});

test('native ABI keeps direct coverage for contract cases without invalidCases', () => {
  const tests = definition.directAbiTests;
  const names = new Set(tests.map((testCase) => testCase.name));
  assert.deepEqual(
    [...names].sort(),
    ['block_catalog_row_incomplete', 'pin_region_not_ready', 'write_batch_too_large'],
  );
  for (const testCase of tests) {
    const index = definition.voxel.errorCodes.indexOf(testCase.errorCode);
    assert(index >= 0, `${testCase.name} must name a frozen voxel error code`);
    assert.equal(testCase.status, testCase.name.split('_').map((word) => word[0].toUpperCase() + word.slice(1)).join(''));
    assert.equal(definition.voxel.errorStatusBase + index, {
      block_catalog_row_incomplete: 1039,
      write_batch_too_large: 1042,
      pin_region_not_ready: 1046,
    }[testCase.name]);
  }
});
