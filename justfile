# LumioCoreEngine 统一命令入口（LCE-P0-001）。
# just 只是命令 facade：逐字转发到锁定 crate CLI，不承载契约语义（规格 §1.4）。
# 当前所有 CLI 均为 BlockedOnArchitectureGate 守卫，执行即结构化报错并退出码 5（预期行为）。
# 依赖 Git Bash 提供 sh（与 tools/*.sh 相同）；sync-contracts/runtime-deps 等后续任务接入。

set shell := ["sh", "-cu"]
set windows-shell := ["sh", "-cu"]

# P0 唯一 TargetProfile（CF-11）：Linux Server / x86_64 / glibc / DynamicLibrary。
profile := "p0-linux"

default:
    @just --list

# ── 基础门禁 ────────────────────────────────────────────────────────────────

# 合并门禁：格式 + 静态检查 + 供应链（deny / about / 工具锁）。
# `nextest` 需要可链接二进制的主机（本仓规格 §20.1 CI 门禁含它），单独 recipe 提供；
# schema 检查与 smoke 分别由 LCE-P0-002 / LCE-P0-017 接入。
check: fmt-check clippy deny about tool-lock runtime-deps

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

deny:
    cargo deny check

about:
    mkdir -p build/reports
    cargo about generate --config about.toml about.hbs -o build/reports/licenses.md

tool-lock:
    bash tools/verify-tool-lock.sh

# 运行时发布闭包的依赖断言（R-00017 / LCE-P0-007 验收项 3、4）。
#
# 为什么必须是门禁而不是「跑一次 cargo tree 看一眼」：这两条是**不变量**，手工跑一次
# 只证明当下状态。没有它，任何人给运行时闭包内的 crate 加一条
# `features = ["test-support"]` 的 normal 依赖，全套门禁都会绿灯放行。
#
# 两条断言：
#   1. 全 workspace 的 normal 依赖图里不得有人启用 `test-support`——该 feature 只允许
#      经 dev-dependency 启用（resolver v2 在非测试构建中不统一 dev-dep 的 feature）。
#   2. platform-contracts 的 normal 依赖集合必须**恰好**是 {lumio-core-contracts}——
#      它在运行时发布闭包内（规格 §3.7），OS 细节归 platform-runtime。
#      用白名单而不是「不含 libc/rustix/…」的黑名单：黑名单对没列进去的新 OS crate
#      （nix / windows / mach2 / core-foundation…）天然漏网，而这里的合法集合只有一项，
#      白名单的成本几乎为零且对 crate 改名免疫。
runtime-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "runtime-deps: 断言 test-support 不在任何 normal 依赖路径上"
    if cargo tree --workspace -e normal --format '{p} {f}' | grep -n 'test-support'; then
        echo "runtime-deps: FAIL: 上列 normal 依赖启用了 test-support；该 feature 只允许经 dev-dependency 启用" >&2
        exit 1
    fi
    echo "runtime-deps: 断言 platform-contracts 的 normal 依赖恰好是白名单"
    expected='lumio-core-contracts lumio-core-platform-contracts'
    actual=$(cargo tree -p lumio-core-platform-contracts -e normal --prefix none --format '{p}' \
        | awk '{ print $1 }' | sort -u | tr '\n' ' ' | sed 's/ $//')
    if [ "$actual" != "$expected" ]; then
        echo "runtime-deps: FAIL: platform-contracts 的 normal 依赖集合是【$actual】，白名单是【$expected】" >&2
        exit 1
    fi
    echo "runtime-deps: OK（test-support 未泄漏；platform-contracts normal 依赖恰为白名单）"

# nextest 0.9.114 起不再自动发现仓库根 nextest.toml（默认改查 .config/nextest.toml），
# 本仓按规格 §3.1 布局保留根文件，故显式 --config-file。
# --no-tests=pass：门禁在仓库任一合法状态下都可判定（B-00001）——nextest 默认
# 「零测试即失败」，会让首张带测试的卡落地前 §20.1 门禁必然红；真实测试失败仍失败。
nextest:
    cargo nextest run --workspace --profile ci --config-file nextest.toml --no-tests=pass

# ── 架构契约镜像（LCE-P0-002）─────────────────────────────────────────────

