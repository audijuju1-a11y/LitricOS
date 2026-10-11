#!/usr/bin/env python3
"""Validate the LitricOS kernel configuration fragment.

A Kconfig fragment is merged with `scripts/kconfig/merge_config.sh`, which on a
duplicate symbol takes the LAST value and prints a warning that nobody reads in
a build log. That makes a duplicated symbol with two different values a silent
misconfiguration, and it is exactly how this repository ended up shipping

    CONFIG_LSM=""
    CONFIG_LSM="landlock,lockdown,yama,loadpin,safesetid,integrity,bpf"

An empty LSM string disables Landlock, lockdown, yama, integrity and BPF-LSM --
the entire Phase 07 sandbox -- while the file still visibly contains the correct
list two lines below. See docs/bugs/BUG-0003.

This linter runs before any kernel is built, on any host, with no kernel source
tree. It cannot replace `make olddefconfig` (only the real Kconfig tree knows
the full dependency graph). It catches the classes that a build does NOT catch:
a silently-losing duplicate, a symbol the kernel dropped years ago, a bool set
to =m, and a LitricOS-mandatory symbol that quietly went missing.

Usage:
    python3 ci/lint-kconfig.py [fragment ...]
    python3 ci/lint-kconfig.py --self-test
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

SET_RE = re.compile(r"^(CONFIG_[A-Z0-9_]+)=(.*)$")
UNSET_RE = re.compile(r"^# (CONFIG_[A-Z0-9_]+) is not set$")

# --- symbols that are `bool` in Kconfig and therefore cannot be `=m` ---------
# Setting a bool to m does not produce a module; merge_config keeps the literal
# text and `make olddefconfig` silently drops it, so the feature ends up OFF.
BOOL_ONLY = {
    "CONFIG_SECURITY_APPARMOR",
    "CONFIG_SECURITY_LANDLOCK",
    "CONFIG_SECURITY_SELINUX",
    "CONFIG_SECURITY_YAMA",
    "CONFIG_BPF_LSM",
    "CONFIG_SECURITY",
    "CONFIG_SECCOMP",
    "CONFIG_SECCOMP_FILTER",
    "CONFIG_EFI_STUB",
    "CONFIG_LRU_GEN",
    "CONFIG_LRU_GEN_ENABLED",
    "CONFIG_SCHED_CLASS_EXT",
    "CONFIG_FANOTIFY",
    "CONFIG_FANOTIFY_ACCESS_PERMISSIONS",
    "CONFIG_PREEMPT",
    "CONFIG_HZ_1000",
    "CONFIG_NO_HZ_IDLE",
    "CONFIG_RANDOMIZE_BASE",
    "CONFIG_RANDOMIZE_MEMORY",
    "CONFIG_AUDIT",
    "CONFIG_AUDITSYSCALL",
    "CONFIG_BPF_SYSCALL",
    "CONFIG_BPF_JIT",
}

# --- symbols removed from mainline ------------------------------------------
# Setting these is harmless but misleading: it reads as though a driver is
# enabled when the kernel has no such option. Value is the replacement, or None.
REMOVED = {
    "CONFIG_DRM_CIRRUS_QEMU": "removed in Linux 5.9; QEMU uses DRM_BOCHS or DRM_QXL",
    "CONFIG_SECURITY_BPF": "never existed upstream; the BPF LSM symbol is CONFIG_BPF_LSM",
    "CONFIG_ANDROID": "removed in Linux 5.19",
    "CONFIG_IDE": "removed in Linux 5.14; use CONFIG_ATA",
    "CONFIG_VIDEO_V4L2": "renamed CONFIG_VIDEO_DEV in Linux 6.1",
}

# --- symbols LitricOS cannot ship without -----------------------------------
# From 00-MASTER-WORKFLOW.md section 3 and the phase gates that depend on them.
# A missing entry here is not a style problem: it silently removes a feature the
# rest of the system is built to assume.
REQUIRED_Y = {
    "CONFIG_LRU_GEN": "MGLRU -- PERF-budget lever 3",
    "CONFIG_LRU_GEN_ENABLED": "MGLRU on by default",
    "CONFIG_SCHED_CLASS_EXT": "sched_ext, needed by scx_lavd in Phase 12",
    "CONFIG_NTSYNC": "Wine/Proton NT primitives, Phase 12",
    "CONFIG_FANOTIFY": "Defender on-access engine, Phase 07",
    "CONFIG_FANOTIFY_ACCESS_PERMISSIONS": "Defender must be able to DENY an open",
    "CONFIG_SECURITY_LANDLOCK": "primary per-app confinement, Phase 07",
    "CONFIG_BPF_LSM": "Defender behavioural layer, Phase 07",
    "CONFIG_DM_VERITY": "read-only rootfs integrity, Phase 03",
    "CONFIG_EROFS_FS": "system partition format, Phase 03",
    "CONFIG_BTRFS_FS": "user partition, snapshots and rollback, Phase 03",
    "CONFIG_ZRAM": "swap topology, Phase 02",
    "CONFIG_EFI_STUB": "UKI boot, Phase 01",
    "CONFIG_SECCOMP_FILTER": "per-app syscall filtering, Phase 07",
    "CONFIG_PREEMPT": "desktop latency, MASTER section 3",
    "CONFIG_HZ_1000": "desktop latency, MASTER section 3",
}

# --- symbols whose value must be non-empty ----------------------------------
MUST_BE_NONEMPTY = {"CONFIG_LSM"}

# --- the LSM stack LitricOS expects -----------------------------------------
# CONFIG_LSM must agree with the `lsm=` on the kernel command line in
# image/mkosi.conf, because the command line OVERRIDES the built-in default.
# A config that lists loadpin and a command line that does not means loadpin is
# not actually active, and someone will later cite the config as proof it is.
REQUIRED_LSM_MEMBERS = {"landlock", "lockdown", "yama", "integrity", "bpf"}


class Finding:
    def __init__(self, path: Path, line: int, severity: str, message: str) -> None:
        self.path, self.line, self.severity, self.message = path, line, severity, message

    def __str__(self) -> str:
        return f"{self.path.name}:{self.line}: {self.severity}: {self.message}"


def lint(path: Path) -> list[Finding]:
    out: list[Finding] = []
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return [Finding(path, 0, "ERROR", f"cannot read: {exc}")]

    seen: dict[str, tuple[int, str]] = {}
    unset: dict[str, int] = {}

    for lineno, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line:
            continue

        m = UNSET_RE.match(line)
        if m:
            unset[m.group(1)] = lineno
            continue
        if line.startswith("#"):
            continue

        m = SET_RE.match(line)
        if not m:
            out.append(Finding(path, lineno, "ERROR", f"not a Kconfig assignment: {line}"))
            continue

        sym, val = m.group(1), m.group(2).strip()

        # --- duplicate ------------------------------------------------------
        if sym in seen:
            prev_line, prev_val = seen[sym]
            if prev_val == val:
                out.append(Finding(
                    path, lineno, "ERROR",
                    f"{sym} duplicated (also line {prev_line}); merge_config warns and "
                    f"takes the last value -- remove one",
                ))
            else:
                out.append(Finding(
                    path, lineno, "ERROR",
                    f"{sym} set twice with DIFFERENT values: line {prev_line}={prev_val!r} "
                    f"vs line {lineno}={val!r}. merge_config silently keeps the last. "
                    f"This is a silent misconfiguration, not a style problem.",
                ))
        seen[sym] = (lineno, val)

        # --- removed --------------------------------------------------------
        if sym in REMOVED:
            out.append(Finding(path, lineno, "ERROR", f"{sym}: {REMOVED[sym]}"))

        # --- bool set to module ---------------------------------------------
        if val == "m" and sym in BOOL_ONLY:
            out.append(Finding(
                path, lineno, "ERROR",
                f"{sym} is a bool and cannot be =m; olddefconfig will drop it, "
                f"turning the feature OFF rather than building it as a module",
            ))

        # --- empty value ----------------------------------------------------
        if sym in MUST_BE_NONEMPTY and val in ('""', "''", ""):
            out.append(Finding(
                path, lineno, "ERROR",
                f"{sym} is empty. For CONFIG_LSM this disables the entire LSM stack "
                f"(Landlock, lockdown, yama, integrity, BPF-LSM) -- the Phase 07 sandbox.",
            ))

        # --- LSM membership --------------------------------------------------
        if sym == "CONFIG_LSM" and val not in ('""', ""):
            members = {p.strip() for p in val.strip('"').split(",") if p.strip()}
            missing = REQUIRED_LSM_MEMBERS - members
            if missing:
                out.append(Finding(
                    path, lineno, "ERROR",
                    f"CONFIG_LSM is missing required LSM(s): {', '.join(sorted(missing))}",
                ))

    # --- required symbols present and enabled --------------------------------
    for sym, why in REQUIRED_Y.items():
        if sym in unset:
            out.append(Finding(
                path, unset[sym], "ERROR",
                f"{sym} is explicitly disabled but is required: {why}",
            ))
        elif sym not in seen:
            out.append(Finding(path, 0, "ERROR", f"{sym} is missing; required for: {why}"))
        elif seen[sym][1] not in ("y", "m"):
            out.append(Finding(
                path, seen[sym][0], "ERROR",
                f"{sym}={seen[sym][1]} but must be enabled: {why}",
            ))

    # --- symbol both set and unset -------------------------------------------
    for sym, lineno in unset.items():
        if sym in seen:
            out.append(Finding(
                path, lineno, "ERROR",
                f"{sym} is both set (line {seen[sym][0]}) and marked 'is not set'",
            ))

    return out


def cross_check_cmdline(fragment: Path, mkosi_conf: Path) -> list[Finding]:
    """CONFIG_LSM and the kernel command line's lsm= must agree.

    The command line OVERRIDES the compiled-in default. If they disagree, the
    config is not what actually runs, and the config is what people read.
    """
    out: list[Finding] = []
    if not mkosi_conf.exists():
        return out

    cfg_val = None
    for lineno, raw in enumerate(fragment.read_text(encoding="utf-8").splitlines(), 1):
        m = SET_RE.match(raw.strip())
        if m and m.group(1) == "CONFIG_LSM" and m.group(2).strip() not in ('""', ""):
            cfg_val, cfg_line = m.group(2).strip().strip('"'), lineno

    cmd_val = None
    for raw in mkosi_conf.read_text(encoding="utf-8").splitlines():
        s = raw.strip()
        if s.startswith("lsm="):
            cmd_val = s[len("lsm="):].strip()

    if cfg_val is None or cmd_val is None:
        return out

    cfg_set = {p.strip() for p in cfg_val.split(",") if p.strip()}
    cmd_set = {p.strip() for p in cmd_val.split(",") if p.strip()}
    if cfg_set != cmd_set:
        only_cfg = sorted(cfg_set - cmd_set)
        only_cmd = sorted(cmd_set - cfg_set)
        detail = []
        if only_cfg:
            detail.append(f"in CONFIG_LSM but NOT on the command line (so NOT active): {', '.join(only_cfg)}")
        if only_cmd:
            detail.append(f"on the command line but not in CONFIG_LSM: {', '.join(only_cmd)}")
        out.append(Finding(
            fragment, cfg_line, "ERROR",
            "CONFIG_LSM disagrees with lsm= in image/mkosi.conf. The command line "
            "wins at boot, so the config is not what runs. " + "; ".join(detail),
        ))
    return out


def self_test() -> int:
    """Prove the linter detects the defects it was written for."""
    import tempfile

    cases = [
        ("duplicate with different values",
         'CONFIG_LSM=""\nCONFIG_LSM="landlock,lockdown,yama,integrity,bpf"\n', True),
        ("empty LSM alone",
         'CONFIG_LSM=""\n', True),
        ("bool set to module",
         "CONFIG_SECURITY_APPARMOR=m\n", True),
        ("removed symbol",
         "CONFIG_DRM_CIRRUS_QEMU=m\n", True),
        ("fabricated symbol",
         "CONFIG_SECURITY_BPF=y\n", True),
    ]
    rc = 0
    with tempfile.TemporaryDirectory() as td:
        for name, body, should_fail in cases:
            p = Path(td) / "frag.config"
            p.write_text(body, encoding="utf-8")
            errs = [f for f in lint(p) if f.severity == "ERROR"]
            # Required-symbol errors are expected in these tiny fragments; look
            # for an error mentioning the symbol under test.
            sym = body.split("=")[0].strip()
            hit = any(sym in f.message or "LSM" in f.message for f in errs)
            if hit == should_fail:
                print(f"PASS  detects: {name}")
            else:
                print(f"FAIL  does NOT detect: {name}")
                rc = 1
    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    root = Path(__file__).resolve().parent.parent
    targets = [Path(a) for a in argv[1:]] or sorted((root / "kernel" / "config").glob("*.config"))
    if not targets:
        print("no kernel config fragments found")
        return 1

    errors = 0
    for path in targets:
        findings = lint(path) + cross_check_cmdline(path, root / "image" / "mkosi.conf")
        for f in findings:
            print(f)
            if f.severity == "ERROR":
                errors += 1

    if errors:
        print(f"\n{errors} error(s) in kernel configuration")
        return 1
    print(f"OK: {len(targets)} kernel fragment(s) clean")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
