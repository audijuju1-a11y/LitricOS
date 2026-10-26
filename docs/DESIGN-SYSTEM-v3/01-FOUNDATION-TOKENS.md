# 01 — FOUNDATION & TOKENS (ผูกกับ MASTER §4-§14)

> กติกา: ทุกค่าที่นี่ต้องอ้าง token เดิมที่มีอยู่จริงใน `litric-design` ก่อนเสมอ — สร้างใหม่เฉพาะช่องที่ไม่มี และต้องผ่าน contrast gate ทดแทนการเดา

## 1. Token Architecture 3 ชั้น (MASTER §4)

```
L1 Primitive   → สิ่งที่ litric-design มี: TypeStep(8), Space, Radius, Motion(4), Rgb
L2 Semantic    → Theme struct: text/text_dim/danger/success/accent + surface(t) 7 ระดับ + glass adaptive
L3 Component   → ยังไม่มีระบบ — สร้าง `component_style()` ใน ui.rs (ดู §2 ด้านล่าง)
```

**กฎเหล็ก:** component ห้ามเรียก `Rgb::hex` ตรง ๆ ยกเว้น wallpaper/demo — lint จับ (§04)

## 2. สิ่งที่ต้องเพิ่มใน L3 (Component Tokens) — ระบุเป็นโค้ดจริง

```rust
// ui.rs — ต่อจาก style_label เดิม (ไม่แก้ของเดิม)
pub struct ControlSpec {           // ทุก control ที่กดได้ต้องผ่าน struct นี้
    pub height_px: f32,            // 28 (sm) / 32 (md) / 40 (lg) × scale
    pub pad_x: f32,                // 12 × scale
    pub gap: f32,                  // 8 × scale
    pub radius: Radius,            // จาก family เดิม
    pub min_hit: f32,              // 24 × scale — hit target ต่ำสุด (a11y)
}
pub const CONTROL_SM: ControlSpec = ...;  // toolbar button, list row control
pub const CONTROL_MD: ControlSpec = ...;  // button, input, segmented
pub const CONTROL_LG: ControlSpec = ...;  // dialog primary action
```

## 3. Typography Roles (แก้จุดอ่อน B จาก audit — bold 7 ครั้งทั้งระบบ)

| Role | Token | น้ำหนัก | ใช้ที่ |
|---|---|---|---|
| `page_title` | Title1 (30) | **bold** | หัว Settings แต่ละหน้า, หัวแอป |
| `view_title` | Title2 (24) | **bold** | ชื่อ view ใน content area |
| `section` | Title3 (20) | **bold** | หัวกลุ่ม card/section |
| `item_primary` | Headline (17) | bold | ชื่อไฟล์, ชื่อ app, หัวข้อ row |
| `item_secondary` | Body (15) | regular | รายละเอียด row, ค่าที่ผู้ใช้แก้ได้ |
| `meta` | Caption (12) | regular, `text_dim` | timestamp, ขนาดไฟล์, hint |
| `label_caps` | Caption (12) | bold + tracked 1.1 | ป้ายกลุ่ม (มีอยู่แล้ว = `style_label`) |

**Audit rule ใหม่:** ทุก view ต้องมี ≥ 2 roles และจำนวน `Style::new` ต่อ 1 `style_bold` ใน view เดียวต้อง ≤ 6 (ไม่งั้น hierarchy แบนอีกครั้ง) — ใส่ใน design lint

## 4. Spacing (MASTER §5) — ใช้ `Space` เดิม + semantic alias

มีอยู่: `Space::Xs..Space::Xl` (4..32) — **เพิ่มแค่ mapping** ใน `ui.rs`:
`pad_control=Space::Md, pad_card=Space::Lg, gap_list=Space::Sm, section_gap=Space::Xl`
ห้ามค่า px ลอยนอก mapping เว้นแต่ optical exception (บันทึกใน comment `// optical:`)

## 5. Radius family (MASTER §6) — ตรวจแล้วมีใน `litric-design::Radius`
ใช้เดิม: control=6-8, card=12, panel=14-18, window=18-24 — **ไม่เพิ่มค่าใหม่เด็ดขาด** จนกว่าจะพบ use case จริง + บันทึกเหตุผล

## 6. Elevation (MASTER §13) — สิ่งเดียวที่ขาดจริง

สร้าง `elevation()` ใน `ui.rs` — shadow เบาตามลำดับชั้น ไม่ใช่ shadow หนักแบบเว็บ:

