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

use crate::apps::{FileKind, FilesHot, SortKey};
use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::icons::{self, Icon};
use crate::shell::{Hot, Overlay, PowerProfile, Priority, Shell};
use crate::shortcuts::{Action, App};
use crate::ui::{self, Material, State, Tone};

/// Draw the whole desktop.
///
/// The caller has already restored the wallpaper into whatever regions are
/// dirty; this repaints every element over it. Elements outside the damaged
/// area are redrawn identically, which costs a little arithmetic and removes an
/// entire class of bug where a partly-drawn frame is presented.
pub fn draw(canvas: &mut Canvas, shell: &mut Shell) {
    shell.hotspots_mut().clear();
    draw_desktop_icons(canvas, shell);
    draw_windows(canvas, shell);
    draw_top_bar(canvas, shell);
    draw_dock(canvas, shell);
    // Overlays (launcher, switcher, quick settings, notifications) draw their own
    // opaque surfaces above everything else, so text under them is covered, not
    // collided with. Tagging them as their own surface lets the layout audit say
    // so instead of reporting a menu over a window as a defect.
    let surface = canvas.set_surface(u32::MAX);
    draw_overlay(canvas, shell);
    canvas.set_surface(surface);
    draw_banners(canvas, shell);
    draw_context_menu(canvas, shell);
    draw_desktop_dialog(canvas, shell);
    if shell.diagnostics {
        draw_diagnostics(canvas, shell);
    }
    let p = shell.pointer();
    ui::cursor(canvas, p.x, p.y, shell.tokens.scale);
}

// ---------------------------------------------------------------------------
// Desktop icons + context menu + desktop dialog
// ---------------------------------------------------------------------------

/// The desktop's icon grid: two columns, icons top-down from the top left,
/// labels under each. Labels elide before they can collide with a neighbour.
fn draw_desktop_icons(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let top = shell.top_bar().h + t.sp(Space::Lg);
    let cell_w = t.px(92.0);
    let cell_h = t.px(88.0);
    let x0 = t.sp(Space::Lg);
    let grid = shell.desktop_icons.clone();
    for (i, item) in grid.items.iter().enumerate() {
        let col = i.checked_div(4).unwrap_or(0);
        let row = i % 4;
        let r = Rect::new(
            x0 + (col as f32) * cell_w,
            top + (row as f32) * cell_h,
            cell_w - t.px(8.0),
            cell_h - t.px(8.0),
        );
        let hot = Hot::DesktopIcon(i);
        let selected = grid.selected == Some(i);
        let hovered = shell.hovered() == Some(hot);
        if selected || hovered {
            canvas.fill_round_rect(
                r,
                t.radius(Radius::Md),
                with_alpha(rgb(0xFFFFFF), if selected { 0.14 } else { 0.07 }),
            );
        }
        let icon = match &item.target {
            crate::apps::IconTarget::App(app) => icon_for(*app),
            crate::apps::IconTarget::Folder(_) => Icon::Files,
        };
        let box_ = Rect::new(
            r.center().0 - t.px(15.0),
            r.y + t.px(8.0),
            t.px(30.0),
            t.px(30.0),
        );
        icons::draw(
            canvas,
            icon,
            box_,
            if selected { t.accent() } else { t.text() },
            1.1,
        );
        let label_style = Style::new(t.text_px(TypeStep::Caption), t.text());
        let shown = ui::elide(&item.label, &label_style, r.w - t.px(4.0));
        let w = font::measure(&shown, &label_style);
        font::draw(
            canvas,
            &shown,
            r.center().0 - w / 2.0,
            box_.bottom() + t.px(14.0),
            &label_style,
        );
        // An icon beneath a window is invisible — the window paints over it
        // — and must be unclickable: registering its hotspot would let a
        // press on the window's title bar or bare body fall through to a
        // shortcut the user cannot even see. Skip covered icons entirely.
        if shell.wm.hit_test(r.center().0, r.center().1).is_none() {
            shell.hotspots_mut().push(r, hot);
        }
    }
}

