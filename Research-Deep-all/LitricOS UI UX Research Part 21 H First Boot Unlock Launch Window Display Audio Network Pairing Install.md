# LitricOS UI/UX Research — Part 21 of N

Chapter 42 (H) — Ten workflows: four do not exist, three are read-only and say
so, and the one that guards the machine has no attempt limit

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P21 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 42 (`H`): `H.01`–`H.10` — first boot, account creation, unlock, app launch and switching, window management, display, audio, network, device pairing, install/uninstall |
| Batch | B10 first pass (`Part1` §2.2). `H.11`–`H.20` are `Part 22` |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (10 notes written; statuses `PARTIAL`), `LitricOS Source Register.csv` (S3839–S3844 appended) |
| Predecessors | `Part 3` (Windows shell and Explorer), `Part 11` (search and errors), `Part 15` (dialogs and confirmation), `Part 18` (loading states), `Part 19`–`Part 20` (keyboard and input) |
| Resume point | `H.11` default apps → `H.20` failure paths |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and used here without redefinition.

---

## 0. Scope, method, and the five states

### 0.1 Ten workflows, one question each

| Workflow | The question this part answers |
|---|---|
| `H.01` First boot and setup | can a person get from power-on to a usable desktop, and what does the OS decide for them? |
| `H.02` Account creation | is there an account, and where is it made? |
| `H.03` Unlock | can the machine be locked, and what stops a guess? |
| `H.04` Launch and switch apps | can an app be started, and does failure show? |
| `H.05` Window management | can a window be arranged, and can the arrangement be undone? |
| `H.06` Display change | can the output change, and is a bad change reversible? |
| `H.07` Audio change | can the volume change, and does the interface know what it changed? |
| `H.08` Network change | can the machine join a network? |
| `H.09` Device pairing | can a Bluetooth device be paired? |
| `H.10` Install and uninstall | can software be added or removed, and who decides? |

### 0.2 The shape of a workflow, and why five states

Every platform in scope documents workflows as if five states existed, whether
or not the article names them:

1. **Entry** — a control that starts the flow, reachable by pointer and keyboard.
2. **Progress** — what the user sees while it is happening, including "nothing
   is happening yet".
3. **Failure** — a specific reason, in the interface, not only in a log.
4. **Cancel** — a way out that leaves the previous state intact.
5. **Retry or rollback** — a way to try again, or to undo the result.

A workflow missing (3) and (5) is a demo. A workflow missing (1) is a
screenshot. This part applies the five states to all ten, which is also the
lens `H.20` will formalise: `H.20` asks for the table, this part fills the first
ten rows of it.

### 0.3 What was measured (MEASURED from the repository)

| Artifact | Value |
|---|---|
| Settings pages | **13** (Appearance, Display, Input, Network, Bluetooth, Power, Region, Environments, Accessibility, Sound, Privacy, Accounts, Diagnostics) |
| First-run setup choices | **3** (theme, language, reduce motion) plus one action (Finish setup) |
| First-run account / network / time zone / keyboard steps | **0 / 0 / 0 / 0** |
| Lock stages | **2** (`Idle`, `Entry`) |
| Lock verdicts | **4** (`Idle`, `Checking`, `Failed`, `NoVerifier`) |
| Failed-password attempt counter or delay | **0** |
| Window-manager public operations | **36** in `manager.rs` |
| Display outputs in the compositor | **1** |
| Display page rows that change anything | **0 of 5** |
| System audio verbs | **4** (`LEVEL`, `VOLUME`, `MUTE`, `LOUDER`) |
| Shell volume controls | **3** hotspots (down, up, mute) |
| Bluetooth verbs | **5** (`Power`, `Scan`, `Pair`, `Connect`, `Forget`) |
| Bluetooth daemons in any image profile | **0** (`bluez` in no profile) |
| Installer / package service in the tree | **0** |

### 0.4 The table this chapter is really asking for

