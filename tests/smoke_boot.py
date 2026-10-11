#!/usr/bin/env python3
"""
LITRIC OS — CI Smoke Boot Test
Headless QEMU + serial console assertion.
Asserts: UEFI → kernel → userspace → login/shell prompt.

Usage:
    pytest tests/smoke_boot.py -v
    python tests/smoke_boot.py              # standalone

Prerequisites:
    - a boot image at build/out/litricos.raw (or build/litricos.raw)
    - OVMF firmware: build/out/OVMF_CODE.4m.fd + OVMF_VARS.fd, or the
      distribution's /usr/share/edk2*/... pair
    - qemu-system-x86_64 installed
    - Python 3.8+

Overrides (all optional, and all also usable to point the harness at a
stub program):
    LITRIC_SMOKE_IMAGE   image to boot
    LITRIC_OVMF_CODE     firmware code image
    LITRIC_OVMF_VARS     firmware variable-store template
    LITRIC_QEMU          QEMU command line (default: qemu-system-x86_64)

STATUS: requires a built image + QEMU to run; without them every case SKIPs
with the exact paths it looked for. See docs/bugs/BUG-0005.

WHAT THIS FILE IS FOR, AND WHAT IT IS NOT
-----------------------------------------
This is the only artifact standing between "we built an image" and "the image
boots" (GATE 01 condition 3). Three separate ways for it to lie were found in
audit and removed, each recorded because the removal is the part that has to
survive:

  1. The boot deadline was not enforced. `self.process.stdout.readline()` blocks
     until a line arrives, and the deadline was only consulted *between* reads,
     so a guest that went silent — a hang, which is exactly the failure mode
     this test exists to catch and which this project has hit before — blocked
     until QEMU exited. Measured on 2026-10-09 with a stub guest: a 4 s deadline
     returned at 25 s (the child's own lifetime). A reader thread now owns the
     pipe and pushes lines into a queue, so the deadline is a property of the
     read loop rather than of the guest's willingness to talk.

  2. Success was detected by substring, including `"#"`, `">"` and `"shell"`.
     Any initrd line ("==> ERROR: Failed to mount root, dropping to emergency
     shell") satisfies all three, so a boot that never reached userspace was
     reported as `SHELL` — PASS. Measured: that exact line returned
     status='SHELL' in 0.0 s. Detection is now anchored regexes that match a
     real login or shell prompt, and emergency-mode text is classified as an
     explicit FAILURE instead of being counted as success.

  3. The artifact paths were stale (build/litric.qcow2, build/OVMF_CODE_4M.fd).
     The current pipeline emits build/out/litricos.raw, so every case skipped
     silently — a gate that reports nothing and looks like an environment
     problem. Paths are now discovered, and a skip names every candidate.

A fourth was a host-side write: `unit=1` pflash was handed the firmware's own
variable store, which QEMU writes to (boot order, NVRAM). tools/boot-test.sh
copies the template to a scratch file first; so does this file now.
"""

from __future__ import annotations

import queue
import shlex
import shutil
import subprocess
import threading
import time
import os
import re
import sys
import signal

import pytest
from pathlib import Path
from datetime import datetime

# =============================================================================
# CONFIGURATION
# =============================================================================

PROJECT_ROOT = Path(__file__).parent.parent.resolve()
BUILD_DIR = PROJECT_ROOT / "build"
LOGS_DIR = PROJECT_ROOT / "tools" / "logs"

# Boot timeout (seconds)
BOOT_TIMEOUT = 60

# The QEMU invocation, overridable so the harness itself can be exercised
# without a VM. Available, and used, since 2026-10-09: a harness with no seam
# is a harness that cannot be tested, which is how BUG-0002 happened.
QEMU_CMD = shlex.split(os.environ.get("LITRIC_QEMU", "qemu-system-x86_64"))

# Candidate image paths, in the order the pipeline actually produces them:
# mkosi writes build/out/litricos.raw (image/mkosi.conf ImageId=litricos,
# OutputDirectory=../build/out). The build/litric.qcow2 name belongs to the
# retired tools/build-image.sh and is kept last so an old tree still runs.
IMAGE_CANDIDATES = [
    BUILD_DIR / "out" / "litricos.raw",
    BUILD_DIR / "litricos.raw",
    BUILD_DIR / "out" / "litricos.qcow2",
    BUILD_DIR / "litric.qcow2",
    BUILD_DIR / "litric.raw",
]

