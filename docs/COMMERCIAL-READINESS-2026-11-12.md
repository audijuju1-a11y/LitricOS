# Commercial readiness — measured assessment, 2026-11-12

**Status: `NOT_READY_FOR_COMMERCIAL`.** This document is the evidence-backed
answer to four requests: make the system commercially usable, install Linux
applications, make networking and the web work, and provide end-to-end proof
while improving the architecture. It records what is true today, what this
session changed, and what still blocks a commercial release. It deliberately
does not claim "100% compatibility": no operating system can, and this one has
not yet been shown, in a boot, to install or run a Flatpak application.

The vocabulary is the one in
[`docs/master-plan/25-beta-1.0-gate-matrix.md`](master-plan/25-beta-1.0-gate-matrix.md):
`READY_FOR_BETA`, `CONDITIONALLY_READY`, `NOT_READY`, `NOT_VERIFIED`.

---

## 1. The four requests, one row each

| Request | State today | What blocks it | Where the blocker lives |
|---|---|---|---|
| Linux applications install and run | **IMPLEMENTED, NOT VERIFIED IN A BOOT** | The executor half now exists: `litric-appd` runs in a unit with the network, home and namespaces Flatpak needs, and `litric-sysd` forwards only already-authorised store work and launches to it over a `0600` socket. The forwarding path is proven over a real socket; a real `flatpak install`/`flatpak run` is still **NOT RUN** (no `flatpak` here, no image build). | Verified by `src/litric-sysd/src/appd.rs`; the boot proof is the open item |
| Internet works | **PARTIAL** | The image now gets DHCP on real hardware (was a hard-coded QEMU lease), and launched applications no longer inherit the broker's `IPAddressDeny=any`: they are started by `litric-appd`, which allows `AF_INET`/`AF_INET6`. Untested on physical hardware. | `litric-appd.service`; physical link **NOT RUN** |
| Web works | **CONDITIONALLY VERIFIED (engine on a host)** | The broker/Chromium path is exercised end to end on the development host. The in-guest proof exists for 2026-11-06. Neither is a claim about a physical machine. | `tools/browser-e2e.py`, `docs/GATES.md` |
| E2E and architecture | **PARTIAL** | Native install/launch/uninstall has a real-socket journey; the broker→runtime-daemon path now has one too (catalog read, install plan, launch, absent-daemon refusal). The Flatpak *tool* is still simulated by a recorded runner. | `src/litric-shell/src/e2e_app_install.rs`, `src/litric-sysd/src/appd.rs` tests |

---

## 2. What was changed on 2026-11-12

### 2.1 Flathub is provisioned, once, after the network is up

`flatpak` ships without any remote. A fresh image therefore had an App Store
that could only ever answer "no results", which is indistinguishable from a
broken catalog. `image/mkosi.extra/usr/lib/litric/flatpak-setup.sh` and
`litric-flatpak-setup.service` add `flathub` idempotently, tolerate a boot with
no network (exit zero, retry next boot), install nothing, and keep the fetch out
of `litric-sysd` by running in their own hardened unit with `AF_INET` allowed.

This makes the **catalog** reachable. It does not, by itself, make an **install**
work: the sandbox blocker and the executor that removes it are §3.1 and §3.2.

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

### 2.4 The broker no longer forks applications

`litric-sysd` had two jobs with contradictory sandboxes. It is now only the
capability broker; a second unit, `litric-appd`, executes store work and
launches in a namespace that has the network, a home and Flatpak's namespaces.
The forwarding seam is `LITRIC_APPD_SOCKET` (set by the sysd unit); without it,
every existing test and a development host keep the old in-process path
unchanged. `litric-sysd` prints `litric.sysd.appd=<path|none>` and
`litric.sysd.execution=appd|local` at start, and `litric-appd` prints
`litric.appd.flatpak=<present|missing>` before it serves — facts a boot proof can
read rather than infer from a unit file.

### 2.5 The home directory, the busy answer, and the executor's journal

Four smaller defects came out of the same reading of the image:

* **`/var/home/litric` did not exist.** `sysusers.d/litric.conf` names it as the
  account's home, and `/var` is its own btrfs partition
  (`mkosi.repart/20-var.conf`), so the comment claiming `/var/home` "cannot be
  created at boot" was wrong for every profile above `nano`.
  `tmpfiles.d/litric-home.conf` now creates `/var/home` (0755) and
  `/var/home/litric` (0700, owned by `litric`). Without it, every tool that
  reads `$HOME` from passwd — D-Bus, xdg-user-dirs, `flatpak --user` — pointed
  at a path that did not exist.
