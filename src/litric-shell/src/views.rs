//! Drawing the desktop from the shell's state.
//!
//! Every function here reads [`crate::shell::Shell`] and writes pixels. None of
//! them decides anything: if a panel is open, the shell decided that, and if a
//! row is highlighted the shell decided that too. The one thing this layer does
//! own is **where** things are, and it registers every interactive rectangle in
//! the shell's hotspot table as it draws — so what you can click is by
//! construction what you can see.

use litric_design::{Radius, Space, TypeStep};
use litric_wm::WindowState;

use crate::appicons;
use crate::apps::{FileKind, FilesHot, SortKey};
use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::chrome;
use crate::font::{self, Style};
use crate::icons::{self, Icon};
use crate::shell::{Hot, Overlay, PowerCommand, PowerProfile, Priority, Shell};
use crate::shortcuts::App;
use crate::ui::{self, Material, State, Tokens, Tone};

/// Draw the whole desktop.
///
/// The caller has already restored the wallpaper into whatever regions are
/// dirty; this repaints every element over it. Elements outside the damaged
/// area are redrawn identically, which costs a little arithmetic and removes an
/// entire class of bug where a partly-drawn frame is presented.
pub fn draw(canvas: &mut Canvas, shell: &mut Shell) {
    crate::i18n::set_current(shell.settings.preferences.language);
    shell.hotspots_mut().clear();
    // The boot intro is its own surface: while it covers the screen nothing of
    // the desktop is drawn under it, so there is nothing to hit and nothing to
    // show through. Its last moments are a cross-fade into whatever comes
    // next (the first-run setup, the lock or the desktop), which is drawn
    // underneath for exactly that long.
    if let Some(progress) = shell.boot_intro_now() {
        let cover = shell.boot_intro_cover();
        if cover >= 1.0 {
            crate::wordmark::draw_intro(canvas, progress);
            return;
        }
        draw_surface(canvas, shell);
        let mut mark = Canvas::new(canvas.width(), canvas.height(), 0);
        crate::wordmark::draw_intro(&mut mark, 1.0);
        canvas.mix_from(&mark, cover);
        return;
    }
    draw_surface(canvas, shell);
}

/// Whatever the shell shows once the intro is out of the way.
fn draw_surface(canvas: &mut Canvas, shell: &mut Shell) {
    // Locked: one surface, and nothing else runs. The lock surface starts by
    // clearing the frame buffer, so what was on screen is gone from the pixels
    // as well as from the input path -- the "clear, then draw" order
    // docs/design/wave1/04-authentication.md U7 requires.
    if shell.lock.locked {
        draw_lock(canvas, shell);
        return;
    }
    // First run: until setup is recorded as done, the desktop stays hidden and
    // the setup panel is the only surface. Its choices reuse the settings
    // handlers, so each one is saved and rolls back the same way.
    if !shell.settings.preferences.onboarded {
        draw_setup(canvas, shell);
        return;
    }
    // Mission Control is a place, not a filter over the desktop. Windows and
    // the dock would be drawn underneath a modal that cannot reach them, and
    // their titles showing through the cards reads as a rendering fault.
    let overview = matches!(shell.overlay, Overlay::Overview { .. });
    if !overview {
        draw_windows(canvas, shell);
        // Menus and tooltips from client applications sit above every window
        // and below the shell's own overlays, which is where a popup belongs.
        shell.draw_client_popups(canvas);
    }
    draw_top_bar(canvas, shell);
    // The launcher is the dock's expanded state and the only state it has left:
    // at rest it draws nothing (see `draw_dock`), so this call is the launcher
    // surface and its closing animation.
    draw_dock(canvas, shell);
    // Overlays (launcher, switcher, quick settings, notifications) draw their own
    // opaque surfaces above everything else, so text under them is covered, not
    // collided with. Tagging them as their own surface lets the layout audit say
    // so instead of reporting a menu over a window as a defect.
    let surface = canvas.set_surface(u32::MAX);
    // A surface that just closed is still pixels: draw it beneath whatever
    // replaced it, through the same reveal spring, for as long as that spring
    // has anything left to show. `draw_retired` diverts its hotspots to a
    // scratch table, so a menu on its way out cannot take the click.
    if let Some(old) = shell.retiring_overlay().cloned() {
        shell.draw_retired(|sh| draw_overlay(canvas, sh, &old));
    }
    let live = shell.overlay.clone();
    draw_overlay(canvas, shell, &live);
    canvas.set_surface(surface);
    draw_banners(canvas, shell);
    draw_context_menu(canvas, shell);
    draw_desktop_dialog(canvas, shell);
    if shell.diagnostics {
        draw_diagnostics(canvas, shell);
    }
    // The pointer is the compositor's: it keeps the pixels the arrow covers so
    // a move is a restore and one small draw, not a whole frame. `views::draw`
    // therefore draws everything except the cursor.
}

/// The lock screen: a surface that covers everything and answers to one thing.
///
/// The lock's drawing lives in [`crate::lock`] because it shares nothing with
/// the desktop: no windows, no top bar, no damage-driven repaint. What this
/// function does is hand it the measured facts it shows and register the
/// rectangles it drew as hotspots, which is the same contract every other view
/// here has.
fn draw_lock(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let seconds = shell.clock_seconds();
    let mut hits: Vec<(Rect, Hot)> = Vec::new();
    // The giant clock is the digits. Whatever follows them ("UTC" when the zone
    // is not known, "AM"/"PM" on a 12-hour locale) is kept, as honest as ever,
    // but moves to the date line instead of running the digits off the screen.
    let clock_text = shell.regional.clock(seconds);
    let (digits, suffix) = match clock_text.split_once(' ') {
        Some((d, s)) => (d.to_owned(), Some(s.to_owned())),
        None => (clock_text, None),
    };
    let date = match suffix {
        Some(s) => format!("{} \u{b7} {s}", shell.regional.date(seconds)),
        None => shell.regional.date(seconds),
    };
    {
        let view = crate::lock::View {
            t: &t,
            state: &shell.lock,
            clock: digits,
            date,
            w: screen.w,
            h: screen.h,
            battery: shell.facts.battery_percent,
            network: shell.facts.network.clone(),
            uptime: shell.facts.uptime_secs,
            verifier: shell.facts.auth_available.clone(),
        };
        crate::lock::draw(canvas, &view, &mut shell.lock_cache, &mut hits);
    }
    for (r, hot) in hits {
        shell.hotspots_mut().push(r, hot);
    }
}

// ---------------------------------------------------------------------------
// Context menu and desktop dialog
// ---------------------------------------------------------------------------

/// The open context menu, if any. Drawn above everything but the pointer.
fn draw_context_menu(canvas: &mut Canvas, shell: &mut Shell) {
    // The menu that is leaving draws first, beneath the one that replaced it.
    if let Some(old) = shell.retiring_context_menu().cloned() {
        shell.draw_retired(|sh| draw_context_menu_panel(canvas, sh, &old));
    }
    if let Some(menu) = shell.context_menu.clone() {
        draw_context_menu_panel(canvas, shell, &menu);
    }
}

/// One context menu at its resting place. Its reveal spring is the same one
/// that retires it, which is what makes a close visible.
fn draw_context_menu_panel(
    canvas: &mut Canvas,
    shell: &mut Shell,
    menu: &crate::apps::ContextMenu,
) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let row_h = t.px(26.0);
    let w = t.px(190.0);
    let h = menu.height(row_h);
    // Keep the menu on screen: a menu that opens off the edge is a bug.
    let x = menu.at.0.min(screen.w - w - t.px(4.0));
    let y = menu.at.1.min(screen.h - h - t.px(4.0));
    let panel = Rect::new(x, y, w, h);
    // Menus open fast rather than animated: a context menu is a request to
    // act, not a surface to admire. The spring still exists so that a
    // close can be observed and a re-open cannot pop mid-frame.
    let reveal = shell.anims().context_menu.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = h * reveal;
    let saved = canvas.push_clip(Rect::new(x, y, w, shown_h));
    ui::surface(canvas, &t, panel, t.radius(Radius::Md), Material::Popover);
    let mut ry = y + t.px(4.0);
    for item in &menu.items {
        match item {
            crate::apps::MenuItem::Section(label) => {
                let s = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
                font::draw(canvas, label, x + t.px(12.0), ry + t.px(15.0), &s);
                ry += row_h;
            }
            crate::apps::MenuItem::Separator => {
                ui::separator(
                    canvas,
                    &t,
                    Rect::new(x + t.px(8.0), ry + row_h * 0.25, w - t.px(16.0), 1.0),
                );
                ry += row_h * 0.5;
            }
            crate::apps::MenuItem::Item(label, action) => {
                let hot = Hot::MenuRow(*action);
                let row = Rect::new(x + t.px(4.0), ry, w - t.px(8.0), row_h);
                let hovered = shell.is_highlighted(hot);
                if hovered {
                    canvas.fill_round_rect(
                        row,
                        t.radius(Radius::Sm),
                        with_alpha(rgb(0xFFFFFF), 0.09),
                    );
                }
                let s = Style::new(t.text_px(TypeStep::Footnote), t.text());
                font::draw(
                    canvas,
                    label,
                    row.x + t.px(8.0),
                    row.y + row.h / 2.0 + s.size * 0.36,
                    &s,
                );
                shell.hotspots_mut().push(row, hot);
                ry += row_h;
            }
        }
    }
    canvas.set_clip(saved);
    // Rows were registered above; a click on empty panel space dismisses the
    // menu through the pointer's press-elsewhere path (Hot::Desktop on the
    // wallpaper). No scrim hotspot here: pushing one after the rows would
    // shadow them, because `hit` picks the last registration.
}

/// The desktop's own modal dialog: New folder from the wallpaper menu, the
/// confirmation for a power transition, or an error to acknowledge.
fn draw_desktop_dialog(canvas: &mut Canvas, shell: &mut Shell) {
    if let Some(old) = shell.retiring_desktop_dialog().cloned() {
        shell.draw_retired(|sh| draw_desktop_dialog_panel(canvas, sh, &old, true));
    }
    if let Some(dialog) = shell.desktop_dialog.clone() {
        draw_desktop_dialog_panel(canvas, shell, &dialog, false);
    }
}

/// One dialog panel. `leaving` suppresses the scrim: a dialog on its way out
/// lets the desktop show while its panel collapses, rather than holding a
/// full-screen dim until the exit ends and then dropping it in one frame.
fn draw_desktop_dialog_panel(
    canvas: &mut Canvas,
    shell: &mut Shell,
    dialog: &crate::apps::DesktopDialog,
    leaving: bool,
) {
    use crate::apps::DesktopDialog;
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let reveal = shell.anims().dialog.value();
    if reveal <= 0.01 {
        return;
    }
    if !leaving {
        ui::surface(
            canvas,
            &t,
            Rect::new(0.0, 0.0, screen.w, screen.h),
            0.0,
            Material::Scrim,
        );
    }
    // The title, the detail line, whether there is a text field, the word on
    // the affirmative button, and its tone. One table, so a dialog cannot be
    // drawn with a button that has nothing behind it.
    let (title, detail, field_hint, ok_label, tone) = match &dialog {
        DesktopDialog::NewFolder => (
            "New folder".to_owned(),
            String::new(),
            Some("Folder name"),
            "Create",
            Tone::Accent,
        ),
        DesktopDialog::PowerConfirm { command } => (
            command.question().to_owned(),
            command.consequence().to_owned(),
            None,
            command.title(),
            if *command == PowerCommand::Suspend {
                Tone::Accent
            } else {
                // Ending the session is the only thing in this shell that gets
                // the failure tone, and it comes from a token, not a literal.
                Tone::Bad
            },
        ),
        DesktopDialog::Error(msg) => (
            "Could not finish".to_owned(),
            msg.clone(),
            None,
            "Close",
            Tone::Bad,
        ),
    };

    let w = t.px(360.0).min(screen.w * 0.9);
    let h = if field_hint.is_some() {
        t.px(150.0)
    } else {
        t.px(170.0)
    };
    let panel = Rect::new((screen.w - w) / 2.0, (screen.h - h) / 2.0, w, h);
    // It opens and closes from the middle, like the switcher. The clip is the
    // spring: at zero there is no panel and no hotspot to click.
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(
        panel.x,
        panel.y + (panel.h - shown_h) / 2.0,
        panel.w,
        shown_h,
    ));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);
    // The click-away hotspot goes in FIRST so the buttons, registered later,
    // win the hit test (`hit` picks the last registration). Clicking away
    // cancels: dismissing a confirmation must never be able to perform one.
    shell.hotspots_mut().push(panel, Hot::DialogCancel);

    let s_title = t.style_label();
    font::draw(
        canvas,
        &title,
        panel.x + t.sp(Space::Md),
        panel.y + t.px(30.0),
        &s_title,
    );
    if !detail.is_empty() {
        let s_detail = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        let shown = ui::elide(&detail, &s_detail, panel.w - t.sp(Space::Md) * 2.0);
        font::draw(
            canvas,
            &shown,
            panel.x + t.sp(Space::Md),
            panel.y + t.px(54.0),
            &s_detail,
        );
    }
    if let Some(hint) = field_hint {
        let field = Rect::new(
            panel.x + t.sp(Space::Md),
            panel.y + t.px(56.0),
            panel.w - t.sp(Space::Md) * 2.0,
            t.px(32.0),
        );
        ui::text_field(canvas, &t, field, &shell.desktop_dialog_field, hint, true);
    }
    let ok = Rect::new(
        panel.right() - t.px(112.0),
        panel.bottom() - t.px(40.0),
        t.px(100.0),
        t.px(28.0),
    );
    ui::button(canvas, &t, ok, ok_label, ui::State::Normal, tone);
    shell.hotspots_mut().push(ok, Hot::DialogConfirm);
    let no = Rect::new(ok.x - t.px(108.0), ok.y, t.px(100.0), t.px(28.0));
    ui::button(canvas, &t, no, "Cancel", ui::State::Normal, Tone::Neutral);
    shell.hotspots_mut().push(no, Hot::DialogCancel);
    canvas.set_clip(saved);
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------

/// The system bar.
///
/// There is no bottom bar of status: one bar carries the machine's state, and
/// the dock carries applications. Left to right it reads the way a menu bar is
/// read: the mark, the application that has the keyboard (in the heavier
/// weight, because it is the one thing that changes), then the system menus.
/// The centre is left empty on purpose. On the right, the status icons sit
/// together as one group, the workspace pips beside them, and the clock last.
///
/// The bar has a surface: a flat 78 % fill and a hairline, no blur. Text on a
/// bare wallpaper was legible on the one wallpaper it was tuned for and on no
/// other; a fill makes the contrast the same everywhere for the price of one
/// rectangle.
/// Text on the bar: a soft shadow one pixel down, then the text itself.
fn bar_text(canvas: &mut Canvas, text: &str, x: f32, baseline: f32, style: &Style, shadow: u32) {
    canvas.unrecorded(|c| {
        font::draw(c, text, x, baseline + 1.0, &style.colored(shadow));
    });
    font::draw(canvas, text, x, baseline, style);
}

fn draw_top_bar(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let bar = shell.top_bar();
    let h = bar.h;
    let pad = t.px(chrome::TOP_BAR_PAD);

    // No background. The bar is the wallpaper with the system's words on it:
    // nothing is filled, so nothing needs to be restored or blended, and the
    // desktop runs to the very top of the screen. What keeps the words legible
    // over any wallpaper is a soft shadow under each, in the theme's base
    // colour (see `bar_text`), not a plate behind them.
    let shadow = with_alpha(ui::rgb_of(t.theme.palette.bg_base), 0.55);

    let size = t.text_px(TypeStep::Footnote);
    let name_style = Style::bold(size, t.text());
    let menu_style = Style::new(size, t.text_dim());
    let menu_lit = Style::new(size, t.text());
    let baseline = h / 2.0 + size * 0.36;

    // The application that has the keyboard. A window that is not one of our
    // applications is still named, by its own title; with nothing focused the
    // bar names the desktop, which is what has focus then.
    let title = shell
        .wm
        .focused()
        .and_then(|id| shell.wm.window(id))
        .map_or_else(
            || crate::i18n::text("Desktop").into_owned(),
            |w| {
                crate::shortcuts::App::from_app_id(&w.app_id)
                    .map_or_else(|| w.title.clone(), |a| a.title().to_owned())
            },
        );
    let title = ui::elide(&title, &name_style, t.px(160.0));

    // --- what the right side needs, decided first -------------------------
    // The menus' room is whatever the right side leaves, so the right side has
    // to be measured before the left is drawn: a menu drawn past the clock is a
    // menu drawn over the time.
    let clock = shell.regional.clock(shell.clock_seconds());
    let clock_style = Style::bold(size, t.text());
    let clock_w = font::measure(&clock, &clock_style);
    let date = short_date(&shell.regional.date(shell.clock_seconds()));
    let date_w = font::measure(&date, &menu_style);

    let count = shell.notifications.len();
    let bell_tone = match shell.notifications.peak_priority() {
        Some(Priority::Critical) => Some(Tone::Bad),
        Some(Priority::High) => Some(Tone::Warning),
        Some(_) => Some(Tone::Accent),
        None => None,
    };
    let shield_tone = if shell.defender.running {
        Some(Tone::Accent)
    } else if shell.defender.app.last_had_threats {
        Some(Tone::Bad)
    } else if matches!(
        shell.defender.app.state,
        crate::apps::ScanState::Done { threats: 0, .. }
    ) {
        Some(Tone::Good)
    } else {
        None
    };
    // Left to right. Battery and network appear only when there is something
    // to say: an icon that means "nothing was measured" is noise, and the
    // quick settings panel is one click away for everything.
    let mut icons_row: Vec<(Icon, Hot, Option<Tone>)> = vec![(Icon::Audio, Hot::StatusAudio, None)];
    if shell.facts.network.as_deref() == Some("offline") {
        icons_row.push((Icon::Network, Hot::StatusNetwork, Some(Tone::Warning)));
    }
    if let Some(percent) = shell.facts.battery_percent {
        let tone = if percent <= 10 {
            Some(Tone::Bad)
        } else if percent <= 25 {
            Some(Tone::Warning)
        } else {
            None
        };
        icons_row.push((Icon::Battery, Hot::StatusBattery, tone));
    }
    icons_row.push((
        Icon::Shield,
        Hot::MenuRow(crate::apps::MenuAction::ScanNow),
        shield_tone,
    ));
    icons_row.push((Icon::Power, Hot::StatusPower, None));
    icons_row.push((Icon::Notifications, Hot::StatusNotifications, bell_tone));

    let user_w = t.px(18.0);
    let step = t.px(chrome::TOP_BAR_STEP);
    let icons_w = step * icons_row.len() as f32;
    let right_w = |with_date: bool| -> f32 {
        user_w
            + t.px(14.0)
            + clock_w
            + if with_date { t.px(8.0) + date_w } else { 0.0 }
            + t.px(14.0)
            + icons_w
    };

    // --- left: the mark, the application, the menus -----------------------
    let mark = t.px(20.0);
    let mut x = pad;
    let logo = Rect::new(x, (h - mark) / 2.0, mark, mark);
    // The mark: the pen-drawn ring and L the boot wordmark begins with, tinted
    // with the text colour so it is right on either theme.
    appicons::draw_tinted(
        canvas,
        appicons::Slot::Mark,
        logo,
        if shell.is_highlighted(Hot::Logo) {
            t.accent()
        } else {
            t.text()
        },
    );
    shell.hotspots_mut().push(
        Rect::new(x - t.px(6.0), 0.0, mark + t.px(12.0), h),
        Hot::Logo,
    );
    x += mark + t.px(12.0);

    let name_w = font::measure(&title, &name_style);
    bar_text(canvas, &title, x, baseline, &name_style, shadow);
    x += name_w + t.px(chrome::TOP_BAR_GAP);

    let menus = [
        ("Apps", Hot::MenuApps),
        ("Workspace", Hot::MenuWorkspace),
        ("Environment", Hot::MenuEnvironment),
    ];
    let menus_w: f32 = menus
        .iter()
        .map(|(label, _)| font::measure(label, &menu_style) + t.px(chrome::TOP_BAR_GAP))
        .sum();
    // The date is the first thing to go: it is also in the lock screen and the
    // calendar, and nothing else on this bar can be recovered as cheaply.
    let wide_enough = bar.w / t.scale >= chrome::TOP_BAR_DATE_MIN_W;
    let show_date = wide_enough && x + menus_w + right_w(true) + pad + t.px(12.0) <= bar.w;
    let menus_end = (bar.w - pad - right_w(show_date) - t.px(12.0)).max(x);

    let mut dropped = 0usize;
    for (label, hot) in menus {
        let w = font::measure(label, &menu_style);
        if x + w > menus_end {
            dropped = dropped.saturating_add(1);
            continue;
        }
        let r = Rect::new(x - t.px(7.0), t.px(4.0), w + t.px(14.0), h - t.px(8.0));
        let lit = shell.is_highlighted(hot);
        if lit {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(t.text(), 0.12));
        }
        bar_text(
            canvas,
            label,
            x,
            baseline,
            if lit { &menu_lit } else { &menu_style },
            shadow,
        );
        shell.hotspots_mut().push(r, hot);
        x += w + t.px(chrome::TOP_BAR_GAP);
    }
    // Whatever did not fit is reachable through the launcher, which is what
    // these menus open anyway. A chevron is drawn rather than a menu silently
    // missing, so the bar never claims fewer choices than it has.
    if dropped > 0 && x + t.px(24.0) <= menus_end {
        let r = Rect::new(x - t.px(7.0), t.px(4.0), t.px(24.0), h - t.px(8.0));
        if shell.is_highlighted(Hot::MenuApps) {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(t.text(), 0.12));
        }
        bar_text(canvas, "\u{203A}", x, baseline, &menu_style, shadow);
        shell.hotspots_mut().push(r, Hot::MenuApps);
    }
    let left_end = x;

    // --- right: laid out from the edge inwards ----------------------------
    let mut rx = bar.w - pad;
    rx -= user_w;
    let user_r = Rect::new(rx, (h - user_w) / 2.0, user_w, user_w);
    canvas.fill_circle(
        user_r.center().0,
        user_r.center().1,
        user_w / 2.0,
        with_alpha(t.text(), 0.14),
    );
    icons::draw(
        canvas,
        Icon::User,
        user_r.inflate(-user_w * 0.2),
        t.text(),
        1.0,
    );
    shell
        .hotspots_mut()
        .push(user_r.inflate(t.px(5.0)), Hot::StatusUser);

    rx -= t.px(14.0) + clock_w;
    bar_text(canvas, &clock, rx, baseline, &clock_style, shadow);
    shell.hotspots_mut().push(
        Rect::new(rx - t.px(6.0), 0.0, clock_w + t.px(12.0), h),
        Hot::StatusClock,
    );
    if show_date {
        rx -= t.px(8.0) + date_w;
        bar_text(canvas, &date, rx, baseline, &menu_style, shadow);
    }

    rx -= t.px(14.0);
    let icon = t.px(chrome::TOP_BAR_ICON);
    // The status icons share one capsule: they are one control, the place the
    // quick settings open from, and a capsule says so where a loose row of
    // glyphs would not.
    let capsule = Rect::new(
        rx - icons_w - t.px(2.0),
        (h - t.px(22.0)) / 2.0,
        icons_w - (step - icon) + t.px(12.0),
        t.px(22.0),
    );
    canvas.fill_round_rect(capsule, capsule.h / 2.0, with_alpha(t.text(), 0.10));
    let mut bell: Option<Rect> = None;
    for (glyph, hot, tone) in icons_row.iter().rev() {
        rx -= step;
        let r = Rect::new(rx, (h - icon) / 2.0, icon, icon);
        if shell.is_highlighted(*hot) {
            canvas.fill_round_rect(
                r.inflate(t.px(5.0)),
                t.radius(Radius::Sm),
                with_alpha(t.text(), 0.12),
            );
        }
        let color = tone.map_or_else(
            || {
                if shell.is_highlighted(*hot) {
                    t.text()
                } else {
                    t.text_dim()
                }
            },
            |x| t.tone(x),
        );
        icons::draw(canvas, *glyph, r, color, 1.0);
        shell.hotspots_mut().push(r.inflate(t.px(5.0)), *hot);
        if *glyph == Icon::Notifications {
            bell = Some(r);
        }
    }
    if let (true, Some(bell)) = (count > 0, bell) {
        // A count badge, not just a coloured bell: the number is the
        // information and colour alone cannot carry it. It grows leftwards
        // from the bell's right edge, because nothing is drawn to the right of
        // the bell but the gap before the clock.
        let label = if count > 9 {
            "9+".to_owned()
        } else {
            count.to_string()
        };
        let s = Style::bold(t.text_px(TypeStep::Caption) * 0.85, rgb(0xFFFFFF));
        let w = font::measure(&label, &s);
        let bw = w + t.px(8.0);
        let br = Rect::new(
            bell.right() - bw + t.px(3.0),
            bell.y - t.px(3.0),
            bw,
            t.px(12.0),
        );
        canvas.fill_round_rect(br, br.h / 2.0, t.tone(bell_tone.unwrap_or(Tone::Accent)));
        font::draw(canvas, &label, br.x + t.px(4.0), br.y + br.h * 0.76, &s);
    }

    // --- workspace pips, beside the status group --------------------------
    // Drawn only if the room is there after everything above has been placed:
    // they are the middle of the three priorities, and a squeezed bar loses
    // them before it loses a menu or a clock.
    let ws_count = shell.wm.workspaces().len();
    let active_ws = shell.wm.active_workspace();
    let pip = t.px(chrome::TOP_BAR_PIP);
    let gap = t.px(7.0);
    let pips_w = (ws_count as f32) * pip + (ws_count.saturating_sub(1) as f32) * gap;
    let pips_x = rx - t.px(18.0) - pips_w;
    if ws_count > 1 && pips_x >= left_end + t.px(24.0) {
        let mut px = pips_x;
        for i in 0..ws_count {
            let r = Rect::new(px, (h - pip) / 2.0, pip, pip);
            let on = i == active_ws;
            canvas.fill_round_rect(
                r,
                pip / 2.0,
                if on {
                    t.accent()
                } else {
                    with_alpha(t.text(), 0.28)
                },
            );
            shell
                .hotspots_mut()
                .push(r.inflate(t.px(5.0)), Hot::Workspace(i as u8));
            px += pip + gap;
        }
    }
}

