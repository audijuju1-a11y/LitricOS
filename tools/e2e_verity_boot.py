#!/usr/bin/env python3
"""Boot a LitricOS disk through OVMF and prove the verified root reaches UI.

This intentionally uses the production UEFI path created by mkosi.  It does
not pass ``-kernel`` or ``-initrd`` to QEMU: firmware loads the signed UKI from
the image ESP, then the initrd creates the dm-verity root mapping.

The diagnostic serial channel confirms the guest can see ``/dev/mapper/root``
and that the Litric shell service is active.  A QMP framebuffer dump is saved
at the same time so a pass has both guest-state and rendered-screen evidence.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys
import tempfile
import time

from photo_tour_lib import boot_qmp, exec_guest


PROBE = (
    "printf __LITRIC_E2E_BEGIN__; "
    "findmnt -n -o SOURCE /; "
    "test -e /dev/mapper/root && echo root_mapper=present || echo root_mapper=missing; "
    "printf shell=; systemctl is-active litric-shell.service; "
    "printf first_run=; systemctl is-active litric-first-run.service || true; "
    "systemctl --failed --no-legend; "
    "printf __LITRIC_E2E_END__"
)


def parse_args() -> argparse.Namespace:
    here = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--image",
        type=Path,
        default=here / "build" / "out" / "litricos.raw",
        help="raw LitricOS disk image to boot",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=here / "build" / "e2e-verity-boot",
        help="directory for serial and framebuffer evidence",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=100.0,
        help="maximum seconds to wait for guest diagnostic shell",
    )
    parser.add_argument(
        "--runtime",
        type=Path,
        help="native-Linux directory for temporary QMP, serial, and UEFI-var files",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    image = args.image.resolve()
    out = args.out.resolve()
    if not image.is_file():
        raise SystemExit(f"image does not exist: {image}")
    if args.timeout <= 0:
        raise SystemExit("--timeout must be positive")

    out.mkdir(parents=True, exist_ok=True)
    if args.runtime is not None:
        runtime = args.runtime.resolve()
    elif str(out).startswith("/mnt/"):
        # The repository may be on a Windows drive under WSL.  Its evidence
        # remains there; only AF_UNIX runtime sockets must live under Linux.
        runtime = Path(tempfile.mkdtemp(prefix="litricos-e2e-verity-"))
    else:
        runtime = out / "runtime"
    runtime.mkdir(parents=True, exist_ok=True)
    os.environ["LITRIC_QEMU_RUNTIME_DIR"] = str(runtime)
    proc = None
    qmp = None
    success = False
    try:
        proc, qmp, serial, diag = boot_qmp(str(out), image=str(image))
        print(f"E2E_QEMU_STARTED image={image}")
        print(f"E2E_SERIAL={serial}")
        print(f"E2E_DIAG={diag}")
        print(f"E2E_RUNTIME={runtime}")

        deadline = time.monotonic() + args.timeout
        next_probe = time.monotonic() + 12.0
        reply = ""
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                raise RuntimeError(f"QEMU exited early with status {proc.returncode}")
            if time.monotonic() >= next_probe:
                try:
                    reply = exec_guest(PROBE, diag, timeout=6.0)
                except OSError as exc:
                    print(f"E2E_DIAG_RETRY {type(exc).__name__}: {exc}")
                else:
                    if "__LITRIC_E2E_END__" in reply:
                        print("E2E_GUEST_REPLY_BEGIN")
                        print(reply)
                        print("E2E_GUEST_REPLY_END")
                        if "/dev/mapper/root" not in reply:
                            raise RuntimeError("guest shell did not mount the root mapper")
                        if "root_mapper=present" not in reply:
                            raise RuntimeError("guest shell did not find /dev/mapper/root")
                        if "shell=active" not in reply:
                            # The desktop deliberately stays gated while the
                            # person is completing the first-run wizard.  Keep
                            # polling instead of treating an interactive setup
                            # screen as a boot failure.
                            print("E2E_WAITING_FOR_INTERACTIVE_SETUP")
                        else:
                            success = True
                            break
                next_probe = time.monotonic() + 4.0
            time.sleep(0.5)

        # QEMU finishes the screendump before it replies, so this is a full
        # framebuffer captured from the guest's actual VGA device.
        screenshot = out / "first-boot-after-verity.ppm"
        qmp.cmd("screendump", filename=str(screenshot))
        print(f"E2E_SCREENSHOT={screenshot}")

        serial_text = Path(serial).read_text(encoding="utf-8", errors="replace")
        (out / "serial-tail.txt").write_text(serial_text[-12000:], encoding="utf-8")
        print("E2E_SERIAL_TAIL_BEGIN")
        print(serial_text[-12000:])
        print("E2E_SERIAL_TAIL_END")
        if not success:
            raise RuntimeError("guest diagnostic console never reached a usable shell")
    finally:
        if qmp is not None:
            try:
                qmp.cmd("quit")
            except (ConnectionError, OSError, RuntimeError):
                pass
            qmp.close()
        if proc is not None:
            try:
                proc.wait(timeout=10)
            except Exception:
                proc.kill()

    print("E2E_BOOT_VERITY_PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