FIRMWARE_CODE_CANDIDATES = [
    BUILD_DIR / "out" / "OVMF_CODE.4m.fd",
    BUILD_DIR / "OVMF_CODE.4m.fd",
    BUILD_DIR / "OVMF_CODE_4M.fd",
    Path("/usr/share/edk2/x64/OVMF_CODE.4m.fd"),
    Path("/usr/share/OVMF/OVMF_CODE.fd"),
    Path("/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"),
]

FIRMWARE_VARS_CANDIDATES = [
    BUILD_DIR / "out" / "OVMF_VARS.fd",
    BUILD_DIR / "OVMF_VARS.4m.fd",
    BUILD_DIR / "OVMF_VARS_4M.fd",
    Path("/usr/share/edk2/x64/OVMF_VARS.4m.fd"),
    Path("/usr/share/OVMF/OVMF_VARS.fd"),
    Path("/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"),
]

# UEFI variable store, copied from the template before every boot.
# QEMU opens unit=1 pflash writable and OVMF stores NVRAM in it, so handing it
# the distribution's file mutates the host's firmware variables -- and GATE 01
# condition 7 is "zero host-disk writes outside the project dir". The scratch
# copy lives inside the project for that reason.
VARS_SCRATCH = BUILD_DIR / "OVMF_VARS.smoke.fd"

# Serial output that means the boot reached a usable prompt.
#
# These are anchored and specific on purpose. The previous set contained "#",
# ">" and "shell", which match ordinary initrd and systemd output and made the
# test report success for a boot that stopped in early userspace.
SHELL_PATTERNS = [
    re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]* login:\s*$"),          # getty
    re.compile(r"^root@[A-Za-z0-9._-]+[:~][^\n]*[#$]\s*$"),         # PS1, root
    re.compile(r"^[A-Za-z0-9._-]+@[A-Za-z0-9._-]+:[^\n]*[#$]\s*$"),  # PS1, any user
]

# Serial output that means the boot cannot reach one, even though the guest is
# still talking. Classified as FAILURE so the reason is named in the report
# instead of being reported as a timeout with no explanation.
FAILURE_PATTERNS = [
    "kernel panic",
    "Kernel panic",
    "You are in emergency mode",
    "emergency shell",
    "Cannot open access to console",
    "root account is locked",
    "sulogin:",
    "general protection fault",
    "double fault",
    "Oops",
    "BUG:",
    "RIP:",
]

# Output worth quoting when the boot does not reach a prompt, but which does
# not on its own decide the verdict -- a service can fail and the system still
# reach a login prompt (`/efi` is mounted nofail for precisely that reason).
WARN_PATTERNS = [
    "Failed to mount",
    "Failed to start",
    "degraded",
    "Timed out",
    "not found",
]

# Retained for callers that refer to the original module-level constants.
UEFI_INDICATORS = [
    "UEFI", "EDK2", "OVMF", "TianoCore",
    "Boot Manager", "Loading",
]

KERNEL_INDICATORS = [
    "Linux version", "Freeing SMP", "Run /init",
    "systemd", "Welcome",
]

SHELL_INDICATORS = SHELL_PATTERNS
PANIC_INDICATORS = FAILURE_PATTERNS


def _resolve(env_var: str, candidates: list, label: str) -> Path:
    """First existing candidate, or the environment override if it is set.

    Falling through to the first candidate when nothing exists keeps the
    subsequent skip message concrete: it names the path the pipeline is
    expected to produce rather than a generic "not found".
    """
    override = os.environ.get(env_var)
    if override:
        return Path(override).expanduser()
    for c in candidates:
        if c.exists():
            return c
    return candidates[0]


def discover_image() -> Path:
    return _resolve("LITRIC_SMOKE_IMAGE", IMAGE_CANDIDATES, "image")


def discover_firmware_code() -> Path:
    return _resolve("LITRIC_OVMF_CODE", FIRMWARE_CODE_CANDIDATES, "OVMF code")


