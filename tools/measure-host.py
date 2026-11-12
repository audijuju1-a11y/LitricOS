#!/usr/bin/env python3
"""Measure the shipping LitricOS binaries on this host, with no image and no display.

WHY THIS EXISTS
---------------
`tools/measure-idle.py` is the real measurement of a booted machine, but it
needs the mkosi image and QEMU. When neither is available, the performance
section of the research corpus and `docs/PERF-budget.md` have no number at all,
and an unmeasured budget is indistinguishable from a met one. This script
measures what this host can actually run: the release binaries in their
headless modes.

WHAT IS MEASURED
----------------
* `litric-shell --png --once` -- the shipping composition path (`views::draw`
  over the real `Compositor`) plus PNG encode, at two canvas sizes. This is the
  one-frame cost; it is **not** a present-to-display frame time.
* `litric-filesd`, `litric-sysd`, `litric-notifyd` in `--serve` mode -- time
  from spawn to an accepting socket, peak RSS, and idle CPU over a quiet
  window, read from the kernel's own `/proc` and `wait4` rusage.

Peak RSS comes from `wait4(2)`'s `ru_maxrss` (one-shot runs) or the process's
own `VmHWM` (daemons), never from an estimate. Every run gets a fresh temporary
root and socket path so one run cannot observe another's state.

WHAT THIS DOES NOT CLAIM
------------------------
* No display: the compositor is composed, never presented. `present()` cost,
  the DRM/KMS path and GPU use are outside this measurement.
* No boot: systemd, the kernel and the image are not in these numbers, so they
  say nothing about boot time or whole-machine idle RAM.
* A daemon's idle CPU here is a daemon with no client, not a desktop with the
  shell polling it every two seconds.
* Host CPU frequency scaling, page cache and scheduler noise move these
  numbers; the median of several runs is reported with min/max beside it so the
  spread is visible rather than hidden.

Usage:
    python3 tools/measure-host.py [--runs 7] [--json PATH]

Exit code: 0 when every component produced at least one measurement, 1 when a
component could not be run (its reason is recorded, never a substituted number).
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import socket
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RELEASE = ROOT / "src" / "target" / "release"
DEFAULT_JSON = ROOT / "validation" / "perf" / "host-measurements.json"
CLK_TCK = os.sysconf("SC_CLK_TCK")
READY_TIMEOUT_S = 10.0
SETTLE_S = 1.5
CPU_WINDOW_S = 2.0


def git_head() -> str:
    try:
        out = subprocess.run(
            ["git", "-C", str(ROOT), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=False,
        )
        return out.stdout.strip() or "unknown"
    except OSError:
        return "unknown"


def proc_field(pid: int, key: str) -> int | None:
    """One integer out of /proc/<pid>/status, in kB for memory keys."""
    try:
        with open(f"/proc/{pid}/status", "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                if line.startswith(key + ":"):
                    parts = line.split()
                    if len(parts) >= 2 and parts[1].isdigit():
                        return int(parts[1])
    except OSError:
        return None
    return None


def cpu_ticks(pid: int) -> int | None:
    """utime + stime, in clock ticks, from /proc/<pid>/stat."""
    try:
        with open(f"/proc/{pid}/stat", "rb") as f:
            data = f.read()
    except OSError:
        return None
    close = data.rfind(b")")
    if close < 0:
        return None
    fields = data[close + 2 :].split()
    if len(fields) < 13:
        return None
    try:
        # fields[0] is the state (field 3 of stat), so utime (14) is fields[11].
        return int(fields[11]) + int(fields[12])
    except ValueError:
        return None


def stats(values: list[float]) -> dict[str, float] | None:
    if not values:
        return None
    return {
        "median": round(statistics.median(values), 3),
        "min": round(min(values), 3),
        "max": round(max(values), 3),
        "n": len(values),
    }


def read_tail(path: Path, limit: int = 400) -> str:
    try:
        text = path.read_text(encoding="utf-8", errors="replace").strip()
    except OSError:
        return ""
    return text[-limit:]


def run_once(cmd: list[str], work: Path) -> dict:
    """One process, measured from just before spawn to just after wait4."""
    work.mkdir(parents=True, exist_ok=True)
    stdout_path = work / "stdout"
    stderr_path = work / "stderr"
    with open(stdout_path, "wb") as out, open(stderr_path, "wb") as err:
        started = time.monotonic()
        proc = subprocess.Popen(cmd, stdout=out, stderr=err, stdin=subprocess.DEVNULL)
        try:
            _pid, status, usage = os.wait4(proc.pid, 0)
        except ChildProcessError:
            proc.wait()
            return {
                "exit": proc.returncode,
                "wall_ms": None,
                "peak_rss_kb": None,
                "stderr": read_tail(stderr_path),
            }
        wall_ms = (time.monotonic() - started) * 1000.0
    code = os.waitstatus_to_exitcode(status)
    proc.returncode = code
    return {
        "exit": code,
        "wall_ms": wall_ms,
        # Linux reports ru_maxrss in kilobytes.
        "peak_rss_kb": int(usage.ru_maxrss),
        "stdout": stdout_path.read_text(encoding="utf-8", errors="replace"),
        "stderr": read_tail(stderr_path) if code != 0 else "",
    }


def wait_for_socket(path: Path, deadline: float) -> bool:
    while time.monotonic() < deadline:
        if path.exists():
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.settimeout(0.2)
            try:
                sock.connect(str(path))
                return True
            except OSError:
                pass
            finally:
                sock.close()
        time.sleep(0.01)
    return False


def run_daemon(cmd: list[str], sock: Path, work: Path) -> dict:
    """Spawn a server, time it to an accepting socket, sample it, stop it."""
    work.mkdir(parents=True, exist_ok=True)
    stderr_path = work / "stderr"
    with open(work / "stdout", "wb") as out, open(stderr_path, "wb") as err:
        started = time.monotonic()
        proc = subprocess.Popen(cmd, stdout=out, stderr=err, stdin=subprocess.DEVNULL)
        ready = wait_for_socket(sock, started + READY_TIMEOUT_S)
        ready_ms = (time.monotonic() - started) * 1000.0
        if not ready:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
            return {
                "ready_ms": None,
                "rss_kb": None,
                "peak_rss_kb": None,
                "idle_cpu_pct": None,
                "reason": "socket did not accept within "
                f"{READY_TIMEOUT_S:.0f}s; stderr: {read_tail(stderr_path) or '(empty)'}",
            }
        time.sleep(SETTLE_S)
        rss_kb = proc_field(proc.pid, "VmRSS")
        peak_kb = proc_field(proc.pid, "VmHWM")
        before = cpu_ticks(proc.pid)
        time.sleep(CPU_WINDOW_S)
        after = cpu_ticks(proc.pid)
        cpu_pct = None
        if before is not None and after is not None:
            cpu_pct = (after - before) / CLK_TCK / CPU_WINDOW_S * 100.0
        proc.terminate()
        try:
            _pid, _status, usage = os.wait4(proc.pid, 0)
            rusage_kb = int(usage.ru_maxrss)
        except ChildProcessError:
            proc.wait()
            rusage_kb = None
        proc.returncode = 0
    reasons = []
    if rss_kb is None:
        reasons.append("VmRSS not readable before shutdown")
    if peak_kb is None:
        reasons.append("VmHWM not readable before shutdown")
    if cpu_pct is None:
        reasons.append("CPU counters not readable")
    return {
        "ready_ms": ready_ms,
        "rss_kb": rss_kb,
        "peak_rss_kb": peak_kb if peak_kb is not None else rusage_kb,
        "idle_cpu_pct": cpu_pct,
        "reason": "; ".join(reasons) or None,
    }


def bench_component(name: str, binary: Path, note: str, runs: int, base: Path) -> dict:
    """Run an example that times itself, and aggregate its per-scene medians.

    `render_bench` prints `scene.metric=value` lines after its own timed runs,
    so the frame numbers come from the same code the desktop draws with rather
    than from a wall clock around the whole process.
    """
    scenes: dict[str, dict[str, list[float]]] = {}
    walls, peaks, failures = [], [], []
    for i in range(runs):
        work = base / f"{name.replace(' ', '_')}" / f"run{i}"
        work.mkdir(parents=True, exist_ok=True)
        got = run_once([str(binary), str(work)], work)
        if got["exit"] != 0 or got["wall_ms"] is None:
            failures.append(f"run {i}: exit {got['exit']} {got['stderr']}".strip())
            continue
        walls.append(got["wall_ms"])
        if got["peak_rss_kb"]:
            peaks.append(float(got["peak_rss_kb"]))
        for line in got["stdout"].splitlines():
            key, sep, value = line.partition("=")
            if not sep or "." not in key:
                continue
            scene, _dot, metric = key.rpartition(".")
            if metric not in ("median_ms", "min_ms"):
                continue
            try:
                number = float(value)
            except ValueError:
                continue
            scenes.setdefault(scene, {}).setdefault(metric, []).append(number)
    aggregate = {
        scene: {
            "median_ms": stats(metrics["median_ms"]) if "median_ms" in metrics else None,
            "min_ms": stats(metrics["min_ms"]) if "min_ms" in metrics else None,
        }
        for scene, metrics in scenes.items()
    }
    return {
        "name": name,
        "kind": "bench",
        "status": "MEASURED" if walls else "NOT RUN",
        "runs_requested": runs,
        "runs_measured": len(walls),
        "process_wall_ms": stats(walls),
        "peak_rss_kb": stats(peaks),
        "scenes": aggregate,
        "reason": "; ".join(failures) or None,
        "notes": note,
    }


def one_shot_component(name: str, cmd: list[str], note: str, runs: int, base: Path) -> dict:
    walls, peaks, failures = [], [], []
    for i in range(runs):
        got = run_once(cmd, base / f"{name.replace(' ', '_')}" / f"run{i}")
        if got["exit"] != 0 or got["wall_ms"] is None:
            failures.append(f"run {i}: exit {got['exit']} {got['stderr']}".strip())
            continue
        walls.append(got["wall_ms"])
        if got["peak_rss_kb"]:
            peaks.append(float(got["peak_rss_kb"]))
    return {
        "name": name,
        "kind": "one-shot",
        "status": "MEASURED" if walls else "NOT RUN",
        "runs_requested": runs,
        "runs_measured": len(walls),
        "wall_ms": stats(walls),
        "peak_rss_kb": stats(peaks),
        "reason": "; ".join(failures) or None,
        "notes": note,
    }


def daemon_component(name: str, cmd: list[str], note: str, runs: int, base: Path) -> dict:
    ready, rss, peak, cpu, failures = [], [], [], [], []
    for i in range(runs):
        work = base / f"{name.replace(' ', '_')}" / f"run{i}"
        work.mkdir(parents=True, exist_ok=True)
        sock = work / "sock"
        got = run_daemon(cmd(sock, work), sock, work)
        if got["ready_ms"] is None:
            failures.append(f"run {i}: {got['reason']}")
            continue
        ready.append(got["ready_ms"])
        if got["rss_kb"] is not None:
            rss.append(float(got["rss_kb"]))
        if got["peak_rss_kb"] is not None:
            peak.append(float(got["peak_rss_kb"]))
        if got["idle_cpu_pct"] is not None:
            cpu.append(got["idle_cpu_pct"])
    return {
        "name": name,
        "kind": "daemon",
        "status": "MEASURED" if ready else "NOT RUN",
        "runs_requested": runs,
        "runs_measured": len(ready),
        "ready_ms": stats(ready),
        "rss_kb": stats(rss),
        "peak_rss_kb": stats(peak),
        "idle_cpu_pct": stats(cpu),
        "reason": "; ".join(failures) or None,
        "notes": note,
    }


def table(components: list[dict]) -> str:
    lines = [
        "component                      n   ready/wall ms (med)   rss kB (med)   peak kB (med)   idle cpu % (med)",
        "-" * 112,
    ]
    for comp in components:
        primary = comp.get("ready_ms") or comp.get("wall_ms") or comp.get("process_wall_ms")
        n = comp.get("runs_measured", 0)
        if comp["status"] != "MEASURED" or primary is None:
            lines.append(f"{comp['name'][:30]:<30} 0   NOT RUN: {(comp.get('reason') or '')[:64]}")
            continue
        rss = comp.get("rss_kb") or {}
        peak = comp.get("peak_rss_kb") or {}
        cpu = comp.get("idle_cpu_pct") or {}
        # A dash, not a zero: a one-shot process has no steady-state RSS to
        # report, and printing 0 kB would read as a measurement of nothing.
        rss_cell = f"{rss['median']:>8.0f}" if rss else "       -"
        peak_cell = f"{peak['median']:>8.0f}" if peak else "       -"
        cpu_cell = f"{cpu['median']:>5.2f}" if cpu else "    -"
        lines.append(
            f"{comp['name'][:30]:<30} {n:<3} "
            f"{primary['median']:>10.1f} [{primary['min']:.1f}-{primary['max']:.1f}]"
            f"   {rss_cell}"
            f"      {peak_cell}"
            f"        {cpu_cell}"
        )
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="Measure the headless LitricOS binaries.")
    ap.add_argument("--runs", type=int, default=7, help="runs per component (default 7)")
    ap.add_argument("--json", type=Path, default=DEFAULT_JSON, help="where to write the results")
    args = ap.parse_args()
    if args.runs < 1:
        raise SystemExit("measure-host: --runs must be at least 1")

    missing = [p.name for p in (RELEASE / "litric-shell", RELEASE / "litric-filesd",
                                RELEASE / "litric-sysd", RELEASE / "litric-notifyd")
               if not p.exists()]
    if missing:
        raise SystemExit(
            "measure-host: missing release binaries: "
            + ", ".join(missing)
            + " (run `cargo build --release --workspace` in src/ first)"
        )

    base = Path(tempfile.mkdtemp(prefix="litric-measure-", dir="/tmp"))
    shell = str(RELEASE / "litric-shell")
    filesd = str(RELEASE / "litric-filesd")
    sysd = str(RELEASE / "litric-sysd")
    notifyd = str(RELEASE / "litric-notifyd")

    components = [
        one_shot_component(
            "litric-shell --once 1280x800",
            [shell, "--png", str(base / "frame-1280.png"), "--size", "1280x800", "--once",
             "--status", str(base / "status-1280"), "--socket", str(base / "none.sock")],
            "process start + one composed frame + PNG encode; no present-to-display",
            args.runs,
            base,
        ),
        one_shot_component(
            "litric-shell --once 1920x1080",
            [shell, "--png", str(base / "frame-1920.png"), "--size", "1920x1080", "--once",
             "--status", str(base / "status-1920"), "--socket", str(base / "none.sock")],
            "same path at 2.25x the pixels, to show how the one-frame cost scales",
            args.runs,
            base,
        ),
        daemon_component(
            "litric-filesd --serve",
            lambda sock, work: [filesd, "--socket", str(sock), "--root", str(work / "root")],
            "spawn to accepting socket; RSS with no client",
            args.runs,
            base,
        ),
        daemon_component(
            "litric-sysd --serve",
            lambda sock, work: [sysd, "--serve", "--socket", str(sock), "--root", "/"],
            "spawn to accepting socket; facts read from this host's real /proc",
            args.runs,
            base,
        ),
        daemon_component(
            "litric-notifyd --serve",
            lambda sock, work: [notifyd, "--serve", "--socket", str(sock),
                                "--state", str(work / "queue.conf")],
            "spawn to accepting socket; empty queue",
            args.runs,
            base,
        ),
    ]
    bench = RELEASE / "examples" / "render_bench"
    components.append(
        bench_component(
            "render_bench",
            bench,
            "the example's own 15-frame timings; composition + file-backed present, no DRM",
            args.runs,
            base,
        )
        if bench.exists()
        else {
            "name": "render_bench",
            "kind": "bench",
            "status": "NOT RUN",
            "runs_requested": args.runs,
            "runs_measured": 0,
            "reason": f"{bench} is missing: build it with "
            "`cargo build --release -p litric-shell --examples`",
        }
    )

    payload = {
        "schema": 1,
        "generated_unix": int(time.time()),
        "git_head": git_head(),
        "host": {
            "uname": platform.platform(),
            "machine": platform.machine(),
            "nproc": os.cpu_count(),
            "mem_total_kb": next(
                (int(line.split()[1]) for line in open("/proc/meminfo", encoding="utf-8")
                 if line.startswith("MemTotal:")),
                None,
            ),
        },
        "runs_per_component": args.runs,
        "components": components,
    }
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    print(table(components))
    print()
    bench_rows = [c for c in components if c.get("scenes")]
    if bench_rows:
        print("render_bench scenes (median of per-run medians, 15 frames each)")
        print("-" * 66)
        for comp in bench_rows:
            for scene, metrics in comp["scenes"].items():
                median = metrics.get("median_ms") or {}
                if median:
                    print(f"  {scene:<38} {median['median']:>8.2f} ms")
        print()
    print(f"json : {args.json.relative_to(ROOT) if args.json.is_relative_to(ROOT) else args.json}")
    print(f"head : {payload['git_head']}")
    print("note : one composed frame + PNG encode, not a present-to-display frame time;")
    print("       no boot, no systemd, no GPU. See the module docstring for the limits.")
    blocked = [c["name"] for c in components if c["status"] != "MEASURED"]
    if blocked:
        print("NOT RUN: " + ", ".join(blocked), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
