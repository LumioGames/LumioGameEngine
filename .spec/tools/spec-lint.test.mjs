// spec-lint 自测:在临时目录搭 fixture 仓库,断言各类违规被抓、合法仓库全绿。
// 运行:node --test tools/
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { mkdtempSync, mkdirSync, writeFileSync, symlinkSync, rmSync } from 'node:fs'
import { join, dirname, resolve } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'

const LINT = join(dirname(fileURLToPath(import.meta.url)), 'spec-lint.mjs')

function createFixtureLink(target, link, type = 'dir') {
  try {
    symlinkSync(target, link, process.platform === 'win32' ? type : undefined)
  } catch (error) {
    if (process.platform !== 'win32' || type !== 'dir' || !['EACCES', 'EPERM', 'ENOTSUP'].includes(error.code)) throw error
    // Directory junctions preserve the same realpath check when native links are unavailable.
    symlinkSync(resolve(dirname(link), target), link, 'junction')
  }
}

/** 生成一个最小合法仓库,返回根路径;overrides 可改写/追加文件(值为 null 表示删除该默认文件)。 */
function fixture(overrides = {}, linkOverrides = {}) {
  const root = mkdtempSync(join(tmpdir(), 'spec-lint-fixture-'))
  const files = {
    'CLAUDE.md': '# CLAUDE.md\n\n@.spec/AGENTS.md\n\n@.spec/knowledge/README.md\n\n@.spec/rules/system.md\n',
    '.spec/AGENTS.md': '# 中心文档\n\n| 名称 | 职责 |\n|------|------|\n| `coder` | 写代码 |\n',
    '.spec/rules/system.md': '# 规则\n',
    '.spec/knowledge/README.md': [
      '---', 'name: knowledge', 'description: 导航', 'metadata:', '  type: index', '---', '',
      '# 导航', '', '| 文档 | 一句话 |', '|------|--------|',
      '| [`standards/workflow.md`](standards/workflow.md) | 工作流 |',
      '| [`features/_TEMPLATE.md`](features/_TEMPLATE.md) | 模板 |', '',
    ].join('\n'),
    '.spec/knowledge/standards/workflow.md':
      '---\nname: workflow\ndescription: 工作流\nmetadata:\n  type: doc\n  status: 已交付\n---\n\n# 工作流\n',
    '.spec/knowledge/features/_TEMPLATE.md':
      '---\nname: template\ndescription: 模板\nmetadata:\n  type: doc\n  status: 设计中\n---\n\n# 模板\n',
    '.spec/agents/coder.agent.md': '---\nname: coder\ndescription: 写代码\n---\n\n# Coder\n',
    '.spec/skills/demo/SKILL.md': '---\nname: demo\ndescription: 演示\n---\n\n# Demo\n',
    ...overrides,
  }
  for (const [rel, content] of Object.entries(files)) {
    if (content === null) continue
    const p = join(root, rel)
    mkdirSync(dirname(p), { recursive: true })
    writeFileSync(p, content)
  }
  mkdirSync(join(root, '.claude'), { recursive: true })
  mkdirSync(join(root, '.agents'), { recursive: true })
  const links = {
    '.claude/agents': '../.spec/agents',
    '.claude/skills': '../.spec/skills',
    '.agents/skills': '../.spec/skills',
    ...linkOverrides,
  }
  for (const [rel, target] of Object.entries(links)) createFixtureLink(target, join(root, rel))
  return root
}

/** 在最小合法仓库上加一个 decisions/ 分区,ADR 内容可覆写。 */
function decisionsFixture(adrs = { 'ADR-050-x.md': '# ADR-050\n\n- **Status**: Draft\n' }) {
  const root = fixture()
  const decisions = join(root, '.spec', 'decisions')
  mkdirSync(decisions, { recursive: true })
  const index = Object.keys(adrs).map((n) => `[${n}](${n})`).join('\n')
  writeFileSync(join(decisions, 'README.md'), `# Decisions\n\n${index}\n`)
  const compatibility = join(root, 'docs', 'adr')
  mkdirSync(compatibility, { recursive: true })
  for (const [name, body] of Object.entries(adrs)) {
    writeFileSync(join(decisions, name), body)
    if (/^ADR-\d{3}-.+\.md$/.test(name)) {
      createFixtureLink(`../../.spec/decisions/${name}`, join(compatibility, name), 'file')
    }
  }
  return root
}

/** 跑 lint,返回 { code, output }。 */
function lint(root) {
  try {
    const out = execFileSync(process.execPath, [LINT, root], { encoding: 'utf8' })
    return { code: 0, output: out }
  } catch (e) {
    return { code: e.status ?? 1, output: `${e.stdout ?? ''}${e.stderr ?? ''}` }
  } finally {
    rmSync(root, { recursive: true, force: true })
  }
}

test('最小合法仓库全绿', () => {
  const { code, output } = lint(fixture())
  assert.equal(code, 0, output)
  assert.match(output, /spec-lint: OK/)
})

