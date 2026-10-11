# Platform parity — Windows / macOS / Linux backends, and what LitricOS should do

**Created:** 2026-10-11 (Stage B of the Beta 1.0 E2E pass)
**Method:** primary/official sources only. Every row below says whether it was
**verified in a fetched page** or **not verified**. Where a source could not be
read, the row says so instead of asserting the behaviour.

> This document is a *design input*, not an implementation. It records what
> Windows/macOS users expect, and — separately — which Linux mechanism LitricOS
> would have to talk to. It does not claim that LitricOS has any of it: see
> [SYSTEM-SCOPE.md](../SYSTEM-SCOPE.md) for the honest per-row status.

---

## 1. Verification boundary (read this before quoting anything below)

| Source family | Status in this pass |
|---|---|
| NetworkManager D-Bus API reference (`networkmanager.dev/docs/api/latest`) | Read; pages show no publication date, only the GTK-Doc generator version (1.34.0) |
| BlueZ API reference (`bluez.readthedocs.io`) | Read; no release version or date shown on the pages |
| systemd / Linux-PAM / shadow man pages | Read via the Debian man-page mirror, because the freedesktop- and kernel.org-hosted pages did not serve readable bodies. Version labels recorded per row |
| polkit reference manual (`polkit.pages.freedesktop.org`) | Read; the manual identifies itself as **124** while the project release page shows **127** — a real docs/release gap; check the shipped version |
| UPower (`upower.freedesktop.org/docs`) | Read; reference manual **1.91.2** |
| PipeWire / WirePlumber (`pipewire.pages.freedesktop.org`) | Read; PipeWire **1.6.9**, WirePlumber **0.5.18** |
| Microsoft Learn / Windows support | Read; several pages show a "last updated" date, recorded per row |
| Apple support guides and HIG | **Only partly readable.** Page bodies for the HIG (dark mode, accessibility, motion, keyboards, focus, notifications) and several Mac guides returned metadata only. Every Apple claim below that depends on those pages is marked **NOT VERIFIED** |
| W3C/WAI | Read; WCAG **2.2**, W3C Recommendation dated **2024-12-12** |

**No access date can be asserted for the vendor documentation pages** — the
fetch record carried no calendar date and the pages do not show one. Rows cite
the document version/date where the page displayed one. Do not present this
document as a dated snapshot.

---

## 2. Capability status vocabulary (one model, used everywhere)

The prompt set and the repository use different words for the same ladder. They
are mapped here once so that no future document invents a third vocabulary, and
so that a status can never be upgraded by rewording:

| Prompt-set vocabulary | Repository vocabulary (`SYSTEM-SCOPE.md`) | Evidence needed to use it |
|---|---|---|
| `WORKING_AND_VERIFIED` | `D` | named test **plus** runtime evidence at the relevant level |
| `IMPLEMENTED_NOT_RUNTIME_VERIFIED` | `P` | real code with a passing test; behaviour not observed on target |
| `PARTIAL` / `UI_ONLY` / `BACKEND_ONLY` | `P` | same; the split must be stated in the row |
| `INHERITED_FROM_BASE_OS` | `U` / `O+U` | the upstream package/service is named **and** integration is tested |
| `HARDWARE_DEPENDENT` | n/a | device identified; separate hardware evidence |
| `NOT_IMPLEMENTED` | `T` | — |
| `DEFERRED` | `S` or `T` with a stated reason | — |
| `BLOCKED` | n/a | the missing prerequisite is named |

The five UI states a control may show must be exactly:
`Available` · `Changing…` · `Enabled/Connected` · `Permission required` ·
`Unsupported` / `Unavailable` / `Failed` / `Unknown` — with the last group
distinguished in text, never by colour alone.

---

## 3. Network — Wi-Fi and Ethernet

