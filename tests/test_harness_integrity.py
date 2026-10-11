#!/usr/bin/env python3
"""Regression tests for the verification harness itself.

The harness that decides whether LitricOS boots is code, it has failed before
(BUG-0002), and it failed again in three separate ways found on 2026-10-09
(BUG-0005, BUG-0006). Its defects are invisible by construction -- a boot check
that cannot fail, or cannot time out, still prints PASS -- so the only way to
keep them fixed is to test the harness against a guest whose behaviour is
chosen rather than observed.

Nothing here builds, boots or launches anything. QEMU is replaced by a stub
program that prints whichever serial transcript a test asks for; the GATE 01
verifier runs against stub systemctl/dmsetup/findmnt/cat on PATH. Every
assertion is about the harness's own logic.

Run:
    pytest tests/test_harness_integrity.py -v
"""

from __future__ import annotations

import ast
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

import pytest

TESTS_DIR = Path(__file__).parent.resolve()
PROJECT_ROOT = TESTS_DIR.parent
sys.path.insert(0, str(TESTS_DIR))

import smoke_boot as sb  # noqa: E402  (path inserted above)

GATE_SCRIPT = TESTS_DIR / "gate01-verify.sh"
DOC_LINTER = PROJECT_ROOT / "ci" / "lint-doc-refs.py"

# Serial transcripts. Each one is a real shape of output this project has
# produced or could produce, and each is chosen to make exactly one property
# of the reader observable.
STUB_PROGRAM = '''
import os, sys, time
scenario = sys.argv[1] if len(sys.argv) > 1 else "prompt"
lines = {
    "prompt": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "[    3.100000] systemd[1]: Reached target Local File Systems.",
        "litricos login:",
    ],
    "rootprompt": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "[    5.000000] systemd[1]: Started Login Service.",
        "root@litricos:~#",
    ],
    # The exact line that the pre-fix indicator set counted as a shell prompt:
    # it contains ">", "#"-free but "shell" -- two of the three loose
    # indicators. A boot that never reached userspace.
    "initrd_fail": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "==> ERROR: Failed to mount root, dropping to emergency shell",
        ":: running hook [verity]",
    ],
    # Isolates the "shell" indicator: no ">" and no "#" anywhere in the line.
    "shell_word": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "[    6.400000] systemd[1]: Entering emergency shell.",
    ],
    # A guest that boots into silence: the hang this deadline exists for.
    "silent": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "[    0.900000] systemd[1]: Startup finished in 900ms.",
    ],
    # Not one byte of output, ever: the strictest form of the same failure.
    "mute": [],
    # QEMU itself dies (bad image, firmware abort) without a prompt.
    "exit": [
        "[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC",
        "BdsDxe: failed to load Boot0001, no bootable option",
    ],
}
for line in lines[scenario]:
    print(line, flush=True)
if scenario == "exit":
    sys.exit(3)
time.sleep(600)
'''


@pytest.fixture
def guest(tmp_path, monkeypatch):
    """A QemuRunner wired to a stub QEMU, with all writes inside tmp_path."""
    stub = tmp_path / "stub_qemu.py"
    stub.write_text(STUB_PROGRAM, encoding="utf-8")

    code = tmp_path / "OVMF_CODE.fd"
    code.write_bytes(b"CODE" * 64)
    vars_template = tmp_path / "OVMF_VARS.fd"
    vars_template.write_bytes(b"VARS" * 64)

    monkeypatch.setattr(sb, "LOGS_DIR", tmp_path / "logs")
    monkeypatch.setattr(sb, "VARS_SCRATCH", tmp_path / "OVMF_VARS.smoke.fd")
    monkeypatch.setattr(sb, "QEMU_CMD", [sys.executable, str(stub)])
    monkeypatch.setattr(sb, "discover_firmware_code", lambda: code)
    monkeypatch.setattr(sb, "discover_firmware_vars", lambda: vars_template)

    def make(scenario: str, timeout: float = 5.0) -> sb.QemuRunner:
        runner = sb.QemuRunner(smp=1, mem="512M", timeout=timeout)
        runner.scenario = scenario
        original = runner._build_cmd

        def build():
            # The scenario goes immediately after the stub program itself (the
            # whole of QEMU_CMD), before QEMU's flags, so the stub does not
            # have to parse flags to find it. Inserting it at index 1 instead
            # would hand it to the interpreter as the script to run.
            cmd = original()
            n = len(sb.QEMU_CMD)
            return cmd[:n] + [scenario] + cmd[n:]

        runner._build_cmd = build
        return runner

    return make