| Workflow | Entry exists | Failure named in UI | Cancel | Retry / undo | Verdict |
|---|---|---|---|---|---|
| `H.01` Setup | yes (first run only) | n/a (no operation) | **no** | **no** (cannot re-enter) | real but thin |
| `H.02` Account creation | **no** | — | — | — | absent |
| `H.03` Unlock | yes | yes (`Failed`, `NoVerifier`) | yes | retry unlimited, **unthrottled** | real, unsafe |
| `H.04` App launch | yes | **no** | n/a | **no** | real, silent failure |
| `H.05` Window management | yes | n/a | yes (`cancel_drag`) | partial (restore for snap/maximise) | real |
| `H.06` Display change | **no** | n/a | — | **no revert timer** | read-only |
| `H.07` Audio change | yes | yes (`audio_note`) | n/a | manual | real, partial |
| `H.08` Network change | **no** | — | — | — | read-only |
| `H.09` Pairing | **no** (facts only) | yes (why it cannot) | — | — | read-only |
| `H.10` Install / uninstall | **no** | — | — | — | absent |

Four of ten have no entry point at all. Three of those four (`H.06`, `H.08`,
`H.09`) are read-only **on purpose** and name the reason on screen, which is a
different failure from a control that lies. The fourth (`H.10`) has its trust
design written and no flow behind it.

---

## H.01 — First boot and setup

### H.01.1 What the platforms require

Windows 11 OOBE is a long, enumerated flow: language, region, keyboard, network
(required on Home SKUs), critical patches, EULA, account sign-in or creation,
Windows Hello, up to seven privacy settings, device-usage intent, OneDrive, a
Microsoft 365 trial, and OEM pages. It restarts mid-flow and repeats pages when
the network was never available, and it ends with a **quiet period** in which no
application may show UI — the desktop opens with Start instead of a pile of
windows ([S3839]).

GNOME's initial setup is shorter and covers the same ground: language, region,
input sources, Wi-Fi (only when there is no connection), time zone, online
accounts, name/password, enterprise login, complete ([S3840]).

The shared claim is not "many pages". It is that **identity, connectivity, time
and input are decided before the desktop appears**, because every later workflow
depends on them.

### H.01.2 What the shell has

`draw_setup` renders one elevated pane, 460 px wide (clamped to the screen) and
360 px tall, at 22 % of screen height, containing a title, one sentence of
introduction, three option rows and a Finish button. The rows reuse the Settings
handlers: `SettingsTheme`, `SettingsLanguage`, `SettingsMotion`.

The gate is absolute — while `settings.preferences.onboarded` is false, the
shell draws the setup panel and returns; windows, top bar, dock, overlays and
banners are never reached. `onboarded` is persisted in `settings.conf` and is one
of two keys (`version`, `onboarded`) deliberately excluded from the Settings
registry, so setup cannot be re-entered from inside the session.

### H.01.3 Findings

- **`H01.01`** — Setup decides appearance and motion, and decides nothing else.
  A finished first boot can leave the machine with **no account** (`H.02`), **no
  network** (`H.08`), **the wrong time zone**, and **a keyboard layout derived
  from the locale rather than chosen**. Windows and GNOME both treat those four
  as setup's reason to exist.
- **`H01.02`** — There is **no skip and no back**. Three rows and one Finish:
  the only exit is forward. Windows' OOBE at least allows a page to be skipped;
  GNOME's design explicitly changes Skip to Next when connectivity arrives.
- **`H01.03`** — Setup cannot be re-run, by design, and there is no supported
  "reset setup" for support or for a demo machine. The exclusion from the
  registry is the right call (a settings row that wipes the desktop is a trap),
  but it leaves recovery to editing a file by hand.
- **`H01.04`** — The panel is honest about its own scope: the sentence says the
  two appearance choices can be changed later in Settings. Nothing claims to
  have set up an account or a network.

Status: **PARTIAL** — Windows and GNOME evidence inspected; the macOS Setup
Assistant article returned HTTP 429 and is registered as `INACCESSIBLE`, so the
macOS comparison for `H.01` and `H.02` is not made here.

---

## H.02 — Account creation

### H.02.1 What the platforms require

