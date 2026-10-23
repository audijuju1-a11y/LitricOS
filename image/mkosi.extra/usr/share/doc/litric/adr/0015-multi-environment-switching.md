# ADR-0015 — LitricOS as host: Windows and Linux as switchable environments

- **Status:** Accepted (design), implementation started at Layer 2
- **Date:** 2026-08-28
- **Supersedes:** nothing. Extends ADR-0009 (unified shell) and the compatibility
  model in `LITRIC_NATIVE_ECOSYSTEM_MASTER_ARCHITECTURE_v1.md` §17.

---

## Requirement

Power on → a LitricOS screen. Optionally a password (a setting the user can turn
off). By default it goes straight into LitricOS. From there:

```
Win+1  →  Windows 11
Win+2  →  LitricOS
Win+3  →  Linux
```

Switching is instant, both other environments keep running, and LitricOS remains
the owner of the machine throughout.

---

## The decision

**LitricOS is the host and the hypervisor.** Windows and Linux are KVM guests.
`litric-shell` owns the display; each environment's framebuffer is a surface it
composites. `Win+N` changes which surface is presented.

```
                 hardware
                    │
              LitricOS kernel  (KVM host)
                    │
   ┌────────────────┼────────────────┐
   │                │                │
litric-shell    Windows VM       Linux VM
(owns display)  (guest)          (guest)
   │                │                │
   └──── surfaces ──┴────────────────┘
                    │
              Win+1 / 2 / 3
           selects which is shown
```

This is **not dual boot.** Nothing reboots. The other environments keep executing
(or are suspended by policy), and the switch is a compositor operation.

### Why this shape and not the alternatives

| Option | Verdict |
|---|---|
| **Dual boot** | Rejected. A reboot per switch is not "switching", and only one OS is alive at a time, so LitricOS cannot stay the trust anchor or the resource owner. |
| **Windows as host, LitricOS as guest** | Rejected. Inverts the dependency direction the whole project exists to establish (ECOSYSTEM §22 rule 15). |
| **Compatibility runtime only (Wine/Proton)** | Insufficient alone — but *retained*, and preferred. See "two tiers" below. |
| **LitricOS as host + KVM guests** | Selected. |

### Two tiers, and the order matters

`litric-winbridge` (already built and tested) runs a Windows `.exe` **inside the
LitricOS desktop** in a per-app bottle. That is the everyday path: no VM, no
second desktop, the app appears as a normal window.

The full Windows VM is the **fallback** for what a compatibility runtime cannot
do — kernel drivers, anti-cheat, undocumented interfaces, hardware-specific
software. Reaching for the VM first would waste several gigabytes of RAM to run
a text editor.

```
Windows .exe
   ├─ runs under the compatibility runtime?  → bottle, in the LitricOS desktop
   └─ needs a real Windows kernel?           → Windows VM, Win+1
```

---

## Constraints that are real, and stated before anyone builds on them

These are not caveats added for safety. Each one changes what can be promised.

### 1. GPU performance in the Windows VM is the hard limit

A guest gets good GPU performance in exactly two ways:

- **VFIO passthrough** — a physical GPU is handed to the guest. Requires a
  *second* GPU (or SR-IOV, which is rare on consumer parts), because a
  passed-through GPU is unavailable to the host while assigned.
- **virtio-gpu with Venus/native-context** — paravirtual Vulkan. Improving fast,
  still short of native, and driver support in a Windows guest is the weak link.

On a single-GPU laptop, the honest position is: **the Windows VM is good for
desktop and productivity, and is not the way to play a demanding game.** Games go
through Proton in LitricOS, or through VFIO on a machine with two GPUs.

Anyone who writes "Win+1 for gaming" in a feature list without this sentence
attached is making a claim the hardware does not support.

### 2. Windows licensing

LitricOS cannot ship Windows. The user supplies their own installation media and
a licence valid for a virtual machine. The Environment Manager installs *from
media the user provides*; it never distributes Windows.

### 3. Windows 11 guest requirements

Windows 11 requires TPM 2.0, Secure Boot and UEFI. All three are available to a
guest: `swtpm` provides a software TPM 2.0, and OVMF has a Secure Boot build.
These are already installed on the build host. This constraint is satisfiable.

### 4. Memory is the budget that bites

Three concurrent environments on a 16 GB machine is tight:

