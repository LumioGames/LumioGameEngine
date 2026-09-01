import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import os from 'node:os';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const manifestPath = path.join(root, 'manifest.json');
const eventsPath = path.join(root, 'events.ndjson');
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const cardDir = path.join(root, 'cards');

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function parseConfig() {
  const configPath = path.join(os.homedir(), '.config', 'workflow', 'config.toml');
  const text = fs.readFileSync(configPath, 'utf8');
  const base = text.match(/base_url\s*=\s*"([^"]+)"/)?.[1];
  const token = text.match(/token\s*=\s*"([^"]+)"/)?.[1];
  if (!base || !token) throw new Error('workflow profile is incomplete');
  return { base: `${base}/api/v1`, host: base, token };
}

const { base, host, token } = parseConfig();
const authHeaders = { authorization: `Bearer ${token}` };

async function request(method, route, body, idempotencyKey) {
  let lastError;
  for (let attempt = 0; attempt < 6; attempt += 1) {
    const headers = { ...authHeaders };
    if (idempotencyKey) headers['Idempotency-Key'] = idempotencyKey;
    const options = { method, headers };
    if (body !== undefined) {
      headers['content-type'] = 'application/json';
      options.body = JSON.stringify(body);
    }
    try {
      const response = await fetch(`${base}${route}`, options);
      const text = await response.text();
      let data = null;
      try { data = text ? JSON.parse(text) : null; } catch { data = { raw: text }; }
      if (response.ok) return data;
      const retryable = response.status === 429 || response.status >= 500;
      if (!retryable || attempt === 5) {
        const trace = data?.traceId ? ` traceId=${data.traceId}` : '';
        throw new Error(`${method} ${route} -> ${response.status}${trace}: ${JSON.stringify(data)}`);
      }
      const retryAfter = Number(response.headers.get('retry-after'));
      await new Promise((resolve) => setTimeout(resolve, Number.isFinite(retryAfter) && retryAfter > 0 ? retryAfter * 1000 : 500 * (attempt + 1)));
    } catch (error) {
      lastError = error;
      if (String(error.message).startsWith(`${method} ${route} ->`)) throw error;
      if (attempt === 2) throw error;
      await new Promise((resolve) => setTimeout(resolve, 500 * (attempt + 1)));
    }
  }
  throw lastError ?? new Error('request failed');
}

let checkpointWrite = Promise.resolve();
function persist() {
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}
function record(op, status, extra = {}) {
  checkpointWrite = checkpointWrite.then(() => {
    op.status = status;
    Object.assign(op, extra);
    if (status === 'verified' && !manifest.checkpoint.completedOpIds.includes(op.opId)) {
      manifest.checkpoint.completedOpIds.push(op.opId);
    }
    manifest.checkpoint.phase = status === 'failed' ? 'partial' : 'uploading';
    manifest.checkpoint.lastError = status === 'failed' ? extra.error ?? null : null;
    persist();
    fs.appendFileSync(eventsPath, `${JSON.stringify({ event: `operation.${status}`, opId: op.opId, ...extra })}\n`);
  });
  return checkpointWrite;
}

function parseCard(localId) {
  const text = fs.readFileSync(path.join(cardDir, `${localId}.md`), 'utf8');
  const field = (name) => text.match(new RegExp(`^- ${name}: (.+)$`, 'm'))?.[1]?.trim() ?? '';
  const acceptanceStart = text.indexOf('## Acceptance');
  const boundaryStart = text.indexOf('## Boundary', acceptanceStart);
  const acceptance = text.slice(acceptanceStart, boundaryStart).split(/\r?\n/)
    .filter((line) => /^- /.test(line)).map((line) => line.slice(2).trim());
  return {
    text,
    targetRepository: field('Target repository'),
    category: field('Category'),
    module: field('Module'),
    priority: field('Priority'),
    risk: field('Risk'),
    acceptance,
  };
}

function operation(opId, kind, localId, requestBody, remotePath, dependsOn = []) {
  return {
    opId,
    kind,
    localId,
    dependsOn,
    request: requestBody,
    remote: { method: kind === 'bindRequirementReference' ? 'PUT' : 'POST', path: remotePath },
    requestDigest: sha256(JSON.stringify(requestBody)),
    status: 'pending',
  };
}

