from pathlib import Path

def patch(path, pairs):
    p = Path(path)
    text = p.read_text()
    for old, new in pairs:
        n = text.count(old)
        assert n == 1, f"{path}: anchor found {n} times: {old[:60]!r}"
        text = text.replace(old, new)
    p.write_text(text)

patch("src/litric-shell/src/lib.rs", [
    ("pub mod monitor;\n", "pub mod monitor;\npub mod revert;\n"),
])

patch("src/litric-shell/src/shell.rs", [
    (
        "    SettingsMotion,\n",
        """    SettingsMotion,
    /// The UI-scale control on the Display page: cycles the scale and arms the
    /// countdown that puts the old one back.
    DisplayScale,
    /// Keep the scale that is on screen and let the countdown go.
    ScaleKeep,
    /// Put the previous scale back now, without waiting out the countdown.
    ScaleUndo,
""",
    ),
    (
        "    /// Shell-owned language, theme and accessibility settings.\n    pub settings: crate::settings::Model,\n",
        """    /// Shell-owned language, theme and accessibility settings.
    pub settings: crate::settings::Model,
    /// A display change that is still waiting to be confirmed. `Some` means the
    /// value on screen is provisional and the clock in it will put the previous
    /// value back; see [`crate::revert`].
    pub scale_revert: Option<crate::revert::RevertWindow<f32>>,
""",
    ),
    (
        "            facts: SystemFacts::default(),\n",
        "            scale_revert: None,\n            facts: SystemFacts::default(),\n",
    ),
    (
        "            Hot::SettingsMotion => self.toggle_reduced_motion(),\n",
        """            Hot::SettingsMotion => self.toggle_reduced_motion(),
            Hot::DisplayScale => self.cycle_ui_scale(),
            Hot::ScaleKeep => {
                self.keep_pending_change();
                Vec::new()
            }
            Hot::ScaleUndo => {
                self.undo_pending_change();
                Vec::new()
            }
""",
    ),
    (
        "    pub fn tick(&mut self, now_ms: u64) {\n        self.set_now(now_ms);\n",
        """    pub fn tick(&mut self, now_ms: u64) {
        // The countdown for a provisional display change is measured against
        // this clock, so elapsed time is read before the clock moves.
        let dt_ms = now_ms.saturating_sub(self.now_ms);
        self.set_now(now_ms);
        self.advance_pending_change(dt_ms);
""",
    ),
    (
        "    /// A desktop for a display of this size.\n    #[must_use]\n    pub fn new(tokens: Tokens, width: f32, height: f32, regional: Regional) -> Self {\n",
        """    /// The UI scales this build offers, smallest first.
    ///
    /// Three steps, not a slider: each one is a real layout the shell has been
    /// measured at, and a continuous range would promise sizes nobody checked.
    pub const UI_SCALES: [f32; 3] = [1.0, 1.25, 1.5];

    /// How long a display change stays provisional.
    ///
    /// Fifteen seconds is long enough to read the countdown and find the button
    /// on a screen that is too large, and short enough that a black panel is not
    /// the rest of the session.
    pub const REVERT_WINDOW_MS: u64 = 15_000;

    /// Apply a UI scale, provisionally.
    ///
    /// The value is applied immediately — a change that is only described is not
    /// a test of whether the screen is usable — and the previous value is held
    /// until the user keeps it or the window closes. A second change while one
    /// is pending is refused rather than stacked: the alternative is a chain of
    /// undo values in which the one the user wants is no longer reachable.
    pub fn apply_ui_scale(&mut self, scale: f32) {
        let previous = self.tokens.scale;
        if (scale - previous).abs() < 0.001 || self.scale_revert.is_some() {
            return;
        }
        self.set_ui_scale(scale);
        self.scale_revert = Some(crate::revert::RevertWindow::arm(
            previous,
            scale,
            "UI scale",
            Self::REVERT_WINDOW_MS,
        ));
        self.request_full_redraw();
    }

    /// Set the scale without arming anything: the layout path, not the choice.
    fn set_ui_scale(&mut self, scale: f32) {
        self.tokens.scale = scale;
        let screen = self.wm.screen();
        self.wm
            .set_display(screen.w, screen.h, self.tokens.px(Self::TOP_BAR_LOGICAL));
        self.request_full_redraw();
    }

    /// Keep the provisional change: the value stays, the clock stops.
    pub fn keep_pending_change(&mut self) {
        if let Some(mut window) = self.scale_revert.take() {
            window.keep();
        }
        self.request_full_redraw();
    }

    /// Put the previous value back now.
    pub fn undo_pending_change(&mut self) {
        let Some(window) = self.scale_revert.take() else {
            return;
        };
        self.set_ui_scale(window.previous());
    }

    /// Advance the provisional change by the time that has passed.
    fn advance_pending_change(&mut self, dt_ms: u64) {
        let Some(window) = self.scale_revert.as_mut() else {
            return;
        };
        let verdict = window.tick(dt_ms);
        if verdict == crate::revert::Verdict::Expired {
            let previous = window.previous();
            self.scale_revert = None;
            self.set_ui_scale(previous);
        } else {
            // The countdown is drawn text: it only moves if the surface is
            // repainted, and a still desktop has no other reason to repaint.
            self.request_full_redraw();
        }
    }

    /// Step to the next scale, wrapping to the smallest.
    fn cycle_ui_scale(&mut self) -> Vec<Effect> {
        if self.scale_revert.is_some() {
            // The banner is the only control while a change is provisional;
            // another step would make the value being tested ambiguous.
            return Vec::new();
        }
        let current = self.tokens.scale;
        let next = Self::UI_SCALES
            .iter()
            .copied()
            .find(|scale| *scale > current + 0.001)
            .unwrap_or(Self::UI_SCALES[0]);
        self.apply_ui_scale(next);
        Vec::new()
    }

    /// A desktop for a display of this size.
    #[must_use]
    pub fn new(tokens: Tokens, width: f32, height: f32, regional: Regional) -> Self {
""",
    ),
])

