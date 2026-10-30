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

// ---------------------------------------------------------------------------
// Sidebars
//
// Two windows in this shell have a sidebar: Files' Quick access places and
// Settings' page list. They were built twice, with two answers to every
// question a sidebar has to answer -- 132 px fixed versus 36% of the window,
// 20 px rows versus 30, no icon versus icon, colour-only selection versus fill
// plus marker, no scroll, no heading. The research corpus records that as
// F.10's largest delta: "a user learns two different sidebars in two adjacent
// windows".
//
// These functions are the one design both use. Geometry comes from the
// platforms' published numbers where they have one (libadwaita's 180-280sp
// pane, Windows' 320 DIP expanded pane, the 40 px general target) and from this
// shell's own list component for the row itself, so a sidebar row and a list
// row cannot drift apart.
// ---------------------------------------------------------------------------

/// Geometry and rows for a window sidebar.
pub mod sidebar {
    use super::{list_item, with_alpha, Canvas, Icon, Radius, Rect, Space, State, Tokens};
    use crate::font;

    /// Narrowest a sidebar may be, in logical pixels (libadwaita's minimum).
    pub const MIN_W: f32 = 180.0;
    /// Widest a sidebar may be, in logical pixels (libadwaita's maximum).
    pub const MAX_W: f32 = 280.0;
    /// A row's height. The shell's `list_item` uses 30 px; the Files pane's
    /// 20 px rows were below every published target, including WCAG's 24 px.
    pub const ROW_H: f32 = 30.0;
    /// Gap between rows.
    pub const GAP: f32 = 2.0;
    /// The narrowest the content beside the sidebar may become before the
    /// sidebar gives up width.
    pub const MIN_CONTENT_W: f32 = 320.0;

