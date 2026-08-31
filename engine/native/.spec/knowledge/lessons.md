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

### 新写下的判据，必须在同一提交里有一条按它构造的反例测试

- 日期：2026-08-29
- 现象：写完一条规则，然后在**同一个提交里**违反它，两次同型。① ADR 0007 第 2 节明写「只用 `toml` crate 读 `*.compose.toml` 与 `tools.lock.toml`」并否决自研子集解析器，而同提交的 `toolchain.rs` 就是手写扫描——把 `supported_hosts` 写成语义相同的合法多行数组即误报「登记缺失」，错误信息还反向误导。② ADR 0009 第 1 节写下「被背书者与背书者必须不同源」，而同提交的 `verify_generated` 把 descriptor 的 `validatorRan`/`entrySymbol` 从被校验对象自己取回去参与重建，逐字节比对对这两个字段恒真——只改 descriptor 一个字段、不动任何产物即可放行。相邻的同族形态还有三次：R-00016 的「平行结构靠注释保持同步」、R-00017 的「由仓级 `cargo tree` 断言覆盖」（该断言从未创建）、R-00018 首版的「descriptor 完整性由它记的每一条间接证明」。
- 根因：写规范与执行规范之间没有机械检查，而**刚写完规范时最容易觉得自己已经遵守了**——判据在脑子里是新鲜的，于是省掉了验证。这类缺陷通读代码发现不了：五次里没有一次是读出来的，全部来自实跑构造的反例。已有的 grep 自验规则只能证明「被声称的 X 存在」，证明不了「X 覆盖的范围等于声称的范围」。
- 规避：① **判据与它的反例测试同时诞生**——新增一条 ADR 判据或「由 X 保证」的声称时，同一提交内必须有一条按该判据构造的**失败**用例（`replacing_a_self_reported_descriptor_field_is_caught` 就是 ADR 0009 第 1 节的那条，它本该和第 1 节同时写出来）。② 反例的**构造方式要覆盖多种形态**：往文件追加一个字节与替换某个字段的值，是完全不同的覆盖面——前者必然改变字节因而总被抓到，后者可能落在自证盲区里。③ 声称「由 X 覆盖」时，除 grep 验证 X 存在外，再问一句「X 挡不住的是什么」，把答案写进文档而不是省略。
- 来源：R-00016 / R-00017 / R-00018 三张卡的 reviewer 退回报告（R-00018 经两轮退回，提交 `4a37934` → `7e7447e` → `56e0be5`）。

### 跨仓 / 跨会话引用交付时，锚点用 `origin/main:<路径>`，不用裸 commit SHA

- 日期：2026-08-28
- 现象：证据评论与设计文档里把本地 commit SHA 当作可复核锚点，锚点随后失效。一天内在四个仓以**四种不同形态**发生：本仓 R-00011..R-00014 四张卡的证据评论引用 `015035b`/`d668426`/`06c954f`/`68e1442`，推送前 rebase 导致四个 SHA 全被重写，且 ADR-0004 正文两处引用同步悬空——而 ADR §4「现状对照」正是 R-00013 验收项的核心依据；LumioGameRuntime 11 张卡引用的 6 个 SHA 只存在于未推送的本地分支；LumioServer 跨仓设计文档引用架构仓分支 SHA `f426278`，合并时被重写成 `a738524`；LumioNativeCore 从「不在 origin/main」推出「未推送」，而该提交其实在特性分支上已推送。
- 根因：commit SHA 不是稳定标识符。rebase、squash merge 都会重写它，而「已提交」「已推送」「已进 main」是三种不同状态，单一 git 命令只能证伪其中一种。更隐蔽的是跨仓引用——它不在任务系统里，没人会去验证它。
- 规避：① **锚点优先用 `origin/main:<路径>`**（分支 + 路径），rebase 打不掉；必须写 SHA 时只写已进 `origin/main` 的。② 引用前按需组合判据，各挡一种形态：`git cat-file -t`（对象存在）→ `git branch -r --contains`（已推送，但读本地 ref 会因未 fetch 而骗人）→ `git ls-remote --heads`（直连远端，且要看**全部分支**不只 main）→ `git merge-base --is-ancestor <sha> origin/main`（已进主干）。③ 结论里带上**测量时刻**——跨仓状态分钟级变化，本轮实测架构仓 40 分钟内 `origin/main` 前进三次、特性分支两次改名、PR 从 OPEN 变 MERGED。④ 扫描历史 SHA 时正则要加 commit 语境限定，裸 `\b[0-9a-f]{7,40}\b` 会把 Workflow UUID 前缀与 sha256 片段当成假阳性，反向导致不必要的返工。
- 来源：本仓 R-00011..R-00014 的 macOS 复核（订正提交 `d87e12e`）；跨会话通报回执 lumiogameruntime-22 / lumioserver-2d / lumionativecore-79。

### 在本机产出的构建与性能证据，必须标注宿主人格

- 日期：2026-08-28
- 现象：开发机是 Apple M5（arm64），但 `rustup` 钉定的 1.89.0 只有 `x86_64-apple-darwin`，`.NET SDK` 整个是 `RID: osx-x64`。于是本机产出的一切构建产物与时延数字都是 x64-on-Rosetta，若不标注就会被当成原生 arm64 结果。派生问题：`uname -m` 在同一台机器上因调用路径不同得出两个值——经 x86_64 二进制（如 `just`）间接调用得 `x86_64`，直接 `bash` 调用得 `arm64`，任何按 `uname -m` 推导 host key 的门禁都会因此不稳定。
- 根因：Rosetta 翻译的是二进制而非 shell（`sysctl.proc_translated` = 0），但 x86_64 进程派生的子 shell 会继承 x86_64 人格。工具链的 host 三元组与机器真实架构脱钩，而脚本通常只问后者。
- 规避：① 证据里显式写明宿主三元组（如 `x86_64-apple-darwin under Rosetta on arm64`、`RID: osx-x64 (Rosetta on arm64)`），不得声称原生。② host 推导改用编译期目标三元组或显式配置，不用 `uname -m`。③ 偏离钉定工具链的验证只能作**补充证据**单列，不得顶替正式验收证据——本轮用本机已有的 `1.94.0-aarch64-apple-darwin` 做过一次原生 arm64 交叉复验（全 workspace 构建通过、7 个 CLI 行为与 x86_64 腿一致），但正式证据仍以钉定的 1.89.0 为准。
- 来源：本仓 R-00011 macOS 复核发现 F3（`tools/verify-tool-lock.sh` 的 host key 同机两值）；跨会话回执 lumionativecore-79（arm64 腿未覆盖）、lumioserver-2d（.NET RID 同构问题）。
