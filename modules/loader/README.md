# loader

> 在一个进程内加载唯一且相容的 CoreEngine Native 包。优先级：P0；状态：设计中。

## 负责什么

- 发现平台包并执行 Manifest、ABI、Capability、Hash、Signature 和符号预检。
- 管理 Loader Registry、单包加载、重复加载拒绝和卸载状态。
- 将加载失败转换为稳定 Error Code 和诊断事件。

## 明确不负责什么

- 不拥有 World、Session、Tick、Release Pool 或业务生命周期。
- 不决定产品语义兼容，不让 Server/Client 各自复制 Native 加载逻辑。

## 输入与输出

- 输入：`platform` 产物、`manifest` 和 `signing` 的验证结果、`root-abi` 契约。
- 输出：已验证的 API Table/加载句柄、能力声明和可关联的失败证据。

## 生命周期与失败行为

建议状态为 `Uninitialized -> Loading -> Ready -> Unloading -> Unloaded`；任意阶段可进入明确 Fault。第二份不兼容包、重复 Worker Pool、包损坏、超时和资源不足必须拒绝。

## 验收范围

覆盖单次加载、重复加载、版本/能力冲突、符号冲突、签名失败、损坏包、卸载和资源回收。

## 相关文档

- [模块索引](../README.md)
- [Root ABI](../root-abi/README.md)
