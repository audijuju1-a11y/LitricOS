# ILTRIC — PHASE 13 PROMPT: PERFORMANCE, MEMORY BUDGET & SMOOTHNESS

**Prerequisite:** GATE 02+. Cross-cutting — re-run after every major phase.
**Deliverable:** the low-RAM, high-smoothness claim, backed by a repeatable harness and published numbers.

> **The claim ILTRIC is allowed to make:** Windows 11 idles around 2.2–3.3 GB on a clean install (and reportedly ~6 GB on an 8 GB machine — Microsoft has publicly acknowledged this and promised optimizations). A minimal Linux base idles in the tens of MB. So the honest, defensible ILTRIC pitch is **a 3–10× memory advantage over Windows**, and a meaningful one over GNOME/KDE (~1–2 GB idle). Everything else must be earned with measurements.

---

## OBJECTIVE
Enforce the RAM budget as a hard gate, and make smoothness a measured property rather than a feeling.

---

## TASKS

1. **RAM budget ledger** — `/docs/PERF-budget.md`
   ```
   kernel + init            <=  80 MB
   core services            <=  60 MB
   compositor               <= 180 MB
   shell                    <= 180 MB
   defender                 <= 100 MB
   networking + power       <=  25 MB
   AI service (idle, unloaded) <= 25 MB
   ------------------------------------
   TOTAL IDLE TARGET        <= 600 MB
   ```
   Every phase must fit its line. **Exceeding a line blocks that phase's gate.** Track it per commit.

2. **Memory instrumentation** — `/tools/bench/mem/`
   `free -m`, `smem`/PSS per process, `systemd-cgtop` per slice, `zramctl` ratios. Automate: one command produces a dated report.

3. **zram tuning experiment** — run the matrix, publish the table:
   | algo | ratio | compress speed | decompress speed | CPU under load | stall time |
   Reference points: lzo-rle is the kernel default (measured ~27% less (de)compress time than lzo in an 80-tab test); zstd gives the best ratio (~5:1 real world) and finishes faster overall by moving less data; lz4 is the throughput king (decompress ~3600 MB/s vs zstd ~1110 MB/s) and is the right pick for latency-critical profiles.
   **Decision rule:** zstd for the desktop profile, lz4 for the Gaming Mode profile — but only after your own numbers agree.

4. **MGLRU verification** — record kswapd CPU %, low-memory kill count, and app launch time with MGLRU on vs off. Google's fleet data behind MGLRU reported ~40% less kswapd CPU, ~85% fewer low-memory kills at p75, and ~18% faster app launch at p50 — use that as a hypothesis to test, not as your result.

5. **Kernel trim pass** — rebuild with unused subsystems disabled; record kernel image size, module count, and boot-time RAM delta in MB before/after.

6. **Latency & smoothness harness** — `/tools/bench/latency/`
   - `cyclictest` — scheduling latency (before/after PREEMPT and sched_ext changes)
   - compositor frame-time histogram + missed-flip counter (Phase 05)
   - `hyperfine` — app launch times
   - `fio` — storage (encrypted vs plain, Btrfs vs erofs)
   - `glmark2` / `vkmark` — GPU baseline
   - MangoHud — in-game frame times and 1% lows
   - `perf` + `bpftrace` — when a number is bad, find out why instead of guessing

7. **Boot time budget** — `systemd-analyze` + `blame` + `critical-chain`. Target ≤ 8 s firmware→login. Any unit over 300 ms needs a justification or a fix.

8. **Regression CI** — every gate's numbers are stored. CI fails the build if any metric regresses more than 10% without an approved note. This is the mechanism that actually keeps ILTRIC light — not good intentions.

9. **Publication rules** — `/docs/PERF-claims.md`
   Every published number must state: hardware, kernel version, ILTRIC build hash, tool + exact command, sample count, and variance. A number without these is deleted, not published.

---

## EXIT GATE — GATE 13 (re-runnable)
- [ ] Idle RAM ≤ 600 MB with compositor + shell + defender running, with a per-process breakdown.
- [ ] Boot ≤ 8 s with `critical-chain` output attached.
- [ ] zram matrix table published from your own runs.
- [ ] MGLRU on/off comparison published from your own runs.
- [ ] Frame-time 99th percentile within budget at the display's refresh rate.
- [ ] `cyclictest` max latency recorded before/after scheduler changes.
- [ ] Regression CI is live and demonstrably fails on an injected 15% regression.
- [ ] Zero claims in the repo violate `/docs/PERF-claims.md`.

## FAILURE MODES
- "Optimizing" by disabling features → any RAM saving must state what capability was removed.
- Benchmark cherry-picking → the harness runs the full set every time; partial runs are not results.
- Measuring in a VM and claiming metal → every number is tagged with its environment.

## NEXT STEP
`15-APPENDIX-TARGETS-MEASUREMENT.md`
