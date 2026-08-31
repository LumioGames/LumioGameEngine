# 配置表管线外部技术调研

**主题：策划配表的权威源 → 编译产物 → 运行时读取**  
**目标画像：Rust 原生内核 × C# Gameplay × 浏览器 .NET WASM × Dedicated Server × 确定性 / Replay / Revision 原子激活**  
**调研日期：2026-08-29**  
**交付版本：Research-1.0**

---

## 信息源可达性声明

- **联网状态：可联网。** 本次实际访问了 RFC、W3C/Unicode、Apache、SQLite、Microsoft、Google、Epic、Unity、Godot、OpenAI、OWASP、TUF/Sigstore/SLSA 等官方规范或官方文档，也访问了 Luban、xresloader、MasterMemory、CastleDB、Tableau、sql.js-httpvfs 等公开仓库的 README/文档页面。
- **规范原文：大部分可读。** JSON/JCS、CBOR、Protobuf、FlatBuffers、Cap'n Proto、Avro、Arrow、Parquet、SQLite、Zstandard、LZ4、Brotli 等关键规范或官方说明均读到主体；IEEE 754 标准入口可达，但完整标准正文受访问条件限制，因此涉及其细节时不把未读正文部分标作 `Verified`。
- **源码级证据：受匿名 GitHub 页面限制。** 遵守“不 clone、不下载整仓”要求，仅用在线页面检索。公开 README、文档与部分文件内容可读，但 GitHub 动态页面没有稳定暴露所有仓库的 commit hash 与行号。凡未取得 `owner/repo@commit:path#Lx-Ly` 永久坐标的内部实现细节，本报告**不冒充源码级 Verified**，而降为 `Reported`；官方 README 中明确承诺的能力只作为“项目公开能力声明”。
- **中文与英文资料：均覆盖。** 中文侧重点覆盖 Luban、xresloader、KSFramework、作者技术文章和中文使用记述；英文侧覆盖格式规范、商业引擎文档、数据库/列存生态和运行时文档。
- **未访问委托方代码库。** 本报告只依据任务书给出的自包含目标画像判断可迁移性，不推定任何未提供的仓库实现。

## 置信度图例

- **`Verified`**：本次亲自读到规范原文、官方文档、论文全文或可核的官方仓库说明；若声称源码内部行为，则必须有稳定源码坐标，否则不使用此等级。
- **`Reported`**：官方项目 README、维护者文章、多个可信来源或社区记述一致，但未核到稳定源码坐标或未取得完整测试条件。
- **`Estimated`**：本报告基于已核机制做出的工程推演、成本模型或建议。每处说明推断依据，不能当作实测数字。

> 矩阵内使用 `[V] / [R] / [E]` 分别表示 `Verified / Reported / Estimated`。

## 版本与方法说明

1. 格式规范尽量写明本次依据版本：JSON RFC 8259、JCS RFC 8785、CBOR RFC 8949、BSON 1.1、Avro 1.12.0、Thrift IDL 0.25.0、Arrow Columnar Format 1.5（网站当前规范快照）、TOML 1.1.0、YAML 1.2.2、bincode 2.0.1 等。
2. 工具项目若公开页面无法可靠解析当前 release，则写“2026-08-29 文档快照”，不编造版本。可核版本包括 CastleDB 1.5、MasterMemory NuGet 3.0.4；但报告不把一个包版本等同于所有生态组件版本。
3. 性能数字只在来源给出条件时转述；没有条件时采用公式、变量和区间，标 `Estimated`。本报告的所有内存数字均是模型，不是目标引擎实测。
4. 评估顺序是：先固定**逻辑语义**，再比较**物理编码**，最后把**网络、缓存、快照、热更**放进同一生命周期验证。单独比较“序列化速度”不足以决定配表系统。

---

# 执行摘要

1. **权威源不应等同于 Excel 文件。** `Verified`：Excel 会自动去掉前导零、对超过 15 位的数字丢失精度、把形似日期的字符串改成日期；相关行为有微软官方文档与基因符号被改写的论文事故支撑。[S072–S076,S112] 结论是：Excel/Sheets 可以继续作为策划工作台，但发布权威面应是“独立 schema + canonical typed text/IR + 可审计提交”，或者至少是由 Excel 单向编译出的文本镜像，并由 CI 阻止两者分叉。
2. **“JSON 先行、二进制后上”可行，但前提不是换解析器，而是第一天冻结逻辑 IR。** `Verified`：Protobuf 官方明确说明 deterministic serialization 并不等于 canonical serialization；FlatBuffers、SQLite、Parquet 也都允许同一逻辑数据对应不同合法物理字节。[S018,S020,S037,S039] 因而 Hash、Revision、默认值、空值、行序、字符串归一化和浮点规则必须独立于物理格式。
3. **本画像最合适的最终形态是“小 manifest + 内容寻址不可变 chunk + 可替换 reader”，而不是一个巨大二进制。** `Estimated`：manifest 固定表/分片索引与投影 Hash；chunk 以表级为默认、超大表再分片，独立压缩并按内容 Hash 复用。这样才能同时满足 WASM 按需下载、Active/Staged 共享未变数据、Tick Barrier 原子切换和旧 Revision 延迟释放。
4. **默认懒加载粒度应是“表级，超阈值后分片级”，不是网络行级。** `Verified`：HTTP Range 可以返回局部字节；sql.js-httpvfs 证明浏览器对静态 SQLite 文件做 Range 读取可行，但该项目也明确指出：无合适索引的全扫描会拉取大量文件，覆盖索引和请求分块至关重要，而且项目本身缺少完整驱逐和测试。[S042,S085] `Estimated`：行级网络加载会把 RTT、请求放大、缓存一致性和首次访问抖动引入 gameplay 热路径，只适合极少数长文本/媒体 sidecar。
5. **懒加载与不可变快照并不冲突，前提是“快照是不可变命名空间，不是已 materialize 对象全集”。** `Estimated`：Active/Staged 各自持有不可变 manifest 根；异步加载请求捕获 Revision/Generation，完成后只能填充对应代际的缓存；Tick Barrier 只原子替换根指针。未变 chunk 按内容 Hash 共享，已取得的旧代句柄继续钉住旧代，直到 epoch/refcount 安全释放。
6. **最终格式不宜直接选 Parquet、rkyv 或 Protobuf 单押。** `Verified/Estimated`：Parquet擅长列裁剪与批量扫描，不擅长“按 ID 取一行的多个字段”；rkyv 的优势集中在 Rust，跨 C# 共享会丢掉生态优势；Protobuf成熟且适合作为 schema/交换层，却需要反序列化且原始字节不可作为长期 canonical Hash。[S018,S028–S038] 当前最值得进入实测决赛的是：`FlatBuffers + 外置索引/分块`、`自研 typed binary + 生成双语言 view`、`SQLite 只读/Range 后端`三类；JSON/JCS作为首期与诊断基线。
7. **内存真正的敌人是对象图，不只是文件体积。** `Verified`：.NET 字符串使用 UTF-16；大对象进入 LOH，LOH 与 Gen2 回收耦合；对象布局与数组/引用带来固定开销。[S094–S097] `Estimated`：在本文固定假设下，100 万行 × 20 列若做“通用 object[] + 装箱”，常驻约 668–1049 MiB；强类型行对象约 219–343 MiB；结构体数组或零拷贝 view 约 117–153 MiB。必须在 CoreCLR、.NET WASM、IL2CPP 三个发布构建分别验证。
8. **Server / Client / Voxel 应从同一源编译成三个投影，不应让客户端验证它不持有的服务器私有字节。** `Estimated`：Release Manifest 同时携带 `SourceRootHash`、每端 `ProjectionRootHash` 与共享公共子集 Hash；握手比较该端应持有的投影和公共契约，而不是要求客户端计算全服务器 Hash。跨投影引用默认编译错误，确有需要时只能降级为显式 opaque ID。
9. **AI 配表的公开工业证据很薄；当前瓶颈主要是接口、校验和审计，不是“模型会不会填单元格”。** `Verified`：结构化输出和工具调用可以按 JSON Schema 约束形状，但仍不保证业务语义正确，且 schema 有规模/关键字子集限制。[S103,S104] `Estimated`：应暴露 `query → propose_patch → validate → compile_preview → semantic_diff → simulate` 工具闭环；AI只提交 typed patch，不能直接激活生产配置，所有改动带 before/after Hash、actor、原因和审批者。
10. **第一天必须定死的不是二进制布局，而是不可逆语义。** 包括稳定 ID 与永不复用、missing/empty/null/default 四者的规则、schema 字段 ordinal、行排序、字符串 UTF-8 与归一化、数值范围/溢出、浮点/定点策略、引用图、分端投影、逻辑 canonical Hash、Revision 句柄语义、机器可读错误格式。物理 chunk 大小、压缩算法、具体索引实现和缓存容量可以后测后改。

## 已知缺口清单

- **公开商业项目规模数据非常少。** 没有找到可核、同时给“表数/行数/文件体积/冷启动/峰值内存/硬件与运行时版本”的完整游戏案例；O 章因此以可执行基准矩阵代替拍脑袋阈值。
- **没有找到成熟的“AI 直接配生产游戏配置”公开案例。** 仅找到零星实验项目与相邻领域的结构化输出、工具调用、静态分析结果规范；M 章把方案明确标为 `Estimated`。
- **没有核到大量团队从 Excel 权威源迁走后的完整复盘。** CastleDB/Luban 的文本化与双向视图能力证明需求真实，但“迁移后是否后悔”的公开、可核长期数据不足。
- **GitHub commit permalink 不完整。** 匿名页面可读 README/文档，却未稳定得到所有关键源码文件的 commit hash 与行号；因此源码内部实现细节整体降级，未按要求假装 `Verified`。
- **.NET WASM × SQLite/FlatBuffers/Parquet 的真实包体、AOT、冷启动与内存数据必须本地实测。** 文档能证明“机制可用”，不能证明“在委托方目标预算内可用”。
- **浮点跨端一致性的业务容忍度未知。** 报告能给编码和规避方案，但无法替委托方决定哪些列必须定点化；需要对技能公式、掉落概率、物理/光照参数做领域分级。

---

# A. 谱系与全景：配表管线的工业界地图

**结论先行 1：** 配表系统的演进不是“文本 → 二进制”的单线升级，而是把编辑体验、schema、验证、运行时布局和发布生命周期逐步拆开。  
**结论先行 2：** 游戏行业至今同时存在五派；不存在一种格式覆盖策划协作、WASM懒加载、Rust/C#零拷贝和语义diff。  
**结论先行 3：** 中文生态在“Excel→多语言代码+二进制”这一派公开工具密度更高，但公开商业采用与规模数据仍然稀缺。

## A.1 形态演进

### 阶段 1：CSV / INI / Lua / 文本直读

`Verified`：CSV 的标准核心只规定记录、字段、引号等交换语法，并不自带列类型、引用、默认值或 schema 演进。[S007] 早期或小型项目把配置文本直接打进包，优点是极低工具投入、易 diff；缺点是运行期才发现类型/引用错误，程序重复写解析代码。中文 KSFramework 文档直接把实践概括为“Excel 编译式”和“策划直接编辑 Lua/TSV”两类，并指出自动生成读取代码可避免手写维护。[S058]

### 阶段 2：Excel / Sheets + 转换器

`Reported`：这一阶段以策划熟悉的表格为工作面，由脚本导出 CSV、Lua、JSON 或自定义二进制。Luban、xresloader、Tableau 等公开工具延续并系统化了此路线。[S048–S052,S056] 它解决“策划不写代码”，但引入二进制源文件的 diff/merge、自动类型转换、公式缓存和 CI 环境一致性问题。

### 阶段 3：Schema 驱动 + 代码生成 + 多端导出

`Verified/Reported`：Protobuf、FlatBuffers、Avro 等证明 schema 可以成为跨语言生成与演进的稳定中心；Luban、xresloader、Tableau把这一思路引入游戏配表，公开声明支持类型、引用验证、多语言生成、多格式输出和分端分组。[S014–S23,S048–S056] 此阶段的关键变化是：Excel 不再决定运行时类型，schema 才决定。

### 阶段 4：引擎内建资产与可寻址资源

`Verified`：Unreal DataTable要求行结构；Data Registry在多数据源之上提供只读访问、同步/异步获取和缓存规则；Unity ScriptableObject把数据纳入资产系统，Addressables通过 key/label异步加载并要求显式释放；Godot ResourceLoader有缓存和线程加载接口。[S061–S071] 这派把“配置”当资产，复用编辑器、依赖图、打包和加载系统，但牺牲跨引擎/跨语言独立性。

### 阶段 5：数据库化、列式化、服务化与 AI 工具化

`Verified/Estimated`：SQLite把索引、查询、页缓存和只读文件打包在一个成熟容器；Arrow/Parquet提供列裁剪、record batch/row group/page和字典/RLE/delta编码；Google Sheets API与结构化工具调用使远端协作和AI代理可编程。[S033–S043,S079–S081,S103–S105] 这些不是必然替代传统配表，而是为超大表、浏览器按需、分析/批处理或自动化提供可借鉴机制。

## A.2 五派分野

| 流派 | 为何存在 | 主动放弃 | 规模上限由什么决定 | 代表实现 |
|---|---|---|---|---|
| 表格源 + 转换器 | 保留策划熟悉的 Excel/Sheets，同时生成强类型数据与代码 | 原生文本 diff、无冲突合并、简单构建链 | 表格协作、编译时间、生成物/对象内存 | Luban、xresloader、Tableau、GameFrameX 集成 [S048–S057] |
| 引擎内建资产 | 复用编辑器、依赖、异步加载、打包与热更 | 跨引擎中立、Rust/C#同字节、独立CI | 资产依赖图、bundle粒度、引擎序列化限制 | Unreal DataTable/Data Registry、Unity ScriptableObject/Addressables、Godot Resource [S061–S071] |
| 文本 DSL / 声明式源 | Git diff/merge、脚本/AI直接改、可复制构建 | Excel公式和自由表格编辑体验 | schema表达力、编辑器体验、文本解析/文件数量 | CastleDB JSON-lines、JSON5/TOML/YAML、Luban文本源 [S004–S006,S048,S054] |
| 嵌入式数据库 | 需要索引、条件查询、页缓存、局部读取和成熟工具 | 纯零拷贝、最小运行库、直接语义diff | page/request模型、查询模式、WASM桥接 | SQLite、sql.js-httpvfs [S039–S043] |
| 列存 / 分析格式 | 大量列扫描、压缩、列裁剪、批处理 | 单行跨列热点的局部性与简单点查 | row group/page配置、codec、reader包体 | Arrow IPC、Parquet [S033–S038] |

## A.3 中文游戏工业界工具生态

### Luban

`Reported`：官方中文 README 与文档声明支持 Excel/JSON/XML/YAML/Lua 等源、复杂类型/容器/多态/可空、客户端/服务器字段分组、多语言生成、binary/Protobuf/MessagePack/FlatBuffers等产物、引用/范围/资源校验、增量与 watch，以及 Excel2TextDiff/LubanAssistant 一类协作设施。[S048,S049] 维护状态从 2026-08-29 可访问的仓库与文档看仍活跃，但本报告未取得可靠 release 版本号和 commit permalink；不引用“300ms”等无完整条件数字作为决策依据。

### xresloader

`Reported`：官方文档把它定义为面向游戏团队的 Excel 转表工具链，支持 Protobuf、JSON、MsgPack、Lua、JavaScript、XML、UE DataTable等输出，并有批量配置、GUI/CLI、读表代码生成和二进制 dump 生态。[S050–S052,S060] 它的代表性价值是把 Protobuf schema、Excel映射、代码生成和调试 dump 组合起来；公开命名产品和规模数据不足。

### Tableau

`Reported`：`tableauio/tableau`公开定位为基于 Protobuf schema，把 Excel/CSV/XML/YAML转为JSON/Text/Binary并生成多语言代码。[S056] 它代表“Protobuf 先定义 schema、表格只是数据输入”的路线。维护活跃度需要下一轮通过 release/commit历史专项核对。

### MasterMemory

`Reported`：MasterMemory公开 README描述为 source-generator based、typed、read-only、in-memory数据库，具主/二级/范围索引、验证器、string interning和MessagePack构建链；NuGet可核到3.0.4，Unity最低版本公开为2022.3.12f1以支持增量source generator。[S053] 它对 C# 访问API和索引设计很有参考价值，但不解决 Rust同字节、WASM网络懒加载。

### CastleDB

`Verified/Reported`：CastleDB官网当前公开版本1.5；其仓库说明强调用带换行的JSON保存，以便Git/SVN diff/merge，本地实验后再提交。[S054,S055] DBDB索引显示项目2026仍有提交，但该第三方索引只作为活跃度旁证；商业采用常被社区关联到 Evoland 2，未找到一手产品复盘，因此不升级为Verified。

### 其它与维护判断

`Reported`：KSFramework等中文文档证明“Excel编译式/文本直写式”的二分长期存在；GameFrameX公开提供Luban集成，说明工具有框架层采用，但不能由此推断商业产品规模。[S057–S060] 中文博客常给具体命令或性能声称；没有版本、硬件、数据规模和源码坐标者，本报告只作线索，不作为格式决策证据。

## A.4 本章结论

`Estimated`：本画像不应照抄单一流派，而应组合三层：

1. **编辑层**借鉴表格源+转换器或专用编辑器；
2. **契约与编译层**借鉴 schema驱动、多端投影、验证和代码生成；
3. **运行层**借鉴引擎资产的异步生命周期、SQLite/Parquet的索引与块设计，但使用适合Rust/C#/WASM的不可变chunk容器。

### 来源

[S004–S008, S014–S043, S048–S071, S117–S119]

---

# B. 权威源形态：Excel 到底该不该当权威源

**结论先行 1：** Excel作为“策划工作台”仍是主流形态之一；作为“唯一不可变权威源”主要是历史惯性与组织便利，而不是数据可靠性优势。  
**结论先行 2：** 能长期活下去的缓解路线都把 schema、验证、canonical导出和版本审计从工作簿中抽离。  
**结论先行 3：** 对本画像，推荐“typed text/IR 为发布权威，Excel/Sheets 为受控视图”；若第一期必须Excel权威，则强制同提交文本镜像与CI单向再生。

## B.1 Excel 的真实工程代价

### 版本控制与冲突

`Verified`：Git LFS提供锁定能力，GitLab也提供文件锁，但锁的本质是把并行合并退化为串行占用，并未让`.xlsx`变成可语义三方合并的文本。[S077,S078] Git attributes/textconv可以把二进制转换成文本供diff，却不能把文本diff安全地反向合并回复杂工作簿。[S134,S135]

**事故模型：** 两名策划分别改同一工作簿不同 sheet/行，Git只看到一个二进制blob；合并时通常只能择一版本，再人工重放另一方改动。表越大、跨sheet公式越多，重放成本和漏改风险越高。`Estimated`：锁表可在小团队稳定运行，但一张“全项目总表”会形成排队热点，征兆是复制临时工作簿、线下传文件、长期大锁和“最后一个人覆盖”。

### 类型保真事故清单

1. **前导零丢失。** `Verified`：微软明确说明Excel默认把数字型输入去掉前导零；需要文本格式、撇号或自定义格式才能保留。[S072]
2. **长整数精度损失。** `Verified`：Excel数值精度为15位，超出后末位会变成0；这对账号ID、64位内容ID、哈希片段是破坏性改写。[S073]
3. **科学计数法与显示/存储分离。** `Verified`：大数字可能以科学计数法显示；自定义显示格式并不改变底层已损失的精度。[S072,S073]
4. **日期自动转换。** `Verified`：Excel有1900/1904日期系统；形似日期的字符串会被解释为日期。基因组论文发现大量基因符号在Excel中被改成日期或浮点表示，2021研究还覆盖Google Sheets。[S074–S076]
5. **区域设置。** `Verified/Estimated`：Excel和表格API的“用户输入”路径会按类似UI方式解释数值/日期；Google Sheets API明确区分 `RAW` 与 `USER_ENTERED`，后者可能解析为数字、日期、公式。[S081] 因而小数点、千分位和日期解析必须在导出器使用Invariant规则，不能相信显示文本。
6. **自动更正/标识符改写。** `Reported`：科研事故说明自动语义转换能悄悄污染标识符；游戏中的`MAR1`、`SEPT2`、`1-2`、长SKU、带前导零地图ID具有同型风险。[S075,S076]
7. **浮点显示值不等于二进制值。** `Verified`：Excel存储数值而显示格式可截断小数；导出器若读取显示文本和读取原始数值会得到不同输入。具体库行为必须固定并测试。

### 公式与跨表引用

`Verified`：openpyxl等库不计算公式，只能读取公式文本或工作簿已有的cached result；若CI机器没有重新计算工作簿，cached value可能过期。[S084] 因而“产物是否采用公式结果”必须成为显式策略：

- **禁止公式进入发布数据**：公式只作为编辑辅助，CI重算后把值冻结并校验；
- **允许公式但编译器负责求值**：需要实现Excel公式语义，成本极高；
- **依赖Excel/Office重算**：构建环境、版本、区域设置和授权成为供应链依赖。

`Estimated`：本画像应选择第一种。编译器只接受值，不把Excel公式引擎纳入确定性边界；同时保存公式文本作审计，检查cached result与规范化值一致。

### 大表体验与多人协作

公开资料很少给“几万行打开/保存时间+硬件+Office版本”的完整数据，不能编阈值。`Verified`：Excel/Microsoft 365和Google Sheets提供共同编辑、版本历史与API，但这解决的是协作与恢复，不自动解决schema、类型保真和可复现CI。[S079–S083,S107]

`Estimated`：实际拐点不只看行数，而看：公式依赖图、条件格式、图片、VLOOKUP/XLOOKUP、跨工作簿链接、宏、多人热点和保存blob大小。早期征兆是保存/重算超过团队容忍、表被按人而不是按领域拆分、公式链断裂只在某台电脑复现。

### 批量重构

`Estimated`：改列名、枚举值或ID前缀涉及几十张表时，手改不可审计。工业界可行手段是：

1. schema迁移脚本生成typed patch；
2. 编译器全图解析引用并给出影响清单；
3. 文本镜像上执行AST/结构化重写；
4. Excel仅作为导入/预览，重构不直接操作单元格坐标。

Luban公开的多源、类型、ref与插件能力说明这类“先解析到统一模型再生成”是现实路线，但具体迁移API未核源码，不把其细节标Verified。[S048,S049]

## B.2 主流缓解手段评估

| 路线 | 谁在用/证据 | 收益 | 代价 | 何时退化 |
|---|---|---|---|---|
| `.xlsx → CSV/TSV` 入库 | 传统导表、KSFramework描述 [S058] | 文本diff、简单运行时 | 丢工作簿结构/公式/格式；CSV无类型 | 复杂容器、多语言、跨表引用增长时 |
| Excel源 + canonical文本镜像 | Luban Excel2TextDiff、通用textconv [S048,S134,S135] | 保留策划体验，review可读 | 双权威风险；必须CI验证镜像可再生 | 镜像可手改、导出非确定、重算环境不一致时 |
| 文本源 + Excel视图/插件 | LubanAssistant、CastleDB JSON [S048,S054] | 真正可merge、脚本/AI友好 | 编辑器/导入导出开发；策划习惯迁移 | 视图无法无损表达复杂结构或双向同步无冲突模型时 |
| 独立 schema | Luban、xresloader、Tableau、Protobuf [S014–S017,S048–S056] | 类型、代码生成、跨端一致、迁移可审 | 程序需维护契约；schema变更有流程 | schema只写类型、不写presence/ID/业务约束时 |
| 按模块拆表 | 多数导表工具支持多文件/分组 [S048–S050] | 缩小锁与增量编译范围 | 跨表引用和发布原子性更复杂 | 按组织而非访问/所有权边界拆，形成循环引用时 |
| Google Sheets | 官方API/协作/版本历史 [S079–S081,S107,S113] | 实时协作、API、权限 | quota、网络/合规、RAW/USER_ENTERED差异、CI外部依赖 | 构建依赖在线可用性或单元格级API请求爆炸时 |
| 专用编辑器 | CastleDB、引擎资产编辑器 [S054,S061–S071] | schema-aware UX、语义校验、领域控件 | 高投入、需培训和长期维护 | 编辑器跟不上策划需求，用户回流Excel并形成旁路时 |

## B.3 Google Sheets / 在线表格路线

`Verified`：Google Sheets API有项目/用户维度配额；官方当前文档列出读写请求的分钟级quota，并建议批量/指数退避。[S079] Values API与BatchUpdate可批量读写；`RAW`按原值保存，`USER_ENTERED`按UI规则解析。[S080,S081,S113]

**可行发布模型：**

- Sheets是协作工作区；Webhook/定时CI拉取一个不可变revision；
- 拉取时使用batch API，记录spreadsheet ID、sheet ID、revision/时间、导出Hash；
- 将所有单元格规范化到typed IR；
- 只有Git中的canonical输出与schema进入Release构建；
- API失败时使用最后一个已签名revision，不允许“部分sheet更新”。

**风险：** quota只是显性限制，更重要的是外部服务可用性、账号权限、审计保留、数据驻留与离线构建。`Estimated`：如果Release构建必须实时访问Sheets，供应链不可复现；应把一次成功拉取固化为内容寻址输入。

## B.4 专用配表编辑器路线

`Verified`：CastleDB把结构化数据保存为带换行JSON并强调本地diff/merge；Unreal/Unity/Godot则把数据嵌入资产编辑器与加载系统。[S054,S061–S071] 专用编辑器的价值不是“表格皮肤”，而是：

