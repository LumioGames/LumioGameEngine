// R-00357/C-3 contract self-check (worktree-local, uncommitted).
// Usage: node .wf-selfcheck-r00357.mjs [path-to-json] [--probe]
// --probe runs the same checks against a deliberately broken in-memory copy to prove detection.
import { readFileSync } from 'node:fs';

const path = process.argv[2] ?? 'engine/wire/account-port-v1.json';
const probe = process.argv.includes('--probe');

const raw = readFileSync(path, 'utf8');
const contract = JSON.parse(raw);

const REQUIRED_KEYS = ['contractId','version','purpose','roles','topology','transport','identity',
  'operations','admissionCredential','botToolCredential','takeover','passwordProfile',
  'errorCodes','errorCodeSemantics','limits','process','testCases','invalidCases'];

function checkContract(c, label) {
  const errors = [];
  const fail = (m) => errors.push(m);
  for (const k of REQUIRED_KEYS) if (!(k in c)) fail(`missing top-level key: ${k}`);
  if (c.contractId !== 'lumio.account-port.v1') fail(`contractId mismatch: ${c.contractId}`);
  if (c.version !== 1) fail(`version must be 1, got ${c.version}`);
  if (!Array.isArray(c.roles) || c.roles.length < 4) fail('roles must list >=4 roles');

  const codes = new Set(c.errorCodes ?? []);
  for (const k of Object.keys(c.errorCodeSemantics ?? {})) {
    if (!codes.has(k)) fail(`errorCodeSemantics key not in errorCodes: ${k}`);
  }
  for (const code of codes) {
    if (!c.errorCodeSemantics?.[code]) fail(`errorCodes entry lacks semantics: ${code}`);
  }
  for (const [op, def] of Object.entries(c.operations ?? {})) {
    for (const code of def.failureCodes ?? []) {
      if (!codes.has(code)) fail(`operations.${op}.failureCodes unknown code: ${code}`);
    }
  }
  const tcs = c.testCases ?? [];
  const ics = c.invalidCases ?? [];
  if (!tcs.length) fail('testCases empty');
  if (!ics.length) fail('invalidCases empty');
  for (const t of tcs) {
    for (const k of ['name','operation','given','expect']) if (!t[k]) fail(`testCase missing ${k}: ${t.name}`);
  }
  for (const i of ics) {
    for (const k of ['name','operation','violates','given','expectedRejection']) if (!i[k]) fail(`invalidCase missing ${k}: ${i.name ?? '?'}`);
    if (!codes.has(i.expectedRejection)) fail(`invalidCase ${i.name} expectedRejection not in errorCodes: ${i.expectedRejection}`);
  }
  // acceptance #2: bot-claim rejection and takeover notices each have positive + failure fixtures
  const botPos = tcs.some(t => /bot/i.test(t.name) && /tool/i.test(t.name));
  const botNeg = ics.some(i => i.expectedRejection.startsWith('bot_'));
  const tkPos = tcs.some(t => /takeover/i.test(t.name));
  const tkNeg = ics.some(i => i.expectedRejection === 'takeover_notice_invalid');
  if (!botPos) fail('no positive bot-claim case');
  if (!botNeg) fail('no failure bot-claim case');
  if (!tkPos) fail('no positive takeover-notice case');
  if (!tkNeg) fail('no failure takeover-notice case');
  // P1-5: ordinary login of EXISTING bot account with correct password must be a frozen failure case
  const p15 = ics.find(i => i.name === 'ordinary_login_existing_bot_rejected');
  if (!p15) fail('missing P1-5 case: ordinary_login_existing_bot_rejected');
  // restart stability fixture (R-00344 acceptance hook)
  if (!tcs.some(t => t.name === 'account_restart_stability')) fail('missing account_restart_stability case');
  // review-fix invariants (P1-1/P1-2/P2-1/P2-2/P2-3/P2-4)
  const ac = c.admissionCredential ?? {};
  if (!ac.framing || !/LumioBinV1/.test(ac.framing) || !/rawSignature\(64 字节\)/.test(ac.framing)) {
    fail('admissionCredential.framing missing or not LumioBinV1(payload)||raw-64-byte form');
  }
  const bt = c.botToolCredential?.format ?? {};
  if (!bt.framing || !/LumioBinV1/.test(bt.framing) || !/rawSignature\(64 字节\)/.test(bt.framing)) {
    fail('botToolCredential.format.framing missing or not LumioBinV1(payload)||raw-64-byte form');
  }
  for (const [sigLabel, sig] of [['admissionCredential', ac.signature], ['botToolCredential', bt.signature]]) {
    if (!sig?.preimage || !/<payloadDigest-hex>/.test(sig.preimage)) fail(`${sigLabel}.signature.preimage missing or lacks payloadDigest-hex`);
  }
  if (!/ADR-042 §2/.test(ac.digest ?? '')) fail('admissionCredential.digest must cite ADR-042 §2 (hex-in-preimage)');
  if (/作验签输入；hex 仅供展示/.test(ac.digest ?? '')) fail('admissionCredential.digest still contains the contradictory bare-digest phrasing');
  if (!/ADR-042 §2/.test(bt.signature?.digest ?? '')) fail('botToolCredential.signature.digest must cite ADR-042 §2');
  if (c.operations.login_or_register.request.optional?.clientName) fail('clientName must be removed from request.optional');
  if (!ac.keyManagement?.keyIdNote || !/不采纳 ADR-042 §3/.test(ac.keyManagement.keyIdNote)) fail('keyManagement.keyIdNote must state non-adoption of ADR-042 §3 derivation');
  const grammar = ics.find(i => i.name === 'invalid_username_grammar');
  if (!grammar || !/长度 1/.test(grammar.given) || /长度 2/.test(grammar.given)) {
    fail('invalid_username_grammar given-note arithmetic still wrong (must say length 1)');
  }
  return errors;
}

function reportProbe(tag, errs) {
  console.log(`PROBE ${tag}: ${errs.length} detection(s) expected >0`);
  for (const e of errs) console.log(`  DETECTED: ${e}`);
  return errs.length > 0;
}

if (probe) {
  const brokenCodes = structuredClone(contract);
  brokenCodes.errorCodes = brokenCodes.errorCodes.filter(c => c !== 'bot_tool_credential_expired');
  const codeOk = reportProbe('codes', checkContract(brokenCodes, 'probe-codes'));

  const brokenFraming = structuredClone(contract);
  delete brokenFraming.admissionCredential.framing;
  delete brokenFraming.botToolCredential.format.framing;
  brokenFraming.admissionCredential.digest = 'SHA-256 over canonical payload bytes（原始 32 字节摘要作验签输入；hex 仅供展示）';
  delete brokenFraming.admissionCredential.signature.preimage;
  delete brokenFraming.botToolCredential.format.signature.preimage;
  const framingOk = reportProbe('framing/preimage', checkContract(brokenFraming, 'probe-framing'));

  process.exit(codeOk && framingOk ? 0 : 1);
}

const errors = checkContract(contract, 'real');
console.log(`contract: ${path}`);
console.log(`errorCodes: ${contract.errorCodes.length}, errorCodeSemantics: ${Object.keys(contract.errorCodeSemantics).length}`);
console.log(`testCases: ${contract.testCases.length}, invalidCases: ${contract.invalidCases.length}`);
if (errors.length) {
  console.log(`SELF-CHECK FAILED (${errors.length}):`);
  for (const e of errors) console.log(`  - ${e}`);
  process.exit(1);
}
console.log('SELF-CHECK OK');
