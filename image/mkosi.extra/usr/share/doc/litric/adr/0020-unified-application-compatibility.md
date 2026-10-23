# ADR-0020 — One desktop, three runtimes: unified application compatibility

- **Status:** Accepted (design). Partially implemented: Windows bottle planning
  exists and is tested (`litric-winbridge`), environment policy exists and is
  tested (`litric-env`).
- **Date:** 2026-09-14
- **Extends:** ADR-0009 (the shell owns the display), ADR-0015 (LitricOS hosts
  Windows and Linux as KVM guests)
- **Complements:** `master-plan/16-compatibility-environments.md`,
  SYSTEM-SCOPE Part F3 (Windows compatibility) and Part J1 (multi-environment).

---

## Requirement

LitricOS is an independent operating system whose kernel and native environment
are the primary platform. The user runs Litric, Windows and Linux applications
**directly, inside one Litric desktop**. Installing and launching a Windows or
Linux program must not mean switching to another desktop, booting another
system, or choosing a runtime by hand. Full Windows and Linux *environments*
exist as a fallback for software a compatibility runtime cannot serve — and in
every case LitricOS remains the primary OS, the security boundary and the
resource owner.

## The decision

### 1. One desktop, by construction

ADR-0009 makes every window a surface in the shell's scene graph. A native
window, a Windows-application window and a Linux-application window are the
same kind of object to the compositor; nothing in the display path knows or
cares which runtime produced a surface. "One desktop" is therefore not a UI
promise to keep — it is the existing process topology, extended to
compatibility runtimes instead of a second desktop being launched.

### 2. The Application Manager is a decision function

Like everything else in LitricOS, runtime selection is decided in a pure,
tested decision layer and executed elsewhere. Selection is automatic; the user
chooses software, not plumbing:

| Input | Selected runtime | Execution |
|---|---|---|
| Litric native app (manifest) | Native runtime — direct platform API access | native process |
| Windows `.exe` | Windows compatibility runtime — per-app bottle (`litric-winbridge`) | bottle, inside this desktop |
| Linux app | **Native** — standard sandbox + portals, no translation layer | normal app process |
| Anything requiring a real foreign kernel | Full environment (Windows VM / Linux VM) | KVM guest, per ADR-0015 |

Escalation from bottle to full Windows VM happens **only on recorded evidence**
— a kernel driver, an anti-cheat, an undocumented interface — never as a
default. ADR-0015's two-tier rule is retained and restated here because it is
the costliest mistake available: reaching for a VM first wastes gigabytes of
RAM to run a text editor.

### 3. The Linux "compatibility runtime" is the platform itself

The requirement as written posits a separate Linux Compatibility Runtime. On
LitricOS that runtime would be a redundant copy of the platform: LitricOS is a
Linux-kernel OS (locked baseline, workflow G5), so a Linux application already
speaks the kernel's ABI natively. What "Linux compatibility" actually means
here is real — but it is packaging and policy, not ABI emulation:

- glibc user space (locked baseline decision, ADR-0001)
- Flatpak and XDG portals wired to the capability broker
- per-app-class sandbox profiles (Landlock/seccomp/namespaces, ADR-0008)

This deviation is documented rather than silently reinterpreted, because the
distinction matters to anyone porting the spec elsewhere.

### 4. Isolation and data access are policy, not convention

An application from any ecosystem is untrusted by default (ECOSYSTEM §22). The
capability broker mediates everything; compatibility runtimes get no ambient
mounts (master-plan/16). The access matrix:

| Actor | User data reach |
|---|---|
| Native app | what the user grants, per scope |
| Windows bottle | application-scoped directories only; no home, no network by default |
| Full Windows / Linux environment | user-approved directories only; never platform state |

Secrets, recovery keys, boot configuration and platform state are **never**
shared with any environment, bottle or guest. Rows 639/640 (no network, no real
home for Windows apps by default) are the enforced expression of this.

### 5. Containment

Application compromise ≠ runtime compromise ≠ environment compromise ≠
platform compromise ≠ data loss. Each boundary has a mechanism (capability
broker, sandbox layers, guest budgets and lifecycle policy, dm-verity,
separate user-data domain) and each must be testable independently. The full
ladder and its recovery consequences are ADR-0021's subject; this ADR claims
only the first three rungs.

## What is honestly not true yet

- A VM guest's output is not a window yet — row 880 (`T`). Today `Win+N`
  presents whole environments (ADR-0015 switching semantics). Per-app windows
  *from a VM guest* arrive with row 880; until then "Windows apps appear on the
  Litric desktop" is true only through the bottle path.
- The bottle execution daemon does not exist (row 626, `T`). Install plans are
  specified and tested; nothing executes them.
- Kernel anti-cheat games: RISK-001 stands. The Application Manager must show
  per-title compatibility honestly, never imply support.
- Single-GPU VM guests are good for desktop work, not demanding games —
  ADR-0015 constraint 1 carries over unchanged.

## Layering — what may be built when

| Part | Layer | State |
|---|---|---|
| Runtime selection for Windows `.exe` (bottle + evidence-based components) | 6 | **built — `litric-winbridge`, tested** |
| Environment policy, budgets, hotkeys | 2 | **built — `litric-env`, tested** |
| Bottle execution daemon | 6 | needs Layer 4 runtime services |
| Linux-app sandbox profile catalogue | 3 | needs GATE 03/04 layer |
| VM guest surfaces composited as windows | 5 | needs row 880 |
| Application Manager UI over the decision function | 6 | needs the shell |

## Revisit if

- Real-world Linux distribution software proves to need non-Linux-kernel ABI
  surfaces we do not provide — then the "Linux runtime is the platform" reading
  must be re-examined with evidence.
- Wine/Proton coverage measurably reaches the point where bottles serve
  essentially all user software — the VM tier demotes from fallback to
  specialist (drivers/anti-cheat only).
- A future platform where guests can present individual windows without full
  environment compositing would let the VM tier serve per-app windows earlier
  than row 880.
