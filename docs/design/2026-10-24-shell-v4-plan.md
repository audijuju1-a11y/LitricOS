# 2026-10-24 — Shell v4: แผน redesign top bar, dock, lock screen และแก้ UI กระตุก

**สถานะ:** แผน + งานที่ลงมือแล้ว (ดูหัวข้อ 8) · branch `redesign/shell-v4`
**ขอบเขตรอบนี้:** top bar, dock/launcher, lock screen, ขนาดตัวอักษร/ระยะ/แผงที่เกี่ยวข้อง, ความกระตุกของเฟรม
**นอกขอบเขตรอบนี้:** แอป (ข้ามตามที่สั่ง), wallpaper PNG, boot wordmark, OOBE — อยู่ใน Wave ถัดไป (หัวข้อ 9)

---

## 1. ปัญหาที่เห็นจากโค้ดและภาพจริง (ไม่ใช่รสนิยม)

ภาพอ้างอิง: `frame_dump` 1280×800 ก่อนแก้ (24 scene), ภาพ QEMU 1920×1080 ที่ผู้ใช้ส่งมา

| # | อาการ | สาเหตุในโค้ด |
|---|---|---|
| T1 | Top bar ดูเป็นตัวหนังสือลอยบน wallpaper | `views.rs::draw_top_bar` ไม่มีพื้นผิวและไม่มีเส้นขอบ ("The bar has no surface") อ่านยากเมื่อ wallpaper สว่าง |
| T2 | ซ้ายมี `Apps / Workspace / Environment` น้ำหนักเท่ากันหมด ไม่มีลำดับ ไม่บอกว่าตอนนี้อยู่แอปไหน | ชื่อแอปที่โฟกัสไปอยู่กลางบาร์ลอยกับจุด workspace |
| T3 | ขวามีไอคอน 5–6 ตัวระยะห่างเท่ากัน เวลา `00:00 UTC` ไม่มีวัน | `status_icons` คำนวณจากจำนวน ไม่ได้จัดกลุ่ม |
| D1 | Dock สูง 30px ไอคอน 22px ช่องค้นหากินที่ไป ~60% ของความกว้าง | `draw_dock`: `dock_h = 30`, `item = 22`, `bubble_w = 300` |
| D2 | ปักแค่ 4 แอป ที่เหลืออีก 4 ต้องเปิด launcher | `DOCK_PINNED = [3, 4, 5, 2]` |
| D3 | ไม่มี indicator แยกแอปที่รันอยู่ชัดเจน (จุด 1.5px) | `fill_circle(..., t.px(1.5))` |
| L1 | การ์ด widget 300×90 สามใบชนท้ายตัวเลขนาฬิกา | `widgets_y = 84 + size + 8` |
| L2 | ช่องรหัสผ่านเป็นกล่อง 460×80 มีอวาตาร์ซ้ายและปุ่มขวา ไม่ใช่โครงที่คนคุ้น | `lock.rs::entry` |
| P1 | เฟรมที่มีหน้าต่างช้า: เงา (shadow) กิน 8.77M จาก 11.36M pixel visits | `Canvas::soft_shadow` วาด 12 วงทับกัน รวมถึงใต้ตัวหน้าต่างเอง (BUG-0015) |
| P2 | Lock ที่ customize เปิดอยู่ 112–196 ms ต่อเฟรม | shadow ของ sheet + glass หลายชั้น (BUG-0018 ส่วน "Remaining work") |

## 2. สิ่งที่ศึกษาจาก macOS และสิ่งที่ "ไม่" ลอก

แหล่ง: Apple HIG *The menu bar* (อ่านฉบับ .md สำเร็จ), บทความ AppleInsider เรื่อง Liquid Glass บน macOS Tahoe, ภาพ lock screen/Dock ที่ผู้ใช้ส่งมา
**ข้อจำกัดที่พบตรง ๆ:** หน้า HIG *Dock* ดึงไม่ได้ (client error) จึงไม่อ้างตัวเลขของ Dock จาก HIG — ตัวเลข dock ด้านล่างเป็นการตัดสินใจออกแบบของเราเอง

