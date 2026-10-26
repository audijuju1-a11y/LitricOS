#!/usr/bin/env python3
"""Check the parity ledger: one row per research item, and no inert claims.

`docs/parity/ledger.csv` is the record of what LitricOS decided to do about each
surface the UI/UX research catalogued. It is only worth having if it cannot
quietly become a wish list, so this linter enforces the five rules the ledger's
own README states:

  1. The ten columns, in order. A ledger with a column nobody validates is a
     ledger nobody is reading.
  2. The cited research item exists in the coverage checklist and is not
     `PENDING`. A row citing unfinished research has no source: it is an idea
     wearing a citation.
  3. `ADOPT` and `ADAPT` name a backend other than `none`. No inert control -- the
     gate matrix's own condition, and the defect that produced the Back/Forward
     buttons that did nothing.
  4. `ADAPT`, `REJECT` and `DEFER` carry a justification of at least 40
     characters: enough to say why, not enough to write an essay.
  5. The evidence can be opened. A `STATIC` path must exist; a `UNIT` test name
     must appear as a `fn` in the tree. This is `ci/lint-doc-refs.py`'s lesson
     applied to a second kind of claim.

Plus two cheap ones that catch real mistakes: `research_id` is unique, and a
non-empty `store_key` is a key the shell's preference store actually writes. A
store key that only a document mentions is a setting that only a document has.

This linter cannot tell whether a backend named `litric-shell::seek::search`
really does what the justification says. It checks that the claim is
well-formed, sourced and openable -- not that it is true. The row's evidence
column is how a reader gets from here to the code.

Usage:
    python3 ci/lint-parity.py              check the ledger
    python3 ci/lint-parity.py --self-test  prove the checker can fail
    python3 ci/lint-parity.py --list       print the decision counts
"""

from __future__ import annotations

import csv
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

LEDGER = ROOT / "docs" / "parity" / "ledger.csv"
CHECKLIST = ROOT / "Research-Deep-all" / "LitricOS Coverage Checklist.csv"
PREFSTORE = ROOT / "src" / "litric-shell" / "src" / "prefstore.rs"
SHELL_SRC = ROOT / "src"

COLUMNS = [
    "research_id",
    "platform",
    "surface",
    "decision",
    "justification",
    "backend",
    "store_key",
    "evidence",
    "milestone",
]

DECISIONS = {"ADOPT", "ADAPT", "REJECT", "DEFER"}
# Decisions whose whole point is that a real thing happens when you use them.
NEEDS_BACKEND = {"ADOPT", "ADAPT"}
# Decisions that must explain themselves.
NEEDS_REASON = {"ADAPT", "REJECT", "DEFER"}
# Research statuses that are finished enough to build from. PENDING is not.
CITABLE = {"COMPLETE", "PARTIAL"}

MIN_REASON = 40

SURFACE_RE = re.compile(r"^[a-z][a-z0-9]*(?:[/-][a-z0-9]+)*(?:/[a-z][a-z0-9-]*)*$")
STORE_KEY_RE = re.compile(r"^[a-z][a-z0-9_]*$")
MILESTONE_RE = re.compile(r"^M[1-6]$")
EVIDENCE = {
    "STATIC": re.compile(r"^STATIC: \S.*$"),
    "UNIT": re.compile(r"^UNIT: [A-Za-z0-9_:]+$"),
    "CONTRACT": re.compile(r"^CONTRACT: \S.*$"),
    "VM E2E": re.compile(r"^VM E2E: \S.*$"),
}


class Finding:
    def __init__(self, line: int, research_id: str, message: str) -> None:
        self.line, self.research_id, self.message = line, research_id, message

    def __str__(self) -> str:
        where = f"ledger.csv:{self.line}" if self.line else "ledger.csv"
        return f"  {where}: {self.research_id or '(no id)'}: {self.message}"


def checklist_status() -> dict[str, str]:
    """`id -> status` from the research coverage checklist."""
    out: dict[str, str] = {}
    with CHECKLIST.open(encoding="utf-8-sig", newline="") as handle:
        for row in csv.DictReader(handle):
            item = (row.get("id") or "").strip()
            if item:
                out[item] = (row.get("status") or "").strip().upper()
    return out


def store_keys() -> set[str]:
    """Keys `prefstore::encode` writes, read from the source of truth."""
    text = PREFSTORE.read_text(encoding="utf-8", errors="replace")
    return set(re.findall(r'out\.push_str\("([a-z_]+)="\)', text))


def rust_source() -> str:
    """Every line of Rust in the workspace, for test-name lookups."""
    chunks: list[str] = []
    for path in SHELL_SRC.rglob("*.rs"):
        if "target" in path.parts:
            continue
        chunks.append(path.read_text(encoding="utf-8", errors="replace"))
    return "\n".join(chunks)


