//! The LitricOS desktop.
//!
//! This is the composition itself: wallpaper, menu bar, a window, the
//! environment switcher and the dock. It is drawn from the tokens in
//! `litric-design` and filled with numbers measured by `litric-sysd`, so the
//! screen a user sees is the design system and the running system, not a
//! picture of them.
//!
//! # The glass is solved, not chosen
//!
//! Every translucent surface here calls
//! [`litric_design::Glass::adaptive_alpha_pct`] against the pixels actually
//! behind it, and uses the most transparent tint that keeps both text roles at
//! WCAG AA. Over the dark wallpaper below the panels settle around 18-40%,
//! which is the transparency the design was aiming for; over a bright wallpaper
//! the same code firms them up on its own. The alpha is reported in the status
//! file so it can be checked from outside rather than admired from inside.

//!
//! # Repainting part of the screen
//!
//! [`Compositor::repaint`] is the frame path the display loop uses. Glass
//! there samples a copy of the wallpaper that was blurred once, not the pixels
//! already on screen: re-blurring whatever the last frame left behind a panel
//! is what made panels darken a little more with every pointer move.

use litric_design::{Glass, Rgb, Theme, ThemeKind};

use crate::canvas::{blend, luminance, rgb, rgba, with_alpha, Canvas, Rect, Repaint};
use crate::font::{self, Style};
use crate::shell::Shell;
use crate::views;
use litric_input::event::Point;

/// WCAG AA for body text. The target the glass solver must meet.
const AA: f64 = 4.5;

/// Everything the desktop puts on screen that came from the running system.
#[derive(Debug, Clone, Default)]
pub struct DesktopState {
    /// Total RAM in MiB.
    pub mem_total_mb: u64,
    /// RAM in use in MiB (total minus available).
    pub mem_used_mb: u64,
    /// Online CPUs.
    pub cpus: u32,
    /// Seconds since boot.
    pub uptime_secs: f64,
    /// Root filesystem type, e.g. `erofs`.
    pub root_fs: String,
    /// Whether dm-verity is protecting the root.
    pub verity: bool,
    /// Items in the trash, and their total size in bytes.
    pub trash: (usize, u64),
    /// Environments: display name, state, hotkey label, and whether active.
    pub environments: Vec<(String, String, String, bool)>,
    /// Where the numbers came from, for the status file and the footer.
    pub source: String,
    /// Wall-clock time as `HH:MM`, in UTC.
    pub clock: String,
}

/// Convert a design token colour to a canvas pixel.
#[must_use]
fn px(c: Rgb) -> u32 {
    rgba(c.r, c.g, c.b, 255)
}

/// Convert a canvas pixel back to a design colour, for the contrast solver.
#[must_use]
fn design(c: u32) -> Rgb {
    Rgb::new(
        ((c >> 16) & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        (c & 0xFF) as u8,
    )
}

/// A glass surface that has been placed on the canvas.
#[derive(Debug, Clone, Copy)]
pub struct GlassPanel {
    /// Where it is.
    pub rect: Rect,
    /// Corner radius used.
    pub radius: f32,
    /// The tint alpha the solver chose, in percent.
    pub alpha_pct: u8,
    /// The colour text is really being drawn on, after compositing.
    pub surface: u32,
}

/// Draw a glass panel: blur what is behind it, boost its colour, lay the tint
/// on at the solved alpha, then a hairline edge and a top highlight.
///
/// Returns what it decided, so the caller can report it and the tests can check
/// it without re-deriving anything.
pub fn glass(canvas: &mut Canvas, theme: &Theme, r: Rect, radius: f32) -> GlassPanel {
    let tint = theme.palette.glass_tint;
    let primary = theme.palette.text_primary;
    let secondary = theme.palette.text_secondary;

    // The worst backdrop under the panel, not the average. Text has to stay
    // readable over the brightest corner of the wallpaper, and an average
    // would quietly excuse exactly that corner. Searched in what the panel
    // will actually show: the attached backdrop when there is one.
    let backdrop = design(canvas.backdrop_brightest(r));
    let a1 = Glass::adaptive_alpha_pct(tint, primary, backdrop, AA);
    let a2 = Glass::adaptive_alpha_pct(tint, secondary, backdrop, AA);
    let alpha_pct = match (a1, a2) {
        (Some(a), Some(b)) => a.max(b),
        _ => theme.glass.tint_alpha_pct,
    };

    // Blur and saturate the backdrop, masked to the panel's own shape. Doing it
    // over the bounding rectangle instead leaves a visible box of shifted
    // colour around every panel, which is what the first render of this
    // desktop looked like.
    let (blur, saturation) = glass_params(theme);
    canvas.glass_backdrop(r, radius, blur, saturation);

    let alpha = f32::from(alpha_pct) / 100.0;
    canvas.fill_round_rect(r, radius, with_alpha(px(tint), alpha));

    // The hairline and the specular top edge are what stop a translucent
    // rectangle from reading as a hole in the wallpaper.
    let border = f32::from(theme.glass.border_alpha_pct) / 100.0;
    canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), border));
    let hl = f32::from(theme.glass.highlight_alpha_pct) / 100.0;
    canvas.fill_round_rect(
        Rect::new(r.x + radius * 0.6, r.y + 0.5, r.w - radius * 1.2, 1.0),
        0.5,
        with_alpha(rgb(0xFFFFFF), hl),
    );

    let surface = blend(px(backdrop), px(tint), alpha);
    GlassPanel {
        rect: r,
        radius,
        alpha_pct,
        surface,
    }
}

/// Draw a glass panel at an alpha the caller has already decided.
///
/// Used by materials that are deliberately firmer than adaptive glass —
/// a popover carrying dense text should not become more transparent just
/// because it happens to be over a dark part of the wallpaper.
pub fn glass_at(
    canvas: &mut Canvas,
    theme: &Theme,
    r: Rect,
    radius: f32,
    alpha_pct: u8,
) -> GlassPanel {
    let tint = theme.palette.glass_tint;
    let backdrop = design(canvas.backdrop_brightest(r));
    let (blur, saturation) = glass_params(theme);
    canvas.glass_backdrop(r, radius, blur, saturation);

    let alpha = f32::from(alpha_pct) / 100.0;
    canvas.fill_round_rect(r, radius, with_alpha(px(tint), alpha));
    let border = f32::from(theme.glass.border_alpha_pct) / 100.0;
    canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), border));

    GlassPanel {
        rect: r,
        radius,
        alpha_pct,
        surface: blend(px(backdrop), px(tint), alpha),
    }
}

