# Commercial readiness — measured assessment, 2026-11-12

**Status: `NOT_READY_FOR_COMMERCIAL`.** This document is the evidence-backed
answer to four requests: make the system commercially usable, install Linux
applications, make networking and the web work, and provide end-to-end proof
while improving the architecture. It records what is true today, what this
session changed, and what still blocks a commercial release. It deliberately
does not claim "100% compatibility": no operating system can, and this one
cannot yet install or run a Flatpak application at all.

The vocabulary is the one in
[`docs/master-plan/25-beta-1.0-gate-matrix.md`](master-plan/25-beta-1.0-gate-matrix.md):
`READY_FOR_BETA`, `CONDITIONALLY_READY`, `NOT_READY`, `NOT_VERIFIED`.

---

## 1. The four requests, one row each

| Request | State today | What blocks it | Where the blocker lives |
|---|---|---|---|
| Linux applications install and run | **NOT WORKING** | The store daemon and every launched application inherit the capability broker's sandbox: no `AF_INET`, no user namespaces, no writable `/home`. `flatpak install` cannot reach Flathub and `flatpak run` cannot create its namespaces. | `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service` (`ProtectHome=yes`, `RestrictAddressFamilies=AF_UNIX AF_NETLINK`, `IPAddressDeny=any`, `RestrictNamespaces=yes`) |
| Internet works | **PARTIAL** | The image now gets DHCP on real hardware (was a hard-coded QEMU lease), and the browser engine has network in its own unit. Applications launched through the broker do **not**: they inherit the same sandbox. | `litric-sysd.service`; untested on physical hardware |
| Web works | **CONDITIONALLY VERIFIED (engine on a host)** | The broker/Chromium path is exercised end to end on the development host. The in-guest proof exists for 2026-11-06. Neither is a claim about a physical machine. | `tools/browser-e2e.py`, `docs/GATES.md` |
| E2E and architecture | **PARTIAL** | Native install/launch/uninstall has a real-socket journey. There is no equivalent journey for the Flatpak path, and the launcher architecture is the reason. | `src/litric-shell/src/e2e_app_install.rs` |

---

## 2. What was changed on 2026-11-12

### 2.1 Flathub is provisioned, once, after the network is up

`flatpak` ships without any remote. A fresh image therefore had an App Store
that could only ever answer "no results", which is indistinguishable from a
broken catalog. `image/mkosi.extra/usr/lib/litric/flatpak-setup.sh` and
`litric-flatpak-setup.service` add `flathub` idempotently, tolerate a boot with
no network (exit zero, retry next boot), install nothing, and keep the fetch out
of `litric-sysd` by running in their own hardened unit with `AF_INET` allowed.

This makes the **catalog** reachable. It does not make an **install** work: see
§3.1.

### 2.2 Wired networking is correct on real hardware again

`20-wired.network` used to assign QEMU's slirp lease (10.0.2.15/24, gateway
10.0.2.2, DNS 10.0.2.3) to every `en*`/`eth*` interface. Inside the proof VM that
is correct; on a physical machine it is an address that is off-subnet or already
in use, so the link comes up and nothing routes. The file now asks for DHCP, and
the deterministic slirp lease moved to `10-qemu-slirp.network`, matched by QEMU's
default NIC MAC (`52:54:00:12:34:56`). `systemd-networkd` applies the first
matching file in lexical order, so the VM keeps its fixed address and physical
hardware gets DHCP.

`ci/lint-network-config.py` makes the split checkable (including a self-test that
proves the linter rejects a static default and a file that sorts too late). This
is a configuration-level change: **no image was built and no link was brought up
in this session** — the host has no `mkosi` and no `qemu-system-*`.

### 2.3 The portal gap is recorded, not papered over

`xdg-desktop-portal` is installed without a backend. The obvious
`xdg-desktop-portal-gtk` pulls in gtk3, which the desktop profile excludes as a
budget decision. A Flatpak application can therefore run but cannot open a file
chooser or a print dialog. The profile now says so at the point where a future
reader would otherwise add the package and silently break the budget.

---

