//! Getting bytes off the devices and events into the desktop.
//!
//! One thread per device, each doing a blocking `read()`, feeding a channel the
//! main loop waits on. That shape is chosen deliberately over the alternatives:
//!
//! * **Polling** with `O_NONBLOCK` would burn a wake-up per device per frame
//!   even when nobody touches the machine, which is the opposite of what
//!   Adaptive Idle is for.
//! * **`epoll`** would be one thread and no polling, and needs `unsafe` or a
//!   crate. Neither is available, and the cost of the thread-per-device shape
//!   is a few kilobytes of stack for a handful of devices.
//!
//! The main loop blocks on the channel with the frame budget as its timeout, so
//! an idle desktop is genuinely asleep in `recv_timeout` and a keypress wakes it
//! immediately rather than at the next tick.

use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;
use std::{fs, io};

use crate::device::{self, DeviceInfo};
use crate::evdev::{parse_stream, Abi, RawEvent};
use crate::event::InputEvent;
use crate::keys::Layout;
use crate::state::InputState;

/// Read buffer per device. Sixty-four records: a fast mouse produces about a
/// thousand a second, so this is a comfortable frame's worth and the buffer
/// never grows.
const READ_BUFFER: usize = 64 * 24;

/// What the input stack managed to open.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendReport {
    /// Devices found, whether or not they opened.
    pub found: Vec<DeviceInfo>,
    /// Devices actually opened for reading.
    pub opened: Vec<DeviceInfo>,
    /// Why each unopened device failed, for the diagnostics.
    pub refused: Vec<(DeviceInfo, String)>,
    /// The absolute range learned from the opened devices via EVIOCGABS,
    /// per axis; 0 where nothing was learned and the default applies.
    pub abs_max: (f32, f32),
}

impl BackendReport {
    /// Whether anything can type.
    #[must_use]
    pub fn has_keyboard(&self) -> bool {
        self.opened.iter().any(|d| d.kinds.keyboard)
    }

    /// Whether anything can point.
    #[must_use]
    pub fn has_pointer(&self) -> bool {
        self.opened.iter().any(|d| d.kinds.pointer || d.kinds.touch)
    }

    /// Lines for the status file and the console.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![
            format!("input.found={}", self.found.len()),
            format!("input.opened={}", self.opened.len()),
            format!("input.keyboard={}", self.has_keyboard()),
            format!("input.pointer={}", self.has_pointer()),
        ];
        // The device's own word about its range: printed once at boot, so
        // any future coordinate-scale bug can be diagnosed from the serial
        // log instead of reverse-engineered from where clicks land.
        if self.abs_max.0 > 0.0 || self.abs_max.1 > 0.0 {
            out.push(format!(
                "input.absmax={:.0}x{:.0}",
                self.abs_max.0, self.abs_max.1
            ));
        }
        for d in &self.opened {
            out.push(format!("input.device={}", d.summary()));
        }
        for (d, why) in &self.refused {
            out.push(format!("input.refused={} ({why})", d.path.display()));
        }
        out
    }
}

/// A source of desktop input events.
///
/// The desktop holds this rather than a device, so a future libinput or Wayland
/// backend replaces one implementation without the shell noticing.
pub trait InputSource {
    /// Wait up to `timeout` for input, then return everything available.
    ///
    /// Returns an empty vector when nothing arrived, which is the normal idle
    /// case and not an error.
    fn poll(&mut self, timeout: Duration) -> Vec<InputEvent>;

    /// The accumulated state: pointer position, modifiers.
    fn state(&self) -> &InputState;

    /// Mutable state, for warping the pointer or changing layout.
    fn state_mut(&mut self) -> &mut InputState;
}

/// The record type a [`Waker`] sends.
///
/// Not an evdev type: the kernel's event types stop at `EV_MAX` (0x1f), so no
/// device can ever produce it, and [`EvdevSource::poll`] drops it before the
/// input state sees it.
pub const EV_WAKE: u16 = 0xFFFF;

/// Wakes a thread blocked in [`EvdevSource::poll`] from another thread.
///
/// The desktop's loop sleeps on the input channel. Work that arrives from
/// somewhere other than a device — a browser frame, file-operation progress —
/// has to be able to end that sleep, or it waits for the next keypress or the
/// next timer. Waking through the same channel keeps one blocking point and
/// adds no polling: an idle desktop still makes no wake-ups at all.
#[derive(Debug, Clone)]
pub struct Waker {
    tx: Sender<RawEvent>,
}

impl Waker {
    /// End the current (or next) `poll` early. Returns `false` once the
    /// source it belongs to has been dropped.
    pub fn wake(&self) -> bool {
        self.tx
            .send(RawEvent {
                sec: 0,
                usec: 0,
                kind: EV_WAKE,
                code: 0,
                value: 0,
            })
            .is_ok()
    }
}