/// The blur radius and saturation every glass surface of a theme uses.
///
/// One function, so the backdrop [`Compositor::new`] makes once and the live
/// path in [`glass`] cannot drift apart.
fn glass_params(theme: &Theme) -> (u32, f32) {
    let blur = u32::from(theme.glass.blur_px)
        .checked_div(3)
        .unwrap_or(1)
        .max(1);
    (blur, f32::from(theme.glass.saturation_pct) / 100.0)
}

/// The wallpaper, the frame drawn over it, and the repaint between them.
///
/// The display loop and the render benchmark both draw through
/// [`Compositor::repaint`], so what is measured is what runs.
#[derive(Debug, Clone)]
pub struct Compositor {
    /// The wallpaper, painted once. A repainted region starts as a copy of it.
    pub wallpaper: Canvas,
    /// The frame that is presented. Glass on it samples a blurred, saturated
    /// copy of the wallpaper made by [`Compositor::new`].
    pub scene: Canvas,
    /// The pointer as the frame holds it: the rectangle it covers and the
    /// pixels it was drawn over.
    ///
    /// A pointer move used to compose the whole desktop because the arrow is
    /// part of [`views::draw`]. Keeping the pixels underneath it makes the move
    /// a restore, a snapshot and one arrow: the twelve thousand pixels of a
    /// cursor instead of the million a frame visits.
    cursor: Option<(Rect, Vec<u32>)>,
}

impl Compositor {
    /// Paint the wallpaper for `theme`, make the glass backdrop from it once,
    /// and start the scene as a copy of the wallpaper with that backdrop
    /// attached.
    #[must_use]
    pub fn new(width: u32, height: u32, theme: &Theme, choice: Wallpaper) -> Self {
        let mut paper = Canvas::new(width, height, rgb(0x000000));
        wallpaper_for(&mut paper, theme, choice);
        let (blur, saturation) = glass_params(theme);
        // The shell composites single-threaded, so the shared backdrop is an
        // Rc: `Canvas` is neither Send nor Sync and never needs to be.
        let source = std::rc::Rc::new(paper.glass_source(blur, saturation));
        let mut scene = paper.clone();
        // The backdrop is made from a canvas of the scene's own size, so the
        // size check in set_backdrop cannot refuse it.
        let _ = scene.set_backdrop(Some(source));
        Self {
            wallpaper: paper,
            scene,
            cursor: None,
        }
    }

    /// Replace the cached wallpaper and glass backdrop after a theme change.
    pub fn retheme(&mut self, theme: &Theme, choice: Wallpaper) {
        let width = self.scene.width();
        let height = self.scene.height();
        let mut paper = Canvas::new(width, height, rgb(0x000000));
        wallpaper_for(&mut paper, theme, choice);
        let (blur, saturation) = glass_params(theme);
        let source = std::rc::Rc::new(paper.glass_source(blur, saturation));
        let mut scene = paper.clone();
        let _ = scene.set_backdrop(Some(source));
        self.wallpaper = paper;
        self.scene = scene;
        self.cursor = None;
    }

    /// Draw a frame: the whole screen when `full`, otherwise only the regions
    /// [`Repaint::plan`] makes of `damage`.
    ///
    /// Each region is restored from the wallpaper and the whole desktop is
    /// drawn over it with the clip set to that region, so every pixel inside
    /// it goes through the same drawing a full frame gives it and no pixel
    /// outside it is visited. Returns the plan, so the caller presents exactly
    /// the rows that were repainted.
    pub fn repaint(&mut self, shell: &mut Shell, full: bool, damage: &[Rect]) -> Repaint {
        let (w, h) = (self.scene.width(), self.scene.height());
        let plan = if full {
            Repaint::Full
        } else {
            Repaint::plan(damage, w, h)
        };
        self.cursor = None;
        for region in plan.rects(w, h) {
            let saved = self.scene.set_clip(Some(region));
            // A locked shell draws over every pixel it visits: the lock surface
            // lays down an opaque background before anything else, so the
            // wallpaper copy underneath it could not be seen. Skipping it is a
            // whole screen of copying removed from the one frame that is always
            // damage (BUG-0018: 1.02M of a locked frame's 2.93M pixels), and it
            // is invisible - `frame_dump`'s fingerprints for 11-lock and
            // 12-lock-password are unchanged by it.
            if !shell.lock.locked {
                self.scene.blit_from(&self.wallpaper, region);
            }
            views::draw(&mut self.scene, shell);
            self.scene.set_clip(saved);
        }
        self.paint_cursor(shell);
        shell.counted_frame();
        plan
    }

    /// Draw the pointer at its current position, keeping what it covers.
    ///
    /// The arrow is drawn here rather than in [`views::draw`] so that the
    /// compositor owns the pixels under it: that ownership is what makes
    /// [`Compositor::move_cursor`] possible.
    fn paint_cursor(&mut self, shell: &Shell) {
        self.cursor = None;
        let p = shell.pointer();
        let rect = crate::ui::cursor_bounds(p.x, p.y, shell.tokens.scale);
        let Some((rect, px)) = self.scene.snapshot(rect) else {
            return;
        };
        crate::ui::cursor(&mut self.scene, p.x, p.y, shell.tokens.scale);
        self.cursor = Some((rect, px));
    }

    /// Move the pointer across the frame without composing anything else.
    ///
    /// `from` and `to` are the pointer's old and new positions. The frame is
    /// only valid if the arrow is still exactly where the last frame left it --
    /// if it is not, `None` says so and the caller composes normally rather
    /// than leaving a trail of arrows behind. On success only the two cursor
    /// rectangles are repainted, which is the difference between a desktop that
    /// stays at its frame rate while the mouse moves and one that composes a
    /// whole scene per millimetre.
    pub fn move_cursor(&mut self, shell: &Shell, from: Point, to: Point) -> Option<Repaint> {
        let (saved_rect, saved_px) = self.cursor.as_ref()?;
        let from_rect = crate::ui::cursor_bounds(from.x, from.y, shell.tokens.scale);
        if *saved_rect != from_rect {
            return None;
        }
        let saved_rect = *saved_rect;
        let saved_px = saved_px.clone();
        self.scene.restore(saved_rect, &saved_px);
        let to_rect = crate::ui::cursor_bounds(to.x, to.y, shell.tokens.scale);
        let (rect, px) = self.scene.snapshot(to_rect)?;
        crate::ui::cursor(&mut self.scene, to.x, to.y, shell.tokens.scale);
        self.cursor = Some((rect, px));
        Some(Repaint::plan(
            &[from_rect, to_rect],
            self.scene.width(),
            self.scene.height(),
        ))
    }
}