/// A date without its year: the bar has room for the day, and the year is
/// never what someone glances at a menu bar to learn.
fn short_date(full: &str) -> String {
    full.split_whitespace()
        .filter(|word| {
            let digits = word.trim_end_matches(',');
            !(digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|word| word.trim_end_matches(','))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------

fn draw_windows(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let chrome = *shell.wm.chrome();
    let windows = shell.visible_windows();

    for (id, frame, title, focused, state) in windows {
        // Everything this window draws carries its id, so the layout audit can
        // tell two stacked windows' text (expected) from two labels colliding
        // inside one window (a defect). Zero is reserved for the chrome.
        let saved_surface = canvas.set_surface(id.saturating_add(1));
        let radius = t.px(chrome.radius);
        // An unfocused window gets a softer shadow and a dimmer frame, so the
        // focused one is obvious without relying on the accent colour alone.
        // The two weights are the endpoints of a spring: focus glides up,
        // blur glides down, instead of snapping between the pair.
        let elevation = shell.anims_mut().focus.elevation(id, focused);
        let mix =
            |focused_v: f32, unfocused_v: f32| unfocused_v + (focused_v - unfocused_v) * elevation;
        canvas.soft_shadow(
            frame,
            radius,
            t.px(mix(30.0, 18.0)),
            t.px(mix(12.0, 7.0)),
            mix(0.60, 0.40),
        );
        ui::surface(canvas, &t, frame, radius, Material::Glass);

        // Title bar.
        let title_r = Rect::new(frame.x, frame.y, frame.w, t.px(chrome.title_h));
        canvas.fill_round_rect(
            Rect::new(frame.x + 1.0, title_r.bottom(), frame.w - 2.0, 1.0),
            0.0,
            with_alpha(rgb(0xFFFFFF), 0.08),
        );

        for (n, icon, tone) in [
            (0u32, Icon::Close, Tone::Bad),
            (1, Icon::Minimize, Tone::Warning),
            (2, Icon::Maximize, Tone::Good),
        ] {
            let b = chrome.button_rect(&to_wm(frame), n);
            let br = Rect::new(b.x, b.y, b.w, b.h);
            canvas.fill_circle(
                br.x + br.w / 2.0,
                br.y + br.h / 2.0,
                br.w / 2.0,
                if focused {
                    t.tone(tone)
                } else {
                    with_alpha(rgb(0xFFFFFF), 0.20)
                },
            );
            // The glyph appears on hover, the way every desktop that uses
            // coloured dots does it: colour at rest, symbol when aimed at.
            let p = shell.pointer();
            if br.inflate(t.px(4.0)).contains(p.x, p.y) {
                icons::draw(canvas, icon, br.inflate(-br.w * 0.22), rgb(0x101010), 1.4);
            }
        }

        let ts = Style::new(
            t.text_px(TypeStep::Footnote),
            if focused { t.text() } else { t.text_dim() },
        );
        let max_w = frame.w - t.px(150.0);
        let shown = ui::elide(&title, &ts, max_w.max(t.px(40.0)));
        // Centred in the bar, as macOS sets it, but never under the traffic lights.
        let shown_w = font::measure(&shown, &ts);
        font::draw(
            canvas,
            &shown,
            frame.x + ((frame.w - shown_w) / 2.0).max(t.px(74.0)),
            title_r.y + title_r.h / 2.0 + ts.size * 0.36,
            &ts,
        );
        if state != WindowState::Normal {
            let label = state_label(state);
            let s = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            let w = font::measure(label, &s);
            font::draw(
                canvas,
                label,
                frame.right() - t.px(14.0) - w,
                title_r.y + title_r.h / 2.0 + s.size * 0.36,
                &s,
            );
        }

        let body = Rect::new(
            frame.x,
            title_r.bottom(),
            frame.w,
            (frame.h - title_r.h).max(0.0),
        );
        // The body claims its own clicks BEFORE any control registers: bare
        // window surface must never fall through `hot.hit` to whatever sits
        // underneath — the bug where clicking a window opened the desktop
        // icon hiding behind it. Controls register after, so they still win.
        shell.hotspots_mut().push(body, Hot::Window(id));
        // Clipped to the window. Without this a panel with more rows than fit
        // draws them down the wallpaper below its own frame, which is what the
        // first interactive build did and what a screenshot makes obvious.
        let saved = canvas.push_clip(body);
        draw_window_body(canvas, shell, id, body);
        canvas.set_clip(saved);
        canvas.set_surface(saved_surface);
    }
}

const fn state_label(state: WindowState) -> &'static str {
    match state {
        WindowState::Maximized => "Maximized",
        WindowState::Snapped(_) => "Snapped",
        WindowState::Fullscreen => "Fullscreen",
        _ => "",
    }
}

fn to_wm(r: Rect) -> litric_wm::Rect {
    litric_wm::Rect::new(r.x, r.y, r.w, r.h)
}

fn draw_window_body(canvas: &mut Canvas, shell: &mut Shell, id: u32, body: Rect) {
    // A foreign application's window is somebody else's pixels. It is painted
    // (and its frame callback answered) before any built-in view is considered,
    // because the window manager cannot tell the two apart by design.
    if let Some(host) = shell.clients.as_mut() {
        if host.draw_window(canvas, id, body) {
            return;
        }
    }
    let app_id = shell
        .wm
        .window(id)
        .map(|w| w.app_id.clone())
        .unwrap_or_default();
    match app_id.as_str() {
        "com.litric.activity" => draw_activity(canvas, shell, body),
        "com.litric.environments" => draw_environments(canvas, shell, body),
        "com.litric.settings" => draw_settings(canvas, shell, body),
        "com.litric.files" => draw_files(canvas, shell, body),
        "com.litric.browser" => draw_browser(canvas, shell, body),
        "com.litric.terminal" => draw_terminal(canvas, shell, body),
        "com.litric.trash" => draw_trash(canvas, shell, body),
        "com.litric.defender" => draw_defender(canvas, shell, body),
        other => draw_not_implemented(canvas, shell, body, other),
    }
}

/// Refresh the Files listing when the model's path has moved past what was
/// last read. The call goes through [`crate::fsbridge`], which keeps the
/// no-symlink-follow confinement on Unix and keeps the shell compiling on
/// every host the tests run on.
fn refresh_files(shell: &mut Shell) {
    if shell.files.listed.as_deref() == Some(shell.files.path.as_str()) {
        return;
    }
    let zone = shell.regional.zone.clone();
    // The same bridge the rest of the shell uses: whatever the daemon policy
    // decided at startup applies to the listing too, and a second `open` here
    // is how a window would come to read the disk directly while every other
    // operation went through the daemon.
    let bridge = match shell.try_files_bridge(zone) {
        Ok(b) => b,
        Err(e) => {
            shell.files.apply_error(e.to_string());
            shell.files.listed = Some(shell.files.path.clone());
            return;
        }
    };
    match bridge.list_dir(&shell.files.path) {
        Ok(entries) => {
            let rows = entries
                .into_iter()
                .map(|e| crate::apps::FileRow {
                    name: e.name,
                    kind: e.kind,
                    size: e.size,
                    mtime: e.mtime,
                })
                .collect();
            shell.files.apply_entries(rows);
        }
        Err(e) => shell.files.apply_error(e.to_string()),
    }
    shell.files.listed = Some(shell.files.path.clone());
}

/// The Files window: places pane, breadcrumb bar, command row, table,
/// details pane.
/// The Files table's columns, minus the ones a narrow window cannot fit.
///
/// Explorer's rule and the one the layout audit needs: a heading is drawn only
/// if it fits inside its own column, so one heading can never run through the
/// next. At 480 pixels wide the date column is 27 pixels and its heading is 65,
/// which is what the audit found. Whatever width a dropped column had is given
/// to the name column, which is the one a person is reading.
fn files_columns(t: &ui::Tokens, table_w: f32, style: &Style) -> Vec<(f32, &'static str, SortKey)> {
    let base = [
        (0.52f32, "Name", SortKey::Name),
        (0.16, "Size", SortKey::Size),
        (0.32, "Modified", SortKey::Modified),
    ];
    let fitted =
        |frac: f32, label: &str| table_w * frac >= font::measure(label, style) + t.px(12.0);
    let dropped: Vec<f32> = base
        .iter()
        .filter(|(frac, label, _)| !fitted(*frac, label))
        .map(|(frac, _, _)| *frac)
        .collect();
    // The name column never goes: with everything else dropped the table is
    // still a list of names, which is what a file manager is for.
    let spare = if fitted(base[0].0, base[0].1) {
        dropped.iter().sum::<f32>()
    } else {
        0.0
    };
    base.into_iter()
        .filter(|(frac, label, _)| fitted(*frac, label))
        .map(|(frac, label, key)| {
            let frac = if key == SortKey::Name {
                frac + spare
            } else {
                frac
            };
            (frac.min(1.0), label, key)
        })
        .collect()
}

/// How many leading breadcrumbs to drop so the rest fit in `available`, and
/// whether the path overflowed at all.
///
/// The last crumb is the folder the user is standing in, so it is never
/// dropped: when even that one cannot fit, `skip` stays 0 and the caller
/// elides its label instead. Dropping the head rather than the tail is what
/// keeps the answer to "where am I?" on screen.
fn crumb_window(available: f32, widths: &[f32], sep: f32, dots: f32) -> (usize, bool) {
    if widths.is_empty() {
        return (0, false);
    }
    let mut total = 0.0;
    for (i, w) in widths.iter().enumerate() {
        total += *w;
        if i > 0 {
            total += sep;
        }
    }
    if total <= available {
        return (0, false);
    }
    let reserve = dots + sep;
    let mut skip = 0;
    while skip + 1 < widths.len() && total + reserve > available {
        total -= widths[skip] + sep;
        skip += 1;
    }
    (skip, true)
}

fn draw_files(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    refresh_files(shell);
    let t = shell.tokens.clone();
    let app = shell.files.clone();
    // The ordered rows for this frame, built once. The readout, the table and
    // each row's selection state all read this list; before, each of the 24
    // drawn rows called `visible()` again, which re-filtered and re-sorted the
    // whole listing and allocated a lowercased copy of every compared name.
    let vis = app.visible();

    // Layout: places | main column (crumbs, commands, table) | details.
    //
    // One width policy for every sidebar in the shell (see `ui::sidebar`):
    // previously this pane was a fixed 132 px while Settings took 36% of its
    // window, which is the delta F.10 records.
    let places_w = ui::sidebar::width(body.w);
    let details_w = if app.details { t.px(180.0) } else { 0.0 };
    let table_x = body.x + places_w;
    let table_w = body.w - places_w - details_w;

    // --- navigation pane (Quick access) ---
    //
    // The same component Settings uses: one width policy, one row height (the
    // shell's 30 px list row), one selection treatment (fill plus the accent
    // marker `list_item` draws), one scroll model. The pane used to be 132 px
    // of 20 px rows with a colour-only highlight and no scroll at all.
    let places = Rect::new(body.x, body.y, places_w, body.h);
    ui::surface(canvas, &t, places, 0.0, ui::Material::Elevated);
    let items: Vec<ui::sidebar::Item<'_>> = app
        .places
        .iter()
        .enumerate()
        .map(|(i, (place, label))| ui::sidebar::Item {
            label,
            icon: Some(Icon::Files),
            selected: app.place_index == Some(i) || place == &app.path,
            highlighted: false,
        })
        .collect();
    // Behind the rows: the wheel over the pane's empty space must scroll it,
    // and a row under the pointer wins the hit test because it is pushed after
    // this. The heading is fixed, so only the rows area needs the hotspot.
    shell.hotspots_mut().push(
        ui::sidebar::rows_area(&t, places, true),
        Hot::Files(FilesHot::PlacesScroll),
    );
    let rows_area = ui::sidebar::rows_area(&t, places, true);
    let scroll = ui::sidebar::clamp(&t, rows_area, items.len(), app.places_scroll);
    shell.files.places_scroll = scroll;
    for (i, row) in ui::sidebar::draw(canvas, &t, places, Some("Quick access"), &items, scroll) {
        shell
            .hotspots_mut()
            .push(row, Hot::Files(FilesHot::Place(i)));
    }

    // --- breadcrumb bar + history buttons ---
    let bar = Rect::new(table_x, body.y, table_w, t.px(30.0));
    ui::surface(canvas, &t, bar, 0.0, ui::Material::Flat);
    let mut bx = bar.x + t.px(6.0);
    // A button with nowhere to go is drawn disabled and registered as nothing:
    // it looked pressable with an empty history and answered a click by doing
    // nothing, which is worse than either state on its own.
    let nav = [
        ("‹", FilesHot::Back, app.can_go_back()),
        ("›", FilesHot::Forward, app.can_go_forward()),
        ("↑", FilesHot::Up, app.can_go_up()),
    ];
    for (label, hot, enabled) in nav {
        let r = Rect::new(bx, bar.y + t.px(4.0), t.px(20.0), t.px(22.0));
        let state = if enabled {
            ui::State::Normal
        } else {
            ui::State::Disabled
        };
        ui::button(canvas, &t, r, label, state, Tone::Neutral);
        if enabled {
            shell.hotspots_mut().push(r, Hot::Files(hot));
        }
        bx += t.px(24.0);
    }
    // The root is a crumb like any other. It used to be painted as text with no
    // hotspot, so the one place every deep path can jump to was the one place
    // that could not be clicked.
    let s_crumb = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let s_sep = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let root_w = font::measure("/", &s_crumb).max(t.px(8.0)) + t.px(8.0);
    let root = Rect::new(bx + t.px(2.0), bar.y + t.px(4.0), root_w, t.px(22.0));
    font::draw(
        canvas,
        "/",
        root.x + t.px(4.0),
        root.y + t.px(14.0),
        &s_crumb,
    );
    shell.hotspots_mut().push(root, Hot::Files(FilesHot::Root));
    let mut cx = root.right() + t.px(2.0);

    let crumbs = app.breadcrumbs();
    let widths: Vec<f32> = crumbs
        .iter()
        .map(|c| font::measure(c, &s_crumb) + t.px(8.0))
        .collect();
    let dots_w = font::measure("...", &s_crumb) + t.px(8.0);
    let available = (bar.right() - t.px(8.0) - cx).max(0.0);
    let (skip, overflow) = crumb_window(available, &widths, t.px(8.0), dots_w);
    if overflow && skip > 0 {
        let r = Rect::new(cx, bar.y + t.px(4.0), dots_w, t.px(22.0));
        font::draw(canvas, "...", cx + t.px(4.0), r.y + t.px(14.0), &s_sep);
        cx += dots_w + t.px(2.0);
    }
    for (i, c) in crumbs.iter().enumerate().skip(skip) {
        let last = i + 1 == crumbs.len();
        let room = (bar.right() - t.px(6.0) - cx - t.px(8.0)).max(t.px(10.0));
        let shown = if overflow {
            ui::elide(c, &s_crumb, room)
        } else {
            (*c).clone()
        };
        let w = font::measure(&shown, &s_crumb) + t.px(8.0);
        let r = Rect::new(cx, bar.y + t.px(4.0), w, t.px(22.0));
        font::draw(canvas, &shown, cx + t.px(4.0), r.y + t.px(14.0), &s_crumb);
        shell.hotspots_mut().push(r, Hot::Files(FilesHot::Crumb(i)));
        cx += w + t.px(2.0);
        if !last {
            font::draw(canvas, "/", cx, r.y + t.px(14.0), &s_sep);
            cx += t.px(8.0);
        }
    }

    // --- command row: New / Paste / Compress / Extract, filter readout,
    // details toggle. The buttons drive the same model the context menu and
    // the shortcuts do, so all three paths stay one behaviour.
    let cmd = Rect::new(table_x, bar.bottom(), table_w, t.px(26.0));
    let s_cmd = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let filter_note = if app.filter.is_empty() {
        format!(
            "{} items · {} selected",
            vis.len(),
            app.selected_paths_in(&vis).len()
        )
    } else {
        format!(
            "filter: {} ({} items · {} selected)",
            app.filter,
            vis.len(),
            app.selected_paths_in(&vis).len()
        )
    };
    // Where the Details toggle will be, decided before the buttons are drawn:
    // the closure below has to know its right-hand limit, and a closure captures
    // only names declared before it.
    let det_label = if app.details {
        "Details ✕"
    } else {
        "Details"
    };
    let dw = font::measure(det_label, &s_cmd) + t.px(12.0);
    let det_r = Rect::new(
        cmd.right() - dw - t.px(6.0),
        cmd.y + t.px(2.0),
        dw,
        t.px(22.0),
    );
    // The readout is drawn after the buttons, in the gap before the Details
    // toggle, rather than at a fixed x. At x=150 the buttons grew into it and
    // "0 items" was drawn over "Compress": 32x16 pixels of one string on top of
    // another, found by the layout audit.
    let btn = |canvas: &mut Canvas,
               shell: &mut Shell,
               x: f32,
               label: &str,
               hot: Hot,
               tone: Tone|
     -> f32 {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text());
        let w = font::measure(label, &s) + t.px(12.0);
        let r = Rect::new(x, cmd.y + t.px(2.0), w, t.px(22.0));
        // A button that would reach the Details toggle is not drawn at all, and
        // every button after it follows, because none of them advance `x`. In a
        // 420-pixel window that is what kept "Paste" and "Compress" from being
        // drawn over "Details ✕" (33x18 pixels of overlap). The same actions
        // stay reachable from the context menu and the keyboard shortcuts, which
        // drive one model with these buttons.
        if r.right() > det_r.x - t.px(6.0) {
            return 0.0;
        }
        ui::button(canvas, &t, r, label, ui::State::Normal, tone);
        shell.hotspots_mut().push(r, hot);
        w + t.px(4.0)
    };
    let mut bx = cmd.x + t.px(4.0);
    bx += btn(
        canvas,
        shell,
        bx,
        "New folder",
        Hot::MenuRow(crate::apps::MenuAction::NewFolder),
        Tone::Neutral,
    );
    bx += btn(
        canvas,
        shell,
        bx,
        "Paste",
        Hot::Files(FilesHot::Paste),
        Tone::Neutral,
    );
    bx += btn(
        canvas,
        shell,
        bx,
        "Compress",
        Hot::Files(FilesHot::Compress),
        Tone::Neutral,
    );
    let _ = bx;
    bx += btn(
        canvas,
        shell,
        bx,
        "Extract",
        Hot::Files(FilesHot::Extract),
        Tone::Neutral,
    );
    let _ = bx + 1.0;
    ui::button(
        canvas,
        &t,
        det_r,
        det_label,
        ui::State::Normal,
        Tone::Neutral,
    );
    shell
        .hotspots_mut()
        .push(det_r, Hot::Files(FilesHot::Details));

    let readout_x = bx + t.px(10.0);
    let readout_room = (det_r.x - readout_x - t.px(10.0)).max(0.0);
    if readout_room > t.px(30.0) {
        let note = ui::elide(&filter_note, &s_cmd, readout_room);
        font::draw(canvas, &note, readout_x, cmd.y + t.px(15.0), &s_cmd);
    }

    // --- table: no grid. Spacing and one hairline under the header carry the
    // structure; vertical rules read as a form, not data.
    let head_h = t.px(22.0);
    let row_h = t.px(24.0);
    let table = Rect::new(table_x, cmd.bottom(), table_w, body.bottom() - cmd.bottom());
    // (columns are chosen below, from the width the table actually got)
    let s_head = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let cols = files_columns(&t, table.w, &s_head);
    let mut hx = table.x;
    let mut hx_end = 0.0;
    for (frac, label, key) in &cols {
        let cw = table.w * *frac;
        let r = Rect::new(hx, table.y, cw, head_h);
        let active = app.sort.0 == *key;
        let arrow = if active {
            if app.sort.1 {
                " ↑"
            } else {
                " ↓"
            }
        } else {
            ""
        };
        let s = s_head.colored(if active { t.text() } else { t.text_dim() });
        // Elided against its own column, the same way a row cell is: a heading
        // that overflows its width is a heading drawn through the next one, and
        // at 480 pixels wide the date column's heading does exactly that.
        font::draw(
            canvas,
            &ui::elide(&format!("{label}{arrow}"), &s, cw - t.px(12.0)),
            hx + t.px(if *key == SortKey::Name { 26.0 } else { 6.0 }),
            r.y + t.px(13.0),
            &s,
        );
        shell
            .hotspots_mut()
            .push(r, Hot::Files(FilesHot::Header(*key)));
        hx += cw;
        hx_end = hx;
    }
    canvas.stroke_line(
        table.x,
        table.y + head_h,
        hx_end,
        table.y + head_h,
        1.0,
        t.border(),
    );

    if let Some(err) = &app.error {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.tone(Tone::Bad));
        font::draw(
            canvas,
            err,
            table.x + t.px(8.0),
            table.y + head_h + t.px(18.0),
            &s,
        );
        return;
    }

    for (i, row) in vis.iter().enumerate().take(24) {
        let y = table.y + head_h + (i as f32) * row_h;
        let selected = app.row_is_selected_in(&vis, i);
        let row_rect = Rect::new(table.x, y, table.w, row_h);
        if selected {
            canvas.fill_round_rect(row_rect, t.radius(Radius::Sm), with_alpha(t.text(), 0.08));
        }
        let s = Style::new(
            t.text_px(TypeStep::Footnote),
            if selected { t.text() } else { t.text_dim() },
        );
        let size_text = match row.kind {
            FileKind::Dir => "—".to_owned(),
            _ => format_size(row.size),
        };
        let date = litric_locale::civil::from_unix(row.mtime as i64);
        let date_text = shell.regional.locale.format_date(date);
        // One cell per column the header kept, in the same order: a row drawn
        // from its own column list would put the date under the sort heading.
        let cells = [row.name.clone(), size_text, date_text];
        let mut cx2 = table.x;
        for (i, (frac, _, _)) in cols.iter().enumerate() {
            let cw = table.w * *frac;
            let text = cells.get(i).map_or("", String::as_str);
            let inset = if i == 0 { t.px(28.0) } else { t.px(6.0) };
            font::draw(
                canvas,
                &ui::elide(text, &s, cw - inset - t.px(6.0)),
                cx2 + inset,
                y + t.px(15.0),
                &s,
            );
            cx2 += cw;
        }
        shell
            .hotspots_mut()
            .push(row_rect, Hot::Files(FilesHot::Row(i)));
        let check = Rect::new(
            table.x + t.px(8.0),
            y + (row_h - t.px(12.0)) / 2.0,
            t.px(12.0),
            t.px(12.0),
        );
        if selected {
            canvas.fill_round_rect(check, t.radius(Radius::Sm), t.accent());
        }
        canvas.stroke_round_rect(
            check,
            t.radius(Radius::Sm),
            1.0,
            if selected { t.accent() } else { t.text_dim() },
        );
        if selected {
            let (cx, cy) = check.center();
            canvas.stroke_line(
                cx - t.px(3.0),
                cy,
                cx - t.px(1.0),
                cy + t.px(2.0),
                1.5,
                t.text(),
            );
            canvas.stroke_line(
                cx - t.px(1.0),
                cy + t.px(2.0),
                cx + t.px(3.0),
                cy - t.px(3.0),
                1.5,
                t.text(),
            );
        }
        let toggle = Hot::Files(FilesHot::ToggleRow(i));
        if shell.is_highlighted(toggle) {
            ui::focus_ring(canvas, &t, check, t.radius(Radius::Sm));
        }
        shell.hotspots_mut().push(check.inflate(t.px(3.0)), toggle);
    }
    if vis.is_empty() && app.error.is_none() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(
            canvas,
            "Empty folder",
            table.x + t.px(8.0),
            table.y + head_h + t.px(18.0),
            &s,
        );
    }

    // --- details pane ---
    if app.details {
        let dr = Rect::new(table.right(), body.y, details_w, body.h);
        ui::surface(canvas, &t, dr, 0.0, ui::Material::Elevated);
        let s_head = t.style_label();
        font::draw(
            canvas,
            "Details",
            dr.x + t.sp(Space::Sm),
            dr.y + t.px(20.0),
            &s_head,
        );
        let s_meta = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let Some(sel) = app.selection() else {
            font::draw(
                canvas,
                "Nothing selected",
                dr.x + t.sp(Space::Sm),
                dr.y + t.px(40.0),
                &s_meta,
            );
            // No row to describe is the pane's own empty state, not the end of
            // the window: returning here skipped the modal dialog and the text
            // viewer below, so New folder with the details pane open and no
            // selection drew no dialog and no Create button to press.
            draw_files_dialog(canvas, shell, body);
            draw_viewer(canvas, shell, body);
            return;
        };
        let lines = [
            sel.name.clone(),
            match sel.kind {
                FileKind::Dir => "Folder".to_owned(),
                FileKind::File => "File".to_owned(),
                FileKind::Link => "Shortcut".to_owned(),
                FileKind::Other => "Other".to_owned(),
            },
            match sel.kind {
                FileKind::Dir => String::new(),
                _ => format_size(sel.size),
            },
        ];
        let mut ly = dr.y + t.px(40.0);
        for l in lines {
            if !l.is_empty() {
                font::draw(
                    canvas,
                    &ui::elide(&l, &s_meta, details_w - t.px(20.0)),
                    dr.x + t.sp(Space::Sm),
                    ly,
                    &s_meta,
                );
            }
            ly += t.px(16.0);
        }
    }

    // --- modal dialog over the window ---
    draw_files_dialog(canvas, shell, body);

    // --- text viewer overlay ---
    draw_viewer(canvas, shell, body);
}

/// The surface id a modal dialog's text carries.
///
/// A dialog is a scrim and a panel over the window it belongs to, so the list
/// behind it is covered rather than read. Tagging the dialog's own text as a
/// surface -- the same thing a window does over the desktop -- is what lets the
/// layout audit tell "a dialog over a table" from "two labels in one row",
/// instead of reporting every dialog as a collision.
const DIALOG_SURFACE: u32 = u32::MAX - 1;

/// The Files window's modal dialog: create, rename, confirm, error.
fn draw_files_dialog(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let Some(dialog) = shell.files.dialog.clone() else {
        return;
    };
    let saved_surface = canvas.set_surface(DIALOG_SURFACE);
    draw_files_dialog_body(canvas, shell, body, &dialog);
    canvas.set_surface(saved_surface);
}

/// The dialog's contents, drawn on [`DIALOG_SURFACE`].
fn draw_files_dialog_body(
    canvas: &mut Canvas,
    shell: &mut Shell,
    body: Rect,
    dialog: &crate::apps::DialogKind,
) {
    let t = shell.tokens.clone();
    ui::surface(canvas, &t, body, 0.0, Material::Scrim);
    let (title, detail, with_field, tone) = match &dialog {
        crate::apps::DialogKind::NewFolder => {
            ("New folder".to_owned(), String::new(), true, Tone::Accent)
        }
        crate::apps::DialogKind::NewFile => (
            "New text file".to_owned(),
            String::new(),
            true,
            Tone::Accent,
        ),
        crate::apps::DialogKind::Rename { current, .. } => (
            "Rename".to_owned(),
            format!("from {current}"),
            true,
            Tone::Accent,
        ),
        crate::apps::DialogKind::Confirm { title, detail, .. } => {
            (title.clone(), detail.clone(), false, Tone::Warning)
        }
        crate::apps::DialogKind::Error(msg) => {
            ("Could not finish".to_owned(), msg.clone(), false, Tone::Bad)
        }
    };
    let w = t.px(360.0);
    let h = if with_field { t.px(160.0) } else { t.px(150.0) };
    let panel = Rect::new(
        body.x + (body.w - w) / 2.0,
        body.y + (body.h - h) / 2.0,
        w.min(body.w - t.px(16.0)),
        h.min(body.h - t.px(16.0)),
    );
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);
    let s_title = t.style_bold(TypeStep::Title3);
    font::draw(
        canvas,
        &title,
        panel.x + t.sp(Space::Md),
        panel.y + t.px(28.0),
        &s_title,
    );
    let s_detail = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    if !detail.is_empty() {
        let shown = ui::elide(&detail, &s_detail, panel.w - t.sp(Space::Md) * 2.0);
        font::draw(
            canvas,
            &shown,
            panel.x + t.sp(Space::Md),
            panel.y + t.px(50.0),
            &s_detail,
        );
    }
    let ok_label = match &dialog {
        crate::apps::DialogKind::Confirm { .. } => "Confirm",
        crate::apps::DialogKind::Error(_) => "OK",
        _ => "Create",
    };
    let ok = Rect::new(
        panel.right() - t.px(88.0),
        panel.bottom() - t.px(38.0),
        t.px(76.0),
        t.px(26.0),
    );
    ui::button(canvas, &t, ok, ok_label, ui::State::Normal, tone);
    shell.hotspots_mut().push(ok, Hot::DialogConfirm);
    if !matches!(dialog, crate::apps::DialogKind::Error(_)) {
        let no = Rect::new(ok.x - t.px(84.0), ok.y, t.px(76.0), t.px(26.0));
        ui::button(canvas, &t, no, "Cancel", ui::State::Normal, Tone::Neutral);
        shell.hotspots_mut().push(no, Hot::DialogCancel);
    }
    if with_field {
        let field = Rect::new(
            panel.x + t.sp(Space::Md),
            panel.bottom() - t.px(74.0),
            panel.w - t.sp(Space::Md) * 2.0,
            t.px(30.0),
        );
        let placeholder = match &dialog {
            crate::apps::DialogKind::Rename { current, .. } => current.clone(),
            _ => "Name".to_owned(),
        };
        ui::text_field(
            canvas,
            &t,
            field,
            &shell.files.dialog_field,
            &placeholder,
            shell.files.dialog_focus,
        );
        shell
            .hotspots_mut()
            .push(field, Hot::Files(FilesHot::Filter));
    }
}