test('knowledge 文档未登记导航被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/knowledge/standards/hidden.md':
      '---\nname: hidden\ndescription: 隐身\nmetadata:\n  type: doc\n  status: 设计中\n---\n\n# 隐身\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /未登记进 knowledge\/README\.md 导航/)
})

test('悬空链接被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/AGENTS.md': '# 中心文档\n\n| 名称 | 职责 |\n|------|------|\n| `coder` | 写代码 |\n\n[不存在](nowhere.md)\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /悬空链接:nowhere\.md/)
})

test('rules 文件缺 @import 行被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/rules/extra.md': '# 另一份规则\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /缺 @import 行:@\.spec\/rules\/extra\.md/)
})

test('status 非枚举被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/knowledge/standards/workflow.md':
      '---\nname: workflow\ndescription: 工作流\nmetadata:\n  type: doc\n  status: 草稿\n---\n\n# 工作流\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /status「草稿」不在枚举/)
})

test('description 多行标量被抓(不再绕过长度校验)', () => {
  const { code, output } = lint(fixture({
    '.spec/knowledge/standards/workflow.md':
      '---\nname: workflow\ndescription: >-\n  很长很长的描述\nmetadata:\n  type: doc\n  status: 已交付\n---\n\n# 工作流\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /description 必须单行明文/)
})

test('名册幽灵行被抓(名册有、文件无)', () => {
  const { code, output } = lint(fixture({
    '.spec/AGENTS.md': '# 中心文档\n\n| 名称 | 职责 |\n|------|------|\n| `coder` | 写代码 |\n| `ghost` | 不存在 |\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /名册幽灵行:「ghost」/)
})

test('缺 CLAUDE.md 给可读报错而非崩栈', () => {
  const { code, output } = lint(fixture({ 'CLAUDE.md': null }))
  assert.equal(code, 1)
  assert.match(output, /缺核心文件:CLAUDE\.md/)
  assert.doesNotMatch(output, /at .*spec-lint\.mjs:\d/) // 无堆栈
})

test('任务卡 status 非枚举被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/tasks/demo-card.md': '---\nstatus: done\n---\n\n# 演示卡\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /status「done」不在枚举/)
})

test('任务卡缺 frontmatter 被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/tasks/demo-card.md': '# 裸卡\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /任务卡缺少 frontmatter/)
})

test('任务卡多余 frontmatter 字段被抓', () => {
  const { code, output } = lint(fixture({
    '.spec/tasks/demo-card.md': '---\nstatus: pending\nowner: me\n---\n\n# 卡\n',
  }))
  assert.equal(code, 1)
  assert.match(output, /只允许 status/)
})

test('合法任务卡通过,子目录与 README 不校验', () => {
  const { code, output } = lint(fixture({
    '.spec/tasks/demo-card.md': '---\nstatus: in_progress\n---\n\n# 卡\n',
    '.spec/tasks/README.md': '# 任务卡目录\n',
    '.spec/tasks/sub-dir/stray-card.md': '---\nstatus: whatever\n---\n\n# 子目录卡\n',
  }))
  assert.equal(code, 0, output)
})

test('软链接缺失被抓', () => {
  const root = fixture()
  rmSync(join(root, '.claude/agents'))
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /软链接缺失/)
})

test('sibling-prefix link target is rejected', () => {
  const { code, output } = lint(fixture({
    '.spec-evil/agents/marker.md': '# outside\n',
  }, {
    '.claude/agents': '../.spec-evil/agents',
  }))
  assert.equal(code, 1)
  assert.match(output, /软链接未解析进 \.spec/)
  assert.match(output, /spec-evil[\\/]agents/)
})

test('case-variant link target follows platform case semantics', () => {
  const { code, output } = lint(fixture({
    '.SPEC/agents/marker.md': '# case variant\n',
  }, {
    '.claude/agents': '../.SPEC/agents',
  }))
  if (process.platform === 'win32') {
    assert.equal(code, 0, output)
    assert.match(output, /spec-lint: OK/)
  } else {
    assert.equal(code, 1, output)
    assert.match(output, /软链接未解析进 \.spec/)
    assert.match(output, /[\\/]\.SPEC[\\/]agents/)
  }
})

test('并行文档根 docs/ 重新出现被抓', () => {
  const root = fixture()
  mkdirSync(join(root, 'docs', 'specs'), { recursive: true })
  writeFileSync(join(root, 'docs', 'specs', 'x.md'), '# x\n')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /并行文档根/)
})

test('并行文档根 .sdd\/ 与 .workflow-drafts\/ 一并被抓', () => {
  const root = fixture()
  mkdirSync(join(root, '.sdd'), { recursive: true })
  mkdirSync(join(root, '.workflow-drafts'), { recursive: true })
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /\.sdd/)
  assert.match(output, /\.workflow-drafts/)
})

test('嵌套的并行文档根被抓(历史病灶 engine/native/docs/ 形态)', () => {
  const root = fixture()
  mkdirSync(join(root, 'engine', 'native', 'docs', 'architecture'), { recursive: true })
  writeFileSync(join(root, 'engine', 'native', 'docs', 'architecture', 'x.md'), '# 副本\n')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /并行文档根/)
  assert.match(output, /docs[\\/]?/)
})