/// The dark wallpaper: an original LitricOS PNG, embedded in the binary so it
/// cannot be missing at boot. Source: `assets/wallpapers/`.
const WAVE_PNG: &[u8] =
    include_bytes!("../../../assets/wallpapers/litric-obsidian-wave-1920x1080.png");

/// Decode an 8-bit RGB or RGBA PNG into opaque canvas pixels, row by row.
///
/// `None` for anything else, so an unreadable file falls back to a solid colour
/// instead of taking the shell down.
fn decode_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u32>)> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let channels: usize = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return None,
    };
    let count = (info.width as usize)
        .checked_mul(info.height as usize)?
        .checked_mul(channels)?;
    let pixels = buf
        .get(..count)?
        .chunks_exact(channels)
        .map(|c| match c {
            [r, g, b, ..] => rgba(*r, *g, *b, 255),
            _ => 0,
        })
        .collect();
    Some((info.width, info.height, pixels))
}

/// Paint an image over the whole canvas, scaled to cover it and centred.
///
/// Proportions are kept at every aspect ratio: the overflow is cropped, never
/// stretched.
fn paint_cover(canvas: &mut Canvas, iw: u32, ih: u32, pixels: &[u32]) {
    let (cw, ch) = (canvas.width(), canvas.height());
    let scale = (cw as f32 / iw as f32).max(ch as f32 / ih as f32);
    let ox = (iw as f32 * scale - cw as f32) / 2.0;
    let oy = (ih as f32 * scale - ch as f32) / 2.0;
    let last_x = iw.saturating_sub(1) as usize;
    let last_y = ih.saturating_sub(1) as usize;
    let stride = iw as usize;
    for y in 0..ch {
        let sy = (((y as f32 + 0.5 + oy) / scale) as usize).min(last_y);
        for x in 0..cw {
            let sx = (((x as f32 + 0.5 + ox) / scale) as usize).min(last_x);
            if let Some(&c) = pixels.get(sy.saturating_mul(stride).saturating_add(sx)) {
                canvas.put(i64::from(x), i64::from(y), c);
            }
        }
    }
}

/// Which wallpaper the desktop paints.
///
/// A choice rather than a path: the shell draws every one of these itself, so a
/// wallpaper cannot arrive from a directory as an executable image decoder's
/// problem (ADR-0009 keeps untrusted decoding out of the shell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wallpaper {
    /// What ships: the embedded obsidian wave on the dark themes, a solid pearl
    /// field on the light ones.
    #[default]
    System,
    /// One flat field of the theme's own base colour. No image, no wash.
    Flat,
    /// A restrained vertical wash from the elevated surface into the base
    /// colour, with a low accent glow in the top-right corner.
    Glow,
}

impl Wallpaper {
    /// Every wallpaper, so the settings UI, the preference file and the tests
    /// enumerate one list.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::System, Self::Flat, Self::Glow]
    }

    /// Stable token, written to the preference file and parsed back from it.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Flat => "flat",
            Self::Glow => "glow",
        }
    }

    /// Parse a stored token. `None` for anything this build does not know, so a
    /// settings file from a newer build falls back instead of failing to load.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::all().iter().copied().find(|w| w.token() == raw)
    }
}

/// An opaque canvas colour from a design token.
fn argb(c: litric_design::Rgb) -> u32 {
    rgb((u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b))
}

/// Paint the wallpaper for `theme`.
///
/// Dark uses the embedded wave. Light is a solid Pearl field: the wave is a dark
/// image and would not read as a light desktop. Both are opaque, so the glass
/// above them samples real content rather than a wash.
pub fn wallpaper(canvas: &mut Canvas, theme: &Theme) {
    wallpaper_for(canvas, theme, Wallpaper::System);
}

/// Paint the chosen wallpaper.
///
/// [`Wallpaper::System`] is byte-for-byte what this desktop painted before the
/// choice existed, so the default changes no pixel. The other two are drawn from
/// the theme's own tokens: a wallpaper cannot introduce a colour the contrast
/// gate has never seen.
pub fn wallpaper_for(canvas: &mut Canvas, theme: &Theme, choice: Wallpaper) {
    let field = Rect::new(0.0, 0.0, canvas.width() as f32, canvas.height() as f32);
    match choice {
        Wallpaper::System => paint_wallpaper(canvas, theme, WAVE_PNG),
        Wallpaper::Flat => canvas.fill_round_rect(field, 0.0, argb(theme.palette.bg_base)),
        Wallpaper::Glow => {
            canvas.fill_vertical_gradient(
                field,
                argb(theme.palette.bg_elevated),
                argb(theme.palette.bg_base),
            );
            // Peak 0.10: the wash is a tint, not a light source. The design
            // material test that keeps a wallpaper a backdrop rather than a
            // gradient demo (`the_wallpaper_stays_a_backdrop_not_a_gradient_demo`)
            // covers this wallpaper too.
            canvas.radial_wash(
                field.w * 0.82,
                field.h * 0.12,
                field.w.max(field.h) * 0.55,
                argb(theme.palette.accent),
                0.10,
            );
        }
    }
}

fn paint_wallpaper(canvas: &mut Canvas, theme: &Theme, png_bytes: &[u8]) {
    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
    if theme.kind == ThemeKind::Light {
        canvas.fill_round_rect(Rect::new(0.0, 0.0, w, h), 0.0, rgb(0xF4_F2_ED));
        return;
    }
    match decode_png(png_bytes) {
        Some((iw, ih, pixels)) => paint_cover(canvas, iw, ih, &pixels),
        None => canvas.fill_round_rect(Rect::new(0.0, 0.0, w, h), 0.0, rgb(0x11_13_18)),
    }
}

/// Everything the desktop drew, for the status file.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Panel name and the alpha its glass settled on.
    pub panels: Vec<(String, u8)>,
    /// Worst text contrast measured on any panel, after compositing.
    pub worst_contrast: f64,
}

