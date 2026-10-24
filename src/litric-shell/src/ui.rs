//! The component layer: one place where a button decides what a button is.
//!
//! Every surface, control and label in the shell is drawn through here, from
//! the same tokens. That is the difference between a desktop and a collection
//! of screens that happen to use similar colours: the top bar, the launcher,
//! a dialog and the settings application share a spacing scale, a type scale, a
//! corner radius and a set of interaction states, and none of them contains a
//! pixel coordinate that was chosen on its own.
//!
//! # Materials, not "glass everywhere"
//!
//! Glass is one material among several and it is used where depth means
//! something: surfaces that float over content the user still needs to see.
//! A dialog is opaque, a sidebar is flat, a toolbar is a solid tint. Applying
//! the most expensive material uniformly makes an interface that is both slower
//! and harder to read, and it stops depth from carrying any information at all.
//!
//! # State is never colour alone
//!
//! Every interaction state changes more than a hue: hover changes the fill,
//! pressed changes the fill *and* insets the content, focus adds a ring,
//! selected adds a fill and a marker, disabled removes contrast and adds
//! transparency. A user who cannot distinguish two of our colours can still
//! tell the states apart.

use litric_design::{Radius, Space, Theme, TypeStep};

use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::icons::{self, Icon};

/// Which material a surface is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Material {
    /// Opaque, the theme's base. Window bodies, the launcher's backdrop.
    Solid,
    /// Opaque, one step lighter. Cards and list rows.
    Elevated,
    /// No fill at all: a region defined only by its contents.
    Flat,
    /// Translucent, blurred, adaptive. The top bar and window frames.
    Glass,
    /// Translucent but firmer than glass, for a surface with dense text over
    /// arbitrary content: menus and popovers.
    Popover,
    /// Opaque and raised. Dialogs, which must never be hard to read.
    Dialog,
    /// A dimming layer over everything behind it.
    Scrim,
}

/// How a control is being interacted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum State {
    /// Idle.
    #[default]
    Normal,
    /// The pointer is over it.
    Hover,
    /// The pointer is down on it.
    Pressed,
    /// It has the keyboard.
    Focused,
    /// It is the current choice in a list.
    Selected,
    /// It cannot be used.
    Disabled,
    /// Work is in flight behind this control. The control stays visible and
    /// draws a motion marker; it never pretends to be idle.
    Loading,
    /// The last operation through this control failed. Signalled with a
    /// danger border and marker, never colour alone.
    Error,
}

impl State {
    /// Whether input should reach it.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

/// Semantic meaning, for badges and status text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Tone {
    /// Ordinary.
    Neutral,
    /// The accent: the current thing, the primary action.
    Accent,
    /// Healthy, connected, on.
    Good,
    /// Degraded, needs attention.
    Warning,
    /// Failed, disconnected, off.
    Bad,
}

/// The resolved design tokens for one display.
///
/// Everything the shell draws asks this for its numbers, so changing the scale
/// factor moves the entire interface at once and no component has to know what
/// a physical pixel is.
#[derive(Debug, Clone)]
pub struct Tokens {
    /// Colours and materials.
    pub theme: Theme,
    /// Logical units to pixels.
    pub scale: f32,
}

impl Tokens {
    /// Tokens for a theme at a scale.
    #[must_use]
    pub const fn new(theme: Theme, scale: f32) -> Self {
        Self { theme, scale }
    }

    /// A spacing step in pixels.
    #[must_use]
    pub fn sp(&self, step: Space) -> f32 {
        f32::from(step.px()) * self.scale
    }

    /// A corner radius in pixels.
    #[must_use]
    pub fn radius(&self, r: Radius) -> f32 {
        f32::from(r.px().min(4096)) * self.scale
    }

    /// A type size in pixels.
    #[must_use]
    pub fn text_px(&self, step: TypeStep) -> f32 {
        f32::from(step.px()) * self.scale
    }

    /// A line height in pixels.
    #[must_use]
    pub fn line_px(&self, step: TypeStep) -> f32 {
        f32::from(step.line_height_px()) * self.scale
    }

    /// An arbitrary logical length in pixels.
    #[must_use]
    pub fn px(&self, logical: f32) -> f32 {
        logical * self.scale
    }

    /// The colour for a tone.
    #[must_use]
    pub fn tone(&self, tone: Tone) -> u32 {
        let p = self.theme.palette;
        match tone {
            Tone::Neutral => rgb_of(p.text_secondary),
            Tone::Accent => rgb_of(p.accent),
            Tone::Good => rgb_of(p.success),
            Tone::Warning => rgb_of(p.warning),
            Tone::Bad => rgb_of(p.danger),
        }
    }

    /// Primary text.
    #[must_use]
    pub fn text(&self) -> u32 {
        rgb_of(self.theme.palette.text_primary)
    }

    /// Secondary text.
    #[must_use]
    pub fn text_dim(&self) -> u32 {
        rgb_of(self.theme.palette.text_secondary)
    }

    /// The accent.
    #[must_use]
    pub fn accent(&self) -> u32 {
        rgb_of(self.theme.palette.accent)
    }

    /// A separator.
    #[must_use]
    pub fn border(&self) -> u32 {
        rgb_of(self.theme.palette.border)
    }