| หลักการ | ที่มา | ที่เรานำมาใช้ | ที่เราหลีกเลี่ยง |
|---|---|---|---|
| เมนูแอปอยู่ซ้าย, menu bar extras อยู่ขวา, ชื่อแอปเป็นอันแรกของเมนู | HIG menu bar | ชื่อแอปที่โฟกัสตัวหนาอยู่ซ้ายสุดหลังโลโก้ สถานะระบบรวมเป็นกลุ่มขวา | ไม่ทำเมนู File/Edit/View ปลอม เพราะแอปของเรายังไม่มีระบบเมนู |
| ความสูง menu bar 24 pt | HIG menu bar | อยู่ในย่านเดียวกัน แต่ 28 logical เพื่อเป้ากดที่ใหญ่ขึ้น | — |
| ระบบซ่อน extras ก่อน เมื่อพื้นที่ไม่พอ ไม่บีบเมนู | HIG menu bar | ลำดับความสำคัญ: นาฬิกา > แจ้งเตือน > เสียง > เครือข่าย > ที่เหลือ | — |
| Tahoe โปร่งแสงบางส่วน อ่านยากบนสีบางแบบ | AppleInsider | พื้นผิว top bar ทึบ 78% (ไม่เบลอ) ให้ contrast คงที่ | ไม่ใช้กระจกเบลอทั้งแถบ (แพงและอ่านยาก) |
| Lock screen: วันที่เล็กบน, นาฬิกาใหญ่, อวาตาร์ + ชื่อ + ช่องรหัสล่าง | ภาพที่ผู้ใช้ส่ง | จัดลำดับเดียวกัน แต่คงโหมดกดปุ่มเพื่อเผยช่องรหัส (stage Idle→Entry) ที่มีอยู่ | ไม่ใช้ wallpaper/สีของ Apple ไม่ใช้ Touch ID กลางจอ |
| Dock magnification | ประสบการณ์ macOS | **ไม่ทำ**: ต้องวาดใหม่ทุกครั้งที่เมาส์ขยับ | ใช้ hover lift 3 px + ป้ายชื่อ แทน |

## 3. รูปแบบที่จะออกมา (หน่วย logical px; ตัวคูณ = กว้างจอ ÷ 1280)

### 3.1 Top bar — สูง 28 (เดิม 24)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ ◧  Files   Apps  Workspace  Environment                  ● ○ ○ ○   ◖◗ ⌁ ▯ 🔔   Sat 24 Oct  03:11  ◯ │
└─────────────────────────────────────────────────────────────────────────────────────────┘
  mark  แอปที่โฟกัส   เมนูระบบ (ของเดิม ทำงานเหมือนเดิม)      workspace   status group      นาฬิกา    user
        13 semibold   13 regular, dim 82%                      pips 6px    icon 16, ห่าง 14   13 regular
```

- พื้นผิว: `bg_base` ทึบ 78% + เส้นล่าง 1 px สีขาว 7% ไม่เบลอ ไม่มี gradient
- กลางบาร์ว่าง (แบบ macOS) — ชื่อแอปย้ายไปซ้าย workspace pips ย้ายไปกลุ่มขวา
- เมื่อพื้นที่ไม่พอ: ตัดวันที่ → ตัด pips → ตัดเมนูที่ท้ายสุด (ลำดับเดียวกับ HIG) ไม่เคยเขียนทับกัน
- ทุก hotspot เดิม (`Hot::Logo`, `MenuApps`, `MenuWorkspace`, `MenuEnvironment`, `Workspace(i)`, `Status*`) คงไว้

### 3.2 Dock — เตี้ย กว้าง ลอยเหนือขอบล่าง 10

```
              ┌───────────────────────────────────────────────────────────────────────┐
              │  ▣  ‖  📁  🌐  >_  ⚙  ◔  🛡  ⌂  ‖  🗑  ‖  🔍 ค้นหา              │   สูง 44
              └───────────────────────────────────────────────────────────────────────┘
                 ▲launcher   ▲แอปทั้งหมด (8)  ▲ถังขยะแยกขวา  ▲ช่องค้นหาย่อ 140
