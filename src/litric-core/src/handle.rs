//! Opaque handles to system-owned objects.
//!
//! Per RUNTIME section 7 and section 5.2: applications reference kernel and
//! service resources through opaque handles, never through exposed structs or
//! guessable integers.
//!
//! Two properties matter and both are enforced by construction:
//!
//! 1. **A handle is meaningless outside its owning process.** The handle table
//!    is per-process; the same numeric value in two processes refers to two
//!    different things, or to nothing.
//! 2. **A handle must not become valid by guessing.** Handles carry a
//!    generation counter, so a stale handle to a closed object is detected as
//!    stale rather than silently resolving to whatever now occupies that slot.
//!    That is the difference between a clean `NOT_FOUND` and a use-after-close
//!    that hands one app another app's file.

use core::fmt;

/// What kind of object a handle refers to.
///
/// Type-tagging handles means a caller cannot pass a window handle where an
/// audio stream is expected and have it resolve to *something*. The service
/// rejects it as [`crate::ErrorKind::InvalidInput`] before any lookup happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum HandleKind {
    /// A process.
    Process = 1,
    /// A thread.
    Thread = 2,
    /// An open file.
    File = 3,
    /// An open directory.
    Directory = 4,
    /// A socket.
    Socket = 5,
    /// A window or surface.
    Window = 6,
    /// A GPU resource (buffer, texture, pipeline).
    GpuResource = 7,
    /// An audio stream.
    AudioStream = 8,
    /// A shared-memory region.
    SharedMemory = 9,
    /// An event or futex-like object.
    Event = 10,
    /// An IPC service endpoint.
    ServiceEndpoint = 11,
    /// A secure-store credential object.
    Credential = 12,
    /// A capability grant.
    Capability = 13,
}

impl HandleKind {
    /// Stable machine-readable token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::Thread => "thread",
            Self::File => "file",
            Self::Directory => "directory",
            Self::Socket => "socket",
            Self::Window => "window",
            Self::GpuResource => "gpu",
            Self::AudioStream => "audio",
            Self::SharedMemory => "shm",
            Self::Event => "event",
            Self::ServiceEndpoint => "endpoint",
            Self::Credential => "credential",
            Self::Capability => "capability",
        }
    }
}

impl fmt::Display for HandleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// An opaque, type-tagged, generation-checked handle.
///
/// Layout is an implementation detail and is **not** part of the ABI; only the
/// 64-bit opaque value that crosses the boundary is. The bit packing below can
/// change without recompiling applications, which is exactly why handles are
/// opaque in the first place.
///
/// ```text
///  63          56 55                    32 31                     0
/// +--------------+------------------------+------------------------+
/// |     kind     |      generation        |         index          |
/// |    8 bits    |        24 bits         |        32 bits         |
/// +--------------+------------------------+------------------------+
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle(u64);

impl Handle {
    const INDEX_BITS: u32 = 32;
    const GEN_BITS: u32 = 24;
    const GEN_SHIFT: u32 = Self::INDEX_BITS;
    const KIND_SHIFT: u32 = Self::INDEX_BITS + Self::GEN_BITS;
    const INDEX_MASK: u64 = (1u64 << Self::INDEX_BITS) - 1;
    const GEN_MASK: u64 = (1u64 << Self::GEN_BITS) - 1;

    /// The reserved invalid handle. Never returned by a successful call.
    pub const INVALID: Self = Self(0);

    /// Largest representable generation before wraparound.
    pub const MAX_GENERATION: u32 = (1u32 << Self::GEN_BITS) - 1;

    /// Build a handle from its parts.
    ///
    /// `generation` is masked to [`Self::MAX_GENERATION`]. Wraparound after 16.7
    /// million reuses of one slot is accepted: at that point the colliding
    /// handle would also have to survive in a caller that never noticed a close,
    /// which is a bug we would rather surface than silently paper over with a
    /// wider field.
    #[must_use]
    pub const fn new(kind: HandleKind, index: u32, generation: u32) -> Self {
        let k = (kind as u64) << Self::KIND_SHIFT;
        let g = ((generation as u64) & Self::GEN_MASK) << Self::GEN_SHIFT;
        let i = (index as u64) & Self::INDEX_MASK;
        Self(k | g | i)
    }

    /// Reconstruct from the opaque wire value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The opaque wire value.
    #[must_use]
    pub const fn to_raw(self) -> u64 {
        self.0
    }

    /// Slot index within the owning process's handle table.
    #[must_use]
    pub const fn index(self) -> u32 {
        (self.0 & Self::INDEX_MASK) as u32
    }

    /// Generation counter, used to detect a stale handle.
    #[must_use]
    pub const fn generation(self) -> u32 {
        ((self.0 >> Self::GEN_SHIFT) & Self::GEN_MASK) as u32
    }

