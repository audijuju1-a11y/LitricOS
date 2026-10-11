#!/bin/bash
# =============================================================================
# LitricOS — GATE 01 Verification Script
# Run inside the VM: bash /tmp/gate01-verify.sh
#
# TWO defects found in audit on 2026-10-09 (docs/bugs/BUG-0006), both of the
# same family: the script could not report what it claimed to report.
#
#   1. `((PASS++))` under `set -e`. A post-increment evaluates to the value
#      BEFORE the increment, so the first increment returned status 1 and set -e
#      killed the script. Executed against stub commands: the run stopped after
#      "PASS: systemd running" and exited 1, never reaching sections 2-7 or the
#      summary. Counters are now assigned (`PASS=$((PASS + 1))`), which always
#      returns 0.
#
#   2. Three probes could never pass, whatever the machine did:
#        - MGLRU read /sys/kernel/mm/lru_gen/enabled and expected the string
#          "y"; the file contains a bitmask. Real boot evidence records
#          "litric.kernel.mglru=0x0007" (docs/evidence/gate01-boot-clean.log),
#          so the check was guaranteed to FAIL on a machine where MGLRU works.
#        - Landlock was probed from /proc/self/attr/current, which reports the
#          *label of the calling process*, not the active LSM stack.
#        - PREEMPT was probed with `grep CONFIG_PREEMPT /proc/version`; that
#          file is the kernel banner and contains no CONFIG_* symbols at all.
#      All three are replaced with the probes this project already ships and
#      trusts in image/mkosi.extra/usr/lib/litric/selftest, the in-guest
#      reporter that found the last five boot defects.
#
# The lesson both defects share is recorded here rather than in a commit
# message: a check that cannot fail, and a check that cannot pass, are equally
# worthless, and neither announces itself.
# =============================================================================
set -euo pipefail

PASS=0
FAIL=0
WARN=0

check() {
    local name="$1" cmd="$2" expected="$3"
    local result
    result=$(eval "$cmd" 2>/dev/null || echo "ERROR")
    if echo "$result" | grep -q "$expected"; then
        echo "  ✅ PASS: $name"
        PASS=$((PASS + 1))
    else
        echo "  ❌ FAIL: $name (expected '$expected', got '$result')"
        FAIL=$((FAIL + 1))
    fi
}

warn_check() {
    local name="$1" cmd="$2" expected="$3"
    local result
    result=$(eval "$cmd" 2>/dev/null || echo "ERROR")
    if echo "$result" | grep -q "$expected"; then
        echo "  ✅ PASS: $name"
        PASS=$((PASS + 1))
    else
        echo "  ⚠️  WARN: $name (expected '$expected', got '$result')"
        WARN=$((WARN + 1))
    fi
}

# ---------------------------------------------------------------------------
# Probes. Same sources as litric-selftest, so the gate and the in-guest report
# cannot disagree about what the machine is doing.
# ---------------------------------------------------------------------------

# dm-verity: the integrity chain is live only if a verity target is mapped.
# `mount | grep verity` was tried and removed -- the mapper device is named
# /dev/mapper/root, so "verity" need not appear in mount output at all.
verity_state() {
    local vt
    vt=$(dmsetup table 2>/dev/null | grep -c verity || echo 0)
    if [[ "$vt" -gt 0 ]]; then echo "verity-active ($vt)"; else echo "verity-absent"; fi
}

# Every probe below reads through `cat` rather than a bash `-r` test, for two
# reasons: it needs no [ ] builtin behaviour that a minimal image might differ
# on, and it makes the probe exercisable against a stub `cat` instead of the
# real /proc and /sys of whatever machine runs the harness.
lsm_state() {
    cat /sys/kernel/security/lsm 2>/dev/null || echo "unavailable"
}

# MGLRU is a bitmask (0x0001 min_ttl, 0x0002 working_set, 0x0004 ...). The
# previous probe asked for "y", which the file never contains.
mglru_state() {
    local v
    v=$(cat /sys/kernel/mm/lru_gen/enabled 2>/dev/null || echo absent)
    v=${v:-absent}
    if [[ "$v" == "absent" ]]; then echo "absent"
    elif [[ "$v" == "0x0000" ]]; then echo "disabled"
    else echo "enabled ($v)"; fi
}

# /proc/version carries the build banner ("#1 SMP PREEMPT_DYNAMIC"), not
# CONFIG_* symbols. Matched with bash's own regex so the probe needs no grep
# flags that busybox grep may not implement.
preempt_state() {
    local v
    v=$(cat /proc/version 2>/dev/null || echo "")
    if [[ "$v" =~ PREEMPT[A-Za-z_]* ]]; then
        printf '%s\n' "${BASH_REMATCH[0]}"
    else
        echo "absent"
    fi
}

