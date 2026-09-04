// verify-wire.mjs — unified validator for every contract under engine/wire/.
//
// Layers, applied to each discovered *.json:
//   1. Structural grammar: required top-level keys (contractId/version/purpose),
//      messages/sharedTypes/errorCodes shape, unique error codes.
//   2. Reference integrity: enum refs resolve; every invalidCase's expectedRejection
//      exists in errorCodes and its `violates` names a registered rule id.
//   3. Case execution (contracts that declare testCases/invalidCases):
//      - testCases must pass grammar + all block semantics.
//      - invalidCases with validatorCheck:true must be REJECTED with the declared code.
//      - invalidCases with validatorCheck:false are checked for declaration completeness only.
//   Block semantics (contracts that declare `mappings` with LumioBinV1 encoding):
//      - payload is lowercase hex; payloadSha256 recomputes from the decoded bytes
//        (checked BEFORE any interpretation, mirroring the wire admission rule);
//      - payload decodes as LumioBinV1 per the mapping's fieldOrder and re-encodes
//        to identical bytes; collection=array is a u32 element count then records;
//      - per-field constraints (maxUtf8Bytes → violationCode);
//      - block kind rules per message type (commands:kind=command; stateBlocks:kind=state;
//        changedBlocks:kind in {event,state});
//      - mappingId strictly ascending and unique within one block array.
//
// hello-wire-v1.json (no embedded cases) passes at the structural layer unchanged.
// Any failure exits non-zero after printing per-contract summaries.