/// The text viewer: a pane over the Files window with page scrolling.
fn draw_viewer(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let panel = body.inflate(-t.sp(Space::Lg));

    let text_area = Rect::new(
        panel.x + t.sp(Space::Sm),
        panel.y + t.px(32.0),
        panel.w - t.sp(Space::Sm) * 2.0,
        panel.h - t.px(70.0),
    );
    let line_h = t.px(16.0);
    let rows = ((text_area.h / line_h) as usize).max(1);

    // Take what the frame needs and let the borrow end here. Cloning the viewer
    // copied the whole file -- text and line index -- into every frame, and the
    // old `window()` then re-split the entire text into owned lines just to
    // draw the twenty-odd that fit.
    let Some((path, scroll, lines, total)) = shell.files.viewer.as_ref().map(|v| {
        let (lines, total) = v.window(rows);
        let lines: Vec<String> = lines.into_iter().map(str::to_owned).collect();
        (v.path.clone(), v.scroll, lines, total)
    }) else {
        // Nothing to show: draw nothing. The scrim and panel used to be
        // painted before this check, which left a dark rectangle over the
        // Files window whenever the viewer was closed.
        return;
    };

    ui::surface(canvas, &t, body, 0.0, Material::Scrim);
    ui::surface(canvas, &t, panel, t.radius(Radius::Md), Material::Popover);

    let s_title = t.style_label();
    let shown_title = ui::elide(&path, &s_title, panel.w - t.px(90.0));
    font::draw(
        canvas,
        &shown_title,
        panel.x + t.sp(Space::Sm),
        panel.y + t.px(22.0),
        &s_title,
    );
    let close_r = Rect::new(
        panel.right() - t.px(28.0),
        panel.y + t.px(8.0),
        t.px(20.0),
        t.px(20.0),
    );
    icons::draw(canvas, Icon::Close, close_r, t.text_dim(), 1.2);
    shell
        .hotspots_mut()
        .push(close_r, Hot::Files(FilesHot::ViewerClose));

    let saved = canvas.push_clip(text_area);
    let s = Style::new(t.text_px(TypeStep::Caption), t.text());
    let mut ly = text_area.y + t.px(11.0);
    for line in &lines {
        font::draw(
            canvas,
            &ui::elide(line, &s, text_area.w),
            text_area.x,
            ly,
            &s,
        );
        ly += line_h;
    }
    canvas.set_clip(saved);

    // Pager.
    let pager = Rect::new(
        panel.x + t.sp(Space::Sm),
        panel.bottom() - t.px(30.0),
        panel.w - t.sp(Space::Sm) * 2.0,
        t.px(20.0),
    );
    let up = Rect::new(pager.x, pager.y, t.px(56.0), t.px(20.0));
    ui::button(canvas, &t, up, "▲ Up", ui::State::Normal, Tone::Neutral);
    shell
        .hotspots_mut()
        .push(up, Hot::Files(FilesHot::ViewerUp));
    let down = Rect::new(pager.right() - t.px(66.0), pager.y, t.px(66.0), t.px(20.0));
    ui::button(canvas, &t, down, "▼ Down", ui::State::Normal, Tone::Neutral);
    shell
        .hotspots_mut()
        .push(down, Hot::Files(FilesHot::ViewerDown));
    let s_page = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let page = format!("line {} of {total}", scroll.saturating_add(1));
    let pw = font::measure(&page, &s_page);
    font::draw(
        canvas,
        &page,
        pager.center().0 - pw / 2.0,
        pager.y + t.px(13.0),
        &s_page,
    );
}
/// Human sizes, the way file managers round them.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = KB * KB;
    const GB: u64 = MB * KB;
    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.0} KB", b as f64 / KB as f64),
        b => format!("{b} B"),
    }
}

/// The Browser window: tab strip, omnibox row, page area.
fn draw_browser(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let app = shell.browser.clone();

    // --- tab strip ---
    //
    // The strip scrolls instead of growing past the window. Windows names
    // scroll bumpers for exactly this case and libadwaita's AdwTabBar scrolls;
    // this strip did neither, so past about `body.w / 152` tabs the right-hand
    // tabs and the + button were laid out beyond the window edge with no way to
    // reach them (finding 38). The keyboard model in `Shell::app_key` can name
    // a tab whether or not its pixels fit; the bumpers are how the pointer
    // reaches one, and selecting a tab the pointer cannot click reveals it.
    let strip_h = t.px(30.0);
    let gap = t.px(2.0);
    let tab_w = t.px(150.0);
    let inset = t.px(4.0);
    let plus_w = t.px(28.0);
    let bumper_w = t.px(20.0);
    let content_w = inset + app.tabs.len() as f32 * (tab_w + gap) + plus_w;
    let overflow = content_w > body.w;
    // Bumpers exist only while the strip overflows: the 20 px they take must
    // not shrink a strip that never needed to scroll.
    let bumper = if overflow { bumper_w } else { 0.0 };
    let area = Rect::new(
        body.x + bumper,
        body.y,
        (body.w - 2.0 * bumper).max(0.0),
        strip_h,
    );
    let max_scroll = (content_w - area.w).max(0.0);
    let mut scroll = app.strip_scroll;
    if app.reveal_active {
        // Bring the focused tab fully inside the strip. Only selection sets
        // this, so a bumper click is never undone by a reveal here.
        let left = inset + app.active as f32 * (tab_w + gap);
        let right = left + tab_w;
        if left < scroll {
            scroll = left;
        }
        if right > scroll + area.w {
            scroll = right - area.w;
        }
        shell.browser.reveal_active = false;
    }
    scroll = if scroll < 0.0 {
        0.0
    } else if scroll > max_scroll {
        max_scroll
    } else {
        scroll
    };
    shell.browser.strip_scroll = scroll;

    let clip = canvas.push_clip(area);
    let mut tx = area.x + inset - scroll;
    let s_tab = Style::new(t.text_px(TypeStep::Footnote), t.text());
    for (i, tab) in app.tabs.iter().enumerate() {
        let active_tab = i == app.active;
        let r = Rect::new(tx, area.y + t.px(2.0), tab_w, strip_h - t.px(4.0));
        tx += tab_w + gap;
        // A tab the strip has scrolled past is neither drawn nor clickable. The
        // clip covers the pixels; a rect wholly outside the strip would still
        // answer a click that never lands inside the window otherwise.
        if r.right() <= area.x || r.x >= area.right() {
            continue;
        }
        ui::surface(
            canvas,
            &t,
            r,
            t.radius(Radius::Sm),
            if active_tab {
                ui::Material::Elevated
            } else {
                ui::Material::Flat
            },
        );
        if active_tab {
            // A material difference alone is one channel, and the shell's own
            // control contract asks for two. Two pixels of accent along the
            // tab's bottom edge still say which page is on screen in a theme
            // where both materials read alike.
            canvas.fill_round_rect(
                Rect::new(
                    r.x + t.px(6.0),
                    r.bottom() - t.px(2.0),
                    r.w - t.px(12.0),
                    t.px(2.0),
                ),
                t.px(1.0),
                t.tone(Tone::Accent),
            );
        }
        // A tab strip shows titles, falling back to the address only while a
        // page has none: `https://` in every tab tells the user nothing about
        // which of six pages is which.
        let label = if tab.title.is_empty() || tab.title == tab.url {
            tab.url.clone()
        } else {
            tab.title.clone()
        };
        // Ellipsized by measured width, not character count: seventeen Latin
        // characters fit a 150 px tab and seventeen CJK glyphs do not.
        let shown = ui::elide(&label, &s_tab, tab_w - t.px(32.0));
        font::draw(canvas, &shown, r.x + t.px(8.0), r.y + t.px(14.0), &s_tab);
        shell.hotspots_mut().push(r, Hot::BrowserTab(i));
        // Close ✕, 20 px rather than 15: a 15 px target is a third of the
        // general 40 px guidance, and it is the control that ends a page.
        let cr = Rect::new(
            r.right() - t.px(23.0),
            r.y + t.px(3.0),
            t.px(20.0),
            t.px(20.0),
        );
        // A close button a bumper covers is not a target, and neither is half
        // of one: the ✕ is only drawn when it is wholly on screen.
        if cr.x >= area.x && cr.right() <= area.right() {
            let s_x = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            font::draw(canvas, "✕", cr.x + t.px(5.0), cr.y + t.px(12.0), &s_x);
            shell.hotspots_mut().push(cr, Hot::BrowserCloseTab(i));
        }
    }
    let plus = Rect::new(tx, area.y + t.px(5.0), t.px(24.0), t.px(20.0));
    if plus.x >= area.x && plus.right() <= area.right() {
        let s_plus = Style::new(t.text_px(TypeStep::Body), t.text_dim());
        font::draw(
            canvas,
            "+",
            plus.x + t.px(7.0),
            plus.y + t.px(14.0),
            &s_plus,
        );
        shell.hotspots_mut().push(plus, Hot::BrowserNewTab);
    }
    canvas.set_clip(clip);

    if overflow {
        // The bumpers. A chevron is drawn with two strokes rather than a glyph:
        // the shell's bitmap face has no arrow characters, and an unknown
        // character draws a box, which is not an affordance.
        let mid = area.y + strip_h / 2.0;
        let dy = t.px(4.0);
        let dx = t.px(5.0);
        let back = Rect::new(body.x, area.y, bumper_w, strip_h);
        let forward = Rect::new(body.right() - bumper_w, area.y, bumper_w, strip_h);
        for (rect, points_forward, enabled, hot) in [
            (back, false, scroll > 0.5, Hot::BrowserStripLeft),
            (
                forward,
                true,
                scroll < max_scroll - 0.5,
                Hot::BrowserStripRight,
            ),
        ] {
            ui::surface(canvas, &t, rect, 0.0, ui::Material::Flat);
            if !enabled {
                continue;
            }
            let cx = rect.x + bumper_w / 2.0;
            let tip = if points_forward { cx + dx } else { cx - dx };
            let tail = if points_forward { cx - dx } else { cx + dx };
            canvas.stroke_line(tail, mid - dy, tip, mid, 1.5, t.text());
            canvas.stroke_line(tip, mid, tail, mid + dy, 1.5, t.text());
            shell.hotspots_mut().push(rect, hot);
        }
    }

    // --- omnibox row ---
    let bar = Rect::new(body.x, body.y + strip_h, body.w, t.px(30.0));
    ui::surface(canvas, &t, bar, 0.0, ui::Material::Flat);
    let focused_tab_url = app.tab().map(|tb| tb.url.clone()).unwrap_or_default();
    let omni_text = if app.omnibox_focus {
        app.omnibox.clone()
    } else {
        focused_tab_url
    };
    let field = Rect::new(
        bar.x + t.px(8.0),
        bar.y + t.px(4.0),
        bar.w - t.px(16.0),
        bar.h - t.px(8.0),
    );
    canvas.stroke_round_rect(field, t.radius(Radius::Sm), 1.0, t.border());
    let s_omni = Style::new(
        t.text_px(TypeStep::Footnote),
        if app.omnibox_focus {
            t.text()
        } else {
            t.text_dim()
        },
    );
    font::draw(
        canvas,
        &omni_text,
        field.x + t.px(8.0),
        field.y + t.px(14.0),
        &s_omni,
    );
    if app.omnibox_focus {
        let caret_x = field.x + t.px(8.0) + font::measure(&omni_text, &s_omni);
        canvas.stroke_line(
            caret_x + 1.0,
            field.y + 3.0,
            caret_x + 1.0,
            field.bottom() - 3.0,
            1.0,
            t.accent(),
        );
    }
    shell.hotspots_mut().push(field, Hot::BrowserOmnibox);

    let items = [
        ("Back", Hot::BrowserBack),
        ("Forward", Hot::BrowserForward),
        ("Reload", Hot::BrowserReload),
        ("Stop", Hot::BrowserStop),
        (
            if app.is_bookmarked() { "Saved" } else { "Save" },
            Hot::BrowserBookmark,
        ),
        ("History", Hot::BrowserHistory),
        ("Bookmarks", Hot::BrowserBookmarks),
        ("Downloads", Hot::BrowserDownloads),
        ("Site", Hot::BrowserSitePermissions),
    ];
    let button_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let desired: Vec<f32> = items
        .iter()
        .map(|(label, _)| font::measure(label, &button_style) + t.px(18.0))
        .collect();
    let gap = t.px(4.0);
    let available = (body.w - t.px(12.0)).max(t.px(120.0));
    let gaps = items.len().saturating_sub(1);
    let single_width: f32 = desired.iter().sum::<f32>() + gap * gaps as f32;
    // The row split is the item count's, not a literal's: a control added to
    // the list above without touching this would otherwise never be drawn,
    // and the buttons that were drawn would keep the old indices.
    let split = items.len().div_ceil(2);
    let groups: Vec<(usize, usize)> = if single_width <= available {
        vec![(0, items.len())]
    } else {
        vec![(0, split), (split, items.len())]
    };
    let controls = Rect::new(
        body.x,
        bar.bottom(),
        body.w,
        if groups.len() == 1 {
            t.px(30.0)
        } else {
            t.px(56.0)
        },
    );
    ui::surface(canvas, &t, controls, 0.0, ui::Material::Flat);
    for (row_index, (start, end)) in groups.iter().copied().enumerate() {
        let row: &[f32] = desired.get(start..end).unwrap_or_default();
        let row_gaps = end.saturating_sub(start).saturating_sub(1);
        let row_width: f32 = row.iter().sum::<f32>() + gap * row_gaps as f32;
        let scale = (available / row_width.max(1.0)).min(1.0);
        let mut bx = controls.x + t.px(6.0);
        for index in start..end {
            let Some((label, hot)) = items.get(index).copied() else {
                continue;
            };
            let width = desired.get(index).copied().unwrap_or(0.0) * scale;
            let rect = Rect::new(
                bx,
                controls.y + t.px(3.0) + row_index as f32 * t.px(26.0),
                width,
                t.px(24.0),
            );
            let disabled = (matches!(hot, Hot::BrowserBack) && !app.can_go_back())
                || (matches!(hot, Hot::BrowserForward) && !app.can_go_forward())
                // Stop over a page that is not arriving stops nothing, and
                // Reload on the shell's own start page has no page to reload.
                || (matches!(hot, Hot::BrowserStop) && !app.can_stop())
                || (matches!(hot, Hot::BrowserReload) && !app.can_reload());
            let selected = (matches!(hot, Hot::BrowserHistory)
                && app.panel == crate::apps::BrowserPanel::History)
                || (matches!(hot, Hot::BrowserBookmarks)
                    && app.panel == crate::apps::BrowserPanel::Bookmarks)
                || (matches!(hot, Hot::BrowserDownloads)
                    && app.panel == crate::apps::BrowserPanel::Downloads)
                || (matches!(hot, Hot::BrowserSitePermissions)
                    && app.panel == crate::apps::BrowserPanel::SitePermissions)
                || (matches!(hot, Hot::BrowserBookmark) && app.is_bookmarked());
            let state = if disabled {
                State::Disabled
            } else if selected {
                State::Selected
            } else if shell.is_highlighted(hot) {
                State::Hover
            } else {
                State::Normal
            };
            let shown_label = ui::elide(label, &button_style, (width - t.px(12.0)).max(t.px(8.0)));
            ui::button(canvas, &t, rect, &shown_label, state, Tone::Neutral);
            if !disabled {
                shell.hotspots_mut().push(rect, hot);
            }
            bx += width + gap;
        }
    }

    // --- page area: engine frame when one arrives, honest state otherwise ---
    let page = Rect::new(
        body.x,
        controls.bottom(),
        body.w,
        body.bottom() - controls.bottom(),
    );
    let Some(tab) = app.tab() else {
        return;
    };
    let s_head = Style::new(t.text_px(TypeStep::Title3), t.text());
    let s_body = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());

    match app.panel {
        crate::apps::BrowserPanel::History | crate::apps::BrowserPanel::Bookmarks => {
            let history = app.panel == crate::apps::BrowserPanel::History;
            let title = if history { "History" } else { "Bookmarks" };
            font::draw(
                canvas,
                title,
                page.x + t.sp(Space::Md),
                page.y + t.px(28.0),
                &s_head,
            );
            let rows: Vec<&String> = if history {
                app.history.iter().take(12).collect()
            } else {
                app.bookmarks.iter().rev().take(12).collect()
            };
            if rows.is_empty() {
                font::draw(
                    canvas,
                    if history {
                        "No pages visited yet"
                    } else {
                        "No bookmarks saved yet"
                    },
                    page.x + t.sp(Space::Md),
                    page.y + t.px(58.0),
                    &s_body,
                );
            }
            for (index, url) in rows.iter().enumerate() {
                let row = Rect::new(
                    page.x + t.sp(Space::Sm),
                    page.y + t.px(42.0) + index as f32 * t.px(30.0),
                    page.w - t.sp(Space::Sm) * 2.0,
                    t.px(28.0),
                );
                let hot = if history {
                    Hot::BrowserHistoryItem(index)
                } else {
                    Hot::BrowserBookmarkItem(index)
                };
                if shell.is_highlighted(hot) {
                    canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.text(), 0.06));
                }
                font::draw(
                    canvas,
                    &ui::elide(url, &s_body, row.w - t.px(12.0)),
                    row.x + t.px(8.0),
                    row.y + t.px(18.0),
                    &s_body,
                );
                shell.hotspots_mut().push(row, hot);
            }
            return;
        }
        crate::apps::BrowserPanel::Downloads => {
            font::draw(
                canvas,
                // The same folder the Files window shows: the browser's
                // downloads are imported into the user's `/Downloads` by the
                // file service, so they survive a reboot and can be opened from
                // either window. It said "(this session)" while they lived in
                // the broker's tmpfs directory.
                "Downloads",
                page.x + t.sp(Space::Md),
                page.y + t.px(28.0),
                &s_head,
            );
            if app.downloads.is_empty() {
                let message = app
                    .downloads_status
                    .as_deref()
                    .unwrap_or("Nothing has been downloaded yet");
                font::draw(
                    canvas,
                    &ui::elide(message, &s_body, page.w - t.sp(Space::Md) * 2.0),
                    page.x + t.sp(Space::Md),
                    page.y + t.px(58.0),
                    &s_body,
                );
            }
            for (index, download) in app.downloads.iter().take(14).enumerate() {
                let row_y = page.y + t.px(42.0) + index as f32 * t.px(28.0);
                font::draw(
                    canvas,
                    &ui::elide(&download.name, &s_body, page.w * 0.62),
                    page.x + t.sp(Space::Md),
                    row_y + t.px(18.0),
                    &s_body,
                );
                let size = format_size(download.size);
                let width = font::measure(&size, &s_body);
                font::draw(
                    canvas,
                    &size,
                    page.right() - t.sp(Space::Md) - width,
                    row_y + t.px(18.0),
                    &s_body,
                );
            }
            return;
        }
        crate::apps::BrowserPanel::SitePermissions => {
            font::draw(
                canvas,
                "Site controls",
                page.x + t.sp(Space::Md),
                page.y + t.px(28.0),
                &s_head,
            );
            let origin = tab.url.split('/').take(3).collect::<Vec<_>>().join("/");
            font::draw(
                canvas,
                &ui::elide(&origin, &s_body, page.w - t.sp(Space::Md) * 2.0),
                page.x + t.sp(Space::Md),
                page.y + t.px(58.0),
                &s_body,
            );
            let notice = "This engine does not surface site permission requests to LitricOS, so this page cannot grant device access.";
            font::draw(
                canvas,
                &ui::elide(notice, &s_body, page.w - t.sp(Space::Md) * 2.0),
                page.x + t.sp(Space::Md),
                page.y + t.px(86.0),
                &s_body,
            );
            return;
        }
        crate::apps::BrowserPanel::Page => {}
    }

    // A frame from the engine session replaces the placeholder entirely. Where
    // it was drawn is published to the shell: pointer positions are mapped
    // through that rectangle, so a click lands on the pixel aimed at.
    if let (Some(frame), Some((w, h))) = (shell.browser_frame.as_ref(), app.last_frame) {
        let placed = blit_engine_frame(canvas, frame, w, h, page);
        shell.set_browser_engine_area(placed);
        return;
    }
    shell.set_browser_engine_area(None);

    let cy = page.center().1;
    if tab.url == "litric://about" {
        // The start page, in the shape users already know: a wordmark, one
        // centered search box, engine chips, and a few quick links. One
        // input path — the box drives the omnibox, so Enter searches with
        // the chosen engine (Google by default).
        let cx = page.center().0;
        let s_word = Style::bold(t.text_px(TypeStep::Title1), t.text());
        let word = "Litric Search";
        let ww = font::measure(word, &s_word);
        font::draw(canvas, word, cx - ww / 2.0, page.y + t.px(56.0), &s_word);

        let fw = t.px(460.0).min(page.w - t.px(40.0));
        let fr = Rect::new(cx - fw / 2.0, page.y + t.px(76.0), fw, t.px(40.0));
        canvas.fill_round_rect(fr, t.radius(Radius::Lg), with_alpha(rgb(0xFFFFFF), 0.08));
        canvas.stroke_round_rect(
            fr,
            t.radius(Radius::Lg),
            1.0,
            if app.omnibox_focus {
                t.accent()
            } else {
                t.border()
            },
        );
        let s_field = Style::new(t.text_px(TypeStep::Body), t.text());
        let typed = if app.omnibox_focus {
            app.omnibox.clone()
        } else {
            String::new()
        };
        if typed.is_empty() && !app.omnibox_focus {
            font::draw(
                canvas,
                "Search the web with Google, or type a URL",
                fr.x + t.px(14.0),
                fr.y + t.px(25.0),
                &s_body,
            );
        } else {
            font::draw(
                canvas,
                &typed,
                fr.x + t.px(14.0),
                fr.y + t.px(25.0),
                &s_field,
            );
        }
        if app.omnibox_focus {
            let caret_x = fr.x + t.px(14.0) + font::measure(&typed, &s_field);
            canvas.stroke_line(
                caret_x + 1.0,
                fr.y + t.px(8.0),
                caret_x + 1.0,
                fr.bottom() - t.px(8.0),
                1.0,
                t.accent(),
            );
        }
        shell.hotspots_mut().push(fr, Hot::BrowserOmnibox);

        // Engine chips: the selected one carries the accent fill and the
        // check is its own glyph — state never rests on colour alone.
        let chips = crate::apps::SearchEngine::ALL;
        let chip_w = t.px(96.0);
        let total = chip_w * (chips.len() as f32) + t.px(8.0) * (chips.len() as f32 - 1.0);
        let mut cx2 = cx - total / 2.0;
        for (i, e) in chips.iter().enumerate() {
            let r = Rect::new(cx2, fr.bottom() + t.px(16.0), chip_w, t.px(26.0));
            let selected = app.engine == *e;
            if selected {
                canvas.fill_round_rect(r, t.radius(Radius::Md), with_alpha(t.accent(), 0.22));
            }
            canvas.stroke_round_rect(
                r,
                t.radius(Radius::Md),
                1.0,
                if selected { t.accent() } else { t.border() },
            );
            let s_chip = Style::bold(t.text_px(TypeStep::Footnote), t.text());
            let lw = font::measure(e.label(), &s_chip);
            font::draw(
                canvas,
                e.label(),
                r.center().0 - lw / 2.0,
                r.center().1 + s_chip.size * 0.36,
                &s_chip,
            );
            shell.hotspots_mut().push(r, Hot::EngineChip(i as u8));
            cx2 += chip_w + t.px(8.0);
        }

        // Quick links: one row of tiles that navigate for real.
        let links = [("Google", 0u8), ("Wikipedia", 1), ("GitHub", 2)];
        let tile_w = t.px(110.0);
        let total = tile_w * (links.len() as f32) + t.px(12.0) * (links.len() as f32 - 1.0);
        let mut lx = cx - total / 2.0;
        for (label, i) in links {
            let r = Rect::new(lx, fr.bottom() + t.px(64.0), tile_w, t.px(48.0));
            let hot = Hot::QuickLink(i);
            let hovered = shell.is_highlighted(hot);
            if hovered {
                canvas.fill_round_rect(r, t.radius(Radius::Md), with_alpha(rgb(0xFFFFFF), 0.09));
            }
            canvas.stroke_round_rect(r, t.radius(Radius::Md), 1.0, t.border());
            let s_link = Style::new(t.text_px(TypeStep::Footnote), t.text());
            let lw = font::measure(label, &s_link);
            font::draw(
                canvas,
                label,
                r.center().0 - lw / 2.0,
                r.center().1 + s_link.size * 0.36,
                &s_link,
            );
            shell.hotspots_mut().push(r, hot);
            lx += tile_w + t.px(12.0);
        }
        return;
    } else {
        font::draw(
            canvas,
            &tab.url,
            page.x + t.sp(Space::Lg),
            cy - t.px(10.0),
            &s_head,
        );
        let note = if app.status.is_empty() {
            "No engine session yet: right-click the page → Start engine session, and litric-browserd renders this address.".to_owned()
        } else {
            app.status.clone()
        };
        font::draw(
            canvas,
            &ui::elide(&note, &s_body, page.w - t.sp(Space::Lg) * 2.0),
            page.x + t.sp(Space::Lg),
            cy + t.px(14.0),
            &s_body,
        );
    }

    // --- engine picker: Google is the default, the choice is real ---
    let pick_h = t.px(24.0);
    let pick = Rect::new(page.x, page.bottom() - pick_h, page.w, pick_h);
    ui::surface(canvas, &t, pick, 0.0, Material::Flat);
    let s_pick = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    font::draw(
        canvas,
        "Search engine:",
        pick.x + t.px(8.0),
        pick.y + t.px(15.0),
        &s_pick,
    );
    let mut ex = pick.x + t.px(96.0);
    for engine in crate::apps::SearchEngine::ALL {
        let label = engine.title();
        let w = font::measure(label, &s_pick) + t.px(14.0);
        let r = Rect::new(ex, pick.y + t.px(3.0), w, pick_h - t.px(6.0));
        let active = app.engine == engine;
        if active {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(t.accent(), 0.18));
        }
        let s = Style::new(
            t.text_px(TypeStep::Caption),
            if active { t.accent() } else { t.text_dim() },
        );
        font::draw(
            canvas,
            label,
            r.x + t.px(6.0),
            r.y + r.h / 2.0 + s.size * 0.36,
            &s,
        );
        shell.hotspots_mut().push(
            r,
            Hot::MenuRow(match engine {
                crate::apps::SearchEngine::Google => crate::apps::MenuAction::EngineGoogle,
                crate::apps::SearchEngine::DuckDuckGo => crate::apps::MenuAction::EngineDuckDuckGo,
                crate::apps::SearchEngine::Bing => crate::apps::MenuAction::EngineBing,
            }),
        );
        ex += w + t.px(6.0);
    }
}

/// Draw an engine frame into the page area: row-by-row RGBA blit, centred,
/// scaled down only when the page does not match the frame. Pixel work is
/// bounded by the clip: a frame larger than the page never costs more than
/// the page area.
fn blit_engine_frame(
    canvas: &mut Canvas,
    frame: &[u8],
    w: u32,
    h: u32,
    page: Rect,
) -> Option<Rect> {
    let needed = u64::from(w)
        .checked_mul(u64::from(h))
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(usize::MAX);
    if frame.len() < needed || w == 0 || h == 0 {
        return None;
    }
    let saved = canvas.push_clip(page);
    let fw = w as f32;
    let fh = h as f32;
    // Fit inside the page, preserving aspect.
    let scale = (page.w / fw).min(page.h / fh).min(1.0);
    let dw = fw * scale;
    let dh = fh * scale;
    let x0 = page.x + (page.w - dw) / 2.0;
    let y0 = page.y + (page.h - dh) / 2.0;
    let step_x = fw / dw;
    let step_y = fh / dh;
    let mut py = y0;
    while py < y0 + dh {
        let src_y = usize::try_from(((py - y0) * step_y) as i64)
            .unwrap_or(0)
            .min((h as usize).saturating_sub(1));
        let mut px = x0;
        while px < x0 + dw {
            let src_x = usize::try_from(((px - x0) * step_x) as i64)
                .unwrap_or(0)
                .min((w as usize).saturating_sub(1));
            let i = src_y
                .checked_mul(w as usize)
                .and_then(|r| r.checked_add(src_x))
                .and_then(|p| p.checked_mul(4))
                .unwrap_or(usize::MAX);
            // A clamped source coordinate can touch the last row's tail;
            // `get` keeps a rounded step from panicking on it.
            let at = |off: usize| i.checked_add(off).and_then(|j| frame.get(j)).copied();
            let (b, g, r, a) = match (at(0), at(1), at(2), at(3)) {
                (Some(b), Some(g), Some(r), Some(a)) => {
                    (u32::from(b), u32::from(g), u32::from(r), u32::from(a))
                }
                _ => {
                    px += 1.0;
                    continue;
                }
            };
            let color = (a << 24) | (r << 16) | (g << 8) | b;
            canvas.put(
                (px as i64).max(0),
                (py as i64).max(0),
                if a == 0xFF {
                    color
                } else {
                    crate::canvas::blend(
                        canvas
                            .get((px as i64).max(0), (py as i64).max(0))
                            .unwrap_or(0),
                        color,
                        1.0,
                    )
                },
            );
            px += 1.0;
        }
        py += 1.0;
    }
    canvas.set_clip(saved);
    Some(Rect::new(x0, y0, dw, dh))
}