/// The open context menu, if any. Drawn above everything but the pointer.
fn draw_context_menu(canvas: &mut Canvas, shell: &mut Shell) {
    let Some(menu) = shell.context_menu.clone() else {
        return;
    };
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
                let hovered = shell.hovered() == Some(hot);
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

/// The desktop's own modal dialog (New folder from the wallpaper menu).
fn draw_desktop_dialog(canvas: &mut Canvas, shell: &mut Shell) {
    let Some(crate::apps::DialogKind::NewFolder) = shell.desktop_dialog else {
        return;
    };
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    ui::surface(
        canvas,
        &t,
        Rect::new(0.0, 0.0, screen.w, screen.h),
        0.0,
        Material::Scrim,
    );
    let w = t.px(340.0);
    let h = t.px(150.0);
    let panel = Rect::new((screen.w - w) / 2.0, (screen.h - h) / 2.0, w, h);
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);
    // The click-away hotspot goes in FIRST so the buttons, registered later,
    // win the hit test (`hit` picks the last registration).
    shell.hotspots_mut().push(panel, Hot::Desktop);
    let s = t.style_label();
    font::draw(
        canvas,
        "New folder",
        panel.x + t.sp(Space::Md),
        panel.y + t.px(28.0),
        &s,
    );
    let field = Rect::new(
        panel.x + t.sp(Space::Md),
        panel.y + t.px(46.0),
        panel.w - t.sp(Space::Md) * 2.0,
        t.px(32.0),
    );
    ui::text_field(
        canvas,
        &t,
        field,
        &shell.desktop_dialog_field,
        "Folder name",
        true,
    );
    let ok = Rect::new(
        panel.right() - t.px(88.0),
        panel.bottom() - t.px(38.0),
        t.px(76.0),
        t.px(26.0),
    );
    ui::button(canvas, &t, ok, "Create", ui::State::Normal, Tone::Accent);
    shell.hotspots_mut().push(ok, Hot::DialogConfirm);
    let no = Rect::new(ok.x - t.px(84.0), ok.y, t.px(76.0), t.px(26.0));
    ui::button(canvas, &t, no, "Cancel", ui::State::Normal, Tone::Neutral);
    shell.hotspots_mut().push(no, Hot::DialogCancel);
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------

/// The single system bar. There is no bottom dock: one bar carries the
/// launcher, the workspace, the environment and every status item, which is
/// what stops the two halves of a desktop from disagreeing about where things
/// live.
fn draw_top_bar(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let bar = shell.top_bar();
    ui::surface(canvas, &t, bar, 0.0, Material::Glass);

    let h = bar.h;
    let pad = t.px(14.0);
    let mut x = pad;

    // --- left: the mark, then the menus ---
    let logo = Rect::new(x, (h - t.px(20.0)) / 2.0, t.px(20.0), t.px(20.0));
    let logo_hit = Rect::new(x - t.px(6.0), 0.0, t.px(32.0), h);
    icons::draw(
        canvas,
        Icon::Litric,
        logo,
        if shell.hovered() == Some(Hot::Logo) {
            t.accent()
        } else {
            t.text()
        },
        1.1,
    );
    shell.hotspots_mut().push(logo_hit, Hot::Logo);
    x += t.px(30.0);

    // How much room the menus have is decided by the status cluster on the
    // right, which is laid out from the bar's right edge inwards. That has to be
    // known *before* the menus are drawn: a menu drawn past the clock is a menu
    // drawn over the time, which is what "Environment" did at 480 pixels wide
    // once the face's side bearings made the word its real width.
    let menu_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let clock_w = font::measure(&shell.regional.clock(shell.clock_seconds()), &menu_style);
    let status_icons = 3u32
        .saturating_add(u32::from(shell.facts.battery_percent.is_some()))
        .saturating_add(u32::from(shell.facts.network.as_deref() == Some("offline")))
        .saturating_add(1);
    let right_cluster = t.px(22.0 + 18.0)
        + clock_w
        + t.px(14.0)
        + t.px(24.0) * f32::from(u16::try_from(status_icons).unwrap_or(3))
        + t.px(8.0);
    let menus_end = (bar.w - pad - right_cluster - t.px(12.0)).max(x);
    let mut dropped = 0usize;
    for (label, hot) in [
        ("Apps", Hot::MenuApps),
        ("Workspace", Hot::MenuWorkspace),
        ("Environment", Hot::MenuEnvironment),
    ] {
        let w = font::measure(label, &menu_style);
        if x + w > menus_end {
            dropped = dropped.saturating_add(1);
            continue;
        }
        let r = Rect::new(x - t.px(6.0), t.px(4.0), w + t.px(12.0), h - t.px(8.0));
        if shell.hovered() == Some(hot) {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(rgb(0xFFFFFF), 0.09));
        }
        font::draw(
            canvas,
            label,
            x,
            h / 2.0 + menu_style.size * 0.36,
            &menu_style,
        );
        shell.hotspots_mut().push(r, hot);
        x += w + t.px(20.0);
    }
    // Anything that did not fit is reachable through the launcher, which is what
    // these menus open anyway (Apps toggles it, Workspace opens the monitor,
    // Environment opens quick settings). A chevron is drawn rather than a menu
    // silently missing, so the bar never claims to have fewer choices than it
    // has.
    if dropped > 0 && x + t.px(24.0) <= menus_end {
        let r = Rect::new(x - t.px(6.0), t.px(4.0), t.px(24.0), h - t.px(8.0));
        if shell.hovered() == Some(Hot::MenuApps) {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(rgb(0xFFFFFF), 0.09));
        }
        font::draw(
            canvas,
            "\u{203A}",
            x,
            h / 2.0 + menu_style.size * 0.36,
            &menu_style,
        );
        shell.hotspots_mut().push(r, Hot::MenuApps);
    }

    // --- centre: workspace pips and the focused window ---
    let ws_count = shell.wm.workspaces().len();
    let active_ws = shell.wm.active_workspace();
    let pip = t.px(7.0);
    let gap = t.px(6.0);
    let pips_w = (ws_count as f32) * pip + ((ws_count.saturating_sub(1)) as f32) * gap;
    let title = shell
        .wm
        .focused()
        .and_then(|id| shell.wm.window(id))
        .map_or_else(
            || format!("Workspace {}", active_ws.saturating_add(1)),
            |w| w.title.clone(),
        );
    let title_style = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let left_end = x;
    let title_w = font::measure(&title, &title_style);
    let centre_w = pips_w + t.px(12.0) + title_w;
    // Three clusters, and a bar too narrow for all three does not get a squeezed
    // middle one: nothing in the middle is drawn at all. At 560 pixels the
    // centred title was drawn over the menus (32x18px) and, once clamped inward,
    // over the clock (39x18px) -- which is why clamping is not the fix. The
    // focused window is named by its own title bar and by the switcher, so the
    // middle cluster is the one that can go. The supported minimum is 960 wide
    // (a 0.75 scale of 1280), where all three fit.
    if bar.w >= left_end + centre_w + t.px(300.0) {
        let mut cx = (bar.w - centre_w) / 2.0;
        for i in 0..ws_count {
            let r = Rect::new(cx, (h - pip) / 2.0, pip, pip);
            let on = i == active_ws;
            canvas.fill_round_rect(
                r,
                pip / 2.0,
                if on {
                    t.accent()
                } else {
                    with_alpha(rgb(0xFFFFFF), 0.25)
                },
            );
            // The active workspace also gets a ring, so the state does not
            // depend on telling the accent apart from grey.
            if on {
                canvas.stroke_round_rect(
                    r.inflate(t.px(2.5)),
                    pip,
                    1.0,
                    with_alpha(t.accent(), 0.55),
                );
            }
            shell
                .hotspots_mut()
                .push(r.inflate(t.px(4.0)), Hot::Workspace(i as u8));
            cx += pip + gap;
        }
        cx += t.px(12.0);
        font::draw(
            canvas,
            &title,
            cx,
            h / 2.0 + title_style.size * 0.36,
            &title_style,
        );
    }

    // --- right: status, laid out from the edge inwards ---
    let mut rx = bar.w - pad;
    let clock = shell.regional.clock(shell.clock_seconds());
    let clock_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let clock_w = font::measure(&clock, &clock_style);

    rx -= t.px(22.0);
    let user_r = Rect::new(rx, (h - t.px(18.0)) / 2.0, t.px(18.0), t.px(18.0));
    icons::draw(canvas, Icon::User, user_r, t.text_dim(), 1.0);
    shell
        .hotspots_mut()
        .push(user_r.inflate(t.px(5.0)), Hot::StatusUser);

    rx -= clock_w + t.px(14.0);
    font::draw(
        canvas,
        &clock,
        rx,
        h / 2.0 + clock_style.size * 0.36,
        &clock_style,
    );
    shell.hotspots_mut().push(
        Rect::new(rx - t.px(6.0), 0.0, clock_w + t.px(12.0), h),
        Hot::StatusClock,
    );

    // Status icons, right to left. Each is a hotspot that opens something.
    let icon_size = t.px(16.0);
    let step = t.px(24.0);
    let count = shell.notifications.len();
    let bell_tone = match shell.notifications.peak_priority() {
        Some(Priority::Critical) => Some(Tone::Bad),
        Some(Priority::High) => Some(Tone::Warning),
        Some(_) => Some(Tone::Accent),
        None => None,
    };

    let status = |shell: &mut Shell,
                  canvas: &mut Canvas,
                  rx: &mut f32,
                  icon: Icon,
                  hot: Hot,
                  tone: Option<Tone>| {
        *rx -= step;
        let r = Rect::new(*rx, (h - icon_size) / 2.0, icon_size, icon_size);
        let color = tone.map_or_else(|| t.text_dim(), |x| t.tone(x));
        if shell.hovered() == Some(hot) {
            canvas.fill_round_rect(
                r.inflate(t.px(5.0)),
                t.radius(Radius::Sm),
                with_alpha(rgb(0xFFFFFF), 0.09),
            );
        }
        icons::draw(canvas, icon, r, color, 1.0);
        shell.hotspots_mut().push(r.inflate(t.px(5.0)), hot);
        r
    };

    let bell = status(
        shell,
        canvas,
        &mut rx,
        Icon::Notifications,
        Hot::StatusNotifications,
        bell_tone,
    );
    if count > 0 {
        // A count badge, not just a coloured bell: the number is the
        // information and colour alone cannot carry it.
        let label = if count > 9 {
            "9+".to_owned()
        } else {
            count.to_string()
        };
        let s = Style::bold(t.text_px(TypeStep::Caption) * 0.85, rgb(0xFFFFFF));
        let w = font::measure(&label, &s);
        // The badge grows *leftwards* from the bell's right edge. Growing right
        // from the icon meant a badge on the bell -- the last icon before the
        // clock -- reached into the clock's text, and one notification was
        // enough: the "1" was drawn over the last letter of "00:00 UTC" (1x11px
        // measured). Nothing is drawn to the right of the bell, so the room a
        // badge may use is the room the icon itself has.
        let bw = w + t.px(8.0);
        let br = Rect::new(bell.right() - bw, bell.y - t.px(3.0), bw, t.px(12.0));
        canvas.fill_round_rect(br, br.h / 2.0, t.tone(bell_tone.unwrap_or(Tone::Accent)));
        font::draw(canvas, &label, br.x + t.px(4.0), br.y + br.h * 0.76, &s);
    }

    status(shell, canvas, &mut rx, Icon::Power, Hot::StatusPower, None);

    // The Defender's shield: coloured when the last scan says something.
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
    status(
        shell,
        canvas,
        &mut rx,
        Icon::Shield,
        Hot::MenuRow(crate::apps::MenuAction::ScanNow),
        shield_tone,
    );

    // Battery, audio and network appear only when the machine actually has
    // them — icons that mean "nothing was measured" are noise, and the quick
    // settings panel is one click away for everything.
    if shell.facts.battery_percent.is_some() {
        let battery_tone = shell.facts.battery_percent.and_then(|p| {
            if p <= 10 {
                Some(Tone::Bad)
            } else if p <= 25 {
                Some(Tone::Warning)
            } else {
                None
            }
        });
        status(
            shell,
            canvas,
            &mut rx,
            Icon::Battery,
            Hot::StatusBattery,
            battery_tone,
        );
    }

    if shell.facts.network.as_deref() == Some("offline") {
        // Only the exception is worth an icon: a machine that is online shows
        // nothing, a machine that cannot reach the network shows why.
        status(
            shell,
            canvas,
            &mut rx,
            Icon::Network,
            Hot::StatusNetwork,
            Some(Tone::Warning),
        );
    }
    status(shell, canvas, &mut rx, Icon::Audio, Hot::StatusAudio, None);
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
        font::draw(
            canvas,
            &shown,
            frame.x + t.px(74.0),
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
    let bridge = match crate::fsbridge::FsBridge::open(std::path::Path::new("/"), zone) {
        Ok(b) => b,
        Err(e) => {
            shell.files.error = Some(e.to_string());
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
        Err(e) => shell.files.error = Some(e.to_string()),
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

fn draw_files(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    refresh_files(shell);
    let t = shell.tokens.clone();
    let app = shell.files.clone();

    // Layout: places | main column (crumbs, commands, table) | details.
    let places_w = t.px(132.0);
    let details_w = if app.details { t.px(180.0) } else { 0.0 };
    let table_x = body.x + places_w;
    let table_w = body.w - places_w - details_w;

    // --- navigation pane (Quick access) ---
    ui::surface(
        canvas,
        &t,
        Rect::new(body.x, body.y, places_w, body.h),
        0.0,
        ui::Material::Elevated,
    );
    let mut py = body.y + t.sp(Space::Md);
    let s_label = t.style_label();
    font::draw(
        canvas,
        "Quick access",
        body.x + t.sp(Space::Sm),
        py + t.px(10.0),
        &s_label,
    );
    py += t.px(22.0);
    for (i, (place, label)) in app.places.iter().enumerate() {
        let r = Rect::new(body.x + t.px(4.0), py, places_w - t.px(8.0), t.px(20.0));
        let selected = app.place_index == Some(i) || place == &app.path;
        if selected {
            canvas.fill_round_rect(r, t.radius(Radius::Sm), with_alpha(t.text(), 0.08));
        }
        let s = Style::new(
            t.text_px(TypeStep::Footnote),
            if selected { t.text() } else { t.text_dim() },
        );
        font::draw(canvas, label, r.x + t.px(8.0), r.y + t.px(13.0), &s);
        shell.hotspots_mut().push(r, Hot::Files(FilesHot::Place(i)));
        py += t.px(22.0);
    }

    // --- breadcrumb bar + history buttons ---
    let bar = Rect::new(table_x, body.y, table_w, t.px(30.0));
    ui::surface(canvas, &t, bar, 0.0, ui::Material::Flat);
    let mut bx = bar.x + t.px(6.0);
    for (label, hot) in [
        ("‹", FilesHot::Back),
        ("›", FilesHot::Forward),
        ("↑", FilesHot::Up),
    ] {
        let r = Rect::new(bx, bar.y + t.px(4.0), t.px(20.0), t.px(22.0));
        ui::button(canvas, &t, r, label, ui::State::Normal, Tone::Neutral);
        shell.hotspots_mut().push(r, Hot::Files(hot));
        bx += t.px(24.0);
    }
    let mut cx = bx + t.px(6.0);
    let crumbs = app.breadcrumbs();
    let s_crumb = Style::new(t.text_px(TypeStep::Footnote), t.text());
    for (i, c) in crumbs.iter().enumerate() {
        let w = font::measure(c, &s_crumb) + t.px(8.0);
        let r = Rect::new(cx, bar.y + t.px(4.0), w, t.px(22.0));
        font::draw(canvas, c, cx + t.px(4.0), r.y + t.px(14.0), &s_crumb);
        shell.hotspots_mut().push(r, Hot::Files(FilesHot::Crumb(i)));
        cx += w + t.px(2.0);
        if i.checked_add(1).is_some_and(|n| n < crumbs.len()) {
            let sep = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
            font::draw(canvas, "/", cx, r.y + t.px(14.0), &sep);
            cx += t.px(8.0);
        }
    }
    if crumbs.is_empty() {
        let sep = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(canvas, "/ (root)", cx, bar.y + t.px(14.0), &sep);
    }

    // --- command row: New / Paste / Compress / Extract, filter readout,
    // details toggle. The buttons drive the same model the context menu and
    // the shortcuts do, so all three paths stay one behaviour.
    let cmd = Rect::new(table_x, bar.bottom(), table_w, t.px(26.0));
    let s_cmd = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    let filter_note = if app.filter.is_empty() {
        format!("{} items", app.visible().len())
    } else {
        format!("filter: {} ({} items)", app.filter, app.visible().len())
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
            hx + t.px(6.0),
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

    let vis = app.visible();
    for (i, row) in vis.iter().enumerate().take(24) {
        let y = table.y + head_h + (i as f32) * row_h;
        let selected = app.selected == Some(i);
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
            font::draw(
                canvas,
                &ui::elide(text, &s, cw - t.px(12.0)),
                cx2 + t.px(6.0),
                y + t.px(15.0),
                &s,
            );
            cx2 += cw;
        }
        shell
            .hotspots_mut()
            .push(row_rect, Hot::Files(FilesHot::Row(i)));
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

/// The Files window's modal dialog: create, rename, confirm, error.
fn draw_files_dialog(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let Some(dialog) = shell.files.dialog.clone() else {
        return;
    };
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
    let Some(viewer) = shell.files.viewer.clone() else {
        return;
    };
    let t = shell.tokens.clone();
    ui::surface(canvas, &t, body, 0.0, Material::Scrim);
    let panel = body.inflate(-t.sp(Space::Lg));
    ui::surface(canvas, &t, panel, t.radius(Radius::Md), Material::Popover);
    let s_title = t.style_label();
    let shown_title = ui::elide(&viewer.path, &s_title, panel.w - t.px(90.0));
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

    let text_area = Rect::new(
        panel.x + t.sp(Space::Sm),
        panel.y + t.px(32.0),
        panel.w - t.sp(Space::Sm) * 2.0,
        panel.h - t.px(70.0),
    );
    let line_h = t.px(16.0);
    let rows = ((text_area.h / line_h) as usize).max(1);
    let (lines, total) = viewer.window(rows);
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
    let page = format!("line {} of {total}", viewer.scroll.saturating_add(1));
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
    let strip_h = t.px(30.0);
    let mut tx = body.x + t.px(4.0);
    let s_tab = Style::new(t.text_px(TypeStep::Footnote), t.text());
    for (i, tab) in app.tabs.iter().enumerate() {
        let active_tab = i == app.active;
        let w = t.px(150.0);
        let r = Rect::new(tx, body.y + t.px(2.0), w, strip_h - t.px(4.0));
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
        let shown = if tab.url.len() > 20 {
            format!("{}…", &tab.url[..17])
        } else {
            tab.url.clone()
        };
        font::draw(canvas, &shown, r.x + t.px(8.0), r.y + t.px(14.0), &s_tab);
        shell.hotspots_mut().push(r, Hot::BrowserTab(i));
        // Close ✕.
        let cr = Rect::new(
            r.right() - t.px(18.0),
            r.y + t.px(3.0),
            t.px(15.0),
            t.px(15.0),
        );
        let s_x = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        font::draw(canvas, "✕", cr.x + t.px(3.0), cr.y + t.px(10.0), &s_x);
        shell.hotspots_mut().push(cr, Hot::BrowserCloseTab(i));
        tx += w + t.px(2.0);
    }
    let plus = Rect::new(tx, body.y + t.px(5.0), t.px(24.0), t.px(20.0));
    let s_plus = Style::new(t.text_px(TypeStep::Body), t.text_dim());
    font::draw(
        canvas,
        "+",
        plus.x + t.px(7.0),
        plus.y + t.px(14.0),
        &s_plus,
    );
    shell.hotspots_mut().push(plus, Hot::BrowserNewTab);

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

    // --- page area: engine frame when one arrives, honest state otherwise ---
    let page = Rect::new(body.x, bar.bottom(), body.w, body.bottom() - bar.bottom());
    let Some(tab) = app.tab() else {
        return;
    };
    let s_head = Style::new(t.text_px(TypeStep::Title3), t.text());
    let s_body = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());

    // A frame from the engine session replaces the placeholder entirely.
    if let (Some(frame), Some((w, h))) = (shell.browser_frame.as_ref(), app.last_frame) {
        blit_engine_frame(canvas, frame, w, h, page);
        return;
    }

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
            let hovered = shell.hovered() == Some(hot);
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
fn blit_engine_frame(canvas: &mut Canvas, frame: &[u8], w: u32, h: u32, page: Rect) {
    let needed = u64::from(w)
        .checked_mul(u64::from(h))
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(usize::MAX);
    if frame.len() < needed || w == 0 || h == 0 {
        return;
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
}

/// The system monitor, filled from measured values only.
///
/// Minimal by rule, not by omission of work: a row exists only when its value
/// was measured. An unmeasurable quantity is simply absent — the interface
/// never shows a control that says it knows nothing.
fn draw_activity(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let facts = shell.facts.clone();
    // The meters and the machine's own numbers on top, the process table under
    // them: the table is the reason the window is open, so it gets the larger
    // share, and both halves say so when they run out of room.
    let table_h = (body.h * 0.52).max(t.px(160.0));
    let top_h = (body.h - table_h).max(t.px(120.0));
    crate::monitor::draw(
        canvas,
        &t,
        &facts,
        &shell.activity,
        Rect::new(body.x, body.y, body.w, top_h),
    );
    let table_rect = Rect::new(body.x, body.y + top_h, body.w, (body.h - top_h).max(0.0));
    crate::ui::separator(canvas, &t, Rect::new(body.x, table_rect.y, body.w, 1.0));
    crate::activity::table(canvas, &t, &shell.activity, table_rect);
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
        let hovered = shell.hovered() == Some(hot);
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

/// Settings: the category list. Every page is `TODO`, and the panel says so
/// rather than showing empty controls.
fn draw_settings(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let sidebar_w = body.w * 0.36;
    let sidebar = Rect::new(body.x, body.y, sidebar_w, body.h);
    canvas.fill_round_rect(sidebar, 0.0, with_alpha(rgb(0x000000), 0.16));

    const CATEGORIES: [(&str, Icon); 8] = [
        ("Appearance", Icon::Grid),
        ("Display", Icon::Window),
        ("Input", Icon::Terminal),
        ("Network", Icon::Network),
        ("Power", Icon::Power),
        ("Region", Icon::Info),
        ("Environments", Icon::Environment),
        ("Accessibility", Icon::User),
    ];

    let mut col = ui::Column::new(sidebar, t.sp(Space::Sm), t.px(2.0));
    for (i, (name, icon)) in CATEGORIES.iter().enumerate() {
        let row = col.row(t.px(34.0));
        ui::list_item(
            canvas,
            &t,
            row,
            Some(*icon),
            name,
            None,
            None,
            if i == 5 {
                State::Selected
            } else {
                State::Normal
            },
        );
    }

    // The Region page is the one that is real, because localisation is.
    let page = Rect::new(body.x + sidebar_w, body.y, body.w - sidebar_w, body.h);
    let mut c = ui::Column::new(page, t.sp(Space::Lg), t.sp(Space::Sm));
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
            let line = crate::monitor::fit(note, &ns, page.w);
            let nrow = c.row(t.px(18.0));
            font::draw(canvas, &line, nrow.x, nrow.y + t.px(12.0), &ns);
        }
    }

    c.space(t.px(10.0));
    let note = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    font::draw(
        canvas,
        "Appearance, Display, Input, Network, Power, Environments and Accessibility have no page yet.",
        c.x,
        c.cursor + t.px(14.0),
        &note,
    );
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
        _ => ("Unknown application", "No view is registered for this window."),
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

fn draw_overlay(canvas: &mut Canvas, shell: &mut Shell) {
    match shell.overlay.clone() {
        Overlay::None => {}
        Overlay::Launcher { query, selected } => draw_launcher(canvas, shell, &query, selected),
        Overlay::Switcher { selected } => draw_switcher(canvas, shell, selected),
        Overlay::QuickSettings => draw_quick_settings(canvas, shell),
        Overlay::Notifications => draw_notification_list(canvas, shell),
    }
}

fn draw_launcher(canvas: &mut Canvas, shell: &mut Shell, query: &str, selected: usize) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    ui::surface(
        canvas,
        &t,
        Rect::new(0.0, 0.0, screen.w, screen.h),
        0.0,
        Material::Scrim,
    );

    let results = App::search(query);
    let row_h = t.px(46.0);
    let row_gap = t.px(2.0);
    let w = t.px(520.0).min(screen.w * 0.8);
    // The height is computed from the layout below rather than guessed at, so
    // the last row is never clipped by a panel that was one gap too short.
    // Getting this wrong hides the last application in the launcher, which is
    // exactly the kind of bug that survives a screenshot.
    let head = t.sp(Space::Md) + t.px(38.0) + t.px(6.0) + t.sp(Space::Sm);
    let body = (results.len().max(1) as f32) * (row_h + row_gap);
    let h = (head + body + t.px(12.0)).min(screen.h * 0.85);
    let panel = Rect::new((screen.w - w) / 2.0, (screen.h - h) * 0.28, w, h);
    let radius = t.radius(Radius::Xl);
    ui::surface(canvas, &t, panel, radius, Material::Popover);

    let field = Rect::new(
        panel.x + t.sp(Space::Md),
        panel.y + t.sp(Space::Md),
        panel.w - t.sp(Space::Md) * 2.0,
        t.px(38.0),
    );
    ui::text_field(canvas, &t, field, query, "Search applications", true);
    shell.hotspots_mut().push(field, Hot::LauncherSearch);

    let saved = canvas.push_clip(panel);
    let mut col = ui::Column::new(
        Rect::new(panel.x, field.bottom() + t.px(6.0), panel.w, panel.h),
        t.sp(Space::Sm),
        row_gap,
    );
    if results.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(
            canvas,
            "No matches",
            col.x + t.px(8.0),
            col.cursor + t.px(24.0),
            &s,
        );
        canvas.set_clip(saved);
        return;
    }
    for (i, app) in results.iter().enumerate() {
        if col.cursor + row_h > panel.bottom() {
            break;
        }
        let row = col.row(row_h);
        let hot = Hot::LauncherItem(i);
        let state = if i == selected {
            State::Selected
        } else if shell.hovered() == Some(hot) {
            State::Hover
        } else {
            State::Normal
        };
        let shortcut = shell.shortcuts.label_for(Action::Open(*app));
        ui::list_item(
            canvas,
            &t,
            row,
            Some(icon_for(*app)),
            app.title(),
            Some(app.subtitle()),
            shortcut.as_deref(),
            state,
        );
        shell.hotspots_mut().push(row, hot);
    }
    canvas.set_clip(saved);
}

/// The dock: a macOS-style app bar that slides up from the bottom edge.
///
/// It reveals when the pointer rests near the bottom of the screen and
/// hides again when it leaves — the open/close the dock owes the user —
/// and it registers its hotspots only when fully revealed, so a click can
/// never pass through a half-hidden dock into whatever sits underneath.
fn draw_dock(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let reveal = shell.anims().dock.value();
    if reveal <= 0.01 {
        return;
    }
    let item = t.px(52.0);
    let icon = t.px(32.0);
    let pad = t.px(10.0);
    let apps = &crate::shortcuts::App::ALL;
    let w = item * (apps.len() as f32) + pad * 2.0;
    let h = t.px(56.0);
    let x = (screen.w - w) / 2.0;
    let y = screen.h - h + (1.0 - reveal) * h;
    let panel = Rect::new(x, y, w, h);
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Dialog);

    let mut ix = panel.x + pad;
    for (i, app) in apps.iter().enumerate() {
        let hot = Hot::DockItem(i as u8);
        let r = Rect::new(ix, panel.y + t.px(6.0), item, item);
        let hovered = shell.hovered() == Some(hot);
        if hovered {
            canvas.fill_round_rect(r, t.radius(Radius::Md), with_alpha(rgb(0xFFFFFF), 0.12));
            // The app's name above the hovered icon, the way a dock labels
            // without tooltips.
            let s = Style::new(t.text_px(TypeStep::Caption), t.text());
            let tw = font::measure(app.title(), &s);
            let lr = Rect::new(
                r.center().0 - tw / 2.0 - t.px(8.0),
                panel.y - t.px(26.0),
                tw + t.px(16.0),
                t.px(20.0),
            );
            ui::surface(canvas, &t, lr, t.radius(Radius::Sm), Material::Elevated);
            font::draw(canvas, app.title(), lr.x + t.px(8.0), lr.y + t.px(13.0), &s);
        }
        icons::draw(
            canvas,
            icon_for(*app),
            Rect::new(
                ix + (item - icon) / 2.0,
                panel.y + t.px(6.0) + (item - icon) / 2.0,
                icon,
                icon,
            ),
            t.text(),
            1.0,
        );
        // Running indicator: one accent dot under a running app.
        let running = shell.wm.all().iter().any(|w| w.app_id == app.app_id());
        if running {
            canvas.fill_circle(
                r.center().0,
                panel.bottom() - t.px(4.0),
                t.px(2.0),
                t.accent(),
            );
        }
        // Hotspots ride the animated panel: the item rects above are cut at
        // the dock's current visual position, so a click lands exactly where
        // the eye already sees a button. The gate exists only to keep a
        // fully hidden dock from eating clicks meant for the desktop under
        // it — a dock that is 60% up is a dock the user is aiming at.
        if reveal >= 0.6 {
            shell.hotspots_mut().push(r.inflate(t.px(2.0)), hot);
        }
        ix += item;
    }
}

/// The Terminal: a real command line against the guest OS.
fn draw_terminal(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let term = shell.terminal.clone();
    canvas.fill_round_rect(body, 0.0, with_alpha(rgb(0x000000), 0.55));
    let line_h = t.px(17.0);
    let mut y = body.y + t.px(10.0);
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
    let s_out = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
    for line in term.lines.iter().skip(start) {
        font::draw(canvas, line, body.x + t.px(10.0), y + line_h, &s_out);
        y += line_h;
    }
    // Prompt + input, with the caret when the field holds the keyboard.
    let prompt = if term.busy { "… " } else { "~ $ " };
    let s_in = Style::new(t.text_px(TypeStep::Body), t.text());
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
        _ => (Icon::ShieldCheck, Tone::Good),
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
        format!(
            "Definitions: {} signatures · no scan yet",
            d.definitions.max(1)
        )
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
        &format!("{} item(s)", app.count()),
        title_end + t.px(10.0),
        head.y + t.px(18.0),
        &s_count,
    );

    // Actions on the right of the header row.
    let has_sel = app.selected.is_some() && app.selected.unwrap_or(0) < app.entries.len();
    let empty_w = t.px(92.0);
    let restore_w = t.px(84.0);
    let empty_r = Rect::new(
        head.right() - empty_w,
        head.y + t.px(2.0),
        empty_w,
        t.px(24.0),
    );
    ui::button(canvas, &t, empty_r, "Empty", ui::State::Normal, Tone::Bad);
    shell.hotspots_mut().push(empty_r, Hot::TrashEmpty);
    let restore_r = Rect::new(
        empty_r.x - restore_w - t.px(8.0),
        head.y + t.px(2.0),
        restore_w,
        t.px(24.0),
    );
    ui::button(
        canvas,
        &t,
        restore_r,
        "Restore",
        if has_sel {
            ui::State::Normal
        } else {
            ui::State::Disabled
        },
        Tone::Neutral,
    );
    shell.hotspots_mut().push(restore_r, Hot::TrashRestore);

    if let Some(err) = &app.error {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.tone(Tone::Bad));
        font::draw(canvas, err, col.x, col.cursor + t.px(14.0), &s);
        return;
    }
    if app.entries.is_empty() {
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
        font::draw(canvas, "Trash is empty", col.x, col.cursor + t.px(14.0), &s);
        return;
    }
    let saved = canvas.push_clip(Rect::new(
        body.x,
        col.cursor,
        body.w,
        body.bottom() - col.cursor,
    ));
    for (i, (name, deleted, origin, is_dir)) in app.entries.iter().enumerate() {
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
        } else if shell.hovered() == Some(hot) {
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
}

fn icon_for_app_id(app_id: &str) -> Icon {
    App::ALL
        .iter()
        .find(|a| a.app_id() == app_id)
        .map_or(Icon::Window, |a| icon_for(*a))
}

fn draw_quick_settings(canvas: &mut Canvas, shell: &mut Shell) {
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let w = t.px(320.0).min(screen.w * 0.9);
    let panel = Rect::new(
        screen.w - w - t.px(12.0),
        shell.top_bar().h + t.px(8.0),
        w,
        t.px(392.0),
    );
    // The panel reveals downward under a clip that grows with its spring,
    // and its rows only become clickable once the spring is far enough
    // along that a click can only be aiming at the panel, not passing
    // through the reveal.
    let reveal = shell.anims().quick_settings.value();
    if reveal <= 0.01 {
        return;
    }
    let shown_h = panel.h * reveal;
    let saved = canvas.push_clip(Rect::new(panel.x, panel.y, panel.w, shown_h));
    ui::surface(canvas, &t, panel, t.radius(Radius::Lg), Material::Popover);

    let mut col = ui::Column::new(panel, t.sp(Space::Md), t.sp(Space::Xs));
    ui::section_label(canvas, &t, (col.x, col.cursor + t.px(10.0)), "POWER");
    col.space(t.px(18.0));

    let row = col.row(t.px(34.0));
    let seg_w = row.w / 4.0;
    for (i, p) in PowerProfile::ALL.iter().enumerate() {
        let r = Rect::new(row.x + (i as f32) * seg_w, row.y, seg_w - t.px(4.0), row.h);
        let hot = Hot::QuickPower(i as u8);
        let state = if *p == shell.quick.power {
            State::Selected
        } else if shell.hovered() == Some(hot) {
            State::Hover
        } else {
            State::Normal
        };
        ui::button(
            canvas,
            &t,
            r,
            p.title(),
            state,
            if *p == shell.quick.power {
                Tone::Accent
            } else {
                Tone::Neutral
            },
        );
        shell.hotspots_mut().push(r, hot);
    }
    col.space(t.px(10.0));
    ui::separator(canvas, &t, col.row(1.0));
    col.space(t.px(8.0));

    ui::section_label(canvas, &t, (col.x, col.cursor + t.px(10.0)), "SYSTEM");
    col.space(t.px(18.0));

    let toggles: [(&str, bool, bool); 4] = [
        (
            "Do Not Disturb",
            shell.quick.do_not_disturb,
            shell.quick.is_applied("do_not_disturb"),
        ),
        (
            "Adaptive Idle",
            shell.quick.adaptive_idle,
            shell.quick.is_applied("adaptive_idle"),
        ),
        (
            "Reduce Transparency",
            shell.quick.reduce_transparency,
            shell.quick.is_applied("reduce_transparency"),
        ),
        (
            "Reduce Motion",
            shell.quick.reduce_motion,
            shell.quick.is_applied("reduce_motion"),
        ),
    ];
    for (i, (label, on, applied)) in toggles.iter().enumerate() {
        let row = col.row(t.px(38.0));
        let hot = Hot::QuickToggle(i as u8);
        if shell.hovered() == Some(hot) {
            canvas.fill_round_rect(row, t.radius(Radius::Md), with_alpha(rgb(0xFFFFFF), 0.06));
        }
        let s = Style::new(t.text_px(TypeStep::Footnote), t.text());
        font::draw(
            canvas,
            label,
            row.x + t.px(8.0),
            row.y + row.h / 2.0 + s.size * 0.36,
            &s,
        );
        if !applied {
            // A control that only records a preference must say so.
            let n = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
            font::draw(
                canvas,
                "preference only",
                row.x + t.px(8.0),
                row.y + row.h - t.px(2.0),
                &n,
            );
        }
        let tr = Rect::new(
            row.right() - t.px(48.0),
            row.y + t.px(8.0),
            t.px(40.0),
            t.px(22.0),
        );
        ui::toggle(canvas, &t, tr, *on, State::Normal);
        shell.hotspots_mut().push(row, hot);
    }

    col.space(t.px(8.0));
    ui::separator(canvas, &t, col.row(1.0));
    col.space(t.px(8.0));
    ui::section_label(canvas, &t, (col.x, col.cursor + t.px(10.0)), "ENVIRONMENT");
    col.space(t.px(16.0));

    let envs = shell.environments.clone();
    for (i, e) in envs.iter().enumerate().take(3) {
        let row = col.row(t.px(30.0));
        let hot = Hot::QuickEnvironment(i as u8);
        let state = if e.active {
            State::Selected
        } else if shell.hovered() == Some(hot) {
            State::Hover
        } else {
            State::Normal
        };
        ui::list_item(
            canvas,
            &t,
            row,
            None,
            &e.title,
            None,
            Some(&e.hotkey),
            state,
        );
        shell.hotspots_mut().push(row, hot);
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
            if shell.hovered() == Some(Hot::NotificationsClear) {
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
            if shell.hovered() == Some(hot) {
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
    let t = shell.tokens.clone();
    let screen = shell.wm.screen();
    let recent: Vec<(String, String, Priority)> = shell
        .notifications
        .items()
        .iter()
        .filter(|n| shell.notifications.should_banner(n.priority))
        // A banner shows for anything that will dismiss itself, and for
        // criticals, which never carry an expiry because what is wrong now
        // does not stop being wrong five seconds later.
        .filter(|n| n.expires_ms.is_some() || n.priority == Priority::Critical)
        .take(3)
        .map(|n| (n.title.clone(), n.body.clone(), n.priority))
        .collect();

    let w = t.px(300.0).min(screen.w * 0.6);
    let mut y = shell.top_bar().h + t.px(10.0);
    for (title, body, priority) in recent {
        let r = Rect::new(screen.w - w - t.px(12.0), y, w, t.px(60.0));
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
        let ts = Style::new(t.text_px(TypeStep::Footnote), t.text());
        font::draw(canvas, &title, r.x + t.px(38.0), r.y + t.px(25.0), &ts);
        let bs = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        let shown = ui::elide(&body, &bs, r.w - t.px(50.0));
        font::draw(canvas, &shown, r.x + t.px(38.0), r.y + t.px(43.0), &bs);
        y += r.h + t.px(8.0);
    }
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

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::canvas::luminance;
    use crate::shell::{DesktopMode, EnvRow, SystemFacts};
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
    fn there_is_no_bottom_bar() {
        // The layout decision, enforced: nothing is drawn in the bottom strip
        // of an empty desktop.
        let mut s = shell();
        let mut c = canvas();
        let before = {
            let mut w = canvas();
            crate::desktop::wallpaper(&mut w, &Theme::of(ThemeKind::Dark));
            ink(&w, Rect::new(0.0, 700.0, 1280.0, 100.0))
        };
        draw(&mut c, &mut s);
        let after = ink(&c, Rect::new(0.0, 700.0, 1280.0, 100.0));
        // The cursor sits in the middle of the screen, so the only difference
        // in the bottom strip should be nothing at all.
        assert_eq!(after, before, "something was drawn along the bottom edge");
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

    #[test]
    fn the_launcher_draws_results_and_registers_a_row_for_each() {
        let mut s = shell();
        s.perform(Action::ToggleLauncher);
        let mut c = canvas();
        draw(&mut c, &mut s);
        for i in 0..App::ALL.len() {
            assert!(
                s.hotspots().rect_of(Hot::LauncherItem(i)).is_some(),
                "launcher row {i} has no hotspot"
            );
        }
        assert!(s.hotspots().rect_of(Hot::LauncherSearch).is_some());
    }

    #[test]
    fn a_launcher_search_with_no_matches_still_draws_a_panel() {
        let mut s = shell();
        s.perform(Action::ToggleLauncher);
        s.handle(
            &litric_input::InputEvent::TextInput {
                text: "zzzz".to_owned(),
            },
            100,
        );
        let mut c = canvas();
        draw(&mut c, &mut s);
        assert!(s.hotspots().rect_of(Hot::LauncherItem(0)).is_none());
        assert!(s.hotspots().rect_of(Hot::LauncherSearch).is_some());
    }

    #[test]
    fn quick_settings_registers_every_control_it_draws() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        // The real loop ticks between input and draw; the spring advances
        // there, and the panel is clickable from the first ticked frame.
        s.tick(16_000);
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

    #[test]
    fn the_notification_list_draws_when_empty_and_when_full() {
        let mut s = shell();
        s.perform(Action::ToggleNotifications);
        s.tick(16_000); // advance the panel's spring, as the loop does
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