patch("src/litric-shell/src/views.rs", [
    (
        """                    (
                        "Display changes".to_owned(),
                        "The framebuffer mode is selected at boot".to_owned(),
                    ),
                ],
            );
        }
        Page::Input => {
""",
        """                    (
                        "Display changes".to_owned(),
                        "The framebuffer mode is selected at boot".to_owned(),
                    ),
                ],
            );
            // The one display value this build can change without a mode-setting
            // service. It is drawn as a control rather than a reading because it
            // is the only row on this page that does something, and because a
            // control that looks like the readings beside it is a control nobody
            // clicks.
            let row = Rect::new(
                pane.x + t.sp(Space::Lg),
                pane.y + t.px(56.0) + t.px(150.0),
                pane.w - t.sp(Space::Lg) * 2.0,
                t.px(34.0),
            );
            ui::surface(canvas, t, row, t.radius(Radius::Md), Material::Popover);
            if shell.is_highlighted(Hot::DisplayScale) {
                ui::focus_ring(canvas, t, row, t.radius(Radius::Md));
            }
            let label_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
            font::draw(
                canvas,
                "UI scale",
                row.x + t.px(10.0),
                row.y + t.px(21.0),
                &label_style,
            );
            let value = format!("{:.2}× · change", shell.tokens.scale);
            let width = font::measure(&value, &label_style);
            font::draw(
                canvas,
                &value,
                row.right() - width - t.px(10.0),
                row.y + t.px(21.0),
                &label_style,
            );
            shell.hotspots_mut().push(row, Hot::DisplayScale);
        }
        Page::Input => {
""",
    ),
    (
        """fn draw_banners(canvas: &mut Canvas, shell: &mut Shell) {
    if shell.overlay == Overlay::Notifications {
        return;
    }
""",
        """fn draw_banners(canvas: &mut Canvas, shell: &mut Shell) {
    if shell.overlay == Overlay::Notifications {
        return;
    }
    draw_revert_banner(canvas, shell);
""",
    ),
    (
        """// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------
""",
        """/// The countdown for a display change that will undo itself.
///
/// It sits below the top bar, centred, because the top bar is the one strip the
/// change cannot move off the screen. Both answers are buttons: a countdown with
/// only "wait" is a countdown a person cannot stop, and the point of the window
/// is that the user gets to decide.
fn draw_revert_banner(canvas: &mut Canvas, shell: &mut Shell) {
    let Some(pending) = shell.scale_revert.as_ref() else {
        return;
    };
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let w = t.px(380.0).min(screen.w * 0.8);
    let r = Rect::new(
        (screen.w - w) / 2.0,
        shell.top_bar().h + t.px(12.0),
        w,
        t.px(96.0),
    );
    ui::surface(canvas, &t, r, t.radius(Radius::Md), Material::Popover);
    let title_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let body_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let title = format!("{} {:.2}× applied", pending.label(), pending.applied());
    font::draw(canvas, &title, r.x + t.px(14.0), r.y + t.px(24.0), &title_style);
    let body = format!(
        "Keep it, or the screen goes back in {}s",
        pending.seconds_left()
    );
    font::draw(canvas, &body, r.x + t.px(14.0), r.y + t.px(44.0), &body_style);
    let button = |canvas: &mut Canvas, t: &Tokens, at: Rect, label: &str, hot: Hot, filled: bool| {
        ui::surface(
            canvas,
            t,
            at,
            t.radius(Radius::Md),
            if filled {
                Material::Popover
            } else {
                Material::Popover
            },
        );
        if shell_highlight(shell, hot) {
            ui::focus_ring(canvas, t, at, t.radius(Radius::Md));
        }
        let style = Style::new(t.text_px(TypeStep::Caption), t.text());
        let width = font::measure(label, &style);
        font::draw(
            canvas,
            label,
            at.x + (at.w - width) / 2.0,
            at.y + at.h / 2.0 + t.px(5.0),
            &style,
        );
        shell.hotspots_mut().push(at, hot);
    };
    let keep = Rect::new(
        r.right() - t.px(14.0) - t.px(78.0) - t.px(8.0) - t.px(78.0),
        r.y + r.h - t.px(14.0) - t.px(28.0),
        t.px(78.0),
        t.px(28.0),
    );
    let undo = Rect::new(keep.right() + t.px(8.0), keep.y, t.px(78.0), t.px(28.0));
    button(canvas, &t, keep, "Keep", Hot::ScaleKeep, true);
    button(canvas, &t, undo, "Revert", Hot::ScaleUndo, false);
}

/// Whether a row should read as hovered or focused, asked through the shell so
/// the banner and every other drawn control answer the same question.
fn shell_highlight(shell: &Shell, hot: Hot) -> bool {
    shell.is_highlighted(hot)
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------
""",
    ),
])

