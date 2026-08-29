---
name: lessons
description: 经验教训——reviewer 反复退回的同类问题与 Agent 常犯坑;开工前与复盘沉淀时查
metadata:
  type: doc
  status: 已交付
---

# 经验教训（Lessons Learned）

复发问题的暂存区：记录 reviewer 反复退回的同类问题与 Agent 常犯的坑，让同一个坑不踩第三次。本文档是规范的**候选池**——条目在这里验证价值，稳定后升格，不在这里长住。

## 收录准入

- **同类问题第二次出现才收录**——单次偶发不收，防噪音。
- 来源：reviewer 退回报告、交回物的 known gaps、用户纠偏。
- 不收待办（走任务卡）；不收项目常识（进 `standards/` 或 feature 文档）。

## 条目格式

一条 lesson 一个小节，新条目加在「条目」节最上方（倒序）：

    ### <一句话规避规则>
    - 日期：YYYY-MM-DD
    - 现象：踩了什么坑、复发几次
    - 根因：为什么会发生
    - 规避：怎么做能不再犯（可验证的行为，不是口号）
    - 来源：reviewer 报告 / known gaps / 用户纠偏（附提交或任务标识）

## 升级路径

某条 lesson 被稳定复用（约第三次引用起）→ 升格为 `knowledge/standards/` 规则或 `rules/` 红线，原条目标注「已升格 → <落点>」，保留不删。

## 条目