import { createHash } from 'node:crypto';
import { readdir, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { test } from 'node:test';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const wireDir = resolve(root, 'engine/wire');

const U64_MAX = Number.MAX_SAFE_INTEGER; // 2^53-1 on the JSON wire

export class Rejection extends Error {
  constructor(code, reason) {
    super(reason);
    this.code = code;
  }
}

export function collectErrorCodes(contract) {
  const codes = contract.errorCodes;
  if (Array.isArray(codes)) return codes.filter((c) => typeof c === 'string');
  if (codes && typeof codes === 'object') {
    const out = [];
    for (const value of Object.values(codes)) {
      if (Array.isArray(value) && value.every((item) => typeof item === 'string')) out.push(...value);
    }
    return out;
  }
  return [];
}

export function admitMessage(contract, message) {
  checkMessageShape(message, contract);
  checkMessageSemantics(message, contract);
}

// ---------- LumioBinV1 (ADR-047 subset used by declared mappings) ----------

class BinReader {
  constructor(bytes) {
    this.bytes = bytes;
    this.pos = 0;
  }
  u32() {
    if (this.pos + 4 > this.bytes.length) throw new Rejection('undecodable_payload', 'u32 runs past end of payload');
    const v = this.bytes.readUInt32LE(this.pos);
    this.pos += 4;
    return v;
  }
  u64() {
    if (this.pos + 8 > this.bytes.length) throw new Rejection('undecodable_payload', 'u64 runs past end of payload');
    const v = this.bytes.readBigUInt64LE(this.pos);
    this.pos += 8;
    if (v > BigInt(U64_MAX)) throw new Rejection('undecodable_payload', 'u64 exceeds JSON-safe 2^53-1 bound');
    return Number(v);
  }
  string() {
    const len = this.u32();
    if (this.pos + len > this.bytes.length) throw new Rejection('undecodable_payload', `string length ${len} runs past end of payload`);
    const text = this.bytes.subarray(this.pos, this.pos + len).toString('utf8');
    // UTF-8 must round-trip byte-exactly (rejects lone surrogates / overlong forms).
    const re = Buffer.from(text, 'utf8');
    if (!re.equals(this.bytes.subarray(this.pos, this.pos + len))) {
      throw new Rejection('undecodable_payload', 'string bytes are not canonical UTF-8');
    }
    this.pos += len;
    return text;
  }
  done() {
    if (this.pos !== this.bytes.length) throw new Rejection('undecodable_payload', `${this.bytes.length - this.pos} trailing bytes after last field`);
  }
}

function encodeField(type, value) {
  if (type === 'u64') {
    const b = Buffer.alloc(8);
    b.writeBigUInt64LE(BigInt(value));
    return b;
  }
  if (type === 'utf8-string') {
    const t = Buffer.from(value, 'utf8');
    const len = Buffer.alloc(4);
    len.writeUInt32LE(t.length);
    return Buffer.concat([len, t]);
  }
  throw new Error(`unsupported LumioBinV1 field type: ${type}`);
}

function encodeU32(value) {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(value);
  return b;
}

function readField(reader, field, fieldName) {
  if (field.type === 'u64') return reader.u64();
  if (field.type === 'utf8-string') return reader.string();
  throw new Rejection('undecodable_payload', `mapping field ${fieldName} has unsupported wire type ${field.type}`);
}

function encodeRecord(mapping, record) {
  let re = Buffer.alloc(0);
  for (const fieldName of mapping.fieldOrder) {
    re = Buffer.concat([re, encodeField(mapping.fields[fieldName].type, record[fieldName])]);
  }
  return re;
}

function encodeMappingPayload(mapping, body) {
  if (mapping.collection === 'array') {
    if (!Array.isArray(body)) throw new Error('array mapping requires an array body');
    let re = encodeU32(body.length);
    for (const record of body) re = Buffer.concat([re, encodeRecord(mapping, record)]);
    return re;
  }
  return encodeRecord(mapping, body);
}

function sha256Hex(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function applyFieldConstraints(mapping, record) {
  for (const fieldName of mapping.fieldOrder) {
    const field = mapping.fields[fieldName];
    if (field.type === 'utf8-string' && typeof field.maxUtf8Bytes === 'number') {
      if (Buffer.byteLength(record[fieldName], 'utf8') > field.maxUtf8Bytes) {
        throw new Rejection(field.violationCode ?? 'bad_envelope', `${fieldName} exceeds maxUtf8Bytes=${field.maxUtf8Bytes}`);
      }
    }
    if (Array.isArray(field.allowedValues) && !field.allowedValues.includes(record[fieldName])) {
      throw new Rejection(
        field.violationCode ?? 'undecodable_payload',
        `${fieldName} value ${JSON.stringify(record[fieldName])} is not in allowedValues`,
      );
    }
  }
}

function checkRecordOrder(mapping, records) {
  const orderBy = mapping.orderBy;
  if (typeof orderBy !== 'string' || orderBy.length === 0) return;
  const code = mapping.orderViolationCode ?? 'block_order_violation';
  for (let i = 1; i < records.length; i++) {
    if (!(records[i - 1][orderBy] < records[i][orderBy])) {
      throw new Rejection(code, `records not strictly ascending by ${orderBy} at index ${i}`);
    }
  }
}

function decodeMappingPayload(mapping, bytes) {
  const reader = new BinReader(bytes);
  if (mapping.collection === 'array') {
    // ADR-047: u32 element count, then records in document order; records are
    // concatenated fieldOrder structs with no padding.
    const count = reader.u32();
    const records = [];
    for (let i = 0; i < count; i++) {
      const record = {};
      for (const fieldName of mapping.fieldOrder) {
        record[fieldName] = readField(reader, mapping.fields[fieldName], fieldName);
      }
      records.push(record);
    }
    reader.done();
    let re = encodeU32(count);
    for (const record of records) re = Buffer.concat([re, encodeRecord(mapping, record)]);
    if (!re.equals(bytes)) throw new Rejection('undecodable_payload', 'decode/re-encode mismatch: payload is not canonical LumioBinV1');
    for (const record of records) applyFieldConstraints(mapping, record);
    checkRecordOrder(mapping, records);
    return records;
  }

  const body = {};
  for (const fieldName of mapping.fieldOrder) {
    body[fieldName] = readField(reader, mapping.fields[fieldName], fieldName);
  }
  reader.done();
  // Canonical re-encode must reproduce the exact bytes (two conforming encoders
  // may not disagree — ADR-049 §2 discipline).
  let re = Buffer.alloc(0);
  for (const fieldName of mapping.fieldOrder) {
    re = Buffer.concat([re, encodeField(mapping.fields[fieldName].type, body[fieldName])]);
  }
  if (!re.equals(bytes)) throw new Rejection('undecodable_payload', 'decode/re-encode mismatch: payload is not canonical LumioBinV1');
  applyFieldConstraints(mapping, body);
  return body;
}

// ---------- Message grammar ----------

function checkPrimitive(value, expr, contract) {
  if (expr.startsWith('const:')) return value === expr.slice(6) ? null : `expected const ${expr.slice(6)}`;
  if (expr.startsWith('enum:')) {
    const ref = expr.slice(5);
    let set = null;
    if (ref === 'roles') set = contract.roles;
    else if (ref === 'errorCodes') set = contract.errorCodes;
    else if (ref === 'mappings') set = contract.mappings ? Object.keys(contract.mappings) : null;
    else if (Array.isArray(contract.enums?.[ref])) set = contract.enums[ref];
    if (set === null) return `unknown enum ref ${ref}`;
    return set.includes(value) ? null : `${JSON.stringify(value)} not in ${ref}`;
  }
  switch (expr) {
    case 'u32':
      return Number.isInteger(value) && value >= 0 && value <= 0xffffffff
        ? null
        : `expected u32 integer in [0, 2^32-1], got ${JSON.stringify(value)}`;
    case 'u64':
    case 'epoch-ms':
      return Number.isInteger(value) && value >= 0 && value <= U64_MAX ? null : `expected u64 integer in [0, 2^53-1], got ${JSON.stringify(value)}`;
    case 'string':
      return typeof value === 'string' ? null : `expected string, got ${typeof value}`;
    case 'bool':
      return typeof value === 'boolean' ? null : `expected bool, got ${typeof value}`;
    case 'hex':
      return typeof value === 'string' && /^[0-9a-f]*$/.test(value) && value.length % 2 === 0 ? null : `expected lowercase hex string, got ${JSON.stringify(String(value)).slice(0, 40)}`;
    case 'sha256-hex':
      return typeof value === 'string' && /^[0-9a-f]{64}$/.test(value) ? null : `expected lowercase sha256 hex (64 chars), got ${JSON.stringify(String(value)).slice(0, 40)}`;
    case 'hex128':
      return typeof value === 'string' && /^[0-9a-f]{32}$/.test(value) ? null : `expected lowercase 128-bit hex (32 chars), got ${JSON.stringify(String(value)).slice(0, 40)}`;
    default:
      return `unknown type expression ${expr}`;
  }
}

function checkFieldShape(value, expr, contract, errors, path) {
  const arrayMatch = expr.match(/^array:(.+)$/);
  if (arrayMatch) {
    if (!Array.isArray(value)) {
      errors.push(new Rejection('bad_envelope', `${path}: expected array for ${expr}`));
      return;
    }
    value.forEach((item, i) => checkValueShape(item, arrayMatch[1], contract, errors, `${path}[${i}]`));
    return;
  }
  checkValueShape(value, expr, contract, errors, path);
}

function checkValueShape(value, expr, contract, errors, path) {
  if (expr.startsWith('const:') || expr.startsWith('enum:') || ['u32', 'u64', 'epoch-ms', 'string', 'bool', 'hex', 'hex128', 'sha256-hex'].includes(expr)) {
    const err = checkPrimitive(value, expr, contract);
    if (err) errors.push(new Rejection('bad_envelope', `${path}: ${err}`));
    return;
  }
  // Shared type reference: { required, optional } with a closed member set.
  const shared = contract.sharedTypes?.[expr];
  if (!shared) {
    errors.push(new Rejection('bad_envelope', `${path}: unknown type ${expr}`));
    return;
  }
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    errors.push(new Rejection('bad_envelope', `${path}: expected object of ${expr}`));
    return;
  }
  const allowed = new Set([...Object.keys(shared.required ?? {}), ...Object.keys(shared.optional ?? {})]);
  for (const key of Object.keys(value)) {
    if (!allowed.has(key)) errors.push(new Rejection('bad_envelope', `${path}.${key}: not a member of closed set ${expr}`));
  }
  for (const [key, typeExpr] of Object.entries(shared.required ?? {})) {
    if (!(key in value)) errors.push(new Rejection('bad_envelope', `${path}.${key}: required member missing`));
    else checkFieldShape(value[key], typeExpr, contract, errors, `${path}.${key}`);
  }
  for (const [key, typeExpr] of Object.entries(shared.optional ?? {})) {
    if (key in value) checkFieldShape(value[key], typeExpr, contract, errors, `${path}.${key}`);
  }
}

function checkMessageShape(message, contract) {
  const errors = [];
  const messageType = message?.messageType;
  const spec = contract.messages?.[messageType];
  if (!spec) {
    throw new Rejection('bad_envelope', `unknown messageType ${JSON.stringify(messageType)}`);
  }
  if (typeof message !== 'object' || message === null || Array.isArray(message)) {
    throw new Rejection('bad_envelope', 'message must be a JSON object');
  }
  const allowed = new Set([...Object.keys(spec.required ?? {}), ...Object.keys(spec.optional ?? {})]);
  for (const key of Object.keys(message)) {
    if (!allowed.has(key)) errors.push(new Rejection('bad_envelope', `${key}: not a member of closed message set ${messageType}`));
  }
  for (const [key, typeExpr] of Object.entries(spec.required ?? {})) {
    if (!(key in message)) errors.push(new Rejection('bad_envelope', `${key}: required member missing`));
    else checkFieldShape(message[key], typeExpr, contract, errors, key);
  }
  for (const [key, typeExpr] of Object.entries(spec.optional ?? {})) {
    if (key in message) checkFieldShape(message[key], typeExpr, contract, errors, key);
  }
  if (errors.length) throw errors[0];
}

// ---------- Block semantics ----------

function checkBlock(block, contract, context) {
  // context: { allowedKinds: Set, unknownCode: string, path: string }
  const mapping = contract.mappings?.[block.mappingId];
  if (!mapping) throw new Rejection(context.unknownCode, `${context.path}: mappingId ${block.mappingId} is not registered`);
  if (!context.allowedKinds.has(mapping.kind)) {
    throw new Rejection(context.unknownCode, `${context.path}: mappingId ${block.mappingId} has kind=${mapping.kind}, allowed here: ${[...context.allowedKinds].join('|')}`);
  }
  const bytes = Buffer.from(block.payload, 'hex');
  const digest = createHash('sha256').update(bytes).digest('hex');
  if (digest !== block.payloadSha256) {
    throw new Rejection('bad_payload_hash', `${context.path}: payloadSha256 does not recompute from payload`);
  }
  if (contract.encoding?.profile?.startsWith('LumioBinV1')) {
    decodeMappingPayload(mapping, bytes); // throws undecodable_payload / constraint violationCode
  }
}

function checkBlockArray(blocks, contract, context) {
  let prev = null;
  blocks.forEach((block, i) => {
    if (prev !== null && block.mappingId <= prev) {
      throw new Rejection('block_order_violation', `${context.path}[${i}]: mappingId ${block.mappingId} not strictly ascending (prev ${prev})`);
    }
    prev = block.mappingId;
    checkBlock(block, contract, { ...context, path: `${context.path}[${i}]` });
  });
}

function checkMessageSemantics(message, contract) {
  const t = message.messageType;
  if (t === 'InputCommand') {
    if (!Array.isArray(message.commands)) throw new Rejection('bad_envelope', 'commands: required array missing');
    checkBlockArray(message.commands, contract, { allowedKinds: new Set(['command']), unknownCode: 'unknown_command_type', path: 'commands' });
    const maxCommands = contract.boundedInput?.rules?.maxCommandsPerEnvelope;
    if (typeof maxCommands === 'number' && message.commands.length > maxCommands) {
      throw new Rejection('bad_envelope', `commands length ${message.commands.length} exceeds maxCommandsPerEnvelope=${maxCommands}`);
    }
  } else if (t === 'FullSnapshot') {
    if (!Array.isArray(message.stateBlocks)) throw new Rejection('bad_envelope', 'stateBlocks: required array missing');
    checkBlockArray(message.stateBlocks, contract, { allowedKinds: new Set(['state']), unknownCode: 'state_block_kind_mismatch', path: 'stateBlocks' });
    const maxBlocks = contract.boundedInput?.rules?.maxBlocksPerEnvelope;
    if (typeof maxBlocks === 'number' && message.stateBlocks.length > maxBlocks) {
      throw new Rejection('bad_envelope', `stateBlocks length ${message.stateBlocks.length} exceeds maxBlocksPerEnvelope=${maxBlocks}`);
    }
  } else if (t === 'Delta') {
    if (!Array.isArray(message.changedBlocks)) throw new Rejection('bad_envelope', 'changedBlocks: required array missing');
    checkBlockArray(message.changedBlocks, contract, { allowedKinds: new Set(['event', 'state']), unknownCode: 'state_block_kind_mismatch', path: 'changedBlocks' });
    const maxBlocks = contract.boundedInput?.rules?.maxBlocksPerEnvelope;
    if (typeof maxBlocks === 'number' && message.changedBlocks.length > maxBlocks) {
      throw new Rejection('bad_envelope', `changedBlocks length ${message.changedBlocks.length} exceeds maxBlocksPerEnvelope=${maxBlocks}`);
    }
  }
}

// ---------- Structural layer ----------

function checkStructure(contract, fileName, problems) {
  const problem = (msg) => problems.push(`${fileName}: ${msg}`);
  if (typeof contract.contractId !== 'string' || !/^[a-z0-9.-]+\.v\d+$/.test(contract.contractId)) {
    problem(`contractId "${contract.contractId}" is not <reverse-dns>.v<n>`);
  }
  if (!Number.isInteger(contract.version) || contract.version < 1) problem('version must be an integer >= 1');
  if (typeof contract.purpose !== 'string' || contract.purpose.length < 10) problem('purpose must be a meaningful string');
  if (contract.messages !== undefined) {
    for (const [name, spec] of Object.entries(contract.messages)) {
      if (!['c2s', 's2c'].includes(spec.dir)) problem(`messages.${name}.dir must be c2s|s2c`);
      if (spec.required === undefined) problem(`messages.${name}.required missing`);
      if (!spec.required?.messageType) problem(`messages.${name}.required.messageType missing`);
    }
  }
  const errorCodes = collectErrorCodes(contract);
  if (contract.errorCodes !== undefined) {
    if (errorCodes.length === 0) {
      problem('errorCodes must be an array of strings or an object containing string-array code lists');
    } else if (new Set(errorCodes).size !== errorCodes.length) {
      problem('errorCodes contains duplicates');
    }
  }
  if (contract.mappings !== undefined) {
    for (const [id, m] of Object.entries(contract.mappings)) {
      if (!Array.isArray(m.fieldOrder) || m.fieldOrder.length === 0) problem(`mappings.${id}.fieldOrder missing/empty`);
      for (const f of m.fieldOrder) if (!m.fields?.[f]) problem(`mappings.${id}.fields.${f} declared in fieldOrder but not in fields`);
      if (!m.dimensions) problem(`mappings.${id}.dimensions missing (persistence/replication/visibility declaration)`);
      if (!['command', 'event', 'componentState', 'state'].includes(m.kind)) problem(`mappings.${id}.kind "${m.kind}" not in the kind vocabulary`);
      if (m.collection !== undefined && m.collection !== 'array') {
        problem(`mappings.${id}.collection "${m.collection}" must be array when present`);
      }
    }
  }
  // Reference integrity for embedded cases and rules.
  if (contract.rules !== undefined) {
    const ids = new Set(contract.rules.map((r) => r.id));
    if (ids.size !== contract.rules.length) problem('rules contains duplicate ids');
    for (const r of contract.rules) {
      if (!['validator', 'receiver'].includes(r.enforcedBy)) problem(`rules.${r.id}.enforcedBy must be validator|receiver`);
      if (!errorCodes.includes(r.onViolation)) problem(`rules.${r.id}.onViolation ${r.onViolation} not in errorCodes`);
    }
  }
  for (const example of contract.hash?.examples ?? []) {
    if (typeof example.payload !== 'string' || typeof example.payloadSha256 !== 'string') {
      problem(`hash.examples ${example.mappingId ?? '?'} missing payload/payloadSha256`);
      continue;
    }
    if (!/^[0-9a-f]*$/.test(example.payload) || example.payload.length % 2 !== 0) {
      problem(`hash.examples ${example.mappingId ?? '?'} payload is not lowercase hex`);
      continue;
    }
    const digest = createHash('sha256').update(Buffer.from(example.payload, 'hex')).digest('hex');
    if (digest !== example.payloadSha256) {
      problem(`hash.examples ${example.mappingId ?? '?'}: payloadSha256 does not recompute (got ${digest})`);
    }
    const mapping = contract.mappings?.[example.mappingId];
    if (mapping && contract.encoding?.profile?.startsWith('LumioBinV1')) {
      try {
        decodeMappingPayload(mapping, Buffer.from(example.payload, 'hex'));
      } catch (error) {
        problem(`hash.examples ${example.mappingId ?? '?'}: payload does not decode (${error.code ?? error.message}: ${error.message})`);
      }
    }
  }
  const allCases = [...(contract.testCases ?? []), ...(contract.invalidCases ?? [])];
  for (const c of contract.invalidCases ?? []) {
    const code = c.expectedRejection;
    const stableCode = typeof code === 'string' && /^[a-z][a-z0-9_]*$/.test(code);
    if (stableCode && errorCodes.length > 0 && !errorCodes.includes(code)) {
      problem(`invalidCases.${c.name}: expectedRejection ${code} not in errorCodes`);
    }
    if (contract.rules && c.violates && !contract.rules.some((r) => r.id === c.violates)) {
      problem(`invalidCases.${c.name}: violates "${c.violates}" names no registered rule`);
    }
  }
  return allCases.length;
}

// ---------- Native timer ABI (C-4′ single-kernel dual-mode) ----------

const TIMER_CONTRACT_ID = 'lumio.native-timer-abi.v1';
const TIMER_ABI_PARAM_TYPES = new Set(['pointer', 'u32', 'u64']);
const TIMER_KERNEL_LAYERS = new Set(['kernel:wallClock', 'kernel:tickFrame']);
const TIMER_ABI_REQUIRED_FUNCTIONS = [
  'timer_create_manager',
  'timer_destroy_manager',
  'timer_register_dispatch',
  'timer_register_scope',
  'timer_teardown_scope',
  'timer_create_slot',
  'timer_bind_slot',
  'timer_close_slot',
  'timer_schedule_one_shot',
  'timer_schedule_repeating',
  'timer_cancel',
  'timer_advance',
  'timer_pump',
  'timer_drain',
];

function parseStatusFnParams(type) {
  const match = String(type ?? '').match(/^fn\((.*)\)\s*->\s*status$/);
  if (!match) return null;
  const inner = match[1].trim();
  return inner.length === 0 ? [] : inner.split(',').map((item) => item.trim());
}

function isForbiddenFnPointerType(type) {
  const text = String(type ?? '');
  if (text.includes('fn(') || text.includes('*fn') || text.includes('fn *')) return true;
  return /function\s*pointer|callback|delegate/i.test(text);
}

function checkNativeTimerContract(contract, fileName, problems) {
  if (contract.contractId !== TIMER_CONTRACT_ID) return;
  const problem = (msg) => problems.push(`${fileName}: ${msg}`);

  if (contract.layers?.hostTimerService) {
    problem('layers.hostTimerService retains a second timer infrastructure; only layers.kernel dual-mode is allowed');
  }
  if (contract.layers?.nativeTickFrameTimerManager) {
    problem('layers.nativeTickFrameTimerManager retains a second timer infrastructure; only layers.kernel dual-mode is allowed');
  }

  const modes = contract.layers?.kernel?.modes;
  if (!modes || typeof modes !== 'object' || Array.isArray(modes)) {
    problem('layers.kernel.modes missing (single-kernel dual-mode requires wallClock and tickFrame)');
  } else {
    for (const mode of ['wallClock', 'tickFrame']) {
      const spec = modes[mode];
      if (!spec || typeof spec !== 'object') {
        problem(`layers.kernel.modes.${mode} missing`);
        continue;
      }
      if (typeof spec.owns !== 'string' || spec.owns.length < 10) {
        problem(`layers.kernel.modes.${mode}.owns must be a meaningful string`);
      }
    }
    const shared = contract.layers.kernel.shared;
    const sharedSet = new Set(Array.isArray(shared) ? shared : []);
    for (const item of ['TimerHandle', 'CallbackSlot', 'errorCodes']) {
      if (!sharedSet.has(item)) problem(`layers.kernel.shared must include ${item}`);
    }
  }

  if (contract.consumers?.reconnectDeadline?.layer !== 'kernel:wallClock') {
    problem('consumers.reconnectDeadline.layer must be kernel:wallClock');
  }

  const surface = contract.abiSurface;
  if (!surface || typeof surface !== 'object' || Array.isArray(surface)) {
    problem('abiSurface missing (hosted reachable timer function set)');
    return;
  }

  const functions = surface.functions;
  if (!Array.isArray(functions) || functions.length === 0) {
    problem('abiSurface.functions must be a non-empty array');
    return;
  }

  const names = functions.map((fn) => fn?.name);
  for (const required of TIMER_ABI_REQUIRED_FUNCTIONS) {
    if (!names.includes(required)) problem(`abiSurface.functions missing ${required}`);
  }
  if (new Set(names.filter((name) => typeof name === 'string')).size !== names.length) {
    problem('abiSurface.functions contains duplicate or unnamed entries');
  }

  if (surface.managerHandleAfterDestroy !== 'shutdown-tombstone') {
    problem('abiSurface.managerHandleAfterDestroy must be shutdown-tombstone (destroy leaves a resolvable handle that returns manager_shutdown)');
  }
  if (surface.afterDestroyStatus !== 'manager_shutdown') {
    problem('abiSurface.afterDestroyStatus must be manager_shutdown');
  }
  const lifecycle = contract.fieldSemantics?.managerLifecycle ?? '';
  if (lifecycle.includes('立即失效')) {
    problem('fieldSemantics.managerLifecycle must not say 立即失效; destroy leaves a shutdown tombstone');
  }
  const firingDispatch = contract.sharedTypes?.FiringRecord?.required?.slotDispatchId;
  const drainDispatch = surface.drainRecord?.fields?.find((field) => field.name === 'slotDispatchId')?.type;
  if (firingDispatch !== 'u32' || drainDispatch !== 'u32' || firingDispatch !== drainDispatch) {
    problem(`FiringRecord.slotDispatchId (${JSON.stringify(firingDispatch)}) must equal drainRecord.slotDispatchId (u32), not a second type`);
  }

  const mapping = surface.errorCodeMapping;
  if (!mapping || typeof mapping !== 'object' || Array.isArray(mapping)) {
    problem('abiSurface.errorCodeMapping missing');
  } else {
    const used = new Set();
    for (const code of collectErrorCodes(contract)) {
      const numeric = mapping[code];
      if (!Number.isInteger(numeric) || numeric < 6) {
        problem(`abiSurface.errorCodeMapping.${code} must be an integer >= 6 (0-5 are ABI/CLR status)`);
        continue;
      }
      if (used.has(numeric)) problem(`abiSurface.errorCodeMapping reuses status ${numeric}`);
      used.add(numeric);
    }
  }

  for (const fn of functions) {
    const name = fn?.name ?? '<unnamed>';
    if (typeof fn?.type !== 'string' || parseStatusFnParams(fn.type) === null) {
      problem(`abiSurface.${name}.type must be fn(...) -> status`);
    }
    if (!Array.isArray(fn?.params)) {
      problem(`abiSurface.${name}.params must be an array`);
      continue;
    }
    const declared = parseStatusFnParams(fn.type) ?? [];
    if (declared.length !== fn.params.length) {
      problem(`abiSurface.${name}.type param count ${declared.length} != params.length ${fn.params.length}`);
    }
    fn.params.forEach((param, index) => {
      const type = param?.type;
      if (isForbiddenFnPointerType(type) || (typeof type === 'string' && type.includes('fn('))) {
        problem(`abiSurface.${name} param ${param?.name ?? index} forbids function-pointer type ${JSON.stringify(type)}`);
        return;
      }
      if (!TIMER_ABI_PARAM_TYPES.has(type)) {
        problem(`abiSurface.${name} param ${param?.name ?? index} type ${JSON.stringify(type)} is not an opaque handle or integer`);
      }
      if (declared[index] && declared[index] !== type) {
        problem(`abiSurface.${name} param ${param?.name ?? index} type ${type} != type-string slot ${declared[index]}`);
      }
    });
  }

  for (const testCase of [...(contract.testCases ?? []), ...(contract.invalidCases ?? [])]) {
    if (testCase.layer && !TIMER_KERNEL_LAYERS.has(testCase.layer)) {
      problem(`${testCase.name}: layer ${JSON.stringify(testCase.layer)} must be kernel:wallClock or kernel:tickFrame`);
    }
  }
}

function checkTimerAbiAlignment(contract, abiDefinition, problems, fileName = 'native-timer-abi-v1.json') {
  if (contract.contractId !== TIMER_CONTRACT_ID) return;
  const problem = (msg) => problems.push(`${fileName}: ${msg}`);
  const fields = abiDefinition?.root?.fields;
  if (!Array.isArray(fields)) {
    problem('native-abi.json root.fields missing while aligning abiSurface');
    return;
  }
  const byName = new Map(fields.map((field) => [field.name, field]));
  for (const fn of contract.abiSurface?.functions ?? []) {
    const field = byName.get(fn.name);
    if (!field) {
      problem(`native-abi.json root.fields missing ${fn.name}`);
      continue;
    }
    if (field.type !== fn.type) {
      problem(`native-abi.json ${fn.name}.type ${JSON.stringify(field.type)} != abiSurface ${JSON.stringify(fn.type)}`);
    }
    const params = parseStatusFnParams(field.type) ?? [];
    for (const paramType of params) {
      if (isForbiddenFnPointerType(paramType) || !TIMER_ABI_PARAM_TYPES.has(paramType)) {
        problem(`native-abi.json ${fn.name} param type ${paramType} is not an opaque handle or integer`);
      }
    }
  }
  const mapping = contract.abiSurface?.errorCodeMapping ?? {};
  const status = abiDefinition.status ?? {};
  for (const [code, numeric] of Object.entries(mapping)) {
    const statusName = `Timer${code.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join('')}`;
    if (status[statusName] !== numeric) {
      problem(`native-abi.json status.${statusName} must equal errorCodeMapping.${code} (${numeric})`);
    }
  }
  const destroyDoc = String(byName.get('timer_destroy_manager')?.doc ?? '');
  if (destroyDoc.includes('立即失效')) {
    problem('native-abi.json timer_destroy_manager doc must not use CLR immediate-invalidate (立即失效); handle stays a shutdown tombstone');
  }
  if (destroyDoc.length > 0 && (!destroyDoc.includes('TimerManagerShutdown') || !/tombstone|墓碑/.test(destroyDoc))) {
    problem('native-abi.json timer_destroy_manager doc must describe a shutdown tombstone that later calls observe as TimerManagerShutdown');
  }
}

// ---------- Entity binding / query (C-2′ Runtime-issued NetEntityId + generated table) ----------

const BINDING_CONTRACT_ID = 'lumio.entity-binding-query.v1';
const BINDING_RECORD_FIELDS = ['accountId', 'roomId', 'netEntityId', 'entityType', 'connectionGeneration'];
const BINDING_RECORD_FIELD_SET = new Set(BINDING_RECORD_FIELDS);
const DECLARATION_ROW_KEYS = ['attributeId', 'persistence', 'replication', 'valueType', 'visibility'];
const N04_ATTRIBUTE_DECLARATIONS_SHA256 = 'fbe1d5e68533dff6f36605d42727bf2cf29382f72c3b73c7747355471c296c9c';

function canonicalizeDeclarationTable(table) {
  if (!Array.isArray(table)) throw new Error('declaration table must be an array');
  const rows = [...table].sort((left, right) => {
    const a = String(left?.attributeId ?? '');
    const b = String(right?.attributeId ?? '');
    if (a < b) return -1;
    if (a > b) return 1;
    return 0;
  });
  let out = '[\n';
  rows.forEach((row, index) => {
    out += '  {\n';
    DECLARATION_ROW_KEYS.forEach((key, keyIndex) => {
      out += `    ${JSON.stringify(key)}: ${JSON.stringify(row?.[key] ?? '')}`;
      if (keyIndex !== DECLARATION_ROW_KEYS.length - 1) out += ',';
      out += '\n';
    });
    out += '  }';
    if (index !== rows.length - 1) out += ',';
    out += '\n';
  });
  out += ']\n';
  return out;
}

function hashDeclarationTable(table) {
  return createHash('sha256').update(canonicalizeDeclarationTable(table), 'utf8').digest('hex');
}

function admitBindingRecord(record) {
  if (typeof record !== 'object' || record === null || Array.isArray(record)) {
    throw new Rejection('invalid_binding_shape', 'binding record must be a JSON object');
  }
  for (const key of Object.keys(record)) {
    if (!BINDING_RECORD_FIELD_SET.has(key)) {
      throw new Rejection('invalid_binding_shape', `binding record carries forbidden field ${key}`);
    }
  }
  for (const key of BINDING_RECORD_FIELDS) {
    if (!(key in record)) {
      throw new Rejection('invalid_binding_shape', `binding record missing ${key}`);
    }
  }
}

function checkEntityBindingContract(contract, fileName, problems) {
  if (contract.contractId !== BINDING_CONTRACT_ID) return;
  const problem = (msg) => problems.push(`${fileName}: ${msg}`);

  const netEntityId = contract.identityModel?.netEntityId ?? '';
  if (!netEntityId.includes('Runtime 身份表发号')) {
    problem('identityModel.netEntityId must say Runtime 身份表发号');
  }
  if (!netEntityId.includes('宿主准入时调用 Runtime 取号')) {
    problem('identityModel.netEntityId must say 宿主准入时调用 Runtime 取号');
  }
  if (!netEntityId.includes('宿主不得自铸')) {
    problem('identityModel.netEntityId must say 宿主不得自铸');
  }

  const record = contract.binding?.record;
  const requiredKeys = Object.keys(record?.required ?? {});
  if (requiredKeys.length !== BINDING_RECORD_FIELDS.length || BINDING_RECORD_FIELDS.some((key) => !requiredKeys.includes(key))) {
    problem(`binding.record.required must be exactly the five-tuple ${BINDING_RECORD_FIELDS.join(' / ')}`);
  }
  if (Object.keys(record?.optional ?? {}).length !== 0) {
    problem('binding.record.optional must be empty; binding records are five-tuple only');
  }
  const admitResult = String(contract.binding?.operations?.admit?.result ?? '');
  if (!/accepted/.test(admitResult) || (/netEntityId/.test(admitResult) && !/不返回 netEntityId/.test(admitResult))) {
    problem('binding.operations.admit.result must be accepted/rejection only and must not return netEntityId');
  }
  if (Object.prototype.hasOwnProperty.call(contract.binding?.operations ?? {}, 'listBindings')) {
    problem('binding.operations.listBindings is removed by C-2');
  }
  const notes = record?.notes ?? '';
  if (!notes.includes('会话号') || !notes.includes('宿主内部句柄') || !notes.includes('不得出现在绑定记录')) {
    problem('binding.record.notes must forbid 会话号 and 宿主内部句柄 on the binding record');
  }

  const decls = contract.attributeDeclarations;
  if (!decls || typeof decls !== 'object' || Array.isArray(decls)) {
    problem('attributeDeclarations missing');
    return;
  }
  if (decls.source !== 'generated-from-field-annotations') {
    problem('attributeDeclarations.source must be generated-from-field-annotations');
  }
  if (Object.prototype.hasOwnProperty.call(decls, 'example')) {
    problem('attributeDeclarations.example is a handwritten second table; delete it');
  }
  if (!Array.isArray(decls.table) || decls.table.length === 0) {
    problem('attributeDeclarations.table must be the generated declaration array');
    return;
  }
  const ids = new Set();
  for (const [index, row] of decls.table.entries()) {
    if (!row || typeof row !== 'object' || Array.isArray(row)) {
      problem(`attributeDeclarations.table[${index}] must be an object`);
      continue;
    }
    const keys = Object.keys(row);
    if (keys.length !== DECLARATION_ROW_KEYS.length || DECLARATION_ROW_KEYS.some((key) => !keys.includes(key))) {
      problem(`attributeDeclarations.table[${index}] must have exactly ${DECLARATION_ROW_KEYS.join(', ')}`);
    }
    if (row.attributeId === 'EntityIdentity.accountId') {
      problem('EntityIdentity.accountId must not appear in the generated declaration table');
    }
    if (ids.has(row.attributeId)) problem(`attributeDeclarations.table duplicate ${row.attributeId}`);
    ids.add(row.attributeId);
  }
  let digest;
  try {
    digest = hashDeclarationTable(decls.table);
  } catch (error) {
    problem(`attributeDeclarations.table is not canonicalizable: ${error.message}`);
    return;
  }
  if (decls.sha256 !== digest) {
    problem(`attributeDeclarations.sha256 ${JSON.stringify(decls.sha256)} does not recompute from table (${digest})`);
  }
  if (ids.has('EntityIdentity.entityType') || ids.has('EntityIdentity.claimedMark') || ids.has('EntityIdentity.unmappedMark') || ids.has('ChatComponent.lastMessagePersistOnly')) {
    problem('attributeDeclarations.table must omit EntityIdentity.* and lastMessagePersistOnly; entityType is derived');
  }
  if (!contract.derived?.entityType || !String(contract.derived.entityType.source).includes('TypeOf')) problem('derived.entityType must come from World.TypeOf');
  if (!String(contract.derived?.tombstoned ?? '').includes('next-issued-counter')) problem('derived.tombstoned must use counter < next-issued-counter and live-set absence');
  if (!contract.claim?.credential || !String(contract.claim.credential).includes('claimBy')) problem('claim credential must use target entity claimBy named-list field');
  if (fileName === 'entity-binding-and-query-v1.json' && decls.sha256 !== N04_ATTRIBUTE_DECLARATIONS_SHA256) {
    problem(`attributeDeclarations.sha256 must be the N-04 digest ${N04_ATTRIBUTE_DECLARATIONS_SHA256}`);
  }

  const readRules = decls.readRules;
  const accountRule = Array.isArray(readRules)
    ? readRules.find((rule) => typeof rule === 'string' && rule.includes('EntityIdentity.accountId'))
    : null;
  if (!accountRule || !accountRule.includes('undeclared_attribute')) {
    problem('readRules must say EntityIdentity.accountId is undeclared_attribute');
  }

  const invalidCases = contract.invalidCases ?? [];
  const byName = new Map(invalidCases.map((item) => [item.name, item]));
  for (const name of ['host_minted_net_entity_id', 'binding_record_carries_session_id']) {
    const item = byName.get(name);
    if (!item) {
      problem(`invalidCases missing ${name}`);
      continue;
    }
    if (item.expectedRejection !== 'invalid_binding_shape') {
      problem(`invalidCases.${name}.expectedRejection must be invalid_binding_shape`);
    }
    try {
      admitBindingRecord(item.payload);
      problem(`invalidCases.${name}: expected rejection, was accepted`);
    } catch (error) {
      if (!(error instanceof Rejection) || error.code !== 'invalid_binding_shape') {
        problem(`invalidCases.${name}: rejected with [${error.code ?? error.message}] but expected [invalid_binding_shape]`);
      }
    }
  }
  const accountCase = invalidCases.find((item) => item.payload?.attributeId === 'EntityIdentity.accountId');
  if (!accountCase) {
    problem('invalidCases must include a query of EntityIdentity.accountId');
  } else if (accountCase.expectedRejection !== 'undeclared_attribute') {
    problem(`${accountCase.name}: expectedRejection must be undeclared_attribute`);
  } else if (ids.has('EntityIdentity.accountId')) {
    problem('EntityIdentity.accountId query case is undeclared_attribute but the id is in the table');
  }

  if (fileName === 'entity-binding-and-query-v1.json') {
    const roomId = contract.identityModel?.roomId ?? '';
    if (!roomId.includes('宿主路由键') || !roomId.includes('World Manager') || !roomId.includes('GameWorld')) {
      problem('identityModel.roomId must describe the host routing key (one process / one World Manager / one GameWorld)');
    }
    const crossRoom = contract.errorCodes?.semantics?.cross_room_reference ?? '';
    if (!crossRoom.includes('宿主路由')) {
      problem('errorCodes.semantics.cross_room_reference must assign cross_room to the host routing layer');
    }
    if (!netEntityId.includes('128') || !netEntityId.includes('32-hex')) {
      problem('identityModel.netEntityId must declare 128-bit = instanceId (high 64) + counter (low 64) with 32-hex encoding');
    }
    if (!notes.includes('IdentityComponent') || !notes.includes('不持独立绑定表')) {
      problem('binding.record.notes must assemble the five-tuple from IdentityComponent fields + host session table; Runtime holds no binding table');
    }
    const adapterNotes = `${decls.notes ?? ''} ${decls.structure?.notes ?? ''}`;
    if (!adapterNotes.includes('薄适配层') || !adapterNotes.includes('无自有存储')) {
      problem('attributeDeclarations must say the query surface is a generated thin adapter with no private storage');
    }
    if (!contract.outcomes?.account_already_online) {
      problem('outcomes.account_already_online missing');
    }
    const admitCodes = contract.errorCodes?.admitOutcomeCodes ?? [];
    const outcomeCodes = contract.errorCodes?.outcomeCodes ?? [];
    if (!admitCodes.includes('account_already_online') && !outcomeCodes.includes('account_already_online')) {
      problem('account_already_online must be listed in admitOutcomeCodes (distinct from invalid_binding_shape)');
    }
    if ((contract.errorCodes?.requestErrorCodes ?? []).includes('account_already_online')) {
      problem('account_already_online must not be folded into requestErrorCodes');
    }
    if (!contract.binding?.operations?.admit) {
      problem('binding.operations.admit missing');
    }
    const testByName = new Map((contract.testCases ?? []).map((item) => [item.name, item]));
    const already = testByName.get('admit_second_connection_account_already_online');
    if (!already) {
      problem('testCases missing admit_second_connection_account_already_online');
    } else if (already.expect !== 'account_already_online') {
      problem('testCases.admit_second_connection_account_already_online.expect must be account_already_online');
    } else if (!String(already.then ?? '').includes('netEntityId')) {
      problem('testCases.admit_second_connection_account_already_online must return the existing netEntityId');
    }
    const shapeCase = byName.get('admit_shape_error_is_not_account_already_online');
    if (!shapeCase) {
      problem('invalidCases missing admit_shape_error_is_not_account_already_online');
    } else if (shapeCase.expectedRejection !== 'invalid_binding_shape') {
      problem('invalidCases.admit_shape_error_is_not_account_already_online.expectedRejection must be invalid_binding_shape');
    } else {
      try {
        admitBindingRecord(shapeCase.payload);
        problem('invalidCases.admit_shape_error_is_not_account_already_online: expected rejection, was accepted');
      } catch (error) {
        if (!(error instanceof Rejection) || error.code !== 'invalid_binding_shape') {
          problem(`invalidCases.admit_shape_error_is_not_account_already_online: rejected with [${error.code ?? error.message}] but expected [invalid_binding_shape]`);
        }
      }
    }
  }
}

// ---------- Contract validation ----------

const ENVELOPE_CONTRACT_ID = 'lumio.gameplay-envelope.v1';
const N04_DECLARATIONS_SHA256 = 'fbe1d5e68533dff6f36605d42727bf2cf29382f72c3b73c7747355471c296c9c';

function serializeAttributeDeclarations(declarations) {
  return `${JSON.stringify(declarations, null, 2)}\n`;
}

function attributeDeclarationsSha256(declarations) {
  return createHash('sha256').update(serializeAttributeDeclarations(declarations)).digest('hex');
}

function checkGameplayEnvelopeContract(contract, fileName, problems) {
  if (contract.contractId !== ENVELOPE_CONTRACT_ID) return;
  const problem = (msg) => problems.push(`${fileName}: ${msg}`);

  // R5-01 C-1 is the World Manager packet shape. Keep these checks ahead of
  // the historical C-1 assertions below so old FullSnapshot/Delta contracts
  // cannot pass by accident.
  if (Object.prototype.hasOwnProperty.call(contract.messages ?? {}, 'Welcome')) {
    const expectedMessages = ['Welcome', 'WorldChange', 'InputCommand', 'ConnectionSuperseded', 'Error'];
    if (JSON.stringify(Object.keys(contract.messages)) !== JSON.stringify(expectedMessages)) problem('messages must be exactly Welcome, WorldChange, InputCommand, ConnectionSuperseded, Error');
    for (const removed of ['FullSnapshot', 'Delta']) if (contract.messages[removed]) problem(`messages.${removed} is removed by C-1`);
    for (const removed of ['entity.identity', 'chat.event', 'chat.component']) if (contract.mappings?.[removed]) problem(`mappings.${removed} is removed by C-1`);
    if (contract.limits?.createsPerPack !== 0) problem('limits.createsPerPack must be 0 (unlimited)');
    const world = contract.messages.WorldChange?.required ?? {};
    for (const key of ['tick', 'creates', 'fields', 'destroys', 'rpcs']) if (!world[key]) problem(`WorldChange.required.${key} missing`);
    const create = contract.sharedTypes?.CreateRecord?.required ?? {};
    const change = contract.sharedTypes?.FieldChange?.required ?? {};
    const destroy = contract.sharedTypes?.DestroyRecord?.required ?? {};
    const rpc = contract.sharedTypes?.ClientRpcRecord?.required ?? {};
    for (const [label, spec] of [['CreateRecord', create], ['FieldChange', change], ['DestroyRecord', destroy]]) if (spec.netEntityId !== 'hex128') problem(`${label}.netEntityId must be hex128`);
    for (const key of ['target', 'sender']) if (rpc[key] !== 'hex128') problem(`ClientRpcRecord.${key} must be hex128`);
    if (rpc.roomSequence !== 'u64') problem('ClientRpcRecord.roomSequence must be u64');
    if (contract.messages.ConnectionSuperseded?.required?.netEntityId !== 'hex128') problem('ConnectionSuperseded.netEntityId must be hex128');
    const input = contract.messages.InputCommand?.required ?? {};
    if (input.commands !== 'array:CommandBlock') problem('InputCommand.commands must carry CommandBlock payloads');
    const hashes = contract.hash?.examples ?? [];
    for (const example of hashes) {
      const digest = sha256Hex(Buffer.from(example.payload ?? '', 'hex'));
      if (digest !== example.payloadSha256) problem(`hash.examples ${example.mappingId} does not recompute`);
    }
    const names = new Set((contract.testCases ?? []).map((item) => item.name));
    for (const required of ['welcome/128-bit-self', 'world-change/creation-field-rpc', 'input/chat']) if (!names.has(required)) problem(`testCases missing ${required}`);
    return;
  }

  const snapshotNotes = contract.messages?.FullSnapshot?.notes ?? '';
  if (!/Room/.test(snapshotNotes) || !/唯一快照载体/.test(snapshotNotes)) {
    problem('messages.FullSnapshot.notes must declare stateBlocks as the Room-path-only snapshot carrier');
  }
  if (!/ADR-045/.test(snapshotNotes)) {
    problem('messages.FullSnapshot.notes must reject the ADR-045 five-field body as this contract\'s FullSnapshot');
  }
  if (!/活体实体/.test(snapshotNotes)) {
    problem('messages.FullSnapshot.notes must require stateBlocks to include replicated state of every live Room entity');
  }
  if (!/entity\.identity/.test(snapshotNotes)) {
    problem('messages.FullSnapshot.notes must name entity.identity as the live-entity census mapping');
  }

  const superseded = contract.messages?.ConnectionSuperseded;
  if (!superseded) {
    problem('messages.ConnectionSuperseded missing');
  } else {
    if (superseded.dir !== 's2c') problem('messages.ConnectionSuperseded.dir must be s2c');
    const required = superseded.required ?? {};
    if (required.messageType !== 'const:ConnectionSuperseded') {
      problem('ConnectionSuperseded.required.messageType must be const:ConnectionSuperseded');
    }
    if (required.reasonCode !== 'const:connection_superseded') {
      problem('ConnectionSuperseded.required.reasonCode must be const:connection_superseded');
    }
    if (required.netEntityId !== 'u64') problem('ConnectionSuperseded.required.netEntityId must be u64');
    if (required.newConnectionGeneration !== 'u64') {
      problem('ConnectionSuperseded.required.newConnectionGeneration must be u64');
    }
    if (!/再关闭/.test(superseded.notes ?? '')) {
      problem('ConnectionSuperseded.notes must require the old connection to receive the notice before the server closes it');
    }
  }

  const generated = contract.generatedAttributeDeclarations;
  if (!generated || generated.source !== 'generated-from-field-annotations') {
    problem('generatedAttributeDeclarations.source must be generated-from-field-annotations');
  } else if (!Array.isArray(generated.declarations)) {
    problem('generatedAttributeDeclarations.declarations missing (N-04 copy)');
  } else {
    const digest = attributeDeclarationsSha256(generated.declarations);
    if (generated.sha256 !== N04_DECLARATIONS_SHA256) {
      problem(`generatedAttributeDeclarations.sha256 ${generated.sha256} != N-04 ${N04_DECLARATIONS_SHA256}`);
    }
    if (digest !== N04_DECLARATIONS_SHA256) {
      problem(`generatedAttributeDeclarations.declarations sha256 ${digest} != N-04 ${N04_DECLARATIONS_SHA256}`);
    }
  }

  for (const [id, mapping] of Object.entries(contract.mappings ?? {})) {
    const dim = mapping.dimensions;
    if (!dim || typeof dim !== 'object') continue;
    if (dim.source !== 'generated-from-field-annotations') {
      problem(`mappings.${id}.dimensions.source must be generated-from-field-annotations`);
    }
    if (dim.sha256 !== N04_DECLARATIONS_SHA256) {
      problem(`mappings.${id}.dimensions.sha256 ${dim.sha256} != N-04 ${N04_DECLARATIONS_SHA256}`);
    }
  }

  const chatDim = contract.mappings?.['chat.component']?.dimensions;
  const chatRows = (generated?.declarations ?? []).filter((row) => String(row.attributeId).startsWith('ChatComponent.'));
  if (chatRows.length > 0 && chatDim) {
    for (const row of chatRows) {
      if (row.persistence !== chatDim.persistence || row.replication !== chatDim.replication || row.visibility !== chatDim.visibility) {
        problem(`chat.component dimensions ${chatDim.persistence}/${chatDim.replication}/${chatDim.visibility} != ChatComponent annotation ${row.attributeId} ${row.persistence}/${row.replication}/${row.visibility}`);
      }
    }
  }

  const eventNotes = contract.mappings?.['chat.event']?.notes ?? '';
  if (!/Delta\.changedBlocks/.test(eventNotes) || !/eventOrder/.test(eventNotes) || !/appliedTicks/.test(eventNotes) || !/restoredWindow/.test(eventNotes)) {
    problem('mappings.chat.event.notes must pin acceptance on client-received Delta.changedBlocks and forbid harness-synthesized eventOrder/appliedTicks/restoredWindow');
  }

  const identity = contract.mappings?.['entity.identity'];
  if (!identity) {
    problem('mappings.entity.identity missing (Room-path live-entity identity census)');
  } else {
    if (identity.kind !== 'state') problem('mappings.entity.identity.kind must be state');
    if (identity.direction !== 's2c') problem('mappings.entity.identity.direction must be s2c');
    if (identity.collection !== 'array') problem('mappings.entity.identity.collection must be array');
    if (identity.orderBy !== 'netEntityId') problem('mappings.entity.identity.orderBy must be netEntityId');
    const order = identity.fieldOrder ?? [];
    if (order.length !== 3 || order[0] !== 'netEntityId' || order[1] !== 'entityType' || order[2] !== 'unmappedMark') {
      problem('mappings.entity.identity.fieldOrder must be netEntityId, entityType, unmappedMark');
    }
    if (identity.fields?.netEntityId?.type !== 'u64') problem('mappings.entity.identity.netEntityId must be u64');
    if (identity.fields?.entityType?.type !== 'utf8-string') {
      problem('mappings.entity.identity.entityType must be utf8-string (no new binary type)');
    }
    const allowed = identity.fields?.entityType?.allowedValues;
    if (!Array.isArray(allowed) || allowed.length !== 2 || allowed[0] !== 'player' || allowed[1] !== 'bot') {
      problem('mappings.entity.identity.entityType.allowedValues must be player, bot');
    }
    if (identity.fields?.unmappedMark?.type !== 'utf8-string') {
      problem('mappings.entity.identity.unmappedMark must be utf8-string');
    }
    if (identity.fields?.claimedMark || order.includes('claimedMark')) {
      problem('mappings.entity.identity must not carry EntityIdentity.claimedMark');
    }
    if (!/claimedMark/.test(identity.notes ?? '')) {
      problem('mappings.entity.identity.notes must say claimedMark is omitted from this census block');
    }
  }
  const stateIds = Object.entries(contract.mappings ?? {})
    .filter(([, spec]) => spec.kind === 'state')
    .map(([id]) => id);
  if (stateIds.length !== 1 || stateIds[0] !== 'entity.identity') {
    problem(`kind=state mappings must be exactly entity.identity, got ${stateIds.join(',') || '(none)'}`);
  }

  const identityDim = contract.mappings?.['entity.identity']?.dimensions;
  const identityRows = (generated?.declarations ?? []).filter(
    (row) => row.attributeId === 'EntityIdentity.entityType' || row.attributeId === 'EntityIdentity.unmappedMark',
  );
  if (identityRows.length > 0 && identityDim) {
    for (const row of identityRows) {
      if (row.persistence !== identityDim.persistence || row.replication !== identityDim.replication || row.visibility !== identityDim.visibility) {
        problem(
          `entity.identity dimensions ${identityDim.persistence}/${identityDim.replication}/${identityDim.visibility} != ${row.attributeId} ${row.persistence}/${row.replication}/${row.visibility}`,
        );
      }
    }
  }

  const testNames = new Set((contract.testCases ?? []).map((item) => item.name));
  if (!testNames.has('snapshot/two-live-entities')) problem('testCases missing snapshot/two-live-entities');
  if (!testNames.has('input/field-write-owner-name')) problem('testCases missing input/field-write-owner-name');
  if (!testNames.has('delta/chat-event')) problem('testCases missing delta/chat-event');

  const names = new Set((contract.invalidCases ?? []).map((item) => item.name));
  for (const requiredName of [
    'full_snapshot_without_state_blocks',
    'full_snapshot_adr045_shape',
    'runtime/connection-superseded-close-before-send',
    'snapshot/unregistered-binding-mapping-id',
    'snapshot/entity-identity-unsorted-records',
    'snapshot/entity-identity-illegal-entity-type',
    'snapshot/event-replay',
    'runtime/field-write-other-entity',
    'runtime/field-write-server-authority',
  ]) {
    if (!names.has(requiredName)) problem(`invalidCases missing ${requiredName}`);
  }

  const roomSeq = contract.fieldSemantics?.roomSequence ?? '';
  if (!roomSeq.includes('世界内') || !/NetEntityId/.test(roomSeq)) {
    problem('fieldSemantics.roomSequence must be the in-world strictly increasing sequence assigned after sorting senders by NetEntityId');
  }

  const identityNotes = identity?.notes ?? '';
  if (!identityNotes.includes('创建记录')) {
    problem('mappings.entity.identity.notes must promote the census block to 创建记录 semantics');
  }

  const fieldWrite = contract.mappings?.['field.write'];
  if (!fieldWrite) {
    problem('mappings.field.write missing (Authority.Owner uplink)');
  } else {
    if (fieldWrite.kind !== 'command') problem('mappings.field.write.kind must be command');
    if (fieldWrite.direction !== 'c2s') problem('mappings.field.write.direction must be c2s');
    const fwOrder = fieldWrite.fieldOrder ?? [];
    if (
      fwOrder.length !== 4
      || fwOrder[0] !== 'netEntityId'
      || fwOrder[1] !== 'componentId'
      || fwOrder[2] !== 'fieldId'
      || fwOrder[3] !== 'value'
    ) {
      problem('mappings.field.write.fieldOrder must be netEntityId, componentId, fieldId, value');
    }
    if (fieldWrite.fields?.netEntityId?.type !== 'u64') {
      problem('mappings.field.write.netEntityId must be u64 (this slice; 128-bit wire encoding is the two-u64 chat.event pair)');
    }
    if (fieldWrite.fields?.componentId?.type !== 'utf8-string' || fieldWrite.fields?.fieldId?.type !== 'utf8-string') {
      problem('mappings.field.write.componentId and fieldId must be utf8-string');
    }
    if (fieldWrite.fields?.value?.type !== 'utf8-string') {
      problem('mappings.field.write.value must be utf8-string for this slice (IdentityComponent.name)');
    }
    const fwNotes = fieldWrite.notes ?? '';
    if (!fwNotes.includes('Authority.Owner') || !fwNotes.includes('权威纠正')) {
      problem('mappings.field.write.notes must describe Authority.Owner uplink and authority correction');
    }
  }

  const chatEvent = contract.mappings?.['chat.event'];
  const eventOrder = chatEvent?.fieldOrder ?? [];
  if (
    eventOrder.length !== 6
    || eventOrder[0] !== 'messageId'
    || eventOrder[1] !== 'roomSequence'
    || eventOrder[2] !== 'senderNetEntityIdInstanceId'
    || eventOrder[3] !== 'senderNetEntityIdCounter'
    || eventOrder[4] !== 'text'
    || eventOrder[5] !== 'appliedTick'
  ) {
    problem('chat.event.fieldOrder must be messageId, roomSequence, senderNetEntityIdInstanceId, senderNetEntityIdCounter, text, appliedTick');
  }
  if (chatEvent?.fields?.senderNetEntityId || eventOrder.includes('senderNetEntityId')) {
    problem('chat.event must not keep a single senderNetEntityId field; ADR-047 has no u128 primitive');
  }
  if (chatEvent?.fields?.senderNetEntityIdInstanceId?.type !== 'u64' || chatEvent?.fields?.senderNetEntityIdCounter?.type !== 'u64') {
    problem('senderNetEntityIdInstanceId and senderNetEntityIdCounter must be u64 (16-byte LE pair)');
  }
  const eventNotesFull = chatEvent?.notes ?? '';
  if (!eventNotesFull.includes('OnChatMessage') || !eventNotesFull.includes('ClientRpc')) {
    problem('mappings.chat.event.notes must name ChatComponent.OnChatMessage ClientRpc');
  }
  if (!eventNotesFull.includes('32-hex')) {
    problem('mappings.chat.event.notes must say the 16-byte pair is the same 128-bit value as C-2 32-hex');
  }
  const inputNotes = contract.mappings?.['chat.input']?.notes ?? '';
  if (!inputNotes.includes('SendMessage') || !inputNotes.includes('ServerRpc')) {
    problem('mappings.chat.input.notes must name ChatComponent.SendMessage ServerRpc');
  }

  const otherEntity = (contract.invalidCases ?? []).find((item) => item.name === 'runtime/field-write-other-entity');
  const serverAuth = (contract.invalidCases ?? []).find((item) => item.name === 'runtime/field-write-server-authority');
  for (const item of [otherEntity, serverAuth]) {
    if (!item) continue;
    if (item.expectedRejection !== 'unauthorized') {
      problem(`invalidCases.${item.name}.expectedRejection must be unauthorized`);
    }
    if (item.validatorCheck !== false) {
      problem(`invalidCases.${item.name}.validatorCheck must be false (receiver-enforced)`);
    }
  }
  if (Array.isArray(contract.errorCodes) && !contract.errorCodes.includes('unauthorized')) {
    problem('errorCodes must include unauthorized for field.write owner-only receiver cases');
  }
}

function validateContract(contract, fileName, abiDefinition) {
  const problems = [];
  const caseCount = checkStructure(contract, fileName, problems);
  checkGameplayEnvelopeContract(contract, fileName, problems);
  checkNativeTimerContract(contract, fileName, problems);
  checkEntityBindingContract(contract, fileName, problems);
  if (abiDefinition) checkTimerAbiAlignment(contract, abiDefinition, problems, fileName);

  let pass = 0;
  let executed = 0;
  const runCase = (label, fn) => {
    executed += 1;
    try {
      fn();
      pass += 1;
      return null;
    } catch (error) {
      if (error instanceof Rejection) return error;
      problems.push(`${fileName} ${label}: internal error ${error.message}`);
      return null;
    }
  };

  const envelopePayload = (item) => {
    const candidate = item?.message ?? item?.payload;
    if (candidate && typeof candidate === 'object' && typeof candidate.messageType === 'string') return candidate;
    return null;
  };

  if (contract.testCases) {
    for (const testCase of contract.testCases) {
      const message = envelopePayload(testCase);
      if (!message) {
        runCase(`testCases.${testCase.name} (declaration)`, () => {
          if (!testCase.name) throw new Rejection('bad_envelope', 'scenario case must have a name');
        });
        continue;
      }
      const rejection = runCase(`testCases.${testCase.name}`, () => {
        checkMessageShape(message, contract);
        checkMessageSemantics(message, contract);
      });
      if (rejection) problems.push(`${fileName} testCases.${testCase.name}: expected valid, rejected [${rejection.code}] ${rejection.message}`);
    }
  }
  if (contract.invalidCases) {
    for (const invalidCase of contract.invalidCases) {
      if (invalidCase.validatorCheck === false) {
        runCase(`invalidCases.${invalidCase.name} (declaration)`, () => {
          if (!invalidCase.payload && !invalidCase.given && !invalidCase.when) {
            throw new Rejection('bad_envelope', 'receiver-side case must still carry a scenario payload or given/when');
          }
        });
        continue;
      }
      const message = envelopePayload(invalidCase);
      if (!message) {
        runCase(`invalidCases.${invalidCase.name} (declaration)`, () => {
          if (!invalidCase.expectedRejection) throw new Rejection('bad_envelope', 'non-envelope invalidCase must declare expectedRejection');
        });
        continue;
      }
      const rejection = runCase(`invalidCases.${invalidCase.name}`, () => {
        checkMessageShape(message, contract);
        checkMessageSemantics(message, contract);
      });
      if (!rejection) problems.push(`${fileName} invalidCases.${invalidCase.name}: expected rejection, was accepted`);
      else if (rejection.code !== invalidCase.expectedRejection) {
        problems.push(`${fileName} invalidCases.${invalidCase.name}: rejected with [${rejection.code}] but expected [${invalidCase.expectedRejection}] (${rejection.message})`);
      }
    }
  }

  const summary = `${fileName}: ${problems.length === 0 ? 'OK' : 'FAIL'} (cases executed: ${executed}, clean passes: ${pass}, problems: ${problems.length})`;
  return { summary, problems };
}

export {
  validateContract,
  wireDir,
  checkNativeTimerContract,
  checkTimerAbiAlignment,
  checkGameplayEnvelopeContract,
  attributeDeclarationsSha256,
  N04_DECLARATIONS_SHA256,
  TIMER_ABI_REQUIRED_FUNCTIONS,
  checkEntityBindingContract,
  admitBindingRecord,
  hashDeclarationTable,
  canonicalizeDeclarationTable,
  N04_ATTRIBUTE_DECLARATIONS_SHA256,
  encodeMappingPayload,
  sha256Hex,
};

async function main() {
  const files = (await readdir(wireDir)).filter((f) => f.endsWith('.json')).sort();
  const abiDefinition = JSON.parse(await readFile(resolve(root, 'engine/abi/native-abi.json'), 'utf8'));
  let failed = false;
  console.log(`verify-wire: ${files.length} contract(s) in engine/wire/`);
  for (const fileName of files) {
    let contract;
    try {
      contract = JSON.parse(await readFile(resolve(wireDir, fileName), 'utf8'));
    } catch (error) {
      console.log(`${fileName}: FAIL (not valid JSON: ${error.message})`);
      failed = true;
      continue;
    }
    const { summary, problems } = validateContract(
      contract,
      fileName,
      contract.contractId === TIMER_CONTRACT_ID ? abiDefinition : undefined,
    );
    console.log(summary);
    for (const problem of problems) console.log(`  - ${problem}`);
    if (contract.contractId === ENVELOPE_CONTRACT_ID) {
      const generated = contract.generatedAttributeDeclarations;
      const digest = Array.isArray(generated?.declarations) ? attributeDeclarationsSha256(generated.declarations) : '(missing)';
      console.log(`  dimensions sha256=${digest} (pinned ${generated?.sha256 ?? '(missing)'}; N-04 ${N04_DECLARATIONS_SHA256})`);
    }
    if (problems.length > 0) failed = true;
  }
  if (failed) {
    console.error('verify-wire: FAILED');
    process.exit(1);
  }
  console.log('verify-wire: all contracts green');
}

const isDirectRun = process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));
if (isDirectRun && !process.env.NODE_TEST_CONTEXT) {
  await main();
}

