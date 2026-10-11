# 02 — COMPONENTS & STATES (MASTER §15-§19, §37)

> หลัก: reuse ก่อนสร้างใหม่เสมอ (MASTER §68) — ตารางนี้ระบุ "สร้างใหม่ / ยกจากเดิม" ของทุก component เพื่อไม่ทำซ้ำสิ่งที่มี

## 1. สถานะปัจจุบันจาก audit

**มีอยู่แล้ว (ใช้ต่อ ห้ามเขียนใหม่):** ปุ่ม, list row, tab, dialog พื้นฐาน, context menu (มี 7 จุดใน fsbridge/desktop), text input (omnibox + rename), launcher grid, toggle ใน Settings, window chrome พร้อม traffic lights

**ขาด:** state ครบชุด, disabled/loading/empty มาตรฐาน, focus ring ระบบเดียว, tooltip/popover, toast/notification มาตรฐาน, segmented control, slider, progress, command palette

## 2. State Matrix บังคับทุก control (MASTER §17)

| State | สัญญาณ (ต้อง ≥1 non-color cue) | Motion | ห้าม |
|---|---|---|---|
| Default | border `divider` | — | — |
| Hover | bg `surface-2` + cursor | INSTANT | ขยับ element |
| Pressed | bg `surface-3` + scale 0.98 | INSTANT | scale > 1 |
| Focused | **focus ring 2px accent (นอก edge 1px)** | INSTANT | ซ่อน ring เมื่อมาจาก keyboard |
| Selected | fill soft accent + check/บอกทางอื่น | FAST | สีอย่างเดียว |
| Disabled | text `disabled` + hit ยังจับเพื่อ tooltip เหตุผล | — | หายไปจนผู้ใช้คิดว่าถูกลบ |
| Loading | spinner 16px แทน trailing icon | FAST loop | แช่แบบไม่บอกเวลาที่คาด |
| Error | icon ⚠ + ข้อความ role `meta` | — | สีแดงอย่างเดียว |

กติกา: state ต้อง implement ใน **จุดเดียว** (draw fn ของ control) ไม่ใช่แต่ละ view เขียนเอง — นี่คือสิ่งที่ทำให้ "แอปอื่นเข้ามาก็เป็นระบบเดียวกัน"

## 3. Interaction Model (MASTER §18-§19) — ผูก Motion token

```
Intent → Action → Immediate Feedback → Final State
click  → INSTANT press    → ผลลัพธ์      → สถานะใหม่คงอยู่
open   → FAST ขยายจากจุด trigger (popover) / NORMAL (panel)
close  → ย้อนเส้นทาง open ด้วย token เดียวกัน
drag   → ตามพอยเตอร์ 1:1, drop มี settle NORMAL
```

**ข้อห้ามสัมบูรณ์:** ไม่มี bounce, ไม่มี overshoot > 4%, ทุก anim ≤ `Motion::MAX_SETTLE_MS`

## 4. Component Worklist (ลำดับตามผลตอบแทน)

### ชุด A — แก้ของเดิมให้ครบ state (ทำก่อน สร้างของใหม่น้อยสุด)
| Component | งาน | ไฟล์ |
|---|---|---|
| Button | focus ring + pressed scale + disabled ชัด | ui.rs draw กลาง |
| List row (Files/Settings) | hover/selected/disabled มาตรฐาน + role typography | apps.rs |
| Text field | caret blink (INSTANT 400ms), selection, IME-safe | apps.rs |
| Dialog | scrim E5 + Focus trap + Esc = cancel เสมอ | shell.rs |
| Context menu | E3 shadow + hover state + disabled item + submenu ไม่ทำในเฟสนี้ | desktop.rs hotspots |

### ชุด B — สร้างใหม่ที่เชลล์ต้องมี (MASTER §15 ที่ยังว่าง)
| Component | Spec สั้น | ที่ไหน |
|---|---|---|
| Toast/Notification | ขวาบน, ซ้อนได้ ≤3, auto-dismiss 6s (hover หน่วง), action ≤2, undo สำหรับ destructive | notification.rs ใหม่ |
| Popover/Tooltip | anchor กับ trigger, เปิด FAST, ปิดเมื่อ blur/click นอก | ui.rs |
| Segmented Control | ใช้แทน dropdown ≤4 ตัวเลือก (theme picker, view mode) | ui.rs |
| Slider | ปรับเสียง/ความสว่าง, keyboard: ←→ step, Home/End | control center |
| Progress (determinate/indeterminate) | scan ไฟล์, boot | ใช้ใน Defender |
| Command Palette | ดู §03 หัวข้อ Search | search.rs ใหม่ |

### ชุด C — Edge/Empty states (MASTER §37) — template เดียวทั้งระบบ
ทุก view ต้องมี 4 state วาดด้วย helper เดียว `empty_state(icon, title, body, action?)`:
- **Empty:** icon dim + ประโยคบอกสิ่งแรกที่ควรทำ + ปุ่ม primary (เช่น "สร้างโฟลเดอร์")
- **Loading:** skeleton แบบเบา หรือ spinner กลาง — ไม่มี blank
- **Error:** icon + เหตุผล + ปุ่ม retry
- **No permission / offline:** ตามประเภท + ปุ่ม "ลองอีกครั้ง"

ห้าม view ใดวางข้อความแขวนลอยเอง (lint: จับ string ตรงกลาง view ที่ไม่มี icon/action)

## 5. UX Writing กับ component (MASTER §44)
- ปุ่ม = กริยาเฉพาะเจาะจง ("ลบ 1 ไฟล์" ไม่ใช่ "ตกลง" เมื่อ destructive)
- Error = เกิดอะไร + ทำอะไรได้ ("ไฟล์ถูกใช้งานโดยแอปอื่น — ปิดแอปแล้วลองอีกครั้ง")
- ไทยเป็นภาษาหลัก: ทดสอบ truncation กับสระบน/ล่างซ้อน (มี OverflowAudit ครอบอยู่แล้ว)

## เกณฑ์ตรวจเฟส Components
- [ ] ทุก control ในชุด A ผ่าน state matrix 8 ชั้น (มี e2e test กดผ่านทุก state)
- [ ] focus ring เห็นชัดในภาพ golden ทุกใบ (QA ตา)
- [ ] Empty/Loading/Error ปรากฏครบใน Files + Settings + Browser (ภาพจริงเก็บใน evidence)
- [ ] ไม่มี draw code ของ control ซ้ำระหว่าง views (grep จุดวาดซ้ำ)
