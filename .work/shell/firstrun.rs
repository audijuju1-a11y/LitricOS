//! The boot splash and the first-run wizard: everything that happens before
//! the desktop, drawn by the shell itself.
//!
//! Before this existed, first boot was a `passwd` prompt on a black text
//! console — a setup flow the graphical system could not draw, so it lived in
//! a shell script on tty1 and any message the kernel printed landed on top of
//! it. The user never sees a console here: the shell starts, draws its own
//! splash (the wordmark written in the stroke face, left to right, the way a
//! pen would), waits for the system to be genuinely ready, and then walks
//! through the only questions that must be answered before a desktop makes
//! sense — language, password, password again — as pages of the same interface
//! the desktop is made of.
//!
//! Nothing here is decoration over a shortcut: the password leaves the shell
//! as [`crate::shell::Effect::SetPassword`] and is set through the daemon's
//! normal credential path, and the "getting ready" page is shown for exactly
//! as long as that round trip takes.

use litric_design::{Motion, Radius, TypeStep};
use litric_input::event::InputEvent;
use litric_input::keys::Key;

use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::i18n::{self, Language};
use crate::shell::{Effect, Hot, Shell};
use crate::textedit::TextEdit;
use crate::ui::{self, Material, State, Tokens, Tone};

/// The account the wizard sets a password for. The same account the text
/// first-run script named: the desktop account, not root — root's password
/// stays locked on an image whose root filesystem cannot be written.
pub const DESKTOP_USER: &str = "litric";

/// The shortest password the wizard accepts, in characters.
const MIN_PASSWORD: usize = 8;

/// How long the wordmark takes to write itself, in milliseconds.
const REVEAL_MS: u64 = 1_800;

/// The shortest time the splash stays, so the reveal is seen rather than
/// flashed. The splash leaves when this *and* the system are done.
const SPLASH_MIN_MS: u64 = 1_200;

/// The longest the splash waits for the system to answer. A machine whose
/// daemon never comes up still reaches its desktop, with the facts it has: a
/// boot that hangs on a splash is the shape BUG-0022 had, and the ceiling is
/// the difference between waiting and hanging.
const SPLASH_MAX_MS: u64 = 10_000;

/// Which question is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// The wordmark writing itself over the starting system.
    Splash,
    /// "Welcome to LitricOS", with the one button that starts the questions.
    Welcome,
    /// Language: two cards, each named in its own script.
    Language,
    /// The password, typed blind.
    Password,
    /// The password again, typed blind.
    Confirm,
    /// The password is being set; the system finishes coming up behind it.
    Preparing,
}

/// What a click on the wizard can mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Leave the welcome page.
    GetStarted,
    /// Choose Thai.
    ChooseThai,
    /// Choose English.
    ChooseEnglish,
    /// Leave the language page.
    LanguageNext,
    /// Leave the password page.
    PasswordNext,
    /// Submit the two passwords.
    Submit,
    /// Back to the previous question.
    Back,
}

/// The whole boot flow: the splash every boot sees, and the wizard only a
/// first boot needs.
#[derive(Debug)]
pub struct FirstRun {
    /// Which page is on screen.
    pub page: Page,
    /// The language chosen so far; applied to the interface immediately.
    pub language: Language,
    /// The first password field.
    pub password: TextEdit,
    /// The confirmation field.
    pub confirm: TextEdit,
    /// Why the last attempt failed, in the system's or this file's own words.
    pub error: Option<String>,
    /// The `SetPassword` round trip is in flight.
    pub pending: bool,
    /// When the splash began, in the shell's milliseconds.
    pub started_ms: u64,
    /// The system has answered with its facts: services are up far enough for
    /// a desktop. This is the splash's honest readiness, not a timer.
    pub system_ready: bool,
    /// Whether the wizard pages are part of this boot: `None` while the
    /// question is still open. A machine whose language is provisioned
    /// (`litric.first_run=skip`, or a second boot) gets the splash and then
    /// the desktop; `None` at the splash's ceiling counts as "no", because a
    /// wizard over a fact nobody ever gave is worse than a desktop.
    pub wizard: Option<bool>,
    /// Page-transition animation: 0 when a page has just been swapped in, 1
    /// when it has arrived. A spring, so the pages ease rather than jump.
    pub arrival: crate::anim::Anim,
}