/* Legacy C-1 tests retained as historical text; R5-01 tests below cover the
 * replacement World Manager packet contract. */
/*
test('gameplay envelope accepts a valid ChatInput InputCommand via shipped admitMessage', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
  const valid = (contract.testCases ?? []).find((c) => c.name === 'input/chat-single-command');
  assert.ok(valid?.message, 'embedded valid ChatInput case must exist');
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
});
*/

test('R5-01 C-1 exposes exactly the five World Manager messages', async () => {
  const contract = await loadEnvelopeContract();
  assert.deepEqual(Object.keys(contract.messages), ['Welcome', 'WorldChange', 'InputCommand', 'ConnectionSuperseded', 'Error']);
  assert.deepEqual(Object.keys(contract.mappings), ['chat.input', 'field.write']);
  assert.equal(contract.limits.createsPerPack, 0);
  assert.deepEqual(validateContract(contract, 'gameplay-command-envelope-v1.json').problems, []);
});

test('R5-01 C-1 accepts Welcome and WorldChange with 128-bit identifiers', async () => {
  const contract = await loadEnvelopeContract();
  for (const item of contract.testCases) assert.doesNotThrow(() => admitMessage(contract, item.message));
  const bad = contract.invalidCases.find((item) => item.name === 'non-128-bit-entity-id');
  assert.throws(() => admitMessage(contract, bad.payload), (error) => error instanceof Rejection && error.code === 'bad_envelope');
});

