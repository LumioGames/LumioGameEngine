# 0007 · compose 配置解析选定 `toml` crate（精确锁 `=1.1.4`），不自研 TOML 子集解析器

- 日期:2026-08-29
- 状态:生效

## 背景

规格 §7.4 把 composition 的 CLI 入口钉成 `lumio-core-compose compose --config config/p0/linux-server-x86_64-glibc.compose.toml`——配置文件是 **TOML**,扩展名与路径都在规格里写死,不是本仓可选项。要读它就得有 TOML 解析能力。

而规格 §4 的方案选型表**没有 TOML 解析这一行**:该表覆盖了构建编排、Canonical Serialization、Digest、签名、加载、SBOM、Schema 校验、测试、归档、供应链门禁,唯独没覆盖「怎么读本仓自己的本地配置」。ADR 0006 第 2 条指定了 BuildPlan 的编码载体是 serde + serde_json,其「非目标」一节又明说它不决定 serde/serde_json 之外的依赖。所以 LCE-P0-004 落地时撞上一个真空:引入 `toml` 没有选型记录,而 ADR 0004 第 5 条要求直接依赖一律先登记 `[workspace.dependencies]` 精确版本、卡面也要求「新依赖必须有仓库要求的选型、许可证、锁定与退出记录」。

本 ADR 补这条记录。它只决定**本仓怎么读本地配置**,不触碰任何公共契约。

编号说明:`0005` 由 LCE-ADR-005(P0 Linux 同对象加载)预留,序列暂缺 0005 不表示遗漏——与 ADR 0006 的编号说明同一情形。

## 决策

### 1. 候选与选定

| 候选 | 结论 | 理由 |
| --- | --- | --- |
| **`toml` crate**(toml-rs,`=1.1.4`) | **选定** | Rust 生态事实标准,`serde` 集成使配置直接反序列化到 typed 结构 + `deny_unknown_fields`;MIT OR Apache-2.0,在 `deny.toml` 白名单内;其依赖闭包(`serde_spanned`/`toml_datetime`/`toml_parser`/`toml_writer`/`winnow`)同为宽松许可证。 |
| 自研 TOML 子集解析器 | 否决 | 与规格 §4 对 Canonical Serialization 的结论同源:引号、转义、多行字符串、行内表、数组嵌套的边界极易漂移,手写解析器是漂移与注入的温床。而且「只支持子集」会让一份**合法** TOML 被静默误读——配置错读比配置读不出来危险得多。 |
| 改用 JSON 配置(复用已引入的 serde_json) | 否决 | 能省一个依赖,但要改规格 §7.4 钉死的 CLI 契约与文件名。为省一个宽松许可证依赖去动公共规格,代价方向反了。 |
| `basic-toml` / `toml_edit` 直用 | 否决 | 前者功能子集且维护弱于 `toml`;后者是保格式编辑器,为「读配置」引入不需要的文档模型。 |

### 2. 使用边界

- **只用于读本仓自己的本地配置**:`*.compose.toml` 与 `tools/tools.lock.toml`。
- **不用于任何公共载荷**。BuildPlan / ProvenanceRecord 的字节由 ADR 0006 的确定性 JSON 编码产出,与本依赖无关;跨仓公共载荷的互操作由架构源 CanonicalSerializer 负责。
- **不用于写**:本仓不生成 TOML,因此不依赖其序列化侧行为。
- 配置解析一律 typed 结构 + `deny_unknown_fields`:配置里多一个键就是配置错了,静默忽略会让人以为改生效了。

### 3. 锁定方式与版本线选择

`[workspace.dependencies]` 精确锁 `toml = "=1.1.4"`,crate 侧只能经 `toml.workspace = true` 消费;`Cargo.lock` 提交。升级走常规依赖升级流程(`cargo deny check` + `cargo about` 复核许可证闭包)。

**为什么是 1.x 而不是 0.9.x**:`deny.toml` 的 `bans.multiple-versions = "deny"` 不允许同一 crate 多版本共存,而 `toml 0.9.12` 的依赖树里 `winnow` 会同时出现 `0.7.15`(toml 直接依赖)与 `1.0.4`(经 `toml_parser`)两个版本,实测 `cargo deny check` 报 `bans FAILED`。`toml 1.1.4` 的树只有 `winnow 1.0.4`,四项检查全过。这条约束以后升级 `toml` 时同样成立——**先跑 `cargo deny check` 再决定版本**,不要为了绕过它去加 `[[bans.skip]]`:白名单制的价值就在于不开这种口子。

### 4. 退出路径

配置格式与解析器解耦:`ComposeRequest::from_config_file` 是唯一解析入口,`ComposeConfig` 系列结构是唯一映射层。换解析器只改这一个文件,`ComposeRequest` 与 `compose()` 的签名、以及计划字节都不变。若日后规格放开配置格式,可整体迁 JSON(serde_json 已在依赖图内)而不触碰其余代码。

## 后果

- 依赖闭包多出 `toml` 及其传递依赖;`cargo-deny` 的 `multiple-versions = "deny"` 意味着日后新增依赖若与其传递依赖版本冲突,需要显式收敛——这是白名单制的既定代价,不是本 ADR 新增的。
- 换来的是:配置**读错**这一类故障基本被排除(typed + 未知键拒绝 + 成熟解析器),且本仓不用维护一份「支持到哪算哪」的私有语法。
- 本 ADR 不改变 ADR 0001—0004、0006 的任何边界,也不影响任何公共 Schema、ID、FFI 与错误语义。