impl FirstRun {
    /// Start the boot flow. `wizard` is what the system has said so far about
    /// the questions being needed, and `None` when it has not answered yet.
    #[must_use]
    pub fn new(now_ms: u64, wizard: Option<bool>) -> Self {
        let mut arrival = crate::anim::Anim::at(1.0, Motion::FAST);
        arrival.set_target(1.0);
        Self {
            page: Page::Splash,
            language: Language::default(),
            password: TextEdit::with_limit(128),
            confirm: TextEdit::with_limit(128),
            error: None,
            pending: false,
            started_ms: now_ms,
            system_ready: false,
            wizard,
            arrival,
        }
    }

    /// Whether the splash may leave: the reveal finished, the grace time
    /// passed, and the system answered. All three, so the splash never shows
    /// a finished wordmark over a system that is not there.
    #[must_use]
    pub fn splash_done(&self, now_ms: u64) -> bool {
        let elapsed = now_ms.saturating_sub(self.started_ms);
        elapsed >= SPLASH_MAX_MS
            || (self.system_ready && self.wizard.is_some() && elapsed >= SPLASH_MIN_MS)
    }

    /// How far the wordmark has been written, 0..1, eased.
    #[must_use]
    pub fn reveal(&self, now_ms: u64) -> f32 {
        let elapsed = now_ms.saturating_sub(self.started_ms);
        let t = (elapsed as f32 / REVEAL_MS as f32).clamp(0.0, 1.0);
        // Ease-out cubic: the pen starts fast and settles into the last
        // letters, which is how a hand writes a word.
        let inv = 1.0_f32 - t;
        1.0 - inv * inv * inv
    }

    /// Swap in a page, restarting its arrival animation. The error is
    /// cleared by this call — a new page has not failed at anything yet — so
    /// callers that carry an error across a swap set it *after* calling here.
    pub(crate) fn go(&mut self, page: Page) {
        self.page = page;
        self.error = None;
        self.arrival = crate::anim::Anim::at(0.0, Motion::FAST);
        self.arrival.set_target(1.0);
    }

    /// The password currently offered, or `None` when it is too short to be a
    /// password. The rule is stated on the page that enforces it.
    fn accepted_password(&self) -> Option<&str> {
        let text = self.password.text().trim();
        if text.chars().count() >= MIN_PASSWORD {
            Some(text)
        } else {
            None
        }
    }

    /// Step the page-transition spring.
    pub fn tick(&mut self, dt_ms: u32) -> bool {
        self.arrival.step(dt_ms)
    }