    /// A text style at a step, in primary text colour.
    #[must_use]
    pub fn style(&self, step: TypeStep) -> Style {
        Style::new(self.text_px(step), self.text())
    }

    /// A bold text style at a step.
    #[must_use]
    pub fn style_bold(&self, step: TypeStep) -> Style {
        Style::bold(self.text_px(step), self.text())
    }

    /// A label style: small, tracked, secondary.
    #[must_use]
    pub fn style_label(&self) -> Style {
        Style::bold(self.text_px(TypeStep::Caption), self.text_dim()).tracked(self.px(1.1))
    }
}

pub(crate) fn rgb_of(c: litric_design::Rgb) -> u32 {
    crate::canvas::rgba(c.r, c.g, c.b, 255)
}

/// Draw a surface of the given material.
///
/// Glass goes through [`crate::desktop::glass`], which solves its own alpha
/// against what is behind it; the rest are flat fills, because a material whose
/// job is to be readable should not depend on the wallpaper.
pub fn surface(canvas: &mut Canvas, t: &Tokens, r: Rect, radius: f32, material: Material) {
    let p = t.theme.palette;
    match material {
        Material::Flat => {}
        Material::Solid => canvas.fill_round_rect(r, radius, rgb_of(p.bg_base)),
        Material::Elevated => {
            canvas.fill_round_rect(r, radius, rgb_of(p.bg_elevated));
            canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), 0.06));
        }
        Material::Glass => {
            crate::desktop::glass(canvas, &t.theme, r, radius);
        }
        Material::Popover => {
            canvas.soft_shadow(r, radius, t.px(28.0), t.px(10.0), 0.55);
            crate::desktop::glass_at(canvas, &t.theme, r, radius, 88);
        }
        Material::Dialog => {
            canvas.soft_shadow(r, radius, t.px(40.0), t.px(14.0), 0.65);
            canvas.fill_round_rect(r, radius, rgb_of(p.bg_elevated));
            canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), 0.12));
        }
        Material::Scrim => {
            canvas.fill_round_rect(r, 0.0, with_alpha(rgb(0x000000), 0.45));
        }
    }
}

/// The fill a control gets in a given state.
fn state_fill(t: &Tokens, state: State, base_alpha: f32) -> u32 {
    match state {
        State::Normal => with_alpha(rgb(0xFFFFFF), base_alpha),
        State::Hover => with_alpha(rgb(0xFFFFFF), base_alpha + 0.07),
        State::Pressed => with_alpha(rgb(0xFFFFFF), base_alpha + 0.13),
        State::Focused => with_alpha(rgb(0xFFFFFF), base_alpha + 0.05),
        State::Selected => with_alpha(t.accent(), 0.22),
        State::Disabled => with_alpha(rgb(0xFFFFFF), base_alpha * 0.4),
        State::Loading => with_alpha(rgb(0xFFFFFF), base_alpha * 0.6),
        State::Error => with_alpha(t.tone(Tone::Bad), 0.16),
    }
}

/// Draw the focus ring. Always the same shape and colour, everywhere, so that
/// keyboard focus is one thing a user learns once.
pub fn focus_ring(canvas: &mut Canvas, t: &Tokens, r: Rect, radius: f32) {
    canvas.stroke_round_rect(
        r.inflate(t.px(2.0)),
        radius + t.px(2.0),
        t.px(2.0).max(1.5),
        with_alpha(t.accent(), 0.9),
    );
}

/// A text button.
///
/// Returns the rectangle it occupied, so the caller can register a hotspot
/// against exactly what was drawn rather than against a second set of numbers.
pub fn button(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    label: &str,
    state: State,
    tone: Tone,
) -> Rect {
    let radius = t.radius(Radius::Md);
    // Selected is a *choice*, not a call to action: it keeps the accent
    // family but sits visibly back from the filled action states, so the
    // eight states never collapse into fewer pictures.
    let fill = match (tone, state) {
        (Tone::Accent, State::Pressed) => with_alpha(t.accent(), 1.0),
        (Tone::Accent, State::Hover) => with_alpha(t.accent(), 0.92),
        (Tone::Accent, State::Selected) => with_alpha(t.accent(), 0.22),
        (Tone::Accent, State::Disabled) => state_fill(t, state, 0.10),
        (Tone::Accent, _) => with_alpha(t.accent(), 0.85),
        (_, state) => state_fill(t, state, 0.10),
    };
    canvas.fill_round_rect(r, radius, fill);
    canvas.stroke_round_rect(
        r,
        radius,
        1.0,
        if state == State::Error {
            t.tone(Tone::Bad)
        } else {
            with_alpha(rgb(0xFFFFFF), 0.10)
        },
    );
    if state == State::Focused {
        focus_ring(canvas, t, r, radius);
    }

    let color = if state == State::Disabled {
        with_alpha(t.text(), 0.4)
    } else if tone == Tone::Accent {
        rgb_of(t.theme.palette.on_accent)
    } else {
        t.text()
    };
    // A loading button speaks with motion, not words: three dots march
    // where the label would be, and the label yields.
    if state == State::Loading {
        let (cx, cy) = r.center();
        for k in 0..3 {
            let dx = (k as f32 - 1.0) * t.px(7.0);
            canvas.fill_circle(
                cx + dx,
                cy,
                t.px(2.0) + if k == 1 { t.px(1.0) } else { 0.0 },
                with_alpha(t.text(), 0.5 + 0.25 * f32::from(k == 1)),
            );
        }
        return r;
    }
    let style = Style::new(t.text_px(TypeStep::Footnote), color);
    let w = font::measure(label, &style);
    // Pressed insets the content by a pixel: the state is visible even to
    // someone who cannot see the fill change at all.
    let dy = if state == State::Pressed {
        t.px(1.0)
    } else {
        0.0
    };
    let (cx, cy) = r.center();
    font::draw(
        canvas,
        label,
        cx - w / 2.0,
        cy + style.size * 0.36 + dy,
        &style,
    );
    r
}

