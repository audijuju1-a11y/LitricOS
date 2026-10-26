#!/usr/bin/env python3
"""Check the memory-budget ledger against the units this repository actually ships.

WHAT THIS CAN AND CANNOT PROVE
------------------------------
Static policy presence, only. It reads `docs/PERF-budget.md` and the systemd
units under `image/mkosi.extra/`, and answers one question: *is every memory
limit the ledger claims to enforce actually configured for the unit the ledger
names, and is every cap the repository ships accounted for in the ledger?* It
never inspects the running system, never starts a unit, and cannot tell you how
much memory anything uses. "Static configuration is compliant" and "measured
runtime consumption is within budget" are different claims, and only the first
is in scope.

It deliberately does NOT check that per-unit caps sum to the profile total. Hard
caps are not usage: `MemoryMax=` is the point at which the cgroup OOM killer
becomes involved, several caps may never be reached simultaneously, and a
component that is not resident at idle contributes nothing to an idle figure.
Summing caps and comparing to an idle target would be arithmetic theatre. The
ledger's own column sums are checked instead, which is a claim the document
actually makes.

WHY EFFECTIVE, NOT LITERAL
--------------------------
A unit's limit is not just the directive in its own file. systemd reports
`EffectiveMemoryMax=`, "the most stringent limit of the unit and parent slices"
(systemd.resource-control(5)), and cgroup v2 states that restrictions closer to
the root cannot be overridden further down
(Documentation/admin-guide/cgroup-v2.html). So a slice's limit governs only the
units that join that slice, and a unit's own directive can be tighter than its
slice's. Both directions are resolved here, because the defect this checker was
written for was a slice carrying the documented intent that no unit had joined.

THE LEDGER CELL GRAMMAR
-----------------------
  `MemoryMax=64M`, `MemoryHigh=56M, MemoryMax=64M`
        A claim. It must be configured for the named unit, and it must be the
        value that actually applies to it. Otherwise this check fails.
  `... [ACK: reason — BUG-0008]`
        An acknowledged gap. Printed and counted, never silent. The reason must
        name a review condition (a bug id, an ADR, or a review-by date); an
        acknowledgement without one is an undocumented exception and fails.
  `TARGET (not enforced: reason)`   same, in the wording the ledger already uses.
  `boot-time measurement`, `dbus-activated`, `—`
        Outside unit-file scope.
  `system.slice` / any other `.slice` name
        Checked as a slice claim: the slice must exist and must set a limit.

Exit code: 0 when nothing is unaccounted for, 1 otherwise. `--strict` also
fails on acknowledged gaps, for when the ledger's exceptions have been closed.

Usage:
    python3 ci/check-memory-budget.py [--strict]
    python3 ci/check-memory-budget.py --self-test
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "PERF-budget.md"
UNIT_DIR = ROOT / "image" / "mkosi.extra" / "usr" / "lib" / "systemd" / "system"

MEM_KEYS = ("MemoryMin", "MemoryLow", "MemoryHigh", "MemoryMax", "MemorySwapMax")
KEY_HOME = {"service": "Service", "slice": "Slice"}

SIZE_RE = re.compile(r"^(\d+)([KMGTPEkmgtpe]?)$")
BARE_BYTES_FLOOR = 1024 * 1024
CLAIM_RE = re.compile(r"(Memory(?:Min|Low|High|Max|SwapMax))=([0-9]+[KMGTPEkmgtpe]?)")
UNIT_NAME_RE = re.compile(r"\b([A-Za-z0-9@._-]+\.(?:service|slice))\b")

# Slices and scopes systemd itself provides. They can never appear in this
# repository, so "not shipped here" is not a defect for them; the defect is a
# ledger line attributing enforcement to one that carries no limit.
SYSTEM_PROVIDED = {"system.slice", "user.slice", "-.slice", "machine.slice",
                   "init.scope", "user-0.slice"}

BUDGET_RE = re.compile(r"^(\d+)\s*MB(?:\s+idle)?$")
ACK_RE = re.compile(r"\[ACK:\s*([^\]]+?)\s*\]")
ACK_REVIEW_RE = re.compile(r"BUG-\d{4}|ADR-\d{4}|REVIEW-BY-\d{4}-\d{2}-\d{2}")

# Which units a ledger line may legitimately be enforced by. Explicit, because a
# wrong mapping silently approves a wrong limit.
COMPONENT_UNITS = {
    "litric-shell": ["litric-shell.service", "litric-shell.slice"],
    "litric-defender": ["litric-defender.slice", "litric-defender.service"],
    "litric-aid": ["litric-ai.slice"],
    "litric-sysd": ["litric-sysd.service"],
    "litric-envd": ["litric-envd.service"],
    "litric-notifyd": ["litric-notifyd.service"],
    "journald": ["systemd-journald.service"],
    "udevd": ["systemd-udevd.service"],
    "dbus-broker": ["dbus-broker.service"],
    "iwd + networkd + resolved": ["iwd.service", "systemd-networkd.service",
                                  "systemd-resolved.service"],
    "seatd": ["seatd.service"],
    "pipewire + wireplumber": ["pipewire.service", "wireplumber.service"],
    "systemd PID 1": ["init.scope"],
}


def _safe_stdout() -> None:
    """Never die because a console cannot encode a character we printed.

    The first version of this file crashed with UnicodeEncodeError on a cp874
    console while reporting a finding -- a checker that fails to report is worse
    than the finding it was reporting.
    """
    try:
        sys.stdout.reconfigure(errors="replace")  # type: ignore[union-attr]
    except Exception:
        pass


class Finding:
    def __init__(self, severity: str, where: str, message: str) -> None:
        self.severity, self.where, self.message = severity, where, message

    def __str__(self) -> str:
        return f"{self.severity:<14} {self.where}: {self.message}"


# ---------------------------------------------------------------------------
# unit files
# ---------------------------------------------------------------------------

class Unit:
    def __init__(self, name: str, kind: str) -> None:
        self.name, self.kind = name, kind
        self.props: dict[str, str] = {}
        self.sections: dict[str, str] = {}
        self.wanted_by: list[str] = []
        self.slice: str | None = None

    def mem(self, key: str) -> str | None:
        return self.props.get(key)

    @property
    def parent_slice(self) -> str:
        if self.slice:
            return self.slice
        if self.kind == "service":
            return "system.slice"
        stem = self.name[: -len(".slice")]
        if "-" in stem:
            return stem.rsplit("-", 1)[0] + ".slice"
        return "-.slice"


def _strip_comment(line: str) -> str:
    """systemd: whole-line comments, and trailing comments after whitespace."""
    s = line.strip()
    if not s or s.startswith("#") or s.startswith(";"):
        return ""
    for marker in (" #", " ;"):
        if marker in s:
            s = s.split(marker, 1)[0].strip()
    return s


def parse_unit(path: Path, name: str | None = None) -> Unit:
    kind = "slice" if path.name.endswith(".slice") else "service"
    unit = Unit(name or path.name, kind)
    section = ""
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = _strip_comment(raw)
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            continue
        if "=" not in line:
            continue
        key, value = (p.strip() for p in line.split("=", 1))
        if key == "Slice":
            unit.slice = value
        elif key in ("WantedBy", "RequiredBy"):
            unit.wanted_by.append(value)
        elif key in MEM_KEYS:
            unit.props[key] = value
            unit.sections[key] = section
    return unit


def load_units(unit_dir: Path) -> dict[str, Unit]:
    units: dict[str, Unit] = {}
    if not unit_dir.is_dir():
        return units
    for path in sorted(unit_dir.iterdir()):
        if path.is_file() and path.suffix in (".service", ".slice"):
            units[path.name] = parse_unit(path)
    return units


def parse_dropins(unit_dir: Path) -> dict[str, dict[str, str]]:
    out: dict[str, dict[str, str]] = {}
    if not unit_dir.is_dir():
        return out
    for d in sorted(unit_dir.glob("*.d")):
        if not d.is_dir():
            continue
        target = d.name[:-2]
        for conf in sorted(d.glob("*.conf")):
            parsed = parse_unit(conf, name=target)
            out.setdefault(target, {}).update(parsed.props)
    return out


def size_bytes(value: str) -> int | None:
    m = SIZE_RE.match(value.strip())
    if not m:
        return None
    mult = {"": 1, "k": 1024, "m": 1024 ** 2, "g": 1024 ** 3,
            "t": 1024 ** 4, "p": 1024 ** 5}[m.group(2).lower()]
    return int(m.group(1)) * mult


def human(n: int | None) -> str:
    if n is None:
        return "unbounded"
    for suffix, div in (("G", 1024 ** 3), ("M", 1024 ** 2), ("K", 1024)):
        if n >= div and n % div == 0:
            return f"{n // div}{suffix}"
    return f"{n}B"


def effective(units: dict[str, Unit], unit: Unit, key: str) -> tuple[int | None, str]:
    """Most stringent value along the unit's slice chain, and where it came from."""
    best, origin, seen = None, "unbounded", set()
    current: Unit | None = unit
    while current is not None and current.name not in seen:
        seen.add(current.name)
        value = current.mem(key)
        if value is not None:
            n = size_bytes(value)
            if n is not None and (best is None or n < best):
                best, origin = n, current.name
        parent = current.parent_slice
        current = units.get(parent) if parent else None
    return best, origin