- 列/字段控件知道类型、范围、枚举和引用；
- 能在保存前做局部校验和跨表跳转；
- 能呈现语义diff、来源层和最终覆盖值；
- 能隐藏服务器私有字段与高危操作。

`Estimated`：投入产出比取决于表复杂度和团队寿命。几十张扁平数值表不值得全自研；容器、多态、资源选择、曲线、行为树和多端可见性增长后，Excel里会出现分隔符字符串、颜色约定和隐藏公式，此时专用编辑器收益反而上升。

## B.5 文本源路线

`Verified`：CastleDB明确用JSON+换行支持RCS diff/merge；Luban公开支持JSON/XML/YAML/Lua等源并提供Excel插件在文本与Excel间转换。[S048,S054] 这证明“文本为真、Excel为视图”不是理论路线。

**文本源的约束：**

- 不应直接选自由YAML作为唯一权威，因为隐式标量、锚点和多种合法表示会扩大canonical面；
- JSON5适合人编辑，但发布前必须编译到严格typed IR；
- 大表不宜一个巨型JSON数组，应按表/分片或稳定行文件组织，避免每次改一行重写全文件；
- schema必须独立，否则“可diff”只把类型错误从二进制隐藏变成文本隐藏。

## B.6 三个关键判断题

### 1. “Excel当权威源”是主流、历史惯性还是已被放弃？

**明确结论：是“仍广泛存在的主流工作面 + 强历史惯性”，不是已被行业整体放弃；但越来越多成熟工具把真正契约移到schema与编译器。** `Reported`：Luban、xresloader的持续维护说明Excel入口仍有强需求；CastleDB和LubanAssistant/Excel2TextDiff又说明协作、文本化需求同样真实。[S048–S055]

### 2. 有没有团队公开记述从Excel权威源迁走？

**明确结论：找到了工具路线和动机，没找到足够的一手长期迁移复盘。** CastleDB以文本JSON为中心，Luban支持文本源+Excel视图；但“某大型商业团队迁移前后指标、后悔与否”的可核公开材料不足。此项列为Known gap，不以零星博客包装成共识。

### 3. “文本源+Excel视图”与“Excel源+文本镜像”，哪条活得更久？

**明确结论：工程寿命上，文本源+Excel视图更稳；组织迁移成本上，Excel源+文本镜像更容易落地。** `Estimated`：前者只有一个可合并权威，适合AI、批量重构和CI；后者若严格单向生成、镜像不可手改，也能长期运行，但一旦允许双向自由修改就形成split-brain。分歧点不在格式，而在“谁拥有最终写权限”。

### 来源

[S004–S008, S048–S060, S072–S084, S107, S112–S119, S134–S137]

---
# C. 中间格式与运行时格式选型

**结论先行 1：** 中间格式和运行时格式应分离；前者优化可审计与迁移，后者优化加载、内存和访问。把二者绑死会让“JSON→二进制”变成契约重写。  
**结论先行 2：** 没有现成格式同时天然满足 Rust/C#双读、WASM按需、行ID点查、列裁剪、canonical Hash和热更共享。  
**结论先行 3：** 应以 canonical typed IR 为中心，先让JSON/JCS后端跑通，再让 FlatBuffers、自研typed binary和SQLite/Range三条候选在同一基准上决赛。

## C.1 中间格式与运行时格式不是同一个概念

### 中间格式（Intermediate Representation, IR）

中间格式服务于编译器、校验器、diff、迁移、AI工具和审计。它需要：

- 能无损表达类型、presence、默认值来源、引用和source location；
- 对行/列有稳定逻辑顺序；
- 能产生canonical逻辑字节或canonical事件流；
- 可从Excel、Sheets、JSON5、专用编辑器等多源导入；
- 不携带目标端对象布局、指针宽度或压缩块细节。

`Verified`：Protobuf的字段presence、Avro的writer/reader schema resolution、CSVW的外置metadata都说明“输入表格”和“类型契约”可以分离。[S008,S015,S032] `Estimated`：本画像的IR可以是内部typed AST，磁盘调试镜像用canonical JSON；不要把JSON对象图本身当唯一内存模型。

### 运行时格式（Runtime Artifact）

运行时格式服务于目标平台：

- 启动只读manifest/index；
- 以表/分片/sidecar为chunk；
- 支持边界校验、相对offset、共享字符串池；
- 压缩块与索引边界一致；
- 按端投影；
- 允许不同物理编码共享一个逻辑Hash。

### 分开的收益

1. **格式可替换。** JSON、FlatBuffers、自研binary或SQLite都实现同一个typed table view。
2. **Hash稳定。** 物理重排、压缩级别、page size、builder版本变化不改变逻辑root hash。
3. **诊断可读。** 二进制产物始终能dump为统一canonical/pretty文本。
4. **迁移可测试。** 同一IR同时输出旧/新格式，逐表比较语义Hash和查询结果。
5. **AI可控。** AI修改的是schema-constrained patch，不直接操纵压缩/offset字节。

`Verified`：Protobuf官方明确警告其deterministic bytes不是跨语言/跨版本canonical；因此把wire bytes直接作为长期配置身份会违反上述分离。[S018]

## C.2 候选格式逐类剖析

### 自描述文本：JSON / JSON5 / YAML / TOML / CSV

- **JSON**：跨语言最稳、诊断生态最好；缺点是数字/字符串解析、字段名重复、对象materialize和无原生主键索引。JCS能定义canonical JSON，但只解决编码层的一部分；Unicode归一化、schema default和行排序仍需应用层规范。[S001–S003]
- **JSON5**：适合人写源文件，注释、尾逗号和宽松数字提高编辑体验；不应直接作为机器发布canonical层。[S004]
- **YAML**：表达力强，但隐式类型、锚点/别名和实现差异扩大确定性面；适合小型工具配置，不推荐百万行运行时表。[S005]
- **TOML**：清晰适合工程配置，宽表和大量重复行不友好。[S006]
- **CSV/TSV**：文本diff与流式处理优秀；无schema/容器/引用，需要CSVW或独立schema补齐。[S007,S008]

**本画像判断：** JSON/JCS适合作为Phase 1运行时与永续诊断格式；JSON5/TOML适合人类源；CSV适合Excel镜像和批量导入，不适合作为最终统一语义。

### 自描述二进制：MessagePack / CBOR / BSON

- **MessagePack**：跨语言广，C#有source generator/AOT路径，编码紧凑；默认map顺序和整数宽度不构成canonical，且无业务索引。[S009,S010]
- **CBOR**：RFC 8949定义deterministic encoding模式，是自描述二进制中canonical基础较强者；仍需固定tag、Unicode、浮点和schema语义。[S011]
- **BSON**：文档长度便于跳过，但键名重复与MongoDB导向使其在包体敏感游戏配置上优势不明显。[S013]

**本画像判断：** MessagePack/CBOR可作中期紧凑交换或单表chunk编码；若仍要外建索引、投影、字符串池和chunk容器，最终复杂度接近自研容器，却不获得真正零拷贝。

### Schema驱动：Protobuf / Thrift / Avro

- **Protobuf**：多语言与演进纪律最成熟；tag不可复用、presence必须显式。缺点是反序列化对象、repeated rows线性点查，以及非canonical bytes。[S014–S018]
- **Thrift**：IDL与字段ID提供兼容基础，强项在RPC而非静态表点查。[S031]
- **Avro**：writer/reader schema resolution适合数据演进与批处理，object container可分block；游戏运行时生态和AOT/WASM形态较弱。[S032]

**本画像判断：** Protobuf非常适合schema权威、工具交换、测试向量或中间产物；若用作最终运行时，必须增加离线主键索引、按表/行block和对象内存控制。不能用序列化字节直接做长期Config Hash。

### 零拷贝：FlatBuffers / Cap'n Proto / rkyv

- **FlatBuffers**：relative offset、little-endian、按字段访问、官方Rust/C#生成器，是跨语言候选中最贴近画像者。[S019–S023] 但vector只按位置O(1)，业务ID仍需排序/索引；压缩块必须先解压；不同builder布局不应直接当canonical。
- **Cap'n Proto**：消息布局和canonical form有规范，读取低成本；C++是参考实现，Rust/C#是独立语言实现，生态一致性与AOT需专项验证。[S024–S027]
- **rkyv**：Rust归档view和bytecheck强，但格式控制与Rust类型绑定，缺乏官方C# reader；本画像若为共享格式，会承担重复协议实现和unsafe审计。[S028,S029]

**本画像判断：** FlatBuffers进入决赛；Cap'n Proto作为研究对照；rkyv仅适合Rust私有Voxel产物，而非默认共用产物。若Voxel最终允许独立artifact，rkyv可另做spike，但会增加双格式一致性成本。

### 紧凑Rust编码：bincode

`Verified`：bincode 2.0.1规范定义整数编码和配置，但它主要面向Rust类型序列化，不提供跨语言schema与业务索引。[S030] `Estimated`：若C#再实现一套reader，等于自研协议却缺少IDL/codegen；不推荐作为共享主格式。

### 列式：Arrow IPC / Parquet / Feather

- **Arrow**：规范化内存列布局，C Data Interface可进程内零拷贝交换；Rust/.NET都有官方实现。[S033–S036] 它天然适合列批处理，但按ID取一行的20个字段需要跨多个buffer，CPU缓存局部性不如行式/混合式。
- **Parquet**：row group→column chunk→page，带dictionary/RLE/delta和page index，可列裁剪、跳页和压缩。[S037,S038,S108,S109] 但点查一行会付出metadata、page定位、解码固定成本，且C# WASM codec/包体需测。

**本画像判断：** Arrow/Parquet很适合离线分析、平衡模拟、AI批量验算、超大本地化/日志型表；不推荐直接作为通用Gameplay点查格式。可以借鉴其“row group/page/index/dictionary”的机制。

### 嵌入式数据库：SQLite

`Verified`：SQLite有稳定文件格式、B-tree索引、页缓存和mmap选项；官方WASM支持OPFS。sql.js-httpvfs展示了只读数据库静态托管+HTTP Range按页读取，但项目自己强调索引、covering query和请求chunk配置的重要性，也承认缺少完整cache eviction/tests。[S039–S043,S110,S111]

**本画像判断：** SQLite是强有力的对照方案：查询灵活、工具成熟、无需自建B-tree；但Rust+C#同一reader、.NET WASM桥接、native/IL2CPP构建、canonical逻辑Hash、压缩与冷热布局都比自定义chunk复杂。适合作为“数据量大、查询不规则”的表族或浏览器Range原型，而非未经实测的唯一格式。

### 自研 typed binary

`Estimated`：自研价值来自把画像特有需求设为一等公民：

- 物理chunk与Revision共享；
- 表/分片/字段sidecar；
- 离线ID索引；
- Rust/C#生成相同bounds-checked view；
- UTF-8共享池与低基数字典；
- per-chunk压缩/Hash；
- Server/Client/Voxel投影；
- canonical逻辑Hash独立于布局。

代价是规范、双语言reader、fuzz、golden corpus、dump、迁移器、安全审计都要自建。只有在三条现成候选实测均无法达标时才应正式锁定；但必须从第一天把容器边界和reader接口预留好。

## C.3 完整格式选型矩阵

> 每格均有 `[V] / [R] / [E]` 与依据或推断。为便于直接决策，原表也保存于 `appendix/format-selection-matrix.csv`。

| 格式 | 零拷贝直读 | 行级随机访问 | 列裁剪 | Schema演进 | Rust支持 | C#支持及AOT/IL2CPP | WASM可用性 | 解码CPU成本 | 常驻内存 | Canonical化 | 压缩配合度 | 可diff/review | 工具链成熟度 | 已知产品/采用案例 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| JSON + JCS | [E] 文本必须扫描/解析，不能在原始字节上按字段零拷贝；可用Utf8JsonReader减少中间字符串，但仍是解析。依据[S001,S092,S097]。 | [E] 单一数组文件无索引时O(n)；必须另建ID→byte-range索引或一行一片。 | [E] 不天然列裁剪；流式读取可跳过字段但仍需扫描语法。 | [V] 自描述，新增未知字段可被宽容读取器忽略；删/改类型/默认值需应用层规则。[S001] | [V] serde_json等成熟社区生态；JCS需专门实现或自写canonical writer。[S002] | [V] System.Text.Json支持source generation，适合AOT；JCS需自定义canonical层。[S092] | [V] 无mmap依赖；浏览器可直接读取网络字节。大文件解析、UTF-16字符串膨胀是主要风险。[S089,S096] | [E] 高于紧凑二进制，成本来自UTF-8词法、数字解析、转义和分配。 | [E] 若materialize为对象图最高；流式/按表解析可降低峰值。 | [V] RFC8785定义JCS，但仅覆盖I-JSON/IEEE754数值；字符串不自动Unicode归一化，需额外入口规范。[S002,S003] | [E] 文本重复度高，通用压缩效果通常好；整包压缩会破坏随机访问，需分块。 | [V] 原始JSON可读；JCS产物可读但格式化差。推荐review非canonical pretty镜像。[S001,S002] | [V] 极成熟；schema、生成器、lint工具多。 | [R] 游戏早期/诊断格式常见；Luban、xresloader均支持JSON输出，但公开商业规模数据不足。[S048,S050] |
| JSON5 | [E] 同JSON，需要解析。 | [E] 同JSON，需外置索引。 | [E] 不天然支持。 | [E] 自描述但注释、尾逗号等不提供兼容协议；演进由应用层定义。[S004] | [R] 多个社区解析器，质量不一。 | [R] 社区库为主，AOT/裁剪能力逐库验证。 | [V] 纯解析器可在WASM工作，无mmap依赖。 | [E] 比严格JSON略高或相近。 | [E] 通常materialize；与库实现有关。 | [E] 规范未定义canonical编码；必须先归一到逻辑IR/JCS。 | [E] 适合分块后压缩。 | [V] 对人类编辑友好，注释收益高；机器间交换不宜依赖宽松语法。[S004] | [R] 编辑/解析生态中等，跨语言一致性弱于JSON。 | [R] 常作源文件/工具配置，公开大型游戏运行时案例不足。 |
| YAML 1.2.2 | [E] 语法复杂，必须解析；锚点/别名增加对象图语义。[S005] | [E] 单文档无随机索引。 | [E] 不天然支持。 | [E] 自描述；schema演进完全由应用层，隐式类型/标签需严格profile。 | [V] serde_yaml等社区实现；规范复杂导致实现差异需测试。 | [R] YamlDotNet等社区库；反射、裁剪、AOT需显式验证。 | [V] 纯托管/纯Rust解析可用；大文档启动与分配不利。 | [E] 通常高于JSON。 | [E] 对象图与字符串分配较高。 | [E] 无通用canonical；锚点、映射顺序、标量表示使同值多编码。 | [E] 压缩好，但需文件/块切分保持按需。 | [V] 人读友好但缩进和隐式类型易制造review盲点。 | [V] 通用DevOps生态成熟，游戏强类型生成链需另建。 | [R] 常用于工具配置；作为大型游戏运行时表的公开证据薄。 |
| TOML 1.1.0 | [E] 必须解析。 | [E] 单文件无主键索引。 | [E] 不天然支持。 | [E] 自描述；表/数组结构可演进，兼容规则由应用层。 | [R] toml生态成熟。 | [R] Tomlyn等社区库；AOT能力需验证。 | [V] 无系统依赖，适合小型源配置。 | [E] 文本解析成本。 | [E] materialize后中等到高。 | [E] 规范不定义canonical字节；需逻辑IR规范。 | [E] 分文件/块压缩可行。 | [V] 适合手写小型配置，超宽表/百万行不友好。 | [V] 通用配置生态成熟，表格编辑/引用校验较弱。 | [R] 工具/工程配置常见；大型游戏表案例不足。 |
| CSV + CSVW | [E] 必须扫描分隔符/引号；可按行切片但字段仍需解析。[S007] | [E] 可离线生成ID→文件偏移；换行/引号使随机定位必须基于已解析索引。 | [E] 行式文件不天然；可按列拆文件。 | [V] CSVW可外置列schema；纯CSV自身无类型与演进协议。[S008] | [V] csv crate成熟。 | [V] CsvHelper等成熟；sourcegen/AOT按库验证，也可自写简单reader。 | [V] 容易在WASM使用。 | [E] 低到中；数字/转义解析是成本。 | [E] 解析成对象会膨胀；流式较低。 | [E] RFC4180允许表示差异且方言众多；需固定方言、行序和数字格式。 | [E] 文本压缩好；分表文件天然按需。 | [V] 很强，但复杂嵌套/多行字段使diff变差。 | [V] 极成熟；类型/引用/容器需schema工具补齐。 | [V] Excel→CSV/TSV是经典中间路线；KSFramework等公开描述此流派。[S058] |
| MessagePack | [E] 顺序编码需解析到目标字段；可保留切片但不是表级零拷贝。[S009] | [E] 默认无主键索引；按行block+偏移表可O(1)/O(log n)。 | [E] 不天然列裁剪。 | [E] 数组/映射可表达新增字段，但兼容规则和key策略由应用层。 | [V] rmp/serde生态成熟。 | [V] MessagePack-CSharp具source generator与AOT路径；Unity需按其AOT说明配置。[S010] | [V] 纯托管可用；无mmap要求。 | [E] 低到中，主要是变长整数、tag与对象构造。 | [E] 紧凑字节低；若生成对象仍有托管膨胀。 | [E] 规范允许map任意顺序、整数多种宽度；需自定义canonical profile。 | [E] 已紧凑，仍可按独立块压缩；整包压缩破坏随机访问。 | [E] 二进制不可直接review，需dump/语义diff。 | [V] 多语言成熟；游戏工具Luban/xresloader支持。[S048,S050] | [R] xresloader推荐协议二进制/MsgPack输出；具体商业项目未公开。[S050,S051] |
| CBOR | [E] 需解析major type；可跳过但非结构内零拷贝。[S011] | [E] 默认无ID索引；可加tag/索引层。 | [E] 不天然。 | [E] 自描述，应用层定义字段兼容；tag可扩展但跨语言支持不一。 | [R] ciborium/serde_cbor等社区库。 | [V] System.Formats.Cbor提供.NET实现；AOT风险较低但目标版本需实测。[S012] | [V] 无mmap依赖。 | [E] 低到中。 | [E] 字节紧凑；对象materialize仍膨胀。 | [V] RFC8949 §4.2给deterministic encoding，但应用还需定map key/Unicode/float策略。[S011] | [E] 可块压缩；自身紧凑。 | [E] 二进制，需诊断导出。 | [V] 标准成熟，游戏专用生成/校验生态较弱。 | [R] 广泛用于协议/IoT；公开游戏配表案例不足。 |
| BSON 1.1 | [E] 文档带长度可跳过，但字段访问仍扫描键，非表零拷贝。[S013] | [E] 单文档/数组无主键索引；数据库层另说。 | [E] 不天然。 | [E] 自描述；字段兼容由应用层。 | [R] bson crate/MongoDB生态。 | [V] MongoDB.Bson成熟；AOT/trim需验证。 | [E] 纯托管可用，但格式相对膨胀，不适合包体敏感首选。 | [E] 中等，键名重复。 | [E] 二进制本体较大，对象图更高。 | [E] 元素有顺序但逻辑map顺序/数值类型仍需固定profile。 | [E] 压缩收益可观；仍需分块。 | [E] 需dump。 | [V] MongoDB工具链成熟，游戏配表生态弱。 | [R] 无有力公开游戏配表产品案例。 |
| Protocol Buffers | [E] wire format需解析；未知字段保留/跳过，但不是随机字段零拷贝。[S014] | [E] repeated rows默认线性；需显式生成key index或每行独立message。 | [E] 行式message不天然列裁剪。 | [V] 字段号+presence规则成熟；不得重用tag，类型变更受限。[S015,S016,S017] | [V] prost/protobuf等成熟，官方跨语言规范。 | [V] 官方C#生成器成熟；AOT通常可通过生成类型实现，反射descriptor/trim仍需测试。 | [V] 可在.NET WASM使用，无mmap依赖。 | [E] 低到中，varint和对象构造。 | [E] wire紧凑；生成对象有明显膨胀。 | [V] 官方明确“deterministic serialization不等于canonical”，跨语言/版本不可依赖原始字节Hash。[S018] | [E] 可按message/块压缩；无内建seek table。 | [E] .proto可review，data binary不可，需dump。 | [V] 极成熟，多语言代码生成/校验生态强。 | [R] xresloader/Tableau等配表工具采用；命名商业游戏公开数据不足。[S050,S056] |
| Apache Thrift | [E] 常用协议需要反序列化。 | [E] 无表主键索引。 | [E] 不天然。 | [V] 字段ID、optional/required、union等提供演进基础；required演进风险大。[S031] | [V] Rust实现存在，成熟度按实现。 | [V] 官方/社区C#生成路径；AOT需生成代码与运行库实测。 | [E] 理论可纯托管，实际包/运行库需验证。 | [E] 低到中。 | [E] 对象materialize。 | [E] 协议未为配置Hash定义canonical。 | [E] 外部分块压缩。 | [E] IDL可review，data不可。 | [V] RPC生态成熟，游戏配表工具生态弱。 | [R] 无充分公开游戏配表案例。 |
| Apache Avro 1.12 | [E] 二进制按writer schema顺序解析。 | [E] object container可block定位但主键仍需索引。 | [E] 行式Avro不天然；与列式不同。 | [V] writer/reader schema resolution、aliases/unions提供成熟演进。[S032] | [R] Rust apache-avro社区/Apache生态。 | [V] Apache.Avro .NET可用；AOT与反射需测。 | [E] 可纯托管，但库体积与动态schema路径需测。 | [E] 中等，schema resolution有成本。 | [E] 通常materialize。 | [E] spec有排序语义但未直接等于跨实现canonical文件；需逻辑Hash。 | [V] object container支持codec/block，随机主键仍需外部index。 | [E] schema可review，binary data需dump。 | [V] 数据工程成熟，游戏运行时生态弱。 | [R] 无充分公开游戏配表案例。 |
| FlatBuffers | [V] 核心能力：buffer内按offset访问，无预解析/对象构造；要求little-endian与对齐规则。[S019,S020] | [E] vector按位置O(1)；按ID需排序后二分或额外hash/index。 | [E] 行式table可只触及读取字段，但物理数据交错，不是列式I/O裁剪。 | [V] 允许末尾加字段、保留ID等受约束演进；改变既有字段类型不安全。[S021] | [V] 官方Rust生成器/运行库。[S023] | [V] 官方C#生成器，支持Span/unsafe选项；AOT可行但生成规模需测。[S022] | [V] 无mmap硬依赖，可在下载到内存的byte buffer上读。 | [V] 低，主要边界检查与间接offset。 | [V] 可直接持有buffer，避免每行对象；索引/字符串解码另计。 | [E] 合法布局可能不同，原始buffer非天然canonical；需canonical builder约束或逻辑IR hash。[S020] | [E] buffer可按表/分片独立压缩；压缩块内需先解压。 | [E] schema可review，data不可；可生成JSON诊断。 | [V] 多语言工具成熟；校验器与游戏表引用语义需另建。 | [R] Luban支持FlatBuffers输出；公开商业游戏与规模数据不足。[S048] |
| Cap'n Proto | [V] 消息即内存布局，读取无需传统解析；指针为相对word offset。[S024] | [E] list位置O(1)；按业务ID仍需索引。 | [E] 可按字段访问但不是列式远程裁剪。 | [V] schema evolution有明确限制，ordinal稳定。[S025] | [R] Rust capnp实现活跃但非参考C++实现。 | [R] C#实现属于第三方语言实现，官方页面明确不由原作者统一审查。[S026] | [E] 可编译到WASM，但C#/Rust双栈成熟度和包体需实测。 | [V] 低，访问包含pointer traversal/边界。 | [V] buffer view低。 | [V] 规范定义canonical form；默认编码器通常不自动产出，需显式canonicalize。[S024,S027] | [E] packing/外部分块；随机块需索引。 | [E] schema可review，data不可。 | [R] 核心规范成熟；C#生态弱于Protobuf/FlatBuffers。 | [R] 未找到成熟游戏配表公开案例。 |
| rkyv | [V] Rust内直接访问Archived类型；bytecheck可验证。[S028] | [E] 可设计数组/索引，但不是自动业务主键索引。 | [E] 取决于自定义布局，默认结构导向。 | [E] 归档布局绑定Rust类型和format controls，迁移能力弱；选项改变可破坏旧数据。[S029] | [V] 原生强项。 | [E] 无官方C# reader；共享同一产物需另写规范/读取器，失去主要收益。 | [V] Rust WASM可用；.NET WASM双读不合适。 | [V] Rust侧很低。 | [V] Rust侧零拷贝低；跨FFI后复杂。 | [E] 默认归档字节不应直接作为长期跨版本canonical；需锁定全部format controls。 | [E] 可外部分块压缩。 | [E] data二进制。 | [V] Rust生态成熟，跨语言工具不足。 | [R] Rust应用采用多；未找到Rust+C#游戏配表公开先例。 |
| bincode 2.0.1 | [E] 紧凑反序列化，不是通用零拷贝。 | [E] 无业务索引。 | [E] 不支持。 | [E] 与Rust类型/serde模型强绑定，schema演进需自建版本/迁移。[S030] | [V] Rust成熟。 | [E] 无官方C#协议/生成器；需重复实现spec。 | [V] Rust WASM可用；.NET双读不优。 | [V] 低。 | [E] bytes紧凑，但通常materialize。 | [E] 配置选项、类型演进必须锁死；不是跨语言canonical标准。 | [E] 可分块压缩。 | [E] data不可review。 | [V] Rust工具成熟，跨语言弱。 | [R] 常作Rust内部持久化；不适合作为本画像共享格式首选。 |
| Arrow IPC / Feather | [V] 列式连续buffer，C Data Interface支持进程内零拷贝交换；IPC用于跨进程/文件。[S033,S034] | [E] 按row取20列需触及多个column buffers；ID索引另建。 | [V] 天然列裁剪，适合批扫描。 | [E] schema可带metadata，字段兼容需应用层版本策略。 | [V] Arrow Rust官方生态。[S036] | [V] Arrow .NET官方，使用Memory/Span；部分大数组限制需留意。[S035] | [V] 不强依赖mmap，可从内存buffer读；库/包体与分配需测。 | [V] 批量列访问低；单行跨列访问缓存局部性差。 | [V] 可持有列buffer；字符串offset/data两buffer。 | [E] IPC存在多种合法批次/字典安排，需逻辑canonical层。 | [V] IPC可分record batch；外层块压缩；随机块需manifest。 | [E] schema可review，data需工具。 | [V] 数据工程工具非常成熟，游戏访问器生态弱。 | [R] 广泛分析系统采用；未找到作为主游戏配表运行时的公开案例。 |
| Parquet | [E] 磁盘列式压缩格式，需解码page，不是运行时字段零拷贝。[S037] | [E] 无业务主键O(1)；row group/page index和统计可跳过范围，点查需专门索引。[S038] | [V] 核心优势，按column chunk/page读取。 | [E] schema evolution由读者/逻辑类型约定，复杂变更需迁移。 | [V] parquet/arrow-rs成熟。 | [R] Parquet.Net/ParquetSharp等；AOT/WASM需专项验证。 | [E] 理论可纯托管，但库体积、codec与range I/O复杂；不宜未测即选。 | [E] 批扫描高效；点查一行的固定开销偏高。 | [V] 压缩列驻留可低，但解码batch/字典/页缓存占峰值。 | [E] 文件布局、page大小、codec等多种合法表示，不适合作原始Hash。 | [V] 行组、列块、页、字典、RLE/delta天然结合压缩与跳读。[S037,S108] | [E] schema/metadata可查看，数据不适合code review。 | [V] 分析生态极成熟。 | [R] 数据湖/分析广泛；作为Gameplay点查表证据不足。 |
| SQLite | [E] B-tree page按需解析，非业务结构零拷贝；mmap可减少复制但WASM不具同样条件。[S039,S040] | [V] PRIMARY KEY/index提供O(log n)点查；covering index可减少page请求。[S110] | [E] 通过SELECT只取列，但行存页仍可能带其它字段；covering index是真裁剪近似。 | [V] ALTER/migration成熟，但发布只读DB仍需显式schema版本。 | [V] rusqlite/sqlx等成熟，需native/wasm构建。 | [V] Microsoft.Data.Sqlite等成熟；IL2CPP/.NET WASM native集成是高风险实测项。 | [R] SQLite官方WASM/OPFS可用；sql.js-httpvfs证明静态Range读取可行，但与.NET WASM直接集成不同。[S041,S042] | [E] 点查低到中，SQL VM+B-tree/page cache；首个range RTT可能主导。 | [E] page cache+query结果；可控但非零。 | [E] 文件可能因page布局/vacuum等变化而字节不同；应hash逻辑导出或发布字节并禁止重写。 | [V] 页级访问与page cache天然；压缩不是SQLite核心能力，外层整库压缩会破坏随机访问。 | [E] schema/SQL可review，DB二进制需语义dump。 | [V] 调试/查询/迁移工具极成熟。 | [R] sql.js-httpvfs公开演示670MiB/8M行，但作者称PoC且缺cache eviction/tests；不能外推游戏体验。[S042] |
| 自研 typed binary（分块容器） | [E] 可设计为relative offsets+validated views，接近FlatBuffers；安全性完全取决于规范与验证器。 | [E] 可把ID→chunk/offset/length作为一等索引，实现O(1)/O(log n)。 | [E] 可按列、字段sidecar或混合布局定制。 | [E] 必须自定义schema版本、兼容矩阵、迁移工具；维护成本最高。 | [E] 可生成Rust reader。 | [E] 可生成C# Span reader，天然AOT；必须控制泛型/代码量。 | [E] 可避免mmap并面向HTTP chunk/range设计。 | [E] 可做到很低；需基准证实。 | [E] 可做到buffer-view+共享string pool；需实测目标runtime。 | [E] 可以把canonical逻辑IR与物理布局分离，最符合本画像；但规范、测试向量和双实现一致性都要自建。 | [E] 可把每chunk设独立Zstd/LZ4 frame并用manifest seek；最灵活。 | [E] 二进制不可review，必须生成canonical text/semantic diff。 | [E] 工具全自建：compiler、validator、dump、fuzzer、golden vectors。 | [R] 大量团队被认为会走向自研，但公开可核产品/规范很少；本报告不把该说法当行业共识。 |