    /// One row of a sidebar.
    pub struct Item<'a> {
        /// Text shown on the row.
        pub label: &'a str,
        /// Leading icon, if the window has them.
        pub icon: Option<Icon>,
        /// Whether this row is the selected destination.
        pub selected: bool,
        /// Whether the pointer is over this row.
        pub highlighted: bool,
    }

    /// The sidebar's width for a window body.
    ///
    /// A quarter of the window, clamped to 180-280 px (libadwaita's pane range,
    /// which also contains Windows' 320 DIP expanded pane at any window this
    /// shell runs in). When the window is too narrow for both a sidebar and a
    /// usable content column, the sidebar takes a third rather than squeezing
    /// the content to nothing: 320 px is the floor the table needs.
    #[must_use]
    pub fn width(body_w: f32) -> f32 {
        if body_w <= 0.0 {
            return 0.0;
        }
        let quarter = body_w * 0.25;
        let target = quarter.clamp(MIN_W, MAX_W);
        let room = body_w - MIN_CONTENT_W;
        let floor = body_w / 3.0;
        target.min(room.max(floor)).max(0.0)
    }

    /// Height of `count` rows, gaps included.
    #[must_use]
    pub fn content_height(t: &Tokens, count: usize) -> f32 {
        if count == 0 {
            return 0.0;
        }
        let rows = count as f32 * ROW_H;
        let gaps = count.saturating_sub(1) as f32 * GAP;
        rows + t.px(gaps)
    }

    /// Largest offset that still shows the last row.
    #[must_use]
    pub fn max_scroll(t: &Tokens, list: Rect, count: usize) -> f32 {
        let view = (list.h - t.sp(Space::Sm) * 2.0).max(0.0);
        (content_height(t, count) - view).max(0.0)
    }

    /// Clamp a scroll offset to the list.
    #[must_use]
    pub fn clamp(t: &Tokens, list: Rect, count: usize, scroll: f32) -> f32 {
        let max = max_scroll(t, list, count);
        if scroll < 0.0 {
            0.0
        } else if scroll > max {
            max
        } else {
            scroll
        }
    }

    /// The area the scrolling rows occupy: `list` minus the fixed heading.
    ///
    /// Clamping and drawing both go through this, so a caller that stores the
    /// clamped offset cannot disagree with the frame about how tall the list is.
    #[must_use]
    pub fn rows_area(t: &Tokens, list: Rect, heading: bool) -> Rect {
        let heading_h = if heading { t.px(26.0) } else { 0.0 };
        Rect::new(
            list.x,
            list.y + heading_h,
            list.w,
            (list.h - heading_h).max(0.0),
        )
    }

    /// Draw the rows, clipped to `list`, and return the visible ones with their
    /// indices so the caller can register its own hotspots.
    ///
    /// A row that is scrolled out of view is neither drawn nor returned: a
    /// hotspot outside the pane would answer a click that never lands on it.
    pub fn draw(
        canvas: &mut Canvas,
        t: &Tokens,
        list: Rect,
        heading: Option<&str>,
        items: &[Item<'_>],
        scroll: f32,
    ) -> Vec<(usize, Rect)> {
        let pad = t.sp(Space::Sm);
        // The heading stays where it is while the rows move: a control that
        // belongs to the pane (Windows puts sidebar-affecting controls above
        // the list) must not scroll away with what it describes.
        let rows = rows_area(t, list, heading.is_some());
        let scroll = clamp(t, rows, items.len(), scroll);
        let clip = canvas.push_clip(rows);
        let mut y = rows.y + pad - scroll;
        let mut visible = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let row = Rect::new(rows.x + pad, y, (rows.w - pad * 2.0).max(0.0), t.px(ROW_H));
            y += t.px(ROW_H + GAP);
            if row.bottom() <= rows.y || row.y >= rows.bottom() {
                continue;
            }
            if item.highlighted {
                canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.text(), 0.055));
            }
            list_item(
                canvas,
                t,
                row,
                item.icon,
                item.label,
                None,
                None,
                if item.selected {
                    State::Selected
                } else {
                    State::Normal
                },
            );
            // The hotspot is the part of the row the pane actually shows: a row
            // half under the fixed heading must not answer a click on the
            // heading, and a row clipped at the bottom must not reach past the
            // pane.
            if let Some(hit) = row.intersection(&rows) {
                visible.push((i, hit));
            }
        }
        canvas.set_clip(clip);
        if let Some(title) = heading {
            let style = t.style_label();
            font::draw(canvas, title, list.x + pad, list.y + t.px(16.0), &style);
        }
        draw_indicator(canvas, t, rows, items.len(), scroll);
        visible
    }

    /// A thin scroll indicator when the rows are taller than the pane.
    ///
    /// The research notes that hover and pointer-only affordances are
    /// discouraged, so the pane says "there is more" without being hovered.
    fn draw_indicator(canvas: &mut Canvas, t: &Tokens, list: Rect, count: usize, scroll: f32) {
        let content = content_height(t, count);
        let view = (list.h - t.sp(Space::Sm) * 2.0).max(1.0);
        if content <= view {
            return;
        }
        // 3 px at 45 % and inset 4 px from the pane's edge: at the shell's 1x
        // scale a 2 px 45 % indicator was indistinguishable from the pane's
        // background, and hard against the edge it read as the divider between
        // the pane and the page rather than as a scrollbar.
        let track = Rect::new(
            list.right() - t.px(7.0),
            list.y + t.sp(Space::Sm),
            t.px(3.0),
            view,
        );
        canvas.fill_round_rect(track, t.px(1.5), with_alpha(t.text(), 0.12));
        let thumb_h = (view * (view / content)).max(t.px(18.0));
        let max = content - view;
        let at = if max > 0.0 {
            (scroll / max) * (view - thumb_h)
        } else {
            0.0
        };
        let thumb = Rect::new(track.x, track.y + at, track.w, thumb_h);
        canvas.fill_round_rect(thumb, t.px(1.5), with_alpha(t.text(), 0.45));
    }
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
    fn the_sidebar_width_policy_is_bounded_at_both_ends() {
        // One policy for both windows: 25 % of the body, held between
        // libadwaita's 180 and 280, and never wider than the body leaves room
        // for a 320 px content column. Settings used to take 36 % (504 px at
        // 1400) and Files a fixed 132 px.
        assert!((sidebar::width(1400.0) - 280.0).abs() < 0.01);
        assert!((sidebar::width(1000.0) - 250.0).abs() < 0.01);
        assert!((sidebar::width(740.0) - 185.0).abs() < 0.01);
        assert!((sidebar::width(600.0) - 180.0).abs() < 0.01);
        // Below 600 there is no room for 180 + 320, so the pane takes a third
        // and the content keeps two.
        assert!((sidebar::width(480.0) - 160.0).abs() < 0.01);
        assert_eq!(sidebar::width(0.0), 0.0);
        // At no width does the pane claim more than its window has.
        for body in [1.0, 120.0, 320.0, 599.0, 640.0, 3000.0] {
            let w = sidebar::width(body);
            assert!(w >= 0.0 && w <= body + 0.01, "{body} gave {w}");
        }
    }

    #[test]
    fn the_sidebar_scroll_stops_at_the_last_row() {
        let t = tokens();
        let list = Rect::new(0.0, 0.0, 240.0, 200.0);
        let rows = sidebar::rows_area(&t, list, true);
        let count = 13;
        let content = sidebar::content_height(&t, count);
        assert!(content > rows.h, "{content} rows in {} px", rows.h);
        let max = sidebar::max_scroll(&t, rows, count);
        assert!(max > 0.0);
        assert_eq!(sidebar::clamp(&t, rows, count, -40.0), 0.0);
        assert_eq!(sidebar::clamp(&t, rows, count, max + 400.0), max);
        // At the maximum offset the content's end sits exactly on the pane's
        // padded bottom: no blank strip below the last row and no clipped row.
        let pad = t.sp(Space::Sm);
        let bottom = rows.y + pad + content - max;
        assert!(
            (bottom - (rows.bottom() - pad)).abs() < 0.01,
            "last row ends at {bottom}, pane's padded bottom is {}",
            rows.bottom() - pad
        );
    }

    #[test]
    fn a_sidebar_draws_only_the_rows_inside_its_pane() {
        let t = tokens();
        let list = Rect::new(0.0, 0.0, 240.0, 200.0);
        let items: Vec<sidebar::Item<'_>> = (0..13)
            .map(|_| sidebar::Item {
                label: "Page",
                icon: None,
                selected: false,
                highlighted: false,
            })
            .collect();
        let mut c = Canvas::new(240, 200, rgb(0x000000));
        let visible = sidebar::draw(&mut c, &t, list, Some("Settings"), &items, 10_000.0);
        assert!(!visible.is_empty());
        assert!(
            visible.len() < items.len(),
            "all {} rows were reported visible in a 200 px pane",
            items.len()
        );
        assert_eq!(
            visible.last().map(|(i, _)| *i),
            Some(items.len() - 1),
            "scrolling to the end must reach the last row"
        );
        let rows = sidebar::rows_area(&t, list, true);
        for (_, row) in &visible {
            assert!(
                row.bottom() <= rows.bottom() + 0.01 && row.y >= rows.y - 0.01,
                "{row:?} is outside {rows:?}"
            );
        }
        // The heading does not scroll away with the rows.
        assert_eq!(rows.y, list.y + t.px(26.0));
    }

    #[test]
    fn a_sidebar_says_there_is_more_only_when_there_is() {
        let t = tokens();
        let items: Vec<sidebar::Item<'_>> = (0..13)
            .map(|_| sidebar::Item {
                label: "Page",
                icon: None,
                selected: false,
                highlighted: false,
            })
            .collect();
        // The indicator is drawn in the last 4 px of the pane, where no row
        // reaches (rows are inset by Space::Sm on both sides).
        let edge = |h: u32| -> usize {
            let mut c = Canvas::new(240, h, rgb(0x000000));
            sidebar::draw(
                &mut c,
                &t,
                Rect::new(0.0, 0.0, 240.0, h as f32),
                None,
                &items,
                0.0,
            );
            (0..i64::from(h))
                .flat_map(|y| (230..240i64).map(move |x| (x, y)))
                .filter(|(x, y)| c.get(*x, *y).unwrap_or(0) != rgb(0x000000))
                .count()
        };
        assert_eq!(edge(520), 0, "a pane that fits its rows drew a scrollbar");
        assert!(edge(200) > 0, "a pane that scrolls drew no indicator");
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

// ===========================================================================
// Chapter F components (batch §70)
//
// The corpus (Parts 16–18) asks for four things the shell did not have: a
// tooltip, a picker whose value space is visible, one grid contract, and an
// empty state with a way out. They live here, next to the state vocabulary they
// reuse, so a control cannot grow a second definition on one surface.
// ===========================================================================

/// How long the pointer must rest on a control before its tooltip appears.
///
/// Windows 11 raises a tooltip automatically after a short hover and macOS
/// does the same; 600 ms is long enough that sweeping the pointer across the
/// bar does not flash a label, and short enough that resting on an unlabelled
/// icon explains it. Keyboard focus does not wait: the user has already
/// committed to the control.
pub const TOOLTIP_DELAY_MS: u64 = 600;

/// What a picker or a grid understands as one keyboard step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// One cell backwards, wrapping at the first.
    Previous,
    /// One cell forwards, wrapping at the last.
    Next,
    /// The first cell.
    First,
    /// The last cell.
    Last,
    /// One row up.
    Up,
    /// One row down.
    Down,
}