/// A square button containing only an icon.
pub fn icon_button(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    icon: Icon,
    state: State,
    tone: Tone,
) -> Rect {
    let radius = t.radius(Radius::Sm);
    if state != State::Normal {
        canvas.fill_round_rect(r, radius, state_fill(t, state, 0.0));
    }
    if state == State::Focused {
        focus_ring(canvas, t, r, radius);
    }
    let color = match state {
        State::Disabled => with_alpha(t.text(), 0.35),
        _ if tone != Tone::Neutral => t.tone(tone),
        _ => t.text(),
    };
    let inset = r.w.min(r.h) * 0.22;
    icons::draw(canvas, icon, r.inflate(-inset), color, 1.0);
    r
}

/// A row in a list: an icon, a title, an optional subtitle, an optional
/// trailing label.
#[allow(clippy::too_many_arguments)]
pub fn list_item(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    icon: Option<Icon>,
    title: &str,
    subtitle: Option<&str>,
    trailing: Option<&str>,
    state: State,
) -> Rect {
    let radius = t.radius(Radius::Md);
    if state != State::Normal {
        canvas.fill_round_rect(r, radius, state_fill(t, state, 0.0));
    }
    if state == State::Selected {
        // Selected is a fill *and* a marker: colour alone would be invisible
        // to somebody who cannot distinguish it from the row behind.
        canvas.fill_round_rect(
            Rect::new(r.x + t.px(3.0), r.y + r.h * 0.25, t.px(3.0), r.h * 0.5),
            t.px(1.5),
            t.accent(),
        );
    }
    if state == State::Focused {
        focus_ring(canvas, t, r, radius);
    }
    if state == State::Error {
        // A danger border plus the marker icon: two channels, not colour
        // alone (design system §02, state matrix).
        canvas.stroke_round_rect(r, radius, 1.0, t.tone(Tone::Bad));
    }
    if state == State::Loading {
        // A row in flight shows its work at the trailing edge — three dots,
        // the same motion vocabulary a loading button uses.
        let cy = r.center().1;
        for (k, mid) in [(0u32, 0.0f32), (1, 1.0), (2, 0.0)] {
            let dx = (f32::from(k as u8) - 1.0) * t.px(6.0);
            canvas.fill_circle(
                r.right() - t.px(18.0) + dx,
                cy,
                t.px(1.8) + if mid > 0.0 { t.px(0.8) } else { 0.0 },
                with_alpha(t.text(), 0.55 + 0.25 * mid),
            );
        }
    }

    let pad = t.sp(Space::Md);
    let mut x = r.x + pad;
    if let Some(i) = icon {
        let s = r.h * 0.5;
        icons::draw(
            canvas,
            i,
            Rect::new(x, r.y + (r.h - s) / 2.0, s, s),
            if state == State::Disabled {
                with_alpha(t.text(), 0.35)
            } else {
                t.text()
            },
            1.0,
        );
        x += s + t.sp(Space::Sm);
    }

    // Typographic role: `item_primary` (§01-3 of the design system). A list
    // row's title is the row's identity, so it carries the bold weight and
    // leaves regular weight for the metadata around it.
    let title_style = Style::bold(
        t.text_px(TypeStep::Body),
        if state == State::Disabled {
            with_alpha(t.text(), 0.4)
        } else {
            t.text()
        },
    );
    // A row's text is elided against the row, never drawn past it. A list item
    // that overflows its own box is text drawn over whatever is beside the list
    // -- in Settings' sidebar the word "Environments" reached into the page and
    // was read on top of the "Keyboard" row's label. Elision is the component's
    // job because the component is the only thing that knows its own box.
    let trailing_w = trailing.map_or(0.0, |tr| {
        font::measure(tr, &Style::new(t.text_px(TypeStep::Caption), t.text_dim())) + t.sp(Space::Sm)
    });
    let inner_w = (r.right() - pad - trailing_w - x).max(0.0);
    match subtitle {
        Some(sub) => {
            font::draw(
                canvas,
                &elide(title, &title_style, inner_w),
                x,
                r.y + r.h * 0.46,
                &title_style,
            );
            let sub_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            font::draw(
                canvas,
                &elide(sub, &sub_style, inner_w),
                x,
                r.y + r.h * 0.80,
                &sub_style,
            );
        }
        None => {
            let (_, cy) = r.center();
            font::draw(
                canvas,
                &elide(title, &title_style, inner_w),
                x,
                cy + title_style.size * 0.36,
                &title_style,
            );
        }
    }

    if let Some(tr) = trailing {
        let style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let w = font::measure(tr, &style);
        let (_, cy) = r.center();
        font::draw(
            canvas,
            tr,
            r.right() - pad - w,
            cy + style.size * 0.36,
            &style,
        );
    }
    r
}

