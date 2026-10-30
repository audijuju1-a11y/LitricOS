//! Offline OpenType text layout for the LitricOS shell.
//!
//! Font files are read only from root-owned system font directories. Applications
//! never provide font bytes. HarfRust shapes the Unicode run and fontdue turns
//! the resulting glyph IDs into grayscale masks; the shell composites those
//! masks into its existing software canvas.

#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use fontdue::{Font, FontSettings};
use harfrust::{Buffer, Direction, ShapeOptions, ShaperFont};

const EM_SCALE: f32 = 1000.0;
/// How many shaped runs the cache keeps.
const CACHE_LIMIT: usize = 512;
/// How many bytes of glyph masks it keeps.
///
/// `CACHE_LIMIT` counts runs, and a run's size is text length times font size
/// times the glyphs a face produces for it -- so 512 short runs and 512 long
/// ones are the same "512" and differ by orders of magnitude in memory. This is
/// the bound that makes the cache's worst case a number instead of a function
/// of what the user typed: 8 MiB is a little under one 1280x800 frame, and it
/// still holds several hundred ordinary UI strings (`docs/PROGRESS-TRACKER.md`
/// §69 measures the per-glyph figure).
const CACHE_BYTE_LIMIT: usize = 8 * 1024 * 1024;

/// Selects one of the trusted installed faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// Noto Sans regular, with Noto Sans Thai fallback for Thai runs.
    Sans,
    /// Noto Sans bold, with Noto Sans Thai bold fallback for Thai runs.
    SansBold,
    /// Noto Sans Thai regular fallback face.
    Thai,
    /// Noto Sans Thai bold fallback face.
    ThaiBold,
    /// JetBrains Mono regular.
    Mono,
    /// JetBrains Mono bold.
    MonoBold,
}

/// One glyph mask, positioned relative to the baseline.
#[derive(Debug, Clone)]
pub struct Bitmap {
    /// X offset from the run origin.
    pub x: f32,
    /// Y offset from the baseline to the mask's top edge.
    pub y: f32,
    /// Mask width in pixels.
    pub width: usize,
    /// Mask height in pixels.
    pub height: usize,
    /// Grayscale coverage, row-major, 0 transparent and 255 opaque.
    pub coverage: Arc<[u8]>,
}

/// A shaped and rasterized text run.
#[derive(Debug, Clone, Default)]
pub struct Run {
    /// Positioned glyph masks.
    pub glyphs: Vec<Bitmap>,
    /// Horizontal advance in pixels, including tracking.
    pub width: f32,
}

struct LoadedFace {
    bytes: Arc<[u8]>,
    rasterizer: Font,
}