## C.4 矩阵的决策解读

### 不进入最终共享格式决赛

- **YAML/TOML/JSON5**：适合源，不适合大规模运行时。
- **BSON/Thrift/Avro**：没有足够画像特有优势。
- **rkyv/bincode**：跨C#弱。
- **Parquet**：访问模式错位；保留给分析/批量表。
- **Cap'n Proto**：规范很强，但C#生态与统一实现风险高于FlatBuffers。

### 保留为辅助层

- **Protobuf**：schema、工具交换、fixture、跨语言兼容。
- **MessagePack/CBOR**：紧凑中间chunk或工具协议。
- **Arrow/Parquet**：离线模拟、分析、AI批处理。
- **SQLite**：不规则查询表或Range POC。

### 进入实测决赛

1. **JSON/JCS基线**：测正确性、启动与对象内存；也是迁移oracle。
2. **FlatBuffers + 外置索引 + chunk容器**：测共享buffer、代码体积、随机取行、压缩后解压峰值。
3. **自研typed binary + generated views**：测最优潜力与实现复杂度。
4. **SQLite readonly + indexed query + Range/本地文件后端**：测浏览器请求数、包体、AOT/native集成和查询灵活性。

## C.5 “JSON起步 → 二进制升级”的不变量

以下每一项都必须在JSON阶段落为测试向量；否则第二期不是“换后端”，而是数据语义迁移。

1. **列类型的精确宽度与有符号性。** JSON只有number语义，必须由schema决定`i32/i64/u32/u64/f32/f64`，并在导入时拒绝越界。
2. **ID的文本形式与运行时形式。** 源可读字符串ID是否编译成整数、整数是否跨版本稳定、删除后是否保留墓碑，第一天决定。
3. **行的canonical顺序。** 即使运行时hash table无序，逻辑Hash必须按stable ID排序；显示顺序另存ordinal。
4. **列的canonical顺序。** 按schema ordinal而非字典遍历顺序。字段名改名与ordinal不能混淆。
5. **missing / empty string / explicit null / defaulted 的区分。** 当前冻结类型没有nullable，但源层仍要能拒绝null并区分缺失和空字符串；默认值应用后是否参与Hash必须固定。
6. **默认值求值时机。** 推荐编译期填充IR并记录`presence=defaulted`用于诊断；Hash对“语义值”统一处理，不因源是否显式填写而变化，或反之——两种均可，但只能选一种。
7. **未知列规则。** 已冻结为拒绝；JSON parser不能悄悄忽略多余属性。
8. **字符串编码。** 统一UTF-8；禁止依赖C# UTF-16对象字节。决定是否在入口NFC归一化、是否保留原始拼写。[S002,S003,S096]
9. **数字文本规范。** 禁止区域格式、千分位、十六进制和科学计数法的隐式接受，或明确白名单。canonical文本用Invariant最短往返表示。
10. **浮点特殊值。** 是否允许NaN、Infinity、-0；推荐配表层拒绝。`f32`应先按f32舍入，再生成canonical位模式/十进制，而不是一直按f64到运行时再截断。
11. **引用表达。** 源用可读ID，编译期解析到目标stable ID/ordinal；跨端不可见引用的处理第一天固定。
12. **字符串池身份。** 逻辑Hash按字符串值而非pool offset；这样二进制重排不改变身份。
13. **schemaVersion / configRevision / sourceHash含义。** schema版本是契约版本；configRevision是内容发布代；sourceHash是canonical输入根，三者不可复用。
14. **错误模型。** 错误码、table/row/column、source span、related reference、fix hint需稳定，供CI、编辑器和AI共用。
15. **投影规则。** Server/Client/Voxel分组必须作用于IR，不由运行时删字段；每个投影有独立root hash。
16. **访问API。** 业务只依赖typed view/port，不依赖`JsonElement`、FlatBuffers生成类型或SQLite connection。
17. **逻辑Hash算法与域分隔。** 固定算法、版本、前缀和树结构；换压缩/布局不换逻辑Hash。
18. **黄金语料。** 包含边界整数、Unicode等价串、空值、默认值、未知列、引用环、坏长度、压缩炸弹、不同顺序等，Rust/C#同时跑。

## C.6 同一份产物服务 Rust 与 C# 的物理约束

### 端序

`Verified`：FlatBuffers固定little-endian；Rust与.NET均提供显式little-endian读取API。[S020,S125,S126] 推荐所有整数、浮点位模式和offset统一little-endian，不允许“native endian”。

### offset而非pointer

使用相对文件/chunk起点的`u32`或`u64` offset，禁止持久化进程指针。`u32`把单chunk限制在4GiB内，但对WASM更友好；超过则分chunk，不轻易升级所有offset为64位。

### 对齐与padding

`Verified`：Rust默认`repr(Rust)`布局不稳定，不能直接把结构体内存当跨语言ABI；必须使用显式layout或逐字段reader。[S124] 推荐格式规范定义字段宽度和alignment，生成reader做`offset + size <= bufferLength`检查，绝不`reinterpret_cast`未验证字节为含引用的托管结构。

### 字符串

统一`UTF-8 data pool + offset + byteLength`。C#按需用`ReadOnlySpan<byte>`或缓存解码string；Rust返回`&str`前先验证UTF-8。字符串pool offset不进入逻辑Hash。

### 容器头与chunk目录

在冻结的二进制头之后，建议物理层为：

```text
ContainerHeader
RevisionManifest
  TableDirectory[]
    tableId, schemaVersion, projection, tableHash
    indexChunkHash, dataChunkHashes[], sidecarHashes[]
ChunkDirectory[]
  contentHash, codec, compressedLength, rawLength, offset/alignment
PayloadChunks...
```

`Estimated`：浏览器部署时，manifest与chunk可拆成独立文件；桌面/服务器可合并为一个容器。两种物理打包复用相同chunk hash与reader。

### 校验责任

- 外层头：长度、最大分配、压缩比、checksum/hash、签名；
- chunk：hash、raw length、codec、边界；
- schema view：offset、vector length、UTF-8、enum范围；
- 业务：ref、范围、唯一性等应在编译期完成，运行时仍把不可信下载当不可信字节，至少复核内存安全条件。

## C.7 本章倾向性结论

`Estimated`：不把“最终格式”绑定到单个第三方协议，而定义一个**Config Artifact Container Profile v1**：manifest、projection、chunk、index、hash、codec和安全边界由委托方规范；每个chunk的payload后端可先是canonical JSON，后切FlatBuffers或typed binary。SQLite作为独立实验后端，不与核心逻辑接口耦合。

### 来源

[S001–S047, S092, S097, S108–S111, S120, S124–S126]

---

# D. 运行时加载与懒加载

**结论先行 1：** 懒加载必须是发布格式、索引、调用API和Revision生命周期共同设计的结果，不能在“全表反序列化API”外面套一个Lazy。  
**结论先行 2：** 默认表级；大表分片级；长文本/本地化/数组做字段sidecar。网络行级加载不是默认能力。  
**结论先行 3：** 快照持有完整不可变manifest与可解析的artifact命名空间，数据chunk可延迟驻留；所有加载结果带代际，原子切根而非逐表替换。

## D.1 四种加载模型

| 模型 | 启动 | 稳态内存 | 首次访问 | 适用 |
|---|---|---|---|---|
| 全量预加载 | 最慢、最确定 | 最高 | 无IO hitch | 小包、服务器预热、Voxel核心表 |
| 启动加载manifest/index，数据按需 | 中低 | 可控 | 首次chunk有网络/解压 | **本画像默认** |
| 完全按需（含索引） | 最低 | 最低 | 首次可能两跳：索引+数据 | 极冷、大量独立DLC，不适合Gameplay关键表 |
| 预测性预取 | 取决于预测窗口 | 较高但可控 | 命中时低 | 场景/关卡/玩法进入前有明确依赖集 |

`Estimated`：启动必须同步获得最小Release Manifest、schema兼容信息、projection root、bootstrap table列表和每表chunk目录；否则运行时无法知道请求哪个revision，也无法在访问前判断是否允许阻塞。

## D.2 “快照”的精确定义

**Active Snapshot不是“所有行对象都在内存”，而是：**

- 一个不可变`RevisionManifest`；
- 对该Revision全部表/分片的内容身份、位置、schema与投影信息的完整承诺；
- 一个与该Revision绑定、可并发填充的**派生缓存**；
- 业务查询只能通过该snapshot句柄解析，不能混用global current。

缓存中“某chunk尚未加载”不改变snapshot内容，只表示本进程尚未拥有其物理字节。类比内容寻址资产：身份已冻结，驻留状态可变。缓存本身是性能状态，不进入配置语义Hash。

## D.3 懒加载粒度

### 表级

**适用：** 大多数Gameplay表、每表几十KB到数MB、查询常跨多列。  
**收益：** 一次IO后表内访问简单；索引小；调用者容易显式warm-up。  
**代价：** 单表过大时首次解压和峰值内存高。

### 分片级

**适用：** 百万行ID表、区域/关卡/章节天然分区、按ID高位或稳定hash可定位。  
**关键：** shard函数必须由manifest记录并跨版本稳定；不能用进程随机hash。  
**推荐分片键：** 业务域键优先（region/chapter/biome），其次`stableId range`，最后固定seed hash bucket。  
**代价：** 跨shard二级索引、范围查询和预取复杂。

### 行级

**适用：** 本地已缓存大文件且每行很大、访问极稀疏；或SQLite B-tree已有页缓存。  
**不适用：** 浏览器每行一次HTTP请求。RTT和headers会压倒几十/几百字节payload，且请求顺序成为卡顿源。  
**实现前提：** 行ID→chunk/offset/length索引常驻；多行请求合并为块；行不得跨压缩块。

### 列级

**适用：** 本地化、分析、编辑器筛选、服务器批量预计算。Arrow/Parquet天然支持。[S033–S038]  
**代价：** Gameplay按ID取整行时要触及多个列buffer；对20列点查不一定比混合行式好。

### 字段内按需 / sidecar

将长描述、富文本、脚本片段、数组blob、本地化字符串或图像metadata从热行拆出：热表只保存`sidecarKey/offset`。这是比“全表列存”更可控的冷热分离。

## D.4 离线索引设计

### 两级目录

```text
Manifest: tableId -> TableDescriptor
TableDescriptor: keyEncoding, shardFunction, primaryIndexChunk, dataChunks, sidecars
PrimaryIndex: stableKey -> (chunkOrdinal, rowOrdinal or byteOffset, byteLength)
SecondaryIndex: secondaryKey -> sorted rowOrdinal list / range
```

### 索引大小模型

若每行索引使用`u32 key + u32 rowOrdinal`，理论payload约`8N`；64位key或存chunk/offset/length可到`16–32N`。哈希表还需空桶/控制字；B-tree有page与内部节点；字符串key若不先intern会显著增大。

`Estimated`：100万行即使数据只懒驻留10%，一个16B/行的全局索引仍约15.3MiB；再加多个二级索引可能反客为主。因此索引也必须分层：常驻稀疏目录→shard索引按需。

### 主键结构选择

| 结构 | 构建期已知只读键下的特点 | 推荐条件 |
|---|---|---|
| 有序数组 + 二分 | 最小、可序列化、跨语言简单、缓存友好；O(log n) | **默认**，尤其WASM与共享格式 [S130] |
| 开放寻址/Swiss风格只读hash | 近O(1)，空间与实现复杂度更高；需固定hash/seed | 极热且百万级点查，基准证明二分不够时 |
| Minimal Perfect Hash | 无冲突、可接近最小；构建与跨语言实现复杂，缺少自然范围序 | 极稳定巨大键集且只做点查；先做spike [S127,S128] |
| B-tree/B+tree | page友好、范围查询、动态数据库成熟 | SQLite后端或大量范围查询 [S110] |
| 稀疏索引 + 块内二分 | 常驻索引小；多一步chunk读取 | 超大表、分片/压缩块明显 |

**明确判断：** `Estimated`：在“只读、构建期已知所有键、跨Rust/C#、WASM包体敏感”前提下，**有序紧凑数组+二分是默认最优工程解**。只有profile证明热路径不足，再引入生成perfect hash或冻结hash table。C#的`FrozenDictionary`说明只读优化是成熟方向，但其内部布局不是跨语言磁盘协议。[S129]

### 组合键与二级索引

- 组合键编译为固定字段tuple，按schema顺序比较；不要用文化相关字符串拼接。
- 二级唯一索引：`key→rowOrdinal`。
- 非唯一索引：`key→(start,count)`指向排序row list。
- 范围索引：按数值/规范化字符串排序，二分上下界。
- 倒排索引和复杂条件筛选只有真实查询需求才生成；否则代码/内存爆炸。

### 索引与压缩块对齐

索引的最小定位单元必须是**独立可解压块**。若索引指向块中行offset，块内offset针对解压后buffer；不得指向压缩流中不可独立解码的位置。Zstandard Seekable用独立frames+seek table，LZ4 frame可配置independent blocks，这正是标准解法。[S044–S046]

## D.5 首次访问卡顿的来源

一次cold miss可能串联：

1. manifest查找；
2. Service Worker/Cache/IndexedDB命中检查；
3. DNS/TLS/HTTP请求或Range；
4. 压缩输入分配；
5. 解压输出分配；
6. hash/checksum/边界校验；
7. 构建/映射主键索引；
8. UTF-8解码和对象materialize；
9. 首次泛型/JIT/AOT静态初始化；
10. GC与旧缓冲释放。

`Estimated`：很多benchmark只测第5或第8步，因此对真实首触延迟没有解释力。

### 摊销手段

- **显式准备阶段：** `PrepareAsync(UsageSet, Revision)`，禁止关键Tick第一次隐式加载。
- **依赖声明：** 场景、玩法、地图或能力包声明需要的table/shard/locale。
- **合并请求：** 相邻chunk或Range按窗口合并；一次拉多个小块。
- **并行但限流：** 网络/解压并行度有上限，避免峰值缓冲叠加。
- **分帧materialize：** 若必须生成对象，在非确定性展示层分批；确定性逻辑层在进入玩法前完成。
- **后台线程/Web Worker：** 现代.NET在Web Worker上可运行C#，但目标浏览器、线程支持、shared memory/cross-origin isolation与发布版本必须实测，不应再笼统假设“WASM永远单线程”。[S090]
- **AOT/source generation：** 避免首次反射、动态代码和被裁剪类型。[S091–S093]

## D.6 缓存与驱逐

### 缓存对象不是配置对象

缓存条目建议状态：

```text
Absent -> Loading(revision, requestId) -> Resident(buffer, verifiedHash)
                         \-> Failed(classified error, retryPolicy)
Resident -> Pinned -> Evictable -> Absent
```

### 驱逐策略

- **Bootstrap/热表：永不驱逐。** schema、manifest、核心ID映射、Voxel热表。
- **普通chunk：大小加权LRU/2Q。** 不是只按条目数；同时考虑解压成本和下一场景依赖。
- **句柄安全：** 返回`RowView`时必须钉住owner chunk，或API把访问限制在借用scope内；绝不能驱逐仍被view引用的buffer。
- **弱引用不适合作唯一安全机制。** GC何时回收不可控，且原生buffer/FFI句柄不受普通弱引用充分管理。
- **Revision隔离：** cache key至少是`(projection, revision, chunkHash)`；内容Hash相同可指向同一物理buffer。

`Estimated`：推荐chunk对象使用引用计数+epoch：Active/Staged manifest持有引用；瞬时查询scope再pin；切换后旧root进入retired list，等所有读epoch退出再释放。

## D.7 浏览器 / WASM 专节

### 路线一：独立内容寻址chunk文件（推荐默认）

```text
/config/<release>/manifest.bin
/chunks/sha256/ab/cd...   # immutable, cache-forever
```

**优点：** CDN/Service Worker缓存自然；每chunk可独立签名/hash/压缩；未变内容跨revision复用；请求失败粒度小。  
**缺点：** 小文件请求数可能爆炸；需bundle/pack小chunk和预取清单。

### 路线二：单一pack + HTTP Range

`Verified`：HTTP Range通过`Range`请求，服务器可返回206；不支持或忽略时可能返回200全体；416表示范围无效。`If-Range`可配ETag/Last-Modified避免对象更新后拼接错版本。[S085]

**硬要求：**

- pack不可原地覆盖；URL/ETag必须内容寻址；
- manifest先给offset/length/hash；
- Range合并、最小请求窗口和并发限制；
- 若CDN/压缩中间层破坏Range，必须检测200并有大小上限；
- 不允许对整个pack再套HTTP内容压缩导致byte range语义落在压缩流上。

### SQLite Range先例

`Verified`：sql.js-httpvfs用sql.js在浏览器中对远端只读SQLite发Range请求。其README明确建议合适索引、covering index、较大数据库page/request chunk，并说明全扫描会下载大量数据；项目也公开列出cache eviction/tests等不足。[S042] 这证明机制可行，也证明“数据库文件+Range”不是免费懒加载。

### 缓存层

- **Cache API**存Request/Response，应用负责版本键与清理；浏览器可按配额驱逐。[S086,S087]
- **IndexedDB**适合按chunk key保存ArrayBuffer与manifest元数据，异步事务模型。[S088]
- **Service Worker**可做cache-first、预取和离线，但更新生命周期必须与Release Manifest一致。
- **持久化请求**可降低被驱逐概率，但不是所有浏览器/上下文都保证成功。[S087]

### 请求数控制

`Estimated`：为每个发布生成三层计划：

1. `bootstrap.pack`：首屏/握手/基础UI；
2. `usage packs`：按玩法/地图合并小chunk；
3. `cold chunks`：真正独立按需。

chunk物理合包不改变逻辑chunk hash：pack目录把hash映射到range。这样既减少请求，也保留内容复用。

### WASM内存

`Verified`：.NET WebAssembly heap可配置且浏览器/设备存在实际上限；移动浏览器常需更保守。[S089] `Estimated`：解压峰值必须按“compressed + raw + validated view + old revision”计算，不能只看最终buffer。严禁同时并行解压几十个大chunk；使用共享工作区或限流池。

## D.8 服务器端

### mmap收益与陷阱

`Verified`：SQLite允许mmap读取数据库页，减少read系统调用与复制，但其文档也提示平台/可靠性权衡。[S040] 一般只读pack也可mmap：

- 多进程可共享只读page cache；
- 启动映射快，不等于数据已进RAM；首次访问仍会page fault；
- 容器内存统计可能把file-backed页计入RSS/cgroup working set；
- 随机访问大表会产生page-fault抖动；
- NUMA机器预热线程在哪个node触页会影响局部性。

`Estimated`：Dedicated Server应在接受流量前执行确定性warm-up清单，记录major/minor faults、resident bytes和P99查询；不要把“mmap调用耗时”当加载完成。

### 多进程共享

内容寻址只读pack + mmap有利于宿主page cache共享；但每进程的C#对象materialization和hash index仍不共享。若目标是共享，应尽量在buffer上view，而非每进程重建百万行对象。

## D.9 懒加载 × 不可变快照的三种形态

### 形态A：快照持有全部原始字节，延迟materialize

- **语义：** Active/Staged都拥有完整artifact bytes；只延迟对象/索引构建。
- **优点：** 不存在网络混代；Replay简单。
- **漏洞：** 双快照大表仍占字节内存；WASM包体/下载未解决；materialize仍有hitch。
- **适合：** 服务器/桌面小中规模。

### 形态B：快照持有完整manifest，Revision按内容寻址目录加载（推荐）

- **语义：** manifest承诺全体chunk hash；bytes按需，加载请求携带revision/root。
- **优点：** 真正网络懒加载；相同hash跨Active/Staged共享；切换只换root。
- **漏洞：** 远端旧chunk必须可长期获取；缓存/下载错误分类复杂。
- **防线：** immutable URL、per-chunk hash、revision-scoped future、retention policy。

### 形态C：快照允许“访问时取当前Revision”的部分缺失

- **语义：** 表句柄只存tableId，miss时从global Active查当前版本。
- **优点：** 实现看似简单。
- **致命漏洞：** 切换前取得句柄、切换后首次加载会读到新表；一个逻辑操作可混合代际。异步旧请求完成也可能覆盖新缓存。
- **结论：** 不应采用。

## D.10 推荐的代际协议

```text
1. StageManifest(newRoot) -> 验签、schema/投影/容量检查
2. Prefetch(requiredUsageSet, newRoot) -> 每个Future捕获newRoot
3. Future完成 -> 校验chunkHash -> PublishToCache(key=(newRoot, chunkHash))
4. Tick Barrier -> atomic ActiveRoot = newRoot
5. oldRoot -> Retired；已持有句柄继续访问oldRoot
6. epoch/refcount归零 -> 释放旧root独有chunk；共享hash保留
```

**取消规则：** 切换/回滚可以取消未开始的I/O；已经完成的旧代load只能进入旧代/共享内容缓存，绝不能“顺便”挂到新代，除非目标chunk hash完全相同。

