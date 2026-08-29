# 2026-08-28 · NativeCore 收敛派活提示词(NC-W1,三道并行)

> 来源盘点:[`../reviews/2026-08-28-nativecore-convergence-audit.md`](../reviews/2026-08-28-nativecore-convergence-audit.md)。
> 三道卡集与文件集互不重叠,可并行;各自在 `~/LumioGames/LumioNativeCore` 开工。
> 工作内容本体在 Workflow 卡上,本提示词只指路 / 立规 / 设禁区。

## 公共纪律(三道通用,嵌入每份提示词)

① 领卡先流转「实现中」(reason 写明);② 证据评论只引用已推送 origin 的提交号,先 push 再回写;③ 测试证据必须是链接执行的真实输出,`cargo check` 不算;④ 交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点,本仓收口门槛必过;⑤ 公共契约缺口 → 停,卡上评论标 BLOCKED 上报,不本地绕过、不手写第二套 Schema/数值;⑥ 只动本仓文件;⑦ 做完流转「验收中」,「已完成」由总调度核验后流转;⑧ **动手前必开隔离 git worktree**(多会话共用工作区,严禁在共享工作区切分支/提交);⑨ 有问题或交付完成用 `SendMessage` 回报派单的架构仓 TD 会话(`ListAgents` 可见),不新开会话。

---

## 道 A · 契约绑定收敛(代码)

```
你是 LumioNativeCore 仓的执行会话,任务:把上游已发布的 Root ABI 真正绑定进本仓,收敛 8 张受波及卡。

【指路】
- 背景事实(已核实,2026-08-28):上游 LumioGameEngineArchitecture origin/main(≥ c712ff4,当前 d812617)已发布:
  · packages/abi/lumio_core.h + packages/abi/root-abi-bundle.json(ADR-040;entrySymbol lumio_core_get_api_v1、layoutProfile linux-x86_64-glibc、tables、typeMapping、compiler.digest、inputHash);
  · packages/index.json 的 rootAbi.consumers 登记了 LumioNativeCore(本仓只消费 Root ABI Bundle,刻意不消费 Rust/C# generated packages);
  · ids/index.json 数值注册表:ErrorCode 43 值、Capability 9 值(均 Architecture 所有);OperationId namespace 未发布。
- 本仓现状:crates/lumio-contract-types/src/generated.rs 自述 "Binding is not done yet",五个 newtype 是 _private 空壳;c180bdd 只收敛了注释。
- 受波及卡与建议顺序:R-00056(adapter 绑定)→ R-00072(layout 断言,对 bundle 的 layoutProfile/Golden)→ R-00074(契约漂移 Gate,对 baselineId + bundle digest)→ R-00069(Registry 只读适配,先逐 namespace 核对 ids/index.json 覆盖度)→ R-00079(架构错误码映射)/R-00083(Capability bits)(仅当 ids/ 覆盖所需数值才解锁)→ R-00179(generated exports + C smoke,依据 lumio_core.h)→ R-00180(symbol/dependency 负向 Gate 扩展)。
- 流程:先读本仓 CLAUDE/AGENTS 与 docs/2026-08-27-native-core-module-implementation-frame.md 的对应 T-* 节;在 R-00007 补一条蓝图修订评论(r2:Root ABI 已发布,列出受影响 R-*);对每张开工卡现查 transitions 重开到工作态(reason 引用上游发布提交与本派单),做完流转回「验收中」并附证据评论。Workflow 操作用 workflow 插件技能(workflow-execute / workflow-ops)。
- 上游真值只认 LumioGameEngineArchitecture 的 origin/main 已推送提交;引用锚点先用 git branch -r --contains 核验(git cat-file 对本地未推送提交会漏报)。

【立规】
- 公共纪律 ①-⑨(见派活文档,含:隔离 worktree、证据锚 origin、链接执行输出、BLOCKED 不绕过、SendMessage 回报)。
- 收口门槛:cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo build --workspace --benches,全绿才可交付;合入 main 走 PR。
- 本机 rustup 默认 x86_64-apple-darwin(Rosetta),如需 aarch64 腿自行加 target,证据写明 host triple。

【设禁区】
- 不碰 crates/{lumio-job,lumio-spatial,lumio-codec,lumio-diagnostics,lumio-platform};lumio-kernel 只允许错误码/capability 映射直接相关文件(仅在 R-00079/R-00083 解锁时)。
- ids/ 未覆盖的数值(含 OperationId)一律 BLOCKED 评论上报,不得发明;不得复制第二套 Schema;不得把 Draft ADR 内容当 Accepted 引用而不标注。
- 只写自己 8 张卡 + R-00007 评论,不动其他卡;不建新卡/新 Room/新里程碑。
- 与 .DS_Store / .gitignore 之类仓务问题不得夹带进契约绑定提交(值得做就单独一笔 chore 提交并在交回物声明)。
```

