# 2026-10-16 — the text stack told the truth, and the review loop reopened

**Scope:** two defects that made UI work unjudgeable — one `Style` drawing at two
different sizes depending on which text path is live, and a language being printed
by a typeface that has no glyphs for it — plus the tooling that makes a frame
possible on a host that is not the image.
**Mode:** real compilation, real tests and real frames on the Windows development
host. Nothing was booted, no image was built, no VM was run.

**Result vocabulary:** PASS (executed and succeeded) · FAIL (executed and found a
defect) · SKIP (deliberately not run, with a reason) · NOT VERIFIED (nothing that
ran can establish the claim).

---

## สรุปผู้บริหาร (Thai)

งาน UI ที่ผ่านมาถูกตัดสินด้วยการ**อ่านโค้ด** เพราะบนเครื่อง Windows ไม่มี
ฟอนต์ระบบที่ image ติดตั้ง `frame_dump` จึงปฏิเสธที่จะรัน — และเฟรมที่มีอยู่ก็
ถูกวาดด้วยฟอนต์สำรองที่**ตีความขนาดตัวอักษรผิด** (อ่าน `Style.size` เป็นความสูง
ตัวพิมพ์ใหญ่ แทนที่จะเป็นขนาดฟอนต์) ทำให้ข้อความใหญ่เกินจริง ~40% และคำสั้น ๆ
ถูกตัดเป็น `Docum…`, `Downlo…` ทั้งที่ควรพอดี

รอบนี้แก้สองอย่างนั้นพร้อมกัน แล้ว**เปิดลูปตรวจงานกลับมา**: `frame_dump --fonts DIR`
ให้ชี้ไปที่สำเนาฟอนต์ 6 ไฟล์ที่ image ติดตั้ง (ดาวน์โหลดไว้ใน `build/` ซึ่ง git ไม่เก็บ)
ทำให้เครื่อง dev สร้างเฟรมจริงได้อีกครั้ง และ **ไม่มีการพิมพ์ภาษาที่ฟอนต์วาดไม่ได้อีก**
— ถ้าไม่มีฟอนต์ไทย ระบบจะใช้ข้อความอังกฤษแทน ไม่ใช่ `▯▯▯▯`

เทสต์: `litric-shell` 349 passed / 0 failed · `litric-text` + `litric-design` ผ่านทั้งหมด
โดยไม่มีเทสต์ไหนถูกลดทอนหรือปิด

---

## 1. Defect A — one `Style`, two sizes

