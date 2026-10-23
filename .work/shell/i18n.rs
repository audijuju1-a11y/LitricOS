//! Small offline UI catalog for the built-in English and Thai interfaces.

use std::borrow::Cow;
use std::cell::Cell;

/// The language used for shell-owned interface copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    /// Thai interface.
    #[default]
    Thai,
    /// English interface.
    English,
}

impl Language {
    /// Stable preference-file value.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Thai => "th",
            Self::English => "en",
        }
    }

    /// Parse a stable preference token.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "th" | "th-th" | "thai" => Some(Self::Thai),
            "en" | "en-us" | "en-gb" | "english" => Some(Self::English),
            _ => None,
        }
    }

    /// User-facing name in the language itself.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Thai => "ไทย",
            Self::English => "English",
        }
    }

    /// Whether the running text stack can draw this language at all.
    ///
    /// English is always drawable: the Latin face is the one the loader refuses
    /// to start without. Thai needs `NotoSansThai`, and on an image that is
    /// missing it the honest behaviour is to say so rather than to print a
    /// screen of empty boxes.
    #[must_use]
    pub fn renderable(self) -> bool {
        match self {
            Self::English => true,
            Self::Thai => crate::font::draws_thai(),
        }
    }

    /// The name to show in a picker, in words the live stack can print.
    ///
    /// A language offered under its own name is the friendly thing to do —
    /// until the face that draws that name is the one that is missing, at which
    /// point the cheerful label is itself a row of boxes. English is the label
    /// of last resort because English is the one language guaranteed to be
    /// drawable, and the reason is stated rather than hidden.
    #[must_use]
    pub fn label(self) -> &'static str {
        if self.renderable() {
            return self.title();
        }
        match self {
            Self::Thai => "Thai — typeface not installed",
            Self::English => "English",
        }
    }
}

thread_local! {
    // Drawing helpers are also used directly by model and renderer tests. Keep
    // their neutral default English; the shell explicitly selects its saved
    // language at the start of every composed frame.
    static CURRENT: Cell<Language> = const { Cell::new(Language::English) };
}

/// Set the drawing language for the current shell thread.
pub fn set_current(language: Language) {
    CURRENT.with(|current| current.set(language));
}

/// Read the drawing language for the current shell thread.
#[must_use]
pub fn current() -> Language {
    CURRENT.with(Cell::get)
}