impl LoadedFace {
    /// Cap height as a fraction of the em, measured from this face.
    fn cap_em_ratio(&self) -> f32 {
        measure_cap_em_ratio(&self.rasterizer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RunKey {
    face: Face,
    size_tenths: u32,
    tracking_tenths: i32,
}

struct CachedRun {
    key: RunKey,
    text: String,
    run: Arc<Run>,
}

/// The bytes one cached run holds, as the allocations it owns.
///
/// Counted, not estimated: the entry itself, its copy of the text, the glyph
/// vector's capacity, and every mask's coverage buffer. Shared runs are counted
/// per entry because that is what the cache is holding; an `Arc` a caller still
/// holds is not the cache's bound to keep.
fn run_bytes(entry: &CachedRun) -> usize {
    let mut bytes = std::mem::size_of::<CachedRun>()
        .saturating_add(entry.text.capacity())
        .saturating_add(
            entry
                .run
                .glyphs
                .capacity()
                .saturating_mul(std::mem::size_of::<Bitmap>()),
        );
    for glyph in &entry.run.glyphs {
        bytes = bytes.saturating_add(glyph.coverage.len());
    }
    bytes
}

/// The bytes a cache holds.
fn cache_bytes(cache: &VecDeque<CachedRun>) -> usize {
    cache
        .iter()
        .fold(0_usize, |sum, entry| sum.saturating_add(run_bytes(entry)))
}

/// Evict from the oldest end until both bounds hold.
///
/// The last entry is never evicted: a single run larger than the byte limit is
/// still the answer to the question being asked, and dropping it would make the
/// cache miss forever on the one string that does not fit. The count bound is
/// applied after the byte bound so a cache of tiny runs still stops at 512.
fn evict_to(cache: &mut VecDeque<CachedRun>, byte_limit: usize) {
    while cache.len() > 1 && cache_bytes(cache) > byte_limit {
        let _ = cache.pop_back();
    }
    while cache.len() > CACHE_LIMIT {
        let _ = cache.pop_back();
    }
}

/// Cap height as a fraction of the em, for the faces the image installs.
///
/// The shell's type tokens are em sizes; its vector fallback is authored on a
/// cap-height grid. The fallback therefore has to be told what the cap is worth,
/// or one `Style` draws at two different sizes depending on which path is live.
/// Noto Sans puts the cap at 1462/2048 em; the value is *checked* against the
/// loaded face by [`TextSystem::cap_em_ratio`], which measures it.
pub const DOCUMENTED_CAP_EM_RATIO: f32 = 0.7139;

/// Which of the six faces actually came from disk.
///
/// Loading used to be all-or-nothing: one missing file (a Thai bold that a
/// distribution packaged under another name, a mono face the image pruned)
/// returned `Err`, and the whole desktop fell back to the vector typeface —
/// including the Latin text whose faces were sitting right there. This is what
/// was loaded, and [`TextSystem::degraded`] is what had to be substituted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Faces {
    /// `NotoSans-Regular`. Required: nothing loads without it.
    pub sans: bool,
    /// `NotoSans-Bold`.
    pub sans_bold: bool,
    /// `NotoSansThai-Regular`. Without it no Thai text can be drawn at all.
    pub thai: bool,
    /// `NotoSansThai-Bold`.
    pub thai_bold: bool,
    /// `JetBrainsMono-*` or another monospace face.
    pub mono: bool,
    /// The bold mono face.
    pub mono_bold: bool,
}

impl Faces {
    /// Whether every face came from disk, so nothing was substituted.
    #[must_use]
    pub const fn complete(self) -> bool {
        self.sans && self.sans_bold && self.thai && self.thai_bold && self.mono && self.mono_bold
    }

    /// Names of the faces that had to be substituted, in a fixed order.
    #[must_use]
    pub fn missing(self) -> Vec<&'static str> {
        [
            ("sans", self.sans),
            ("sans bold", self.sans_bold),
            ("Thai", self.thai),
            ("Thai bold", self.thai_bold),
            ("mono", self.mono),
            ("mono bold", self.mono_bold),
        ]
        .into_iter()
        .filter_map(|(name, present)| (!present).then_some(name))
        .collect()
    }
}

/// The installed faces and bounded run cache.
pub struct TextSystem {
    sans: LoadedFace,
    sans_bold: LoadedFace,
    thai: LoadedFace,
    thai_bold: LoadedFace,
    mono: LoadedFace,
    mono_bold: LoadedFace,
    faces: Faces,
    cap_em_ratio: f32,
    runs: Mutex<VecDeque<CachedRun>>,
}

/// Where the six faces are looked for, relative to a font root.
///
/// One table, shared by the system loader and by the development-host loader, so
/// a face cannot be found on the machine and missed on the reviewer's. The bare
/// file name is last in every list: that is the shape a directory of copied
/// faces has, and the review tools are the only caller that gets there.
const FACE_FILES: &[(&str, &[&str])] = &[
    (
        "sans",
        &[
            "noto/NotoSans-Regular.ttf",
            "TTF/NotoSans-Regular.ttf",
            "NotoSans-Regular.ttf",
        ],
    ),
    (
        "sans bold",
        &[
            "noto/NotoSans-Bold.ttf",
            "TTF/NotoSans-Bold.ttf",
            "NotoSans-Bold.ttf",
        ],
    ),
    (
        "Thai",
        &[
            "noto/NotoSansThai-Regular.ttf",
            "TTF/NotoSansThai-Regular.ttf",
            "NotoSansThai-Regular.ttf",
        ],
    ),
    (
        "Thai bold",
        &[
            "noto/NotoSansThai-Bold.ttf",
            "TTF/NotoSansThai-Bold.ttf",
            "NotoSansThai-Bold.ttf",
        ],
    ),
    (
        "mono",
        &[
            "TTF/JetBrainsMonoNerdFontMono-Regular.ttf",
            "TTF/JetBrainsMonoNerdFont-Regular.ttf",
            "TTF/JetBrainsMono-Regular.ttf",
            "noto/NotoSansMono-Regular.ttf",
            "JetBrainsMono-Regular.ttf",
        ],
    ),
    (
        "mono bold",
        &[
            "TTF/JetBrainsMonoNerdFontMono-Bold.ttf",
            "TTF/JetBrainsMonoNerdFont-Bold.ttf",
            "TTF/JetBrainsMono-Bold.ttf",
            "noto/NotoSansMono-Bold.ttf",
            "JetBrainsMono-Bold.ttf",
        ],
    ),
];

/// The roots the shipping image installs its faces under.
const SYSTEM_FONT_ROOTS: &[&str] = &["/usr/share/fonts"];

impl TextSystem {
    /// Loads the expected system faces from standard, root-owned font paths.
    ///
    /// The desktop image installs `noto-fonts` and
    /// `ttf-jetbrains-mono-nerd`. A face that is absent is *substituted*, not
    /// fatal: the desktop keeps drawing with the faces it has, and
    /// [`TextSystem::faces`] reports which were missing so the shell can say so
    /// instead of guessing. Only a total absence of Latin text fails the load.
    pub fn load_system() -> Result<Self, String> {
        Self::load_roots(SYSTEM_FONT_ROOTS)
    }

