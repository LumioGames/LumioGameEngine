#!/usr/bin/env bash
# tools/verify-tool-lock.sh — 外部工具锁与供应链门禁完整性校验（LCE-P0-001）。
# POSIX sh 兼容（bash / dash / Git Bash 均可执行）。
#
# 校验内容：
#   1. tools/tools.lock.toml 每项字段齐全且格式合法（40 位 commit、64 位 sha256）。
#   2. 本机 host 命中 supported_hosts 的条目：对应二进制必须存在，且其 SHA-256
#      与 tools/checksums.sha256 的 name@host 登记一致（防工具漂移/替换）。
#      host key 取 rustc 编译期目标三元组，不用 uname -m（B-00003：混合架构宿主上
#      uname -m 随调用链中二进制人格漂移，同机两值；锁内工具经 cargo install 产出，
#      其架构即钉定工具链 host 三元组，以 rustc 为准才是机器的稳定函数）。
#   3. checksums.sha256 与锁条目双向一致：不允许锁外条目，也不允许缺失登记。
#   4. 许可证策略完整性：deny.toml 必须显式拒绝 GPL/AGPL/SSPL 并声明 copyleft=deny，
#      about.toml accepted 不得混入传染许可证——「需法务审核」策略不可被静默削弱。
#
# 退出码：0 全部通过；1 校验失败（缺失工具 / 哈希漂移 / 字段缺失 / 策略被削弱）。
# 本机不在任何 supported_hosts（applicable=0）时二进制完整性是空跑：仍 exit 0
# （P0 目标平台与 CI 为 Linux，不因未登记宿主阻断门禁），但输出显式 WARNING 并将
# 末行 OK 与「已校验」区分，防止误读为工具链已被校验（B-00002 选项二）。
#
# 环境变量（均为 R-00265 引入，默认全不设 = 原有行为）：
#   LCE_REQUIRE_BINARY_VERIFICATION=1
#       要求本次运行**真的校验过二进制**，空跑即失败。CI 设此开关——否则「二进制校验
#       已执行」只能靠人读日志判断，而空跑与已校验都是 exit 0。
#   LCE_EXPECT_VERIFIED_HASHES=<n>
#       期望校验条数。上一条只挡「全空跑」，挡不住有人把若干条目的 supported_hosts
#       改窄后「校验了 1 条也算绿」。
#   LCE_BUILD_ENVIRONMENT=p0-build
#       额外把 `<host>-p0-build` 计入 applicable。该 host key 登记的是 rustc/cargo/cc
#       这类**摘要只在钉定构建环境内成立**的工具：开发机上 `command -v rustc` 命中
#       rustup shim，摘要必然不同，默认不参与判定，否则 Linux 开发机的 `just check`
#       会永远红在一条与其无关的漂移上。

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
# 从 rustc -vV 的 host 三元组推导，不用 uname -m（见头部注释 B-00003 条）。
rustc_host=$(rustc -vV 2>/dev/null | sed -n 's/^host: //p')
[ -n "$rustc_host" ] ||
    fail "无法从 rustc -vV 解析 host 三元组（本仓工具锁按钉定 Rust 工具链的 host 判定，需 rustc 在场）"
case "$rustc_host" in
    x86_64-pc-windows-*) host=windows-x86_64 ;;
    aarch64-pc-windows-*) host=windows-arm64 ;;
    x86_64-unknown-linux-*) host=linux-x86_64 ;;
    aarch64-unknown-linux-*) host=linux-arm64 ;;
    x86_64-apple-darwin) host=darwin-x86_64 ;;
    aarch64-apple-darwin) host=darwin-arm64 ;;
    *) fail "未登记的 rustc host 三元组：$rustc_host（先在本脚本映射表补对应 host key）" ;;
esac

# 本次判定为 applicable 的 host key 集合。
# 默认只有本机 host。`<host>-p0-build` 是**钉定的 P0 构建环境**（CI 的 ubuntu-22.04
# runner + 钉定 apt/rustup 版本），它登记的是 rustc/cargo/cc 这类摘要只在该环境成立的
# 工具：开发机上 `command -v rustc` 命中的是 rustup shim，摘要必然不同，把它算作本机
# 漂移会让 Linux 开发机的 `just check` 永远红。要在该环境内校验，设
# LCE_BUILD_ENVIRONMENT=p0-build（CI 已设）。
applicable_hosts="$host"
if [ "${LCE_BUILD_ENVIRONMENT:-}" = "p0-build" ]; then
    applicable_hosts="$applicable_hosts $host-p0-build"
