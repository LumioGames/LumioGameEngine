#!/usr/bin/env bash
# 测试发现失效守护（R-00266）——采集层。
#
# 为什么需要它：`just nextest` 用 `--no-tests=pass`（B-00001）——那条口径解决的是
# 「仓库合法状态下可能一个测试都没有」，代价是**发现失效与真的没有测试长得一样**。
# 实测：给某个 package 注入 `autotests = false`，30 个测试凭空消失，
# `cargo nextest run --no-tests=pass` 仍然 exit 0 并报「80 tests run: 80 passed」。
#
# 分工：本文件只负责**采集**两份输入，判据在 tools/verify-test-discovery.py。
# 判据是输入的纯函数，因此可以拿构造输入直接驱动——反例探针不必在发货路径上开测试后门。
#
# 两份输入是**互相独立的来源**：
#   cargo metadata  → workspace 的权威 package 集合（据此判「清单登记了不存在的包」）
#   cargo nextest   → 当下发现到的测试
# 期望集合则来自受版本控制、人工评审的 tools/test-discovery.manifest。判据不锚在被守护
# 对象自己身上：拿当下发现结果生成期望，恒等式证明不了任何东西。
#
# 用法：bash tools/verify-test-discovery.sh
# 退出码：0 通过；1 判据失败；2 前置/采集失败。

set -euo pipefail

cd "$(dirname "$0")/.."

MANIFEST="tools/test-discovery.manifest"

if [ ! -f "$MANIFEST" ]; then
    echo "verify-test-discovery.sh: FAIL: 找不到清单 $MANIFEST" >&2
    exit 2
fi

for tool in python3 cargo; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "verify-test-discovery.sh: FAIL: 需要 $tool（宿主基础设施）" >&2
        exit 2
    fi
done

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

if ! cargo metadata --no-deps --format-version 1 --offline \
        >"$WORKDIR/workspace.json" 2>"$WORKDIR/metadata.err"; then
    echo "verify-test-discovery.sh: FAIL: cargo metadata 执行失败" >&2
    tail -20 "$WORKDIR/metadata.err" >&2 || true
    exit 2
fi

# 采集失败必须是门禁失败，不能退化成「发现到 0 个测试」——那两件事的处置不同。
if ! cargo nextest list --workspace --config-file nextest.toml --message-format json \
        >"$WORKDIR/discovery.json" 2>"$WORKDIR/list.err"; then
    echo "verify-test-discovery.sh: FAIL: cargo nextest list 执行失败（发现失败 ≠ 没有测试）" >&2
    tail -20 "$WORKDIR/list.err" >&2 || true
    exit 2
fi

exec python3 tools/verify-test-discovery.py \
    "$WORKDIR/workspace.json" "$WORKDIR/discovery.json" "$MANIFEST"
