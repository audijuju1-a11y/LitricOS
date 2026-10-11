# LITRIC — PHASE 06 PROMPT: LITRIC DESKTOP SHELL

**Prerequisite:** GATE 05 passed.
**Deliverable:** the shell — this is where LITRIC stops being "a distro" and becomes a platform with an identity.

---

## OBJECTIVE
Build the LITRIC desktop from first principles: fast, clean, consistent, keyboard/mouse/touch friendly, accessible, customizable. Not a Windows or macOS pixel clone.

## WHY
Kernel, Mesa, PipeWire are shared upstream. The shell is the part users actually experience and the part that is genuinely ours.

---

## TASKS

1. **Interaction model first, pixels second** — `/docs/UX-06-model.md`
   Decide and justify before drawing anything:
   - window model (floating / tiling / hybrid — pick one default, allow the other)
   - navigation primitive (workspaces? overview? single keystroke launcher?)
   - what a user does in the first 10 seconds after login
   - keyboard-complete: every action reachable without a mouse

2. **Shell components** — `/shell/`
   - compositor shell layer (panel/dock/overview) via `wlr-layer-shell` or equivalent
   - launcher + unified search (apps, files, settings, actions)
   - task/navigation system
   - file manager
   - settings (see task 4)
   - notification service (with per-app permission + Do-Not-Disturb + Game Mode suppression)
   - system information panel
   - application manager (install/permissions/updates)

3. **Design tokens** — `/shell/tokens/`
   One source of truth for color, type scale, spacing, radius, motion durations. Light + dark + high-contrast. Every component consumes tokens; no hardcoded values anywhere.

4. **Settings architecture — "deep settings" is a first-class feature**
   Three tiers, always visible as tiers:
   ```
   Simple    - one switch, safe defaults
   Advanced  - real knobs with explanations and measured impact
   Expert    - raw sysctl/sysfs/kernel params, each with a WARNING and a REVERT button
   ```
   Every Expert knob shows: current value, default, what it changes, measured effect if known, security cost if any. Nothing in Expert is enabled by default.

5. **Accessibility** — keyboard navigation, screen-reader hooks, scalable UI (fractional scale respected), reduced-motion, high contrast. Treat as a gate item, not a wish.

6. **Memory discipline** — the shell has a `MemoryHigh=` in its slice. Any component that exceeds it in normal use is a bug, not a tuning issue.

7. **Session robustness** — the shell must be restartable in place. If the panel crashes, the session survives; if the compositor crashes, open documents survive via app-side state.

---

## EXIT GATE — GATE 06
- [ ] Full session: login → desktop → launch app → switch windows → notification → settings → log out. No crashes.
- [ ] **Desktop idle RAM ≤ 600 MB total** (compositor + shell + services). Record the breakdown per process.
- [ ] Every user-facing action is reachable by keyboard alone.
- [ ] Expert settings tier: at least 10 real knobs, each with revert; changing one and reverting leaves the system byte-identical in that setting.
- [ ] Shell component crash → auto-restart within 2 s, session intact. Demonstrated by killing each component.
- [ ] 4-hour soak test: RSS growth < 5% (no leaks).
- [ ] A screenshot set exists so later phases can regression-check visual changes.

## FAILURE MODES
- Feature creep → the shell ships the 8 components listed above and nothing else this phase.
- "Looks like GNOME/Windows" → if a reviewer cannot tell it apart in a blind screenshot, the interaction model in task 1 was not actually decided.

## NEXT STEP
`08-PHASE-07-SECURITY-DEFENDER.md`