```

| ค่า | เดิม | ใหม่ |
|---|---|---|
| ความสูงแถบ | 30 | **44** |
| ไอคอนแอป | 22 | **32** บน tile มุม 9 (squircle ไม่ใช่วงกลม) |
| ระยะห่างไอคอน / padding ข้าง | 10 / 16 | **8 / 10** |
| จำนวนแอปบนแถบ | 4 | **7 + ถังขยะ** (ทุกแอป) |
| ช่องค้นหา | 300 | **140** (ขยายเมื่อโฟกัส) |
| ระยะลอยจากขอบ | 24 | **10** |
| รัศมีแถบ | 15 (pill) | **16** |
| running indicator | จุด 1.5 px | จุด **4 px** ใต้ tile |
| hover | เติม tile + ป้ายชื่อ | เติม tile + ยกขึ้น 3 px + ป้ายชื่อ Caption |
| shadow | 40 / dy 14 | **24 / dy 8** (เบากว่า ถูกกว่า) |

การ morph เป็น launcher panel ใช้กลไกเดิม (`launcher_morph`) เปลี่ยนแค่ค่าความสูงแถวล่าง

### 3.3 Lock screen — เรียงใหม่จากบนลงล่าง

```
                                         ◖◗ ▯ 84%        ← ขวาบน: เครือข่าย/แบตเป็นชิปเล็ก (เดิมเป็นการ์ดใหญ่)

                                  Saturday, 24 October                 ← Title3 20, alpha 85%
                                         03:11                         ← ตัวเลขใหญ่ (preset เดิม S/M/L)

                       ┌────────┐ ┌────────┐ ┌────────┐
                       │ Uptime │ │ Network│ │ Battery│                ← widget ชิปเตี้ย 52 (เดิม 90) ใต้นาฬิกา มีช่องไฟ ≥ 24
                       └────────┘ └────────┘ └────────┘

                                           ◯  72                       ← อวาตาร์ 72
                                          root                         ← Headline 17 semibold
                                 ┌──────────────────────────┐
                                 │ ● ● ● ● ●            (→) │          ← ช่อง 280×40 pill ปุ่มลูกศร 32 ในช่อง
                                 └──────────────────────────┘
                              Press Space or Enter to unlock           ← Caption 12 เฉพาะ stage Idle
                                         [ Customize ]