/// The system monitor, filled from measured values only.
///
/// Minimal by rule, not by omission of work: a row exists only when its value
/// was measured. An unmeasurable quantity is simply absent — the interface
/// never shows a control that says it knows nothing.
fn draw_activity(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let facts = shell.facts.clone();
    // The monitor has several measured sections; at the old 340px default the
    // last of them painted through the process table. Keep the overview and
    // table in separate clipped panes, and bias the dashboard toward enough
    // vertical space for the CPU/memory cards and device summary.
    let table_h = (body.h * 0.38)
        .clamp(t.px(158.0), t.px(280.0))
        .min(body.h * 0.55);
    let top_h = (body.h - table_h).max(0.0);
    let overview = Rect::new(body.x, body.y, body.w, top_h);
    let saved = canvas.push_clip(overview);
    crate::monitor::draw(canvas, &t, &facts, &shell.activity, overview);
    canvas.set_clip(saved);
    let table_rect = Rect::new(body.x, body.y + top_h, body.w, table_h);
    crate::ui::separator(canvas, &t, Rect::new(body.x, table_rect.y, body.w, 1.0));

    // Search stays above the clipped process table so it remains visible when
    // the window is short. Typing filters the actual TaskManager rows.
    let search = Rect::new(
        table_rect.x + t.sp(Space::Sm),
        table_rect.y + t.sp(Space::Xs),
        (table_rect.w - t.sp(Space::Sm) * 2.0).min(t.px(300.0)),
        t.px(28.0),
    );
    let search_hot = Hot::ActivityFilter;
    ui::text_field(
        canvas,
        &t,
        search,
        &shell.activity.prefs.filter_text,
        "Search processes",
        shell.is_highlighted(search_hot),
    );
    shell.hotspots_mut().push(search, search_hot);

    let table_body = Rect::new(
        table_rect.x,
        search.bottom() + t.sp(Space::Xxs),
        table_rect.w,
        (table_rect.bottom() - search.bottom() - t.sp(Space::Xxs)).max(0.0),
    );
    let saved = canvas.push_clip(table_body);
    crate::activity::table(canvas, &t, &shell.activity, table_body);
    canvas.set_clip(saved);

    // Register exactly the visible heading widths used by the table renderer.
    // Clicking a heading toggles its sort; the keyboard focus ring uses the
    // same rectangle as pointer hit-testing.
    let inner = Rect::new(
        table_body.x + t.sp(Space::Sm),
        table_body.y + t.sp(Space::Xs),
        (table_body.w - t.sp(Space::Sm) * 2.0).max(0.0),
        table_body.h,
    );
    let (columns, _) = crate::activity::column_widths(&shell.activity, t.scale, inner.w);
    let header_style = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());
    let header_h = header_style.size + t.sp(Space::Sm);
    let mut x = inner.x;
    for (column, width) in columns {
        let rect = Rect::new(x, inner.y, width, header_h);
        let hot = Hot::ActivitySort(column);
        shell.hotspots_mut().push(rect, hot);
        if shell.is_highlighted(hot) {
            canvas.stroke_round_rect(rect, t.radius(Radius::Sm), 1.0, t.accent());
        }
        x += width;
    }
}

/// The environment list: real states, from the daemon.
fn draw_environments(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let rows = shell.environments.clone();
    let mut col = ui::Column::new(body, t.sp(Space::Md), t.sp(Space::Xs));

    if rows.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(
            canvas,
            "Unavailable - litric-sysd did not answer",
            col.x,
            col.cursor + t.px(20.0),
            &s,
        );
        return;
    }

    // Calm list: dot + title + hotkey. State shows only where it varies from
    // "not running" — a green dot for the active row, nothing for the rest —
    // instead of a badge shouting every state in capitals.
    for (i, e) in rows.iter().enumerate() {
        let row = col.row(t.px(40.0));
        let hot = Hot::QuickEnvironment(i as u8);
        let hovered = shell.is_highlighted(hot);
        if e.active || hovered {
            canvas.fill_round_rect(
                row,
                t.radius(Radius::Sm),
                with_alpha(t.text(), if e.active { 0.07 } else { 0.045 }),
            );
        }
        let dot_y = row.y + row.h / 2.0;
        let dot = t.px(6.0);
        canvas.fill_circle(
            row.x + dot,
            dot_y,
            dot / 2.0,
            if e.active {
                t.tone(Tone::Good)
            } else {
                with_alpha(t.text_dim(), 0.5)
            },
        );
        let s = Style::new(
            t.text_px(TypeStep::Footnote),
            if e.active { t.text() } else { t.text_dim() },
        );
        font::draw(
            canvas,
            &e.title,
            row.x + t.px(18.0),
            dot_y + s.size * 0.36,
            &s,
        );
        if !e.hotkey.is_empty() {
            let s_key = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            let w = font::measure(&e.hotkey, &s_key);
            font::draw(
                canvas,
                &e.hotkey,
                row.right() - w,
                dot_y + s_key.size * 0.36,
                &s_key,
            );
        }
        shell.hotspots_mut().push(row, hot);
    }
}

/// Settings: real shell-owned preferences and measured system facts.
fn draw_settings(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    // The same sidebar Files uses. Settings' pane used to be 36% of the window
    // -- 504 px at 1400, well past libadwaita's 280sp maximum -- with thirteen
    // undifferentiated 30 px rows and no scroll, so on a short window the last
    // pages could not be reached at all.
    let sidebar_w = ui::sidebar::width(body.w);
    let sidebar = Rect::new(body.x, body.y, sidebar_w, body.h);
    // The same pane material as Files', so the two windows do not disagree about
    // what a sidebar is made of either. Settings used to paint 16 % black.
    ui::surface(canvas, &t, sidebar, 0.0, ui::Material::Elevated);
    let items: Vec<ui::sidebar::Item<'_>> = crate::settings::Page::ALL
        .iter()
        .copied()
        .enumerate()
        .map(|(i, page)| ui::sidebar::Item {
            label: page.title(),
            icon: Some(settings_icon(page)),
            selected: page == shell.settings.page,
            highlighted: shell.is_highlighted(Hot::SettingsCategory(i as u8)),
        })
        .collect();
    shell.hotspots_mut().push(
        ui::sidebar::rows_area(&t, sidebar, true),
        Hot::SettingsSidebarScroll,
    );
    let rows_area = ui::sidebar::rows_area(&t, sidebar, true);
    let scroll = ui::sidebar::clamp(&t, rows_area, items.len(), shell.settings.sidebar_scroll);
    shell.settings.sidebar_scroll = scroll;
    for (i, row) in ui::sidebar::draw(canvas, &t, sidebar, Some("Settings"), &items, scroll) {
        shell
            .hotspots_mut()
            .push(row, Hot::SettingsCategory(i as u8));
    }

    let pane = Rect::new(body.x + sidebar_w, body.y, body.w - sidebar_w, body.h);
    let selected_page = shell.settings.page;
    let mut c = ui::Column::new(pane, t.sp(Space::Lg), t.sp(Space::Sm));
    let title_style = t.style_bold(TypeStep::Title3);
    font::draw(
        canvas,
        selected_page.title(),
        c.x,
        c.cursor + t.px(24.0),
        &title_style,
    );
    c.space(t.px(38.0));

    if selected_page != crate::settings::Page::Region {
        draw_settings_page(canvas, shell, pane, selected_page, &t);
        return;
    }

    ui::section_label(
        canvas,
        &t,
        (c.x, c.cursor + t.px(10.0)),
        "REGION AND LANGUAGE",
    );
    c.space(t.px(20.0));

    let l = &shell.regional.locale;
    let z = &shell.regional.zone;
    let pairs = [
        ("Locale".to_owned(), l.tag.clone()),
        ("Detected from".to_owned(), l.source.token()),
        ("Time zone".to_owned(), z.name().to_owned()),
        ("Zone source".to_owned(), z.source().token()),
        (
            "Clock".to_owned(),
            match l.hour_cycle {
                litric_locale::HourCycle::H12 => "12-hour".to_owned(),
                litric_locale::HourCycle::H24 => "24-hour".to_owned(),
            },
        ),
        ("First day".to_owned(), l.first_day.short_name().to_owned()),
        ("Keyboard".to_owned(), l.keyboard_layout.clone()),
        (
            "Units".to_owned(),
            match l.measurement {
                litric_locale::Measurement::Metric => "Metric".to_owned(),
                litric_locale::Measurement::Imperial => "Imperial".to_owned(),
            },
        ),
    ];
    for (k, v) in pairs {
        let row = c.row(t.px(26.0));
        let ks = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(canvas, &k, row.x, row.y + t.px(16.0), &ks);
        let vs = Style::new(t.text_px(TypeStep::Footnote), t.text());
        // The value takes what the label leaves. Eliding it against the pane
        // instead let "Detected from" and its locale overwrite each other by 73
        // pixels in a narrow window -- two strings in one band, which is the
        // "text on top of text" this audit exists to catch.
        let room = (row.w - font::measure(&k, &ks) - t.px(12.0)).max(t.px(24.0));
        let shown = crate::monitor::fit(&v, &vs, room);
        let w = font::measure(&shown, &vs);
        font::draw(canvas, &shown, row.right() - w, row.y + t.px(16.0), &vs);
    }

    // --- System: what this machine is, from the machine ----------------
    // Read-only by design: every row here is a measurement, so there is no
    // switch that could disagree with the machine.
    c.space(t.px(12.0));
    ui::section_label(canvas, &t, (c.x, c.cursor + t.px(10.0)), "SYSTEM");
    c.space(t.px(20.0));
    let facts = shell.facts.clone();
    let mut rows: Vec<(String, String)> =
        vec![("Version".to_owned(), env!("CARGO_PKG_VERSION").to_owned())];
    if let Some(fs) = &facts.root_fs {
        rows.push(("Root filesystem".to_owned(), fs.clone()));
    }
    if let Some(model) = &facts.cpu_model {
        rows.push(("Processor".to_owned(), model.clone()));
    }
    if let (Some(used), Some(total)) = (facts.mem_used_mb, facts.mem_total_mb) {
        rows.push(("Memory".to_owned(), format!("{used} of {total} MB")));
    }
    rows.extend(crate::monitor::system_rows(&facts));
    if facts.source.is_empty() {
        rows.push(("Measured by".to_owned(), "nothing yet".to_owned()));
    } else {
        rows.push(("Measured by".to_owned(), facts.source.clone()));
    }
    for (k, v) in rows {
        let row = c.row(t.px(26.0));
        let ks = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(canvas, &k, row.x, row.y + t.px(16.0), &ks);
        let vs = Style::new(t.text_px(TypeStep::Footnote), t.text());
        let room = (row.w - font::measure(&k, &ks) - t.px(12.0)).max(t.px(24.0));
        let shown = crate::monitor::fit(&v, &vs, room);
        let w = font::measure(&shown, &vs);
        font::draw(canvas, &shown, row.right() - w, row.y + t.px(16.0), &vs);
    }

    // --- Storage: the answer to "what is EROFS" -----------------------
    c.space(t.px(12.0));
    ui::section_label(canvas, &t, (c.x, c.cursor + t.px(10.0)), "STORAGE");
    c.space(t.px(20.0));
    let volumes = crate::monitor::storage_rows(&facts);
    if volumes.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(
            canvas,
            "Unavailable - the kernel reported no mounted volumes",
            c.x,
            c.cursor + t.px(14.0),
            &s,
        );
        c.space(t.px(20.0));
    }
    for (mount, detail, note) in volumes {
        let row = c.row(t.px(26.0));
        let ms = Style::new(t.text_px(TypeStep::Footnote), t.text());
        let shown = crate::monitor::fit(&mount, &ms, row.w * 0.6);
        font::draw(canvas, &shown, row.x, row.y + t.px(16.0), &ms);
        let ds = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let dw = font::measure(&detail, &ds);
        font::draw(canvas, &detail, row.right() - dw, row.y + t.px(16.0), &ds);
        if let Some(note) = note {
            let ns = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            let line = crate::monitor::fit(note, &ns, pane.w);
            let nrow = c.row(t.px(18.0));
            font::draw(canvas, &line, nrow.x, nrow.y + t.px(12.0), &ns);
        }
    }
}

/// What the running text stack actually is, for the Diagnostics page.
///
/// Three answers, not two: the stack has always been able to be *partly*
/// installed, and a substituted face changes every word width on the screen. A
/// page whose job is to report the truth about the machine cannot round that to
/// "offline shaping is working".
fn text_renderer_state() -> String {
    if !crate::font::initialize() {
        return crate::i18n::text("Vector fallback · system fonts unavailable").into_owned();
    }
    let faces = crate::font::face_report();
    if faces.complete() {
        return crate::i18n::text("OpenType shaping · offline").into_owned();
    }
    format!(
        "{} {}",
        crate::i18n::text("Substituted face:"),
        faces.missing().join(", ")
    )
}

/// What an input device's capability class is called on the page.
///
/// The platform's words are lower-case tokens (`keyboard`, `tablet`); a row of
/// Settings values starts with a capital, and a device whose class the platform
/// did not recognise says `Input device` rather than nothing.
fn kind_label(kind: &str) -> &'static str {
    match kind {
        "keyboard" => "Keyboard",
        "mouse" => "Mouse",
        "tablet" => "Tablet",
        _ => "Input device",
    }
}

fn settings_icon(page: crate::settings::Page) -> Icon {
    use crate::settings::Page;
    match page {
        Page::Appearance => Icon::Grid,
        Page::Display => Icon::Window,
        Page::Input => Icon::Terminal,
        Page::Network => Icon::Network,
        Page::Bluetooth => Icon::Bluetooth,
        Page::Power => Icon::Power,
        Page::Region => Icon::Info,
        Page::Environments => Icon::Environment,
        Page::Accessibility => Icon::User,
        Page::Sound => Icon::Audio,
        Page::Privacy => Icon::Shield,
        Page::Accounts => Icon::User,
        Page::Diagnostics => Icon::Activity,
    }
}

/// One editable row on a Settings page: the store key it changes, the words on
/// the button, and the hotspot that presses it.
struct SettingRow<'a> {
    /// The `settings.conf` key, compared against a deep link's focus.
    key: &'static str,
    /// The button's text.
    label: &'a str,
    /// What clicking it does.
    hot: Hot,
}

fn settings_button(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &Tokens,
    pane: Rect,
    y: f32,
    row: SettingRow<'_>,
) {
    let r = Rect::new(
        pane.x + t.sp(Space::Lg),
        y,
        (pane.w - t.sp(Space::Lg) * 2.0).min(t.px(230.0)),
        t.px(30.0),
    );
    // A row a `litric://settings/<page>/<key>` link named is marked with a ring
    // rather than a different fill: the accent fill already means "this is a
    // control", and a second fill would read as a different kind of control.
    let focused = shell.settings.focus == Some(row.key);
    ui::button(canvas, t, r, row.label, State::Normal, Tone::Accent);
    if focused {
        canvas.stroke_round_rect(r, t.radius(Radius::Sm), t.px(2.0), t.text());
    }
    shell.hotspots_mut().push(r, row.hot);
}

fn settings_rows(canvas: &mut Canvas, t: &Tokens, pane: Rect, rows: &[(String, String)]) {
    let mut y = pane.y + t.px(56.0);
    for (label, value) in rows {
        let row = Rect::new(
            pane.x + t.sp(Space::Lg),
            y,
            pane.w - t.sp(Space::Lg) * 2.0,
            t.px(28.0),
        );
        let label_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let value_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
        font::draw(canvas, label, row.x, row.y + t.px(18.0), &label_style);
        let room = (row.w - font::measure(label, &label_style) - t.px(10.0)).max(t.px(30.0));
        let shown = ui::elide(value, &value_style, room);
        let width = font::measure(&shown, &value_style);
        font::draw(
            canvas,
            &shown,
            row.right() - width,
            row.y + t.px(18.0),
            &value_style,
        );
        y += t.px(30.0);
    }
}

fn draw_settings_page(
    canvas: &mut Canvas,
    shell: &mut Shell,
    pane: Rect,
    page: crate::settings::Page,
    t: &Tokens,
) {
    use crate::settings::{Page, SaveState};
    let facts = shell.facts.clone();
    let mut y = pane.y + t.px(48.0);
    match page {
        Page::Appearance => {
            let theme = shell.settings.preferences.theme;
            let label = format!("{} · {}", "Change theme", theme_name(theme));
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "theme",
                    label: &label,
                    hot: Hot::SettingsTheme,
                },
            );
            y += t.px(40.0);
            // The accent is a preference about decoration, never about
            // contrast: a high-contrast theme keeps its own accent and the row
            // says so instead of offering a control that would do nothing.
            let accent_label = if shell.settings.preferences.theme.requires_aaa() {
                "Accent · fixed in high contrast themes".to_owned()
            } else {
                format!(
                    "{} · {}",
                    "Change accent",
                    accent_name(shell.settings.preferences.accent)
                )
            };
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "accent",
                    label: &accent_label,
                    hot: Hot::SettingsAccent,
                },
            );
            y += t.px(40.0);
            let wallpaper_label = format!(
                "{} · {}",
                "Change wallpaper",
                wallpaper_name(shell.settings.preferences.wallpaper)
            );
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "wallpaper",
                    label: &wallpaper_label,
                    hot: Hot::SettingsWallpaper,
                },
            );
            y += t.px(40.0);
            // `label` rather than `title`: a language offered under its own
            // name is the friendly thing to do, until the face that draws that
            // name is the missing one — then the cheerful label is itself a row
            // of boxes, and the reader learns nothing. When Thai cannot be
            // drawn the button says so and says why.
            let label = format!(
                "{} · {}",
                "Change language",
                shell.settings.preferences.language.label()
            );
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "language",
                    label: &label,
                    hot: Hot::SettingsLanguage,
                },
            );
            y += t.px(40.0);
            // The schedule is its own row rather than a fourth theme: it
            // answers "when", and the theme row answers "which". The detail
            // line under it is the same `effective` call the shell makes when it
            // paints, so the sentence cannot outlive the behaviour.
            let label = format!(
                "{} · {}",
                "Automatic light and dark",
                shell.settings.preferences.theme_schedule.title()
            );
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "theme_schedule",
                    label: &label,
                    hot: Hot::SettingsSchedule,
                },
            );
            if shell.settings.preferences.theme_schedule == crate::settings::ThemeSchedule::Auto {
                let hour = shell.local_hour();
                let detail = crate::registry::schedule_detail(&shell.settings.preferences, hour);
                let style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
                font::draw(
                    canvas,
                    &detail,
                    pane.x + t.sp(Space::Lg),
                    y + t.px(42.0),
                    &style,
                );
            }
        }
        Page::Display => {
            settings_rows(
                canvas,
                t,
                pane,
                &[
                    (
                        "Current framebuffer".to_owned(),
                        format!("{} × {} px", shell.wm.screen().w, shell.wm.screen().h),
                    ),
                    (
                        "QEMU default resolution".to_owned(),
                        "1920 × 1080".to_owned(),
                    ),
                    ("Compatibility mode".to_owned(), "1280 × 800".to_owned()),
                    ("UI scale".to_owned(), format!("{:.2}×", t.scale)),
                    (
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
            // The devices, by name, from the kernel's own list. This page used
            // to say "Pointer and keyboard: interactive" whether or not one was
            // plugged in, which is the one claim a Settings page must not get
            // wrong: it is the page a person opens when typing does nothing.
            let mut rows = vec![
                (
                    "Keyboard layout".to_owned(),
                    shell.regional.locale.keyboard_layout.clone(),
                ),
                ("Pointer and keyboard".to_owned(), shell.mode.token()),
                (
                    "Input service".to_owned(),
                    "evdev · read-only device access".to_owned(),
                ),
            ];
            if facts.input_devices.is_empty() {
                rows.push((
                    "Attached devices".to_owned(),
                    "No input device reported by the kernel".to_owned(),
                ));
            } else {
                for device in &facts.input_devices {
                    rows.push((
                        device.name.clone(),
                        format!("{} · /dev/input/{}", kind_label(&device.kind), device.node),
                    ));
                }
            }
            settings_rows(canvas, t, pane, &rows);
        }
        Page::Network => {
            let network = facts
                .network
                .clone()
                .unwrap_or_else(|| "Unavailable".to_owned());
            let mut rows = vec![("Connection".to_owned(), network)];
            if facts.net_ifaces.is_empty() {
                rows.push((
                    "Interfaces".to_owned(),
                    "No interface reported by the kernel".to_owned(),
                ));
            } else {
                for iface in &facts.net_ifaces {
                    let kind = if iface.wireless {
                        "Wi-Fi"
                    } else if iface.name == "lo" {
                        "Loopback"
                    } else {
                        "Ethernet"
                    };
                    rows.push((
                        iface.name.clone(),
                        format!("{kind} · {} · {}", iface.state, iface.mac),
                    ));
                }
            }
            rows.push((
                "Network changes".to_owned(),
                "Managed by system services".to_owned(),
            ));
            settings_rows(canvas, t, pane, &rows);
        }
        Page::Bluetooth => {
            // Four answers, not two. "Unavailable" for a machine with no radio
            // would hide the fact that the machine is fine and the emulator has
            // no radio to give it; "ready" for a radio with no pairing daemon
            // would promise a pairing that cannot happen.
            let state = facts.bt_state.as_deref().unwrap_or("unknown");
            let verdict = match state {
                "no-adapter" => "No radio in this machine",
                "blocked" => "Radio blocked by firmware or a switch",
                "no-daemon" => "Radio present · no pairing service in this image",
                "ready" => "Ready",
                _ => "Unavailable",
            };
            let mut rows = vec![("Radio".to_owned(), verdict.to_owned())];
            for adapter in &facts.bt_adapters {
                rows.push((adapter.clone(), "Adapter".to_owned()));
            }
            rows.push((
                "Pairing service".to_owned(),
                match facts.bt_daemon {
                    Some(true) => "Installed".to_owned(),
                    Some(false) => "Not installed in this image".to_owned(),
                    None => "Unavailable".to_owned(),
                },
            ));
            settings_rows(canvas, t, pane, &rows);
        }
        Page::Power => {
            let battery = facts.battery_percent.map_or_else(
                || "No battery reading".to_owned(),
                |percent| format!("{percent}%"),
            );
            settings_rows(
                canvas,
                t,
                pane,
                &[
                    ("Battery".to_owned(), battery),
                    (
                        "Power profile".to_owned(),
                        format!(
                            "{} requested; backend unavailable",
                            shell.quick.power.title()
                        ),
                    ),
                    (
                        "Suspend / restart / power off".to_owned(),
                        "Confirmation goes through litric-sysd".to_owned(),
                    ),
                ],
            );
        }
        Page::Region => {}
        Page::Environments => {
            if shell.environments.is_empty() {
                settings_rows(
                    canvas,
                    t,
                    pane,
                    &[(
                        "Environments".to_owned(),
                        "Unavailable: litric-sysd did not answer".to_owned(),
                    )],
                );
            } else {
                let rows = shell.environments.clone();
                for (index, env) in rows.iter().enumerate() {
                    let row = Rect::new(
                        pane.x + t.sp(Space::Lg),
                        y,
                        pane.w - t.sp(Space::Lg) * 2.0,
                        t.px(34.0),
                    );
                    let title = if env.active {
                        format!("{} · {}", env.title, "Active")
                    } else {
                        env.title.clone()
                    };
                    let style = Style::new(t.text_px(TypeStep::Footnote), t.text());
                    font::draw(canvas, &title, row.x, row.y + t.px(22.0), &style);
                    let state = format!("{} · {}", env.state, env.hotkey);
                    let state_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
                    let shown = ui::elide(&state, &state_style, row.w * 0.45);
                    let width = font::measure(&shown, &state_style);
                    font::draw(
                        canvas,
                        &shown,
                        row.right() - width,
                        row.y + t.px(22.0),
                        &state_style,
                    );
                    shell
                        .hotspots_mut()
                        .push(row, Hot::QuickEnvironment(index as u8));
                    y += t.px(36.0);
                }
            }
        }
        Page::Accessibility => {
            let label = format!(
                "{} · {}",
                "Reduce motion",
                if shell.settings.preferences.reduce_motion {
                    "On"
                } else {
                    "Off"
                }
            );
            settings_button(
                canvas,
                shell,
                t,
                pane,
                y,
                SettingRow {
                    key: "reduce_motion",
                    label: &label,
                    hot: Hot::SettingsMotion,
                },
            );
            y += t.px(42.0);
            let note = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            font::draw(
                canvas,
                "High contrast themes are available in Appearance.",
                pane.x + t.sp(Space::Lg),
                y + t.px(16.0),
                &note,
            );
        }
        Page::Sound => {
            // Cards from the kernel, and the server the platform saw. An empty
            // list is a finding — this machine has no sound device — and the
            // page says that rather than "no state is exposed", which reads as
            // a shell that did not look.
            let mut rows = Vec::new();
            if facts.audio_cards.is_empty() {
                rows.push((
                    "Sound cards".to_owned(),
                    "No sound card reported by the kernel".to_owned(),
                ));
            } else {
                for card in &facts.audio_cards {
                    rows.push((card.clone(), "Playback and capture".to_owned()));
                }
            }
            rows.push((
                "Sound server".to_owned(),
                facts
                    .audio_server
                    .clone()
                    .unwrap_or_else(|| "Unavailable".to_owned()),
            ));
            rows.push((
                "Volume and routing".to_owned(),
                "Managed by the sound service".to_owned(),
            ));
            settings_rows(canvas, t, pane, &rows);
        }
        Page::Privacy => settings_rows(
            canvas,
            t,
            pane,
            &[
                (
                    "Browser permissions".to_owned(),
                    "Per-site prompts are not connected yet".to_owned(),
                ),
                (
                    "File access".to_owned(),
                    format!("Persistent store: {}", shell.files_root.display()),
                ),
            ],
        ),
        Page::Accounts => settings_rows(
            canvas,
            t,
            pane,
            &[
                (
                    "Desktop account".to_owned(),
                    facts
                        .auth_account
                        .clone()
                        .or_else(|| facts.session_user.clone())
                        .unwrap_or_else(|| "Unavailable".to_owned()),
                ),
                (
                    "Password verification".to_owned(),
                    format!(
                        "{} / account password: {}",
                        facts.auth_available.as_deref().unwrap_or("Unavailable"),
                        facts.auth_password.as_deref().unwrap_or("unknown")
                    ),
                ),
                (
                    "Lock status".to_owned(),
                    if shell.can_lock() {
                        "Available".to_owned()
                    } else if facts.auth_lock == Some(true) {
                        format!(
                            "Refused: {}",
                            facts.auth_reason.as_deref().unwrap_or("no reason given")
                        )
                    } else {
                        "Unavailable until a verifier and an account password are ready".to_owned()
                    },
                ),
            ],
        ),
        Page::Diagnostics => settings_rows(
            canvas,
            t,
            pane,
            &[
                ("Desktop state".to_owned(), shell.mode.token()),
                (
                    "Facts source".to_owned(),
                    if facts.source.is_empty() {
                        "Unavailable".to_owned()
                    } else {
                        facts.source
                    },
                ),
                ("Text renderer".to_owned(), text_renderer_state()),
            ],
        ),
    }
    if matches!(page, Page::Appearance | Page::Accessibility) {
        let note = match &shell.settings.save {
            SaveState::Idle => None,
            SaveState::Saving => Some(("Saving…", Tone::Neutral)),
            SaveState::Saved => Some(("Saved", Tone::Good)),
            SaveState::Failed(_) => Some(("Save failed", Tone::Bad)),
        };
        if let Some((status, tone)) = note {
            let style = Style::new(t.text_px(TypeStep::Caption), t.tone(tone));
            font::draw(
                canvas,
                status,
                pane.x + t.sp(Space::Lg),
                pane.bottom() - t.px(18.0),
                &style,
            );
        }
        let failed = if let SaveState::Failed(message) = &shell.settings.save {
            Some(message.as_str())
        } else {
            None
        };
        if let Some(message) = failed {
            let style = Style::new(t.text_px(TypeStep::Caption), t.tone(Tone::Bad));
            font::draw(
                canvas,
                &ui::elide(message, &style, pane.w - t.sp(Space::Lg) * 2.0),
                pane.x + t.sp(Space::Lg),
                pane.bottom() - t.px(34.0),
                &style,
            );
        }
        let hint = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(
            canvas,
            "Ctrl+Alt+T theme · Ctrl+A accent · Ctrl+W wallpaper · Ctrl+L language · Ctrl+M motion · Tab next control",
            pane.x + t.sp(Space::Lg),
            pane.bottom() - t.px(8.0),
            &hint,
        );
    }
}

fn wallpaper_name(wallpaper: crate::desktop::Wallpaper) -> &'static str {
    match wallpaper {
        crate::desktop::Wallpaper::System => "Default",
        crate::desktop::Wallpaper::Flat => "Flat",
        crate::desktop::Wallpaper::Glow => "Glow",
    }
}