/// One tooltip's visibility, as a state machine over the clock.
///
/// Nothing here runs in the paint path: [`Tooltip::sync`] and
/// [`Tooltip::tick`] are O(1) comparisons over the shell's clock, and the
/// brush only reads [`Tooltip::visible_for`]. A tooltip that is due is the one
/// wake the idle plan has to honour, reported by [`Tooltip::next_wake_ms`].
#[derive(Debug, Clone, Copy)]
pub struct Tooltip<T> {
    target: Option<T>,
    due_ms: u64,
    shown: bool,
}

impl<T> Default for Tooltip<T> {
    fn default() -> Self {
        Self {
            target: None,
            due_ms: 0,
            shown: false,
        }
    }
}

impl<T: Copy + PartialEq> Tooltip<T> {
    /// Point the tooltip at whatever the pointer or the keyboard is on.
    ///
    /// Focus wins over hover: a keyboard user is looking at the focused
    /// control, and a pointer resting somewhere else must not label it.
    pub fn sync(&mut self, hovered: Option<T>, focused: Option<T>, now_ms: u64) {
        let (want, delay) = match focused {
            Some(focused) => (Some(focused), 0),
            None => (hovered, TOOLTIP_DELAY_MS),
        };
        if want != self.target {
            self.target = want;
            self.due_ms = now_ms.saturating_add(delay);
            self.shown = false;
        }
    }