class Context:
    """Everything a row is checked against.

    Gathered once rather than per row: the checklist has 597 entries and the
    Rust tree is a few megabytes, and a linter that takes a minute is a linter
    people stop running.
    """

    def __init__(
        self,
        statuses: dict[str, str],
        keys: set[str],
        source: str,
        root: Path,
    ) -> None:
        self.statuses = statuses
        self.keys = keys
        self.source = source
        self.root = root

    @classmethod
    def load(cls) -> "Context":
        return cls(checklist_status(), store_keys(), rust_source(), ROOT)

    def test_exists(self, name: str) -> bool:
        return re.search(rf"\bfn\s+{re.escape(name)}\s*\(", self.source) is not None

    def path_exists(self, token: str) -> bool:
        return (self.root / token).exists()


def check_rows(
    rows: list[dict[str, str]],
    header: list[str],
    ctx: Context,
) -> list[Finding]:
    """Validate parsed rows. Pure: the self-test calls this with fixtures."""
    findings: list[Finding] = []

    if header != COLUMNS:
        findings.append(
            Finding(
                0,
                "",
                f"columns are {header}, expected {COLUMNS}",
            )
        )

    seen: dict[str, int] = {}
    for index, row in enumerate(rows, start=2):
        rid = (row.get("research_id") or "").strip()
        decision = (row.get("decision") or "").strip()
        justification = (row.get("justification") or "").strip()
        backend = (row.get("backend") or "").strip()
        store_key = (row.get("store_key") or "").strip()
        evidence = (row.get("evidence") or "").strip()
        surface = (row.get("surface") or "").strip()
        milestone = (row.get("milestone") or "").strip()
        platform = (row.get("platform") or "").strip()

        def bad(message: str) -> None:
            findings.append(Finding(index, rid, message))

        if not rid:
            bad("no research id")
            continue

        # Rule 2: the item exists and is finished enough to cite.
        status = ctx.statuses.get(rid)
        if status is None:
            bad("no such item in the coverage checklist")
        elif status not in CITABLE:
            bad(
                f"the checklist says {status}; only {sorted(CITABLE)} can be "
                "cited, because a PENDING item has not been researched"
            )

        if rid in seen:
            bad(f"a second row for the same item (first on line {seen[rid]})")
        else:
            seen[rid] = index

        if not platform:
            bad("no platform")
        if not SURFACE_RE.match(surface):
            bad(f"surface {surface!r} is not a lower-case path like settings/appearance")

        # Rule 1 (value side): the decision is one of the four.
        if decision not in DECISIONS:
            bad(f"decision {decision!r} is not one of {sorted(DECISIONS)}")

        # Rule 3: no inert control.
        if decision in NEEDS_BACKEND and backend in ("", "none"):
            bad(
                f"{decision} with backend {backend or '(empty)'!r}: an adopted "
                "behaviour must name the API that makes it work"
            )

        # Rule 4: a reason, or it did not happen.
        if decision in NEEDS_REASON and len(justification) < MIN_REASON:
            bad(
                f"{decision} needs a justification of at least {MIN_REASON} "
                f"characters, got {len(justification)}"
            )

        # Rule 5: the evidence can be opened.
        kind = next((k for k in EVIDENCE if evidence.startswith(k + ":")), None)
        if kind is None:
            bad(f"evidence {evidence!r} does not start with one of {sorted(EVIDENCE)}")
        elif not EVIDENCE[kind].match(evidence):
            bad(f"evidence {evidence!r} is not well formed for {kind}")
        elif kind == "STATIC":
            token = evidence.split(":", 1)[1].strip()
            if not ctx.path_exists(token):
                bad(f"STATIC evidence {token!r} is not a file in this repository")
        elif kind == "UNIT":
            name = evidence.split(":", 1)[1].strip().rsplit("::", 1)[-1]
            if not ctx.test_exists(name):
                bad(f"UNIT evidence {name!r} is not a test function in the tree")

        # Store keys must be keys the store writes.
        if store_key and not STORE_KEY_RE.match(store_key):
            bad(f"store_key {store_key!r} is not a settings.conf key")
        elif store_key and store_key not in ctx.keys:
            bad(
                f"store_key {store_key!r} is not written by the preference store; "
                "the setting would exist only in this document"
            )

        if not MILESTONE_RE.match(milestone):
            bad(f"milestone {milestone!r} is not M1..M6")

    return findings


