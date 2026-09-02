#!/usr/bin/env node
/**
 * spec-lint — .spec/ 结构一致性机械校验。改完 .spec/ 必须跑一次;CI 里随 workflow 执行。
 * 用法:node .spec/tools/spec-lint.mjs [仓库根目录]   (省略参数时取本脚本上级目录)
 *
 * 校验项清单(本注释是全仓「lint 能力清单」的单一权威,其他文档只指回这里):
 *  1. 核心文件存在:CLAUDE.md、.spec/AGENTS.md、.spec/knowledge/README.md 缺失时给可读报错,不崩栈。
 *  2. 文档 frontmatter(knowledge/features、knowledge/standards、plans/、reviews/):
 *     name / description / metadata.type / metadata.status 齐全;
 *     status 只能取枚举(设计中 / 实施中 / 已交付 / 历史归档);description ≤ 120 字符,
 *     且必须单行明文(禁 YAML 多行标量 > / |,防止绕过长度校验)。
 *  3. 导航覆盖:features/ 与 standards/ 及 knowledge 根下的 .md 必须被 knowledge/README.md
 *     链接到(索引漂移 = 知识隐身);decisions/ 下每条 ADR 必须登记进 decisions/README.md 索引。
 *  4. 链接可达:.spec 下全部 .md 与根 README.md / AGENTS.md 的相对链接必须指向存在的文件
 *     (剥围栏代码块与行内代码,避免代码里的 [T](x) 误判)。
 *  5. 强制载入完整性:rules/ 下每个 .md、AGENTS.md、knowledge/README.md 都必须有根 CLAUDE.md
 *     的对应 @import 行(漏一行 = init 静默失效)。
 *  6. agents / skills frontmatter:只允许 name + description,且 name 与文件 / 目录名一致。
 *  7. 名册一致(双向):agents/ 下每个角色必须出现在 AGENTS.md 名册表;名册表每行也必须有
 *     对应的 .agent.md 文件(幽灵行)。
 *  8. 软链接存活:.claude/agents、.claude/skills、.agents/skills 必须存在且解析进 .spec/。
 * 8b. 无并行文档根:docs/ / .sdd/ / .workflow-drafts/ 不得存在于仓内任何层级(历史病灶
 *     包括嵌套的 engine/native/docs/);唯一例外是仓根 docs/adr/ 作为 .spec/decisions
 *     的 git mode 120000 镜像。仓根之外不得有第二个 .spec/(防止第二套
 *     LumioAgent 框架随 subtree 合并混入)。扫描有界:跳过 .git 与构建产物目录,深度 ≤7。
 * 8c. ADR 状态枚举:decisions/ 下每条 ADR 前 12 行内必须有状态行,取值只能是
 *     Historical / Draft / Accepted / Reserved / Superseded / 生效 / 废止
 *     (写法两收:`- **Status**: X` 或 `- 状态：X`;nativecore/ 子命名空间用中文写法)。
 *  9. 任务卡 frontmatter:.spec/tasks/ 根目录每张卡(README 除外)必须有 frontmatter,
 *     且只允许 status 字段,枚举 pending / in_progress / completed(契约见 tasks/README.md);
 *     子目录不校验。
 */
import { readFileSync, readdirSync, existsSync, statSync, lstatSync, realpathSync, readlinkSync } from 'node:fs'
import { join, dirname, basename, resolve, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = process.argv[2]
  ? resolve(process.argv[2])
  : resolve(dirname(fileURLToPath(import.meta.url)), '..', '..')
const SPEC = join(ROOT, '.spec')
const STATUS_ENUM = new Set(['设计中', '实施中', '已交付', '历史归档'])
const errors = []
const err = (file, msg) => errors.push(`${relative(ROOT, file)}: ${msg}`)
const normalizeContainmentPath = (path) =>
  process.platform === 'win32' ? path.toLowerCase() : path

function walk(dir, filter) {
  if (!existsSync(dir)) return []
  const out = []
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) out.push(...walk(p, filter))
    else if (filter(p)) out.push(p)
  }
  return out
}

