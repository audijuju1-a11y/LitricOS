# 04 — QA, DESIGN LINT, GOVERNANCE (MASTER §51-§59, §63-§64)

> หลักการ: "ห้ามถือว่างานเสร็จเพียงเพราะภาพสวย" แปลเป็นเครื่องจักร — เรามีต้นแบบแล้ว 2 อย่าง (`OverflowAudit`, golden-frame e2e) หัวข้อนี้ขยายเป็นระบบ lint + gate ที่รันได้จริงทุก build

## 1. Design Lint — `cargo test -p litric-shell design_lint`

Lint ทำงานบน **โค้ดจริง** (ไม่ใช่ภาพ) — static check ใน test ใหม่ `design_lint.rs`:

| กฎ | วิธีตรวจเชิงกล | FAIL ถ้า |
|---|---|---|
| สีนอก token | scan AST/regex `Rgb::hex(0x...)` ใน litric-shell ยกเว้น wallpaper.rs + demo + tests | พบนอก whitelist |
| px ลอยนอก scale | scan literal `f32` เข้า `Space`/`text_px` ที่ไม่ใช่ค่า scale × scale | พบ (ยกเว้น comment `// optical:`) |
| radius นอก family | `Radius::` ใช้เฉพาะ variant ที่ประกาศ | พบ custom f32 |
| bold starvation | นับ `Style::new` : `style_bold` ต่อ view fn | > 6:1 |
| control ไม่มี disabled | draw fn ของ control ไม่มี branch disabled | พบ |
| focus ring ขาด | ทุก interactable ต้องผ่าน `focus_ring()` helper | พบวาดเอง |
| animation ไม่มี token | `anim()` เรียกด้วย `Motion` enum เท่านั้น ห้าม magic stiffness | พบค่าลอย |
| text ไม่ผ่าน audit | ทุก view ใหม่ต้องมี `OverflowAudit` ใน e2e | view ไม่มี |

กระบวนการ: **REPORT → FIX → RECHECK** — lint fail = CI/build fail (ไม่มีเส้นโค้ง)

## 2. Golden Screens (MASTER §52) — 10 ใบบังคับ

1. Desktop idle 2. Desktop + 3 windows overlap (focus 3 ระดับ) 3. Files (grid, มีไฟล์) 4. Files (empty + edge state) 5. Settings/Appearance 6. Control Center เปิด 7. Seek เปิดมีผลลัพธ์ 8. Notification Center 9. Dialog (danger ลบไฟล์) 10. Lock screen

- ตรวจด้วย e2e golden-frame เดิม + **pixel-diff threshold**: แก้ UI แล้ว diff เกิน 2% ต้องอัปเดต golden พร้อมเหตุผลใน commit
- ทุก golden เก็บภาพจริงจาก VM (ไม่ใช่ render เดโม่) — pipeline photo-tour ที่มีอยู่ใช้ได้เลย

## 3. Visual QA 4 ระดับ (MASTER §53) — ใครตรวจอะไร

| ระดับ | ตรวจด้วย | อัตโนมัติ? |
|---|---|---|
| L1 Pixel | OverflowAudit + lint กฎ px/radius/spacing | ✅ ทุก build |
| L2 Component | e2e state matrix (กดครบ 8 state ต่อ control) | ✅ ทุก build |
| L3 Application | golden screens + ภาพทัวร์จริงจาก VM | 🔶 ทุกรอบ release |
| L4 System | ภาพ 9 หน้าเปิดพร้อมกันเทียบ cross-app (MASTER §58) + contrast gate | 🔶 release + ก่อน image build |

## 4. Self-Critique Loop (MASTER §55) — ทำเป็น artifact ไม่ใช่คำพูด

ทุกฟีเจอร์ UI ก่อน mark เสร็จ ต้องมีไฟล์ `docs/DESIGN-SYSTEM-v3/reviews/<feature>.md` ครบ 17 ข้อของ MASTER (cheap/generic/inconsistent/รก/ว่าง/hierarchy/สีเกิน/effect เกิน/a11y/responsive/edge/performance) — และ **ภาพ before/after** จาก VM แนบใน evidence/

## 5. Cheapness Audit checklist (MASTER §56) — ย่อเป็น 6 คำถามเชิงกล
1. ค่าใดใน diff นี้ไม่ผ่าน token? (lint ตอบแล้ว)
2. สิ่งใดเพิ่ม effect โดยไม่เพิ่ม hierarchy? (diff + golden ตอบ)
3. state ไหนยังไม่มี? (state matrix test ตอบ)
4. ถ้าซ่อนสีออก ผู้ใช้ยังแยกสถานะได้ไหม? (screenshot grayscale — QA ตา, เก็บใน reviews/)
5. แอปอื่นจะ reuse อะไรจากงานนี้? (ถ้าตอบไม่ได้ = สร้างแยกเกิน)
6. "นี่คือ LitricOS ควรเป็นจริงหรือ?" (§72)

## 6. Governance (MASTER §51)

- **Version:** DESIGN_SYSTEM_VERSION = 3.0, bump เมื่อ token/หลักการเปลี่ยน — token เปลี่ยนต้องอัปเดต contrast gate + golden ทันทีใน PR เดียว
- **ทุก PR UI แตะ 1 ใน 9 ข้อ:** consistency / accessibility / theme / localization / responsive / motion / keyboard / performance / edge — ระบุใน description ว่าตรวจอะไรแล้ว ผลเป็นอย่างไร
- **ห้ามแก้ component กลางโดยไม่รัน:** full workspace tests + lint + golden update

## 7. Performance UX Gate (MASTER §46)

- ทุก effect ใหม่ต้องมีงบ: blur ≤ 1 ชั้นต่อ overlay, shadow ≤ 2 ชั้นต่อ view, frame budget **16.6ms** — วัดด้วย frame counter ที่มีอยู่ (serial log นับ frame จริง)
- ถ้า fps ต่ำกว่า 55 ในภาพทัวร์ = ถอน effect นั้น (ไม่ต่อรอง)

## เกณฑ์ตรวจเฟส QA
- [ ] design_lint รันได้และตอนนี้เขียว (แก้ของพังที่มันจับก่อน)
- [ ] golden 10 ใบอยู่ใน evidence/golden/ + สคริปต์ diff
- [ ] reviews/ มีอย่างน้อย 1 ไฟล์ต้นแบบ (ทำจริงกับฟีเจอร์แรก)
- [ ] frame budget มีตัวเลขจริงแนบใน release note
