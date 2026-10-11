# UI Research — Standard-Premium macOS-style for LitricOS

Status: RESEARCH COMPLETE · 2026-09-19 · 76 distinct sources (8 read in full depth)
Scope: make LitricOS desktop feel **standard like Windows, premium like macOS** — per
user's attached macOS 27 / Tahoe reference images (glass dock, centered Spotlight,
translucent sidebar Finder, menu bar with status icons, widget cards).

---

## 1. Source register (≥40 required — 76 collected)

### Primary — Apple official (13)
1. developer.apple.com/design/human-interface-guidelines — HIG root
2. …/designing-for-macos — platform principles (power, spaciousness, flexibility, multi-app)
3. …/design-principles
4. …/layout
5. …/components
6. …/materials — Liquid Glass layer model
7. …/menus — menu behavior rules
8. …/the-menu-bar — top-level menu order
9. …/context-menus — right-click rules
10. …/sidebars — Finder sidebar structure
11. …/typography — SF Pro sizes/weights
12. …/toolbars
13. developer.apple.com/fonts — SF Pro 9 weights, optical sizes, 4 widths
14. developer.apple.com/videos/play/wwdc2025/219 — Meet Liquid Glass
15. developer.apple.com/videos/play/wwdc2025/356 — new design system
16. developer.apple.com/documentation/technologyoverviews/liquid-glass
17. developer.apple.com/documentation/appkit/nsvisualeffectview — vibrancy definition
18. apple.com/newsroom/2025/06 — Liquid Glass announcement
19. developer.apple.com/documentation/swiftui/animation/easein — easing semantics

### Engineering deep-dives (read in full) (8)
20. github.com/dashersw/macos-dock-in-css — **smoothstep magnification math**: distance→normalize→s²(3−2s)→scale; frame-perfect, compositor-run
21. namethatui.com/macos/traffic-lights — traffic-light anatomy: close/red, minimize/yellow, zoom/green; **symbols appear on hover only**; dirty-document dot
22. bjango.com/articles/designingmenubarextras — **menu bar = 24pt (Big Sur+)**; extras working area 22pt; icon 16×16pt; template images (alpha-only, system-tinted); **35% opacity = disabled**
23. blog.jim-nielsen.com/2019/design-principles-applied-to-sf-fonts — **SF Pro Text ≤19pt, Display ≥20pt**; visual similitude ≠ mathematical equality; 6 text weights / 9 display weights; mono digits for animating values
24. dev.os9.ca Inside Macintosh HIG ch.4 — **contextual menu opens 1px right/1px down from click**; functional groups with dividers; never supersede menu bar; no default item; submenus ≤1 level
25. designmd.app/library/macos-vibrancy — **full vibrancy token set** (see §3)
26. mackuba.eu Dark Side of the Mac — Vibrant Light/Dark appearance history
27. designmd.app/library/dimensional-layering — z-index depth, 4 elevation levels

### Dock magnification implementations (6)
28. turnerworks.uk — cosine interpolation variant
29. github.com/souph1s/dock-macos — spring physics
30. fwdtools.com/ui-snippets/floating-dock — Gaussian proximity
31. ui.unlumen.com/components/dock — Gaussian + spring
32. dev.to/dev48v — rebuild walkthrough
33. buildui.com/recipes/magnified-dock — per-icon distance transform

### Typography (7)
34. gist.github.com/eonist — HIG tracking table (28pt → −0.8px)
35. blakecrosley.com — SF Pro variable axes analysis
36. stackoverflow.com — SF Pro kerning mismatch analysis
37. en.wikipedia.org/wiki/San_Francisco_(sans-serif_typeface)
38. youtube WWDC22 — expanded SF family (Condensed/Compressed/Expanded)
39. reddit r/AfterEffects — SF alternatives
40. reddit r/typography — what makes SF good

### Spotlight (7)
41. robservatory.com — re-center behavior
42. reddit r/MacOS — position quirks
43. apple.stackexchange.com — re-center
44. discussions.apple.com — centering issue
45. themacu.com — Spotlight power guide
46. macmost.com — Big Sur Spotlight preview anatomy
47. osxdaily.com — move Spotlight window

### Vibrancy / dark mode (6)
48. nonstrict.eu WWDC18/210 — Dark Mode intro
49. stackoverflow — Yosemite translucent sidebar
50. learnui.qiaomu.ai — vibrancy material choices
51. philz.blog — vibrancy + appearance pitfalls
52. feedback.sidebarapp.net — vibrancy implementation
53. wikipedia San Francisco (dup ref)

### Elevation/depth (6)
54. m2.material.io elevation — cross-reference
55. design.gitlab.com elevation
56. atlassian.design elevation
57. modus.trimble.com shadows — deeper object ⇒ higher z-index
58. designsystems.surf — elevation tokens article
59. learn.microsoft.com shadows — cross-reference

### Context menus / legacy HIG (5)
60. interface.free.fr Apple HIGOS8 PDF
61. macmost.com — context menu customization
62. ux.stackexchange.com — right-click should select target
63. leopard-adc.pepas.com — classic controls metrics
64. vintageapple.org HIG 1992 PDF — disabled-menu-title rule