* **Every tool script lacked the execute bit.** `tools/build.sh` is documented as
  `./tools/build.sh` in this repository and failed with "Permission denied";
  only `tools/browser-e2e.py` was `0755`. All of `tools/*.sh` and `tools/*.py`
  are now executable, and `./tools/gen-dev-keys.sh` runs (idempotent; it refuses
  to overwrite an existing key without `--force`).
* **A second application operation queued behind the first.** One `dispatch`
  held the state lock for the whole plan — up to 900 s for an install — so a
  second store write waited with no way to tell "busy" from "hung".
  `server.rs` now takes a separate operation slot with `try_lock` and answers
  `ERR busy` at once. Catalog reads are not application operations and are
  unaffected.
* **The executor left no record.** The broker audits decisions; nothing recorded
  what was executed. `litric-appd` writes one journal line per request
  (`litric.appd.request=<kind> result=ok|err kind=… lines=N ms=…`) and prints
  `litric.appd.pid=` and `litric.appd.home=` at start, so "the install ran and
  took 400 s" is readable rather than inferred.

A failed install is retried by re-sending the same request: Flatpak's install is
idempotent for an already-installed id, and `ERR busy` says when not to try. No
automatic retry loop is added — a broker that retried by itself would turn a
broken remote into an hourly download.

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
unit running as `litric` with `AF_INET AF_INET6` allowed. The application path
now follows the same pattern; §3.2 is what was built.

### 3.2 The fix implemented: `litric-appd`

The broker and the executor are now two processes:

| Unit | Decides | Runs | Sandbox |
|---|---|---|---|
| `litric-sysd` | capability, consent, audit | nothing application-related | no network, no home, no namespaces |
| `litric-appd` | nothing | Flatpak catalog reads/writes and every launch | network, home, Flatpak's namespaces |

`litric-sysd` forwards an already-authorised request over
`/run/litric/appd.sock` and returns the daemon's reply unchanged. The protocol
is closed: four catalog reads (`STORE LIST/SEARCH/UPDATES/SANDBOX`), the store
and sandbox writes, and one `RUN` line whose target, arguments and environment
are hex-encoded so a path containing a space cannot become a second field. The
daemon has no capability broker and no client identity; it re-validates every
argument with the same functions the broker used (`store::valid_app_id`,
`compat::resolve`, `launch::validate`).

Three decisions are worth stating because they are not the obvious ones:

1. **It runs as root in Phase 1.** The image has no `/var/home` (the root
   filesystem is erofs behind dm-verity), and `flatpak --system` writes
   `/var/lib/flatpak` without a polkit prompt only for root. A `--user`
   installation would need a writable home that does not exist.
2. **Applications therefore run as root too.** That is a real limitation, not a
   detail: Flatpak reports that it is running as root and relaxes parts of its
   own policy. User separation is Phase 2, and its blocker is named: the
   compositor's Wayland socket lives in `/run/litric` (mode `0700`, owned by
   root), so an application started as `litric` cannot reach the display. The
   fix is to make the socket reachable by the session's group, which is a
   session-model change and must be proven by a boot.
3. **It is not a security boundary of its own.** Its socket is `0600` inside a
   `0700` directory, so only the broker can speak to it, and the protocol
   cannot ask for a power transition, a password change or a network change.
   The unit's sandbox is deliberately thin (see the unit file for the
   item-by-item reasons): the isolation that matters for a GUI application is
   Flatpak's own `bubblewrap` sandbox, which a second generic filter here has
   not been measured against.

The portal gap remains: `xdg-desktop-portal` still has no backend, so a Flatpak
application can run but cannot open a file chooser (§2.3).

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
| Workspace tests | `cd src && cargo test --workspace --no-fail-fast` | 2,132 passed, 2 failed (fixture glibc; see §4.1) |
| Broker tests | `cargo test -p litric-sysd` | PASS, 369 passed across 6 targets |
| Broker → runtime daemon socket journey | `cargo test -p litric-sysd appd::tests` | PASS, 11/11 (see §4.3) |
| Busy answer under a running operation | `cargo test -p litric-sysd server::tests` | PASS, 5/5 |
| Browser engine e2e | `tools/browser-e2e.py --broker ... --chromium /usr/bin/chromium` | PASS, 12/12 stages (see §4.2) |
| Network config lint + self-test | `python3 ci/lint-network-config.py [--self-test]` | PASS |
| Unit reference lint | `python3 ci/lint-unit-binaries.py` | PASS (19 references: 10 resolvable, 9 skipped, 0 broken — `litric-appd` resolves) |
| mkosi schema lint | `./tools/gen-dev-keys.sh && python3 ci/lint-mkosi.py` | OK: 6 file(s) clean (a checkout with no generated `keys/dev/*` still reports the same 6 errors; `tools/build.sh` generates them itself before this lint) |
| Rust formatting | `cargo fmt --check` | **NOT RUN** — no `rustfmt` component in this toolchain; the mirror returns 404 for it |
| Image build | `tools/build.sh --profile desktop` | **NOT RUN** — no `mkosi`/Arch toolchain on this host |
| QEMU boot / link-up | `tools/qemu-guest-proof.sh` | **NOT RUN** — no `qemu-system-x86_64` on this host |
| Flatpak install of a real app | — | **NOT RUN** — `flatpak` is not installed on this host, and no image was built to run it in |