def _run(runner):
    """Boot the stub guest and read until a verdict."""
    runner.start()
    try:
        lines, status, detail = runner.read_output()
    finally:
        runner.stop()
    return lines, status, detail


# ---------------------------------------------------------------------------
# 1. Success is detected only when success actually happened
# ---------------------------------------------------------------------------

def test_login_prompt_is_detected(guest):
    lines, status, detail = _run(guest("prompt"))
    assert status == "SHELL", f"a real login prompt was not detected: {detail}"


def test_root_ps1_is_detected(guest):
    _lines, status, detail = _run(guest("rootprompt"))
    assert status == "SHELL", f"a root prompt was not detected: {detail}"


def test_initrd_failure_is_not_a_shell(guest):
    """The pre-fix reader returned SHELL here, in 0.0 s (BUG-0005).

    The line contains ">" and "shell", two of the loose indicators that used to
    mean success, so an image that dropped to an emergency prompt in early
    userspace was reported as a booted system.
    """
    _lines, status, detail = _run(guest("initrd_fail"))
    assert status != "SHELL", f"a boot that never reached userspace passed: {detail}"
    assert status == "FAILURE", f"expected an explicit FAILURE verdict, got {status}"
    assert "emergency" in detail.lower(), f"the reason should be named: {detail}"


def test_emergency_shell_word_alone_is_not_success(guest):
    """Isolates the old "shell" indicator, with no ">" or "#" in the line."""
    _lines, status, _detail = _run(guest("shell_word"))
    assert status != "SHELL", "the word 'shell' must not be a success indicator"


# ---------------------------------------------------------------------------
# 2. The deadline is enforced, not advisory
# ---------------------------------------------------------------------------

def test_silent_guest_times_out_within_the_deadline(guest):
    """Pre-fix, a 4 s deadline returned at 25 s (the child's own lifetime)."""
    runner = guest("silent", timeout=2.0)
    runner.start()
    try:
        t0 = time.time()
        _lines, status, detail = runner.read_output()
        elapsed = time.time() - t0
        assert status == "TIMEOUT", f"silent guest reported {status}"
        assert elapsed < 8.0, f"deadline not enforced: returned after {elapsed:.1f}s"
        assert runner.process.poll() is None, \
            "the guest was still running; the timeout must not wait for exit"
    finally:
        runner.stop()


def test_mute_guest_times_out_within_the_deadline(guest):
    """A guest that never writes a single byte at all."""
    runner = guest("mute", timeout=2.0)
    runner.start()
    try:
        t0 = time.time()
        _lines, status, _detail = runner.read_output()
        elapsed = time.time() - t0
        assert status == "TIMEOUT"
        assert elapsed < 8.0, f"deadline not enforced: returned after {elapsed:.1f}s"
    finally:
        runner.stop()


def test_early_qemu_exit_is_reported_as_such(guest):
    _lines, status, detail = _run(guest("exit"))
    assert status == "TIMEOUT"
    assert "exited" in detail.lower(), f"the exit should be named: {detail}"


def test_timeout_detail_says_where_the_boot_stopped(guest):
    _lines, _status, detail = _run(guest("silent"))
    assert "stage" in detail.lower(), \
        f"a timeout must place the failure stage, got: {detail}"


# ---------------------------------------------------------------------------
# 3. The harness points at artifacts that exist
# ---------------------------------------------------------------------------

def test_image_candidates_match_the_pipeline_output():
    """mkosi writes build/out/litricos.raw (image/mkosi.conf ImageId=litricos).

    The pre-fix constant was build/litric.qcow2, from the retired
    tools/build-image.sh, so every case in this file skipped on a machine with
    a freshly built image -- a gate that reports nothing and looks like a
    missing prerequisite.
    """
    first = sb.IMAGE_CANDIDATES[0]
    assert first == sb.BUILD_DIR / "out" / "litricos.raw", first
    assert any(c.name == "litric.qcow2" for c in sb.IMAGE_CANDIDATES), \
        "the legacy name should still resolve, after the real one"


def test_env_override_selects_the_image(tmp_path, monkeypatch):
    fake = tmp_path / "custom.raw"
    fake.write_bytes(b"x")
    monkeypatch.setenv("LITRIC_SMOKE_IMAGE", str(fake))
    assert sb.discover_image() == fake