test('R5-01 C-1 recomputes command payload hashes and rejects mismatches', async () => {
  const contract = await loadEnvelopeContract();
  const bad = contract.invalidCases.find((item) => item.name === 'bad-input-hash');
  assert.throws(() => admitMessage(contract, bad.payload), (error) => error instanceof Rejection && error.code === 'bad_payload_hash');
  const valid = contract.testCases.find((item) => item.name === 'input/chat');
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
});

async function loadEnvelopeContract() {
  return JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
}

test('R5-01 C-2 admit is asynchronous and declaration projections are derived', async () => {
  const contract = JSON.parse(await readFile(resolve(root, 'engine/wire/entity-binding-and-query-v1.json'), 'utf8'));
  assert.match(contract.binding.operations.admit.result, /^accepted/);
  assert.equal(Object.prototype.hasOwnProperty.call(contract.binding.operations, 'listBindings'), false);
  assert.equal(contract.attributeDeclarations.table.some((row) => row.attributeId.startsWith('EntityIdentity.')), false);
  assert.ok(contract.derived.entityType.source.includes('TypeOf'));
  assert.match(contract.derived.tombstoned, /next-issued-counter/);
  assert.match(contract.claim.credential, /claimBy/);
  assert.deepEqual(validateContract(contract, 'entity-binding-and-query-v1.json').problems, []);
});

