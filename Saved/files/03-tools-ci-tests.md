# Batch 03 — CI Linter Suite, Top-Level Boot/Harness Tests, and Developer/VM Tooling

Total files documented in this file: **55**

---

## F0105 — `ci/check-golden-frames.py`

### A. Identity
- **Path**: `ci/check-golden-frames.py`
- **Type / Language**: `.py` (Python)
- **Size**: 18,268 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Compare dumped desktop frames with the recorded golden set. WHAT THIS GATE IS ----------------- `litric-shell`'s `frame_dump` example composes the *shipping* desktop -- the same `views::draw` and `desktop::wallpaper` that `main.rs` calls on a booted machine -- into an in-memory canvas and writes each scene as a P6 PPM. This script turns
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 461 lines
- **Key Declarations / Directives / Functions**:
  - L103: `class FrameError(Exception):`
  - L107: `def read_ppm(path: Path) -> tuple[int, int, bytes]:`
  - L142: `def grid_of(width: int, height: int, pixels: bytes) -> list[int]:`
  - L169: `def fingerprint_of(width: int, height: int, pixels: bytes) -> dict[str, object]:`
  - L185: `def fnv1a(data: bytes) -> str:`
  - L193: `def compare(golden: dict[str, object], fresh: dict[str, object]) -> tuple[float, float]:`
  - L212: `def load_frames(directory: Path) -> dict[str, dict[str, object]]:`
  - L220: `def describe_change(golden: dict[str, object], fresh: dict[str, object], limit: int = 3) -> str:`
  - L238: `def write_ppm(path: Path, width: int, height: int, pixels: bytes) -> None:`
  - L243: `def self_test() -> int:`
  - L300: `def self_test_report(failures: list[str]) -> int:`
  - L313: `def main() -> int:`
  - L422: `def report(`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/check-golden-frames.py`, `docs/evidence/golden/fingerprints.json`, `tools/photo-tour`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0106 — `ci/check-memory-budget.py`

### A. Identity
- **Path**: `ci/check-memory-budget.py`
- **Type / Language**: `.py` (Python)
- **Size**: 29,763 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Check the memory-budget ledger against the units this repository actually ships. WHAT THIS CAN AND CANNOT PROVE ------------------------------ Static policy presence, only. It reads `docs/PERF-budget.md` and the systemd units under `image/mkosi.extra/`, and answers one question: *is every memory limit the ledger claims to enforce actually configured for the unit the ledger
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 722 lines
- **Key Declarations / Directives / Functions**:
  - L105: `def _safe_stdout() -> None:`
  - L118: `class Finding:`
  - L119: `def __init__(self, severity: str, where: str, message: str) -> None:`
  - L122: `def __str__(self) -> str:`
  - L130: `class Unit:`
  - L131: `def __init__(self, name: str, kind: str) -> None:`
  - L138: `def mem(self, key: str) -> str | None:`
  - L142: `def parent_slice(self) -> str:`
  - L153: `def _strip_comment(line: str) -> str:`
  - L164: `def parse_unit(path: Path, name: str | None = None) -> Unit:`
  - L188: `def load_units(unit_dir: Path) -> dict[str, Unit]:`
  - L198: `def parse_dropins(unit_dir: Path) -> dict[str, dict[str, str]]:`
  - L212: `def size_bytes(value: str) -> int | None:`
  - L221: `def human(n: int | None) -> str:`
  - L230: `def effective(units: dict[str, Unit], unit: Unit, key: str) -> tuple[int | None, str]:`
  - L250: `class Row:`
  - L251: `def __init__(self, lineno: int, cells: list[str]) -> None:`
  - L255: `def component(self) -> str:`
  - L260: `def enforced(self) -> str:`
  - L264: `def unit_refs(self) -> list[str]:`
  - L269: `def resolve_component(component: str) -> list[str] | None:`
  - L282: `def idle_rows(path: Path) -> list[Row]:`
  - L304: `def total_row(path: Path) -> int | None:`
  - L313: `def capped_rows(path: Path) -> list[Row]:`
  - L334: `def ack_text(cell: str) -> str | None:`
  - L347: `def check(root: Path, ledger: Path, unit_dir: Path) -> list[Finding]:`
  - L605: `def _fixture(root: Path, ledger_text: str, units: dict[str, str]):`
  - L616: `def self_test() -> int:`
  - L685: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/check-memory-budget.py`, `docs/PERF-budget.md`, `image/mkosi.extra`, `image/mkosi.extra/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0107 — `ci/check-python-syntax.py`

### A. Identity
- **Path**: `ci/check-python-syntax.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,841 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Parse every Python file this repository ships, without writing bytecode. Two reasons this exists instead of `python -m py_compile`: 1. `py_compile` writes `__pycache__/` into the tree it is checking. A verification step should not add files to the repository it verifies -- the `tests/__pycache__/smoke_boot.cpython-314.pyc` in this tree is exactly that residue, and it predates this file.
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 115 lines
- **Key Declarations / Directives / Functions**:
  - L36: `def files() -> list[Path]:`
  - L49: `def label(path: Path) -> str:`
  - L63: `def parse(path: Path) -> str | None:`
  - L74: `def self_test() -> int:`
  - L95: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/check-python-syntax.py`, `tests/__pycache__/smoke_boot.cpython-314.pyc`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0108 — `ci/count-scope.py`

### A. Identity
- **Path**: `ci/count-scope.py`
- **Type / Language**: `.py` (Python)
- **Size**: 6,319 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Verify that the SYSTEM-SCOPE tally matches the rows it summarises. A hand-maintained progress number drifts, and a drifting progress number is how a project starts believing it is further along than it is. This parses the rows and fails if the summary table disagrees. It caught its own motivating bug immediately: the first version of the tally was written before the rows were finished and claimed 707 rows across 4 status
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 173 lines
- **Key Declarations / Directives / Functions**:
  - L46: `def parse_rows(text: str) -> list[tuple[int, str]]:`
  - L60: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/count-scope.py`, `docs/EVIDENCE.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0109 — `ci/dep-policy-allow.txt`

### A. Identity
- **Path**: `ci/dep-policy-allow.txt`
- **Type / Language**: `.txt` (Plaintext)
- **Size**: 789 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Dependencies that a workspace member is allowed to take from outside this workspace, one `crate: dependency` pair per line. This file is empty on purpose. `src/Cargo.toml` states that the core crates take no third-party dependencies at all, and every dependency in every member is a path dependency today, so nothing needs an exception. A line here is a decision, not a convenience: it means someone accepted that a
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 15 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/adr/0005-compositor.md`, `src/Cargo.toml`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0110 — `ci/lint-deps.py`