/// The evdev backend: real devices, real threads.
#[derive(Debug)]
pub struct EvdevSource {
    rx: Receiver<RawEvent>,
    state: InputState,
    report: BackendReport,
    /// Kept so the channel stays open while the reader threads run, and cloned
    /// into every [`Waker`]. The threads are detached deliberately: they block
    /// in `read()` and will not notice a shutdown until the next event, which
    /// for a keyboard may be never. They exit with the process.
    tx: Sender<RawEvent>,
}

impl EvdevSource {
    /// Open every usable device under `root`.
    ///
    /// Never fails as a whole. A machine with no keyboard should still get a
    /// desktop it can look at, and a machine with no input at all should say so
    /// rather than refuse to start — so the failures are collected into the
    /// report instead of being returned as an error.
    #[must_use]
    pub fn open(root: &Path, width: f32, height: f32) -> Self {
        let found = device::discover(root);
        let (tx, rx) = mpsc::channel();
        let mut opened = Vec::new();
        let mut refused = Vec::new();
        // The largest absolute range seen across opened devices. Never
        // guessed: each absolute device reports its own via EVIOCGABS, and
        // the state adopts it so a click lands where it was aimed on any
        // hardware, not just on the one tablet the default encodes.
        let mut learned = (0.0f32, 0.0f32);

        for dev in found.iter().filter(|d| d.kinds.is_useful()) {
            match fs::File::open(&dev.path) {
                Ok(file) => {
                    let (qx, qy) = absinfo::query(&file);
                    learned.0 = learned.0.max(qx);
                    learned.1 = learned.1.max(qy);
                    let sender = tx.clone();
                    let name = dev.name.clone();
                    match thread::Builder::new()
                        .name(format!("litric-input:{name}"))
                        .stack_size(64 * 1024)
                        .spawn(move || pump(file, &sender))
                    {
                        Ok(_handle) => opened.push(dev.clone()),
                        Err(e) => refused.push((dev.clone(), format!("thread: {e}"))),
                    }
                }
                Err(e) => refused.push((dev.clone(), describe_io(&e))),
            }
        }

        let mut state = InputState::new(width, height);
        state.set_abs_max(learned.0, learned.1);
        Self {
            rx,
            state,
            report: BackendReport {
                found,
                opened,
                refused,
                abs_max: learned,
            },
            tx,
        }
    }

    /// A handle that can end a blocked [`InputSource::poll`] from another
    /// thread. The wake produces no input event.
    #[must_use]
    pub fn waker(&self) -> Waker {
        Waker {
            tx: self.tx.clone(),
        }
    }

    /// What was found and opened.
    #[must_use]
    pub const fn report(&self) -> &BackendReport {
        &self.report
    }

    /// Change the keyboard layout, e.g. when the locale changes.
    pub fn set_layout(&mut self, layout: Layout) {
        self.state.set_layout(layout);
    }
}

fn describe_io(e: &io::Error) -> String {
    match e.kind() {
        io::ErrorKind::PermissionDenied => {
            "permission denied (needs read on /dev/input)".to_owned()
        }
        io::ErrorKind::NotFound => "no such device node".to_owned(),
        _ => e.to_string(),
    }
}

/// Asking a device for its own absolute range (EVIOCGABS).
///
/// The divisor for ABS→screen mapping must be the device's real maximum, and
/// only the device knows it. Guessing it wrong is not a cosmetic bug: with a
/// guessed divisor twice the real one, every click aimed at the dock landed
/// near the top-left corner and the desktop looked dead to automation. So
/// when a device is opened, the backend asks it directly and the answer is
/// adopted — the serial report then carries `input.absmax=` as the device's
/// own word, and a future range bug is a one-line diagnosis instead of a
/// week of reverse-engineering from where clicks land.
#[cfg(target_os = "linux")]
mod absinfo {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    /// `_IOR('E', 0x40 + axis, struct input_absinfo)` for x and y. The
    /// struct is six i32s (value, minimum, maximum, fuzz, flat, resolution),
    /// so the maximum sits at byte offset 8.
    const EVIOCGABS_X: u64 = 0x8018_4540;
    const EVIOCGABS_Y: u64 = 0x8018_4541;

    // The one sanctioned use of unsafe in this crate: a single read-only
    // ioctl that sysfs cannot answer. Everything else stays safe Rust.
    #[allow(unsafe_code)]
    extern "C" {
        fn ioctl(fd: i32, request: std::os::raw::c_ulong, ...) -> i32;
    }

    /// `(maximum_x, maximum_y)`; 0 for an axis the device does not answer
    /// for, which the caller treats as "keep the default".
    pub fn query(file: &File) -> (f32, f32) {
        let fd = file.as_raw_fd();
        (maximum(fd, EVIOCGABS_X), maximum(fd, EVIOCGABS_Y))
    }