/// Draw the whole desktop.
///
/// `scale` is 1.0 at 1280x800 and scales the whole layout with the display, so
/// the same composition works on a small VM console and a real screen.
pub fn render(canvas: &mut Canvas, theme: &Theme, state: &DesktopState) -> Report {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let s = (w / 1280.0).min(h / 800.0).clamp(0.55, 3.0);

    wallpaper(canvas, theme);

    let mut report = Report::default();
    let mut worst = f64::INFINITY;
    let mut note = |r: &mut Report, name: &str, p: &GlassPanel, theme: &Theme| {
        r.panels.push((name.to_owned(), p.alpha_pct));
        let surf = design(p.surface);
        let c1 = theme.palette.text_primary.contrast(surf);
        let c2 = theme.palette.text_secondary.contrast(surf);
        worst = worst.min(c1).min(c2);
    };

    let bar = menubar(canvas, theme, state, s);
    note(&mut report, "menubar", &bar, theme);

    let win = window(canvas, theme, state, s);
    note(&mut report, "activity", &win, theme);

    let sw = switcher(canvas, theme, state, s);
    note(&mut report, "switcher", &sw, theme);

    let dk = dock(canvas, theme, s);
    note(&mut report, "dock", &dk, theme);

    report.worst_contrast = if worst.is_finite() { worst } else { 0.0 };
    report
}

fn menubar(canvas: &mut Canvas, theme: &Theme, state: &DesktopState, s: f32) -> GlassPanel {
    let h = 34.0 * s;
    let r = Rect::new(0.0, 0.0, canvas.width() as f32, h);
    let p = glass(canvas, theme, r, 0.0);

    let size = 13.0 * s;
    let base = h / 2.0 + size * 0.36;
    let pri = px(theme.palette.text_primary);
    let sec = px(theme.palette.text_secondary);

    let mut x = 18.0 * s;
    x = font::draw(
        canvas,
        "LitricOS",
        x,
        base,
        &Style::bold(size, pri).tracked(0.2 * s),
    );
    x += 18.0 * s;
    for item in ["File", "Edit", "View", "Window"] {
        x = font::draw(canvas, item, x, base, &Style::new(size, sec));
        x += 16.0 * s;
    }

    // Right-hand status, laid out from the right edge so it stays put when the
    // text changes width.
    let right = canvas.width() as f32 - 18.0 * s;
    let clock_w = font::measure(&state.clock, &Style::new(size, 0));
    font::draw(
        canvas,
        &state.clock,
        right - clock_w,
        base,
        &Style::new(size, pri),
    );

    let verity_text = if state.verity {
        "Verity active"
    } else {
        "Verity off"
    };
    let vt_w = font::measure(verity_text, &Style::new(size, 0));
    let vx = right - clock_w - 22.0 * s - vt_w;
    font::draw(canvas, verity_text, vx, base, &Style::new(size, sec));
    let dot = if state.verity {
        px(theme.palette.success)
    } else {
        px(theme.palette.warning)
    };
    canvas.fill_circle(vx - 10.0 * s, h / 2.0, 3.4 * s, dot);

    p
}

fn window(canvas: &mut Canvas, theme: &Theme, state: &DesktopState, s: f32) -> GlassPanel {
    let w = 470.0 * s;
    let h = 246.0 * s;
    let r = Rect::new(76.0 * s, 132.0 * s, w, h);
    let radius = 18.0 * s;

    // A graded shadow, so the window sits above the wallpaper rather than in
    // it. It has to be soft: at 18% tint the panel is transparent enough that a
    // hard-edged shadow shows straight through as a dark plate.
    canvas.soft_shadow(r, radius, 26.0 * s, 10.0 * s, 0.55);

    let p = glass(canvas, theme, r, radius);

    let pri = px(theme.palette.text_primary);
    let sec = px(theme.palette.text_secondary);
    let title = 14.0 * s;

    // Title bar
    let tb = r.y + 30.0 * s;
    canvas.fill_round_rect(
        Rect::new(r.x + 1.0, tb, r.w - 2.0, 1.0),
        0.0,
        with_alpha(rgb(0xFFFFFF), 0.08),
    );
    for (i, c) in [
        theme.palette.danger,
        theme.palette.warning,
        theme.palette.success,
    ]
    .into_iter()
    .enumerate()
    {
        let cx = r.x + 18.0 * s + (i as f32) * 15.0 * s;
        canvas.fill_circle(cx, r.y + 15.0 * s, 5.0 * s, px(c));
    }
    font::draw(
        canvas,
        "Activity",
        r.x + 74.0 * s,
        r.y + 20.0 * s,
        &Style::new(title, sec),
    );

    // Body
    let pad = 20.0 * s;
    let mut y = tb + 30.0 * s;
    let body = 13.5 * s;
    let line = 30.0 * s;
    let right = r.right() - pad;

    let mem_label = format!("{} / {} MB", state.mem_used_mb, state.mem_total_mb);
    row(
        canvas,
        r.x + pad,
        right,
        y,
        body,
        "Memory",
        &mem_label,
        pri,
        sec,
        s,
    );
    y += 16.0 * s;

    // The memory bar is the one piece of real telemetry on screen; it moves
    // with the machine, which is the whole point of drawing it from measured
    // numbers rather than from a mockup.
    let frac = if state.mem_total_mb == 0 {
        0.0
    } else {
        (state.mem_used_mb as f32 / state.mem_total_mb as f32).clamp(0.0, 1.0)
    };
    let track = Rect::new(r.x + pad, y, r.w - pad * 2.0, 5.0 * s);
    canvas.fill_round_rect(track, 2.5 * s, with_alpha(rgb(0xFFFFFF), 0.12));
    canvas.fill_round_rect(
        Rect::new(track.x, track.y, track.w * frac, track.h),
        2.5 * s,
        px(theme.palette.accent),
    );
    y += line;

    let up = format_uptime(state.uptime_secs);
    row(
        canvas,
        r.x + pad,
        right,
        y,
        body,
        "Uptime",
        &up,
        pri,
        sec,
        s,
    );
    y += line;
    row(
        canvas,
        r.x + pad,
        right,
        y,
        body,
        "Root",
        &format!("{} \u{00B7} read-only", state.root_fs),
        pri,
        sec,
        s,
    );
    y += line;
    let (n, bytes) = state.trash;
    row(
        canvas,
        r.x + pad,
        right,
        y,
        body,
        "Trash",
        &format!("{n} items \u{00B7} {}", human_bytes(bytes)),
        pri,
        sec,
        s,
    );
    y += line;
    row(
        canvas,
        r.x + pad,
        right,
        y,
        body,
        "CPUs",
        &format!("{}", state.cpus),
        pri,
        sec,
        s,
    );

    p
}

#[allow(clippy::too_many_arguments)]
fn row(
    canvas: &mut Canvas,
    left: f32,
    right: f32,
    baseline: f32,
    size: f32,
    label: &str,
    value: &str,
    label_color: u32,
    value_color: u32,
    _s: f32,
) {
    font::draw(
        canvas,
        label,
        left,
        baseline,
        &Style::new(size, label_color),
    );
    let vw = font::measure(value, &Style::new(size, 0));
    font::draw(
        canvas,
        value,
        right - vw,
        baseline,
        &Style::new(size, value_color),
    );
}