**What was wrong.** `litric-design`'s type scale is an em scale and says so:
`TypeStep::px()` is *"size in logical pixels"*
([lib.rs:414](../../src/litric-design/src/lib.rs#L414)), `is_large_text()` is tested
against WCAG's 18.66/24px thresholds, and Body is 15 with a 22px line height — a
1.47 ratio that only means anything for a font size.

The OpenType path honoured that: `style.size` went straight to HarfRust/fontdue as
the em size. The vector fallback did not:

```rust
let scale = size / CAP;   // CAP = 10.0 — reads `size` as a CAP HEIGHT
```

Three call sites ([font.rs:1692](../../src/litric-shell/src/font.rs#L1692),
[1719](../../src/litric-shell/src/font.rs#L1719),
[1764](../../src/litric-shell/src/font.rs#L1764)) drew every glyph `1 / 0.714`
— about 40% — too large. The shell's own centring arithmetic is the corroborating
witness: `cy + style.size * 0.36` appears throughout `ui.rs` and `views.rs`, and
0.36 is half of a 0.72-em cap. The vector path was the only thing in the
repository that thought `size` was a cap height.

**Why it mattered.** Every layout audit, every overflow check and every frame
produced on a machine without the installed faces described a UI the device never
shows. On the evidence frames it produced `Docum…`, `Downlo…`, `Setting…` and
`Termin…` inside an 80px label box that `Documents`, `Downloads`, `Settings` and
`Terminal` would have fitted.

**How it is fixed.** `Style::size` now means one thing, stated on the field, and
the fallback asks the live face what a cap is worth:

| Site | Change |
|---|---|
| `litric-text/src/lib.rs` | `DOCUMENTED_CAP_EM_RATIO` (0.7139), **measured** per face by rasterising `H` at 512px — flat top and bottom, so `H`'s ink height *is* the cap height |
| `font.rs` | `cap_px(em)` / `cap_em_ratio()`; the three `size / CAP` sites and the missing-glyph box now scale by the cap |
| `font.rs` | `Style::size` documented as the em, with the WCAG and centring evidence |

**The test that holds it.** `font::tests::a_capital_is_as_tall_as_the_tokens_say_it_is`
draws `H` at 12, 17 and 38px and asserts its ink height is the cap height the
tokens ask for — on *both* paths, because `cap_px` reads the live face either way.
It fails against the old code (which drew ~1.4× the cap) and passes now.

## 2. Defect B — printing a language the typeface cannot draw

**What was wrong.** `i18n` has a Thai catalog and `Preferences::default()` selects
Thai. The vector fallback has no Thai glyphs, and a missing glyph is drawn as a
box by design — the right signal for *one* character, the wrong signal for a whole
interface. With the Thai face absent and Thai selected, every label in the shell
became `▯▯▯▯` and the desktop read as a wireframe. The evidence frames show it:
`Setting…` next to four boxes, `Litric Defender` next to rows of them.

**How it is fixed.** `i18n::text` returns a translation only when the live stack
has a face for it, and says why when it does not:

| Site | Change |
|---|---|
| `i18n.rs` | `Language::renderable()`, `Language::label()`; `text()` falls back to English when `font::draws_thai()` is false |
| `font.rs` | `draws_thai()`, `cap_em_ratio()`, `face_report()` |
| `views.rs` | Settings → Appearance shows `Change language · Thai — typeface not installed`; Diagnostics names the **substituted faces** instead of rounding a partial install to "OpenType shaping · offline" |
| `i18n.rs` | `"Substituted face:"` added to the catalog so the degraded diagnostic row is not half-translated |

`i18n::tests::a_language_with_no_face_is_not_printed_as_boxes` asserts the
invariant directly: the translation happens **iff** a face can draw it. It passes
on a host with the face and on one without, instead of skipping on the machine the
developer is actually using.

## 3. Defect C — the font stack failed as a unit, and could not be seen

**What was wrong.** `TextSystem::load_system` loaded six faces and returned `Err`
if *any* one was missing, so one absent file degraded Latin body text and the
terminal's monospace face along with it. And `frame_dump` — correctly — refuses to
produce a frame from the fallback, which left the development host with no way to
produce a frame at all.

**How it is fixed.**

| Site | Change |
|---|---|
| `litric-text` | `FACE_FILES` — one table of file names per face, shared by both loaders; `load_roots` substitutes an absent face with the closest one present and records it in `Faces`; only a missing Latin face is fatal |
| `litric-text` | `load_dir(dir)` — the same six faces from one directory tree. The shipping shell never calls it; an application still cannot supply a font |
| `font.rs` | `initialize_from_dir(dir)` |
| `frame_dump` | `--fonts DIR`; usage printed on failure; a substituted install is announced before the frames are written |

Tests: `a_directory_with_no_latin_face_is_refused_with_the_paths_it_tried`,
`a_missing_script_face_is_substituted_rather_than_fatal`,
`the_documented_cap_ratio_matches_the_installed_face`.

## 4. Verification log

Every row was executed in this session, on the Windows development host.

| # | Command | Result |
|---|---|---|
| 1 | `cargo check --workspace --all-targets` (GNU) | **PASS**, exit 0 |
| 2 | `cargo test -p litric-shell --lib` | **PASS**, 349 passed / 0 failed |
| 3 | `cargo test -p litric-text -p litric-design` | **PASS**, 5 + 2 passed / 0 failed |
| 4 | `LITRIC_FONT_DIR=build/fonts cargo test -p litric-text` | **PASS** — the real-font assertions *ran* instead of skipping: `H` measured against `DOCUMENTED_CAP_EM_RATIO` within 0.02 |
| 5 | `frame_dump --fonts build/fonts build/frames-now` | **PASS**, 15 frames, Thai copy drawn through `NotoSansThai` |
| 6 | `frame_dump --fonts build/fonts-latin build/frames-latin` | **PASS**, 15 frames; loader reported `substituted face(s): Thai, Thai bold` |
| 7 | `cargo clippy --workspace --all-targets` | **FAIL (pre-existing)** — see §6 |

Baseline contrast: before this work `frame_dump` on this host exited 1 with
*"installed OpenType faces unavailable; vector fallback would make this visual
review unrepresentative"* and wrote no frames at all.

## 5. What is NOT verified

| Claim | Status | Why |
|---|---|---|
| The frames match the device pixel for pixel | **NOT VERIFIED** | they are set in Noto Sans and Noto Sans Thai as downloaded, not as the image's package build; the loader says so and the frame set is labelled |
| `NotoSansThai-Bold.ttf` is where the loader looks on the image | **NOT VERIFIED** | checked against the Arch `noto-fonts` file list (it is), but not against a booted image |
| The desktop looks right | **NOT VERIFIED** | the frames were decoded and read programmatically, not reviewed on a display |
| Boot, image build, frame budget | **SKIP** | no Linux runtime, no `mkosi`, no QEMU on this host |

## 6. Pre-existing conditions this session did NOT introduce, and did not fix

* `cargo clippy --workspace --all-targets` **fails** on work that was already
  uncommitted in the tree: `indexing may panic` in `shell.rs` (3164, 3179, 3194,
  4139), `views.rs` (1608–1613) and `apps.rs` (2246–2248), plus `integer division`
  in `lock.rs:735`. None of these lines are in this session's diff; the files it
  did not touch are the ones that carry them. It is reported here rather than
  quietly inherited.
* `desktop::render` is still a second, complete desktop composition with no
  callers. It was not deleted in this session: its test module is the only
  exercise of `glass`, the wallpaper and the light theme, so the fixture has to be
  re-pointed at `views::draw` first, and that is its own change.

## 7. Decisions and rejected alternatives

* **Measured the cap ratio, not hard-coded it.** Rejected: a single constant for
  all faces. The constant exists for the fallback's default, and a test checks it
  against the real face — but the running number comes from the font.
* **Kept `frame_dump`'s refusal to draw from the fallback.** Rejected: an
  `--allow-fallback` flag. The refusal is right — a frame in the wrong typeface has
  the wrong word widths. The fix is to make the right typeface available, not to
  lower the bar.
* **Gated the language instead of the glyph.** Rejected: teach the vector face
  Thai. A hand-drawn Thai face is a different project; the honest behaviour when
  the face is missing is to speak a language the machine can print.
* **Substituted faces instead of failing.** Rejected: keep all-or-nothing and
  report the missing package at boot. A desktop that cannot draw its own
  monospace text because a Thai bold file is absent is a worse failure than one
  that says which face it used instead.

## 8. แนวคิด / what the next round should do

1. **Look at the frames with eyes.** They exist now
   ([evidence/2026-10-16/frames/](../evidence/2026-10-16/frames/)); nobody has
   reviewed them on a display since the loop reopened.
2. **Wave 2 of [the plan](../design/2026-10-16-why-it-reads-as-a-mockup.md#4-sequencing):**
   a dock that is present when nothing is open, desktop-icon labels that fit a
   real name, a system bar whose centre names the focused application, and a
   wallpaper with depth.
3. **Graphs in Activity.** `grep spark|graph|chart` still returns nothing.
4. **Re-point `desktop.rs`'s tests at `views::draw` and delete the twin
   composition.** It is the last item of the design system's "no two render paths"
   rule that is still standing.
5. **A golden-frame diff.** Now that frames are byte-stable per host, the ten
   required screens can be compared run to run instead of re-reviewed.