```
LitricOS host + shell     ~0.4 GB
Windows 11 guest          ~4.0 GB minimum, 8 GB to be pleasant
Linux guest               ~2.0 GB
                          ─────────
                          ~6.4-10.4 GB before the user opens anything
```

This is why the Environment Manager suspends and hibernates rather than leaving
everything resident, and why `Win+N` on a memory-constrained machine may resume
from suspend rather than switch instantly. **Instant switching is a property of
having the RAM, not of the design.** The system must say which it did.

### 5. Nested virtualization

If LitricOS itself runs in a VM (as during development), the guests need nested
virtualization. It works on modern hardware but is slower and is not a
configuration to benchmark against.

---

## Switching semantics

`Win+N` maps to an intent, not directly to an action. The Environment Manager
decides what actually happens based on the target's current state:

| Target state | What `Win+N` does | Latency |
|---|---|---|
| `RUNNING` | Compositor swaps the presented surface | ~1 frame |
| `SUSPENDED` | Resume, then swap | ~1–3 s |
| `HIBERNATED` | Restore from disk, then swap | ~10–20 s |
| `STOPPED` | Start the environment, then swap | boot time |
| `NOT_INSTALLED` | Offer to install; do not pretend to switch | — |

The system reports which of these happened. "Switching took 12 seconds" is a
useful thing to say; silently appearing to hang is not.

### The keys are a default, not a law

`Win+1/2/3` is the default binding. It is remappable, because `Win+N` collides
with taskbar-position shortcuts inside a Windows guest. While a guest has focus,
the switch chord must be **grabbed by the host compositor before the guest sees
it**, or the user will find that Win+2 sometimes opens the second taskbar item in
Windows instead of returning to LitricOS. Host-level grab is mandatory, not a
nicety.

An escape binding (default `Ctrl+Alt+Home`) always returns to LitricOS, for the
case where a guest has grabbed input or is hung.

---

## What LitricOS keeps owning while a guest is in front

This is the property that makes it a platform rather than a VM launcher. Even
while the user is working in Windows:

- **Storage** — guest disks are files or volumes LitricOS owns; guests do not get
  raw access to the physical disk
- **Security** — the capability broker still mediates what the guest may reach
- **Resources** — CPU, memory and power budgets are host-enforced (cgroups, KVM
  limits), so a guest cannot starve the control plane
- **Recovery** — LitricOS Recovery boots without any environment being healthy
- **Update** — the host updates independently of guests
- **Boot** — guests cannot modify the LitricOS boot configuration

A guest crashing, being compromised, or exhausting its memory is contained to
that guest. `docs/SYSTEM-SCOPE.md` §I3 already treats this as a differentiator;
this ADR is where it becomes a mechanism.

---

## First-run and login

```
power on
   └─ LitricOS boot (verity-verified)
        └─ LITRIC screen
             ├─ password, if enabled (Settings → Accounts → "Require password")
             └─ default target: LitricOS
```

The password toggle is a real setting with a real consequence, and the setting
UI must say it: with it off, anyone with physical access reaches the desktop.
Disk encryption is a separate control and is the one that actually protects data
at rest — the two must not be presented as if either substitutes for the other.

---

## Layering — what may be built when

The Environment Manager spans layers, and only the lowest part is buildable now.

| Part | Layer | State |
|---|---|---|
| Environment model, states, transitions, switch policy, resource budgets | 2 (Core Services) | **buildable now — `litric-env`** |
| Hotkey binding model and conflict rules | 2 | **buildable now** |
| VM lifecycle (KVM, swtpm, OVMF, disk management) | 3 | needs a working `core` profile |
| Guest surface compositing, `Win+N` presentation | 5 | needs `litric-shell` |
| Install-from-media flows, Environment Manager UI | 6 | needs the shell |

Building the policy engine first is deliberate: it is pure logic, it is testable
today without a hypervisor, and it is the part that decides what the machine
does. The parts that need hardware come later and call into it.

---

## Revisit if

- Single-GPU paravirtual Vulkan (Venus/native-context) reaches a state where a
  Windows guest can drive a demanding game, which would remove constraint 1.
- Measured switch latency for the `RUNNING → RUNNING` case exceeds ~2 frames, in
  which case surface compositing is the wrong mechanism.
- Memory pressure makes three concurrent environments impractical often enough
  that a different default (one guest resident, others hibernated) is warranted.
