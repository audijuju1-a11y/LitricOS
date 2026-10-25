# From the research corpus to the UI — what is built, what is next

The corpus lives in this repository: `Research-Deep-all/` — 15 files, 37 822
lines, added by the owner from `audijuju1-a11y/LitricOS`. This document is the
ledger between it and the shell: which research facts are implemented, where the
code is, and what the corpus says to build next. A claim in here is a claim about
`src/litric-shell`, with a test or a frame named next to it.

## 1. What the corpus is

| File | What it gives us |
|---|---|
| `LitricOS UI UX Research Part 5  macOS 27 System Settings.md` | 38 sidebar panes (MD.01.p01–p38), per-pane controls, and the URLs Apple documents them at |
| `LitricOS UI UX Research Part 6  macOS 27 Desktop Model.md` | Menu bar, Dock, Spotlight, Mission Control, window model |
| `LitricOS UI UX Research Part 2 Windows 11 26H2 Settings.md` | Settings shell behaviour: search (Ctrl+E), keyboard navigation, Home cards, per-page catalogue WB.01–WB.13 |
| `LitricOS UI UX Research Part 3  Windows 11 Shell and File Explorer.md` | Shell surfaces and Explorer model |
| `LitricOS UI UX Research Part 4  Fluent and WinUI Design System.md` | Fluent tokens: type ramp, elevation, control metrics |
| `LitricOS UI UX Research Part1.md` | Index, method and status rules |
| `macOS Design Token Register.csv` (1 806 rows) | Token-by-token register |
| `Fluent Token Register.csv` (3 554 rows) | Fluent token register |
| `macOS System Settings URL Register.csv`, `Windows ms-settings URI Register.csv` | Deep-link maps: every pane reachable by URL |
| `macOS Keyboard Shortcut Register.csv`, `Windows Keyboard Shortcut Register.csv` | Shortcut registers, platform by platform |
| `LitricOS Coverage Checklist.csv`, `LitricOS Source Register.csv` | Coverage against the shells being studied |

## 2. Implemented from it

| Research fact | What the shell does now | Where |
|---|---|---|
| macOS Accessibility has an **Increase contrast** switch (Part 5 §13.10) | `Preferences.contrast` → outlines blended 45% toward the body colour, secondary text 50% | `src/litric-shell/src/settings.rs` |
| macOS Appearance has an **accent colour** choice (Part 5 §13.12) | `settings::Accent`: System / Blue / Graphite / Green / Orange / Pink / Purple, applied to `palette.accent` with the better of white/near-black on top | `settings.rs`, `views.rs` (`settings_swatches`) |
| Settings pages are a **list of labelled rows with a control on the trailing edge**, and the row — not the 22 px switch — is the target | `ui::switch`, `views::settings_switch`; the whole 44 px row is the hotspot | `ui.rs`, `views.rs` |
| **State is never colour alone** (governance rule, Part 4 §control states) | The switch moves the knob *and* changes the track; the chosen swatch gets a ring | `ui.rs` |
| Settings must read as one interface in the user's language | Thai catalog entries for every new label, and the two composed labels ("Change theme · …") now translate their static half | `i18n.rs` |
| Settings **search** — Ctrl+E focuses the box, arrows browse the results, Enter opens one (Part 2 WB.00.F04, WB.00.F08; Part 5 §13.1) | Search field at the head of the sidebar; results replace the category list while a query is typed; Ctrl+E reaches it from the desktop and opens Settings | `settings.rs` (`search`, `Hit`), `shell.rs` (`settings_key`) |
| **Home page with cards** (Part 2 WB.00.F12) | `Page::Home` is the first page and the first sidebar entry: four cards, each showing its page's *current value* (theme · accent, motion · contrast, resolution, network state) | `settings.rs` (`HOME_CARDS`), `views.rs` (`settings_card_value`) |
| **Back control** (Part 5 §13.1; Backspace per Part 2 WB.00.F03) | `Model::history` (bounded to 16), a back control drawn only when there is somewhere to go, and Backspace-with-empty-query in the search field | `settings.rs` (`go`, `back`) |