/// A small status pill: a dot and a word.
pub fn badge(canvas: &mut Canvas, t: &Tokens, at: (f32, f32), text: &str, tone: Tone) -> Rect {
    let style = Style::bold(t.text_px(TypeStep::Caption), t.tone(tone)).tracked(t.px(0.6));
    let tw = font::measure(text, &style);
    let h = t.px(18.0);
    let dot = t.px(4.0);
    let pad = t.px(8.0);
    let w = pad + dot * 2.0 + t.px(6.0) + tw + pad;
    let r = Rect::new(at.0, at.1 - h / 2.0, w, h);
    canvas.fill_round_rect(r, h / 2.0, with_alpha(t.tone(tone), 0.16));
    canvas.fill_circle(r.x + pad + dot, r.y + h / 2.0, dot, t.tone(tone));
    font::draw(
        canvas,
        text,
        r.x + pad + dot * 2.0 + t.px(6.0),
        r.y + h / 2.0 + style.size * 0.36,
        &style,
    );
    r
}

/// A horizontal progress or usage bar.
pub fn progress(canvas: &mut Canvas, t: &Tokens, r: Rect, fraction: f32, tone: Tone) {
    let radius = r.h / 2.0;
    canvas.fill_round_rect(r, radius, with_alpha(rgb(0xFFFFFF), 0.12));
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.0 {
        canvas.fill_round_rect(
            Rect::new(r.x, r.y, (r.w * f).max(r.h), r.h),
            radius,
            t.tone(tone),
        );
    }
}

/// A switch. On is accent-filled with the knob to the right *and* a tick, so
/// the state does not rest on colour.
pub fn toggle(canvas: &mut Canvas, t: &Tokens, r: Rect, on: bool, state: State) -> Rect {
    let radius = r.h / 2.0;
    let fill = if on {
        with_alpha(
            t.accent(),
            if state == State::Disabled { 0.4 } else { 0.95 },
        )
    } else {
        with_alpha(rgb(0xFFFFFF), 0.16)
    };
    canvas.fill_round_rect(r, radius, fill);
    canvas.stroke_round_rect(r, radius, 1.0, with_alpha(rgb(0xFFFFFF), 0.14));
    if state == State::Focused {
        focus_ring(canvas, t, r, radius);
    }
    let knob = r.h - t.px(4.0);
    let kx = if on {
        r.right() - t.px(2.0) - knob
    } else {
        r.x + t.px(2.0)
    };
    canvas.fill_circle(kx + knob / 2.0, r.y + r.h / 2.0, knob / 2.0, rgb(0xFFFFFF));
    if on {
        icons::draw(
            canvas,
            Icon::Check,
            Rect::new(
                r.x + t.px(4.0),
                r.y + t.px(4.0),
                knob - t.px(2.0),
                knob - t.px(2.0),
            ),
            with_alpha(rgb(0xFFFFFF), 0.85),
            1.0,
        );
    }
    r
}

/// A slider. Returns the track rectangle for hit-testing.
pub fn slider(canvas: &mut Canvas, t: &Tokens, r: Rect, value: f32, state: State) -> Rect {
    let track_h = t.px(4.0);
    let track = Rect::new(r.x, r.y + (r.h - track_h) / 2.0, r.w, track_h);
    progress(canvas, t, track, value, Tone::Accent);
    let knob = t.px(if state == State::Pressed { 9.0 } else { 7.0 });
    let kx = r.x + r.w * value.clamp(0.0, 1.0);
    canvas.fill_circle(kx, r.y + r.h / 2.0, knob, rgb(0xFFFFFF));
    if state == State::Focused {
        focus_ring(canvas, t, r, r.h / 2.0);
    }
    r
}

/// A single-line text field. `caret` shows the insertion point when focused.
pub fn text_field(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    text: &str,
    placeholder: &str,
    focused: bool,
) -> Rect {
    let radius = t.radius(Radius::Md);
    canvas.fill_round_rect(r, radius, with_alpha(rgb(0xFFFFFF), 0.08));
    canvas.stroke_round_rect(
        r,
        radius,
        1.0,
        if focused {
            with_alpha(t.accent(), 0.8)
        } else {
            with_alpha(rgb(0xFFFFFF), 0.12)
        },
    );

    let pad = t.sp(Space::Md);
    let icon_size = r.h * 0.42;
    icons::draw(
        canvas,
        Icon::Search,
        Rect::new(
            r.x + pad,
            r.y + (r.h - icon_size) / 2.0,
            icon_size,
            icon_size,
        ),
        t.text_dim(),
        1.0,
    );

    let x = r.x + pad + icon_size + t.sp(Space::Sm);
    let (_, cy) = r.center();
    let showing_placeholder = text.is_empty();
    let style = Style::new(
        t.text_px(TypeStep::Body),
        if showing_placeholder {
            t.text_dim()
        } else {
            t.text()
        },
    );
    let shown = if showing_placeholder {
        placeholder
    } else {
        text
    };
    let end = font::draw(canvas, shown, x, cy + style.size * 0.36, &style);
    if focused && !showing_placeholder {
        canvas.fill_round_rect(
            Rect::new(
                end + t.px(2.0),
                cy - style.size * 0.55,
                t.px(1.5),
                style.size * 1.1,
            ),
            0.0,
            t.accent(),
        );
    } else if focused {
        canvas.fill_round_rect(
            Rect::new(x, cy - style.size * 0.55, t.px(1.5), style.size * 1.1),
            0.0,
            t.accent(),
        );
    }
    r
}

