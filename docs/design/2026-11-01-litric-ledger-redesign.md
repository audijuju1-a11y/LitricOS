# S92 — Litric Ledger

**Date:** 2026-11-01

**Scope:** system-wide desktop visual language, with page-level restructuring where the old layout no longer held

**Evidence:** [46 after screens](../ui-review/2026-11-01/index.html) · [five light-theme screens](../ui-review/2026-11-01/light-theme.png) · [before/after](../ui-review/2026-11-01/before-after.png) · [contact sheet](../ui-review/2026-11-01/all-screens.png)

## The direction

LitricOS is a small operating system, so the interface should look like a tool rather than a
showcase. The redesign takes its cues from an engineer's ledger: mineral-dark paper, a warm
off-white ink, fired-clay accents used as registration marks, hairline rules, and small
uppercase labels that name the instrument rather than decorate it.

This replaced two earlier habits that had become inconsistent at scale: a photographic wave
wallpaper that competed with window content, and blue selection/accent colours that made every
control feel equally primary. The new language has one accent, used for position and action,
and a neutral scale for everything else.

The light theme is the same composition, not a separate skin. Its field is a warm paper tone,
its rules and ink darken, and its accent remains the same registration colour.

## What is shared

### Colour and material

- Surfaces are layered by **material** (flat, raised, inset, scrim), not by unrelated fills.
  Adjacent surfaces separate with a hairline before they separate with a shadow.
- The accent marks selection, primary actions, the running application and the wallpaper
  registration spine. Ordinary actions are neutral ledger rows.
- Both dark and light palettes are defined in `litric-design`; page code consumes semantic
  tokens (`text`, `text_dim`, `border`, `accent`, `tone`) rather than raw palette entries.
- Destructive, warning and success states keep separate semantic tones. The accent does not
  double as a warning.

### Type

- The scale is built around working sizes, not display sizes: 13 px footnote, 14 px body,
  16 px body-large, 18/22/26 px headings, 32/42 px display. Settings and file metadata no
  longer fall back to 11 px labels that were legible only in a screenshot.
- Headings use the sans face; code and terminal-like data use the mono face. A terminal line
  that contains Thai switches to the sans face, because the installed mono face has no Thai
  glyphs; ASCII terminal output remains monospaced.
- Small labels use tracking and uppercase, not a second colour, so they remain visible in both
  themes and at high contrast.
- The text audit over all 45 fixture frames reports **no overlapping text runs**.

### Geometry

- Radius steps are 5/8/12/16/20 px. Windows, dialogs, cards and dock plates take the same
  family of corners instead of mixing hard rectangles and pills.
- Window controls are 20 px glyph-only buttons with 3 px gaps. The close, minimise and maximise
  marks are drawn as strokes; the old coloured traffic-light dots are gone.
- The window title is left-aligned after the controls and keeps the application's own name.
  Narrow client windows reserve room for the title before the state chip; a 250 px
  `simple-shm` window now renders `simple-shm`, not `Si...`.
- The Settings sidebar uses a 26 px row so all 19 pages fit without compressed labels. The
  selected row is an accent plate; the rest are quiet neutral rows.
- Default working windows are larger: Settings 980×620, Files 980×600, Browser 1000×620,
  Activity 920×620, generic clients 900×600. The redesign removed the need to work inside a
  720 px window.

### Chrome and navigation

- Every window uses the same Litric rail: monochrome glyph controls, left title, optional
  accent underline and a state chip only when a state exists.
- The dock is a monochrome instrument rail. The open application becomes the single accent
  plate; hover is a neutral lift, not a colour change. Launcher tiles use the same icon
  geometry as the dock.
- Mission Control, quick settings, the lock surface and desktop dialogs use the same material,
  radius, type and spacing rules.

## Page-level work

| Surface | Change |
|---|---|
| Desktop | Procedural ledger wallpaper with a measured margin, registration spine, horizontal rules, corner crop marks and right-edge tick marks. The old wave image is retained only as a test fixture for the legacy decoder. |
| Settings | `LITRIC / SETTINGS` header, page counter, full 19-row sidebar, larger page titles. Appearance rows, action rows and the Bluetooth dialog were rebuilt as ledger rows; only selection and the primary install/apply action carry accent. |
| Files | Empty details pane now explains what selecting a file will show. Detail rows use a consistent line step; location and size no longer collide with the actions. |
| Activity | Single-column composition at normal width. The old trend graph was removed from the meters' space rather than drawn over them; CPU, memory, uptime, network and disk are readable in order. Wide bodies still use the measured graph strip. |
| App Store / App Sandbox | Consistent section spacing, unclipped descriptions, install/remove actions aligned with their row state, and a dialog that follows the shared surface rules. |
| Games | The Game library page inherits the Settings header and ledger rows; the test game's Play action is the page's primary accent action. The fullscreen Game Mode overlay keeps a visible Stop control. |
| Browser / Terminal / Defender | Inherit the new window rail, palette, type scale, surface material and dock state. Browser reading and terminal Thai fallback were repaired rather than masked with mock-ups. |
| Lock | The account, direct unlock field and customize sheet use the new spacing and type. The full unlock instruction remains visible at the caption size rather than being elided. |

## Before / after

The [before/after sheet](../ui-review/2026-11-01/before-after.png) pairs eight representative
surfaces: desktop, Files, Settings, Browser, Activity, Control Centre, App Store and Games.
Every one of the 46 after screens has a corresponding before image at
`docs/ui-review/2026-11-01/before/`.

## Verification

- `cargo test --workspace`: **2,049 passed, 0 failed, 0 ignored**.
- Golden fingerprints: 45 fixture frames re-recorded after the redesign; the gate compares a
  tolerance, not byte identity.
- Text audit: **no overlapping text runs** across the 45 fixture frames.
- Screenshot set: 46 dark-theme frames, 45 from `frame_dump` and one (`39-linux-app`) from the
  real-socket e2e that installs, launches and maps `weston-simple-shm`.
- Light-theme evidence: five real 1280×800 frames (desktop, Files, Settings, Browser and
  Activity) rendered with `frame_dump --theme light`. The harness now attaches the same glass
  backdrop as `Compositor::new`, so light window frames no longer blend their own shadow.
- Light/dark tokens and high-contrast semantic tones are additionally covered by design/shell
  tests. High-contrast themes were not photographed.

## Limits

- This host has no `mkosi`, `qemu-system-x86_64`, physical display, GPU, audio or input device.
  No image was built, booted or scanned out; the frames prove `views::draw`, not a booted VM.
- The real application frame proves one genuine Wayland client and the shell's composition
  path. It does not prove Steam/Proton, gamescope, anti-cheat or general Linux application
  compatibility.
- `.deb`, `.rpm`, `.AppImage` and downloaded-package installation remain unimplemented; the
  installable format is still the local executable bundle.
- No RAM, frame-time or power measurement was taken for S92. The redesign changes drawing and
  layout, not the process model; no performance claim is made.
- The screenshot set is 1280×800. It does not establish behaviour on every HiDPI scale,
  fractional scale, rotation or multi-monitor arrangement.

## Rollback

Revert the S92 commit. The token palette, ledger wallpaper, window chrome, page layouts,
screenshots and docs return to the S91 state; the S91 Games work and earlier functionality
remain.