fn switcher(canvas: &mut Canvas, theme: &Theme, state: &DesktopState, s: f32) -> GlassPanel {
    let w = 266.0 * s;
    let rows = state.environments.len().max(1) as f32;
    // 44 rather than 34: each row carries a name and, when the environment is
    // not running, its state underneath. At 34 the two lines overlapped, which
    // the first render made obvious and no amount of reading the code would
    // have.
    let row_h = 44.0 * s;
    let h = 38.0 * s + rows * row_h + 24.0 * s;
    let r = Rect::new(canvas.width() as f32 - w - 76.0 * s, 132.0 * s, w, h);
    let radius = 16.0 * s;

    canvas.soft_shadow(r, radius, 22.0 * s, 8.0 * s, 0.5);
    let p = glass(canvas, theme, r, radius);

    let pri = px(theme.palette.text_primary);
    let sec = px(theme.palette.text_secondary);
    let size = 13.5 * s;

    font::draw(
        canvas,
        "Environments",
        r.x + 18.0 * s,
        r.y + 23.0 * s,
        &Style::bold(11.0 * s, sec).tracked(1.2 * s),
    );

    let mut y = r.y + 36.0 * s;
    for (name, st, key, active) in &state.environments {
        let item = Rect::new(r.x + 8.0 * s, y, r.w - 16.0 * s, row_h - 6.0 * s);
        if *active {
            canvas.fill_round_rect(item, 10.0 * s, with_alpha(px(theme.palette.accent), 0.20));
            canvas.stroke_round_rect(
                item,
                10.0 * s,
                1.0,
                with_alpha(px(theme.palette.accent), 0.55),
            );
        }
        let running = st == "running";
        // A running environment has one line, centred; a stopped one has two.
        let name_base = if running {
            item.y + item.h / 2.0 + size * 0.36
        } else {
            item.y + item.h * 0.44
        };
        let color = if *active { pri } else { sec };
        font::draw(
            canvas,
            name,
            r.x + 20.0 * s,
            name_base,
            &Style::new(size, color),
        );

        if !running {
            font::draw(
                canvas,
                st,
                r.x + 20.0 * s,
                item.y + item.h * 0.86,
                &Style::new(9.5 * s, with_alpha(sec, 0.7)).tracked(0.4 * s),
            );
        }

        // The hotkey, boxed, vertically centred on the row. This is the promise
        // the whole multi-environment design rests on, so it is on screen
        // rather than in a manual.
        let ksize = 10.5 * s;
        let kw = font::measure(key, &Style::new(ksize, 0).tracked(0.6 * s));
        let kh = 16.0 * s;
        let kb = Rect::new(
            r.right() - 20.0 * s - kw - 11.0 * s,
            item.y + (item.h - kh) / 2.0,
            kw + 11.0 * s,
            kh,
        );
        canvas.stroke_round_rect(kb, 4.5 * s, 1.0, with_alpha(rgb(0xFFFFFF), 0.24));
        font::draw(
            canvas,
            key,
            kb.x + 5.5 * s,
            kb.y + kh / 2.0 + ksize * 0.36,
            &Style::new(ksize, sec).tracked(0.6 * s),
        );

        y += row_h;
    }

    font::draw(
        canvas,
        "Host \u{00B7} cannot be suspended",
        r.x + 18.0 * s,
        r.bottom() - 10.0 * s,
        &Style::new(10.0 * s, with_alpha(sec, 0.8)).tracked(0.3 * s),
    );

    p
}

fn dock(canvas: &mut Canvas, theme: &Theme, s: f32) -> GlassPanel {
    let icons = 6usize;
    let tile = 46.0 * s;
    let gap = 12.0 * s;
    let pad = 12.0 * s;
    let w = (icons as f32) * tile + ((icons - 1) as f32) * gap + pad * 2.0;
    let h = tile + pad * 2.0;
    let r = Rect::new(
        (canvas.width() as f32 - w) / 2.0,
        canvas.height() as f32 - h - 22.0 * s,
        w,
        h,
    );
    let radius = 20.0 * s;

    canvas.soft_shadow(r, radius, 26.0 * s, 9.0 * s, 0.5);
    let p = glass(canvas, theme, r, radius);

    let accent = px(theme.palette.accent);
    let fg = px(theme.palette.text_primary);
    for i in 0..icons {
        let x = r.x + pad + (i as f32) * (tile + gap);
        let t = Rect::new(x, r.y + pad, tile, tile);
        let first = i == 0;
        canvas.fill_round_rect(
            t,
            12.0 * s,
            if first {
                with_alpha(accent, 0.92)
            } else {
                with_alpha(rgb(0xFFFFFF), 0.13)
            },
        );
        canvas.stroke_round_rect(t, 12.0 * s, 1.0, with_alpha(rgb(0xFFFFFF), 0.16));
        let ink = if first {
            px(theme.palette.on_accent)
        } else {
            fg
        };
        icon(canvas, i, t, ink, s);
    }

    p
}