/// A section heading inside a panel.
pub fn section_label(canvas: &mut Canvas, t: &Tokens, at: (f32, f32), text: &str) {
    // Letter-spaced capitals are a Latin form: Thai has no capitals, and its
    // vowel marks land on each other at this size. A translated "SYSTEM"
    // arrived as a smear while "POWER" and "ENVIRONMENT" were fine, so the
    // structural keyword is drawn literally and the dictionary is left to prose.
    font::draw_literal(canvas, text, at.0, at.1, &t.style_label());
}

/// A hairline separator.
pub fn separator(canvas: &mut Canvas, t: &Tokens, r: Rect) {
    canvas.fill_round_rect(
        Rect::new(r.x, r.y, r.w, 1.0),
        0.0,
        with_alpha(t.border(), 0.9),
    );
}

/// A vertical stack of rows, handing out one rectangle at a time.
///
/// Every panel in the shell lays out through one of these rather than by
/// writing coordinates, which is what keeps the spacing identical between the
/// launcher, quick settings and the settings application.
#[derive(Debug, Clone, Copy)]
pub struct Column {
    /// Where the next row starts.
    pub cursor: f32,
    /// Left edge.
    pub x: f32,
    /// Available width.
    pub width: f32,
    /// Gap between rows.
    pub gap: f32,
}

impl Column {
    /// A column inside a rectangle, inset by `pad`.
    #[must_use]
    pub fn new(area: Rect, pad: f32, gap: f32) -> Self {
        Self {
            cursor: area.y + pad,
            x: area.x + pad,
            width: (area.w - pad * 2.0).max(0.0),
            gap,
        }
    }

    /// Take a row of this height.
    pub fn row(&mut self, height: f32) -> Rect {
        let r = Rect::new(self.x, self.cursor, self.width, height);
        self.cursor += height + self.gap;
        r
    }

    /// Skip some space.
    pub fn space(&mut self, amount: f32) {
        self.cursor += amount;
    }

    /// How far down the column has reached.
    #[must_use]
    pub const fn bottom(&self) -> f32 {
        self.cursor
    }
}

/// A list of rectangles paired with identifiers, for hit-testing what was
/// actually drawn.
///
/// The alternative — a second set of coordinates in the input handler — is how
/// a button ends up needing to be clicked two pixels above where it appears.
#[derive(Debug, Clone)]
pub struct Hotspots<T> {
    items: Vec<(Rect, T)>,
}

impl<T> Default for Hotspots<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T: Copy> Hotspots<T> {
    /// Forget everything. Called at the start of each frame's layout.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Register a rectangle.
    pub fn push(&mut self, r: Rect, id: T) {
        self.items.push((r, id));
    }

    /// What is at this point. Later entries win, matching draw order.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<T> {
        self.items
            .iter()
            .rev()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, id)| *id)
    }

    /// How many hotspots are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Identifiers in paint order, including repeated identifiers for distinct
    /// rows only once. Keyboard navigation uses the same registered controls as
    /// pointer hit-testing, so a control cannot be focusable without being drawn.
    pub fn ids(&self) -> impl Iterator<Item = T> + '_
    where
        T: PartialEq,
    {
        let mut seen = Vec::new();
        self.items.iter().filter_map(move |(_, id)| {
            if seen.contains(id) {
                None
            } else {
                seen.push(*id);
                Some(*id)
            }
        })
    }

    /// Every registered rectangle, for damage tracking.
    #[must_use]
    pub fn rects(&self) -> Vec<Rect> {
        self.items.iter().map(|(r, _)| *r).collect()
    }
}

impl<T: Copy + PartialEq> Hotspots<T> {
    /// Where an identifier was drawn.
    #[must_use]
    pub fn rect_of(&self, id: T) -> Option<Rect> {
        self.items.iter().find(|(_, i)| *i == id).map(|(r, _)| *r)
    }
}

/// The state a control should be drawn in, given what the pointer and keyboard
/// are doing.
#[must_use]
pub fn resolve_state(hovered: bool, pressed: bool, focused: bool, enabled: bool) -> State {
    if !enabled {
        State::Disabled
    } else if pressed {
        State::Pressed
    } else if hovered {
        State::Hover
    } else if focused {
        State::Focused
    } else {
        State::Normal
    }
}

/// Draw the pointer.
///
/// A filled arrow with a contrasting outline, so it stays visible over a white
/// window and a black one without any per-background logic.
pub fn cursor(canvas: &mut Canvas, x: f32, y: f32, scale: f32) {
    let s = 1.0_f32.max(scale);
    let pts: [(f32, f32); 8] = [
        (0.0, 0.0),
        (0.0, 16.0),
        (3.6, 12.6),
        (6.2, 18.4),
        (9.0, 17.2),
        (6.4, 11.6),
        (11.2, 11.4),
        (0.0, 0.0),
    ];
    let map = |p: (f32, f32)| (x + p.0 * s, y + p.1 * s);
    // The outline first and thicker, then the fill on top: two passes with the
    // same path, which keeps the shape identical and needs no second geometry.
    for (width, color) in [(3.4 * s, rgb(0x000000)), (1.7 * s, rgb(0xFFFFFF))] {
        let mut prev: Option<(f32, f32)> = None;
        for p in pts {
            let cur = map(p);
            if let Some(a) = prev {
                canvas.stroke_line(a.0, a.1, cur.0, cur.1, width, color);
            }
            prev = Some(cur);
        }
    }
    canvas.fill_circle(x + 3.0 * s, y + 7.0 * s, 2.6 * s, rgb(0xFFFFFF));
}

