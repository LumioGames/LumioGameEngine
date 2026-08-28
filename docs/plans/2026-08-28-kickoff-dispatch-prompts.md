# 开工派活提示词(2026-08-28 · W0/W1)

> 每仓一份、可直接粘贴到该仓新会话的开工提示词。**工作内容的唯一真值在 Workflow 需求单**,提示词只负责指路与纪律;卡片正文/评论/验收项才是做什么的依据。
> 依据:[七仓进度评估与对账报告](../reviews/2026-08-28-seven-repo-progress-assessment.md);机制:`.spec/skills/cross-repo-delivery`、`.spec/skills/td-progress-audit`。

## 公共纪律(已内嵌进每份提示词)

① 开工先把领的卡流转到「实现中」(reason 写明);② 证据评论**只引用已推送 origin 的提交号**,先 push 再回写;③ 测试证据必须是**链接执行**的真实输出,cargo check / type-check 不算;④ 交付 = 改动清单 + 验证证据 + known gaps + 沉淀落点,本仓收口门槛必过;⑤ 公共契约缺口 → 停,卡上评论标 BLOCKED 上报,不本地绕过;⑥ 只动本仓文件;⑦ 做完流转「验收中」,「已完成」由总调度核验后流转。

---

## 1 · LumioVoxelEngine(W0 最高优先:解除验证债)

```text
【目标仓】~/LumioGames/LumioVoxelEngine。进场先读 CLAUDE.md / AGENTS.md 指到的 .spec 三件套,全程守本仓规范。
【任务来源】Workflow(lumiogamesengine.workflow.games,本仓 .workflow 已绑定)。工作内容以需求单为唯一真值;
用 workflow 插件按单号读卡开工,读卡必须四路读全(正文+评论+附件+验收项)——评论里有 2026-08-28 的对账与架构裁决信息。
【本轮卡·按序】
1. R-00143([测试·B0])、R-00145([测试·B2])、R-00146([测试·集成] MVP 端到端):三卡在「评审中」,代码已在树上,
   缺的是真实链接执行的测试证据。本机首次真实跑 `cargo test --workspace --all-features`,修复暴露的失败
   (预算 1–2 轮),把 B0/B2/MVP evidence 从 type-check 口径改写为 linked PASS。
2. 全绿后按 R-00203([审查·MVP])卡面流程重审(目标 verdict=APPROVE),再核 R-00204(QA 放行门)前置是否满足。
3. R-00057..R-00060 决策门卡的评论载有架构所有者裁决(确认书 LGE-V1.4-VOX-D-P0-2026-08-28):
   VoxelConfigSnapshot::from_generated 可将 VOX-D-001..004 按 approved 处理;按卡完成 evidence 更新。
【环境】rust-toolchain.toml pin 1.98.0(本机需 rustup 联网下载);本机 rustup 默认 x86_64(Rosetta),
必须明确用 aarch64-apple-darwin,并把 host triple 记入 evidence。
【纪律】①领卡先流转「实现中」;②证据只引用已推送 origin 的提交;③测试证据必须链接执行,check 不算;
④交付=改动清单+验证证据+known gaps+沉淀落点,收口门槛必过;⑤契约缺口→停,标 BLOCKED 上报;
⑥只动本仓文件;⑦做完流转「验收中」,「已完成」留给总调度。
```

## 2 · LumioGameRuntime(W0→W1:主链启动)

```text
【目标仓】~/LumioGames/LumioGameRuntime。进场先读 CLAUDE.md / AGENTS.md 指到的 .spec 三件套。
【任务来源】Workflow(lumiogamesengine.workflow.games)。按单号读卡,四路读全——五张在途卡的评论里有对账与缺陷记录。
【本轮卡·按序】
0. 先修 R-00112 评论记录的 SDK pin 缺陷:global.json 锁 `10.0.11`(不存在的 SDK 版本)且 rollForward=disable,
   本机直接 SDK_MISMATCH 不可构建。改为可解析口径(语义=「SDK 携带 runtime 10.0.11 的版本族」),
   eng/verify-sdk.* 同步双口径;这是本仓一切验收的前提。
1. R-00112 / R-00127 / R-00131 / R-00133 / R-00138(均在「实现中」,交付物在 origin `fbaca12`):
   逐卡对照验收项在本机实跑验收命令,补全缺口,附真实输出证据,流转「验收中」。
2. 继续 room 内 wave(同仓串行,一张做完验一张):
   R-00139(config 校验/六层合并)→ R-00140(config snapshot/tick 激活)、R-00141(persistence canonical codec)
   → R-00149 → R-00150 → R-00152(ecs 三卡)。
【纪律】(同公共纪律七条)
```

## 3 · LumioNativeCore(补验收欠账 + I1 续跑)

```text
【目标仓】~/LumioGames/LumioNativeCore。进场先读 .spec 三件套。
【任务来源】Workflow RM-00002:66 张卡停在「评审中」,而交付已在 origin(HEAD 0fcb1f0)——本轮任务是把状态欠账补平。
【本轮做法】
1. 全量验证:`cargo build --workspace && cargo test --workspace` + xtask 检查,取真实输出(记录 host triple)。
2. 按模块分组(contract-types/error/capability/handle/memory/platform/context/job/spatial/codec/diagnostics/ffi/test-support),
   逐卡对照验收项补证据评论(origin 提交号 + 命令输出摘要),流转到「验收中」;个别验收项不满足的如实记差距,不硬流转。
3. 欠账补平后继续 I1 既有节奏(codec/diagnostics feature-gated 原型推进)。
【纪律】(同公共纪律七条)
```