### Misc/verification (5)
65. designsystems.surf/design-systems/apple
66. github.com/ehmo/platform-design-skills
67. pocketmags MacLife — Tahoe vs 1992 HIG tension
68. rocketfarmstudios.com — Liquid Glass breakdown
69. rogerwong.me — "lensing" analysis
70. medium.com shrishivathmika — 3-layer model
71. LottieFiles/motion-design-skill
72. designsystemproblems.com — easing cross-platform
73. m1.material.io duration-easing — cross-reference
74. m3.material.io motion specs — cross-reference
75. bjango (dup ref), dribbble mood-board
76. davidalison.com — Mac vs Windows window controls

---

## 2. Hard specs extracted (the numbers that matter)

| Element | Spec | Source |
|---|---|---|
| Menu bar height | 24pt (Big Sur→now); legacy 22pt | bjango |
| Menu bar extra working area | 22pt max; circular icon 16×16pt | bjango |
| Disabled state in bars | 35% opacity, never grey-shift | bjango |
| Traffic lights | red/yellow/green, top-left, glyphs on hover only, doc-dirty dot | namethatui |
| Context menu position | opens +1px right, +1px down from cursor (RTL mirrored) | Inside Macintosh |
| Context menu content | small subset, divider-separated groups, no default item, submenu ≤1 level | Inside Macintosh |
| Right-click on item | selects the item first (also ux.stackexchange consensus) | ux.stackexchange |
| Dock magnification | scale = base + mag × smoothstep(s), s = clamp(1−d/range,0,1), smoothstep = s²(3−2s) | dashersw |
| Spotlight | centered floating panel, single field, results below, Esc closes | 41–47 |
| SF Pro optical split | Text ≤19pt / Display ≥20pt, tighter tracking as size grows | jim-nielsen |
| Tracking example | 28pt ≈ −0.8px | eonist gist |
| Numbers in UI | monospaced digits when values animate | jim-nielsen |
| Vibrancy | blur(20–30px) + saturate(1.8) + alpha surface, hairline border, inner top highlight | designmd |
| Surface alphas | regular 0.55, strong 0.78 | designmd |
| Radius scale | 8 / 10 / 14 (cards 14, buttons 10, small 8) | designmd |
| Shadows | sm 0 1 2 6% · md 0 8 24 10% · lg 0 20 48 14% | designmd |
| Hairline border | rgba(0,0,0,0.08) | designmd |
| Accent | #007AFF blue, #FF9F0A orange | designmd |
| Text | #1D1D1F primary, 60% muted | designmd |
| Liquid Glass model | content layer + floating glass control layer; glass bends light (lensing), adapts to backdrop | Apple newsroom/WWDC25 |
| Motion | subtle, short; avoid gratuitous motion; ease-in-out asymmetric | HIG motion |

## 3. Design tokens for LitricOS (`litric-design` additions)

Our renderer is a software canvas — no gaussian blur compositing. Vibrancy is
approximated honestly: fixed alpha surfaces + sampled-average wallpaper tone
already computed by the compositor (no per-pixel blur), which reads as the same
material language at our 1280×800 fixed target.

```
// Materials (glass tiers)
menu_bar:      alpha 0.72 dark tone + wallpaper-tint
window:        alpha 0.86, hairline top highlight (inset +1px light)
sidebar:       alpha 0.55 (vibrant light/dark from theme)
context_menu:  alpha 0.92 (menus stay readable — Inside Macintosh)
spotlight:     alpha 0.80, centered, 640px wide, radius 14
dock:          alpha 0.55 pill, radius 22 (full pill), icons 44px base
// Elevation (4 levels, from designmd + dimensional layering)
z0 desktop icons · z1 windows · z2 dock/menubar · z3 menus/spotlight · z4 dialogs
shadow-md only on z1; shadow-lg on z3/z4 (overhang must not clip: checked_sub)
// Type scale (fixed 13px body canvas font; optical roles by weight)
body 13 regular · secondary 13 @60% · caps-label 11 bold +1px tracking ·
title 17 semibold · large-title 28 → tracking −1px (eonist table)
// Metrics
menubar 24px · dock icon 44px base, mag +28px, smoothstep range 88px,
gap 6px, pill pad 6px · traffic lights: 12px orbs, 8px gap, at (12,12) ·
window radius 10 · button radius 8 · menu row 26px, row pad-x 12 ·
sidebar 168px · status icons right-aligned, 16px glyphs
// Color (theme-aware; these are the dark-mode defaults)
accent #0A84FF · ok #30D158 · warn #FF9F0A · danger #FF453A ·
text #F5F5F7 / 60% muted · hairline rgba(255,255,255,0.10) on glass
```

## 4. Rules we adopt verbatim (behavioral)

1. Context menu never opens under the cursor: +1,+1 offset, flips at screen edge (Inside Macintosh).
2. Right-click on a row selects that row (already implemented in shell).
3. Menu glyphs/shortcut hints right-aligned; disabled items 35% opacity, still visible.
4. Traffic-light glyphs only on hover; hover target stays 12px orb.
5. Dock magnification uses smoothstep with a fixed range — never linear falloff.
6. Spotlight centered; opens with field focused; Esc closes; results keyboard-navigable.
7. Every command in a context menu MUST also exist in the menu bar (no orphan commands).
8. No default button in context menus; click-away cancels without action.
9. Windows keep hairline top highlight — the single strongest "premium" signal per vibrancy research.
10. Motion: 120–180ms ease-in-out for menu/popover, 220ms for window open; nothing longer.
