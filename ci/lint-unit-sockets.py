#!/usr/bin/env python3
"""Fail when a LitricOS socket unit is connectable by someone it is not for.

The defect this was written for: `image/units-pending/litric-capd.socket` set
`SocketMode=0666` and said so was deliberate — "every process must be able to
ASK, authorisation happens inside the broker using SO_PEERCRED plus the
caller's cgroup-derived application identity". None of that authorisation
exists. There is no `litric-capd` binary in `src/`, `litric-caps` is a pure
library with no socket and no I/O, and ADR-0016 puts the broker inside
`litric-sysd`, whose own socket is `0600`. A world-connectable socket in front
of a broker that cannot attribute a peer is a door with no lock.

A comment claiming a check is not a check. This linter reads the unit files and
requires the property to be stated in the place systemd actually enforces it:

  * every `*.socket` under `image/` declares `SocketMode=0600` — no allowlist,
    because the only multi-user socket this tree wants would need peer
    credentials that are not implemented (widening it must first add the code
    and the tests, and then a reviewed exception here with the reason);
  * every listen path is a socket file under `/run/litric/`, so a unit cannot
    quietly open a second, wider channel;
  * `Accept=` is present, and `Accept=no` names `SocketUser=` and
    `SocketGroup=`, because a long-running listener's ownership must be
    explicit rather than inherited from whoever started systemd;
  * `litric-files.socket` is owned by `litric`, the user its service runs as.
    A root-owned socket would require root's DAC override for the ordinary
    client path and would contradict the threat model; this rule is the
    regression test for that ownership decision;
  * `RemoveOnStop=yes`, so a stopped unit leaves no stale socket behind (the
    daemons replace a stale socket, but leaving one is still a lie about who is
    listening).

What it deliberately does not do: it does not boot anything, so it proves what
the unit *declares*, not what systemd applies at runtime. The daemon-side bind
mode is proven by the Rust tests in each crate (`0600`, live socket refused),
and the two statements now agree.

Usage:
    python3 ci/lint-unit-sockets.py             check
    python3 ci/lint-unit-sockets.py --self-test prove it can fail
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IMAGE = ROOT / "image"

# Every LitricOS socket is a file under this directory. The shell's Wayland
# socket lives here too, which is also why the directory itself is 0700.
RUNTIME_PREFIX = "/run/litric/"

EXPECTED_MODE = "0600"

# The Files daemon's service runs `User=litric`. The listening socket must be
# owned by the same identity; otherwise the shell would depend on root's DAC
# override rather than on the daemon user's own permission.
SOCKET_OWNERS = {"litric-files.socket": ("litric", "litric")}


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

        listens = [key for key in ("ListenStream", "ListenDatagram") if key in values]
        if not listens:
            errors.append(f"{rel}: no ListenStream= or ListenDatagram=")
        for key in listens:
            target = values[key]
            if not target.startswith(RUNTIME_PREFIX):
                errors.append(
                    f"{rel}: {key}={target} is not a socket file under {RUNTIME_PREFIX}"
                )

        mode = values.get("SocketMode")
        if mode is None:
            errors.append(
                f"{rel}: no SocketMode=, so the caller's umask would decide who can connect"
            )
        elif mode != EXPECTED_MODE:
            errors.append(
                f"{rel}: SocketMode={mode}, expected {EXPECTED_MODE}: "
                "no peer-credential check is implemented for this socket"
            )

        accept = values.get("Accept")
        if accept is None:
            errors.append(f"{rel}: no Accept=, so the activation mode is not stated")
        elif accept == "no":
            for key in ("SocketUser", "SocketGroup"):
                if key not in values:
                    errors.append(
                        f"{rel}: Accept=no needs {key}= so the listener's ownership is explicit"
                    )

        expected_owner = SOCKET_OWNERS.get(path.name)
        if expected_owner is not None:
            for key, expected in zip(("SocketUser", "SocketGroup"), expected_owner):
                actual = values.get(key)
                if actual != expected:
                    errors.append(
                        f"{rel}: {key}={actual or 'missing'}, expected {expected}: "
                        "litric-filesd runs as that user, so the socket must be owned by it"
                    )

        if values.get("RemoveOnStop") != "yes":
            errors.append(f"{rel}: RemoveOnStop=yes is required so a stop leaves no socket file")

    return errors


def self_test() -> int:
    """Prove the checker fails a planted world-connectable unit."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "good.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/good.sock\n"
            "SocketUser=root\n"
            "SocketGroup=root\n"
            f"SocketMode={EXPECTED_MODE}\n"
            "Accept=no\n"
            "RemoveOnStop=yes\n",
            encoding="utf-8",
        )
        good = check_root(root)
        if good:
            print("FAIL: the checker rejected a conforming unit:")
            for line in good:
                print(f"  {line}")
            return 1

        (root / "bad.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/bad.sock\n"
            "SocketMode=0666\n"
            "Accept=no\n",
            encoding="utf-8",
        )
        bad = check_root(root)
        if not bad:
            print("FAIL: the checker accepted a world-connectable socket")
            return 1
        (root / "litric-files.socket").write_text(
            "[Socket]\n"
            "ListenStream=/run/litric/files.sock\n"
            "SocketUser=root\n"
            "SocketGroup=root\n"
            f"SocketMode={EXPECTED_MODE}\n"
            "Accept=yes\n"
            "RemoveOnStop=yes\n",
            encoding="utf-8",
        )
        wrong_owner = check_root(root)
        if not any("SocketUser=root, expected litric" in line for line in wrong_owner):
            print("FAIL: the checker accepted a root-owned Files socket")
            for line in wrong_owner:
                print(f"  {line}")
            return 1

        print(
            "OK: checker rejects a planted 0666 unit and a root-owned Files socket"
        )
        return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    errors = check_root(IMAGE)
    if errors:
        print(f"FAIL: {len(errors)} socket-unit problem(s)")
        for line in errors:
            print(f"  {line}")
        return 1
    count = len(sorted(IMAGE.rglob("*.socket")))
    print(f"OK: {count} socket unit(s) are owner-only, explicit and removed on stop")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