/// The dock icons, drawn as strokes rather than loaded as images.
fn icon(canvas: &mut Canvas, which: usize, t: Rect, color: u32, s: f32) {
    let (cx, cy) = t.center();
    let u = t.w * 0.5;
    let lw = (2.0 * s).max(1.2);
    match which {
        // Files: a folder.
        0 => {
            let b = Rect::new(cx - u * 0.62, cy - u * 0.40, u * 1.24, u * 0.86);
            canvas.stroke_round_rect(b, u * 0.16, lw, color);
            canvas.stroke_line(
                cx - u * 0.62,
                cy - u * 0.40,
                cx - u * 0.30,
                cy - u * 0.62,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.30,
                cy - u * 0.62,
                cx - u * 0.02,
                cy - u * 0.62,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.02,
                cy - u * 0.62,
                cx + u * 0.10,
                cy - u * 0.40,
                lw,
                color,
            );
        }
        // Browser: a globe.
        1 => {
            canvas.stroke_round_rect(
                Rect::new(cx - u * 0.62, cy - u * 0.62, u * 1.24, u * 1.24),
                u * 0.62,
                lw,
                color,
            );
            canvas.stroke_line(cx - u * 0.62, cy, cx + u * 0.62, cy, lw, color);
            canvas.stroke_round_rect(
                Rect::new(cx - u * 0.26, cy - u * 0.62, u * 0.52, u * 1.24),
                u * 0.26,
                lw,
                color,
            );
        }
        // Terminal: a prompt.
        2 => {
            canvas.stroke_line(cx - u * 0.5, cy - u * 0.3, cx - u * 0.1, cy, lw, color);
            canvas.stroke_line(cx - u * 0.1, cy, cx - u * 0.5, cy + u * 0.3, lw, color);
            canvas.stroke_line(
                cx + u * 0.06,
                cy + u * 0.3,
                cx + u * 0.52,
                cy + u * 0.3,
                lw,
                color,
            );
        }
        // Settings: a dial.
        3 => {
            canvas.stroke_round_rect(
                Rect::new(cx - u * 0.5, cy - u * 0.5, u, u),
                u * 0.5,
                lw,
                color,
            );
            canvas.fill_circle(cx, cy, u * 0.16, color);
            canvas.stroke_line(cx, cy - u * 0.5, cx, cy - u * 0.74, lw, color);
            canvas.stroke_line(cx, cy + u * 0.5, cx, cy + u * 0.74, lw, color);
            canvas.stroke_line(cx - u * 0.5, cy, cx - u * 0.74, cy, lw, color);
            canvas.stroke_line(cx + u * 0.5, cy, cx + u * 0.74, cy, lw, color);
        }
        // Activity: a sparkline.
        4 => {
            let pts = [
                (cx - u * 0.6, cy + u * 0.25),
                (cx - u * 0.25, cy - u * 0.30),
                (cx + u * 0.05, cy + u * 0.05),
                (cx + u * 0.35, cy - u * 0.50),
                (cx + u * 0.6, cy - u * 0.10),
            ];
            let mut prev: Option<(f32, f32)> = None;
            for p in pts {
                if let Some(a) = prev {
                    canvas.stroke_line(a.0, a.1, p.0, p.1, lw, color);
                }
                prev = Some(p);
            }
        }
        // Trash: a bin.
        _ => {
            canvas.stroke_line(
                cx - u * 0.5,
                cy - u * 0.42,
                cx + u * 0.5,
                cy - u * 0.42,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.18,
                cy - u * 0.42,
                cx - u * 0.14,
                cy - u * 0.62,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.14,
                cy - u * 0.62,
                cx + u * 0.14,
                cy - u * 0.62,
                lw,
                color,
            );
            canvas.stroke_line(
                cx + u * 0.14,
                cy - u * 0.62,
                cx + u * 0.18,
                cy - u * 0.42,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.36,
                cy - u * 0.30,
                cx - u * 0.28,
                cy + u * 0.56,
                lw,
                color,
            );
            canvas.stroke_line(
                cx + u * 0.36,
                cy - u * 0.30,
                cx + u * 0.28,
                cy + u * 0.56,
                lw,
                color,
            );
            canvas.stroke_line(
                cx - u * 0.28,
                cy + u * 0.56,
                cx + u * 0.28,
                cy + u * 0.56,
                lw,
                color,
            );
        }
    }
}

