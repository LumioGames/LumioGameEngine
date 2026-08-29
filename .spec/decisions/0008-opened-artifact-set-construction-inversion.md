# 0008 · `OpenedArtifactSet` / `MappedNativeImage` 用构造反转保持私有构造器，feature gate 不用于跨 crate 可见性

- 日期:2026-08-29
- 状态:生效

## 背景

规格 §9.3 有一条规范性要求：

> `OpenedArtifactSet` 生产构造器不公开；Verifier/Loader 只能读取。

它要挡的是一类具体风险：Loader / Verifier 能自己造一个 `OpenedArtifactSet`，就等于绕开了「所有包内字节必须经一次安全打开」这条路径——`openat2(RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS)`（§6.3）那一层被跳过，而类型系统看不出区别。

难点在于 Rust **没有跨 crate 的 friend 可见性**，而这三方分处三个 crate：

- `lumio-core-platform-contracts` 拥有 `OpenedArtifactSet`；
- `lumio-core-platform-runtime` 实现 `LoadBackend`，**必须**能产出该类型；
- `lumio-core-loader` / `lumio-core-runtime-verifier` 消费它，**必须不能**产出。

直觉结论是「所以构造器只能 `pub`」。LCE-P0-007 首次交付就是这么做的，并在类型文档里如实记录了「因此保证不了字节来自安全打开」。审查指出该结论不成立：前提为真，但存在可行的替代解法。本 ADR 记录采纳的解法，以及一条**明确不要走的岔路**——后者比结论本身更容易被下一个人重新踩中。

## 决策

### 1. 采纳构造反转（construction inversion）

后端不再产出成品，只交出零件；成品由 contracts 自己组装：

```rust
pub trait LoadBackend: Send + Sync {
    // 后端实现这两个
    fn open_parts(&self, request: OpenPackageRequest) -> Result<OpenedParts, PlatformRuntimeError>;
    fn map_native_payload(&self, opened: &OpenedArtifactSet, artifact: &PackagePath)
        -> Result<Arc<dyn NativeImagePayload>, PlatformRuntimeError>;

    // 这两个是**默认方法**，签名与规格 §9.3 一字不差
    fn open_package(&self, request: OpenPackageRequest) -> Result<OpenedArtifactSet, PlatformRuntimeError> {
        let (control, artifacts) = self.open_parts(request)?;
        OpenedArtifactSet::from_opened_parts(control, artifacts) // pub(crate)
    }
    fn map_native(&self, opened: &OpenedArtifactSet, artifact: &PackagePath)
        -> Result<Arc<MappedNativeImage>, PlatformRuntimeError> { /* 同理 */ }
}
```

关键机制：**trait 默认方法体在定义它的 crate 内编译**，因此够得着 `pub(crate)` 构造器。于是：

- `OpenedArtifactSet::from_opened_parts` 与 `MappedNativeImage::new` 都是 `pub(crate)`；
- platform-runtime 从不 name 构造器，只填零件；
- Loader / Verifier 既 name 不到构造器，也无法经 trait 绕道——它们不实现 `LoadBackend`；
- 规格 §9.3 的消费面签名完全不变。

默认方法在语法上可被覆盖，但覆盖者造不出返回值（构造器私有），所以覆盖不可行。这是**语言机制消除可见性泄漏**，不是靠纪律约束。

### 2. 明确否决：不用 feature gate 做跨 crate 可见性

「加一个 `backend-impl` feature，只让 platform-runtime 开」看起来与 `test-support` 同构，实际是**假类比**，会给出虚假的安全感：

| | `test-support` | 假想的 `backend-impl` |
| --- | --- | --- |
| 由谁启用 | runtime-verifier 的 **dev**-dependency | platform-runtime 的 **normal** 依赖 |
| resolver v2 行为 | 非测试构建中**不统一** dev-dep 的 feature | 同一 build graph 内**统一** |
| 结果 | loader 的运行时构建里该 feature 关闭 | loader 一并拿到该 feature |

也就是说 feature gate 在这里挡不住任何人。`test-support` 之所以有效，靠的是 dev-dependency 这一特定通道，不是 feature 机制本身。

### 3. 该保证仍然到不了的边界，写进类型文档

构造反转保证的是「只有 `LoadBackend` 实现能产出 `OpenedArtifactSet`」。它**不**保证「这些字节确实来自一次安全打开」——那取决于后端 `open_parts` 的实现质量，本 crate 无从校验。类型文档必须如实写明这条边界，不得表述成「本类型保证字节来自安全打开」。

### 4. 相邻不变量的机器保证

`test-support` 不进运行时闭包这条，由 `justfile` 的 `runtime-deps` recipe 断言（`cargo tree --workspace -e normal` 无 `test-support`；platform-contracts 的 normal 依赖无 OS crate），已接入 `just check`。任何文档若声称某条不变量「由 X 覆盖」，X 必须真实存在且可 grep 到——LCE-P0-007 首次交付正是在这一点上写了一条从未创建的门禁。

## 后果

- `LoadBackend` 的实现者要写四个方法而不是两个；换来的是构造器无需公开。多出的两个是纯粹的零件供给，没有语义负担。
- `MappedNativeImage` 现在持有 `Arc<dyn NativeImagePayload>`：后端把自己的映像状态放进去，Loader 拿到的 `&dyn NativeImagePayload` 上没有任何方法，OS handle 不外泄（规格 §9.1）。
- LCE-P0-014（Linux DynamicLibrary Backend）按本 ADR 实现 `open_parts` / `map_native_payload`，不要去找 `OpenedArtifactSet` 的公开构造器——没有。
- 本 ADR 不改变 ADR 0001—0004 的任何边界，不新增依赖边，也不定义任何公共 Schema / ID / FFI 语义。
