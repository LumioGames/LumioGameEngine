import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';

const definition = JSON.parse(
  await readFile(new URL('../engine/abi/native-abi.json', import.meta.url)),
);

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
