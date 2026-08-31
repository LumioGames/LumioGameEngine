# loader

> 在一个进程内锁定并加载唯一 PackageIdentity 的 CoreEngine Native 包。优先级：P0；状态：设计中。

## 负责什么

- 发现平台包并对**实际打开的文件句柄**执行 Manifest、ABI、Capability、Digest、Signature 和符号预检（消费 `runtime-verifier` 的 VerifiedPackageDescriptor，验证与映射针对同一组句柄，防止 TOCTOU）。
- 首次成功 Acquire 后锁定进程唯一 **PackageIdentity**；同一身份重复 Acquire 为幂等 Lease/引用计数；任何不同身份一律返回稳定 `PackageIdentityConflict`，不做「看起来兼容」判断。
- 输出 **LoaderLease + RootApiTableView**（不是裸 Library Handle）、能力声明和同步 `VerificationResult`。
- 将加载失败转换为稳定 Error Code、诊断事件和 Failure Evidence Fragment。

## 明确不负责什么

- 不拥有 World、Session、Tick、Release Pool 或业务生命周期。
- 不信任构建阶段的离线验证结论：运行时必须重新验证实际打开的包。
- 不编译依赖 Manifest 生成器或 `signer-tool` 的内部实现，只消费公开 Schema 与 Verifier 接口。
- 不决定产品语义兼容，不让 Server/Client 各自复制 Native 加载逻辑。
- 包内数据不得自举为信任根。

## 输入与输出

- 输入：`platform` 包产物与 LoadBackend 契约、`manifest` 包 Schema、`signing` 的 `runtime-verifier` 接口、`root-abi` RootApiTable 契约。
- 输出：LoaderLease、RootApiTableView、能力声明、同步 VerificationResult、失败证据 Fragment。

## 依赖关系

- 消费：`manifest::包 Schema`、`signing::RuntimeVerifier 接口`、`root-abi::RootApiTable 契约`、`platform::LoadBackend 契约`。
- 被消费：Runtime Host Adapter（LoaderLease 与 RootApiTableView）、Host 审计（VerificationResult）。
- StaticLinked 与 DynamicLibrary 两个 Backend 共享同一逻辑状态机与 PackageIdentity 语义。

## 生命周期与失败行为

状态机（设计基线，公共冻结见架构源 ADR-019）：

```text
Uninitialized -> Preflighting -> Verified -> Binding -> ApiReady -> Leased
任意瞬态状态 -> FailedRolledBack
Leased -> Quiescing -> Released
```

- 并发 Acquire：同一身份只映射一次，幂等返回同一 Lease；不同身份稳定拒绝。
- Timeout、Cancel、OOM、Partial Load 必须完整回滚，不留下半初始化 Registry、残留符号或文件句柄。
- Loader 在规定 Host 线程初始化；Native Worker 不回调 Managed 热路径。
- **Design Requirement**（架构源 ADR-019 已冻结）：V1 采用 No-Physical-Unload——`Released` 释放 Lease 与 API Table 视图，但进程生命周期内不物理卸载映像；物理卸载留待 V2 以新 ADR 取代，届时必须定义 Host Quiescence、Handle Drain 和 Worker Join。

## 验收范围

覆盖单次/并发/重复 Acquire、不同 PackageIdentity 拒绝、符号缺失与冲突、签名失败、损坏包、Timeout/OOM/Partial 回滚、Static/Dynamic Backend 行为一致性和资源回收。

## 相关文档

- [模块索引](../README.md)
- [Root ABI](../root-abi/README.md)
- [Signing](../signing/README.md)