Evidence: ten tests in `settings.rs` (default accent leaves the theme
byte-identical, accent replaces the accent and keeps text ≥4.5:1, contrast is two
blends, tokens round-trip, search finds a setting by its own name and its page,
every page is findable by its title, back walks what was visited, the history is
bounded, the selection wraps) and four in `shell.rs` (a swatch changes the accent
the whole shell draws with and emits a persistence effect; the contrast switch
changes hairlines and refuses a stale index; Ctrl+E → type → Down/Up → Enter →
Backspace → Escape driven through `handle`, the keyboard's own path; home cards
and the back control are drawn *and* hittable, and the back control exists only
after a step). Rendered frames: `artifacts/settings-home.png`,
`artifacts/settings-search.png`, `artifacts/settings-appearance.png`,
`artifacts/settings-accessibility.png`, `artifacts/settings-accent-pink.png`.

## 3. What the corpus says to build next, in order

### 3.1 Settings shell

1. **More panes**, each backed by facts the daemon already reports before any
   fake control is invented: Notifications, Focus, Wallpaper, Desktop & Dock,
   Menu Bar, Spotlight, Lock Screen, Users & Groups (MD.01.p14, p16, p18, p20,
   p22, p24, p28). The sidebar now has room for fourteen rows above its bottom
   edge at 26 px; a fifteenth needs a scroll, so the next pane should arrive with
   one.
2. **Deep links** — the two URI registers turn every pane into a
   `litric://settings/<pane>` route, and a search result into
   `<pane>#<row>`.
3. **Search that reaches rows, not only pages.** Today a hit is a page and one of
   its keywords; the deep link above is what lets Enter land on the *switch* as
   well. The keyword tables in `Page::keywords` are the index either way.
4. **Recents and suggestions** — Part 5 §13.1 says search covers settings,
   suggestions and recents; the model already keeps a history of visited pages,
   so recents is a list, not a new mechanism.

### 3.2 Motion and desktop model (Part 6)

6. Dock magnification parameters and the menu-bar model already match
   `docs/UI-RESEARCH-MACOS.md`; the corpus adds the **menu bar extras** pane and
   the **Dock** pane as settings surfaces.
7. Panel **exit transitions** are still missing (`docs/bugs/BUG-0025`,
   `BUG-0026`): the panels travel in and are erased out.

### 3.3 Foundations

8. Type ramp and elevation from `Fluent Token Register.csv` versus
   `macOS Design Token Register.csv`: where the two disagree, this tree's tokens
   already pick one number and say why (`docs/DESIGN-SYSTEM-v3`). A systematic
   pass over the registers is the next foundation task, not part of it.
9. The design lint's remaining exception (`views.rs`: the dock tile's squircle
   radius is a fraction of its own side, not a step on the radius family) needs a
   decision — a token, or an explicit shape rule in `ui.rs` — before the rule can
   be green (`cargo test -p litric-shell --test design_lint`).

## 4. How to check this document is still true

```text
cargo test -p litric-shell --all-targets          # 440 lib + 23 integration
cargo run --release -p litric-shell --example render_bench   # panel frames
cargo run --release -p litric-shell --example frame_dump -- --fonts DIR OUT
python3 ci/check-golden-frames.py --frames OUT     # golden set, 2% / 0.5 level
```

`05-settings` and `03-windows-overlap` are the two scenes whose fingerprints
differ from HEAD: both draw the Settings window, and its pane has new pixels (the
accent row, the translated labels) and a new first page (Home). The other 22
scenes are byte-identical. The golden set in `docs/evidence/golden` was recorded
before this work and on a host with the real faces, so `ci/check-golden-frames.py`
fails here for every frame that draws a substituted font — identical failure at
HEAD, which is how it was established that this change is not the cause.