fi

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

echo "host: $host (rustc host: $rustc_host)"

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

    for h in $applicable_hosts; do
        case " $supported_hosts " in
            *" $h "*)
                applicable=$((applicable + 1))
                bin_path=$(command -v "$name" 2>/dev/null || true)
                [ -n "$bin_path" ] ||
                    fail "条目 $name: 本机（$h）未安装，invocation=$invocation；按锁内 source_url 取分发制品安装：$source_url"
                expected_sum=$(awk -v key="$name@$h" '
                    $2 == key { print $1; found = 1 }
                    END { if (!found) print "" }
                ' "$checksums_file")
                [ -n "$expected_sum" ] || fail "条目 $name: checksums.sha256 缺少 $name@$h 登记"
                # GNU sha256sum 对含反斜杠的文件名输出加前导 `\` 转义标记，取哈希前剥离。
                actual_sum=$(sha256sum "$bin_path" | awk '{ sub(/^\\/, ""); print $1 }')
                [ "$actual_sum" = "$expected_sum" ] ||
                    fail "条目 $name: 本机二进制 SHA-256 漂移（$bin_path 实际 $actual_sum，登记 $expected_sum）；本仓只认锁内分发制品那一份，按 source_url 重装：$source_url"
                checked_hashes=$((checked_hashes + 1))
                echo "  $name $version ($h): sha256 OK [$actual_sum]"
                ;;
        esac
    done
done <<EOF
$lock_entries
EOF

# ── 3. checksums 与锁双向一致 ──────────────────────────────────────────────
# 逐键双向比对，不比行数：行数相等掩盖不了「缺一个键 + 重复另一个键」——
# 这种组合会让某个 host 的登记静默消失而门禁仍绿。
checksum_keys=$(awk 'NF { print $2 }' "$checksums_file" | tr '\n' ' ')

duplicate=$(printf '%s\n' $checksum_keys | sort | uniq -d)
[ -z "$duplicate" ] ||
    fail "checksums.sha256 有重复键：$(printf '%s' "$duplicate" | tr '\n' ' ')"

for key in $expected_keys; do
    case " $checksum_keys " in
        *" $key "*) : ;;
        *) fail "checksums.sha256 缺少锁内登记 $key" ;;
    esac
done
for key in $checksum_keys; do
    case " $expected_keys " in
        *" $key "*) : ;;
        *) fail "checksums.sha256 条目 $key 在 tools.lock.toml 中没有对应登记（孤儿校验和）" ;;
    esac
done

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

# 失败先于总结输出，避免日志读成「OK / OK / FAIL」。
if [ "${LCE_REQUIRE_BINARY_VERIFICATION:-0}" = "1" ]; then
    [ "$checked_hashes" -gt 0 ] ||
        fail "LCE_REQUIRE_BINARY_VERIFICATION=1 要求本次真实校验二进制，但 $host 上 hashes verified: 0（applicable: $applicable）——空跑不得计为通过"
    # 只挡「全空跑」挡不住「部分空跑」：有人从若干条目里删掉本 host，门禁仍绿、
    # 只是校验条数悄悄变少。调用方给出期望条数即可让缩水也变红。
    if [ -n "${LCE_EXPECT_VERIFIED_HASHES:-}" ]; then
        [ "$checked_hashes" -eq "$LCE_EXPECT_VERIFIED_HASHES" ] ||
            fail "期望校验 $LCE_EXPECT_VERIFIED_HASHES 个二进制，实际 $checked_hashes（登记被删或 supported_hosts 被改窄）"
    fi
fi

echo "tools.lock: $total entries OK (applicable hosts: $applicable_hosts, applicable entries: $applicable, hashes verified: $checked_hashes)"
echo "license-policy: 白名单制（deny-by-default），GPL/AGPL/SSPL/MPL 等传染许可证 -> 拒绝（需法务审核），策略完整性 OK"

if [ "$applicable" -eq 0 ]; then
    # 非绿灯信号（B-00002）：空跑不得与「已校验」共用同一句 OK。
    echo "tools/verify-tool-lock.sh: WARNING: $host 不在任何 supported_hosts，本机工具二进制完整性未校验（hashes verified: 0）；本次通过只覆盖锁结构、checksums 双向登记与许可证策略。要覆盖本机，按规格 §4 选型流程在 tools.lock.toml 与 checksums.sha256 登记 $host 制品。" >&2
    echo "tools/verify-tool-lock.sh: OK (binary integrity NOT verified on $host)"
else
    echo "tools/verify-tool-lock.sh: OK"
fi
