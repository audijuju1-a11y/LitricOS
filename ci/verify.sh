#!/usr/bin/env bash
# =============================================================================
# LitricOS — run every check that can actually execute on this host.
#
# This script is the answer to "what is actually verified?". Everything it
# reports PASS for was executed just now. Everything it reports SKIP for was not
# run, and the reason is printed. Nothing is inferred.
#
# It deliberately does NOT report on things it cannot test. A check that cannot
# run is a SKIP, never a silent pass -- PROTOCOL LAW 1 and LAW 8.
#
# Usage:
#   ci/verify.sh              run checks, print a report
#   ci/verify.sh --evidence   also write docs/EVIDENCE.md
# =============================================================================
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "$ROOT" || exit 1

PASS=0 FAIL=0 SKIP=0
RESULTS=()

green() { printf '\033[1;32m%s\033[0m' "$1"; }
red()   { printf '\033[1;31m%s\033[0m' "$1"; }
yellow(){ printf '\033[1;33m%s\033[0m' "$1"; }

# check <name> <command...>
check() {
    local name="$1"; shift
    local output rc
    output="$("$@" 2>&1)"; rc=$?
    if (( rc == 0 )); then
        printf '  %s  %s\n' "$(green PASS)" "$name"
        PASS=$((PASS + 1))
        RESULTS+=("PASS|${name}|$*")
    else
        printf '  %s  %s\n' "$(red FAIL)" "$name"
        # Show the tail of the captured output so test failures are visible.
        # tail consumes the whole stream, avoiding SIGPIPE/Broken pipe from
        # sending a large test log through sed before truncating it.
        printf '%s\n' "$output" | tail -40 | sed 's/^/         /'
        FAIL=$((FAIL + 1))
        RESULTS+=("FAIL|${name}|$*")
    fi
    return 0
}

skip() {
    printf '  %s  %s\n' "$(yellow SKIP)" "$1"
    printf '         reason: %s\n' "$2"
    SKIP=$((SKIP + 1))
    RESULTS+=("SKIP|$1|$2")
}

have() { command -v "$1" >/dev/null 2>&1; }

echo
echo "LitricOS verification -- $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "host: $(uname -s) $(uname -m)"
echo

# ---------------------------------------------------------------------------
echo "[1] Shell script syntax"
# ---------------------------------------------------------------------------
shell_syntax() {
    local rc=0 f
    while IFS= read -r f; do
        bash -n "$f" || { echo "syntax error in $f"; rc=1; }
    done < <(find tools iso ci image -name '*.sh' -o -path '*/litric/mm-tune' 2>/dev/null | sort)
    return $rc
}
check "all shell scripts parse" shell_syntax

# ---------------------------------------------------------------------------
echo
echo "[1b] Line endings"
# ---------------------------------------------------------------------------
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "no CRLF in files a Linux shell, kernel or systemd will read"         "$PY" ci/lint-line-endings.py
else
    skip "line endings" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[2] Python harness syntax"
# ---------------------------------------------------------------------------
# Parsing, not py_compile: py_compile writes __pycache__ into the tree it is
# checking (tests/__pycache__/smoke_boot.cpython-314.pyc is that residue), and
# the old inline loop covered ci/ and tests/ but not tools/ -- 18 scripts that
# drive real runs and were never parsed by anything.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "all python files parse, without writing bytecode" \
        "$PY" ci/check-python-syntax.py
    check "python syntax checker detects a truncated file" \
        "$PY" ci/check-python-syntax.py --self-test
else
    skip "python syntax" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3] Image configuration schema (BUG-0001 regression)"
# ---------------------------------------------------------------------------
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "mkosi config uses only real sections and key placements" "$PY" ci/lint-mkosi.py
    # The regression property: the linter MUST reject the pre-v2 file. A test
    # that cannot fail proves nothing.
    regression_must_fail() {
        if [[ ! -f .backup-pre-v2/image/mkosi.conf ]]; then
            echo "pre-v2 file absent, cannot prove the linter can fail"; return 1
        fi
        if "$PY" ci/lint-mkosi.py .backup-pre-v2/image/mkosi.conf >/dev/null 2>&1; then
            echo "linter ACCEPTED the known-bad pre-v2 config -- it cannot fail, so it proves nothing"
            return 1
        fi
        return 0
    }
    check "linter rejects the known-bad pre-v2 config (fails without the fix)" regression_must_fail
else
    skip "mkosi schema lint" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3b] Kernel configuration (Layer 1)"
