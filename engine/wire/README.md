# engine/wire — 开发态公共契约

本目录是预上线 Living Architecture 的公共契约落点（Owner 2026-09-01 裁定）：每张冻结卡一份自包含 JSON，**不扩展** `hello-wire-v1.json`，不恢复已删除的 Schema/ID/Fixture/Baseline/七仓镜像体系。

统一校验入口：`node eng/verify-wire.mjs`（自动发现本目录 `*.json`）。自测：`node --test eng/verify-wire.mjs`（驱动已装运校验器，不另写一份语义）。`engine/abi/native-abi.json` 仅在托管/Native 二进制边界变化时改动。

| 文件 | contractId | 用途 | 专有校验 |
| --- | --- | --- | --- |
| [`hello-wire-v1.json`](hello-wire-v1.json) | `lumio.hello-wire.v1` | MS-00002 Hello World 最小 WebSocket 契约 | `node eng/verify-hello-wire.mjs` 与 `node --test eng/verify-hello-wire.mjs` 仍有效且必须继续通过 |
| [`gameplay-command-envelope-v1.json`](gameplay-command-envelope-v1.json) | `lumio.gameplay-envelope.v1` | RM-00011 C-1 通用玩法命令信封 + Chat 映射（ADR-049） | 由 `eng/verify-wire.mjs` 执行内嵌正反例 |
| [`entity-binding-and-query-v1.json`](entity-binding-and-query-v1.json) | `lumio.entity-binding-query.v1` | RM-00011 C-2 连接绑定与 Attribute Query（ADR-053） | 由 `eng/verify-wire.mjs` 做结构/码表/声明级用例 |

hello-wire 仍是 Hello World 消息形状、字段语义、进程边界与审计词表的唯一真值。消费方不得在实现仓另写一份协议真值。本目录契约是开发态最小契约，不是 Baseline；进入正式硬化阶段时再按治理顺序升级为版本化公共合同。

下游实现仓在对应 C 卡合入 architecture `origin/main` 之后拉取 JSON 消费；解析新信封/端口/查询/定时的生产代码不得早于该合并 SHA 出现在各仓 main。
