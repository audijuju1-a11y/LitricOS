# 00 — AUDIT: สถานะ UI จริงของ LitricOS (หลักฐาน file:line)

> **สถานะ:** ตรวจจากโค้ดจริงบนดิสก์ (src/litric-shell, src/litric-design) + ภาพจริง 114 ภาพจากทัวร์ VM
> **สรุปเดียว:** รากฐานมีอยู่จริงและดีกว่าที่คิด (token, contrast gate, spring motion, overflow audit) — แต่ **ครึ่งหลังของระบบไม่ถูกใช้** ทำให้ UI ที่เห็น = โครงสร้างดีแต่ไร้ชีวิต ไร้ hierarchy ทางสายตา และไม่มาตรฐานเชลล์

---

## 1. สิ่งที่มีอยู่แล้วและใช้งานจริง (อย่าทำทับ — ต่อยอด)

| หลักฐาน | รายละเอียด |
|---|---|
| `litric-design/src/lib.rs:395-435` | `TypeStep` 8 ขั้น (Caption 12 → Display 38) — typography scale ครบ |
| `litric-design/src/lib.rs:580-655` | `Motion` springs 4 token (INSTANT 120ms → SLOW 450ms) + `MAX_SETTLE_MS=500` + `reduced()` — **แต่ shell ไม่เคยเรียกใช้เลย** |
| `litric-design` tests | contrast gate: body text ผ่าน AA บนทุก surface ทุก theme (รันอยู่แล้ว) |
| `litric-shell/src/ui.rs:130-146` | `text_px/line_px/px` ยึด scale กลาง — ไม่มี magic number |
| `litric-shell/src/ui.rs:185-199` | `style/style_bold/style_label` — style ผ่าน token |
| `src/litric-shell/tests/overflow_audit.rs` | `OverflowAudit` — พิสูจน์ "ตัวอักษรไม่หลุดกรอบ" ด้วยการนับจริง (เป็น integration test ไม่ได้อยู่ในไบนารีของ shell — แก้เส้นทางเมื่อ 2026-10-09 เพราะเดิมอ้าง litric-shell/src/overflow.rs ซึ่งไม่มีอยู่จริงในรีโป) |
| `litric-shell/src/e2e.rs` | golden-frame test — เปรียบเทียบภาพเดสก์ท็อปเชิงกล |
| `litric-shell/src/shell.rs:1659-1746` | Alt+Tab มี state จริง (cycle ถูกต้อง), Tab navigation ใน shell |
| `views.rs:229` | "one bar carries the system" — system bar เดียว ไม่มี dock ซ้ำซ้อน (decision ถูกต้องตาม HIG) |

## 2. จุดพังที่ทำให้ "UI กาก/ไม่มาตรฐาน" — ระบุด้วยหลักฐาน

### A. Motion เขียนไว้แต่ไม่ใช้ (สาเหตุหลัก "กระตุก ไม่มีชีวิต")
- `grep Motion:: litric-shell/src/*.rs` → **0 results** — ทุก state change เกิดทันที 100% ไม่มี transition
- ผลที่ตามมา: การเปิด/ปิดเมนู, focus เปลี่ยน, หน้าต่างยกขึ้น — ทั้งหมด "กระโดด" ไม่มี continuity
- **ชั้นแก้:** ต่อ spring integrator เข้า frame loop (มี frame จริง 60Hz จาก serial log "16 frames") แล้วให้ overlay/panel/window ใช้ `Motion::FAST/NORMAL`

### B. Hierarchy ทางสายตาแบนเพราะ bold แทบไม่ถูกใช้
- `Style::bold` ถูกเรียกทั้งโปรเจกต์แค่ **7 จุด** (desktop.rs 2, ui.rs 3, views.rs 2) ในขณะที่ `Style::new` ถูกใช้เป็นร้อย
- ผล: title, list header, ปุ่มหลัก, ค่า settings — ทุกอย่างน้ำหนักเดียวกัน = สายตาไม่รู้ว่าอ่านตรงไหน (ขัด Principle C "HIERARCHY OVER DECORATION")
- **ชั้นแก้:** typographic role table (§01) + audit ให้ทุก view มี ≥2 ระดับ

### C. เชลล์ขาดองค์ประกอบมาตรฐาน OS (MASTER §23-§30)
ยืนยันจาก grep: ไม่มี dock (ตัดสินใจถูก — แต่ต้องมี **app grid/launchpad** มาตรฐานแทน), ไม่มี Control Center, ไม่มี Notification Center, ไม่มี Universal Search/Command Palette, ไม่มี window snapping (เฉพาะ fullscreen)
- **ชั้นแก้:** §03-SHELL-SYSTEM วางสถาปัตยกรรมครบทั้ง 5 อย่างที่ขาด

### D. Theme มีแต่ token ไม่มีทางเปลี่ยนจริง
`ThemeKind::all()` มีหลาย theme และ contrast gate รันครบ แต่ไม่มี Settings UI เปลี่ยน theme ต่อ session
- **ชั้นแก้:** Settings → Appearance ต้องเขียน theme selection ลง persistent config

### E. Desktop render path ซ้ำซ้อน (หลักฐานจากรอบก่อน)
`desktop.rs::render` (เดโม่สถิต + `radial_wash`) อยู่ร่วมกับ `views.rs::draw` (ของจริง) — สองเส้นทาง = โค้ดตาย + สับสนตอนแก้
- **ชั้นแก้:** ลบ/รวมเส้นทางเดโม่เป็นเพียง wallpaper provider เดียว

### F. Icon grammar ยังไม่ได้ผ่านกฎ MASTER §14
`icons.rs` วาดด้วย monoline stroke เดียว (จากหัวไฟล์: "uniform stroke weight, single-...") — ยังไม่มี grid rule/optical alignment test
- **ชั้นแก้:** เพิ่ม icon audit test (ความหนา stroke ต้องคงที่, พื้นที่ optical ใกล้กัน)

## 3. สิ่งที่ภาพจริง 114 ภาพยืนยัน
- ~~หน้าต่างไม่มี elevation ต่างกัน~~ **แก้ไขหลังตรวจเชิงลึก:** `views.rs:490-497` มี shadow คู่ focused (blur 30/alpha 0.60) และ unfocused (blur 18/alpha 0.40) + title สี dim อยู่แล้ว — สิ่งที่ขาดคือ **motion** ของการเปลี่ยนสถานะ ไม่ใช่ค่า shadow
- ไม่มี animation ใด ๆ ระหว่าง state (เข้ากับหลักฐาน A) — **แก้แล้วใน Wave 0.1:** `anim.rs` เชื่อม Motion tokens เข้า frame loop ผ่าน `Shell::tick`
- ข้อความไม่หลุดกรอบ (OverflowAudit ทำงาน) — จุดที่เคยร้องเรียนได้รับการแก้และพิสูจน์แล้ว

## 4. ข้อสรุปเชิงกลยุทธ์
**ไม่ใช่การเขียน UI ใหม่ — เป็นการ "เปิดระบบที่ถูกเขียนไว้แต่ปิดอยู่"**
1. ทำ Motion ให้ทำงานจริง (ผลตอบแทนสูงสุด, เสี่ยงน้อยสุด, มี token รออยู่แล้ว)
2. ยกระดับ hierarchy ด้วย typographic roles ที่มี audit
3. เติมเชลล์ที่ขาด (Control Center, Search, Notification Center, Snapping)
4. เชื่อม Theme engine เข้า Settings
5. รวม render path เดียว
ทุกข้อผูกกับ QA gates ใน §04 ทุกเฟสผ่าน audit ก่อนไปต่อ