def test_missing_image_reports_the_path_it_expects(monkeypatch):
    monkeypatch.setenv("LITRIC_SMOKE_IMAGE", "")
    monkeypatch.delenv("LITRIC_SMOKE_IMAGE", raising=False)
    resolved = sb.discover_image()
    assert str(resolved).endswith("litricos.raw"), resolved


@pytest.mark.parametrize("name,fmt", [
    ("litricos.raw", "raw"),
    ("litric.raw", "raw"),
    ("litric.qcow2", "qcow2"),
])
def test_drive_format_follows_the_extension(name, fmt):
    assert sb.image_drive_format(Path(name)) == fmt


# ---------------------------------------------------------------------------
# 4. The firmware variable store is copied, never booted in place
# ---------------------------------------------------------------------------

def test_vars_template_is_copied_not_mutated(guest, tmp_path):
    """QEMU writes NVRAM into unit=1 pflash.

    Handing it the distribution's OVMF_VARS mutates the host firmware
    variables, which is exactly what GATE 01 condition 7 is about. Every other
    launcher in this repository copies the template first
    (tools/boot-test.sh, tools/run-qemu-gui.sh, tools/photo_tour_lib.py).
    """
    runner = guest("prompt")
    runner.start()
    try:
        scratch = sb.VARS_SCRATCH
        assert scratch.exists(), "no scratch variable store was created"
        assert scratch.resolve() != runner.vars_template.resolve()
        assert scratch.read_bytes() == runner.vars_template.read_bytes(), \
            "the scratch store must start from the template"
    finally:
        runner.stop()


def test_cmd_line_marks_only_the_code_image_read_only(guest):
    runner = guest("prompt")
    cmd = runner._build_cmd()
    joined = " ".join(cmd)
    assert joined.count("readonly=on") == 1, cmd
    assert f"unit=0,readonly=on,file={runner.code}" in joined
    assert f"unit=1,file={sb.VARS_SCRATCH}" in joined


# ---------------------------------------------------------------------------
# 5. The GATE 01 verifier runs to completion and can fail
# ---------------------------------------------------------------------------

GATE_STUBS = {
    "systemctl": "#!/bin/sh\ncase \"$1\" in\n  is-system-running) echo \"${SYSTEM_STATE:-running}\" ;;\n"
                 "  is-active) echo active ;;\n  list-units) : ;;\nesac\nexit 0\n",
    "dmsetup": "#!/bin/sh\necho \"root: 0 1048576 verity 1 254:1 254:2 4096 4096 262144 1 sha256 a b 1 restart_on_corruption\"\n",
    "findmnt": "#!/bin/sh\ncase \"$*\" in\n  *FSTYPE*) echo erofs ;;\n  *OPTIONS*) echo ro,relatime ;;\nesac\nexit 0\n",
    "systemd-analyze": "#!/bin/sh\necho \"Startup finished in 3.5s (firmware) + 2.1s (loader) + 1.2s (kernel) + 6.9s (userspace) = 13.7s\"\n",
    "cat": "#!/bin/sh\ncase \"$1\" in\n"
           "  /proc/meminfo) printf 'MemTotal: 2048000 kB\\nMemAvailable: 1800000 kB\\n' ;;\n"
           "  /proc/version) echo 'Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP PREEMPT_DYNAMIC' ;;\n"
           "  /sys/kernel/security/lsm) echo 'capability,landlock,lockdown,yama,safesetid,bpf' ;;\n"
           "  /sys/kernel/mm/lru_gen/enabled) echo '0x0007' ;;\n"
           "  /proc/sys/kernel/seccomp/actions_avail) echo 'kill_process errno allow' ;;\n"
           "  /proc/sys/kernel/randomize_va_space) echo 2 ;;\n"
           "  /proc/cmdline) echo 'ro roothash=deadbeef lsm=landlock,lockdown,yama' ;;\n"
           "  *) echo '' ;;\nesac\nexit 0\n",
}


