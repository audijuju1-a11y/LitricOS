# LITRIC — PHASE 11 PROMPT: REAL HARDWARE BRING-UP & POWER

**Prerequisite:** GATE 10 passed. **Do not start this phase early.**
**Deliverable:** LITRIC running on real machines, with power management, and an honest hardware compatibility list.

---

## OBJECTIVE
Move from VM to metal without ever putting the developer's primary OS at risk.

> **SAFETY RULE:** first real-hardware target is a **spare machine or an external USB/NVMe enclosure**. Dual-boot on the primary machine is only permitted after a successful install on spare hardware, with a verified full backup, and an explicit request from the user. Every install script must refuse to write to a disk that contains an existing OS unless a `--i-really-mean-it` flag plus the disk serial is supplied.

---

## TASKS

1. **Bring-up ladder** — one step at a time, gate each:
   ```
   QEMU -> spare laptop/desktop -> second machine (different vendor)
        -> discrete GPU -> Wi-Fi/BT variety -> laptop power/suspend -> other arch
   ```

2. **Firmware & updates** — ship `linux-firmware`; integrate `fwupd` + LVFS for device firmware and microcode. Firmware updates go through the Phase 08 signing/rollback discipline.

3. **Storage & boot on metal** — NVMe/SATA detection, GPT/ESP creation, Secure Boot key enrollment flow documented for real firmware (not just OVMF).

4. **Power management** — `/hw/power/`
   - **power-profiles-daemon (PPD 0.30+)** as the default (integrates `amd_pstate` EPP and platform profiles; avoids TLP conflicts)
   - expose TLP-style deep knobs in the Expert settings tier
   - governors: `schedutil` default; `performance` under Gaming Mode
   - `amd_pstate=active` / EPP values (`power`, `balance_performance`, `performance`) selectable
   - **Measure battery life** in three profiles with a repeatable workload. Publish the numbers or publish nothing.

5. **Laptop subsystems** — libinput touchpad gestures, backlight, `iio-sensor-proxy` (rotation/ALS), `fprintd`, lid/suspend via **s2idle** (S3 is deprecated on modern platforms), Thunderbolt/USB4 via `bolt`.

6. **Wireless** — `iwd` on real chipsets; BlueZ + LE Audio. Record association times and roaming behavior per chipset.

7. **Hardware compatibility list** — `/docs/HCL.md`
   Every entry must be one of: `TESTED-WORKING`, `TESTED-PARTIAL` (with what fails), `TESTED-BROKEN`, `UNTESTED`.
   **"Works on all hardware" is a banned phrase.**

8. **Thermals & sustained load** — 30-minute sustained GPU+CPU load; record throttling behavior, fan curve interaction, and frame-time stability at minute 1 vs minute 30.

---

## EXIT GATE — GATE 11
- [ ] Clean install on spare hardware from a USB image, unattended, without touching any other disk. Demonstrated twice on different machines.
- [ ] Suspend/resume 20× consecutively with no failure; measure resume time.
- [ ] Wi-Fi + Bluetooth + audio + display + touchpad all `TESTED-WORKING` on at least one laptop.
- [ ] Battery life measured in all three power profiles with the same workload; deltas recorded.
- [ ] Sustained-load test shows frame-time stability within 10% from minute 1 to minute 30, or the throttling is documented.
- [ ] HCL published with zero unverified claims.
- [ ] All previous gates re-run on real hardware; regressions listed.

## FAILURE MODES & ROLLBACK
- No display on boot (GPU driver) → the image must always ship a `nomodeset`/basic-framebuffer boot entry.
- Wi-Fi firmware missing → installer detects and warns *before* wiping the install medium.
- Secure Boot enrollment bricks boot → documented firmware-reset procedure and a signed fallback shim path.

## NEXT STEP
`13-PHASE-12-GAMING-MODE.md` (may be run any time after GATE 05)