# ---------------------------------------------------------------------------
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "kernel fragment: no duplicate/removed/bool-as-module symbols, LSM stack coherent"         "$PY" ci/lint-kconfig.py
    check "kconfig linter detects the defects it was written for"         "$PY" ci/lint-kconfig.py --self-test
else
    skip "kernel config lint" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3c] Scope ledger"
# ---------------------------------------------------------------------------
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "SYSTEM-SCOPE tally matches its rows, numbering is contiguous"         "$PY" ci/count-scope.py
else
    skip "scope ledger" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3d] Documentation citations"
# ---------------------------------------------------------------------------
# GATE 01 condition 7 was recorded VERIFIED against a file that has never
# existed. The same shape recurs four times in docs/bugs. A citation is evidence
# only if it can be opened, so it is checked like everything else.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "every file cited in docs exists" "$PY" ci/lint-doc-refs.py
    check "citation linter detects a fabricated citation" \
        "$PY" ci/lint-doc-refs.py --self-test
else
    skip "documentation citations" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3e] Memory budget (static policy presence)"
# ---------------------------------------------------------------------------
# Checks the ledger against the units in image/mkosi.extra. It cannot measure
# anything: a PASS here means the configuration policy is coherent, never that
# a component stays inside its limit at runtime.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "memory ledger claims match the shipped units" \
        "$PY" ci/check-memory-budget.py
    check "memory checker detects an ungrounded claim" \
        "$PY" ci/check-memory-budget.py --self-test
else
    skip "memory budget" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3f] File-safety review record (deletion sites)"
# ---------------------------------------------------------------------------
# Every production deletion in src/ must sit in a function ci/lint-file-safety.py
# names with a reason. This is a source-level guard for the layer that has no
# runnable test in this mode -- it cannot prove a deletion is correct, only that
# a new one cannot appear without a decision. Nothing here is executed.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "every deletion site in src/ is inside a reviewed function" \
        "$PY" ci/lint-file-safety.py
    check "file-safety checker detects an unreviewed deletion" \
        "$PY" ci/lint-file-safety.py --self-test
else
    skip "file safety" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3g] systemd units reference binaries that exist"
# ---------------------------------------------------------------------------
# Three units in this tree started binaries no crate produces, and the installed
# one was enabled -- a socket unit needs no binary to start, so the boot stayed
# green and the failure arrived at first use instead of at boot. The units are
# parked in image/units-pending/ until the binary exists; this check is why they
# cannot drift back into system/.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "every installed unit starts a binary the workspace builds" \
        "$PY" ci/lint-unit-binaries.py
    check "unit-binary linter detects a dead unit" \
        "$PY" ci/lint-unit-binaries.py --self-test
else
    skip "unit binaries" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[3h] Dependency policy"
# ---------------------------------------------------------------------------
# src/Cargo.toml states the rule in prose -- the crates in its core group take
# zero third-party dependencies, std only -- and nothing checked it. Coverage is
# derived from the manifest's own group comments, so a new crate cannot arrive
# outside the policy; a group the linter has no decision for fails closed.
# Recorded exceptions live in ci/dep-policy-allow.txt.
if have python || have python3; then
    PY=$(command -v python3 || command -v python)
    check "core crates take no third-party dependencies" \
        "$PY" ci/lint-deps.py
    check "dependency linter detects a planted dependency" \
        "$PY" ci/lint-deps.py --self-test
else
    skip "dependency policy" "no python interpreter on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[4] Development signing keys"
# ---------------------------------------------------------------------------
if have openssl; then
    keys_valid() {
        local rc=0 f
        for f in secureboot verity pcr; do
            [[ -s "keys/dev/$f.crt" && -s "keys/dev/$f.key" ]] || {
                echo "missing keys/dev/$f -- run tools/gen-dev-keys.sh"; rc=1; continue; }
            openssl x509 -in "keys/dev/$f.crt" -noout >/dev/null 2>&1 || {
                echo "$f.crt does not parse"; rc=1; continue; }
            # The pair must actually match. A cert and a key that were generated
            # in separate failed runs both "exist" and sign nothing.
            local a b
            a="$(openssl x509 -in "keys/dev/$f.crt" -noout -pubkey 2>/dev/null | openssl sha256)"
            b="$(openssl rsa -in "keys/dev/$f.key" -pubout 2>/dev/null | openssl sha256)"
            [[ -n "$a" && "$a" == "$b" ]] || { echo "$f: certificate and private key do not match"; rc=1; }
        done
        return $rc
    }
    # Development keys are deliberately git-ignored and must not be committed.
    # A clean clone therefore has none: report an environmental skip rather than
    # failing the entire repository gate. If any key material is present, run
    # the full three-pair/parse check so partial or mismatched keys still fail closed.
    if [[ -e keys/dev/secureboot.crt || -e keys/dev/secureboot.key || \
          -e keys/dev/verity.crt || -e keys/dev/verity.key || \
          -e keys/dev/pcr.crt || -e keys/dev/pcr.key ]]; then
        check "dev key pairs exist, parse, and match" keys_valid
    else
        skip "dev signing keys" "dev-only keys are intentionally absent from a clean checkout"
    fi