### A. Identity
- **Path**: `ci/lint-deps.py`
- **Type / Language**: `.py` (Python)
- **Size**: 17,858 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Fail when a workspace member that must stay dependency-free gains a dependency. `src/Cargo.toml` opens with a HARD RULE in prose: the crates in its `core-crates` group have ZERO third-party dependencies, std only. Two reasons are given -- every dependency of a system component is code that runs with that component's authority (the capability broker does not get to pull 200 transitive crates from the internet), and those crates must compile and test on a Windows host with no
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 410 lines
- **Key Declarations / Directives / Functions**:
  - L80: `class Member:`
  - L81: `def __init__(self, name: str, group: str, group_text: str, covered: bool, why: str) -> None:`
  - L90: `def members_of(manifest_text: str) -> tuple[list[tuple[str, str, str]], str | None]:`
  - L123: `def is_dependency_table(table: str) -> bool:`
  - L129: `def parse(manifest: Path, member: Member, member_dirs: set[str]) -> None:`
  - L173: `def load_allow() -> set[str]:`
  - L184: `def classify(manifest: Path, name: str, group: str, group_text: str) -> Member:`
  - L200: `def collect(root: Path) -> tuple[list[Member], list[str]]:`
  - L225: `def check() -> int:`
  - L273: `def fixture(root: Path) -> Path:`
  - L306: `def verdict(src: Path, name: str) -> tuple[list[str], list[str]]:`
  - L320: `def self_test() -> int:`
  - L327: `def write(name: str, body: str) -> None:`
  - L332: `def expect(what: str, name: str, want: bool, needle: str = "") -> None:`
  - L403: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/dep-policy-allow.txt`, `ci/lint-deps.py`, `docs/master-plan/24-feature-parity.md`, `src/Cargo.toml`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0111 — `ci/lint-doc-refs.py`

### A. Identity
- **Path**: `ci/lint-doc-refs.py`
- **Type / Language**: `.py` (Python)
- **Size**: 8,105 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Verify that files cited in documentation actually exist. This is the regression test for the defect pattern this project keeps hitting, recorded four times before it was mechanised: * BUG-0001 -- `ci/lint-config.sh` does not exist; the linter is `ci/lint-mkosi.py`. The bug report cites the wrong name four times. * BUG-0002 -- `tests/smoke_boot.py` was listed as a completed deliverable and
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 215 lines
- **Key Declarations / Directives / Functions**:
  - L94: `class Citation:`
  - L95: `def __init__(self, path: Path, line: int, token: str, status: str) -> None:`
  - L98: `def __str__(self) -> str:`
  - L103: `def documents() -> list[Path]:`
  - L113: `def citations_in(path: Path, text: str) -> list[Citation]:`
  - L138: `def resolve(token: str, doc: Path) -> Path | None:`
  - L146: `def check() -> int:`
  - L177: `def self_test() -> int:`
  - L208: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/lint-config.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-safety.sh`, `image/mkosi.extra/`, `kernel/patches/`, `src/ui.rs`, `tests/budget/`, `tests/budget/test_mem.py`, `tests/smoke_boot.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0112 — `ci/lint-file-safety.py`

### A. Identity
- **Path**: `ci/lint-file-safety.py`
- **Type / Language**: `.py` (Python)
- **Size**: 16,715 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Check that every file-deletion call site lives in a function someone reviewed. Why this exists. The workspace has no runtime test for the file layer: no booted image, no compiled crate, no QEMU (see docs/audit/2026-10-09-continuation-recovery-audit.md). What it does have is a small number of places that destroy data, and a set of defects found in them (BUG-0010, BUG-0011, BUG-0012). A new `fs::remove_dir_all` appearing in an unreviewed function is therefore a change
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 403 lines
- **Key Declarations / Directives / Functions**:
  - L98: `def blank(chunk):`
  - L113: `def code_only(text):`
  - L198: `def scan_text(text, path):`
  - L232: `def collect(root):`
  - L245: `def decide(hits, allowed):`
  - L258: `def report(allowed_hits, violations):`
  - L347: `def self_test():`
  - L378: `def main(argv):`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/lint-file-safety.py`, `docs/audit/2026-10-09-continuation-recovery-audit.md`, `src/a.rs`, `src/litric-bench/src/bin/litricos-test.rs`, `src/litric-browser/src/bin/browserd.rs`, `src/litric-files/src/ops.rs`, `src/litric-files/src/server.rs`, `src/litric-files/src/trash.rs`, `src/litric-shell/examples/render_bench.rs`, `src/litric-shell/src/e2e.rs`, `src/litric-shell/src/fsbridge.rs`, `src/litric-shell/src/main.rs`, `src/litric-sysd/src/providers.rs`, `src/litric-sysd/src/server.rs`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0113 — `ci/lint-kconfig.py`

### A. Identity
- **Path**: `ci/lint-kconfig.py`
- **Type / Language**: `.py` (Python)
- **Size**: 12,855 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Validate the LitricOS kernel configuration fragment. A Kconfig fragment is merged with `scripts/kconfig/merge_config.sh`, which on a duplicate symbol takes the LAST value and prints a warning that nobody reads in a build log. That makes a duplicated symbol with two different values a silent misconfiguration, and it is exactly how this repository ended up shipping CONFIG_LSM=""
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 324 lines
- **Key Declarations / Directives / Functions**:
  - L110: `class Finding:`
  - L111: `def __init__(self, path: Path, line: int, severity: str, message: str) -> None:`
  - L114: `def __str__(self) -> str:`
  - L118: `def lint(path: Path) -> list[Finding]:`
  - L221: `def cross_check_cmdline(fragment: Path, mkosi_conf: Path) -> list[Finding]:`
  - L264: `def self_test() -> int:`
  - L298: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/lint-kconfig.py`, `docs/bugs/BUG-0003.`, `image/mkosi.conf`, `image/mkosi.conf.`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0114 — `ci/lint-line-endings.py`

### A. Identity
- **Path**: `ci/lint-line-endings.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,964 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Detect CRLF line endings in files destined for a Linux system. Every file in this repository that a Linux kernel, shell or systemd will read must use LF. A single carriage return at the end of a line causes failures that look like anything except what they are: * `set -euo pipefail\\r` -> bash reports "pipefail: invalid option name" * `#!/usr/bin/env bash\\r` -> "bad interpreter: no such file or directory",
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 125 lines
- **Key Declarations / Directives / Functions**:
  - L67: `def candidates() -> list[Path]:`
  - L84: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `ci/lint-line-endings.py`, `image/mkosi.extra/usr/lib/litric/mm-tune`, `keys/dev/README`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0115 — `ci/lint-mkosi.py`

### A. Identity
- **Path**: `ci/lint-mkosi.py`
- **Type / Language**: `.py` (Python)
- **Size**: 22,654 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Validate mkosi configuration against the real mkosi schema. This is the regression test for BUG-0001, in which a configuration file was written from recollection of what an image builder's schema "should" look like, using sections that do not exist. It failed at parse time and nothing it described was ever built -- yet a gate was recorded as passed on its existence. The test must FAIL against the pre-v2 file preserved in .backup-pre-v2/ and PASS
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 533 lines
- **Key Declarations / Directives / Functions**:
  - L110: `class Finding:`
  - L111: `def __init__(self, path: Path, line: int, severity: str, message: str) -> None:`
  - L114: `def __str__(self) -> str:`
  - L182: `def check_path_value(path: Path, lineno: int, key: str, value: str) -> list[Finding]:`
  - L205: `def lint(path: Path) -> list[Finding]:`
  - L283: `def read_list_setting(path: Path, keys: set[str]) -> dict[str, list[str]]:`
  - L323: `def check_module_sets(paths: list[Path]) -> list[Finding]:`
  - L406: `def self_test() -> int:`
  - L497: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `/usr/lib/modules`, `/usr/lib/modules.`, `ci/lint-mkosi.py`, `docs/bugs/BUG-0001-fabricated-mkosi-config.md`, `docs/bugs/BUG-0020.`, `image/mkosi.conf`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0116 — `ci/lint-unit-binaries.py`

### A. Identity
- **Path**: `ci/lint-unit-binaries.py`
- **Type / Language**: `.py` (Python)
- **Size**: 11,295 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Fail when an installed systemd unit starts a binary that does not exist. The defect this was written for, found in the 2026-10-11 Stage A audit: three units in the image started binaries no crate in `src/` produces. * `litric-capd.service` / `.socket` -> /usr/bin/litric-capd * `litric-defender.service`         -> /usr/bin/litric-defender * `litric-files@.service`           -> /usr/bin/litric-filesd
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 260 lines
- **Key Declarations / Directives / Functions**:
  - L65: `def workspace_members(root: Path) -> list[Path]:`
  - L74: `def producible_binaries(root: Path) -> dict[str, str]:`
  - L106: `def units(tree: Path) -> list[Path]:`
  - L112: `def program_of(value: str) -> str | None:`
  - L122: `def judge(program: str, produced: dict[str, str], tree: Path) -> tuple[str, str]:`
  - L153: `def scan(root: Path) -> tuple[list[str], list[str], list[str]]:`
  - L179: `def check() -> int:`
  - L195: `def self_test() -> int:`
  - L253: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin`, `/usr/bin/`, `/usr/bin/cat`, `/usr/bin/env`, `/usr/bin/litric-capd`, `/usr/bin/litric-dead`, `/usr/bin/litric-defender`, `/usr/bin/litric-filesd`, `/usr/bin/litric-real`, `/usr/bin/systemctl`, `/usr/lib/litric/`, `/usr/lib/litric/missing-helper`, `/usr/lib/litric/selftest`, `/usr/sbin/`, `ci/lint-unit-binaries.py`, `image/mkosi.extra/`, `image/mkosi.postinst.chroot`, `image/units-pending/`, `image/units-pending/README.md`, `src/Cargo.toml`, `src/main.rs`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0117 — `ci/verify.sh`

### A. Identity
- **Path**: `ci/verify.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 24,488 bytes
- **Classification**: `tests` | Role: `ci-verification`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — run every check that can actually execute on this host. This script is the answer to "what is actually verified?". Everything it reports PASS for was executed just now. Everything it reports SKIP for was not run, and the reason is printed. Nothing is inferred. It deliberately does NOT report on things it cannot test. A check that cannot
- **Subsystem**: `ci-verification`