fn accent_name(accent: litric_design::Accent) -> &'static str {
    match accent {
        litric_design::Accent::Default => "Default",
        litric_design::Accent::Sky => "Sky",
        litric_design::Accent::Violet => "Violet",
        litric_design::Accent::Teal => "Teal",
        litric_design::Accent::Amber => "Amber",
        litric_design::Accent::Rose => "Rose",
        _ => "Default",
    }
}

fn theme_name(theme: litric_design::ThemeKind) -> &'static str {
    match theme {
        litric_design::ThemeKind::Dark => "Dark",
        litric_design::ThemeKind::Light => "Light",
        litric_design::ThemeKind::HighContrastDark => "High contrast dark",
        litric_design::ThemeKind::HighContrastLight => "High contrast light",
        _ => "Dark",
    }
}

/// A window for something that does not exist yet.
///
/// It says exactly that. An empty panel looks like a bug; a panel that claims
/// to be a file manager and lists nothing is worse.
fn draw_not_implemented(canvas: &mut Canvas, shell: &mut Shell, body: Rect, app_id: &str) {
    let t = shell.tokens.clone();
    let (title, detail) = match app_id {
        "com.litric.files" => (
            "File manager",
            "Not implemented. litric-storage resolves paths and litric-trash decides what deleting means; nothing enumerates a directory yet.",
        ),
        "com.litric.browser" => (
            "Litric Browser",
            "Not implemented. The window, the tab model and the privacy policy are next; a rendering engine is integrated, never written from scratch.",
        ),
        "com.litric.terminal" => (
            "Terminal",
            "Not implemented. Needs a pseudo-terminal and a shell process, which needs the process model.",
        ),
        "com.litric.trash" => (
            "Trash",
            "Not implemented in the interface. litric-trash decides collisions, restores and retention; nothing has read the on-disk trash yet.",
        ),
        _ => (
            "Unknown application",
            "No view is registered for this window.",
        ),
    };

    let mut col = ui::Column::new(body, t.sp(Space::Xl), t.sp(Space::Sm));
    col.space(t.px(10.0));
    icons::draw(
        canvas,
        Icon::Info,
        Rect::new(col.x, col.cursor, t.px(28.0), t.px(28.0)),
        t.text_dim(),
        1.0,
    );
    col.space(t.px(40.0));
    font::draw(
        canvas,
        title,
        col.x,
        col.cursor,
        &t.style_bold(TypeStep::Title3),
    );
    col.space(t.px(26.0));

    // Wrap the detail by measuring, so a narrow window does not lose the end
    // of the sentence off its right edge.
    let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let mut line = String::new();
    for word in detail.split_whitespace() {
        let trial = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if font::measure(&trial, &s) > col.width {
            font::draw(canvas, &line, col.x, col.cursor, &s);
            col.space(t.px(18.0));
            line = word.to_owned();
        } else {
            line = trial;
        }
    }
    if !line.is_empty() {
        font::draw(canvas, &line, col.x, col.cursor, &s);
    }
}

// ---------------------------------------------------------------------------
// Overlays
// ---------------------------------------------------------------------------

fn draw_overlay(canvas: &mut Canvas, shell: &mut Shell, overlay: &Overlay) {
    match overlay {
        Overlay::None => {}
        // The launcher is not drawn here: it *is* the dock, expanded. One
        // surface morphs from the pill at the bottom edge into the panel, so
        // there is one thing on screen rather than two that cross-fade -- which
        // is the whole point of the design (`screens/desktop/b-dock-morph`).
        Overlay::Launcher { .. } => {}
        Overlay::Switcher { selected } => draw_switcher(canvas, shell, *selected),
        Overlay::Overview { selected } => draw_overview(canvas, shell, *selected),
        Overlay::QuickSettings => draw_quick_settings(canvas, shell),
        Overlay::Power => draw_power_menu(canvas, shell),
        Overlay::Litro => draw_litro_menu(canvas, shell),
        Overlay::Notifications => draw_notification_list(canvas, shell),
    }
}

/// The applications on the resting dock, as indices into [`App::ALL`]: Files,
/// Browser, Terminal, Settings, Activity, Defender, Environments, and the
/// Trash last, set apart by a separator. A dock that shows four of eight
/// applications and sends the other four through a launcher says that half the
/// machine is somewhere else. [`Hot::DockItem`] keeps meaning an index into
/// `App::ALL`, so nothing that clicks a dock item changes.
const DOCK_PINNED: [usize; 8] = [3, 4, 5, 2, 0, 7, 1, 6];

/// The position in [`DOCK_PINNED`] of the first tile after the separator.
const DOCK_SEPARATE_BEFORE: usize = 7;

/// Distance from the pointer, in logical pixels, at which a dock icon starts
/// to grow.
const DOCK_MAG_RANGE: f32 = 88.0;
/// The most a dock icon grows at the pointer, in logical pixels.
const DOCK_MAG_PEAK: f32 = 28.0;

/// Extra size, in logical pixels, for a dock icon `distance` pixels from the
/// pointer. A smoothstep falloff, `s * s * (3 - 2 * s)` with
/// `s = 1 - |d| / range`, so growth starts and ends without a kink. A linear
/// falloff leaves a visible ridge where the range ends.
#[must_use]
pub fn dock_magnification(distance: f32) -> f32 {
    let s = (1.0 - distance.abs() / DOCK_MAG_RANGE).clamp(0.0, 1.0);
    DOCK_MAG_PEAK * s * s * (3.0 - 2.0 * s)
}

/// The dock, and the launcher it becomes.
///
/// One surface, two states. At rest it is a glass pill at the bottom edge
/// carrying the launcher button, the pinned applications and a search field.
/// The system bar owns the clock, so it appears once. Click the button — or
/// the field — and the same surface grows
/// into the launcher: a title row and a grid of application tiles, with the
/// dock's own row still across the bottom so the
/// icons the user was aiming at have not moved.
///
/// Ported from `screens/desktop/b-dock-morph`, with two changes that are ours:
/// the grid carries this system's eight applications rather than the
/// prototype's eight mock ones, and the shortcut row is a real row of system
/// actions rather than a link to a design page.
///
/// The dock is not an overlay: it is on screen whenever the desktop is, so its
/// hotspots are registered on the same terms every other control's are. A dock
/// that had to be summoned -- and whose clicks had to be withheld until it had
/// finished sliding -- could not show which applications are running, because
/// the indicator was the thing being hidden.
fn draw_dock(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    // The morph: 0 at rest, 1 fully expanded. Its spring has its own target so
    // that closing the launcher animates back down instead of snapping.
    let morph = shell.anims().launcher_morph.value().clamp(0.0, 1.0);
    let (query, selected) = match &shell.overlay {
        Overlay::Launcher { query, selected } => (query.clone(), *selected),
        _ => (String::new(), 0usize),
    };
    let searching = !query.trim().is_empty();

    // The bar is a strip, not a shelf: 30 pixels tall -- a little over a third
    // of the 76 it started at, and 1.6 times shorter than the 48-pixel version
    // before it -- and stretched out to the width of the launcher so the row
    // does not sit in a small pill in the middle of a 1920-wide screen. What is
    // left is the same controls, closer to the edge they belong to.
    let pad = t.px(chrome::DOCK_PAD);
    let item = t.px(chrome::DOCK_ICON);
    let gap = t.px(chrome::DOCK_GAP);
    let sep = t.px(chrome::DOCK_SEPARATOR);
    let float = t.px(chrome::DOCK_FLOAT);
    let apps = &crate::shortcuts::App::ALL;
    let pinned: Vec<(usize, App)> = DOCK_PINNED
        .iter()
        .filter_map(|i| apps.get(*i).map(|a| (*i, *a)))
        .collect();

    // The resting pill, measured from what it holds. Time already lives in
    // the system bar, so the dock has more room for its useful search field.
    let bubble_w = t.px(chrome::DOCK_SEARCH_W);
    let measured = pad * 2.0
        + item
        + t.px(10.0)
        + item * pinned.len() as f32
        + gap * (pinned.len().saturating_sub(1)) as f32
        + sep * 2.0
        + gap
        + bubble_w;
    let dock_h = t.px(chrome::DOCK_H);

    // The expanded panel.
    let cols = 4usize;
    let rows = apps
        .len()
        .max(1)
        .saturating_add(cols.saturating_sub(1))
        .checked_div(cols)
        .unwrap_or(1);
    let tile = t.px(176.0);
    let tile_h = t.px(136.0);
    let panel_w = t.px(760.0).min(screen.w - t.px(40.0));
    // Stretched, not merely measured: a bar sized only by what it holds is a
    // pill a third of the screen wide at 1920. It is as wide as the launcher it
    // becomes -- never past the screen's margins, never narrower than its
    // contents -- so the field and the icons breathe, and the morph changes
    // height alone instead of resizing the surface under the user's pointer.
    let dock_w = measured.min(screen.w - t.px(40.0));
    // The row's left edge, fixed by the *resting* width. The surface grows
    // outwards from the middle when it becomes the launcher, so a row anchored
    // here does not move: the tile the pointer was aiming at stays under it.
    let rest_x = (screen.w - dock_w) / 2.0;
    // The panel grows from the bottom edge, so a search that matches two
    // things is a small panel rather than a grid-sized one with a hole in it:
    // shrinking it moves the top down and leaves the field where it was.
    let panel_h = if searching {
        let shown = crate::seek::search(shell, &query).len().clamp(1, 6);
        t.px(48.0) + t.px(66.0) * shown as f32 + dock_h + t.px(24.0)
    } else {
        t.px(72.0)          // title row
            + tile_h * rows as f32
            + dock_h
            + t.px(24.0)
    };

    let w = dock_w + (panel_w - dock_w) * morph;
    let h = dock_h + (panel_h - dock_h) * morph;
    // At rest the strip is a full pill: half its own height. It settles to the
    // launcher's 36 as the panel grows.
    let rest_radius = t.px(chrome::DOCK_RADIUS);
    let radius = rest_radius + (t.px(36.0) - rest_radius) * morph;
    let x = (screen.w - w) / 2.0;
    // Bottom-anchored, 24 logical px above the edge. The panel is never scaled
    // by a spring: that pushed titles and text below the framebuffer once, and
    // the dock is a surface the user reads, not an animation.
    let y = screen.h - float - h;
    let panel = Rect::new(x, y, w, h);

    // No scrim. A full-screen dim would make every frame of the morph a
    // whole-screen repaint -- the dim's alpha changes with the spring, so the
    // desktop under it has to be repainted, and that is 2 million pixels of
    // wallpaper and glass per frame for a panel that covers a fifth of the
    // screen. That was the stutter. The glass panel is opaque enough to read
    // on its own, and the dock is not a modal: it dims nothing while closed, so
    // it does not need to dim the desktop on its way open either.

    // The rectangle this surface occupies fully open, published for the frame
    // loop: while the morph runs, that is the only part of the screen that can
    // have changed, so that is all it repaints. It is the *smallest* rectangle
    // that contains every state of the morph -- the wider of the strip and the
    // panel, and the taller of the browsing panel and the results panel -- so
    // a frame that shrinks the panel still restores the pixels around it.
    let browsing_h = t.px(72.0) + tile_h * rows as f32 + dock_h + t.px(24.0);
    let settled_w = dock_w.max(panel_w);
    let settled_h = panel_h.max(browsing_h);
    shell.set_dock_settled(Rect::new(
        (screen.w - settled_w) / 2.0,
        screen.h - float - settled_h,
        settled_w,
        settled_h,
    ));

    // The shadow is drawn only once the panel has arrived. Mid-morph it is a
    // forty-pixel blur of a rectangle that moves every frame -- several
    // million pixel visits -- and a shadow the eye cannot resolve on a surface
    // that is still growing is not worth a frame.
    if morph >= 0.999 || morph <= 0.001 {
        canvas.soft_shadow(panel, radius, t.px(26.0), t.px(8.0), 0.45);
    }
    // Glass, and genuinely so: the wallpaper's wave shows through the strip. It
    // is cheaper here than it looks -- the scene carries a pre-blurred backdrop
    // made once, so this copies a 30-pixel band out of it instead of blurring
    // live pixels, which is why the bar can be translucent without costing the
    // frame rate. The panel darkens as it grows, so the grid stays readable.
    let veil = if morph <= 0.01 { 58 } else { 82 };
    crate::desktop::glass_at(canvas, &t.theme, panel, radius, veil);

    let saved = canvas.push_clip(panel);
    // The dock is not an overlay: nothing about it is waiting for a reveal, so
    // every control it draws is a control the user can press.

    // ---- the expanded layers, which fade in with the morph ----------------
    if morph > 0.01 {
        let lay = morph;
        // While a query is typed the panel is only results and the field, as in
        // the design's search state: no title, no lock pill, no shortcut row.
        let browsing = !searching;
        if browsing && panel.y + t.px(60.0) < screen.h {
            // Title row: what this is, and the one shortcut that belongs at
            // the top right of it.
            let title = Style::new(t.text_px(TypeStep::Title3), with_alpha(t.text(), lay));
            font::draw(
                canvas,
                "All applications",
                panel.x + t.px(28.0),
                panel.y + t.px(44.0),
                &title,
            );
            let lock_style = Style::new(t.text_px(TypeStep::Footnote), with_alpha(t.text(), lay));
            let lock_w = font::measure("Lock screen", &lock_style) + t.px(50.0);
            let lock = Rect::new(
                panel.right() - t.px(28.0) - lock_w,
                panel.y + t.px(22.0),
                lock_w,
                t.px(36.0),
            );
            let lock_hover = shell.is_highlighted(Hot::DockLock);
            canvas.fill_round_rect(
                lock,
                lock.h / 2.0,
                with_alpha(rgb(0xFFFFFF), if lock_hover { 0.16 } else { 0.09 }),
            );
            icons::draw(
                canvas,
                Icon::Lock,
                Rect::new(
                    lock.x + t.px(14.0),
                    lock.y + t.px(10.0),
                    t.px(16.0),
                    t.px(16.0),
                ),
                with_alpha(t.text(), 0.9 * lay),
                1.0,
            );
            font::draw(
                canvas,
                "Lock screen",
                lock.x + t.px(36.0),
                lock.y + t.px(23.0),
                &lock_style,
            );
            shell.hotspots_mut().push(lock, Hot::DockLock);
        }

        // The grid, or the results of what has been typed.
        let body_top = if searching { t.px(20.0) } else { t.px(72.0) };
        let body = Rect::new(
            panel.x + t.px(28.0),
            panel.y + body_top,
            panel.w - t.px(56.0),
            panel.bottom() - dock_h - body_top,
        );
        if body.y < screen.h - t.px(16.0) && searching {
            launcher_rows(canvas, shell, &t, body, &query, selected, lay);
        } else if body.y < screen.h - t.px(16.0) {
            launcher_tiles(canvas, shell, &t, body, cols, tile, tile_h, lay);
        }
    }

    // ---- the dock row, always at the bottom of the same surface ----------
    let row_y = panel.bottom() - dock_h;
    let tile_y = row_y + (dock_h - item) / 2.0;
    let button = Rect::new(rest_x + pad, tile_y, item, item);
    let button_hover = shell.is_highlighted(Hot::DockToggle);
    // The launcher is the pale tile in the set: every application has a colour
    // of its own, so the way into all of them is the one that has none. It
    // rises on hover like the rest.
    let button_r = if button_hover {
        Rect::new(
            button.x,
            button.y - t.px(chrome::DOCK_LIFT),
            button.w,
            button.h,
        )
    } else {
        button
    };
    let grown = |r: Rect, hovered: bool| {
        if hovered {
            let k = item * 0.14;
            Rect::new(r.x - k / 2.0, r.y - k, r.w + k, r.h + k)
        } else {
            r
        }
    };
    appicons::draw(
        canvas,
        appicons::Slot::Launcher,
        grown(button_r, button_hover),
        1.0,
    );
    shell.hotspots_mut().push(button, Hot::DockToggle);

    // Pinned tiles: shown in both states -- the tiles the user was aiming at
    // must not move when the surface grows -- and folded away for a search,
    // which is the one state that needs the row for the query.
    //
    // No magnification. Re-cutting every icon on every pointer move is work this
    // strip cannot afford, and a hover lift of a few pixels says "this one" at
    // the cost of one tile.
    let mut ix = button.right() + t.px(10.0);
    let mut tip: Option<(Rect, &'static str)> = None;
    for (n, (i, app)) in pinned.iter().map(|(i, a)| (*i, a)).enumerate() {
        if searching {
            break;
        }
        if n == DOCK_SEPARATE_BEFORE {
            // The Trash is not an application the user runs; it is where things
            // go. A hairline between the two groups says so without a word.
            canvas.fill_round_rect(
                Rect::new(
                    ix + sep / 2.0 - 0.5,
                    tile_y + t.px(4.0),
                    1.0,
                    item - t.px(8.0),
                ),
                0.5,
                with_alpha(t.text(), 0.18),
            );
            ix += sep;
        }
        let hot = Hot::DockItem(u8::try_from(i).unwrap_or(u8::MAX));
        let home = Rect::new(ix, tile_y, item, item);
        let hovered = shell.is_highlighted(hot);
        // The tile rises; its hotspot does not. A hotspot that rose with it
        // would leave the pointer below the tile's edge the moment it lifted,
        // un-hover it, and the tile would flicker between the two.
        let r = if hovered {
            Rect::new(home.x, home.y - t.px(chrome::DOCK_LIFT), home.w, home.h)
        } else {
            home
        };
        if hovered {
            tip = Some((r, app.title()));
        }
        // Original artwork, scaled once and kept: see `appicons`.
        appicons::draw(canvas, appicons::Slot::App(*app), grown(r, hovered), 1.0);
        // Running indicator: one dot under a running app, in paper white, on
        // the strip's own padding below the tile.
        if shell.wm.all().iter().any(|w| w.app_id == app.app_id()) {
            canvas.fill_circle(
                home.center().0,
                home.bottom() + t.px(4.0),
                t.px(2.0),
                with_alpha(t.text(), 0.9),
            );
        }
        shell.hotspots_mut().push(home.inflate(t.px(2.0)), hot);
        ix += item + gap;
    }

    // The search field. With the tiles folded away it starts where they would
    // have, which is the room the query needs.
    let bubble_x = if searching {
        button.right() + t.px(10.0)
    } else {
        ix + sep - gap
    };
    if !searching {
        canvas.fill_round_rect(
            Rect::new(
                bubble_x - sep / 2.0 - 0.5,
                tile_y + t.px(4.0),
                1.0,
                item - t.px(8.0),
            ),
            0.5,
            with_alpha(t.text(), 0.18),
        );
    }
    let bubble = Rect::new(
        bubble_x,
        tile_y,
        if searching {
            (panel.right() - pad - bubble_x).max(t.px(80.0))
        } else {
            bubble_w
        },
        item,
    );
    let bubble_state = if shell.is_highlighted(Hot::DockSearch) {
        State::Hover
    } else {
        State::Normal
    };
    canvas.fill_round_rect(
        bubble,
        bubble.h / 2.0,
        with_alpha(
            rgb(0xFFFFFF),
            if searching {
                0.14
            } else {
                0.09 + 0.05 * f32::from(bubble_state == State::Hover)
            },
        ),
    );
    canvas.stroke_round_rect(
        bubble,
        bubble.h / 2.0,
        1.0,
        if searching {
            t.accent()
        } else {
            with_alpha(rgb(0xFFFFFF), 0.14)
        },
    );
    let magnifier = Rect::new(
        bubble.x + t.px(8.0),
        bubble.center().1 - t.px(7.0),
        t.px(14.0),
        t.px(14.0),
    );
    icons::draw(
        canvas,
        Icon::Search,
        magnifier,
        if searching {
            t.accent()
        } else {
            with_alpha(t.text(), 0.7)
        },
        1.0,
    );
    if searching {
        // The query, with a caret: the launcher's search field is the shell's,
        // not a text widget's, so it draws its own insertion point.
        let style = Style::new(t.px(13.0), t.text());
        let shown = ui::elide(&query, &style, bubble.w - t.px(64.0));
        font::draw(
            canvas,
            &shown,
            magnifier.right() + t.px(8.0),
            bubble.center().1 + style.size * 0.36,
            &style,
        );
        let caret = magnifier.right() + t.px(8.0) + font::measure(&shown, &style) + t.px(1.0);
        canvas.fill_round_rect(
            Rect::new(caret, bubble.center().1 - t.px(7.0), t.px(1.5), t.px(14.0)),
            t.px(0.75),
            t.accent(),
        );
        let close = Rect::new(
            bubble.right() - t.px(24.0),
            bubble.y + t.px(2.0),
            t.px(18.0),
            t.px(18.0),
        );

        canvas.fill_circle(
            close.center().0,
            close.center().1,
            close.w / 2.0,
            with_alpha(rgb(0xFFFFFF), 0.10),
        );
        icons::draw(
            canvas,
            Icon::Close,
            close.inflate(-t.px(5.0)),
            t.text(),
            1.0,
        );
        shell.hotspots_mut().push(close, Hot::DockCollapse);
    } else {
        let style = Style::new(t.px(13.0), with_alpha(t.text(), 0.7));
        font::draw(
            canvas,
            "Search",
            magnifier.right() + t.px(8.0),
            bubble.center().1 + style.size * 0.36,
            &style,
        );
    }
    if !searching {
        shell.hotspots_mut().push(bubble, Hot::DockSearch);
    }

    canvas.set_clip(saved);

    // The name of the tile under the pointer, drawn after the clip is lifted:
    // it sits above the strip, outside the rectangle the strip clips to, and
    // drawn inside that clip it was cut away entirely.
    if let Some((r, title)) = tip {
        let style = Style::new(t.text_px(TypeStep::Caption), t.text());
        let tw = font::measure(title, &style);
        let lr = Rect::new(
            r.center().0 - tw / 2.0 - t.px(9.0),
            panel.y - t.px(30.0),
            tw + t.px(18.0),
            t.px(22.0),
        );
        ui::surface(canvas, &t, lr, t.radius(Radius::Sm), Material::Elevated);
        font::draw(
            canvas,
            title,
            lr.x + t.px(9.0),
            lr.y + lr.h / 2.0 + style.size * 0.36,
            &style,
        );
    }
}

/// The grid of application tiles: a rounded square carrying the application's
/// own colour, and its name under it, as in `screens/desktop/b-dock-morph`.
#[allow(clippy::too_many_arguments)]
fn launcher_tiles(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &crate::ui::Tokens,
    body: Rect,
    cols: usize,
    tile: f32,
    tile_h: f32,
    opacity: f32,
) {
    let x0 = body.x + (body.w - tile * cols as f32) / 2.0;
    for (i, app) in App::ALL.iter().enumerate() {
        let col = i.checked_rem(cols).unwrap_or(0);
        let row = i.checked_div(cols).unwrap_or(0);
        let cell = Rect::new(
            x0 + (col as f32) * tile,
            body.y + (row as f32) * tile_h,
            tile,
            tile_h,
        );
        // During the opening spring, the last row may not have entered the
        // panel yet. Wait until the full tile and its label fit before drawing
        // them; clipping pixels while retaining out-of-frame text is not a
        // usable transition.
        if cell.bottom() > body.bottom() {
            continue;
        }
        let hot = Hot::LauncherItem(i);
        let hovered = shell.is_highlighted(hot);
        // The same artwork as the dock, larger: one icon per application
        // wherever it appears. It rises a little under the pointer.
        let chip = Rect::new(
            cell.center().0 - t.px(41.0),
            cell.y + t.px(if hovered { 0.0 } else { 4.0 }),
            t.px(82.0),
            t.px(82.0),
        );
        appicons::draw(canvas, appicons::Slot::App(*app), chip, opacity);
        let style = Style::new(
            t.text_px(TypeStep::Footnote),
            with_alpha(t.text(), 0.9 * opacity),
        );
        let name = ui::elide(app.title(), &style, cell.w - t.px(12.0));
        font::draw(
            canvas,
            &name,
            cell.center().0 - font::measure(&name, &style) / 2.0,
            cell.y + t.px(86.0) + t.px(24.0),
            &style,
        );
        shell.hotspots_mut().push(cell, hot);
    }
}

/// The launcher's results, when something has been typed: a small heading, then
/// one row per match — the application's tile, its name, what it is, and on the
/// selected row the key that opens it.
#[allow(clippy::too_many_arguments)]
fn launcher_rows(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &crate::ui::Tokens,
    body: Rect,
    query: &str,
    selected: usize,
    opacity: f32,
) {
    let heading = Style::new(
        t.text_px(TypeStep::Caption),
        with_alpha(t.text(), 0.6 * opacity),
    );
    font::draw(
        canvas,
        "Search results",
        body.x + t.px(8.0),
        body.y + t.px(14.0),
        &heading,
    );
    let body = Rect::new(body.x, body.y + t.px(24.0), body.w, body.h - t.px(24.0));
    let results = crate::seek::search(shell, query);
    if results.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), with_alpha(t.text(), 0.6));
        font::draw(
            canvas,
            "Nothing matches that",
            body.x + t.px(8.0),
            body.y + t.px(32.0),
            &s,
        );
        return;
    }
    let row_h = t.px(62.0);
    let mut y = body.y;
    for (i, result) in results.iter().enumerate() {
        if y + row_h > body.bottom() + t.px(4.0) {
            break;
        }
        let row = Rect::new(body.x, y, body.w, row_h);
        let hot = Hot::LauncherItem(i);
        let on = i == selected;
        let hovered = shell.is_highlighted(hot);
        if on || hovered {
            canvas.fill_round_rect(
                row,
                t.px(16.0),
                with_alpha(rgb(0xFFFFFF), if on { 0.08 } else { 0.05 }),
            );
        }
        let tile = Rect::new(
            row.x + t.px(14.0),
            row.y + t.px(9.0),
            t.px(44.0),
            t.px(44.0),
        );
        canvas.fill_round_rect(tile, t.px(12.0), with_alpha(rgb(0xFFFFFF), 0.09));
        // One icon per kind of result, so a row says what it will do before it
        // is read: an application, a settings page, a window, or an action.
        let icon = match result.hit {
            crate::seek::Hit::App(app) => icon_for(app),
            crate::seek::Hit::Setting(page) => settings_icon(page),
            crate::seek::Hit::SettingRow(entry) => settings_icon(entry.page),
            crate::seek::Hit::Window(_) => Icon::Window,
            crate::seek::Hit::Action(_) => Icon::Search,
        };
        icons::draw(canvas, icon, tile.inflate(-t.px(11.0)), t.accent(), 1.0);
        let name_style = Style::new(t.text_px(TypeStep::Body), t.text());
        let name_x = tile.right() + t.px(14.0);
        font::draw(
            canvas,
            &result.title,
            name_x,
            row.center().1 + t.px(6.0),
            &name_style,
        );
        let sub = Style::new(t.text_px(TypeStep::Footnote), with_alpha(t.text(), 0.55));
        font::draw(
            canvas,
            &result.detail,
            name_x + font::measure(&result.title, &name_style) + t.px(14.0),
            row.center().1 + t.px(6.0),
            &sub,
        );
        // The key that opens the selected row. A row the pointer is on shows
        // its own shortcut instead, when it has one.
        let trailing = if on {
            Some("Enter".to_owned())
        } else {
            result.key.clone()
        };
        if let Some(label) = trailing {
            let w = font::measure(&label, &sub);
            font::draw(
                canvas,
                &label,
                row.right() - t.px(16.0) - w,
                row.center().1 + t.px(6.0),
                &sub,
            );
        }
        shell.hotspots_mut().push(row, hot);
        y += row_h + t.px(4.0);
    }
}

