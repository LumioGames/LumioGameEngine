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