def read_ledger(path: Path) -> tuple[list[dict[str, str]], list[str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.reader(handle)
        try:
            header = next(reader)
        except StopIteration:
            return [], []
        rows = [dict(zip(header, row)) for row in reader if row]
    return rows, header


def counts(rows: list[dict[str, str]]) -> dict[str, int]:
    out: dict[str, int] = {}
    for row in rows:
        decision = (row.get("decision") or "").strip() or "(none)"
        out[decision] = out.get(decision, 0) + 1
    return out


def check() -> int:
    if not LEDGER.exists():
        print(f"no ledger at {LEDGER.relative_to(ROOT)}")
        return 1
    rows, header = read_ledger(LEDGER)
    ctx = Context.load()
    findings = check_rows(rows, header, ctx)

    for decision, count in sorted(counts(rows).items()):
        print(f"  {decision:6} {count:3}")
    print(f"  {'total':6} {len(rows):3} rows, {len(ctx.statuses)} research items in the checklist")

    if findings:
        print(f"\n{len(findings)} problem(s) in the parity ledger:")
        for finding in findings:
            print(finding)
        print(
            "\nA row is a decision, not a hope: name the backend that makes it "
            "real, or decide DEFER and say what is blocking it. See "
            "docs/parity/README.md."
        )
        return 1
    return 0


def self_test() -> int:
    """Prove the checker fails on each defect it claims to catch.

    A linter that has never been seen to fail is a linter nobody knows is
    running. Every case below is a real mistake this project has made in some
    form, or a way a ledger row could quietly become a wish.
    """
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs").mkdir()
        (root / "docs" / "exists.md").write_text("x", encoding="utf-8")
        ctx = Context(
            statuses={"WB.01": "PARTIAL", "WB.02": "PENDING", "WB.03": "COMPLETE"},
            keys={"theme", "accent"},
            source="fn a_test() {}\nfn another_test() {}\n",
            root=root,
        )

        def row(**over: str) -> dict[str, str]:
            base = {
                "research_id": "WB.01",
                "platform": "windows-26h2",
                "surface": "settings/appearance",
                "decision": "ADOPT",
                "justification": "",
                "backend": "litric-shell::settings",
                "store_key": "theme",
                "evidence": "STATIC: docs/exists.md",
                "milestone": "M1",
            }
            base.update(over)
            return base

        good = row()
        if check_rows([good], COLUMNS, ctx):
            print("self-test: a valid row was rejected")
            return 1

        cases = [
            ("wrong column order", COLUMNS[:-1] + ["evidence2"], [good]),
            ("pending research cited", COLUMNS, [row(research_id="WB.02")]),
            ("unknown research id", COLUMNS, [row(research_id="WB.99")]),
            ("duplicate row", COLUMNS, [good, row()]),
            ("inert ADOPT", COLUMNS, [row(backend="none")]),
            ("inert ADAPT", COLUMNS, [row(decision="ADAPT", backend="", justification="a" * 50)]),
            ("ADAPT with a one-word excuse", COLUMNS, [row(decision="ADAPT", justification="because")]),
            ("DEFER without a reason", COLUMNS, [row(decision="DEFER", backend="none")]),
            ("REJECT without a reason", COLUMNS, [row(decision="REJECT", backend="none", justification="no")]),
            ("evidence names a missing file", COLUMNS, [row(evidence="STATIC: docs/missing.md")]),
            ("evidence names a missing test", COLUMNS, [row(evidence="UNIT: registry::tests::nope")]),
            ("evidence has no kind", COLUMNS, [row(evidence="docs/exists.md")]),
            ("store key the store never writes", COLUMNS, [row(store_key="night_light")]),
            ("bad milestone", COLUMNS, [row(milestone="M9")]),
            ("bad surface", COLUMNS, [row(surface="Settings Appearance")]),
            ("bad decision", COLUMNS, [row(decision="MAYBE")]),
        ]
        failures = 0
        for name, header, rows in cases:
            found = check_rows(rows, header, ctx)
            if not found:
                print(f"self-test: {name} was accepted")
                failures += 1
            else:
                print(f"  caught: {name}")

        # The positive controls: the forms that must be accepted.
        for name, over in [
            ("ADAPT with a reason", {"decision": "ADAPT", "justification": "no DRM on this image, so the mode is read only"}),
            ("DEFER with a reason", {"decision": "DEFER", "backend": "none", "justification": "blocked on an audio server that is not in the tree yet"}),
            ("REJECT with a reason", {"decision": "REJECT", "backend": "none", "justification": "copying the vendor's icon set is trade dress, not behaviour"}),
            ("UNIT evidence", {"evidence": "UNIT: registry::tests::a_test"}),
            ("empty store key", {"store_key": ""}),
        ]:
            if check_rows([row(**over)], COLUMNS, ctx):
                print(f"self-test: {name} was rejected")
                failures += 1

        if failures:
            return 1
        print("self-test: 16 defects caught, 5 valid forms accepted")
        return 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    if "--list" in sys.argv:
        rows, _ = read_ledger(LEDGER)
        for decision, count in sorted(counts(rows).items()):
            print(f"{decision},{count}")
        return 0
    return check()


if __name__ == "__main__":
    sys.exit(main())
