---
status: in_progress
---

# 锁定并只读镜像 Architecture V1.2 输入（LCE-P0-002 / Workflow R-00012）

一句话：以 `architecture.lock.json` 钉死架构源仓 pin 提交的 required paths 与逐文件 SHA-256，由 `sync-contracts` 从 pin 提交重建只读镜像、`check-contracts` 离线校验，任何缺失/漂移/未登记文件逐条报错。来源规格：`docs/LumioCoreEngine_Framework_Scaffolding_Spec_v1.0.md` §3.4—§3.6、§18 LCE-P0-002（基线 commit f3c9920）；架构基线 `LGE-V1.2-2026-08-27` @ `2d7980d95b163404e33cc6212db13ac948d30d40`。

## 涉及范围

- `architecture.lock.json`（新增；仅可由 `tools/sync-architecture.sh --update-lock` 生成，本卡由该命令产出）
- `generated/architecture/LGE-V1.2-2026-08-27/`：`schemas/`（31）、`ids/`（2）、`fixtures/`（72）、`decisions/`（24，源 `.spec/decisions/`）、`tools/`（2）共 131 个 required path + 镜像根 `generation-record.json`（§3.6 生成记录）与 `.gitattributes`（`* -text` 字节冻结声明）
- `tools/sync-architecture.sh`（新增）与 `tools/verify-architecture-lock.sh`（新增，均已 chmod +x）
- `justfile`：新增 `sync-contracts`、`check-contracts` 两个 recipe（其余 recipe 未动）
- 本任务卡

## 验收标准

- [x] `just sync-contracts` 成功（exit 0）
- [x] `just check-contracts` 成功（exit 0，重复运行稳定）
- [x] 手改任一镜像字节后 `just check-contracts` 稳定非零（连续 3 次 exit 1，漂移逐条列出）
- [x] 删除 required path 后 `just check-contracts` 明确列出缺失路径（exit 1）
- [x] `docs/architecture/` 不被用作生成输入（结构 + 行为双证据）
- [x] 源内容与 lock 不同 → sync 失败且不更新 lock；lock 只能经显式 `--update-lock` 重生成
- [x] 镜像 131 文件与 pin 提交逐字节一致（独立 cmp 交叉验证）

## 依赖

- 0001-workspace-tool-locks（R-00011：workspace/justfile/工具锁；本卡在其交付上叠加）

## 证据记录（在途；验收勾选归主 loop/reviewer）

实现前 Red（基线 48f109b，worktree wf/r12）：

- `just sync-contracts` / `just check-contracts` → `error: justfile does not contain recipe …`，均 exit 1；`architecture.lock.json`、`generated/architecture/LGE-V1.2-2026-08-27/`、两脚本均不存在（ls exit 2）。

实现后 Green（Git Bash，just 1.58.0）：

- `bash tools/sync-architecture.sh --update-lock` 首次生成 lock+镜像：131 required paths，pin 提交枚举 `schemas ids fixtures .spec/decisions tools` 五子树全集（含规格 §3.5 点名的 ABI/Manifest/ArtifactIndex/SignatureEnvelope/TargetProfile/VPD/LoggingEvent/FailureBundle/common/ID Registry/全部 Fixture/ADR-017—020/023/Canonical 工具 lumio_contract.py）。
- `just sync-contracts` exit 0（含「旧镜像替换」路径：移走旧树→rename 新树→删旧树）；输出 `lock: … 未修改（只读消费）`。
- `just check-contracts` exit 0 ×2（稳定）：逐文件 SHA-256 全一致；生成记录 SHA-256 与 lock 一致；Input Hash == Output Hash == `202b22f07d08482b8d115caff2ea6a7992f60bb96100b15120ba2137484c8d83`；compilerSha256 与 `tools/sync-architecture.sh` 一致（CRLF 归一化口径，兼容跨平台检出）；`read-only: 0/133 可写`（chmod a-w 断言生效）。
- 生成记录 `generation-record.json` SHA-256 = `421a0d4d0e010a5f1c71da7ee78f81696c13f35e646332f5532d65c2fbe5a97d`（即 lock.generatedArtifactDescriptorSha256）；执行记录（带时间戳）落 `build/reports/architecture-sync.json`（不提交）。
- 独立交叉验证：131 个镜像文件逐一 `git show <pin>:<path> | cmp - 镜像文件` → 131/131 byte-identical，0 mismatch（不经本卡工具的第三方路径）。
- `git check-attr text` → 镜像内文件 `text: unset`（镜像根 `.gitattributes` `* -text` 生效，任何 core.autocrlf 配置下检出不改字节；本机 core.autocrlf=true 已验证工作正常）。