def discover_firmware_vars() -> Path:
    return _resolve("LITRIC_OVMF_VARS", FIRMWARE_VARS_CANDIDATES, "OVMF vars")


IMAGE = discover_image()
OVMF_CODE = discover_firmware_code()
OVMF_VARS = discover_firmware_vars()


def image_drive_format(path: Path) -> str:
    """qcow2 must be declared, and corruption offsets only mean anything raw."""
    suffix = path.suffix.lower()
    if suffix == ".qcow2":
        return "qcow2"
    return "raw"


# =============================================================================
# QEMU PROCESS MANAGEMENT
# =============================================================================

class QemuRunner:
    """Manages a QEMU process for smoke testing.

    Serial output is drained by a reader thread into a queue. That is the whole
    point of the class being more than four lines: a blocking read on the pipe
    makes the timeout advisory, and an advisory timeout on the one test that
    gates the project is how a hang gets recorded as an unfinished run instead
    of a failure.
    """

    def __init__(self, smp=2, mem="2G", timeout=BOOT_TIMEOUT):
        self.smp = smp
        self.mem = mem
        self.timeout = timeout
        self.process = None
        self.output = []
        self.warnings = []
        self.log_file = None
        # Overridable so a test can boot a corrupted copy.
        self.image = IMAGE
        self.code = discover_firmware_code()
        self.vars_template = discover_firmware_vars()
        self._queue = None
        self._reader = None
        self._reader_eof = threading.Event()
        # How far the boot got, for a timeout that says where it stopped
        # rather than only that nothing arrived.
        self.stages = {"firmware": False, "kernel": False}

    def _prepare_vars(self):
        """Copy the firmware variable template into the project before boot.

        Refreshed every run: QEMU writes NVRAM into it, and a stale NVRAM can
        hold a boot entry from a previous image, which changes what is booted
        relative to a clean machine -- the opposite of what a smoke test wants.
        """
        if self.vars_template is None or not self.vars_template.exists():
            raise FileNotFoundError(
                f"OVMF variable template not found: {self.vars_template}"
            )
        VARS_SCRATCH.parent.mkdir(parents=True, exist_ok=True)
        if self.vars_template.resolve() != VARS_SCRATCH.resolve():
            shutil.copyfile(self.vars_template, VARS_SCRATCH)
        return VARS_SCRATCH

    def _build_cmd(self):
        """Build the QEMU command line."""
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        self.log_file = LOGS_DIR / f"smoke-{ts}.log"
        vars_path = self._prepare_vars()

        return QEMU_CMD + [
            "-machine", "q35,accel=tcg,kernel-irqchip=off",
            "-cpu", "max",
            "-smp", str(self.smp),
            "-m", self.mem,
            "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={self.code}",
            "-drive", f"if=pflash,format=raw,unit=1,file={vars_path}",
            "-drive",
            f"if=virtio,format={image_drive_format(self.image)},file={self.image}",
            "-device", "virtio-net-pci,netdev=n0",
            "-netdev", "user,id=n0",
            "-serial", "mon:stdio",
            "-nographic",
            "-no-reboot",
            # No -append: QEMU only honours it with -kernel (direct kernel
            # boot). This image boots a UKI from the ESP, whose command line
            # is baked into the signed PE. console=ttyS0 is set there instead
            # (see image/mkosi.conf KernelCommandLine=).
        ]

    def _read_loop(self, stream):
        """Drain the child's stdout into the queue until EOF."""
        try:
            for line in iter(stream.readline, ""):
                self._queue.put(line.rstrip("\n"))
        except Exception:  # pragma: no cover - a closed pipe during teardown
            pass
        finally:
            self._reader_eof.set()
            self._queue.put(None)

    def start(self):
        """Start the QEMU process."""
        self.code = discover_firmware_code()
        self.vars_template = discover_firmware_vars()
        cmd = self._build_cmd()

        if not shutil.which(cmd[0]) and not Path(cmd[0]).exists():
            raise FileNotFoundError(
                f"QEMU executable not found: {cmd[0]!r} "
                f"(set LITRIC_QEMU to override)"
            )

        self.process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            # Explicit, not locale-dependent: serial output is byte-oriented
            # and a single invalid byte must not kill the reader thread on a
            # host whose default encoding is not UTF-8.
            encoding="utf-8",
            errors="replace",
            bufsize=1,
            # New process group so stop() can kill the whole tree. POSIX only;
            # the build host is Linux, and on Windows this is simply omitted
            # rather than raising at start().
            **((
                {"start_new_session": True}
                if hasattr(os, "setsid")
                else {}
            )),
        )

        self._queue = queue.Queue()
        self._reader = threading.Thread(
            target=self._read_loop, args=(self.process.stdout,), daemon=True
        )
        self._reader.start()

        return self.process

    @staticmethod
    def classify(line: str) -> str:
        """One line -> 'PANIC' | 'FAILURE' | 'SHELL' | 'WARN' | ''."""
        for pattern in SHELL_PATTERNS:
            if pattern.search(line):
                return "SHELL"
        for indicator in FAILURE_PATTERNS:
            if indicator in line:
                return "FAILURE"
        for indicator in WARN_PATTERNS:
            if indicator in line:
                return "WARN"
        return ""

    def read_output(self, timeout=None):
        """Read serial output until the deadline or a verdict line.

        Returns (lines, status, detail) with status one of SHELL, FAILURE,
        TIMEOUT. The deadline is enforced by this loop, so a guest that stops
        talking produces TIMEOUT at `timeout` seconds rather than whenever the
        child happens to exit.
        """
        if timeout is None:
            timeout = self.timeout

        if self._queue is None:
            raise RuntimeError("read_output() called before start()")

        deadline = time.time() + timeout
        lines = []

        while True:
            remaining = deadline - time.time()
            if remaining <= 0:
                self._save_log()
                detail = (
                    f"No shell prompt within {timeout}s "
                    f"(timeout is enforced; the guest may still be running)"
                    f" | furthest stage: {self.furthest_stage()}"
                )
                if self.warnings:
                    detail += " | last warning: " + self.warnings[-1]
                return lines, "TIMEOUT", detail
            if self._reader_eof.is_set() and self._queue.empty():
                break
            try:
                line = self._queue.get(timeout=min(0.25, remaining))
            except queue.Empty:
                continue
            if line is None:
                break

            lines.append(line)
            self.output.append(line)
            self._note_stage(line)

            verdict = self.classify(line)
            if verdict == "SHELL":
                self._save_log()
                return lines, "SHELL", line
            if verdict == "FAILURE":
                self._save_log()
                return lines, "FAILURE", line
            if verdict == "WARN":
                self.warnings.append(line)

        # The child's stdout closed: QEMU exited (or was killed) without a
        # prompt. Report TIMEOUT so callers keep their existing contract, but
        # say which of the two happened.
        # EOF on the pipe means the child closed it, so it has exited or is
        # about to. poll() alone can still race that by a few milliseconds and
        # report "output ended" for a process that exited with a status; wait
        # briefly for the real code, because the code is the diagnosis.
        rc = None
        if self.process is not None:
            try:
                rc = self.process.wait(timeout=2)
            except Exception:
                rc = self.process.poll()
        self._save_log()
        if rc is not None:
            detail = f"QEMU exited with status {rc} without a shell prompt"
        else:
            detail = "QEMU output ended without a shell prompt"
        detail += f" | furthest stage: {self.furthest_stage()}"
        if self.warnings:
            detail += " | last warning: " + self.warnings[-1]
        return lines, "TIMEOUT", detail

    def _note_stage(self, line: str) -> None:
        """Record how far the boot got, for a timeout message that places it."""
        if not self.stages["firmware"]:
            self.stages["firmware"] = any(i in line for i in UEFI_INDICATORS)
        if not self.stages["kernel"]:
            self.stages["kernel"] = any(i in line for i in KERNEL_INDICATORS)

    def furthest_stage(self) -> str:
        if self.stages["kernel"]:
            return "userspace or kernel"
        if self.stages["firmware"]:
            return "firmware (no kernel line seen)"
        return "nothing above the serial port"

    def stop(self):
        """Stop the QEMU process and the reader thread."""
        if self.process and self.process.poll() is None:
            try:
                if hasattr(os, "killpg"):
                    os.killpg(os.getpgid(self.process.pid), signal.SIGTERM)
                else:
                    self.process.terminate()  # Windows: no process groups here
                self.process.wait(timeout=5)
            except Exception:
                try:
                    if hasattr(os, "killpg"):
                        os.killpg(os.getpgid(self.process.pid), signal.SIGKILL)
                    else:
                        self.process.kill()
                except Exception:
                    pass
        if self._reader is not None:
            self._reader.join(timeout=2)

    def _save_log(self):
        """Save captured output to log file."""
        if self.log_file:
            with open(self.log_file, "w", encoding="utf-8") as f:
                f.write("\n".join(self.output))