/// Seconds as `1h 04m` or `12s`.
#[must_use]
pub fn format_uptime(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let h = total.checked_div(3600).unwrap_or(0);
    let m = total
        .checked_rem(3600)
        .unwrap_or(0)
        .checked_div(60)
        .unwrap_or(0);
    let s = total.checked_rem(60).unwrap_or(0);
    if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

/// Bytes in the largest unit that keeps the number readable.
#[must_use]
pub fn human_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if n >= GB {
        format!("{:.1} GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}

/// `HH:MM` in UTC from seconds since the Unix epoch.
///
/// UTC and labelled as such, because the image carries no time zone database
/// and a clock that guesses local time is a clock that is wrong by hours
/// without saying so.
#[must_use]
pub fn utc_clock(epoch_secs: u64) -> String {
    let day = epoch_secs.checked_rem(86_400).unwrap_or(0);
    let h = day.checked_div(3600).unwrap_or(0);
    let m = day
        .checked_rem(3600)
        .unwrap_or(0)
        .checked_div(60)
        .unwrap_or(0);
    format!("{h:02}:{m:02} UTC")
}

/// The average luminance of a canvas, used by the tests and by the status file
/// to prove that something was actually drawn.
#[must_use]
pub fn mean_luminance(canvas: &Canvas) -> f32 {
    let px = canvas.pixels();
    if px.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0f32;
    let mut n = 0u32;
    for (i, p) in px.iter().enumerate() {
        if i.checked_rem(97).unwrap_or(0) == 0 {
            sum += luminance(*p);
            n = n.saturating_add(1);
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / (n as f32)
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn state() -> DesktopState {
        DesktopState {
            mem_total_mb: 946,
            mem_used_mb: 218,
            cpus: 2,
            uptime_secs: 9.17,
            root_fs: "erofs".to_owned(),
            verity: true,
            trash: (4, 13_421_772),
            environments: vec![
                (
                    "Windows 11".to_owned(),
                    "not-installed".to_owned(),
                    "Win+1".to_owned(),
                    false,
                ),
                (
                    "LitricOS".to_owned(),
                    "running".to_owned(),
                    "Win+2".to_owned(),
                    true,
                ),
                (
                    "Linux".to_owned(),
                    "not-installed".to_owned(),
                    "Win+3".to_owned(),
                    false,
                ),
            ],
            source: "test".to_owned(),
            clock: "09:41 UTC".to_owned(),
        }
    }

    #[test]
    fn a_rendered_desktop_is_not_a_blank_screen() {
        let mut c = Canvas::new(640, 400, rgb(0x000000));
        let theme = Theme::of(ThemeKind::Dark);
        let report = render(&mut c, &theme, &state());

        let mean = mean_luminance(&c);
        assert!(mean > 0.01, "the screen is black: mean luminance {mean}");
        assert!(mean < 0.85, "the screen is white: mean luminance {mean}");
        assert_eq!(report.panels.len(), 4, "four glass surfaces were expected");
    }

    #[test]
    fn every_glass_surface_keeps_text_readable() {
        // The claim the whole material rests on, checked on the real
        // composition rather than on a synthetic backdrop.
        let mut c = Canvas::new(1280, 800, rgb(0x000000));
        let theme = Theme::of(ThemeKind::Dark);
        let report = render(&mut c, &theme, &state());
        assert!(
            report.worst_contrast >= AA,
            "worst contrast was {:.2}, below AA: {:?}",
            report.worst_contrast,
            report.panels
        );
    }

    #[test]
    fn glass_over_a_dark_wallpaper_stays_transparent() {
        // The point of the adaptive solver. Over this wallpaper the panels must
        // come out far more transparent than the static 82% floor, or the
        // desktop looks like frosted plastic.
        let mut c = Canvas::new(1280, 800, rgb(0x000000));
        let theme = Theme::of(ThemeKind::Dark);
        let report = render(&mut c, &theme, &state());
        for (name, alpha) in &report.panels {
            assert!(
                *alpha < theme.glass.tint_alpha_pct,
                "{name} used {alpha}%, no better than the static {}%",
                theme.glass.tint_alpha_pct
            );
        }
    }

    #[test]
    fn glass_over_a_white_backdrop_firms_up_on_its_own() {
        let mut c = Canvas::new(200, 120, rgb(0xFFFFFF));
        let theme = Theme::of(ThemeKind::Dark);
        let p = glass(&mut c, &theme, Rect::new(10.0, 10.0, 180.0, 100.0), 8.0);
        assert!(
            p.alpha_pct > 70,
            "over white the tint only reached {}%",
            p.alpha_pct
        );
    }

    #[test]
    fn the_light_theme_produces_a_light_desktop() {
        let mut c = Canvas::new(640, 400, rgb(0x000000));
        let dark = mean_luminance({
            render(&mut c, &Theme::of(ThemeKind::Dark), &state());
            &c
        });
        let mut c2 = Canvas::new(640, 400, rgb(0x000000));
        let light = mean_luminance({
            render(&mut c2, &Theme::of(ThemeKind::Light), &state());
            &c2
        });
        assert!(
            light > dark,
            "light {light} is not brighter than dark {dark}"
        );
    }

    #[test]
    fn the_wallpaper_is_not_one_flat_colour() {
        // Glass over a flat colour proves nothing about glass.
        // A grid, not three hand-picked points: the wave leaves most of the
        // top of the frame at the solid base colour, so a few points can land
        // there and say nothing about the ribbon.
        let mut c = Canvas::new(320, 200, rgb(0x000000));
        wallpaper(&mut c, &Theme::of(ThemeKind::Dark));
        let mut seen = std::collections::BTreeSet::new();
        for gy in 0..10 {
            for gx in 0..16 {
                seen.insert(c.get(gx * 20 + 10, gy * 20 + 10).unwrap_or(0));
            }
        }
        assert!(
            seen.len() > 8,
            "wallpaper is nearly flat: {} colours",
            seen.len()
        );
    }

    #[test]
    fn rendering_survives_an_absurdly_small_screen() {
        // A VGA text console handed to the shell must not take it down.
        let mut c = Canvas::new(64, 40, rgb(0x000000));
        let report = render(&mut c, &Theme::of(ThemeKind::Dark), &state());
        assert_eq!(report.panels.len(), 4);
    }

    #[test]
    fn rendering_with_no_environments_still_draws() {
        let mut st = state();
        st.environments.clear();
        let mut c = Canvas::new(400, 300, rgb(0x000000));
        let report = render(&mut c, &Theme::of(ThemeKind::Dark), &st);
        assert_eq!(report.panels.len(), 4);
    }

    #[test]
    fn zero_total_memory_does_not_divide_by_zero() {
        let mut st = state();
        st.mem_total_mb = 0;
        st.mem_used_mb = 0;
        let mut c = Canvas::new(400, 300, rgb(0x000000));
        let _ = render(&mut c, &Theme::of(ThemeKind::Dark), &st);
    }

    #[test]
    fn uptime_formats_at_every_scale() {
        assert_eq!(format_uptime(9.17), "9s");
        assert_eq!(format_uptime(75.0), "1m 15s");
        assert_eq!(format_uptime(3_600.0), "1h 00m");
        assert_eq!(format_uptime(7_845.0), "2h 10m");
        assert_eq!(format_uptime(-5.0), "0s");
    }

    #[test]
    fn byte_sizes_read_the_way_a_person_expects() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(999), "999 B");
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(13_421_772), "12.8 MB");
        assert_eq!(human_bytes(3 * 1024 * 1024 * 1024), "3.0 GB");
    }

    #[test]
    fn the_clock_is_utc_and_says_so() {
        assert_eq!(utc_clock(0), "00:00 UTC");
        assert_eq!(utc_clock(34_860), "09:41 UTC");
        assert_eq!(utc_clock(86_399), "23:59 UTC");
        assert_eq!(utc_clock(86_400), "00:00 UTC", "midnight rolls over");
    }

    #[test]
    fn the_embedded_wave_decodes_at_full_size() {
        let (w, h, pixels) = decode_png(WAVE_PNG).expect("the shipped wave must decode");
        assert_eq!((w, h), (1920, 1080));
        assert_eq!(pixels.len(), 1920 * 1080);
    }

    #[test]
    fn an_unreadable_wave_falls_back_to_solid_obsidian() {
        let mut c = Canvas::new(320, 200, rgb(0x000000));
        paint_wallpaper(&mut c, &Theme::of(ThemeKind::Dark), b"not a png");
        for (x, y) in [(0, 0), (319, 0), (160, 100), (319, 199)] {
            assert_eq!(c.get(x, y), Some(rgb(0x11_13_18)), "at ({x}, {y})");
        }
    }

    #[test]
    fn the_light_wallpaper_is_solid_pearl() {
        let mut c = Canvas::new(320, 200, rgb(0x000000));
        wallpaper(&mut c, &Theme::of(ThemeKind::Light));
        for (x, y) in [(0, 0), (160, 100), (319, 199)] {
            assert_eq!(c.get(x, y), Some(rgb(0xF4_F2_ED)), "at ({x}, {y})");
        }
    }

    // --- UI quality regression guards ------------------------------------
    //
    // This desktop once shipped with a neon gradient wallpaper, 180% glass
    // saturation and a pencil-thin typeface; users read the result as "not
    // minimal, low quality". These pins hold the corrected material language
    // in place the same way the contrast floor is held: by test, not by style
    // sheet.

    /// Channel spread of a pixel, 0..=255: a cheap saturation proxy that
    /// needs no colour-space crate.
    fn chroma(px: u32) -> u32 {
        let r = (px >> 16) & 0xFF;
        let g = (px >> 8) & 0xFF;
        let b = px & 0xFF;
        let hi = r.max(g).max(b);
        let lo = r.min(g).min(b);
        hi - lo
    }

    #[test]
    fn every_wallpaper_choice_paints_something_different() {
        // A picker whose entries all look the same is a picker that lies about
        // having options. Compared as whole frames rather than by sampling one
        // pixel: two wallpapers may agree in a corner and differ everywhere a
        // person actually looks.
        let theme = Theme::of(ThemeKind::Dark);
        let mut painted: Vec<(Wallpaper, Vec<u32>)> = Vec::new();
        for choice in Wallpaper::all() {
            let mut c = Canvas::new(320, 200, rgb(0x000000));
            wallpaper_for(&mut c, &theme, *choice);
            painted.push((*choice, c.pixels().to_vec()));
        }
        for (i, (a, pa)) in painted.iter().enumerate() {
            for (b, pb) in painted.iter().skip(i.saturating_add(1)) {
                assert_ne!(pa, pb, "{a:?} and {b:?} paint the same wallpaper");
            }
        }
    }

    #[test]
    fn the_flat_wallpaper_is_one_colour_and_the_glow_is_not() {
        // The two claims the names make, in every theme: `Flat` is one field,
        // `Glow` is not. If `Glow` collapsed to a flat fill the setting would
        // still "work" and change nothing visible.
        for &kind in ThemeKind::all() {
            let theme = Theme::of(kind);

            let mut flat = Canvas::new(320, 200, rgb(0x000000));
            wallpaper_for(&mut flat, &theme, Wallpaper::Flat);
            let first = flat.get(0, 0);
            for (x, y) in [(0, 0), (319, 0), (160, 100), (319, 199)] {
                assert_eq!(
                    flat.get(x, y),
                    first,
                    "{kind:?}: flat wallpaper is not flat at ({x}, {y})"
                );
            }

            // Sampled at the glow's own centre and at a corner outside it. The
            // corners alone are not enough: in the high-contrast light theme the
            // gradient runs white to white, so two corners can agree while the
            // wash is still painted where it belongs.
            let mut glow = Canvas::new(320, 200, rgb(0x000000));
            wallpaper_for(&mut glow, &theme, Wallpaper::Glow);
            let centre = glow.get(262, 24);
            assert_ne!(
                centre,
                glow.get(0, 0),
                "{kind:?}: the glow wallpaper is a flat colour"
            );
        }
    }

    #[test]
    fn wallpaper_tokens_are_unique_and_round_trip() {
        // The preference file stores these strings. A duplicate token means a
        // user's choice loads back as a different wallpaper.
        let mut seen: Vec<&str> = Vec::new();
        for choice in Wallpaper::all() {
            assert!(
                !seen.contains(&choice.token()),
                "duplicate token {}",
                choice.token()
            );
            seen.push(choice.token());
            assert_eq!(Wallpaper::parse(choice.token()), Some(*choice));
        }
        assert_eq!(Wallpaper::parse("  glow "), Some(Wallpaper::Glow));
        assert_eq!(Wallpaper::parse("aurora-borealis"), None);
        assert_eq!(Wallpaper::parse(""), None);
    }

    #[test]
    fn every_wallpaper_still_leaves_the_glass_a_readable_alpha() {
        // A wallpaper is a user choice; legibility is not. The solver that
        // firms the glass up over a bright backdrop has to find an alpha in its
        // permitted range for every wallpaper the picker offers, or the new
        // option is a way to make text unreadable without touching contrast.
        for kind in [ThemeKind::Dark, ThemeKind::Light] {
            let theme = Theme::of(kind);
            for choice in Wallpaper::all() {
                let mut c = Canvas::new(480, 300, rgb(0x000000));
                wallpaper_for(&mut c, &theme, *choice);
                let brightest = c.brightest(Rect::new(0.0, 0.0, 480.0, 300.0));
                let backdrop = litric_design::Rgb::new(
                    ((brightest >> 16) & 0xFF) as u8,
                    ((brightest >> 8) & 0xFF) as u8,
                    (brightest & 0xFF) as u8,
                );
                let target = 4.5;
                let alpha = litric_design::Glass::adaptive_alpha_pct(
                    theme.palette.glass_tint,
                    theme.palette.text_primary,
                    backdrop,
                    target,
                );
                assert!(
                    alpha.is_some(),
                    "{kind:?} + {choice:?}: no glass alpha keeps text at {target}:1 over {backdrop}"
                );
            }
        }
    }

    #[test]
    fn the_wallpaper_stays_a_backdrop_not_a_gradient_demo() {
        // Saturated washes read through every glass surface as a coloured
        // haze. At each wash anchor the composited colour must stay inside
        // the muted band; 90 would have failed the old 0x2E5BFF@0.55 wash
        // (chroma 124) and passes every current one with margin.
        // Every wallpaper the picker offers, not just the shipped one: the
        // accent glow is exactly the kind of wash this test exists to keep
        // muted.
        for &kind in ThemeKind::all() {
            for choice in Wallpaper::all() {
                let mut c = Canvas::new(1280, 800, rgb(0x000000));
                wallpaper_for(&mut c, &Theme::of(kind), *choice);
                let anchors = [
                    c.get((1280.0 * 0.14) as i64, (800.0 * 0.10) as i64),
                    c.get((1280.0 * 0.88) as i64, (800.0 * 0.22) as i64),
                    c.get((1280.0 * 0.62) as i64, (800.0 * 0.98) as i64),
                ];
                for px in anchors.into_iter().flatten() {
                    assert!(
                        chroma(px) <= 90,
                        "{kind:?} {choice:?} wallpaper wash drifted back to neon: \
                         {px:#010x} has chroma {}",
                        chroma(px)
                    );
                }
            }
        }
    }

    #[test]
    fn glass_may_not_magnify_the_colours_behind_it() {
        // 180% saturation turned every panel into a filter over the
        // wallpaper. Glass boosts light, not chroma.
        for &kind in ThemeKind::all() {
            let glass = Theme::of(kind).glass;
            assert!(
                glass.saturation_pct <= 125,
                "{kind:?} glass saturation {} amplifies the backdrop into a haze",
                glass.saturation_pct
            );
        }
    }

    #[test]
    fn ui_text_never_renders_as_pencil_strokes() {
        // A regular weight of 0.105 rasterised below a hairline at footnote
        // size and read as handwriting. The floor is the band product UI
        // fonts occupy (SF Pro, Inter land near 0.13 at regular).
        //
        // These compare crate constants, which clippy flags as "constant"
        // assertions; pinning the contract is the whole point, so the lint is
        // scoped rather than the checks dropped.
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(
                crate::font::WEIGHT_REGULAR >= 0.12,
                "regular weight {} fell back into pencil territory",
                crate::font::WEIGHT_REGULAR
            );
            assert!(
                crate::font::WEIGHT_BOLD > crate::font::WEIGHT_REGULAR,
                "bold must stay heavier than regular"
            );
        }
    }
}