if (false) {
test('gameplay envelope rejects digest-mismatch with bad_payload_hash via shipped admitMessage', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
  const bad = (contract.invalidCases ?? []).find((c) => c.name === 'input/digest-mismatch');
  assert.equal(bad.expectedRejection, 'bad_payload_hash');
  assert.throws(
    () => admitMessage(contract, bad.payload),
    (error) => error instanceof Rejection && error.code === 'bad_payload_hash',
  );
});

test('shipped envelope contract passes the shipped validator', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
});
if (false) {
test('hello-wire-v1 still passes the unified validator and is not the envelope', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'hello-wire-v1.json'), 'utf8'));
  assert.equal(contract.contractId, 'lumio.hello-wire.v1');
  assert.ok(!contract.mappings);
  const { problems } = validateContract(contract, 'hello-wire-v1.json');
  assert.deepEqual(problems, []);
});

test('wrong-kind InputCommand is unknown_command_type via shipped invalidCase', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'input/registered-wrong-kind');
  assert.ok(ic, 'missing shipped invalidCase input/registered-wrong-kind');
  assert.equal(ic.expectedRejection, 'unknown_command_type');
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
  ic.expectedRejection = 'state_block_kind_mismatch';
  const flipped = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.ok(flipped.problems.some((p) => p.includes('unknown_command_type') && p.includes('input/registered-wrong-kind')));
});

