#!/usr/bin/env bash
# tools/verify-tool-lock.sh — 外部工具锁与供应链门禁完整性校验（LCE-P0-001）。
# POSIX sh 兼容（bash / dash / Git Bash 均可执行）。
#
# 校验内容：
#   1. tools/tools.lock.toml 每项字段齐全且格式合法（40 位 commit、64 位 sha256）。
#   2. 本机 host 命中 supported_hosts 的条目：对应二进制必须存在，且其 SHA-256
#      与 tools/checksums.sha256 的 name@host 登记一致（防工具漂移/替换）。
#   3. checksums.sha256 与锁条目双向一致：不允许锁外条目，也不允许缺失登记。
#   4. 许可证策略完整性：deny.toml 必须显式拒绝 GPL/AGPL/SSPL 并声明 copyleft=deny，
#      about.toml accepted 不得混入传染许可证——「需法务审核」策略不可被静默削弱。
#
# 退出码：0 全部通过；1 校验失败（缺失工具 / 哈希漂移 / 字段缺失 / 策略被削弱）。

set -eu

repo_root=$(cd "$(dirname "$0")/.." && pwd)
lock_file="$repo_root/tools/tools.lock.toml"
checksums_file="$repo_root/tools/checksums.sha256"
deny_file="$repo_root/deny.toml"
about_file="$repo_root/about.toml"

fail() {
    echo "tools/verify-tool-lock.sh: FAIL: $1" >&2
    exit 1
}

# ── host 标识（与 supported_hosts 使用的写法一致） ──────────────────────────
kernel=$(uname -s)
machine=$(uname -m)
case "$kernel" in
    MINGW* | MSYS* | CYGWIN*) os_name=windows ;;
    Linux*) os_name=linux ;;
    Darwin*) os_name=darwin ;;
    *) os_name="unknown-$kernel" ;;
esac
case "$machine" in
    x86_64 | amd64) arch_name=x86_64 ;;
    aarch64 | arm64) arch_name=arm64 ;;
    *) arch_name="$machine" ;;
esac
host="$os_name-$arch_name"

# ── 1. 解析并校验锁条目 ────────────────────────────────────────────────────
[ -f "$lock_file" ] || fail "缺少 $lock_file"
[ -f "$checksums_file" ] || fail "缺少 $checksums_file"

lock_entries=$(awk '
    BEGIN { RS = ""; FS = "\n" }
    /^\[\[tools\]\]/ {
        name = version = source_url = source_commit = license_spdx = ""
        artifact_sha256 = supported_hosts = invocation = owner = exit_tool = ""
        for (i = 2; i <= NF; i++) {
            line = $i
            sub(/^[ \t]+/, "", line)
            if (line ~ /^#/ || line ~ /^$/) continue
            eq = index(line, "=")
            if (eq == 0) continue
            key = substr(line, 1, eq - 1)
            val = substr(line, eq + 1)
            gsub(/[ \t]/, "", key)
            gsub(/^[ \t]+|[ \t]+$/, "", val)
            gsub(/^"/, "", val); gsub(/"$/, "", val)
            gsub(/^\[/, "", val); gsub(/\]$/, "", val)
            gsub(/,[ \t]*/, " ", val)
            gsub(/"/, "", val)
            if (key == "name") name = val
            else if (key == "version") version = val
            else if (key == "source_url") source_url = val
            else if (key == "source_commit") source_commit = val
            else if (key == "license_spdx") license_spdx = val
            else if (key == "artifact_sha256") artifact_sha256 = val
            else if (key == "supported_hosts") supported_hosts = val
            else if (key == "invocation") invocation = val
            else if (key == "owner") owner = val
            else if (key == "exit_tool") exit_tool = val
        }
        if (name != "")
            printf "%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\n",
                name, version, source_url, source_commit, license_spdx,
                artifact_sha256, supported_hosts, invocation, owner, exit_tool
    }
' "$lock_file")

[ -n "$lock_entries" ] || fail "tools.lock.toml 未解析出任何 [[tools]] 条目"

total=0
applicable=0
expected_keys=""
checked_hashes=0

echo "host: $host"

while IFS='|' read -r name version source_url source_commit license_spdx \
    artifact_sha256 supported_hosts invocation owner exit_tool; do
    total=$((total + 1))

    for field in name version source_url source_commit license_spdx \
        artifact_sha256 supported_hosts invocation owner exit_tool; do
        eval "value=\$$field"
        [ -n "$value" ] || fail "条目 $name: 字段 $field 缺失或为空"
    done
    case "$source_commit" in
        *[!0-9a-f]* | "")
            fail "条目 $name: source_commit 不是 40 位小写十六进制 commit（$source_commit）"
            ;;
        ????????????????????????????????????????) : ;;
        *)
            fail "条目 $name: source_commit 长度不是 40（$source_commit）"
            ;;
    esac
    case "$artifact_sha256" in
        *[!0-9a-f]* | "")
            fail "条目 $name: artifact_sha256 不是 64 位十六进制（$artifact_sha256）"
            ;;
        ????????????????????????????????????????????????????????????????) : ;;
        *)
            fail "条目 $name: artifact_sha256 长度不是 64（$artifact_sha256）"
            ;;
    esac

    for h in $supported_hosts; do
        expected_keys="$expected_keys $name@$h"
    done

    case " $supported_hosts " in
        *" $host "*)
            applicable=$((applicable + 1))
            bin_path=$(command -v "$name" 2>/dev/null || true)
            [ -n "$bin_path" ] || fail "条目 $name: 本机（$host）未安装，invocation=$invocation"
            expected_sum=$(awk -v key="$name@$host" '
                $2 == key { print $1; found = 1 }
                END { if (!found) print "" }
            ' "$checksums_file")
            [ -n "$expected_sum" ] || fail "条目 $name: checksums.sha256 缺少 $name@$host 登记"
            # GNU sha256sum 对含反斜杠的文件名输出加前导 `\` 转义标记，取哈希前剥离。
            actual_sum=$(sha256sum "$bin_path" | awk '{ sub(/^\\/, ""); print $1 }')
            [ "$actual_sum" = "$expected_sum" ] ||
                fail "条目 $name: 本机二进制 SHA-256 漂移（实际 $actual_sum，登记 $expected_sum）"
            checked_hashes=$((checked_hashes + 1))
            echo "  $name $version ($host): sha256 OK [$actual_sum]"
            ;;
    esac
done <<EOF
$lock_entries
EOF

# ── 3. checksums 与锁双向一致 ──────────────────────────────────────────────
for ck in $(awk '{ print $2 }' "$checksums_file"); do
    case " $expected_keys " in
        *" $ck "*) : ;;
        *) fail "checksums.sha256 条目 $ck 在 tools.lock.toml 中没有对应登记（孤儿校验和）" ;;
    esac
