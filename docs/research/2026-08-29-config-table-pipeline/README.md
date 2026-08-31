# 配置表管线外部技术调研包

**交付日期：** 2026-08-29  
**主题：** 策划配表的权威源 → 编译产物 → 运行时读取  
**目标画像：** Rust原生内核、C# Gameplay、浏览器.NET WASM、Dedicated Server、确定性/Replay、Staged/Active不可变快照与Tick Barrier原子激活  
**主报告规模：** A–R共18章，约101,611个非空白字符（Markdown文件111,779字符）  
**来源总表：** 146条公开来源记录

## 本包是什么

这是一次纯外部、证据驱动的配表管线调研。报告不访问委托方代码库，只按任务书给出的冻结契约和目标运行时画像，比较权威源、schema、运行时格式、索引、懒加载、压缩、内存、三端投影、确定性、热更、访问API、AI接口和工程化。

所有关键论断使用：

- `Verified`：实际读到规范、官方文档、论文或可核官方仓库说明；
- `Reported`：可信项目/维护者/社区记述，但未核到稳定源码坐标或完整条件；
- `Estimated`：工程推演或估算模型，并说明依据。

## 建议阅读顺序

1. **决策者：** 先读本README执行摘要，再读主报告R章；随后看C章矩阵、D章懒加载、E章内存。
2. **框架实现者：** C → D → E → F → G → J → K → L → N。
3. **内容工具/策划负责人：** B → H → I → M → N → Q。
4. **证据复核：** `sources.md`，再按章节末尾的`[Sxxx]`回指。

## 包内文件

| 文件 | 用途 |
|---|---|
| `README.md` | 导读、章节索引、执行摘要全文、Known gaps |
| `report/config-table-pipeline-research-2026-08-29.md` | A–R主报告全文 |
| `sources.md` | 146条来源总表：类型、标题、URL/定位、访问状态、支撑章节 |
| `appendix/format-selection-matrix.csv` | 19种格式 × 14个决策维度完整矩阵 |
| `appendix/memory-estimates.csv` | 两种规模 × 四种持有形态内存估算 |
| `appendix/memory-model.md` | 内存公式与假设 |
| `appendix/benchmark-plan.csv` | 16组可执行选型基准 |
| `appendix/config-artifact-container-sketch.md` | 非规范性容器/Revision草图 |
| `appendix/validation-error-example.json` | 面向CI/编辑器/AI的结构化错误示例 |

## 章节索引

- [A. 谱系与全景：配表管线的工业界地图](report/config-table-pipeline-research-2026-08-29.md#a-谱系与全景：配表管线的工业界地图)
- [B. 权威源形态：Excel 到底该不该当权威源](report/config-table-pipeline-research-2026-08-29.md#b-权威源形态：excel-到底该不该当权威源)
- [C. 中间格式与运行时格式选型](report/config-table-pipeline-research-2026-08-29.md#c-中间格式与运行时格式选型)
- [D. 运行时加载与懒加载](report/config-table-pipeline-research-2026-08-29.md#d-运行时加载与懒加载)
- [E. 压缩与内存占用](report/config-table-pipeline-research-2026-08-29.md#e-压缩与内存占用)
- [F. 表分类与可见性切分：Server / Client / Voxel](report/config-table-pipeline-research-2026-08-29.md#f-表分类与可见性切分：server-/-client-/-voxel)
- [G. 跨语言与跨运行时：Rust × C# × WASM × AOT](report/config-table-pipeline-research-2026-08-29.md#g-跨语言与跨运行时：rust-×-c#-×-wasm-×-aot)
- [H. Schema 定义、类型系统与演进](report/config-table-pipeline-research-2026-08-29.md#h-schema-定义、类型系统与演进)
- [I. 引用完整性、ID 与索引](report/config-table-pipeline-research-2026-08-29.md#i-引用完整性、id-与索引)
- [J. 确定性、Hash 与两端一致性](report/config-table-pipeline-research-2026-08-29.md#j-确定性、hash-与两端一致性)
- [K. 热更、Revision 与激活语义](report/config-table-pipeline-research-2026-08-29.md#k-热更、revision-与激活语义)
- [L. 访问 API 形态：代码生成 vs 反射 vs 零拷贝 View](report/config-table-pipeline-research-2026-08-29.md#l-访问-api-形态：代码生成-vs-反射-vs-零拷贝-view)
- [M. AI 友好的配表](report/config-table-pipeline-research-2026-08-29.md#m-ai-友好的配表)
- [N. 工具链与工程化](report/config-table-pipeline-research-2026-08-29.md#n-工具链与工程化)
- [O. 规模与实测数据](report/config-table-pipeline-research-2026-08-29.md#o-规模与实测数据)
- [P. 具体方案深挖](report/config-table-pipeline-research-2026-08-29.md#p-具体方案深挖)
- [Q. 批评、失败案例与边界](report/config-table-pipeline-research-2026-08-29.md#q-批评、失败案例与边界)
- [R. 完整性评估与选型建议](report/config-table-pipeline-research-2026-08-29.md#r-完整性评估与选型建议)

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

## Known gaps

- **公开商业项目规模数据非常少。** 没有找到可核、同时给“表数/行数/文件体积/冷启动/峰值内存/硬件与运行时版本”的完整游戏案例；O 章因此以可执行基准矩阵代替拍脑袋阈值。
- **没有找到成熟的“AI 直接配生产游戏配置”公开案例。** 仅找到零星实验项目与相邻领域的结构化输出、工具调用、静态分析结果规范；M 章把方案明确标为 `Estimated`。
- **没有核到大量团队从 Excel 权威源迁走后的完整复盘。** CastleDB/Luban 的文本化与双向视图能力证明需求真实，但“迁移后是否后悔”的公开、可核长期数据不足。
- **GitHub commit permalink 不完整。** 匿名页面可读 README/文档，却未稳定得到所有关键源码文件的 commit hash 与行号；因此源码内部实现细节整体降级，未按要求假装 `Verified`。
- **.NET WASM × SQLite/FlatBuffers/Parquet 的真实包体、AOT、冷启动与内存数据必须本地实测。** 文档能证明“机制可用”，不能证明“在委托方目标预算内可用”。
- **浮点跨端一致性的业务容忍度未知。** 报告能给编码和规避方案，但无法替委托方决定哪些列必须定点化；需要对技能公式、掉落概率、物理/光照参数做领域分级。

## 一句话选型结论

> **Schema-first、文本可审计权威、Excel/Sheets受控视图；统一typed IR编译Server/Client/Voxel三投影；格式独立canonical语义Hash；小manifest+内容寻址不可变chunk；默认表级、超大表分片级lazy；热数值行+UTF-8池+sidecar混合布局；Rust/C#生成bounds-checked typed view；显式usage prepare；Revision根原子切换与旧代延迟释放；AI只经typed patch/validator/simulation提案。**

最终payload先以JSON/JCS做正确性基线，再让`FlatBuffers + 外置索引/分块`、`自研typed binary`、`SQLite readonly/Range`在同一数据集和目标平台上实测决赛。