**读取规则：**

- `Snapshot.GetTable<T>()`返回revision-bound handle；
- `handle.TryGet(id)`不得隐式网络阻塞；
- `PrepareAsync`完成后才允许进入确定性玩法阶段；
- 展示层可使用`GetAsync`，但结果带revision供UI丢弃陈旧响应。

## D.11 按需加载与确定性Replay是否冲突

**明确结论：不必冲突，但“第一次访问触发阻塞/失败并改变逻辑分支”会冲突。**

`Estimated`：Replay记录固定`ConfigRootHash/ProjectionHash`；逻辑求值只看值，不看加载顺序、缓存命中、完成时间。开始Replay/Tick前必须完成所需usage set，或在缺失时fail-stop并等待恢复，不能以默认值继续。缓存统计、LRU顺序和网络时序不参与状态Hash。

## D.12 可直接实现的最小接口语义

```text
IConfigRevisionStore
  StageAsync(manifestBytes) -> StagedRevision
  PrepareAsync(revision, UsageSet) -> PrepareReceipt
  ActivateAtBarrier(stagedRevision) -> ActiveRevision
  OpenSnapshot(revision?) -> SnapshotLease

SnapshotLease
  RevisionId / ProjectionRootHash
  OpenTable<TTable>() -> TableLease<TTable>

TableLease<TTable>
  IsResident
  TryGet(stableId, out RowView)   // 绝不发网络、绝不阻塞
  EnumerateDeterministic()        // schema规定顺序
```

实现可以变化，但四个语义不能变：**代际绑定、显式准备、TryGet不阻塞、确定顺序**。

### 来源

[S019–S046, S062–S071, S085–S093, S110–S111, S114–S116, S127–S130]

---

# E. 压缩与内存占用

**结论先行 1：** 整表压缩与随机访问根本冲突；标准解法是独立块/页、seek目录和块内索引，而不是寻找“既整包最高压缩又任意seek”的魔法codec。  
**结论先行 2：** 内存优化优先级是“避免对象图 → 字符串池/字典 → 紧凑位宽 → 块压缩”，不是先换压缩算法。  
**结论先行 3：** Active/Staged、压缩输入、解压输出、验证工作区与旧代retire叠加决定峰值；只报文件体积或稳态RSS会误导。

## E.1 压缩与随机访问的根本矛盾

通用压缩通过跨较大窗口寻找重复获得比率；随机访问要求从目标位置独立开始解码。窗口越跨块，随机起点越少。

### 整表压缩

- **优点：** 实现简单，metadata少，通常获得较好整体压缩。
- **缺点：** 访问一行也要解完整表；需要`compressed + full raw`峰值；任何局部变更会重写整个压缩流；Active/Staged无法细粒度共享。
- **结论：** 只适合永远全量预载的小表或bootstrap pack，不适合标称懒加载的大表。

### 块/帧级压缩 + seek索引

`Verified`：Zstandard Seekable格式把数据拆为独立Zstd frames并在尾部记录seek table；LZ4 Frame支持block independence选项。[S044,S046] 这允许`chunk/frame ordinal→compressed offset/length→raw length`。

**设计要求：**

- 每块独立校验长度、hash和最大解压比；
- 主键索引定位到块，不直接定位跨块压缩流；
- 块大小是可测参数，不进逻辑Hash；
- 同一块内按行offset或列page访问；
- 更新时只替换变化块。

### 页级压缩

`Verified`：Parquet在row group内按column chunk/page组织，支持字典、RLE、delta等编码与page index；SQLite则以固定page和B-tree/page cache实现随机读取，但不以内建通用页压缩为核心。[S037,S038,S039,S108–S111]

**启发：** Gameplay表可采用混合“shard→column group/row page”，不用照搬Parquet全部复杂度。

### 只做紧凑编码

热表、Voxel ID映射和几十KB小表可能无需通用压缩：

- 省去解压CPU和双buffer；
- CDN HTTP压缩可用于完整bootstrap响应，但Range pack不要再套全体内容编码；
- 通过string pool、位图和varint/delta已获得主要收益。

## E.2 编码级省空间手段

### 字符串池化 / 驻留

假设一列有`N`行、平均UTF-8长度`L`、唯一率`u`：

- 每行独立托管字符串近似：`N × (stringHeader + 2L + alignment) + N×reference`；
- UTF-8共享池：`uN × (L + length/meta) + N×offset`。

当枚举名、资源路径、tag、locale key大量重复时收益显著。`Verified`：MasterMemory公开声明string interning；Arrow字符串布局本身就是offset buffer + data buffer。[S033,S053]

**风险：** 全局pool会让一个冷字符串钉住巨大buffer；推荐按projection/revision/chunk或稳定字典域分池。

### 字典编码

低基数字符串/枚举/ref列用`dictionary[] + index[]`。Arrow/Parquet规范化了这一路径。[S108,S109] 若基数`K≤256`，index可1 byte；但跨版本字典ordinal不能作为stable ID，逻辑Hash按值或显式ID。

### 位宽压缩与bit packing

若schema范围已知，例如`0..15`，物理可4 bit；bool用bitset。`Estimated`：是否值得取决于访问热点——bit extraction节省内存带宽，却增加指令和生成器复杂度。先对大低基数列使用，不对所有标量做极限打包。

### RLE

适合排序后长连续相同值：版本flag、biome、稀疏分类。随机点查需要run index或prefix count；若数据无序，收益低。

### Delta编码

递增ID、offset、时间或有序数值存首值+delta；Parquet有delta编码先例。[S108] 点查需要block restart点，不能从文件头累加百万项。

### 稀疏/可选字段

- presence bitmap：`N/8`字节/字段；
- 非空值紧密数组 + rank/select或prefix index；
- 宽表大量空字段时，拆侧表通常比每行20个nullable offset更清晰。

当前冻结类型没有nullable；仍需在源层区分“缺失可选→默认”与“空字符串”。若未来加入nullable，必须明确bitmap语义。

### 行去重与共享

大量近似行可存base row + patch，但会增加访问间接和热更依赖。`Estimated`：只对模板化静态定义使用；不要把通用继承链带入热路径。编译期最好扁平化为最终值，保留provenance用于调试。

### 行存、列存与混合布局

- **行存：** 按ID取整行缓存友好；列扫描浪费。
- **列存：** 压缩、向量化、列裁剪强；单行跨列有多buffer访问。
- **混合：** 把常用数值/ID放热行结构，长字符串/数组/低频列放sidecar；本画像默认最合适。

## E.3 常驻内存的真实构成

### 托管对象图的放大项

`Verified`：

- .NET `String`是UTF-16字符序列。[S096]
- 大于约85,000 bytes的对象通常进入LOH，LOH与Gen2回收相关。[S094]
- 托管对象有对象头、方法表引用、对齐；实际布局依运行时。[S095]
- `Span<T>/Memory<T>`可在连续托管/非托管内存上提供切片，而不要求每行对象。[S097,S120]

典型坏形态：

```text
Dictionary<int, Dictionary<string, object>>
```

每行字典、bucket/entry数组、每列key/string引用、每个数值装箱，再加原始JSON文档，可能同时保留多份。此形态即使文件只有几十MB，也可能产生数百MB/GB对象图和大量GC roots。

### 替代持有形态

1. **强类型行对象：** 语法最好，仍有每行对象和string对象；适合小中表。
2. **结构体数组：** 标量内联、连续；字符串用offset/pool；C# API可返回`ref readonly`或value view。
3. **列buffer：** 批处理/筛选好；按行访问用generated row facade。
4. **原生/共享buffer + 托管句柄：** GC压力最低；必须清晰管理pin、生命周期和FFI边界。
5. **按需解码字段：** 字符串/复杂sidecar首次访问解码并有有界cache。

## E.4 内存估算表

> 假设：20列由8个4-byte标量、4个8-byte标量、4个bool/小枚举、4个平均12 ASCII字符的字符串组成；字符串总体50%重复；只有主键索引。区间是 `Estimated`，对象布局在CoreCLR/.NET WASM/IL2CPP不同。完整公式见 `appendix/memory-model.md`，CSV见 `appendix/memory-estimates.csv`。

| 规模 | 持有形态 | 估算公式（N为行数） | 每行下界(B) | 每行上界(B) | 常驻内存下界(MiB) | 常驻内存上界(MiB) | 主要未计入项/测量注意 |
|---|---|---|---|---|---|---|---|
| 1万行 × 20列 | 全量通用对象：object[]行 + 装箱字段 | N × [行容器184 + 16个装箱值384 + 平均唯一字符串96 + 行引用8 + 主键索引(28~428)]，按700~1100 B/行取区间 | 700 | 1100 | 6.68 | 10.49 | 若使用Dictionary<string,object>、保留列名、调试元数据或重复字符串不池化，可能显著超过上界；GC元数据/碎片依运行时。 |
| 1万行 × 20列 | 全量强类型托管行对象 | N × [对象头16 + 固定字段/对齐104 + 行引用8 + 平均唯一字符串96 + 主键索引(6~136)]，按230~360 B/行取区间 | 230 | 360 | 2.19 | 3.43 | 对象头、引用宽度、字符串对象布局在CoreCLR/.NET WASM/IL2CPP不同；未含LOH碎片与加载峰值。 |
| 1万行 × 20列 | 结构体数组 + UTF-8共享字符串池 | N × [紧凑行结构88~104 + 字符串池32 + 主键索引8~16 + presence/对齐0~8]，按128~160 B/行 | 128 | 160 | 1.22 | 1.53 | 要求禁止每行string对象；字符串以offset/length指向共享UTF-8池。若字符串重复率低或平均长度更长，按实际池大小线性增加。 |
| 1万行 × 20列 | 零拷贝列/混合view + UTF-8池 | N × [数值列64 + bool位图0.5 + 4个字符串offset约16 + 字符串池32 + presence位图2.5 + 主键索引8~16]，按123~132 B/行 | 123 | 132 | 1.17 | 1.26 | 不含压缩输入缓冲、解压输出的瞬时双份、reader代码和页/块缓存元数据；按行取20列会增加cache miss而非常驻字节。 |
| 100万行 × 20列 | 全量通用对象：object[]行 + 装箱字段 | N × [行容器184 + 16个装箱值384 + 平均唯一字符串96 + 行引用8 + 主键索引(28~428)]，按700~1100 B/行取区间 | 700 | 1100 | 667.57 | 1049.04 | 若使用Dictionary<string,object>、保留列名、调试元数据或重复字符串不池化，可能显著超过上界；GC元数据/碎片依运行时。 |
| 100万行 × 20列 | 全量强类型托管行对象 | N × [对象头16 + 固定字段/对齐104 + 行引用8 + 平均唯一字符串96 + 主键索引(6~136)]，按230~360 B/行取区间 | 230 | 360 | 219.35 | 343.32 | 对象头、引用宽度、字符串对象布局在CoreCLR/.NET WASM/IL2CPP不同；未含LOH碎片与加载峰值。 |
| 100万行 × 20列 | 结构体数组 + UTF-8共享字符串池 | N × [紧凑行结构88~104 + 字符串池32 + 主键索引8~16 + presence/对齐0~8]，按128~160 B/行 | 128 | 160 | 122.07 | 152.59 | 要求禁止每行string对象；字符串以offset/length指向共享UTF-8池。若字符串重复率低或平均长度更长，按实际池大小线性增加。 |
| 100万行 × 20列 | 零拷贝列/混合view + UTF-8池 | N × [数值列64 + bool位图0.5 + 4个字符串offset约16 + 字符串池32 + presence位图2.5 + 主键索引8~16]，按123~132 B/行 | 123 | 132 | 117.3 | 125.89 | 不含压缩输入缓冲、解压输出的瞬时双份、reader代码和页/块缓存元数据；按行取20列会增加cache miss而非常驻字节。 |

### 读表方式

- 100万行的“通用object[]+装箱”下界已接近0.65GiB；若使用每行Dictionary或不池化字符串，可能更高。
- 强类型行对象改善明显，但仍有百万对象与字符串GC roots。
- 结构体/零拷贝两者在本文假设下相近；真正差别会由字符串长度、索引、可选列、访问局部性和解压缓存决定。
- 10%数据chunk驻留不等于10%总内存：manifest、全局/稀疏索引、schema、字符串字典和bootstrap表仍常驻。

## E.5 Active/Staged共享与峰值公式

采用内容寻址chunk时：

```text
Steady = SharedUnchangedChunks
       + ActiveOnlyChunks
       + StagedOnlyPreparedChunks
       + IndexesAndManifest
       + DecodedCaches

ActivationPeak = Steady
               + InFlightCompressedBuffers
               + DecompressionOutputs
               + ValidationWorkspace
               + RetiredOldGenerationStillPinned
```

`Estimated`：未变chunk的物理buffer可共享，因为两代都只读且hash相同；索引若内嵌chunk也共享。不得通过“比较tableId相同”共享，必须比较内容Hash+schema/layout profile。

## E.6 字符串专项

字符串往往是内存大头，原因包括：

- 源UTF-8→C# UTF-16翻倍；
- 每个string有对象头/对齐；
- JSON字段名和临时token可能额外分配；
- 本地化多语言若同包全载呈倍数增长；
- 路径前缀重复但未池化。

**推荐：**

- 运行时artifact全用UTF-8；
- 资源路径、tag、locale key采用dictionary或前缀无关的pool；
- 本地化按语言独立chunk，只加载当前locale+fallback；
- API区分`Utf8View`与`string GetDecoded()`；
- 热路径比较interned ID，不反复解码字符串；
- 不使用进程随机`string.GetHashCode()`作为持久索引。

## E.7 GC与加载峰值

- 反序列化大数组可能进入LOH；反复热更会留下碎片和Gen2停顿。[S094]
- 解压先分raw byte[]，再生成对象，峰值至少两份；若旧Active仍在，可能三代叠加。
- 使用ArrayPool/MemoryPool可复用临时缓冲，但归还前必须清除敏感数据并确保没有Span/view逃逸。
- `MemoryMappedFile`/native buffer在服务器减少GC托管字节，不代表操作系统内存为0。
- 基准要记录`GC.GetAllocatedBytesForCurrentThread`、各代collection、LOH size、working set、WASM heap high-watermark和native allocations。

## E.8 压缩codec选择

### Zstandard

`Verified`：RFC 8878定义Zstandard；官方seekable扩展提供独立frame与seek table。[S044,S045] 支持字典，适合重复schema/短表，但字典本身要版本化并进入物理manifest。

### LZ4

`Verified`：LZ4 Frame可使用independent blocks，解压路径简单。[S046] 适合CPU/延迟优先的热chunk；比率需目标数据实测。

### Brotli

`Verified`：RFC 7932定义流格式。[S047] 浏览器HTTP内容编码生态成熟，但不提供本报告所需的应用级随机seek目录；适合完整bootstrap响应或独立chunk，不适合一个巨大Range pack的外层连续流。

### 决策方法

不引用无条件“某算法快X倍”。对每个候选以同一chunk corpus测：compressed size、cold decode、warm decode、峰值buffer、WASM包体、Rust/C#实现一致性、字典构建成本。codec是每chunk元数据，可按热/冷表不同，不需要全局单选。

## E.9 加密

### 是否值得

`Verified`：客户端中的硬编码密钥可被逆向获取，OWASP把它列为明确风险。[S101] 因而客户端配表加密不能提供真正秘密性，只能提高静态扫描和低成本dump门槛。服务器私有数据根本不应下发。

### 顺序

**先压缩，再加密。** 加密后字节接近高熵，压缩无效；每独立chunk使用AEAD并有唯一nonce/关联数据，关联`revision/table/chunkHash/header`。流式大文件可借鉴secretstream分块。[S102]

### 与随机访问

每chunk独立AEAD可随机取；整pack单AEAD需要从头验证或复杂分段，不适合Range。目录可明文但签名，或单独加密；不要把offset目录藏起来却仍期待CDN Range无需前置下载。

### 密钥

- 客户端密钥只能是混淆/会话下发，无法对拥有客户端执行权的攻击者保密；
- 生产签名私钥绝不在客户端，客户端只持公钥；
- 服务端敏感表用访问控制，不靠共享客户端密钥。

## E.10 规模拐点与征兆

没有普适“X MB必崩”。用预算触发：

- **全量对象化拐点：** 配置分配开始触发多次Gen2/LOH、启动P95超过预算、热更峰值接近heap上限。
- **整表压缩拐点：** 单表cold decode超过可隐藏窗口，或一次更新必须重下/重解大量未变数据。
- **索引拐点：** 索引占resident的比例持续上升，10%数据驻留却仍常驻大半内存。
- **请求拐点：** 一个玩法warm-up需要数百小请求，RTT/headers超过payload。
- **字符串拐点：** decoded string cache成为最大类别，切换locale不能释放旧语言。

`Estimated`：将每次发布生成“表/分片 compressed/raw/index/string/cardinality/访问热度”清单，预算超限时自动建议拆shard或sidecar，比固定行数阈值可靠。

### 来源

[S033–S047, S053, S089, S094–S102, S108–S111, S120]

---
# F. 表分类与可见性切分：Server / Client / Voxel

**结论先行 1：** 可见性必须在编译IR阶段裁剪，不能把全量产物发到客户端后靠运行时“不读”。  
**结论先行 2：** 默认整表分类；列级分类只在同一业务实体必须共享ID/结构时使用，并由schema显式标注。  
**结论先行 3：** Voxel热表与普通Gameplay冷表访问模式不同：前者应预载、连续、数值化，并在Rust信任边界重新做内存安全校验。

## F.1 工业界常见切分形态

### 全量下发

实现最简单、两端表结构一致，但客户端拿到的数据即公开数据。`Verified`：客户端硬编码/静态资源中的秘密可以被逆向提取。[S101] 任何掉率、隐藏AI参数、防作弊阈值、经济风控规则，只要进入浏览器或移动包，就不能再视为秘密。

### 按端整表裁剪

schema或build target将表标为Server/Client/Shared/Voxel；编译器从同一IR生成不同manifest。优点是简单、审计清楚；缺点是共享实体可能重复结构，跨端引用需设计。

### 按列裁剪

同一源表的某些列只进入Server。Luban公开声明支持表/字段级分组与多端导出，证明这条工具路线存在。[S048,S049] 代价包括：

- 服务器/客户端行布局不同；
- 生成代码与schema投影不同；
- 同一行的Hash不能直接相同；
- ref验证需要知道目标投影；
- 新增列若漏标visibility可能泄露。

### 服务器私有表完全隔离

最安全。内容仓可同库，但发布流水线、artifact store和访问权限分离；客户端manifest根本不列出私有tableId。

## F.2 推荐的可见性模型

每个schema字段拥有：

```text
visibility: Server | Client | Voxel | Shared
sensitivity: Public | Internal | SecretForbidden
```

- `SecretForbidden`表示不能进入普通配表，贯彻已冻结的Secret分离。
- 整表有default visibility；字段只能缩小或显式扩展，扩展需review。
- 编译器生成三个projection graph：Server、Client、Voxel；Shared是各投影中的公共子集，不是第四份随意包。
- 可见性变更属于schema变更并进入审计，不是普通数据改动。

`Estimated`：默认整表。只有以下条件同时成立才列级切分：两端必须共享同一stable ID与大部分字段；拆表会造成大量重复引用；隐私review可自动化。否则拆成`ItemPublic`/`ItemServerRules`更清楚。

## F.3 同一真相，多份产物

发布事务应为：

```text
Source snapshot + Schema snapshot
  -> canonical typed IR
  -> validate complete source graph
  -> project(Server/Client/Voxel)
  -> validate each projection and cross-projection rules
  -> emit artifacts + manifests + hashes
  -> sign one Release Manifest binding all projection roots
```

Release Manifest至少记录：

- `SourceRootHash`
- `SchemaRootHash`
- `ServerProjectionRootHash`
- `ClientProjectionRootHash`
- `VoxelProjectionRootHash`
- `SharedPublicRootHash`
- compiler/profile/version、签名与过期/回滚元数据。

`Estimated`：禁止分别运行三个无关联的导出任务。一次编译要么产出全部投影，要么失败；CI验证工作区生成物可由源完全再生，防止“改了源只重编客户端”。

## F.4 跨分类引用

### 默认规则：编译期报错

客户端可见字段若`ref`到Server-only行，运行时无法验证/解引用，必须在编译期拒绝。Voxel到Gameplay也一样，除非依赖被明确发布到Voxel投影。

### 允许的降级：opaque ID

确实只需把ID传回服务器而不查看目标内容时，字段应明确声明为`opaque_id<Namespace>`，而不是伪装成可解引用`ref`。这不是当前冻结类型之一；兼容做法是仍用`u32/u64/string`，但在schema附加用途元数据并禁止本地dereference。若公共契约不允许扩展元数据，则拆列/拆表。

### 共享摘要

客户端需要展示而服务器保留规则时，编译生成public projection：例如`LootTablePublicHint`仅含展示等级，不含真实权重。不能从私有表运行时裁剪，因为字节已在客户端。

## F.5 作弊面与红线

公开资料很少有团队主动披露“掉率表被dump”的完整事故，不能编造具体产品名。可确定的原则是：

- WebAssembly、IL2CPP、加密bundle都不能让客户端秘密成为真正秘密；
- 服务器权威判定所需阈值、反作弊模型、隐藏生成规则留Server；
- 客户端预测所需参数一旦下发，视为公开，服务器仍独立校验；
- 加密只提高提取成本，不改变信任边界。[S101]

`Estimated`：CI应生成“client disclosure report”：表/列/示例值/新增可见字段；高敏visibility变更需要安全owner审批。

## F.6 Voxel专用表

### 典型形态与访问模式

- BlockType：稠密block ID→flags/material/collision/light。
- Material：渲染/物理材质ID、纹理索引、透明/遮挡属性。
- Drop/interaction映射：block ID→规则ID或服务器侧opaque hook。
- Meshing/lighting参数：频繁随机只读，通常处于内循环。

`Estimated`：这类表和普通Gameplay表不同：ID往往稠密、列少、访问频率极高、工作集稳定。首选`array[index]`或结构体数组/SoA，进入世界前全量预载并常驻；不应在mesh/light热路径触发网络、字典查找或UTF-8解码。

### Rust直读同一份二进制

可行条件：

- 明确little-endian、offset、alignment、UTF-8；
- 不持久化Rust默认struct layout。[S124–S126]
- 所有slice/view先验证bounds、length乘法溢出、enum值和UTF-8；
- buffer生命周期覆盖所有borrow；热更切换时旧buffer在读epoch结束前不释放。

FlatBuffers天然具相对offset和Rust/C#生成器；自研格式需达到同等验证纪律。[S019–S023]

### 谁负责校验

**明确结论：Rust不能仅信任C#“已经验证过”。**

原因：

1. 下载/解压字节可能绕过C#路径；
2. FFI长度/指针可传错；
3. C#业务校验不等于Rust内存安全校验；
4. Rust的unsafe view一旦越界，后果比普通配置错误严重。

推荐分层：C#/公共artifact层验签、hash、业务schema；Rust入口再次验证容器、chunk、offset、length、alignment、UTF-8和目标Voxel schemaVersion。验证通过后产生不可伪造的`VerifiedVoxelConfigHandle`。

### Revision对齐

- Release Manifest绑定Client和Voxel root；
- C# Stage成功后把同一Revision的Voxel manifest交给Rust stage；
- Rust预载/验证成功返回receipt；
- Tick Barrier由上层一次提交两个root；
- 任一侧失败，整次切换失败；
- 旧C#与Rust代都按同一epoch退休。

## F.7 配表与运行时状态的分界

### 可操作判据

一个数据应是“配表”，当且仅当大体满足：

- 在Release/Revision边界生成；
- 对同一Revision只读；
- 可由内容仓与schema完整重建；
- 不因单个玩家、实体、会话或实时世界事件变化；
- 两端若需要，预置各自投影，不靠状态同步获得。

一个数据应是“同步状态”，当它：

- 由运行时权威模拟产生；
- 每实体/玩家/会话不同；
- 随Tick、事件或玩家行为变化；
- 需要脏标记、AOI、快照/增量同步。

### 容易判错的例子1：技能定义 vs 技能冷却

技能的基础伤害、资源ID、公式ID是配表；某角色当前冷却剩余、层数、已应用modifier是运行时状态。公式如何求值见GAS专项，本报告只管定义的存取。

### 容易判错的例子2：体素类型 vs 体素世界

BlockType的碰撞/材质/光照常量是Voxel配表；世界中某坐标当前块ID、损坏度、流体状态是Rust权威运行时状态，属于ECS/存档/同步专项。

### 来源

[S019–S023, S048–S049, S061–S071, S101, S124–S126]

---

# G. 跨语言与跨运行时：Rust × C# × WASM × AOT

**结论先行 1：** 同字节双读的权威不是Rust结构体或C#类，而是独立schema、格式profile与跨语言golden corpus。  
**结论先行 2：** AOT/裁剪要求生成reader和显式注册；反射/动态代码只能做编辑器或CoreCLR辅助路径。  
**结论先行 3：** 数值一致性必须在导入时完成；运行时再按语言/区域解析十进制字符串会把确定性问题推迟到最危险的位置。

## G.1 三种跨语言做法

### 共享二进制 + 各语言reader

**优点：** 一个artifact hash、下载/缓存一次、Rust/C#可对同字节验证。  
**风险：** reader实现偏差、alignment/overflow、schema演进不一致。  
**适合：** 本画像推荐，前提是生成器+golden vectors。

### 各语言各自产物