async function loadEnvelopeContract() {
  return JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
}

test('C-1 FullSnapshot without stateBlocks is bad_envelope via full_snapshot_without_state_blocks', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'full_snapshot_without_state_blocks');
  assert.ok(ic, 'missing shipped invalidCase full_snapshot_without_state_blocks');
  assert.equal(ic.expectedRejection, 'bad_envelope');
  assert.equal(ic.validatorCheck, true);
  assert.equal('stateBlocks' in ic.payload, false);
  assert.throws(
    () => admitMessage(contract, ic.payload),
    (error) => error instanceof Rejection && error.code === 'bad_envelope' && /stateBlocks/.test(error.message),
  );
});

test('C-1 FullSnapshot ADR-045 five-field body is bad_envelope via full_snapshot_adr045_shape', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'full_snapshot_adr045_shape');
  assert.ok(ic, 'missing shipped invalidCase full_snapshot_adr045_shape');
  assert.equal(ic.expectedRejection, 'bad_envelope');
  assert.equal(ic.validatorCheck, true);
  const body = ic.payload;
  assert.equal(body.messageType, 'FullSnapshot');
  for (const key of ['snapshotId', 'tickId', 'sessionRevisionVector', 'schemaEpoch', 'mappingSetHash']) {
    assert.ok(key in body, `ADR-045 shape missing ${key}`);
  }
  assert.equal('stateBlocks' in body, false);
  assert.throws(
    () => admitMessage(contract, body),
    (error) => error instanceof Rejection && error.code === 'bad_envelope',
  );
  const notes = contract.messages?.FullSnapshot?.notes ?? '';
  assert.match(notes, /Room/);
  assert.match(notes, /唯一快照载体/);
  assert.match(notes, /ADR-045/);
  assert.match(notes, /活体实体/);
});

test('C-1 ConnectionSuperseded is a required s2c notice accepted by admitMessage', async () => {
  const contract = await loadEnvelopeContract();
  const spec = contract.messages?.ConnectionSuperseded;
  assert.ok(spec, 'messages.ConnectionSuperseded missing');
  assert.equal(spec.dir, 's2c');
  assert.equal(spec.required?.messageType, 'const:ConnectionSuperseded');
  assert.equal(spec.required?.reasonCode, 'const:connection_superseded');
  assert.equal(spec.required?.netEntityId, 'u64');
  assert.equal(spec.required?.newConnectionGeneration, 'u64');
  assert.match(spec.notes ?? '', /再关闭/);
  const valid = (contract.testCases ?? []).find((c) => c.name === 's2c/connection-superseded');
  assert.ok(valid?.message, 'embedded valid ConnectionSuperseded case must exist');
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
});

test('C-1 close-before-send ConnectionSuperseded is a receiver invalidCase', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'runtime/connection-superseded-close-before-send');
  assert.ok(ic, 'missing shipped invalidCase runtime/connection-superseded-close-before-send');
  assert.equal(ic.validatorCheck, false);
  assert.equal(ic.expectedRejection, 'session_closed');
  assert.match(JSON.stringify(ic.payload ?? {}), /close|先关|关闭/);
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
});

test('C-1 mapping dimensions are generated-from-field-annotations and sha256-match N-04', async () => {
  const contract = await loadEnvelopeContract();
  assert.equal(contract.contractId, ENVELOPE_CONTRACT_ID);
  const generated = contract.generatedAttributeDeclarations;
  assert.equal(generated?.source, 'generated-from-field-annotations');
  assert.equal(generated?.sha256, N04_DECLARATIONS_SHA256);
  assert.ok(Array.isArray(generated?.declarations), 'N-04 copy must be embedded as generatedAttributeDeclarations.declarations');
  const digest = attributeDeclarationsSha256(generated.declarations);
  assert.equal(digest, N04_DECLARATIONS_SHA256);
  console.log(`C-1 dimensions sha256=${digest} (N-04 pinned ${N04_DECLARATIONS_SHA256})`);
  for (const [id, mapping] of Object.entries(contract.mappings ?? {})) {
    assert.equal(mapping.dimensions?.source, 'generated-from-field-annotations', `${id}.dimensions.source`);
    assert.equal(mapping.dimensions?.sha256, N04_DECLARATIONS_SHA256, `${id}.dimensions.sha256`);
  }
  const fromEnv = process.env.LUMIO_ATTRIBUTE_DECLARATIONS;
  if (fromEnv) {
    const external = JSON.parse(await readFile(fromEnv, 'utf8'));
    const externalDigest = attributeDeclarationsSha256(external);
    assert.equal(externalDigest, digest);
    assert.deepEqual(external, generated.declarations);
    console.log(`C-1 dimensions sha256 matches LUMIO_ATTRIBUTE_DECLARATIONS (${externalDigest})`);
  }
});

test('C-1 chat.component dimensions match ChatComponent field annotations', async () => {
  const contract = await loadEnvelopeContract();
  const dim = contract.mappings?.['chat.component']?.dimensions;
  assert.ok(dim, 'chat.component.dimensions missing');
  const rows = (contract.generatedAttributeDeclarations?.declarations ?? []).filter((row) =>
    String(row.attributeId).startsWith('ChatComponent.'),
  );
  assert.ok(rows.length >= 2, 'N-04 copy must include ChatComponent rows');
  const persistence = new Set(rows.map((row) => row.persistence));
  const replication = new Set(rows.map((row) => row.replication));
  const visibility = new Set(rows.map((row) => row.visibility));
  assert.deepEqual([...persistence], ['persistent']);
  assert.deepEqual([...replication], ['not-replicated']);
  assert.deepEqual([...visibility], ['server-only']);
  assert.equal(dim.persistence, 'persistent');
  assert.equal(dim.replication, 'not-replicated');
  assert.equal(dim.visibility, 'server-only');
});

test('C-1 chat.event notes require client-observed Delta.changedBlocks and forbid harness synthesis', async () => {
  const contract = await loadEnvelopeContract();
  const notes = contract.mappings?.['chat.event']?.notes ?? '';
  assert.match(notes, /Delta\.changedBlocks/);
  assert.match(notes, /eventOrder/);
  assert.match(notes, /appliedTicks/);
  assert.match(notes, /restoredWindow/);
  assert.match(notes, /不得/);
});

const ENTITY_IDENTITY_TWO_LIVE_PAYLOAD =
  '02000000650000000000000006000000706c617965720100000061660000000000000003000000626f740100000062';
const ENTITY_IDENTITY_TWO_LIVE_SHA256 = '4ae28198083875a42260bcd2c9493077c1726f351eace497c21c51f136d247b1';
const ENTITY_IDENTITY_UNSORTED_PAYLOAD =
  '02000000660000000000000003000000626f740100000062650000000000000006000000706c617965720100000061';
const ENTITY_IDENTITY_UNSORTED_SHA256 = '77ec132763f1b98a81795499e84e99bbd23ecad9c14e89af4e951078030dfabe';
const ENTITY_IDENTITY_ILLEGAL_TYPE_PAYLOAD = '010000006500000000000000030000006e70630100000061';
const ENTITY_IDENTITY_ILLEGAL_TYPE_SHA256 = 'cff67ab1300f6f4487eb136eef0741c0489ee06ebeacd0892a2ac1fbea903da0';

test('C-1 two-live-entity FullSnapshot is a valid entity.identity census', async () => {
  const contract = await loadEnvelopeContract();
  const valid = (contract.testCases ?? []).find((c) => c.name === 'snapshot/two-live-entities');
  assert.ok(valid?.message, 'embedded two-live-entity snapshot case must exist');
  assert.equal(valid.message.stateBlocks.length, 1);
  assert.equal(valid.message.stateBlocks[0].mappingId, 'entity.identity');
  assert.equal(valid.message.stateBlocks[0].payload, ENTITY_IDENTITY_TWO_LIVE_PAYLOAD);
  assert.equal(valid.message.stateBlocks[0].payloadSha256, ENTITY_IDENTITY_TWO_LIVE_SHA256);
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
});

test('C-1 entity.identity is the sole kind=state census mapping and omits claimedMark', async () => {
  const contract = await loadEnvelopeContract();
  const mapping = contract.mappings?.['entity.identity'];
  assert.ok(mapping, 'mappings.entity.identity missing');
  assert.equal(mapping.kind, 'state');
  assert.equal(mapping.direction, 's2c');
  assert.equal(mapping.collection, 'array');
  assert.deepEqual(mapping.fieldOrder, ['netEntityId', 'entityType', 'unmappedMark']);
  assert.equal(mapping.fields?.netEntityId?.type, 'u64');
  assert.equal(mapping.fields?.entityType?.type, 'utf8-string');
  assert.deepEqual(mapping.fields?.entityType?.allowedValues, ['player', 'bot']);
  assert.equal(mapping.fields?.unmappedMark?.type, 'utf8-string');
  assert.equal(mapping.dimensions?.persistence, 'ephemeral');
  assert.equal(mapping.dimensions?.replication, 'replicated');
  assert.equal(mapping.dimensions?.visibility, 'room-public');
  assert.equal(Object.prototype.hasOwnProperty.call(mapping.fields ?? {}, 'claimedMark'), false);
  assert.equal((mapping.fieldOrder ?? []).includes('claimedMark'), false);
  const stateIds = Object.entries(contract.mappings ?? {})
    .filter(([, spec]) => spec.kind === 'state')
    .map(([id]) => id);
  assert.deepEqual(stateIds, ['entity.identity']);
  const example = (contract.hash?.examples ?? []).find((item) => item.mappingId === 'entity.identity');
  assert.ok(example, 'hash.examples must include a two-record entity.identity payload');
  assert.equal(example.payload, ENTITY_IDENTITY_TWO_LIVE_PAYLOAD);
  assert.equal(example.payloadSha256, ENTITY_IDENTITY_TWO_LIVE_SHA256);
});

