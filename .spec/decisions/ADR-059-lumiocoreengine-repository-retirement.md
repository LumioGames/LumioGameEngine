# ADR-059：LumioCoreEngine 仓库退役与 Owner 指针归属

状态：Accepted（2026-09-04，Owner 裁决：本地与远端一并删除）
取代：无（不取代任何 ADR；为旧 ADR 中 `LumioCoreEngine` Owner 指针提供唯一前向重定向）
Owner：`LumioGameEngine`（`engine/native/` 聚合根）

## 背景

`LumioCoreEngine` 曾是 Native 发布层的独立仓库：锁定 NativeCore/VoxelEngine 的 Source 与 Feature，承担 Root ABI、Loader、ArtifactIndex、签名、SBOM 与平台产物边界。2026-08-31（该仓 `9488468`）它被标记 `DEPRECATED`，全部活跃职责迁入本仓 `engine/native/`；此后未再接受任何功能提交。

2026-09-04 Owner 裁决把它从「保留只读」推进到「删除」：本地工作副本与 GitHub 远端仓库（`LumioGames/LumioCoreEngine`）一并删除。删除前核实无未推送提交（14 个本地分支均不超前 `origin/main`）、无 stash、两个 detached worktree 的提交均已在 `origin/main`、远端 0 fork / 0 open PR / 0 open issue。

删除留下两类指向已消失仓库的引用，需要一个落档的归属结论，否则每个读到旧 ADR 的人都要重新推断一次：

1. **12 条 `Historical` 段 ADR** 以 `LumioCoreEngine` 为 Owner——ADR-006、017、018、019、020、040、041、042、043、044、046、048。
2. **本仓生成物**中的仓库 URL：`engine/native/modules/root-abi/contracts/generated-contract-artifact.json` 的 `repository` 字段。

## 决策

1. **`LumioCoreEngine` 作为仓库实体终止存在**，不保留归档副本、不保留只读远端。其全部职责由本仓 `engine/native/` 承担，`LumioGameEngine` 是该范围的唯一 Owner。
2. **本 ADR 是旧 ADR `LumioCoreEngine` Owner 指针的唯一前向重定向。** 凡在 ADR-006、017、018、019、020、040、041、042、043、044、046、048 中读到 Owner 为 `LumioCoreEngine` 的条目，一律按 `LumioGameEngine`（`engine/native/`）理解。
3. **不就地改写那 12 条 ADR 的 Owner 行。** 它们记录的是决策作出时的事实归属，改写会伪造历史记录。本目录 README 对 `Historical` 段放开的是「修正失效的**引用路径**」，Owner 归属不属于路径。
4. **生成物中的仓库 URL 经生成源更新。** `generated-contract-artifact.json` 的 `repository` 由 `modules/root-abi/contracts/tests/generated_integrity.rs` 硬编码写出；改该生成源并重跑生成命令，生成物随之更新（该字段与生成器自身的 `generatorSha256` 一并变化）。生成物不手改。
5. **`engine/native/` 内的遗留命名不在本次范围。** `justfile` 抬头、模块 README、`about.hbs`、`tools/tools.lock.toml` 的 owner 字段等仍写作 `LumioCoreEngine`，属命名债、不是死链。其中 `tools/verify-architecture-lock.sh` 的 `consumer_name="LumioCoreEngine"` 是**活契约标识**——与镜像 `packages/index.json` 的 `consumers` 条目逐字匹配，擅改即弄坏 architecture lock 校验。整体更名需单独决策与配套改镜像，另起 ADR。

## 替代方案

- **远端改 Archive（只读保留）而非删除**：本可保住全部历史与 URL 可达性，是删除前提给 Owner 的推荐项；Owner 明确选择删除，理由是「如无必要勿增实体」——一个既无消费者又无新提交的仓库不值得继续占据组织命名空间。**代价已知并接受**：GitHub 约 90 天恢复时窗过后，该仓历史不可恢复。
- **就地改写 12 条 ADR 的 Owner 行**：拒绝——见决策 3，伪造历史归属；且一处仓库退役散成 12 处静默改动，读者无从知道发生过什么。
- **不落 ADR，只在 README 写一句**：拒绝——Owner 归属变更是跨边界决策，`decisions/` 是全仓唯一落点；README 不是决策载体。
- **顺手把 `engine/native/` 内全部 `LumioCoreEngine` 字样改名**：拒绝——见决策 5，其中含活契约标识，属独立工程，混进来会让本次改动无法安全审查。

## 接口

无公共接口（wire / ABI）变化。`engine/abi/native-abi.json` 与 `engine/wire/` 下全部契约不变。

唯一受影响的生成物字段：`engine/native/modules/root-abi/contracts/generated-contract-artifact.json` 的 `repository`，由 `https://github.com/LumioGames/LumioCoreEngine` 变为 `https://github.com/LumioGames/LumioGameEngine`。该文件的 `kind` 是 `contracts-wrapper-generation-record`（本仓本地生成记录），`repository` 是其溯源字段，不在 ADR-023 定义的 `generated-contract-artifact.schema.json` 必填集内，因此该 schema 无需变更。

生成源：`engine/native/modules/root-abi/contracts/tests/generated_integrity.rs`。
生成命令：`LUMIO_CONTRACTS_REGENERATE=1 cargo test -p lumio-core-contracts --locked --test generated_integrity`。

## 失败语义

无运行时失败语义变化。唯一可失败点是生成物与生成源不一致：`generated_integrity` 测试在无 `LUMIO_CONTRACTS_REGENERATE` 时比对生成结果与在库文件，不一致即测试失败——这正是防止生成物被手改的机制，本次改动保持该机制不变。

## 兼容影响

不影响任何运行时行为、二进制边界或 wire 契约。`architecture.lock.json` 的 `repository`（指向 `LumioGameEngineArchitecture`）不变——该 URL 是本仓改名前的旧名，GitHub 重命名重定向仍可达，不是死链；其归一化属命名债，与本 ADR 同理另议。

对读者的影响：旧 ADR 中的 `LumioCoreEngine` Owner 指针不再指向可访问的仓库，须经本 ADR 重定向理解。

## 迁移方案

一次性完成，无分阶段：

1. 改生成源 `generated_integrity.rs` 中的仓库 URL。
2. 跑生成命令重生成 `generated-contract-artifact.json`（`repository` 与 `generatorSha256` 同时更新）。
3. 修正 `README.md` / `README.en.md` 中被删除动作证伪的表述——原文称该仓「仅保留历史审计与回滚用途」，仓库已不存在。
4. 新增本 ADR 并登记进 `decisions/README.md` 索引。

下游仓库无需动作：无任何仓库以 submodule、包依赖或 remote 形式引用 `LumioCoreEngine`（已全仓 grep 核实，`.gitmodules` 无命中）。

## 验证

- `node .spec/tools/spec-lint.mjs` —— `.spec/` 结构与索引登记一致性。
- `cargo test -p lumio-core-contracts --locked --test generated_integrity`（不带 `LUMIO_CONTRACTS_REGENERATE`）—— 证明生成物与改后的生成源一致，且未被手改。
- `node eng/generate-abi.mjs` —— 证明本改动对 ABI 生成零差异。
- `gh repo view LumioGames/LumioCoreEngine` 返回 `Could not resolve to a Repository` —— 远端已删除的事实证据。