### 「有一份看起来在守护的东西」必须用对照组探针证明它真的会响
- 日期：2026-08-29
- 现象：一天之内四例守护形同虚设,分布四仓、四种机制:①架构仓 `sha256.rs` 的 K[28] 轮常量写错(`0xc6eabbdc` vs FIPS 180-4 的 `0xc6e00bf3`),对任意输入都算错摘要,且当时无 CI 跑 `cargo test`;②GameRuntime `eng/generate-contracts.sh` 的重生成判据**哈希的是经 git attribute 转换后物化的字节,而不是 git 对象本身**——架构源 `.gitattributes` 首行 `* text=auto`,`git archive` 按**调用方**的 `core.autocrlf` 做行尾转换,于是判据是 `(commit, 机器 attribute 栈)` 的函数而非 commit 的函数;③LumioClient 与 ④LumioServer **各自独立**踩中 `Microsoft.CodeAnalysis.BannedApiAnalyzers` 只识别 `BannedSymbols.txt` / `BannedSymbols.*.txt` 的 AdditionalFile,文件名叫 `banned-public-api.txt` 时整份禁令**静默忽略、不报错不警告**——两仓的禁令清单从未生效。四例全部通过过此前的人工审查。
- 根因：守护机制的「存在」与「生效」是两件事,而阅读代码/配置只能确认前者。命名约定型工具(分析器、lint 插件)静默降级尤其危险;记录型守护(manifest/Golden/审计哈希)则会在锚点漂移后腐烂成谎言,同样不报错。
- 规避：①**任何声称有守护的验收项,证据必须是对照组探针**——故意制造一处违规,证明守护会红,再移除证明会绿;只写「build 通过 / 测试通过」不构成守护生效的证据(LumioClient T-00003 的「build 通过即为不使用 Socket 的证据」在闸门空转期间等于零校验);②记录型守护(重生成比对、哈希登记)必须锚定**已提交对象**(`git archive` 只读物化优于 `git worktree add`,后者会向被并发编辑的仓写 worktree 注册记录),并把「产物未被手改」与「与上游同步」拆成两条独立检查——后者只报告不 fail,否则上游每改一次生成器就打断下游;③C# 仓引入 BannedApiAnalyzers 时,AdditionalFile 必须命名为 `BannedSymbols.txt` 或 `BannedSymbols.<描述>.txt`;④**探针要打在「判据本身会不会变」上,不能只打在「改坏了会不会被抓」上**——改 manifest / 填假 commit / 手改 JSON 这类探针全绿,却完全测不到行尾转换那一面;真正抓住它的探针是注入 `core.autocrlf=true` 再跑一遍看结论动不动。跨机比对哈希的地方一律加这类「环境变量注入」探针;⑤**闭合到哪一步就只声称到哪一步**:修完行尾一层后极易立刻写出「判据是 commit 的纯函数」,而全局 `~/.gitattributes` 的 `* eol=crlf` 仍能压过 `-c core.eol=lf`(实测注入敌意 `core.attributesfile` 后干净树报 DRIFT);正确表述是「`(commit, attribute 栈)` 的函数」,并点名未闭合通道(`$GIT_DIR/info/attributes` 无 config 开关、`filter` smudge)。彻底闭合需直接哈希 blob。
- 来源：2026-08-28 K[28] 三方核验与 KAT 补测(架构仓 `bcc8eb9`、`.spec/tasks/sha256-kat-and-three-way-consistency.md`);2026-08-29 SR 会话 R-00131(PR #3 → `6a2ab80`)的两轮 reviewer 证伪与修法;SL 会话 T-00007 与 SA 会话 R-00270/R-00272 的 RS0030 对照组实测
- **更正(2026-08-29,本条初版归因有误)**：初版把 GameRuntime 一例写成「provenance 是假的」。经 reviewer 证伪并由实现方独立复现:五个溯源戳里**四个是同一 commit 在 Windows/`autocrlf=true` 上的忠实渲染,不是伪造**(LF 导出与 `git cat-file blob` 逐字节相等,212 文件 0 不符),唯 `compilerHash` 两种渲染都对不上、确系读了脏工作区。**只修「引用可变来源」那一层,只是把移动靶的轴从「什么时候跑」换成「在哪台机器上跑」**,`4dfc00e`↔`66a71b0` 那次翻烧饼会换个轴无限重演。修法:`git archive -c core.autocrlf=false -c core.eol=lf -c core.attributesfile=<空> ` + pathspec 限定(顺带消掉全树导出的 42 个符号链接在 Windows 上创建失败的问题)。

### SPIKE / 审查结论文档里的 P0 行动项必须当轮落单,否则会躺在文档里过夜
- 日期：2026-08-29
- 现象：LumioClient `docs/spikes/2026-08-28-spike-hybridclr-63.md` §4.7 早在 2026-08-28 就以 probeA/probeB 对照实测记录了「`banned-public-api.txt` 完全没有生效——它是一个 no-op」,§7 风险 B3/R-2 判为「门禁虚假安全感」并写明「必须先修门,再写 loader」,§6 列出 P0-1/P0-2/P0-3 修法与判据。结论正确、证据充分、修法具体——但**没有一条落成任务卡**,于是挂了一夜无人执行,直到次日另一会话独立重复验证后才被处理。同期 LumioServer 又独立踩中同一坑,重复付出发现成本。
- 根因：结论文档的交付定义止于「文档合入」,其行动项没有落单机制;文档评审关注「结论对不对」,不检查「行动项去哪了」。
- 规避：①SPIKE / 审查 / 评估类文档交付时,**P0 级行动项必须在同一轮内落单**,或显式记为「待授权立卡」并写进交回物的 known gaps——两者必居其一,不得只留在正文里;②评审此类文档的卡时,把「P0 行动项是否已落单或显式挂起」列为验收项,不因结论正确就整体放行;③总调度收到含 P0 行动项的文档交付,当轮决定落单归属。
- 来源：LumioClient R-00256(SPIKE-HYBRIDCLR-63)交付与 2026-08-29 SL 会话自我更正;总调度已在 R-00256 补执行状态评论(P0-1/P0-2 已由 T-00007 承接,P0-3 未执行)

### 交回物证据必须引用已推送 origin 的提交,核验第一步是 ls-remote 比对
- 日期：2026-08-28
- 现象：六张「验收中」卡(CoreEngine R-00011..14、Server R-00186/00188)的证据评论引用的提交不在 origin——四张引用 Windows 工作区未推送的本地提交,两张引用的提交虽在 origin 但**不包含所声称的交付文件**(R-00188 声称交付 Cargo workspace,实际引用的是纯文档提交)。同类问题一次核验中发现六处,跨两仓复发。
- 根因：单人多机开发(Windows `C:/Work/LumioGames` 与 macOS `~/LumioGames`),交付会话在本地 main 记证据即流转状态,未推送、未做「提交内容 ⊇ 改动清单」比对;核验方也未曾以 origin 为准复核。
- 规避：①证据评论只准引用**已推送 origin** 的提交号;②总调度核验清单第一步固定为 `git ls-remote` 确认提交在 origin + `git show --stat` 确认提交内容覆盖改动清单;③不满足的卡不得停留在「验收中」以上状态,补差异评论并要求推送后重核。
- 来源：2026-08-28 总调度对账(docs/reviews/2026-08-28-seven-repo-progress-assessment.md §4.2;六张卡各有差异记录评论)

### 工具链 pin 落卡前先在两台开发机上实测可满足
- 日期：2026-08-28
- 现象：环境性失配一轮对账发现三处:①VoxelEngine 全部测试因 Windows 缺 `link.exe` 从未链接执行(2.4 万行代码只过了 cargo check,审查被迫 RETURN);②GameRuntime `global.json` 锁 SDK `10.0.11` 且 rollForward=disable——该 SDK 版本不存在(是 runtime 版本号),任何机器都无法字面满足,macOS 直接 SDK_MISMATCH 不可构建;③macOS rustup 默认 `x86_64-apple-darwin`(Rosetta),与 aarch64 实机不符。
- 根因：pin 与验收口径按单机现状写死,落卡前未在第二台机器验证过;「验证通过」的声称建立在 type-check 而非链接执行上。
- 规避：①任何 SDK/工具链 pin 写入卡面或仓配置前,须在 Windows 与 macOS 两侧实测该 pin 可解析;②版本 pin 用「SDK 族 + runtime 版本」双口径,不锁不存在的字面值;③`cargo check` 通过不得当作测试证据,验收必须有链接执行的测试输出;④Rust 仓 evidence 记录 host triple。
- 来源：2026-08-28 总调度对账(同上报告 §6.2;R-00112 对账评论、R-00203 审查链)
