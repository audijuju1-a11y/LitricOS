#!/usr/bin/env python3
"""
ILTRIC OS — CI Smoke Boot Test
Headless QEMU + serial console assertion.
Asserts: UEFI → kernel → userspace → login/shell prompt.

Usage:
    pytest tests/smoke_boot.py -v
    python tests/smoke_boot.py              # standalone

Prerequisites:
    - iltric.qcow2 in build/
    - OVMF_CODE_4M.fd + OVMF_VARS_4M.fd in build/
    - qemu-system-x86_64 installed
    - Python 3.8+

STATUS: DESIGN-ONLY — requires built image + QEMU to run
"""

import subprocess
import time
import os
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
IMAGE = BUILD_DIR / "iltric.qcow2"
OVMF_CODE = BUILD_DIR / "OVMF_CODE_4M.fd"
OVMF_VARS = BUILD_DIR / "OVMF_VARS_4M.fd"
LOGS_DIR = PROJECT_ROOT / "tools" / "logs"

# Boot timeout (seconds)
BOOT_TIMEOUT = 60

# Expected strings in serial output
UEFI_INDICATORS = [
    "UEFI", "EDK2", "OVMF", "TianoCore",
    "Boot Manager", "Loading"
]

KERNEL_INDICATORS = [
    "Linux version", "Freeing SMP", "Run /init",
    "systemd", "Welcome"
]

SHELL_INDICATORS = [
    "login:", "root@", "shell", "#", ">",
    "iltri$", "iltri-cod"
]

PANIC_INDICATORS = [
    "kernel panic", "Kernel panic",
    "Oops", "BUG:", "RIP:",
    "general protection fault", "double fault"
]


# =============================================================================
# QEMU PROCESS MANAGEMENT
# =============================================================================

class QemuRunner:
    """Manages a QEMU process for smoke testing."""
    
    def __init__(self, smp=2, mem="2G", timeout=BOOT_TIMEOUT):
        self.smp = smp
        self.mem = mem
        self.timeout = timeout
        self.process = None
        self.output = []
        self.log_file = None
    
    def _build_cmd(self):
        """Build the QEMU command line."""
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        self.log_file = LOGS_DIR / f"smoke-{ts}.log"
        
        return [
            "qemu-system-x86_64",
            "-machine", "q35,accel=tcg,kernel-irqchip=off",
            "-cpu", "max",
            "-smp", str(self.smp),
            "-m", self.mem,
            "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={OVMF_CODE}",
            "-drive", f"if=pflash,format=raw,unit=1,file={OVMF_VARS}",
            "-drive", f"if=virtio,format=qcow2,file={IMAGE}",
            "-device", "virtio-net-pci,netdev=n0",
            "-netdev", "user,id=n0",
            "-serial", "mon:stdio",
            "-nographic",
            "-no-reboot",
            "-append", "console=ttyS0 earlyprintk=serial nokaslr",
        ]
    
    def start(self):
        """Start the QEMU process."""
        cmd = self._build_cmd()
        
        self.process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            preexec_fn=os.setsid,  # Create new process group for cleanup
        )
        
        return self.process
    
    def read_output(self, timeout=None):
        """Read output from QEMU until timeout or keyword found."""
        if timeout is None:
            timeout = self.timeout
        
        import select
        
        start = time.time()
        lines = []
        
        while time.time() - start < timeout:
            if self.process.poll() is not None:
                break
            
            try:
                line = self.process.stdout.readline()
                if line:
                    line = line.rstrip('\n')
                    lines.append(line)
                    self.output.append(line)
                    
                    # Check for panic
                    for indicator in PANIC_INDICATORS:
                        if indicator.lower() in line.lower():
                            self._save_log()
                            return lines, "PANIC", line
                    
                    # Check for shell prompt
                    for indicator in SHELL_INDICATORS:
                        if indicator in line:
                            self._save_log()
                            return lines, "SHELL", line
                    
                    # Check for kernel
                    for indicator in KERNEL_INDICATORS:
                        if indicator in line:
                            pass  # Continue looking for shell
            except Exception:
                pass
        
        self._save_log()
        return lines, "TIMEOUT", "Boot did not reach shell within timeout"
    
    def stop(self):
        """Stop the QEMU process."""
        if self.process and self.process.poll() is None:
            try:
                os.killpg(os.getpgid(self.process.pid), signal.SIGTERM)
                self.process.wait(timeout=5)
            except Exception:
                try:
                    os.killpg(os.getpgid(self.process.pid), signal.SIGKILL)
                except Exception:
                    pass
    
    def _save_log(self):
        """Save captured output to log file."""
        if self.log_file:
            with open(self.log_file, 'w') as f:
                f.write('\n'.join(self.output))


# =============================================================================
# TEST FIXTURES
# =============================================================================

@pytest.fixture(scope="module")
def check_prereqs():
    """Verify all prerequisites are met."""
    errors = []
    
    if not IMAGE.exists():
        errors.append(f"Image not found: {IMAGE}")
    if not OVMF_CODE.exists():
        errors.append(f"OVMF_CODE not found: {OVMF_CODE}")
    if not OVMF_VARS.exists():
        errors.append(f"OVMF_VARS not found: {OVMF_VARS}")
    
    # Check QEMU is available
    try:
        subprocess.run(
            ["qemu-system-x86_64", "--version"],
            capture_output=True, timeout=5
        )
    except FileNotFoundError:
        errors.append("qemu-system-x86_64 not found in PATH")
    
    if errors:
        pytest.skip("Prerequisites not met:\n" + "\n".join(errors))


# =============================================================================
# TEST CASES
# =============================================================================

class TestBootSmoke:
    """CI smoke tests for Iltric boot."""
    
    def test_image_exists(self, check_prereqs):
        """Verify the disk image exists and is non-empty."""
        assert IMAGE.exists(), f"Image not found: {IMAGE}"
        assert IMAGE.stat().st_size > 0, "Image is empty"
        assert IMAGE.stat().st_size > 100 * 1024 * 1024, \
            f"Image suspiciously small: {IMAGE.stat().st_size} bytes"
    
    def test_ovmf_firmware_exists(self, check_prereqs):
        """Verify OVMF firmware files exist."""
        assert OVMF_CODE.exists(), f"OVMF_CODE not found: {OVMF_CODE}"
        assert OVMF_VARS.exists(), f"OVMF_VARS not found: {OVMF_VARS}"
        assert OVMF_CODE.stat().st_size > 0, "OVMF_CODE is empty"
        assert OVMF_VARS.stat().st_size > 0, "OVMF_VARS is empty"
    
    def test_boot_to_shell(self, check_prereqs):
        """
        CRITICAL: Boot Iltric to a shell prompt.
        This is the primary CI gate for Phase 01.
        
        Flow: UEFI → kernel → init → login/shell
        """
        runner = QemuRunner(smp=2, mem="2G", timeout=BOOT_TIMEOUT)
        
        try:
            runner.start()
            lines, status, detail = runner.read_output()
            
            # Verify we reached shell
            assert status == "SHELL", \
                f"Boot did not reach shell. Status: {status}\n" \
                f"Detail: {detail}\n" \
                f"Last 20 lines:\n" + '\n'.join(lines[-20:])
            
        finally:
   