def _run_gate(tmp_path, system_state: str):
    bash = shutil.which("bash")
    if bash is None:
        pytest.skip("bash not available to run the GATE 01 shell script")
    if not GATE_SCRIPT.exists():
        pytest.skip(f"{GATE_SCRIPT} not found")

    stub_dir = tmp_path / "stub"
    stub_dir.mkdir(parents=True, exist_ok=True)
    for name, body in GATE_STUBS.items():
        p = stub_dir / name
        p.write_text(body, encoding="utf-8")
        p.chmod(0o755)

    env = dict(os.environ)
    env["PATH"] = str(stub_dir) + os.pathsep + env.get("PATH", "")
    env["SYSTEM_STATE"] = system_state
    # Explicit UTF-8: the verifier prints ✅/❌/⚠️, and on a host whose locale
    # is not UTF-8 the default text decoding raises inside subprocess instead
    # of returning the output being tested.
    return subprocess.run(
        [bash, str(GATE_SCRIPT)],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        env=env, timeout=120,
    )


def test_gate_verifier_runs_to_completion(tmp_path):
    """Pre-fix this script died after one check and exited 1 (BUG-0006).

    `((PASS++))` returns the value before the increment, so the first
    increment returned status 1 and `set -e` terminated the script: no section
    2-7, no summary, and a non-zero exit for a machine that had just passed
    its first check.
    """
    proc = _run_gate(tmp_path, "running")
    assert "GATE 01 Results" in proc.stdout, (
        "the verifier did not reach its summary:\n" + proc.stdout + proc.stderr
    )
    assert "GATE 01: PASS" in proc.stdout, proc.stdout
    assert proc.returncode == 0, proc.stdout + proc.stderr
    # Section 7 is the last one; reaching it proves no earlier abort.
    assert "7. Boot Time" in proc.stdout, proc.stdout


def test_gate_verifier_reports_a_failure_and_exits_nonzero(tmp_path):
    """A machine that is not running must not be reported as PASS."""
    proc = _run_gate(tmp_path, "degraded")
    assert "GATE 01 Results" in proc.stdout, proc.stdout
    assert "GATE 01: FAIL" in proc.stdout, proc.stdout
    assert proc.returncode == 1, proc.stdout


def test_gate_probes_can_pass_on_a_healthy_machine(tmp_path):
    """The kernel probes are the ones that silently could not pass.

    MGLRU was read and compared to "y" while the file holds a bitmask
    (evidence records 0x0007), Landlock was read from /proc/self/attr/current,
    and PREEMPT was grepped out of the kernel banner as a CONFIG_ symbol.
    """
    proc = _run_gate(tmp_path, "running")
    for name in ("MGLRU enabled", "dm-verity active", "Landlock LSM active",
                 "PREEMPT enabled", "seccomp filter support"):
        line = next((l for l in proc.stdout.splitlines() if name in l), "")
        assert "PASS" in line, f"{name} cannot pass even on a healthy machine: {line!r}"
    assert "0 failed" in proc.stdout, proc.stdout


# ---------------------------------------------------------------------------
# 6. Documentation cites artifacts that exist
# ---------------------------------------------------------------------------

def test_doc_citation_linter_self_test():
    proc = subprocess.run([sys.executable, str(DOC_LINTER), "--self-test"],
                          capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=60)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "PASS" in proc.stdout, proc.stdout


def test_every_cited_artifact_exists():
    """GATE 01 condition 7 was VERIFIED against a file that never existed."""
    proc = subprocess.run([sys.executable, str(DOC_LINTER)],
                          capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=120)
    assert proc.returncode == 0, (
        "documents cite paths that do not exist:\n" + proc.stdout + proc.stderr
    )


# ---------------------------------------------------------------------------
# 7. Guards on the smoke test file itself
# ---------------------------------------------------------------------------

def test_smoke_test_parses_and_has_no_append_hack():
    """QEMU ignores -append without -kernel; BUG-0002 shipped that mistake."""
    src = (TESTS_DIR / "smoke_boot.py").read_text(encoding="utf-8")
    ast.parse(src)  # raises on a truncated file, which is how BUG-0002 shipped
    assert '"-append"' not in src, "QEMU ignores -append for a UKI boot"
    assert "LITRIC_QEMU" in src, "the harness needs a seam to be testable at all"


def test_no_crlf_in_harness_files():
    """CRLF turned `set -euo pipefail` into an invalid option once already."""
    for f in (TESTS_DIR / "smoke_boot.py", GATE_SCRIPT, DOC_LINTER):
        data = f.read_bytes()
        assert b"\r\n" not in data, f"{f.name} has CRLF line endings"
