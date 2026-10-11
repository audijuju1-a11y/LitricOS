# ILTRIC — PHASE 04 PROMPT: NETWORKING (PERMISSION-AWARE)

**Prerequisite:** GATE 03 passed.
**Deliverable:** lean, fast networking where network access is a *capability*, not a default.

---

## OBJECTIVE
Ethernet + Wi-Fi + IPv4/IPv6 + DNS + TLS working, with per-application network permission enforced at the OS level.

---

## TASKS

1. **Stack selection** — `iwd` + `systemd-networkd` as the core (lower memory, faster association than wpa_supplicant + NetworkManager). Provide an optional NetworkManager profile for laptops that need aggressive roaming/VPN UX.
   **Measure:** resident RAM of the networking stack and time-to-associate on Wi-Fi, both variants. Record both; choose with the number.

2. **Config** — `/core/network/`
   - `systemd-networkd` `.network` files for virtio (QEMU), Ethernet, Wi-Fi
   - `iwd` main.conf with `EnableNetworkConfiguration=false` (networkd owns addressing)
   - `systemd-resolved` with DNS-over-TLS `opportunistic` by default, `strict` toggle in settings

3. **Permission-aware networking** — implement `iltric.net`
   - Default for a sandboxed app: **no network**.
   - `net.client` grants outbound; `net.server` grants listening sockets.
   - Enforcement: per-app network namespace + a proxy/socket-activation path, or cgroup/BPF socket hooks. Choose one, document the trade-off (namespace = strong isolation, higher cost; BPF hook = cheap, coarser).
   - Every grant is audit-logged.

4. **Firewall + egress policy** — `/core/firewall/`
   nftables ruleset: default-deny inbound, allow established, per-slice egress marking so `game.slice` and `ai.slice` can be shaped or blocked independently.

5. **Latency tuning knobs** (exposed, not defaulted) — document in `/docs/NET-04-tuning.md`
   ```
   net.core.busy_poll / busy_read      # latency vs power
   NIC coalescing (ethtool -C)
   net.ipv4.tcp_congestion_control=bbr
   txqueuelen / fq_codel on the default qdisc
   ```
   These become Gaming Mode toggles in Phase 12. Each must carry a measured before/after.

6. **VPN + Bluetooth networking** — WireGuard support in-tree config; BlueZ PAN documented as phase-11 work.

7. **Offline correctness** — the system must fully boot, log in, and reach the desktop with **no network at all**. No blocking network calls in the boot path. Test with `-netdev none`.

---

## EXIT GATE — GATE 04
- [ ] DHCP + static + IPv6 SLAAC all work in QEMU.
- [ ] DNS-over-TLS verified with a packet capture (no plaintext :53 leaving the VM).
- [ ] A test app **without** `net.client` cannot open a socket — demonstrated, and the denial is audit-logged.
- [ ] A test app **with** `net.client` works normally.
- [ ] Boot with zero network is ≤ Gate 01 boot time (no network stalls).
- [ ] Networking stack resident RAM recorded; iwd vs wpa_supplicant compared with numbers.

## FAILURE MODES
- Network namespace per app breaks mDNS/local discovery → document which capabilities re-enable it (`net.local`) rather than weakening the default.
- resolved strict mode breaks captive portals → provide an explicit, temporary, user-initiated "portal mode".

## NEXT STEP
`06-PHASE-05-GRAPHICS.md`