/// The Terminal: a real command line against the guest OS.
fn draw_terminal(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let term = shell.terminal.clone();
    canvas.fill_round_rect(body, 0.0, with_alpha(rgb(0x000000), 0.55));
    let line_h = t.px(17.0);
    let toolbar = Rect::new(body.x, body.y, body.w, t.px(32.0));
    canvas.fill_round_rect(toolbar, 0.0, with_alpha(rgb(0xFFFFFF), 0.045));
    let s_toolbar = Style::bold(t.text_px(TypeStep::Footnote), t.text());
    let title_x = toolbar.x + t.px(10.0);
    font::draw(
        canvas,
        "Command session",
        title_x,
        toolbar.y + t.px(20.0),
        &s_toolbar,
    );
    let status = if term.busy {
        "Process running"
    } else {
        "Ready"
    };
    let toolbar_button_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let new_w = (font::measure("New session", &toolbar_button_style) + t.px(20.0)).max(t.px(86.0));
    let clear_w = (font::measure("Clear", &toolbar_button_style) + t.px(20.0)).max(t.px(64.0));
    let new_session = Rect::new(
        toolbar.right() - new_w - t.px(8.0),
        toolbar.y + t.px(4.0),
        new_w,
        t.px(24.0),
    );
    let clear = Rect::new(
        new_session.x - clear_w - t.px(6.0),
        toolbar.y + t.px(4.0),
        clear_w,
        t.px(24.0),
    );
    let s_status = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let status_x = title_x + font::measure("Command session", &s_toolbar) + t.px(12.0);
    let status_room = (clear.x - t.px(10.0) - status_x).max(0.0);
    if status_room > t.px(12.0) {
        font::draw(
            canvas,
            &ui::elide(status, &s_status, status_room),
            status_x,
            toolbar.y + t.px(19.0),
            &s_status,
        );
    }
    ui::button(canvas, &t, clear, "Clear", ui::State::Normal, Tone::Neutral);
    shell.hotspots_mut().push(clear, Hot::TerminalClear);
    ui::button(
        canvas,
        &t,
        new_session,
        "New session",
        if term.busy {
            ui::State::Disabled
        } else {
            ui::State::Normal
        },
        Tone::Accent,
    );
    if !term.busy {
        shell
            .hotspots_mut()
            .push(new_session, Hot::TerminalNewSession);
    }

    let mut y = toolbar.bottom() + t.px(8.0);
    if let Some(note) = &term.note {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.tone(Tone::Warning));
        font::draw(canvas, note, body.x + t.px(10.0), y + line_h, &s);
        y += line_h * 1.6;
    }
    // Scrollback, newest at the bottom, only as many as fit.
    let prompt_row = 1usize;
    let visible = ((body.bottom() - y) / line_h) as usize;
    let visible = visible.saturating_sub(prompt_row).max(1);
    let start = term.lines.len().saturating_sub(visible);
    let s_out = Style::new(t.text_px(TypeStep::Footnote), t.text_dim()).monospace();
    for line in term.lines.iter().skip(start) {
        font::draw(canvas, line, body.x + t.px(10.0), y + line_h, &s_out);
        y += line_h;
    }
    // Prompt + input, with the caret when the field holds the keyboard.
    let prompt = if term.busy { "… " } else { "~ $ " };
    let s_in = Style::new(t.text_px(TypeStep::Body), t.text()).monospace();
    let in_x = body.x + t.px(10.0);
    let in_y = body.bottom() - t.px(10.0);
    font::draw(canvas, prompt, in_x, in_y, &s_in);
    let text_x = in_x + font::measure(prompt, &s_in);
    font::draw(canvas, &term.input, text_x, in_y, &s_in);
    if term.focused {
        let cx = text_x + font::measure(&term.input, &s_in);
        canvas.stroke_line(
            cx + 1.0,
            in_y - s_in.size,
            cx + 1.0,
            in_y + 3.0,
            1.0,
            t.accent(),
        );
    }
    let row = Rect::new(body.x, in_y - t.px(20.0), body.w, t.px(26.0));
    shell.hotspots_mut().push(row, Hot::TerminalInput);
}

/// The icon for an application.
#[must_use]
pub const fn icon_for(app: App) -> Icon {
    match app {
        App::Activity => Icon::Activity,
        App::Environments => Icon::Environment,
        App::Settings => Icon::Settings,
        App::Files => Icon::Files,
        App::Browser => Icon::Browser,
        App::Terminal => Icon::Terminal,
        App::Trash => Icon::Trash,
        App::Defender => Icon::Shield,
    }
}

/// The Litric Defender window: scan button, progress, verdict table.
fn draw_defender(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let d = shell.defender.app.clone();
    let mut col = ui::Column::new(body, t.sp(Space::Lg), t.sp(Space::Sm));

    // Header: shield + status sentence.
    let head = col.row(t.px(40.0));
    let (icon, tone) = match d.state {
        crate::apps::ScanState::Done { threats, .. } if threats > 0 => (Icon::Warning, Tone::Bad),
        crate::apps::ScanState::Scanning { .. } => (Icon::Shield, Tone::Accent),
        crate::apps::ScanState::Done { .. } => (Icon::ShieldCheck, Tone::Good),
        crate::apps::ScanState::Idle => (Icon::Shield, Tone::Neutral),
    };
    icons::draw(
        canvas,
        icon,
        Rect::new(head.x, head.y + t.px(6.0), t.px(28.0), t.px(28.0)),
        t.tone(tone),
        1.1,
    );
    let s_head = t.style_bold(TypeStep::Title3);
    // Title and status are 26 pixels apart, not 14. At 14 the status line sat
    // inside the title's own band -- 196x7 pixels of "Definitions: ..." drawn
    // over "Litric Defender" -- which reads as a rendering fault rather than as
    // a layout mistake.
    font::draw(
        canvas,
        "Litric Defender",
        head.x + t.px(40.0),
        head.y + t.px(18.0),
        &s_head,
    );
    let s_status = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let status = if d.message.is_empty() {
        "On-demand scan · real-time protection unavailable".to_owned()
    } else {
        d.message.clone()
    };
    font::draw(
        canvas,
        &ui::elide(&status, &s_status, head.w - t.px(48.0)),
        head.x + t.px(40.0),
        head.y + t.px(44.0),
        &s_status,
    );

    // Scan button. While a scan runs it is a Loading control: motion
    // marker, no click that could double-start the engine.
    let row = col.row(t.px(30.0));
    let (scan_label, scan_state) = if shell.defender.running {
        ("Scanning", ui::State::Loading)
    } else {
        ("Scan now", ui::State::Normal)
    };
    let bw = t.px(110.0);
    let br = Rect::new(row.x, row.y, bw, row.h);
    ui::button(canvas, &t, br, scan_label, scan_state, Tone::Accent);
    if !shell.defender.running {
        shell
            .hotspots_mut()
            .push(br, Hot::MenuRow(crate::apps::MenuAction::ScanNow));
    }

    // Progress bar while scanning.
    if let crate::apps::ScanState::Scanning { done, total } = d.state {
        let frac = if total == 0 {
            0.0
        } else {
            done as f32 / total as f32
        };
        let bar = col.row(t.px(6.0));
        ui::progress(
            canvas,
            &t,
            Rect::new(bar.x, bar.y, bar.w, bar.h),
            frac,
            Tone::Accent,
        );
        let s = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let row2 = col.row(t.px(16.0));
        font::draw(
            canvas,
            &format!("{done} of {total}"),
            row2.x,
            row2.y + t.px(11.0),
            &s,
        );
    }

    // Results table: detections always; clean files as a bounded sample.
    ui::section_label(canvas, &t, (col.x, col.cursor + t.px(10.0)), "RESULTS");
    col.space(t.px(16.0));
    let saved = canvas.push_clip(Rect::new(
        body.x,
        col.cursor,
        body.w,
        body.bottom() - col.cursor,
    ));
    for (i, r) in d.rows.iter().enumerate().take(40) {
        let row = col.row(t.px(24.0));
        if col.cursor > body.bottom() {
            break;
        }
        let (icon, tone) = if r.detected {
            (Icon::Warning, Tone::Bad)
        } else {
            (Icon::Check, Tone::Good)
        };
        icons::draw(
            canvas,
            icon,
            Rect::new(row.x, row.y + t.px(5.0), t.px(14.0), t.px(14.0)),
            t.tone(tone),
            1.0,
        );
        let s = Style::new(
            t.text_px(TypeStep::Caption),
            if r.detected { t.text() } else { t.text_dim() },
        );
        let name_w = row.w * 0.55;
        font::draw(
            canvas,
            &ui::elide(&r.path, &s, name_w),
            row.x + t.px(22.0),
            row.y + t.px(15.0),
            &s,
        );
        let vs = Style::new(t.text_px(TypeStep::Caption), t.tone(tone));
        let vw = font::measure(&r.verdict, &vs);
        font::draw(
            canvas,
            &r.verdict,
            row.right() - vw,
            row.y + t.px(15.0),
            &vs,
        );
        let _ = i;
    }
    canvas.set_clip(saved);
}

/// The Trash window: entries, restore, empty.
fn draw_trash(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    refresh_trash_view(shell);
    let t = shell.tokens.clone();
    let app = shell.trash.clone();
    let mut col = ui::Column::new(body, t.sp(Space::Md), t.sp(Space::Xs));

    let head = col.row(t.px(30.0));
    let s_head = t.style_bold(TypeStep::Title3);
    // The count is placed after the title, not at a guessed offset: the title is
    // set in the face's own width, so a fixed 80 pixels put "0 item(s)" under
    // the "h" of "Trash" as soon as the typeface's side bearings made the word
    // wider than that guess. A gap belongs between the two, so the gap is what
    // is asked for.
    let title_end = font::draw(canvas, "Trash", head.x, head.y + t.px(18.0), &s_head);
    let s_count = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    font::draw(
        canvas,
        &format!("{} / {} items", app.visible_indices().len(), app.count()),
        title_end + t.px(10.0),
        head.y + t.px(18.0),
        &s_count,
    );

    let toolbar = col.row(t.px(30.0));
    let has_sel = app.has_visible_selection();
    let reserved = if app.confirm.is_some() {
        t.px(188.0)
    } else if has_sel {
        t.px(252.0)
    } else {
        t.px(76.0)
    };
    let search_w = (toolbar.w - reserved - t.px(12.0))
        .max(t.px(96.0))
        .min(t.px(260.0));
    let search = Rect::new(toolbar.x, toolbar.y, search_w, toolbar.h);
    ui::text_field(
        canvas,
        &t,
        search,
        &app.filter,
        "Search trash",
        shell.is_highlighted(Hot::TrashFilter),
    );
    shell.hotspots_mut().push(search, Hot::TrashFilter);

    if let Some(pending) = &app.confirm {
        let label = match pending {
            crate::apps::TrashConfirmation::Empty => "Empty permanently?",
            crate::apps::TrashConfirmation::Purge(_) => "Delete forever?",
        };
        let confirm_w = t.px(112.0);
        let cancel_w = t.px(70.0);
        let cancel = Rect::new(toolbar.right() - cancel_w, toolbar.y, cancel_w, toolbar.h);
        let confirm = Rect::new(
            cancel.x - t.px(6.0) - confirm_w,
            toolbar.y,
            confirm_w,
            toolbar.h,
        );
        ui::button(canvas, &t, confirm, label, ui::State::Normal, Tone::Bad);
        ui::button(
            canvas,
            &t,
            cancel,
            "Cancel",
            ui::State::Normal,
            Tone::Neutral,
        );
        shell.hotspots_mut().push(confirm, Hot::TrashConfirm);
        shell.hotspots_mut().push(cancel, Hot::TrashCancel);
    } else {
        let empty_w = t.px(76.0);
        let empty = Rect::new(toolbar.right() - empty_w, toolbar.y, empty_w, toolbar.h);
        ui::button(canvas, &t, empty, "Empty", ui::State::Normal, Tone::Bad);
        shell.hotspots_mut().push(empty, Hot::TrashEmpty);
        if has_sel {
            let restore_w = t.px(76.0);
            let purge_w = t.px(88.0);
            let restore = Rect::new(
                empty.x - t.px(6.0) - restore_w,
                toolbar.y,
                restore_w,
                toolbar.h,
            );
            let purge = Rect::new(
                restore.x - t.px(6.0) - purge_w,
                toolbar.y,
                purge_w,
                toolbar.h,
            );
            ui::button(canvas, &t, purge, "Delete", ui::State::Normal, Tone::Bad);
            ui::button(
                canvas,
                &t,
                restore,
                "Restore",
                ui::State::Normal,
                Tone::Neutral,
            );
            shell.hotspots_mut().push(purge, Hot::TrashPurge);
            shell.hotspots_mut().push(restore, Hot::TrashRestore);
        }
    }

    if let Some(err) = &app.error {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.tone(Tone::Bad));
        font::draw(canvas, err, col.x, col.cursor + t.px(14.0), &s);
        return;
    }
    let visible = app.visible_indices();
    if visible.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        let message = if app.entries.is_empty() {
            "Trash is empty"
        } else {
            "No items match this search"
        };
        font::draw(canvas, message, col.x, col.cursor + t.px(14.0), &s);
        return;
    }
    let saved = canvas.push_clip(Rect::new(
        body.x,
        col.cursor,
        body.w,
        body.bottom() - col.cursor,
    ));
    for i in visible {
        let Some((name, deleted, origin, is_dir)) = app.entries.get(i) else {
            continue;
        };
        if col.cursor + t.px(26.0) > body.bottom() {
            break;
        }
        let row = col.row(t.px(26.0));
        let hot = Hot::TrashRow(i);
        let selected = app.selected == Some(i);
        if selected {
            canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.text(), 0.08));
        }
        let icon = if *is_dir { Icon::Files } else { Icon::Window };
        icons::draw(
            canvas,
            icon,
            Rect::new(row.x + t.px(2.0), row.y + t.px(6.0), t.px(14.0), t.px(14.0)),
            t.text_dim(),
            1.0,
        );
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text());
        font::draw(
            canvas,
            &ui::elide(name, &s, row.w * 0.4),
            row.x + t.px(24.0),
            row.y + t.px(16.0),
            &s,
        );
        let s_dim = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let origin_x = row.x + row.w * 0.5;
        font::draw(
            canvas,
            &ui::elide(&format!("from {origin}"), &s_dim, row.w * 0.3),
            origin_x,
            row.y + t.px(16.0),
            &s_dim,
        );
        let dw = font::measure(deleted, &s_dim);
        font::draw(
            canvas,
            deleted,
            row.right() - dw,
            row.y + t.px(16.0),
            &s_dim,
        );
        shell.hotspots_mut().push(row, hot);
    }
    canvas.set_clip(saved);
}

/// Refresh the Trash listing when its window is drawn and the model is stale.
fn refresh_trash_view(shell: &mut Shell) {
    if !shell.trash.listed {
        shell.refresh_trash();
    }
}

/// Mission Control: every window on every workspace, as cards over a dimmed
/// desktop.
///
/// The card is a map of a window, not a screenshot of one: the software canvas
/// has no render-to-texture path, and drawing five live windows at once to
/// shrink them would cost more than a frame for a screen that is, by
/// definition, waiting for the user to choose. What the card does carry is
/// everything a person needs to choose: which application, what it is called,
/// which workspace it is on, and whether it is minimized.
fn draw_overview(canvas: &mut Canvas, shell: &mut Shell, selected: usize) {
    let t = shell.tokens.clone();
    let wm_screen = shell.wm.screen();
    let screen = Rect::new(wm_screen.x, wm_screen.y, wm_screen.w, wm_screen.h);
    // Mission Control is a place, but it arrives and leaves like a panel: the
    // dim follows the same spring, so closing it fades the desktop back in
    // instead of cutting to it.
    let reveal = shell.anims().overview.value();
    if reveal <= 0.01 {
        return;
    }

    // The desktop stays visible under the dim, so the overview reads as a layer
    // over the session rather than a different screen.
    canvas.fill_round_rect(
        screen,
        0.0,
        with_alpha(rgb(0x000000), 0.74 * reveal.clamp(0.0, 1.0)),
    );
    shell.hotspots_mut().push(screen, Hot::OverviewBackdrop);

    let ids = shell.overview_ids();
    let count = ids.len();
    let workspaces = shell.wm.workspaces().len().max(1);

    let title = t.style_bold(TypeStep::Title1);
    font::draw(
        canvas,
        "Mission Control",
        t.px(48.0),
        shell.top_bar().h + t.px(58.0),
        &title,
    );
    let meta = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let summary = format!(
        "{} window{} on {} workspace{}",
        count,
        if count == 1 { "" } else { "s" },
        workspaces,
        if workspaces == 1 { "" } else { "s" }
    );
    font::draw(
        canvas,
        &summary,
        t.px(48.0),
        shell.top_bar().h + t.px(84.0),
        &meta,
    );

    let hint = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let hint_text = "Arrow keys to choose  ·  Enter to open  ·  Esc to close";
    let hint_w = font::measure(hint_text, &hint);
    font::draw(
        canvas,
        hint_text,
        (screen.w - hint_w) / 2.0,
        screen.h - t.px(28.0),
        &hint,
    );

    if count == 0 {
        let empty = Style::new(t.text_px(TypeStep::Body), t.text_dim());
        let line = "No windows are open. Open an application and it will appear here.";
        let w = font::measure(line, &empty);
        font::draw(canvas, line, (screen.w - w) / 2.0, screen.h / 2.0, &empty);
        return;
    }

    let area = Rect::new(
        t.px(48.0),
        shell.top_bar().h + t.px(104.0),
        screen.w - t.px(96.0),
        screen.h - shell.top_bar().h - t.px(104.0) - t.px(56.0),
    );
    let gap = t.px(20.0);
    let cells = crate::overview::grid(area, count, gap);

    // Read the window data before drawing, so a card never holds a borrow of
    // the manager while registering its hotspot.
    let cards: Vec<(WindowState, String, String, usize, bool)> = ids
        .iter()
        .filter_map(|id| {
            shell.wm.window(*id).map(|w| {
                (
                    w.state,
                    w.title.clone(),
                    w.app_id.clone(),
                    w.workspace,
                    w.state == WindowState::Minimized,
                )
            })
        })
        .collect();

    for (i, cell) in cells.iter().enumerate() {
        let Some((state, title, app_id, workspace, minimized)) = cards.get(i) else {
            break;
        };
        let hot = Hot::OverviewItem(i);
        let on = i == selected;
        let hovered = shell.is_highlighted(hot);

        canvas.soft_shadow(*cell, t.radius(Radius::Lg), t.px(24.0), t.px(10.0), 0.35);
        ui::surface(
            canvas,
            &t,
            *cell,
            t.radius(Radius::Lg),
            if on || hovered {
                Material::Dialog
            } else {
                Material::Elevated
            },
        );
        if hovered && !on {
            canvas.fill_round_rect(
                *cell,
                t.radius(Radius::Lg),
                with_alpha(rgb(0xFFFFFF), 0.045),
            );
        }
        if on {
            ui::focus_ring(canvas, &t, *cell, t.radius(Radius::Lg));
        }

        let inset = t.px(14.0);
        // Title row: the application's icon, its window title, and where it is.
        let icon_box = Rect::new(cell.x + inset, cell.y + inset, t.px(24.0), t.px(24.0));
        icons::draw(canvas, icon_for_app_id(app_id), icon_box, t.accent(), 1.0);
        let name = Style::new(t.text_px(TypeStep::Body), t.text());
        let name_x = icon_box.right() + t.px(9.0);
        // A minimized card carries a badge in the same row; the title stops
        // before it instead of running underneath it.
        let badge_w = if *minimized { t.px(84.0) } else { t.px(12.0) };
        let name_w = (cell.right() - badge_w - name_x).max(t.px(20.0));
        let shown = ui::elide(title, &name, name_w);
        font::draw(canvas, &shown, name_x, icon_box.y + t.px(15.0), &name);

        let state_label = match state {
            WindowState::Normal => "Normal",
            WindowState::Minimized => "Minimized",
            WindowState::Maximized => "Maximized",
            WindowState::Fullscreen => "Full screen",
            WindowState::Snapped(_) => "Snapped",
            // WindowState is non-exhaustive: a state added later shows its own
            // name rather than failing to compile this frame.
            _ => "Window",
        };
        let sub = format!(
            "Workspace {}  ·  {}",
            workspace.saturating_add(1),
            state_label
        );
        let sub_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(canvas, &sub, name_x, icon_box.y + t.px(32.0), &sub_style);

        // The map of the window: a title bar with its three controls, then a
        // few content lines whose widths differ so it reads as a document
        // rather than a loading skeleton.
        let preview = Rect::new(
            cell.x + inset,
            icon_box.bottom() + t.px(12.0),
            cell.w - inset * 2.0,
            cell.bottom() - icon_box.bottom() - t.px(26.0),
        );
        if preview.h > t.px(24.0) {
            canvas.fill_round_rect(preview, t.px(8.0), with_alpha(rgb(0x000000), 0.28));
            let bar = Rect::new(preview.x, preview.y, preview.w, t.px(16.0));
            canvas.fill_round_rect(bar, t.px(8.0), with_alpha(rgb(0xFFFFFF), 0.05));
            for (n, alpha) in [(0.45f32, 0.5f32), (0.3, 0.45), (0.0, 0.4)] {
                let dot = t.px(4.0);
                let cx = bar.x + t.px(9.0) + n * t.px(9.0);
                canvas.fill_circle(
                    cx,
                    bar.y + bar.h / 2.0,
                    dot,
                    with_alpha(rgb(0xFFFFFF), alpha),
                );
            }
            let line_x = preview.x + t.px(10.0);
            // Lines run to the bottom of the card so a tall card reads as a
            // page of text rather than a small block in a large empty frame.
            let widths = [
                0.72f32, 0.86, 0.48, 0.64, 0.9, 0.58, 0.78, 0.44, 0.7, 0.83, 0.52, 0.68, 0.88, 0.6,
                0.75, 0.46,
            ];
            let room = (preview.bottom() - bar.bottom() - t.px(16.0)).max(0.0);
            let count = usize::from((room / t.px(9.0)) as u16).min(widths.len());
            for (n, fraction) in widths.iter().take(count).enumerate() {
                let y = bar.bottom() + t.px(10.0) + (n as f32) * t.px(9.0);
                canvas.fill_round_rect(
                    Rect::new(line_x, y, (preview.w - t.px(20.0)) * fraction, t.px(3.0)),
                    t.px(2.0),
                    with_alpha(rgb(0xFFFFFF), 0.10),
                );
            }
        }

        if *minimized {
            let badge = Rect::new(
                cell.right() - t.px(84.0),
                cell.y + t.px(14.0),
                t.px(70.0),
                t.px(20.0),
            );
            canvas.fill_round_rect(badge, t.px(10.0), with_alpha(t.accent(), 0.18));
            let bs = Style::new(t.text_px(TypeStep::Caption), t.accent());
            let label = "Minimized";
            let w = font::measure(label, &bs);
            font::draw(
                canvas,
                label,
                badge.x + (badge.w - w) / 2.0,
                badge.y + t.px(14.0),
                &bs,
            );
        }

        shell.hotspots_mut().push(*cell, hot);
    }
}

fn draw_switcher(canvas: &mut Canvas, shell: &mut Shell, selected: usize) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let windows: Vec<(u32, String, String, bool)> = shell
        .wm
        .switcher_order()
        .iter()
        .map(|w| {
            (
                w.id,
                w.title.clone(),
                w.app_id.clone(),
                w.state == WindowState::Minimized,
            )
        })
        .collect();
    if windows.is_empty() {
        return;
    }

    let row_h = t.px(40.0);
    let w = t.px(420.0).min(screen.w * 0.7);
    let h = (windows.len() as f32) * row_h + t.px(52.0);
    let panel = Rect::new((screen.w - w) / 2.0, (screen.h - h) / 2.0, w, h);
    // The switcher unfolds from its middle and closes the same way: the spring
    // decides how much of the panel its clip shows.
    let reveal = shell.anims().switcher.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(
        panel.x,
        panel.y + (panel.h - shown_h) / 2.0,
        panel.w,
        shown_h,
    ));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);

    ui::section_label(
        canvas,
        &t,
        (panel.x + t.sp(Space::Md), panel.y + t.px(24.0)),
        "WINDOWS",
    );

    let mut col = ui::Column::new(
        Rect::new(panel.x, panel.y + t.px(32.0), panel.w, panel.h),
        t.sp(Space::Sm),
        t.px(2.0),
    );
    for (i, (_, title, app_id, minimized)) in windows.iter().enumerate() {
        let row = col.row(row_h);
        let hot = Hot::SwitcherItem(i);
        let state = if i == selected {
            State::Selected
        } else if shell.is_highlighted(hot) {
            State::Hover
        } else {
            State::Normal
        };
        ui::list_item(
            canvas,
            &t,
            row,
            Some(icon_for_app_id(app_id)),
            title,
            None,
            if *minimized { Some("Minimized") } else { None },
            state,
        );
        shell.hotspots_mut().push(row, hot);
    }
    canvas.set_clip(saved);
}
fn icon_for_app_id(app_id: &str) -> Icon {
    App::ALL
        .iter()
        .find(|a| a.app_id() == app_id)
        .map_or(Icon::Window, |a| icon_for(*a))
}

/// The panel's width, its gutter and the room between two cards.
///
/// The sheet is wider than the panel it replaced (320) because a card holds the
/// label, the value and the control on one row, and 344 is the first width where
/// "Reduce Transparency" and its switch stop competing for the same pixels.
const QUICK_W: f32 = 344.0;
const QUICK_PAD: f32 = 12.0;
const QUICK_GAP: f32 = 10.0;

/// The height of a card's own label row.
const QUICK_LABEL_H: f32 = 26.0;
/// The height of the sentence a row carries when the value above it does not say
/// everything: a note that is drawn is a note that is counted.
const QUICK_NOTE_H: f32 = 16.0;

/// One card of the quick-settings sheet: what it is called, and how tall it is.
///
/// The cards are described once, in [`quick_cards`], and read twice -- the
/// panel's height is their sum and the drawing walks the same list -- so a card
/// cannot be drawn outside the sheet that holds it. The constant that used to
/// stand in for this sum (420 pixels) was already shorter than its own content,
/// so the last section was drawn into the clip that reveals the panel and never
/// appeared -- and a row outside the panel is a row no click can reach.
struct QuickCard {
    label: &'static str,
    h: f32,
}

/// Every card of the quick-settings sheet, top to bottom.
///
/// The order is the order of the drawing, and each card is measured from what it
/// holds: the notes that are only sometimes there -- network and Bluetooth report
/// one when the last attempt to change the radio had something to say -- are in
/// the sum only when they will be drawn.
fn quick_cards(shell: &Shell, t: &Tokens) -> Vec<QuickCard> {
    let label = t.px(QUICK_LABEL_H);
    let note = t.px(QUICK_NOTE_H);
    let switch_row = t.px(44.0);
    let mut cards = vec![
        QuickCard {
            label: "POWER",
            h: label + t.px(46.0),
        },
        QuickCard {
            label: "SYSTEM",
            h: label + 4.0 * switch_row + t.px(6.0),
        },
        QuickCard {
            label: "NETWORK",
            h: label
                + t.px(38.0)
                + if shell.quick.net_note.is_empty() {
                    0.0
                } else {
                    note
                },
        },
        QuickCard {
            label: "BLUETOOTH",
            h: label
                + t.px(38.0)
                + if shell.quick.bt_note.is_empty() {
                    0.0
                } else {
                    note
                },
        },
        // The volume card always carries its sentence, whether it says where the
        // level came from or that nothing has answered yet.
        QuickCard {
            label: "VOLUME",
            h: label + t.px(34.0) + t.px(16.0) + note,
        },
    ];
    let env_rows = shell.environments.len().min(3);
    if env_rows > 0 {
        cards.push(QuickCard {
            label: "ENVIRONMENT",
            h: label + env_rows as f32 * t.px(32.0) + t.px(8.0),
        });
    }
    cards
}

