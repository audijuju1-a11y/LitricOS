# 08 — Accessibility, localisation, and input

## v1 accessibility contract

Every Litric-owned interface must be operable with keyboard alone, expose a
semantic accessibility tree, preserve focus across asynchronous updates, show
focus visibly, honour high contrast/text scale/reduced motion, and never make a
security confirmation mouse-only or colour-only.

## Required features

- Screen-reader bridge, magnification hooks, switch/alternate input hooks,
  sticky/slow/bounce keys, pointer controls, on-screen keyboard and captions
  controls.
- Semantic roles, names, values, states, descriptions, bounds, actions and
  notifications for compositor/shell widgets and native SDK widgets.
- Dynamic type scale, contrast themes, colour filters, motion reduction, audio
  visualisation and configurable timeout behaviour.
- Thai locale, Unicode grapheme-aware editing, Thai IME, Unicode normalization,
  locale-aware date/time/number/currency, left-to-right correctness, and a plan
  for RTL/complex scripts before claiming them supported.

## Design rules

Accessibility metadata is authored with the UI state, never guessed from pixels.
The accessibility process receives only semantic data necessary for its role;
password/secure fields are redacted. Input method text is treated as sensitive
when the client marks it so. Keyboard shortcut discovery must be localised and
remappable.

## Verification

Use automated tree snapshots, keyboard traversal replay, contrast checks,
reduced-motion screenshots, localisation pseudo-language expansion, Unicode
fuzz/property tests, IME composition tests, and manual assistive-technology
acceptance sessions. A critical user journey failing keyboard-only use blocks
release.

