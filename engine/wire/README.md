# engine/wire — 开发态公共 wire 契约

`hello-wire-v1.json` 是 MS-00002 Hello World 里程碑的最小公共 wire 契约：WebSocket 消息形状、字段语义、进程边界（readiness/shutdown/退出码）与审计事件词表的唯一真值。Rust Server、C# Runtime、Browser、Bot 与 LumioGame 集成验收都消费本文件，不得在任何实现仓另写一份协议真值。

- 校验入口：`node eng/verify-hello-wire.mjs`（结构校验 + 示例哈希核对）。
- 自测（含失败探针）：`node --test eng/verify-hello-wire.mjs`。
- 可复用 API：`loadContract()` / `validateMessage(contract, message)` / `computePayloadSha256(payload)`，供集成启动器与跨仓一致性测试引用。
- 本契约是开发态最小契约，不是 Baseline；进入正式硬化阶段时再按治理顺序升级为版本化公共合同。