    fn maximum(fd: i32, request: u64) -> f32 {
        let mut buf = [0u8; 24];
        #[allow(unsafe_code)]
        let rc = unsafe { ioctl(fd, request as std::os::raw::c_ulong, buf.as_mut_ptr()) };
        if rc < 0 {
            return 0.0;
        }
        let max = i32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if max > 0 {
            max as f32
        } else {
            0.0
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod absinfo {
    use std::fs::File;

    /// Non-Linux builds have no evdev to ask; the state's default applies.
    pub fn query(_file: &File) -> (f32, f32) {
        (0.0, 0.0)
    }
}

/// One device's reader thread.
///
/// Exits on any read error or when the receiver is gone. A device that is
/// unplugged returns an error from `read()`, so this is also the unplug path.
fn pump(mut file: fs::File, tx: &Sender<RawEvent>) {
    let abi = Abi::native();
    let mut buf = vec![0u8; READ_BUFFER];
    let mut carry: Vec<u8> = Vec::new();
    loop {
        let n = match file.read(&mut buf) {
            Ok(0) => return,
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return,
        };
        let Some(chunk) = buf.get(..n) else {
            return;
        };
        carry.extend_from_slice(chunk);
        let (events, used) = parse_stream(abi, &carry);
        carry.drain(..used.min(carry.len()));
        for e in events {
            if tx.send(e).is_err() {
                return;
            }
        }
        // A stream that never yields a whole record is a stream that is not
        // evdev. Refusing to grow without bound turns that into a dead device
        // rather than a dead machine.
        if carry.len() > READ_BUFFER {
            return;
        }
    }
}

impl InputSource for EvdevSource {
    fn poll(&mut self, timeout: Duration) -> Vec<InputEvent> {
        let mut out = Vec::new();
        match self.rx.recv_timeout(timeout) {
            // A wake record is not input. It must not reach the state machine,
            // which would take its zero timestamp as the current time and break
            // double-click detection.
            Ok(raw) if raw.kind == EV_WAKE => {}
            Ok(raw) => out.extend(self.state.feed(raw)),
            Err(RecvTimeoutError::Timeout) => return out,
            Err(RecvTimeoutError::Disconnected) => return out,
        }
        // Drain whatever else is queued: a mouse produces a burst per frame and
        // handling one record per frame would make the pointer crawl.
        while let Ok(raw) = self.rx.try_recv() {
            if raw.kind != EV_WAKE {
                out.extend(self.state.feed(raw));
            }
        }
        out
    }

    fn state(&self) -> &InputState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut InputState {
        &mut self.state
    }
}

/// An input source that produces whatever it is given.
///
/// Used by the tests and by headless mode, where there is no device but the
/// desktop should still run and say why it is not interactive.
#[derive(Debug)]
pub struct ScriptedSource {
    queue: Vec<RawEvent>,
    state: InputState,
}

impl ScriptedSource {
    /// A source that will replay these records, in order.
    #[must_use]
    pub fn new(width: f32, height: f32, events: Vec<RawEvent>) -> Self {
        let mut queue = events;
        queue.reverse();
        Self {
            queue,
            state: InputState::new(width, height),
        }
    }

    /// An empty source: a machine with no input devices.
    #[must_use]
    pub fn empty(width: f32, height: f32) -> Self {
        Self::new(width, height, Vec::new())
    }

    /// Add more records to replay.
    pub fn push(&mut self, events: &[RawEvent]) {
        for e in events.iter().rev() {
            self.queue.insert(0, *e);
        }
    }

    /// Whether anything is left.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl InputSource for ScriptedSource {
    fn poll(&mut self, _timeout: Duration) -> Vec<InputEvent> {
        let mut out = Vec::new();
        while let Some(raw) = self.queue.pop() {
            let sync = raw.is_sync();
            out.extend(self.state.feed(raw));
            if sync {
                break;
            }
        }
        out
    }

    fn state(&self) -> &InputState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut InputState {
        &mut self.state
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::evdev::{EV_KEY, EV_REL, EV_SYN, REL_X, SYN_REPORT, VALUE_DOWN};
    use crate::event::Point;
    use crate::keys::Key;

    fn ev(kind: u16, code: u16, value: i32) -> RawEvent {
        RawEvent {
            sec: 1,
            usec: 0,
            kind,
            code,
            value,
        }
    }

    #[test]
    fn a_scripted_source_replays_a_packet_at_a_time() {
        let mut s = ScriptedSource::new(
            800.0,
            600.0,
            vec![
                ev(EV_REL, REL_X, 10),
                ev(EV_SYN, SYN_REPORT, 0),
                ev(EV_KEY, 30, VALUE_DOWN),
                ev(EV_SYN, SYN_REPORT, 0),
            ],
        );
        let first = s.poll(Duration::from_millis(0));
        assert_eq!(first.len(), 1, "{first:?}");
        assert!(matches!(
            first.first(),
            Some(InputEvent::PointerMove { .. })
        ));

        let second = s.poll(Duration::from_millis(0));
        assert!(second.iter().any(|e| matches!(
            e,
            InputEvent::KeyDown {
                key: Key::Char('a'),
                ..
            }
        )));
        assert!(s.is_empty());
        assert!(s.poll(Duration::from_millis(0)).is_empty());
    }

    #[test]
    fn an_empty_source_polls_empty_forever() {
        let mut s = ScriptedSource::empty(640.0, 480.0);
        assert!(s.poll(Duration::from_millis(0)).is_empty());
        assert_eq!(s.state().pointer(), Point::new(320.0, 240.0));
    }

    #[test]
    fn opening_a_root_with_no_devices_reports_that_rather_than_failing() {
        // A machine with no input must still get a desktop, and the report has
        // to say why it is not interactive.
        let s = EvdevSource::open(Path::new("no-such-root-91cd"), 1280.0, 800.0);
        assert!(s.report().found.is_empty());
        assert!(!s.report().has_keyboard());
        assert!(!s.report().has_pointer());
        let lines = s.report().lines();
        assert!(
            lines.contains(&"input.keyboard=false".to_owned()),
            "{lines:?}"
        );
    }

    #[test]
    fn a_device_that_cannot_be_opened_is_reported_with_a_reason() {
        // The sysfs entry exists; the device node does not. That is exactly
        // what a permission problem or a race with udev looks like, and the
        // desktop must name it rather than silently having no keyboard.
        let root = std::env::temp_dir().join("litric-input-backend");
        let _ = fs::remove_dir_all(&root);
        let caps = root.join("sys/class/input/event0/device/capabilities");
        fs::create_dir_all(&caps).expect("dirs");
        fs::write(caps.join("ev"), "120013").expect("ev");
        fs::write(
            caps.join("key"),
            "402000000 3803078f800d001 feffffdfffefffff fffffffffffffffe",
        )
        .expect("key");
        fs::write(caps.join("rel"), "0").expect("rel");

        let s = EvdevSource::open(&root, 800.0, 600.0);
        assert_eq!(s.report().found.len(), 1);
        assert!(s.report().opened.is_empty());
        assert_eq!(s.report().refused.len(), 1);
        let lines = s.report().lines();
        assert!(
            lines.iter().any(|l| l.starts_with("input.refused=")),
            "{lines:?}"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_waker_ends_a_blocked_poll_without_producing_input() {
        // The desktop sleeps in poll(). A browser frame arriving on another
        // thread must end that sleep promptly and must not look like input.
        let mut s = EvdevSource::open(Path::new("no-such-root-wake-a"), 800.0, 600.0);
        let waker = s.waker();
        let t = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            waker.wake()
        });
        let started = std::time::Instant::now();
        let events = s.poll(Duration::from_secs(20));
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the wake did not end the poll"
        );
        assert!(events.is_empty(), "a wake produced input: {events:?}");
        assert_eq!(t.join().ok(), Some(true), "the wake was not delivered");
        assert_eq!(
            s.state().pointer(),
            Point::new(400.0, 300.0),
            "the wake reached the input state"
        );
    }

    #[test]
    fn a_wake_between_real_records_is_dropped_and_the_records_survive() {
        let mut s = EvdevSource::open(Path::new("no-such-root-wake-b"), 800.0, 600.0);
        let waker = s.waker();
        s.tx.send(ev(EV_REL, REL_X, 10)).expect("send");
        assert!(waker.wake());
        s.tx.send(ev(EV_SYN, SYN_REPORT, 0)).expect("send");
        let events = s.poll(Duration::from_millis(100));
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            events.first(),
            Some(InputEvent::PointerMove { .. })
        ));
    }

    #[test]
    fn the_report_lists_what_opened() {
        let r = BackendReport {
            found: Vec::new(),
            opened: Vec::new(),
            refused: Vec::new(),
            abs_max: (0.0, 0.0),
        };
        assert_eq!(r.lines().first().map(String::as_str), Some("input.found=0"));
        // Nothing learned from a device: the range line is omitted rather
        // than printed as a lie.
        assert!(!r.lines().iter().any(|l| l.starts_with("input.absmax=")));
    }

    #[test]
    fn a_learned_range_is_reported_in_the_devices_own_words() {
        let r = BackendReport {
            found: Vec::new(),
            opened: Vec::new(),
            refused: Vec::new(),
            abs_max: (32767.0, 32767.0),
        };
        let lines = r.lines();
        assert!(
            lines.contains(&"input.absmax=32767x32767".to_owned()),
            "{lines:?}"
        );
    }
}
