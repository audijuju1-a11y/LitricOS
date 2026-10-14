# BUG-0017 — the boot smoke test listened to a console that never gets a prompt

Bug ID        : BUG-0017
Run           : the first image built from this tree, 2026-10-11
No collision  : BUG-0016 is *The typeface printed boxes, letters touched, and three
layouts collided*, which is a different bug in a different subsystem. This record
was written under that number by mistake and renumbered.


**Severity:** S1 — the test that gates GATE 01 condition 3 could not pass, and its
failure said "TIMEOUT", which reads as a slow machine rather than as a harness
looking at the wrong port.
**Found:** 2026-10-11, by booting the image built from this tree.
**Status:** **fixed** in `tests/smoke_boot.py`, with the measurement below.

## What was wrong

The image has two serial ports and they are not the same console:

| Port | What runs on it | Where that is decided |
|---|---|---|
| `ttyS0` | the kernel and systemd log | `KernelCommandLine=console=tty0 console=ttyS0,115200` in `image/mkosi.conf` |
| `ttyS1` | an autologin root shell | `serial-getty@ttyS1` enabled in `image/mkosi.postinst.chroot`, autologin from `image/mkosi.extra/usr/lib/systemd/system/serial-getty@ttyS1.service.d/autologin.conf` |

`tests/smoke_boot.py` attached **only** `-serial mon:stdio`, which is `ttyS0`, and
then waited for a shell prompt on it. There is no getty on `ttyS0`, so no prompt is
ever written there: the harness was waiting for a line the image is designed never
to produce. `tools/boot-test.sh` attaches the same port and has the same problem.

There was a second, independent reason it could not have matched even if it had
been listening on the right port: the prompt patterns were anchored on the user
name (`^root@host:~#`), and Arch's `/etc/bash.bashrc` sets
`PS1='[\u@\h \W]\$ '`, so this image prints `[root@litricos ~]# ` — with a leading
bracket and no colon.

## The measurement

```
$ LITRIC_SMOKE_IMAGE=build/boot/litricos.raw python3 tests/smoke_boot.py
status: TIMEOUT
detail: No shell prompt within 300s (timeout is enforced; the guest may still be
        running) | furthest stage: userspace or kernel
```

The same image, booted with both ports captured (`-serial file:…s0.log
-serial file:…s1.log`), was sitting at a prompt on ttyS1 the entire time:

```
Arch Linux 7.2.9-arch1-1 (ttyS1)

litricos login: root (automatic login)

[root@litricos ~]#
```

and `ttyS0` at the same moment showed 78 `Reached target` lines, `bochs-drm`
framebuffer initialisation, `systemd-networkd` starting, and the shell reporting
`litric.shell.frames=34`. In other words the system was up and the test said
nothing had happened.

## The fix

1. `tests/smoke_boot.py` opens the second serial port as a unix socket
   (`-chardev socket,id=cons1,path=build/smoke-console.sock,server=on,wait=off`
   plus `-serial chardev:cons1`) and reads it into the same queue as `ttyS0`, so
   both ports are evidence for the one question the test asks. The socket is
   removed on `stop()`; a leftover socket makes the next run's `chardev` fail to
   bind.
2. A prompt pattern for what the image actually prints
   (`^\[root@[A-Za-z0-9._-]+[^\]]*\][#$]\s*$`) was added, with the measured
   string in the comment. The existing patterns stay: a different image may have
   a different `PS1`, and dropping them would trade one blind spot for another.
3. The console tail is flushed when it matches a prompt pattern rather than
   waiting for a newline, because a serial console's prompt arrives without one.
   That single detail is why a line-based reader missed it.
4. Two things the boot path needed and did not have: `LITRIC_ACCEL` (the harness
   hardcoded `accel=tcg`, which is 20-40x slower than KVM and, measured here,
   does not finish inside 300 s), and `LITRIC_BOOT_TIMEOUT` with the 60 s default
   unchanged. The acceleration is printed alongside a result rather than assumed.
5. `tools/console-probe.py` is added: it boots the image, waits for the prompt on
   ttyS1, runs a command and prints the transcript. That is how the facts quoted
   in `docs/EVIDENCE.md` were read out of the running guest.

## Proof the fix works, and that it did not become a rubber stamp

```
tests/smoke_boot.py::TestBootSmoke::test_image_exists                    PASSED
tests/smoke_boot.py::TestBootSmoke::test_ovmf_firmware_exists            PASSED
tests/smoke_boot.py::TestBootSmoke::test_boot_to_shell                   PASSED
tests/smoke_boot.py::TestBootSmoke::test_smoke_test_can_actually_fail    PASSED
======================== 4 passed in 180.66s =========================
```

with `LITRIC_ACCEL=kvm LITRIC_BOOT_TIMEOUT=150`. The fourth case is the one that
matters for this bug: it boots a deliberately corrupted image and requires the
harness to **fail** it, so a change that made the prompt match anything would have
turned that case red.

## What is still not established

- No run under the harness's default `accel=tcg` has reached a prompt on this
  host. TCG reached `System Initialization` at 160 s and had not reached
  `multi-user.target` at 300 s, so the TCG result is "not reached inside any
  deadline tested", not "fails".
- The boot took ~75 s to a shell under KVM in a 2 vCPU / 2-3 GiB QEMU. The
  desktop profile's budget is 3.5 s (`docs/PERF-budget.md`). That budget is not
  met by this measurement and is not claimed to be; a QEMU boot is not the
  hardware the budget is about, and the discrepancy needs its own investigation.