# ---------------------------------------------------------------------------
# ledger
# ---------------------------------------------------------------------------

class Row:
    def __init__(self, lineno: int, cells: list[str]) -> None:
        self.lineno, self.cells = lineno, cells

    @property
    def component(self) -> str:
        """The component name, with markdown emphasis removed."""
        return self.cells[1].replace("*", "").replace("`", "").strip()

    @property
    def enforced(self) -> str:
        return self.cells[3]

    @property
    def unit_refs(self) -> list[str]:
        return UNIT_NAME_RE.findall(self.enforced) + \
            UNIT_NAME_RE.findall(" ".join(self.cells[4:]))


def resolve_component(component: str) -> list[str] | None:
    """Map a ledger component name to unit files, exactly then by prefix."""
    if component in COMPONENT_UNITS:
        return COMPONENT_UNITS[component]
    stem = component.split("(")[0].strip()
    if stem in COMPONENT_UNITS:
        return COMPONENT_UNITS[stem]
    hits = [k for k in COMPONENT_UNITS if stem.startswith(k)]
    if len(hits) == 1:
        return COMPONENT_UNITS[hits[0]]
    return None


def idle_rows(path: Path) -> list[Row]:
    rows: list[Row] = []
    in_table = False
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if line.startswith("| # | Component | Budget | Enforced by"):
            in_table = True
            continue
        if not in_table:
            continue
        if not line.startswith("|"):
            if rows:
                break
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 4 or set(cells[0]) <= set("-: "):
            continue
        if "TOTAL" in line:
            continue
        rows.append(Row(lineno, cells))
    return rows


