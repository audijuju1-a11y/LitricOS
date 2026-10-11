# BUG-0005 — The boot smoke test could not time out, could pass on a broken boot, and looked for artifacts that no longer exist

```
Bug ID        : BUG-0005
Severity      : S1 (CRITICAL) — the only test standing between "we built an image"
                and "the image boots" reported a wedged guest as a hang to be
                investigated and a failed boot as PASS
Class         : TEST (PROTOCOL §8: false-negative and false-positive test)
Subsystem     : tests/
First Observed: 2026-10-09, code-only audit (no build, no VM, no GUI)
Supersedes    : BUG-0002 — same file, same defect class, different mechanisms
```

## Reproducer

Nothing in this reproducer builds, boots or launches anything. QEMU is replaced
by a stub program that prints a serial transcript and then behaves as instructed,
so the reader's behaviour is observable without a guest:

```python
import sys, time
sys.argv[1]                 # scenario name, injected by the test harness
print("[    0.000000] Linux version 7.1.10-arch1-1 (linux@archlinux) #1 SMP")
print("==> ERROR: Failed to mount root, dropping to emergency shell")
time.sleep(600)
```

`tests/test_harness_integrity.py` drives exactly this through the real
`QemuRunner`; the four defects below were each measured against the unmodified
file before any repair was made.

## Expected

`read_output(timeout=4)` returns within 4 s with a status that says what
happened, and the test's verdict is about the guest, not about the harness.

## Actual — four defects, all in one 329-line file

### 1. The timeout was advisory (guest hangs block the test forever)

`self.process.stdout.readline()` blocks until a line arrives; the deadline was
only checked *between* reads. A guest that goes silent — a hang, which is the
failure this test exists to detect and which this project has hit before — left
the reader blocked until QEMU itself exited.

Measured, stub guest that prints two lines and sleeps 600 s:

| Run | Deadline | Returned after | Verdict |
|---|---:|---:|---|
| stub sleeps 600 s, external 20 s killer | 4 s | 19.5 s (killed by the wrapper) | TIMEOUT, but only because something outside the test killed the child |
| stub sleeps 25 s | 4 s | 25.0 s | TIMEOUT — the deadline did not fire; the child's lifetime did |

CI consequence: instead of a failure at 60 s, the job blocks until the runner's
job-level timeout, and the reported reason is a hung build rather than a guest
that did not boot.

### 2. Success was detected by substring, including `"#"`, `">"` and `"shell"`

All three match ordinary early-userspace output. Measured against the
unmodified reader with the stub above:

```
status='SHELL'  elapsed=0.0s  detail='==> ERROR: Failed to mount root, dropping to emergency shell'
```

A boot that never reached userspace was reported as a booted system. The same
indicator set also contained `"iltri$"` and `"iltri-cod"`, strings that match
nothing a shell or a test can produce — evidence the list was written rather
than observed.

### 3. The artifact paths were stale, so every case skipped

| Constant | Pre-fix value | What the pipeline produces |
|---|---|---|
| `IMAGE` | build/litric.qcow2 | build/out/litricos.raw (`ImageId=litricos`, `OutputDirectory=../build/out`) |
| `OVMF_CODE` / `OVMF_VARS` | build/OVMF_CODE_4M.fd, build/OVMF_VARS_4M.fd | build/out/OVMF_CODE.4m.fd, build/out/OVMF_VARS.fd (or /usr/share/edk2/x64/…) |

`build/litric.qcow2` is the output name of the retired `tools/build-image.sh`.
`tests/smoke_boot.py` is the documented post-build step (`tools/build.sh` prints
it as the next command), so on a machine holding a freshly built image all four
cases `pytest.skip`. A gate that reports nothing looks like an environment
problem, and that is how a stale path survives: nothing distinguished it from a
missing prerequisite.

### 4. The test wrote to the host's firmware variable store

`unit=1` pflash was handed `OVMF_VARS` directly. QEMU opens that file
read-write and OVMF stores NVRAM in it, so a smoke test run mutated the
firmware's own variable store — and if the path had been the distribution's
`/usr/share/edk2/.../OVMF_VARS.fd`, it would have written outside the project
directory, which is precisely GATE 01 condition 7.

Every other launcher in this repository copies the template first:
`tools/boot-test.sh` copies it to `${LOG_DIR}/OVMF_VARS.run.fd`,
`tools/run-qemu-gui.sh` seeds `build/OVMF_VARS.gui.fd`, and
`tools/photo_tour_lib.py` copies it to `build/OVMF_VARS.tour.fd`. The smoke test
was the outlier.

## Impact

`docs/GATES.md` GATE 01 condition 3 — *"Smoke test passes, and fails on a
corrupted image"* — depends entirely on this file. Before this repair the
condition could not be met by running the test:

* a broken image could be recorded as PASS (defect 2),
* a wedged image blocked the job instead of failing it (defect 1),
* a good image skipped, because the paths named a pipeline that no longer
  existed (defect 3),
* and the run itself wrote to firmware NVRAM (defect 4).

## Root Cause

Same cause as BUG-0002, one level deeper. BUG-0002 was *a test that had never
been executed*. This is *a test that cannot be executed*: it had no seam through
which controlled behaviour could be supplied, so validating it required a built
image and a real guest, and the parts of it that only ever ran for real were
never exercised at all. Concretely, nothing in the file could answer "what does
the reader do when the guest stops talking?" without a guest that stops talking.

## Fix

1. A reader thread owns the pipe and pushes lines into a queue. The deadline is
   now a property of the read loop, so it fires whether or not the guest talks.
2. Success requires an anchored prompt pattern (getty `login:`, or a `user@host`
   PS1). Emergency-mode and panic text is classified as an explicit `FAILURE`
   with the reason in the detail, and warnings such as `Failed to mount` are
   collected and quoted rather than deciding the verdict.
3. Artifacts are discovered against the real output locations, with the legacy
   names kept as later fallbacks, and a skip lists every candidate it probed. A
   `.raw.zst` sitting next to the expected image is reported as such.
4. The variable-store template is copied to `build/OVMF_VARS.smoke.fd` before
   every boot, mirroring `tools/boot-test.sh`.
5. A documented seam: `LITRIC_QEMU`, `LITRIC_SMOKE_IMAGE`, `LITRIC_OVMF_CODE`,
   `LITRIC_OVMF_VARS`. Without it, this file is untestable by construction.
6. `stop()` falls back to `terminate()`/`kill()` where `os.killpg` does not
   exist, so a Windows host no longer leaks the QEMU process.
7. The corruption test now writes four spans instead of one, and requires a raw
   image: the first and last 8 KiB (both GPT headers — UEFI falls back to the
   backup copy, so the primary alone is not guaranteed to stop the boot), the
   ESP boot sector at 1 MiB for determinism, and the 64 MiB span that remains
   the dm-verity probe.

## Regression Test

`tests/test_harness_integrity.py` — 23 cases, all executed on a stub guest.
The four that pin this bug:

| Test | Property |
|---|---|
| `test_silent_guest_times_out_within_the_deadline` | a 2 s deadline returns within 8 s **and** the child is still running |
| `test_mute_guest_times_out_within_the_deadline` | a guest that writes nothing at all also returns |
| `test_initrd_failure_is_not_a_shell` | the exact line that used to return `SHELL` now returns `FAILURE` |
| `test_emergency_shell_word_alone_is_not_success` | `"shell"` alone is not success |
| `test_image_candidates_match_the_pipeline_output` | the first candidate is build/out/litricos.raw |
| `test_vars_template_is_copied_not_mutated` | the template is not the file handed to QEMU |

## Verification

Executed 2026-10-09, Windows host, Python 3.14.6, pytest 9.1.1. No build, no
VM, no GUI, no image; QEMU replaced by a stub program.

| Command | Result |
|---|---|
| `python -m pytest tests/test_harness_integrity.py -q` | **23 passed** |
| `python -m pytest tests/smoke_boot.py -q` | **4 skipped** with the candidate paths named (no image, no QEMU on this host) |
| `python tests/smoke_boot.py` | exit **77** (skipped), naming build/out/litricos.raw as the expected image |
| `bash -n tests/smoke_boot.py`-equivalent (`ast.parse`) | **PASS** |

## Residual Risk

**The behaviour of this file against a real guest is NOT VERIFIED.** Every
result above comes from a stub whose output was chosen by the test. The stub
proves the reader's logic, the deadline and the paths; it cannot prove that the
anchored patterns match what a real LitricOS console actually prints. In
particular the PS1 pattern (`root@host:~#`) is written from the project's
documented prompt, not observed, and the image this repository can currently
build has never been booted together with this file.

Therefore GATE 01 condition 3 is **not** satisfied by this change. It becomes
satisfiable: it now needs one run against a built image — which passes — and one
against the corrupted copy — which fails — on a Linux build host.

The corruption offsets are likewise unverified against a real image. They are
derived from `image/mkosi.conf` (`SectorSize=512`), `image/mkosi.repart/00-esp.conf`
(ESP, 512 MiB) and the mkosi default 1 MiB partition alignment. If a future
layout moves the ESP, the deterministic corruption lands elsewhere and the test
can pass for the wrong reason; the assertion is still sound in the direction it
matters (a corrupted image must not reach a shell), and the deep 64 MiB span
keeps the dm-verity property as long as the root partition covers it.
