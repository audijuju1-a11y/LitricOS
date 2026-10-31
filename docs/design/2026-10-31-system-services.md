# System services: AppArmor, Polkit, firewalld, Flatpak and NetworkManager

**Date:** 2026-10-31 (S88)
**Status:** implemented; image integration is build-time, not boot-verified here.
**References:** KDE System Settings and KDE Discover were used as interaction
references, not as dependencies. The implementation follows ADR-0023's
provider boundary: the shell asks `litric-sysd`, and the daemon uses the
subsystem's own files and base tools.

## The gap this closes

The Settings window could report CPU, memory, networking, Bluetooth, audio,
power, updates and accounts, but had no surface for the security and
application-management services the operating system relies on:

| Surface | Before S88 |
|---|---|
| AppArmor | Not enabled in the kernel; no package, no status reader |
| Polkit | Not installed; no reader |
| firewalld | Not installed; no reader |
| App Store / Flatpak | Flatpak was installed, but the Settings window had no catalogue state |
| NetworkManager | Not installed; the shell uses `systemd-networkd` + `iwd` |
| App sandbox | bubblewrap was present, but no surface stated whether it was usable |

The request was not for decorative rows. The result is five real fact
paragraphs, four bounded reload actions, a scoped capability, and an image
that actually enables the security stack.

## Decision

### 1. The daemon owns the read model

`src/litric-sysd/src/services.rs` reads:

| Service | Files and tools | Reported states |
|---|---|---|
| AppArmor | `/sys/module/apparmor/parameters/enabled`, `/sys/kernel/security/apparmor/profiles`, `apparmor_parser`, `aa-status` | `enforce`, `complain`, `mixed`, `enabled-no-profiles`, `disabled`, `unknown` |
| Polkit | `polkitd`, `pkcheck`, `*.policy` action files | `ready`, `partial`, `absent` |
| firewalld | `firewall-cmd`, `/etc/firewalld`, `/run/firewalld`, zone XML | `running`, `configured`, `installed`, `config-only`, `absent` |
| Flatpak | `flatpak`, `bwrap`/`flatpak-bwrap`, `/var/lib/flatpak/app`, repo config | `ready`, `no-sandbox`, `client-missing`, `absent` |
| NetworkManager | `nmcli`, `/etc/NetworkManager`, `/run/NetworkManager`, connection profiles | `running`, `configured`, `installed`, `absent` |

The reader never invents a zero. A missing service line leaves the shell's
field absent; a reported `zones=0` stays a real zero. AppArmor's profiles are
counted from securityfs, and a profile in neither enforce nor complain still
counts as loaded.

The facts travel as stable `svc.*` lines in the existing `FACTS` reply. There
is no D-Bus client in the shell or in the daemon, and no new dependency.

### 2. Actions are closed plans, not shell commands

`ServiceAction` has exactly four variants:

| Action | Plan | Deadline |
|---|---|---|
| `apparmor.reload` | `systemctl try-reload-or-restart apparmor` | 30 s |
| `firewall.reload` | `firewall-cmd --reload` | 30 s |
| `store.refresh` | `flatpak update --appstream -y` | 60 s |
| `networkmanager.reload` | `nmcli general reload` | 30 s |

Every plan is a constant argument vector. There is no request-supplied
argument, no shell, and no path interpolation. The provider contract records
the action in tests and runs it through `Runner` in production.

### 3. Permission is scoped and consented

The four actions use the new `service.control` capability, not the unscoped
`system.settings` capability. Each grant names one scope:

```text
service://apparmor
service://firewall
service://store
service://networkmanager
```

The local session's policy grants those four scopes. A click is a live user
action, so the shell sends `CONFIRM`; the daemon still checks the capability,
the scope, the plan and its deadline. Reloading security or network policy is
an administrative act and remains `Risk::High`.

### 4. Settings draws only what the machine can do

The new Settings page is **Services**, last in the sidebar. It shows one row
for each subsystem and draws a reload button only when the corresponding state
is actionable:

* AppArmor: kernel enabled, parser present, status client present.
* firewalld: `firewall-cmd` present and the runtime directory says it is running.
* Flatpak: client present.
* NetworkManager: `nmcli` present and the runtime directory says it is running.

If no control is actionable the page says so. If a reload is refused, the
daemon's own `ERR` line is shown verbatim in the page and in a notification.

## Image integration

* `kernel/config/litric-base.config` enables `CONFIG_SECURITY_APPARMOR` and
  adds `apparmor` to `CONFIG_LSM`.
* `image/mkosi.conf` adds `apparmor=1` and the same LSM order to the kernel
  command line. `ci/lint-kconfig.py` enforces that the two lists agree.
* The desktop profile installs `apparmor`, `polkit`, `firewalld` and
  `networkmanager`.
* `mkosi.postinst.chroot` enables `apparmor.service` and `firewalld.service`.
  Polkit is D-Bus-activated. NetworkManager is deliberately not enabled.

## Limits

* LitricOS's native network stack remains `systemd-networkd` + `iwd`.
  Installing NetworkManager does not start a second manager on the same links.
  The page reports the native stack when NetworkManager is absent or stopped.
* NetworkManager Wi-Fi joining still needs a secret-agent design. This batch
  adds status and a configuration reload, not a second Wi-Fi client.
* The Flatpak count is the system installation under `/var/lib/flatpak`; user
  installations under a home directory are not included.
* AppArmor profiles are the distribution package's profiles. LitricOS does not
  yet ship a reviewed profile set of its own.
* The image changes are source/build changes. No `mkosi` image was built and no
  QEMU boot was run on this host. The daemon readers and plans are unit-tested
  against fixtures; the Settings page is fixture-rendered.
* Real-card C2 KMS modeset/page-flip claims remain **NOT MADE**.
