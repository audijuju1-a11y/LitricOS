# Wave 1 — network

**Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)).
**Facts implemented** in `src/litric-sysd/src/platform/net.rs`, fixture-tested on
any host, and since 2026-10-12 **four actions are implemented too** in
`src/litric-sysd/src/providers.rs`: bring a link up or down (`ip link set`), ask
`systemd-networkd` to re-read one interface (`networkctl reconfigure`), join a
named Wi-Fi network (write `iwd`'s credential file, then `iwctl station <iface>
connect`), and forget one (remove that file, then `networkctl reload`). Every
argument is validated before a plan exists. **Not implemented:** the networkd
drop-in writer, Wi-Fi scanning, and any DNS or proxy editing. §3 and §8 below say
exactly which of the surface is real.
**Scope:** wired networking, Wi-Fi as a *known-network* client, firewall state,
DNS resolution state. Not in scope: VPN, captive-portal detection, tethering.

## 1. What exists in the tree today

| Thing | Where | State |
|---|---|---|
| `iwd` (Wi-Fi supplicant), `nftables`, `systemd-resolvconf` | `image/mkosi.profiles/core.conf` | shipped, unmanaged |
| Wired configuration: static QEMU slirp addressing, `[Match] Name=en* eth*` | `image/mkosi.extra/usr/lib/systemd/network/20-wired.network` | shipped, works, documented reason for static over DHCP |
| Interface facts | `/sys/class/net/*`, `/proc/net/dev` | read by nobody |
| The shell's network indicator | `src/litric-shell/src/main.rs` (`read_network`) | reads a state string; it does not know what an interface is |
| A connection manager | — | **absent by decision**: `NetworkManager` is rejected in `docs/PERF-budget.md` row 6 |

## 2. The provider, and how it is reached

`systemd-networkd` is the manager; `iwd` owns Wi-Fi. ADR-0023 decision 2 says
Phase 1 reaches both through the interfaces that are already files:

- **Read:** interface list and state from `/sys/class/net/<if>/` (`operstate`,
  `address`, `mtu`, `carrier`, `speed`), counters from `/proc/net/dev`, IPv4
  routes from `/proc/net/route`, IPv6 addresses from `/proc/net/if_inet6`, DNS
  from `/run/systemd/resolve/resolv.conf` when `resolved` is running.
  IPv4 *addresses* are the one exception to "read a file": the kernel publishes
  them only through netlink, and a netlink client needs `unsafe`, which the
  daemon forbids. They come from `ip -o -4 addr show` with a fixed argument
  vector and no shell — `iproute2` is a hard dependency of `systemd` on this
  distribution, so the command is present in every profile including `nano`.
  When it is not, the reply says `net.addr.v4=none` rather than reporting an
  empty address list as if the machine had none.
- **Configure (wired):** write a drop-in to `/run/systemd/network/`, then
  `systemctl reload systemd-networkd`. A reload is a re-read of configuration that
  cannot lose an established link; a restart can, which is why the design says
  reload and the test has to assert it. **The drop-in writer is still not
  written** (§8 ticket 3); what is written is the link's administrative state,
  which the kernel owns, and the re-read of a configuration someone else
  installed.
- **Configure (Wi-Fi, known network only):** `iwd` reads credentials from
  `/var/lib/iwd/<ssid>.psk`; writing one joins that network. **Implemented**: the
  file is written `0600` under the daemon's root before `iwctl` is asked to
  connect, and the passphrase never reaches an argument vector — a test asserts
  that the reply and the argv contain neither it nor the SSID's path form. The
  file holds `Passphrase=` rather than a derived key: deriving one means
  implementing PBKDF2 to duplicate work `iwd` already does. **Scanning needs
  D-Bus and is Phase 2.** The Settings pane therefore shows known networks and a
  manual "join a network I can name" field, and says plain why there is no list.

**Two limits this implementation states rather than hides.** A network name whose
bytes are not printable ASCII, or which contains `/` or `\`, is **refused**: `iwd`
has its own escaping for the file name it stores a credential under, and a second
implementation of that escaping is how a credential ends up in a file the daemon
later looks for under a different name. A passphrase that is not 8 to 63
printable ASCII characters is refused for the same reason — a newline in one would
end the `Passphrase=` line and start an attribute nobody asked for. Both are
tested, and both are the kind of gap that is cheap to widen later and expensive to
get wrong now. IPv6 SSIDs and hex pre-shared keys are §8 work.

## 3. Data surface (facts)

New, all read-only, all parsed from text with fixture tests in
`litric-sysd::platform`:

```
net.iface       name=<name> state=up|down|unknown carrier=<0|1|-> mac=<hex|-> mtu=<n|-> rx=<bytes> tx=<bytes>
net.addr        <name> family=4|6 addr=<a> prefix=<n> scope=global|link|host|site|unknown
net.route       destination=<a/n> via=<gw|-> dev=<name> metric=<n>
net.dns         nameserver=<a>
net.dns         search=<domain>
net.dns.source  resolved|none
net.addr.v4     iproute2|none|unknown
net.iface.count <n>
```

Implemented as written, with three deliberate gaps stated rather than implied.
Each is a place the design was wider than the kernel's file surface:

| Gap | Why | What the reply says instead |
|---|---|---|
| no IPv6 routes | `/proc/net/ipv6_route` is a fixed-width hex dump whose field set differs between kernels, and a wrong parse yields plausible addresses rather than an error | no `net.route` line for an IPv6 route; nothing claims one |
| no IPv4 addresses without `iproute2` | netlink needs `unsafe` (see §2) | `net.addr.v4=none` |
| `carrier` can be absent | some drivers publish no `carrier` file, and some publish it as a permission-denied for an unprivileged reader | `carrier=-`, never `carrier=0` |

An absent fact is written `-` and never `0`. A `0` that is indistinguishable
from a measured zero is how a facts reader starts lying, and it is the one rule
every parser in `platform/` is shaped around.

Everything above is a machine fact. Nothing in this section requires privilege.

## 4. Action surface (capabilities)

| Action | Capability | Why that class | State |
|---|---|---|---|
| Write a networkd drop-in | `NetConfigure` (new, `SystemSettings`-adjacent) | it changes how the machine reaches the network for every process | **not implemented** |
| Write an `iwd` credential file | `NetConfigure` @ `wifi://<ssid>` | scope-limited: one network, not "the network" | **implemented** (`NET WIFI JOIN`, `NET WIFI FORGET`) |
| Bring an interface up/down | `NetConfigure` @ `iface://<name>` | reversible, per-interface | **implemented** (`NET LINK <iface> UP\|DOWN`) |
| Re-read configuration for one interface | `NetConfigure` @ `iface://<name>` | reversible, and it changes nothing that was not already written | **implemented** (`NET RECONFIGURE <iface>`) |
| Read facts | none | reading what the machine is doing is not a capability | implemented |

None of the four is destructive in the vocabulary's sense (`Scope::is_destructive`
is about `power://system/*`), so none needs live consent: a link, a re-read and a
credential are all reversible, and a dialog in front of a Settings toggle is a
worse interface for the same safety. The shell holds `iface://` and `wifi://`
as two grants of one capability, and every individual request is still checked
and audited.

The shell holds these by grant like any other application
(`docs/SEC-01-capabilities.md`); the broker is what decides, and every write is an
audit entry.

## 5. Threat model

| # | Abuse case | Mitigation in this design |
|---|---|---|
| N1 | A non-privileged app rewrites the network configuration and reroutes traffic through itself (a proxy/MITM) | `NetConfigure` is a capability no unprivileged app holds; the drop-in writer refuses a path outside `/run/systemd/network/` and refuses a name that is not an interface it can list |
| N2 | The drop-in writer is given a name containing `../` and writes outside `/run/` | the writer validates the interface name against `/sys/class/net` before it composes a path; a name that is not a current interface is a refusal, not an escape |
| N3 | A malicious credential file (symlink, world-writable, oversized) is planted for `iwd` to read | the writer creates files with `0600`, `O_NOFOLLOW` semantics, and a size ceiling; `/var/lib/iwd` is the only directory it will write into |
| N4 | The facts reader is pointed at an attacker-controlled root (`read_facts(root)`) and reports fiction as machine state | that parameter exists for tests; the daemon passes `/` and the API documents that a caller-supplied root is a test seam, never a runtime option. A fact line that came from a different root than `/` is not served |
| N5 | DNS is silently redirected by a written `resolv.conf` | the writer never writes `/etc/resolv.conf` (read-only root by design, [ADR-0017](../adr/0017-framebuffer-shell.md)); DNS stays `resolved`'s business |
| N6 | A firewall rule added by an app opens a port and never closes it | nftables rules are written by the system only; apps have no firewall capability, and `NetServer` is what announces an inbound listener, not what opens the port |

## 6. What is verifiable today

- The parsers, against fixture text, on any host — `cargo test -p litric-sysd`.
  **Implemented**: `platform::net` reads a whole fixture machine from a directory
  and asserts the interface, address, route and DNS lines it produces.
- That the image ships the providers it claims: the profile package list.
- The reload-not-restart rule as a design assertion only, until a booted image
  can be driven; it is listed as NOT RUN below.

## 7. NOT verified, and what it would take

| Claim | Needs |
|---|---|
| the image brings up the wired link | a booted image + QEMU slirp (the configuration is written for exactly that bus) — **done 2026-10-11**: `net.iface enp0s2 state=up carrier=1` with the slirp address |
| a reload applies a new drop-in without dropping the link | a booted image, a second interface state to observe (the writer itself is not written yet either) |
| writing an `iwd` credential file joins a real network | a wireless device (`iwd` cannot be exercised in QEMU's default NIC set). **NOT verified**, and the booted image cannot make it more true — it has no radio |
| the link action reaches the kernel in a booted image | a booted image — **done 2026-10-12**: the in-guest self test runs `NET LINK lo up` and records `litric.sysd.net_link_lo=ok` |
| scanning is impossible without D-Bus | nothing: it is a property of `iwd`'s API, recorded as the reason for the Phase 2 decision |

## 8. Tickets this note implies

1. ~~`litric-sysd::platform`: interface/address/route facts + fixture tests.~~
   **Done** — `src/litric-sysd/src/platform/net.rs`.
2. ~~Capability `NetConfigure` with scope grammar (`wifi://`, `iface://`) and tests.~~
   **Done** — `litric-caps`, with `Scope::parse_resource`.
3. The drop-in writer (path validation, `0600`, reload), with a refusal test for a
   name that is not an interface. **Not done**: this is the first ticket that
   writes to `/run`, and it needs the capability broker to be the writer's
   caller rather than the shell.
4. Settings pane: wired state, known Wi-Fi networks, "join a named network".
   **Not done** — the facts are served and the actions exist, the pane is not
   drawn (`src/litric-shell/src/views.rs` line ~1995 still says Network has no
   page). This is now the largest gap in this note: everything behind the pane is
   real as of 2026-10-12, and a person still cannot reach it.
5. Non-ASCII and escaped network names, and a hex pre-shared key instead of a
   passphrase. **Not done** — refused rather than guessed at, see §2.
6. Phase 2 only: the D-Bus decision from ADR-0023, then scanning, then a network
   editor that can express more than one network.
