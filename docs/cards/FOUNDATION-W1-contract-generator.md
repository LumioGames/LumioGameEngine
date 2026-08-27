# [LumioGameEngineArchitecture] Foundation W1：契约生成器（六类 Artifact，Rust/C# 双输出）

> **状态**：已落地（本仓实现并发布 `packages/`；未另走 Workflow 落单。Voxel R-00037 / R-00045 消费该发布面）
> **module**：LumioGameEngineArchitecture
> **wave**：Foundation W1（本阶段第一张卡，无同 wave 并行卡）
> **前置**：无（`LGE-V1.4-2026-08-27` 已发布并推送；ADR-037/038/039 已 Accepted）

## 卡片正文（自包含 Agent 提示词）

BaselineId：`LGE-V1.4-2026-08-27`。

在 `LumioGameEngineArchitecture` 仓 `tools/` 下实现**契约生成器**。它是 ADR-007/023 定义的工具链的第一个生成器实体：消费本仓 `schemas/`（含 `common.schema.json` 公共 `$defs` 与 `state-machine-descriptor` 描述符契约）、`ids/index.json` 和 `fixtures/`（12 个状态机描述符实例是生成输入，不只是测试样例），产出 ADR-023 六类 Artifact（`ProtocolPermissionValidator`、`MappingTable`、`CanonicalSerializer`、`LanguageBinding`、`ContractTypes`、`ContractRuntime`），每类 Rust / C# 双输出。目标：让 checksum 公式、状态转移表、envelope codec 从「只活在 `lumio_contract.py`」变成有 Hash 记录的生成物，供七个实现仓以只读包引用。

约束（全部来自已冻结 ADR，违反即退回）：

1. **发布方唯一**（ADR-023）：生成物只由本仓发布；`implementationDependencies` 必须为空，`forbiddenDependents` 固定为 `LumioClient`/`LumioGame`。
2. **ContractRuntime 双重零**（ADR-039）：纯 Rust crate + 纯 C# assembly，零 Native 依赖（C# 形态必须能跑 PureHeadless 存档剖面，§16.1）、零领域语义（hash 链读写器、canonical 编解码 helper、有界缓冲守卫；链断裂只报分类，不定策略）。
3. **状态转移表单源**（ADR-038）：`StateTransitionTable` 属 `ContractTypes` 族，从 12 个描述符 fixture 生成；落地后把 `lumio_contract.py` 里手写的 `_ABILITY_TRANSITIONS`/`_EFFECT_TRANSITIONS` 表改为从描述符派生，消除双写。
4. **canonical 序列化字节域收口**（ADR-037 §2.2 的遗留）：`snapshot-header.checksum` 覆盖的字节域随 `CanonicalSerializer` 一起冻结（D-002 族裁决在此卡内完成并回写 `DECISIONS_PENDING.md`），在此之前实现仓不得自行发明公式。
5. **每个 Artifact 记录五元组证据**：`baselineId`、`schemaEpoch`、`compilerHash`、`inputHash`、`outputHash`，并产出通过 `generated-contract-artifact.schema.json` 校验的描述符；同输入重复生成必须字节级一致（可复现构建）。
6. **阻塞项不动**：被 D-009（protocol-dispatch）与 D-011（Auth wire）阻塞的生成目标保持 blocked，不做临时实现、不占用 Artifact 命名。

## 结构化验收项

- [x] `tools/` 下生成器入口可用（命令形式自定，如 `python3 tools/lumio_contract.py generate --out <dir>`），一次运行产出六类 Artifact 的 Rust + C# 形态与对应 descriptor JSON。
- [x] 全部 descriptor JSON 通过 `python3 tools/lumio_contract.py validate` 所用的同一 schema 门（`gencfg/*` 语义检查含在内）。
- [x] 连续两次生成，`outputHash` 逐 Artifact 相等（确定性证明，命令输出附交回物）。
- [x] `StateTransitionTable` 覆盖 12 台状态机且与描述符 fixture 集合相等；`lumio_contract.py` 的 GAS 双表改为派生后，`validate` 仍 160 fixture 全绿。
- [x] ContractRuntime C# 形态在无任何 Native 库的环境（PureHeadless 剖面）通过 hash 链读写 + 截断恢复的自测样例；Rust 形态不链接任何引擎 crate（`cargo tree` 证据）。
- [x] canonical 字节域裁决已写回 `DECISIONS_PENDING.md`（D-002 族状态更新），`snapshot-header.checksum` 语义在生成物文档中有明确定义。
- [x] 收口门槛全绿：`node .spec/tools/spec-lint.mjs && node --test .spec/tools/spec-lint.test.mjs && python3 -m py_compile tools/lumio_contract.py && python3 tools/lumio_contract.py validate`；`repository-policy` CI 绿。
- [x] known gaps 列明且不含 P0/P1（D-009/D-011 阻塞项在列，标 blocked 而非 gap）。

## 范围外（明确不做）

- 七个实现仓对生成物的消费接入（后续 wave 逐仓卡）。
- protocol-dispatch、Auth wire（D-009/D-011 未裁决）。
- Mod/P2 相关生成目标（ADR-015 Reserved）。