    /// Handle one input event, returning effects for `main`.
    pub fn input(&mut self, ev: &InputEvent) -> Vec<Effect> {
        match ev {
            InputEvent::TextInput { text } => {
                if matches!(self.page, Page::Password | Page::Confirm) {
                    let field = if self.page == Page::Password {
                        &mut self.password
                    } else {
                        &mut self.confirm
                    };
                    field.insert(text);
                }
                Vec::new()
            }
            InputEvent::KeyDown {
                key: Key::Backspace,
                ..
            } => {
                if matches!(self.page, Page::Password | Page::Confirm) {
                    let field = if self.page == Page::Password {
                        &mut self.password
                    } else {
                        &mut self.confirm
                    };
                    field.backspace(false);
                }
                Vec::new()
            }
            InputEvent::KeyDown {
                key: Key::Enter, ..
            } => match self.page {
                Page::Splash => Vec::new(),
                Page::Welcome => {
                    self.go(Page::Language);
                    Vec::new()
                }
                Page::Language => {
                    self.go(Page::Password);
                    Vec::new()
                }
                Page::Password => self.leave_password(),
                Page::Confirm => self.submit(),
                Page::Preparing => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// A click landed on one of the wizard's own controls. The shell
    /// hit-tests its hotspots and hands the winner here, the same contract
    /// every other surface has.
    pub fn activate(&mut self, hot: Action) -> Vec<Effect> {
        match hot {
            Action::GetStarted => {
                self.go(Page::Language);
                Vec::new()
            }
            Action::ChooseThai => {
                self.language = Language::Thai;
                i18n::set_current(Language::Thai);
                Vec::new()
            }
            Action::ChooseEnglish => {
                self.language = Language::English;
                i18n::set_current(Language::English);
                Vec::new()
            }
            Action::LanguageNext => {
                self.go(Page::Password);
                Vec::new()
            }
            Action::PasswordNext => self.leave_password(),
            Action::Submit => self.submit(),
            Action::Back => {
                match self.page {
                    Page::Language => self.go(Page::Welcome),
                    Page::Password => self.go(Page::Language),
                    Page::Confirm => self.go(Page::Password),
                    _ => {}
                }
                Vec::new()
            }
        }
    }

    /// Leave the password page: accept it, or say why not, in place.
    fn leave_password(&mut self) -> Vec<Effect> {
        if self.accepted_password().is_none() {
            self.error = Some(i18n::text("Use at least 8 characters").into_owned());
            return Vec::new();
        }
        self.go(Page::Confirm);
        Vec::new()
    }

    /// Submit both fields. Matching and long enough is the only thing this
    /// file checks; whether the system accepts the password is the daemon's
    /// answer, shown verbatim if it refuses.
    fn submit(&mut self) -> Vec<Effect> {
        if self.accepted_password().is_none() {
            self.go(Page::Password);
            self.error = Some(i18n::text("Use at least 8 characters").into_owned());
            return Vec::new();
        }
        if self.password.text() != self.confirm.text() {
            self.go(Page::Confirm);
            self.error = Some(i18n::text("Passwords do not match").into_owned());
            return Vec::new();
        }
        self.pending = true;
        self.go(Page::Preparing);
        vec![Effect::SetPassword {
            user: DESKTOP_USER.to_owned(),
            secret: crate::lock::Secret::new(self.password.text()),
        }]
    }

    /// The daemon answered the password request.
    pub fn password_result(&mut self, ok: bool, reason: Option<String>) -> bool {
        self.pending = false;
        if ok {
            return true;
        }
        self.go(Page::Password);
        self.error = Some(
            reason.unwrap_or_else(|| i18n::text("The password could not be set").into_owned()),
        );
        false
    }
}

/// The wizard's fields are drawn with dots, not glyphs: the same reason the
/// lock screen draws them that way — a shoulder surfer reads the screen, and a
/// password field that echoes is a password field that leaks.
fn draw_dots(canvas: &mut Canvas, t: &Tokens, rect: Rect, count: usize, focused: bool) {
    let radius = t.px(8.0);
    canvas.fill_round_rect(rect, radius, with_alpha(rgb(0xFFFFFF), 0.06));
    canvas.stroke_round_rect(
        rect,
        radius,
        1.0,
        if focused {
            with_alpha(t.accent(), 0.9)
        } else {
            with_alpha(rgb(0xFFFFFF), 0.12)
        },
    );
    let dot = t.px(3.0);
    let gap = t.px(11.0);
    let y = rect.y + rect.h / 2.0;
    let mut x = rect.x + t.px(16.0);
    for _ in 0..count {
        if x > rect.right() - t.px(16.0) {
            break;
        }
        canvas.fill_circle(x, y, dot, with_alpha(t.text(), 0.85));
        x += gap;
    }
}

/// A page title and its one-line explanation.
fn heading(
    canvas: &mut Canvas,
    t: &Tokens,
    x: f32,
    y: f32,
    w: f32,
    title: &str,
    body: &str,
) -> f32 {
    font::draw(canvas, title, x, y, &t.style_bold(TypeStep::Title3));
    let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let mut cursor = y + t.px(28.0);
    let mut line = String::new();
    for word in body.split_whitespace() {
        let trial = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if font::measure(&trial, &s) > w && !line.is_empty() {
            font::draw(canvas, &line, x, cursor, &s);
            cursor += t.px(20.0);
            line = word.to_owned();
        } else {
            line = trial;
        }
    }
    if !line.is_empty() {
        font::draw(canvas, &line, x, cursor, &s);
        cursor += t.px(20.0);
    }
    cursor
}

// --- drawing -----------------------------------------------------------

/// Draw the boot flow over the whole screen and return its hotspots, for the
/// shell to register against exactly what was drawn. The splash has no
/// controls: it leaves when the system is ready, not when it is clicked.
#[must_use]
pub fn draw(canvas: &mut Canvas, shell: &Shell) -> Vec<(Rect, Hot)> {
    let Some(fr) = shell.first_run.as_ref() else {
        return Vec::new();
    };
    let t = shell.tokens.clone();
    // The window manager thinks in its own geometry; the canvas in its own.
    // Converting here means every drawing routine below can take one `Rect`
    // type and never mention the difference again.
    let s = shell.wm.screen();
    let screen = Rect::new(s.x, s.y, s.w, s.h);
    let now = shell.now_ms();
    let mut hits = Vec::new();

    // The boot flow owns the screen: an opaque base, because there is no
    // desktop behind it yet and a half-started system must never show through.
    canvas.fill_vertical_gradient(screen, rgb(0x0B1220), rgb(0x05080E));

    match fr.page {
        Page::Splash => splash(canvas, &t, fr, screen, now),
        Page::Welcome => welcome(canvas, shell, &t, fr, screen, &mut hits),
        Page::Language => language(canvas, shell, &t, fr, screen, &mut hits),
        Page::Password => credentials(canvas, shell, &t, fr, screen, &mut hits, false),
        Page::Confirm => credentials(canvas, shell, &t, fr, screen, &mut hits, true),
        Page::Preparing => preparing(canvas, &t, fr, screen, now),
    }
    hits
}

/// One wizard page: a centred glass card on the boot background.
fn page_card(canvas: &mut Canvas, t: &Tokens, screen: Rect) -> Rect {
    let w = (screen.w - t.px(96.0)).min(t.px(560.0)).max(t.px(300.0));
    let h = t.px(356.0);
    let r = Rect::new(
        screen.x + (screen.w - w) / 2.0,
        screen.y + (screen.h - h) / 2.0,
        w,
        h,
    );
    ui::surface(canvas, t, r, t.radius(Radius::Md), Material::Glass);
    r
}

/// A wizard button, drawn and registered in the same step so the hotspot can
/// never disagree with the pixels.
fn wizard_button(
    canvas: &mut Canvas,
    shell: &Shell,
    hits: &mut Vec<(Rect, Hot)>,
    r: Rect,
    label: &str,
    action: Action,
    tone: Tone,
) {
    let t = &shell.tokens;
    let hot = Hot::FirstRun(action);
    let state = if shell.is_highlighted(hot) {
        State::Hover
    } else {
        State::Normal
    };
    let drawn = ui::button(canvas, t, r, label, state, tone);
    hits.push((drawn, hot));
}

/// The back / primary pair at the foot of a wizard page.
fn nav_row(
    canvas: &mut Canvas,
    shell: &Shell,
    hits: &mut Vec<(Rect, Hot)>,
    card: Rect,
    primary_label: &str,
    primary: Action,
    with_back: bool,
) {
    let t = &shell.tokens;
    let pad = t.px(28.0);
    let h = t.px(40.0);
    let y = card.bottom() - pad - h;
    if with_back {
        wizard_button(
            canvas,
            shell,
            hits,
            Rect::new(card.x + pad, y, t.px(110.0), h),
            &i18n::text("Back"),
            Action::Back,
            Tone::Neutral,
        );
    }
    let w = t.px(176.0);
    wizard_button(
        canvas,
        shell,
        hits,
        Rect::new(card.right() - pad - w, y, w, h),
        primary_label,
        primary,
        Tone::Accent,
    );
}

/// The splash: the wordmark writing itself in, left to right, over a system
/// that is still coming up. The reveal is a clip wipe with a faint ghost of
/// the whole word under it and a pen-tip glow at the moving edge — the word
/// appears to be written, not faded in.
fn splash(canvas: &mut Canvas, t: &Tokens, fr: &FirstRun, screen: Rect, now: u64) {
    let cx = screen.x + screen.w / 2.0;
    let cy = screen.y + screen.h / 2.0;
    canvas.fill_circle(
        cx,
        cy - t.px(24.0),
        screen.h * 0.34,
        with_alpha(t.accent(), 0.05),
    );

    let size = (screen.w * 0.075).clamp(34.0, 88.0);
    let word = "LitricOS";
    let bright = Style::new(size, t.text());
    let ghost = Style::new(size, with_alpha(t.text(), 0.10));
    let w = font::measure(word, &bright);
    let x = cx - w / 2.0;
    let baseline = cy - t.px(6.0);
    font::draw(canvas, word, x, baseline, &ghost);

    let reveal = fr.reveal(now);
    if reveal >= 1.0 {
        font::draw(canvas, word, x, baseline, &bright);
    } else if reveal > 0.0 {
        let saved = canvas.set_clip(Some(Rect::new(
            x - t.px(6.0),
            baseline - size * 1.3,
            w * reveal + t.px(12.0),
            size * 1.8,
        )));
        font::draw(canvas, word, x, baseline, &bright);
        canvas.set_clip(saved);
    }
    // The pen tip: a soft accent dot at the edge of the written part, gone by
    // the time the word is complete.
    if reveal > 0.02 && reveal < 0.995 {
        let tip_x = x + w * reveal;
        let tip_y = baseline - size * 0.22;
        canvas.fill_circle(tip_x, tip_y, size * 0.075, with_alpha(t.accent(), 0.85));
        canvas.fill_circle(tip_x, tip_y, size * 0.16, with_alpha(t.accent(), 0.14));
    }

    // Readiness, stated as three milestones that light only when true: the
    // display is up (this code is drawing on it), the daemon answered with
    // facts, and the splash is done holding.
    let seg_w = t.px(56.0);
    let seg_gap = t.px(10.0);
    let seg_h = t.px(4.0);
    let total = seg_w * 3.0 + seg_gap * 2.0;
    let mut sx = cx - total / 2.0;
    let sy = baseline + t.px(34.0);
    let done = fr.splash_done(now);
    for lit in [true, fr.system_ready, done] {
        canvas.fill_round_rect(
            Rect::new(sx, sy, seg_w, seg_h),
            seg_h / 2.0,
            if lit {
                with_alpha(t.accent(), 0.9)
            } else {
                with_alpha(rgb(0xFFFFFF), 0.12)
            },
        );
        sx += seg_w + seg_gap;
    }

    let caption = if done {
        i18n::text("Preparing your desktop")
    } else if fr.system_ready {
        i18n::text("Almost there")
    } else {
        i18n::text("Starting LitricOS")
    };
    let style = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let cw = font::measure(&caption, &style);
    font::draw(canvas, &caption, cx - cw / 2.0, sy + t.px(30.0), &style);
}

/// Welcome: the first question is whether to start at all.
fn welcome(
    canvas: &mut Canvas,
    shell: &Shell,
    t: &Tokens,
    fr: &FirstRun,
    screen: Rect,
    hits: &mut Vec<(Rect, Hot)>,
) {
    let card = page_card(canvas, t, screen);
    let dy = (1.0 - fr.arrival.value()) * t.px(18.0);
    let pad = t.px(28.0);
    let x = card.x + pad;
    let w = card.w - pad * 2.0;
    heading(
        canvas,
        t,
        x,
        card.y + pad + t.px(30.0) + dy,
        w,
        &i18n::text("Welcome to LitricOS"),
        &i18n::text(
            "A calm, complete desktop that stays out of your way. A few quick questions and it is yours.",
        ),
    );
    nav_row(
        canvas,
        shell,
        hits,
        card,
        &i18n::text("Get started"),
        Action::GetStarted,
        false,
    );
}

/// Language: two cards, each named in its own script — the choice is shown
/// the way its own users read it.
fn language(
    canvas: &mut Canvas,
    shell: &Shell,
    t: &Tokens,
    fr: &FirstRun,
    screen: Rect,
    hits: &mut Vec<(Rect, Hot)>,
) {
    let card = page_card(canvas, t, screen);
    let dy = (1.0 - fr.arrival.value()) * t.px(18.0);
    let pad = t.px(28.0);
    let x = card.x + pad;
    let w = card.w - pad * 2.0;
    let after = heading(
        canvas,
        t,
        x,
        card.y + pad + t.px(26.0) + dy,
        w,
        &i18n::text("Choose your language"),
        &i18n::text("You can change this later in Settings."),
    );

    let gap = t.px(14.0);
    let cw = (w - gap) / 2.0;
    let ch = t.px(88.0);
    let cy = after + t.px(12.0) + dy;
    let options = [
        ("ไทย", "ภาษาไทย", Action::ChooseThai, Language::Thai),
        (
            "English",
            "English",
            Action::ChooseEnglish,
            Language::English,
        ),
    ];
    let mut ox = x;
    for (name, sub, action, lang) in options {
        let r = Rect::new(ox, cy, cw, ch);
        let selected = fr.language == lang;
        ui::surface(canvas, t, r, t.radius(Radius::Md), Material::Glass);
        canvas.stroke_round_rect(
            r,
            t.radius(Radius::Md),
            1.5,
            if selected {
                t.accent()
            } else {
                with_alpha(rgb(0xFFFFFF), 0.10)
            },
        );
        let hot = Hot::FirstRun(action);
        if shell.is_highlighted(hot) && !selected {
            canvas.stroke_round_rect(r, t.radius(Radius::Md), 1.0, with_alpha(t.accent(), 0.45));
        }
        font::draw(
            canvas,
            name,
            ox + t.px(18.0),
            cy + t.px(36.0),
            &t.style_bold(TypeStep::Body),
        );
        font::draw(
            canvas,
            sub,
            ox + t.px(18.0),
            cy + t.px(62.0),
            &Style::new(t.text_px(TypeStep::Footnote), t.text_dim()),
        );
        if selected {
            let d = t.px(6.0);
            canvas.fill_circle(ox + cw - t.px(20.0), cy + t.px(20.0), d, t.accent());
        }
        hits.push((r, hot));
        ox += cw + gap;
    }

    nav_row(
        canvas,
        shell,
        hits,
        card,
        &i18n::text("Next"),
        Action::LanguageNext,
        true,
    );
}

/// The password pages: the same drawing for both fields, named by the page.
fn credentials(
    canvas: &mut Canvas,
    shell: &Shell,
    t: &Tokens,
    fr: &FirstRun,
    screen: Rect,
    hits: &mut Vec<(Rect, Hot)>,
    confirm: bool,
) {
    let card = page_card(canvas, t, screen);
    let dy = (1.0 - fr.arrival.value()) * t.px(18.0);
    let pad = t.px(28.0);
    let x = card.x + pad;
    let w = card.w - pad * 2.0;
    let (title, body) = if confirm {
        (
            i18n::text("Confirm your password"),
            i18n::text("Type the same password once more."),
        )
    } else {
        (
            i18n::text("Set a password"),
            i18n::text("Use at least 8 characters. It unlocks your desktop and your files."),
        )
    };
    let after = heading(
        canvas,
        t,
        x,
        card.y + pad + t.px(26.0) + dy,
        w,
        &title,
        &body,
    );

    let field = Rect::new(x, after + t.px(10.0) + dy, w, t.px(52.0));
    let count = if confirm {
        fr.confirm.text().chars().count()
    } else {
        fr.password.text().chars().count()
    };
    draw_dots(canvas, t, field, count, true);

    if let Some(err) = &fr.error {
        let style = Style::new(t.text_px(TypeStep::Footnote), t.tone(Tone::Bad));
        font::draw(canvas, err, x, field.bottom() + t.px(28.0) + dy, &style);
    }

    let (label, action) = if confirm {
        (i18n::text("Set password"), Action::Submit)
    } else {
        (i18n::text("Next"), Action::PasswordNext)
    };
    nav_row(canvas, shell, hits, card, &label, action, true);
}

/// "Getting things ready for you": the password is on its way to the daemon
/// and the system finishes coming up behind it. The dots move because work is
/// actually in flight — this page is never a timer.
fn preparing(canvas: &mut Canvas, t: &Tokens, fr: &FirstRun, screen: Rect, now: u64) {
    let card = page_card(canvas, t, screen);
    let dy = (1.0 - fr.arrival.value()) * t.px(18.0);
    let pad = t.px(28.0);
    let x = card.x + pad;
    let w = card.w - pad * 2.0;
    let after = heading(
        canvas,
        t,
        x,
        card.y + pad + t.px(30.0) + dy,
        w,
        &i18n::text("Getting things ready for you"),
        &i18n::text(if fr.pending {
            "Setting your password…"
        } else {
            "Preparing your desktop…"
        }),
    );

    // Three marching dots, phase-shifted, at the accent. f32 maths is the
    // allowed kind here: this is motion, not counting.
    let step = t.px(20.0);
    let y = after + t.px(30.0) + dy;
    let start_x = x + w / 2.0 - step;
    for i in 0..3 {
        let phase = now as f32 / 260.0 - i as f32 * 0.8;
        let bob = phase.sin() * t.px(5.0);
        let alpha = 0.45 + 0.55 * (phase.sin() + 1.0) / 2.0;
        canvas.fill_circle(
            start_x + i as f32 * step,
            y + bob,
            t.px(4.5),
            with_alpha(t.accent(), alpha),
        );
    }
}

#[cfg(test)]
mod tests {
    use litric_input::keys::Modifiers;

    use super::*;

    fn enter() -> InputEvent {
        InputEvent::KeyDown {
            key: Key::Enter,
            code: 28,
            mods: Modifiers::NONE,
            repeat: false,
        }
    }

    #[test]
    fn the_splash_leaves_when_the_system_answered_and_at_the_ceiling_when_it_never_does() {
        let mut fr = FirstRun::new(1_000, Some(false));
        assert!(!fr.splash_done(1_000), "not at zero elapsed");
        assert!(!fr.splash_done(2_500), "not before the system answers");
        fr.system_ready = true;
        assert!(!fr.splash_done(2_000), "not inside the grace time");
        assert!(fr.splash_done(2_200), "the answer plus the grace is done");
        // The question "does this machine need setup" is part of being ready:
        // an answer that has not arrived keeps the splash holding, not the
        // wizard guessing.
        let mut open = FirstRun::new(0, None);
        open.system_ready = true;
        assert!(!open.splash_done(3_000), "the question is still open");
        // A machine whose daemon never answers still reaches its desktop:
        // waiting is not hanging.
        let stuck = FirstRun::new(0, Some(true));
        assert!(stuck.splash_done(SPLASH_MAX_MS));
    }

    #[test]
    fn the_reveal_writes_the_word_from_the_left_and_ends_complete() {
        let fr = FirstRun::new(0, Some(true));
        assert!(fr.reveal(0) <= f32::EPSILON, "nothing written at the start");
        assert!(
            (fr.reveal(REVEAL_MS) - 1.0).abs() <= f32::EPSILON,
            "the whole word at the end"
        );
        // Ease-out: the hand starts fast, so half the time has written more
        // than half the word — and never less than half, or the wipe would
        // crawl at the start.
        let half = fr.reveal((REVEAL_MS as f32 / 2.0) as u64);
        assert!(half > 0.5 && half < 1.0, "halfway wrote {half}");
        let mut prev = 0.0;
        for step in 0..=10_u64 {
            let v = fr.reveal((REVEAL_MS as f32 * step as f32 / 10.0) as u64);
            assert!(v >= prev, "the reveal never runs backwards");
            prev = v;
        }
    }

    #[test]
    fn a_short_password_is_refused_in_place_with_a_reason() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Password);
        fr.password.insert("short");
        assert!(
            fr.activate(Action::PasswordNext).is_empty(),
            "no effect leaves"
        );
        assert_eq!(fr.page, Page::Password, "it stays where the problem is");
        assert!(fr.error.is_some(), "and it says why");
    }

    #[test]
    fn mismatched_passwords_are_refused_on_the_confirm_page() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Password);
        fr.password.insert("correct horse");
        fr.activate(Action::PasswordNext);
        assert_eq!(fr.page, Page::Confirm);
        fr.confirm.insert("correct hrose");
        assert!(fr.activate(Action::Submit).is_empty());
        assert_eq!(
            fr.page,
            Page::Confirm,
            "the second field is where the mismatch is"
        );
        assert!(fr.error.is_some());
    }

    #[test]
    fn a_matching_password_leaves_the_shell_as_exactly_one_set_password() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Password);
        fr.password.insert("correct horse");
        fr.activate(Action::PasswordNext);
        fr.confirm.insert("correct horse");
        let effects = fr.activate(Action::Submit);
        assert_eq!(fr.page, Page::Preparing);
        assert!(fr.pending, "the round trip is in flight");
        assert_eq!(effects.len(), 1, "one secret, one effect: {effects:?}");
        assert!(
            matches!(&effects[..], [Effect::SetPassword { user, .. }] if user == DESKTOP_USER),
            "{effects:?}"
        );
        assert!(fr.password_result(true, None), "the daemon's yes ends it");
    }

    #[test]
    fn a_refused_password_shows_the_daemons_own_words_and_asks_again() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Password);
        fr.password.insert("correct horse");
        fr.activate(Action::PasswordNext);
        fr.confirm.insert("correct horse");
        fr.activate(Action::Submit);
        assert!(!fr.password_result(false, Some("chpasswd exited 1".to_owned())));
        assert_eq!(fr.page, Page::Password);
        // Verbatim: the daemon's words are the diagnosis, and a paraphrase
        // would throw away exactly the part a person needs.
        assert_eq!(fr.error.as_deref(), Some("chpasswd exited 1"));
    }

    #[test]
    fn enter_walks_the_questions_forward_and_back_walks_them_back() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Welcome);
        let _ = fr.input(&enter());
        assert_eq!(fr.page, Page::Language);
        let _ = fr.input(&enter());
        assert_eq!(fr.page, Page::Password);
        let _ = fr.activate(Action::Back);
        assert_eq!(fr.page, Page::Language);
        let _ = fr.activate(Action::Back);
        assert_eq!(fr.page, Page::Welcome);
    }

    #[test]
    fn choosing_a_language_moves_the_selection_without_leaving_the_page() {
        let mut fr = FirstRun::new(0, Some(true));
        fr.go(Page::Language);
        assert!(fr.activate(Action::ChooseThai).is_empty());
        assert_eq!(fr.language, Language::Thai);
        assert_eq!(fr.page, Page::Language, "a card is a choice, not a jump");
        let _ = fr.activate(Action::ChooseEnglish);
        assert_eq!(fr.language, Language::English);
    }
}