### C. Internal Structure
- **Line Count**: 542 lines
- **Key Declarations / Directives / Functions**:
  - L25: `green() { printf '\033[1;32m%s\033[0m' "$1"; }`
  - L26: `red()   { printf '\033[1;31m%s\033[0m' "$1"; }`
  - L27: `yellow(){ printf '\033[1;33m%s\033[0m' "$1"; }`
  - L30: `check() {`
  - L47: `skip() {`
  - L54: `have() { command -v "$1" >/dev/null 2>&1; }`
  - L81: `shell_syntax() {`
  - L125: `regression_must_fail() {`
  - L250: `keys_valid() {`
  - L298: `cargo_in_src() { ( cd src && cargo ${TC:+$TC} "$@" ); }`
  - L304: `no_warnings() {`
  - L343: `frames_dump() {`
  - L386: `mkosi_parses() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/dev/null`, `/proc/meminfo`, `/proc/stat`, `/proc/uptime`, `/usr/bin/env`, `ci/check-golden-frames.py`, `ci/check-memory-budget.py`, `ci/check-python-syntax.py`, `ci/count-scope.py`, `ci/dep-policy-allow.txt.`, `ci/lint-deps.py`, `ci/lint-doc-refs.py`, `ci/lint-file-safety.py`, `ci/lint-kconfig.py`, `ci/lint-line-endings.py`, `ci/lint-mkosi.py`, `ci/lint-unit-binaries.py`, `ci/verify.sh`, `docs/EVIDENCE.md`, `docs/PERF-budget.md`, `docs/bugs.`, `image/mkosi.conf`, `image/mkosi.extra.`, `image/units-pending/`
- **Environment Variables / Flags**: `LITRIC_ACCEL`, `LITRIC_BOOT_TIMEOUT`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0797 — `tests/gate01-verify.sh`

