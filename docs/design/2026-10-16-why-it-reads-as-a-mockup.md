# 2026-10-16 — ทำไม UI ยังอ่านเป็น "mockup" และแผนแก้ทั้งเครือข่าย

**สถานะ:** รีวิวจากโค้ดจริง + ภาพจริง (VM 1920×1080, frame dump 1280×800) + เอกสาร design ทั้งชุด
**ขอบเขต:** เชลล์ทั้งใบ (top bar, desktop, dock/launcher, lock, 8 แอป) — ไม่ใช่ไฟล์เดียว
**ผลลัพธ์ที่ต้องการ:** เดสก์ท็อปที่ "เป็นระบบจริง" แบบ macOS: ทุกอย่างมีชั้น (hierarchy), ไม่มีข้อความทับกัน, มีข้อมูลจริงและกราฟ, และมีคนออกแบบจริง ๆ อยู่เบื้องหลัง

---

## สรุปผู้บริหาร (Thai)

สิ่งที่ทำให้ UI อ่านเป็น mockup **ไม่ใช่เพราะสีไม่สวย** แต่เพราะ 5 อย่างนี้ และทั้ง 5 อย่างเป็น *defect ที่พิสูจน์ได้* ไม่ใช่รสนิยม:

1. **หน่วยของตัวอักษรสองเส้นทางไม่ตรงกัน** — token บอกว่า `TypeStep::px()` คือ *ขนาดฟอนต์ (em)* แต่เส้นทาง vector fallback ตีความเป็น *ความสูงตัวพิมพ์ใหญ่ (cap height)* ทำให้ข้อความเดียวกันใหญ่กว่า ~40% บนเครื่องที่ไม่มีฟอนต์ระบบ → คำสั้น ๆ ถูกตัดเป็น `Docum…`, `Downlo…`, `Setting…` ทั้งที่ความจริงควรพอดี
2. **ไม่มี dock** — ตอนพักหน้าจอ เหลือปุ่ม "Apps" ปุ่มเดียวกลางล่าง ฉากหลัง 90% เป็น wallpaper เปล่า ไม่มีอะไรบอกว่านี่คือระบบที่ใช้งานได้
3. **ศูนย์กลาง top bar ไร้ความหมาย** — แสดง `Workspace 1` ซึ่งไม่ใช่โมเดลในหัวผู้ใช้ ควรเป็นชื่อแอปที่โฟกัสอยู่
4. **ไม่มีกราฟเลยทั้งระบบ** — `grep spark|graph|chart` ใน Activity/monitor = 0 ผลลัพธ์ Activity มีมิเตอร์ 2 อัน ไม่มีประวัติเวลา
5. **มองไม่เห็นผลงาน** — `frame_dump` ปฏิเสธที่จะรันบนเครื่อง dev ที่ไม่มีฟอนต์ ⇒ ทุกการแก้ UI ปัจจุบันถูกตัดสินด้วยการอ่านโค้ด ไม่ใช่ด้วยการดูภาพ

แก้ 5 ข้อนี้ = งานหลายไฟล์พร้อมกัน (font.rs, i18n.rs, litric-text, views.rs, shell.rs, desktop.rs, activity.rs, examples/frame_dump.rs, image profile)

---

## 1. What I actually looked at

| Evidence | What it is |
|---|---|
| [qemu-latest-desktop.png](evidence/2026-10-15/qemu-latest-desktop.png) | A real 1920×1080 QEMU screendump — the device, not a mock renderer |
| [precision-final-desktop.png](evidence/2026-10-15/precision-final-desktop.png) | The newest full desktop frame (1280×800) |
| [precision-final-lock.png](evidence/2026-10-15/precision-final-lock.png) | The newest lock frame |
| [precision-current-frames/](evidence/2026-10-15/precision-current-frames/) | 15 frames, one per surface, generated before the current font work |
| `src/litric-shell/src/{views,desktop,font,ui,anim,i18n,settings,activity,monitor}.rs` | The drawing path, the component layer, the text stack, the motion integrator |
| `docs/DESIGN-SYSTEM-v3/*`, [precision-atelier.md](design/precision-atelier.md), [UI-INVENTORY.md](UI-INVENTORY.md) | The contract this UI is supposed to meet |

To read the frames without a display I wrote a small diagnostic that prints a
frame as luminance text (`build/pngtext.py`, git-ignored, development only). It
is how the clock, the top-bar words and the tofu boxes below were identified.

## 2. The five causes, with evidence

### A. Two text paths disagree about what a pixel size means — the biggest one

`litric-design`'s type scale is an **em** scale, and says so:

* `TypeStep::px()` — *"Size in logical pixels"* ([litric-design/src/lib.rs:414](src/litric-design/src/lib.rs#L414))
* `is_large_text()` is tested against WCAG's 18.66/24px thresholds, which are em sizes ([lib.rs:446](src/litric-design/src/lib.rs#L446))
* Body is 15 with a 22px line — a 1.47 ratio that only makes sense as a font size

The OpenType path honours that: `system.layout(text, face, style.size, …)` passes
`size` straight to HarfRust/fontdue, where it is the em size
([font.rs:1678](src/litric-shell/src/font.rs#L1678)).

The vector fallback does not: `let scale = size / CAP;` where `CAP = 10.0`
([font.rs:1692](src/litric-shell/src/font.rs#L1692),
[1719](src/litric-shell/src/font.rs#L1719),
[1764](src/litric-shell/src/font.rs#L1764)). One `Style` therefore renders **~40%
larger** on a machine without the installed faces — and every audit, every
overflow check and every frame produced on such a machine describes a UI the
device never shows. Caps at 0.714 em put the real size at 12 × 0.714 = 8.6px
against a fallback that draws 12px.

This is what produced the over-elision visible in the frames: `Documents`,
`Downloads`, `Settings`, `Terminal` all truncated to five letters (`Docum…`,
`Downlo…`, `Setting…`, `Termin…`) inside a 80px label box.

### B. The desktop at rest has nothing on it

`draw_dock` ([views.rs:2848](src/litric-shell/src/views.rs#L2848)) draws a
*compact "Apps" pill* whenever the dock's reveal spring is at rest; the dock
proper — apps, search, running indicators — only exists while it is open. On the
1280×800 frame, the entire lower half is wallpaper.

macOS and Windows 11 both answer "what is this machine for" from the resting
screen: a dock/taskbar with the user's apps in it. A single pill does not.

### C. The centre of the system bar names a thing the user does not have

`draw_top_bar` ([views.rs:356](src/litric-shell/src/views.rs#L356)) puts
workspace pips and `Workspace {n}` in the centre. Every desktop that has solved
this puts the **focused application and its menus** there; the focused window's
title is already in its own title bar and in the switcher. The middle cluster is
currently a number the user never asked for.

### D. There is no data visualisation anywhere in the system

`grep -n "spark\|graph\|plot\|series\|chart" activity.rs monitor.rs` → **no
matches**. The Activity window draws two meters
([06-APP-REDESIGN.md §2](DESIGN-SYSTEM-v3/06-APP-REDESIGN.md) already asks for
live graphs with history and a per-core grid). A system monitor with no history
is a screenshot of a system monitor.

### E. Nothing can be looked at

`frame_dump` now refuses to run when the OpenType faces are absent, on purpose
and for a good reason ([frame_dump.rs:400](src/litric-shell/examples/frame_dump.rs#L400)):
a fallback-rendered frame is not evidence about the device. The consequence is
that on the Windows development host **no frame can be produced at all**, so
every UI change since that check landed has been reviewed as an argument.

Add to that: `desktop::render` ([desktop.rs:323](src/litric-shell/src/desktop.rs#L323))
is a second, complete desktop composition — menubar, window, switcher, dock,
icons, ~600 lines — with **zero callers**. The design system forbids exactly
this ("ห้ามสร้างเส้นทาง render คู่"), and it is the file a newcomer edits by
mistake.

### F. The font stack fails as a unit

`TextSystem::load_system` loads six faces and returns `Err` if **any** one is
missing ([litric-text/src/lib.rs:100](src/litric-text/src/lib.rs#L100)). One
missing `NotoSansThai-Bold.ttf` degrades Latin body text to the vector fallback
as well — including the mono face the terminal needs. A per-face capability
model would keep everything that can be drawn.

Two smaller inconsistencies worth recording: the desktop profile installs
`inter-font` ([desktop.conf:51](image/mkosi.profiles/desktop.conf#L51)) but
nothing ever loads it (the stack loads Noto Sans), and a missing glyph is drawn
as a box by design ([font.rs:1776](src/litric-shell/src/font.rs#L1776)) — which
is honest for *one* character and catastrophic for *one language*.

## 3. The network of edits (what "fix the UI" actually touches)

```
litric-design ── tokens (unchanged; the scale was right all along)
      │
litric-text ──── per-face loading, cap-height query, dev font directory
      │
litric-shell/font.rs ── em/cap unification, draw+measure must agree
      │
litric-shell/i18n.rs ── language the live stack can actually draw
      │
litric-shell/views.rs ── top bar, dock at rest, desktop icons, app surfaces
litric-shell/shell.rs ── hotspots + state the above registers
litric-shell/desktop.rs ── wallpaper; delete the dead twin composition
litric-shell/activity.rs + monitor.rs ── Graphs, with history
examples/frame_dump.rs ── the review loop that makes all of it visible
```

## 4. Sequencing

**Wave 0 — see it.** `--fonts DIR` for `frame_dump`, with copies of the six
installed faces in the git-ignored `build/fonts/`, so a frame can be produced on
a host that is not the image. The dead `desktop::render` composition stays for
now: removing it is a one-block deletion, but its own test module uses it as the
only exercise of `glass`, the wallpaper and the light theme, so the fixture has
to be re-pointed at `views::draw` first rather than deleted with it.

**Wave 1 — the text stack stops lying.** `Style.size` means one thing on both
paths; the fallback's cap height is checked against the constants; Thai is not
selected when no live face can draw it; per-face loading so one missing file
degrades one face, not the desktop.

*Done in this session — see [the audit](audit/2026-10-16-text-stack-and-review-loop.md).*

**Wave 2 — the desktop at rest.** A dock that is present when nothing is open
(icons, running indicators, tooltips), desktop icons whose labels fit, a top bar
whose centre names the focused app, and a wallpaper with depth that does not
compete with text.

**Wave 3 — the apps measure things.** Activity gets histories and graphs
(CPU per core, memory composition, disk and network rates) drawn from
`SystemFacts`; empty/loading/error states land in Files, Settings and Browser
through one helper.

**Wave 4 — the locks and gates.** Golden frames for the 10 required screens,
the design lint extended to the new rules (one render path, type roles present,
no language without a face), and the frame budget re-measured.

## 5. What this session does

Wave 0 and Wave 1. They are the prerequisite for judging anything else: right
now the frames cannot be produced, and the frames that do exist are 40% the wrong
size. Nothing in Wave 2–4 can be honestly reviewed until those two are done.
