# ADR-0022 — Litric Defender: architecture, roadmap and the honesty contract

- **Status:** Accepted. Phase 0 implemented (`src/litric-defender`, zero
  third-party dependencies, std-only); Phases 1–5 specified with entry
  conditions.
- **Date:** 2026-09-14
- **Extends:** ADR-0008 (security model), `master-plan/12-threat-model-and-
  sandboxing.md`, SYSTEM-SCOPE E3 (Defender rows 529–546)
- **Baseline input:** workflow §3 locked baseline — fanotify permission events +
  YARA + BPF-LSM; **ClamAV is on-demand only** (a resident clamd costs
  ~500 MB–1 GB RSS and would break the ≤600 MB idle budget by itself).

---

## Requirement

LitricOS ships a built-in defender that is always on, never eats the RAM
budget, never silently deletes a file, never blocks the desktop because the
scanner hung, and is **honest about detection limits** — sandbox-first,
advisory-grade detection, with commercial-AV coverage explicitly not promised
(RISK-006 stands).

## The decision

### 1. Layered engine, decision separated from enforcement

The same decision/execution split used everywhere else in LitricOS applies to
security twice over: the **engine decides** (pure, testable, fuzzable), and
the **enforcer acts** (fanotify responder, quarantine executor), because a
defender that cannot say "deny" is worse than none.

```text
Layer 0  hash verdicts        SHA-256 denylist/allowlist — exact, fast, FPR≈0
Layer 1  byte patterns        Aho-Corasick-style literal families (.ndb-class)
Layer 2  rule engine          YARA-compatible rules (yara-x when vendored)
Layer 3  static heuristics    entropy, PE/ELF structure, import anomalies
Layer 4  behavioural          BPF-LSM: mass-rename, entropy-spike-on-write,
                              unexpected outbound — scoring, never auto-kill
Layer 5  on-demand deep scan  ClamAV spawned, scan, stopped — never resident
```

Layer ordering is also **cost ordering**: 0–1 are microseconds and always run;
2–3 run on suspicious or unverified content; 4 is continuous but asynchronous;
5 exists only when the user asks for it.

### 2. The policy core is a pure function

`policy::verdict(signals, file_class) -> Verdict` — no I/O, no clock, no
environment. This is what makes the dangerous parts testable: fail-open vs
fail-closed, timeout behaviour, and exclusion logic are all decided here, with
tables of tests, before any kernel-adjacent code exists.

Three rules the policy encodes, each a lesson from production AV outages:

| Rule | Why |
|---|---|
| **Fail-open with a logged warning** on scanner timeout/unavailability | A hung scanner must never freeze the desktop (E3 row 531). A defender that can DoS its own OS is a self-inflicted outage — the CrowdStrike 2024 lesson at the availability extreme. |
| **Fail-closed** only for a small, explicit denylist of exact hash matches | Known-bad hashes are the one verdict class with effectively zero false-positive risk. Everything advisory stays advisory until the scoring layer exists. |
| **Never delete silently; quarantine is reversible** (E3 rows 538–539) | False positives are the most expensive AV failure mode. Every action records original path, verdict, evidence and a restore path. |

### 3. The verdict cache is the whole performance story

Key: `(content-hash, file-class, definition-generation)`. A file whose hash and
the definition generation are unchanged is **never re-scanned**; verity-backed
system files are allowlisted by construction (E3 row 544). On-access latency
on cached files must be single-digit milliseconds (GATE 07), which the cache
makes reachable and the Phase 0 implementation already structures.

Definition updates bump the generation, which invalidates cached verdicts in
O(1) — the same `generation` guard pattern the capability layer uses.

### 4. Resource budgets are hard, in the unit files

`defender.slice` carries `MemoryMax=100M` (E3 row 545 states the stricter 60 MB
engine target; the slice cap is the hard backstop). Layer 5 (ClamAV) runs in a
separate transient scope so its 1–2 GB transient footprint can never count
against the resident budget. Hash databases are mmap'd sorted tables with a
bloom pre-filter, not resident vectors — ClamAV's 2.4 GiB reload spike is the
anti-pattern named in the baseline input.