    /// Advance the clock. Returns whether a tooltip is showing.
    pub fn tick(&mut self, now_ms: u64) -> bool {
        if self.target.is_some() && !self.shown && now_ms >= self.due_ms {
            self.shown = true;
        }
        self.shown
    }

    /// Whether this control's tooltip is the one on screen.
    #[must_use]
    pub fn visible_for(&self, id: T) -> bool {
        self.shown && self.target == Some(id)
    }

    /// What the tooltip is for, if anything.
    #[must_use]
    pub const fn target(&self) -> Option<T> {
        self.target
    }

    /// When the tooltip becomes due, while it is still waiting.
    #[must_use]
    pub const fn next_wake_ms(&self) -> Option<u64> {
        if self.target.is_some() && !self.shown {
            Some(self.due_ms)
        } else {
            None
        }
    }

    /// Forget the current target, e.g. when the surface under it closes.
    pub fn clear(&mut self) {
        self.target = None;
        self.shown = false;
    }
}

/// Draw a tooltip above its anchor, or below when there is no room above.
///
/// The label is supplementary, never the only place a control's name exists
/// (GNOME's rule), and it is clamped to the canvas so a control at the edge of
/// the screen still shows the whole sentence.
pub fn tooltip(canvas: &mut Canvas, t: &Tokens, anchor: Rect, text: &str) -> Rect {
    let style = Style::new(t.text_px(TypeStep::Caption), t.text());
    let pad_x = t.px(8.0);
    let pad_y = t.px(5.0);
    let w = font::measure(text, &style) + pad_x * 2.0;
    let h = t.line_px(TypeStep::Caption) + pad_y * 2.0;
    let cw = canvas.width() as f32;
    let x = (anchor.center().0 - w / 2.0).clamp(t.px(4.0), (cw - w - t.px(4.0)).max(t.px(4.0)));
    let mut y = anchor.y - h - t.px(6.0);
    if y < t.px(4.0) {
        y = anchor.bottom() + t.px(6.0);
    }
    let r = Rect::new(x, y, w, h);
    surface(canvas, t, r, t.radius(Radius::Sm), Material::Elevated);
    canvas.stroke_round_rect(r, t.radius(Radius::Sm), 1.0, t.border());
    font::draw(
        canvas,
        text,
        r.x + pad_x,
        r.y + r.h - pad_y - t.px(2.0),
        &style,
    );
    r
}