**优点：** Rust可用rkyv，C#可用MasterMemory/MessagePack，各自最优。  
**风险：** 两份物理数据可能语义漂移；投影/Hash/replay更复杂；构建和存储翻倍。  
**适合：** Voxel完全独立且能用IR语义Hash对账时，作为优化分支。

### 一个语言读取，另一个透传API

例如C#加载全部表，Rust经FFI查询。**优点**是单reader；**缺点**是高频Voxel访问跨ABI、生命周期和线程开销。适合低频管理查询，不适合meshing/lighting热路径。

## G.2 多语言代码生成

推荐权威链：

```text
Schema DSL/AST
  -> normalized schema descriptor + schema hash
  -> Rust generated IDs/views
  -> C# generated IDs/views
  -> TypeScript/editor types (可选)
  -> cross-language test vectors
```

生成物是否入库：

- **CI必能重生。** 发布构建只信生成器版本+schema。
- **可入库供IDE/消费者。** 但CI比较无diff；不允许手改。
- 每份生成物嵌入同一`SchemaHash`，运行时reader拒绝不匹配artifact。

`Verified`：Protobuf、FlatBuffers、Thrift均采用schema→多语言生成；Luban/Tableau把该模式用于配表。[S014–S025,S031,S048,S049,S056]

## G.3 C# AOT / IL2CPP

`Verified`：Native AOT与trimming要求所有运行期代码可静态分析；System.Text.Json官方提供source generation来减少反射并适配AOT。[S091,S092] Unity IL2CPP不支持`Reflection.Emit`，反射访问的类型还可能被剥离，需要preserve/link配置。[S093]

**直接不可作为生产核心的形态：**

- 运行时生成serializer/accessor IL；
- 扫描所有程序集发现table type；
- 只通过字符串反射访问的构造器/字段；
- 根据数据动态闭包大量泛型实例；
- 依赖未保留的attribute metadata。

**推荐：**

- source generator输出静态reader、table registry和schema metadata；
- 按projection/usage group生成，不为每个不使用的表带完整API；
- 使用非泛型核心view + 轻量typed facade，控制AOT实例化；
- CI跑trimmed/AOT publish smoke test，而不只在编辑器Mono/CoreCLR测试。

MessagePack-CSharp公开说明source generator和AOT路径，MasterMemory以增量source generator为核心，都是可借鉴路线。[S010,S053]

## G.4 .NET WASM

- `Span<T>/Memory<T>`可以在连续buffer上做无复制切片。[S097,S120]
- Web Worker允许把部分C#工作移到worker，但目标浏览器与发布模式需验证。[S090]
- heap配置与浏览器设备限制意味着不能依赖桌面CoreCLR的可用内存。[S089]
- 浏览器网络/IndexedDB/Cache API是异步；同步`Get(id)`不能隐藏网络。

`Estimated`：C# reader应分两层：`async artifact/chunk provider`负责网络与缓存；`sync verified view`只对已驻留buffer做查询。不要让生成表属性返回`Task<T>`，而是显式prepare/lease。

## G.5 Rust侧约束

### 格式与unsafe

- `repr(Rust)`布局不稳定，跨语言持久化必须显式。[S124]
- 使用checked arithmetic计算`offset + count*stride`，先防整数溢出再slice。
- 如果使用`unsafe`把字节视为结构，应由一个小而审计过的模块封装，外部只拿安全view。
- rkyv的format controls会影响归档布局，升级时必须锁定/迁移。[S028,S029]

### FFI所有权

三种安全模型：

1. Rust拥有buffer，C#持opaque handle；
2. C#拥有pinned/native buffer，Rust借用且lease期间不可移动/释放；
3. 两边各自映射/加载同内容hash字节，不共享指针。

`Estimated`：浏览器WASM通常更适合一个线性内存内的明确owner；桌面/服务器可各自mmap同文件。无论哪种，Revision lease必须跨FFI可计数，不能在切换后释放仍被另一语言借用的旧buffer。

## G.6 数值表示

### 整数

- schema导入时检查范围，拒绝截断；
- 明确溢出语义：配置值参与运行时算术时，业务层使用checked/saturating/wrapping哪一种不属于存储格式，但测试向量必须覆盖边界；
- 不把JSON number直接先转C# `double`再转`u64`，因为大整数可能丢精度；解析原始token为目标整数。

### 浮点

风险链：

```text
Excel内部数值/显示文本
 -> 导出十进制
 -> 编译器解析为binary64或binary32
 -> 物理编码
 -> Rust/C#读取
 -> 运算与Hash
```

应固定：

- 读取原始值还是显示文本；
- 十进制语法和Invariant Culture；
- `f32`列在编译器阶段立即舍入为IEEE 32位并记录bits；
- 拒绝NaN/±Infinity，规范化或拒绝`-0`；
- canonical Hash对浮点采用固定bits或规范化十进制，不依赖语言`ToString`默认。

`Verified`：JCS规定基于IEEE754/ECMAScript数值序列化，但不能替代`f32`先舍入的业务规则。[S002] .NET提供Invariant格式化，但格式字符串/运行时版本仍需固定测试。[S123]

### 定点数

**不是所有浮点列都必须定点化。**

推荐分级：

- 参与权威战斗、经济、掉落、Replay分支的数：使用`i32/i64` + schema scale，例如`scale=10000`；
- 纯展示、材质、客户端视觉参数：可保留f32/f64；
- 物理/体素光照若跨端参与状态Hash，需专项决定固定点或严格确定性数学。

当前冻结类型没有decimal/fixed；兼容做法是以整数列存储并在schema/生成API附加scale，C#暴露`Fixed32`值对象而非裸int。若附加元数据也被冻结，则用命名与独立schema文档，但这是较差退路。

## G.7 时间、本地化和路径

### 时间

不要把Excel日期serial或本地时区日期直接进运行时。保存：

- UTC instant：`i64 Unix ticks/milliseconds`；
- 或日历日期：`YYYY-MM-DD`经编译成独立year/month/day整数；
- 时区ID另列并限定IANA/映射版本。

微软文档证明1900/1904系统差异真实存在。[S074]

### 本地化

主表保存稳定`LocKey`，语言文本按locale分chunk；不要在主表直接复制所有语言。回放/状态Hash通常包含LocKey而非渲染文本。Luban公开支持静态/动态本地化，说明工具链常把它作为一等能力。[S048]

### 路径

资源路径使用逻辑asset ID，不保存平台文件系统绝对路径；统一`/`、大小写和Unicode规则。生成时验证目标资源存在，运行时通过asset registry解析。

### 来源

[S002–S003, S010, S014–S032, S048–S056, S089–S097, S120–S126, S139]

---

# H. Schema 定义、类型系统与演进

**结论先行 1：** schema必须独立、版本化、可生成；把类型写在Excel表头可作为视图，但不应是唯一契约。  
**结论先行 2：** 当前冻结的标量+enum+ref足以启动，却缺少工业工具普遍具备的容器、嵌套、多态、nullable和本地化类型。  
**结论先行 3：** 在不改冻结契约的前提下，列表/字典优先正规化成子表，不允许用逗号字符串偷渡结构。

## H.1 Schema放在哪里

| 位置 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| Excel表头内联 | 策划所见即类型，复制表方便 | 二进制diff、跨表复用差、易被误改 | 可作生成视图，不作唯一权威 |
| 独立schema文件 | 可review、代码生成、多源共用、演进清晰 | 需工具把错误映射回表格 | **推荐权威** |
| 代码attribute/class | IDE重构强、运行时类型自然 | 非程序角色难改；多语言权威冲突 | 可作生成产物，不作多语言源 |
| 数据库metadata | 查询与迁移成熟 | 工具/发布绑定DB；文本review弱 | SQLite方案可用，仍导出schema快照 |

Protobuf/Thrift/Avro/FlatBuffers均以独立schema为核心；Luban/xresloader/Tableau也公开采用schema驱动。[S014–S032,S048–S056]

## H.2 类型能力对照

| 能力 | 冻结画像 | Luban公开能力 | Protobuf/Avro/FlatBuffers | 缺失后常见退化 |
|---|---|---|---|---|
| 标量/enum/ref | 有 | 有 | 标量/enum/message refs | 基础足够 |
| list/array | 无 | 有 | repeated/array/vector | `1,2,3`字符串、固定宽列展开 |
| map/dictionary | 无 | 有 | map或数组记录 | `k:v;k:v`自定义语法 |
| nested struct | 无 | 有 | message/record/table | 超宽表、重复列前缀 |
| nullable/union | 无 | 有 | optional/union/oneof | 空单元格语义混乱 |
| polymorphism | 无 | 有 | oneof/union/多表组合 | 类型列+大量空列 |
| localization type | 无 | 有公开支持 | 通常应用层 | 主表复制多语言/字符串ID混用 |

`Verified/Reported`：Luban官方文档公开列出容器、嵌套、继承/多态、可空和本地化；Protobuf/Avro/FlatBuffers规范有repeated/array/union/vector/nested能力。[S017,S021,S025,S032,S048,S049]

## H.3 当前类型缺口的兼容表达

### 列表

不要存`"1001,1002,1003"`。用子表：

```text
DropList(id, ...)
DropEntry(ownerDropListId ref, ordinal u32, itemId ref, weight u32)
```

`ordinal`确保顺序；编译器可在运行时binary中重新聚合成连续vector，而源/契约仍只用标量+ref。

### 字典

```text
StatMap(id)
StatEntry(ownerId ref, key enum/string, value i64)
```

加`unique(ownerId,key)`业务校验。若key集合闭合，优先enum或拆固定列。

### 嵌套与多态

- 分表：`AbilityBase` + `ProjectileAbility` / `AuraAbility`侧表，类型列决定唯一一个侧表；
- 编译期检查1:1与穷尽；
- runtime可生成discriminated view，但逻辑IR仍是多表关系。

这比宽表大量空列可审计，也比字符串嵌JSON安全。

## H.4 默认值、空单元格、空字符串、null

这是最危险语义之一。建议源层四态：

1. `Missing`：列/单元格未提供；
2. `ExplicitEmptyString`：字符串值长度0；
3. `ExplicitNull`：只有schema允许nullable时合法；当前冻结类型应拒绝；
4. `ExplicitValue`。

编译后再产生：

5. `DefaultedValue`：optional且schema有default时由编译器填充。

### Hash策略

有两种合理策略：

- **语义Hash：** explicit default与missing→default相同；有利于“不因补写默认值改变内容身份”。
- **源审计Hash：** 两者不同；有利于追踪作者意图。

`Estimated`：同时保留两个Hash：`SourceRootHash`区分源写法，`ProjectionSemanticHash`对最终值统一。运行时与Replay使用语义Hash，审计使用源Hash。

### API暴露

当前冻结运行时表经过完整验证和默认填充后，required/optional非nullable列都应返回确定值；不要在热API里再次返回`object?`。诊断API可查询`ValueOrigin = Explicit | Defaulted | Override(layer)`。

## H.5 Schema演进规则

### 加列

- optional + default：通常向后兼容；
- required无default：旧数据不兼容；已冻结默认不能补必填，必须迁移所有行。

### 删列

- 保留字段ordinal/tag墓碑，不复用；
- 先deprecated一个或多个发布，再删除生成API；
- Replay/旧artifact reader按支持窗口保留兼容。

Protobuf明确要求不重用field number/name并建议reserve。[S016]

### 改类型

默认视为破坏性。即使wire兼容的整数类型，也可能有符号/范围/Hash语义变化。正确做法：新列/新field ID、迁移、双读验证、淘汰旧列。

### 改enum

- 新增值：旧reader遇到未知值的行为需定义；
- 重命名：保持数值/ID，显示名可变；
- 删除：墓碑；不复用旧值；
- 改语义但ID不变是数据迁移，不应伪装为重命名。

### 改默认值

这是**全行语义变化**：所有missing行都会改变。编译器必须生成影响行数/ID摘要，不能在review里只显示schema一行变化。

### 旧存档

存档通常引用stable config ID，不嵌表结构。读取旧存档时：

- 使用存档/Replay钉住的config revision；或
- 运行迁移映射`oldId→newId/tombstone`；
- 找不到旧revision且无迁移，明确失败，不能把ID复用到新行。

## H.6 多态与继承

工业表达：

1. 宽表留空：编辑直观，空值多、校验弱；
2. 分表：关系清楚，查询多一次；
3. type列 + union/variant：运行时紧凑，schema/编辑器复杂；
4. base + side tables：在当前冻结标量系统下最稳。

`Estimated`：源与IR采用base+side tables，binary生成阶段可折叠为tagged union。这样未来公共类型系统增加容器/variant时有迁移路径。

## H.7 本地化

建议：

```text
Main table: nameLocKey, descriptionLocKey
Locale manifest: locale -> chunk hashes
Locale table: locKey -> UTF-8 text / rich text sidecar
Fallback chain: zh-Hans-CN -> zh-Hans -> en
```

- LocKey稳定且不复用；
- 每locale单独Hash，主Gameplay语义Hash只含LocKey；
- 语言切换是presentation资源切换，不改变权威模拟Revision；
- 服务器一般不加载正文，只加载必要key/审计文本。

Unreal社区公开过CSV string table key不稳定问题讨论，说明本地化键生成若依赖导入器/路径可制造长期引用事故。[S065]

### 来源

[S008, S014–S032, S048–S056, S064–S065, S071]

---

# I. 引用完整性、ID 与索引

**结论先行 1：** 人类可读ID与运行时稠密ID可以并存；可读ID是源身份，运行时ordinal是某个Revision的优化，不能跨版本持久化。  
**结论先行 2：** 删除ID默认永不复用；旧存档、Replay和跨服务引用要求墓碑与迁移映射。  
**结论先行 3：** 懒加载不应把引用完整性推迟到首次访问；编译器应在完整IR图上验证，运行时只按已验证索引加载。

## I.1 ID体系

### 字符串ID

优点：可读、diff友好、跨团队命名空间自然。缺点：内存、比较、拼写、Unicode/大小写规范。

### 整数ID

优点：紧凑、数组/索引快。缺点：手工分配冲突、review语义差、错误复用危险。

### 双ID推荐

```text
sourceId: "item.weapon.iron_sword"   # stable, canonical ASCII/normalized string
stableNumericId: u64                  # 可选，持久跨版本，不复用
revisionOrdinal: u32                  # 编译期稠密，仅当前artifact内部
```

- `ref`在源中可写sourceId；编译器解析到stableNumericId或revisionOrdinal；
- 存档/网络/Replay使用stableNumericId或sourceId，不使用revisionOrdinal；
- 运行时热表以ordinal数组访问；调试器可反查可读ID。

## I.2 字符串ID编译为稠密整数

收益：

- ref列从重复字符串变4字节ordinal；
- 比较、索引和cache局部性改善；
- 引用不存在在编译期失败。

代价：

- 每Revision ordinal可能变化；
- 日志只打印数字难调试；
- 错误把ordinal持久化会让旧存档指向错误行。

`Estimated`：生成`ordinal↔stableId↔debugName`映射；Release构建可把debugName放可选诊断chunk，服务器保留，客户端按预算决定。

## I.3 稳定性、删除与墓碑

规则：

1. stable ID永不复用；
2. 删除先标`deprecatedSince`，再`removedSince`；
3. 发布保留`tombstone(id, replacementId?, reason)`；
4. 编译器拒绝新行使用墓碑ID；
5. 旧存档迁移显式决定“替换/退款/删除/失败”。

`Verified`：Protobuf要求字段号/枚举值不要复用，理由包括数据损坏与兼容问题；Godot ResourceUID通过稳定ID保持移动/改名后的引用。[S016,S071] 这虽不是游戏行ID的同一机制，却支持“稳定标识与名称/位置分离”的设计原则。

## I.4 引用图

编译器建立有向图：

- 节点：`table,row`或至少table；
- 边：ref字段，带source location和visibility；
- 检查：目标存在、类型/命名空间匹配、投影可见、deprecated警告、循环策略。

### 循环引用

不是所有循环都错：Item↔Recipe可能合法；继承/base链循环通常错误。schema为ref标注：

- `acyclic`：编译器拓扑检查；
- `runtime_link`：允许循环，但加载器不得递归同步materialize；
- `weak/opaque`：不要求本地目标存在。

## I.5 懒加载下的引用完整性

**编译期全图验证，运行时不重复“是否存在”的业务判断。**

当A行ref到尚未驻留的B表：

- ref值本身可作为stable ID/ordinal读取；
- 真正dereference使用`TryResolve`，若B未resident返回`NotPrepared`而非“引用不存在”；
- 确定性逻辑进入前的UsageSet应把需要遍历的依赖closure预取；
- 禁止`GetA().B.C`在属性getter里同步网络加载。

## I.6 多人ID冲突

方案优先级：

1. 可读命名空间ID（`team/domain/name`）+编译期全局唯一；
2. 中央ID服务或数据库sequence；
3. 号段分配；
4. 随机UUID（冲突低但内存/可读性差）；
5. 手工自增整数（最易冲突）。

`Estimated`：内容仓离线协作优先namespaced string + 构建期分配stable u64；分配映射本身入版本库且只追加。不要按当前排序自动产生跨版本stable ID，只能产生revisionOrdinal。

## I.7 查询能力边界

默认运行时提供：

- 主键点查；
- deterministic全表枚举；
- schema声明的少量secondary unique/non-unique/range index；
- 不提供任意字符串谓词或LINQ全表扫描作为热路径API。

新增索引必须提交“查询调用点、频率、基数、resident成本”。若查询高度动态、组合多、数据很大，SQLite可能比不断自研二级索引合理。[S110]

### 来源

[S014–S018, S039, S071, S110, S127–S130]

---

# J. 确定性、Hash 与两端一致性

**结论先行 1：** 配置身份应Hash“规范化逻辑树”，不Hash任意序列化器输出。  
**结论先行 2：** 两端持有不同子集时，不可能诚实比较同一个全量字节Hash；必须比较投影根与共享子树。  
**结论先行 3：** 懒加载顺序不影响Hash：Revision root在manifest签名时已固定，缓存驻留只是性能状态。

## J.1 配置参与状态哈希的层级

公开引擎文档普遍描述内容版本、catalog/hash或资源ID，但很少公开完整“配置逐行进入确定性状态Hash”的商业实现。Unity Addressables update build使用catalog/hash与不变bundle复用，是内容版本锁定的相邻证据。[S068] TUF规范专门防rollback、freeze、mix-and-match，也证明多元数据版本组合错误是实际供应链威胁。[S098]

`Estimated`：本画像采用三层：

1. **Release root**：绑定server/client/config/content/protocol等manifest；
2. **Projection root**：Server/Client/Voxel各自完整配置；
3. **Table/chunk/row logical hashes**：用于diff、诊断、增量与Merkle proof；运行时状态Hash通常只混入projection root/revision，而不是每Tick重Hash全部表。

## J.2 Canonical逻辑编码

推荐domain-separated Merkle规则：

```text
H("cfg-schema-v1" || canonicalSchema)
H("cfg-row-v1" || tableId || stableRowId || presence/value events...)
H("cfg-table-v1" || tableId || schemaHash || orderedRowHashes)
H("cfg-projection-v1" || projectionId || orderedTableHashes)
H("cfg-release-v1" || sourceRoot || schemaRoot || projectionRoots || protocol...)
```

所有长度使用固定varint或固定宽度，字符串按UTF-8 byte length，列表按ordinal，表/行按stable key排序。Hash算法和profile版本在header/manifest中显式记录。

## J.3 Canonical坑逐条

1. **map顺序。** JSON object、MessagePack map、Protobuf map都不能依赖运行时遍历。按schema ordinal/规范化key排序。[S001,S009,S018]
2. **optional省略 vs 显式默认。** Protobuf presence文档说明这两者可能不同；必须在IR层决定Hash语义。[S015]
3. **浮点多种表示。** `1`、`1.0`、`1e0`文本不同；NaN payload多种；`-0`与`0`比较/位模式不同。导入后Hash规范位模式。
4. **整数宽度。** MessagePack/CBOR可用多种合法宽度；逻辑事件携带schema type而不是原始tag。
5. **字符串Unicode。** JCS不自动归一化字符串；视觉等价NFC/NFD可有不同bytes。入口选择NFC或严格保持并禁止混用。[S002,S003]
6. **换行。** CRLF/LF、末尾换行不进入逻辑值；多行字符串是否规范化另定。
7. **未知字段。** 冻结规则为拒绝，避免一个reader忽略而另一readerHash进来。
8. **默认值变更。** 即使源文件未改行，语义值全体变化；table hash必须变化。
9. **字符串池/offset。** 物理pool顺序、压缩、page size不进入逻辑Hash。
10. **时间与区域。** Excel date serial和本地格式先归一为明确UTC/日历值。
11. **排序稳定性。** 比较字符串按UTF-8 code unit或明确Unicode collation profile，不能用CurrentCulture。[S123]
12. **重复键。** JSON RFC对重复object name互操作有风险；编译器直接拒绝。[S001]

## J.4 为什么不直接Hash格式bytes

- Protobuf官方明确deterministic不等于canonical。[S018]
- FlatBuffers builder可有不同合法布局/字段构建次序。[S020]
- SQLite page分配、VACUUM/index构建可改变文件bytes而逻辑行相同。[S039]
- Parquet row group/page/codec选择产生不同文件。[S037]

物理bytes仍需`ArtifactHash`用于下载完整性；逻辑`SemanticHash`用于版本对账。两者都保留，不能混名。

## J.5 两端子集对账

### 方案一：各投影独立Hash + 共享公共Hash（推荐）

握手清单：

```text
ReleaseId
SchemaRootHash
ServerProjectionRootHash   # 客户端只接收期望值，不自行重算
ClientProjectionRootHash   # 客户端本地重算/验证
SharedPublicRootHash       # 两端都重算
VoxelProjectionRootHash    # 需要时由Rust验收回执
```

服务器校验客户端提交的Client/Shared root是否属于允许Release；客户端不证明自己拥有Server私有数据。

### 方案二：Merkle投影

Source tree的叶子带visibility；每端获得自己叶子和必要的兄弟hash，可证明其投影来自同一source root。`Estimated`：密码学上可行，但实现/调试复杂；只有需要第三方证明或分布式内容服务时值得。普通握手用签名Release Manifest已足够。

### 方案三：只对公共子集Hash

简单，但无法检测客户端专属UI/表现配置错包；因此仍需Client projection hash。

**明确结论：** 使用方案一；manifest签名把三个投影绑定为同一发布。Merkle proof作为后续优化预留。

## J.6 浮点确定性

配置Hash可对编译后bits一致，但运行时计算仍可能因运算顺序、FMA、平台数学库不同而分歧；这属于确定性/GAS实现问题。本报告在存储层的责任是：

- 同一源十进制编译成唯一f32/f64 bits；
- Rust/C#读取bits相同；
- 不在各端重新parse原始十进制；
- 权威分支数值优先定点整数；
- Hash不使用语言默认格式化。

## J.7 加载顺序无关性

- `ProjectionRootHash`来自manifest，不由“当前resident tables”增量累积；
- 行枚举按stable顺序，不按hash map插入/加载顺序；
- derived cache只能由纯函数从table值构建，结果排序固定；
- background load完成顺序不影响哪一Revision可见；只有Tick Barrier切根；
- 缺chunk是`NotPrepared/fail-stop`，不能回退默认值继续模拟。

## J.8 Replay与历史配置

Replay头记录：

- ReleaseId、ConfigRevision；
- Server/Shared/Voxel projection roots；
- schema/logic version；
- 必要时每玩法usage set或artifact retention locator。

历史配置保存：

- artifact/chunk内容寻址，不覆盖；
- retention至少覆盖最长Replay/客服/审计周期；
- compiler/reader兼容窗口或离线迁移工具；
- 若删除旧artifact，Replay必须先做“配置冻结包”归档。

TUF的rollback/freeze/mix-and-match威胁模型可直接借鉴：manifest需版本、过期、签名和一致性检查，不能只验证某个chunk hash就接受任意旧组合。[S098]

### 来源

[S001–S003, S009, S015–S020, S037–S039, S068, S098, S123, S139–S140]

---
# K. 热更、Revision 与激活语义

**结论先行 1：** 热更的最小安全单位不是“文件”，而是一个签名、完整、可准备的Revision根。  
**结论先行 2：** 原子激活只能保证读者看到旧代或新代；不能自动撤销按新配置已经发生的业务副作用。  
**结论先行 3：** 懒加载系统必须把Revision写进manifest、future、cache key和row/table handle；少一处就有混代入口。

## K.1 工业界热更形态

| 形态 | 生效/回滚 | 优点 | 主要风险 |
|---|---|---|---|
| 整包替换 | 慢；回滚清楚 | 一致性最简单 | 下载/峰值大，未变内容重复 |
| 增量补丁 | 快；依赖base | 节省流量 | base错配、补丁链、混包 |
| 单表替换 | 快 | 操作直观 | 跨表引用和同Tick一致性破坏 |
| 内容寻址chunk + 新manifest | 快；回滚切旧root | 未变chunk复用、一代一个完整视图 | manifest/保留/代际缓存更复杂 |
| 远端参数服务/feature flag | 秒级 | 审计和灰度灵活 | 不适合大量强类型表；网络与状态语义 |

Unity Addressables的内容更新构建会生成catalog/hash，并复用未变化bundle、为变化内容生成新bundle名称，是“immutable content + new manifest”相邻先例。[S068]

**推荐：** 对外逻辑是整Revision切换；物理传输是内容寻址chunk增量。不要暴露“直接替换Active某一张表”的生产API。

## K.2 激活原子性

### 读者应看到什么

