# LitricOS Master Plan

This directory turns the high-level roadmap into an implementable, evidence-led
programme. It complements rather than replaces `docs/SYSTEM-SCOPE.md`: that file
remains the complete feature ledger and this directory supplies ordering,
interfaces, invariants, ownership boundaries, and release gates.

## Reading order

1. [00 Vision and non-goals](00-vision-and-non-goals.md)
2. [01 Product requirements](01-product-requirements.md)
3. [02 System architecture](02-system-architecture.md)
4. [03 Native contract governance](03-native-contract-governance.md)
5. [04 Boot integrity and recovery](04-boot-integrity-recovery.md)
6. [05 Kernel and driver strategy](05-kernel-driver-strategy.md)
7. [06 Storage and data lifecycle](06-storage-data-lifecycle.md)
8. [07 Compositor and shell protocol](07-compositor-shell-protocol.md)
9. [08 Accessibility, localisation, and input](08-accessibility-i18n-input.md)
10. [09 Application model, portals, and packaging](09-app-model-portals-packaging.md)
11. [10 Native SDK and developer tools](10-native-sdk-and-devtools.md)
12. [11 System services, network, and audio](11-system-services-network-audio.md)
13. [12 Threat model and sandboxing](12-threat-model-and-sandboxing.md)
14. [13 Supply chain and updates](13-supply-chain-updates.md)
15. [14 Performance methodology](14-performance-methodology.md)
16. [15 Power, thermal, and resource engine](15-power-thermal-resource-engine.md)
17. [16 Compatibility environments](16-compatibility-environments.md)
18. [17 Test lab and fuzzing](17-test-lab-and-fuzzing.md)
19. [18 Observability, crash, and support](18-observability-crash-support.md)
20. [19 First run, settings, and built-ins](19-first-run-settings-builtins.md)
21. [20 Release and security response](20-release-security-response.md)
22. [21 Feature ledger and release mapping](21-feature-ledger-and-release-mapping.md)
23. [22 Unified compatibility](22-unified-compatibility.md) — one desktop,
    three ecosystems; runtime selection as a decision function
24. [23 Deep recovery and data protection](23-deep-recovery.md) — generations,
    snapshots vs backups, recovery priority, decision engine
25. [24 Feature parity](24-feature-parity.md) — 144 features vs Windows,
    macOS, Linux; every row mapped to a crate, a ledger row, or a non-goal

## Programme and sequencing

These come after the design documents: they say what blocks Beta, in what
order the remaining work runs, and what proof each batch owes.

26. [25 Beta 1.0 gate matrix](25-beta-1.0-gate-matrix.md) — priority, blocker
    and required evidence level per area; never a second row matrix
27. [26 Execution plan](26-execution-plan.md) — work streams W0–W10 and the
    order of work
28. [27 The plan the UI/UX research implies](27-research-driven-plan.md) —
    parity ledger, work packages WP-01–WP-12, milestones M1–M6, Track R
29. [28 Whole-system improvement plan](28-whole-system-improvement-plan.md) —
    the measured baseline, the ordering rule, the batch contract and the next
    twelve batches

## Rules

- `VERIFIED` means a command, test, measurement, or hardware observation is
  recorded in `docs/EVIDENCE.md`; planned work is never described as verified.
- Native Litric remains the primary platform. Windows and Linux compatibility
  run downward through Litric services and never become a core dependency.
- Each implementation change must update the relevant SYSTEM-SCOPE row, this
  document set where the contract changes, an ADR for material decisions, and
  a test/evidence record.
- A release cannot use a performance target as a public result until the
  methodology in `14-performance-methodology.md` has produced raw evidence.

## Release labels

`FND` = Foundation; `DP` = Developer Preview; `BETA` = Public Beta;
`V1` = supported v1; `POST` = intentionally deferred.

