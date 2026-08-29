# 2026-08-29 - Dedicated Server 与 MS-00001 目标剖面对齐附录

> **性质**：对最新架构提交的增量复核，不改 BaselineId，不写入 Workflow。
> **复核锚点**：架构仓 `origin/main` `d59afa9`；本地合并工作区 `d571cc8`。
> **关联文档**：[`2026-08-29-ds-server-architecture.md`](../specs/2026-08-29-ds-server-architecture.md)、[`mvp-browser-voxel-multiplayer.md`](../plans/mvp-browser-voxel-multiplayer.md)、[`2026-08-29-seven-repo-progress-assessment.md`](2026-08-29-seven-repo-progress-assessment.md)。

## 1. 新增事实

架构仓在上一轮盘点后合入了 Dedicated Server 定稿（`d59afa9`）。该提交明确：

- DS 底层核心归 `LumioServer` 的 Rust：准入、连接代次、会话、WebSocket、token bucket、WorldSlot、维护与接口；见定稿 §1、§4、§8。
- 语义层归 `LumioGameRuntime` 的 C#：13 相 Tick、ECS/GAS 真值、复制变更集、视野表、发送调度；见定稿 §1、§4。
- VoxelEngine 保持独立自治，但必须与 DS 共享连接预算、唯一提交点和确认/回滚单元。
- 定稿声明没有推翻 Accepted ADR，也没有修改 `LGE-V1.4-2026-08-27` 的 BaselineId；它是目标架构约束的收口，不是生成契约版本跃迁。

当前实现仓事实没有同步改变：

| 面 | `origin/main` 事实 | 结论 |
|---|---|---|
| `LumioServer` Rust | 只有 `modules/process` 与 `lumio-host-testkit` 的生产 Rust 代码骨架 | 定稿要求的 Rust DS 核心尚未实现 |
| `LumioServer` C# | `mvp-host` 已有 platform/wire/transport/auth 基础与测试 | 可作为现有 MVP 测试宿主，但不能自动宣称符合 Rust DS V1 边界 |
| `LumioGameRuntime` | `origin/main` 仍主要是 observability/generated contracts；本机另有未跟踪 `modules/ecs/src/`，不属于交付证据 | Runtime 仍是关键实现缺口 |
| `MS-00001` 计划 | Adopted 文档仍写明 MVP 用 C# 测试宿主，Rust Host + `coreclr-host` 延后 | 与新 DS 定稿存在目标剖面差异 |

### W0 候选发布物实测

在不触碰 `packages/` 的临时目录执行正式 generator，得到 12 个 artifact，且重复生成的 `outputHash` 稳定。候选身份为：

```text
compilerHash       6f51b99ebd1b64f3045aff9a3bbd8047bd707ff2d5ec0c9b80e476b83d89e745
inputHash          d2ed2c9e4046fe7bd5ed81e2dd74ef02db6a5671cb971e9163835f763f87bb2f
Root ABI digest    708ccb7e1bd25cb3c66caa3a13bdadfa5446ff4403a0d043333f59e737eae583
Root ABI inputHash 50743b7785279a04976dc414623ccfa41068ba552831f6d2f2768544374a2959
```

当前发布的 `packages/` 仍记录 `compilerHash=0aaf61...`、`inputHash=bb95d870...`、Root ABI digest `02dce705...`。对 70 个受跟踪发布文件做原始字节比较：13 个相同、57 个不同、无新增或删除；其中一部分是生成器统一行尾，身份元数据差异则是真实差异。正式覆盖前必须完成下游 pin 影响核对，不能挑文件手改。

## 2. 冲突的准确边界

这不是“C# 还是 Rust 的代码风格选择”，而是两个层级的目标被同时称为 V1/MVP：

| 选择 | 当前文档承诺 | 能证明什么 | 不能证明什么 |
|---|---|---|---|
| **MVP bootstrap profile** | C# `mvp-host` 先承载 A0/A1 语义闭环，Rust Host 后置 | 固定 Tick、事务、复制、预测、断线恢复等语义可行 | Dedicated Server Rust 底层性能/边界/宿主契约已落地 |
| **DS V1 profile** | Rust `LumioServer` 承载准入/连接/会话/传输/WorldSlot，C# Runtime 只做语义调用方 | 定稿 §4 的真实分层与生产宿主路径 | 当前仓库尚无足够 Rust 实现，现有 A1 进度不能直接复用为完成证据 |