def total_row(path: Path) -> int | None:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("|") and "**TOTAL**" in line:
            m = re.search(r"\*\*(\d+)\*\*\s*\|", line)
            if m:
                return int(m.group(1))
    return None


def capped_rows(path: Path) -> list[Row]:
    """Secondary table: units with a cap that are not resident at idle."""
    rows: list[Row] = []
    in_table = False
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if line.startswith("| Unit | MemoryMax |"):
            in_table = True
            continue
        if not in_table:
            continue
        if not line.startswith("|"):
            if rows:
                break
            continue
        cells = [c.strip().strip("`") for c in line.strip().strip("|").split("|")]
        if len(cells) < 2 or set(cells[0]) <= set("-: "):
            continue
        rows.append(Row(lineno, cells))
    return rows


def ack_text(cell: str) -> str | None:
    m = ACK_RE.search(cell)
    if m:
        return m.group(1)
    if "not enforced" in cell.lower():
        return cell.strip()
    return None


# ---------------------------------------------------------------------------
# the check
# ---------------------------------------------------------------------------

def check(root: Path, ledger: Path, unit_dir: Path) -> list[Finding]:
    out: list[Finding] = []
    units = load_units(unit_dir)
    dropins = parse_dropins(unit_dir)

    if not units:
        return [Finding("ERROR", str(unit_dir),
                        "no unit files discovered -- the check would pass by "
                        "finding nothing, which is the failure mode it must not have")]

    # Drop-ins are part of the effective configuration. A drop-in whose target
    # unit comes from the distribution (systemd-journald.service and friends) is
    # the normal, correct way to bound such a unit, so create a placeholder for
    # it: without one, a repository that HAD configured journald would still be
    # reported as if it had not.
    for name, props in dropins.items():
        if name not in units:
            units[name] = Unit(name, "slice" if name.endswith(".slice") else "service")
        units[name].props.update(props)

    # --- values systemd would refuse, or that mean something else -----------
    for unit in units.values():
        for key, value in unit.props.items():
            if value in ("infinity", "max"):
                continue
            n = size_bytes(value)
            if n is None:
                out.append(Finding(
                    "ERROR", f"{unit.name}: {key}={value}",
                    "systemd will not accept this value and rejects the unit; "
                    "check the suffix (K/M/G)"))
                continue
            if value != "0" and n < BARE_BYTES_FLOOR:
                out.append(Finding(
                    "ERROR", f"{unit.name}: {key}={value}",
                    f"a bare number is bytes, so this reads as {n} bytes; "
                    f"did you mean {value}M?"))
            home = KEY_HOME.get(unit.kind)
            if home and unit.sections.get(key) != home:
                out.append(Finding(
                    "ERROR", f"{unit.name}: {key} in [{unit.sections.get(key)}]",
                    f"resource control for a {unit.kind} belongs in [{home}]; "
                    f"systemd ignores it where it is, so nothing is configured"))

    # --- ledger ---------------------------------------------------------------
    rows = idle_rows(ledger)
    if not rows:
        out.append(Finding("ERROR", str(ledger.name),
                           "no ledger rows parsed -- has the table format changed?"))
        return out

    declared, unset_budget = 0, []
    acknowledged_rows: dict[str, str] = {}
    for row in rows:
        component, cell = row.component, row.enforced
        where = f"{ledger.name}:{row.lineno} ({component})"
        ack = ack_text(cell)
        if ack:
            acknowledged_rows[component] = ack
            if not ACK_REVIEW_RE.search(ack):
                out.append(Finding(
                    "ERROR", where,
                    f"acknowledgement '{ack}' names no review condition; add a "
                    f"bug id (BUG-0000), an ADR, or REVIEW-BY-YYYY-MM-DD"))

        budget = row.cells[2].replace("*", "").strip()
        m = BUDGET_RE.match(budget)
        if m:
            declared += int(m.group(1))
        elif "unset" in budget.lower():
            unset_budget.append(component)
        else:
            out.append(Finding(
                "ERROR", where,
                f"Budget cell {budget!r} is neither '<n> MB', '<n> MB idle', nor 'unset'"))

        # The acknowledgement text describes the numbers; it must not be read as
        # if it were itself a claim. Without this, an ACK that quotes the
        # effective value invents a second, conflicting claim.
        claim_source = ACK_RE.sub("", cell) if ACK_RE.search(cell) else cell
        claimed = CLAIM_RE.findall(claim_source)
        if not claimed:
            for ref in sorted(set(UNIT_NAME_RE.findall(cell))):
                target = units.get(ref)
                if target is not None and any(k in target.props for k in MEM_KEYS):
                    continue  # a real slice with a real limit: nothing to say
                if target is None and ref not in SYSTEM_PROVIDED:
                    msg = (f"names {ref} as the enforcement, but this repository "
                           f"ships no such unit and no drop-in creates one")
                elif target is None:
                    msg = (f"names {ref} as the enforcement; systemd provides it "
                           f"and nothing here sets a limit on it")
                else:
                    msg = (f"names {ref} as the enforcement, but that slice sets no "
                           f"memory limit")
                out.append(Finding("ACKNOWLEDGED" if ack else "ERROR", where, msg))
            if ack and not UNIT_NAME_RE.findall(cell):
                out.append(Finding("ACKNOWLEDGED", where, f"gap recorded: {ack}"))
            continue

        mapped = resolve_component(component)
        if mapped is None:
            out.append(Finding(
                "ERROR", where,
                "claims a memory limit but has no unit mapping; add one to "
                "COMPONENT_UNITS or drop the claim"))
            continue

        for key, want in claimed:
            # Resolve the claim in the unit's own slice chain.
            chain: list[Unit] = []
            for uname in mapped:
                unit = units.get(uname)
                if unit is None:
                    continue
                seen = set()
                current: Unit | None = unit
                while current is not None and current.name not in seen:
                    seen.add(current.name)
                    chain.append(current)
                    parent = current.parent_slice
                    current = units.get(parent) if parent else None

            on_chain = sorted(u.name for u in chain if u.mem(key) == want)
            elsewhere = sorted(u.name for u in units.values()
                               if u.mem(key) == want and u not in chain)
            unshipped = sorted(m for m in mapped if m not in units)

            if not on_chain:
                reason = (f"the value exists in the tree on {', '.join(elsewhere)}, "
                          f"none of which govern {component}" if elsewhere else
                          "no shipped unit or drop-in configures it")
                if unshipped:
                    reason += (f" ({', '.join(unshipped)} come from the distribution, "
                               f"so a drop-in under image/mkosi.extra is what would "
                               f"have to carry it)")
                sev = "ACKNOWLEDGED" if ack else "ERROR"
                tail = f" ... acknowledged: {ack}" if ack else \
                    "; configure it, or write 'TARGET (not enforced: ...)' in the cell"
                out.append(Finding(
                    sev, where,
                    f"claims {key}={want}, but it is not configured for the unit the "
                    f"ledger names: {reason}{tail}"))
                continue

            # Even an acknowledged claim is resolved, so the divergence it
            # admits to stays visible in every run instead of only in the text.

            divergences = []
            want_n = size_bytes(want)
            for uname in mapped:
                unit = units.get(uname)
                if unit is None:
                    continue
                got, origin = effective(units, unit, key)
                if got != want_n:
                    divergences.append(
                        f"{uname}: effective {key}={human(got)} (from {origin}) "
                        f"!= claimed {want}")
            if divergences:
                sev = "ACKNOWLEDGED" if ack else "ERROR"
                tail = f" ... acknowledged: {ack}" if ack else \
                    "; change the unit or acknowledge the divergence in the cell"
                out.append(Finding(
                    sev, where,
                    f"{key}={want} is configured on {', '.join(on_chain)} but is not "
                    f"what applies: " + "; ".join(divergences) + tail))

    total = total_row(ledger)
    if total is not None and declared != total:
        out.append(Finding(
            "ERROR", f"{ledger.name} (TOTAL row)",
            f"declares {total} MB but the budget column sums to {declared} MB"))
    if unset_budget:
        out.append(Finding(
            "INFO", f"{ledger.name} (TOTAL row)",
            "resident units with no budget yet, excluded from the sum: "
            + ", ".join(unset_budget)))

    # --- every shipped cap must be accounted for ----------------------------
    listed = set()
    for row in rows:
        listed.update(UNIT_NAME_RE.findall(row.enforced))
        listed.update(UNIT_NAME_RE.findall(" ".join(row.cells[4:])))
        mapped = resolve_component(row.component)
        if mapped:
            listed.update(mapped)
    for row in capped_rows(ledger):
        listed.add(row.cells[0].strip())
    for name, props in dropins.items():
        if any(k in props for k in MEM_KEYS):
            listed.add(name)

    for unit in sorted(units.values(), key=lambda u: u.name):
        if not any(k in unit.props for k in MEM_KEYS) or unit.name in listed:
            continue
        out.append(Finding(
            "ERROR", unit.name,
            "ships a memory limit the ledger does not list; add it to the idle "
            "table or to the non-idle table so the budget accounts for it"))

    # --- slices with limits that nothing joins ------------------------------
    ack_for: dict[str, str] = {}
    for row in rows:
        mapped = resolve_component(row.component) or []
        ack = ack_text(row.enforced)
        if ack:
            for uname in mapped:
                piece = units.get(uname)
                if piece is not None and piece.kind == "slice":
                    ack_for[uname] = f"{ledger.name}:{row.lineno}"

    for unit in sorted(units.values(), key=lambda u: u.name):
        if unit.kind != "slice" or not any(k in unit.props for k in MEM_KEYS):
            continue
        members = [u.name for u in units.values()
                   if u.name != unit.name and u.parent_slice == unit.name]
        if members:
            continue
        if unit.name in ack_for:
            out.append(Finding(
                "ACKNOWLEDGED", unit.name,
                f"carries memory limits but no shipped unit joins it, so nothing "
                f"is governed by them; acknowledged at {ack_for[unit.name]}"))
        else:
            out.append(Finding(
                "ERROR", unit.name,
                "carries memory limits but no shipped unit joins it, so nothing is "
                f"governed by them; a unit must set Slice={unit.name} to be covered, "
                "or the limits should be removed"))

    order = {"ERROR": 0, "ACKNOWLEDGED": 1, "INFO": 2}
    out.sort(key=lambda f: (order.get(f.severity, 9), str(f)))
    return out