function parseFrontmatter(file) {
  const text = readFileSync(file, 'utf8')
  if (!text.startsWith('---\n')) return null
  const end = text.indexOf('\n---\n', 4)
  if (end === -1) return null
  const body = text.slice(4, end)
  const fm = { __keys: [] }
  let inMetadata = false
  for (const line of body.split('\n')) {
    if (!line.trim()) continue
    const m = line.match(/^(\s*)([\w-]+):\s*(.*)$/)
    if (!m) continue
    const [, indent, key, rawValue] = m
    const value = rawValue.replace(/\s+#.*$/, '').trim()
    if (indent === '') {
      inMetadata = key === 'metadata'
      fm.__keys.push(key)
      if (!inMetadata) fm[key] = value
    } else if (inMetadata) {
      fm[`metadata.${key}`] = value
    }
  }
  return fm
}

function mdLinks(file) {
  const text = readFileSync(file, 'utf8')
    .replace(/```[\s\S]*?```/g, '') // 剥围栏代码块,避免代码里的 [T](x) 误判为链接
    .replace(/`[^`\n]*`/g, '')
  const links = []
  for (const m of text.matchAll(/\[[^\]]*\]\(([^)\s]+)\)/g)) {
    const target = m[1]
    if (/^([a-zA-Z][a-zA-Z0-9+.-]*:|#)/.test(target)) continue // 任何带 scheme 的外部链接
    links.push(decodeURIComponent(target.split('#')[0]))
  }
  return links
}

// ── 1. 核心文件存在 ───────────────────────────────────────────────────────
const CORE = {
  claudeMd: join(ROOT, 'CLAUDE.md'),
  agentsMd: join(SPEC, 'AGENTS.md'),
  navFile: join(SPEC, 'knowledge', 'README.md'),
}
for (const [label, file] of Object.entries({
  'CLAUDE.md(强制载入入口)': CORE.claudeMd,
  '.spec/AGENTS.md(中心文档)': CORE.agentsMd,
  '.spec/knowledge/README.md(知识导航)': CORE.navFile,
})) {
  if (!existsSync(file)) err(file, `缺核心文件:${label}`)
}

// ── 2. knowledge frontmatter ──────────────────────────────────────────────
const knowledgeDir = join(SPEC, 'knowledge')
const featureDocs = walk(join(knowledgeDir, 'features'), (p) => p.endsWith('.md'))
const standardDocs = walk(join(knowledgeDir, 'standards'), (p) => p.endsWith('.md'))
// 过程物(plans / reviews)同受 frontmatter 约束,但不进 knowledge/README.md 导航——
// 导航是每次 init 的强制税,过程物按需翻目录即可。
const processDocs = [
  ...walk(join(SPEC, 'plans'), (p) => p.endsWith('.md') && basename(p) !== 'README.md'),
  ...walk(join(SPEC, 'reviews'), (p) => p.endsWith('.md') && basename(p) !== 'README.md'),
]
for (const file of [...featureDocs, ...standardDocs, ...processDocs]) {
  const fm = parseFrontmatter(file)
  if (!fm) { err(file, '缺少 frontmatter'); continue }
  for (const key of ['name', 'description']) if (!fm[key]) err(file, `frontmatter 缺 ${key}`)
  for (const key of ['metadata.type', 'metadata.status']) if (!fm[key]) err(file, `frontmatter 缺 ${key}`)
  const status = fm['metadata.status']
  if (status && !STATUS_ENUM.has(status)) {
    err(file, `status「${status}」不在枚举(${[...STATUS_ENUM].join(' / ')})——历史在 git,不进文档`)
  }
  if (fm.description && /^[>|]/.test(fm.description)) {
    err(file, 'description 必须单行明文——YAML 多行标量会绕过长度校验')
  } else if (fm.description && [...fm.description].length > 120) {
    err(file, `description 超过 120 字符(${[...fm.description].length})——一句话是什么+何时查`)
  }
}

// ── 3. 导航覆盖 + ADR 索引覆盖 ────────────────────────────────────────────
if (existsSync(CORE.navFile)) {
  const navLinkSet = new Set(mdLinks(CORE.navFile).map((l) => resolve(knowledgeDir, l)))
  const rootDocs = readdirSync(knowledgeDir)
    .filter((n) => n.endsWith('.md') && n !== 'README.md')
    .map((n) => join(knowledgeDir, n))
  for (const file of new Set([...featureDocs, ...standardDocs, ...rootDocs])) {
    if (!navLinkSet.has(file)) err(file, '未登记进 knowledge/README.md 导航(索引漂移 = 知识隐身)')
  }
}
const decisionsDir = join(SPEC, 'decisions')
if (existsSync(decisionsDir)) {
  const adrIndex = join(decisionsDir, 'README.md')
  const adrLinks = existsSync(adrIndex)
    ? new Set(mdLinks(adrIndex).map((l) => resolve(decisionsDir, l)))
    : new Set()
  for (const file of walk(decisionsDir, (p) => p.endsWith('.md') && basename(p) !== 'README.md')) {
    if (!adrLinks.has(file)) err(file, '未登记进 decisions/README.md 索引')
  }
}

// ── 4. 链接可达(.spec 下全部 .md + 根入口) ──────────────────────────────
const linkScanFiles = [
  ...walk(SPEC, (p) => p.endsWith('.md')),
  join(ROOT, 'README.md'),
  join(ROOT, 'AGENTS.md'),
].filter(existsSync)
for (const file of linkScanFiles) {
  for (const link of mdLinks(file)) {
    const target = resolve(dirname(file), link)
    if (!existsSync(target)) err(file, `悬空链接:${link}`)
  }
}

// ── 5. 强制载入完整性(CLAUDE.md @import) ────────────────────────────────
if (existsSync(CORE.claudeMd)) {
  const claudeMd = readFileSync(CORE.claudeMd, 'utf8')
  const imports = new Set([...claudeMd.matchAll(/^@(\.spec\/\S+)$/gm)].map((m) => m[1]))
  const rulesDir = join(SPEC, 'rules')
  const mustImport = [
    '.spec/AGENTS.md',
    '.spec/knowledge/README.md',
    ...(existsSync(rulesDir)
      ? readdirSync(rulesDir)
          .filter((n) => n.endsWith('.md') && n !== 'README.md')
          .map((n) => `.spec/rules/${n}`)
      : []),
  ]
  for (const path of mustImport) {
    if (!imports.has(path)) err(CORE.claudeMd, `缺 @import 行:@${path}(漏了 = init 静默不加载)`)
  }
}

// ── 6+7. agents / skills frontmatter 与名册双向一致 ───────────────────────
const agentsMd = existsSync(CORE.agentsMd) ? readFileSync(CORE.agentsMd, 'utf8') : ''
const agentFiles = walk(join(SPEC, 'agents'), (p) => p.endsWith('.agent.md'))
for (const file of agentFiles) {
  const fm = parseFrontmatter(file)
  const base = basename(file).replace('.agent.md', '')
  if (!fm) { err(file, '缺少 frontmatter'); continue }
  const keys = fm.__keys.filter((k) => k !== '__keys').sort()
  if (keys.join(',') !== 'description,name') err(file, `frontmatter 只允许 name+description,实际:${keys.join(',')}`)
  if (fm.name !== base) err(file, `frontmatter name「${fm.name}」与文件名「${base}」不一致`)
  if (agentsMd && !new RegExp(`^\\|\\s*\`${base}\``, 'm').test(agentsMd)) {
    err(file, '角色未登记进 AGENTS.md 名册表')
  }
}
// 名册反向:名册表里的每行都要有对应 .agent.md(幽灵行)
if (agentsMd) {
  const rosterNames = [...agentsMd.matchAll(/^\|\s*`([\w-]+)`/gm)].map((m) => m[1])
  for (const name of rosterNames) {
    const file = join(SPEC, 'agents', `${name}.agent.md`)
    if (!existsSync(file)) err(CORE.agentsMd, `名册幽灵行:「${name}」没有对应的 agents/${name}.agent.md`)
  }
}
for (const file of walk(join(SPEC, 'skills'), (p) => basename(p) === 'SKILL.md')) {
  const fm = parseFrontmatter(file)
  const dir = basename(dirname(file))
  if (!fm) { err(file, '缺少 frontmatter'); continue }
  const keys = fm.__keys.filter((k) => k !== '__keys').sort()
  if (keys.join(',') !== 'description,name') err(file, `frontmatter 只允许 name+description,实际:${keys.join(',')}`)
  if (fm.name !== dir) err(file, `frontmatter name「${fm.name}」与目录名「${dir}」不一致`)
}