/// The quick-settings sheet's rectangle, measured from the cards drawn in it.
///
/// Its top is below the bar, so the clock is never covered, and it is clamped to
/// the screen, because a row below the bottom edge is a row no click can reach.
fn quick_panel_rect(shell: &Shell, t: &Tokens) -> Rect {
    let screen = shell.wm.screen();
    let w = t.px(QUICK_W).min(screen.w * 0.92);
    let top = shell.top_bar().h + t.px(6.0);
    Rect::new(
        screen.w - w - t.px(10.0),
        top,
        w,
        quick_content_height(shell, t).min(screen.h - top - t.px(12.0)),
    )
}

/// The height of everything the quick-settings sheet draws, in pixels.
///
/// The sum and the drawing are kept in step by
/// `no_panel_row_is_drawn_outside_the_panel`, which fails when a card is added
/// without a term in [`quick_cards`].
fn quick_content_height(shell: &Shell, t: &Tokens) -> f32 {
    let cards = quick_cards(shell, t);
    let pad = t.px(QUICK_PAD);
    let gap = t.px(QUICK_GAP);
    pad * 2.0 + cards.iter().map(|c| c.h).sum::<f32>() + gap * cards.len().saturating_sub(1) as f32
}

fn draw_quick_settings(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    // The panel reveals downward under a clip that grows with its spring, and
    // its rows only become clickable once the spring is far enough along that a
    // click can only be aiming at the panel, not passing through the reveal.
    let reveal = shell.anims().quick_settings.value();
    if reveal <= 0.01 {
        return;
    }
    let panel = quick_panel_rect(shell, &t);
    let shown_h = panel.h * reveal;
    // The shadow is drawn under a wider clip than the glass. Cut to the panel's
    // own rectangle it fills the gaps its rounded corners leave, and the panel
    // looks square-cornered on a pale desktop.
    let reach = t.px(40.0);
    let shadow_clip = canvas.push_clip(Rect::new(
        panel.x - reach,
        panel.y,
        panel.w + reach * 2.0,
        shown_h + reach,
    ));
    canvas.soft_shadow(
        Rect::new(panel.x, panel.y, panel.w, shown_h),
        t.px(22.0),
        t.px(28.0),
        t.px(10.0),
        0.55,
    );
    canvas.set_clip(shadow_clip);
    let saved = canvas.push_clip(Rect::new(panel.x, panel.y, panel.w, shown_h));
    crate::desktop::glass_at(canvas, &t.theme, panel, t.px(22.0), 88);

    // One card per thing, on one glass sheet: a fill and a hairline of light,
    // both from the theme's own text colour, so a card reads on any wallpaper.
    let card_radius = t.px(16.0);
    let card = |canvas: &mut Canvas, r: Rect| {
        canvas.fill_round_rect(r, card_radius, with_alpha(t.text(), 0.07));
        canvas.stroke_round_rect(r, card_radius, 1.0, with_alpha(chrome::WHITE, 0.07));
    };
    let on_accent = ui::rgb_of(t.theme.palette.bg_base);
    let footer = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let value = Style::new(t.text_px(TypeStep::Footnote), t.text());

    // The cards, measured once, at the rectangle each one will be drawn in.
    let cards = quick_cards(shell, &t);
    let pad = t.px(QUICK_PAD);
    let gap = t.px(QUICK_GAP);
    let inner_w = panel.w - pad * 2.0;
    let label_h = t.px(QUICK_LABEL_H);
    let mut y = panel.y + pad;
    let mut rects = Vec::with_capacity(cards.len());
    for c in &cards {
        rects.push(Rect::new(panel.x + pad, y, inner_w, c.h));
        y += c.h + gap;
    }
    let label_at = |canvas: &mut Canvas, r: Rect, text: &str| {
        ui::section_label(canvas, &t, (r.x + t.px(14.0), r.y + t.px(19.0)), text);
    };
    // A row of a card: the card's own width, inset by what its padding and its
    // hairline need.
    let row_of = |r: Rect, top: f32, h: f32| -> Rect {
        Rect::new(r.x + t.px(6.0), r.y + top, r.w - t.px(12.0), h)
    };

    // The labels and the rectangles walk together, in the order
    // [`quick_cards`] describes them: the label a card draws is the label it was
    // measured from, so the two cannot drift apart, and nothing here indexes a
    // list whose length the reader would have to count.
    let mut pending = cards.iter().zip(rects.iter());
    // ---- power profile ------------------------------------------------------
    let Some((power_label, power)) = pending.next().map(|(c, r)| (c.label, *r)) else {
        return;
    };
    card(canvas, power);
    label_at(canvas, power, power_label);
    let track = Rect::new(
        power.x + t.px(8.0),
        power.y + label_h + t.px(2.0),
        power.w - t.px(16.0),
        t.px(34.0),
    );
    canvas.fill_round_rect(track, track.h / 2.0, with_alpha(t.text(), 0.08));
    let seg_w = track.w / 4.0;
    // Four labels in a third of the screen: "Performance" does not fit a
    // 70-pixel segment at the footnote step, so each segment gets the longest
    // label that measures inside it -- the full word when it fits, a compact one
    // when it does not -- and an ellipsis as the last resort.
    let seg_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let room = seg_w - t.px(12.0);
    let compact = ["Eco", "Balanced", "Perf", "Max"];
    for (i, p) in PowerProfile::ALL.iter().enumerate() {
        let r = Rect::new(track.x + i as f32 * seg_w, track.y, seg_w, track.h);
        let hot = Hot::QuickPower(i as u8);
        let full = p.title();
        let label = if font::measure(full, &seg_style) <= room {
            full.to_owned()
        } else {
            compact.get(i).copied().unwrap_or(full).to_owned()
        };
        let label = ui::elide(&label, &seg_style, room);
        let selected = *p == shell.quick.power;
        if selected {
            canvas.fill_round_rect(r.inflate(-t.px(3.0)), (r.h - t.px(6.0)) / 2.0, t.accent());
        } else if shell.is_highlighted(hot) {
            canvas.fill_round_rect(
                r.inflate(-t.px(3.0)),
                (r.h - t.px(6.0)) / 2.0,
                with_alpha(t.text(), 0.10),
            );
        }
        let style = if selected {
            Style::bold(seg_style.size, on_accent)
        } else {
            seg_style
        };
        font::draw(
            canvas,
            &label,
            r.x + (r.w - font::measure(&label, &style)) / 2.0,
            r.y + r.h / 2.0 + style.size * 0.36,
            &style,
        );
        shell.hotspots_mut().push(r, hot);
    }

    // ---- system switches ----------------------------------------------------
    let Some((sys_label, sys)) = pending.next().map(|(c, r)| (c.label, *r)) else {
        return;
    };
    card(canvas, sys);
    label_at(canvas, sys, sys_label);
    let switch_h = (sys.h - label_h - t.px(6.0)) / 4.0;
    let toggles: [(&str, Icon, bool, bool); 4] = [
        (
            "Do Not Disturb",
            Icon::Notifications,
            shell.quick.do_not_disturb,
            shell.quick.is_applied("do_not_disturb"),
        ),
        (
            "Adaptive Idle",
            Icon::Power,
            shell.quick.adaptive_idle,
            shell.quick.is_applied("adaptive_idle"),
        ),
        (
            "Reduce Transparency",
            Icon::Window,
            shell.quick.reduce_transparency,
            shell.quick.is_applied("reduce_transparency"),
        ),
        (
            "Reduce Motion",
            Icon::Activity,
            shell.quick.reduce_motion,
            shell.quick.is_applied("reduce_motion"),
        ),
    ];
    for (i, (label, icon, on, applied)) in toggles.iter().enumerate() {
        let row = row_of(sys, label_h + switch_h * i as f32, switch_h);
        let hot = Hot::QuickToggle(i as u8);
        if shell.is_highlighted(hot) {
            canvas.fill_round_rect(row, t.px(12.0), with_alpha(t.text(), 0.07));
        }
        // The glyph sits in a disc that fills with the accent when the switch is
        // on, so the state reads from across the panel before the switch does.
        let disc = Rect::new(
            row.x + t.px(8.0),
            row.y + (row.h - t.px(30.0)) / 2.0,
            t.px(30.0),
            t.px(30.0),
        );
        canvas.fill_circle(
            disc.center().0,
            disc.center().1,
            disc.w / 2.0,
            if *on {
                t.accent()
            } else {
                with_alpha(t.text(), 0.12)
            },
        );
        icons::draw(
            canvas,
            *icon,
            disc.inflate(-t.px(8.0)),
            if *on { on_accent } else { t.text() },
            1.1,
        );
        let text_x = disc.right() + t.px(12.0);
        let name = value;
        let sw = Rect::new(
            row.right() - t.px(8.0) - t.px(40.0),
            row.y + (row.h - t.px(22.0)) / 2.0,
            t.px(40.0),
            t.px(22.0),
        );
        let room = sw.x - t.px(8.0) - text_x;
        let label = ui::elide(label, &name, room);
        if *applied {
            font::draw(
                canvas,
                &label,
                text_x,
                row.y + row.h / 2.0 + name.size * 0.36,
                &name,
            );
        } else {
            // A control that only records a preference must say so.
            font::draw(
                canvas,
                &label,
                text_x,
                row.y + row.h / 2.0 - t.px(1.0),
                &name,
            );
            font::draw(
                canvas,
                &ui::elide("preference only", &footer, room),
                text_x,
                row.y + row.h / 2.0 + t.px(12.0),
                &footer,
            );
        }
        ui::toggle(canvas, &t, sw, *on, State::Normal);
        shell.hotspots_mut().push(row, hot);
    }

    // ---- network: one card, over the provider litric-sysd already has -------
    // The state is the kernel's, read from the facts; the controls are the two
    // actions the daemon implements for a link (`NET LINK`, `NET RECONFIGURE`).
    // A machine that reports no interface gets the sentence and no controls.
    let Some((net_label, net)) = pending.next().map(|(c, r)| (c.label, *r)) else {
        return;
    };
    card(canvas, net);
    label_at(canvas, net, net_label);
    {
        let row = row_of(net, label_h, t.px(38.0));
        let iface = shell.primary_iface().map(|i| i.name.clone());
        let up = shell.primary_iface().is_some_and(|i| i.state == "up");
        let summary = shell.net_summary();
        // The label yields to the state line, which is the part that says which
        // interface this row is about.
        let room = if iface.is_some() {
            row.w - t.px(160.0)
        } else {
            row.w - t.px(56.0)
        };
        font::draw(
            canvas,
            "Network",
            row.x + t.px(8.0),
            row.y + t.px(16.0),
            &value,
        );
        font::draw(
            canvas,
            &ui::elide(&summary, &footer, room.max(t.px(40.0))),
            row.x + t.px(8.0),
            row.y + row.h - t.px(5.0),
            &footer,
        );
        let sw = Rect::new(
            row.right() - t.px(48.0),
            row.y + t.px(8.0),
            t.px(40.0),
            t.px(22.0),
        );
        if iface.is_some() {
            let re = Rect::new(sw.x - t.px(80.0), sw.y, t.px(74.0), t.px(22.0));
            ui::button(
                canvas,
                &t,
                re,
                "Reconnect",
                if shell.is_highlighted(Hot::QuickLinkReconnect) {
                    State::Hover
                } else {
                    State::Normal
                },
                Tone::Neutral,
            );
            shell.hotspots_mut().push(re, Hot::QuickLinkReconnect);
            ui::toggle(
                canvas,
                &t,
                sw,
                up,
                if shell.is_highlighted(Hot::QuickLinkToggle) {
                    State::Hover
                } else {
                    State::Normal
                },
            );
            shell.hotspots_mut().push(sw, Hot::QuickLinkToggle);
        } else {
            // No interface: the switch is drawn in its off position and is not
            // clickable, because there is nothing to bring up.
            ui::toggle(canvas, &t, sw, false, State::Disabled);
        }
        if !shell.quick.net_note.is_empty() {
            let note = row_of(net, label_h + t.px(38.0), t.px(QUICK_NOTE_H));
            font::draw(
                canvas,
                &ui::elide(shell.quick.net_note.as_str(), &footer, note.w),
                note.x + t.px(2.0),
                note.y + t.px(11.0),
                &footer,
            );
        }
    }

    // ---- Bluetooth: the radio's state, and its switch -----------------------
    // The switch has a position only where BlueZ has been asked, and exists at
    // all only where the platform found a radio, nothing is blocking it and a
    // service is installed. Everywhere else the card says which of those is
    // missing instead of drawing a control that could not work.
    let Some((bt_label, bt)) = pending.next().map(|(c, r)| (c.label, *r)) else {
        return;
    };
    card(canvas, bt);
    label_at(canvas, bt, bt_label);
    {
        let row = row_of(bt, label_h, t.px(38.0));
        let measured = shell.quick.bt_powered;
        let live = shell.bt_controllable() && measured.is_some();
        let summary = shell.bt_summary();
        font::draw(
            canvas,
            "Bluetooth",
            row.x + t.px(8.0),
            row.y + t.px(16.0),
            &value,
        );
        // The row has one control, not two: the state line gets the width the
        // switch does not need, so an adapter address is readable rather than cut
        // at "hci0 00:1a:7d:da:71:...".
        font::draw(
            canvas,
            &ui::elide(&summary, &footer, row.w - t.px(60.0)),
            row.x + t.px(8.0),
            row.y + row.h - t.px(5.0),
            &footer,
        );
        let sw = Rect::new(
            row.right() - t.px(48.0),
            row.y + t.px(8.0),
            t.px(40.0),
            t.px(22.0),
        );
        let state = if !live {
            State::Disabled
        } else if shell.is_highlighted(Hot::QuickBluetoothPower) {
            State::Hover
        } else {
            State::Normal
        };
        ui::toggle(canvas, &t, sw, measured.unwrap_or(false), state);
        if live {
            shell.hotspots_mut().push(sw, Hot::QuickBluetoothPower);
        }
        if !shell.quick.bt_note.is_empty() {
            let note = row_of(bt, label_h + t.px(38.0), t.px(QUICK_NOTE_H));
            font::draw(
                canvas,
                &ui::elide(shell.quick.bt_note.as_str(), &footer, note.w),
                note.x + t.px(2.0),
                note.y + t.px(11.0),
                &footer,
            );
        }
    }

    // ---- volume: a measured level, and the three things you can do to it ----
    // The reading is the sound server's, not this panel's: with no answer yet it
    // says so instead of drawing a zero, and the sentence underneath says what
    // the last change did.
    let Some((vol_label, vol)) = pending.next().map(|(c, r)| (c.label, *r)) else {
        return;
    };
    card(canvas, vol);
    label_at(canvas, vol, vol_label);
    {
        let row = row_of(vol, label_h, t.px(34.0));
        let s_value = Style::bold(t.text_px(TypeStep::Body), t.text());
        let reading = match shell.quick.volume {
            Some(v) if shell.quick.muted => format!("{v}%  muted"),
            Some(v) => format!("{v}%"),
            None => "unavailable".to_owned(),
        };
        font::draw(
            canvas,
            &reading,
            row.x + t.px(8.0),
            row.y + row.h / 2.0 + s_value.size * 0.36,
            &s_value,
        );
        // Two step buttons and a mute switch, right-aligned. Each one is disabled
        // when there is no measured level to step from: a control that would send
        // a number nobody measured is not offered.
        let known = shell.quick.volume.is_some();
        let bw = t.px(34.0);
        let mute = Rect::new(
            row.right() - t.px(58.0),
            row.y + t.px(6.0),
            t.px(50.0),
            t.px(22.0),
        );
        let up = Rect::new(mute.x - bw - t.px(6.0), mute.y, bw, t.px(22.0));
        let down = Rect::new(up.x - bw - t.px(6.0), mute.y, bw, t.px(22.0));
        ui::button(
            canvas,
            &t,
            down,
            "-",
            if !known {
                State::Disabled
            } else if shell.is_highlighted(Hot::QuickVolumeDown) {
                State::Hover
            } else {
                State::Normal
            },
            Tone::Neutral,
        );
        ui::button(
            canvas,
            &t,
            up,
            "+",
            if !known {
                State::Disabled
            } else if shell.is_highlighted(Hot::QuickVolumeUp) {
                State::Hover
            } else {
                State::Normal
            },
            Tone::Neutral,
        );
        ui::button(
            canvas,
            &t,
            mute,
            if shell.quick.muted { "Unmute" } else { "Mute" },
            if !known {
                State::Disabled
            } else if shell.is_highlighted(Hot::QuickVolumeMute) {
                State::Hover
            } else {
                State::Normal
            },
            if shell.quick.muted {
                Tone::Accent
            } else {
                Tone::Neutral
            },
        );
        if known {
            shell.hotspots_mut().push(down, Hot::QuickVolumeDown);
            shell.hotspots_mut().push(up, Hot::QuickVolumeUp);
            shell.hotspots_mut().push(mute, Hot::QuickVolumeMute);
        }
        // Why the level reads "unavailable" belongs on the sentence line, not
        // squeezed under the value: at the body step two lines of text in a
        // 34-pixel row are drawn through each other.
        let note = if !shell.quick.audio_note.is_empty() {
            shell.quick.audio_note.as_str()
        } else if shell.quick.volume.is_none() {
            "the sound server has not answered"
        } else {
            "Set through litric-sysd, which asks the sound server and reads the level back."
        };
        let note_row = row_of(vol, label_h + t.px(34.0) + t.px(16.0), t.px(QUICK_NOTE_H));
        font::draw(
            canvas,
            &ui::elide(note, &footer, note_row.w),
            note_row.x + t.px(2.0),
            note_row.y + t.px(11.0),
            &footer,
        );
    }

    // ---- environment --------------------------------------------------------
    if let Some((env_label, env)) = pending.next().map(|(c, r)| (c.label, *r)) {
        card(canvas, env);
        label_at(canvas, env, env_label);
        let rows = shell.environments.len().min(3) as f32;
        let env_row_h = (env.h - label_h - t.px(8.0)) / rows;
        let envs = shell.environments.clone();
        for (i, e) in envs.iter().enumerate().take(3) {
            let row = row_of(env, label_h + env_row_h * i as f32, env_row_h);
            let hot = Hot::QuickEnvironment(i as u8);
            let state = if e.active {
                State::Selected
            } else if shell.is_highlighted(hot) {
                State::Hover
            } else {
                State::Normal
            };
            ui::list_item(
                canvas,
                &t,
                row,
                None,
                &ui::elide(&e.title, &value, row.w - t.px(70.0)),
                None,
                Some(&e.hotkey),
                state,
            );
            shell.hotspots_mut().push(row, hot);
        }
    }
    canvas.set_clip(saved);
}
fn draw_power_menu(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let items = ["Lock", "Sleep", "Shut down", "Restart"];
    let row_h = t.px(34.0);
    let pad = t.sp(Space::Xs);
    let icon = t.px(18.0);
    let gap = t.px(12.0);
    let style = Style::new(t.text_px(TypeStep::Body), t.text());
    let widest = items
        .iter()
        .map(|l| font::measure(&crate::i18n::text(l), &style))
        .fold(0.0_f32, f32::max);
    let w = (pad * 2.0 + icon + gap + widest + t.px(12.0)).min(screen.w - t.px(24.0));
    let h = pad * 2.0 + row_h * items.len() as f32;
    let panel = Rect::new(
        (screen.w - w - t.px(12.0)).max(t.px(8.0)),
        shell.top_bar().h + t.px(8.0),
        w,
        h.min(screen.h - shell.top_bar().h - t.px(20.0)),
    );
    let reveal = shell.anims().quick_settings.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(panel.x, panel.y, panel.w, shown_h));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);
    let mut y = panel.y + pad;
    for (i, label) in items.iter().enumerate() {
        let row = Rect::new(panel.x + pad, y, panel.w - pad * 2.0, row_h);
        let hot = Hot::PowerItem(i as u8);
        if shell.is_highlighted(hot) {
            canvas.fill_round_rect(row, t.radius(Radius::Md), with_alpha(rgb(0xFFFFFF), 0.06));
        }
        let glyph = if i == 0 { Icon::Lock } else { Icon::Power };
        icons::draw(
            canvas,
            glyph,
            Rect::new(row.x + t.px(4.0), row.y + (row_h - icon) / 2.0, icon, icon),
            t.text(),
            1.6,
        );
        let text_x = row.x + t.px(4.0) + icon + gap;
        let text = ui::elide(
            &crate::i18n::text(label),
            &style,
            row.right() - text_x - t.px(4.0),
        );
        font::draw(
            canvas,
            &text,
            text_x,
            row.y + row_h / 2.0 + style.size * 0.35,
            &style,
        );
        shell.hotspots_mut().push(row, hot);
        y += row_h;
    }
    canvas.set_clip(saved);
}

/// The LitricOS menu, under the wordmark at the left of the system bar.
///
/// This is where the session's own settings moved to when the quick-settings
/// panel's bottom section was taken out: the mark is the one label on the bar
/// that means "the system itself", so the settings for the session belong
/// under it, beside the lock.
fn draw_litro_menu(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let items = [
        (crate::i18n::text("Session settings"), Hot::LitroSession),
        (crate::i18n::text("Lock screen"), Hot::LitroLock),
    ];
    let pad = t.sp(Space::Xs);
    let row_h = t.px(34.0);
    let style = Style::new(t.text_px(TypeStep::Body), t.text());
    let widest = items
        .iter()
        .map(|(l, _)| font::measure(l, &style))
        .fold(0.0_f32, f32::max);
    // Room for the widest label plus its side padding, and a little more: a
    // menu measured to the pixel leaves its last glyph against the edge, and
    // the shaped run is a hair wider than its measure.
    // A shaped Thai run is a little wider than its Latin measure, so the menu
    // also has a floor: two short words that must stay readable, not two words
    // that fit only because the measure said so.
    let w = (widest + pad * 2.0 + t.px(22.0))
        .max(t.px(184.0))
        .min(screen.w - t.px(16.0));
    let h = pad * 2.0 + row_h * items.len() as f32;
    let panel = Rect::new(t.px(8.0), shell.top_bar().h + t.px(6.0), w, h);
    let reveal = shell.anims().quick_settings.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(panel.x, panel.y, panel.w, shown_h));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);
    let mut y = panel.y + pad;
    for (label, hot) in &items {
        let row = Rect::new(panel.x + pad, y, panel.w - pad * 2.0, row_h);
        if shell.is_highlighted(*hot) {
            canvas.fill_round_rect(row, t.radius(Radius::Md), with_alpha(rgb(0xFFFFFF), 0.06));
        }
        let text = ui::elide(label, &style, row.w - t.px(16.0));
        font::draw(
            canvas,
            &text,
            row.x + t.px(8.0),
            row.y + row_h / 2.0 + style.size * 0.35,
            &style,
        );
        shell.hotspots_mut().push(row, *hot);
        y += row_h;
    }
    canvas.set_clip(saved);
}

fn draw_notification_list(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let w = t.px(340.0).min(screen.w * 0.9);
    let items: Vec<(u32, String, String, Priority)> = shell
        .notifications
        .items()
        .iter()
        .map(|n| (n.id, n.title.clone(), n.source.clone(), n.priority))
        .collect();

    let row_h = t.px(56.0);
    let h = (t.px(64.0) + (items.len().max(1) as f32) * row_h).min(screen.h * 0.7);
    let panel = Rect::new(
        screen.w - w - t.px(12.0),
        shell.top_bar().h + t.px(8.0),
        w,
        h,
    );
    // The same downward reveal as the quick-settings panel: one motion
    // grammar for every panel that drops from the top bar.
    let reveal = shell.anims().notifications.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(panel.x, panel.y, panel.w, shown_h));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);

    ui::section_label(
        canvas,
        &t,
        (panel.x + t.sp(Space::Md), panel.y + t.px(24.0)),
        "NOTIFICATIONS",
    );
    if !items.is_empty() {
        let br = Rect::new(
            panel.right() - t.px(84.0),
            panel.y + t.px(10.0),
            t.px(72.0),
            t.px(22.0),
        );
        ui::button(
            canvas,
            &t,
            br,
            "Clear all",
            if shell.is_highlighted(Hot::NotificationsClear) {
                State::Hover
            } else {
                State::Normal
            },
            Tone::Neutral,
        );
        shell.hotspots_mut().push(br, Hot::NotificationsClear);
    }

    let mut col = ui::Column::new(
        Rect::new(panel.x, panel.y + t.px(36.0), panel.w, panel.h),
        t.sp(Space::Sm),
        t.px(2.0),
    );
    if items.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(
            canvas,
            "Nothing waiting",
            col.x + t.px(8.0),
            col.cursor + t.px(22.0),
            &s,
        );
        canvas.set_clip(saved);
        return;
    }
    for (i, (_, title, source, priority)) in items.iter().enumerate() {
        if col.cursor + row_h > panel.bottom() - t.px(6.0) {
            break;
        }
        let row = col.row(row_h);
        let hot = Hot::NotificationItem(i);
        ui::list_item(
            canvas,
            &t,
            row,
            Some(priority_icon(*priority)),
            title,
            Some(source),
            None,
            if shell.is_highlighted(hot) {
                State::Hover
            } else {
                State::Normal
            },
        );
        shell.hotspots_mut().push(row, hot);
    }
    canvas.set_clip(saved);
}

const fn priority_icon(p: Priority) -> Icon {
    match p {
        Priority::Critical | Priority::High => Icon::Warning,
        _ => Icon::Info,
    }
}

/// Banners for notifications that arrived recently.
fn draw_banners(canvas: &mut Canvas, shell: &mut Shell) {
    if shell.overlay == Overlay::Notifications {
        return;
    }
    draw_revert_banner(canvas, shell);
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    // The fifth field says whether the banner is still live. A retiring
    // banner is a picture of something that has already gone; it must not
    // answer a click, because its id is no longer in the queue.
    let mut rows: Vec<(u32, String, String, Priority, bool)> = shell
        .notifications
        .items()
        .iter()
        .filter(|n| shell.notifications.should_banner(n.priority))
        // A banner shows for anything that will dismiss itself, and for
        // criticals, which never carry an expiry because what is wrong now
        // does not stop being wrong five seconds later.
        .filter(|n| n.expires_ms.is_some() || n.priority == Priority::Critical)
        .take(3)
        .map(|n| (n.id, n.title.clone(), n.body.clone(), n.priority, true))
        .collect();
    // A dismissed banner keeps its pixels for the length of its exit spring.
    // The live queue is authoritative for everything else, which is why this
    // only adds ids it no longer holds.
    for banner in shell.retiring_banners() {
        if !rows.iter().any(|(id, ..)| *id == banner.id) {
            rows.push((
                banner.id,
                banner.title.clone(),
                banner.body.clone(),
                banner.priority,
                false,
            ));
        }
    }

    let w = t.px(300.0).min(screen.w * 0.6);
    let mut y = shell.top_bar().h + t.px(10.0);
    for (id, title, body, priority, live) in rows {
        let reveal = shell.anims().banner_reveal(id).clamp(0.0, 1.0);
        if reveal <= 0.0 {
            continue;
        }
        // The banner slides in from the screen edge and leaves the same way.
        // The software canvas has no compositing alpha, so travel is the
        // honest exit, and the spring says how far it has come: at reveal 0 it
        // sits entirely past the edge it belongs to.
        let r = Rect::new(
            screen.w - w - t.px(12.0) + (1.0 - reveal) * (w + t.px(12.0)),
            y,
            w,
            t.px(60.0),
        );
        ui::surface(canvas, &t, r, t.radius(Radius::Md), Material::Popover);
        let tone = match priority {
            Priority::Critical => Tone::Bad,
            Priority::High => Tone::Warning,
            _ => Tone::Accent,
        };
        canvas.fill_round_rect(
            Rect::new(r.x, r.y + t.px(10.0), t.px(3.0), r.h - t.px(20.0)),
            t.px(1.5),
            t.tone(tone),
        );
        icons::draw(
            canvas,
            priority_icon(priority),
            Rect::new(r.x + t.px(14.0), r.y + t.px(14.0), t.px(16.0), t.px(16.0)),
            t.tone(tone),
            1.0,
        );
        // Text stops short of the close box rather than running under it.
        let text_w = r.w - t.px(76.0);
        let ts = Style::new(t.text_px(TypeStep::Footnote), t.text());
        font::draw(
            canvas,
            &ui::elide(&title, &ts, text_w),
            r.x + t.px(38.0),
            r.y + t.px(25.0),
            &ts,
        );
        let bs = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let shown = ui::elide(&body, &bs, text_w);
        font::draw(canvas, &shown, r.x + t.px(38.0), r.y + t.px(43.0), &bs);
        // A banner is a summary, so it answers what a summary invites: the
        // body opens the centre to read the rest, and the close box takes
        // this one away without hunting for it in a list. A banner already
        // on its way out answers neither — its id has left the queue, and a
        // click on a leaving rectangle must not act on a different note.
        if live {
            let close = Rect::new(
                r.right() - t.px(26.0),
                r.y + t.px(8.0),
                t.px(18.0),
                t.px(18.0),
            );
            let state = if shell.is_highlighted(Hot::BannerClose(id)) {
                State::Hover
            } else {
                State::Normal
            };
            ui::icon_button(canvas, &t, close, Icon::Close, state, Tone::Neutral);
            shell.hotspots_mut().push(r, Hot::BannerOpen(id));
            shell.hotspots_mut().push(close, Hot::BannerClose(id));
        }
        y += r.h + t.px(8.0);
    }
}

