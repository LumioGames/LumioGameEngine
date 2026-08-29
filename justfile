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
check: fmt-check clippy deny about tool-lock

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

# nextest 0.9.114 起不再自动发现仓库根 nextest.toml（默认改查 .config/nextest.toml），
# 本仓按规格 §3.1 布局保留根文件，故显式 --config-file。
nextest:
    cargo nextest run --workspace --profile ci --config-file nextest.toml

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
    cargo run --locked -p lumio-core-root-abi-generator -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --architecture-lock architecture.lock.json --out modules/root-abi/generated/LGE-V1.2-2026-08-27

check-generated:
    cargo run --locked -p lumio-core-root-abi-generator -- verify-generated --architecture-lock architecture.lock.json --generated modules/root-abi/generated/LGE-V1.2-2026-08-27

build-platform p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-platform-build -- build-staging --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --plan-digest-file build/plans/p0-linux-server-x86_64-glibc/build-plan.sha256 --abi modules/root-abi/generated/LGE-V1.2-2026-08-27 --out build/platform/linux-server-x86_64-glibc/staging

evidence p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-evidence-generator -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --staging build/platform/linux-server-x86_64-glibc/staging --out build/evidence/linux-server-x86_64-glibc

manifest p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-manifest -- generate --plan build/plans/p0-linux-server-x86_64-glibc/build-plan.json --abi-descriptor modules/root-abi/generated/LGE-V1.2-2026-08-27/generated-contract-artifact.json --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --artifact-index build/platform/linux-server-x86_64-glibc/finalized/metadata/artifact-index.json --evidence build/evidence/linux-server-x86_64-glibc --out build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.json

sign-test p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-signer-tool --features test-provider -- sign --manifest build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.json --manifest-digest-file build/platform/linux-server-x86_64-glibc/finalized/metadata/core-engine-manifest.sha256 --trust-domain Test --provider test-file --key-file modules/smoke/fixtures/test-keys/p0-ed25519-private.key --out build/platform/linux-server-x86_64-glibc/finalized/metadata/signature-envelope.json

verify p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-smoke -- verify-package --package-root build/platform/linux-server-x86_64-glibc/finalized --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --trust-metadata modules/smoke/fixtures/test-keys/p0-ed25519-public.json --report build/reports/verify-package.json

load-smoke p="p0-linux": (assert-profile p)
    cargo run --locked -p lumio-core-smoke -- load --package-root build/platform/linux-server-x86_64-glibc/finalized --target-profile config/p0/linux-server-x86_64-glibc.target-profile.json --trust-metadata modules/smoke/fixtures/test-keys/p0-ed25519-public.json --report build/reports/load-smoke.json