# ---------------------------------------------------------------------------
# self-test
# ---------------------------------------------------------------------------

LEDGER_OK = """\
# fake ledger

| # | Component | Budget | Enforced by | Notes |
|---|---|---:|---|---|
| 1 | litric-shell | 110 MB | `MemoryMax=140M` | |
| 2 | portals | 0 MB idle | dbus-activated | |
| | **TOTAL** | **110** | | |
"""

UNITS_OK = {
    "litric-shell.service":
        "[Unit]\nDescription=x\n[Service]\nSlice=litric-shell.slice\nExecStart=/bin/true\n",
    "litric-shell.slice":
        "[Unit]\nDescription=x\n[Slice]\nMemoryAccounting=yes\nMemoryMax=140M\n",
}


def _fixture(root: Path, ledger_text: str, units: dict[str, str]):
    ledger = root / "docs" / "PERF-budget.md"
    ledger.parent.mkdir(parents=True, exist_ok=True)
    ledger.write_text(ledger_text, encoding="utf-8")
    unit_dir = root / "unit"
    unit_dir.mkdir(parents=True, exist_ok=True)
    for name, body in units.items():
        (unit_dir / name).write_text(body, encoding="utf-8")
    return ledger, unit_dir


def self_test() -> int:
    rc = 0
    cases: list[tuple[str, str, dict[str, str], str, str]] = [
        # name, ledger transformation, unit changes, expected severity, needle
        ("consistent fixture", "none", {}, "", ""),
        ("ungrounded claim", "claim64", {}, "ERROR", "no shipped unit"),
        ("claim not effective for the unit", "dejoin", {}, "ERROR", "not what applies"),
        ("acknowledged divergence", "dejoin+ack", {}, "ACKNOWLEDGED", "acknowledged"),
        ("commented-out directive", "none", {"litric-shell.slice": "commented"}, "ERROR",
         "no shipped unit"),
        ("undeclared cap", "none", {"litric-other.service": "cap"}, "ERROR",
         "ledger does not list"),
        ("bare-byte limit", "none", {"litric-shell.slice": "barebytes"}, "ERROR", "bytes"),
        ("wrong section", "none", {"litric-shell.slice": "wrongsection"}, "ERROR",
         "belongs in"),
        ("arithmetic mismatch", "sum", {}, "ERROR", "sums to"),
        ("empty unit tree", "none", {"__empty__": ""}, "ERROR", "no unit files"),
    ]
    for name, transform, unit_changes, want_sev, needle in cases:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger_text = LEDGER_OK
            if transform == "claim64":
                ledger_text = LEDGER_OK.replace("`MemoryMax=140M`", "`MemoryMax=64M`")
            elif transform == "dejoin":
                ledger_text = LEDGER_OK.replace("Slice=litric-shell.slice\n", "")
            elif transform == "dejoin+ack":
                ledger_text = LEDGER_OK.replace(
                    "`MemoryMax=140M`",
                    "`MemoryMax=140M` [ACK: slice membership pending — BUG-0008]")
            elif transform == "sum":
                ledger_text = LEDGER_OK.replace("**110**", "**99**")

            units = dict(UNITS_OK)
            if "dejoin" in transform:
                units["litric-shell.service"] = "[Service]\nExecStart=/bin/true\n"
            for uname, kind in unit_changes.items():
                if uname == "__empty__":
                    continue
                if kind == "commented":
                    units[uname] = "[Slice]\n# MemoryMax=140M\nMemoryAccounting=yes\n"
                elif kind == "cap":
                    units[uname] = "[Service]\nMemoryMax=64M\nExecStart=/bin/true\n"
                elif kind == "barebytes":
                    units[uname] = "[Slice]\nMemoryMax=140\nMemoryAccounting=yes\n"
                elif kind == "wrongsection":
                    units[uname] = "[Unit]\nMemoryMax=140M\n[Slice]\nMemoryAccounting=yes\n"
            if "__empty__" in unit_changes:
                units = {}

            ledger, unit_dir = _fixture(root, ledger_text, units)
            findings = check(root, ledger, unit_dir)
            if want_sev == "":
                ok = not findings
                detail = "; ".join(str(f) for f in findings)
            else:
                hits = [f for f in findings if f.severity == want_sev and needle in f.message]
                ok = bool(hits)
                detail = "; ".join(str(f) for f in findings[:3])
            print(f"{'PASS' if ok else 'FAIL'}  self-test: {name}"
                  + ("" if ok else f"\n        {detail}"))
            rc |= 0 if ok else 1
    return rc


# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------

def main(argv: list[str]) -> int:
    _safe_stdout()
    if "--self-test" in argv:
        return self_test()

    findings = check(ROOT, LEDGER, UNIT_DIR)
    counts: dict[str, int] = {}
    for f in findings:
        print(f)
        counts[f.severity] = counts.get(f.severity, 0) + 1

    units = load_units(UNIT_DIR)
    capped = [u for u in units.values() if any(k in u.props for k in MEM_KEYS)]
    print()
    print(f"units discovered : {len(units)} "
          f"({sum(1 for u in units.values() if u.kind == 'slice')} slices, "
          f"{len(capped)} with a memory limit)")
    for sev in ("ERROR", "ACKNOWLEDGED", "INFO"):
        if counts.get(sev):
            print(f"{sev:<16} : {counts[sev]}")

    strict = "--strict" in argv
    errors = counts.get("ERROR", 0)
    acked = counts.get("ACKNOWLEDGED", 0)
    if errors or (strict and acked):
        print("\nRESULT: FAIL -- static policy check only; says nothing about "
              "measured runtime memory")
        return 1
    print("\nRESULT: PASS -- static policy only: every ledger claim is configured, "
          "every shipped cap is accounted for, and every divergence is "
          "acknowledged with a review condition")
    print("NOT VERIFIED: whether any component stays within its limit at runtime; "
          "that needs a booted image and a cgroup reader.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