### A. Identity
- **Path**: `tests/gate01-verify.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 8,295 bytes
- **Classification**: `tests` | Role: `system-test`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — GATE 01 Verification Script Run inside the VM: bash /tmp/gate01-verify.sh TWO defects found in audit on 2026-10-09 (docs/bugs/BUG-0006), both of the same family: the script could not report what it claimed to report. 1. `((PASS++))` under `set -e`. A post-increment evaluates to the value
- **Subsystem**: `system-test`

### C. Internal Structure
- **Line Count**: 219 lines
- **Key Declarations / Directives / Functions**:
  - L39: `check() {`
  - L52: `warn_check() {`
  - L73: `verity_state() {`
  - L83: `lsm_state() {`
  - L89: `mglru_state() {`
  - L101: `preempt_state() {`
  - L111: `seccomp_state() {`
  - L117: `root_state() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/mapper/root`, `/dev/null`, `/proc/cmdline`, `/proc/meminfo`, `/proc/self/attr/current`, `/proc/sys/kernel/randomize_va_space`, `/proc/sys/kernel/seccomp/actions_avail`, `/proc/version`, `/sys/kernel/mm/lru_gen/enabled`, `/sys/kernel/security/lsm`, `docs/bugs/BUG-0006`, `docs/evidence/gate01-boot-clean.log`, `image/mkosi.extra/usr/lib/litric/selftest`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0798 — `tests/smoke_boot.py`

### A. Identity
- **Path**: `tests/smoke_boot.py`
- **Type / Language**: `.py` (Python)
- **Size**: 34,523 bytes
- **Classification**: `tests` | Role: `system-test`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LITRIC OS — CI Smoke Boot Test Headless QEMU + serial console assertion. Asserts: UEFI → kernel → userspace → login/shell prompt. Usage: pytest tests/smoke_boot.py -v python tests/smoke_boot.py              # standalone
- **Subsystem**: `system-test`

### C. Internal Structure
- **Line Count**: 836 lines
- **Key Declarations / Directives / Functions**:
  - L113: `def _timeout_from_env() -> int:`
  - L248: `def _resolve(env_var: str, candidates: list, label: str) -> Path:`
  - L264: `def discover_image() -> Path:`
  - L268: `def discover_firmware_code() -> Path:`
  - L272: `def discover_firmware_vars() -> Path:`
  - L281: `def image_drive_format(path: Path) -> str:`
  - L293: `class QemuRunner:`
  - L303: `def __init__(self, smp=2, mem="2G", timeout=BOOT_TIMEOUT):`
  - L328: `def _prepare_vars(self):`
  - L344: `def _build_cmd(self):`
  - L382: `def _read_loop(self, stream):`
  - L393: `def start(self):`
  - L438: `def _read_console_loop(self, path):`
  - L486: `def classify(line: str) -> str:`
  - L499: `def read_output(self, timeout=None):`
  - L574: `def _note_stage(self, line: str) -> None:`
  - L581: `def furthest_stage(self) -> str:`
  - L588: `def stop(self):`
  - L618: `def _save_log(self):`
  - L630: `def check_prereqs():`
  - L663: `class TestBootSmoke:`
  - L666: `def test_image_exists(self, check_prereqs):`
  - L674: `def test_ovmf_firmware_exists(self, check_prereqs):`
  - L694: `def test_boot_to_shell(self, check_prereqs):`
  - L718: `def test_smoke_test_can_actually_fail(self, check_prereqs):`
  - L775: `def smash(offset: int, length: int, byte: int = 0xFF) -> None:`
  - L794: `def _assert_a_broken_image_fails(self, runner, broken):`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`, `/usr/share/edk2`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `docs/bugs/BUG-0005.`, `docs/bugs/BUG-0017`, `docs/bugs/BUG-0017-smoke-test-listened-to-the-wrong-console.md`, `image/mkosi.conf`, `image/mkosi.extra/usr/lib/systemd/system/serial-getty`, `image/mkosi.postinst.chroot`, `tests/smoke_boot.py`, `tools/boot-test.sh`, `tools/build-image.sh`, `tools/build.sh`, `tools/console-probe.py`
- **Environment Variables / Flags**: `ACCEL`, `LITRIC_ACCEL`, `LITRIC_BOOT_TIMEOUT`, `LITRIC_OVMF_CODE`, `LITRIC_OVMF_VARS`, `LITRIC_QEMU`, `LITRIC_SMOKE_IMAGE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0799 — `tests/test_harness_integrity.py`

### A. Identity
- **Path**: `tests/test_harness_integrity.py`
- **Type / Language**: `.py` (Python)
- **Size**: 17,508 bytes
- **Classification**: `tests` | Role: `system-test`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Regression tests for the verification harness itself. The harness that decides whether LitricOS boots is code, it has failed before (BUG-0002), and it failed again in three separate ways found on 2026-10-09 (BUG-0005, BUG-0006). Its defects are invisible by construction -- a boot check that cannot fail, or cannot time out, still prints PASS -- so the only way to keep them fixed is to test the harness against a guest whose behaviour is
- **Subsystem**: `system-test`

### C. Internal Structure
- **Line Count**: 420 lines
- **Key Declarations / Directives / Functions**:
  - L93: `def guest(tmp_path, monkeypatch):`
  - L109: `def make(scenario: str, timeout: float = 5.0) -> sb.QemuRunner:`
  - L114: `def build():`
  - L129: `def _run(runner):`
  - L143: `def test_login_prompt_is_detected(guest):`
  - L148: `def test_root_ps1_is_detected(guest):`
  - L153: `def test_initrd_failure_is_not_a_shell(guest):`
  - L166: `def test_emergency_shell_word_alone_is_not_success(guest):`
  - L176: `def test_silent_guest_times_out_within_the_deadline(guest):`
  - L192: `def test_mute_guest_times_out_within_the_deadline(guest):`
  - L206: `def test_early_qemu_exit_is_reported_as_such(guest):`
  - L212: `def test_timeout_detail_says_where_the_boot_stopped(guest):`
  - L222: `def test_image_candidates_match_the_pipeline_output():`
  - L236: `def test_env_override_selects_the_image(tmp_path, monkeypatch):`
  - L243: `def test_missing_image_reports_the_path_it_expects(monkeypatch):`
  - L255: `def test_drive_format_follows_the_extension(name, fmt):`
  - L263: `def test_vars_template_is_copied_not_mutated(guest, tmp_path):`
  - L283: `def test_cmd_line_marks_only_the_code_image_read_only(guest):`
  - L314: `def _run_gate(tmp_path, system_state: str):`
  - L341: `def test_gate_verifier_runs_to_completion(tmp_path):`
  - L359: `def test_gate_verifier_reports_a_failure_and_exits_nonzero(tmp_path):`
  - L367: `def test_gate_probes_can_pass_on_a_healthy_machine(tmp_path):`
  - L386: `def test_doc_citation_linter_self_test():`
  - L394: `def test_every_cited_artifact_exists():`
  - L408: `def test_smoke_test_parses_and_has_no_append_hack():`
  - L416: `def test_no_crlf_in_harness_files():`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/proc/cmdline`, `/proc/meminfo`, `/proc/self/attr/current`, `/proc/sys/kernel/randomize_va_space`, `/proc/sys/kernel/seccomp/actions_avail`, `/proc/version`, `/sys/kernel/mm/lru_gen/enabled`, `/sys/kernel/security/lsm`, `/usr/bin/env`, `image/mkosi.conf`, `tests/test_harness_integrity.py`, `tools/boot-test.sh`, `tools/build-image.sh`, `tools/photo_tour_lib.py`, `tools/run-qemu-gui.sh`
- **Environment Variables / Flags**: `LITRIC_QEMU`, `LITRIC_SMOKE_IMAGE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0800 — `tests/test_root_verity_udev_rule.py`

### A. Identity
- **Path**: `tests/test_root_verity_udev_rule.py`
- **Type / Language**: `.py` (Python)
- **Size**: 1,732 bytes
- **Classification**: `tests` | Role: `system-test`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Regression guard for the initrd dm-verity readiness rule. The meaningful proof is the UEFI QEMU boot test.  This small test prevents a future packaging cleanup from deleting or broadening the narrow udev rule that allows systemd to notice its already verified root device.
- **Subsystem**: `system-test`

### C. Internal Structure
- **Line Count**: 47 lines
- **Key Declarations / Directives / Functions**:
  - L25: `def test_root_verity_ready_rule_is_narrow_and_runs_after_systemd_rule() -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0801 — `tools/boot-test.sh`

### A. Identity
- **Path**: `tools/boot-test.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 5,836 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — boot the built image in QEMU and capture the serial console ./tools/boot-test.sh                  boot, 180 s cap ./tools/boot-test.sh --timeout 300    longer cap ./tools/boot-test.sh --secureboot     boot with Secure Boot firmware ./tools/boot-test.sh --tcg            software emulation, no KVM
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 145 lines
- **Key Declarations / Directives / Functions**:
  - L30: `log()  { printf '\033[1;32m[boot]\033[0m %s\n' "$*"; }`
  - L31: `warn() { printf '\033[1;33m[boot]\033[0m %s\n' "$*" >&2; }`
  - L32: `die()  { printf '\033[1;31m[boot]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L60: `find_fw() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/dev/null`, `/usr/bin/env`, `/usr/share/OVMF/`, `/usr/share/edk2-ovmf/x64/`, `/usr/share/edk2/x64/`, `tools/boot-test.sh`, `tools/build.sh`
- **Environment Variables / Flags**: `ACCEL`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0802 — `tools/bootstrap-windows.ps1`

### A. Identity
- **Path**: `tools/bootstrap-windows.ps1`
- **Type / Language**: `.ps1` (PowerShell)
- **Size**: 14,122 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ----------------------------------------------------------------------------- Preflight. Report everything before changing anything, so the user knows what they are agreeing to. ----------------------------------------------------------------------------- WSL2 runs on the Hyper-V hypervisor, which needs virtualization enabled in firmware. This is by far the most common reason `wsl --install` appears to
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 321 lines
- **Key Declarations / Directives / Functions**:
  - L53: `param(`
  - L63: `function Write-Step { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Green }`
  - L64: `function Write-Warn { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Yellow }`
  - L65: `function Write-Err  { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Red }`
  - L67: `function Test-Admin {`
  - L91: `function Invoke-Native {`
  - L92: `param(`
  - L115: `function Invoke-Preflight {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/build.sh`, `tools/provision-builder.sh`, `tools/provision-builder.sh.`, `tools/run-qemu.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0803 — `tools/build-image.sh`

### A. Identity
- **Path**: `tools/build-image.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 6,939 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LITRIC OS — Image Build Script Builds the Litric disk image from mkosi configuration Prerequisites: - Linux host (or WSL2) - mkosi (systemd ≥ 256)
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 221 lines
- **Key Declarations / Directives / Functions**:
  - L38: `log() { echo -e "${GREEN}[BUILD]${NC} $*"; }`
  - L39: `warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }`
  - L40: `err() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }`
  - L46: `preflight() {`
  - L76: `download_ovmf() {`
  - L115: `build_image() {`
  - L141: `convert_qcow2() {`
  - L163: `record_measurements() {`
  - L190: `main() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `/usr/share/ovmf/x64/OVMF_CODE_4M.fd`, `/usr/share/ovmf/x64/OVMF_VARS_4M.fd`, `tools/build-image.sh`, `tools/run-qemu.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0804 — `tools/build.sh`

### A. Identity
- **Path**: `tools/build.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 18,932 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — build an image ./tools/build.sh --profile nano          minimal, the GATE 01 target ./tools/build.sh --profile core ./tools/build.sh --profile desktop ./tools/build.sh --profile gaming
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 419 lines
- **Key Declarations / Directives / Functions**:
  - L39: `log()  { printf '\033[1;32m[build]\033[0m %s\n' "$*"; }`
  - L40: `warn() { printf '\033[1;33m[build]\033[0m %s\n' "$*" >&2; }`
  - L41: `die()  { printf '\033[1;31m[build]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L42: `step() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin`, `/usr/bin/env`, `/usr/share/doc/litric`, `/usr/share/doc/litric.`, `ci/lint-kconfig.py`, `ci/lint-mkosi.py`, `docs/GATES.md.`, `docs/adr/`, `docs/bugs/BUG-0020`, `image/mkosi.conf`, `image/mkosi.extra`, `iso/assemble-iso.sh`, `keys/dev/`, `keys/dev/secureboot.crt`, `keys/dev/secureboot.key`, `src/target/release/`, `tests/smoke_boot.py`, `tools/bootstrap-windows.ps1`, `tools/build.sh`, `tools/gen-dev-keys.sh`, `tools/probe-initrd.py`, `tools/provision-builder.sh`, `tools/run-qemu.sh`, `tools/trim-initrd.sh`
- **Environment Variables / Flags**: `LITRIC_PROFILE`, `MAKE_ISO`, `PROFILE`, `SOURCE_DATE_EPOCH`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0805 — `tools/console-probe.py`

### A. Identity
- **Path**: `tools/console-probe.py`
- **Type / Language**: `.py` (Python)
- **Size**: 7,547 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — run a command on the guest's interactive console and print it. WHY THIS EXISTS --------------- The image has two serial ports and they are not the same console: * **ttyS0** is the kernel and systemd log. `console=tty0 console=ttyS0,115200` is in `image/mkosi.conf`, and it is where every boot message goes.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 198 lines
- **Key Declarations / Directives / Functions**:
  - L67: `def discover_firmware() -> Path:`
  - L74: `def start_qemu(args: argparse.Namespace) -> subprocess.Popen:`
  - L111: `def connect(path: str, timeout: float) -> socket.socket:`
  - L127: `def read_until(sock: socket.socket, needle: str, deadline: float) -> str:`
  - L145: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `image/mkosi.conf`, `image/mkosi.extra/usr/lib/systemd/system/serial-getty`, `image/mkosi.postinst.chroot`, `tests/smoke_boot.py`, `tools/boot-test.sh`, `tools/console-probe.py`
- **Environment Variables / Flags**: `LITRIC_QEMU`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0806 — `tools/design-page.py`

### A. Identity
- **Path**: `tools/design-page.py`
- **Type / Language**: `.py` (Python)
- **Size**: 47,306 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Render the LitricOS design page from measured token data. The page is *generated* rather than written, for the same reason ``ci/count-scope.py`` rewrites the scope tally instead of trusting it: a number typed into a document stops being true the moment the source moves, and documents do not fail a build. Every colour, ratio and alpha on the page below comes from ``cargo run -p litric-design --example tokens``.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 1115 lines
- **Key Declarations / Directives / Functions**:
  - L1079: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/proc/meminfo`, `/usr/bin/env`, `ci/count-scope.py`, `ci/lint-line-endings.py`, `docs/evidence/desktop-boot-2026-08-29.png`, `image/png`, `src/litric-design/examples/tokens.rs`, `src/litric-design/src/color.rs`, `src/litric-design/src/lib.rs`, `tools/design-page.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0807 — `tools/desktop-app.sh`

### A. Identity
- **Path**: `tools/desktop-app.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 7,222 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS as an application window. WHAT THIS IS ------------ One command that opens the real operating system in a window on this machine: UEFI firmware, the signed image, systemd, litric-sysd, and the shell drawing into the emulated framebuffer. Mouse and keyboard are USB HID devices the
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 184 lines
- **Key Declarations / Directives / Functions**:
  - L55: `usage() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/bus/usb`, `/dev/null`, `/usr/bin/env`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `docs/DESKTOP-APP.md.`, `tools/build.sh`
- **Environment Variables / Flags**: `LITRIC_DEBUG_CONSOLE`, `LITRIC_DEBUG_SOCK`, `LITRIC_DISK`, `LITRIC_DISPLAY`, `LITRIC_FIRMWARE`, `LITRIC_FULLSCREEN`, `LITRIC_MEMORY`, `LITRIC_NET`, `LITRIC_QMP`, `LITRIC_SERIAL`, `LITRIC_SHARE`, `LITRIC_SMP`, `LITRIC_USB`, `LITRIC_VARS`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0808 — `tools/extract-uki.py`

### A. Identity
- **Path**: `tools/extract-uki.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,774 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Extract sections from a UKI (PE/COFF binary) and rebuild with trimmed initrd.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 117 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `keys/dev/pcr.crt`, `keys/dev/pcr.key`, `keys/dev/secureboot.crt`, `keys/dev/secureboot.key`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0809 — `tools/frame-ascii.py`

### A. Identity
- **Path**: `tools/frame-ascii.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,644 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Render a PPM screenshot as coarse luminance text, for terminals and CI logs. WHY THIS EXISTS --------------- A frame is a picture, and this environment has no image viewer: an agent or a CI job that has just captured the framebuffer needs to answer "is the desktop up, or is this still a black console?" without a human. Downsampling to a
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 100 lines
- **Key Declarations / Directives / Functions**:
  - L29: `def read_ppm(path: Path) -> tuple[int, int, bytes]:`
  - L51: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `tools/frame-ascii.py`, `tools/ppm2png.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0810 — `tools/frames-page.py`

### A. Identity
- **Path**: `tools/frames-page.py`
- **Type / Language**: `.py` (Python)
- **Size**: 5,402 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Build a review page from the frames ``frame_dump`` produced. Why this exists: the governance plan asks for golden screens and requires them to come from the machine. Until an image is booted, a reviewer has no way to see what a change did, and "the diff looks right" is not a review. This turns the output of ``cargo run --example frame_dump`` into one page a person can actually look at, with the numbers attached to each picture:
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 135 lines
- **Key Declarations / Directives / Functions**:
  - L40: `def parse_metrics(path: Path) -> dict[str, dict[str, str]]:`
  - L57: `def frames(directory: Path) -> list[tuple[str, Path, Path]]:`
  - L67: `def data_uri(png: Path) -> str:`
  - L71: `def card(scene: str, png: Path, metrics: dict[str, dict[str, str]]) -> str:`
  - L87: `def page(directory: Path, metrics_path: Path) -> str:`
  - L119: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `image/png`, `tools/frames-page.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0811 — `tools/gen-dev-keys.sh`

### A. Identity
- **Path**: `tools/gen-dev-keys.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 6,604 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — generate DEVELOPMENT signing keys Produces the Secure Boot and dm-verity key pairs that image/mkosi.conf refers to. Without them mkosi fails at the validation stage, which is why ci/lint-mkosi.py treats a missing key as an error rather than a warning. keys/dev/secureboot.key / .crt   signs the Unified Kernel Image
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 141 lines
- **Key Declarations / Directives / Functions**:
  - L38: `log()  { printf '\033[1;32m[keys]\033[0m %s\n' "$*"; }`
  - L39: `warn() { printf '\033[1;33m[keys]\033[0m %s\n' "$*" >&2; }`
  - L40: `die()  { printf '\033[1;31m[keys]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L82: `gen_pair() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `ci/lint-mkosi.py`, `image/mkosi.conf`, `keys/.gitignore`, `keys/dev`, `keys/dev/secureboot.key`, `keys/dev/verity.key`, `tools/gen-dev-keys.sh`, `tools/gen-dev-keys.sh.`
- **Environment Variables / Flags**: `LITRIC_DEV_KEY_DAYS`, `SOURCE_DATE_EPOCH`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0812 — `tools/guest-console.py`

### A. Identity
- **Path**: `tools/guest-console.py`
- **Type / Language**: `.py` (Python)
- **Size**: 7,363 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Log into a running LitricOS guest over its ttyS1 socket and run commands. The guest's second serial port has a getty on it. When the launcher is started with LITRIC_DEBUG_CONSOLE=1 that port is a unix socket instead of a file, so the host can type at the login prompt and read what the guest says back. This is how "does the guest really have a lease, a driver, or a unit running" is answered with the guest's own output rather than an inference from the outside.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 186 lines
- **Key Declarations / Directives / Functions**:
  - L35: `class Console:`
  - L36: `def __init__(self, path: Path, timeout: float = 30.0) -> None:`
  - L42: `def read_until_count(self, pattern: str, count: int, timeout: float = 30.0) -> str:`
  - L69: `def read_until(self, pattern: str | bytes, timeout: float = 30.0) -> str:`
  - L86: `def write(self, data: str) -> None:`
  - L89: `def drain(self, timeout: float = 0.6) -> None:`
  - L102: `def command(self, cmd: str, marker: str, timeout: float = 30.0) -> str:`
  - L112: `def clean(raw: bytes) -> str:`
  - L117: `def main(argv: list[str]) -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `tools/guest-console.py`
- **Environment Variables / Flags**: `LITRIC_DEBUG_CONSOLE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0813 — `tools/live-desktop.py`

### A. Identity
- **Path**: `tools/live-desktop.py`
- **Type / Language**: `.py` (Python)
- **Size**: 27,099 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Use the LitricOS desktop from a browser: boot it headless, stream its screen. WHY THIS EXISTS --------------- `tools/show-desktop.sh` and `tools/photo-tour.sh` photograph a boot: they prove what the system looks like, and they cannot be interrupted. That is the right shape for evidence and the wrong shape for using the thing.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 679 lines
- **Key Declarations / Directives / Functions**:
  - L64: `class Qmp:`
  - L65: `def __init__(self, sock_path: Path) -> None:`
  - L73: `def connect(self, timeout: float = 30.0) -> None:`
  - L99: `def _read_one(self) -> dict:`
  - L113: `def _cmd(self, name: str, **arguments) -> dict:`
  - L127: `def cmd(self, name: str, **arguments) -> dict:`
  - L134: `def screendump(self, target: Path) -> bytes:`
  - L147: `def send_events(self, events: list[dict]) -> None:`
  - L150: `def status(self) -> dict:`
  - L160: `def ppm_to_png(data: bytes) -> bytes:`
  - L186: `def chunk(tag: bytes, payload: bytes) -> bytes:`
  - L229: `def key_events(text: str) -> list[dict]:`
  - L233: `def press(code: str, shift: bool = False) -> None:`
  - L253: `def _key(code: str, down: bool) -> dict:`
  - L432: `class Handler(BaseHTTPRequestHandler):`
  - L438: `def log_message(self, fmt: str, *args) -> None:  # quiet: this is a UI`
  - L441: `def _send(self, code: int, body: bytes, ctype: str) -> None:`
  - L449: `def do_GET(self) -> None:  # noqa: N802 -- http.server's spelling`
  - L484: `def do_POST(self) -> None:  # noqa: N802`
  - L501: `def send_input(qmp: Qmp, batch: list[dict], size: tuple[int, int]) -> None:`
  - L521: `def _abs(pos: int, extent: int) -> int:`
  - L527: `def events_for_key(key: str) -> list[dict]:`
  - L540: `def pick_display_device(explicit: str | None) -> str:`
  - L566: `def start_qemu(args: argparse.Namespace) -> subprocess.Popen[bytes]:`
  - L612: `def main() -> int:`
  - L656: `def shutdown(*_args) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `image/png`, `tools/build.sh`, `tools/build.sh.`, `tools/live-desktop.py`, `tools/photo-tour.sh`, `tools/render_bench`, `tools/show-desktop.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0814 — `tools/measure-idle.py`

### A. Identity
- **Path**: `tools/measure-idle.py`
- **Type / Language**: `.py` (Python)
- **Size**: 15,286 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Measure an idle LitricOS machine from inside the machine. tools/measure-idle.py [--json] [--memory 1536] [--settle 20] WHY THIS EXISTS --------------- `docs/PERF-budget.md` carries targets — idle RAM, idle CPU, boot time — and its first line says every number on it is a TARGET, not a MEASURED value. The
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 356 lines
- **Key Declarations / Directives / Functions**:
  - L75: `def first_existing(paths: list[Path]) -> Path:`
  - L82: `class Console:`
  - L85: `def __init__(self, sock: socket.socket) -> None:`
  - L90: `def attach(cls, path: Path, timeout: float) -> "Console":`
  - L105: `def expect(self, pattern: str, timeout: float) -> str:`
  - L124: `def send(self, text: str) -> None:`
  - L127: `def run(self, command: str, timeout: float = 20.0) -> str:`
  - L137: `def boot(image: Path, accel: str, memory: int) -> tuple[subprocess.Popen, Path, float]:`
  - L170: `def main(argv: list[str]) -> int:`
  - L255: `def cpu_sample() -> dict:`
  - L327: `def format_report(report: dict) -> str:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/dev/null`, `/proc/meminfo`, `/proc/stat`, `/proc/uptime`, `/usr/bin/env`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `docs/PERF-budget.md`, `image/mkosi.extra/usr/lib/systemd/system/litric-`, `tools/build.sh`, `tools/measure-idle.py`
- **Environment Variables / Flags**: `LITRIC_QEMU`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0815 — `tools/measure-ui.py`

### A. Identity
- **Path**: `tools/measure-ui.py`
- **Type / Language**: `.py` (Python)
- **Size**: 1,898 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Measure the rendered desktop against the muted-material targets. Reads the PPM the QEMU screendump actually wrote and reports the statistics the design pass is accountable for: mean chroma (saturation proxy) and mean luminance over the whole frame, plus the chroma at the three wallpaper wash anchors. Old build vs new build through the same ruler.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 60 lines
- **Key Declarations / Directives / Functions**:
  - L13: `def read_ppm(path):`
  - L37: `def main():`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0816 — `tools/monitor-cmd.py`

### A. Identity
- **Path**: `tools/monitor-cmd.py`
- **Type / Language**: `.py` (Python)
- **Size**: 1,064 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Send one QEMU monitor command through the show-desktop monitor socket. Usage: monitor-cmd.py <command> [args...] Designed for `wsl.exe -d archlinux -e python3 /mnt/c/.../monitor-cmd.py screendump <path>`.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 45 lines
- **Key Declarations / Directives / Functions**:
  - L15: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0817 — `tools/photo-tour-apps.py`

### A. Identity
- **Path**: `tools/photo-tour-apps.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,217 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — deep app tour over QMP (companion to photo-tour.py). Drives the launcher and every app window: for each app, launch it, photograph the window, open its app menu, open a context menu inside it, then close. Section folders: 20-launcher, 21-<app>, ... python3 tools/photo-tour-apps.py [--out DIR]
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 113 lines
- **Key Declarations / Directives / Functions**:
  - L33: `def main() -> int:`
  - L42: `def S(s: float) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `tools/photo-tour-apps.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0818 — `tools/photo-tour-real.py`

### A. Identity
- **Path**: `tools/photo-tour-real.py`
- **Type / Language**: `.py` (Python)
- **Size**: 5,400 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — precise interaction tour with code-derived coordinates. Icon grid (views.rs draw_desktop_icons): top_bar=28, space Lg=16 → top=44; cell 92x88, x0=16, COLUMN-major: index i → col=i/4, row=i%4. Icons: 0 Files, 1 Browser, 2 Terminal, 3 Defender, 4 Settings, 5 Trash, 6 Documents, 7 Downloads (apps.rs DesktopIcons::standard). Launcher opens with a bare Meta press+release (shortcuts.rs).
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 188 lines
- **Key Declarations / Directives / Functions**:
  - L23: `def icon_center(i: int) -> tuple[int, int]:`
  - L30: `def meta_launcher(q) -> None:`
  - L35: `def main() -> int:`
  - L47: `def S(s: float) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0819 — `tools/photo-tour.py`

### A. Identity
- **Path**: `tools/photo-tour.py`
- **Type / Language**: `.py` (Python)
- **Size**: 13,790 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — full-feature photo tour over QMP. Boots the disposable image headless and drives the REAL guest input path: QMP input-send-event (absolute pointer to the usb-tablet, qcode keyboard to the i8042/evdev stack), then photographs the framebuffer with a screendump issued through the same QMP socket. Screenshots land in section folders so the gallery reads like a narrative of the OS.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 406 lines
- **Key Declarations / Directives / Functions**:
  - L46: `class Qmp:`
  - L49: `def __init__(self, path: str) -> None:`
  - L55: `def _read(self) -> dict:`
  - L65: `def _greeting(self) -> None:`
  - L69: `def cmd(self, name: str, **args) -> dict:`
  - L79: `def close(self) -> None:`
  - L86: `def move(q: Qmp, x: int, y: int) -> None:`
  - L97: `def button(q: Qmp, which: str, down: bool) -> None:`
  - L106: `def click(q: Qmp, x: int, y: int, hold: float = 0.08) -> None:`
  - L115: `def rclick(q: Qmp, x: int, y: int) -> None:`
  - L124: `def dblclick(q: Qmp, x: int, y: int) -> None:`
  - L129: `def key_event(q: Qmp, qcode: str, down: bool) -> None:`
  - L141: `def key(q: Qmp, name: str) -> None:`
  - L152: `def type_text(q: Qmp, text: str) -> None:`
  - L160: `class Tour:`
  - L161: `def __init__(self, q: Qmp, base: str) -> None:`
  - L168: `def shoot(self, name: str) -> None:`
  - L189: `def goto(self, section: str) -> None:`
  - L194: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `tools/photo-tour.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0820 — `tools/photo-tour.sh`

### A. Identity
- **Path**: `tools/photo-tour.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 10,035 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — FULL FEATURE TOUR: 100+ screenshots of every surface Boots the disposable image headless (-display none), drives the REAL input path (QEMU monitor mouse_move + sendkey → guest evdev → our WM), and photographs every step. Screenshots are grouped into section folders under build/tour/<stamp>/ so the gallery can present a narrative.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 292 lines
- **Key Declarations / Directives / Functions**:
  - L31: `log()  { printf '\033[1;36m[tour]\033[0m %s\n' "$*"; }`
  - L32: `warn() { printf '\033[1;33m[tour]\033[0m %s\n' "$*" >&2; }`
  - L91: `mon() { python3 "${SCRIPT_DIR}/qemu-monitor.py" "$MON" "$1" >/dev/null 2>&1; }`
  - L94: `shoot() {`
  - L112: `click() {`
  - L121: `rclick() {`
  - L130: `key() {`
  - L135: `w() { sleep "$1"; }`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/dev/null`, `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_CODE.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.fd`, `tools/photo-tour.sh`
- **Environment Variables / Flags**: `ACCEL`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0821 — `tools/photo_tour_lib.py`

### A. Identity
- **Path**: `tools/photo_tour_lib.py`
- **Type / Language**: `.py` (Python)
- **Size**: 10,573 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Shared QMP boot and input helpers for the LitricOS photo tours.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 305 lines
- **Key Declarations / Directives / Functions**:
  - L43: `class Qmp:`
  - L46: `def __init__(self, path: str) -> None:`
  - L52: `def _read(self) -> dict:`
  - L62: `def _greeting(self) -> None:`
  - L66: `def cmd(self, name: str, **args) -> dict:`
  - L76: `def close(self) -> None:`
  - L83: `def boot_qmp(out_dir: str, image: str | None = None):`
  - L156: `def exec_guest(cmd: str, sock_path: str, timeout: float = 45.0,`
  - L202: `def move(q: Qmp, x: int, y: int) -> None:`
  - L220: `def button(q: Qmp, which: str, down: bool) -> None:`
  - L226: `def click(q: Qmp, x: int, y: int, hold: float = 0.08) -> None:`
  - L235: `def rclick(q: Qmp, x: int, y: int) -> None:`
  - L244: `def dblclick(q: Qmp, x: int, y: int) -> None:`
  - L249: `def key_event(q: Qmp, qcode: str, down: bool) -> None:`
  - L255: `def key(q: Qmp, name: str) -> None:`
  - L269: `def type_text(q: Qmp, text: str) -> None:`
  - L279: `class Tour:`
  - L282: `def __init__(self, q: Qmp, base: str) -> None:`
  - L287: `def shoot(self, name: str) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0822 — `tools/playtest.py`

### A. Identity
- **Path**: `tools/playtest.py`
- **Type / Language**: `.py` (Python)
- **Size**: 11,441 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Playtest LitricOS the way its first real user would — on an image COPY. Never touches a VM the user has open: boots build/playtest.raw (a copy), drives the real input path (QMP abs/btn/key -> evdev), and confirms every step from inside the guest (serial diagnostics + ttyS1 console), never from assumption. Clicks are computed from the shipped geometry (dock formula, window frames from `window=` diagnostics, chrome buttons from
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 249 lines
- **Key Declarations / Directives / Functions**:
  - L41: `def dock_xy(app: str) -> tuple[int, int]:`
  - L45: `def drag(q: Qmp, x0: int, y0: int, x1: int, y1: int, steps: int = 8) -> None:`
  - L57: `def parse_windows(lines: list[str]) -> dict[str, tuple[int, int, int, int, str]]:`
  - L82: `def main() -> int:`
  - L94: `def check(name: str, ok: bool, detail: str = "") -> None:`
  - L99: `def guest(sock: str, log: str, n: int, cmd: str, timeout: float = 50.0) -> list[str]:`
  - L112: `def S(sec: float) -> None:`
  - L115: `def diag() -> dict:`
  - L127: `def wait_for(name: str, pred, timeout: float = 16.0) -> dict:`
  - L243: `def _moved(d: dict, ax: int, ay: int) -> bool:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/proc/uptime`, `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0823 — `tools/ppm2png.py`

### A. Identity
- **Path**: `tools/ppm2png.py`
- **Type / Language**: `.py` (Python)
- **Size**: 2,200 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Convert a QEMU screendump (PPM, whatever its extension) to a real PNG. QEMU's `screendump` writes P6 PPM; naming the output .png does not change the format, which is why the file would not open on the host. This converts it losslessly with nothing but zlib and struct, and reports the scanout resolution the guest is actually driving.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 69 lines
- **Key Declarations / Directives / Functions**:
  - L14: `def read_ppm(path: str) -> tuple[int, int, bytes]:`
  - L42: `def write_png(path: str, w: int, h: int, raw: bytes) -> None:`
  - L43: `def chunk(tag: bytes, payload: bytes) -> bytes:`
  - L60: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0824 — `tools/probe-browser.py`

### A. Identity
- **Path**: `tools/probe-browser.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,261 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Probe: does a dock click open the Browser, and does the engine answer? Staged so a failure names its stage instead of failing at the end: 1. boot, settle 2. rest pointer on the dock edge, wait for the slide, click Browser 3. screendump -> window present? 4. click omnibox, type, Enter
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 95 lines
- **Key Declarations / Directives / Functions**:
  - L21: `def md5(path: str) -> str:`
  - L26: `def main() -> None:`
  - L34: `def S(s: float) -> None:`
  - L37: `def shot(name: str) -> str:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0825 — `tools/probe-initrd.py`

### A. Identity
- **Path**: `tools/probe-initrd.py`
- **Type / Language**: `.py` (Python)
- **Size**: 22,822 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Check that the UKI's initrd can actually mount the root filesystem. WHY THIS EXISTS (BUG-0020) -------------------------- The image's system partition is erofs behind dm-verity, and the kernel it ships builds erofs as a *module*. A module that is not inside the initrd cannot be loaded before `/sysroot` exists, so the boot reached an emergency shell with
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 572 lines
- **Key Declarations / Directives / Functions**:
  - L78: `def run(argv: list[str], data: bytes) -> bytes:`
  - L93: `def zstd_frame_size(data: bytes, pos: int) -> int | None:`
  - L159: `def members(blob: bytes) -> list[tuple[str, bytes]]:`
  - L205: `def parse_entry(blob: bytes, pos: int) -> tuple[str, int, int] | None:`
  - L242: `def round4(value: int) -> int:`
  - L247: `def cpio_names(blob: bytes) -> list[str]:`
  - L300: `def extract_section(uki: Path, section: str) -> bytes | None:`
  - L319: `def read_initramfs(uki: Path) -> tuple[bytes | None, list[tuple[str, list[str]]]]:`
  - L332: `def main() -> int:`
  - L431: `def cpio_member(entries: list[tuple[str, bytes]]) -> bytes:`
  - L460: `def cpio_member_trailer(offset: int) -> bytes:`
  - L468: `def self_test() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/stdout`, `/usr/bin/env`, `docs/bugs/BUG-0020`, `image/mkosi.conf`, `kernel/drivers/md/dm-mod.ko.zst`, `kernel/drivers/md/dm-verity.ko.zst`, `kernel/fs/erofs/erofs.ko.zst`, `kernel/fs/x/other.ko.zst`, `tools/photo-tour`, `tools/probe-initrd.py`, `tools/screen-test.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0826 — `tools/provision-builder.sh`

### A. Identity
- **Path**: `tools/provision-builder.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 11,589 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — provision a Linux build host Runs as root inside WSL (or on any Arch machine) and installs everything tools/build.sh needs. Idempotent: safe to run repeatedly. mkosi builds an Arch image by driving `pacman` ON THE HOST. There is no way around that: bootstrapping a distribution requires that distribution's
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 260 lines
- **Key Declarations / Directives / Functions**:
  - L33: `log()  { printf '\033[1;32m[provision]\033[0m %s\n' "$*"; }`
  - L34: `warn() { printf '\033[1;33m[provision]\033[0m %s\n' "$*" >&2; }`
  - L35: `die()  { printf '\033[1;31m[provision]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L130: `keycount() { pacman-key --list-keys 2>/dev/null | grep -c '^pub' || true; }`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE_4M.fd`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `docs/DEV-02-build.md`, `image/mkosi.conf`, `tools/build.sh`, `tools/provision-builder.sh`, `tools/run-qemu.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0827 — `tools/qemu-monitor.py`

### A. Identity
- **Path**: `tools/qemu-monitor.py`
- **Type / Language**: `.py` (Python)
- **Size**: 2,007 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Send one command to a QEMU human monitor over a Unix socket. Used by tools/screen-test.sh to ask QEMU for a screenshot of the guest's display. socat and netcat would both do this and both are optional packages; a screenshot tool that silently does nothing because netcat is not installed is precisely the failure the screenshot exists to rule out, so this uses the Python that is already required to convert the image.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 69 lines
- **Key Declarations / Directives / Functions**:
  - L22: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `tools/qemu-monitor.py`, `tools/screen-test.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0828 — `tools/rebuild-uki.sh`

### A. Identity
- **Path**: `tools/rebuild-uki.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 1,517 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Extract kernel Get cmdline Get os-release content Rebuild UKI with trimmed initrd Replace in ESP
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 51 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `keys/dev/secureboot.crt`, `keys/dev/secureboot.key`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0829 — `tools/run-qemu-gui.sh`

### A. Identity
- **Path**: `tools/run-qemu-gui.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 2,549 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= run-qemu-gui.sh — open LitricOS in a visible window (WSLg) for real use. Difference from the headless verification runs: the window is real, the pointer/keyboard are yours, and audio goes through WSLg's PulseAudio server. The USB tablet's absolute range is what the guest measures with EVIOCGABS (0xFFFF — the same constant photo_tour_lib.py scales to and litric-input
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 66 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `tools/run-qemu-gui.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0830 — `tools/run-qemu.ps1`

### A. Identity
- **Path**: `tools/run-qemu.ps1`
- **Type / Language**: `.ps1` (PowerShell)
- **Size**: 12,463 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LITRIC OS - QEMU Launcher (Windows Host) Launches Litric in QEMU with WHPX acceleration Prerequisites: - QEMU for Windows (https://qemu.weilnetz.de/w64/), or: winget install SoftwareFreedomConservancy.QEMU - litric.qcow2 in build/ (see tools/build-image.sh - needs a Linux host with mkosi)
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 292 lines
- **Key Declarations / Directives / Functions**:
  - L32: `param(`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/build-image.sh`, `tools/run-qemu-gui.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0831 — `tools/run-qemu.sh`

### A. Identity
- **Path**: `tools/run-qemu.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 4,924 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LITRIC OS — QEMU Launcher (Linux/Mac Host) Launches Litric in QEMU with KVM acceleration Prerequisites: - QEMU (qemu-system-x86_64) - OVMF_CODE_4M.fd and OVMF_VARS_4M.fd in build/
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 164 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `tools/build-image.sh`, `tools/run-qemu.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0832 — `tools/scan-nonascii.py`

### A. Identity
- **Path**: `tools/scan-nonascii.py`
- **Type / Language**: `.py` (Python)
- **Size**: 2,212 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: List the non-ASCII characters the shell puts inside string literals. The typeface is drawn rather than loaded, so a character it does not have draws as a box. `font::draw` does that on purpose -- silence would look like a spacing bug -- but it means the face has to cover everything the interface actually prints. This walks the shell's sources and reports which characters those are, most used first, so the typeface's coverage can be checked against reality
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 70 lines
- **Key Declarations / Directives / Functions**:
  - L26: `def literals(source: str):`
  - L50: `def main() -> int:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`, `src/litric-shell/src`, `tools/scan-nonascii.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0833 — `tools/screen-test.sh`

### A. Identity
- **Path**: `tools/screen-test.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 8,514 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — boot with a real display and photograph the screen ./tools/screen-test.sh [--profile nano] [--timeout 90] [--at 25] WHY THIS EXISTS tools/boot-test.sh runs QEMU with -nographic, which means the guest has no display device at all. That is the right harness for "did it boot", and it is
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 218 lines
- **Key Declarations / Directives / Functions**:
  - L48: `log()  { printf '\033[1;32m[screen]\033[0m %s\n' "$*"; }`
  - L49: `warn() { printf '\033[1;33m[screen]\033[0m %s\n' "$*" >&2; }`
  - L50: `die()  { printf '\033[1;31m[screen]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L135: `cleanup() {`
  - L149: `monitor() {`
  - L153: `shoot() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/fb0`, `/dev/fb0.`, `/dev/kvm`, `/dev/null`, `/usr/bin/env`, `/usr/share/OVMF/OVMF_CODE.fd`, `/usr/share/OVMF/OVMF_VARS.fd`, `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd`, `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`, `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `/usr/share/edk2/x64/OVMF_CODE.fd`, `/usr/share/edk2/x64/OVMF_VARS.4m.fd`, `/usr/share/edk2/x64/OVMF_VARS.fd`, `tools/boot-test.sh`, `tools/build.sh`, `tools/ppm2png.py`, `tools/screen-test.sh`
- **Environment Variables / Flags**: `ACCEL`, `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0834 — `tools/show-desktop.sh`

### A. Identity
- **Path**: `tools/show-desktop.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 6,171 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — boot the built image with a real, interactive window ./tools/show-desktop.sh                  # window on your screen ./tools/show-desktop.sh --mem 2048       # more RAM for the guest ./tools/show-desktop.sh --backend sdl    # different UI backend (default: gtk) ./tools/show-desktop.sh --tcg            # force software emulation
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 143 lines
- **Key Declarations / Directives / Functions**:
  - L36: `log()  { printf '\033[1;32m[show]\033[0m %s\n' "$*"; }`
  - L37: `die()  { printf '\033[1;31m[show]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L64: `find_fw() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/dev/null`, `/usr/bin/env`, `/usr/share/OVMF/`, `/usr/share/edk2-ovmf/x64/`, `/usr/share/edk2/x64/`, `tools/boot-test.sh`, `tools/build.sh`, `tools/screen-test.sh`, `tools/show-desktop.sh`
- **Environment Variables / Flags**: `ACCEL`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0835 — `tools/sync-builder.sh`

### A. Identity
- **Path**: `tools/sync-builder.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 5,163 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — copy the working tree onto the native builder, and prove it landed ./tools/sync-builder.sh [--dest ~/litricos] [--check-only] WHY THIS EXISTS The project lives on a Windows mount, which cannot store Unix ownership or permission bits, so tools/build.sh refuses to build there and the real build
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 122 lines
- **Key Declarations / Directives / Functions**:
  - L38: `log()  { printf '\033[1;32m[sync]\033[0m %s\n' "$*"; }`
  - L39: `warn() { printf '\033[1;33m[sync]\033[0m %s\n' "$*" >&2; }`
  - L40: `die()  { printf '\033[1;31m[sync]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L62: `find_args() {`
  - L71: `manifest() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `kernel/artifacts`, `src/target`, `tools/build.sh`, `tools/sync-builder.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0836 — `tools/trim-initrd.sh`

### A. Identity
- **Path**: `tools/trim-initrd.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 2,218 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= Trim initrd for nano profile — remove binaries not needed for boot ============================================================================= Remove unnecessary binaries Remove directories Remove empty dirs
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 60 lines
- **Key Declarations / Directives / Functions**:
  - L9: `say()  { printf '\033[1;36m[trim-initrd]\033[0m %s\n' "$*"; }`
  - L10: `warn() { printf '\033[1;33m[trim-initrd]\033[0m %s\n' "$*" >&2; }`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/`, `/usr/bin/env`, `/usr/lib/systemd/boot/efi`, `/usr/lib/udev/hwdb.bin`, `/usr/lib/udev/hwdb.d`, `/usr/share/bash-completion`, `/usr/share/doc`, `/usr/share/file`, `/usr/share/info`, `/usr/share/locale`, `/usr/share/man`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0837 — `tools/verify-google-pixels.py`

### A. Identity
- **Path**: `tools/verify-google-pixels.py`
- **Type / Language**: `.py` (Python)
- **Size**: 3,697 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Prove the browser really renders Google's page — by pixels, not hope. Boots the VM, opens the browser from the dock, types a query, commits it, screendumps, and counts pixels in the frame that match Google's four brand colors plus the share of near-white (a real page on our dark shell). A page that only *changed* but did not render would fail the color test.
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 105 lines
- **Key Declarations / Directives / Functions**:
  - L20: `def read_ppm(path: str):`
  - L29: `def main() -> None:`
  - L37: `def S(s: float) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0838 — `tools/verify-tour.py`

### A. Identity
- **Path**: `tools/verify-tour.py`
- **Type / Language**: `.py` (Python)
- **Size**: 8,611 bytes
- **Classification**: `project source` | Role: `tooling-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — deterministic functional self-test on the disposable VM. Every surface this build touched is exercised with the REAL input path (QMP input-send-event) and every transition is asserted in-flight by screenshot hash: a feature that leaves the screen unchanged has failed, and the tour says so instead of hoping an eyeball catches it later. Determinism rules:
- **Subsystem**: `tooling-script`

### C. Internal Structure
- **Line Count**: 233 lines
- **Key Declarations / Directives / Functions**:
  - L26: `def md5_png(path: str) -> str:`
  - L31: `def dock_xy(i: int) -> tuple[int, int]:`
  - L36: `def cascade(n: int) -> tuple[int, int]:`
  - L42: `def main() -> None:`
  - L50: `def S(s: float) -> None:`
  - L55: `def note(name: str, verdict: str) -> None:`
  - L59: `def last_shot() -> str:`
  - L64: `def shoot_and_note(name: str, shot: str) -> None:`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---