test('.sdd-scratch 工作区不在禁名单里(subagent-driven-development 的落点)', () => {
  const root = fixture()
  mkdirSync(join(root, '.sdd-scratch'), { recursive: true })
  writeFileSync(join(root, '.sdd-scratch', 'progress.md'), 'Task 1: complete\n')
  const { code, output } = lint(root)
  assert.equal(code, 0, output)
})

test('仓根之外的第二个 .spec 被抓(subtree 合并带进来的框架副本)', () => {
  const root = fixture()
  mkdirSync(join(root, 'engine', 'native', '.spec', 'decisions'), { recursive: true })
  writeFileSync(join(root, 'engine', 'native', '.spec', 'AGENTS.md'), '# 第二套\n')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /第二套 LumioAgent 框架/)
})

test('ADR 缺状态行被抓', () => {
  const { code, output } = lint(decisionsFixture({ 'ADR-050-x.md': '# ADR-050\n\n没有状态行。\n' }))
  assert.equal(code, 1)
  assert.match(output, /缺状态行/)
})

test('ADR 状态非枚举被抓', () => {
  const { code, output } = lint(decisionsFixture({ 'ADR-050-x.md': '# ADR-050\n\n- **Status**: Cooking\n' }))
  assert.equal(code, 1)
  assert.match(output, /ADR 状态「Cooking」不在枚举/)
})

test('ADR 状态两种写法都收,Historical 前缀合法', () => {
  const { code, output } = lint(decisionsFixture({
    'ADR-050-x.md': '# ADR-050\n\n- **Status**: Historical · Accepted (旧基线)\n',
    'ADR-051-y.md': '# ADR-051\n\n状态：Accepted（2026-08-31）\n',
    '0001-nativecore.md': '# 0001\n\n- 状态:生效\n',
  }))
  assert.equal(code, 0, output)
  assert.match(output, /spec-lint: OK/)
})

test('plans\/ 文档缺 frontmatter 被抓,且不要求进 knowledge 导航', () => {
  const root = fixture()
  mkdirSync(join(root, '.spec', 'plans'), { recursive: true })
  writeFileSync(join(root, '.spec', 'plans', 'p.md'), '# 计划\n')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /缺少 frontmatter/)
  assert.doesNotMatch(output, /未登记进 knowledge\/README\.md 导航/)
})

test('reviews\/ 合法文档全绿,不需登记进 knowledge 导航', () => {
  const root = fixture()
  mkdirSync(join(root, '.spec', 'reviews'), { recursive: true })
  writeFileSync(
    join(root, '.spec', 'reviews', 'r.md'),
    '---\nname: r\ndescription: 审查报告\nmetadata:\n  type: doc\n  status: 已交付\n---\n\n# 审查\n',
  )
  const { code, output } = lint(root)
  assert.equal(code, 0, output)
})

test('docs/adr-only 120000 mirrors are allowed (8b exception)', () => {
  const { code, output } = lint(decisionsFixture())
  assert.equal(code, 0, output)
  assert.match(output, /spec-lint: OK/)
})

test('root ADR without docs/adr symlink is caught', () => {
  const root = fixture()
  const decisions = join(root, '.spec', 'decisions')
  mkdirSync(decisions, { recursive: true })
  writeFileSync(join(decisions, 'README.md'), '# Decisions\n\n[ADR-053](ADR-053-entity-binding-and-attribute-query.md)\n')
  writeFileSync(
    join(decisions, 'ADR-053-entity-binding-and-attribute-query.md'),
    '# ADR-053\n\n- **Status**: Accepted\n',
  )
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /compatibility ADR/)
})

test('docs/specs still forbidden when docs/adr exists', () => {
  const root = decisionsFixture()
  mkdirSync(join(root, 'docs', 'specs'), { recursive: true })
  writeFileSync(join(root, 'docs', 'specs', 'x.md'), '# x\n')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /并行文档根/)
})

test('compatibility ADR regular files are rejected', () => {
  const root = fixture()
  const decisions = join(root, '.spec', 'decisions')
  mkdirSync(decisions, { recursive: true })
  writeFileSync(join(decisions, 'README.md'), '# Decisions\n\n[ADR-050](ADR-050-gas-a1-contracts.md)\n')
  writeFileSync(join(decisions, 'ADR-050-gas-a1-contracts.md'), '# ADR-050\n\n- **Status**: Accepted\n')
  mkdirSync(join(root, 'docs', 'adr'), { recursive: true })
  writeFileSync(join(root, 'docs', 'adr', 'ADR-050-gas-a1-contracts.md'), '../../.spec/decisions/ADR-050-gas-a1-contracts.md')
  const { code, output } = lint(root)
  assert.equal(code, 1)
  assert.match(output, /compatibility ADR must be a Git mode 120000 symbolic link/)
})
