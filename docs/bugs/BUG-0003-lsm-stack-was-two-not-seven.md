# BUG-0003 — The LSM stack listed seven modules and enabled two

```
Bug ID        : BUG-0003
Severity      : S1 (CRITICAL) — the Phase 07 sandbox was substantially absent
                while the configuration read as though it were present
Class         : SEC (PROTOCOL §8)
Subsystem     : kernel/config/
First Observed: 2026-08-26, first run of ci/lint-kconfig.py
```

## Reproducer

```bash
python3 ci/lint-kconfig.py kernel/config/litric-base.config
```

## Expected

The seven Linux Security Modules named in `CONFIG_LSM` are compiled into the
kernel and active.

## Actual

```
CONFIG_LSM=""
CONFIG_LSM="landlock,lockdown,yama,loadpin,safesetid,integrity,bpf"
```

Two defects, one on top of the other:

1. **`CONFIG_LSM` was set twice, the first time to the empty string.**
   `scripts/kconfig/merge_config.sh` takes the last value and emits a warning
   into a build log nobody reads. The correct value happened to win — but a file
   containing an empty LSM stack two lines above the real one is a landmine, and
   any reordering, any tool that takes the first value, would silently disable
   every LSM on the system.

2. **Five of the seven named modules had no symbol enabled at all.**

   | Named in `CONFIG_LSM` | Symbol present? | Actually active? |
   |---|---|---|
   | landlock | `CONFIG_SECURITY_LANDLOCK=y` | yes |
   | bpf | `CONFIG_BPF_LSM=y` | yes |
   | lockdown | **missing** | **no** |
   | yama | **missing** | **no** |
   | loadpin | **missing** | **no** |
   | safesetid | **missing** | **no** |
   | integrity | **missing** | **no** |

   **The kernel silently ignores an unknown name in the LSM list.** No warning at
   boot, no error, no indication anywhere that five of the seven do not exist.

Two further defects in the same section:

3. `CONFIG_SECURITY_APPARMOR=m` — AppArmor is a `bool`. A bool cannot be a
   module; `make olddefconfig` drops the line, so the config claimed a
   confinement mechanism that was never built.
4. `CONFIG_SECURITY_BPF=y` — not a symbol that has ever existed upstream. The
   BPF LSM is `CONFIG_BPF_LSM`, which was separately (and correctly) set.

## Impact

The security posture of the system was **two LSMs while the configuration
documented seven**. Specifically absent:

- **lockdown** — without it, root can modify the running kernel: kexec an
  unsigned image, write `/dev/mem`, write raw MSRs, load unsigned modules. This
  makes the entire Secure Boot and dm-verity chain defeasible from userspace
  root. The Phase 03 property "a tampered rootfs fails closed" is undermined if
  root can simply load a module that lies about it.
- **yama** — `kernel.yama.ptrace_scope=1` was set in
  `usr/lib/sysctl.d/30-litric-security.conf`. Without the yama LSM that sysctl
  does not exist, so the write fails and cross-process ptrace stayed
  unrestricted. Two files agreed with each other and neither was in effect.
- **loadpin** — intended to pin module and firmware loading to the
  dm-verity-protected system partition. This is one of the strongest properties
  the LitricOS architecture makes available, and it was not on.
- **safesetid**, **integrity** — likewise absent.

Nothing was mis-*stated* to a user, because none of this had shipped. But had it
shipped, `docs/SEC-00-threatmodel.md` would have described mitigations that were
not running.

## Root Cause

The LSM list and the LSM symbols are two separate places that must agree, and
nothing checked that they did. The kernel's own tolerance of unknown LSM names —
correct behaviour, since it lets one config boot across kernel versions — removes
the last chance to notice.

Compounding it: **the fragment had never been merged into a kernel tree.** Even
`make olddefconfig` would have dropped the AppArmor line and revealed something
was wrong. Same root cause as BUG-0001 and BUG-0002: an artifact was treated as
correct because it existed, not because anything had processed it.

## Why Existing Tests Missed It

There were none. `ci/lint-kconfig.py` did not exist before this bug.

## Fix

1. `CONFIG_LSM=""` removed.
2. All five missing LSMs enabled with their real symbols, each with a comment
   saying what it buys — `CONFIG_SECURITY_YAMA`, `CONFIG_SECURITY_LOCKDOWN_LSM`
   (+`_EARLY`), `CONFIG_SECURITY_LOADPIN` (+`_ENFORCE`),
   `CONFIG_SECURITY_SAFESETID`, `CONFIG_INTEGRITY` (+`_SIGNATURE`,
   +`_ASYMMETRIC_KEYS`).
3. `CONFIG_SECURITY_APPARMOR` explicitly `is not set`, with the Phase 07
   rationale recorded inline rather than left as a silently-dropped `=m`.
4. `CONFIG_SECURITY_BPF` removed.
5. Five duplicate symbol declarations removed.
6. `CONFIG_DRM_CIRRUS_QEMU` removed (dropped from Linux in 5.9).
7. The `lsm=` command line in `image/mkosi.conf` reconciled with `CONFIG_LSM`.
   The command line **overrides** the compiled-in default, so a disagreement
   means the config is not what runs — and the config is what people read.

## Regression Test

`ci/lint-kconfig.py`, wired into `ci/verify.sh`. It fails on:
duplicate symbols (loudly when the values differ), an empty `CONFIG_LSM`, a name
in `CONFIG_LSM` whose symbol is absent, a `bool` set to `=m`, a symbol removed
from mainline, a required LitricOS symbol that is missing or disabled, and a
disagreement between `CONFIG_LSM` and the kernel command line.

`--self-test` proves the linter detects each of these, so the test can fail.
Verified: 5/5 detected.

## Verification

- `ci/lint-kconfig.py`: **clean** (2026-08-26).
- `ci/lint-kconfig.py --self-test`: **5/5 defect classes detected**.
- **NOT RUN:** the fragment has still never been merged into a real kernel tree.
  `merge_config.sh` plus `make olddefconfig` against actual kernel sources is the
  only thing that validates a symbol truly exists and that its dependencies are
  satisfiable. The linter checks a curated list; it is not a Kconfig parser.

## Residual Risk

**Moderate and specific.** The linter validates what it knows about. A symbol
that exists but whose *dependencies* are unmet will still be silently dropped by
`olddefconfig` — for example `CONFIG_SECURITY_LOCKDOWN_LSM` requires
`CONFIG_SECURITY=y`, which is set, but a future edit could break that without
this linter noticing. The real check is a kernel build, and it has not happened.

Until then, no claim may be made about which LSMs LitricOS runs. The correct
statement is: "the configuration requests seven; none has been verified against a
compiled kernel."