// ── 8. 软链接存活 ─────────────────────────────────────────────────────────
for (const rel of ['.claude/agents', '.claude/skills', '.agents/skills']) {
  const link = join(ROOT, rel)
  try {
    lstatSync(link)
  } catch {
    err(link, '软链接缺失(宿主自动发现依赖它)')
    continue
  }
  try {
    const real = realpathSync(link)
    const specReal = realpathSync(SPEC)
    const comparableReal = normalizeContainmentPath(real)
    const comparableSpec = normalizeContainmentPath(specReal)
    if (comparableReal !== comparableSpec && !comparableReal.startsWith(`${comparableSpec}${sep}`)) {
      err(link, `软链接未解析进 .spec/(实际指向 ${real})`)
    }
  } catch {
    err(link, '软链接悬空(目标不存在)')
  }
}

// ── 8b. 无并行文档根:全仓文档只有 .spec/ 一个根 ──────────────────────────
// 2026-09-01 治理收敛前,仓里同时存在 docs/ / .sdd/ / .workflow-drafts/ 与
// engine/native/.spec/ 四个并行文档根,其中三个不受任何机器校验,于是两套互相
// 矛盾的制度的文档平铺在一起、外观完全一样。这条断言防止它再长回来。
function isDocsAdrCompatibilityTree(docsDir) {
  let entries
  try { entries = readdirSync(docsDir, { withFileTypes: true }) } catch { return false }
  return entries.length > 0 && entries.every((e) => e.name === 'adr' && e.isDirectory())
}

