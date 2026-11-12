#!/usr/bin/env python3
"""Fail when the shipped wired-network configuration stops matching the plan.

The defect this exists for: `20-wired.network` hard-coded QEMU's slirp lease
(10.0.2.15/24, gateway 10.0.2.2, DNS 10.0.2.3) for every `en*`/`eth*`
interface. That is correct inside the proof VM and wrong on every physical
machine -- the link comes up with an address that is either off-subnet or
already in use, and nothing routes. It is invisible from a QEMU-only test suite,
which is exactly why it needs a static check rather than a boot.

What this asserts, from the files themselves:

  * one `.network` file matches the generic wired names (`en*` / `eth*`) and it
    asks for DHCP, never a static 10.0.2.0/24 lease;
  * one `.network` file matches QEMU's default NIC MAC and carries the static
    slirp trio;
  * the MAC-matched file sorts before the generic one, because
    `systemd-networkd` applies the first matching file and a later static file
    would never be consulted;
  * no second file claims the same generic name match, so the ordering cannot
    be changed by adding a file that sorts earlier without this failing.

This is a configuration check. It does not boot an image, and it makes no claim
about link state at runtime.

Usage:
    python3 ci/lint-network-config.py
    python3 ci/lint-network-config.py --self-test
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NETWORK_DIR = ROOT / "image/mkosi.extra/usr/lib/systemd/network"

QEMU_MAC = "52:54:00:12:34:56"
SLIRP = ("Address=10.0.2.15/24", "Gateway=10.0.2.2", "DNS=10.0.2.3")


def parse(path: Path) -> dict[str, list[str]]:
    """Return section -> raw lines, preserving order and ignoring comments."""
    sections: dict[str, list[str]] = {}
    current = ""
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.startswith(";"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1].strip()
            sections.setdefault(current, [])
            continue
        sections.setdefault(current, []).append(line)
    return sections


def values(sections: dict[str, list[str]], name: str) -> list[str]:
    out: list[str] = []
    for lines in sections.values():
        for line in lines:
            key, sep, value = line.partition("=")
            if sep and key.strip() == name:
                out.append(value.strip())
    return out


def matches_generic(sections: dict[str, list[str]]) -> bool:
    for value in values(sections, "Name"):
        if any(token in {"en*", "eth*"} for token in value.split()):
            return True
    return False


def matches_qemu_mac(sections: dict[str, list[str]]) -> bool:
    return any(
        QEMU_MAC.lower() in value.lower()
        for value in values(sections, "MACAddress")
    )


def check_tree(directory: Path) -> list[str]:
    errors: list[str] = []
    files = sorted(directory.glob("*.network"))
    if not files:
        return [f"{directory}: no .network files"]

    generic: list[Path] = []
    qemu: list[Path] = []
    for path in files:
        sections = parse(path)
        if matches_generic(sections):
            generic.append(path)
        if matches_qemu_mac(sections):
            qemu.append(path)
        for address in values(sections, "Address"):
            if address.startswith("10.0.2.") and not matches_qemu_mac(sections):
                errors.append(
                    f"{path.name}: a generic wired file sets the QEMU slirp "
                    f"address {address}; real hardware needs DHCP"
                )

    if len(generic) != 1:
        errors.append(
            "expected exactly one .network matching en*/eth*, found "
            + (", ".join(p.name for p in generic) or "none")
        )
    else:
        sections = parse(generic[0])
        dhcp = values(sections, "DHCP")
        if dhcp != ["yes"]:
            errors.append(
                f"{generic[0].name}: the wired default must set DHCP=yes, found {dhcp or 'nothing'}"
            )

    if len(qemu) != 1:
        errors.append(
            f"expected exactly one .network matching QEMU MAC {QEMU_MAC}, found "
            + (", ".join(p.name for p in qemu) or "none")
        )
    else:
        sections = parse(qemu[0])
        for needle in SLIRP:
            key, _, value = needle.partition("=")
            if value not in values(sections, key):
                errors.append(f"{qemu[0].name}: missing static slirp line {needle}")

    if generic and qemu:
        if not (qemu[0].name < generic[0].name):
            errors.append(
                f"{qemu[0].name} must sort before {generic[0].name}: networkd "
                "applies the first matching file and the static VM config would "
                "never be used"
            )

    return errors


FIXTURES = {
    "good/10-qemu-slirp.network": """\
[Match]
MACAddress=52:54:00:12:34:56

[Network]
Address=10.0.2.15/24
Gateway=10.0.2.2
DNS=10.0.2.3
""",
    "good/20-wired.network": """\
[Match]
Name=en* eth*

[Network]
DHCP=yes
""",
    "bad-static/10-fixed.network": """\
[Match]
Name=en* eth*

[Network]
Address=10.0.2.15/24
Gateway=10.0.2.2
DNS=10.0.2.3
""",
    "bad-order/50-qemu.network": """\
[Match]
MACAddress=52:54:00:12:34:56

[Network]
Address=10.0.2.15/24
Gateway=10.0.2.2
DNS=10.0.2.3
""",
    "bad-order/20-wired.network": """\
[Match]
Name=en* eth*

[Network]
DHCP=yes
""",
}


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        base = Path(tmp)
        for relative, text in FIXTURES.items():
            path = base / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")

        failures = 0
        cases = [
            ("good", 0),
            ("bad-static", 1),
            ("bad-order", 1),
        ]
        for name, expected_nonzero in cases:
            errors = check_tree(base / name)
            accepted = not errors
            if accepted == bool(expected_nonzero):
                failures += 1
                print(f"self-test: {name} produced {len(errors)} error(s), expected "
                      f"{'at least one' if expected_nonzero else 'none'}")
                for error in errors:
                    print(f"  {error}")

        if failures:
            print(f"self-test FAILED: {failures} case(s) behaved incorrectly")
            return 1
        print("self-test passed: good config accepted, both defects rejected")
        return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    errors = check_tree(NETWORK_DIR)
    if errors:
        for error in errors:
            print(error)
        print(f"network configuration: {len(errors)} problem(s)")
        return 1
    print("network configuration: DHCP default + MAC-matched VM slirp, ordering correct")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