```

- stage `Idle` แสดงอวาตาร์ + ชื่อ + บรรทัดบอกให้กดปุ่ม; `Entry` เผยช่องรหัส (พฤติกรรมเดิม)
- รักษา `Hot::LockWake / LockEntry / LockSubmit / LockCustomize` และลำดับการ push hotspot (ตัวที่วาดทีหลังชนะ)
- ไม่แตะ customize sheet, preset และ tweak ทั้งหมด

### 3.4 ตัวอักษร (ใช้ `TypeStep` เท่านั้น ห้ามเลขดิบ)

| ที่ใช้ | step | น้ำหนัก |
|---|---|---|
| ชื่อแอปใน top bar | Footnote 13 | semibold |
| เมนู / นาฬิกา | Footnote 13 | regular |
| ป้ายชื่อ dock, ข้อความช่วย lock | Caption 12 | regular |
| ชื่อผู้ใช้ lock | Headline 17 | semibold |
| วันที่ lock | Title3 20 | regular |

## 4. Tokens ที่เพิ่ม (ที่เดียว: `litric-shell/src/chrome.rs`)

`TOP_BAR_H=28`, `TOP_BAR_SURFACE_ALPHA=0.78`, `DOCK_H=44`, `DOCK_ICON=32`, `DOCK_GAP=8`, `DOCK_PAD=10`, `DOCK_LIFT=3`, `DOCK_FLOAT=10`, `DOCK_RADIUS=16`, `LOCK_AVATAR=72`, `LOCK_FIELD_W=280`, `LOCK_FIELD_H=40`, `LOCK_CHIP_H=52` — ไม่มีค่าสุ่มกระจายในแต่ละ view

## 5. ไฟล์ที่จะแก้ และเหตุผล

| ไฟล์ | การเปลี่ยน |
|---|---|
| `litric-shell/src/canvas.rs` | `soft_shadow` หนึ่ง pass (**ทำแล้ว**) + test ความเท่ากันทุกพิกเซล |
| `litric-shell/src/chrome.rs` (ใหม่) | tokens ในหัวข้อ 4 |
| `litric-shell/src/shell.rs` | `TOP_BAR_LOGICAL` อ่านจาก `chrome` |
| `litric-shell/src/views.rs` | เขียน `draw_top_bar` และ `draw_dock` (แถวล่าง) ใหม่ |
| `litric-shell/src/lock.rs` | จัดเลย์เอาต์ `draw`, `widgets`, `entry` ใหม่ |
| `litric-shell/tests/*` | อัปเดตค่าที่อิงขนาดเก่า, เพิ่ม test: hotspot อยู่ในบาร์, ไม่มีข้อความทับกันที่ 960/1280/1920 |
| `docs/bugs/BUG-0015…`, `BUG-0018…` | บันทึกตัวเลขหลังแก้ |

**ไม่แตะ:** boot flow, authentication (`Secret`, verifier), filesystem, window manager, แอปทั้งหมด

## 6. ความกระตุก — แผนวัดและเป้า

1. **(ทำแล้ว)** shadow หนึ่ง pass: pixel visits ฉากสามหน้าต่าง 11.36M → 3.39M, shadow 8.77M → 0.80M, ภาพเหมือนเดิมทุกพิกเซลใน 24 scene
2. top bar ใหม่เป็นการเติมสีทึบ ไม่เบลอ → ไม่เพิ่ม blur
3. dock เหลือ shadow เดียวและเบากว่า, ไม่มี magnification
4. lock: การ์ด 90→52 ลดพิกเซลของ glass และ backdrop cache; ตรวจ `lock_frame` ด้วย `render_bench`
5. เป้า (ตัวเลข host ใช้เทียบ "ก่อน/หลัง" บนเครื่องเดียวกัน ไม่ใช่คำสัญญาของเครื่องจริง): partial frame ขณะมีหน้าต่าง ≤ 16.67 ms, locked frame ≤ 33 ms
   **ยังไม่วัดบน QEMU จริง** — ตัวเลขบน image ต้องวัดต่างหาก (BUG-0015 "What is not claimed")

## 7. เกณฑ์ผ่าน

- `cargo test --workspace` ผ่านทั้งหมด และ `design_lint`, `overflow_audit`, `text_layout_audit` ไม่ตก
- `frame_dump` สร้างภาพทุก scene ได้ และไม่มีข้อความทับกันที่ 960×600, 1280×800, 1920×1080
- ไม่มี gradient ใหม่ใน top bar/dock/lock (พื้นผิวเป็นสีทึบ/alpha เท่านั้น)
- `reduced motion` ยังทำงาน (ไม่เพิ่ม animation ใหม่นอก spring เดิม)
- ภาพ before/after จริงจาก `frame_dump` เก็บใน `docs/evidence/2026-10-24/`

## 8. สถานะ

- [x] audit โค้ด + ภาพ baseline 24 scene
- [x] shadow หนึ่ง pass + test (commit `1ee2d70`)
- [ ] chrome.rs + top bar
- [ ] dock
- [ ] lock
- [ ] tests / evidence / อัปเดต BUG-0015, BUG-0018

## 9. Wave ถัดไป (ยังไม่ทำรอบนี้ — ไม่อ้างว่าเสร็จ)

wallpaper PNG ต้นฉบับ (ตอนนี้ `desktop::wallpaper` วาดด้วยโค้ดและมี glow ที่ขัดกับกฎ "ไม่ใช้ gradient" ของโปรเจกต์), wordmark ลายมือตอนบูต, OOBE ทั้ง flow, Settings ใหม่, ประเมินบน QEMU/จอจริง