for (const forbidden of ['docs', '.sdd', '.workflow-drafts']) {
  const dir = join(ROOT, forbidden)
  if (!existsSync(dir)) continue
  if (forbidden === 'docs' && isDocsAdrCompatibilityTree(dir)) continue
  err(dir, `并行文档根:全仓文档只有 .spec/ 一个根,${forbidden}/ 不得重新出现(设计→knowledge/features、计划→plans、审查→reviews、决策→decisions、任务→tasks;docs/adr 除外)`)
}
// 有界扫描:跳过 .git 与构建产物,并容忍悬空软链(它们由第 8 项单独报)。
// 嵌套的 docs/ 同样在禁名单里——2026-09 收敛前的重复副本恰恰长在 engine/native/docs/。
const SKIP_DIRS = new Set(['.git', 'node_modules', 'target', 'bin', 'obj', 'dist', '.venv', '__pycache__'])
const FORBIDDEN_DOC_DIRS = new Set(['docs', '.sdd', '.workflow-drafts'])
function findForbiddenDirs(dir, depth = 0, out = []) {
  if (depth > 6) return out
  let entries
  try { entries = readdirSync(dir, { withFileTypes: true }) } catch { return out }
  for (const e of entries) {
    if (!e.isDirectory() || SKIP_DIRS.has(e.name)) continue
    const child = join(dir, e.name)
    try { if (lstatSync(child).isSymbolicLink()) continue } catch { continue }
    if (e.name === '.spec' && child !== SPEC) { out.push({ path: child, kind: 'spec' }); continue }
    if (FORBIDDEN_DOC_DIRS.has(e.name)) {
      if (e.name === 'docs' && depth === 0 && normalizeContainmentPath(dir) === normalizeContainmentPath(ROOT)
        && isDocsAdrCompatibilityTree(child)) {
        continue
      }
      if (depth > 0) out.push({ path: child, kind: 'docroot' }) // 根层命中由上面的 existsSync 检查报错
      continue
    }
    findForbiddenDirs(child, depth + 1, out)
  }
  return out
}
for (const { path, kind } of findForbiddenDirs(ROOT)) {
  err(path, kind === 'spec'
    ? '第二套 LumioAgent 框架:全仓只允许仓根一个 .spec/(subtree 合并会把下游仓的框架副本一起带进来)'
    : `并行文档根:${basename(path)}/ 不得在仓内任何层级出现(文档一律进 .spec/,历史病灶正是嵌套的 engine/native/docs/)`)
}