# =============================================================================
# TEST FIXTURES
# =============================================================================

@pytest.fixture(scope="module")
def check_prereqs():
    """Verify all prerequisites are met, or skip naming every path probed."""
    errors = []

    image = discover_image()
    code = discover_firmware_code()
    vars_template = discover_firmware_vars()

    if not image.exists():
        errors.append(
            f"image not found: {image}"
            f" (looked for: {', '.join(str(p) for p in IMAGE_CANDIDATES)})"
        )
        comp = image.with_suffix(image.suffix + ".zst")
        if comp.exists():
            errors.append(f"  found {comp} -- decompress it first (zstd -d)")
    if not code.exists():
        errors.append(f"OVMF code not found: {code}")
    if not vars_template.exists():
        errors.append(f"OVMF vars not found: {vars_template}")

    # Check QEMU is available
    if not shutil.which(QEMU_CMD[0]) and not Path(QEMU_CMD[0]).exists():
        errors.append(f"qemu not found: {QEMU_CMD[0]} (set LITRIC_QEMU to override)")

    if errors:
        pytest.skip("Prerequisites not met:\n" + "\n".join(errors))


# =============================================================================
# TEST CASES
# =============================================================================

class TestBootSmoke:
    """CI smoke tests for Litric boot."""

    def test_image_exists(self, check_prereqs):
        """Verify the disk image exists and is non-empty."""
        image = discover_image()
        assert image.exists(), f"Image not found: {image}"
        assert image.stat().st_size > 0, "Image is empty"
        assert image.stat().st_size > 100 * 1024 * 1024, \
            f"Image suspiciously small: {image.stat().st_size} bytes"

    def test_ovmf_firmware_exists(self, check_prereqs):
        """Verify OVMF firmware files exist and the variable store is copiable."""
        code = discover_firmware_code()
        vars_template = discover_firmware_vars()
        assert code.exists(), f"OVMF_CODE not found: {code}"
        assert vars_template.exists(), f"OVMF_VARS not found: {vars_template}"
        assert code.stat().st_size > 0, "OVMF_CODE is empty"
        assert vars_template.stat().st_size > 0, "OVMF_VARS is empty"

        # The template must be readable and a scratch copy must be writable;
        # booting the template in place would write to the host's firmware
        # variables (GATE 01 condition 7).
        runner = QemuRunner()
        runner.vars_template = vars_template
        scratch = runner._prepare_vars()
        assert scratch.exists(), f"scratch variable store not created: {scratch}"
        assert scratch.stat().st_size == vars_template.stat().st_size
        assert scratch.resolve() != vars_template.resolve(), \
            "the firmware template must not be booted in place; QEMU writes to unit=1"

    def test_boot_to_shell(self, check_prereqs):
        """
        CRITICAL: Boot Litric to a shell prompt.
        This is the primary CI gate for Phase 01.

        Flow: UEFI → kernel → init → login/shell
        """
        runner = QemuRunner(smp=2, mem="2G", timeout=BOOT_TIMEOUT)

        try:
            runner.start()
            lines, status, detail = runner.read_output()

            # Verify we reached a shell prompt. An emergency shell, a panic or
            # silence is a failure of this condition, not a pass with a caveat.
            assert status == "SHELL", \
                f"Boot did not reach a shell. Status: {status}\n" \
                f"Detail: {detail}\n" \
                f"Last 20 lines:\n" + '\n'.join(lines[-20:])

        finally:

            runner.stop()

    def test_smoke_test_can_actually_fail(self, check_prereqs, tmp_path):
        """The test that tests the test.

        GATE 01 requires the smoke test to "fail correctly when a deliberate
        panic is injected". A boot check that reports success on a broken image
        is worse than no check: it turns a broken build green.

        Injection is by CORRUPTING A COPY of the image, not by appending to the
        kernel command line. That distinction matters and cost a debugging cycle
        to learn: QEMU's -append is only honoured together with -kernel (direct
        kernel boot). This image boots a Unified Kernel Image from the ESP via
        UEFI, so -append is silently ignored and a command-line injection would
        appear to work while testing nothing.

        Where the corruption goes, and why here:

          * first 8 KiB     -- protective MBR and the PRIMARY GPT header.
          * last 8 KiB      -- the BACKUP GPT header. Both are written: UEFI
            falls back to the backup copy when the primary is unusable, so
            corrupting only the primary is not guaranteed to stop the boot and
            a passing assertion could mean "the fallback worked" rather than
            "the check detected damage".
          * 1 MiB + 4 KiB   -- the FAT boot sector of the ESP, which is where
            mkosi places the first partition. This is what makes the failure
            deterministic: the firmware cannot read the volume and therefore
            cannot find the UKI at all.
          * 64 MiB + 16 KiB -- well inside the verity-protected system
            partition. This one is the Phase 03 probe: dm-verity must reject
            the tampered hash tree rather than mount it. It is kept next to the
            deterministic corruption because a verity failure alone may be
            masked by caches or by a partition layout change, and a
            false-alarm failure here would be recorded as a defect that does
            not exist.

        Offset writes require a RAW image: on qcow2 a file offset is not a guest
        offset, so the assertion would be meaningless rather than wrong.
        """
        image = discover_image()
        fmt = image_drive_format(image)
        if fmt != "raw":
            pytest.skip(
                f"offset-level corruption needs a raw image; found {image.name} "
                f"({fmt}): a file offset is not a guest offset in that format"
            )

        broken = tmp_path / "litricos-corrupt.raw"
        shutil.copy2(image, broken)
        size = broken.stat().st_size

        def smash(offset: int, length: int, byte: int = 0xFF) -> None:
            with open(broken, "r+b") as fh:
                fh.seek(offset)
                fh.write(bytes([byte]) * length)

        smash(0, 8 * 1024)                      # MBR + primary GPT
        smash(size - 8 * 1024, 8 * 1024)        # backup GPT
        smash(1024 * 1024, 4 * 1024)            # ESP FAT boot sector
        with open(broken, "r+b") as fh:         # verity probe span
            fh.seek(64 * 1024 * 1024)
            fh.write(bytes([0xDE, 0xAD, 0xBE, 0xEF]) * 4096)

        runner = QemuRunner(smp=1, mem="1G", timeout=BOOT_TIMEOUT)
        runner.image = broken
        try:
            runner.start()
            _lines, status, detail = runner.read_output()
            assert status != "SHELL", (
                "A deliberately corrupted image booted to a shell. The smoke "
                "test cannot detect failure, so a passing run proves nothing. "
                f"detail: {detail}"
            )
        finally:
            runner.stop()


if __name__ == "__main__":
    # Standalone mode, for running without pytest on a build host.
    missing = []
    image = discover_image()
    if not image.exists():
        missing.append(f"image: {image}")
    if not discover_firmware_code().exists():
        missing.append(f"OVMF code: {discover_firmware_code()}")
    if not discover_firmware_vars().exists():
        missing.append(f"OVMF vars: {discover_firmware_vars()}")
    if not shutil.which(QEMU_CMD[0]) and not Path(QEMU_CMD[0]).exists():
        missing.append(f"{QEMU_CMD[0]} on PATH")

    if missing:
        print("NOT RUN -- prerequisites missing:")
        for m in missing:
            print(f"  - {m}")
        print("\nBuild an image first: tools/build.sh --profile nano")
        sys.exit(77)  # 77 = skipped, distinct from 1 = failed

    runner = QemuRunner()
    try:
        runner.start()
        lines, status, detail = runner.read_output()
        print(f"status: {status}")
        print(f"detail: {detail}")
        sys.exit(0 if status == "SHELL" else 1)
    finally:
        runner.stop()