若不先选定 profile，`R-00277` 之后的 Server 卡会出现验收口径分裂：同一份跨进程测试既可能被判为“完成 MVP”，也可能被判为“未实现 DS V1”。

## 3. 对完成度的修正

上一份报告的结论仍适用于“架构设计资产”和“垂直切片”两个维度，但 Server 的百分比必须标注分母：

- **架构语义/Governance**：约 **90%**。DS 分层、复制调度、慢客户端、准入顺序和回图条件已补齐；发布门仍未绿。
- **架构发布可消费性**：仍为 **0% green**。`lumio_contract.py validate` 仍报告 Root ABI compiler digest `0aaf61...` 与锁定 `6f51b9...` 不一致；DS 文档提交没有修复生成物身份。
- **Server C# MVP bootstrap**：约 **30% - 40%**，只按现有 `mvp-host` 语义宿主能力计；WorldSlot/Session/App/真实跨进程闭环仍缺。
- **Server DS V1（Rust 核心）**：**<10%**，按定稿 §4 的 Rust 必须模块计；连接/准入/会话/预算/WorldSlot 还没有可消费生产实现。
- **MS-00001 有效垂直切片**：在 profile 未裁决前继续记 **15% - 20%**；不能因为设计定稿增加就上调实现完成度。

## 4. 必须先做的 W0.5 守门

在继续派 Server 卡或把 A1 结果写成里程碑证据前，完成以下一次性决策记录：

1. **确认 MS-00001 的验收 profile**：
   - 若选 bootstrap：把 C# 宿主明确命名为“语义/验收 harness”，把 Rust DS 替换列为后续独立里程碑；A1 报告必须写明“不等同 DS V1”。
   - 若选 DS V1：把 Rust DS 核心加入当前关键路径，重排 `R-00277` 及其后置卡，并补 Rust↔C# 接缝、真实 WS 和 WorldSlot 验收。
2. **登记替换/兼容关系**：不改公共 Schema 的前提下，记录 Host Profile、进程边界、可宣称的验收名称和替换退出条件；如需改变公共字段或状态，走 ADR → Schema/ID → Fixture → generator。
3. **按选定 profile 重算分母和截止风险**：目标日 `2026-10-31` 只有在 bootstrap profile 下仍具可行性；选择 DS V1 后必须重新估算，不得沿用旧排期。
4. **W0 发布门仍先行**：无论选哪条路，都先完成 generator/validate、下游 pin 和契约测试收口；profile 决策不能绕过发布身份问题。

### 推荐方向

若 `MS-00001` 的首要目标仍是 2026-10-31 前证明“两客户端互见方块”，建议采用 **bootstrap profile**，但把边界写成显式的临时验收剖面：

- A0/A1 先验证 GameRuntime 语义和协议闭环；
- C# `mvp-host` 只作为测试/演示宿主，不称为 DS V1；
- Rust DS V1 单列为下一阶段，先实现准入/连接/会话/预算/WorldSlot，再替换同一套 Runtime 接缝；
- 任何对外 Dedicated Server/V1 声称都必须等 Rust profile 的验收通过。

如果 Owner 的目标是“MS-00001 本身就是 DS V1”，则不应按现有 C# Server prompt 继续派活，应先调整 Workflow 卡面、依赖图和目标日期。

## 5. 本附录后的派发顺序

`W0 generator/validate → W0.5 profile 决策 → 下游 pin → Runtime/Server/Voxel/CoreEngine/Client/Game foundation → A0 → A1-alpha`。

W0.5 未完成时：

- 不把 `R-00277` 及后续 Server 交付标为 DS V1 完成；
- 不新增或重开 Workflow 卡；
- 不在 Server 中私造 Rust/C# 双套公共协议；
- 不把本机未跟踪 Runtime 文件计入完成度。

本轮仍未写入 Workflow、未修改 `packages/`、未 push 架构仓。本地工作区新增的文档提交只用于保存盘点证据。
