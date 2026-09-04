---
name: testing
description: 测试与验收——测试分层政策、TDD 时机、验收 DoD 与验证证据;实现功能/修 bug 时查
metadata:
  type: doc
  status: 已交付
---

# 测试与验收（含 TDD 政策）

> 本文定**政策**（测什么、何时测、怎么算过）；“先写失败测试再实现”的**方法**在技能 [`skills/test-driven-development`](../../skills/test-driven-development/SKILL.md)。

## 测试分层（通用政策）

- **单元测试**：默认层，随项目验证命令（`AGENTS.md`「收口门槛」）每次跑，快、无外部依赖。
- **集成测试**（真库 / 真服务）：显式触发，不进默认验证命令，保持收口快。
- **端到端 / E2E**：显式触发；关键主链路至少一条。

## 何时走 TDD

- 必须走：新功能、修 bug（先写能复现的失败测试，修完留作回归测试）、改无测试保护的关键逻辑。
- 可不走：纯文档改动、一次性脚本。豁免在交回物里声明。
- 写测试、加 mock、想给生产类加 test-only 方法前，先查反模式清单：[`testing-anti-patterns.md`](../../skills/test-driven-development/testing-anti-patterns.md)——测 mock 行为、test-only 方法入生产、不理解依赖就 mock、不完整 mock，一律禁止。

## 验证证据

形式要求以 `AGENTS.md`「交回物格式」为单一权威——「已通过」三个字不是证据。

## 验收标准（Definition of Done）

- [ ] 收口门槛命令全绿（spec-lint、自测、Python 编译和 Contract validate 全部通过）。
- [ ] 新增 / 修改行为有测试覆盖；bug 修复留有回归测试。
- [ ] 无 lint / 类型错误、无调试残留。
- [ ] 相关知识文档已更新（见 [`workflow.md`](./workflow.md)）。

## 项目测试栈与命令

完整本地门槛：

```text
python3 -m pip install -r requirements-dev.txt
node .spec/tools/spec-lint.mjs
node --test .spec/tools/spec-lint.test.mjs
python3 -m py_compile tools/lumio_contract.py
python3 tools/lumio_contract.py validate
```

文档环境没有 `jsonschema` 时校验器会运行确定性子集；CI 与正式验收必须安装锁定依赖以覆盖 Draft 2020-12 完整语义。单 Fixture 可用 `python3 tools/lumio_contract.py validate --fixture <id>`，机器证据可用 `--json` 输出。

## 架构契约验收

- 每个新增/修改公共 Schema 至少有一份正向 Fixture 和一份失败 Fixture，失败原因可稳定分类。
- Schema、Fixture、ID Registry 和各自索引必须双向一致；公共 ID 不重复、不复用。
- Commit ordering、Revision monotonicity、Release exact match、Maintenance action pairing 与 V1 Mod 限制等语义检查必须通过。
- 修改架构正文后，通过 CI 的构建与 Host 装载证明（Living Architecture 不再维护 BaselineId 与正文 Hash 清单）。
- 影响实现仓库的变更必须列出八仓同步范围；Schema/ADR 文档样例不能替代实现仓的单元、Benchmark 或故障测试。