| Level | ใช้ที่ | Shadow (dy, blur, alpha) | เพิ่มเติม |
|---|---|---|---|
| E0 | พื้นหลัง view | ไม่มี | border `divider` เดิม |
| E1 | card ใน view | (0, 2, 0.10) | — |
| E2 | panel/toolbar float | (0, 4, 0.14) | — |
| E3 | context menu/popover | (0, 8, 0.18) | border `surface-elevated` |
| E4 | window ที่ focus | (0, 12, 0.22) | **+ border สว่างกว่า unfocused 1 ขั้น** |
| E4u | window unfocused | (0, 6, 0.12) | + title text `text_dim` |
| E5 | modal/system overlay | (0, 16, 0.28) | scrim หลัง modal |

**ผูกกับของจริง:** ปัจจุบัน window focus แยกด้วย title สีเดียว — ต้องยกเป็น E4/E4u คู่ (แก้ที่ view draw ของ window chrome)

## 7. Material (MASTER §12) — ใช้ `surface(t)` + `glass` เดิม
Glass **ห้าม default** — จำกัดที่: system bar, Control Center, Spotlight panel, notification banner (semantic M5/M6 เท่านั้น) ตามที่ `desktop::glass` adaptive ทำอยู่

## 8. Motion (MASTER §20-§22) — token มีแล้ว งานคือ "ต่อไฟ"
| Context | Token | ที่มา |
|---|---|---|
| hover/press ของ control | `Motion::INSTANT` | litric-design 580 |
| เมนู/popover เปิด-ปิด | `Motion::FAST` | 585 |
| หน้าต่าง snap/ยก focus, panel ดันขึ้น | `Motion::NORMAL` | 585 |
| workspace/launchpad พับ | `Motion::SLOW` | 585 |
| Reduce Motion เปิด | `Motion::reduced()` → 1 frame | 640 |

**กติกา implement:** integrator เดียวใน shell (`anim.rs` ใหม่): ทุกค่าที่ animate เก็บเป็น `f32` ปัจจุบัน + เป้าหมาย + `Motion` และ step ด้วย semi-implicit Euler ที่ dt ของ frame — interruptible ตาม ADR-0009, settle ≤ 500ms

## 9. Icon (MASTER §14) — กฎ 4 ข้อ + test
- grid 24 logical, stroke = `WEIGHT_REGULAR` เสมอ (ไม่มี icon หนาบางปนกัน)
- sizes ใช้ 16/20/24/32 เท่านั้น
- optical test ใหม่ใน `icons.rs` tests: bounding box ของทุก icon ต้องเบี่ยงจากกึ่งกลาง ≤ 1px และ ink coverage คู่ที่ "ควรเท่ากัน" (เช่น arrow 4 ทิศ) ต่างกัน ≤ 8%

## 10. Color — ห้ามแตะ palette ใหม่
`Theme` มี accent/semantic ครบ + AA gate รันอยู่ งานนี้ = **discipline ไม่ใช่แต่ง**: lint จับ `Rgb::hex` นอก wallpaper/demo, และทุกสีที่สื่อสถานะต้องมี non-color cue คู่ (icon/รูปทรง/ข้อความ) ตาม MASTER §42

**เพิ่ม 2026-10-25 — Accent slot:** `Accent` (default/sky/violet/teal/amber/rose) เป็น
*ช่อง* ไม่ใช่ hex: แต่ละ theme ถือค่าของช่องนั้นเอง (`Theme::with_accent`) เพราะฟ้าเดียวกัน
ที่อ่านได้บนพื้นเกือบดำไม่ใช่ฟ้าเดียวกันที่อ่านได้บนพื้นขาว. `with_accent` แตะเฉพาะคู่
`accent`/`on_accent` — พื้น, ตัวอักษร และสีสถานะไม่ขยับ และ high-contrast theme คืนค่าเดิม
เพราะ accent ที่นั่นเป็นส่วนหนึ่งของสัญญา AAA. ด่านคือ
`every_accent_keeps_text_legible_on_its_own_fill` + `every_accent_is_readable_as_text_on_the_theme_backgrounds`
ใน `src/litric-design/src/lib.rs` (40 tests ทั้ง crate).

## เกณฑ์ตรวจเฟส Foundation
- [ ] `cargo test -p litric-design` contrast gate ยังเขียว
- [ ] ไม่มี px ลอยใหม่นอก mapping (grep audit)
- [ ] `ControlSpec` ถูกใช้โดย button/input อย่างน้อย 2 จุดก่อนขยาย
- [ ] elevation ปรากฏจริงใน window focus/unfocus diff ที่วัดได้ (e2e pixel test)
