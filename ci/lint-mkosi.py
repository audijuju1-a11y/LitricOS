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
        "KernelCommandLine", "KernelModules", "KernelInitrdModules",
        # The mkosi 26 spellings live in LEGACY_MODULE_KEYS, not here: they parse,
        # but they parse as regular expressions, and a module list written in that
        # grammar silently matches nothing (BUG-0020). Accepted by mkosi, rejected
        # by this linter, which is the whole point of the table.
        "Locale",
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
    #
    # The image builds with mkosi 27 as of 2026-10-16, and the [Content] table
    # above was re-checked against it: `KernelInitrdModules` added, the mkosi 26
    # `KernelModulesInitrdInclude` kept because mkosi 27 still parses it.
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

# Module settings whose mkosi 26 spelling is a silent trap.
#
# mkosi 27 still parses these, but through the *legacy* code path, which treats
# each entry as a regular expression rather than a glob:
#
#     KernelModulesInclude=virtio_*    ->  re:virtio_*   = "virtio" + "_"*
#     KernelModulesExclude=.*          ->  re:.*         = everything
#
# The first matches no real module, the second excludes every remaining one, and
# the result is an image with an empty /usr/lib/modules. mkosi builds its extra
# initrd by copying from that directory, so the initrd is empty too and a
# dm-verity root cannot be mounted. `mkosi summary` will tell you -- it prints
# `re:` in front of exactly these values -- but nothing in the build does.
LEGACY_MODULE_KEYS = {
    "KernelModulesInclude": "KernelModules",
    "KernelModulesExclude": "KernelModules (write the pattern as -*)",
    "KernelModulesInitrdInclude": "KernelInitrdModules",
    "KernelModulesInitrdExclude": "KernelInitrdModules (write the pattern as -*)",
}

# Every boot needs these before /sysroot can be mounted: the root's own driver
# and the device mapper the root sits on. Named as (label, predicate) so the
# message can say which one is absent.
BOOT_CRITICAL_MODULES = (
    ("erofs", lambda p: p == "erofs"),
    ("dm-*", lambda p: p.startswith("dm-") or p.startswith("dm_")),
)

# mkosi discovers these automatically when they sit next to mkosi.conf.
# Declaring them as well applies them twice.
AUTO_DISCOVERED = {"mkosi.extra", "mkosi.skeleton", "mkosi.repart", "mkosi.profiles"}


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


def read_list_setting(path: Path, keys: set[str]) -> dict[str, list[str]]:
    """Collect `key=` plus its indented continuation lines, per key.

    Values are returned as written: stripping trailing comments is deliberately
    NOT done, because mkosi does not strip them either, so a stray `# note`
    becomes a glob -- and hiding that here would hide a real defect.
    """
    values: dict[str, list[str]] = {}
    current: str | None = None
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return values
    for raw in lines:
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith(";"):
            continue
        if raw[:1] in (" ", "\t"):
            if current is not None:
                values[current].append(line)
            continue
        if "=" not in line:
            current = None
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        if key in keys:
            current = key
            values.setdefault(key, [])
            if value.strip():
                values[key].append(value.strip())
        else:
            current = None
    return values


# Patterns that, used as an exclusion, remove every module there is.
MATCHES_EVERYTHING = {".*", "*", "/.*", "/**"}