    /// Loads the same six faces from one directory tree.
    ///
    /// For the development host and the frame dump. A review that cannot run on
    /// the machine doing the work is not a review, and the alternative —
    /// reviewing a frame rendered in a different typeface — is worse than
    /// useless, because the reviewer is looking at the wrong word widths. The
    /// shipping shell never calls this: it reads the installed faces only, and
    /// an application still cannot supply one.
    pub fn load_dir<P: AsRef<Path>>(dir: P) -> Result<Self, String> {
        let dir = dir.as_ref();
        let root = dir
            .to_str()
            .ok_or_else(|| "font directory is not valid UTF-8".to_owned())?;
        Self::load_roots(&[root])
    }

    fn load_roots(roots: &[&str]) -> Result<Self, String> {
        let mut bytes: Vec<Option<Vec<u8>>> = Vec::with_capacity(FACE_FILES.len());
        let mut tried: Vec<String> = Vec::new();
        for (_name, names) in FACE_FILES {
            let mut found = None;
            for root in roots {
                for name in *names {
                    let path = Path::new(root).join(name);
                    tried.push(path.display().to_string());
                    if let Ok(data) = fs::read(&path) {
                        if valid_sfnt(&data) {
                            found = Some(data);
                            break;
                        }
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            bytes.push(found);
        }
        let [sans, sans_bold, thai, thai_bold, mono, mono_bold]: [Option<Vec<u8>>; 6] = bytes
            .try_into()
            .map_err(|_| format!("face table and loader disagree; tried {}", tried.join(", ")))?;
        let Some(sans) = sans else {
            return Err(format!(
                "no Latin text face is installed; tried {}",
                tried.join(", ")
            ));
        };
        Self::substituting(sans, sans_bold, thai, thai_bold, mono, mono_bold)
    }

    /// Creates a text system from explicitly supplied trusted font bytes.
    ///
    /// Every face is required: this is the path tests and the frame dump use,
    /// and a caller holding all six has nothing to substitute.
    pub fn from_bytes(
        sans: Vec<u8>,
        sans_bold: Vec<u8>,
        thai: Vec<u8>,
        thai_bold: Vec<u8>,
        mono: Vec<u8>,
        mono_bold: Vec<u8>,
    ) -> Result<Self, String> {
        Self::substituting(
            sans,
            Some(sans_bold),
            Some(thai),
            Some(thai_bold),
            Some(mono),
            Some(mono_bold),
        )
    }

    /// Builds the system, filling absent faces with the closest one present.
    fn substituting(
        sans: Vec<u8>,
        sans_bold: Option<Vec<u8>>,
        thai: Option<Vec<u8>>,
        thai_bold: Option<Vec<u8>>,
        mono: Option<Vec<u8>>,
        mono_bold: Option<Vec<u8>>,
    ) -> Result<Self, String> {
        let sans = load_face(sans)?;
        let faces = Faces {
            sans: true,
            sans_bold: sans_bold.is_some(),
            thai: thai.is_some(),
            thai_bold: thai_bold.is_some(),
            mono: mono.is_some(),
            mono_bold: mono_bold.is_some(),
        };
        // Substitution order, and the reason for each: a bold face falls back to
        // its regular sibling, script faces fall back to Latin, and a monospace
        // face falls back to proportional text. The last resort is always the
        // Latin face that had to be present for us to reach this point.
        let sans_bold = match sans_bold {
            Some(data) => load_face(data)?,
            None => reload(&sans)?,
        };
        let thai = match thai {
            Some(data) => load_face(data)?,
            None => reload(&sans)?,
        };
        let thai_bold = match thai_bold {
            Some(data) => load_face(data)?,
            None => reload(&thai)?,
        };
        let mono = match mono {
            Some(data) => load_face(data)?,
            None => reload(&sans)?,
        };
        let mono_bold = match mono_bold {
            Some(data) => load_face(data)?,
            None => reload(&mono)?,
        };
        let cap_em_ratio = sans.cap_em_ratio();
        Ok(Self {
            sans,
            sans_bold,
            thai,
            thai_bold,
            mono,
            mono_bold,
            faces,
            cap_em_ratio,
            runs: Mutex::new(VecDeque::new()),
        })
    }

    /// Which faces came from disk and which were substituted.
    #[must_use]
    pub const fn faces(&self) -> Faces {
        self.faces
    }

    /// Whether a face that can draw Thai script is loaded.
    ///
    /// Not a stylistic question: without it every Thai character is a missing
    /// glyph, and a screen of missing glyphs is a screen of boxes.
    #[must_use]
    pub const fn draws_thai(&self) -> bool {
        self.faces.thai
    }

    /// Cap height as a fraction of the em, measured from the Latin face.
    #[must_use]
    pub const fn cap_em_ratio(&self) -> f32 {
        self.cap_em_ratio
    }

    /// Shapes and rasterizes a run at physical pixel size.
    pub fn layout(&self, text: &str, face: Face, size: f32, tracking: f32) -> Option<Arc<Run>> {
        if !size.is_finite() || size <= 0.0 || text.is_empty() {
            return Some(Arc::new(Run::default()));
        }
        let size_tenths = (size * 10.0).round().clamp(1.0, 655_350.0) as u32;
        let tracking_tenths = (tracking * 10.0).round().clamp(-327_680.0, 327_670.0) as i32;
        let key = RunKey {
            face,
            size_tenths,
            tracking_tenths,
        };
        if let Some(run) = self.find_cached(key, text) {
            return Some(run);
        }

        if matches!(face, Face::Sans | Face::SansBold) {
            let mut combined = Run::default();
            let mut current_face = None;
            let mut start = 0;
            for (index, ch) in text
                .char_indices()
                .chain(std::iter::once((text.len(), '\0')))
            {
                if index == text.len() {
                    if start < index {
                        self.append_segment(
                            &mut combined,
                            &text[start..index],
                            current_face?,
                            size_tenths,
                            tracking_tenths,
                        )?;
                    }
                    break;
                }
                let thai = ('\u{0e00}'..='\u{0e7f}').contains(&ch);
                let selected = match (face, thai) {
                    (Face::Sans, true) => Face::Thai,
                    (Face::SansBold, true) => Face::ThaiBold,
                    (f, false) => f,
                    _ => face,
                };
                if current_face.is_some_and(|old| old != selected) {
                    self.append_segment(
                        &mut combined,
                        &text[start..index],
                        current_face?,
                        size_tenths,
                        tracking_tenths,
                    )?;
                    start = index;
                }
                current_face = Some(selected);
            }
            let result = Arc::new(combined);
            self.remember(key, text, Arc::clone(&result));
            return Some(result);
        }
        let result = self.layout_single(text, face, size_tenths, tracking_tenths)?;
        self.remember(key, text, Arc::clone(&result));
        Some(result)
    }

    fn append_segment(
        &self,
        into: &mut Run,
        text: &str,
        face: Face,
        size: u32,
        tracking: i32,
    ) -> Option<()> {
        let part = self.layout_single(text, face, size, tracking)?;
        for glyph in &part.glyphs {
            let mut shifted = glyph.clone();
            shifted.x += into.width;
            into.glyphs.push(shifted);
        }
        into.width += part.width;
        Some(())
    }

    fn layout_single(
        &self,
        text: &str,
        face: Face,
        size_tenths: u32,
        tracking_tenths: i32,
    ) -> Option<Arc<Run>> {
        let loaded = self.face(face);
        let source: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(Arc::clone(&loaded.bytes));
        let font = harfrust::font::Font::new(source, 0)?;
        let shaper = ShaperFont::new(&font).with_scale(EM_SCALE as i32);
        let mut buffer = Buffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.set_script(None);
        buffer.set_language(None);
        harfrust::shape(&shaper, &mut buffer, ShapeOptions::new()).ok()?;

        let size = size_tenths as f32 / 10.0;
        let tracking = tracking_tenths as f32 / 10.0;
        let mut run = Run::default();
        let mut pen_x = 0.0_f32;
        let infos = buffer.glyph_infos();
        let positions = buffer.glyph_positions();
        for (index, (info, position)) in infos.iter().zip(positions.iter()).enumerate() {
            let glyph_id = u16::try_from(info.glyph_id).ok()?;
            let (metrics, pixels) = loaded.rasterizer.rasterize_indexed(glyph_id, size);
            let scale = size / EM_SCALE;
            let x = pen_x + (position.x_offset as f32 * scale) + metrics.xmin as f32;
            let y =
                -((position.y_offset as f32 * scale) + metrics.ymin as f32 + metrics.height as f32);
            if metrics.width > 0 && metrics.height > 0 {
                let expected = metrics.width.checked_mul(metrics.height)?;
                if pixels.len() == expected {
                    run.glyphs.push(Bitmap {
                        x,
                        y,
                        width: metrics.width,
                        height: metrics.height,
                        coverage: Arc::from(pixels),
                    });
                }
            }
            pen_x += position.x_advance as f32 * scale;
            if index.saturating_add(1) < infos.len() {
                pen_x += tracking;
            }
        }
        run.width = pen_x.max(0.0);
        Some(Arc::new(run))
    }

    fn face(&self, face: Face) -> &LoadedFace {
        match face {
            Face::Sans => &self.sans,
            Face::SansBold => &self.sans_bold,
            Face::Thai => &self.thai,
            Face::ThaiBold => &self.thai_bold,
            Face::Mono => &self.mono,
            Face::MonoBold => &self.mono_bold,
        }
    }

    fn find_cached(&self, key: RunKey, text: &str) -> Option<Arc<Run>> {
        let mut cache = self.runs.lock().ok()?;
        let index = cache
            .iter()
            .position(|entry| entry.key == key && entry.text == text)?;
        let entry = cache.remove(index)?;
        let run = Arc::clone(&entry.run);
        cache.push_front(entry);
        Some(run)
    }

    fn remember(&self, key: RunKey, text: &str, run: Arc<Run>) {
        let Ok(mut cache) = self.runs.lock() else {
            return;
        };
        if let Some(index) = cache
            .iter()
            .position(|entry| entry.key == key && entry.text == text)
        {
            let _ = cache.remove(index);
        }
        cache.push_front(CachedRun {
            key,
            text: text.to_owned(),
            run,
        });
        evict_to(&mut cache, CACHE_BYTE_LIMIT);
    }

    /// How many runs the cache holds, and how many bytes they hold.
    #[must_use]
    pub fn cache_stats(&self) -> (usize, usize) {
        match self.runs.lock() {
            Ok(cache) => (cache.len(), cache_bytes(&cache)),
            Err(_) => (0, 0),
        }
    }

    /// The bound [`TextSystem::cache_stats`] is held to, for a caller that wants
    /// to print it beside the figure.
    #[must_use]
    pub const fn cache_byte_limit() -> usize {
        CACHE_BYTE_LIMIT
    }
}

fn valid_sfnt(bytes: &[u8]) -> bool {
    bytes.len() >= 12
        && bytes.get(0..4).is_some_and(|sig| {
            sig == b"\0\x01\0\0" || sig == b"OTTO" || sig == b"true" || sig == b"typ1"
        })
}

/// Rebuilds a face from bytes another face is already holding.
///
/// Used when a face has to be substituted: `fontdue::Font` is not `Clone`, and
/// the alternative — keeping a copy of every byte string for a case that only
/// happens on a machine missing an installed file — is a page of memory spent on
/// the wrong thing.
fn reload(face: &LoadedFace) -> Result<LoadedFace, String> {
    load_face(face.bytes.to_vec())
}

fn load_face(bytes: Vec<u8>) -> Result<LoadedFace, String> {
    let bytes: Arc<[u8]> = Arc::from(bytes);
    let rasterizer = Font::from_bytes(Arc::clone(&bytes), FontSettings::default())
        .map_err(|error| format!("fontdue rejected an installed OpenType face: {error:?}"))?;
    Ok(LoadedFace { bytes, rasterizer })
}

/// Cap height as a fraction of the em, for this face.
///
/// Measured, not assumed: rasterise `H` — which has flat top and bottom and so
/// has exactly the cap height — at a large size and divide. A face that cannot
/// rasterise `H` reports the documented ratio rather than zero, because zero
/// would silently collapse every fallback glyph to nothing.
fn measure_cap_em_ratio(font: &Font) -> f32 {
    const PROBE: f32 = 512.0;
    let (metrics, _) = font.rasterize('H', PROBE);
    let ratio = metrics.height as f32 / PROBE;
    if ratio.is_finite() && ratio > 0.1 && ratio < 1.0 {
        ratio
    } else {
        DOCUMENTED_CAP_EM_RATIO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The installed faces, or a directory named by `LITRIC_FONT_DIR`.
    ///
    /// The variable is how the review tools point at a local copy of the
    /// image's faces on a host that does not have the packages. The tests read
    /// it too, so a developer can *run* the real-font assertions locally
    /// instead of watching them skip — a check that always skips is a check that
    /// never fails, which is how BUG-0005 happened.
    fn system() -> Option<TextSystem> {
        match std::env::var("LITRIC_FONT_DIR") {
            Ok(dir) => TextSystem::load_dir(dir).ok(),
            Err(_) => TextSystem::load_system().ok(),
        }
    }

    #[test]
    fn a_directory_with_no_latin_face_is_refused_with_the_paths_it_tried() {
        // The one failure that must stay fatal: with no Latin face there is no
        // text at all, and the message has to say where we looked.
        let error = TextSystem::load_dir("this-directory-does-not-exist")
            .err()
            .expect("a missing font root must fail");
        assert!(error.contains("NotoSans-Regular.ttf"), "unhelpful: {error}");
    }

    #[test]
    fn a_missing_script_face_is_substituted_rather_than_fatal() {
        // One absent file used to take the whole desktop down to the vector
        // fallback, mono face included. It must now degrade one face and say so.
        let Some(full) = system() else {
            return;
        };
        let sans = full.sans.bytes.to_vec();
        let bare = TextSystem::substituting(sans, None, None, None, None, None)
            .expect("a Latin face alone is enough to draw something");
        assert!(
            bare.faces().sans,
            "the face we passed must be reported present"
        );
        assert!(!bare.faces().thai, "no Thai face was supplied");
        assert!(
            !bare.draws_thai(),
            "without the Thai face the shell must not claim it can draw Thai"
        );
        assert_eq!(
            bare.faces().missing(),
            vec!["sans bold", "Thai", "Thai bold", "mono", "mono bold"],
            "the report has to name what was substituted"
        );
        assert!(
            !bare.faces().complete(),
            "a substituted face is not a complete install"
        );
        // Substitution must not change the geometry the shell measures with.
        assert_eq!(bare.cap_em_ratio(), full.cap_em_ratio());
    }

    #[test]
    fn the_documented_cap_ratio_matches_the_installed_face() {
        // The shell's vector fallback is authored on a cap-height grid and is
        // told this ratio; if the real face disagrees, every style drawn through
        // the fallback is a different size from the same style drawn through
        // OpenType, and the audits describe a UI the machine never shows.
        let Some(text) = system() else {
            return;
        };
        let measured = text.cap_em_ratio();
        assert!(
            (measured - DOCUMENTED_CAP_EM_RATIO).abs() < 0.02,
            "measured cap/em {measured} against documented {DOCUMENTED_CAP_EM_RATIO}"
        );
    }

    fn ink(run: &Run) -> u64 {
        run.glyphs
            .iter()
            .flat_map(|glyph| glyph.coverage.iter())
            .map(|px| u64::from(*px))
            .sum()
    }

    #[test]
    fn latin_regular_and_bold_have_distinct_real_weights() {
        let Some(text) = system() else {
            return;
        };
        let regular = text.layout("Precision", Face::Sans, 24.0, 0.0).unwrap();
        let bold = text.layout("Precision", Face::SansBold, 24.0, 0.0).unwrap();
        assert_ne!(ink(&regular), ink(&bold));
    }

    #[test]
    fn mixed_english_and_thai_uses_shaped_thai_fallback() {
        let Some(text) = system() else {
            return;
        };
        let thai = text.layout("ลิทริก", Face::Sans, 24.0, 0.0).unwrap();
        let direct = text.layout_single("ลิทริก", Face::Thai, 240, 0).unwrap();
        assert_eq!(thai.width, direct.width);
        assert_eq!(thai.glyphs.len(), direct.glyphs.len());

        let mixed = text.layout("LitricOS ลิทริก", Face::Sans, 24.0, 0.0).unwrap();
        let latin = text.layout("LitricOS ", Face::Sans, 24.0, 0.0).unwrap();
        assert!((mixed.width - (latin.width + thai.width)).abs() < 0.1);
        assert_eq!(mixed.glyphs.len(), latin.glyphs.len() + thai.glyphs.len());
    }

    #[test]
    #[allow(clippy::arithmetic_side_effects)]
    fn the_run_cache_is_capped_in_bytes_as_well_as_in_runs() {
        // No face is installed on this host, so real masks cannot be measured
        // here (see `docs/PROGRESS-TRACKER.md` §69). What can be measured is the
        // eviction the shell uses, driven through the same `run_bytes` and
        // `evict_to` the cache calls, with runs whose byte size is known: one
        // byte of coverage per mask pixel, 16x16 masks.
        let entry = |text: &str, glyphs: usize| CachedRun {
            key: RunKey {
                face: Face::Sans,
                size_tenths: 130,
                tracking_tenths: 0,
            },
            text: text.to_owned(),
            run: Arc::new(Run {
                glyphs: (0..glyphs)
                    .map(|_| Bitmap {
                        x: 0.0,
                        y: 0.0,
                        width: 16,
                        height: 16,
                        coverage: Arc::from(vec![0_u8; 16 * 16]),
                    })
                    .collect(),
                width: 100.0,
            }),
        };
        let one = entry("Hello, LitricOS", 8);
        let per_run = run_bytes(&one);
        let per_glyph = per_run as f64 / 8.0;
        println!(
            "run_bytes={per_run} bytes_per_glyph={per_glyph:.1} coverage_per_glyph=256 \
             count_limit={CACHE_LIMIT} byte_limit={CACHE_BYTE_LIMIT}"
        );
        assert!(
            per_glyph >= 256.0,
            "a glyph's mask must be counted: {per_glyph} bytes for a 256-byte mask"
        );

        // Forty runs, each the size of the one above, against a limit that holds
        // three of them. The cache must come out under the limit, oldest gone.
        let mut cache: VecDeque<CachedRun> = VecDeque::new();
        for i in 0..40 {
            cache.push_front(entry(&format!("run {i}"), 8));
        }
        let before = cache_bytes(&cache);
        let limit = per_run.saturating_mul(3).saturating_add(1);
        evict_to(&mut cache, limit);
        let after = cache_bytes(&cache);
        println!(
            "cache_before_bytes={before} cache_after_bytes={after} cache_limit={limit} \
             runs_kept={}",
            cache.len()
        );
        assert!(
            after <= limit,
            "the byte cap did not hold: {after} bytes against a {limit}-byte limit"
        );
        assert!(
            cache.len() >= 2,
            "the byte cap evicted every run, which would make the cache useless"
        );
        assert_eq!(
            cache.front().map(|e| e.text.as_str()),
            Some("run 39"),
            "eviction must take the oldest run, not the newest"
        );

        // The run count is still a bound for runs too small to reach the bytes.
        let mut many: VecDeque<CachedRun> = VecDeque::new();
        for i in 0..CACHE_LIMIT.saturating_add(50) {
            many.push_front(entry(&format!("t{i}"), 0));
        }
        evict_to(&mut many, CACHE_BYTE_LIMIT);
        assert_eq!(
            many.len(),
            CACHE_LIMIT,
            "the run bound must still stop a cache of tiny runs at {CACHE_LIMIT}"
        );
    }
}