## 3. The architecture defect that stops application compatibility

### 3.1 Applications inherit the broker's sandbox

`litric-sysd` is the capability broker. Its unit is deliberately locked down:

```
ProtectHome=yes
RestrictAddressFamilies=AF_UNIX AF_NETLINK
IPAddressDeny=any
RestrictNamespaces=yes
```

`LAUNCH`, `LAUNCHENV` and `LAUNCHPROFILE` fork the application **from this
process** (`src/litric-sysd/src/launch.rs`, `src/litric-sysd/src/compat.rs`), so
the child inherits every one of those restrictions. The consequences are not
subtle:

* a native application has **no network** and cannot read the user's home;
* `flatpak install --system` cannot reach Flathub, so the App Store write half
  can never succeed;
* `flatpak run` needs an unprivileged user namespace, which
  `RestrictNamespaces=yes` refuses;
* running Flatpak as the daemon's user is the wrong user model anyway — the
  desktop account is `litric`, and a system package manager runs as root.

The browser escapes this only because `litric-browser@.service` is a separate
unit running as `litric` with `AF_INET AF_INET6` allowed. That is the pattern the
application path needs, generalised.

### 3.2 The proposed fix: `litric-appd`

Introduce a session application runtime daemon, `litric-appd`, running as the
desktop user (`litric`), owning:

* Flatpak catalog reads and writes (`flatpak list/search/install/uninstall`);
* `flatpak run` and Steam/Proton launches;
* native launches that need the session's network and home.

`litric-sysd` keeps what it is good at: consent, capability decisions, and the
audit trail. It forwards a validated request to `litric-appd` over a socket in
`/run/litric/`, and returns the daemon's reply unchanged. The application child
is then a child of a unit whose sandbox is written for applications, not for the
broker.

Required properties, in this order:

1. `litric-appd` is a **separate unit**, `User=litric`, with
   `RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6 AF_NETLINK`, no
   `IPAddressDeny`, and `RestrictNamespaces=` limited to the namespace types
   Flatpak needs (`user net`).
2. It never decides *whether* an action is allowed. It executes a plan that
   `litric-sysd` already authorised, and it re-validates the plan's arguments
   with the same functions (`store::valid_app_id`, `compat::valid_flatpak_id`).
3. Store writes used to be `--system`; under a user daemon the choice is
   explicit. A single-user desktop should use `--user` (no polkit prompt, data
   under `~/.local/share/flatpak`). If system-wide installation is kept, the
   write needs a privileged helper, which is a second design decision and must
   not be hidden inside `litric-appd`.
4. Portal backend: a Litric-native implementation of the handful of portal
   interfaces the store and file chooser need, or a documented decision to ship
   gtk3 for the desktop profile. Doing neither is what exists now.

### 3.3 What "compatibility" can honestly mean

Flatpak is a **user-space application** runtime. It does not give:

* kernel modules or out-of-tree drivers;
* system services compiled against an older glibc;
* 32-bit-only binaries (unless the runtime ships the matching extension);
* anti-cheat or kernel-level game protection;
* GPU features the host driver does not expose.

So the commercial claim must be a **supported matrix**, not a percentage:
"GUI applications published as Flatpak on Flathub, on x86-64, with software
rendering or a supported Mesa driver". Windows applications through
`litric-winbridge` remain a separate, partial tier. Any "100%" language in a
store page or a sales deck would be false, and this repository's `G3` forbids
fabricated results.

---

## 4. End-to-end evidence run in this session

Filled from the commands actually executed on the development host; anything not
run is listed as **NOT RUN**.