/// The countdown for a display change that will undo itself.
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
    font::draw(
        canvas,
        &title,
        r.x + t.px(14.0),
        r.y + t.px(24.0),
        &title_style,
    );
    let body = format!(
        "Keep it, or the screen goes back in {}s",
        pending.seconds_left()
    );
    font::draw(
        canvas,
        &body,
        r.x + t.px(14.0),
        r.y + t.px(44.0),
        &body_style,
    );
    let mut button =
        |canvas: &mut Canvas, t: &Tokens, at: Rect, label: &str, hot: Hot, filled: bool| {
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

fn draw_diagnostics(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let lines = shell.diagnostic_lines();
    let s = Style::new(t.text_px(TypeStep::Caption), t.text());
    let line_h = t.px(14.0);
    let w = t.px(320.0);
    let h = (lines.len() as f32) * line_h + t.px(20.0);
    let screen = shell.wm.screen();
    let r = Rect::new(
        t.px(12.0),
        (screen.h - h - t.px(12.0)).max(shell.top_bar().h),
        w,
        h.min(screen.h * 0.8),
    );
    // Deliberately plain: a diagnostic overlay that looks like the rest of the
    // interface is a diagnostic overlay somebody will mistake for the product.
    canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(rgb(0x000000), 0.80));
    canvas.stroke_round_rect(r, t.radius(Radius::Sm), 1.0, with_alpha(t.accent(), 0.5));

    let mut y = r.y + t.px(16.0);
    for line in lines {
        if y > r.bottom() - t.px(4.0) {
            break;
        }
        font::draw(
            canvas,
            &ui::elide(&line, &s, r.w - t.px(16.0)),
            r.x + t.px(8.0),
            y,
            &s,
        );
        y += line_h;
    }
}

/// First-run setup: language, theme and motion, then Finish. Every control is
/// a hotspot, so the same pointer and keyboard path as the lock screen works.
fn draw_setup(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let pane_w = t.px(460.0).min(screen.w - t.sp(Space::Lg) * 2.0);
    let pane = Rect::new(
        (screen.w - pane_w) / 2.0,
        screen.h * 0.22,
        pane_w,
        t.px(360.0),
    );
    ui::surface(
        canvas,
        &t,
        pane,
        t.radius(Radius::Lg),
        ui::Material::Elevated,
    );

    let x = pane.x + t.sp(Space::Lg);
    let mut y = pane.y + t.px(32.0);
    let title = crate::i18n::text("Welcome to LitricOS");
    font::draw(canvas, &title, x, y, &t.style_bold(TypeStep::Title1));
    y += t.line_px(TypeStep::Title1) + t.px(8.0);
    let intro =
        crate::i18n::text("Choose your language and theme. You can change both later in Settings.");
    let body = t.style(TypeStep::Footnote);
    for line in ui::wrap(&intro, &body, pane.w - t.sp(Space::Lg) * 2.0, 3) {
        font::draw(canvas, &line, x, y, &body);
        y += t.line_px(TypeStep::Footnote);
    }
    y += t.px(24.0);

    let theme = shell.settings.preferences.theme;
    let label = format!("{}: {}", crate::i18n::text("Theme"), theme_name(theme));
    setup_button(canvas, shell, &t, pane, y, &label, Hot::SettingsTheme);
    y += t.px(40.0);
    let label = format!(
        "{}: {}",
        crate::i18n::text("Language"),
        shell.settings.preferences.language.label()
    );
    setup_button(canvas, shell, &t, pane, y, &label, Hot::SettingsLanguage);
    y += t.px(40.0);
    let state = if shell.settings.preferences.reduce_motion {
        crate::i18n::text("On")
    } else {
        crate::i18n::text("Off")
    };
    let label = format!("{}: {}", crate::i18n::text("Reduce motion"), state);
    setup_button(canvas, shell, &t, pane, y, &label, Hot::SettingsMotion);
    y += t.px(56.0);
    setup_button(
        canvas,
        shell,
        &t,
        pane,
        y,
        &crate::i18n::text("Finish setup"),
        Hot::SetupDone,
    );
}

fn setup_button(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &Tokens,
    pane: Rect,
    y: f32,
    label: &str,
    hot: Hot,
) {
    let r = Rect::new(
        pane.x + t.sp(Space::Lg),
        y,
        pane.w - t.sp(Space::Lg) * 2.0,
        t.px(32.0),
    );
    let state = if shell.is_highlighted(hot) {
        State::Hover
    } else {
        State::Normal
    };
    ui::button(canvas, t, r, label, state, Tone::Accent);
    shell.hotspots_mut().push(r, hot);
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn dock_magnification_follows_smoothstep_golden_values() {
        // s = 1 - d/88, growth = 28 * s^2 * (3 - 2s).
        let at = dock_magnification;
        assert!((at(0.0) - 28.0).abs() < 1e-4, "peak at the pointer");
        assert!(
            (at(22.0) - 23.625).abs() < 1e-3,
            "s=0.75 -> 23.625, got {}",
            at(22.0)
        );
        assert!(
            (at(44.0) - 14.0).abs() < 1e-4,
            "half range is half the peak"
        );
        assert!(at(88.0).abs() < 1e-4, "zero at the edge of the range");
        assert!(at(132.0).abs() < 1e-4, "zero well beyond the range");
        assert!(
            (at(-30.0) - at(30.0)).abs() < 1e-5,
            "symmetric about the pointer"
        );
    }
    use crate::canvas::luminance;
    use crate::shell::{DesktopMode, EnvRow, NetIface, SystemFacts};
    use crate::shortcuts::Action;
    use crate::ui::Tokens;
    use litric_design::{Theme, ThemeKind};
    use litric_locale::Regional;
    use litric_wm::HitZone;

    fn shell() -> Shell {
        let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
        s.mode = DesktopMode::Ready;
        s.environments = vec![
            EnvRow {
                token: "windows".to_owned(),
                title: "Windows 11".to_owned(),
                state: "not-installed".to_owned(),
                active: false,
                hotkey: "Win+1".to_owned(),
            },
            EnvRow {
                token: "litric".to_owned(),
                title: "LitricOS".to_owned(),
                state: "running".to_owned(),
                active: true,
                hotkey: "Win+2".to_owned(),
            },
        ];
        s.facts.mem_total_mb = Some(946);
        s.facts.mem_used_mb = Some(647);
        s.facts.cpu_fraction = Some(0.08);
        s.facts.source = "test".to_owned();
        s
    }

    fn canvas() -> Canvas {
        let mut c = Canvas::new(1280, 800, rgb(0x000000));
        crate::desktop::wallpaper(&mut c, &Theme::of(ThemeKind::Dark));
        c
    }

    fn ink(c: &Canvas, r: Rect) -> usize {
        let mut n = 0;
        for y in (r.y as i64)..(r.bottom() as i64) {
            for x in (r.x as i64)..(r.right() as i64) {
                if luminance(c.get(x, y).unwrap_or(0)) > 0.25 {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn the_top_bar_draws_and_registers_its_hotspots() {
        let mut s = shell();
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(ink(&c, s.top_bar()) > 200, "the top bar is empty");
        for hot in [
            Hot::Logo,
            Hot::MenuApps,
            Hot::MenuWorkspace,
            Hot::MenuEnvironment,
            Hot::StatusClock,
            Hot::StatusNotifications,
            Hot::StatusUser,
        ] {
            assert!(
                s.hotspots().rect_of(hot).is_some(),
                "{hot:?} was drawn without a hotspot"
            );
        }
    }

    #[test]
    fn every_top_bar_hotspot_lies_inside_the_top_bar() {
        // A status item whose clickable area hangs below the bar steals clicks
        // from whatever window is underneath.
        let mut s = shell();
        let mut c = canvas();
        draw(&mut c, &mut s);
        let bar = s.top_bar();
        for hot in [
            Hot::Logo,
            Hot::MenuApps,
            Hot::StatusClock,
            Hot::StatusNetwork,
            Hot::StatusAudio,
            Hot::StatusBattery,
            Hot::StatusPower,
            Hot::StatusNotifications,
            Hot::StatusUser,
        ] {
            let Some(r) = s.hotspots().rect_of(hot) else {
                continue;
            };
            assert!(
                r.bottom() <= bar.bottom() + 6.0,
                "{hot:?} extends to {} below a {}-tall bar",
                r.bottom(),
                bar.h
            );
        }
    }

    #[test]
    fn the_desktop_has_a_short_bar_at_rest_and_the_launcher_offers_everything() {
        // The bar is back, and it is a strip rather than a shelf: the launcher
        // button, the pinned applications and the search bubble, in 48 pixels
        // instead of 76, with no icon grid on the wallpaper.
        let mut s = shell();
        let mut c = canvas();
        let before = canvas();
        draw(&mut c, &mut s);
        let band = Rect::new(0.0, 700.0, 1280.0, 100.0);
        assert!(
            ink(&c, band) > ink(&before, band),
            "the resting desktop drew no bar across the bottom edge"
        );
        assert!(s.hotspots().rect_of(Hot::DockToggle).is_some());
        assert!(s.hotspots().rect_of(Hot::DesktopIcon(0)).is_none());

        // Open it, and let the morph spring arrive.
        s.perform(crate::shortcuts::Action::ToggleLauncher);
        let mut t = 1_000u64;
        for _ in 0..60 {
            t = t.saturating_add(16);
            s.tick(t);
        }
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(s.hotspots().rect_of(Hot::DockSearch).is_some());
        for i in DOCK_PINNED {
            let hot = Hot::DockItem(u8::try_from(i).unwrap_or(u8::MAX));
            assert!(
                s.hotspots().rect_of(hot).is_some(),
                "pinned launcher item {i} is not clickable once the launcher is open"
            );
        }
    }

    #[test]
    fn the_power_menu_offers_four_transitions_inside_the_screen() {
        // The power glyph opens four rows -- lock, sleep, shut down, restart --
        // and every one of them has to be on screen to be clickable. A menu
        // that runs off the right edge is the defect this measures.
        let mut s = shell();
        s.perform(crate::shortcuts::Action::TogglePowerMenu);
        let mut t = 1_000u64;
        for _ in 0..60 {
            t = t.saturating_add(16);
            s.tick(t);
        }
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        for label in ["Lock", "Sleep", "Shut down", "Restart"] {
            let want = crate::i18n::text(label).into_owned();
            assert!(
                runs.contains(&want),
                "{label} is not in the power menu: {runs:?}"
            );
        }
        for i in 0..4u8 {
            let r = s
                .hotspots()
                .rect_of(Hot::PowerItem(i))
                .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));
            assert!(r.w > 0.0, "power row {i} was not drawn");
            assert!(
                r.right() <= 1280.0 && r.bottom() <= 800.0,
                "power row {i} runs off the screen: {r:?}"
            );
        }
    }

    #[test]
    fn the_system_bars_centre_names_the_focused_application() {
        // The bar's centre is the one place that can say which application the
        // next keystroke reaches. It used to spend that room on the workspace
        // number, which the pips beside it already say, so the text is checked
        // on a real frame: the application's name is on the bar, and no
        // "Workspace N" is.
        let mut s = shell();
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        // The expectation goes through the same catalogue the drawing does.
        // `App::title()` is the English name; the bar draws whatever
        // `i18n::text` makes of it, and with Thai selected (the default) those
        // are different strings. Comparing the run against the English name
        // asserts a locale the shell was not drawing in, and says "the centre is
        // wrong" about a centre that is right.
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        let desktop = crate::i18n::text("Desktop").into_owned();
        assert!(
            runs.contains(&desktop),
            "nothing focused names the desktop (expected {desktop:?}): {runs:?}"
        );

        s.open_app(App::Files);
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        let files = crate::i18n::text(App::Files.title()).into_owned();
        assert!(
            runs.contains(&files),
            "the centre does not name the focused application \
             (expected {files:?}): {runs:?}"
        );
        assert!(
            !runs
                .iter()
                .any(|r| r.starts_with("Workspace ") && r.len() <= 12),
            "the centre is still a workspace number: {runs:?}"
        );
    }

    #[test]
    fn a_long_label_in_a_menu_is_elided_to_the_width_it_has() {
        // Text longer than its row is cut with an ellipsis rather than drawn
        // past the panel's edge: the row is the space the label has, and a
        // label that overflows makes a menu look broken.
        let mut s = shell();
        let long = "an environment name nobody would ever type in full";
        s.environments = vec![crate::shell::EnvRow {
            token: "a".to_owned(),
            title: long.to_owned(),
            state: "running".to_owned(),
            active: true,
            hotkey: "1".to_owned(),
        }];
        s.perform(crate::shortcuts::Action::ToggleQuickSettings);
        let mut t = 1_000u64;
        for _ in 0..60 {
            t = t.saturating_add(16);
            s.tick(t);
        }
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(
            !runs.iter().any(|r| r == long),
            "the long label was drawn in full over the panel: {runs:?}"
        );
        assert!(
            runs.iter().any(|r| r.ends_with("...")),
            "the long label was not elided with an ellipsis: {runs:?}"
        );
    }

    #[test]
    fn a_window_draws_its_chrome_where_the_hit_test_expects_it() {
        let mut s = shell();
        s.open_app(App::Activity);
        let mut c = canvas();
        draw(&mut c, &mut s);
        let id = s.wm.focused().unwrap_or(0);
        let frame =
            s.wm.window(id)
                .map(|w| w.frame)
                .unwrap_or(litric_wm::Rect::ZERO);
        let b = s.wm.chrome().button_rect(&frame, 0);
        // The close dot is drawn filled, so its centre is bright.
        assert!(
            luminance(c.get(b.center().0 as i64, b.center().1 as i64).unwrap_or(0)) > 0.2,
            "no close button where the hit test says it is"
        );
        assert_eq!(
            s.wm.hit_test(b.center().0, b.center().1).map(|h| h.zone),
            Some(HitZone::Close)
        );
    }

    #[test]
    fn breadcrumbs_drop_their_head_before_their_tail() {
        // Three 100 px crumbs and two 8 px separators: 316 px in total.
        let widths = [100.0, 100.0, 100.0];
        assert_eq!(crumb_window(400.0, &widths, 8.0, 20.0), (0, false));
        // 250 px cannot hold all three; the head goes and the tail stays.
        assert_eq!(crumb_window(250.0, &widths, 8.0, 20.0), (1, true));
        // One crumb with no room: it is kept and elided, not hidden.
        assert_eq!(crumb_window(20.0, &[100.0], 8.0, 20.0), (0, true));
        assert_eq!(crumb_window(20.0, &[], 8.0, 20.0), (0, false));
    }

    #[test]
    fn the_activity_window_shows_unavailable_rather_than_zero() {
        let mut s = shell();
        s.facts.gpu_fraction = None;
        s.open_app(App::Activity);
        let mut c = canvas();
        draw(&mut c, &mut s);
        // Not a pixel assertion: the string is what matters, and the renderer
        // is given it by SystemFacts. This pins the contract.
        assert_eq!(SystemFacts::or_unknown(s.facts.gpu_fraction), "Unavailable");
    }

    /// Reveal the dock the way a user does: rest the pointer on the bottom
    /// edge and let the spring arrive, then expand it.
    fn open_launcher(s: &mut Shell) {
        let _ = s.handle(
            &litric_input::InputEvent::PointerMove {
                at: litric_input::event::Point::new(640.0, 795.0),
                delta: litric_input::event::Point::new(1.0, 1.0),
            },
            1_000,
        );
        for _ in 0..60 {
            s.tick(1_016);
        }
        s.perform(Action::ToggleLauncher);
        // The morph is a spring: the panel is clickable from the first ticked
        // frame, and the test ticks so that it is fully out.
        for _ in 0..60 {
            s.tick(2_000);
        }
    }

    #[test]
    fn the_launcher_is_the_dock_expanded_and_registers_a_tile_for_each_app() {
        let mut s = shell();
        open_launcher(&mut s);
        let mut c = canvas();
        draw(&mut c, &mut s);
        for i in 0..App::ALL.len() {
            assert!(
                s.hotspots().rect_of(Hot::LauncherItem(i)).is_some(),
                "launcher tile {i} has no hotspot"
            );
        }
        // The dock's own controls are still there, in the same surface: the
        // pill did not become a second thing, it grew.
        assert!(s.hotspots().rect_of(Hot::DockToggle).is_some());
        assert!(s.hotspots().rect_of(Hot::DockSearch).is_some());
        assert!(s.hotspots().rect_of(Hot::DockLock).is_some());
    }

    #[test]
    fn a_launcher_search_with_no_matches_says_so_and_keeps_the_field() {
        let mut s = shell();
        open_launcher(&mut s);
        let _ = s.handle(
            &litric_input::InputEvent::TextInput {
                text: "zzzz".to_owned(),
            },
            3_000,
        );
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(s.hotspots().rect_of(Hot::LauncherItem(0)).is_none());
        // The X that collapses the search: a launcher with no matches must
        // still offer a way back, and Backspace alone is not discoverable.
        assert!(s.hotspots().rect_of(Hot::DockCollapse).is_some());
    }

    #[test]
    fn quick_settings_registers_every_control_it_draws() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        // The real loop ticks between input and draw; the spring advances
        // there, and the panel is clickable from the first ticked frame. The
        // tick has to name the time that passed — one 60 Hz frame — because
        // the spring moves by the time the tick gives it, not by the number of
        // times it is called.
        s.tick_after(16);
        let mut c = canvas();
        draw(&mut c, &mut s);
        for i in 0..4u8 {
            assert!(
                s.hotspots().rect_of(Hot::QuickToggle(i)).is_some(),
                "toggle {i}"
            );
            assert!(
                s.hotspots().rect_of(Hot::QuickPower(i)).is_some(),
                "power {i}"
            );
        }
    }

    /// Everything the quick-settings panel can show, so that the layout is
    /// measured at its tallest.
    fn quick_shell_with_a_full_panel() -> Shell {
        let mut s = shell();
        s.facts.net_ifaces = vec![NetIface {
            name: "enp0s2".to_owned(),
            state: "up".to_owned(),
            mac: "02:00:00:00:00:01".to_owned(),
            wireless: false,
        }];
        s.facts.bt_state = Some("ready".to_owned());
        s.facts.bt_adapters = vec!["hci0 00:1a:7d:da:71:13".to_owned()];
        s.quick.bt_powered = Some(true);
        s.quick.volume = Some(40);
        s.quick.net_note =
            "enp0s2 was asked to come down; the state below is read from the kernel.".to_owned();
        s.quick.bt_note =
            "the radio was asked to turn on; the state below is what BlueZ reports.".to_owned();
        s.quick.audio_note = "Muted, as the sound server confirmed.".to_owned();
        s
    }

    #[test]
    fn no_panel_row_is_drawn_outside_the_panel() {
        // The panel's height is a sum of the rows it draws, and this is what
        // keeps the sum honest: the constant it replaced was already shorter
        // than its own content, so the last section was drawn into the clip that
        // reveals the panel and never appeared -- and a row outside the panel is
        // a row no click can reach.
        let mut s = quick_shell_with_a_full_panel();
        s.perform(Action::ToggleQuickSettings);
        s.tick_after(16);
        let mut c = canvas();
        draw(&mut c, &mut s);
        let t = s.tokens.clone();
        let panel = quick_panel_rect(&s, &t);
        // And the panel must fit the screen rather than being clamped: a clamped
        // panel has rows below its last visible pixel.
        let room = s.wm.screen().h - s.top_bar().h - t.px(20.0);
        let needed = quick_content_height(&s, &t);
        assert!(
            needed <= room,
            "the panel needs {needed} pixels and has {room}"
        );

        let inside = |r: Rect| {
            r.x >= panel.x - 0.5
                && r.right() <= panel.right() + 0.5
                && r.y >= panel.y - 0.5
                && r.bottom() <= panel.bottom() + 0.5
        };
        let mut checked = 0;
        let mut check = |s: &Shell, hot: Hot| {
            let r = s
                .hotspots()
                .rect_of(hot)
                .unwrap_or_else(|| panic!("{hot:?} was not registered"));
            assert!(inside(r), "{hot:?} at {r:?} is outside {panel:?}");
            checked += 1;
        };
        for i in 0..4u8 {
            check(&s, Hot::QuickPower(i));
            check(&s, Hot::QuickToggle(i));
        }
        for hot in [
            Hot::QuickLinkToggle,
            Hot::QuickLinkReconnect,
            Hot::QuickBluetoothPower,
            Hot::QuickVolumeDown,
            Hot::QuickVolumeUp,
            Hot::QuickVolumeMute,
        ] {
            check(&s, hot);
        }
        for i in 0..2u8 {
            check(&s, Hot::QuickEnvironment(i));
        }
        assert!(checked >= 16, "only {checked} controls were checked");
    }

    #[test]
    fn the_panel_offers_no_control_where_the_machine_reports_no_provider() {
        // A machine with no interface, no radio and no answer from the sound
        // server: the rows say so and nothing is clickable. This is the rule
        // WP-05 is accepted on -- no tile without a provider.
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        s.tick_after(16);
        let mut c = canvas();
        draw(&mut c, &mut s);
        for hot in [
            Hot::QuickLinkToggle,
            Hot::QuickLinkReconnect,
            Hot::QuickBluetoothPower,
            Hot::QuickVolumeDown,
            Hot::QuickVolumeUp,
            Hot::QuickVolumeMute,
        ] {
            assert!(
                s.hotspots().rect_of(hot).is_none(),
                "{hot:?} is offered with nothing behind it"
            );
        }
    }

    #[test]
    fn the_notification_list_draws_when_empty_and_when_full() {
        let mut s = shell();
        s.perform(Action::ToggleNotifications);
        s.tick_after(16); // advance the panel's spring, as the loop does
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(s.hotspots().rect_of(Hot::NotificationsClear).is_none());

        s.notify(
            "test",
            "Disk almost full",
            "12% remaining",
            Priority::High,
            None,
        );
        let mut c2 = canvas();
        draw(&mut c2, &mut s);
        assert!(s.hotspots().rect_of(Hot::NotificationItem(0)).is_some());
        assert!(s.hotspots().rect_of(Hot::NotificationsClear).is_some());
    }

    #[test]
    fn the_switcher_lists_one_row_per_window() {
        let mut s = shell();
        s.open_app(App::Activity);
        s.open_app(App::Files);
        s.perform(Action::NextWindow);
        // The switcher unfolds from its middle now, so the first frame of the
        // spring is genuinely empty. The loop ticks before it draws, and so
        // does this test.
        s.tick_after(16);
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(s.hotspots().rect_of(Hot::SwitcherItem(0)).is_some());
        assert!(s.hotspots().rect_of(Hot::SwitcherItem(1)).is_some());
        assert!(s.hotspots().rect_of(Hot::SwitcherItem(2)).is_none());
    }

    #[test]
    fn the_diagnostic_overlay_only_appears_when_asked_for() {
        let mut s = shell();
        let mut c = canvas();
        draw(&mut c, &mut s);
        let quiet = ink(&c, Rect::new(0.0, 400.0, 340.0, 400.0));

        s.diagnostics = true;
        let mut c2 = canvas();
        draw(&mut c2, &mut s);
        let loud = ink(&c2, Rect::new(0.0, 400.0, 340.0, 400.0));
        assert!(loud > quiet + 100, "the diagnostic overlay drew nothing");
    }

    #[test]
    fn drawing_an_unimplemented_application_says_so() {
        // The rule the whole project runs on: an empty window would look like a
        // bug, and a fake file list would be a lie.
        let mut s = shell();
        s.open_app(App::Files);
        let mut c = canvas();
        draw(&mut c, &mut s);
        let id = s.wm.focused().unwrap_or(0);
        let frame =
            s.wm.window(id)
                .map(|w| w.frame)
                .unwrap_or(litric_wm::Rect::ZERO);
        let body = Rect::new(frame.x, frame.y + 40.0, frame.w, frame.h - 40.0);
        assert!(ink(&c, body) > 150, "the placeholder drew nothing");
    }

    #[test]
    fn the_input_page_names_the_keyboard_that_is_attached() {
        // A Settings page that says "input: interactive" regardless of what is
        // plugged in is the page a person opens when typing does nothing, and it
        // tells them nothing. The names come from the kernel, so the test hands
        // the shell the same rows the platform would and asks for them back.
        let mut s = shell();
        s.settings.page = crate::settings::Page::Input;
        s.facts.input_devices = vec![
            crate::shell::InputDevice {
                node: "event0".to_owned(),
                name: "QEMU USB Keyboard".to_owned(),
                kind: "keyboard".to_owned(),
            },
            crate::shell::InputDevice {
                node: "event2".to_owned(),
                name: "QEMU USB Tablet".to_owned(),
                kind: "tablet".to_owned(),
            },
        ];
        s.open_app(App::Settings);
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(
            runs.iter().any(|r| r == "QEMU USB Keyboard"),
            "the attached keyboard is not named: {runs:?}"
        );
        assert!(
            runs.iter()
                .any(|r| r.contains("keyboard") || r.contains("Keyboard")),
            "the device class is not shown: {runs:?}"
        );
    }

    #[test]
    fn the_bluetooth_page_says_no_radio_rather_than_unavailable() {
        // This is the emulated machine's actual state: QEMU has no Bluetooth
        // radio to give the guest. "Unavailable" would blame the machine; the
        // sentence has to blame the absence of hardware.
        let mut s = shell();
        s.settings.page = crate::settings::Page::Bluetooth;
        s.facts.bt_state = Some("no-adapter".to_owned());
        s.facts.bt_daemon = Some(false);
        s.open_app(App::Settings);
        let mut c = canvas();
        c.record_text(true);
        draw(&mut c, &mut s);
        let runs: Vec<String> = c.text_runs().iter().map(|r| r.text.clone()).collect();
        let joined = runs.join(" | ");
        assert!(
            joined.contains("No radio") || joined.contains("ไม่มีวิทยุ"),
            "the page does not say there is no radio: {joined}"
        );
    }

    #[test]
    fn drawing_is_stable_across_frames_when_nothing_changes() {
        let mut s = shell();
        s.open_app(App::Activity);
        let mut a = canvas();
        draw(&mut a, &mut s);
        let mut b = canvas();
        draw(&mut b, &mut s);
        assert_eq!(a.pixels(), b.pixels(), "two identical frames differ");
    }

    #[test]
    fn drawing_survives_a_tiny_display() {
        let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 0.6);
        let mut s = Shell::new(tokens, 320.0, 200.0, Regional::fallback("test"));
        s.mode = DesktopMode::Ready;
        s.open_app(App::Activity);
        s.perform(Action::ToggleLauncher);
        let mut c = Canvas::new(320, 200, rgb(0x000000));
        draw(&mut c, &mut s);
        assert_eq!(c.width(), 320);
    }
}