test('C-1 unregistered Binding mappingId in stateBlocks is state_block_kind_mismatch', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'snapshot/unregistered-binding-mapping-id');
  assert.ok(ic, 'missing shipped invalidCase snapshot/unregistered-binding-mapping-id');
  assert.equal(ic.expectedRejection, 'state_block_kind_mismatch');
  assert.equal(ic.validatorCheck, true);
  const ids = (ic.payload?.stateBlocks ?? []).map((block) => block.mappingId);
  assert.ok(ids.includes('mapping-entity-identity-entity-type'));
  assert.ok(ids.includes('claimed-mark'));
  assert.throws(
    () => admitMessage(contract, ic.payload),
    (error) => error instanceof Rejection && error.code === 'state_block_kind_mismatch',
  );
});

test('C-1 entity.identity unsorted records are block_order_violation', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'snapshot/entity-identity-unsorted-records');
  assert.ok(ic, 'missing shipped invalidCase snapshot/entity-identity-unsorted-records');
  assert.equal(ic.expectedRejection, 'block_order_violation');
  assert.equal(ic.validatorCheck, true);
  assert.equal(ic.payload.stateBlocks[0].payload, ENTITY_IDENTITY_UNSORTED_PAYLOAD);
  assert.equal(ic.payload.stateBlocks[0].payloadSha256, ENTITY_IDENTITY_UNSORTED_SHA256);
  assert.throws(
    () => admitMessage(contract, ic.payload),
    (error) => error instanceof Rejection && error.code === 'block_order_violation',
  );
});

test('C-1 entity.identity illegal entityType is undecodable_payload', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'snapshot/entity-identity-illegal-entity-type');
  assert.ok(ic, 'missing shipped invalidCase snapshot/entity-identity-illegal-entity-type');
  assert.equal(ic.expectedRejection, 'undecodable_payload');
  assert.equal(ic.validatorCheck, true);
  assert.equal(ic.payload.stateBlocks[0].payload, ENTITY_IDENTITY_ILLEGAL_TYPE_PAYLOAD);
  assert.equal(ic.payload.stateBlocks[0].payloadSha256, ENTITY_IDENTITY_ILLEGAL_TYPE_SHA256);
  assert.throws(
    () => admitMessage(contract, ic.payload),
    (error) => error instanceof Rejection && error.code === 'undecodable_payload',
  );
});

test('C-1 chat.event still cannot enter FullSnapshot.stateBlocks', async () => {
  const contract = await loadEnvelopeContract();
  const ic = (contract.invalidCases ?? []).find((c) => c.name === 'snapshot/event-replay');
  assert.ok(ic, 'missing shipped invalidCase snapshot/event-replay');
  assert.equal(ic.expectedRejection, 'state_block_kind_mismatch');
  assert.throws(
    () => admitMessage(contract, ic.payload),
    (error) => error instanceof Rejection && error.code === 'state_block_kind_mismatch',
  );
  const empty = (contract.testCases ?? []).find((c) => c.name === 'snapshot/empty-state-blocks');
  assert.ok(empty?.message, 'empty stateBlocks case must remain valid');
  assert.doesNotThrow(() => admitMessage(contract, empty.message));
});

const TIMER_ERROR_CODES = [
  'stale_handle',
  'scope_invalid',
  'scope_generation_mismatch',
  'invalid_due_tick',
  'invalid_interval',
  'schedule_budget_exceeded',
  'slot_closed',
  'slot_unbound',
  'slot_dispatch_mismatch',
  'slot_queue_full',
  'late_completion',
  'manager_shutdown',
];

function timerStatusName(code) {
  return `Timer${code.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join('')}`;
}

function minimalTimerContract() {
  const errorCodeMapping = Object.fromEntries(TIMER_ERROR_CODES.map((code, index) => [code, 6 + index]));
  const functions = TIMER_ABI_REQUIRED_FUNCTIONS.map((name) => ({
    name,
    type: 'fn(pointer) -> status',
    params: [{ name: 'manager', type: 'pointer' }],
  }));
  return {
    contractId: TIMER_CONTRACT_ID,
    version: 1,
    purpose: 'single-kernel dual-mode timer ABI used by validator negative cases',
    layers: {
      kernel: {
        modes: {
          wallClock: { owns: 'monotonic wall-clock deadlines in milliseconds' },
          tickFrame: { owns: 'deterministic tick and frame gameplay schedules' },
        },
        shared: ['TimerHandle', 'CallbackSlot', 'errorCodes'],
      },
    },
    consumers: { reconnectDeadline: { layer: 'kernel:wallClock' } },
    errorCodes: TIMER_ERROR_CODES,
    sharedTypes: {
      FiringRecord: {
        required: { slotDispatchId: 'u32' },
      },
    },
    abiSurface: {
      functions,
      errorCodeMapping,
      managerHandleAfterDestroy: 'shutdown-tombstone',
      afterDestroyStatus: 'manager_shutdown',
      drainRecord: {
        bytes: 40,
        fields: [{ name: 'slotDispatchId', type: 'u32' }],
      },
    },
  };
}

function stubTimerAbi(contract) {
  const status = {
    Success: 0,
    InvalidArgument: 1,
    UnsupportedVersion: 2,
    ClrInitFailed: 3,
    ClrEntryFailed: 4,
    BufferTooSmall: 5,
  };
  for (const [code, numeric] of Object.entries(contract.abiSurface.errorCodeMapping)) {
    status[timerStatusName(code)] = numeric;
  }
  return {
    root: {
      fields: contract.abiSurface.functions.map((fn) => ({ name: fn.name, type: fn.type })),
    },
    status,
  };
}

async function loadTimerAndAbi() {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'native-timer-abi-v1.json'), 'utf8'));
  const abiDefinition = JSON.parse(await readFile(resolve(root, 'engine/abi/native-abi.json'), 'utf8'));
  return { contract, abiDefinition };
}

test('C-4 layers is single-kernel dual-mode and reconnect window is kernel:wallClock', async () => {
  const { contract, abiDefinition } = await loadTimerAndAbi();
  assert.equal(contract.contractId, 'lumio.native-timer-abi.v1');
  assert.equal(contract.layers?.hostTimerService, undefined);
  assert.equal(contract.layers?.nativeTickFrameTimerManager, undefined);
  assert.ok(contract.layers?.kernel?.modes?.wallClock);
  assert.ok(contract.layers?.kernel?.modes?.tickFrame);
  assert.deepEqual(new Set(contract.layers.kernel.shared), new Set(['TimerHandle', 'CallbackSlot', 'errorCodes']));
  assert.equal(contract.consumers?.reconnectDeadline?.layer, 'kernel:wallClock');
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', abiDefinition);
  assert.deepEqual(problems, []);
});

test('C-4 abiSurface is the hosted timer set with no function-pointer parameters', async () => {
  const { contract, abiDefinition } = await loadTimerAndAbi();
  const names = (contract.abiSurface?.functions ?? []).map((fn) => fn.name);
  for (const required of TIMER_ABI_REQUIRED_FUNCTIONS) {
    assert.ok(names.includes(required), `abiSurface missing ${required}`);
  }
  for (const fn of contract.abiSurface.functions) {
    for (const param of fn.params) {
      assert.equal(typeof param.type, 'string');
      assert.equal(param.type.includes('fn('), false, `${fn.name} param ${param.name} is a function pointer`);
      assert.ok(['pointer', 'u32', 'u64'].includes(param.type), `${fn.name} param ${param.name} type ${param.type}`);
    }
  }
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', abiDefinition);
  assert.deepEqual(problems, []);
});

test('native-abi.json root fields include every C-4 abiSurface timer function', async () => {
  const { contract, abiDefinition } = await loadTimerAndAbi();
  assert.ok(Array.isArray(contract.abiSurface?.functions), 'C-4 abiSurface.functions must exist');
  const fields = new Map(abiDefinition.root.fields.map((field) => [field.name, field]));
  for (const fn of contract.abiSurface.functions) {
    assert.ok(fields.has(fn.name), `native-abi.json missing ${fn.name}`);
    assert.equal(fields.get(fn.name).type, fn.type);
  }
  const alignment = [];
  checkTimerAbiAlignment(contract, abiDefinition, alignment);
  assert.deepEqual(alignment, []);
});

test('validator accepts a dual-mode abiSurface fixture and rejects a second kernel', () => {
  const contract = minimalTimerContract();
  const abiDefinition = stubTimerAbi(contract);
  assert.deepEqual(validateContract(contract, 'native-timer-abi-v1.json', abiDefinition).problems, []);
  const bad = JSON.parse(JSON.stringify(contract));
  bad.layers.hostTimerService = { owns: 'second wall-clock kernel' };
  const { problems } = validateContract(bad, 'native-timer-abi-v1.json', abiDefinition);
  assert.ok(problems.some((item) => item.includes('hostTimerService') && item.includes('second timer infrastructure')));
});

test('validator rejects abiSurface function-pointer parameters', () => {
  const contract = minimalTimerContract();
  const target = contract.abiSurface.functions.find((fn) => fn.name === 'timer_register_dispatch');
  target.params.push({ name: 'callback', type: 'fn(pointer) -> status' });
  target.type = 'fn(pointer, fn(pointer) -> status) -> status';
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', stubTimerAbi(contract));
  assert.ok(problems.some((item) => item.includes('function-pointer')));
});

test('validator rejects reconnectDeadline leaving the wallClock kernel', () => {
  const contract = minimalTimerContract();
  contract.consumers.reconnectDeadline.layer = 'hostTimerService';
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', stubTimerAbi(contract));
  assert.ok(problems.some((item) => item.includes('reconnectDeadline.layer') && item.includes('kernel:wallClock')));
});

test('timer_destroy_manager leaves a shutdown tombstone so manager_shutdown is observable', async () => {
  const { contract, abiDefinition } = await loadTimerAndAbi();
  assert.equal(contract.abiSurface.managerHandleAfterDestroy, 'shutdown-tombstone');
  assert.equal(contract.abiSurface.afterDestroyStatus, 'manager_shutdown');
  const destroy = abiDefinition.root.fields.find((field) => field.name === 'timer_destroy_manager');
  assert.ok(destroy?.doc, 'native-abi.json timer_destroy_manager must have a doc');
  assert.equal(/立即失效/.test(destroy.doc), false, 'destroy must not immediately invalidate like CLR host');
  assert.match(destroy.doc, /TimerManagerShutdown/);
  assert.match(destroy.doc, /tombstone|墓碑/);
  assert.equal(/立即失效/.test(contract.fieldSemantics.managerLifecycle), false);
  const shutdownCase = (contract.invalidCases ?? []).find((item) => item.name === 'manager_shutdown_rejects_all_operations');
  assert.equal(shutdownCase.expectedRejection, 'manager_shutdown');
  assert.match(shutdownCase.when, /pump/);
  assert.match(shutdownCase.when, /drain/);
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', abiDefinition);
  assert.deepEqual(problems, []);
});

test('FiringRecord.slotDispatchId is the same u32 as drainRecord', async () => {
  const { contract, abiDefinition } = await loadTimerAndAbi();
  const firing = contract.sharedTypes.FiringRecord.required.slotDispatchId;
  const drain = contract.abiSurface.drainRecord.fields.find((field) => field.name === 'slotDispatchId');
  assert.equal(firing, 'u32');
  assert.ok(drain, 'drainRecord.slotDispatchId missing');
  assert.equal(drain.type, 'u32');
  assert.equal(firing, drain.type);
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', abiDefinition);
  assert.deepEqual(problems, []);
});

test('validator rejects destroy immediate-invalidate paired with manager_shutdown', () => {
  const contract = minimalTimerContract();
  contract.abiSurface.managerHandleAfterDestroy = 'immediate-invalidate';
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', stubTimerAbi(contract));
  assert.ok(problems.some((item) => item.includes('shutdown-tombstone')));
});

test('validator rejects FiringRecord.slotDispatchId string vs drainRecord u32', () => {
  const contract = minimalTimerContract();
  contract.sharedTypes.FiringRecord.required.slotDispatchId = 'string';
  const { problems } = validateContract(contract, 'native-timer-abi-v1.json', stubTimerAbi(contract));
  assert.ok(problems.some((item) => item.includes('slotDispatchId')));
});

function fiveTuplePayload(extra = {}) {
  return {
    accountId: 'acct-07',
    roomId: 'room-01',
    netEntityId: 'N1',
    entityType: 'player',
    connectionGeneration: 1,
    ...extra,
  };
}

