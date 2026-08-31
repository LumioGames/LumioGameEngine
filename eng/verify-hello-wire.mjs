// Hello World wire contract validator (MS-00002).
// Single consumer entry: node eng/verify-hello-wire.mjs  -> validates engine/wire/hello-wire-v1.json.
// Reusable API: loadContract(root?) / validateMessage(contract, messageType, value) for downstream tooling
// (integration launcher, cross-repo consistency tests).
// Self-checks: node --test eng/verify-hello-wire.mjs

import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

export const CONTRACT_RELATIVE_PATH = 'engine/wire/hello-wire-v1.json';

const TYPE_PATTERN = /^(const:[A-Za-z0-9_.-]+|enum:([A-Za-z0-9_]+)|string|u64|bool|sha256-hex|epoch-ms|array:[A-Za-z0-9_]+)$/;

export async function loadContract(root = resolve(new URL('..', import.meta.url).pathname.replace(/^\/(.:)/, '$1'))) {
  const path = resolve(root, CONTRACT_RELATIVE_PATH);
  const bytes = await readFile(path);
  const contract = JSON.parse(bytes);
  const issues = validateContractShape(contract);
  if (issues.length > 0) {
    throw new Error(`hello-wire contract shape invalid:\n${issues.map((i) => `  - ${i}`).join('\n')}`);
  }
  return contract;
}

export function validateContractShape(contract) {
  const issues = [];
  const push = (issue) => issues.push(issue);
  if (contract.contractId !== 'lumio.hello-wire.v1') push('contractId must be lumio.hello-wire.v1');
  if (!Number.isInteger(contract.version) || contract.version !== 1) push('version must be 1');
  if (contract.transport?.encoding !== 'utf8-json-text-frame') push('transport.encoding must be utf8-json-text-frame');
  if (!Array.isArray(contract.roles) || contract.roles.length === 0) push('roles must be a non-empty array');
  if (!Array.isArray(contract.errorCodes) || contract.errorCodes.length === 0) push('errorCodes must be a non-empty array');
  if (!contract.messages || typeof contract.messages !== 'object') {
    push('messages missing');
    return issues;
  }
  const enums = { roles: contract.roles ?? [], errorCodes: contract.errorCodes ?? [] };
  const sharedTypes = contract.sharedTypes ?? {};
  const checkFields = (container, label) => {
    for (const [name, spec] of Object.entries(container ?? {})) {
      if (!TYPE_PATTERN.test(spec)) push(`${label}.${name} has invalid type spec: ${spec}`);
      if (spec.startsWith('enum:')) {
        const enumName = spec.slice(5);
        if (!enums[enumName]) push(`${label}.${name} references unknown enum ${enumName}`);
      }
      if (spec.startsWith('const:')) {
        const value = spec.slice(6);
        if (value.length === 0) push(`${label}.${name} const value must be non-empty`);
      }
      if (spec.startsWith('array:')) {
        const itemName = spec.slice(6);
        if (!sharedTypes[itemName]) push(`${label}.${name} references unknown shared type ${itemName}`);
      }
    }
  };
  for (const [messageName, message] of Object.entries(contract.messages)) {
    if (message.dir !== 'c2s' && message.dir !== 's2c') push(`messages.${messageName}.dir must be c2s or s2c`);
    const declared = message.required?.messageType;
    if (declared !== `const:${messageName}`) push(`messages.${messageName}.required.messageType must be const:${messageName}`);
    checkFields(message.required, `messages.${messageName}.required`);
    checkFields(message.optional, `messages.${messageName}.optional`);
  }
  for (const [typeName, type] of Object.entries(sharedTypes)) {
    checkFields(type.required, `sharedTypes.${typeName}.required`);
    checkFields(type.optional, `sharedTypes.${typeName}.optional`);
  }
  const example = contract.hash?.example;
  if (example) {
    const computed = createHash('sha256').update(example.payload, 'utf8').digest('hex');
    if (computed !== example.payloadSha256) push(`hash.example.payloadSha256 mismatch: expected ${computed}`);
  }
  const auditKinds = contract.process?.auditEventKinds ?? {};
  for (const [kind, spec] of Object.entries(auditKinds)) {
    if (!Array.isArray(spec.required) || spec.required.length === 0) push(`process.auditEventKinds.${kind}.required must be a non-empty array`);
  }
  const botKinds = contract.process?.botTraceEventKinds ?? {};
  for (const [kind, spec] of Object.entries(botKinds)) {
    if (!Array.isArray(spec.required) || spec.required.length === 0) push(`process.botTraceEventKinds.${kind}.required must be a non-empty array`);
  }
  if (!Number.isInteger(contract.limits?.maxSessions) || contract.limits.maxSessions < 2) push('limits.maxSessions must be >= 2');
  if (!Number.isInteger(contract.limits?.maxPayloadBytes) || contract.limits.maxPayloadBytes < 1) push('limits.maxPayloadBytes must be >= 1');
  return issues;
}