# 从锁定的架构源提交重建只读镜像（architecture.lock.json 为唯一真值，规格 §3.4/§3.5）。
# 内容一律经 git 从 pin commit 提取，绝不读源仓工作区与 docs/architecture/；
# 源仓定位：LUMIO_ARCHITECTURE_REPO 环境变量或同级 ../LumioGameEngineArchitecture。
# lock 更新须显式 `bash tools/sync-architecture.sh --update-lock` 且单独 PR。
sync-contracts:
    bash tools/sync-architecture.sh

# 校验镜像与 architecture.lock.json 完全一致（逐文件 SHA-256、缺失/漂移/未登记文件清单、
# packages/ consumers 断言；输入仅本仓 lock 与镜像，不触架构源仓，也不读 docs/architecture/）。
check-contracts:
    bash tools/verify-architecture-lock.sh

# 按 lock 所 pin 的提交号另行获取架构仓校验器工具链（tools/** → build/architecture-tools/
# <commit>/，不提交）。tools/** 是实现而非契约，不入镜像与 requiredPaths（R-00263/D-5：
# 上游改 tools/ 不再打断本仓门禁）。
fetch-architecture-tools:
    bash tools/sync-architecture.sh --fetch-tools

# ── 垂直切片命令（转发到 crate CLI；当前全部 blocked，见各 CLI 守卫） ────────

# 断言只接受 P0 profile。
[private]
assert-profile p:
    @test "{{p}}" = "{{profile}}" || { echo "未知 profile：{{p}}（当前仅支持 {{profile}}）" >&2; exit 1; }

compose p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-composition --bin lumio-core-compose -- compose --config config/p0/linux-server-x86_64-glibc.compose.toml --out build/plans/p0-linux-server-x86_64-glibc/build-plan.json

generate-abi p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-root-abi-generator -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --architecture-lock architecture.lock.json --out modules/root-abi/generated/LGE-V1.4-2026-08-27

# 生成物完整性（规格 §20.1「重新生成零差异」）。两段：
#   1. lumio-core-contracts 的锁定生成器校验（LCE-P0-003：descriptor Input/Output Hash
#      字节重算、上游 provenance 与镜像对账、重渲染零差异）；
#   2. root-abi 生成目录的回读校验（LCE-P0-005，本 recipe 原注释预留的接入点）——
#      逐份产物与上游 bundle 声明摘要对账、descriptor 按同一规则重建后逐字节比对、
#      文件集合与登记表完全一致。没有这一段，手改生成物不会被任何门禁发现。
check-generated:
    cargo test -p lumio-core-contracts --locked --test generated_integrity
    cargo run --locked -q -p lumio-core-root-abi-generator -- verify --root modules/root-abi/generated/LGE-V1.4-2026-08-27 --architecture-lock architecture.lock.json

build-platform p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-platform-build -- build-staging --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --plan-digest-file build/plans/p0-linux-server-x86_64-glibc/build-plan.sha256 --abi modules/root-abi/generated/LGE-V1.4-2026-08-27 --out build/platform/linux-server-x86_64-glibc/staging

evidence p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-evidence-generator -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --staging build/platform/linux-server-x86_64-glibc/staging --out build/evidence/linux-server-x86_64-glibc

manifest p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-manifest -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --abi-descriptor modules/root-abi/generated/LGE-V1.4-2026-08-27/generated-contract-artifact.json --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --artifact-index build/platform/linux-server-x86_64-glibc/finalized/metadata/artifact-index.json --evidence build/evidence/linux-server-x86_64-glibc --out build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.json

sign-test p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-signer-tool --features test-provider -- sign --manifest build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.json --manifest-digest-file build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.sha256 --trust-domain Test --provider test-file --key-file modules/smoke/fixtures/test-keys/p0-ed25519-private.key --out build/platform/linux-server-x86_64-glibc/finalized/metadata/signature-envelope.json

verify p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-smoke -- verify-package --package-root build/platform/linux-server-x86_64-glibc/finalized --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --trust-metadata modules/smoke/fixtures/test-keys/p0-ed25519-public.json --report build/reports/verify-package.json

load-smoke p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-smoke -- load --package-root build/platform/linux-server-x86_64-glibc/finalized --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --trust-metadata modules/smoke/fixtures/test-keys/p0-ed25519-public.json --report build/reports/load-smoke.json
