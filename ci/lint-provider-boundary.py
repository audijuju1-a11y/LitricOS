#!/usr/bin/env python3
"""Fail when the shell grows a D-Bus client or the provider contract disappears.

C1 is a boundary, not a provider implementation. The shell already reaches
network, Bluetooth and audio through `litric-sysd`; the daemon owns the base
tools and the capability checks. A D-Bus library in `litric-shell` would move a
protocol client and its attack surface into the process that draws the password
prompt, which is exactly what ADR-0023 decision 1 refuses.

This checker enforces two facts:

  * `src/litric-shell/Cargo.toml` has no D-Bus dependency, and no Rust source
    under `src/litric-shell` imports or paths through a `zbus`/`dbus` crate;
  * `src/litric-sysd/src/provider_contract.rs` still defines the `Provider`
    trait and the `FakeProvider` used by contract tests.

It reads source text, not a running process: it cannot prove what a binary links
at runtime, and does not claim to. The dependency and import checks are the
source-level guard for the boundary the plan asks for.

Usage:
    python3 ci/lint-provider-boundary.py             check
    python3 ci/lint-provider-boundary.py --self-test prove it can fail
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHELL = "src/litric-shell"
CONTRACT = "src/litric-sysd/src/provider_contract.rs"

FORBIDDEN_DEPS = {"dbus", "dbus-sys", "libdbus", "libdbus-sys", "zbus", "zbus-sys"}
DEP_TABLE_RE = re.compile(r"^\s*\[([^\]]+)\]\s*$")
DEP_KEY_RE = re.compile(r"^\s*([A-Za-z0-9_.-]+)\s*=")
USE_RE = re.compile(r"^\s*(?:pub\s+)?(?:use|extern\s+crate)\s+(?:::)?(zbus|dbus|libdbus)\b")
PATH_RE = re.compile(r"\b(zbus|dbus|libdbus)::")


def boundary_errors(root: Path) -> list[str]:
    """D-Bus use in the shell, as one sentence per finding."""
    errors: list[str] = []
    manifest = root / SHELL / "Cargo.toml"
    if not manifest.is_file():
        return [f"{SHELL}/Cargo.toml: missing; the shell boundary cannot be checked"]
    table = ""
    for raw in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        table_match = DEP_TABLE_RE.match(line)
        if table_match:
            table = table_match.group(1)
            continue
        if not table.endswith("dependencies"):
            continue
        key = DEP_KEY_RE.match(line)
        if key and key.group(1).lower() in FORBIDDEN_DEPS:
            errors.append(
                f"{SHELL}/Cargo.toml: dependency {key.group(1)!r} is a D-Bus client; "
                "the shell must go through the daemon provider contract (ADR-0023)"
            )

    for path in sorted((root / SHELL).rglob("*.rs")):
        for number, line in enumerate(
            path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1
        ):
            if USE_RE.match(line) or PATH_RE.search(line):
                errors.append(
                    f"{path.relative_to(root).as_posix()}:{number}: D-Bus client use; "
                    "the desktop process must not own this protocol"
                )
    return errors


def contract_errors(root: Path) -> list[str]:
    """The provider seam and its fake must still be present."""
    path = root / CONTRACT
    if not path.is_file():
        return [f"{CONTRACT}: missing; C1's provider contract is gone"]
    text = path.read_text(encoding="utf-8", errors="replace")
    errors: list[str] = []
    if "pub trait Provider" not in text:
        errors.append(f"{CONTRACT}: no `pub trait Provider`")
    if "pub struct FakeProvider" not in text:
        errors.append(f"{CONTRACT}: no `pub struct FakeProvider`")
    return errors


def check_root(root: Path) -> list[str]:
    return boundary_errors(root) + contract_errors(root)


def self_test() -> int:
    """A clean shell passes; a planted dependency and a planted import fail."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        shell = root / SHELL
        (shell / "src").mkdir(parents=True)
        (shell / "Cargo.toml").write_text(
            "[package]\nname = \"litric-shell\"\n\n[dependencies]\nlitric-sysd = { path = \"../litric-sysd\" }\n",
            encoding="utf-8",
        )
        (shell / "src/main.rs").write_text(
            "use litric_sysd::provider_contract::Provider;\n", encoding="utf-8"
        )
        if boundary_errors(root):
            print("FAIL: the checker rejected a shell with no D-Bus client")
            return 1

        (shell / "Cargo.toml").write_text(
            "[package]\nname = \"litric-shell\"\n\n[dependencies]\nzbus = \"4\"\n",
            encoding="utf-8",
        )
        if not any("zbus" in line for line in boundary_errors(root)):
            print("FAIL: the checker accepted a planted zbus dependency")
            return 1

        (shell / "Cargo.toml").write_text(
            "[package]\nname = \"litric-shell\"\n\n[dependencies]\n",
            encoding="utf-8",
        )
        (shell / "src/main.rs").write_text(
            "use zbus::Connection;\nfn main() {}\n", encoding="utf-8"
        )
        if not any("D-Bus client use" in line for line in boundary_errors(root)):
            print("FAIL: the checker accepted a planted zbus import")
            return 1

    print("OK: checker rejects a planted zbus dependency and a planted zbus import")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    errors = check_root(ROOT)
    if errors:
        print(f"FAIL: {len(errors)} provider-boundary finding(s)")
        for line in errors:
            print(f"  {line}")
        return 1
    print(
        "OK: shell has no D-Bus dependency or client path; "
        f"{CONTRACT} exposes Provider and FakeProvider"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
