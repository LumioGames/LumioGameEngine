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
//        to identical bytes;
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

function decodeMappingPayload(mapping, bytes) {
  const reader = new BinReader(bytes);
  const body = {};
  for (const fieldName of mapping.fieldOrder) {
    const field = mapping.fields[fieldName];
    if (field.type === 'u64') body[fieldName] = reader.u64();
    else if (field.type === 'utf8-string') body[fieldName] = reader.string();
    else throw new Rejection('undecodable_payload', `mapping field ${fieldName} has unsupported wire type ${field.type}`);
  }
  reader.done();
  // Canonical re-encode must reproduce the exact bytes (two conforming encoders
  // may not disagree — ADR-049 §2 discipline).
  let re = Buffer.alloc(0);
  for (const fieldName of mapping.fieldOrder) {
    re = Buffer.concat([re, encodeField(mapping.fields[fieldName].type, body[fieldName])]);
  }
  if (!re.equals(bytes)) throw new Rejection('undecodable_payload', 'decode/re-encode mismatch: payload is not canonical LumioBinV1');
  // Per-field constraints.
  for (const fieldName of mapping.fieldOrder) {
    const field = mapping.fields[fieldName];
    if (field.type === 'utf8-string' && typeof field.maxUtf8Bytes === 'number') {
      if (Buffer.byteLength(body[fieldName], 'utf8') > field.maxUtf8Bytes) {
        throw new Rejection(field.violationCode ?? 'bad_envelope', `${fieldName} exceeds maxUtf8Bytes=${field.maxUtf8Bytes}`);
      }
    }
  }
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
    if (set === null) return `unknown enum ref ${ref}`;
    return set.includes(value) ? null : `${JSON.stringify(value)} not in ${ref}`;
  }
  switch (expr) {
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
  if (expr.startsWith('const:') || expr.startsWith('enum:') || ['u64', 'epoch-ms', 'string', 'bool', 'hex', 'sha256-hex'].includes(expr)) {
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
const TIMER_ABI_PARAM_TYPES = new Set(['pointer', 'u32', 'u64', 'i32']);
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
}

// ---------- Contract validation ----------

function validateContract(contract, fileName, abiDefinition) {
  const problems = [];
  const caseCount = checkStructure(contract, fileName, problems);
  checkNativeTimerContract(contract, fileName, problems);
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
  TIMER_ABI_REQUIRED_FUNCTIONS,
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

test('gameplay envelope accepts a valid ChatInput InputCommand via shipped admitMessage', async () => {
  const contract = JSON.parse(await readFile(resolve(wireDir, 'gameplay-command-envelope-v1.json'), 'utf8'));
  const valid = (contract.testCases ?? []).find((c) => c.name === 'input/chat-single-command');
  assert.ok(valid?.message, 'embedded valid ChatInput case must exist');
  assert.doesNotThrow(() => admitMessage(contract, valid.message));
});

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
    abiSurface: { functions, errorCodeMapping },
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
      assert.ok(['pointer', 'u32', 'u64', 'i32'].includes(param.type), `${fn.name} param ${param.name} type ${param.type}`);
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
