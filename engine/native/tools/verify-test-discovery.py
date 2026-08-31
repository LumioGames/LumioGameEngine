#!/usr/bin/env python3
"""测试发现失效守护的判据（R-00266）。

本文件是**输入的纯函数**：读两份 JSON 加一份清单，判定并给退出码。不执行 cargo、
不读环境状态——采集交给 tools/verify-test-discovery.sh，判据留在这里，这样判据本身
可以拿构造输入直接驱动（反例探针不需要在发货路径上开测试后门）。

用法：verify-test-discovery.py <cargo-metadata.json> <nextest-list.json> [清单路径]

退出码：0 通过；1 判据失败；2 输入不可用。
"""

import json
import sys

DEFAULT_MANIFEST = "tools/test-discovery.manifest"


def load_manifest(path):
    """读清单；返回 package 名集合。重复登记直接判失败——清单要能被逐行核对。"""
    entries = []
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.split("#", 1)[0].strip()
            if line:
                entries.append(line)
    duplicates = sorted({p for p in entries if entries.count(p) > 1})
    if duplicates:
        raise ValueError(f"清单有重复条目：{', '.join(duplicates)}")
    return set(entries)


def discovered_counts(discovery):
    """按 package 汇总 nextest 发现到的 testcase 数。"""
    counts = {}
    for suite in discovery.get("rust-suites", {}).values():
        name = suite.get("package-name")
        if name is None:
            continue
        counts[name] = counts.get(name, 0) + len(suite.get("testcases", {}))
    return counts


def evaluate(expected, members, counts):
    """返回失败原因列表；空列表即通过。"""
    failures = []

    if not members:
        failures.append("cargo metadata 没有报告任何 workspace package——采集本身有问题")
        return failures

    if sum(counts.values()) == 0:
        failures.append(
            "全仓一个测试都没有发现到——这正是 --no-tests=pass 会静默放行的状态"
        )

    missing = sorted(expected - members)
    if missing:
        failures.append(
            "清单登记了 workspace 中不存在的 package（改名或删除后未同步清单）："
            + ", ".join(missing)
        )

    broken = sorted(p for p in expected if p in members and counts.get(p, 0) == 0)
    if broken:
        failures.append(
            "下列 package 应含测试但发现到 0 个（测试发现失效，或测试被移除而清单未同步）："
            + ", ".join(broken)
        )

    unregistered = sorted(p for p, n in counts.items() if n > 0 and p not in expected)
    if unregistered:
        failures.append(
            "下列 package 发现到测试但未登记，处于无守护状态，请加入清单："
            + ", ".join(unregistered)
        )

    return failures


def main(argv):
    if not 3 <= len(argv) <= 4:
        print(__doc__, file=sys.stderr)
        return 2

    manifest_path = argv[3] if len(argv) == 4 else DEFAULT_MANIFEST

    try:
        expected = load_manifest(manifest_path)
    except OSError as error:
        print(f"verify-test-discovery: FAIL: 读不到清单 {manifest_path}：{error}", file=sys.stderr)
        return 2
    except ValueError as error:
        print(f"verify-test-discovery: FAIL: {error}", file=sys.stderr)
        return 1

    try:
        with open(argv[1], encoding="utf-8") as handle:
            workspace = json.load(handle)
        with open(argv[2], encoding="utf-8") as handle:
            discovery = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        print(f"verify-test-discovery: FAIL: 输入不可用：{error}", file=sys.stderr)
        return 2

    members = {p["name"] for p in workspace.get("packages", [])}
    counts = discovered_counts(discovery)

    for package in sorted(counts):
        if counts[package]:
            mark = "OK " if package in expected else "!! "
            print(f"  {mark}{counts[package]:5d}  {package}")

    failures = evaluate(expected, members, counts)
    if failures:
        print("", file=sys.stderr)
        for failure in failures:
            print(f"verify-test-discovery: FAIL: {failure}", file=sys.stderr)
        print(
            f"\n清单：{manifest_path}（判据是集合相等，维护规则见文件头注释）",
            file=sys.stderr,
        )
        return 1

    total = sum(counts.values())
    print(
        f"verify-test-discovery: OK（{len(expected)} 个登记 package 全部发现到测试，"
        f"全仓合计 {total} 个；无未登记的有测试 package）"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