## 道 B · 验收判定批次一(QA,不改代码)

```
你是 LumioNativeCore 仓的验收(QA)会话,任务:对 28 张「验收中」卡在系统内完成验收项判定。你不写生产代码。

【指路】
- 卡集(28 张,独占,只写这些卡):R-00010、R-00075、R-00077、R-00082、R-00084、R-00085、R-00086、R-00087、R-00088、R-00089、R-00090、R-00091、R-00092、R-00094、R-00095、R-00097、R-00098、R-00099、R-00100、R-00101、R-00102、R-00129、R-00130、R-00132、R-00160、R-00161、R-00165、R-00168。
- 背景:每卡已有交付评论与 2026-08-28 状态对账评论(锚 origin/main 0e18106),但全部验收项 systemSemantic=not_started——判定从未在系统内执行。你的工作:逐卡读单(正文+评论+验收项三路),在固定锚点(取当前 origin/main HEAD,记录之)的隔离 worktree 里实跑每条验收项对应的检查,逐项在 Workflow 里判定通过/不通过,并附一条汇总证据评论(命令+关键输出+host triple+锚点提交号)。
- Workflow 操作用 workflow 插件技能(workflow-qa 做验收判定;验收类型/状态值是项目自定义,必须现查,不得猜)。
- 判定纪律:逐条对照验收项字面口径;能跑的必须跑;通过才判通过。

【立规】
- 公共纪律 ①-⑨;尤其:③ 链接执行的真实输出才算证据;⑤ 发现验收口径与代码事实不符 → 不硬判,评论记差异并上报;⑦ 卡状态保持「验收中」,不流转 done(总调度收口)。
- 实跑一律在你自建的隔离 git worktree(锚 origin/main),不污染共享工作区,收尾 git worktree remove。

【设禁区】
- 不改任何仓库文件、不提交、不 push;只写上列 28 张卡的验收项判定与评论。
- 无法在本机(macOS)验证的项(如三平台 smoke 的 Windows/Linux 腿)不判通过:留未判定 + 缺口评论(写明缺什么宿主),汇总进回报。
- 不动其他卡,不建新卡。判定拿不准的项列入回报由总调度裁决。
```

## 道 C · 验收判定批次二(QA,不改代码)

```
你是 LumioNativeCore 仓的验收(QA)会话,任务:对 31 张「验收中」卡在系统内完成验收项判定。你不写生产代码。

【指路】
- 卡集(31 张,独占,只写这些卡):R-00103、R-00105、R-00106、R-00107、R-00108、R-00109、R-00110、R-00111、R-00113、R-00114、R-00115、R-00117、R-00118、R-00120、R-00122、R-00123、R-00124、R-00125、R-00126、R-00128、R-00144、R-00147、R-00148、R-00156、R-00158、R-00171、R-00173、R-00175、R-00177、R-00185、R-00261。
- 其余同道 B 的【指路】【立规】【设禁区】逐字适用(锚点、worktree、workflow-qa、判定纪律、禁区一致)。
- 附注:R-00261 的证据锚是 03d6bd7(PR #1);其五条验收项以 origin/main 当前 HEAD 复验为准。codec/diagnostics 卡注意 feature-gated(default-off)口径——验收命令若需 --features 按卡面/交付评论执行。
```

---

## 收口(NC-W2,总调度自留)

A/B/C 全部 SendMessage 回报后:抽样复核 → 批量 acceptance→done 流转(逐卡 reason)→ R-00007 蓝图收尾评估 → 视 A 道上报决定是否请授权在架构仓立注册表增补卡。