Windows makes the account part of OOBE: sign in with a Microsoft account or
create a local one, then enrol Windows Hello; Home SKUs require a network to
finish ([S3839]). macOS creates the first administrator inside Setup Assistant.
GNOME creates the user, password and (optionally) enterprise login inside initial
setup ([S3840]).

### H.02.2 What the shell has

There is **no account creation path in the tree**: no `useradd`, no user
creation request in the daemon protocol, no page that creates an account. The
Accounts page reports facts — the desktop account a lock would check, and
whether a password verifier exists (`unix-chkpwd`, `pam-helper`) — and the lock
checks the account named by logind's session record rather than "whichever
session is listed first".

Provisioning is therefore out of band: the image ships a user, and the shell
describes it.

### H.02.3 Findings

- **`H02.01`** — Account creation is **absent**, and setup does not pretend
  otherwise. For a single-user appliance image this is defensible; for the
  product the rest of the tree describes it is a hole, because the preference
  store explicitly reasons about "a multi-account machine" while no surface can
  add, switch or remove an account.
- **`H02.02`** — The lock's account choice is a genuine strength: it follows the
  daemon's `auth.account`, so a second logged-in session cannot redirect a
  password check. That is the one place the account model is implemented
  correctly.
- **`H02.03`** — The Accounts page is a report, not a control. Nothing on it
  changes an account, a password or a session.

Status: **PARTIAL** — shell measured; Windows and GNOME inspected; macOS
`INACCESSIBLE` (S3844).

---

## H.03 — Unlock

### H.03.1 What the platforms require

Windows offers password, PIN, Windows Hello biometrics and security keys, with
lock-screen and sign-in options as separate surfaces and automatic lock on
absence ([S0421], [S0423], [S0426], [S0344]). macOS uses the login window with
password and Touch ID. GNOME uses the lock screen with password and fingerprint.
All three treat **rate limiting** as part of the flow even when the article does
not say so: the platform, not the UI, is what slows guessing down.

### H.03.2 What the shell has

Two stages (`Idle` → `Entry`) and four verdicts: `Idle`, `Checking`,
`Failed(String)`, `NoVerifier(String)`. `NoVerifier` is the interesting one: when
the machine has no password verifier, the surface **refuses to collect a secret
it could never check** and says so, and the shell refuses to lock at all in that
state (covered by an end-to-end test). The typed password leaves as a `Secret`
whose only formatting method is hexadecimal, so it cannot be printed into a
status line by accident.

There is **no attempt counter, no delay, no backoff and no lockout**: the strings
`attempt`, `throttle`, `backoff`, `cooldown` appear in the lock and shell only as
comments and as an unrelated network probe constant.

### H.03.3 Findings

- **`H03.01`** — **No rate limit.** A machine that can verify a password can be
  asked an unlimited number of times at whatever rate the helper allows. Every
  platform in scope treats this as the platform's job, and here the shell is the
  only place it could live. This is the most serious finding in this part.
- **`H03.02`** — Refusing to lock without a verifier is better than the common
  behaviour of showing a field that can never succeed. It is also the reason
  `H03.03` matters: the surface distinguishes "wrong password" from "this machine
  cannot check".
- **`H03.03`** — No hint for the two failures users actually hit: caps lock, and
  a keyboard layout that is not the one they think. The password field is a
  single pill with no state indicator beyond the verdict line.
- **`H03.04`** — No alternate authenticator: no PIN, no biometric, no security
  key, and no "sign in as another user".

Status: **PARTIAL** — shell measured; Windows inspected; macOS and GNOME lock
screens documented in `Part 19` only at guideline level.

---

## H.04 — Launch and switch apps

### H.04.1 What the platforms require

Windows launches from Start search and the taskbar, switches with Alt+Tab and
Task View, and shows a failure (a dialog or a toast) when an app cannot start.
macOS launches from the Dock, Launchpad and Spotlight and switches with ⌘Tab.
GNOME launches from the Activities overview and switches with Super+Tab. In all
three, **switching is ordered by recency of use**, not by creation order.

### H.04.2 What the shell has