/// One choice in a segmented picker.
#[derive(Debug, Clone, Copy)]
pub struct Segment<'a> {
    /// The visible label. The value space is shown, not hidden behind a cycle.
    pub label: &'a str,
}

/// The next choice a picker key asks for.
///
/// Returns `None` when the key would not move: a single-choice picker has no
/// next, and a wrapped step back onto the same choice changes nothing.
#[must_use]
pub fn pick_step(selected: usize, len: usize, step: Step) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len.saturating_sub(1);
    let next = match step {
        Step::Previous | Step::Up => selected.checked_sub(1).unwrap_or(last),
        Step::Next | Step::Down => selected.saturating_add(1).checked_rem(len).unwrap_or(0),
        Step::First => 0,
        Step::Last => last,
    };
    (next != selected).then_some(next)
}

/// Draw a segmented picker and return one rect per drawn segment.
///
/// The segments are the value space: every choice is readable at once, which
/// is the difference between a picker and the cycling row it replaces. The
/// returned rects are what the caller registers as hotspots, so the keyboard
/// path and the pointer path address the same controls.
pub fn segmented(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    segments: &[Segment<'_>],
    selected: usize,
    focused: bool,
) -> Vec<Rect> {
    let mut out = Vec::new();
    if segments.is_empty() || r.w <= 0.0 || r.h <= 0.0 {
        return out;
    }
    let size = t.text_px(TypeStep::Footnote);
    let pad = t.px(10.0);
    let gap = t.px(6.0);
    let mut x = r.x;
    for (i, segment) in segments.iter().enumerate() {
        let w = font::measure(segment.label, &Style::new(size, t.text())) + pad * 2.0;
        if x + w > r.right() {
            break;
        }
        let seg = Rect::new(x, r.y, w, r.h);
        let on = i == selected;
        if on {
            canvas.fill_round_rect(seg, t.radius(Radius::Sm), with_alpha(t.accent(), 0.22));
            canvas.stroke_round_rect(seg, t.radius(Radius::Sm), 1.0, t.accent());
        } else {
            canvas.fill_round_rect(seg, t.radius(Radius::Sm), with_alpha(t.text(), 0.06));
        }
        if on && focused {
            focus_ring(canvas, t, seg, t.radius(Radius::Sm));
        }
        let style = Style::new(size, if on { t.text() } else { t.text_dim() });
        font::draw(
            canvas,
            segment.label,
            seg.x + pad,
            seg.y + seg.h * 0.5 + size * 0.35,
            &style,
        );
        out.push(seg);
        x += w + gap;
    }
    out
}

/// Where one cell of a grid sits, or `None` when it would overflow the area.
///
/// A cell that does not fit is not drawn and not registered: returning a rect
/// outside the grid's own box is how a tile ends up painted under the dock.
#[must_use]
pub fn grid_cell(
    r: Rect,
    columns: usize,
    index: usize,
    cell: (f32, f32),
    gap: f32,
) -> Option<Rect> {
    if columns == 0 || cell.0 <= 0.0 || cell.1 <= 0.0 {
        return None;
    }
    let col = index.checked_rem(columns)?;
    let row = index.checked_div(columns)?;
    let x = r.x + (col as f32) * (cell.0 + gap);
    let y = r.y + (row as f32) * (cell.1 + gap);
    let rect = Rect::new(x, y, cell.0, cell.1);
    (rect.right() <= r.right() + 0.5 && rect.bottom() <= r.bottom() + 0.5).then_some(rect)
}

/// The next cell a grid key asks for, in reading order.
///
/// `Up` and `Down` move a whole row, which is the point of a grid: a list that
/// steps by one on every arrow key cannot be navigated as the rows it draws.
/// The move is clamped rather than wrapped, so holding `Down` on the last row
/// does not teleport the selection back to the first tile.
#[must_use]
pub fn grid_step(selected: usize, count: usize, columns: usize, step: Step) -> Option<usize> {
    if count == 0 || columns == 0 {
        return None;
    }
    let cols = columns.min(count);
    let next = match step {
        Step::Previous | Step::Up => selected.checked_sub(1),
        Step::Next | Step::Down => selected.checked_add(1),
        Step::First => Some(0),
        Step::Last => Some(count.saturating_sub(1)),
    };
    let next = match step {
        Step::Up => selected.checked_sub(cols),
        Step::Down => selected.checked_add(cols).filter(|next| *next < count),
        other => {
            let _ = other;
            next
        }
    };
    next.filter(|n| *n < count && *n != selected)
}

/// The contents of an empty state: a title, a sentence, and a way out.
///
/// This is the GNOME `AdwStatusPage` contract the corpus quotes: the shell's
/// eight empty states were sentences with no action, so a folder with nothing
/// in it did not offer to make anything.
#[derive(Debug, Clone, Copy)]
pub struct EmptyState<'a> {
    /// What the state is, in the user's terms.
    pub title: &'a str,
    /// Why it is empty, or what to do about it.
    pub description: &'a str,
    /// The action's label, when there is one.
    pub action: Option<&'a str>,
}