Negative（均实测）：

- 手改 6 处镜像字节（schemas/common.schema.json、ids/index.json、fixtures/valid/id-registry.json、decisions/ADR-017、tools/lumio_contract.py、generation-record.json 各追加 1 字节）→ check exit 1，5 个 required path 漂移 + 生成记录摘要漂移 + Input/Output Hash 重算不一致逐条列出；重复 3 次均 exit 1（稳定非零）。`just sync-contracts` 恢复后 check exit 0，lock SHA-256 前后一致（`3ed50830…8645`，未被触碰）。
- 删除 2 个 required path（fixtures/valid/id-registry.json、schemas/native-managed-abi.schema.json）→ exit 1，逐条输出「缺失: generated/architecture/LGE-V1.2-2026-08-27/<path>」；重跑仍 exit 1；sync 恢复后 exit 0。
- 镜像塞入未登记文件 schemas/extra-rogue.json → exit 1「未登记」逐条列出；移除后恢复。
- 篡改 lock.commit（改 aaaa…）→ check exit 1「与钉死值不符（基线漂移）」；恢复后 OK。
- `docs/architecture/` 非生成输入：结构上两脚本与 justfile 对 `docs/architecture` 的引用只出现在「绝不读取」注释；行为上向 `docs/architecture/LumioGameEngine_Architecture_v1.2.md` 追加篡改探针后 `just sync-contracts` exit 0 且镜像整树哈希前后一致（`61b16e04…9523`），check 仍绿；docs 文件已还原。
- 源内容与 lock 不同：本地 scratch clone（`git replace` 使 pin 提交解析到被篡改内容）+ `LUMIO_ARCHITECTURE_REPO` 指向它 → sync exit 1，输出 lock→源差异（schemas/common.schema.json 期望/实际 SHA-256），lock 与镜像均未被修改，check 仍绿；scratch 已清理，架构源仓只读未动。
- 源仓无 pin 提交（空仓）→ exit 1「锁定提交不存在（基线不可复现）」；origin 指向他仓 → exit 1「源仓身份不符」；未知参数 `--oops` → exit 1。

收口门槛：

- `node .spec/tools/spec-lint.mjs` → OK（exit 0）。
- `node --test .spec/tools/spec-lint.test.mjs` → 结果见交付报告（R-00011 已记录该测试在 Windows 宿主因 EPERM symlink 预先失败，非本卡引入）。

备注（设计口径 / 非本卡改动）：

- `generatedArtifactDescriptorPath/Sha256` 解释为镜像生成记录 `generation-record.json`：pin 提交上不存在上游 `generated-contract-artifact.json`（LGE-GATE-P0-001 未关闭，伪造公共制品被明确禁止）；上游制品发布后经显式 `--update-lock`（单独 PR）进入 requiredPaths。该解释已在交付报告 Deviations 声明，供 reviewer 裁决。
- `requiredPaths` 为架构源仓内路径（`.spec/decisions/**` → 镜像 `decisions/**` 投影），lock = 纯源仓 pin；镜像投影规则固定在两脚本内。
- `--update-lock` 的 commit/baseline/repository 由脚本常量钉死：不会因相邻仓出现 V1.4 而静默升级；升级基线 = 改脚本常量 = 必然单独评审。
- 镜像只读位不随 git 传输（git 仅记录 exec bit）；重新检出后 check 的只读计数为通告项，`just sync-contracts` 重设只读。本卡将 check 的只读检查定为通告而非失败，避免 CI 以 root 运行或重新检出后误红。
- `just check` 聚合 recipe 未加入 check-contracts（本卡文件集限制「只加 recipe、不动其他 recipe」）；聚合接线留待后续卡/主 loop。
- 会话开始时 `.agents/skills`、`.claude/agents`、`.claude/skills` 三个 symlink 已被外部进程改写致 git status 显示 D（R-00011 已记录同象），本卡未触碰；`.spec/decisions/0004-*`（并行方 R-00013 文件）未触碰。
- 主 loop 提交注意：mirror 内 133 文件建议整目录 `git add generated/architecture architecture.lock.json tools/sync-architecture.sh tools/verify-architecture-lock.sh justfile .spec/tasks/p0/0002-architecture-lock-mirror.md`（勿 `git add -A`，会带入 symlink 删除）；两脚本执行位已 chmod +x。