seccomp_state() {
    local a
    a=$(cat /proc/sys/kernel/seccomp/actions_avail 2>/dev/null || echo "")
    if [[ -n "$a" ]]; then echo "seccomp-actions"; else echo "no-seccomp-sysctl"; fi
}

root_state() {
    local fs ro
    fs=$(findmnt -n -o FSTYPE / 2>/dev/null || echo unknown)
    if findmnt -n -o OPTIONS / 2>/dev/null | grep -qE '(^|,)ro(,|$)'; then ro="read-only"; else ro="read-write"; fi
    echo "root-fstype=$fs $ro"
}

echo "============================================"
echo "  LitricOS GATE 01 Verification"
echo "============================================"
echo ""

# --- 1. Boot completed ---
echo "1. Boot Completion"
check "systemd running" "systemctl is-system-running" "running"
check "graphical.target reached" "systemctl is-active graphical.target" "active"
echo ""

# --- 2. Zero failed units ---
echo "2. Failed Units"
FAILED=$(systemctl list-units --state=failed --no-pager --no-legend 2>/dev/null | wc -l) || FAILED=0
if [ "$FAILED" -eq 0 ]; then
    echo "  ✅ PASS: Zero failed units"
    PASS=$((PASS + 1))
else
    echo "  ❌ FAIL: $FAILED failed units"
    systemctl list-units --state=failed --no-pager --no-legend 2>/dev/null || true
    FAIL=$((FAIL + 1))
fi
echo ""

# --- 3. Memory ---
echo "3. Memory"
MEMINFO=$(cat /proc/meminfo 2>/dev/null || echo "")
if [ -n "$MEMINFO" ]; then
    USED_KB=$(printf '%s\n' "$MEMINFO" | awk '/^MemTotal/ {total=$2} /^MemAvailable/ {avail=$2} END {print total-avail}')
else
    USED_KB=0
fi
USED_MB=$((USED_KB / 1024))
echo "  📊 Used memory: ${USED_MB}MB"
if [ "$USED_MB" -le 320 ]; then
    echo "  ✅ PASS: Idle RAM ≤320MB (${USED_MB}MB)"
    PASS=$((PASS + 1))
else
    echo "  ⚠️  WARN: Idle RAM >320MB (${USED_MB}MB)"
    WARN=$((WARN + 1))
fi
echo ""

# --- 4. Kernel features ---
echo "4. Kernel Features"
check "MGLRU enabled" "mglru_state" "enabled"
check "dm-verity active" "verity_state" "verity-active"
check "Landlock LSM active" "lsm_state" "landlock"
check "PREEMPT enabled" "preempt_state" "PREEMPT"
echo ""

# --- 5. Security ---
echo "5. Security"
check "seccomp filter support" "seccomp_state" "seccomp-actions"
check "ASLR enabled" "cat /proc/sys/kernel/randomize_va_space" "2"
check "KASLR active" "grep 'nokaslr' /proc/cmdline || echo kaslr_active" "kaslr_active"
echo ""

# --- 5b. Root filesystem integrity ---
# The read-only verity root is the design's central structural claim; observing
# it costs nothing and a read-write root is a finding, not a detail.
echo "5b. Root Filesystem"
echo "  📊 $(root_state)"
check "root is mounted read-only" \
    "findmnt -n -o OPTIONS / | grep -oE '(^|,)ro(,|\$)' | head -1" "ro"
echo ""

# --- 6. Core services ---
echo "6. Core Services"
check "litric-sysd running" "systemctl is-active litric-sysd" "active"
check "litric-shell running" "systemctl is-active litric-shell" "active"
check "journald running" "systemctl is-active systemd-journald" "active"
check "networkd running" "systemctl is-active systemd-networkd" "active"
check "resolved running" "systemctl is-active systemd-resolved" "active"
echo ""

# --- 7. Boot time ---
echo "7. Boot Time"
BOOT_SEC=$(systemd-analyze 2>/dev/null | grep -oP 'userspace\) = \K[0-9.]+' || echo "unknown")
GRAPHICAL_SEC=$(systemd-analyze 2>/dev/null | grep -oP 'graphical.target reached after \K[0-9.]+' || echo "unknown")
echo "  📊 Total boot: ${BOOT_SEC}s"
echo "  📊 Graphical target: ${GRAPHICAL_SEC}s in userspace"
echo ""

# --- Summary ---
echo "============================================"
echo "  GATE 01 Results: $PASS passed, $FAIL failed, $WARN warnings"
echo "============================================"

if [ "$FAIL" -eq 0 ]; then
    echo "  🎉 GATE 01: PASS"
    exit 0
else
    echo "  ❌ GATE 01: FAIL ($FAIL failures)"
    exit 1
fi
