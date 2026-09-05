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

export function admitSequence(contract, messages) {
  if (!Array.isArray(messages) || messages.length === 0) throw new Rejection('bad_envelope', 'packet sequence must be non-empty');
  if (messages[0]?.messageType !== 'Welcome') throw new Rejection('bad_envelope', 'Welcome must precede WorldChange');
  for (const message of messages) admitMessage(contract, message);
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
  if (fileName === 'entity-binding-and-query-v1.json' && (!/accepted/.test(admitResult) || (/netEntityId/.test(admitResult) && !/不返回 netEntityId/.test(admitResult)))) {
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
  if (fileName === 'entity-binding-and-query-v1.json' && (ids.has('EntityIdentity.entityType') || ids.has('EntityIdentity.claimedMark') || ids.has('EntityIdentity.unmappedMark') || ids.has('ChatComponent.lastMessagePersistOnly'))) {
    problem('attributeDeclarations.table must omit EntityIdentity.* and lastMessagePersistOnly; entityType is derived');
  }
  if (fileName === 'entity-binding-and-query-v1.json') {
    if (!contract.derived?.entityType || !String(contract.derived.entityType.source).includes('TypeOf')) problem('derived.entityType must come from World.TypeOf');
    if (!String(contract.derived?.tombstoned ?? '').includes('next-issued-counter')) problem('derived.tombstoned must use counter < next-issued-counter and live-set absence');
    if (!contract.claim?.credential || !String(contract.claim.credential).includes('claimBy')) problem('claim credential must use target entity claimBy named-list field');
  }
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
    const controls = contract.runtimeManagerControls;
    if (!controls || typeof controls !== 'object' || Array.isArray(controls)) {
      problem('runtimeManagerControls missing');
    } else {
      if (controls.transport !== 'in-process') problem('runtimeManagerControls.transport must be in-process');
      if (controls.entryPoint !== 'WorldManager.Enqueue') problem('runtimeManagerControls.entryPoint must be WorldManager.Enqueue');
      const messages = controls.messages;
      const expectedNames = ['admit', 'disconnect', 'rebind'];
      if (!messages || typeof messages !== 'object' || Array.isArray(messages)) {
        problem('runtimeManagerControls.messages missing');
      } else {
        if (JSON.stringify(Object.keys(messages)) !== JSON.stringify(expectedNames)) {
          problem('runtimeManagerControls.messages must be exactly admit, disconnect, rebind');
        }
        const expected = {
          admit: { type: 'AdmitConnectionMessage', required: ['connection', 'accountId', 'roomId', 'entityType'] },
          disconnect: { type: 'DisconnectConnectionMessage', required: ['connection'] },
          rebind: { type: 'RebindConnectionMessage', required: ['connection', 'accountId', 'roomId', 'mode'] },
        };
        for (const name of expectedNames) {
          const spec = messages[name];
          if (!spec || typeof spec !== 'object' || Array.isArray(spec)) {
            problem(`runtimeManagerControls.messages.${name} missing`);
            continue;
          }
          if (spec.type !== expected[name].type) problem(`runtimeManagerControls.messages.${name}.type must be ${expected[name].type}`);
          if (JSON.stringify(spec.required) !== JSON.stringify(expected[name].required)) {
            problem(`runtimeManagerControls.messages.${name}.required must be exactly ${expected[name].required.join(', ')}`);
          }
        }
        if (JSON.stringify(messages.admit?.entityType) !== JSON.stringify(['player', 'bot'])) {
          problem('runtimeManagerControls.messages.admit.entityType must be exactly player, bot');
        }
        if (JSON.stringify(messages.rebind?.mode) !== JSON.stringify(['reconnect', 'takeover'])) {
          problem('runtimeManagerControls.messages.rebind.mode must be exactly reconnect, takeover');
        }
      }
      if (controls.result !== 'accepted-or-error-without-netEntityId' || /netEntityId/i.test(String(controls.result)) && !/without-netEntityId/i.test(String(controls.result))) {
        problem('runtimeManagerControls.result must not permit synchronous netEntityId');
      }
      if (controls.connectionRouting !== 'adapter-callback') problem('runtimeManagerControls.connectionRouting must be adapter-callback');
      if (controls.persistence !== 'none') problem('runtimeManagerControls.persistence must be none');
    }
    const ownerControls = contract.ownerThreadControls;
    if (!ownerControls || typeof ownerControls !== 'object' || Array.isArray(ownerControls)) {
      problem('ownerThreadControls missing');
    } else {
      const expectedRequests = {
        expire: { type: 'ExpireEntityMessage', required: ['requestId', 'netEntityId'], optional: ['connection'] },
        resolve: { type: 'ResolveBindingMessage', required: ['requestId', 'roomId', 'netEntityId'], optional: ['connectionGeneration', 'connection'] },
        attribute: { type: 'AttributeQueryMessage', required: ['requestId', 'callerScope', 'roomId', 'netEntityId', 'attributeId'], optional: ['connectionGeneration', 'connection'] },
      };
      if (ownerControls.transport !== 'in-process') problem('ownerThreadControls.transport must be in-process');
      if (ownerControls.messageBaseType !== 'WorldMessage') problem('ownerThreadControls.messageBaseType must be WorldMessage');
      if (ownerControls.entryPoint !== 'WorldManager.Enqueue') problem('ownerThreadControls.entryPoint must be WorldManager.Enqueue');
      if (ownerControls.execution !== 'owner-thread-during-Tick') problem('ownerThreadControls.execution must be owner-thread-during-Tick');
      const requests = ownerControls.requests;
      if (!requests || typeof requests !== 'object' || Array.isArray(requests)) {
        problem('ownerThreadControls.requests missing');
      } else {
        if (JSON.stringify(Object.keys(requests)) !== JSON.stringify(Object.keys(expectedRequests))) problem('ownerThreadControls.requests must be exactly expire, resolve, attribute');
        for (const [name, expected] of Object.entries(expectedRequests)) {
          const request = requests[name];
          const path = `ownerThreadControls.requests.${name}`;
          if (!request || typeof request !== 'object' || Array.isArray(request)) { problem(`${path} missing`); continue; }
          if (request.type !== expected.type) problem(`${path}.type must be ${expected.type}`);
          if (JSON.stringify(request.required) !== JSON.stringify(expected.required)) problem(`${path}.required must be exactly ${expected.required.join(', ')}`);
          if (JSON.stringify(request.optional) !== JSON.stringify(expected.optional)) problem(`${path}.optional must be exactly ${expected.optional.join(', ')}`);
          const declared = request.fields;
          if (!declared || typeof declared !== 'object' || Array.isArray(declared)) { problem(`${path}.fields missing`); continue; }
          const expectedFields = [...expected.required, ...expected.optional];
          if (JSON.stringify(Object.keys(declared)) !== JSON.stringify(expectedFields)) problem(`${path}.fields must cover required and optional fields`);
          for (const field of Object.keys(declared)) if (!expectedFields.includes(field)) problem(`${path}.fields contains an undeclared field`);
        }
      }
      const expectedResults = {
        expire: {
          type: 'ExpireEntityResult', required: ['requestId', 'outcome'], optional: ['code', 'detail'],
          outcomes: ['accepted', 'tombstoned', 'non_existent', 'request_error'],
          fields: { requestId: 'string', outcome: 'enum:accepted|tombstoned|non_existent|request_error', code: 'enum:requestErrorCodes', detail: 'string' },
          shapes: { accepted: ['requestId', 'outcome'], tombstoned: ['requestId', 'outcome'], non_existent: ['requestId', 'outcome'], request_error: ['requestId', 'outcome', 'code', 'detail'] },
        },
        resolve: {
          type: 'ResolveBindingResult', required: ['requestId', 'outcome'], optional: ['binding', 'observedRevision', 'code', 'detail'],
          outcomes: ['ok', 'non_existent', 'stale_generation', 'invisible', 'unauthorized', 'tombstoned', 'request_error'],
          fields: { requestId: 'string', outcome: 'enum:ok|outcomeCodes|request_error', binding: 'binding.record', observedRevision: 'u64', code: 'enum:requestErrorCodes', detail: 'string' },
          shapes: { ok: ['requestId', 'outcome', 'binding', 'observedRevision'], non_existent: ['requestId', 'outcome'], stale_generation: ['requestId', 'outcome'], invisible: ['requestId', 'outcome'], unauthorized: ['requestId', 'outcome'], tombstoned: ['requestId', 'outcome'], request_error: ['requestId', 'outcome', 'code', 'detail'] },
        },
        attribute: {
          type: 'AttributeQueryResult', required: ['requestId', 'outcome'], optional: ['netEntityId', 'roomId', 'attributeId', 'value', 'observedRevision', 'observedTick', 'code', 'detail'],
          outcomes: ['ok', 'non_existent', 'stale_generation', 'invisible', 'unauthorized', 'tombstoned', 'request_error'],
          fields: { requestId: 'string', outcome: 'enum:ok|outcomeCodes|request_error', netEntityId: 'net-entity-id', roomId: 'string', attributeId: 'attribute-id', value: 'declared-type', observedRevision: 'u64', observedTick: 'u64', code: 'enum:requestErrorCodes', detail: 'string' },
          shapes: { ok: ['requestId', 'outcome', 'netEntityId', 'roomId', 'attributeId', 'value', 'observedRevision', 'observedTick'], non_existent: ['requestId', 'outcome'], stale_generation: ['requestId', 'outcome'], invisible: ['requestId', 'outcome'], unauthorized: ['requestId', 'outcome'], tombstoned: ['requestId', 'outcome'], request_error: ['requestId', 'outcome', 'code', 'detail'] },
        },
      };
      const resultList = (actual, expected, path, name) => {
        if (!Array.isArray(actual) || JSON.stringify(actual) !== JSON.stringify(expected)) { problem(`${path}.${name} must be exactly ${expected.join(', ') || '(none)'}`); return false; }
        return true;
      };
      const records = ownerControls.results?.records;
      if (!ownerControls.results || typeof ownerControls.results !== 'object' || Array.isArray(ownerControls.results)) problem('ownerThreadControls.results missing');
      else {
        if (ownerControls.results.transport !== 'drain.queries') problem('ownerThreadControls.results.transport must be drain.queries');
        if (ownerControls.results.internal !== true) problem('ownerThreadControls.results.internal must be true');
        if (!records || typeof records !== 'object' || Array.isArray(records)) problem('ownerThreadControls.results.records missing');
        else {
          const names = Object.keys(expectedResults);
          if (JSON.stringify(Object.keys(records)) !== JSON.stringify(names)) problem('ownerThreadControls.results.records must be exactly expire, resolve, attribute');
          for (const name of names) {
            const expected = expectedResults[name];
            const record = records[name];
            const path = `ownerThreadControls.results.records.${name}`;
            if (!record || typeof record !== 'object' || Array.isArray(record)) { problem(`${path} missing`); continue; }
            if (record.type !== expected.type) problem(`${path}.type must be ${expected.type}`);
            resultList(record.required, expected.required, path, 'required');
            resultList(record.optional, expected.optional, path, 'optional');
            resultList(record.outcomes, expected.outcomes, path, 'outcomes');
            const expectedFields = [...expected.required, ...expected.optional];
            if (!record.fields || typeof record.fields !== 'object' || Array.isArray(record.fields)) problem(`${path}.fields missing`);
            else {
              if (JSON.stringify(Object.keys(record.fields)) !== JSON.stringify(expectedFields)) problem(`${path}.fields must declare exactly ${expectedFields.join(', ')}`);
              for (const field of Object.keys(record.fields)) if (!expectedFields.includes(field)) problem(`${path}.fields contains an undeclared field`);
              for (const [field, type] of Object.entries(expected.fields)) if (record.fields[field] !== type) problem(`${path}.fields.${field} must be ${type}`);
            }
            const shapes = record.outcomeShapes;
            if (!shapes || typeof shapes !== 'object' || Array.isArray(shapes)) problem(`${path}.outcomeShapes missing`);
            else {
              if (JSON.stringify(Object.keys(shapes)) !== JSON.stringify(expected.outcomes)) problem(`${path}.outcomeShapes must be exactly the declared outcomes`);
              for (const outcome of expected.outcomes) {
                const shape = shapes[outcome];
                const shapePath = `${path}.outcomeShapes.${outcome}`;
                if (!shape || typeof shape !== 'object' || Array.isArray(shape)) { problem(`${shapePath} missing`); continue; }
                resultList(shape.required, expected.shapes[outcome], shapePath, 'required');
                resultList(shape.optional, [], shapePath, 'optional');
                if (Object.keys(shape).some((key) => !['required', 'optional'].includes(key))) problem(`${shapePath} contains an undeclared field`);
                const shapeFields = [...(Array.isArray(shape.required) ? shape.required : []), ...(Array.isArray(shape.optional) ? shape.optional : [])];
                if (new Set(shapeFields).size !== shapeFields.length) problem(`${shapePath} required/optional must not contain duplicates`);
                if (shapeFields.some((field) => !expectedFields.includes(field))) problem(`${shapePath} contains an undeclared result field`);
              }
            }
          }
        }
      }
      if (JSON.stringify(ownerControls.hostEntryOperations) !== JSON.stringify(['boot', 'enqueue', 'tick', 'drain', 'snapshot', 'restore'])) problem('ownerThreadControls.hostEntryOperations must be exactly the six HostEntry operations');
      if (JSON.stringify(ownerControls.c1MessageTypes) !== JSON.stringify(['Welcome', 'WorldChange', 'InputCommand', 'ConnectionSuperseded', 'Error'])) problem('ownerThreadControls.c1MessageTypes must preserve the frozen C-1 message set');
    }
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

  // R5-01 C-1 is the World Manager packet shape.
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
    for (const required of ['welcome/128-bit-self', 'world-change/creation-field-rpc', 'world-change/destroy', 'world-change/field-sync', 'world-change/field-correction', 'world-change/owner-visible-to-bound-observer', 'input/chat']) if (!names.has(required)) problem(`testCases missing ${required}`);
    const invalidNames = new Set((contract.invalidCases ?? []).map((item) => item.name));
    for (const required of ['sequence/world-change-before-welcome', 'world-change/owner-leaked-to-non-owner']) if (!invalidNames.has(required)) problem(`invalidCases missing ${required}`);
    return;
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

/*
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

test('R5-01 C-1 covers destroy, field reasons, Owner projection, and packet ordering', async () => {
  const contract = await loadEnvelopeContract();
  const byName = new Map(contract.testCases.map((item) => [item.name, item]));
  for (const name of ['world-change/destroy', 'world-change/field-sync', 'world-change/field-correction', 'world-change/owner-visible-to-bound-observer']) {
    assert.doesNotThrow(() => admitMessage(contract, byName.get(name).message), name);
  }
  assert.equal(byName.get('world-change/field-sync').message.fields[0].reason, 'sync');
  assert.equal(byName.get('world-change/field-correction').message.fields[0].reason, 'correction');
  assert.equal(byName.get('world-change/owner-visible-to-bound-observer').given.observerNetEntityId, byName.get('world-change/owner-visible-to-bound-observer').message.fields[0].netEntityId);
  const reversed = contract.invalidCases.find((item) => item.name === 'sequence/world-change-before-welcome');
  assert.throws(() => admitSequence(contract, reversed.messages), (error) => error instanceof Rejection && error.code === 'bad_envelope');
  const ownerLeak = contract.invalidCases.find((item) => item.name === 'world-change/owner-leaked-to-non-owner');
  assert.equal(ownerLeak.expectedRejection, 'unauthorized');
  assert.notEqual(ownerLeak.given.observerNetEntityId, ownerLeak.message.fields[0].netEntityId);
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

test('R5-01 C-2 declares the closed in-process Runtime Manager controls table', async () => {
  const contract = await loadBindingContract();
  const controls = contract.runtimeManagerControls;
  assert.ok(controls, 'runtimeManagerControls missing');
  assert.equal(controls.transport, 'in-process');
  assert.equal(controls.entryPoint, 'WorldManager.Enqueue');
  assert.deepEqual(Object.keys(controls.messages), ['admit', 'disconnect', 'rebind']);
  assert.deepEqual(controls.messages.admit.required, ['connection', 'accountId', 'roomId', 'entityType']);
  assert.deepEqual(controls.messages.disconnect.required, ['connection']);
  assert.deepEqual(controls.messages.rebind.required, ['connection', 'accountId', 'roomId', 'mode']);
  assert.deepEqual(controls.messages.admit.entityType, ['player', 'bot']);
  assert.deepEqual(controls.messages.rebind.mode, ['reconnect', 'takeover']);
  assert.equal(controls.result, 'accepted-or-error-without-netEntityId');
  assert.equal(controls.connectionRouting, 'adapter-callback');
  assert.equal(controls.persistence, 'none');
  assert.deepEqual(validateContract(contract, 'entity-binding-and-query-v1.json').problems, []);
});

test('R5-01 C-2 rejects malformed Runtime Manager controls declarations', async () => {
  const contract = await loadBindingContract();
  assert.ok(contract.runtimeManagerControls, 'runtimeManagerControls missing');
  const mutations = [
    ['unknown control key', (value) => { value.messages.extra = { type: 'Unexpected', required: [] }; }, 'messages must be exactly admit, disconnect, rebind'],
    ['missing required field', (value) => { value.messages.admit.required = ['connection']; }, 'messages.admit.required must be exactly'],
    ['unsupported rebind mode', (value) => { value.messages.rebind.mode = ['reconnect']; }, 'messages.rebind.mode must be exactly reconnect, takeover'],
    ['wrong transport', (value) => { value.transport = 'websocket'; }, 'transport must be in-process'],
    ['synchronous entity id result', (value) => { value.result = 'accepted-with-netEntityId'; }, 'result must not permit synchronous netEntityId'],
  ];
  for (const [label, mutate, expected] of mutations) {
    const mutated = JSON.parse(JSON.stringify(contract));
    mutate(mutated.runtimeManagerControls);
    const { problems } = validateContract(mutated, 'entity-binding-and-query-v1.json');
    assert.ok(problems.some((problem) => problem.includes(expected)), `${label} was accepted: ${problems.join('; ')}`);
  }
});

test('hello-wire-v1 still passes the unified validator and is not the envelope', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'hello-wire-v1.json'), 'utf8'));
  assert.equal(contract.contractId, 'lumio.hello-wire.v1');
  assert.ok(!contract.mappings);
  assert.deepEqual(validateContract(contract, 'hello-wire-v1.json').problems, []);
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
      attributeId: 'ChatComponent.lastMessageText',
      persistence: 'persistent',
      replication: 'not-replicated',
      valueType: 'utf8-string',
      visibility: 'server-only',
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
  assert.equal(ids.some((id) => id.startsWith('EntityIdentity.')), false);
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
    attributeId: 'ChatComponent.lastMessageText',
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

test('A2 owner-thread controls declare typed requests, internal drain queries, and frozen bridge boundaries', async () => {
  const contract = await loadBindingContract();
  const controls = contract.ownerThreadControls;
  assert.equal(controls.transport, 'in-process');
  assert.equal(controls.messageBaseType, 'WorldMessage');
  assert.equal(controls.entryPoint, 'WorldManager.Enqueue');
  assert.equal(controls.execution, 'owner-thread-during-Tick');
  assert.deepEqual(Object.keys(controls.requests), ['expire', 'resolve', 'attribute']);
  assert.deepEqual(controls.requests.expire, {
    type: 'ExpireEntityMessage',
    required: ['requestId', 'netEntityId'],
    optional: ['connection'],
    fields: { requestId: 'string', netEntityId: 'net-entity-id', connection: 'opaque-connection-ref' },
  });
  assert.deepEqual(controls.requests.resolve.required, ['requestId', 'roomId', 'netEntityId']);
  assert.deepEqual(controls.requests.attribute.required, ['requestId', 'callerScope', 'roomId', 'netEntityId', 'attributeId']);
  assert.equal(controls.results.transport, 'drain.queries');
  assert.equal(controls.results.internal, true);
  assert.deepEqual(controls.results.records.expire.outcomeShapes.request_error.required, ['requestId', 'outcome', 'code', 'detail']);
  assert.deepEqual(controls.results.records.resolve.outcomeShapes.ok.required, ['requestId', 'outcome', 'binding', 'observedRevision']);
  assert.deepEqual(controls.results.records.attribute.outcomeShapes.ok.required, ['requestId', 'outcome', 'netEntityId', 'roomId', 'attributeId', 'value', 'observedRevision', 'observedTick']);
  assert.deepEqual(controls.hostEntryOperations, ['boot', 'enqueue', 'tick', 'drain', 'snapshot', 'restore']);
  assert.deepEqual(controls.c1MessageTypes, ['Welcome', 'WorldChange', 'InputCommand', 'ConnectionSuperseded', 'Error']);
  assert.deepEqual(validateContract(contract, 'entity-binding-and-query-v1.json').problems, []);
});

test('A2 validator rejects undeclared result fields, wrong types, and open bridge expansions', async () => {
  const contract = await loadBindingContract();
  const mutations = [
    ['undeclared result field', (value) => { value.results.records.expire.fields.extra = 'string'; }, 'fields must declare exactly'],
    ['wrong result field type', (value) => { value.results.records.resolve.fields.binding = 'string'; }, 'fields.binding must be binding.record'],
    ['wrong result outcome shape', (value) => { value.results.records.attribute.outcomeShapes.ok.required.pop(); }, 'outcomeShapes.ok.required must be exactly'],
    ['seventh host operation', (value) => { value.hostEntryOperations.push('expire'); }, 'must be exactly the six HostEntry operations'],
    ['new C-1 frame', (value) => { value.c1MessageTypes.push('QueryResult'); }, 'must preserve the frozen C-1 message set'],
  ];
  for (const [label, mutate, expected] of mutations) {
    const mutated = JSON.parse(JSON.stringify(contract));
    mutate(mutated.ownerThreadControls);
    const { problems } = validateContract(mutated, 'entity-binding-and-query-v1.json');
    assert.ok(problems.some((problem) => problem.includes(expected)), `${label} was accepted: ${problems.join('; ')}`);
  }
});
