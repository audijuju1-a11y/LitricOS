# 00 — Vision and non-goals

## Product statement

LitricOS is a native-first personal computing platform for x86-64 laptops and
desktops with 8 GB RAM or more. It is designed to make everyday work feel
immediate, private, recoverable, and resource-efficient. Its differentiator is
not a claim to replace every operating system or application: it is a coherent
desktop whose performance, security, and recovery characteristics are measured
and published.

## Success criteria

| Outcome | Evidence required |
|---|---|
| A user reaches a usable desktop reliably | cold-boot, login, recovery and safe-mode tests on supported hardware |
| Background work does not degrade interaction | p99 input-to-present, PSI and foreground contention traces |
| The system is light at idle | reproducible RAM, CPU, wakeup, I/O and battery measurements |
| A failure is contained and recoverable | injected app, service, update and storage failures |
| Data and permissions remain under user control | sandbox, encryption, revocation and audit tests |

## Target users

- Personal laptop and desktop users who value fast local work, understandable
  privacy controls, keyboard efficiency, and long battery life.
- Developers who need a stable, sandboxed desktop platform and first-class
  profiling/debugging tools.
- Early adopters prepared to use native and Flatpak applications while Windows
  and Linux environments remain explicitly experimental.

## Non-goals for v1

- Universal Windows application, anti-cheat, peripheral, or driver support.
- ARM64 support, cloud-first AI, full enterprise MDM, mobile phone integration,
  or a bespoke browser engine.
- Replacing Linux kernel, Mesa, PipeWire, systemd, Flatpak, or cryptographic
  update foundations with a Litric rewrite.
- Publishing unmeasured claims such as "fastest" or "lowest RAM".

## Product principles

1. Native contracts before compatibility shims.
2. Event driven rather than polling; low wakeups are a user-experience feature.
3. Defaults must be safe, privacy-preserving, and reversible.
4. Every privileged action has a visible, revocable permission path.
5. A broken update must produce recovery, not a support dead-end.
6. Accessibility is an equal functional requirement, not post-release polish.

## Decision gates

Feature proposals must name: user problem, affected release, owner, native
contract impact, permissions, resource cost, failure behaviour, and a test that
would catch an incorrect implementation. Missing any security, recovery, or
measurement element blocks design approval.