function prepareOperations() {
  if (manifest.operations.length) return;
  const roomNode = manifest.nodes.find((node) => node.localId === 'room');
  const roomDescription = [
    'workflow-plan: ecs-entity-chat-20260901/r1',
    '规划范围：正式 ECS Entity、Account Server、Room admission、通用 NetEntityId/Attribute Query、ChatComponent、ReplicaWorld、五分钟重连与 ECS Snapshot/Restore。',
    '主验收场景：100 BotEntity + 1 PlayerEntity = 101 Game ECS Entities。',
    'RM-00010 Hello World 保持 archived 且不修改；本次不创建里程碑。',
  ].join('\n\n');
  manifest.operations.push({
    ...operation('create-room', 'createRoom', roomNode.localId, {
      name: roomNode.title,
      description: roomDescription,
      module: 'ECS / Account / Room / Chat',
    }, '/rooms'),
    remote: { method: 'POST', path: '/rooms' },
  });

  for (const node of manifest.nodes.filter((item) => item.objectType === 'requirement')) {
    const card = parseCard(node.localId);
    const body = {
      title: node.title,
      description: card.text,
      priority: card.priority,
      risk: card.risk,
      module: card.module,
      category: card.category,
      reason: `User-authorized ECS formal entity/chat blueprint ${manifest.source.revision}; source ${node.localId}.`,
    };
    const op = operation(`create-${node.localId.toLowerCase()}`, 'createRequirement', node.localId, body, '/requirements', ['create-room']);
    op.request.roomId = '__ROOM_ID__';
    op.requestDigest = sha256(JSON.stringify(op.request));
    manifest.operations.push(op);
  }
  persist();
}

function replaceRoomPlaceholder(op, roomId) {
  if (op.request?.roomId === '__ROOM_ID__') {
    op.request.roomId = roomId;
    op.requestDigest = sha256(JSON.stringify(op.request));
  }
}

async function getAllRequirements() {
  const items = [];
  let cursor = '';
  do {
    const query = cursor ? `&cursor=${encodeURIComponent(cursor)}` : '';
    const page = await request('GET', `/requirements?limit=50${query}`);
    items.push(...(page.items ?? []));
    cursor = page.nextCursor ?? '';
  } while (cursor);
  return items;
}

async function getAllRooms() {
  const page = await request('GET', '/rooms?limit=50');
  return page.items ?? [];
}

async function preflight() {
  const me = await request('GET', '/me');
  const project = await request('GET', '/projects/current');
  if (project.project?.id !== manifest.project.projectId || project.project?.status !== 'active') {
    throw new Error('project binding changed or project is not active');
  }
  const queries = manifest.dedupe?.queries ?? [];
  const searchResults = [];
  const knownTitles = new Set([
    manifest.planning.roomName,
    ...manifest.nodes.filter((node) => node.objectType === 'requirement').map((node) => node.title),
  ]);
  for (const query of queries) {
    const result = await request('GET', `/search?q=${encodeURIComponent(query)}&scope=mixed&limit=50`);
    const knownIds = new Set([
      manifest.remoteMap.room?.id,
      ...Object.values(manifest.remoteMap).filter((value) => value?.id).map((value) => value.id),
    ]);
    const unexpected = (result.items ?? []).filter((item) => !knownIds.has(item.id) && !knownTitles.has(item.title));
    searchResults.push({ query, count: (result.items ?? []).length, knownCount: (result.items ?? []).length - unexpected.length });
    if (unexpected.length) throw new Error(`global dedupe hit for ${query}`);
  }
  const [rooms, requirements] = await Promise.all([getAllRooms(), getAllRequirements()]);
  const marker = 'workflow-plan: ecs-entity-chat-20260901/r1';
  const roomHit = rooms.find((room) => room.name === manifest.planning.roomName || room.description?.includes(marker));
  const roomOp = manifest.operations.find((op) => op.opId === 'create-room');
  if (roomHit && !(roomOp?.status === 'verified' && manifest.remoteMap.room?.id === roomHit.id)) {
    throw new Error(`room already exists: ${roomHit.displayKey ?? roomHit.id}`);
  }
  const requirementHits = requirements.filter((item) => item.description?.includes(marker));
  if (requirementHits.length) throw new Error(`blueprint requirements already exist: ${requirementHits.map((item) => item.displayKey).join(',')}`);
  if (roomHit) {
    for (const op of manifest.operations.filter((item) => item.kind === 'createRequirement' && item.status !== 'verified')) {
      const hit = requirements.find((item) => item.title === op.request.title && item.roomId === roomHit.id);
      if (!hit) continue;
      if (hit.description !== op.request.description) throw new Error(`${op.localId} existing requirement body differs`);
      manifest.remoteMap[op.localId] = { id: hit.id, displayKey: hit.displayKey, title: hit.title, deepLink: hit.deepLink };
      await record(op, 'verified', { remoteId: hit.id, displayKey: hit.displayKey, updatedAt: hit.updatedAt, recovered: true });
    }
  }
  manifest.dedupe = { ...(manifest.dedupe ?? {}), phase: 'verified', lastRunAt: new Date().toISOString(), searchResults, listCounts: { rooms: rooms.length, requirements: requirements.length } };
  manifest.audit = { projectUser: me.userId, projectId: project.project.id };
  persist();
}