p = Path("src/litric-shell/src/e2e.rs")
text = p.read_text()
text += """
#[test]
fn journey_a_display_scale_change_undoes_itself_unless_it_is_kept() {
    // A scale that is too large moves its own undo control off the edge of the
    // screen, so the only safe design is a change that expires. This journey
    // drives both endings: the user who does nothing gets the old scale back,
    // and the user who says keep keeps the new one.
    let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
    let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
    s.mode = DesktopMode::Ready;

    s.activate(Hot::DisplayScale);
    assert!((s.tokens.scale - 1.25).abs() < 0.001, "the change is applied at once");
    let pending = s.scale_revert.as_ref().expect("a provisional change");
    assert_eq!(pending.previous(), 1.0);
    assert_eq!(pending.seconds_left(), 15);

    // Another step while a change is provisional must not stack: it would make
    // the value being tested ambiguous, and the first undo value unreachable.
    s.activate(Hot::DisplayScale);
    assert!((s.tokens.scale - 1.25).abs() < 0.001);

    s.tick(10_000);
    assert!((s.tokens.scale - 1.25).abs() < 0.001, "still inside the window");
    assert_eq!(
        s.scale_revert.as_ref().map(|w| w.seconds_left()),
        Some(5)
    );
    s.tick(15_500);
    assert!((s.tokens.scale - 1.0).abs() < 0.001, "the window closed and put it back");
    assert!(s.scale_revert.is_none());

    // The other ending: the user keeps it, and the clock no longer applies.
    s.activate(Hot::DisplayScale);
    assert!((s.tokens.scale - 1.25).abs() < 0.001);
    s.activate(Hot::ScaleKeep);
    assert!(s.scale_revert.is_none());
    s.tick(60_000);
    assert!(
        (s.tokens.scale - 1.25).abs() < 0.001,
        "a kept change survives the clock that would have undone it"
    );
}
"""
p.write_text(text)
print("patched")
