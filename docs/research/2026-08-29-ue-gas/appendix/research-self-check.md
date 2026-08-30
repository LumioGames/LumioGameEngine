# 交付前自检

自检时间：2026-08-29

## 机械检查结果

- [x] 根目录包含 `README.md`、`report/`、`sources.md`、`appendix/`。
- [x] 主报告包含 A–R **18 个字母章节**，顺序为 `ABCDEFGHIJKLMNOPQR`。
- [x] 每章均有三行“结论先行”与章末“来源”小节。
- [x] H、I 为按字符计数最厚的两章：H 约 34k 字符，I 约 29k 字符。
- [x] 主报告中引用的 `[Sxxx]` 均可在 `sources.md` 找到；范围引用如 `[S004]–[S010]` 表示连续条目。
- [x] `sources.md` 共 72 条，包含类型、标题、定位、访问状态与支撑章节。
- [x] 没有引用 `EpicGames/UnrealEngine` 作为源码；没有 clone 或下载源码。
- [x] 指定 `Go1c/UnrealEngine` 不可达现象在报告开头、`sources.md` 与 access log 三处声明。
- [x] 没有伪造 `Go1c/UnrealEngine@commit:path#Lx-Ly`；所有需函数体的结论均降为 `Reported`/`Estimated`。
- [x] 包内没有源码、仓库副本、二进制、截图、临时文件。
- [x] 附录 CSV 为 UTF-8 with BOM，便于常见表格工具打开；它们只包含本报告的对照数据。

## 验收清单映射

- [x] A–R 全部有实质内容；查不到的项目在正文明确写出原因。
- [x] H 覆盖复制全景、三种模式、FastArray、属性/Ability上行下行、batching、优化、Iris、时间、失败、late join与重实现协议。
- [x] I 明确回答“GAS 是乐观收敛，不是回滚重演”，并与 CharacterMovement、Network Prediction、Chaos 对比。
- [x] 关键断言标 `Verified` / `Reported` / `Estimated`；官方与社区来源分离。
- [x] R 包含十条洞察、同步精髓、回滚精髓、状态机对照、不可照搬清单、推迟能力风险与从零重写。
- [x] E 明确回答 Modifier 顺序、同组Effect换序是否等价、Stack/Refresh/Cancel时序证据缺口。
- [x] Effect inhibited 状态映射为 Active 内正交字段，而非擅自增加目标顶层状态。
- [x] C 区分 Definition/Instance/Handle/PredictionId并讨论ABA。
- [x] L 明确说明裸 GAS 不提供目标要求的稳定全域状态哈希。
- [x] N 未编造每 ASC/Effect 字节数或 N/M 容量数字。
- [x] M 对无法核准的 cvar/命令明确标“名称待核/Reported”，未凑清单。
- [x] README 可独立阅读，包含完整执行摘要、章节索引、阅读顺序和 Known gaps。

## 人工复核的限制

“每条关键论断”是语义标准，无法靠正则完全证明；本次人工处理原则是：章节结论、机制判断、时序/网络/迁移结论均带置信度，表格中的派生项沿用其上方标注。若取得指定 5.6 源码镜像，下一轮最重要的工作不是扩写篇幅，而是逐项把 `Reported` 的控制流换成 commit permalink 与行号。