- 在Barrier之前打开的`SnapshotLease`始终看到旧Revision；
- Barrier之后新开的lease看到新Revision；
- 一个lease内跨多表查询不能跨代；
- 读者无需持全局锁遍历所有表。

实现可选：

- 原子root pointer + immutable graph；
- RCU/epoch；
- generation counter + refcount；
- 读写锁（简单但热路径争用）。

`Estimated`：推荐root pointer + epoch/refcount。C#层可用不可变root对象和原子交换；Rust层使用Arc/epoch；跨FFI由统一revision lease持有两侧token。

## K.3 旧Revision对象在切换后继续存在

**明确规定：这是允许的特性，但不能悄悄参与新Tick。**

- 长生命周期业务对象不应缓存`RowView`；应缓存stable ID，需要值时从当前Tick的snapshot解析。
- 临时计算可在一个snapshot lease内持有旧view并完成。
- UI异步请求带Revision；结果返回时若UI已经切代，可以丢弃或标“旧数据”。
- 任何跨Tick缓存的派生值记录生成Revision；新Tick若Revision不同则重建。

## K.4 回滚

### 配置层回滚

内容寻址保留旧manifest/chunk，Barrier切回旧root即可。前提：旧artifact未被GC、reader仍兼容、签名/有效期策略允许应急回滚。

### 业务副作用不可自动回滚

如果新配置已导致：发奖、扣费、掉落、世界生成、持久化状态变化，切回旧配置不会逆转。需要：

- 发布前simulation/canary；
- 每个高风险表声明`sideEffectClass`；
- 运营补偿/迁移脚本；
- 事件日志记录使用的ConfigRevision；
- 必要时只允许新会话/新副本生效，而非全局即时切。

### 回滚窗口

`Estimated`：manifest记录`rollbackCompatibleFrom/To`或迁移策略。结构变更和内容变更分开：若新schema reader不兼容旧artifact，二进制回滚会失败，即使内容文件还在。

## K.5 签名与信任链

`Verified`：TUF把rollback、freeze、mix-and-match和无穷数据列为需防御威胁；支持角色、版本、过期和threshold签名。[S098] Sigstore/Cosign可对blob签名验证，SLSA provenance可记录产物由何构建过程产生。[S099,S100]

**适用于配置的最小链：**

1. 构建服务从固定source/schema/compiler生成projection；
2. 输出provenance、semantic roots、artifact hashes；
3. 离线/受控密钥签Release Manifest；
4. 客户端内置/轮换公钥，先验签manifest；
5. chunk按manifest hash校验；
6. 版本/过期/rollback policy验证；
7. 验签失败保持旧Active并上报，不降级到未签名生产配置。

签名不等于内容安全：仍要边界、解压比和schema校验。

## K.6 混合版本经典Bug

### Bug 1：旧请求完成写入“当前表缓存”

时间线：A代miss→发请求；切B；A请求完成→以tableId为key写缓存；B读到A字节。修复：cache key必须含chunk hash/revision，future捕获root。

### Bug 2：已加载表留旧，未加载表取新

若Active只是`Dictionary<tableId,object>`逐表替换，就会自然形成混合代。修复：root manifest一次切换，所有lookup先由lease选择root。

### Bug 3：共享判断只看tableId/schemaVersion

两代同schema但内容不同，误共享。修复：只有payload content hash与layout profile相同才共享。

### Bug 4：回滚后复用已变异派生缓存

配置对象只读，但二级索引/派生曲线被原地更新。修复：派生缓存也以`(revision, derivationVersion, inputHash)`命名，不原地改旧代。

## K.7 Incremental Patch格式

JSON Patch和JSON Merge Patch是标准化文本patch机制。[S132,S133] 适合review/AI proposal，但生产激活不应逐操作应用到Active：

- patch作用于source/IR；
- 编译器生成完整新Revision；
- 验证、预取、签名后切换；
- patch记录用于审计与传输优化，不成为运行时半完成状态。

## K.8 远端配置服务边界

Feature flags/小参数可服务化，但必须：

- 映射到同一schema和Revision；
- 拉取结果固化为签名manifest；
- 不在Tick中直接HTTP查询；
- 明确session-scoped override是否参与Replay/Hash；
- User/Session层覆盖若已冻结，调试器必须显示来源层和最终值。

### 来源

[S068, S098–S100, S132–S133, S138]

---

# L. 访问 API 形态：代码生成 vs 反射 vs 零拷贝 View

**结论先行 1：** 对AOT/WASM，生成强类型facade是默认公共API；底层可落在JSON对象、结构体数组、FlatBuffers或自研view。  
**结论先行 2：** 不要为每张表生成一套重型容器/字典/serializer；把通用算法集中在非泛型核心，生成物只携带schema常量与薄访问器。  
**结论先行 3：** `TryGet`不得触发I/O；异步加载属于revision/table准备API，而不是字段getter。

## L.1 三种API形态

| 形态 | 类型安全/IDE | 体积与启动 | 性能 | AOT | 最适用 |
|---|---|---|---|---|---|
| 强类型生成访问器 | 最强 | 生成代码可能膨胀 | 可内联/无字符串查找 | 最友好 | 生产Gameplay |
| 通用行/字段访问 | 弱，重构靠字符串 | 核心代码小，metadata大 | 字段名hash/类型分支 | 可做但易反射 | 编辑器、调试、AI工具 |
| 零拷贝view | 强度取决于生成器 | reader小到中 | 最低分配；间接offset | 友好 | 大表、WASM/Rust双读 |

推荐组合：生产用typed facade，调试器用generic metadata view，两者共享同一verified buffer与schema descriptor。

## L.2 生成API示例语义

```text
using var snapshot = config.OpenActiveSnapshot();
var items = snapshot.Tables.Item;
if (!items.TryGet(itemId, out var row)) { ... } // 只查resident data
var damage = row.Damage;                        // 已按schema类型读取
```

长字符串：

```text
var nameKey = row.NameLocKey;        // 热路径stable ID
var utf8 = localeTable.GetUtf8(nameKey);
```

避免：

```text
await config.GetTable("Item").GetRow(id).GetIntAsync("damage");
```

后者把网络、字符串查找、类型错误和生命周期隐藏在字段访问里。

## L.3 代码生成体积问题

没有找到可核的公开数据同时给“上千表、生成IL大小、WASM包体、AOT时间、启动类型初始化”的完整条件，不能给行业数字。

### 可复现成本模型

设：

- `T`张表；
- 每表`C`列；
- 每生成getter平均`g`字节IL/metadata；
- 每表registry/ctor/index wrapper平均`r`字节；
- AOT机器码放大因子`a`依工具链。

```text
Generated IL/metadata ≈ T*r + T*C*g
AOT native contribution ≈ reachable generic/template variants × a
```

真正危险的不是属性名，而是：

- 每表复制泛型hash实现；
- 每列生成反射metadata/attribute对象；
- 每表静态初始化大字典；
- 每种列类型×索引类型产生泛型组合；
- 全部表被一个静态registry引用，trimmer无法删除未使用组。

### 缓解

1. 通用`TableView`非泛型核心；生成table只保存schema ID/offset常量。
2. 以projection/usage assembly分组，客户端不生成Server表。
3. getter使用小型inline读函数，避免每列复杂校验重复；buffer在创建view时一次结构验证。
4. 索引算法共享；生成索引descriptor，不生成每表字典代码。
5. 不生成每行class；生成`readonly ref struct`/轻量value facade时注意不能跨async/heap逃逸。
6. 可选表通过显式registration列表，而不是assembly scan。
7. AOT发布后用link map/size report按schema group归因。

## L.4 反射/通用访问的合理位置

- 配表编辑器；
- semantic diff；
- console/GM查询；
- AI工具；
- schema浏览器；
- migration脚本。

这些路径可使用`tableId/columnId`和typed union，不代表运行时Gameplay也要每次按字符串查列。AOT下generic metadata应由生成器产生静态descriptor，避免Reflection.Emit。[S091–S093]

## L.5 生成物工程属性

- 生成物头写`DO NOT EDIT`、generator版本、schema hash；
- 可入库供IDE与包消费者，但CI重生后`git diff --exit-code`；
- 不把时间戳、绝对路径、机器名写入生成物，保证reproducible build；
- 生成失败时不沿用旧产物发布；
- schema变更同时生成兼容报告与API diff；
- C#和Rust生成器共享normalized schema AST，不各自解析一套DSL。

可复现构建资料强调消除时间、路径和环境等非确定输入。[S139,S140]

## L.6 热路径成本模型

一次`Get(id).Field`可拆为：

1. snapshot/root读取；
2. table descriptor读取；
3. index lookup（二分/hash/direct ordinal）；
4. chunk resident/pin检查；
5. row base定位；
6. offset/bit读取；
7. 可选UTF-8解码/对象分配。

比较：

- **生成对象+Dictionary：** 3为hash，6是字段读取，成本低但内存/GC高。
- **generic row：** 3+列名hash/metadata lookup+类型switch。
- **zero-copy view：** 3+offset/边界；字符串若只返回span无分配。

`Estimated`：对频繁ID点查，索引与字符串解码往往比“Protobuf vs FlatBuffers”格式标签更决定性能。基准需分开测numeric field与string decode。

## L.7 空值/缺失API

- 编译后required缺失不可能存在，否则artifact无效；
- optional有default，typed getter返回最终值；
- 若未来nullable，使用显式`Optional<T>/TryGetX`，不把null与missing混为一谈；
- ref解析使用`TryResolve`区分`NotPrepared`、`ProjectionUnavailable`、`InvalidArtifact`，不能都返回null；
- generic API返回`ConfigValue(kind,presence,origin)`。

### 来源

[S010, S019–S023, S053, S091–S093, S097, S120, S129, S139–S140]

---

# M. AI 友好的配表

**结论先行 1：** 公开证据厚度低：未找到成熟、可核的商业游戏“AI直接配表并生产激活”案例；本章架构建议统一标 `Estimated`。  
**结论先行 2：** AI最适合操作typed patch与工具API，不适合直接改二进制工作簿或拥有生产激活权。  
**结论先行 3：** 当前瓶颈排序是：校验/审计基础设施 > 接口形态 > 模型生成能力。

## M.1 证据厚度评估

`Verified`：OpenAI Structured Outputs可用JSON Schema约束输出形状，但官方也说明只支持schema子集、存在规模限制，且结构合法不等于值在业务上正确。[S103] MCP工具规范提供带input schema的工具发现/调用；SARIF提供结构化错误、位置、rule ID和结果交换的成熟相邻范式。[S104,S105]

`Reported`：找到一个Luban Config Skill Query→Patch实验仓库，说明社区在探索“查询/补丁”形态，但缺少商业生产、审计、规模和长期维护证据。[S106]

**明确声明：以下生产路线是基于相邻领域机制的工程推演，不是行业共识。**

## M.2 五种AI参与形态

### 1. AI直接读写本地xlsx/CSV

`Estimated`：模型代理可调用openpyxl/Office API，但xlsx的单元格坐标、公式、合并单元格、格式和自动类型转换让语义脆弱；openpyxl不计算公式。[S084] CSV更简单，却缺schema和多表事务。

**适合：** 一次性导入、清洗、生成草案。  
**不适合：** 直接提交生产权威。

### 2. AI经Google Sheets / Microsoft Graph API

`Verified`：Sheets API与Graph Excel API可程序化读写；Sheets有quota，且`RAW/USER_ENTERED`语义不同。[S079–S081,S113,S137]

**优点：** 权限、协作、版本历史、评论。  
**风险：** API限流、表格语义、外部服务、细粒度写导致部分成功。  
**要求：** AI只调用事务式业务工具，工具内部批量更新并返回revision，不让模型直接发任意cell update。

### 3. AI读写文本源

`Estimated`：这是最友好路线，因为LLM能直接理解diff、schema和上下文；Git天然提供review/rollback。文本仍需typed parser，不能让AI绕过schema。

推荐patch单位：稳定ID和column ID，而非行号：

```json
{
  "baseSourceHash": "...",
  "operations": [
    {"op":"replace", "tableId":"Item", "rowId":"item.iron_sword",
     "column":"damage", "old":12, "value":14,
     "reason":"close TTK gap in tier-1 simulation"}
  ]
}
```

JSON Patch是标准机制，但表格域应在其上增加stable row/column语义，避免数组index随排序变化。[S132]

### 4. AI经工具/函数调用操作管线（推荐）

工具集：

- `config.search(query, projection, revision)`
- `config.get_schema(tableId)`
- `config.get_rows(tableId, ids, columns)`
- `config.propose_patch(baseHash, operations)`
- `config.validate(patchId, validationLevel)`
- `config.compile_preview(patchId, targets)`
- `config.semantic_diff(base, candidate)`
- `config.run_simulation(candidate, suiteId)`
- `config.submit_for_review(candidate)`

AI不获得`activate_production`。人类/CI签名服务在审批后生成Release。

### 5. 编辑器内助手

`Estimated`：在专用编辑器中提供“解释错误、批量填充、引用搜索、影响分析、生成变体”，比开放整库更可控。编辑器只给模型当前schema/选中行/依赖摘要，降低上下文和泄露。

## M.3 Schema作为AI接口

冻结的列类型、范围、enum、refTarget、required/default正是结构化生成所需约束。需要补充：

- stable row ID规则；
- cross-field业务规则；
- visibility/permission；
- examples与反例；
- patchable/readonly字段；
- monetary/security risk class。

`Verified`：Structured Outputs当前官方限制包括JSON Schema子集、对象属性/嵌套/总字符串规模等约束，且常要求`additionalProperties:false`和显式required；具体限制会随产品更新，集成时必须以当时官方文档再核，不应把整个上千表schema一次塞入模型。[S103]

**策略：** schema按任务裁剪：先检索table/rows，再只发送相关字段与引用摘要。模型不需要知道全部1000张表。

## M.4 校验反馈闭环

### 结构化错误格式

借鉴SARIF，建议：

```text
code: CFG_REF_NOT_FOUND
severity: Error | Warning | Info
tableId, rowId, columnId
sourceUri, sheetName, cell, sourceSpan
schemaPath
actualValue, expectedConstraint
relatedLocations[]
message
fixHints[]
revision/baseHash
```

`Verified`：SARIF把rule、result、location、message结构化，适合工具链交换。[S105]

### 闭环步骤

```text
AI propose patch
 -> syntactic/schema validation
 -> table-local validation
 -> cross-table graph validation
 -> projection/security validation
 -> compile preview
 -> simulation/business validation
 -> semantic diff + risk summary
 -> human review
```

每轮修正必须基于新candidate hash，防止并发策划改动后AI覆盖旧基线。

### 错误信息质量

“invalid config”无法供人或AI修正。错误应给稳定code、精确位置、实际值、约束、相关目标和建议；但fix hint不能自动掩盖业务决定，例如缺ref不能随便创建目标行。

## M.5 审计与回滚

AI改动记录在旁路provenance，不污染业务列：

- actor/user/service；
- model/provider/version（如允许记录）；
- tool schema/version；
- task/prompt摘要或受控引用；
- baseHash、candidateHash、operations；
- validator/simulation结果；
- reviewer/approver；
- activation ReleaseId。

Git commit可承载一部分，但在线表格/编辑器也要写append-only audit log。NIST SSDF/SLSA强调可追溯构建与受控变更，可作为治理参考。[S100,S138]

## M.6 权限与安全

权限按表/列/动作：

- AI可提案：低风险平衡、文本、测试数据；
- AI只读：经济总开关、反作弊、安全阈值、签名策略；
- AI不可见：Secret与服务器敏感实现；
- AI不可做：绕过校验、签名、生产激活、删除审计。

高风险patch要求双人审批/owner；跨visibility改变单独安全review。AI输出视为不可信外部输入，仍受最大行数、最大patch、最大字符串与超时限制。

## M.7 数值平衡自动化

公开AI游戏配表案例不足，但非AI时代的自动simulation/参数搜索可作为基础：

- 固定seed、固定ConfigRevision跑战斗/经济模拟；
- 输出胜率、TTK、货币通胀、掉落分布、异常边界；
- AI只根据指标提出patch；
- 不能以单一平均值优化，需约束公平性、极端分位和回归suite；
- simulation版本、代码hash和随机种子进入结果provenance。

`Estimated`：没有可重复模拟器时，AI只能模仿文本和局部规律，最容易生成“schema合法、系统性错误”的配置。

## M.8 为AI友好需要付出的代价

- schema约束必须显式，隐含在策划口头/颜色/公式里的规则要迁出；
- 错误必须结构化；
- 源需可稳定patch，行号不能是身份；
- 复杂Excel公式要降级为编译器/模拟器中的可版本代码；
- 所有改动要可重放、可diff、可回滚；
- 工具API要权限、quota、审计和幂等。

**对人类策划的影响：总体是变好。** 同样的显式schema、错误定位、语义diff和模拟反馈也减少人工事故。代价主要是早期工具投入和放弃Excel自由技巧。

## M.9 关键判断题

### “让AI操作Excel” vs “文本权威、Excel视图”

**明确结论：文本权威+Excel视图明显更AI友好。** 前者可按stable ID生成小patch、Git审计、合并；直接操作Excel必须处理cell坐标、公式缓存、二进制冲突和自动转换。对人类策划，保留Excel视图可减少习惯冲击；真正改变的是发布写权限。

### 当前瓶颈是什么

**明确结论：第一是校验与审计，第二是接口，第三才是模型。** 结构化输出能保证形状，不保证引用、经济闭环和玩法体验。[S103] 没有精准validator、simulation和review，即使模型完美写入单元格也不能安全生产。

### 来源

[S079–S084, S100, S103–S106, S113, S131–S138]

---

# N. 工具链与工程化

**结论先行 1：** 配表采用率由“策划能否本地一键验证并得到可理解错误”决定，不能把整个开发SDK暴露给非工程角色。  
**结论先行 2：** 增量编译必须基于source/schema/reference/projection依赖图，不只是文件mtime。  
**结论先行 3：** 语义diff、层级覆盖追踪和当前Active诊断是生产能力，不是锦上添花。

## N.1 编译管线形态

### 本地CLI

核心、可复现、CI同命令。提供单一封装：

```text
config-tool validate --changed
config-tool build --target preview
config-tool diff --base main
```

策划不安装Rust/.NET SDK：发布自包含可执行或桌面壳，自动下载匹配compiler profile。

### IDE/Excel插件

只做交互与调用daemon，不复制编译逻辑：单元格错误、跳转ref、preview。LubanAssistant、Excel2TextDiff与watch实践说明这类需求真实。[S048,S059]

### CI服务

权威构建：干净环境、固定compiler、全图校验、全投影生成、reproducibility check、签名请求。

### Sheets webhook

收到变更只创建“待编译source snapshot”；不能直接改Active。批量拉取并固化hash，避免每个cell触发一次构建。[S079–S081,S113]

## N.2 建议流水线

```text
Fetch/Import
 -> Normalize source cells
 -> Parse against schema
 -> Build typed IR
 -> Layer merge with provenance
 -> Referential/business validation
 -> Projection
 -> Logical hash tree
 -> Runtime artifact emit
 -> Cross-reader golden verification
 -> Semantic diff/report
 -> Package/provenance/sign
```

任何一步失败都不产出“看似成功的旧文件”。输出目录先写临时内容寻址路径，全部成功后原子发布manifest。

## N.3 增量编译

### 依赖图节点

- source file/sheet/range；
- schema type/enum/validator；
- table IR；
- reference edge；
- projection；
- generated code group；
- runtime chunk/index；
- semantic report。

### 失效规则

- 改一行值：重解析该source partition，重验证入/出ref，重建所在chunk/table hash和受影响projection root；
- 改enum/schema：所有引用字段、代码生成和相关表失效；
- 改default：所有缺失该列的行语义失效；
- 改visibility：两个或多个projection重建并安全review；
- 改validator代码：其声明scope内重新验证，即使数据没变。

仅mtime缓存会漏掉compiler/schema版本；cache key应包含输入content hash、normalized schema hash、compiler profile和validator version。

公开工具声称增量/快速生成，但缺少统一测试条件，本报告不引用具体毫秒数字。[S048,S060]

## N.4 校验器分层

1. **语法/输入层：** 单元格类型、数字语法、Unicode、公式策略、未知列。
2. **schema层：** required/default/range/enum。
3. **表内层：** 主键唯一、组合唯一、排序/重复。
4. **跨表层：** ref、投影可见、循环/依赖。
5. **业务层：** 权重和、经济边界、曲线单调、资源存在。
6. **发布层：** 签名、revision、包大小、WASM预算、敏感字段。

错误分级：Error阻断；Warning需明确owner/可过期豁免；Info只提示。豁免包含rule ID、scope、owner、reason、expiry，不能永久`ignore all`。

xresloader维护者公开介绍外部Excel/文本验证器，Luban公开ref/range/path验证，说明验证扩展是成熟配表工具的核心而非附属。[S048,S050,S119]

## N.5 CI门禁

- schema lint与兼容性报告；
- changed-source快速验证；
- 全图ref/业务验证；
- 三projection生成；
- 生成物重生无diff；
- Rust/C# reader同golden corpus；
- canonical semantic hash一致；
- artifact安全fuzz corpus（坏offset/长度/压缩比）；
- client disclosure diff；
- size/memory/load benchmark阈值或趋势；
- provenance；
- 审批后签名。

对非工程角色的反馈应回到表/行/列或编辑器，不只发CI日志链接。

## N.6 本地预览与调试

运行时Inspector必须回答：

- 当前Active/Staged Revision和投影Hash；
- 某表/分片是否resident、大小、来源URL/cache、最后加载耗时；
- 某行某列最终值；
- 值来自Engine/Platform/Server/Product/Environment/User/Session哪一层；
- 哪一层覆盖了谁；
- 源文件/sheet/cell/commit；
- 是否explicit/defaulted；
- 旧/新Revision语义diff；
- 哪个usage set钉住该chunk。

`Estimated`：分层覆盖调试应把合并过程保留为provenance DAG或至少`WinningSource + ShadowedSources[]`。否则线上只看到最终数字，无法解释“为什么是这个值”。

## N.7 Diff与Review

### 需要三种diff

1. **源diff：** 文本/Excel textconv，展示编辑动作；
2. **语义diff：** 类型化的新增/删除/修改、默认值影响、ref变化；
3. **发布diff：** 每projection大小/chunk/hash/敏感可见性变化。

二进制artifact不直接review；生成：

```text
Item[item.iron_sword].damage: 12 -> 14
Affected derived rows: 0
Client visibility: unchanged
Server chunk: +84 bytes raw, reused 7/8 chunks
Risk: CombatBalance / simulation required
```

Git textconv能帮助查看二进制，但真正合并应在source/IR，不在生成物。[S134,S135]

## N.8 协作与权限

- 策划：修改允许域、跑本地校验/preview；
- 程序：schema、validator、runtime profile；
- QA：创建回归suite、对比Revision、验证回滚；
- Build/Release：可复现构建、签名、保留；
- Security/Economy owner：审批高风险visibility/经济表；
- AI：只读+提案，权限小于普通策划。

每张表有owner、reviewers、risk class、release cadence。频繁远端参数和版本内静态表不应强行同一发布节奏，但最终都要被Release/Session Revision钉住。

### 来源

[S048–S060, S077–S084, S100, S105, S113, S119, S132–S140]

---
# O. 规模与实测数据

**结论先行 1：** 公开世界缺少可直接套用的游戏配表规模基准；能找到的数字大多缺数据模型、硬件、冷暖状态或运行时版本。  
**结论先行 2：** 不以“多少行”单独决定架构；以启动预算、峰值内存、冷miss P99、请求数、索引占比和热更重叠峰值决定。  
**结论先行 3：** 选型前必须用委托方真实列分布和访问trace跑四后端、三平台、冷/暖/切Revision完整基准。

## O.1 找到的公开数字及可用性

| 来源 | 公开数字 | 条件完整性 | 本报告如何使用 |
|---|---|---|---|
| sql.js-httpvfs | README/演示关联约8百万行、约670MiB远端SQLite | 非游戏；浏览器/查询/网络细节不完整；项目自称PoC并列出驱逐/测试缺口 [S042] | 只证明大远端文件Range点查可行，不外推帧体验 |
| MasterMemory README | 声称相对SQLite高倍查询、示例DB体积222KB vs 3560KB | schema、查询、硬件、版本和完整benchmark harness条件不足 [S053,S143] | 仅作工具自述，不能进入决策评分数字 |
| Luban云生成Wiki | 声称日常增量约300ms、MMORPG项目约1s内 | 没有表数、硬件、缓存命中、compiler版本 [S141] | 说明其目标是增量/云缓存；不设委托方SLA |
| 中文Unity项目观察 | 摘要提到Lua配置启动后约150MB | 只读到摘要；数据规模、Lua版本、字符串/表结构不明 [S142] | 只作为“文本文件小、对象图可大”的事故线索 |
| Luban运行时教程 | 示例先把所有配置数据文件加载内存后构造Tables，支持async [S144] | 教程，不是规模测试 | 说明成熟导表工具默认示例也可能是全量加载，懒加载需额外架构 |

**结论：** 没有一组公开数字满足“游戏、浏览器.NET WASM、Rust/C#双读、冷启动、峰值内存、百万行、完整硬件/版本”。Known gap成立。

## O.2 规模拐点不是单一行数

### 全量加载不可接受的征兆

- 配置在首屏前全部下载/解析，而首屏只需要少数表；
- config解析占启动关键路径的显著份额；
- `Active + Staged + raw + object graph`接近浏览器heap高水位；
- 热更一次触发多次Gen2/LOH或浏览器OOM；
- 多语言字符串成为最大resident类别；
- 服务器多进程为同一表重复构建对象图。