/// Draw an empty state centred in `r`, returning the action's rect.
///
/// A returned `None` means no action button was drawn, so the caller registers
/// no hotspot: an empty state that cannot act offers nothing to press.
pub fn empty_state(
    canvas: &mut Canvas,
    t: &Tokens,
    r: Rect,
    state: &EmptyState<'_>,
) -> Option<Rect> {
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let title_size = t.text_px(TypeStep::Footnote) * 1.15;
    let title_style = Style::bold(title_size, t.text());
    let body_style = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let action_style = Style::new(t.text_px(TypeStep::Footnote), rgb(0xFFFFFF));
    let cx = r.center().0;
    let mut y = r.y + r.h * 0.38;
    let title_w = font::measure(state.title, &title_style);
    font::draw(
        canvas,
        state.title,
        cx - title_w / 2.0,
        y + title_size,
        &title_style,
    );
    y += title_size + t.px(10.0);
    let body_w = font::measure(state.description, &body_style);
    font::draw(
        canvas,
        state.description,
        cx - body_w / 2.0,
        y + t.text_px(TypeStep::Footnote),
        &body_style,
    );
    y += t.text_px(TypeStep::Footnote) + t.px(18.0);
    let label = state.action?;
    let pad = t.px(14.0);
    let w = font::measure(label, &action_style) + pad * 2.0;
    let h = t.px(30.0);
    if y + h > r.bottom() {
        return None;
    }
    let button = Rect::new(cx - w / 2.0, y, w, h);
    canvas.fill_round_rect(button, t.radius(Radius::Sm), t.accent());
    font::draw(
        canvas,
        label,
        button.x + pad,
        button.y + button.h * 0.5 + t.text_px(TypeStep::Footnote) * 0.35,
        &action_style,
    );
    Some(button)
}

#[cfg(test)]
mod chapter_f_tests {
    use super::*;
    use litric_design::ThemeKind;

    fn tokens() -> Tokens {
        Tokens::new(Theme::of(ThemeKind::Dark), 1.0)
    }

    #[test]
    fn a_tooltip_waits_for_the_rest_and_focus_does_not() {
        let mut tip: Tooltip<u8> = Tooltip::default();
        tip.sync(Some(3), None, 1_000);
        assert!(!tip.tick(1_599), "the pointer just arrived");
        assert_eq!(tip.next_wake_ms(), Some(1_600));
        assert!(tip.tick(1_600));
        assert!(tip.visible_for(3));
        assert_eq!(tip.next_wake_ms(), None, "a shown tooltip asks for no wake");
        tip.sync(None, None, 2_000);
        assert!(!tip.visible_for(3), "leaving clears the label");

        tip.sync(None, Some(4), 3_000);
        assert!(tip.tick(3_000), "focus is already a decision");
        assert!(tip.visible_for(4));
    }

    #[test]
    fn a_picker_step_never_leaves_the_list() {
        assert_eq!(pick_step(0, 6, Step::Previous), Some(5));
        assert_eq!(pick_step(0, 6, Step::Next), Some(1));
        assert_eq!(pick_step(4, 6, Step::Last), Some(5));
        assert_eq!(pick_step(3, 6, Step::First), Some(0));
        assert_eq!(pick_step(0, 1, Step::Next), None, "one choice cannot move");
        assert_eq!(pick_step(0, 0, Step::Next), None, "no choice, no step");
    }