// Compatibility ADR entries must be Git mode 120000 symbolic links to the
// authoritative .spec/decisions files. Minimal lint fixtures may omit docs/adr
// when they have no root ADR-NNN files.
const compatibilityAdrDir = join(ROOT, 'docs', 'adr')
const rootAdrFiles = existsSync(decisionsDir)
  ? readdirSync(decisionsDir).filter((name) => /^ADR-\d{3}-.+\.md$/.test(name))
  : []
if (rootAdrFiles.length > 0) {
  for (const name of rootAdrFiles) {
    const expectedTarget = `../../.spec/decisions/${name}`
    const link = join(compatibilityAdrDir, name)
    try {
      const stat = lstatSync(link)
      if (!stat.isSymbolicLink()) {
        err(link, 'compatibility ADR must be a Git mode 120000 symbolic link (Windows materialization is environmental only)')
        continue
      }
      const target = readlinkSync(link)
      const resolvedTarget = resolve(dirname(link), target)
      const expectedResolved = resolve(dirname(link), expectedTarget)
      if (normalizeContainmentPath(resolvedTarget) !== normalizeContainmentPath(expectedResolved)) {
        err(link, `compatibility ADR target must resolve to ${expectedResolved}`)
      }
    } catch {
      err(link, 'compatibility ADR link is missing or unresolved')
    }
  }
}

// ── 8c. ADR 状态枚举 ──────────────────────────────────────────────────────
// 旧制度下 ADR 状态完全不受校验,于是「Accepted 不可改写」这条规则谁都能绕过。
// 两种写法都收:英文 `- **Status**: X` 与中文 `状态：X`(ADR-052 用后者)。
const ADR_STATUS_ENUM = new Set(['Historical', 'Draft', 'Accepted', 'Reserved', 'Superseded', '生效', '废止'])
if (existsSync(decisionsDir)) {
  for (const file of walk(decisionsDir, (p) => p.endsWith('.md') && basename(p) !== 'README.md')) {
    const head = readFileSync(file, 'utf8').split(/\r?\n/).slice(0, 12).join('\n')
    const m = head.match(/^\s*-?\s*\*\*Status\*\*\s*[:：]\s*(\S+)/m) || head.match(/^\s*-?\s*状态\s*[:：]\s*(\S+)/m)
    if (!m) { err(file, 'ADR 前 12 行内缺状态行(`- **Status**: X` 或 `状态：X`)'); continue }
    const first = m[1].replace(/[（(].*$/, '').trim()
    if (!ADR_STATUS_ENUM.has(first)) {
      err(file, `ADR 状态「${first}」不在枚举(${[...ADR_STATUS_ENUM].join(' / ')})`)
    }
  }
}

// ── 9. tasks/ 任务卡 frontmatter(格式契约见 .spec/tasks/README.md) ────────
const tasksDir = join(SPEC, 'tasks')
const TASK_STATUS_ENUM = new Set(['pending', 'in_progress', 'completed'])
if (existsSync(tasksDir)) {
  for (const name of readdirSync(tasksDir)) {
    const p = join(tasksDir, name)
    if (statSync(p).isDirectory() || !name.endsWith('.md') || name === 'README.md') continue
    const fm = parseFrontmatter(p)
    if (!fm) { err(p, '任务卡缺少 frontmatter(格式契约见 tasks/README.md)'); continue }
    const keys = fm.__keys.filter((k) => k !== '__keys')
    if (keys.join(',') !== 'status') err(p, `任务卡 frontmatter 只允许 status,实际:${keys.join(',')}`)
    if (!TASK_STATUS_ENUM.has(fm.status)) {
      err(p, `status「${fm.status ?? ''}」不在枚举(${[...TASK_STATUS_ENUM].join(' / ')})`)
    }
  }
}

// ── 汇总 ─────────────────────────────────────────────────────────────────
if (errors.length > 0) {
  console.error(`spec-lint: ${errors.length} 处不一致\n`)
  for (const e of errors) console.error(`  ✗ ${e}`)
  process.exit(1)
}
console.log('spec-lint: OK')