def check_module_sets(paths: list[Path]) -> list[Finding]:
    """The module lists must be consistent, and must cover the boot.

    Four defects, each of which produced a bootable-looking image that could not
    boot, and none of which any schema check can see:

      1. a legacy key (see LEGACY_MODULE_KEYS) -- accepted by mkosi, matches
         nothing;
      2. an initrd list without the root's driver or the device mapper, which
         only shows up as "Failed to mount /sysroot" ninety seconds into a boot;
      3. an initrd list asking for a module the image's whitelist drops -- the
         extra initrd is copied out of the image's /usr/lib/modules, so the name
         is unreachable even though both lists look right in isolation;
      4. a whitelist next to an exclude-everything pattern. mkosi's rule is that
         the exclude branch wins and an include list is a whitelist inside it, so
         `-.*` plus `virtio*` keeps nothing -- and says nothing.
    """
    findings: list[Finding] = []
    image_patterns: list[str] = []
    initrd_modules: list[str] = []

    for path in paths:
        legacy = read_list_setting(path, set(LEGACY_MODULE_KEYS))
        for key in legacy:
            findings.append(Finding(
                path, 0, "ERROR",
                f"{key}= uses the mkosi 26 spelling, which mkosi 27 parses as a "
                f"regular expression, not a glob: `virtio_*` matches no module and "
                f"`.*` excludes every one. Use {LEGACY_MODULE_KEYS[key]}. "
                f"See docs/bugs/BUG-0020.",
            ))

        lists = read_list_setting(path, {"KernelModules", "KernelInitrdModules"})
        image_patterns.extend(lists.get("KernelModules", []))
        initrd_modules.extend(lists.get("KernelInitrdModules", []))

    if not initrd_modules:
        findings.append(Finding(
            paths[0] if paths else Path("image/mkosi.conf"), 0, "ERROR",
            "no KernelInitrdModules= list: the extra initrd gets no modules, so a "
            "root on erofs behind dm-verity cannot be mounted",
        ))
        return findings

    include = [p for p in image_patterns if not p.startswith("-")]
    exclude = [p.lstrip("-") for p in image_patterns if p.startswith("-")]

    for pattern in exclude:
        if pattern in MATCHES_EVERYTHING and include:
            findings.append(Finding(
                paths[0] if paths else Path("image/mkosi.conf"), 0, "ERROR",
                f"KernelModules= carries -{pattern} next to {len(include)} "
                f"include pattern(s). mkosi evaluates the exclude branch first and "
                f"the include list only decides *within* what survives it, so this "
                f"keeps no modules at all: an empty /usr/lib/modules, an empty "
                f"initrd, and a root that cannot be mounted. Drop the exclusion, or "
                f"drop the include list and exclude individual patterns.",
            ))

    for label, predicate in BOOT_CRITICAL_MODULES:
        if not any(predicate(p) for p in initrd_modules):
            findings.append(Finding(
                paths[0] if paths else Path("image/mkosi.conf"), 0, "ERROR",
                f"the initrd list has no {label!r} pattern. It is a module, not "
                f"built in, and the boot cannot mount /sysroot without it.",
            ))

    if include:
        for pattern in initrd_modules:
            if pattern.startswith(("-", "!")):
                continue
            if pattern not in include:
                findings.append(Finding(
                    paths[0] if paths else Path("image/mkosi.conf"), 0, "ERROR",
                    f"the initrd asks for {pattern!r}, which the image's "
                    f"whitelist does not keep. mkosi copies the extra initrd out of "
                    f"the image's /usr/lib/modules, so the name exists on paper and "
                    f"nowhere else. Add it to that list, or drop the whitelist so "
                    f"the image carries every module.",
                ))
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

    # The module-set check has to be able to fail too, so it is run against a
    # synthetic config that reproduces BUG-0020 exactly: the mkosi 26 spellings,
    # a regex-shaped include list, and an initrd list that is a subset of nothing.
    scratch = root / "build"
    scratch.mkdir(parents=True, exist_ok=True)
    cases = {
        # The original defect: the mkosi 26 spellings, whose values are regexes.
        "selftest-bug-0020.conf": (
            "[Content]\n"
            "KernelModulesExclude=.*\n"
            "KernelModulesInclude=\n"
            "        virtio_*\n"
            "        dm[-_]*\n"
            "KernelModulesInitrdInclude=\n"
            "        erofs\n"
            "        dm-verity\n",
            ["mkosi 26 spelling", "no KernelInitrdModules"],
        ),
        # The second act: an exclude-everything pattern beside a whitelist, which
        # mkosi silently resolves by keeping nothing.
        "selftest-whipsaw.conf": (
            "[Content]\n"
            "KernelModules=\n"
            "        -.*\n"
            "        erofs\n"
            "        virtio*\n"
            "KernelInitrdModules=\n"
            "        erofs\n"
            "        dm-*\n",
            ["keeps no modules at all"],
        ),
        # And the quiet one: an initrd list the image's whitelist would drop.
        "selftest-whitelist.conf": (
            "[Content]\n"
            "KernelModules=\n"
            "        virtio*\n"
            "KernelInitrdModules=\n"
            "        erofs\n"
            "        dm-*\n",
            ["whitelist does not keep"],
        ),
    }
    try:
        for name, (text, expected) in cases.items():
            path = scratch / name
            path.write_text(text, encoding="utf-8")
            found = [f for f in check_module_sets([path]) if f.severity == "ERROR"]
            messages = " ".join(f.message for f in found)
            missing = [e for e in expected if e not in messages]
            if missing:
                print(f"FAIL  module-set check missed {missing} in {name}")
                rc = 1
            else:
                print(f"PASS  regression: {name} is rejected ({len(found)} error(s))")
            path.unlink(missing_ok=True)
    finally:
        for name in cases:
            (scratch / name).unlink(missing_ok=True)

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
    linted: list[Path] = []
    for path in targets:
        # repart files use systemd-repart syntax, not mkosi syntax.
        if "mkosi.repart" in path.parts:
            continue
        linted.append(path)
        for f in lint(path):
            print(f)
            if f.severity == "ERROR":
                errors += 1

    for f in check_module_sets(linted):
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