async function createRoom(op) {
  if (op.status === 'verified' && manifest.remoteMap.room?.id) return manifest.remoteMap.room;
  const result = await request('POST', '/rooms', op.request, `${manifest.bundleId}:${op.opId}`);
  const readBack = await request('GET', `/rooms/${result.id}`);
  if (readBack.name !== op.request.name || readBack.description !== op.request.description) throw new Error('room read-back mismatch');
  manifest.remoteMap.room = { id: readBack.id, displayKey: readBack.displayKey, title: readBack.name, deepLink: readBack.deepLink };
  await record(op, 'verified', { remoteId: readBack.id, displayKey: readBack.displayKey, updatedAt: readBack.updatedAt });
  return manifest.remoteMap.room;
}

async function createRequirement(op, roomId) {
  if (op.status === 'verified' && manifest.remoteMap[op.localId]?.id) return manifest.remoteMap[op.localId];
  replaceRoomPlaceholder(op, roomId);
  const result = await request('POST', '/requirements', op.request, `${manifest.bundleId}:${op.opId}`);
  const readBack = await request('GET', `/requirements/${result.id}`);
  if (readBack.title !== op.request.title || readBack.roomId !== roomId || readBack.description !== op.request.description) {
    throw new Error(`${op.localId} requirement read-back mismatch`);
  }
  const remote = { id: readBack.id, displayKey: readBack.displayKey, title: readBack.title, deepLink: readBack.deepLink };
  manifest.remoteMap[op.localId] = remote;
  await record(op, 'verified', { remoteId: readBack.id, displayKey: readBack.displayKey, updatedAt: readBack.updatedAt });
  return remote;
}

function addAcceptanceOperations() {
  if (manifest.operations.some((op) => op.kind === 'createAcceptanceItem')) return;
  const acceptanceTypeId = manifest.acceptanceConfig.acceptanceTypeId;
  const statusId = manifest.acceptanceConfig.statusId;
  for (const node of manifest.nodes.filter((item) => item.objectType === 'requirement')) {
    const card = parseCard(node.localId);
    const parentOp = `create-${node.localId.toLowerCase()}`;
    card.acceptance.forEach((text, index) => {
      const body = {
        text,
        acceptanceTypeId,
        statusId,
        sourceKind: 'ai',
        sourceRef: `workflow-plan: ecs-entity-chat-20260901/r1/${node.localId}#acceptance-${index + 1}`,
        sortOrder: index + 1,
        reason: `Blueprint acceptance item from ${node.localId}.`,
      };
      manifest.operations.push({
        ...operation(`acceptance-${node.localId.toLowerCase()}-${index + 1}`, 'createAcceptanceItem', node.localId, body, '/requirements/__REQUIREMENT_ID__/acceptance-items', [parentOp]),
        acceptanceIndex: index,
      });
    });
  }
}

function addEdgeOperations() {
  if (manifest.operations.some((op) => op.kind === 'bindRequirementReference')) return;
  for (const edge of manifest.edges ?? []) {
    const upstream = edge.upstream.localId;
    const downstream = edge.downstream.localId;
    manifest.operations.push({
      opId: `edge-${edge.edgeId}`,
      kind: 'bindRequirementReference',
      localId: edge.edgeId,
      dependsOn: [`create-${upstream.toLowerCase()}`, `create-${downstream.toLowerCase()}`],
      edge,
      request: {},
      remote: { method: 'PUT', path: '/requirements/__UPSTREAM_ID__/references/__DOWNSTREAM_ID__' },
      requestDigest: sha256(JSON.stringify({ upstream, downstream })),
      status: 'pending',
    });
  }
}

async function runBounded(ops, limit, workerFn) {
  let next = 0;
  async function worker() {
    while (true) {
      const op = ops[next++];
      if (!op) return;
      if (op.status === 'verified') continue;
      try { await workerFn(op); }
      catch (error) { await record(op, 'failed', { error: String(error) }); throw error; }
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, ops.length) }, worker));
}

async function createAcceptance(op) {
  if (op.status === 'verified') return;
  const parent = manifest.remoteMap[op.localId];
  if (!parent?.id) throw new Error(`missing parent requirement for ${op.opId}`);
  const route = `/requirements/${parent.id}/acceptance-items`;
  const result = await request('POST', route, op.request, `${manifest.bundleId}:${op.opId}`);
  const list = await request('GET', route);
  const found = (list.items ?? []).find((item) => item.id === result.id);
  if (!found || found.text !== op.request.text || found.sortOrder !== op.request.sortOrder || found.sourceRef !== op.request.sourceRef) {
    throw new Error(`${op.opId} acceptance read-back mismatch`);
  }
  await record(op, 'verified', { remoteId: found.id, updatedAt: found.updatedAt });
}

