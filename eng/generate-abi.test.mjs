import assert from 'node:assert/strict';
import { cp, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { test } from 'node:test';

const execFileAsync = promisify(execFile);

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

async function runGeneratorWithDefinitionMutation(mutate) {
  const sourceRoot = dirname(dirname(fileURLToPath(import.meta.url)));
  const tempRoot = await mkdtemp(join(dirname(sourceRoot), 'generate-abi-negative-'));
  try {
    const definitionCopy = JSON.parse(await readFile(join(sourceRoot, 'engine/abi/native-abi.json'), 'utf8'));
    mutate(definitionCopy);
    await mkdir(join(tempRoot, 'engine', 'abi'), { recursive: true });
    await mkdir(join(tempRoot, 'engine', 'wire'), { recursive: true });
    await mkdir(join(tempRoot, 'eng'), { recursive: true });
    await writeFile(join(tempRoot, 'engine', 'abi', 'native-abi.json'), JSON.stringify(definitionCopy));
    await cp(join(sourceRoot, 'engine', 'wire', 'voxel-world-v1.json'), join(tempRoot, 'engine', 'wire', 'voxel-world-v1.json'));
    await cp(join(sourceRoot, 'eng', 'generate-abi.mjs'), join(tempRoot, 'eng', 'generate-abi.mjs'));
    return await execFileAsync(process.execPath, [join(tempRoot, 'eng', 'generate-abi.mjs')], { cwd: tempRoot });
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }
}

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

test('the C root declaration preserves the validated source field order and signatures', () => {
  const rootBody = header.match(/typedef struct lumio_engine_root_api_v1_t \{([\s\S]*?)\n\} lumio_engine_root_api_v1_t;/)?.[1] ?? '';
  const members = rootBody.split('\n')
    .map((line) => line.match(/^\s+(?:uint32_t|uint8_t|void\*|lumio_[a-z0-9_]+_fn)\s+(\w+)/)?.[1])
    .filter(Boolean);
  assert.deepEqual(members, definition.root.fields.map((field) => field.name));
  for (const field of definition.root.fields.filter((candidate) => candidate.name.startsWith('block_') || candidate.name.startsWith('section_') || candidate.name.startsWith('residency_'))) {
    const typedefName = field.name === 'block_read_box' || field.name === 'block_read_column'
      ? 'lumio_block_read_batch_fn'
      : `lumio_${field.name}_fn`;
    assert.match(header, new RegExp(`typedef lumio_status_t \\(\\*${typedefName}\\)`));
  }
});

test('all 52 voxel error names are unique, ordered, and mapped identically in C, Rust, and C#', () => {
  const wireCodes = wireContract.errorCodes;
  const abiCodes = definition.voxel.errorCodes;
  assert.equal(wireCodes.length, 52);
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
  assert.equal(definition.voxel.constants.cellOffset.formula, '(worldY & 15) * 256 + (worldZ & 15) * 16 + (worldX & 15)');
  assert.deepEqual(definition.voxel.constants.cellOffset.strides, { y: 256, z: 16, x: 1 });
  assert.equal(definition.voxel.constants.cellOffset.min, 0);
  assert.equal(definition.voxel.constants.cellOffset.max, 4095);
  assert.deepEqual(definition.voxel.constants.cellOffset.inverse, {
    y: '(cellOffset >> 8) & 15',
    z: '(cellOffset >> 4) & 15',
    x: 'cellOffset & 15',
  });
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
  assert.equal(definition.voxel.layout.rootStructSize, 304);
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

test('generator rejects any root order, count, duplicate, or signature drift', async () => {
  await assert.rejects(
    runGeneratorWithDefinitionMutation((copy) => {
      [copy.root.fields[4], copy.root.fields[5]] = [copy.root.fields[5], copy.root.fields[4]];
    }),
    /root fields must exactly match/,
  );
  await assert.rejects(
    runGeneratorWithDefinitionMutation((copy) => {
      copy.root.fields[8].name = copy.root.fields[9].name;
    }),
    /root fields must exactly match/,
  );
  await assert.rejects(
    runGeneratorWithDefinitionMutation((copy) => {
      copy.root.fields[22].type = 'fn(pointer, bogus) -> status';
    }),
    /root fields must exactly match/,
  );
});

test('generator rejects cell-offset formula, range, inverse, and stride drift', async () => {
  for (const mutate of [
    (copy) => { copy.voxel.constants.cellOffset.formula = '(worldX & 15) * 256 + (worldZ & 15) * 16 + (worldY & 15)'; },
    (copy) => { copy.voxel.constants.cellOffset.max = 4094; },
    (copy) => { copy.voxel.constants.cellOffset.inverse.x = '(cellOffset >> 8) & 15'; },
    (copy) => { copy.voxel.constants.cellOffset.strides.z = 1; },
  ]) {
    await assert.rejects(runGeneratorWithDefinitionMutation(mutate), /cellOffset/);
  }
});

test('generator emits the header in one pass with no fragile replacement stage', () => {
  assert.doesNotMatch(generator, /\.replace\(/);
  assert.match(generator, /entryDeclaration/);
  assert.match(generator, /rootFields\.map/);
});

test('generator rejects unsupported voxel ABI field types in every binding', async () => {
  await assert.rejects(
    runGeneratorWithDefinitionMutation((copy) => {
      copy.voxel.types.world_coordinate.fields[1].type = 'bogus';
    }),
    /unsupported ABI field type.*bogus/,
  );
});

test('A-4 ABI appends exactly raycast, sweep, and overlap root slots', () => {
  assert.equal(definition.root.fields.length, 35);
  assert.deepEqual(
    definition.root.fields.slice(-3).map((field) => field.name),
    ['raycast', 'sweep', 'overlap'],
  );
  assert.deepEqual(
    definition.root.fields.slice(-3).map((field) => field.type),
    [
      'fn(pointer, pointer, pointer) -> status',
      'fn(pointer, pointer, pointer) -> status',
      'fn(pointer, pointer, pointer, u32, pointer) -> status',
    ],
  );
  assert.deepEqual(
    definition.root.fields.slice(0, 32).map((field) => [field.name, field.type]),
    [
      ['abi_version', 'u32'], ['struct_size', 'u32'], ['abi_hash', 'bytes32'], ['build_id', 'bytes16'],
      ['ping', 'fn(pointer) -> status'],
      ['create_clr_host', 'fn(cstring, cstring, cstring, cstring, pointer) -> status'],
      ['clr_host_call', 'fn(pointer, pointer, u32, pointer, u32, pointer) -> status'],
      ['destroy_clr_host', 'fn(pointer) -> status'],
      ['timer_create_manager', 'fn(u32, pointer) -> status'], ['timer_destroy_manager', 'fn(pointer) -> status'],
      ['timer_register_dispatch', 'fn(pointer, u32) -> status'], ['timer_register_scope', 'fn(pointer, u64, u32, pointer) -> status'],
      ['timer_teardown_scope', 'fn(pointer, u64) -> status'], ['timer_create_slot', 'fn(pointer, pointer) -> status'],
      ['timer_bind_slot', 'fn(pointer, pointer, u32) -> status'], ['timer_close_slot', 'fn(pointer, pointer) -> status'],
      ['timer_schedule_one_shot', 'fn(pointer, u64, u32, u32, u64, pointer, pointer) -> status'],
      ['timer_schedule_repeating', 'fn(pointer, u64, u32, u32, u64, u64, pointer, pointer) -> status'],
      ['timer_cancel', 'fn(pointer, pointer) -> status'], ['timer_advance', 'fn(pointer, u64) -> status'],
      ['timer_pump', 'fn(pointer, u64) -> status'], ['timer_drain', 'fn(pointer, pointer, u32, pointer) -> status'],
      ['block_read_cell', 'fn(pointer, pointer, pointer) -> status'],
      ['block_read_box', 'fn(pointer, pointer, pointer, u32, pointer, pointer, u32, pointer, pointer) -> status'],
      ['block_read_column', 'fn(pointer, pointer, pointer, u32, pointer, pointer, u32, pointer, pointer) -> status'],
      ['block_write_prepare', 'fn(pointer, u64, pointer, u32, pointer) -> status'],
      ['block_write_commit', 'fn(pointer, pointer, pointer, u32, pointer) -> status'],
      ['block_write_abort', 'fn(pointer, pointer) -> status'], ['section_revision_query', 'fn(pointer, pointer, pointer) -> status'],
      ['residency_pin_declare', 'fn(pointer, pointer, u32, u32, pointer) -> status'],
      ['residency_pin_release', 'fn(pointer, pointer) -> status'], ['residency_pin_status', 'fn(pointer, pointer, pointer) -> status'],
    ],
  );
});

test('A-4 query results make three-state resolution and unresolved Section explicit', () => {
  assert.deepEqual(definition.voxel.enums.query_resolution, {
    type: 'u32',
    values: { Hit: 0, Miss: 1, Unresolved: 2 },
    unresolvedSectionField: 'unresolved_section',
    unresolvedIsNormalNotAnError: true,
  });
  for (const typeName of ['raycast_result', 'sweep_result', 'overlap_result']) {
    const type = definition.voxel.types[typeName];
    assert.ok(type, `${typeName} must be declared`);
    assert.ok(type.fields.some((field) => field.name === 'resolution' && field.type === 'query_resolution'));
    assert.ok(type.fields.some((field) => field.name === 'unresolved_section' && field.type === 'section_key'));
  }
  assert.doesNotMatch(header, /Unresolved.*lumio_status_t|lumio_status_t.*Unresolved/);
  assert.match(rust, /pub enum VoxelQueryResolution/);
  assert.match(csharp, /internal enum VoxelQueryResolution : uint/);
});

test('A-4 overlap ABI uses caller-owned hits, capacity, actual count, and truncated', () => {
  const overlap = definition.voxel.types.overlap_result;
  assert.ok(overlap.fields.some((field) => field.name === 'actual_count' && field.type === 'u32'));
  assert.ok(overlap.fields.some((field) => field.name === 'truncated' && field.type === 'u8'));
  assert.match(header, /lumio_overlap_fn\)\(void\*, const lumio_voxel_overlap_request_t\*, lumio_voxel_overlap_hit_t\*, uint32_t, lumio_voxel_overlap_result_t\*\)/);
  assert.match(rust, /pub type OverlapFn = unsafe extern "C" fn\(\*mut core::ffi::c_void, \*const VoxelOverlapRequest, \*mut VoxelOverlapHit, u32, \*mut VoxelOverlapResult\)/);
  assert.match(csharp, /internal delegate int OverlapFn\(nint world, nint request, nint hits, uint capacity, out VoxelOverlapResult result\);/);
  assert.doesNotMatch(header, /overlap[^\n]*void\*\*|lumio_overlap_fn[^\n]*void\*\*/);
});

test('A-4 query constants and mappings preserve unsigned BlockId and world-y types', () => {
  assert.equal(definition.voxel.types.raycast_result.fields.find((field) => field.name === 'block_id')?.type, 'u32');
  assert.equal(definition.voxel.types.sweep_result.fields.find((field) => field.name === 'block_id')?.type, 'u32');
  assert.equal(definition.voxel.types.overlap_hit.fields.find((field) => field.name === 'block_id')?.type, 'u32');
  assert.equal(definition.voxel.types.world_coordinate.fields.find((field) => field.name === 'y')?.type, 'u8');
  assert.match(csharp, /public uint BlockId;/);
  assert.match(csharp, /public byte Y;/);
});

test('A-4 query errors remain distinct stable mappings and v1 has no batch query slot', () => {
  const queryErrors = [
    'unresolved_hit_treated_as_air',
    'unresolved_hit_treated_as_solid',
    'query_buffer_overflow',
    'query_result_divergence',
    'collision_behavior_not_from_material_table',
    'query_mutates_world',
  ];
  const indices = queryErrors.map((code) => definition.voxel.errorCodes.indexOf(code));
  assert.deepEqual(indices, [30, 31, 32, 33, 34, 35]);
  assert.deepEqual(new Set(indices).size, queryErrors.length);
  assert.equal(definition.root.fields.filter((field) => /batch/i.test(field.name)).length, 0);
  assert.deepEqual(definition.voxel.layout.physicsSlotOffsets, { raycast: 280, sweep: 288, overlap: 296 });
  assert.equal(definition.voxel.layout.rootStructSize, 304);
});

test('A-4 generator rejects root tail order, count, and duplicate drift', async () => {
  for (const mutate of [
    (copy) => { const last = copy.root.fields.length - 1; [copy.root.fields[last - 2], copy.root.fields[last - 1]] = [copy.root.fields[last - 1], copy.root.fields[last - 2]]; },
    (copy) => { copy.root.fields.pop(); },
    (copy) => { const last = copy.root.fields.length - 1; copy.root.fields[last].name = copy.root.fields[last - 1].name; },
  ]) {
    await assert.rejects(runGeneratorWithDefinitionMutation(mutate), /root fields must exactly match/);
  }
});

test('A-4 generator rejects any root slot offset drift', async () => {
  await assert.rejects(
    runGeneratorWithDefinitionMutation((copy) => {
      copy.voxel.layout.rootSlotOffsets.raycast = 288;
    }),
    /append-only root offsets/,
  );
});
