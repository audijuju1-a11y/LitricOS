# ILTRIC — APPENDIX: TARGETS, BASELINES & MEASUREMENT PROTOCOL

Paste this alongside any phase when numbers are in play.

---

## 1. TARGETS ILTRIC IS HELD TO

| Metric | ILTRIC target | Gate |
|---|---|---|
| Idle RAM, headless | ≤ 120 MB | 02 |
| Idle RAM, full desktop + defender | **≤ 600 MB** | 06 / 07 / 13 |
| Boot, firmware → login | **≤ 8 s** (SSD) | 01 / 13 |
| Update apply + commit | ≤ 60 s (excl. download) | 08 |
| Rollback | 1 reboot, automatic | 08 |
| Defender resident RAM | ≤ 100 MB | 07 |
| Defender on-access penalty (cached) | ≤ single-digit ms | 07 |
| Audio latency, desktop | ~5.3 ms (quantum 256 @ 48 kHz) | audio |
| Audio latency, pro/gaming | 1.3–2.7 ms (quantum 64–128 @ 48 kHz) | audio |
| AI 3B Q4_K_M | TTFT < 500 ms, > 15 tok/s | 09 |
| Frame time | 60 Hz 16.67 / 120 Hz 8.33 / 144 Hz 6.94 / 240 Hz 4.17 ms | 05 / 12 |
| Gaming vs Windows 11, same HW | match or beat in ≥2 of 3 titles | 12 |

---

## 2. BASELINES TO BEAT (measured elsewhere — re-verify locally before quoting)

| System | Idle RAM |
|---|---|
| Bare kernel + BusyBox initramfs | ~17 MB |
| Alpine base (musl/BusyBox) | < 50 MB |
| Arch base, no DE | < 100 MB |
| Void / Debian minimal | ~100–200 MB |
| labwc / Sway / Hyprland desktop | ~300–500 MB |
| XFCE / LXQt | ~500–700 MB |
| GNOME / KDE Plasma 6 | ~1–2 GB |
| **Windows 11, clean idle** | **2.2–3.3 GB** (~6 GB on an 8 GB PC) |

Gaming reference: SteamOS vs Windows 11 on identical Legion Go S hardware, 1920×1200 High — Returnal 33 FPS steady vs Windows "barely managed 18 FPS with Lenovo's official drivers"; Cyberpunk ~60 vs ~45; Witcher 3 76 vs 66; SteamOS won 4 of 5 titles (~24% average across wins), Windows's one win by ~2%. Battery ~15–20% better, some titles nearly double.

**Note:** desktop-class deltas differ from handheld APU deltas. Do not generalize handheld results to desktops.

---

## 3. MEASUREMENT PROTOCOL (mandatory for every number)

Every recorded number carries:
```
metric      :
value       :  (mean ± stddev, n = ?)
tool        :
exact cmd   :
environment :  QEMU | metal  |  host OS  |  CPU/GPU/RAM
kernel      :
ILTRIC build:  <hash>
date        :
```
Rules:
- minimum **n = 5** runs for any timing metric; report mean and stddev
- 60 s settle before any idle measurement
- discard the first run after boot (cold cache) unless cold-start is the metric
- VM numbers are never presented as metal numbers
- if you did not run it, the value is `NOT RUN` — never an estimate wearing a number's clothes

---

## 4. STANDARD TOOL SET

```
boot        systemd-analyze / blame / critical-chain, bootchart
memory      free -m, smem (PSS), systemd-cgtop, zramctl
CPU/latency cyclictest, perf, bpftrace
storage     fio (seq + 4K random, encrypted vs plain)
CLI timing  hyperfine
GPU         glmark2, vkmark, Unigine
in-game     MangoHud (frame times + 1% lows)
suite       phoronix-test-suite
compositor  presentation-time deltas, missed-flip counter
network     iperf3, ping (p99), packet capture for privacy checks
AI          TTFT + tok/s from the runtime's own timing output
```

---

## 5. HONEST LIMITATIONS (must appear in ILTRIC's public docs)

1. **Kernel-anti-cheat titles will not work.** EAC/BattlEye support requires per-developer opt-in; several major titles have declined or withdrawn it. ILTRIC cannot change this.
2. **Windows-host QEMU cannot do accelerated guest graphics** (virgl/venus generally disabled in Windows builds) and **virtiofs is Linux-host-only**. Graphics work needs a Linux host or WSL2.
3. **WHPX has a live SMP boot bug** (QEMU #3178) plus reported CPUID/AVX issues — keep a TCG fallback.
4. **NPU acceleration on Linux is essentially Intel-only in practice** (intel_vpu + OpenVINO). AMD XDNA driver is upstream but the runtime stack is immature. No broad NPU claims.
5. **A built-in lightweight defender will not match commercial AV detection rates** and carries false-positive risk. It is sandbox-first and advisory.
6. **bcachefs is not production-ready for this project.** ZFS is excellent but CDDL keeps it out-of-tree.
7. **musl was rejected on evidence**, not taste: documented multi-thread malloc lock contention (a 13× futex penalty in one analysis; ~4× slower multi-thread in Chainguard's testing) plus small default thread stacks and DNS edge cases.
8. **Effort is real.** ~18–36 person-months to a daily-driver prototype for a small team doing integration — not from-scratch kernel work. Solo timelines can double.
9. **ILTRIC will not beat Linux and Windows on everything.** The winnable fights are memory footprint, smoothness, and a curated gaming experience. It will consciously trail on breadth of hardware support and anti-cheat coverage. Say so.

---

## 6. LICENSING NOTES

- Linux kernel: **GPLv2** — ship modified kernels only with corresponding source available. Userspace using syscalls is not derivative.
- Mesa: MIT. PipeWire: MIT. systemd: LGPL/GPL. Flatpak/bubblewrap: LGPL/GPL.
- **ClamAV: GPLv2** — keep it a separable, on-demand component.
- **YARA: BSD-3** — permissive, safe to embed.
- A fully closed-source ILTRIC would face real constraints. Keep GPL components separately packaged and offer source.

---

## 7. PRIOR ART — WHAT TO COPY, WHAT TO AVOID

| Project | Lesson |
|---|---|
| SerenityOS | Passion + public building creates community — but rewriting everything means it mostly runs in emulators. **Don't rewrite the kernel.** |
| Redox | Microkernel purity costs years before hardware/app support arrives. |
| Haiku | Cohesion and lightness are genuinely felt by users; a small app ecosystem is the price. |
| **ChimeraOS / Bazzite / CachyOS / Nobara** | **The most relevant models.** Integration distros on mainline Linux already delivering the gaming/desktop wins. ILTRIC's fastest credible path is the same shape — plus a from-first-principles shell, defender, and capability platform. |

---

## 8. EFFORT ESTIMATE (small team, integration-first)

| Phase | Person-months |
|---|---|
| P00–P01 architecture + bootable image | 1–2 |
| P02–P03 base userland + storage + updates | 2–4 |
| P05–P06 compositor + shell | 4–8 |
| audio | 0.5–1 |
| P12 gaming mode | 2–4 |
| P07 defender + sandbox | 2–4 |
| P09 AI | 2–3 |
| P11 hardware + power | 1–2 |
| P10 app platform | 2–3 |
| **Total to daily-driver prototype** | **~18–36** |