## 4 · LumioClient(SPIKE 预研,4 卡可并行)

```text
【目标仓】~/LumioGames/LumioClient。进场先读 .spec 三件套。Foundation(Wave0-6)已完成,不要重做。
【本轮卡】四张 SPIKE 预研卡,互不依赖、不依赖上游,可并行领:
R-00253(SPIKE-REMOTE-AOT:Unity/IL2CPP Socket·Pipelines·TLS 验证)
R-00254(SPIKE-OTEL-IL2CPP:OpenTelemetry exporter IL2CPP 验证)
R-00255(SPIKE-UNITY-63-AOT-MATRIX:Unity 6.3 LTS 设备与 AOT 矩阵)
R-00256(SPIKE-HYBRIDCLR-63:官方版本、许可与 AOT metadata 发行路径)
R-00055(Wave7 垂直切片计划)只读不开工——多数项等 Runtime/Server 产物,待总调度发令。
【纪律】(同公共纪律七条;SPIKE 交付=可复核的验证记录/结论文档,同样要证据)
```

## 5 · LumioServer(仅设计卡;Rust 面冻结)

```text
【目标仓】~/LumioGames/LumioServer。进场先读 .spec 三件套。
【本轮卡】只开 R-00260([LumioServer] MVP C# 宿主设计:WebSocket transport/auth 存根/session/world-slot 最小面)。
设计文档卡:只出文档与首批实现卡拆分,不写实现代码;卡正文有完整目标/验收/边界。
【禁区】①不动 51 张 Rust 卡的范围与文件;②R-00186/R-00188 虽在「验收中」但证据已被判不可复核
(见卡上 2026-08-28 差异评论)——不要碰,等 Windows 侧推送后由总调度重核;③不触 protocol-dispatch(D-009 冻结)。
【纪律】(同公共纪律七条)
```

## 6 · LumioCoreEngine(守门:等 Windows 推送)

```text
【目标仓】~/LumioGames/LumioCoreEngine。进场先读 .spec 三件套。
【守门检查(第一步,必做)】
git fetch origin && git ls-tree -r origin/main --name-only | grep -c '^Cargo.toml\|/Cargo.toml'
- origin 仍无 Cargo workspace → **停**:R-00011..R-00014 的交付(15 crate workspace,约 53 文件)滞留
  Windows 工作区未推送(见四张卡上的 2026-08-28 差异评论),在 Mac 重做会造成分叉。
  回报「等待 Windows 推送」即收工,不得重新实现。
- origin 已有 workspace → 拉取后按 R-00011..R-00014 卡面验收项在本机重跑验证、补证据评论,
  然后从 RM-00004 backlog 按 wave 顺序领下一批(P0 骨架族,「缺契约即失败」形态;实现波次前置=架构仓 Gate 卡关闭)。
【纪律】(同公共纪律七条)
```

## 7 · LumioGame(设计卡开仓)

```text
【目标仓】~/LumioGames/LumioGame。进场先读 .spec 三件套。本仓当前零设计文档零模块目录,本轮是开仓第一卡。
【本轮卡】R-00259([LumioGame] 模块脚手架设计与 MVP 内容规格,设计文档卡)。
只出两份设计文档(模块脚手架设计 + MVP 内容规格:PlaceVoxelAbility/mapping/最小 config),不写实现代码;
工程基线版本口径必须在 Windows 与 macOS 双机实测可解析(吸取 GameRuntime SDK pin 教训,卡正文有说明)。
【纪律】(同公共纪律七条)
```

## 8 · LumioGameEngineArchitecture(Gate 裁决主线)

```text
【目标仓】~/LumioGames/LumioGameEngineArchitecture(架构源)。进场先读 .spec 三件套;公共语义变更严格走
ADR → Schema/ID → 正反 Fixture → README/Baseline → 七仓镜像顺序;收口门槛含 spec-lint 全套 + 契约校验。
【本轮卡·按序】
1. R-00003([LGE-GATE-P0-001] Root ABI Generated Contract Bundle):先对照 packages/ 已发布的 W1 六类 Artifact
   逐项核对覆盖度,能关即关,缺口补 Golden。
2. R-00258(TransportProfile WebSocket 档 Capability 登记):MVP A1 前置探明;不解冻 D-009/D-011。
3. R-00004(Canonical/Digest Profiles)→ R-00005(Signature/Trust)/ R-00006(Loader Fixture)/ R-00008(Evidence Profiles)。
4. R-00257(D-014 处置声明)低优先,Voxel P2 启动前完成即可;R-00009(P1)顺延。
【纪律】(同公共纪律七条)
```

---

**派活边界提醒(总调度侧):** 同仓串行、异仓并行;VoxelEngine 与 GameRuntime 是 W0 关键路径,先派;CoreEngine 在 Windows 推送前只做守门检查;「已完成」流转统一由总调度核验后执行。