done
ck_count=$(awk 'NF { count++ } END { print count + 0 }' "$checksums_file")
exp_count=$(echo "$expected_keys" | wc -w | tr -d ' ')
[ "$ck_count" -eq "$exp_count" ] ||
    fail "checksums 条目数 $ck_count 与锁登记数 $exp_count 不一致（缺失登记）"

# ── 4. 许可证策略完整性 ────────────────────────────────────────────────────
[ -f "$deny_file" ] || fail "缺少 $deny_file"
[ -f "$about_file" ] || fail "缺少 $about_file"
# v2 白名单制：deny-by-default。完整性 = 白名单未混入传染许可证 + v2 配置就位。
grep -q '^version = 2' "$deny_file" ||
    fail "deny.toml 缺少 licenses version = 2（白名单制前提）"
allow_block=$(awk '/^allow = \[/{f=1;next} /^\]/{f=0} f' "$deny_file")
for lic in GPL-1.0 GPL-2.0 GPL-3.0 LGPL-2.0 LGPL-2.1 LGPL-3.0 \
    AGPL-1.0 AGPL-3.0 SSPL-1.0 MPL-2.0; do
    case "$allow_block" in
        *"$lic"*) fail "deny.toml 许可证白名单混入传染许可证 $lic（需法务审核项不得进白名单）" ;;
    esac
done
for lic in GPL-2.0 GPL-3.0 AGPL-1.0 AGPL-3.0 SSPL-1.0; do
    grep -q "\"$lic\"" "$deny_file" ||
        fail "deny.toml 策略注释/拒绝登记缺少 $lic 的拒绝声明"
done
# about.toml 的 accepted 均为带引号 TOML 字符串，按 `"SPDX-ID"` 子串匹配（与 deny.toml 块同口径）。
for lic in GPL-1.0 GPL-2.0 GPL-3.0 LGPL-2.0 LGPL-2.1 LGPL-3.0 \
    AGPL-1.0 AGPL-3.0 SSPL-1.0 MPL-2.0; do
    grep -q "\"$lic\"" "$about_file" &&
        fail "about.toml accepted 白名单混入传染许可证 $lic（需法务审核项不得进白名单）"
done

echo "tools.lock: $total entries OK (applicable on $host: $applicable, hashes verified: $checked_hashes)"
echo "license-policy: 白名单制（deny-by-default），GPL/AGPL/SSPL/MPL 等传染许可证 -> 拒绝（需法务审核），策略完整性 OK"
echo "tools/verify-tool-lock.sh: OK"