### 4.1 Workspace tests

`cargo test --workspace --no-fail-fast` on 2026-11-12: **2,132 passed, 2 failed**,
0 ignored. (An earlier run without `--no-fail-fast` reported 1,536 passed: cargo
stops scheduling test binaries after the first failing package, so that figure
was a prefix of the suite, not its size.) The two failures are the journeys that
exec the committed `weston-simple-shm` fixture
(`journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed` and
`journey_test_game_plays_fullscreen_and_stops`). The fixture's bundled
`libwayland-client.so.0` needs `GLIBC_2.38` and this host is `GLIBC 2.35`, so the
child exits before mapping a window. That is an environment mismatch, not a
regression, and it leaves the real-client path **not green on this host**.

One run also reported `terminal::tests::a_pty_session_stops_a_command_with_ctrl_c`
as failed under full parallel load; the test passes on its own (10 s) and in the
`--no-fail-fast` run above, so it is recorded as timing-sensitive rather than as
a defect.

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

### 4.3 The broker → runtime daemon journey

`src/litric-sysd/src/appd.rs` contains a test that starts a real listener, serves
it with an `Executor`, and drives it through a `Sysd` built with
`with_appd_socket`. It proves, over the socket rather than in-process:

* a catalog read returns the runtime daemon's `store.state=ready` and the parsed
  row (`org.gnome.Builder`);
* a `STORE INSTALL` that the broker authorises arrives at the runner as exactly
  `flatpak install --system --noninteractive --assumeyes flathub org.gnome.Builder`;
* a `LAUNCH` reply carries a `pid=` the runtime daemon produced, and the program
  really ran (a marker file appears);
* a configured-but-absent daemon returns `ERR appd-unavailable` instead of
  silently executing in the broker's namespace;
* a `RUN` target the resolver refuses comes back as `ERR bad-argument`;
* a second application operation while one is running gets `ERR busy` without
  waiting for the first, while a `PING` is still served.

A second test closes the loop over **both** sockets: a client sends
`STORE LIST` and `STORE INSTALL org.gnome.Builder flathub confirm` to the
broker's socket, the broker authorises, forwards over `/run/litric/appd.sock`,
and the runtime daemon's runner receives the install. That is the path the shell
takes (`shell.sysd_socket`), with the executor outside the broker's process.

None of it proves that `flatpak` itself installs an application: the runner is a
recorder, and no `flatpak` binary exists on this host. That remains the first
item of §5.

---

## 5. Release plan with acceptance criteria

Ordered by what unblocks the most user-visible truth. Each phase ends with a
measurement, not a sentence.

1. **`litric-appd` + Flatpak install/run (P0).** A real Flatpak application
   installed from Flathub and launched into the compositor, with a store journey
   that fails when the network is unavailable rather than pretending a cached
   catalog is a live one. Acceptance: guest proof with a real app id, a window,
   and its pixels; `cargo test` journey over the appd socket.
   *State:* the daemon, its unit, the closed protocol and the socket journey are
   **implemented** (§2.4, §4.3). The guest proof with a real app id and pixels is
   **NOT RUN** — it needs an image build and a boot. User separation (apps not
   running as root) is Phase 2; its two blockers are named in §3.2.
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
  the catalog reachable; the executor split removes the sandbox blocker (§2.4),
  but a booted install is still `NOT RUN`.
* It does not claim "100% compatibility" for any runtime, and it gives the
  reasons in §3.3.
* It does not replace the gate matrix. `docs/master-plan/25-...` remains the
  authoritative row-per-capability view; this document adds the measured state
  and the next architectural step.
