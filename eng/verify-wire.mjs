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

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const wireDir = resolve(root, 'engine/wire');

const U64_MAX = Number.MAX_SAFE_INTEGER; // 2^53-1 on the JSON wire

class Rejection extends Error {
  constructor(code, reason) {
    super(reason);
    this.code = code;
  }
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
    throw new Rejection('state_block_kind_mismatch', `${context.path}: mappingId ${block.mappingId} has kind=${mapping.kind}, allowed here: ${[...context.allowedKinds].join('|')}`);
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
  } else if (t === 'Delta') {
    if (!Array.isArray(message.changedBlocks)) throw new Rejection('bad_envelope', 'changedBlocks: required array missing');
    checkBlockArray(message.changedBlocks, contract, { allowedKinds: new Set(['event', 'state']), unknownCode: 'state_block_kind_mismatch', path: 'changedBlocks' });
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
  if (contract.errorCodes !== undefined) {
    if (!Array.isArray(contract.errorCodes) || contract.errorCodes.some((c) => typeof c !== 'string')) {
      problem('errorCodes must be an array of strings');
    } else if (new Set(contract.errorCodes).size !== contract.errorCodes.length) {
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
      if (!contract.errorCodes?.includes(r.onViolation)) problem(`rules.${r.id}.onViolation ${r.onViolation} not in errorCodes`);
    }
  }
  const allCases = [...(contract.testCases ?? []), ...(contract.invalidCases ?? [])];
  for (const c of contract.invalidCases ?? []) {
    if (!contract.errorCodes?.includes(c.expectedRejection)) problem(`invalidCases.${c.name}: expectedRejection ${c.expectedRejection} not in errorCodes`);
    if (contract.rules && !contract.rules.some((r) => r.id === c.violates)) problem(`invalidCases.${c.name}: violates "${c.violates}" names no registered rule`);
  }
  return allCases.length;
}

// ---------- Contract validation ----------

function validateContract(contract, fileName) {
  const problems = [];
  const caseCount = checkStructure(contract, fileName, problems);

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

  if (contract.testCases) {
    for (const testCase of contract.testCases) {
      const rejection = runCase(`testCases.${testCase.name}`, () => {
        checkMessageShape(testCase.message, contract);
        checkMessageSemantics(testCase.message, contract);
      });
      if (rejection) problems.push(`${fileName} testCases.${testCase.name}: expected valid, rejected [${rejection.code}] ${rejection.message}`);
    }
  }
  if (contract.invalidCases) {
    for (const invalidCase of contract.invalidCases) {
      if (invalidCase.validatorCheck === false) {
        runCase(`invalidCases.${invalidCase.name} (declaration)`, () => {
          if (!invalidCase.payload) throw new Rejection('bad_envelope', 'receiver-side case must still carry a scenario payload');
        });
        continue;
      }
      const rejection = runCase(`invalidCases.${invalidCase.name}`, () => {
        checkMessageShape(invalidCase.payload, contract);
        checkMessageSemantics(invalidCase.payload, contract);
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

// ---------- Main ----------

const files = (await readdir(wireDir)).filter((f) => f.endsWith('.json')).sort();
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
  const { summary, problems } = validateContract(contract, fileName);
  console.log(summary);
  for (const problem of problems) console.log(`  - ${problem}`);
  if (problems.length > 0) failed = true;
}
if (failed) {
  console.error('verify-wire: FAILED');
  process.exit(1);
}
console.log('verify-wire: all contracts green');