### 懒加载开始产生可感知卡顿的征兆

- gameplay第一次`Get`出现网络/解压；
- P50很好、P99被cold miss主导；
- 一次usage warm-up有大量小HTTP请求；
- cache eviction后同一玩法反复抖动；
- IndexedDB/Cache命中率在移动端下降；
- 低端设备解压峰值与GC比网络更慢。

### 编译时间影响迭代的征兆

- 改一行触发全仓全表重编；
- schema/default改变时依赖失效不准确，团队被迫全量；
- 本地和CI生成结果不一致；
- 策划绕过校验、直接交给程序代改；
- 生成物大到review只看“几万个文件变化”。

## O.3 移动端与浏览器约束

不提供脱离产品的“通用内存预算”。正确方法是建立设备层级：

- 低端移动浏览器；
- 主流桌面浏览器；
- Dedicated Server容器；
- Unity IL2CPP目标机。

每档测：可用heap、线性内存增长失败点、后台/前台切换后缓存、IndexedDB quota/eviction、网络RTT/带宽、单线程/worker能力。MDN明确说明存储quota与驱逐因浏览器而异；.NET WASM heap也可配置而非无限。[S087,S089]

## O.4 性能测量的经典坑

1. **只测稳态查询。** 漏掉下载、解压、验证、索引构建、JIT/AOT初始化。
2. **只测反序列化吞吐。** 漏掉对象常驻和GC。
3. **用小表外推大表。** CPU cache、LOH、page fault、索引层级和网络请求模式会变。
4. **只测桌面CoreCLR。** .NET WASM/IL2CPP包体、内存和AOT行为不同。
5. **压缩输入已在OS cache。** 冒充冷启动。
6. **不测Revision重叠。** Active/Staged/old-retired同时存在才是峰值。
7. **只报平均。** 首触P95/P99和最大frame hitch更重要。
8. **把mmap建立映射当加载完成。** 首次触页仍有page fault。
9. **忽略字符串解码。** numeric benchmark无法代表本地化/路径表。
10. **忽略请求头和CDN行为。** 1KB Range在高RTT下可能比解压慢得多。
11. **不同格式使用不同schema/索引。** 比较不公平。
12. **不验证查询结果和Hash。** 快但语义不一致的reader没有价值。

## O.5 委托方应执行的基准数据集

### 数据规模

- `S`: 10,000行 × 20列；
- `M`: 100,000行 × 20列；
- `L`: 1,000,000行 × 20列；
- `XL`: 10个百万行级shard，仅在Server/桌面压力测试。

### 数据分布

- 数值密集；
- 4字符串列：平均12/64/256 bytes三档，唯一率10%/50%/100%；
- 低基数enum/ref；
- 10%/50%稀疏optional；
- ID顺序、随机和业务分区三种；
- ref图含热点目标与长尾。

### 候选后端

1. canonical JSON + source-generated reader；
2. FlatBuffers + sorted primary index + chunk；
3. 自研typed binary混合布局；
4. SQLite read-only indexed；
5. Arrow/Parquet仅作为分析对照。

### 访问trace

- 冷启动bootstrap；
- 随机10万次`Get(id).numeric`；
- 随机`Get(id).4 strings`；
- 顺序扫描一列/整行；
- 80/20热点Zipf；
- 场景usage warm-up；
- cache 25%/50%/100%容量；
- Active→Staged 1%/10%/100%变化；
- Replay固定revision；
- 缺chunk/坏hash/压缩炸弹失败路径。

## O.6 必测指标与定案门槛

| 类别 | 指标 | 定案方式 |
|---|---|---|
| 包体 | reader/AOT代码、manifest、index、compressed artifact | 与产品下载预算比较；分schema group归因 |
| 启动 | manifest verify、bootstrap fetch/decode/prepare | P50/P95/P99；低端设备硬门槛 |
| 冷miss | 网络RTT、bytes、decode、validate、publish | 禁止发生在权威Tick；UI可有单独SLA |
| 查询 | ns/op或ops/s、分配/op、cache miss | numeric/string分开；warm/cold分开 |
| 内存 | steady、peak activation、LOH/GC、WASM high-water | 按目标设备保留安全余量 |
| 网络 | 请求数、range放大、cache hit、重复下载 | usage set级衡量，不只单chunk |
| 编译 | changed row/table/schema/default/visibility | 本地交互与CI全量分别设SLA |
| 正确性 | Rust/C# semantic hash、golden queries | 必须100%一致，性能不换正确性 |
| 安全 | invalid corpus rejection、最大分配/解压比 | 所有reader一致fail-closed |

具体数值门槛必须由产品首屏/内存/网络预算填写，报告不凭空替代。

## O.7 建议基准执行顺序

1. 先用JSON基线确认IR、Hash和查询oracle；
2. 同一IR输出三候选；
3. 在CoreCLR服务器测CPU/内存上限；
4. 在浏览器.NET WASM测真实HTTP/Cache/IndexedDB；
5. 在Unity IL2CPP发布构建测包体与裁剪；
6. 引入真实访问trace而非均匀随机；
7. 压缩参数、chunk size做网格扫描；
8. 最后用故障与Revision切换测试验证语义。

### 来源

[S042, S053, S087, S089, S094, S141–S146]

---

# P. 具体方案深挖

**结论先行 1：** 选择八个方案覆盖“中文表格编译器、C#内存库、文本数据库、商业引擎资产、零拷贝、Range数据库、列存”七类能力。  
**结论先行 2：** 没有一个可原样照搬；价值在于分别抄schema、多端分组、typed索引、文本协作、异步生命周期、relative offset、页索引与列编码。  
**结论先行 3：** 排除方案并非“差”，而是与Rust+C#+WASM+确定性+懒加载的约束组合不匹配。

## P.1 选择与排除

### 入选

1. Luban：最完整的中文游戏schema/多源/多端/多格式工具代表。
2. xresloader：Protobuf驱动Excel转表和读表生态代表。
3. MasterMemory：C# source generator + immutable typed in-memory DB代表。
4. CastleDB：文本权威+专用编辑器+RCS协作代表。
5. Unreal DataTable/Data Registry：商业引擎资产、异步、缓存与多源覆盖代表。
6. FlatBuffers：Rust/C#共同零拷贝候选。
7. SQLite + sql.js-httpvfs：B-tree/page cache/浏览器Range代表。
8. Arrow/Parquet：列存、页/编码/列裁剪代表。

### 排除但保留参考

- **Cap'n Proto：** canonical规范有价值，但C#实现生态风险高于FlatBuffers。[S024–S027]
- **rkyv/bincode：** Rust私有优秀，跨C#共享不匹配。[S028–S030]
- **Protobuf单独深挖：** 已作为xresloader与格式章节核心基础，不再占独立方案名额。
- **Unity ScriptableObject/Addressables、Godot Resource：** 已在A/D章对照，跨Rust共享更弱；Unreal Data Registry公开的缓存/数据源语义更贴近配表。
- **Tableau：** 与Luban/xresloader谱系重叠，公开运行时/规模资料较少。[S056]

## P.2 Luban

1. **定位与流派：** `Reported`，schema驱动的多源游戏配置编译器，属于“表格源+转换器+代码生成”。[S048,S049]
2. **权威源与流程：** 支持Excel族、JSON/XML/YAML/Lua等；统一模型后校验、生成代码和数据。权威可以是多源，不必只Excel。
3. **中间/运行时格式：** 公开支持binary、JSON、Protobuf、MessagePack、FlatBuffers及自定义模板；说明前端和后端可分离。
4. **Schema/type：** 公开列出标量、enum、ref、list/set/map、结构、继承/多态、nullable、本地化、external type。
5. **加载/懒加载：** 文档示例常先把数据文件加载后构造Tables，也支持异步加载；原生“跨WASM网络分片懒加载”不是已核核心能力。[S144]
6. **压缩/内存：** 后端可定制；公开材料不足以证明默认chunk/seek/string pool策略。
7. **多语言：** 公开支持C#、C++、Java、Go、Lua、TS等，旧README还列Rust；具体当前模板质量需按目标版本验证。
8. **热更/版本：** 公开声明原子热更新、main+patches与watch；未核到适配本画像Tick Barrier的源码语义。
9. **工具/调试：** Excel2TextDiff、LubanAssistant、ref/range/path校验、增量/云生成是亮点。
10. **规模/采用：** 官方Wiki声称大型/MMORPG增量很快，但缺完整条件；公开命名商业采用不足。[S141]
11. **最值得抄：** **统一typed IR + 可插拔source/code/data targets + 表/字段分组。** 这正好让Excel、文本、Rust/C#产物与三投影解耦。
12. **最不该抄：** 不要因为工具支持很多后端就把后端可切换误解为运行时语义已稳定；必须自行冻结canonical Hash、chunk、lazy与Revision handle。

## P.3 xresloader

1. **定位：** `Reported`，以Protobuf schema映射Excel并导出多格式的跨平台游戏转表套件。[S050,S051]
2. **源/流程：** Excel配置字段映射，CLI/GUI批量转换；配套conf、code generator和dump工具。
3. **格式：** Protobuf binary、MsgPack、JSON、Lua、JS、XML、UE DataTable等。
4. **Schema/type：** 核心借助Protobuf message/enum/repeated/map/oneof及扩展配置；具体限制依版本。
5. **加载：** xres-code-generator提供多语言读表生成/加载路径；默认主键索引/懒加载细节未核稳定源码坐标。[S052]
6. **压缩/内存：** 非其最突出公开能力；需要外层容器/运行库决定。
7. **多语言：** Protobuf生态强；Rust/C#生成均成熟，但runtime object materialization需控制。
8. **热更：** code generator公开有reload线索，精确原子性未核；本画像需另加Revision根。
9. **工具：** GUI/CLI、validator、二进制dump有利于非工程角色和诊断。[S050,S119]
10. **规模/采用：** 维护者长期中文技术文章说明持续演进；商业产品/规模未公开。
11. **最值得抄：** **把schema、Excel映射、批量配置、reader generation和human-readable dump组成完整套件。** 二进制没有dump就无法运营。
12. **最不该抄：** 不要把Protobuf wire bytes当Config canonical Hash；官方明确否定该假设。[S018]

## P.4 MasterMemory

1. **定位：** `Reported`，.NET/Unity的source-generated typed readonly in-memory database。[S053,S143]
2. **源/流程：** 以C#类型/schema为中心，从CSV等构建MessagePack数据库，source generator生成访问API。
3. **格式：** MessagePack承载，运行时构建/载入内存DB。
4. **Schema/type：** C# class/attributes，主键、二级/范围索引、validator；跨语言不是主目标。
5. **加载：** 核心是内存数据库，不是浏览器网络lazy；可借鉴一次build后immutable查询。
6. **压缩/内存：** 公开强调小DB、string interning和zero allocation query，但具体数字条件不足，需自测。
7. **多语言：** C#强；Rust同字节弱。Unity最低版本当前公开为2022.3.12f1以支持增量source generator。
8. **热更：** immutable DB可整实例替换；本画像需增加Staged/Active与chunk共享。
9. **工具：** source generator、typed indexes、validator、diagnostics。
10. **规模/采用：** NuGet 3.0.4可核；没有满足本报告条件的商业规模数据。
11. **最值得抄：** **索引声明生成typed API、字符串驻留和immutable数据库实例。** 对C# Gameplay访问面价值高。
12. **最不该抄：** 不要全量materialize为C#中心DB后让Rust经FFI频繁查询；会损失双读与WASM懒加载。

## P.5 CastleDB

1. **定位：** `Verified/Reported`，专用结构化静态数据库/编辑器，文本DSL派。[S054,S055]
2. **源/流程：** 编辑器写JSON-with-newlines文件，本地提交到Git/SVN。
3. **格式：** 人类可读JSON为权威；运行时通常由各项目加载/生成。
4. **Schema/type：** 列、sheet和结构化数据，表达力以工具版本为准；不等于Protobuf级跨语言IDL。
5. **加载：** 不以大规模网络lazy为核心。
6. **压缩/内存：** 交给宿主；没有公开统一策略。
7. **多语言：** Haxe生态友好，其他语言需导出/loader。
8. **热更：** 文本版本与Git回滚清楚，生产原子激活需另建。
9. **工具：** 专用编辑器、地图编辑、本地实验、diff/merge是核心价值。
10. **规模/采用：** 官网版本1.5；社区关联Evoland系列但一手采用复盘不足。
11. **最值得抄：** **权威文件天然可diff/merge，编辑器只是结构化视图。** 这是AI和批量重构友好的长期架构。
12. **最不该抄：** 不要直接把单个大JSON文件当WASM运行时artifact；仍需编译、索引、分块和强校验。

## P.6 Unreal DataTable + Data Registry

1. **定位：** `Verified`，引擎内建数据资产派；DataTable以行结构驱动，Data Registry在多源上提供统一只读访问。[S061,S062]
2. **源/流程：** CSV/JSON/资产导入为DataTable，注册到资产系统；Data Registry定义sources/rules。
3. **格式：** Unreal资产格式/内存对象，不是跨Rust/C#通用文件。
4. **Schema/type：** USTRUCT行类型；引擎反射和资产系统支持复杂属性。
5. **加载：** Asset Manager/soft reference/async loading；Data Registry支持同步/异步取得和缓存规则。[S062–S064,S114]
6. **压缩/内存：** 由pak/IoStore/资产系统决定；硬引用会拉依赖，soft reference延迟加载。
7. **多语言：** C++/Blueprint核心，与本画像独立Rust/C#不兼容。
8. **热更/版本：** 依赖引擎资产patch/registry source override；并非直接对应Tick Barrier。
9. **工具：** 编辑器、资产依赖、Data Registry调试。
10. **规模/采用：** 商业引擎广泛使用，但具体游戏表规模通常不公开。
11. **最值得抄：** **把数据源、缓存规则、异步获取、soft reference和registry key纳入统一生命周期。** 说明lazy API不能只看文件格式。
12. **最不该抄：** 不要复制UObject/反射资产模型；它会破坏引擎中立、WASM包体和Rust同字节。

## P.7 FlatBuffers

1. **定位：** `Verified`，schema驱动零拷贝格式，本画像最终格式决赛候选。[S019–S023]
2. **源/流程：** `.fbs` schema经`flatc`生成各语言builder/reader；配表编译器填充buffer。
3. **格式：** little-endian、relative offset、table/vector/string。
4. **Schema/type：** 标量、struct/table、vector、union；演进要求保留field IDs/末尾新增等。
5. **加载：** 下载/读取buffer后按offset访问；业务ID索引与chunk必须自行设计。
6. **压缩/内存：** 原始buffer可直接view；压缩后需先解目标chunk，适合per-table/shard frame。
7. **多语言：** 官方Rust/C#生成；C#支持Span/unsafe模式，AOT要实测代码体积。
8. **热更：** buffer immutable，适合root切换；未变chunk共享需外层manifest。
9. **工具：** `flatc`、schema演进文档、JSON互转；游戏ref/semantic diff另建。
10. **规模/采用：** 广泛基础设施使用；公开游戏配表规模不足，Luban支持其输出。[S048]
11. **最值得抄：** **相对offset、按字段view与明确端序。** 双语言读取不必materialize百万对象。
12. **最不该抄：** 不要把“zero-copy”理解为“无索引、无验证、无解压、无字符串成本”；也不要Hash任意builder字节。

## P.8 SQLite + sql.js-httpvfs

1. **定位：** `Verified/Reported`，嵌入式数据库派与浏览器Range先例。[S039–S043]
2. **源/流程：** 编译器建只读SQLite，创建PK/secondary/covering indexes，发布文件或静态Range服务。
3. **格式：** SQLite稳定page/B-tree文件；不自带通用列压缩。
4. **Schema/type：** SQL表/索引/约束；严格类型仍需编译器和schema profile。
5. **加载：** 本地page cache/mmap；浏览器httpvfs按Range取页。点查取决于索引与请求chunk。
6. **压缩/内存：** page cache可控；若外层压缩整个DB会破坏random access。
7. **多语言：** Rust/C#绑定成熟；.NET WASM、IL2CPP native集成和同一VFS需专项spike。
8. **热更：** 发布immutable DB文件+新manifest；不能原地改Active DB。
9. **工具：** SQL查询、EXPLAIN、sqlite CLI、成熟迁移/索引工具。
10. **规模/采用：** httpvfs有大文件演示但项目自述PoC，非游戏生产证据。
11. **最值得抄：** **把点查、范围、covering index、page cache和查询计划交给成熟数据库。** 对不规则查询表非常有吸引力。
12. **最不该抄：** 不要认为“有SQL索引”就自动适合浏览器；一次查询触发多少Range、CDN RTT和reader包体必须测。

## P.9 Arrow / Parquet

1. **定位：** `Verified`，列式内存/磁盘格式派。[S033–S038]
2. **源/流程：** typed arrays/record batches写Arrow IPC或Parquet row groups。
3. **格式：** Arrow连续buffers；Parquet row group→column chunk→page。
4. **Schema/type：** 丰富标量、nested/list/struct/dictionary等。
5. **加载：** 选择列/row group/page；point lookup需外置ID index。
6. **压缩/内存：** dictionary/RLE/delta/page codec强；解码batch与字典有峰值。
7. **多语言：** Rust/.NET官方Arrow；Parquet C#生态多实现，WASM/AOT需测。
8. **热更：** immutable files/row groups；细粒度复用需要外层manifest。
9. **工具：** 数据工程、Python、SQL/分析生态强。
10. **规模/采用：** 大数据行业广泛；作为Gameplay配表主格式没有有力公开案例。
11. **最值得抄：** **列统计、page index、dictionary、RLE/delta和record batch。** 可直接启发sidecar、分析和AI模拟管线。
12. **最不该抄：** 不要为追求压缩把所有Gameplay表列存化；按ID取一整行会跨多buffer/page，访问模式错位。

## P.10 综合抄法

最终方案可以组合：

- Luban式typed IR、多源、多端、验证；
- CastleDB式文本权威/语义diff；
- MasterMemory式typed index API；
- Unreal式usage/async/cache生命周期；
- FlatBuffers式relative offset view；
- SQLite式B-tree/covering index用于特殊表；
- Parquet式page/dictionary/delta用于分析和冷sidecar。

不能组合的是多个权威Hash、多个ID语义和多个热更代际。组合发生在物理后端，不发生在契约语义。

### 来源

[S018–S043, S048–S071, S110–S111, S114, S119, S141, S143–S144]

---
# Q. 批评、失败案例与边界

**结论先行 1：** 配表事故通常不是“序列化库有bug”，而是权威源自动改值、presence/default不清、ID复用、混代、客户端泄露和首次访问隐藏I/O。  
**结论先行 2：** 能核到的一手事故集中在Excel自动转换和引擎导入/本地化；商业团队对热更、作弊和迁移失败的公开披露明显不足。  
**结论先行 3：** 本章把已核事故、机制性风险和未找到证据的领域严格分开。

## Q.1 公认痛点与技术原因

| 痛点 | 技术原因 | 早期征兆 |
|---|---|---|
| 编译慢 | 无依赖图、全量重算公式/全图、生成文件过多 | 改一行仍跑全量；本地不愿验证 |
| 生成物爆炸 | 每表/列复制代码、泛型实例、三端全生成 | PR成千文件；WASM/AOT体积上涨 |
| 错误难懂 | validator只抛异常，无row/column/ref链 | 策划把错误转发给程序，不自修 |
| 策划改不动 | schema与编辑器脱节、需要SDK/命令行 | 私下复制表、程序代改、绕过CI |
| 热更翻车 | 逐表替换、异步旧请求、旧代缓存复用 | 切换后偶现某表旧值 |
| 表越大越崩 | 对象图、字符串、双快照、解压峰值 | 文件不大但GC/heap急升 |
| lazy首次卡顿 | getter隐藏I/O、无usage warm-up | 进入玩法第一次用技能/打开UI掉帧 |
| review失效 | xlsx/binary不可语义diff、默认影响未展开 | 审批只看文件大小/“binary changed” |

## Q.2 已核或可明确定位的事故

### 事故1：Excel把标识符改成日期/数字

`Verified`：2016 Genome Biology论文发现基因名称被Excel自动转成日期或浮点，错误广泛进入公开补充表；2021 PLOS论文继续分析Excel和Google Sheets的自动转换。[S075,S076] 微软文档同时确认前导零、15位精度和自动转换机制。[S072–S074,S112]

**可迁移教训：** 游戏ID、版本号、SKU、资源ID必须以schema目标类型读取并校验原始文本；导入器发现单元格类型与schema冲突时拒绝，不“帮忙转换”。

### 事故2：长数字最后几位被改成0

`Verified`：微软明确说明Excel只保留15位精度，继续输入的位会变0。[S073] 对64位整数ID，这不是显示问题，而是数据已不可逆改变。

**防线：** 字符串ID优先；整数列导入前检查工作簿cell type/原始文本；生成模板预设文本格式；CI用边界fixture验证。

### 事故3：DataTable JSON导入/导出擦除数据

`Verified`：Unreal Engine 5.8 release notes包含修复——JSON DataTable导入/导出会清空instanced struct data的缺陷。[S064]

**教训：** 即使商业引擎内建工具也会在“文本↔typed资产”往返时丢复杂字段。任何源/视图双向转换都必须有round-trip语义Hash测试，而不是只看命令成功。

### 事故4：本地化键不稳定

`Reported`：Unreal社区讨论从CSV导入String Table时生成/变化不稳定key的问题，并提出确定key的workaround。[S065]

**教训：** 本地化引用必须由源中稳定LocKey决定，不能由文件路径、行号、导入顺序或随机GUID临时生成。

### 事故5：公式cached value过期

`Verified`：openpyxl不计算公式，只能读取公式或已有cached value。[S084] CI若不通过同一公式引擎重算，可能发布旧结果。

**教训：** 生产编译器不依赖工作簿公式结果；公式要么转为编译器/模拟器规则，要么在受控重算后冻结并验证。

### 事故6：过早取得“后台加载”结果仍会阻塞

`Verified`：Godot后台加载文档/ResourceLoader说明，在加载完成前调用获取结果会阻塞。[S070,S116]

**教训：** “有异步API”不等于“任何getter都无hitch”。必须在usage barrier显式等待完成，热路径只使用`TryGet`。

## Q.3 机制性高风险，但未找到可公开命名事故

### ID复用导致旧存档错指

没找到可核商业游戏复盘。`Verified`的相邻证据是Protobuf明确禁止复用field number/enum value，因为会造成反序列化歧义和数据损坏。[S016] `Estimated`：配置行ID复用具有相同身份混淆，应使用墓碑。

### 客户端表泄露作弊

没有找到满足证据纪律、可命名产品且来源可靠的完整事故。能确定的是客户端硬编码秘密可被逆向，因而“客户端字节即公开”是安全边界，而不是某个游戏的轶事。[S101]

### 热更导致线上奖励/经济事故

没有找到愿意公开完整配置、时间线、回滚结果的可靠案例。机制上，切回旧配置无法撤销已发奖励，故需要revision事件日志和业务补偿设计；这一结论标`Estimated`。

### 代码生成导致WASM/启动膨胀

公开AOT/裁剪文档证明动态代码与反射限制真实，[S091–S093] 但没有找到“上千配置表生成器造成X MB”的可核游戏数据。必须由O章基准关闭。

### lazy首触被玩家感知

引擎文档能证明过早同步取得异步资源会阻塞，[S116] 但未找到配表专属、带帧时间与设备的公开事故。仍应作为高风险设计点测量。

## Q.4 哪些规模/品类下路线会崩

### Excel单一权威会崩

触发条件：多人同时改同一大工作簿、跨表公式多、ID含长数字/日期样式、频繁批量重构、AI/CI自动写。征兆：锁表排队、冲突重放、隐藏sheet/颜色成为协议、导出只在某台电脑成功。

### 全量JSON对象会崩

触发条件：浏览器首要、百万行、字符串多、Active/Staged并存。征兆：下载尚可但解析/GC慢、heap高水位远高于文件、热更短时OOM。

### 一张大压缩包会崩

触发条件：必须随机按需、网络Range、频繁小更新。征兆：取一行要解整个表、Range返回200全体、改1%重下100%。

### 行级网络lazy会崩

触发条件：高RTT、一次玩法需成百上千行、无批量预取。征兆：请求数远大于chunk数、headers/RTT主导、cache miss后连续掉帧。

### 纯列存Gameplay会崩

触发条件：核心模式是按ID取整行多个字段。征兆：一次查询触及多page/buffer，点查比批扫差，生成row facade复杂。

### 纯rkyv/bincode共享会崩

触发条件：C#必须同字节、长期schema演进。征兆：开始手写C# reader、复制Rust布局、每次Rust升级都改协议。

### 反射通用API会崩

触发条件：Unity IL2CPP/AOT、WASM裁剪、千表热路径。征兆：link.xml不断增长、发布版缺类型、字段字符串拼错只在运行时发现。

## Q.5 已放弃路线的公开复盘

**Known gap：** 没有找到足够一手材料回答“某团队从Excel权威迁到文本/数据库，后来是否后悔”。CastleDB的文本权威、LubanAssistant/Excel2TextDiff和Luban对多人xlsx冲突的公开说明证明痛点与替代设计存在，[S048,S054] 但不能替代长期项目复盘。下一轮建议对公开演讲、GDC/Unite/腾讯/网易/米哈游等技术分享做定向人工检索和作者访谈。

### 来源

[S016, S048, S054, S064–S065, S070, S072–S076, S084, S091–S093, S101, S112, S116, S142, S144–S145]

---

# R. 完整性评估与选型建议