    #[test]
    fn a_segmented_picker_draws_each_choice_it_can_fit() {
        let t = tokens();
        let mut c = Canvas::new(320, 80, rgb(0x101010));
        let segments = [
            Segment { label: "Sky" },
            Segment { label: "Teal" },
            Segment { label: "Rose" },
        ];
        let rects = segmented(
            &mut c,
            &t,
            Rect::new(8.0, 8.0, 300.0, 30.0),
            &segments,
            1,
            true,
        );
        assert_eq!(rects.len(), 3, "every choice is its own control");
        for r in &rects {
            assert!(
                r.x >= 8.0 && r.right() <= 308.0,
                "a segment escaped the picker"
            );
        }
        assert!(
            rects
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a.right() <= b.x)),
            "segments must not overlap"
        );
        let painted = (0i64..80)
            .flat_map(|y| (0i64..320).map(move |x| (x, y)))
            .filter(|(x, y)| c.get(*x, *y).unwrap_or(0) != rgb(0x101010))
            .count();
        assert!(painted > 200, "the picker painted {painted} pixels");
    }

    #[test]
    fn a_grid_cell_is_inside_the_grid_or_not_at_all() {
        let grid = Rect::new(10.0, 20.0, 200.0, 100.0);
        let first = grid_cell(grid, 4, 0, (40.0, 30.0), 6.0).expect("first cell");
        assert_eq!((first.x, first.y), (10.0, 20.0));
        let second = grid_cell(grid, 4, 1, (40.0, 30.0), 6.0).expect("second cell");
        assert_eq!(second.x, 56.0);
        // Four columns of 40 + 6 gap exceed a 170-wide grid, so the fourth
        // cell is refused rather than drawn under what sits beside the grid.
        let narrow = Rect::new(10.0, 20.0, 170.0, 100.0);
        assert!(grid_cell(narrow, 4, 3, (40.0, 30.0), 6.0).is_none());
        assert!(grid_cell(grid, 0, 0, (40.0, 30.0), 6.0).is_none());
    }

    #[test]
    fn a_grid_key_moves_a_row_at_a_time() {
        assert_eq!(grid_step(0, 10, 4, Step::Down), Some(4));
        assert_eq!(grid_step(4, 10, 4, Step::Up), Some(0));
        assert_eq!(grid_step(0, 10, 4, Step::Next), Some(1));
        assert_eq!(
            grid_step(0, 10, 4, Step::Previous),
            None,
            "the first cell stays"
        );
        // The last row is partial: 8 + a row is 12, which does not exist.
        assert_eq!(grid_step(8, 10, 4, Step::Down), None);
        assert_eq!(
            grid_step(9, 10, 4, Step::Last),
            None,
            "already the last cell"
        );
        assert_eq!(
            grid_step(0, 10, 4, Step::First),
            None,
            "already the first cell"
        );
        assert_eq!(
            grid_step(0, 0, 4, Step::Next),
            None,
            "an empty grid cannot move"
        );
    }

    #[test]
    fn an_empty_state_offers_its_action_and_none_when_it_has_none() {
        let t = tokens();
        let mut c = Canvas::new(400, 300, rgb(0x101010));
        let state = EmptyState {
            title: "Empty folder",
            description: "Nothing in this folder yet.",
            action: Some("New folder"),
        };
        let action = empty_state(&mut c, &t, Rect::new(0.0, 0.0, 400.0, 300.0), &state)
            .expect("the action button");
        assert!(action.w > 0.0 && action.h > 0.0);
        assert!(
            action.bottom() <= 300.0,
            "the action stayed inside the pane"
        );

        let none = empty_state(
            &mut c,
            &t,
            Rect::new(0.0, 0.0, 400.0, 300.0),
            &EmptyState {
                title: "A",
                description: "B",
                action: None,
            },
        );
        assert!(none.is_none(), "no action means no hotspot");

        let tight = empty_state(&mut c, &t, Rect::new(0.0, 0.0, 400.0, 60.0), &state);
        assert!(
            tight.is_none(),
            "a pane too short must not draw a clipped button"
        );
    }
}