A single static app registry, the dock/launcher surface, and a window manager
that opens windows through `open`/`open_at`. Switching is `cycle_focus`,
`cycle_app` and `switcher_order`, which is recency-ordered for the switcher.

### H.04.3 Findings

- **`H04.01`** — **Launch has no failure surface.** If an app fails to start
  there is no dialog, no banner and no error state: the window simply does not
  appear. `Part 11` established that errors exist as a component; this workflow
  does not use one.
- **`H04.02`** — Launch has no progress state either. A slow start is
  indistinguishable from a click that missed, because nothing is drawn between
  the click and the window (compare `H.06`'s read-only honesty).
- **`H04.03`** — Switching is implemented and recency-ordered, which is the part
  most shells get wrong; there is no "open with" and no per-app jump list, so
  the launch surface is one path deep.

Status: **PARTIAL** — shell measured; platform behaviour from `Part 3` and this
part's sources.

---

## H.05 — Window management

### H.05.1 What the platforms require

Windows: drag to snap, hover the maximise button for Snap Layouts, snap groups
to restore a pair, virtual desktops. macOS: Spaces, full screen, Stage Manager,
and a green button that offers tile-left/right. GNOME: dynamic workspaces,
drag-to-edge tiling in 51, and window switching per workspace.

The common requirement is not the gesture set. It is that **every arrangement
can be undone** and that a drag can be abandoned.

### H.05.2 What the shell has

`manager.rs` exposes **36 public operations**, including workspaces (`set`,
`cycle`, `move_to_workspace`), `snap`, `toggle_maximize`, `restore`, `minimize`,
`unminimize`, `center`, `focus`, `raise`, `reveal`, `hit_test`, and a drag
lifecycle with `begin_drag` → `update_drag` → `end_drag` **and `cancel_drag`**.
Damage rectangles are tracked per operation, and diagnostics are exposed as
strings.

The compositor has **one output**: `set_display(width, height, scale)` takes a
single triple, with no per-output geometry, and one workspace list.

### H.05.3 Findings

- **`H05.01`** — `cancel_drag` is a real strength: an abandoned drag restores the
  frame instead of leaving a window half-snapped. Most shells implement this as
  a side effect of pointer release; here it is a named operation.
- **`H05.02`** — Undo is asymmetric: `restore` reverses snap and maximise, and
  `unminimize` reverses minimise, but **closing a window cannot be undone**. The
  close path is the one arrangement users actually regret.
- **`H05.03`** — Snap is edge-based only. There are no Snap Layouts, no snap
  groups, no tiling layouts beyond halves/quarters, and no per-monitor
  workspaces, because there is one monitor (`H.06`).

Status: **PARTIAL** — shell measured; platform behaviour from `Part 3` and
`Part 18`.

---

## H.06 — Display change

### H.06.1 What the platforms require

Windows: Settings → System → Display, resolution, scale, orientation, multiple
displays, HDR, night light; a mode change is applied with a **countdown and
automatic revert** if the user does not confirm, which is the only thing that
makes an unreadable mode safe ([S0115]). macOS and GNOME do the same with a
confirmation dialog.

### H.06.2 What the shell has

The Display page has **five read-only rows**: current framebuffer, QEMU default
resolution (1920 × 1080), compatibility mode (1280 × 800), UI scale, and one that
answers the obvious question — "Display changes: **The framebuffer mode is
selected at boot**".

### H.06.3 Findings

- **`H06.01`** — There is **no display-change workflow**, and the page says why
  in the row where a control would be. That is the correct way to ship an
  absent feature; it is still an absent feature.
- **`H06.02`** — **No revert timer exists anywhere in the shell.** If mode
  switching is added later, the safe pattern (apply → keep this mode? → revert)
  has no infrastructure to build on, and neither does any other destructive
  setting. This is the finding that `H.20` will generalise.
- **`H06.03`** — The page mixes facts from three sources (live framebuffer,
  emulator default, compatibility mode) without labelling which is current. A
  reader can infer it from the first row, but the other two are not marked as
  "this machine".

Status: **PARTIAL** — shell measured; Windows inspected; macOS and GNOME
documented in `Part 6`/`Part 20` at token level.

---

## H.07 — Audio change

### H.07.1 What the platforms require

Windows: volume flyout with per-app mixer, output-device picker, mute, and the
Sound settings page. macOS: menu-bar slider, output device in Control Center,
Sound settings with input/output and balance. GNOME: quick-settings slider, mute,
and per-app stream volume in Settings → Sound.

### H.07.2 What the shell has

The daemon speaks **four verbs** — `AUDIO LEVEL` (query), `AUDIO VOLUME <pct>`,
`AUDIO MUTE on|off`, `AUDIO LOUDER <step>` — through `wpctl`, the WirePlumber
client, with presence and control separated: a socket without `wpctl` reports
`audio.control=none` rather than implying a control that does not exist.

The shell has **three hotspots** (down, up, mute) in quick settings, a
`volume: Option<u8>` where `None` means "unknown" and is deliberately different
from `0`, a `muted` flag, and `audio_note` — the interface's own words for what
the last change did.

### H.07.3 Findings

- **`H07.01`** — `Option<u8>` is the right model: "no sound server" cannot be
  rendered as "volume zero". `Part 20` found the same discipline in the token
  layer; here it survives into the UI state.
- **`H07.02`** — There is **no output-device picker and no per-app mixer**. The
  default sink is the only sink a person can influence from the UI.
- **`H07.03`** — Failure is named (`audio_note`) but has **no retry action**: a
  refused volume change leaves a sentence and no button. The user must press the
  control again and hope.

Status: **PARTIAL** — shell measured; Windows and GNOME articles exist in the
register from `Part 2`; macOS at guideline level.

---

## H.08 — Network change

### H.08.1 What the platforms require

Windows: network flyout with Wi-Fi list, connect with credentials, metered
settings, VPN (`S0278`, `S0280`). macOS: menu-bar Wi-Fi with the same list.
GNOME: Wi-Fi panel, and initial setup shows Wi-Fi only when there is no
connection ([S3840]). Joining a network is a first-class workflow everywhere
because OOBE depends on it.

### H.08.2 What the shell has

The network module reads **the kernel's own surfaces only** — `/sys/class/net`,
`/proc/net/dev`, `/proc/net/route`, `/proc/net/if_inet6`, `ip -o -4 addr show`,
and `resolved`'s `resolv.conf` — and states plainly that `systemd-networkd` and
`iwd` own the configuration. Wi-Fi scanning needs `iwd`'s D-Bus API and is
deferred to Phase 2; the Settings pane shows known networks and says why there
is no list.

The Network page is a report: connection state, interfaces, addresses. **Zero
controls** change anything.

### H.08.3 Findings

- **`H08.01`** — **No network workflow.** Combined with `H.01`, a fresh image has
  no path from the UI to the internet — the two absent workflows are the same
  absence seen twice.
- **`H08.02`** — The read-only decision is implemented with unusual rigour: the
  module explains why it does not parse IPv6 routes, why `ip` is the one external
  command, and why "no address" is printed as `none` rather than as an empty
  list. This is what a boundary looks like when it is documented instead of
  implied.
- **`H08.03`** — A "down" is earned by repeated probes rather than read from one
  failure, which is the correct shape for a status row that will later gain a
  connect button.

Status: **PARTIAL** — shell measured; Windows inspected (`S0278`, `S0280`);
GNOME design inspected (`S3840`).

---

## H.09 — Device pairing

### H.09.1 What the platforms require

Windows: Bluetooth tile in quick settings, Add device → Bluetooth, device made
discoverable, pair, Done; Swift Pair raises a notification for supported devices
in range, and paired devices reconnect automatically ([S3843]). macOS and GNOME
follow the same three steps.

### H.09.2 What the shell has

The daemon defines **five Bluetooth verbs** — `Power`, `Scan`, `Pair`,
`Connect`, `Forget` — with capability scopes: radio operations are scoped to
`bt://` ("every Bluetooth resource on this machine") and per-device operations
to the address they name. The facts layer reads adapters from sysfs and reports
radio blocks.

But **`bluez` is in no image profile**, so there is no `bluetoothd` to pair
through; the module says pairing, trust and connect "are not implementable here
yet". The Settings page makes **four** answers out of the usual two: no radio at
all, radio blocked, radio ready but nothing that could pair, and ready.

### H.09.3 Findings

- **`H09.01`** — The verbs and their authority model exist; the transport does
  not. This is a rarer and better failure than a button that silently does
  nothing: the capability design is done, the daemon is missing.
- **`H09.02`** — The four-answer state is the correct UI for it. "No radio" and
  "no daemon" are different facts, and only one of them is a hardware problem.
- **`H09.03`** — There is no device list, because there is nothing to list. When
  `bluez` arrives, the missing pieces are discovery results, a pairing dialog
  with a passkey, and a Forgot-this-device confirmation — `Part 15` has the
  dialog component and `Part 18` the empty/loading states.

Status: **PARTIAL** — shell and daemon measured; Windows inspected; macOS and
GNOME at guideline level.

---

## H.10 — Install and uninstall

### H.10.1 What the platforms require

Windows: install from a package or the Store; uninstall from Start (right-click →
Uninstall), Settings → Apps → Installed apps (More → Uninstall), or Control
Panel, with the explicit note that **some apps cannot be uninstalled** and that
built-in ones cannot be removed at all ([S3841]).

macOS: Gatekeeper verifies the Developer ID signature and, since Catalina,
requires notarisation; the first launch asks for approval; a blocked app is
opened through Privacy & Security → Open Anyway; known malware is moved to the
Trash with a notification ([S3842]).

Both platforms put the trust decision **in front of the first run**, not inside
the installer's own window.

### H.10.2 What the shell has

`litric-manifest` parses application manifests as a strict INI subset, before any
trust decision. Its rules are the interesting part:

- an unknown capability is a **hard error**, never ignored, because "ignoring it
  would mean a manifest could request something the install dialog does not
  display";
- a duplicate key is a hard error rather than last-wins;
- the declaration is what the install dialog renders, ordered by risk, in
  plain language.

There is **no package service, no installer and no uninstall surface** in the
tree; the crate's own documentation says "the package service decides what to do
with it".

### H.10.3 Findings

- **`H10.01`** — The trust chain is designed from the correct end: the parser is
  written so the dialog **cannot understate** what an app asks for. That is the
  single hardest property to retrofit, and it is already enforced by tests.
- **`H10.02`** — No install or uninstall workflow exists. The install dialog has
  content — risk-ordered capability lines — and no dialog.
- **`H10.03`** — No uninstall-ability metadata. Windows' article exists because
  "Uninstall" is greyed out for built-in apps; a manifest format that never
  records "this cannot be removed" will produce a button that fails, which is
  exactly the failure `H09.02` avoids.

Status: **PARTIAL** — shell and manifest measured; Windows and macOS inspected.

---

## 1. What the ten say together

Ordered by what would hurt a user first:

1. **`H03.01`** — an unlock with no attempt limit. This is the only finding here
   that is a security defect rather than a missing feature, and it is fixable
   inside the shell with a counter and a delay.
2. **`H01.01` + `H08.01`** — first boot ends without identity, network or time,
   and the Network page cannot fix any of them. A machine can be set up and still
   be offline, clock-wrong and account-less.
3. **`H10.02` + `H02.01`** — the two flows that make an OS installable and
   shareable are absent, while both have their data models written.
4. **`H04.01`** — launch failures are silent. A click that does nothing is
   indistinguishable from a broken app.
5. **`H06.02`** — no revert timer. Today it costs nothing because nothing
   changes a mode; tomorrow it is the difference between a recoverable display
   change and a black screen.
6. **`H09.01`** — pairing verbs without a daemon. Honest, and blocked on an image
   decision rather than on UI work.

The pattern across all ten: **the shell is strongest where it refuses and
weakest where it accepts.** It refuses to lock without a verifier, refuses to
show a display control it cannot honour, refuses to report volume as zero when it
means unknown, and refuses a manifest capability it cannot display. It accepts a
password without limit, accepts a click that may start nothing, and accepts that
setup is finished when three preferences are set.

## 2. What this part does not establish

- **macOS first boot and account creation** are not compared: the Setup Assistant
  deployment article returned HTTP 429 (`S3844`), and no substitute was fetched.
  `H.01` and `H.02` are therefore Windows + GNOME only.
- **No rate limiting claim is made about the daemon's helper.** The finding is
  scoped to the shell: no counter, delay or lockout exists in `litric-shell`.
  PAM may impose its own delay, which this part did not measure.
- **No timing measurements.** "Unthrottled" means no code path delays a retry,
  not that a retry completes in a stated number of milliseconds.
- **The install dialog is not inspected visually** — it does not exist. The
  capability lines are read from the manifest crate's API and tests.
- **Window management was measured through the manager's API**, not through a
  session with a pointer; gestures and hit-testing constants come from `Part 3`
  and `Part 18`.
- **`H.11`–`H.20` remain `PENDING`**, including the failure-path table that this
  part's §0.4 begins.

## 3. Sources

New in this part:

| ID | Source | Status | Used for |
|---|---|---|---|
| S3839 | [Customize the out-of-box experience (OOBE)](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/customize-oobe-in-windows-11) — Microsoft Learn, updated 2026-03-13 | INSPECTED | `H.01`, `H.02` |
| S3840 | [Design/OS/InitialSetup](https://wiki.gnome.org/Design/OS/InitialSetup) — GNOME Wiki (archived) | INSPECTED | `H.01`, `H.02`, `H.08` |
| S3841 | [Uninstall or remove apps and programs in Windows](https://support.microsoft.com/en-us/windows/uninstall-or-remove-apps-and-programs-in-windows-4b55f974-2cc6-2d2b-d092-5905080eaf98) — Microsoft Support | INSPECTED | `H.10` |
| S3842 | [Safely open apps on your Mac](https://support.apple.com/en-us/102445) — Apple Support, 2026-05-27 | INSPECTED | `H.10` |
| S3843 | [Pair a Bluetooth device in Windows](https://support.microsoft.com/en-us/windows/hardware/bluetooth/pair-a-bluetooth-device-in-windows) — Microsoft Support | INSPECTED | `H.09` |
| S3844 | [Manage Setup Assistant for Apple devices](https://support.apple.com/guide/deployment/manage-setup-assistant-depdeff4a547/web) — Apple Support | INACCESSIBLE (HTTP 429) | `H.01`, `H.02` (not used for a claim) |

Reused from earlier parts: `S0115` (screen resolution and layout), `S0278`
(connect to Wi-Fi), `S0280` (essential network settings), `S0344` (lock screen),
`S0421`/`S0424` (sign-in options), `S0423` (Windows Hello), `S0426` (automatic
lock).

## 4. Status changes

| Item | Before | After | Why |
|---|---|---|---|
| `H.01` | PENDING | PARTIAL | shell measured; Windows + GNOME inspected; macOS inaccessible |
| `H.02` | PENDING | PARTIAL | same |
| `H.03` | PENDING | PARTIAL | shell measured; Windows inspected; no macOS lock article |
| `H.04` | PENDING | PARTIAL | shell measured; platform behaviour from `Part 3` |
| `H.05` | PENDING | PARTIAL | manager measured; platform behaviour from `Part 3`/`Part 18` |
| `H.06` | PENDING | PARTIAL | shell measured; Windows inspected; macOS/GNOME at token level |
| `H.07` | PENDING | PARTIAL | shell and daemon measured; platform articles from `Part 2` |
| `H.08` | PENDING | PARTIAL | shell measured; Windows + GNOME inspected |
| `H.09` | PENDING | PARTIAL | daemon verbs measured; Windows inspected |
| `H.10` | PENDING | PARTIAL | manifest measured; Windows + macOS inspected |
| `H` (chapter) | PENDING | PENDING | `H.11`–`H.20` are `Part 22` |
