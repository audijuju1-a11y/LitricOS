# 03 — SHELL SYSTEM (MASTER §23-§36)

> เชลล์คือที่ที่ผู้ใช้ "รู้สึกว่าเป็น OS" — audit พบว่าเรามี: window manager + Alt+Tab จริง + system bar เดียว + context menu + launcher (Meta) — ขาด 5 ระบบใหญ่ หัวข้อนี้ spec ทีละอันพร้อมจุดเชื่อมโค้ดจริง

## 1. Window System (MASTER §23)

State ที่ต้องมีครบ: `Normal / Focused / Unfocused / Fullscreen / Maximized / Minimized / Snapped / Dragging / Resizing / Busy`

**มีแล้ว:** focus model จริง (shell.rs:1659 Alt+Tab ถูกต้อง), fullscreen, drag, resize
**เพิ่ม:**
- **Snapping:** ลากชนขอบซ้าย/ขวา = ครึ่งจอ, ชนบน = maximize; แสดง preview overlay E2 ระหว่างลาก (opacity 0.18 accent) ก่อนปล่อยจึงย้ายจริง; คีย์ลัด `Meta+←/→` สลับครึ่ง, `Meta+↑` maximize
- **Minimize → ไปไหน:** เพราะเราไม่มี dock (views.rs:229 ยืนยัน) — หน้าต่างเล็กลงบินเข้า icon แอปใน launcher grid + running indicator จุดกลมใต้ icon (Motion::NORMAL) — นี่คือ Litric Signature ข้อแรก
- **Busy:** app ค้าง > 2s → title แสดง spinner 12px แทน icon, hit test ยังรับ event (ไม่ lock เชลล์)

## 2. Desktop Layer Contract (MASTER §24-§25) — z-order เหล็ก

```
L0 wallpaper → L1 desktop icons → L2 windows → L3 floating (menus/popover)
L4 notifications → L5 system overlay (Control Center/Spotlight) → L6 critical modal
```
- `Hotspots::hit` ต้องเดิน z จากสูงลงต่ำเสมอ (มีอยู่แล้ว — เพิ่ม test ล็อกพฤติกรรม: เมนูเหนือหน้าต่างเสมอ)
- Desktop context menu = มาตรฐานเดียวกับ §02 ชุด A (E3 + hover + disabled)

## 3. System Bar (MASTER §27) — จาก "แถบเดียว" สู่ "แถบที่สมบูรณ์"

มี: นาฬิกา, Defender shield, notification banner (รอบก่อนทำงานแล้ว)
**เติมให้ครบมาตรฐาน โดยไม่แปลงเป็น Windows taskbar:**
- ซ้าย: **Litric menu** (กลางจากปุ่มเดิม): About / Settings / Lock / Restart / Shutdown
- ขวา: แบตเตอรี่ (ถ้ามี), เสียง, เครือข่าย, **Control Center icon**, นาฬิกา
- ทุก icon มี hover INSTANT + click เปิด popover E3 — ห้ามเปิดหน้าต่างใหม่จากแถบ

## 4. Control Center (MASTER §28) — popover จาก system bar

Panel E3 glass (semantic ตาม §01-7), กว้าง 320 logical, ตาราง tile 2 คอลัมน์:
Wi-Fi / Bluetooth / Night Light / Volume slider / Brightness slider / Focus mode / Power
- ทุก tile: Off/On/Active/Disabled/Loading/Error ตาม §02 state matrix (สถานะจริงอ่านจาก fsbridge/sysfs ที่มีอยู่)
- เปิด FAST + fade scrim, ปิดด้วย Esc/click นอก

## 5. Universal Search / Command Palette (MASTER §30)

