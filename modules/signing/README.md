# signing

> 为 CoreEngine 平台包提供签名、供应链证明、SBOM 和 License 清单。优先级：P1；状态：设计中。

## 负责什么

- 组织 Manifest 与平台包的签名载荷并执行验证。
- 记录信任根、Key Rotation 元数据和验证结果。
- 生成 SBOM、许可证清单和依赖审计证据。

## 明确不负责什么

- 不把生产密钥写入仓库、Prompt 或日志。
- 不实现用户认证、业务权限或产品 Release 路由。
- 不修改 ABI、Manifest 或平台二进制内容。

## 输入与输出

- 输入：Canonical Manifest、平台包、调试产物和依赖清单。
- 输出：签名包、验证结果、SBOM、License 清单和可供 Loader 消费的证明。

## 生命周期与失败行为

`Collect -> Build Payload -> Sign -> Verify -> Publish Evidence`。内容篡改、签名无效、信任根未知、SBOM 缺失或许可证检查失败必须明确拒绝。

## 验收范围

覆盖有效/无效签名、载荷篡改、Key Rotation 元数据、SBOM 完整性和许可证审计；密钥托管由受控外部系统负责。

## 相关文档

- [模块索引](../README.md)
- [Manifest](../manifest/README.md)