/// Translate shell-owned static labels. User data and daemon messages pass
/// through unchanged so the interface never rewrites a path or a service fact.
///
/// # Never print what cannot be drawn
///
/// A translation is returned only when the live text stack has a face that can
/// render it. A missing glyph draws a box ([`crate::font::draw`]), which is the
/// right signal for one character and the wrong signal for a whole interface:
/// with the Thai face absent and Thai selected, every label in the shell became
/// `▯▯▯▯`, and the desktop read as a wireframe rather than as a system. The
/// fallback is the English string, which the Latin face is guaranteed to have,
/// and [`Language::renderable`] is what a surface asks to explain the choice.
#[must_use]
pub fn text<'a>(english: &'a str) -> Cow<'a, str> {
    if current() == Language::English {
        return Cow::Borrowed(english);
    }
    if !current().renderable() {
        return Cow::Borrowed(english);
    }
    let thai = match english {
        "Activity" => "กิจกรรม",
        "Environments" => "สภาพแวดล้อม",
        "Settings" => "การตั้งค่า",
        "Files" => "ไฟล์",
        "Litric Browser" => "เบราว์เซอร์ Litric",
        "Browser" => "เบราว์เซอร์",
        "Terminal" => "เทอร์มินัล",
        "Trash" => "ถังขยะ",
        "Defender" | "Litric Defender" => "Litric Defender",
        "Quick access" => "เข้าถึงด่วน",
        "New folder" => "โฟลเดอร์ใหม่",
        "New text file" => "ไฟล์ข้อความใหม่",
        "Paste" => "วาง",
        "Compress" => "บีบอัด",
        "Extract" => "แตกไฟล์",
        "Details" => "รายละเอียด",
        "Details ✕" => "รายละเอียด ✕",
        "Empty folder" => "โฟลเดอร์ว่าง",
        "Nothing selected" => "ยังไม่ได้เลือกรายการ",
        "Folder" => "โฟลเดอร์",
        "File" => "ไฟล์",
        "Shortcut" => "ทางลัด",
        "Other" => "อื่น ๆ",
        "Create" => "สร้าง",
        "Confirm" => "ยืนยัน",
        "Cancel" => "ยกเลิก",
        "Could not finish" => "ทำรายการไม่สำเร็จ",
        "Name" => "ชื่อ",
        "Search the web with Google, or type a URL" => "ค้นหาเว็บหรือพิมพ์ที่อยู่เว็บไซต์",
        "Search engine:" => "เครื่องมือค้นหา:",
        "Search" => "ค้นหา",
        "Back" => "ย้อนกลับ",
        "Forward" => "ถัดไป",
        "Up" => "ขึ้น",
        "Save" => "บันทึก",
        "History" => "ประวัติ",
        "Bookmarks" => "ที่คั่นหน้า",
        "Downloads" => "รายการดาวน์โหลด",
        "Site" => "เว็บไซต์",
        "No pages visited yet" => "ยังไม่มีประวัติการเข้าชม",
        "No bookmarks saved yet" => "ยังไม่มีที่คั่นหน้า",
        "No downloads reported by the browser engine yet" => "ยังไม่มีรายการดาวน์โหลดจากเบราว์เซอร์",
        "Site controls" => "สิทธิ์ของเว็บไซต์",
        "This engine does not surface site permission requests to LitricOS, so this page cannot grant device access." => "เอนจินนี้ยังไม่ส่งคำขอสิทธิ์เว็บไซต์ให้ LitricOS จึงยังอนุญาตการเข้าถึงอุปกรณ์จากหน้านี้ไม่ได้",
        "Search processes" => "ค้นหาโปรเซส",
        "Appearance" => "รูปลักษณ์",
        "Display" => "จอภาพ",
        "Input" => "อุปกรณ์ป้อนข้อมูล",
        "Network" => "เครือข่าย",
        "Power" => "พลังงาน",
        "Region" => "ภูมิภาคและภาษา",
        "Accessibility" => "การช่วยการเข้าถึง",
        "Sound" => "เสียง",
        "Privacy" => "ความเป็นส่วนตัว",
        "Accounts" => "บัญชีผู้ใช้",
        "Diagnostics" => "การวินิจฉัย",
        "Dark" => "มืด",
        "Light" => "สว่าง",
        "High contrast dark" => "มืดคอนทราสต์สูง",
        "High contrast light" => "สว่างคอนทราสต์สูง",
        "Language" => "ภาษา",
        "Theme" => "ธีม",
        "Reduce motion" => "ลดภาพเคลื่อนไหว",
        "On" => "เปิด",
        "Off" => "ปิด",
        "Saved" => "บันทึกแล้ว",
        "Saving…" => "กำลังบันทึก…",
        "Save failed" => "บันทึกไม่สำเร็จ",
        "Unavailable" => "ไม่พร้อมใช้งาน",
        "System information" => "ข้อมูลระบบ",
        "Version" => "เวอร์ชัน",
        "Processor" => "หน่วยประมวลผล",
        "Memory" => "หน่วยความจำ",
        "Storage" => "พื้นที่จัดเก็บ",
        "SYSTEM" => "ระบบ",
        "STORAGE" => "พื้นที่จัดเก็บ",
        "REGION AND LANGUAGE" => "ภูมิภาคและภาษา",
        "Locale" => "รูปแบบภูมิภาค",
        "Detected from" => "ตรวจพบจาก",
        "Time zone" => "เขตเวลา",
        "Zone source" => "แหล่งข้อมูลเขตเวลา",
        "Clock" => "นาฬิกา",
        "12-hour" => "12 ชั่วโมง",
        "24-hour" => "24 ชั่วโมง",
        "First day" => "วันแรกของสัปดาห์",
        "Keyboard" => "แป้นพิมพ์",
        "Units" => "หน่วยวัด",
        "Metric" => "เมตริก",
        "Imperial" => "อิมพีเรียล",
        "Root filesystem" => "ระบบไฟล์ราก",
        "Measured by" => "วัดโดย",
        "Current framebuffer" => "เฟรมบัฟเฟอร์ปัจจุบัน",
        "QEMU default resolution" => "ความละเอียดเริ่มต้นของ QEMU",
        "Compatibility mode" => "โหมดรองรับเครื่องรุ่นเก่า",
        "UI scale" => "สเกลส่วนติดต่อ",
        "Display changes" => "การเปลี่ยนแปลงจอภาพ",
        "The framebuffer mode is selected at boot" => "เลือกโหมดเฟรมบัฟเฟอร์ตอนเริ่มระบบ",
        "Keyboard layout" => "รูปแบบแป้นพิมพ์",
        "Pointer and keyboard" => "เมาส์และแป้นพิมพ์",
        "Input service" => "บริการอุปกรณ์ป้อนข้อมูล",
        "Attached devices" => "อุปกรณ์ที่เชื่อมต่อ",
        "No input device reported by the kernel" => "เคอร์เนลไม่พบอุปกรณ์ป้อนข้อมูล",
        "Interfaces" => "อินเทอร์เฟซเครือข่าย",
        "No interface reported by the kernel" => "เคอร์เนลไม่พบอินเทอร์เฟซเครือข่าย",
        "Ethernet" => "อีเทอร์เน็ต",
        "Wi-Fi" => "ไวไฟ",
        "Loopback" => "ลูปแบ็ก",
        "Bluetooth" => "บลูทูธ",
        "Radio" => "วิทยุ",
        "No radio in this machine" => "เครื่องนี้ไม่มีวิทยุ",
        "Radio blocked by firmware or a switch" => "วิทยุถูกปิดโดยเฟิร์มแวร์หรือสวิตช์",
        "Radio present · no pairing service in this image" => "มีวิทยุแต่ไม่มีบริการจับคู่ในอิมเมจนี้",
        "Adapter" => "อะแดปเตอร์",
        "Pairing service" => "บริการจับคู่",
        "Installed" => "ติดตั้งแล้ว",
        "Not installed in this image" => "ไม่ได้ติดตั้งในอิมเมจนี้",
        "Sound cards" => "การ์ดเสียง",
        "No sound card reported by the kernel" => "เคอร์เนลไม่พบการ์ดเสียง",
        "Sound server" => "เซิร์ฟเวอร์เสียง",
        "Volume and routing" => "ระดับเสียงและการส่งสัญญาณ",
        "Managed by the sound service" => "จัดการโดยบริการเสียง",
        "Playback and capture" => "เล่นและบันทึกเสียง",
        "Mouse" => "เมาส์",
        "Tablet" => "แท็บเล็ต",
        "Input device" => "อุปกรณ์ป้อนข้อมูล",
        "Connection" => "การเชื่อมต่อ",
        "Address source" => "แหล่งที่อยู่เครือข่าย",
        "Measured from the active system state" => "อ่านจากสถานะระบบปัจจุบัน",
        "Network changes" => "การตั้งค่าเครือข่าย",
        "Managed by system services" => "จัดการโดยบริการระบบ",
        "No battery reading" => "ไม่มีข้อมูลแบตเตอรี่",
        "Battery" => "แบตเตอรี่",
        "Power profile" => "รูปแบบการใช้พลังงาน",
        "Suspend / restart / power off" => "พัก / เริ่มระบบใหม่ / ปิดเครื่อง",
        "Confirmation goes through litric-sysd" => "ส่งการยืนยันผ่าน litric-sysd",
        "Active" => "กำลังใช้งาน",
        "High contrast themes are available in Appearance." => "เลือกธีมคอนทราสต์สูงได้ในรูปลักษณ์",
        "Audio" => "เสียง",
        "No audio state is currently exposed to the shell" => "ระบบยังไม่ส่งสถานะเสียงให้เชลล์",
        "Browser permissions" => "สิทธิ์เบราว์เซอร์",
        "Per-site prompts are not connected yet" => "ยังไม่ได้เชื่อมคำขอสิทธิ์แยกตามเว็บไซต์",
        "File access" => "การเข้าถึงไฟล์",
        "Uses the user's home sandbox" => "ใช้พื้นที่ไฟล์ส่วนตัวของผู้ใช้",
        "Current user" => "ผู้ใช้ปัจจุบัน",
        "Password verification" => "การตรวจสอบรหัสผ่าน",
        "Lock status" => "สถานะล็อกหน้าจอ",
        "Refused" => "ถูกปฏิเสธ",
        "Enabled when a verifier is available" => "ใช้ได้เมื่อมีบริการตรวจสอบรหัสผ่าน",
        "Desktop state" => "สถานะเดสก์ท็อป",
        "Facts source" => "แหล่งข้อมูลระบบ",
        "Text renderer" => "ตัววาดข้อความ",
        "OpenType shaping · offline" => "จัดวางตัวอักษร OpenType · ออฟไลน์",
        "Vector fallback · system fonts unavailable" => "ใช้ฟอนต์เวกเตอร์สำรอง · ไม่พบฟอนต์ระบบ",
        "Substituted face:" => "ฟอนต์ที่ใช้แทน:",
        "No threats found" => "ไม่พบภัยคุกคาม",
        "Scan now" => "สแกนตอนนี้",
        "Scanning" => "กำลังสแกน",
        "RESULTS" => "ผลการตรวจสอบ",
        "Trash is empty" => "ถังขยะว่าง",
        "Search trash" => "ค้นหาในถังขยะ",
        "Empty permanently?" => "ล้างถังขยะถาวรหรือไม่?",
        "Delete forever?" => "ลบถาวรหรือไม่?",
        "Delete" => "ลบ",
        "No items match this search" => "ไม่พบรายการที่ตรงกับคำค้น",
        "Clear" => "ล้างหน้าจอ",
        "New session" => "เซสชันใหม่",
        "Command session" => "เซสชันคำสั่ง",
        "Process running" => "กำลังทำงาน",
        "Ready" => "พร้อม",
        "Application" => "แอปพลิเคชัน",
        "All applications" => "แอปทั้งหมด",
        "Lock screen" => "ล็อกหน้าจอ",
        "Search results" => "ผลการค้นหา",
        "No application matches that" => "ไม่พบแอปที่ตรงกัน",
        "Unknown application" => "แอปที่ไม่รู้จัก",
        "Minimized" => "ย่อหน้าต่างแล้ว",
        "Empty" => "ล้างถังขยะ",
        "Restore" => "กู้คืน",
        "Processes" => "โปรเซส",
        "CPU" => "หน่วยประมวลผล",
        "RAM" => "หน่วยความจำ",
        "No engine session yet: right-click the page → Start engine session, and litric-browserd renders this address." => "ยังไม่มีเซสชันเบราว์เซอร์: คลิกขวาแล้วเลือกเริ่มเซสชัน",
        _ => return Cow::Borrowed(english),
    };
    Cow::Borrowed(thai)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_and_thai_catalogs_are_selectable() {
        set_current(Language::English);
        assert_eq!(text("Settings"), "Settings");
        set_current(Language::Thai);
        // Thai is returned only when a live face can draw it. The assertion is
        // written against the capability rather than against a literal so that
        // it holds on both a machine with the face and one without, instead of
        // skipping on the machine the developer is actually using.
        let translated = text("Settings") == "การตั้งค่า";
        assert_eq!(translated, Language::Thai.renderable());
        set_current(Language::English);
    }

    #[test]
    fn a_language_with_no_face_is_not_printed_as_boxes() {
        // The defect this guards: with Thai selected and no Thai face installed,
        // every label in the shell was a row of empty boxes, and the desktop
        // read as a wireframe. Drawing a box is the right signal for one missing
        // character and the wrong signal for a whole interface.
        let drawable = crate::font::draws_thai();
        assert_eq!(Language::Thai.renderable(), drawable);
        assert!(Language::English.renderable(), "Latin is required to load");

        set_current(Language::Thai);
        let settings = text("Settings");
        if drawable {
            assert_eq!(settings, "การตั้งค่า");
        } else {
            assert_eq!(
                settings, "Settings",
                "Thai copy was emitted with no Thai face loaded"
            );
            assert!(
                Language::Thai.label().contains("not installed"),
                "the picker must say why Thai is not offered: {:?}",
                Language::Thai.label()
            );
        }
        set_current(Language::English);
        assert_eq!(text("Settings"), "Settings", "English is never translated");
    }

    #[test]
    fn language_tokens_round_trip() {
        for language in [Language::Thai, Language::English] {
            assert_eq!(Language::parse(language.token()), Some(language));
        }
    }
}