function minimalBindingContract() {
  const table = [
    {
      attributeId: 'EntityIdentity.entityType',
      persistence: 'ephemeral',
      replication: 'replicated',
      valueType: 'enum:entityType',
      visibility: 'room-public',
    },
  ];
  return {
    contractId: BINDING_CONTRACT_ID,
    version: 1,
    purpose: 'binding query contract used by validator negative cases',
    identityModel: {
      netEntityId: '由 Runtime 身份表发号；宿主准入时调用 Runtime 取号；宿主不得自铸。',
    },
    binding: {
      record: {
        required: {
          accountId: 'string',
          roomId: 'string',
          netEntityId: 'string',
          entityType: 'enum:entityType',
          connectionGeneration: 'u64',
        },
        optional: {},
        notes: '五元组即绑定记录全部字段；会话号与宿主内部句柄不得出现在绑定记录。',
      },
    },
    attributeDeclarations: {
      source: 'generated-from-field-annotations',
      sha256: hashDeclarationTable(table),
      table,
      readRules: ['EntityIdentity.accountId 不声明为可查属性；查询返回 undeclared_attribute。'],
    },
    errorCodes: {
      outcomeCodes: ['non_existent', 'stale_generation', 'invisible', 'unauthorized', 'tombstoned'],
      requestErrorCodes: [
        'invalid_attribute_id',
        'undeclared_attribute',
        'cross_room_reference',
        'storage_access_forbidden',
        'binding_not_found',
        'invalid_binding_shape',
        'scope_violation',
      ],
    },
    invalidCases: [
      {
        name: 'host_minted_net_entity_id',
        expectedRejection: 'invalid_binding_shape',
        payload: fiveTuplePayload({ mintedBy: 'host' }),
      },
      {
        name: 'binding_record_carries_session_id',
        expectedRejection: 'invalid_binding_shape',
        payload: fiveTuplePayload({ session_id: 'sess-9' }),
      },
      {
        name: 'query_undeclared_account_id',
        expectedRejection: 'undeclared_attribute',
        payload: {
          callerScope: 'server-authoritative',
          roomId: 'room-01',
          netEntityId: 'N1',
          attributeId: 'EntityIdentity.accountId',
        },
      },
    ],
  };
}

async function loadBindingContract() {
  return JSON.parse(await readFile(resolve(wireDir, 'entity-binding-and-query-v1.json'), 'utf8'));
}

test('C-2 NetEntityId is issued by the Runtime identity table; host minting is invalid_binding_shape', async () => {
  const contract = await loadBindingContract();
  assert.match(contract.identityModel.netEntityId, /Runtime 身份表发号/);
  assert.match(contract.identityModel.netEntityId, /宿主准入时调用 Runtime 取号/);
  assert.match(contract.identityModel.netEntityId, /宿主不得自铸/);
  const minted = (contract.invalidCases ?? []).find((item) => item.name === 'host_minted_net_entity_id');
  assert.ok(minted, 'missing invalidCase host_minted_net_entity_id');
  assert.equal(minted.expectedRejection, 'invalid_binding_shape');
  assert.throws(
    () => admitBindingRecord(minted.payload),
    (error) => error instanceof Rejection && error.code === 'invalid_binding_shape',
  );
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('C-2 binding record is five-tuple only; session_id is invalid_binding_shape', async () => {
  const contract = await loadBindingContract();
  assert.deepEqual(Object.keys(contract.binding.record.required), BINDING_RECORD_FIELDS);
  assert.deepEqual(contract.binding.record.optional, {});
  assert.match(contract.binding.record.notes, /会话号/);
  assert.match(contract.binding.record.notes, /宿主内部句柄/);
  const session = (contract.invalidCases ?? []).find((item) => item.name === 'binding_record_carries_session_id');
  assert.ok(session, 'missing invalidCase binding_record_carries_session_id');
  assert.equal(session.expectedRejection, 'invalid_binding_shape');
  assert.ok(Object.prototype.hasOwnProperty.call(session.payload, 'session_id'));
  assert.throws(
    () => admitBindingRecord(session.payload),
    (error) => error instanceof Rejection && error.code === 'invalid_binding_shape',
  );
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('C-2 attributeDeclarations is generated-from-field-annotations and matches N-04 sha256', async () => {
  const contract = await loadBindingContract();
  const decls = contract.attributeDeclarations;
  assert.equal(decls.source, 'generated-from-field-annotations');
  assert.equal(Object.prototype.hasOwnProperty.call(decls, 'example'), false);
  assert.ok(Array.isArray(decls.table), 'embedded generated table missing');
  const digest = hashDeclarationTable(decls.table);
  assert.equal(decls.sha256, N04_ATTRIBUTE_DECLARATIONS_SHA256);
  assert.equal(digest, N04_ATTRIBUTE_DECLARATIONS_SHA256);
  const ids = decls.table.map((row) => row.attributeId);
  assert.equal(ids.includes('EntityIdentity.accountId'), false);
  assert.ok(ids.includes('ChatComponent.lastMessageText'));
  assert.ok(ids.includes('EntityIdentity.entityType'));
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('C-2 readRules return undeclared_attribute for EntityIdentity.accountId', async () => {
  const contract = await loadBindingContract();
  const rules = contract.attributeDeclarations.readRules ?? [];
  assert.ok(rules.some((rule) => rule.includes('EntityIdentity.accountId') && rule.includes('undeclared_attribute')));
  const account = (contract.invalidCases ?? []).find((item) => item.payload?.attributeId === 'EntityIdentity.accountId');
  assert.ok(account, 'missing invalidCase that queries EntityIdentity.accountId');
  assert.equal(account.expectedRejection, 'undeclared_attribute');
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('validator accepts a generated-declaration binding fixture and rejects a handwritten example', () => {
  const contract = minimalBindingContract();
  assert.deepEqual(validateContract(contract, 'binding-fixture.json').problems, []);
  const bad = JSON.parse(JSON.stringify(contract));
  bad.attributeDeclarations.example = {
    attributeId: 'EntityIdentity.entityType',
    valueType: 'enum:entityType',
    persistence: 'ephemeral',
    replication: 'replicated',
    visibility: 'room-public',
  };
  const { problems } = validateContract(bad, 'binding-fixture.json');
  assert.ok(problems.some((item) => item.includes('example') && item.includes('handwritten')));
});

test('validator rejects binding records that carry session_id', () => {
  assert.throws(
    () => admitBindingRecord(fiveTuplePayload({ session_id: 'sess-9' })),
    (error) => error instanceof Rejection && error.code === 'invalid_binding_shape',
  );
  assert.doesNotThrow(() => admitBindingRecord(fiveTuplePayload()));
});

test('validator rejects host-minted NetEntityId as invalid_binding_shape', () => {
  assert.throws(
    () => admitBindingRecord(fiveTuplePayload({ mintedBy: 'host' })),
    (error) => error instanceof Rejection && error.code === 'invalid_binding_shape',
  );
});

test('validator rejects attributeDeclarations sha256 that does not recompute from table', () => {
  const contract = minimalBindingContract();
  contract.attributeDeclarations.sha256 = '0'.repeat(64);
  const { problems } = validateContract(contract, 'binding-fixture.json');
  assert.ok(problems.some((item) => item.includes('sha256') && item.includes('does not recompute')));
});

test('C-2 admit returns account_already_online on a second well-shaped connection; shape errors stay invalid_binding_shape', async () => {
  const contract = await loadBindingContract();
  assert.ok(contract.outcomes?.account_already_online, 'outcomes.account_already_online missing');
  assert.ok(
    (contract.errorCodes?.admitOutcomeCodes ?? []).includes('account_already_online'),
    'admitOutcomeCodes must list account_already_online',
  );
  assert.equal((contract.errorCodes?.requestErrorCodes ?? []).includes('account_already_online'), false);
  assert.notEqual(contract.outcomes.account_already_online.definition, contract.errorCodes?.semantics?.invalid_binding_shape);
  const already = (contract.testCases ?? []).find((item) => item.name === 'admit_second_connection_account_already_online');
  assert.ok(already, 'missing testCase admit_second_connection_account_already_online');
  assert.equal(already.expect, 'account_already_online');
  assert.match(String(already.then ?? ''), /netEntityId/);
  const shape = (contract.invalidCases ?? []).find((item) => item.name === 'admit_shape_error_is_not_account_already_online');
  assert.ok(shape, 'missing invalidCase admit_shape_error_is_not_account_already_online');
  assert.equal(shape.expectedRejection, 'invalid_binding_shape');
  assert.throws(
    () => admitBindingRecord(shape.payload),
    (error) => error instanceof Rejection && error.code === 'invalid_binding_shape',
  );
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('C-2 roomId is the host routing key; binding record is IdentityComponent plus host session table; NetEntityId is 128-bit 32-hex', async () => {
  const contract = await loadBindingContract();
  assert.match(contract.identityModel.roomId, /宿主路由键/);
  assert.match(contract.identityModel.roomId, /World Manager/);
  assert.match(contract.identityModel.roomId, /GameWorld/);
  assert.match(contract.errorCodes.semantics.cross_room_reference, /宿主路由/);
  assert.match(contract.identityModel.netEntityId, /128/);
  assert.match(contract.identityModel.netEntityId, /32-hex/);
  assert.match(contract.binding.record.notes, /IdentityComponent/);
  assert.match(contract.binding.record.notes, /不持独立绑定表/);
  const adapterNotes = `${contract.attributeDeclarations.notes ?? ''} ${contract.attributeDeclarations.structure?.notes ?? ''}`;
  assert.match(adapterNotes, /薄适配层/);
  assert.match(adapterNotes, /无自有存储/);
  assert.ok(contract.binding.operations.admit, 'binding.operations.admit missing');
  const { problems } = validateContract(contract, 'entity-binding-and-query-v1.json');
  assert.deepEqual(problems, []);
});

test('C-1 roomSequence is the in-world sequence; entity.identity is a creation record', async () => {
  const contract = await loadEnvelopeContract();
  assert.match(contract.fieldSemantics.roomSequence, /世界内/);
  assert.match(contract.fieldSemantics.roomSequence, /NetEntityId/);
  assert.match(contract.mappings['entity.identity'].notes, /创建记录/);
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
});

test('C-1 field.write is an Owner uplink command with positive and unauthorized receiver cases', async () => {
  const contract = await loadEnvelopeContract();
  const mapping = contract.mappings['field.write'];
  assert.ok(mapping, 'mappings.field.write missing');
  assert.equal(mapping.kind, 'command');
  assert.equal(mapping.direction, 'c2s');
  assert.deepEqual(mapping.fieldOrder, ['netEntityId', 'componentId', 'fieldId', 'value']);
  const valid = (contract.testCases ?? []).find((item) => item.name === 'input/field-write-owner-name');
  assert.ok(valid?.message, 'embedded field.write positive case must exist');
  const encoded = encodeMappingPayload(mapping, {
    netEntityId: 101,
    componentId: 'IdentityComponent',
    fieldId: 'name',
    value: 'Alice',
  });
  assert.equal(valid.message.commands[0].payload, encoded.toString('hex'));
  assert.equal(valid.message.commands[0].payloadSha256, sha256Hex(encoded));
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
  for (const name of ['runtime/field-write-other-entity', 'runtime/field-write-server-authority']) {
    const ic = (contract.invalidCases ?? []).find((item) => item.name === name);
    assert.ok(ic, `missing invalidCase ${name}`);
    assert.equal(ic.validatorCheck, false);
    assert.equal(ic.expectedRejection, 'unauthorized');
  }
  assert.ok(contract.errorCodes.includes('unauthorized'));
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
});

test('C-1 chat.event senderNetEntityId is a 16-byte two-u64 LE pair recomputed by the encoder', async () => {
  const contract = await loadEnvelopeContract();
  const mapping = contract.mappings['chat.event'];
  assert.deepEqual(mapping.fieldOrder, [
    'messageId',
    'roomSequence',
    'senderNetEntityIdInstanceId',
    'senderNetEntityIdCounter',
    'text',
    'appliedTick',
  ]);
  assert.equal(mapping.fields.senderNetEntityIdInstanceId.type, 'u64');
  assert.equal(mapping.fields.senderNetEntityIdCounter.type, 'u64');
  assert.equal(Object.prototype.hasOwnProperty.call(mapping.fields, 'senderNetEntityId'), false);
  assert.match(mapping.notes, /32-hex/);
  assert.match(mapping.notes, /ClientRpc/);
  assert.match(contract.mappings['chat.input'].notes, /ServerRpc/);
  const body = {
    messageId: 1,
    roomSequence: 1,
    senderNetEntityIdInstanceId: 0,
    senderNetEntityIdCounter: 101,
    text: 'gg',
    appliedTick: 7,
  };
  const encoded = encodeMappingPayload(mapping, body);
  assert.equal(encoded.length, 8 + 8 + 8 + 8 + 4 + 2 + 8);
  const digest = sha256Hex(encoded);
  const hex = encoded.toString('hex');
  const example = (contract.hash?.examples ?? []).find((item) => item.mappingId === 'chat.event');
  assert.ok(example, 'hash.examples must include chat.event');
  assert.equal(example.payload, hex);
  assert.equal(example.payloadSha256, digest);
  const valid = (contract.testCases ?? []).find((item) => item.name === 'delta/chat-event');
  assert.equal(valid.message.changedBlocks[0].payload, hex);
  assert.equal(valid.message.changedBlocks[0].payloadSha256, digest);
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
  const { problems } = validateContract(contract, 'gameplay-command-envelope-v1.json');
  assert.deepEqual(problems, []);
});
}
}