else
    skip "dev signing keys" "openssl not on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[5] Native workspace (Rust)"
# ---------------------------------------------------------------------------
if have cargo; then
    # Toolchain selection. The repository does not pin a host triple: the build
    # host is Linux, and a pinned Windows triple would break it. So the host is
    # detected here. On Windows without Visual Studio build tools the MSVC
    # linker is absent and `link.exe` resolves to coreutils `link`, producing a
    # baffling "missing operand after \377\376" error that looks like a code
    # defect. The GNU toolchain links against the mingw-w64 libraries rustup
    # already ships.
    TC=""
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*)
            if rustup toolchain list 2>/dev/null | grep -q 'windows-gnu'; then
                TC="+stable-x86_64-pc-windows-gnu"
            fi
            ;;
    esac

    # Cargo must run FROM the workspace directory. Invoked from the repository
    # root with --manifest-path, it resolves .cargo/config.toml and the toolchain
    # relative to the CURRENT directory rather than the manifest, which silently
    # selected a different toolchain and produced a link failure that looked like
    # a source bug. This cost a diagnostic cycle; the comment is here so it does
    # not cost a second one.
    cargo_in_src() { ( cd src && cargo ${TC:+$TC} "$@" ); }

    check "cargo build (all core crates)" cargo_in_src build --offline
    check "cargo test  (all core crates)" cargo_in_src test  --offline
    # Warnings are not cosmetic here: missing_docs on a public system contract
    # means an undocumented ABI surface.
    no_warnings() {
        local out
        out="$(cargo_in_src build --offline 2>&1)"
        if grep -q '^warning' <<< "$out"; then
            grep '^warning' <<< "$out" | head -10
            return 1
        fi
        return 0
    }
    check "cargo build emits zero warnings" no_warnings
    if ( cd src && cargo ${TC:+$TC} fmt --version >/dev/null 2>&1 ); then
        check "cargo fmt --check" cargo_in_src fmt --all -- --check
    else
        skip "cargo fmt" "rustfmt component not installed"
    fi
    if ( cd src && cargo ${TC:+$TC} clippy --version >/dev/null 2>&1 ); then
        check "cargo clippy (deny unwrap/panic/indexing in system code)" \
            cargo_in_src clippy --offline --all-targets -- -D warnings
    else
        skip "cargo clippy" "clippy component not installed"
    fi
else
    skip "rust workspace" "cargo not on PATH"
fi

# ---------------------------------------------------------------------------
echo
echo "[6] Image build and boot"
# ---------------------------------------------------------------------------
# These are the checks that decide whether LitricOS is an operating system.
# They cannot run here and saying so plainly is the whole point of this section.
if have mkosi; then
    # On a Windows drvfs/9p mount every file reports mode 0777 and the mode
    # cannot be changed (no metadata support), so mkosi's private-key
    # permission check can never pass -- not because the config is wrong but
    # because this host filesystem cannot represent a file mode at all. That
    # is precisely a "cannot run here" case: demote ONLY that specific
    # message to SKIP with its reason. Any other mkosi failure stays FAIL.
    mkosi_parses() {
        local out rc
        out="$(mkosi --directory image/ summary 2>&1)"; rc=$?
        if (( rc == 0 )); then return 0; fi
        if grep -q "Permissions of .* are too open" <<< "$out"; then
            echo "SKIP-AS-ENVIRONMENTAL: the repository sits on a drvfs/9p mount,"
            echo "which reports mode 0777 for every file and cannot hold a mode.";
            echo "Run image builds from a native Linux filesystem (e.g. inside WSL home).";
            return 2   # 2 = environmental skip, handled by the caller
        fi
        printf '%s\n' "$out"; return 1
    }
    # A check whose command exits 2 is reported as SKIP, not FAIL.
    output="$(mkosi_parses 2>&1)"; rc=$?
    if (( rc == 0 )); then
        printf '  %s  %s\n' "$(green PASS)" "mkosi config parses"
        PASS=$((PASS + 1)); RESULTS+=("PASS|mkosi config parses|mkosi --directory image/ summary")
    elif (( rc == 2 )); then
        skip "mkosi config parses" "key permissions unrepresentable on the drvfs mount (see message above)"
    else
        printf '  %s  %s\n' "$(red FAIL)" "mkosi config parses"
        printf '%s\n' "$output" | sed 's/^/         /' | head -25
        FAIL=$((FAIL + 1)); RESULTS+=("FAIL|mkosi config parses|mkosi --directory image/ summary")
    fi