function fieldIssues(contract, spec, value, label, issues) {
  const enums = { roles: contract.roles ?? [], errorCodes: contract.errorCodes ?? [] };
  const sharedTypes = contract.sharedTypes ?? {};
  if (value === undefined || value === null) {
    issues.push(`${label} is required`);
    return;
  }
  if (spec === 'string' && typeof value !== 'string') issues.push(`${label} must be a string`);
  if (spec === 'u64') {
    if (!Number.isInteger(value) || value < 0) issues.push(`${label} must be a non-negative integer`);
  }
  if (spec === 'bool' && typeof value !== 'boolean') issues.push(`${label} must be a boolean`);
  if (spec === 'sha256-hex' && !/^[0-9a-f]{64}$/.test(String(value))) issues.push(`${label} must be lowercase sha256 hex`);
  if (spec === 'epoch-ms') {
    if (!Number.isInteger(value) || value < 0) issues.push(`${label} must be a non-negative integer epoch-ms`);
  }
  if (spec.startsWith('const:')) {
    if (value !== spec.slice(6)) issues.push(`${label} must equal ${spec.slice(6)}`);
  }
  if (spec.startsWith('enum:')) {
    const options = enums[spec.slice(5)] ?? [];
    if (!options.includes(value)) issues.push(`${label} must be one of ${options.join('|')}`);
  }
  if (spec.startsWith('array:')) {
    const itemName = spec.slice(6);
    const itemSpec = sharedTypes[itemName];
    if (!Array.isArray(value)) {
      issues.push(`${label} must be an array of ${itemName}`);
    } else {
      value.forEach((item, index) => {
        for (const [field, fieldSpec] of Object.entries(itemSpec?.required ?? {})) {
          fieldIssues(contract, fieldSpec, item?.[field], `${label}[${index}].${field}`, issues);
        }
      });
    }
  }
}

// Validates a decoded wire message against the contract. Returns {valid, issues[]}.
export function validateMessage(contract, message) {
  const issues = [];
  if (message === null || typeof message !== 'object' || Array.isArray(message)) {
    return { valid: false, issues: ['message must be a JSON object'] };
  }
  const messageType = message.messageType;
  if (typeof messageType !== 'string') {
    return { valid: false, issues: ['messageType is required'] };
  }
  const spec = contract.messages[messageType];
  if (!spec) {
    return { valid: false, issues: [`unknown messageType: ${messageType}`], code: 'unknown_mapping' };
  }
  for (const [name, typeSpec] of Object.entries(spec.required ?? {})) {
    fieldIssues(contract, typeSpec, message[name], `${messageType}.${name}`, issues);
  }
  return { valid: issues.length === 0, issues, dir: spec.dir };
}

export function computePayloadSha256(payload) {
  return createHash('sha256').update(payload, 'utf8').digest('hex');
}

async function main() {
  const contract = await loadContract();
  console.log(`CONTRACT_ID=${contract.contractId}`);
  console.log(`MESSAGES=${Object.keys(contract.messages).length}`);
  console.log(`ERROR_CODES=${contract.errorCodes.length}`);
  console.log('HELLO_WIRE_OK');
}