### 5. Update pipeline safety (the CrowdStrike contract)

Definition updates are the highest-risk input the defender accepts, because
they execute in the path of every file open. Non-negotiable:

1. **Signed definitions** —   Ed25519/RSA over the definition manifest; unsigned
   or wrongly-signed updates are refused, never "load with a warning".
2. **Schema and field-count validation** before activation — the Channel File
   291 root cause was an unvalidated field-count mismatch reaching a parser.
3. **Canary rollout** — a staged percentage before wide release.
4. **Rollback** — the previous generation of definitions is retained and
   restorable, including from a boot entry with on-access disabled (E3 row 8
   boot entry; GATE 07 "Defender safe mode").
5. **Fuzzing the parser** — definitions are untrusted input (master-plan/17).

### 6. Roadmap, with entry conditions

| Phase | Deliverable | Entry condition | State |
|---|---|---|---|
| 0 | Policy core + SHA-256 + denylist + patterns + quarantine *plan* (this crate) | none — pure std | **implemented** |
| 1 | File inspection pipeline: magic, entropy, archive walking with zip-bomb limits (ratio + depth + cumulative bytes) | Phase 0 green in CI | specified |
| 2 | fanotify `FAN_OPEN_PERM` responder on Linux: self-exclusion (no recursion), verdict cache persistence, hard timeout | a booted `core` profile; Linux host | specified |
| 3 | BPF-LSM behavioural signals (exec, mass-rename, outbound) + scoring with evidence; ransomware canary files | kernel rows 25/534 live; fuzz harness | specified |
| 4 | Quarantine execution: encrypted store, restore, FP reporting path | Phase 2 stable | specified |
| 5 | Signed definition channel + canary rollout + rollback + UI honesty surface | update system (Phase 08) exists | specified |

Phase 2 is where the crate touches the kernel's fanotify permission path —
the highest-risk integration in the project after the boot chain, and the
reason Phases 0–1 must be boring first.

### 7. Dependency and license position

The workspace rule — zero third-party crates in core system components —
applies: **Phase 0 is std-only** (own SHA-256, own pattern matcher). When
Layer 2 needs a real rule engine, `yara-x` (Rust, permissive license) is the
designated dependency, vendored and reviewed; libclamav is **not linked into
any Litric binary** — ClamAV runs as an external on-demand process precisely
so its GPLv2 never contaminates the workspace. Elastic-License-v2 rule sets
are excluded from shipped rule packs.

### 8. Integration with recovery (ADR-0021)

The defender is a **signal source** for the recovery decision engine:
mass-rename/entropy signals mark an environment `Compromised` (driving
`litric-env`'s already-tested isolate-and-fallback), canary encryption raises
an incident record, and quarantine integrates with snapshot policy so a
restore never resurrects a quarantined binary into an executable path.

## What is honestly not true yet

Everything except Phase 0. There is no on-access scanning (row 529 `S`), no
fanotify responder, no quarantine execution, no behavioural layer, no signed
update channel. EICAR is not yet detected on any running system because no
running system asks this crate yet. Phase 0's claim is exactly: **the policy
core exists, is pure, and its tables are tested** — including that an
unavailable scanner fails open, a denylisted hash fails closed, and unknown
content with no signals is allowed rather than guessed at.

## Revisit if

- On-access latency (Phase 2) exceeds ~50–100 ms per open on ordinary files —
  tighten cache scope to risky file classes rather than weakening the cache.
- Behavioural scoring (Phase 3) shows a false-positive rate that blocks real
  user work — demote signals to telemetry-only; the blocking path stays
  hash+pattern.
- Resident RSS trends above 500 MB with real definition sets — move more
  tables from resident structures to mmap'd lookup before considering
  anything else.
- A kernel upgrade rejects the BPF programs — the defender degrades to
  Layer 0–2 and warns; it must never block boot (GATE 07 failure mode).
