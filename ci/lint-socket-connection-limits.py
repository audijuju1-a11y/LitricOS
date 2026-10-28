#!/usr/bin/env python3
"""Fail when a socket-activated LitricOS unit leaves its concurrency to the default.

`Accept=yes` makes systemd start one process per connection (`litric-files@.service`,
`litric-browser@.service`), so the number of simultaneous connections — and
therefore the per-peer memory the machine must hold — is a socket-unit setting,
not a daemon setting. If `MaxConnections=` is absent, systemd applies its own
default of **64** (`man systemd.socket`, verified against systemd 255 here),
which is a cap nobody chose, for a service whose own unit declares
`MemoryMax=256M` (files) or `MemoryMax=1536M` (browser). A missing line is not
"no limit", it is "sixty-four of them".

What this checks, for every `*.socket` under `image/`:

  * `Accept=` is present (the activation mode must be stated, not inherited);
  * when `Accept=yes`, `MaxConnections=` is present, is a plain integer, and is
    within `1..=MAX_ALLOWED`. The upper bound is the daemon's own
    `MAX_CONNECTIONS` constant (16, in `litric-files/src/server.rs` and
    `litric-sysd/src/server.rs`): a unit that promised more concurrency than the
    code serves would document a capacity that does not exist;
  * when `Accept=no`, `MaxConnections=` is not required: systemd states it has
    no effect there (same man page), so demanding it would be cargo cult. If it
    is present anyway it is still checked against the range, so a value cannot
    be quietly out of bounds on the parked capd socket either.

What it deliberately does not do: it does not run systemd, so it proves what the
units declare. `systemd-analyze verify` on the real units is the parse-level
check (`rc=0` for both shipped sockets when this was written); a boot is not
available in this environment and is not claimed.

Usage:
    python3 ci/lint-socket-connection-limits.py             check
    python3 ci/lint-socket-connection-limits.py --self-test prove it can fail
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IMAGE = ROOT / "image"

# `MAX_CONNECTIONS` in the daemons. A unit may ask for less, never for more.
MAX_ALLOWED = 16


def parse_socket_section(path: Path) -> dict[str, str]:
    """Return the `[Socket]` keys of one unit file. First occurrence wins."""
    values: dict[str, str] = {}
    section = ""
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line[0] in "#;":
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if section != "Socket" or "=" not in line:
            continue
        key, _, value = line.partition("=")
        values.setdefault(key.strip(), value.strip())
    return values


def check_root(root: Path) -> list[str]:
    """Every rule violation under `root`, as one sentence each."""
    errors: list[str] = []
    units = sorted(root.rglob("*.socket"))
    if not units:
        return [f"no .socket unit found under {root}"]
    for path in units:
        rel = path.relative_to(root).as_posix()
        values = parse_socket_section(path)

        accept = values.get("Accept")
        if accept is None:
            errors.append(
                f"{rel}: no Accept=, so whether this socket is per-connection is unstated"
            )
            continue

        raw = values.get("MaxConnections")
        if accept == "yes" and raw is None:
            errors.append(
                f"{rel}: Accept=yes without MaxConnections=, so systemd's default of 64 applies"
            )
            continue
        if raw is None:
            continue  # Accept=no: the setting has no effect, so it is not required.
        if not raw.isdigit():
            errors.append(f"{rel}: MaxConnections={raw} is not a plain integer")
            continue
        value = int(raw, 10)
        if not 1 <= value <= MAX_ALLOWED:
            errors.append(
                f"{rel}: MaxConnections={value} is outside 1..={MAX_ALLOWED}, "
                "the ceiling the daemons' own MAX_CONNECTIONS documents"
            )
    return errors


def self_test() -> int:
    """Prove the checker fails a fixture that omits or overshoots the cap."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "good.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/good.sock\n"
            "Accept=yes\n"
            "MaxConnections=8\n",
            encoding="utf-8",
        )
        (root / "parked.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/parked.sock\n"
            "Accept=no\n",
            encoding="utf-8",
        )
        good = check_root(root)
        if good:
            print("FAIL: the checker rejected conforming fixtures:")
            for line in good:
                print(f"  {line}")
            return 1

        (root / "missing.socket").write_text(
            "[Socket]\nListenStream=/run/litric/missing.sock\nAccept=yes\n",
            encoding="utf-8",
        )
        missing = check_root(root)
        if not missing:
            print("FAIL: the checker accepted Accept=yes without MaxConnections=")
            return 1

        (root / "overshoot.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/overshoot.sock\n"
            "Accept=yes\n"
            f"MaxConnections={MAX_ALLOWED + 1}\n",
            encoding="utf-8",
        )
        overshoot = check_root(root)
        if not overshoot:
            print("FAIL: the checker accepted a value above the daemon's ceiling")
            return 1
        print(
            "OK: checker rejects a missing cap and an out-of-range cap "
            f"({len(missing)} + {len(overshoot)} finding(s))"
        )
        return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    errors = check_root(IMAGE)
    if errors:
        print(f"FAIL: {len(errors)} socket concurrency problem(s)")
        for line in errors:
            print(f"  {line}")
        return 1
    count = len(sorted(IMAGE.rglob("*.socket")))
    print(f"OK: {count} socket unit(s) state their activation mode and concurrency cap")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
