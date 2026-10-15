#!/usr/bin/env python3
"""Validate mkosi configuration against the real mkosi schema.

This is the regression test for BUG-0001, in which a configuration file was
written from recollection of what an image builder's schema "should" look like,
using sections that do not exist. It failed at parse time and nothing it
described was ever built -- yet a gate was recorded as passed on its existence.

The test must FAIL against the pre-v2 file preserved in .backup-pre-v2/ and PASS
against the current one. A regression test that cannot fail proves nothing
(PROTOCOL section 32).

Two classes of defect are caught:

  1. A section name that mkosi does not define.
  2. A key placed in the wrong section. This is the subtler and more dangerous
     one: mkosi rejects unknown keys, so a key in the wrong section is a build
     failure, but reviewers routinely miss it because the key name is real.

Schema source: mkosi 25.3 documentation. Pinned by MinimumVersion= in
image/mkosi.conf; when that pin moves, this table moves with it.

Usage:
    python3 ci/lint-mkosi.py [config.conf ...]
    python3 ci/lint-mkosi.py --self-test
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

# --- the real mkosi schema -------------------------------------------------
# key -> section it belongs in. Only keys LitricOS actually uses are listed;
# an unknown key is reported as unverified rather than as an error, because
# this table is a subset by design and must not block legitimate configuration.
SCHEMA: dict[str, set[str]] = {
    "Config": {
        "Profiles", "Dependencies", "MinimumVersion", "PassEnvironment",
        "Include", "Images",
    },
    "Distribution": {
        "Distribution", "Release", "Architecture", "Mirror", "LocalMirror",
        "RepositoryKeyCheck", "RepositoryKeyFetch", "Repositories",
    },
    "Output": {
        "Format", "ManifestFormat", "Output", "CompressOutput", "CompressLevel",
        "OutputDirectory", "WorkspaceDirectory", "ImageId", "ImageVersion",
        "SplitArtifacts", "RepartDirectories", "SectorSize", "Overlay",
        "UseSubvolumes", "Seed",
    },
    "Content": {
        "Packages", "BuildPackages", "VolatilePackages", "PackageDirectories",
        "VolatilePackageDirectories", "WithRecommends", "WithDocs", "BaseTrees",
        "SkeletonTrees", "ExtraTrees", "RemovePackages", "RemoveFiles",
        "CleanPackageMetadata", "SyncScripts", "PrepareScripts", "BuildScripts",
        "PostInstallationScripts", "FinalizeScripts", "PostOutputScripts",
        "BuildSources", "BuildSourcesEphemeral", "Environment",
        "EnvironmentFiles", "WithTests", "WithNetwork", "Bootable", "Bootloader",
        "BiosBootloader", "ShimBootloader", "UnifiedKernelImages",
        "UnifiedKernelImageFormat", "Initrds", "InitrdPackages", "MicrocodeHost",
        "KernelCommandLine", "KernelModulesInclude", "KernelModulesExclude",
        "KernelModulesInitrdInclude", "KernelModulesInitrdExclude", "Locale",
        "LocaleMessages", "Keymap", "Timezone", "Hostname", "RootPassword",
        "RootShell", "Autologin", "MakeInitrd", "SELinuxRelabel",
    },
    "Validation": {
        "SecureBoot", "SecureBootAutoEnroll", "SecureBootKey",
        "SecureBootCertificate", "SecureBootSignTool", "SecureBootKeySource",
        "VerityKey", "VerityCertificate", "VerityKeySource", "Verity",
        "SignExpectedPcr", "SignExpectedPcrKey", "SignExpectedPcrCertificate",
        "SignExpectedPcrKeySource", "SignExpectedPcrCertificateSource",
        "Passphrase", "Checksum", "Sign", "Key",
    },
    # Verified against mkosi 26, which rejects CacheDirectory in [Output] with
    # "should be configured in [Build]". This table previously had the directory
    # keys in [Output]; the linter therefore passed a configuration that mkosi
    # refused, which is a false negative -- the worst kind of check.
    "Build": {
        "Environment", "EnvironmentFiles", "BuildSources", "BuildDirectory",
        "CacheDirectory", "PackageCacheDirectory", "WithTests", "WithNetwork",
        "Incremental",
    },
    "Runtime": {
        "RuntimeTrees", "RuntimeSize", "RuntimeScratch", "RuntimeNetwork",
        "RuntimeBuildSources", "VirtualMachineMonitor", "VMM", "Console",
        "CPUs", "RAM", "KVM", "VSock", "VSockConnectionId", "TPM", "CDROM",
        "Firmware", "FirmwareVariables", "Linux", "Drives", "QemuArgs",
    },
    "Host": set(),   # deprecated alias for Runtime; accepted, warned about
    "Match": set(),  # any key, used in drop-ins
    "Include": set(),
}

VALID_SECTIONS = set(SCHEMA)

# Keys whose canonical home differs from where people habitually put them.
# Reported with the correct section so the fix is obvious.
KEY_HOME = {key: section for section, keys in SCHEMA.items() for key in keys}


class Finding:
    def __init__(self, path: Path, line: int, severity: str, message: str) -> None:
        self.path, self.line, self.severity, self.message = path, line, severity, message

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.severity}: {self.message}"


# Keys whose value is an enum, with the values mkosi 26 actually accepts.
#
# A key name being valid says nothing about its value. `UnifiedKernelImages=yes`
# passed every check this linter had, and mkosi coerced it to `signed`, which
# means the opposite of what was intended and failed the build twenty minutes in.
# A boolean silently accepted where an enum was expected is the same defect class
# as a key in the wrong section: real name, wrong meaning.
ENUM_VALUES = {
    "UnifiedKernelImages": {"none", "auto", "signed", "unsigned"},
    "Verity": {"disabled", "hash", "signed", "defer", "auto"},
    "Bootable": {"yes", "no", "auto"},
    "Bootloader": {"none", "systemd-boot", "uki", "grub", "systemd-boot-signed", "uki-signed", "grub-signed"},
    "BiosBootloader": {"none", "grub"},
    "ShimBootloader": {"none", "unsigned", "signed"},
    "Format": {"directory", "tar", "cpio", "disk", "uki", "esp", "oci", "sysext",
               "confext", "portable", "addon", "none"},
    "SecureBootAutoEnroll": {"yes", "no"},
    "SignExpectedPcr": {"yes", "no", "auto"},
    "CompressOutput": {"yes", "no", "none", "zstd", "zst", "xz", "bz2", "gz", "lz4", "lzma"},
}

# Keys whose value names a path that must exist. A declared-but-absent tree is
# a build failure that no amount of schema checking catches, because the key and
# the section are both perfectly valid. Found in review of our own v2 config:
# SkeletonTrees= named a directory that was never created.
PATH_KEYS = {
    "SkeletonTrees", "ExtraTrees", "BaseTrees", "PackageDirectories",
    "SecureBootKey", "SecureBootCertificate", "VerityKey", "VerityCertificate",
    "SignExpectedPcrKey", "SignExpectedPcrCertificate",
}

# These six signing artifacts are intentionally generated locally and excluded
# from Git. A fully clean checkout may omit the whole set; a partial set must
# still be treated as an error because it cannot produce a signed image.
DEV_KEY_FILES = (
    "secureboot.key", "secureboot.crt",
    "verity.key", "verity.crt",
    "pcr.key", "pcr.crt",
)


# mkosi discovers these automatically when they sit next to mkosi.conf.
# Declaring them as well applies them twice.
AUTO_DISCOVERED = {"mkosi.extra", "mkosi.skeleton", "mkosi.repart", "mkosi.profiles"}


def is_absent_dev_key(path: Path, target: str, resolved: Path) -> bool:
    """Whether a missing reference is one of the intentionally absent dev keys.

    This only suppresses the error when *all six* local-only signing artifacts
    are absent. Once any key exists, missing companions must fail the check.
    """
    if not target.startswith("../keys/dev/") or resolved.name not in DEV_KEY_FILES:
        return False
    config_root = path.resolve().parent.parent
    dev_dir = (config_root / "keys" / "dev").resolve()
    if resolved.parent != dev_dir:
        return False
    return not any((dev_dir / name).exists() for name in DEV_KEY_FILES)


def check_path_value(path: Path, lineno: int, key: str, value: str) -> list[Finding]:
    out: list[Finding] = []
    for item in value.replace(",", " ").split():
        target = item.split(":", 1)[0]
        if not target or target.startswith("/"):
            continue
        base = target.rstrip("/").split("/")[-1]
        if base in AUTO_DISCOVERED:
            out.append(Finding(
                path, lineno, "ERROR",
                f"{key}= names {target!r}, which mkosi discovers automatically; "
                f"declaring it applies it twice",
            ))
            continue
        resolved = (path.parent / target).resolve()
        if not resolved.exists():
            if is_absent_dev_key(path, target, resolved):
                continue
            out.append(Finding(
                path, lineno, "ERROR",
                f"{key}= names {target!r}, which does not exist at {resolved}",
            ))
    return out


def lint(path: Path) -> list[Finding]:
    findings: list[Finding] = []
    section: str | None = None

    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return [Finding(path, 0, "ERROR", f"cannot read: {exc}")]

    for lineno, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith(";"):
            continue

        if line.startswith("["):
            if not line.endswith("]"):
                findings.append(Finding(path, lineno, "ERROR", f"malformed section header: {line}"))
                continue
            section = line[1:-1].strip()
            # Match sections in drop-ins look like [Match] or [Distribution]
            if section not in VALID_SECTIONS:
                findings.append(
                    Finding(
                        path, lineno, "ERROR",
                        f"[{section}] is not an mkosi section. "
                        f"Valid: {', '.join(sorted(VALID_SECTIONS))}. "
                        f"Partition geometry belongs in mkosi.repart/*.conf, not here.",
                    )
                )
            elif section == "Host":
                findings.append(
                    Finding(path, lineno, "WARN", "[Host] is deprecated; use [Runtime]")
                )
            continue

        # Continuation lines (indented values) belong to the previous key.
        if raw[:1] in (" ", "\t"):
            continue

        if "=" not in line:
            findings.append(Finding(path, lineno, "ERROR", f"not a key=value line: {line}"))
            continue

        key = line.split("=", 1)[0].strip()
        if section is None:
            findings.append(Finding(path, lineno, "ERROR", f"key {key!r} before any section"))
            continue
        if section not in VALID_SECTIONS or section in ("Match", "Include"):
            continue

        value = line.split("=", 1)[1].strip()
        if key in PATH_KEYS and value:
            findings.extend(check_path_value(path, lineno, key, value))

        if key in ENUM_VALUES and value and value not in ENUM_VALUES[key]:
            findings.append(Finding(
                path, lineno, "ERROR",
                f"{key}={value!r} is not a valid value. "
                f"Accepted: {', '.join(sorted(ENUM_VALUES[key]))}. "
                f"mkosi may silently coerce an unexpected value rather than reject it.",
            ))

        home = KEY_HOME.get(key)
        if home is None:
            findings.append(
                Finding(path, lineno, "INFO", f"key {key!r} not in the pinned schema subset (unverified)")
            )
        elif key not in SCHEMA[section]:
            findings.append(
                Finding(
                    path, lineno, "ERROR",
                    f"key {key!r} is not valid in [{section}]; it belongs in [{home}]",
                )
            )

    return findings


def self_test() -> int:
    """Prove the linter fails on the known-bad file and passes on the fixed one."""
    root = Path(__file__).resolve().parent.parent
    bad = root / ".backup-pre-v2" / "image" / "mkosi.conf"
    good = root / "image" / "mkosi.conf"

    rc = 0
    if bad.exists():
        errs = [f for f in lint(bad) if f.severity == "ERROR"]
        if errs:
            print(f"PASS  regression: pre-v2 config rejected with {len(errs)} error(s)")
            for f in errs[:6]:
                print(f"        {f.message[:110]}")
        else:
            print("FAIL  regression: pre-v2 config was NOT rejected -- the linter cannot fail,")
            print("      so it proves nothing (PROTOCOL section 32).")
            rc = 1
    else:
        print(f"SKIP  regression: {bad} not present")

    errs = [f for f in lint(good) if f.severity == "ERROR"]
    if errs:
        print(f"FAIL  current config has {len(errs)} error(s):")
        for f in errs:
            print(f"        {f}")
        rc = 1
    else:
        print("PASS  current config uses only real sections and correct key placement")
    # Clean checkouts intentionally omit local signing keys, but a partial
    # set must fail. Test this invariant with a temporary repository fixture.
    try:
        with tempfile.TemporaryDirectory(prefix="litric-mkosi-self-test-") as tmp:
            fixture = Path(tmp)
            config = fixture / "image" / "mkosi.conf"
            config.parent.mkdir(parents=True)
            dev_dir = fixture / "keys" / "dev"
            dev_dir.mkdir(parents=True)
            absent = check_path_value(config, 1, "SecureBootKey", "../keys/dev/secureboot.key")
            if absent:
                print("FAIL: clean checkout with all dev keys absent was rejected")
                rc = 1
            else:
                print("PASS  clean checkout with all dev keys absent is accepted")
            (dev_dir / "secureboot.key").write_text("test fixture", encoding="utf-8")
            partial = check_path_value(
                config, 1, "SecureBootCertificate", "../keys/dev/secureboot.crt"
            )
            if any(f.severity == "ERROR" for f in partial):
                print("PASS  partial dev key set remains an error")
            else:
                print("FAIL: partial dev key set was incorrectly accepted")
                rc = 1
    except OSError as exc:
        print(f"FAIL: could not create self-test fixture: {exc}")
        rc = 1

    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    root = Path(__file__).resolve().parent.parent
    targets = [Path(a) for a in argv[1:]] or sorted(
        list((root / "image").glob("*.conf"))
        + list((root / "image" / "mkosi.profiles").glob("*.conf"))
        + list((root / "image" / "mkosi.repart").glob("*.conf"))
    )

    errors = 0
    for path in targets:
        # repart files use systemd-repart syntax, not mkosi syntax.
        if "mkosi.repart" in path.parts:
            continue
        for f in lint(path):
            print(f)
            if f.severity == "ERROR":
                errors += 1

    if errors:
        print(f"\n{errors} error(s). See docs/bugs/BUG-0001-fabricated-mkosi-config.md")
        return 1
    print(f"OK: {len([t for t in targets if 'mkosi.repart' not in t.parts])} file(s) clean")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