| Aspect | Windows / macOS expectation | Linux mechanism (verified) | LitricOS target | Verified? |
|---|---|---|---|---|
| Radio on/off | Taskbar Quick Settings toggle; state is authoritative | NetworkManager property `org.freedesktop.NetworkManager.WirelessEnabled` (r/w, **on the Manager**, not the device); `WirelessHardwareEnabled` (r/o) reports an rfkill block; `Enable()` toggles *all* networking, not just Wi-Fi | One toggle, three distinguishable causes: hardware block, software off, daemon absent | Yes — [Manager API](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html) |
| Scan | Windows opens a list that repopulates | `RequestScan(a{sv})` (option `ssids`), completion observed via `PropertiesChanged` on `LastScan` (`x`, `CLOCK_BOOTTIME` ms, `-1` = never). `GetAccessPoints` is **deprecated** and hides unknown hidden SSIDs; use `GetAllAccessPoints`. Signals `AccessPointAdded`/`Removed` | Loading state driven by `LastScan`, not by a timer | Yes — [Device.Wireless](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Device.Wireless.html) |
| Signal & security | Bars and a lock glyph | `Strength` (`y`) is **already a percentage**; `Flags` (`u`) `PRIVACY=0x1`; `WpaFlags`/`RsnFlags` bitmasks including `KEY_MGMT_PSK=0x100`, `KEY_MGMT_802_1X=0x200`, `KEY_MGMT_SAE=0x400`, `KEY_MGMT_OWE=0x800` | Decode flags; never guess security from the SSID | Yes — [AccessPoint](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.AccessPoint.html), [types](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html) |
| Connect | One click, password prompt when needed | Manager methods `ActivateConnection(o connection, o device, o specific_object)`, `AddAndActivateConnection(a{sa{sv}}, o, o)`, and `AddAndActivateConnection2(...a{sv} options...)` where `persist` defaults to `"disk"` (`"memory"` / `"volatile"` also exist) | Use `AddAndActivateConnection2` so persistence is an explicit decision, not a side effect | Yes — [Manager API](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html) |
| Persistence of profiles | Saved networks list, forget button | `AddConnection` persists; `AddConnectionUnsaved` and `AddConnection2` (`to-disk`/`in-memory`) make it explicit. Profile persistence is the default and is **not** the same thing as secret storage | "Forget" must delete the profile, not just disconnect | Yes — [Settings](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.Settings.html) |
| State model | Connected / no internet / limited | Three separate enums: `NMState` (`DISCONNECTED=20`, `CONNECTING=40`, `CONNECTED_LOCAL=50`, `CONNECTED_SITE=60`, `CONNECTED_GLOBAL=70`; `ASLEEP=10` is deprecated in favour of `DISABLED=10` since 1.56), `NMDeviceState` (`NEED_AUTH=60`, `IP_CONFIG=70`, `ACTIVATED=100`, `FAILED=120`), `NMActiveConnectionState` (`ACTIVATING=1`, `ACTIVATED=2`, …) | The UI must not collapse these: "connected to Wi-Fi" ≠ "internet reachable" | Yes — [types](https://networkmanager.dev/docs/api/latest/nm-dbus-types.html) |
| Wrong password | Retry prompt | **There is no stable D-Bus error name meaning "wrong password."** Failure arrives as a state reason (`NO_SECRETS=7`, `SUPPLICANT_TIMEOUT=11`, `IP_CONFIG_UNAVAILABLE=5`) or as a secret-agent retry hint (`REQUEST_NEW = 0x2`) | Model *stages*, not an error string; never show "wrong password" unless the backend actually said so | Yes — [SecretAgent](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.SecretAgent.html), [libnm errors](https://networkmanager.dev/docs/libnm/latest/libnm-nm-errors.html) |
| Secrets | Password stored by the OS | Secret agent registers at `/org/freedesktop/NetworkManager/AgentManager` (`Register` / `RegisterWithCapabilities`) and implements `org.freedesktop.NetworkManager.SecretAgent` (`GetSecrets`, `CancelGetSecrets`, `SaveSecrets`, `DeleteSecrets`). NetworkManager will not call `SaveSecrets` back to the agent that supplied the secret | LitricOS must implement an agent or the password cannot be retained at all; a LitricOS-side config file is not an option | Yes — [AgentManager](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.AgentManager.html) |
| Authorization | A UAC-style prompt appears | polkit actions exist: `org.freedesktop.NetworkManager.enable-disable-wifi`, `...enable-disable-network`, `...wifi.scan`, `...network-control`, `...settings.modify.own`, `...settings.modify.system`. `GetPermissions()` answers `yes`/`auth`/`no` | Query `GetPermissions()`; do not hard-code an action-id map — the docs give none | Yes (action ids) — [Manager API](https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html), [example](https://networkmanager.dev/docs/api/latest/nmcli-examples.html) |

**Not yet verified:** whether the `nano`/`core`/`desktop` image profiles ship
NetworkManager at all. `image/mkosi.profiles/*.conf` was not parsed for it in
this pass — `UNKNOWN`, and it is the first thing a network ticket must establish.

---

## 4. Bluetooth

| Aspect | Windows / macOS expectation | Linux mechanism (verified) | LitricOS target | Verified? |
|---|---|---|---|---|
| Radio | A toggle in Quick Settings | `org.bluez.Adapter1.Powered` (r/w). The docs state the value is **not persistent** across restart or unplug | Read state after every write; never cache it as truth | Yes — [Adapter API](https://bluez.readthedocs.io/en/latest/adapter-api/) |
| Discoverability | Only while the settings pane is open | `Discoverable` (r/w) — the docs say it should only be used by the **Settings application**; `Pairable` (r/w) governs incoming pairing | Good: LitricOS Settings is that application | Yes |
| Scan | A list that fills in | `StartDiscovery()` / `StopDiscovery()`; discovery is shared between clients and `Discovering` is read-only. Device objects appear as `org.bluez.Device1` | Stop discovery when the pane closes — the client count is shared, so leaking a session blocks other users of the bus | Yes |
| Device list updates | Live | `GetManagedObjects()` to seed, `InterfacesAdded`/`InterfacesRemoved` while scanning. The Adapter page confirms device objects are created by discovery but **does not itself document the ObjectManager signal contract** — that lives in the D-Bus/ObjectManager spec | Subscribe; do not poll | Partly — the signal contract is **not** sourced from BlueZ docs here |
| Pair vs connect | "Paired" and "connected" are visibly different | `org.bluez.Device1.Pair()`, `Connect()`, `Disconnect()`; read-only `Paired` and read-only `Connected`; `Trusted` (r/w); `Adapter1.RemoveDevice()` removes the device **and its pairing information** | Two distinct indicators; "Forget" = `RemoveDevice` | Yes — [Device API](https://bluez.readthedocs.io/en/latest/device-api/) |
| Passkey / confirmation | "Compare this code on both devices" | Agent model: `AgentManager1.RegisterAgent(o agent, s capability)` with capabilities `DisplayOnly`, `DisplayYesNo`, `KeyboardOnly`, `NoInputNoOutput`, `KeyboardDisplay`. `Agent1` callbacks: `RequestPinCode`, `RequestPasskey`, `DisplayPasskey`, `RequestConfirmation` (empty reply accepts, error rejects), `DisplayPinCode`, `RequestAuthorization`, `AuthorizeService`, and `Cancel()` for a withdrawn request | Modal dialog wired to a registered agent; `Cancel()` must dismiss it | Yes — [Agent API](https://bluez.readthedocs.io/en/latest/agent-api/) |
| Authorization | n/a | **No canonical `org.bluez.*` polkit action id could be verified.** Upstream ships a D-Bus policy file that grants the default context access; distributions vary. Pairing authorization is mediated by the Agent API | Do not assume NetworkManager-style polkit ids exist for BlueZ; inspect the target host's policy | **NOT VERIFIED** — [upstream `bluetooth.conf`](https://github.com/bluez/bluez/blob/master/src/bluetooth.conf) |

---

## 5. Session, login, lock and power

| Aspect | Windows / macOS expectation | Linux mechanism (verified) | LitricOS target | Verified? |
|---|---|---|---|---|
| Who owns "the desktop is locked" | The OS | **logind does not detect a locked screen.** The session object has `LockedHint` (read) and `CanLock`; the desktop calls `SetLockedHint()`. Lock *requests* arrive as the session object's `Lock()` / `Unlock()` signals, and the session manager is expected to listen and activate its own lock screen | LitricOS must own the lock screen and report truthfully to logind; logind is the request channel, not the implementation | Yes — [systemd desktop guidance](https://systemd.io/WRITING_DESKTOP_ENVIRONMENTS/), [org.freedesktop.login1(5)](https://manpages.debian.org/unstable/systemd/org.freedesktop.login1.5.en.html) |
| Power actions | Menu with sleep/restart/shut down | `CanSuspend/CanHibernate/CanReboot/CanPowerOff` (return `na`/`yes`/`no`/`challenge`/`inhibited`/`inhibitor-blocked`) and `Suspend/Hibernate/Reboot/PowerOff(interactive: b)`. The docs recommend logind as *the* mechanism so policy and inhibitors are honoured | Use logind's methods and show its `Can*` answer; do not implement a private policy | Yes |
| Asking for permission | UAC / Touch ID | polkit action ids **are context-dependent**: `org.freedesktop.login1.power-off` vs `…-multiple-sessions` vs `…-ignore-inhibit` (same pattern for reboot/suspend/hibernate). `lock-sessions` exists, but the D-Bus page maps **no** lock method to a specific check | Do not infer an authorization contract for lock calls from the existence of a policy action | Yes — [policy file](https://github.com/systemd/systemd/raw/refs/heads/main/src/login/org.freedesktop.login1.policy) |
| Sleep preparation | Apps finish work before sleeping | Subscribe to the Manager's `PrepareForSleep(start)` signal. `true` = preparing, `false` = sleep attempt over/awake. **Taking a delay inhibitor first is required**, or the check races | LitricOS must take an inhibitor before promising "locked before sleep" | Yes — [Inhibitor Locks](https://systemd.io/INHIBITOR_LOCKS/) |
| Recent behaviour change | — | systemd **257** changed `block` inhibitors to apply to privileged requests too; the old behaviour is `block-weak` | Any power UX depends on the shipped systemd version | Yes |
| Password verification | OS dialog | If a client links PAM: `pam_start` → `pam_authenticate` → `pam_acct_mgmt` → `pam_setcred` → `pam_open_session`/`pam_close_session` → `pam_end`; one handle per thread. **There is no standard PAM IPC protocol**, so a UI that must not link PAM uses a small constrained helper that runs the configured stack rather than hard-coding a module | A helper, running `/etc/pam.d`, not a hand-written verifier | Yes — [pam(3)](https://manpages.debian.org/unstable/libpam0g-dev/pam.3.en.html) |
| Password change | System dialog | `pam_chauthtok()` through the password stack; modules act on `PAM_UPDATE_AUTHTOK`. Hash method comes from `ENCRYPT_METHOD` in `/etc/login.defs`; `pam_unix` documents yescrypt support | Never write our own hash; never read `/etc/shadow` from the desktop | Yes — [pam_sm_chauthtok(3)](https://manpages.debian.org/unstable/libpam0g-dev/pam_sm_chauthtok.3.en.html), [pam_unix(8)](https://manpages.debian.org/unstable/libpam-modules/pam_unix.8.en.html) |
| Hashing rules | — | The PAM pages do **not** state a blanket "never write your own hashing" rule. That guidance is OWASP's, not PAM's: purpose-built slow hashes, per-password salt, no fast general-purpose hash for storage | Record the rule as an engineering decision with the OWASP citation, not as a PAM requirement | Corrected — [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) |
| Account-existence disclosure | Generic failure text | PAM distinguishes `PAM_USER_UNKNOWN` from `PAM_AUTH_ERR`, but the fetched pages **do not prescribe UI wording**. Showing one public failure message for both is a LitricOS policy choice, not a documented requirement | Policy: identical public message; detail only in protected diagnostics | Policy, sourced as policy |
| "Logged in" vs "booted" | — | Reaching `graphical.target` is **not** proof a user authenticated. `display-manager.service` precedes it. `systemd-analyze` timings measure services being **spawned**, not initialised, and `boot-complete.target` is an opt-in synchronisation point that units must pull in | Boot-to-desktop needs its own instrumented milestones; `systemd-analyze` alone cannot define them | Yes — [systemd.special(7)](https://manpages.debian.org/unstable/systemd/systemd.special.7.en.html), [systemd-analyze(1)](https://manpages.debian.org/unstable/systemd/systemd-analyze.1.en.html), [sd_notify(3)](https://manpages.debian.org/testing/libsystemd-dev/sd_notify.3.en.html) |
| Authorization framework | UAC / Touch ID | polkit: system bus interface `org.freedesktop.PolicyKit1.Authority`, method `CheckAuthorization(subject, action_id, details, flags, cancellation_id)`; `AllowUserInteraction` only when a user action triggered the check. The JS rule engine is **not** a competing client API. Unprivileged programs normally do not call polkit directly — the privileged mechanism does | The shell talks to a mechanism; there is no one-to-one operation↔action mapping to rely on | Yes — [Authority API](https://polkit.pages.freedesktop.org/polkit/eggdbus-interface-org.freedesktop.PolicyKit1.Authority.html), [writing polkit apps](https://polkit.pages.freedesktop.org/polkit/polkit-apps.html) |

---

## 6. Audio and power state

| Aspect | Windows / macOS expectation | Linux mechanism (verified) | LitricOS target | Verified? |
|---|---|---|---|---|
| Default output/volume/mute | Per-app mixer, device picker | PipeWire provides the media graph; **a session manager (WirePlumber) owns device monitoring and default-node selection**. WirePlumber 0.5.18 documents `wpctl` (`set-default`, `get-volume`, `set-volume`, `set-mute`, `@DEFAULT_SINK@`, `@DEFAULT_SOURCE@`) | Use the WirePlumber C API (https://pipewire.pages.freedesktop.org/wireplumber/library/c_api.html) or the PulseAudio-compatible protocol for the panel; `wpctl` is for diagnostics, and parsing CLI output is explicitly not a stable API | Yes — [PipeWire overview 1.6.9](https://pipewire.pages.freedesktop.org/pipewire/page_overview.html), [WirePlumber 0.5.18](https://pipewire.pages.freedesktop.org/wireplumber/) |
| Battery percentage | Shown with time estimate | UPower `org.freedesktop.UPower.Device.Percentage` is a double 0–100, and the docs warn it is **an approximation when coarse reporting is in use** — prefer `BatteryLevel` when present, and check `PowerSupply` before treating a device as the system battery | Never invent a percentage; show unknown when absent | Yes — [UPower Device](https://upower.freedesktop.org/docs/Device.html) |
| Power profiles | Balanced / Performance slider | The reference page documents interface **`org.freedesktop.UPower.PowerProfiles`** at `/org/freedesktop/UPower/PowerProfiles` with `ActiveProfile`, `Profiles`, `Actions`, `PerformanceDegraded`, `ActiveProfileHolds`, `HoldProfile()`/`ReleaseProfile()`; `PerformanceInhibited` is deprecated since 0.9. This is **not** the `net.hadess.PowerProfiles` name in common use | Introspect the bus name at runtime; do not hard-code either name | **Namespace conflict recorded, not resolved** — [Power Profiles reference](https://freedesktop-team.pages.debian.net/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html) |

---

## 7. UX benchmark: Settings, quick controls, accessibility

| Question | Verified finding | Source |
|---|---|---|
| Windows 11 Settings navigation | Labelled navigation panel; browse by category or search by keyword; a category opens grouped settings with subpages. On Copilot+ PCs only, Microsoft documents intent-aware search and a Settings agent, and says the user must select **Apply** for suggested changes | [Exploring Windows Settings](https://support.microsoft.com/en-us/windows/experience/exploring-windows-settings) |
| Windows 11 top-level categories | Home; System; Bluetooth & devices; Network & internet; Personalization; Apps; Accounts; Time & Language; Gaming; Accessibility; Privacy & security; Windows Update — as listed in Microsoft's current article, not an immutable taxonomy | same |
| "Unsupported" presentation | **No universal pattern is documented.** Microsoft's Location example shows cause-sensitive behaviour: a device control is hidden from non-admins, and a user-level control cannot be enabled while the device service is off | [Windows location service and privacy](https://support.microsoft.com/en-us/windows/privacy/windows-location-service-and-privacy) |
| Risky display change | The "Keep these display settings?" keep-or-revert prompt is real: choosing **Revert** restores the previous resolution and doing nothing also reverts. The cited article is a troubleshooting scenario and does **not** establish that every mode change uses it, nor the timeout | [Wallpaper/resolution support article](https://support.microsoft.com/en-us/windows/hardware/display-graphics/the-wallpaper-might-not-correctly-fit-the-screen-after-connecting-an-external-display-in-windows) |
| Colour and accent | Windows guidance: light and dark modes with contrast-adjusted neutrals; accent emphasises important UI and indicates state; use it sparingly. Explicit contrast rule: *"Make sure that elements and images have sufficient contrast to differentiate between them, regardless of the accent color or theme."* Fluent 2: *"Don't use color as the only way to communicate."* Neither page gives a numeric ratio | [Color in Windows](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/color) (last updated 2026-07-14), [Fluent 2 Color](https://fluent2.microsoft.design/color) |
| Keyboard and focus | Microsoft calls keyboard accessibility a **primary interaction model**, not a fallback; logical, predictable tab order; pointer-activatable UI must also be keyboard-activatable; custom focusable controls need a visible focus indicator; disabled controls leave the tab order | [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) (last updated 2026-10-04) |
| Interaction-state direction | Fluent 2 notes colours generally darken toward hover/selected, but **Windows reverses that direction**; on focus the control colour does not change and the container gets a thicker stroke | [Fluent 2 Color](https://fluent2.microsoft.design/color) |
| Motion | Microsoft describes motion as reactive, direct and context-appropriate, brief and consistent. **A requirement for how apps must honour a reduced-motion preference could not be verified** in the fetched guidance | [Motion in Windows](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/motion) (last updated 2026-02-28) |
| macOS System Settings rename | Apple's Ventura announcement: *"System Settings is the new name for System Preferences"*, with a streamlined design familiar to iPhone/iPad users. It does **not** document the category regrouping | [Ventura announcement](https://www.apple.com/newsroom/2022/06/macos-ventura-brings-powerful-productivity-tools-new-continuity-features-to-mac/) (2022-06-06) |
| macOS Lock Screen settings | Documented path: Apple menu ▸ System Settings ▸ **Lock Screen** in the sidebar; controls include display-off timing on battery vs adapter, when a password is required after sleep/screensaver, login-window contents, and a lock message | [Change Lock Screen settings on Mac](https://support.apple.com/guide/mac-help/change-lock-screen-settings-on-mac-mh11784/mac) |
| macOS wallpaper/Login Items/display revert, Control Center, notification privacy, HIG dark mode/accessibility/motion/details | **NOT VERIFIED** — the relevant page bodies were not readable in this pass. Do not use these as established facts without opening the pages | locators only: [System Settings](https://support.apple.com/guide/mac-help/change-system-settings-mh15217/mac), [Login Items](https://support.apple.com/guide/mac-help/change-login-items-extensions-settings-mtusr003/mac), [Wallpaper](https://support.apple.com/guide/mac-help/choose-your-desktop-picture-mchlp3013/mac), [resolution](https://support.apple.com/guide/mac-help/change-your-mac-displays-resolution-mchl86d72b76/mac), [Control Center](https://support.apple.com/guide/mac-help/quickly-change-settings-with-control-center-mchlc9d0e1f2/mac), [Notifications](https://support.apple.com/guide/mac-help/change-notifications-settings-on-mac-mchl205da693/mac), [HIG](https://developer.apple.com/design/human-interface-guidelines/) |
| Windows Quick Settings / notifications | Quick Settings lives in the taskbar area with network/volume/battery, opens by click, **Windows+A**, or swipe, and is rearrangeable. Notifications: banners plus a centre opened from the clock/bell with **Windows+N**; expand, act, dismiss, clear; per-app settings and priority; **hide content on the lock screen**; Do Not Disturb silences or routes most notifications to the centre | [Customize the taskbar](https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-taskbar-in-windows), [Notifications and Do Not Disturb](https://support.microsoft.com/en-us/windows/experience/notifications-and-do-not-disturb-in-windows) |
| Accessibility conformance numbers to design against | **WCAG 2.2**, W3C Recommendation **2024-12-12**: **SC 1.4.3** AA text contrast ≥ **4.5:1** (≥ 3:1 large text); **SC 1.4.11** AA non-text/component-state contrast ≥ **3:1**; **SC 1.4.1** A colour must not be the **only** means of conveying information; **SC 2.4.7** AA keyboard focus indicator visible; **SC 2.5.8** AA pointer targets ≥ **24×24** CSS px with spacing exceptions. WCAG governs web content on devices including desktops — it is a benchmark for a native shell, not a conformance claim about it | [WCAG 2.2](https://www.w3.org/TR/WCAG22/), [1.4.3](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html), [1.4.11](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html), [1.4.1](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html), [2.4.7](https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html), [2.5.8](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) |

The repository already measures the WCAG text rule on real compositions: the
glass surfaces settle at 18% tint with a worst-case text contrast of **5.19:1**
against the 4.5 requirement (ADR-0017 §4). That is the existing local evidence a
Settings UI must not regress.

---

## 8. Decision log — where Windows and macOS differ, pick one LitricOS policy

| Behaviour | Windows | macOS | LitricOS decision | Basis |
|---|---|---|---|---|
| Where quick toggles live | Quick Settings (Windows+A) | Control Center in the menu bar | **One Control Center**, opened by a system-bar control and one keyboard shortcut | Prompt set §6.3 |
| Display change safety | keep-or-revert prompt | not verified | **Implement keep-or-revert with a visible countdown** for any change that could blank the display; revert on timeout. Only once a backend exists | Windows pattern verified; Apple unverified, so no claim of parity |
| Lock-screen content | content can be hidden per app | not verified | **Not implemented, and not to be simulated.** No authentication subsystem exists; policy is to hide everything until PAM-backed auth exists | Stage A §5; `SYSTEM-SCOPE.md` B6 |
| Accent usage | accent emphasises and must stay contrast-safe | not verified | Reuse the existing `litric-design` tokens; any new accent token ships with a contrast test | WCAG SC 1.4.3/1.4.11 |
| List of settings categories | Home + 11 categories | sidebar-based, grouping not documented | **Follow the prompt set's category list**, which already matches `SYSTEM-SCOPE.md` Part D, rather than copying either vendor | avoids inventing a taxonomy that cannot be sourced |
| Motion reduction | not verified as a requirement | not verified | **Keep the existing reduced-motion token path** in `litric-shell`; it is already implemented and tested | `docs/DESIGN-SYSTEM-v3`, ADR-0017 |

---

## 9. What this research changes about the planned waves

1. **Network (Wave 3) needs a secret-agent design before any UI.** Without
   `org.freedesktop.NetworkManager.SecretAgent`, a Wi-Fi password either cannot
   be stored or gets stored somewhere LitricOS invented — which the prompt set
   forbids. This is a design ticket, not a code ticket.
2. **"Wrong password" cannot be a first-class UI state** in the way a mock would
   suggest: the NetworkManager docs define no such error. The state machine must
   be driven by `NMDeviceState` + state reasons.
3. **Bluetooth "forget" and "disconnect" are different backend calls**
   (`RemoveDevice` vs `Disconnect`), and discovery is shared, so a settings pane
   that leaks a discovery session affects the whole machine.
4. **Logind cannot tell us the screen is locked**; it can only ask us to lock.
   The lock is ours to build and ours to prove — and until PAM integration
   exists, it must not exist at all.
5. **Boot-to-desktop needs its own milestones.** `systemd-analyze` measures
   spawn, not readiness, and `graphical.target` is not "a user is logged in".
   The existing `9.17 s` figure is a spawn-era number and cannot be compared to
   a "desktop ready" target without redefining the metric.