    /// The object kind, or `None` if the tag is not one we know.
    ///
    /// Returns `Option` rather than panicking: the value came from a client and
    /// is therefore untrusted input, and untrusted input must never be able to
    /// abort a system service.
    #[must_use]
    pub const fn kind(self) -> Option<HandleKind> {
        match (self.0 >> Self::KIND_SHIFT) as u8 {
            1 => Some(HandleKind::Process),
            2 => Some(HandleKind::Thread),
            3 => Some(HandleKind::File),
            4 => Some(HandleKind::Directory),
            5 => Some(HandleKind::Socket),
            6 => Some(HandleKind::Window),
            7 => Some(HandleKind::GpuResource),
            8 => Some(HandleKind::AudioStream),
            9 => Some(HandleKind::SharedMemory),
            10 => Some(HandleKind::Event),
            11 => Some(HandleKind::ServiceEndpoint),
            12 => Some(HandleKind::Credential),
            13 => Some(HandleKind::Capability),
            _ => None,
        }
    }

    /// Whether this is the reserved invalid handle.
    #[must_use]
    pub const fn is_invalid(self) -> bool {
        self.0 == 0
    }

    /// Whether this handle claims to be of `expected` kind.
    ///
    /// The service calls this **before** touching its handle table, so a
    /// mistyped or forged handle costs a comparison rather than a lookup.
    #[must_use]
    pub fn is_kind(self, expected: HandleKind) -> bool {
        self.kind() == Some(expected)
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_invalid() {
            return f.write_str("Handle(INVALID)");
        }
        match self.kind() {
            Some(k) => write!(f, "Handle({}#{}.{})", k, self.index(), self.generation()),
            None => write!(f, "Handle(unknown-kind raw={:#018x})", self.0),
        }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_all_fields() {
        let h = Handle::new(HandleKind::File, 0x0123_4567, 42);
        assert_eq!(h.kind(), Some(HandleKind::File));
        assert_eq!(h.index(), 0x0123_4567);
        assert_eq!(h.generation(), 42);
        assert_eq!(Handle::from_raw(h.to_raw()), h);
    }

    #[test]
    fn boundary_values_survive() {
        let h = Handle::new(HandleKind::Capability, u32::MAX, Handle::MAX_GENERATION);
        assert_eq!(h.index(), u32::MAX);
        assert_eq!(h.generation(), Handle::MAX_GENERATION);
        assert_eq!(h.kind(), Some(HandleKind::Capability));

        let z = Handle::new(HandleKind::Process, 0, 0);
        assert_eq!(z.index(), 0);
        assert_eq!(z.generation(), 0);
        assert!(!z.is_invalid(), "kind tag alone must make it non-zero");
    }

    #[test]
    fn generation_overflow_wraps_without_corrupting_neighbours() {
        // Passing a generation larger than the field must mask, not bleed into
        // the kind tag. If it bled, a stale file handle could present as a
        // capability handle.
        let h = Handle::new(HandleKind::File, 7, Handle::MAX_GENERATION + 1);
        assert_eq!(
            h.kind(),
            Some(HandleKind::File),
            "kind tag corrupted by overflow"
        );
        assert_eq!(h.index(), 7, "index corrupted by overflow");
        assert_eq!(h.generation(), 0);
    }

    #[test]
    fn stale_generation_is_a_different_handle() {
        // This is the use-after-close defence: same slot, later generation, and
        // the two values must not compare equal.
        let old = Handle::new(HandleKind::File, 12, 3);
        let reused = Handle::new(HandleKind::File, 12, 4);
        assert_ne!(old, reused);
        assert_eq!(old.index(), reused.index());
    }

    #[test]
    fn unknown_kind_tag_does_not_panic() {
        // Untrusted input. A client can send any 64 bits it likes.
        for raw in [
            u64::MAX,
            0xFF00_0000_0000_0000,
            0x00FF_FFFF_FFFF_FFFF,
            0xDEAD_BEEF_CAFE_BABE,
        ] {
            let h = Handle::from_raw(raw);
            let _ = h.kind();
            let _ = h.index();
            let _ = h.generation();
            let _ = format!("{h:?}");
        }
        assert_eq!(Handle::from_raw(u64::MAX).kind(), None);
    }

    #[test]
    fn invalid_handle_is_zero_and_typed_checks_fail() {
        assert!(Handle::INVALID.is_invalid());
        assert_eq!(Handle::INVALID.kind(), None);
        assert!(!Handle::INVALID.is_kind(HandleKind::File));
    }

    #[test]
    fn kind_confusion_is_rejected() {
        let window = Handle::new(HandleKind::Window, 1, 1);
        assert!(window.is_kind(HandleKind::Window));
        assert!(!window.is_kind(HandleKind::AudioStream));
        assert!(!window.is_kind(HandleKind::Credential));
    }
}