**结论先行 1：** 现在应冻结逻辑IR、ID/presence/Hash/投影/Revision句柄，不应冻结最终codec和chunk大小。  
**结论先行 2：** 推荐“schema-first文本权威 + Excel视图 + 内容寻址分块artifact + 表级/分片级lazy + typed双语言view”。  
**结论先行 3：** 最终物理后端由FlatBuffers、自研typed binary和SQLite实测决赛；JSON/JCS是第一期和永续oracle，不是临时废品。

## R.1 十条核心设计洞察

### 洞察1：把“编辑器”与“权威源”拆开

设计是：Excel/Sheets继续服务策划，但发布权威是独立schema和canonical typed source/IR。它解决自动类型转换、二进制冲突、AI/脚本难以稳定patch的问题；不拆会让同一ID在导出前已被Excel改写，且多人合并只能人工重放。代价是要开发导入/导出插件和向策划解释“保存工作簿不等于发布”。

### 洞察2：Hash逻辑值，不Hash某个序列化器

设计是：对schema ordinal、stable row ID、presence和规范值形成canonical事件流，再构造table/projection/release root。它解决Protobuf非canonical、FlatBuffers布局、SQLite page与Parquet codec变化造成的同值异bytes；不这么做，换builder/压缩参数都会像内容变更，跨Rust/C#也可能对不上。代价是要维护一份独立canonical规范和双语言golden corpus。

### 洞察3：快照是不可变命名空间，不是全量对象全集

设计是：Active/Staged各持有完整不可变manifest根，chunk可按需驻留，cache不参与语义。它解决lazy与immutable表面冲突；不这么做，只能全量下载，或让“未加载表”在切换时不知属于哪代。代价是所有future、cache key和handle都要带Revision。

### 洞察4：默认表级lazy，超大表再分片

表级兼顾调用简单、索引小和跨列访问；大表按稳定业务域/range分片，长字段做sidecar。它解决网络行级请求放大和整表大解压；不分层会在小表上过度复杂、在巨表上首次卡顿。代价是编译器要生成usage set、shard函数与依赖closure。

### 洞察5：内容寻址共享是双快照内存可行性的关键

未变chunk以payload hash+layout profile复用，Active/Staged只各持root和变化内容。它解决“大表双份”峰值；不做时每次热更即使只改一行也可能复制整表。代价是chunk粒度、retention、refcount/epoch和缓存GC复杂。

### 洞察6：运行时API必须把I/O赶出getter

业务使用同步、typed、revision-bound `TryGet`；I/O只在`PrepareAsync`和usage barrier。它解决首次访问hitch、异步结果混代和Replay时序依赖；不这么做，某个新技能第一次使用可能触发网络/解压并改变Tick行为。代价是玩法/场景要声明配置依赖，开发者不能随处直接全局取表。

### 洞察7：字符串与对象图优先于codec优化

UTF-8共享池、结构体/column buffers和按需解码通常比把JSON换成另一种二进制更影响常驻内存。它解决百万行对象头、装箱、UTF-16和GC roots；不做时文件压到10MB也能展开成数百MB。代价是API从“随手拿string/class”变为view/ID，并需要生命周期纪律。

### 洞察8：三端对账要比较投影，不比较不存在的全量

一次IR编译出Server/Client/Voxel roots，并由签名Release Manifest绑定；两端共同比较Shared root。它解决客户端不持有私有表却被要求计算全量Hash的逻辑矛盾；不这么做，要么泄露数据，要么握手验证是假的。代价是compiler和manifest必须理解visibility与cross-projection ref。

### 洞察9：AI的生产基础是validator与审计

AI只提交stable-ID typed patch，经结构、引用、投影、模拟和人审闭环。它解决“模型能写单元格但不知道经济/玩法后果”；不这么做，AI会产生schema合法却系统错误的数据，且无法追责。代价是错误格式、权限、provenance和simulation需要先投资。

### 洞察10：JSON第一期必须成为长期oracle

第一期JSON实现完整schema、canonical Hash、typed API、Revision、投影和测试向量；后期二进制只是reader/provider替换。它解决第一期“先随便写、以后重构”的迁移陷阱；不这么做，第二期会同时改ID、默认值、Hash和调用API，无法双跑验证。代价是第一期不会是最快的最小脚本，但会形成可持续基线。

## R.2 完整性缺口清单

| 缺什么 | 谁有/怎么做 | 不补会在哪炸 | 分级 |
|---|---|---|---|
| 容器/嵌套类型 | Luban、Protobuf/Avro/FlatBuffers有list/map/nested [S017,S021,S032,S048] | 掉落列表、条件数组被塞分隔字符串，无法类型校验/引用检查 | **必须现在补表达策略**：先正规化子表；公共类型扩展可推迟 |
| null/missing/empty/default四态 | Protobuf presence、Avro union [S015,S032] | 空单元格在不同导出器变0/空串/default，Hash与逻辑悄变 | **必须现在补** |
| stable ID与墓碑 | Protobuf reserve、Godot UID [S016,S071] | 旧存档/Replay引用被新行接管 | **必须现在补** |
| source ID→revision ordinal双层 | 只读DB/生成工具常做索引；本报告推演 | 把稠密ordinal持久化后跨版本错指 | **必须现在补** |
| schema字段ordinal与兼容矩阵 | Protobuf/FlatBuffers字段ID [S016,S021] | 改名/重排导致二进制和Hash不兼容 | **必须现在补** |
| 引用图与投影验证 | Luban/xresloader ref校验 [S048,S050] | 客户端ref到Server私有行，首次访问才失败 | **必须现在补** |
| canonical逻辑Hash profile | JCS/CBOR给编码基础，Protobuf警告非canonical [S002,S011,S018] | Rust/C#、JSON/二进制、压缩升级对不上 | **必须现在补** |
| Unicode归一化政策 | UAX #15 [S003] | 视觉相同ID产生两行/不同Hash | **必须现在补** |
| f32/f64导入与fixed policy | JCS/IEEE/.NET格式资料 [S002,S122,S123] | 跨端位值/分支不一致，经济累计漂移 | **必须现在补分类与规则** |
| UsageSet/prepare barrier | Unreal/Unity/Godot异步资产 [S062,S067,S070] | 第一次访问在Tick内触发I/O/hitch | **必须现在补API语义** |
| Revision-bound handle/future | 引擎缓存/内容版本相邻机制，本报告推演 | 热更混代，旧请求污染新缓存 | **必须现在补** |
| 内容寻址chunk共享 | Unity update build复用不变bundle [S068] | Staged/Active大表翻倍，热更峰值OOM | **可以推迟实现，但现在预留hash/chunk identity** |
| 分层覆盖provenance调试 | 成熟远端配置/资产系统常有source rules；本报告推演 | 线上看到最终值却找不到哪层覆盖 | **必须现在定义元数据；UI可推迟** |
| 增量编译依赖图 | Luban云生成/工具生态 [S141] | 改default/schema要么漏重编要么永远全量 | **可以推迟优化，但现在定义cache key/依赖** |
| 语义diff | CastleDB文本、Excel2TextDiff [S048,S054] | 二进制/大表改动无法review | **必须现在有最小版** |
| 结构化错误 | SARIF [S105] | 策划/AI无法自修，错误只流向程序 | **必须现在补** |
| 本地化独立chunk/LocKey | Luban/引擎本地化经验 [S048,S065] | 多语言内存倍增、key漂移 | **可以推迟正文加载，但LocKey稳定性现在补** |
| client disclosure report | 安全边界 [S101] | 新列误发客户端、作弊信息永久公开 | **必须现在补门禁** |
| 签名的rollback/expiry/mix防护 | TUF [S098] | 合法签名旧包/混包仍可被接受 | **必须现在补信任链语义** |
| cross-reader golden/fuzz corpus | 格式安全通用要求 | Rust接受、C#拒绝或反之；坏offset触发内存风险 | **必须现在补** |
| 预取/驱逐策略 | Addressables/Data Registry [S062,S067,S069] | lazy命中率低、重复下载和hitch | **可以推迟调优，但API与遥测现在预留** |
| AI patch/audit接口 | Structured Outputs/MCP/SARIF [S103–S105] | AI直接改单元格不可审计 | **基础设施先补；模型集成可以推迟** |
| 任意SQL/条件查询 | SQLite [S110] | 若真实需求多，手写二级索引爆炸 | **明确可以不做，直到trace证明需要** |
| 全局列存运行时 | Parquet [S037] | Gameplay点查模式错位 | **明确可以不做；保留分析导出** |

## R.3 格式选型建议

### 第一阶段：JSON基线

- canonical typed IR；
- 每表/分片JSON chunk，不是一个全包巨大JSON；
- manifest/index独立；
- System.Text.Json source-generated C# reader；Rust serde_json或专用parser；
- 编译后所有数字已按目标类型验证；
- 业务只依赖`IConfigSnapshot/TableView`；
- JCS/自定义canonical事件用于semantic hash，不要求运行时pretty JSON本身就是Hash字节。

### 第二阶段决赛

#### 候选A：FlatBuffers payload

**优势：** 官方Rust/C#、relative offset、buffer view、AOT友好。  
**需补：** 主键索引、chunk/manifest、canonical逻辑Hash、压缩、visibility/ref业务校验。

#### 候选B：自研typed binary payload

**优势：** 可为混合行/列、string pool、位图、分片和双语言生成精准优化。  
**风险：** 最大自建维护/安全成本。只有FlatBuffers实测不能达到内存/包体/访问目标时选择。

#### 候选C：SQLite后端

**优势：** 不规则查询、B-tree、covering index、工具与浏览器Range先例。  
**风险：** .NET WASM/IL2CPP集成、请求数、native包体、逻辑Hash和Rust/C#统一VFS。适合特定表族，不一定做全局后端。

### 不建议作为默认最终格式

- Protobuf：对象materialization、点查索引、非canonical；保留schema/交换。
- rkyv：C#弱；只考虑Rust私有Voxel spike。
- Parquet：分析/列扫强，Gameplay点查错位。
- MessagePack：可作过渡，但最终仍要自建索引/chunk/canonical profile。

### 定案前必须完成的实测

1. 100万行20列、真实字符串分布的steady/activation peak；
2. 浏览器真实CDN冷/暖usage set请求数与P99；
3. Rust/C#同字节10万随机numeric/string点查；
4. reader+generated code的WASM/IL2CPP包体；
5. 1%行变化时下载/重用/峰值；
6. bad corpus、压缩炸弹、Range返回200/ETag变化；
7. SQLite covering/non-covering query请求放大；
8. FlatBuffers vs custom的string decode与index成本。

## R.4 **“JSON起步 → 二进制升级”必须第一天定死的不变量清单**

| 不变量 | 第一天做错的第二期代价 |
|---|---|
| 独立schema权威与字段ordinal | 二进制字段无法稳定映射；所有生成API和数据迁移 |
| stable row ID、命名空间、永不复用 | 旧存档/Replay不可安全迁移，只能做人工映射 |
| source ID / stable numeric ID / revision ordinal区分 | 已持久化ordinal全面失效 |
| missing/empty/null/default语义 | JSON旧数据无法判断作者意图；Hash和业务值批量变化 |
| 默认值是在编译期还是运行时应用 | 双后端对同一缺失单元格给不同值 |
| 行/列canonical顺序 | Hash重算、diff噪声、跨语言不一致 |
| UTF-8与Unicode normalization | 既有ID/字符串Hash变化，引用断裂 |
| 整数精确解析与范围拒绝 | 已被double/Excel截断的数据无法恢复 |
| f32/f64先舍入、特殊值、fixed分类 | 二进制bits与JSON解析不同，Replay失配 |
| ref在编译期解析与投影规则 | 二进制期才发现跨端ref，需拆表/改API |
| SourceHash / SemanticHash / ArtifactHash三分 | 压缩/布局升级被误判内容变更，或源审计丢失 |
| projection root与shared root | 客户端无法证明版本，只能泄露全量或重做握手 |
| Revision-bound snapshot/table/row handle | 后端切换后仍有global current混代，调用点全面重写 |
| `TryGet`不I/O、`PrepareAsync`显式 | JSON期业务已依赖隐式同步加载，二进制lazy无法落地 |
| 机器可读错误格式 | 第二期AI/编辑器/CI全部要重新解析散乱日志 |
| generated API与backend隔离 | 业务直接依赖`JsonElement`，换格式等于全项目重构 |
| golden corpus与逻辑query oracle | 无法证明新旧后端语义等价，只能“大爆炸切换” |
| visibility元数据与client disclosure门禁 | 既有源无法可靠裁剪，客户端历史泄露无法收回 |

**可以推迟：** 最终codec、压缩级别、chunk目标大小、二分/哈希/MPHF具体实现、Cache API vs IndexedDB策略、是否在某表用SQLite、调试UI皮肤。这些必须被profile/manifest抽象覆盖，但不必第一天锁值。

## R.5 懒加载 × 不可变快照 × 热更的兼容方案

### 方案A：完整字节双快照 + 延迟materialize

**语义：** Stage时完整下载/验签新包；Active/Staged都持完整bytes，表对象按需建。  
**失败模式：** WASM下载和raw bytes仍双份；大表对象构建hitch；不满足真正网络lazy。  
**复杂度：** 低。  
**推荐场景：** Foundation/服务器早期，作为语义正确基线。

### 方案B：完整manifest双快照 + 内容寻址chunk（推荐目标）

**语义：** 两代manifest完整；数据按需，future与cache按root/hash隔离；Barrier切root，旧代epoch退休。  
**失败模式：** 旧artifact保留不足、cache key漏revision、Range/CDN失配、预取声明漏表。  
**复杂度：** 中高。  
**优势：** 真lazy、未变共享、WASM/CDN友好、回滚清楚。

### 方案C：SQLite immutable DB per Revision + page cache

**语义：** 每Revision一个只读DB URL/file；连接/reader绑定revision，Barrier切连接root。  
**失败模式：** 两DB page cache重叠、HTTP Range请求放大、.NET WASM/IL2CPP集成、旧查询lease。  
**复杂度：** 中高但索引/查询少自建。  
**适用：** 查询不规则的大表族或POC，不作为无实测默认。

### 倾向推荐

Phase 1先实现A的“完整语义”，但artifact已经按manifest/table chunk组织；Phase 2切到B，无需改业务API。不要实现形态C式“tableId miss时查global current”。

## R.6 Server / Client / Voxel落地建议

### 切分粒度

- 默认整表；
- 共享实体采用`PublicBase + ServerRules + ClientPresentation`分表；
- 只有重复巨大且schema稳定时列级projection；
- Voxel表单独projection和profile，允许更紧凑热布局。

### 打包

```text
release.manifest
server.manifest + server packs/chunks
client.manifest + bootstrap/usage/cold packs
voxel.manifest + preloaded hot pack
shared public subtree hashes
```

### 跨类引用

- `ref`必须目标在同projection；
- 跨projection只允许opaque stable ID，API不能dereference；
- 编译器给完整引用路径错误；
- Voxel热表引用Gameplay复杂对象时，改为小整数rule ID/dispatch ID，不跨FFI追对象。

### 一致性

- 签名Release Manifest绑定三个root；
- 客户端验证Client+Shared；服务器验证Server+Shared；Rust返回Voxel root receipt；
- 握手比较该端应持有root；
- Replay记录Server/Shared/Voxel roots；
- 不要求客户端重算Server私有root。

## R.7 AI配表路线图

### 现在就能做

- canonical text/semantic diff；
- stable-ID patch格式；
- `get_schema/search/get_rows/propose_patch/validate`只读+提案工具；
- 结构化错误；
- Git/审计记录actor、base/candidate hash；
- AI无生产激活权限。

### 需要先建基础设施

- cross-table/business validators；
- impact analysis；
- deterministic simulation harness；
- table/column RBAC与敏感分类；
- compile preview和projection disclosure report；
- review UI、并发baseHash检测、provenance存储。

### 目前不成熟，建议观望

- AI自主大范围经济平衡并直接发布；
- AI自由操作复杂公式/合并单元格工作簿；
- 让模型决定签名/回滚/安全阈值；
- 把上千表全schema一次塞入上下文；
- 以自然语言聊天记录替代typed patch和审计。

## R.8 冻结项风险提示

> 以下不是要求推翻冻结项，而是让决策者知晓兼容成本。

### 冻结项：仅标量 + enum + ref

**冲突：** 同类工具普遍有list/map/nested/polymorphism/nullable；掉落列表、条件数组、本地化天然需要复合结构。[S017,S021,S032,S048]  
**代价：** 若用分隔字符串，会失去类型/引用/语义diff。  
**兼容改法：** 正规化子表+ordinal+1:N校验；binary阶段可聚合vector，不改公共列类型。

### 冻结项：Staged/Active两份不可变快照

**冲突：** 与大表内存不是语义冲突，而是物理实现风险。  
**代价：** 若快照被实现成对象全集，热更近似翻倍/三倍峰值。  
**兼容改法：** 快照=manifest root；内容Hash共享chunk；old generation延迟释放。

### 冻结项：Tick Barrier原子激活

**冲突：** async lazy若未完成，Barrier不能凭空使数据可用。  
**代价：** 进入新Revision后首Tick miss/fail-stop。  
**兼容改法：** `RequiredUsageSet`必须在Barrier前Prepare并给receipt；非关键冷表可之后加载，但其访问不参与权威Tick或必须先显式准备。

### 冻结项：f32/f64存在

**冲突：** 配置bits可一致，跨平台运算不一定完全确定。  
**代价：** 参与分支/累积时Replay漂移。  
**兼容改法：** 编译期固定位模式、拒绝特殊值；权威经济/战斗列使用scaled integer，f32/f64限视觉或经专项确定性验证。

### 冻结项：Canonical序列化

**风险：** 若定义为“当前JSON/Protobuf字节”，与官方格式事实冲突。[S018]  
**兼容改法：** 明确为格式独立canonical逻辑事件；物理artifact另算Hash。

### 冻结项：生产未签名拒绝

**缺口：** 仅验签不防合法旧包、过期、mix-and-match。  
**兼容改法：** 加Release版本、expiry、rollback counter、projection root绑定和可信key rotation，借鉴TUF。[S098]

### 冻结项：层级覆盖

**风险：** 若运行时逐层临时查询，Hash、调试和原子性复杂。  
**兼容改法：** 编译/Stage时合并为不可变effective snapshot，同时保留provenance；User/Session动态层若会变化，明确是否独立于发布Revision并进入Replay。

### 冻结项：默认只能补可选

**评价：** 与强schema实践一致；风险在“空单元格是否算missing”。  
**兼容改法：** 源适配器统一四态，不能由Excel库自行判断。

## R.9 风险清单

### 一定会踩（若不设防）

| 风险 | 触发条件 | 早期征兆 |
|---|---|---|
| Excel自动改ID/数字/日期 | 直接读用户输入、不按schema校验 | 前导零消失、ID末位0、日期样式cell |
| object graph内存膨胀 | JSON→每行对象/装箱/string | 文件小但heap/GC大 |
| 客户端数据公开 | Server私有列进入Client projection | dump能看到掉率/阈值 |
| 混Revision | cache/future/handle只按tableId | 偶现旧表+新表组合 |
| default/empty歧义 | 源adapter未定义四态 | “清空”操作变回default或0 |

### 大概率会踩

| 风险 | 触发条件 | 早期征兆 |
|---|---|---|
| 小chunk请求爆炸 | 每表/行独立URL无usage pack | warm-up数百请求 |
| 字符串成为第一内存大类 | 多语言/路径全解码UTF-16 | string/char[]占比上升 |
| 生成代码/AOT膨胀 | 每表复制泛型/registry | build时间、WASM包体持续涨 |
| 旧artifact保留不足 | lazy/replay依赖旧revision | 回滚或历史replay下载404 |
| 校验错误不可行动 | 只给stack trace | 策划依赖程序定位 |
| visibility漏标 | 默认全端、无disclosure diff | 客户端新增敏感列无人注意 |
| 公式缓存不一致 | CI不重算/库只读cached | 本地与CI值不同 |

### 视规模而定

| 风险 | 触发条件 | 早期征兆 |
|---|---|---|
| 全局索引反客为主 | 百万行+多二级索引 | 10%数据驻留但索引占大头 |
| FlatBuffers代码/访问不达标 | 千表、字符串多、索引外置 | 生成体积或string decode主导 |
| SQLite Range延迟高 | 非covering query、高RTT | 每次点查多个Range |
| Zstd解压峰值 | 大chunk并行、双快照 | activation high-water突然上升 |
| MPHF维护不值 | 键集频繁变、范围查询 | 构建复杂、收益小于二分 |
| 列存点查退化 | 按行访问多列 | page/buffer触达数过多 |

## R.10 如果从零重做

### 1. 权威源

```text
/schema/*.cfgschema.json          # 独立、版本化、字段ordinal/visibility/constraint
/data/<domain>/<table>/*.jsonl    # 每行一个stable-ID typed source record
/data/_tombstones/*.jsonl
/tools/spreadsheet-views/*.xlsx   # 生成/导入视图，不是发布权威
```

- 策划可在Excel/Sheets编辑；插件把选择范围转成stable-ID patch。
- patch应用到文本权威，CI再生成Excel预览；不做自由双向同步。
- 复杂列表用子表正规化，直到公共schema加入容器。

### 2. 编译器

一个可复现CLI/服务，核心步骤：import→normalize→typed IR→layer merge+provenance→全图validate→三projection→logical Merkle roots→artifact emit→dual-reader verify→report→sign request。

compiler profile固定：schema DSL版本、Unicode policy、decimal parser、hash算法、projection规则、payload backend、compression profile。输入/输出全部内容寻址。

### 3. 第一阶段artifact

```text
manifest.cjson / manifest.bin
index/<table>.idx
chunks/<hash>.json.zst-or-none
```

小表独立chunk；大表按stable range/业务域分片；本地化/长文本sidecar。JSON仅是payload，外层manifest、Revision、index、projection和安全头从第一天存在。

### 4. 第二阶段artifact

把热点chunk切为：

- fixed-width/mixed row area；
- UTF-8 string dictionary/pool；
- presence bitmaps；
- sorted key index；
- relative offsets、little-endian；
- independent Zstd/LZ4 frames。

优先试FlatBuffers payload；若实测显示其index/layout/代码体积不能达标，再用同schema生成自研view。SQLite用于确有多条件/范围查询的特殊表，Arrow/Parquet作为分析导出。

### 5. 加载

- 浏览器：manifest+bootstrap pack；Service Worker/Cache/IndexedDB；usage packs；冷chunk；Range pack可选。
- 服务器：本地content store+mmap/read；启动预热；多进程共享file-backed pages。
- Voxel：进入世界前全量准备热pack，Rust二次安全验证，连续数组/SoA常驻。

### 6. 生命周期

`RevisionRoot`不可变；cache key含projection/root/chunkHash；future捕获root；Tick Barrier原子切Active；旧代epoch退休；相同chunkHash共享。Replay钉root，旧artifact按政策保留。

### 7. API

- `PrepareAsync(revision, usageSet)`；
- `OpenSnapshot()`；
- generated `TableView<T>.TryGet`；
- generic debug/AI schema view；
- no I/O in getters；
- row/view lease钉住chunk。

### 8. 工具面

- Excel/Sheets插件：schema控件、ref选择、错误定位、patch；
- CLI/CI：changed validate、full build、repro check；
- Inspector：Active/Staged、resident、layer provenance、semantic diff；
- AI tools：query/propose/validate/simulate/submit，不激活；
- disclosure report、size report、benchmark trend、artifact dump。

### 9. 第一阶段里程碑

1. schema/IR/Hash/golden corpus；
2. Excel→typed text单向发布；
3. JSON per-table chunks + typed API；
4. three projections + signed manifest；
5. Revision-bound Active/Staged + prepare barrier；
6. browser bootstrap/IDB cache；
7. 100万行基准；
8. FlatBuffers/SQLite/custom spikes；
9. 选定binary profile；
10. AI patch/validator接口。

### 最终倾向性答案

在“Rust内核+C# Gameplay、浏览器WASM首要、确定性与状态Hash、大表和懒加载、Staged/Active原子切换”的组合下，配表管线应长成：

> **Schema-first、文本可审计权威、Excel/Sheets受控视图；统一typed IR编译三端投影；格式独立canonical语义Hash；小manifest+内容寻址不可变chunk；默认表级、超大表分片级lazy；热数值行+UTF-8池+sidecar混合布局；Rust/C#生成bounds-checked typed view；显式usage prepare；Revision根原子切换与旧代延迟释放；AI只经typed patch/validator/simulation提案。**

现在必须决定语义与生命周期；codec、chunk大小、缓存容量和某些表是否用SQLite可以推迟到实测后。

### 来源

[S001–S146；核心结论重点依赖 S002–S003, S015–S023, S033–S046, S048–S055, S072–S084, S085–S105, S108–S111, S120–S140]

---

# 附：验收自检

- [x] A–R十八章均有实质内容；无空章。
- [x] C/D/E为全文最厚的核心章节之一，含完整格式矩阵、实现级lazy设计、压缩/内存模型。
- [x] C章矩阵每格有内容及置信度；CSV另附。
- [x] E章内存表基于公式与假设；CSV和说明另附。
- [x] M章明确说明证据薄，工业方案标Estimated。
- [x] B/D/M关键判断题均给出明确结论。
- [x] R章含十条洞察、缺口、格式建议、不变量、兼容方案、三端、AI路线、冻结风险、风险表、从零方案。
- [x] 数字均带来源条件或估算模型；未用无条件benchmark做决策。
- [x] 中英文来源均覆盖；每章末尾有来源回指。
- [x] 外部仓库未clone、未打包源码；GitHub永久行号缺口已降级并声明。