/// The rectangle the cursor occupies, for damage tracking.
#[must_use]
pub fn cursor_bounds(x: f32, y: f32, scale: f32) -> Rect {
    let s = 1.0_f32.max(scale);
    Rect::new(x - 3.0 * s, y - 3.0 * s, 20.0 * s, 26.0 * s)
}

/// Break `text` into lines that each fit `max_width`, at most `max_lines` of
/// them, ending in an ellipsis when even that is not enough.
///
/// A desktop icon's label is a file name, and a file name is long:
/// `Screencast-2026-10-16-14-12-35.webm` does not fit one 120-pixel line, and
/// eliding it to `Scree...` throws away the only part that tells it apart from
/// the file beside it. A label is therefore wrapped rather than cut, and only
/// the text that will not fit `max_lines` is elided -- into the last line, so
/// the ellipsis still appears where the name continues.
///
/// Lines break at the places a file name already has -- a space, a hyphen, an
/// underscore, a dot, a slash -- and fall back to breaking between characters
/// for a name with no break in it at all.
#[must_use]
pub fn wrap(text: &str, style: &Style, max_width: f32, max_lines: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() || max_lines == 0 || max_width <= 0.0 {
        return Vec::new();
    }
    // Every line, as the index it starts at and the text it holds.
    let mut lines: Vec<(usize, String)> = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let mut end = start;
        let mut width = 0.0_f32;
        let mut last_break: Option<usize> = None;
        while let Some(&ch) = chars.get(end) {
            // The first character of a line is taken even when it is wider than
            // the room: a line that holds nothing never ends. Every *other*
            // character pays for itself. This used to charge the first one 0px
            // always, which made every line's width under-count by one glyph:
            // `report-2026-10-16.txt` measures 120.14px at 12px and was accepted
            // into a 120px cell, so a long name never wrapped at all -- the exact
            // five-letter elision the wrap exists to replace, one glyph smaller.
            let advance = font::measure(&ch.to_string(), style);
            if width + advance > max_width && end > start {
                break;
            }
            width += advance;
            if is_break_char(ch) {
                last_break = Some(end.saturating_add(1));
            }
            end = end.saturating_add(1);
        }
        // A break opportunity inside the line is where the line ends, so a
        // wrapped name breaks between its words rather than mid-word.
        let mut cut = end;
        if end < chars.len() {
            if let Some(at) = last_break {
                if at > start {
                    cut = at;
                }
            }
        }
        if cut <= start {
            cut = start.saturating_add(1);
        }
        let Some(slice) = chars.get(start..cut) else {
            break;
        };
        let line: String = slice.iter().collect();
        lines.push((start, trim_trailing_space(&line)));
        start = cut;
        while chars.get(start) == Some(&' ') {
            start = start.saturating_add(1);
        }
    }
    if lines.len() > max_lines {
        // More text than the icon can show: keep the first lines and let the
        // last one say that the name continues.
        let keep: Vec<(usize, String)> = lines.drain(..max_lines.saturating_sub(1)).collect();
        let tail_start = lines.first().map_or(0, |(i, _)| *i);
        let tail: String = chars
            .get(tail_start..)
            .map_or_else(String::new, |s| s.iter().collect());
        let mut out: Vec<String> = keep.into_iter().map(|(_, l)| l).collect();
        out.push(elide(&tail, style, max_width));
        return out;
    }
    lines.into_iter().map(|(_, l)| l).collect()
}

/// Whether a character is a place a wrapped file name may break at.
fn is_break_char(ch: char) -> bool {
    matches!(ch, ' ' | '-' | '_' | '.' | '/' | ',')
}

/// `line` without the space it broke at. Nothing else is trimmed: a hyphen or
/// a dot at the end of a line is part of the name -- `report-2026-10-16.txt`
/// wraps to `report-2026-10-16.` and `txt`, and dropping that dot would show a
/// name whose extension reads as part of its stem.
fn trim_trailing_space(line: &str) -> String {
    line.trim_end_matches(' ').to_owned()
}