const isDirectRun = process.argv[1] && resolve(process.argv[1]) === resolve(new URL(import.meta.url).pathname.replace(/^\/(.:)/, '$1'));
if (isDirectRun) {
  main().catch((error) => {
    console.error(String(error));
    process.exit(1);
  });
}

// ---------------------------------------------------------------------------
// Self-checks (node --test eng/verify-hello-wire.mjs)
// ---------------------------------------------------------------------------
import { test } from 'node:test';
import assert from 'node:assert/strict';

test('contract file loads and validates', async () => {
  const contract = await loadContract();
  assert.equal(contract.contractId, 'lumio.hello-wire.v1');
  for (const messageName of ['Handshake', 'HandshakeAck', 'FullSnapshot', 'BaselineAck', 'InputCommand', 'Delta', 'Error', 'Shutdown']) {
    assert.ok(contract.messages[messageName], `missing message ${messageName}`);
  }
});

test('valid InputCommand passes validation', async () => {
  const contract = await loadContract();
  const result = validateMessage(contract, {
    messageType: 'InputCommand',
    sender: 'browser',
    sequence: 1,
    kind: 'hello',
    payload: 'Hello World',
    payloadSha256: computePayloadSha256('Hello World'),
    sentAtMs: Date.now(),
  });
  assert.deepEqual(result.issues, []);
  assert.equal(result.valid, true);
});

test('InputCommand missing payloadSha256 is rejected', async () => {
  const contract = await loadContract();
  const result = validateMessage(contract, {
    messageType: 'InputCommand',
    sender: 'browser',
    sequence: 1,
    kind: 'hello',
    payload: 'Hello World',
    sentAtMs: Date.now(),
  });
  assert.equal(result.valid, false);
  assert.ok(result.issues.some((i) => i.includes('payloadSha256')));
});

test('unknown messageType maps to unknown_mapping', async () => {
  const contract = await loadContract();
  const result = validateMessage(contract, { messageType: 'Mystery' });
  assert.equal(result.valid, false);
  assert.equal(result.code, 'unknown_mapping');
});

test('unknown role is rejected', async () => {
  const contract = await loadContract();
  const result = validateMessage(contract, {
    messageType: 'InputCommand',
    sender: 'attacker',
    sequence: 1,
    kind: 'hello',
    payload: 'Hello World',
    payloadSha256: computePayloadSha256('Hello World'),
    sentAtMs: Date.now(),
  });
  assert.equal(result.valid, false);
  assert.ok(result.issues.some((i) => i.includes('sender')));
});

test('Delta with structurally valid hex but mismatched payload hash stays shape-valid (hash equality is a consumer check)', async () => {
  const contract = await loadContract();
  const wrongHash = 'f'.repeat(64);
  const result = validateMessage(contract, {
    messageType: 'Delta',
    tickId: 1,
    revision: 1,
    sender: 'bot',
    sequence: 1,
    kind: 'hello',
    payload: 'Hello World',
    payloadSha256: wrongHash,
    originSentAtMs: 1,
    committedAtMs: 2,
    commandSequence: 1,
  });
  assert.equal(result.valid, true);
  assert.notEqual(computePayloadSha256('Hello World'), wrongHash);
});

test('contract shape rejects mutated const field (red probe)', async () => {
  const contract = await loadContract();
  const mutated = structuredClone(contract);
  mutated.messages.InputCommand.required.messageType = 'const:WrongType';
  const issues = validateContractShape(mutated);
  assert.ok(issues.some((i) => i.includes('InputCommand.required.messageType')));
});

test('contract shape rejects broken hash example (red probe)', async () => {
  const contract = await loadContract();
  const mutated = structuredClone(contract);
  mutated.hash.example.payloadSha256 = '0'.repeat(64);
  const issues = validateContractShape(mutated);
  assert.ok(issues.some((i) => i.includes('hash.example.payloadSha256')));
});

test('contract shape rejects unknown shared type reference (red probe)', async () => {
  const contract = await loadContract();
  const mutated = structuredClone(contract);
  mutated.messages.FullSnapshot.required.helloLog = 'array:MissingRecord';
  const issues = validateContractShape(mutated);
  assert.ok(issues.some((i) => i.includes('MissingRecord')));
});