**แทน Spotlight แบบ copy** — เรียก **"Litric Seek"** (signature):
- เปิดด้วย `Meta` (bare release ที่ไม่ได้เปิด launcher อื่น) หรือ `Ctrl+Space` — หนึ่ง entry point เดียวรวม launcher+search (ลดโครงสร้างซ้ำ: launcher grid เดิม = ผลลัพธ์ category "Apps")
- โครง: ช่องค้นเดี่ยว + ผลลัพธ์จัดกลุ่ม `Apps / Files / Settings / Actions`
- **Actions** = command ระบบ ("เปลี่ยนธีมสว่าง", "เปิด Night Light", "ล้างถังขยะ") — นี่คือ Command Palette ในตัว ไม่สร้าง UI คู่
- keyboard: ↑↓ เลือก, Enter รัน, Esc ปิด; จำ query ล่าสุด 5 รายการ
- จุดเชื่อม: launcher เดิมใน shell.rs (Meta handling มีแล้ว) — ยกระดับจาก grid สู่ palette ที่มี grid เป็น tab แรก

## 6. Notification System (MASTER §29)

มีแล้ว: banner + shield Defender — ยกระดับเป็นระบบ:
- **Notification Center:** คลิกนาฬิกา → panel ขวาดึงลง รวม notification ที่ยังไม่อ่าน จัดกลุ่มตามแอป, ปุ่ม "ล้างทั้งหมด", แต่ละอันมี dismiss/undo
- ประเภท 8 แบบ (Info/Success/Warning/Error/System/Application/Security/Permission) — icon + สี soft + **ข้อความ role เสมอ** (ไม่สื่อด้วยสีเดียว)
- Destructive action ใน notification ต้องมี undo 5s
- เก็บ history ใน memory + เขียน log ทาง fsbridge (ไม่ persist ข้ามบูตในเฟสนี้)

## 7. Files as Finder (MASTER §32) — งานที่ค้างจาก UI-PLAN v1

มีแล้ว: sidebar, New Folder, Rename, Delete, Trash, context menu
**เติม:** Tabs (≤4), path breadcrumb, view switch Grid/List, preview pane (ขนาดไฟล์+วันที่), Properties dialog, **edge cases ทั้ง 10 ข้อของ MASTER ตาม template §02-4**

## 8. Dialog System (MASTER §34)

- ประเภท 10 แบบตาม MASTER — วาดด้วย dialog helper เดียว `dialog(kind, title, body, actions)`
- **Danger:** ปุ่ม destructive สี danger + ข้อความระบุ object ("ลบ 'รายงาน.pdf' ถาวร?") + ปุ่มรองคือ Cancel เสมอ + Esc ยกเลิก + ไม่มี timeout ให้กดเอง
- ทุก dialog มี Focus trap + ปุ่ม primary รับ Enter

## 9. Login/Boot/Lock (MASTER §35)

- Boot: โลโก้ + progress เส้นเดียว (มีจริงอยู่บางส่วน) → fade เข้าเดสก์ท็อป SLOW ครั้งเดียวต่อบูต
- Lock: wallpaper เดิม + blur เบา + การ์ดผู้ใช้กลาง; password field ใช้ text field มาตรฐาน (masked)
- ห้าม UI ใหม่แยก design language — ใช้ token เดียวกัน

## 10. Settings (MASTER §31)

- โครง Category → Section → Setting → Description → Control (มี sidebar แล้ว)
- **เติม Search ใน Settings** ค้นชื่อ setting → กรอง + highlight section
- หน้า Appearance เป็นหน้าแรกที่เชื่อม Theme Engine จริง: การ์ดตัวอย่าง 4 theme (Dark/Light/OLED/High Contrast) + accent picker 7 สี (จาก palette เดิมเท่านั้น)

## เกณฑ์ตรวจเฟส Shell
- [ ] สแนป/ลากหน้าต่างมี preview และกลับสถานะเดิมได้เสมอ (ไม่มีทางติดครึ่ง)
- [ ] Seek ค้นแอป/ไฟล์/setting/action ได้จริง e2e (พิมพ์ → ผล → Enter → เกิดจริง)
- [ ] Control Center toggle เปลี่ยนสถานะระบบจริง (ไฟล์ sysfs/mock ที่ตรวจได้)
- [ ] Notification Center เก็บ/ล้าง/undo ครบ
- [ ] ภาพ golden ใหม่: desktop + control center + seek + notification ครบ 4 ใบ