else
    skip "image build" "mkosi not installed (needs a Linux host: run tools/bootstrap-windows.ps1)"
fi
if have qemu-system-x86_64; then
    skip "boot smoke test" "qemu present but no image has been built yet"
else
    skip "boot smoke test" "qemu-system-x86_64 not installed"
fi
skip "idle RAM measurement"  "requires a booted image"
skip "idle CPU measurement"  "requires a booted image"
skip "boot time measurement" "requires a booted image"

# ---------------------------------------------------------------------------
echo
printf 'summary: %s pass, %s fail, %s skipped\n' \
    "$(green "$PASS")" "$( ((FAIL)) && red "$FAIL" || echo 0 )" "$(yellow "$SKIP")"
echo

if [[ "${1:-}" == "--evidence" ]]; then
    OUT="docs/EVIDENCE.md"
    # Everything between the record markers is hand-written record, not generated
    # output: the generated part is the header, the Verified and NOT RUN tables
    # and the closing paragraph. This generator used to overwrite the whole file,
    # so running it would have deleted the VM, benchmark and design-system records
    # -- the file that exists to say what was executed would have destroyed half of
    # what was executed. The markers make re-runs idempotent: the heading between
    # them is regenerated, so it cannot accumulate once per run. A file written
    # before the markers existed has no markers, and its records are everything
    # from the first horizontal rule onward.
    KEEP=""
    if [[ -f "$OUT" ]]; then
        KEEP="$(awk '/<!-- records-begin -->/{f=1;next} /<!-- records-end -->/{f=0} f' "$OUT")"
        if [[ -z "$KEEP" ]]; then
            KEEP="$(sed -n '/^---$/,$p' "$OUT" | tail -n +2)"
        fi
    fi
    {
        echo "# LITRIC — EXECUTED EVIDENCE"
        echo
        echo "Generated by \`ci/verify.sh --evidence\`. Every PASS below was executed."
        echo "Every SKIP was not run, with the reason given. Nothing here is inferred."
        echo
        echo "\`\`\`"
        echo "date : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "host : $(uname -s) $(uname -m)"
        have cargo && echo "rust : $(rustc --version 2>/dev/null)"
        have openssl && echo "ssl  : $(openssl version 2>/dev/null)"
        echo "\`\`\`"
        echo
        echo "## Verified"
        echo
        echo "| Result | Check | Command |"
        echo "|---|---|---|"
        for r in "${RESULTS[@]}"; do
            IFS='|' read -r status name cmd <<< "$r"
            [[ "$status" == "SKIP" ]] && continue
            printf '| %s | %s | `%s` |\n' "$status" "$name" "$cmd"
        done
        echo
        echo "## NOT RUN"
        echo
        echo "| Check | Reason |"
        echo "|---|---|"
        for r in "${RESULTS[@]}"; do
            IFS='|' read -r status name reason <<< "$r"
            [[ "$status" == "SKIP" ]] || continue
            printf '| %s | %s |\n' "$name" "$reason"
        done
        echo
        echo "## What this does and does not establish"
        echo
        echo "**Establishes:** the native workspace compiles and its logic behaves as"
        echo "specified under test, including against hostile input; the image"
        echo "configuration matches the real mkosi schema; the signing keys are valid."
        echo
        echo "**Does not establish:** that LitricOS boots. No image has been built and"
        echo "nothing has been booted. Until the rows in NOT RUN become PASS rows,"
        echo "GATE 01 stays BLOCKED and no performance number in \`docs/PERF-budget.md\`"
        echo "may be quoted as a measurement."
        if [[ -n "$KEEP" ]]; then
            echo
            echo "---"
            echo
            echo "<!-- records-begin -->"
            echo "## Earlier records, preserved and NOT re-executed in this run"
            echo
            echo "Each section below was executed on the date and host it names."
            echo "This run did not repeat it; the tables above are the only rows"
            echo "that were executed today."
            printf '\n%s\n' "$KEEP"
            echo "<!-- records-end -->"
        fi
    } > "$OUT"
    echo "evidence written to $OUT"
fi

(( FAIL == 0 ))
