# 05 — ROADMAP: จากแผนสู่โค้ด (MASTER §60, §65 — mapping 34 phases)

> หลักบริหาร: 34 เฟสของ MASTER จับกลุ่มเป็น 6 Wave ตาม **ผลตอบแทนต่อความเสี่ยง** — ทุก Wave จบด้วย gate จริง (lint + golden + ภาพ VM) ก่อนเริ่ม wave ถัดไป ห้ามข้าม

## Wave 0 — รื้นฐานที่พัง (เล็ก, สูงผลตอบแทน) ≈ เฟส 03-10 ของ MASTER
| # | งาน | ไฟล์ | Gate |
|---|---|---|---|
| 0.1 | ~~ต่อไฟ Motion~~ **เสร็จ:** `anim.rs` (8 tests) + `ShellAnims` + tick ผูก frame loop + panel reveal ใน views | ✅ |
| 0.2 | ~~Elevation focus/unfocus~~ **ตรวจแล้วมีอยู่** (views.rs:490 shadow คู่ + title dim) — งานจริงคือ motion ตอน focus เปลี่ยน | เหลือ: focus transition |
| 0.3 | Typographic roles — **เสร็จชุดแรก:** `list_item` = `item_primary` bold (helper เดียว → Files/Settings/QuickSettings/Trash ทุกแอป) | ที่เหลือ: title/view roles |
| 0.4 | ~~ControlSpec~~ focus_ring/state_fill มีอยู่แล้ว (ui.rs:287/265) — งานเหลือ: ControlSpec เมื่อ Wave 1 แตะ controls |
| 0.5 | **เสร็จ:** design_lint.rs 4 กฎ — สีใน draw call (ยกเว้น neutral alpha), Radius family + escape-proof `let radius`, bold floor ≥8, Motion tokens เท่านั้น — ทดสอบย้อนกลับแล้วด้วย probe |
| 0.6 | **เสร็จ:** workspace 941 tests ✓ · clippy ✓ · fmt ✓ · image ใหม่ EXIT=0 · ทัวร์ VM 54 ภาพ (35 unique) → evidence/desktop-tour-after/ |

**ออกจาก Wave 0 เมื่อ:** เปิดเมนู/โฟกัสหน้าต่าง/กดปุ่ม มี motion จริง + ภาพใหม่ดู "มีชั้น" ขึ้นจากภาพเดิม (แนบ before/after)

## Wave 1 — Component state ครบ (MASTER §15-§19) ≈ เฟส 11-13
| # | งาน |
|---|---|
| 1.1 | ชุด A จาก §02: Button/Row/TextField/Dialog/ContextMenu ครบ 8 state + e2e กดครบ |
| 1.2 | empty_state helper + ใส่ Files/Settings/Browser ครบ 4 state |
| 1.3 | Toast + Popover/Tooltip (จาก §02 ชุด B) |

## Wave 2 — เชลล์ใหม่ที่ขาด (MASTER §23-§30) ≈ เฟส 14-17
| # | งาน |
|---|---|
| 2.1 | Snapping + preview + คีย์ลัด Meta+ลูกศร |
| 2.2 | Minimize → launcher icon + running dot (Litric Signature #1) |
| 2.3 | System Bar สมบูรณ์ + Litric menu |
| 2.4 | Control Center (tiles อ่าน/เขียนสถานะจริงผ่าน fsbridge) |
| 2.5 | Litric Seek = launcher+search+actions รวมเดียว (Meta) |
| 2.6 | Notification Center (รวม banner เดิม, เพิ่ม history+undo) |

## Wave 3 — แอปเป็นระบบเดียว (MASTER §31-§33) ≈ เฟส 18-20
| # | งาน |
|---|---|
| 3.1 | Files: tabs/breadcrumb/2 view modes/preview/properties + edge cases 10 แบบ |
| 3.2 | Settings: search + Appearance ผูก Theme Engine (4 theme + accent 7) |
| 3.3 | Browser/Terminal/Defender ผ่าน component กลาง (ตัด custom ที่หลุด token) |
| 3.4 | Defender UI: สแกนมี progress determinate + ผลเป็น list rows + quarantine dialog danger |

## Wave 4 — ความครอบคลุม (MASTER §22, §35-§43) ≈ เฟส 21-26
| # | งาน |
|---|---|
| 4.1 | Login/Lock/Boot ด้วย token เดียวกัน |
| 4.2 | Reduce Motion จริง (Motion::reduced ผ่าน settings) + High Contrast theme ผ่าน AA ทุก view |
| 4.3 | Localization: ไทย/อังกฤษสมบูรณ์ + ทดสอบ text expansion |
| 4.4 | Density (Comfortable/Compact) + responsive: 1280×800 เป็นฐาน, 1024×640 ยังใช้ได้, multi-monitor = จอตาม display settings เดิม |

## Wave 5 — ระบบคุณภาพถาวร (MASTER §27-§29, §51-§59) ≈ เฟส 27-34
| # | งาน |
|---|---|
| 5.1 | design_lint ครบ 8 กฎ (จาก 4) + CI fail ได้จริง |
| 5.2 | Golden Screens 10 ใบ + สคริปต์ diff + ภาพจาก VM อัตโนมัติ (ต่อยอด photo-tour) |
| 5.3 | reviews/ + Self-Critique artifact ต้นแบบ 1 ฟีเจอร์ |
| 5.4 | Design Review Output §63: ก่อนสร้าง A-K / หลังสร้าง A-F — ทำใน PR template |

## กติกาทำงานที่บังคับทุก Wave
1. **ก่อนเขียนโค้ด:** Design Review Output ส่วน A-K (MASTER §63) ลง PR/notes
2. **ระหว่าง:** lint ต้องเขียวตลอด ห้ามสะสม
3. **ก่อนจบ Wave:** ภาพ VM จริง (photo-tour pipeline) + golden อัปเดต + self-critique + สรุป WHAT CHANGED/WHY/ISSUES
4. **Conflict:** ยึดลำดับ Safety/A11y → Function → Clarity → Consistency → Performance → Visual (MASTER §67)

## สิ่งที่ห้ามทำในทุก Wave
- ห้ามสร้างเส้นทาง render คู่ (บทเรียน desktop.rs เดโม่)
- ห้าม token ใหม่ลอย ๆ โดยไม่บันทึกเหตุผล + audit
- ห้ามประกาศเสร็จโดยไม่มี states/errors/loading (No Fake Completeness §59)
- ห้ามแตะ palette/typography scale ใหม่โดยไม่ผ่าน contrast gate + §63

## Definition of Done ของทั้งระบบ = Final Quality Gate (MASTER §71)
ทำเครื่องหมายได้เมื่อ: lint 0 critical · golden 10 ใบผ่าน · ภาพ VM ทุกหน้ามาตรฐานเดียวกัน · motion ทุกจุดผ่าน token · ทุก control ครบ state · theme 4 แบบผ่าน AA · keyboard ผ่าน · ไทย/อังกฤษผ่าน · frame budget ผ่าน · ไม่มีส่วนใดดู template/clone