async function bindEdge(op) {
  if (op.status === 'verified') return;
  const upstream = manifest.remoteMap[op.edge.upstream.localId]?.id;
  const downstream = manifest.remoteMap[op.edge.downstream.localId]?.id;
  if (!upstream || !downstream) throw new Error(`missing edge endpoints for ${op.opId}`);
  const route = `/requirements/${upstream}/references/${downstream}`;
  await request('PUT', route, undefined, `${manifest.bundleId}:${op.opId}`);
  const graph = await request('GET', '/requirement-graph');
  if (graph.truncated) throw new Error(`${op.opId} graph read-back truncated`);
  const pair = (graph.edges ?? []).some((edge) => edge.type === 'requirement_reference' && edge.source === upstream && edge.target === downstream)
    || (graph.edges ?? []).some((edge) => edge.type === 'requirement_reference' && edge.source === downstream && edge.target === upstream);
  if (!pair) throw new Error(`${op.opId} reference missing from graph`);
  await record(op, 'verified', { remoteId: `${upstream}/${downstream}`, graphTruncated: Boolean(graph.truncated) });
}

async function main() {
  prepareOperations();
  await preflight();
  const types = await request('GET', `/projects/${manifest.project.projectId}/acceptance/types`);
  const requirementType = (types.items ?? []).find((item) => item.name === '需求验收' && item.status === 'active');
  const requirementStatus = requirementType?.statuses?.find((item) => item.systemSemantic === 'not_started' && item.status === 'active');
  if (!requirementType || !requirementStatus) throw new Error('active 需求验收 type/status not found');
  manifest.acceptanceConfig = {
    acceptanceTypeId: requirementType.id,
    acceptanceTypeName: requirementType.name,
    statusId: requirementStatus.id,
    statusName: requirementStatus.name,
    observedAt: new Date().toISOString(),
  };
  addAcceptanceOperations();
  addEdgeOperations();
  persist();

  manifest.checkpoint.phase = 'uploading';
  persist();
  const room = await createRoom(manifest.operations.find((op) => op.opId === 'create-room'));
  const requirementOps = manifest.operations.filter((op) => op.kind === 'createRequirement');
  await runBounded(requirementOps, manifest.upload.concurrency, (op) => createRequirement(op, room.id));
  const acceptanceOps = manifest.operations.filter((op) => op.kind === 'createAcceptanceItem');
  // Acceptance subresources are rate-limited per token; keep their writes ordered.
  await runBounded(acceptanceOps, 1, createAcceptance);
  const edgeOps = manifest.operations.filter((op) => op.kind === 'bindRequirementReference');
  for (const op of edgeOps) {
    try { await bindEdge(op); }
    catch (error) { await record(op, 'failed', { error: String(error) }); throw error; }
  }

  const finalRoom = await request('GET', `/rooms/${room.id}`);
  const finalRequirements = await request('GET', `/requirements?roomId=${room.id}&limit=50`);
  const expectedRequirements = manifest.nodes.filter((node) => node.objectType === 'requirement').length;
  if (finalRoom.id !== room.id || finalRequirements.items?.length !== expectedRequirements) {
    throw new Error(`final room verification failed: requirements=${finalRequirements.items?.length}, expected=${expectedRequirements}`);
  }
  const graph = await request('GET', '/requirement-graph');
  manifest.checkpoint.phase = 'complete';
  manifest.checkpoint.lastError = null;
  manifest.checkpoint.summary = {
    room: { id: finalRoom.id, displayKey: finalRoom.displayKey, title: finalRoom.name },
    requirements: finalRequirements.items.map((item) => ({ id: item.id, displayKey: item.displayKey, title: item.title, roomId: item.roomId })),
    acceptanceItems: acceptanceOps.length,
    directEdges: edgeOps.length,
    graphTruncated: Boolean(graph.truncated),
  };
  persist();
  fs.appendFileSync(eventsPath, `${JSON.stringify({ event: 'bundle.complete', bundleId: manifest.bundleId, room: finalRoom.displayKey, requirements: expectedRequirements, acceptanceItems: acceptanceOps.length, directEdges: edgeOps.length })}\n`);
  console.log(JSON.stringify({ bundleId: manifest.bundleId, phase: manifest.checkpoint.phase, room: finalRoom.displayKey, requirements: expectedRequirements, acceptanceItems: acceptanceOps.length, directEdges: edgeOps.length }));
}

main().catch((error) => {
  manifest.checkpoint.phase = 'partial';
  manifest.checkpoint.lastError = String(error);
  persist();
  console.error(error.stack ?? String(error));
  process.exitCode = 1;
});