| Check | Command | Result |
|---|---|---|
| Workspace tests | `cd src && cargo test --workspace` | 1,536 passed, 2 failed (fixture glibc; see §4.1) |
| Browser engine e2e | `tools/browser-e2e.py --broker ... --chromium /usr/bin/chromium` | PASS, 12/12 stages (see §4.2) |
| Network config lint + self-test | `python3 ci/lint-network-config.py [--self-test]` | PASS |
| Unit reference lint | `python3 ci/lint-unit-binaries.py` | PASS (new unit's `bash` is a base binary; skipped) |
| mkosi schema lint | `python3 ci/lint-mkosi.py` | 6 errors, all missing `keys/dev/*` in this checkout; **not caused by this change** |
| Image build | `tools/build.sh --profile desktop` | **NOT RUN** — no `mkosi`/Arch toolchain on this host |
| QEMU boot / link-up | `tools/qemu-guest-proof.sh` | **NOT RUN** — no `qemu-system-x86_64` on this host |
| Flatpak install of a real app | — | **NOT RUN** — `flatpak` is not installed on this host, and §3.1 blocks it in the image |

### 4.1 Workspace tests

`cargo test --workspace` on 2026-11-12: **1,536 passed, 2 failed**. The failures
are the two `litric-shell` journeys that exec the committed `weston-simple-shm`
fixture (`journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed`
and `journey_test_game_plays_fullscreen_and_stops`). The fixture's bundled
`libwayland-client.so.0` needs `GLIBC_2.38` and this host is `GLIBC 2.35`, so the
child exits before mapping a window. That is an environment mismatch, not a
regression, and it leaves the real-client path **not green on this host**.

The suite includes the native install/launch/uninstall journey over a real Unix
socket and the shell's browser journeys; it does not include a Flatpak journey,
because there is no working Flatpak launch path to drive.

### 4.2 Browser engine e2e

`tools/browser-e2e.py` drives `litric-browserd` over its real LBP/1 socket with a
real Chromium: handshake, keepalive, policy refusals, navigation to an HTTPS
page, a non-blank frame at the announced size, scroll, input, a second tab, and
shutdown. **All 12 stages passed, 0 skipped** (`navigation` reported
`title='Example Domain'`; `rendering` reported a 1280x800 BGRA frame with 254
sampled colours; the link click reached
`https://www.iana.org/help/example-domains`). It was run as an unprivileged user
because Chromium refuses to start as root without `--no-sandbox`, which the
broker correctly refuses to add. This proves the engine path on the development
host; it does not prove the in-guest path, and it does not prove an application
other than the browser has network.

---

## 5. Release plan with acceptance criteria

Ordered by what unblocks the most user-visible truth. Each phase ends with a
measurement, not a sentence.

1. **`litric-appd` + Flatpak install/run (P0).** A real Flatpak application
   installed from Flathub and launched into the compositor, with a store journey
   that fails when the network is unavailable rather than pretending a cached
   catalog is a live one. Acceptance: guest proof with a real app id, a window,
   and its pixels; `cargo test` journey over the appd socket.
2. **Portal backend (P0 for daily use).** File chooser and open-URI for Flatpak
   apps. Acceptance: a Flatpak app opens a file through the portal in the guest.
3. **Installer and first-boot account (P0).** The gate matrix records
   "no installer pipeline at all". Acceptance: install to an empty disk in QEMU,
   reboot from the installed disk, log in, reach the desktop.
4. **Physical-network proof (P0).** DHCP, DNS via `resolved`, Wi-Fi join, and a
   real page load on a physical NIC. Acceptance: recorded link state, lease,
   resolver answer, and browser frame.
5. **Compatibility matrix and honesty text (P1).** Publish what Flatpak, native
   ELF, Steam and Windows each do and do not cover; wire the matrix into the
   store UI so an unsupported request is refused before download.
6. **Compatibility environments and Windows tier (P2).** `litric-winbridge`
   bottles and the guest-window surface, with the resource cost stated before a
   VM starts.

---

## 6. What this document does not claim

* It does not claim a build, a boot, a link-up, or a Flatpak install: none of
  those ran here, and §4 marks them `NOT RUN`.
* It does not claim that provisioning Flathub makes the App Store work. It makes
  the catalog reachable; §3.1 is the remaining blocker.
* It does not claim "100% compatibility" for any runtime, and it gives the
  reasons in §3.3.
* It does not replace the gate matrix. `docs/master-plan/25-...` remains the
  authoritative row-per-capability view; this document adds the measured state
  and the next architectural step.