/// Truncate text to fit a width, ending in an ellipsis.
#[must_use]
pub fn elide(text: &str, style: &Style, max_width: f32) -> String {
    if font::measure(text, style) <= max_width {
        return text.to_owned();
    }
    let dots = "...";
    let mut out = String::new();
    for ch in text.chars() {
        let mut trial = out.clone();
        trial.push(ch);
        trial.push_str(dots);
        if font::measure(&trial, style) > max_width {
            break;
        }
        out.push(ch);
    }
    out.push_str(dots);
    out
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::luminance;
    use litric_design::ThemeKind;

    fn tokens() -> Tokens {
        Tokens::new(Theme::of(ThemeKind::Dark), 1.0)
    }

    fn ink(c: &Canvas) -> usize {
        (0..i64::from(c.height()))
            .flat_map(|y| (0..i64::from(c.width())).map(move |x| (x, y)))
            .filter(|(x, y)| luminance(c.get(*x, *y).unwrap_or(0)) > 0.12)
            .count()
    }

    #[test]
    fn spacing_and_radius_scale_together() {
        let one = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        let two = Tokens::new(Theme::of(ThemeKind::Dark), 2.0);
        assert!((two.sp(Space::Md) - one.sp(Space::Md) * 2.0).abs() < 1e-3);
        assert!((two.radius(Radius::Lg) - one.radius(Radius::Lg) * 2.0).abs() < 1e-3);
        assert!((two.text_px(TypeStep::Body) - one.text_px(TypeStep::Body) * 2.0).abs() < 1e-3);
    }

    #[test]
    fn the_full_radius_does_not_overflow_when_scaled() {
        // Radius::Full is a huge sentinel; multiplying it by a scale factor
        // must not produce an infinity that swallows the whole canvas.
        let t = Tokens::new(Theme::of(ThemeKind::Dark), 3.0);
        let r = t.radius(Radius::Full);
        assert!(r.is_finite() && r > 0.0, "got {r}");
    }

    #[test]
    fn every_state_looks_different_from_normal() {
        // If two states render identically the user cannot tell them apart,
        // which is the whole reason states exist.
        let t = tokens();
        let render = |state: State| -> usize {
            let mut c = Canvas::new(120, 40, rgb(0x000000));
            button(
                &mut c,
                &t,
                Rect::new(10.0, 8.0, 100.0, 24.0),
                "Open",
                state,
                Tone::Neutral,
            );
            ink(&c)
        };
        let normal = render(State::Normal);
        for s in [
            State::Hover,
            State::Pressed,
            State::Focused,
            State::Disabled,
        ] {
            assert_ne!(render(s), normal, "{s:?} renders the same as Normal");
        }
    }

    #[test]
    fn a_selected_row_is_marked_as_well_as_tinted() {
        // Colour alone is not an accessible way to show selection.
        let t = tokens();
        let mut plain = Canvas::new(200, 40, rgb(0x000000));
        list_item(
            &mut plain,
            &t,
            Rect::new(0.0, 0.0, 200.0, 40.0),
            None,
            "Files",
            None,
            None,
            State::Normal,
        );
        let mut selected = Canvas::new(200, 40, rgb(0x000000));
        list_item(
            &mut selected,
            &t,
            Rect::new(0.0, 0.0, 200.0, 40.0),
            None,
            "Files",
            None,
            None,
            State::Selected,
        );
        // The marker is a solid accent bar at the left edge.
        let bar = selected.get(4, 20).unwrap_or(0);
        assert!(luminance(bar) > luminance(plain.get(4, 20).unwrap_or(0)) + 0.05);
    }

    #[test]
    fn a_toggle_shows_its_state_by_position_and_by_a_tick() {
        let t = tokens();
        let render = |on: bool| -> Canvas {
            let mut c = Canvas::new(80, 40, rgb(0x000000));
            toggle(
                &mut c,
                &t,
                Rect::new(10.0, 12.0, 44.0, 22.0),
                on,
                State::Normal,
            );
            c
        };
        let off = render(false);
        let on = render(true);
        // The knob is at the other end.
        assert!(
            luminance(off.get(17, 23).unwrap_or(0)) > 0.5,
            "knob left when off"
        );
        assert!(
            luminance(on.get(46, 23).unwrap_or(0)) > 0.5,
            "knob right when on"
        );
        assert_ne!(ink(&off), ink(&on));
    }

    #[test]
    fn progress_never_overflows_its_track() {
        let t = tokens();
        let mut c = Canvas::new(120, 20, rgb(0x000000));
        progress(
            &mut c,
            &t,
            Rect::new(10.0, 8.0, 100.0, 4.0),
            5.0,
            Tone::Accent,
        );
        assert!(
            luminance(c.get(115, 10).unwrap_or(0)) < 0.05,
            "a fraction above 1.0 drew past the track"
        );
        progress(
            &mut c,
            &t,
            Rect::new(10.0, 8.0, 100.0, 4.0),
            -1.0,
            Tone::Accent,
        );
    }

    #[test]
    fn hotspots_hit_the_topmost_and_forget_on_clear() {
        let mut h: Hotspots<u8> = Hotspots::default();
        h.push(Rect::new(0.0, 0.0, 100.0, 100.0), 1);
        h.push(Rect::new(20.0, 20.0, 20.0, 20.0), 2);
        assert_eq!(h.hit(30.0, 30.0), Some(2), "later entries are on top");
        assert_eq!(h.hit(5.0, 5.0), Some(1));
        assert_eq!(h.hit(400.0, 400.0), None);
        assert_eq!(h.rect_of(2), Some(Rect::new(20.0, 20.0, 20.0, 20.0)));
        h.clear();
        assert!(h.is_empty());
        assert_eq!(h.hit(30.0, 30.0), None);
    }

    #[test]
    fn state_resolution_has_a_fixed_precedence() {
        assert_eq!(resolve_state(true, true, true, false), State::Disabled);
        assert_eq!(resolve_state(true, true, true, true), State::Pressed);
        assert_eq!(resolve_state(true, false, true, true), State::Hover);
        assert_eq!(resolve_state(false, false, true, true), State::Focused);
        assert_eq!(resolve_state(false, false, false, true), State::Normal);
    }

    #[test]
    fn the_cursor_draws_inside_the_bounds_it_reports() {
        // Damage tracking repaints exactly these bounds; anything outside them
        // is a trail of cursor left behind on the desktop.
        let mut c = Canvas::new(80, 80, rgb(0x000000));
        cursor(&mut c, 20.0, 20.0, 1.0);
        let b = cursor_bounds(20.0, 20.0, 1.0);
        for y in 0..80i64 {
            for x in 0..80i64 {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.05 {
                    assert!(
                        b.contains(x as f32 + 0.5, y as f32 + 0.5),
                        "cursor drew at {x},{y}, outside {b:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_cursor_is_visible_on_both_black_and_white() {
        for bg in [rgb(0x000000), rgb(0xFFFFFF)] {
            let mut c = Canvas::new(60, 60, bg);
            cursor(&mut c, 10.0, 10.0, 1.0);
            let different = (0..60i64)
                .flat_map(|y| (0..60i64).map(move |x| (x, y)))
                .filter(|(x, y)| c.get(*x, *y).unwrap_or(0) != bg)
                .count();
            assert!(different > 40, "cursor invisible on {bg:08x}");
        }
    }

    #[test]
    fn eliding_keeps_text_within_the_width_it_was_given() {
        let style = Style::new(14.0, rgb(0xFFFFFF));
        let long = "Environments and everything else";
        let out = elide(long, &style, 80.0);
        assert!(font::measure(&out, &style) <= 80.0, "{out:?} is too wide");
        assert!(out.ends_with("..."));
        assert_eq!(elide("short", &style, 500.0), "short", "no need to elide");
    }

    #[test]
    fn a_long_file_name_wraps_instead_of_eliding_to_five_letters() {
        let style = Style::new(12.0, rgb(0xFFFFFF));
        let name = "report-2026-10-16.txt";
        let lines = wrap(name, &style, 120.0, 2);
        assert_eq!(
            lines.len(),
            2,
            "a long name takes the second line: {lines:?}"
        );
        for line in &lines {
            assert!(
                font::measure(line, &style) <= 120.0,
                "{line:?} is wider than the cell it was given"
            );
        }
        // The whole name is on the icon, not the first five letters of it.
        assert_eq!(lines.concat(), name, "two lines hold every character");
    }

    #[test]
    fn a_name_too_long_for_two_lines_is_elided_where_it_stops_fitting() {
        let style = Style::new(12.0, rgb(0xFFFFFF));
        let name = "a-very-long-file-name-that-no-two-lines-can-hold-anyway.tar.gz";
        let lines = wrap(name, &style, 120.0, 2);
        assert_eq!(lines.len(), 2);
        for line in &lines {
            assert!(font::measure(line, &style) <= 120.0, "{line:?} is too wide");
        }
        assert!(
            lines.last().is_some_and(|l| l.ends_with("...")),
            "the last line has to say the name continues: {lines:?}"
        );
        // What *is* shown is still most of the name, which is the point: an
        // elision at two lines beats an elision at five letters.
        let shown: String = lines.join("");
        assert!(shown.len() > 20, "{shown:?} is too little of the name");
    }

    #[test]
    fn a_short_name_is_one_line_and_an_empty_one_is_none() {
        let style = Style::new(12.0, rgb(0xFFFFFF));
        assert_eq!(wrap("Documents", &style, 120.0, 2), vec!["Documents"]);
        assert!(wrap("", &style, 120.0, 2).is_empty());
    }

    #[test]
    fn a_column_hands_out_rows_that_do_not_overlap() {
        let mut col = Column::new(Rect::new(0.0, 0.0, 200.0, 300.0), 12.0, 8.0);
        let a = col.row(30.0);
        let b = col.row(30.0);
        assert_eq!(a.x, 12.0);
        assert_eq!(a.w, 176.0);
        assert!(b.y >= a.bottom(), "rows overlap: {a:?} then {b:?}");
        assert_eq!(b.y - a.bottom(), 8.0, "gap not applied");
    }

    #[test]
    fn every_material_except_flat_paints_something() {
        let t = tokens();
        for m in [
            Material::Solid,
            Material::Elevated,
            Material::Glass,
            Material::Popover,
            Material::Dialog,
            Material::Scrim,
        ] {
            let mut c = Canvas::new(100, 60, rgb(0x101010));
            surface(&mut c, &t, Rect::new(10.0, 10.0, 80.0, 40.0), 8.0, m);
            let changed = (0..60i64)
                .flat_map(|y| (0..100i64).map(move |x| (x, y)))
                .filter(|(x, y)| c.get(*x, *y).unwrap_or(0) != rgb(0x101010))
                .count();
            assert!(changed > 100, "{m:?} painted {changed} pixels");
        }

        let mut c = Canvas::new(100, 60, rgb(0x101010));
        surface(
            &mut c,
            &t,
            Rect::new(10.0, 10.0, 80.0, 40.0),
            8.0,
            Material::Flat,
        );
        assert!(
            c.pixels().iter().all(|p| *p == rgb(0x101010)),
            "Flat is supposed to paint nothing"
        );
    }
}
