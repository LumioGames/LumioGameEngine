# platform

> 定义平台包目录、链接方式和平台兼容矩阵。优先级：P1；状态：设计中。

## 负责什么

- 管理 Linux、Windows Server、Desktop、iOS 和 Android 的 Target、SDK 与编译约束。
- 声明静态/动态链接方式、文件命名、库布局和调试符号布局。
- 生成平台可用性矩阵，供 Manifest、Loader 和 Smoke 使用。

## 明确不负责什么

- 不实现 Unity 渲染、Host Profile、网络传输或移动端业务逻辑。
- 不允许上层重复实现平台选择逻辑。
- 不把供应商 SDK 类型泄漏进稳定 ABI。

## 输入与输出

- 输入：`composition` 构建输入、`root-abi` 产物、平台 SDK 和工具链约束。
- 输出：平台 Native 包、导出符号、调试文件、目录布局和链接矩阵。

## 生命周期与失败行为

`Select Target -> Build -> Layout -> Verify -> Package`。Target、链接模式、符号前缀或平台声明不一致时必须失败。

## 验收范围

每个声明平台都必须通过包完整性检查、Loader 预检和 NativeHeadless Smoke；平台不支持或 SDK 缺失必须给出稳定原因。

## 相关文档

- [模块索引](../README.md)
- [Loader](../loader/README.md)
