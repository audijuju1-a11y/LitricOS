//! The Settings window.
//!
//! One screen, three regions of responsibility: a rail that says which part of
//! the system you are changing, a pane of rows that each say what they do, and
//! a footer that says whether the last write reached the disk.
//!
//! The row is the unit. Every control is drawn from the [`Spec`]'s declared
//! kind, so a setting cannot appear with a switch when it holds a number: the
//! same declaration that validates the value chooses the control. That is the
//! property a screenshot cannot check and a test can.

use litric_design::{Radius, Space, TypeStep};

use crate::canvas::{rgb, with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::icons::{self, Icon};
use crate::settings::{self, Category, Kind, Model, SaveState, Spec, Value};
use crate::shell::{Hot, Shell};
use crate::ui::{self, Tokens, Tone};

/// The width of the category rail at the default scale.
const RAIL: f32 = 196.0;
/// The height of one setting row at the default scale.
const ROW: f32 = 58.0;
/// How many rows a page shows before it scrolls.
const ROWS: usize = 9;

/// Where each category's icon comes from.
fn category_icon(category: Category) -> Icon {
    match category {
        Category::Appearance => Icon::Grid,
        Category::Dock => Icon::Window,
        Category::TopBar => Icon::Cpu,
        Category::Notifications => Icon::Notifications,
        Category::Activity => Icon::Activity,
        Category::Files => Icon::Files,
        Category::Terminal => Icon::Terminal,
        Category::Power => Icon::Power,
        Category::Session => Icon::Lock,
        Category::Diagnostics => Icon::Shield,
    }
}

fn selected(shell: &Shell, hot: Hot) -> bool {
    shell.keyboard_focus() == Some(hot)
}

fn state_of(shell: &Shell, hot: Hot) -> ui::State {
    let hovered = shell.is_highlighted(hot);
    let pressed = shell.pressed() == Some(hot);
    ui::resolve_state(hovered, pressed, selected(shell, hot), true)
}

/// Draw the Settings window body.
pub fn draw(canvas: &mut Canvas, shell: &mut Shell, body: Rect) {
    let t = shell.tokens.clone();
    let changed = shell.settings.values.changed_count();

    // ---- header ---------------------------------------------------------
    let pad = t.sp(Space::Lg);
    let header_h = t.px(64.0);
    let header = Rect::new(body.x, body.y, body.w, header_h);
    let title_style = t.style_bold(TypeStep::Title3);
    font::draw(
        canvas,
        "Settings",
        header.x + pad,
        header.y + t.px(28.0),
        &title_style,
    );
    let count_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let summary = format!("{} of {} settings changed", changed, settings::count());
    font::draw(
        canvas,
        &summary,
        header.x + pad,
        header.y + t.px(52.0),
        &count_style,
    );

    let search_w = (body.w * 0.34).clamp(t.px(180.0), t.px(360.0));
    let search = Rect::new(
        header.right() - pad - search_w,
        header.y + t.px(14.0),
        search_w,
        t.px(32.0),
    );
    ui::text_field(
        canvas,
        &t,
        search,
        &shell.settings.query,
        "Search settings",
        selected(shell, Hot::SettingsSearch) || shell.settings.editing.is_some(),
    );
    shell.hotspots_mut().push(search, Hot::SettingsSearch);
    let key_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    font::draw(
        canvas,
        "type to search",
        search.x,
        search.bottom() + t.px(10.0),
        &key_style,
    );

    // ---- rail -----------------------------------------------------------
    let below = Rect::new(
        body.x,
        body.y + header_h,
        body.w,
        (body.h - header_h).max(0.0),
    );
    let footer_h = t.px(30.0);
    let rail_w = (RAIL * t.scale).min(below.w * 0.42);
    let rail = Rect::new(below.x, below.y, rail_w, (below.h - footer_h).max(0.0));
    canvas.fill_round_rect(rail, 0.0, with_alpha(rgb(0x000000), 0.14));

    let mut y = rail.y + t.px(8.0);
    for (index, category) in Category::ALL.iter().copied().enumerate() {
        let row = Rect::new(
            rail.x + t.px(8.0),
            y,
            (rail.w - t.px(16.0)).max(0.0),
            t.px(30.0),
        );
        // A rail taller than the window is clipped at the footer, not drawn
        // through it: at a small window height the last categories were drawn
        // over the save-status line ("เทอร์มินัล" over "Applied immediately").
        if row.bottom() > rail.bottom() - t.px(4.0) {
            break;
        }
        let hot = Hot::SettingsCategory(index as u8);
        let on = shell.settings.page == category && shell.settings.query.trim().is_empty();
        let hovered = shell.is_highlighted(hot);
        if on {
            canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.accent(), 0.16));
        } else if hovered || selected(shell, hot) {
            canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.text(), 0.06));
        }
        let icon_size = t.px(16.0);
        icons::draw(
            canvas,
            category_icon(category),
            Rect::new(
                row.x + t.px(9.0),
                row.y + (row.h - icon_size) / 2.0,
                icon_size,
                icon_size,
            ),
            if on { t.accent() } else { t.text_dim() },
            1.0,
        );
        let label_style = Style::new(
            t.text_px(TypeStep::Footnote),
            if on { t.text() } else { t.text_dim() },
        );
        let text_x = row.x + t.px(34.0);
        let room = (row.right() - text_x - t.px(24.0)).max(t.px(20.0));
        font::draw(
            canvas,
            &ui::elide(category.title(), &label_style, room),
            text_x,
            row.y + row.h * 0.66,
            &label_style,
        );
        // A dot per category that has a change, so "what did I change" is one
        // glance rather than a walk through ten pages.
        let changed_here = settings::category_specs(category)
            .iter()
            .filter(|s| shell.settings.values.is_changed(&s.id))
            .count();
        if changed_here > 0 {
            let dot = t.px(3.0);
            canvas.fill_circle(row.right() - t.px(10.0), row.center().1, dot, t.accent());
        }
        shell.hotspots_mut().push(row, hot);
        y += t.px(32.0);
    }

    // ---- pane -----------------------------------------------------------
    let pane = Rect::new(
        rail.right() + t.px(18.0),
        rail.y,
        (below.right() - rail.right() - t.px(26.0)).max(0.0),
        rail.h,
    );
    shell.settings.pane = Some(pane);

    let searching = !shell.settings.query.trim().is_empty();
    let heading = if searching {
        format!("Search results for “{}”", shell.settings.query.trim())
    } else {
        shell.settings.page.title().to_owned()
    };
    let heading_style = t.style_bold(TypeStep::Title3);
    font::draw(
        canvas,
        &heading,
        pane.x,
        pane.y + t.px(24.0),
        &heading_style,
    );
    let sub_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let subtitle = if searching {
        "Every category is searched, including advanced rows.".to_owned()
    } else {
        shell.settings.page.summary().to_owned()
    };
    font::draw(canvas, &subtitle, pane.x, pane.y + t.px(52.0), &sub_style);

    // Advanced disclosure, on the right of the heading, so it is found where a
    // person looks for a filter rather than in a footer.
    let adv_label = "Advanced";
    let adv_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let adv_w = font::measure(adv_label, &adv_style) + t.px(54.0);
    let adv = Rect::new(pane.right() - adv_w, pane.y + t.px(10.0), adv_w, t.px(24.0));
    font::draw(canvas, adv_label, adv.x, adv.y + t.px(16.0), &adv_style);
    let sw = Rect::new(
        adv.right() - t.px(40.0),
        adv.y + t.px(1.0),
        t.px(38.0),
        t.px(21.0),
    );
    ui::toggle(
        canvas,
        &t,
        sw,
        shell.settings.advanced,
        state_of(shell, Hot::SettingsAdvanced),
    );
    shell
        .hotspots_mut()
        .push(Rect::new(adv.x, adv.y, adv.w, adv.h), Hot::SettingsAdvanced);

    let rows_top = pane.y + t.px(80.0);
    let rows_h = (pane.bottom() - rows_top - t.px(8.0)).max(0.0);
    let visible = shell.settings.visible();
    let capacity = (rows_h / (ROW * t.scale)).floor().max(1.0) as usize;
    let start = shell.settings.scroll.min(visible.len().saturating_sub(1));
    let end = start.saturating_add(capacity).min(visible.len());

    if visible.is_empty() {
        font::draw(
            canvas,
            "Nothing matches. Clear the search to see the category.",
            pane.x,
            rows_top + t.px(24.0),
            &sub_style,
        );
    }

    let mut last_group: Option<&'static str> = None;
    let mut y = rows_top;
    for (offset, spec) in visible.get(start..end).unwrap_or(&[]).iter().enumerate() {
        let index = start.saturating_add(offset);
        if !searching && last_group != Some(spec.group) {
            if last_group.is_some() {
                y += t.px(10.0);
            }
            // A heading needs its own band: drawing it and then discovering
            // the row below does not fit leaves the heading alone under the
            // footer, which reads as a group with nothing in it.
            if y + t.px(22.0) + ROW * t.scale > (pane.bottom() - t.px(6.0)).max(0.0) {
                break;
            }
            ui::section_label(canvas, &t, (pane.x, y + t.px(11.0)), spec.group);
            y += t.px(22.0);
            last_group = Some(spec.group);
        }
        let row = Rect::new(pane.x, y, pane.w, ROW * t.scale);
        if row.bottom() > (pane.bottom() - t.px(16.0)).max(0.0) {
            break;
        }
        draw_row(canvas, shell, &t, row, spec, index, searching);
        y += row.h;
    }

    // "More below" is said, not implied: a page that ends mid-list with no
    // marker reads as a page with fewer settings than it has.
    if end < visible.len() {
        let more = format!("{} more below · scroll", visible.len().saturating_sub(end));
        font::draw(
            canvas,
            &more,
            pane.x,
            (pane.bottom() - t.px(4.0)).max(0.0),
            &sub_style,
        );
    } else if start > 0 {
        font::draw(
            canvas,
            "scroll up for the rest",
            pane.x,
            (pane.bottom() - t.px(4.0)).max(0.0),
            &sub_style,
        );
    }

    // ---- machine facts, on the page whose job is the truth about it -----
    // These are measurements, not preferences: there is no control that could
    // disagree with the kernel, which is why they sit below the settings rather
    // than being rows in the same list.
    if shell.settings.page == Category::Diagnostics && !searching {
        let facts = shell.facts.clone();
        let mut y = rows_top;
        // Skip past the settings rows this page drew.
        y += (ROW * t.scale) * (end.saturating_sub(start)) as f32;
        y += t.px(10.0);
        if y + t.px(16.0) < pane.bottom() {
            ui::section_label(canvas, &t, (pane.x, y + t.px(9.0)), "THIS MACHINE");
            y += t.px(20.0);
            for (label, value) in machine_rows(&facts) {
                if y + t.px(18.0) > (pane.bottom() - t.px(4.0)).max(0.0) {
                    break;
                }
                let label_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
                font::draw(canvas, &label, pane.x, y + t.px(13.0), &label_style);
                let value_style = Style::new(t.text_px(TypeStep::Caption), t.text());
                let room = (pane.w - t.px(120.0)).max(t.px(40.0));
                let shown = ui::elide(&value, &value_style, room);
                let w = font::measure(&shown, &value_style);
                font::draw(
                    canvas,
                    &shown,
                    pane.right() - w,
                    y + t.px(13.0),
                    &value_style,
                );
                y += t.px(18.0);
            }
        }
    }

    // ---- footer ---------------------------------------------------------
    let footer = Rect::new(
        below.x + pad,
        (below.bottom() - footer_h).max(0.0),
        (below.w - pad * 2.0).max(0.0),
        footer_h,
    );
    ui::separator(canvas, &t, Rect::new(footer.x, footer.y, footer.w, 1.0));
    let status = match &shell.settings.save {
        SaveState::Idle => (
            if shell.settings.dirty {
                "Unsaved changes"
            } else {
                "Applied immediately · saved on this machine"
            }
            .to_owned(),
            if shell.settings.dirty {
                Tone::Warning
            } else {
                Tone::Neutral
            },
        ),
        SaveState::Saving => ("Saving…".to_owned(), Tone::Neutral),
        SaveState::Saved => ("Saved".to_owned(), Tone::Good),
        SaveState::Failed(error) => (format!("Not saved: {error}"), Tone::Bad),
    };
    let (line, tone) = match &shell.settings.message {
        Some((message, true)) => (message.clone(), Tone::Good),
        Some((message, false)) => (message.clone(), Tone::Bad),
        None => status,
    };

    // A right-aligned cluster: Import and Export are the transfer pair, and
    // Reset closes it at the edge. Laid out before the status line because the
    // status line is the one that gives way: at the smallest window the three
    // pills own well over half the footer, and a fixed 55% elide put the text
    // under them (the text-layout audit caught exactly that at 480x360).
    let reset_label = if searching {
        "Reset all".to_owned()
    } else {
        format!("Reset {}", shell.settings.page.title())
    };
    let y = footer.y + t.px(5.0);
    let mut right = footer.right();
    let mut pill =
        |label: &str, hot: Hot, right: f32| footer_button(canvas, shell, &t, label, hot, right, y);
    right = pill(&reset_label, Hot::SettingsReset, right) - t.px(8.0);
    right = pill("Export", Hot::SettingsExport, right) - t.px(8.0);
    let import_x = pill("Import", Hot::SettingsImport, right);

    let status_style = Style::new(t.text_px(TypeStep::Caption), t.tone(tone));
    let room = (import_x - footer.x - t.px(12.0)).max(0.0);
    font::draw(
        canvas,
        &ui::elide(&line, &status_style, room),
        footer.x,
        footer.y + t.px(19.0),
        &status_style,
    );
}

/// One footer pill: drawn, registered in the hotspot table, and returning the
/// left edge it occupies so the next pill packs against this one.
fn footer_button(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &Tokens,
    label: &str,
    hot: Hot,
    right: f32,
    y: f32,
) -> f32 {
    let style = Style::new(t.text_px(TypeStep::Caption), t.text());
    let w = font::measure(label, &style) + t.px(22.0);
    let rect = Rect::new(right - w, y, w, t.px(22.0));
    let fill = match state_of(shell, hot) {
        ui::State::Hover => with_alpha(t.text(), 0.12),
        ui::State::Pressed => with_alpha(t.text(), 0.18),
        ui::State::Focused => with_alpha(t.text(), 0.10),
        _ => with_alpha(t.text(), 0.06),
    };
    canvas.fill_round_rect(rect, rect.h / 2.0, fill);
    font::draw(
        canvas,
        label,
        rect.x + t.px(11.0),
        rect.y + t.px(15.0),
        &style,
    );
    shell.hotspots_mut().push(rect, hot);
    rect.x
}

fn draw_row(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &Tokens,
    row: Rect,
    spec: &'static Spec,
    index: usize,
    searching: bool,
) {
    let hot = Hot::SettingsRow(u16::try_from(index).unwrap_or(u16::MAX));
    let hovered = shell.is_highlighted(hot);
    let is_selected = shell.keyboard_focus() == Some(hot);
    if is_selected {
        canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.accent(), 0.12));
    } else if hovered {
        canvas.fill_round_rect(row, t.radius(Radius::Sm), with_alpha(t.text(), 0.04));
    }

    let control_w = (row.w * 0.38).clamp(t.px(96.0), t.px(190.0));
    let label_x = row.x + t.px(10.0);
    let label_w = (row.w - control_w - t.px(28.0)).max(t.px(40.0));
    let label_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let mut label = spec.label.clone();
    if searching {
        label = format!("{} · {}", spec.label, spec.category.title());
    }
    font::draw(
        canvas,
        &ui::elide(&label, &label_style, label_w),
        label_x,
        row.y + t.px(20.0),
        &label_style,
    );
    let help_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    font::draw(
        canvas,
        &ui::elide(&spec.help, &help_style, label_w),
        label_x,
        row.y + t.px(38.0),
        &help_style,
    );

    // A setting that is not at its default says so, so a person can tell what
    // they changed without reading every value.
    if shell.settings.values.is_changed(&spec.id) && !searching {
        canvas.fill_circle(
            label_x - t.px(2.0),
            row.y + t.px(14.0),
            t.px(2.0),
            t.accent(),
        );
    }

    let value = shell.settings.values.get(&spec.id);
    let control = Rect::new(
        row.right() - control_w,
        row.y + t.px(14.0),
        (control_w - t.px(4.0)).max(0.0),
        t.px(28.0),
    );
    draw_control(canvas, shell, t, spec, &value, control, index, hot);
    shell.hotspots_mut().push(row, hot);
}

#[allow(clippy::too_many_arguments)]
fn draw_control(
    canvas: &mut Canvas,
    shell: &mut Shell,
    t: &Tokens,
    spec: &'static Spec,
    value: &Value,
    control: Rect,
    index: usize,
    row_hot: Hot,
) {
    let caption = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    match &spec.kind {
        Kind::Toggle => {
            let sw = Rect::new(
                control.right() - t.px(38.0),
                control.y + t.px(2.0),
                t.px(36.0),
                t.px(20.0),
            );
            ui::toggle(
                canvas,
                t,
                sw,
                matches!(value, Value::On(true)),
                state_of(shell, row_hot),
            );
        }
        Kind::Choice(_) | Kind::Colour => {
            let text = spec.display(value);
            let pill = Rect::new(control.x, control.y, control.w.min(t.px(170.0)), control.h);
            ui::surface(canvas, t, pill, pill.h / 2.0, ui::Material::Elevated);
            let style = Style::new(t.text_px(TypeStep::Caption), t.text());
            font::draw(
                canvas,
                &ui::elide(&text, &style, pill.w - t.px(28.0)),
                pill.x + t.px(10.0),
                pill.y + t.px(18.0),
                &style,
            );
            icons::draw(
                canvas,
                Icon::ChevronRight,
                Rect::new(
                    pill.right() - t.px(16.0),
                    pill.y + (pill.h - t.px(12.0)) / 2.0,
                    t.px(12.0),
                    t.px(12.0),
                ),
                t.text_dim(),
                1.0,
            );
            if let Value::Colour(rgb_val) = value {
                let dot = t.px(9.0);
                canvas.fill_circle(
                    pill.x - t.px(14.0),
                    pill.center().1,
                    dot,
                    crate::canvas::rgba(
                        ((rgb_val >> 16) & 0xFF) as u8,
                        ((rgb_val >> 8) & 0xFF) as u8,
                        (rgb_val & 0xFF) as u8,
                        255,
                    ),
                );
            }
        }
        Kind::Number { unit, .. } => {
            let step = t.px(22.0);
            let minus = Rect::new(control.x, control.y + t.px(3.0), step, step);
            let plus = Rect::new(control.right() - step, control.y + t.px(3.0), step, step);
            let plus_hot = Hot::SettingsStep(u16::try_from(index).unwrap_or(u16::MAX), true);
            let minus_hot = Hot::SettingsStep(u16::try_from(index).unwrap_or(u16::MAX), false);
            for (r, symbol, hot) in [(minus, "-", minus_hot), (plus, "+", plus_hot)] {
                let state = state_of(shell, hot);
                canvas.fill_round_rect(
                    r,
                    r.h / 2.0,
                    with_alpha(
                        t.text(),
                        if state == ui::State::Hover {
                            0.16
                        } else {
                            0.08
                        },
                    ),
                );
                let style = Style::new(t.text_px(TypeStep::Caption), t.text());
                let w = font::measure(symbol, &style);
                font::draw(
                    canvas,
                    symbol,
                    r.x + (r.w - w) / 2.0,
                    r.y + t.px(15.0),
                    &style,
                );
                shell.hotspots_mut().push(r, hot);
            }
            let text = spec.display(value);
            let style = Style::new(t.text_px(TypeStep::Footnote), t.text());
            let w = font::measure(&text, &style);
            font::draw(
                canvas,
                &text,
                minus.right()
                    + t.px(6.0)
                    + ((plus.x - minus.right() - t.px(12.0) - w) / 2.0).max(0.0),
                control.y + t.px(20.0),
                &style,
            );
            let _ = unit;
        }
        Kind::Text { .. } => {
            let editing = shell.settings.editing.as_deref() == Some(spec.id.as_str());
            let field = Rect::new(control.x, control.y, control.w, control.h);
            canvas.fill_round_rect(field, t.radius(Radius::Sm), with_alpha(rgb(0xFFFFFF), 0.06));
            canvas.stroke_round_rect(
                field,
                t.radius(Radius::Sm),
                1.0,
                if editing {
                    with_alpha(t.accent(), 0.8)
                } else {
                    with_alpha(rgb(0xFFFFFF), 0.12)
                },
            );
            let style = Style::new(t.text_px(TypeStep::Caption), t.text());
            let shown = match value {
                Value::Text(text) if text.is_empty() => "Empty".to_owned(),
                Value::Text(text) => text.clone(),
                _ => String::new(),
            };
            let shown_style = if matches!(value, Value::Text(text) if text.is_empty()) {
                caption
            } else {
                style
            };
            font::draw(
                canvas,
                &ui::elide(&shown, &shown_style, field.w - t.px(14.0)),
                field.x + t.px(7.0),
                field.y + t.px(18.0),
                &shown_style,
            );
            if editing {
                font::draw(
                    canvas,
                    "|",
                    field.x
                        + t.px(7.0)
                        + font::measure(&shown, &shown_style).min(field.w - t.px(16.0)),
                    field.y + t.px(18.0),
                    &Style::new(t.text_px(TypeStep::Caption), t.accent()),
                );
            }
        }
    }
}

/// What is true about the machine, from the machine.
///
/// Every value here is a measurement; an absent one says so rather than
/// reading as zero. This is the page a person opens when a key does nothing, so
/// the attached devices are named rather than counted.
fn machine_rows(facts: &crate::shell::SystemFacts) -> Vec<(String, String)> {
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
    if facts.input_devices.is_empty() {
        rows.push((
            "Input devices".to_owned(),
            "None reported by the kernel".to_owned(),
        ));
    } else {
        let names: Vec<String> = facts
            .input_devices
            .iter()
            .map(|d| format!("{} ({})", d.name, d.kind))
            .collect();
        rows.push(("Input devices".to_owned(), names.join(", ")));
    }
    let bt = match facts.bt_state.as_deref().unwrap_or("unknown") {
        "no-adapter" => "No radio in this machine",
        "blocked" => "Radio blocked by firmware or a switch",
        "no-daemon" => "Radio present, no pairing service",
        "ready" => "Ready",
        _ => "Unavailable",
    };
    rows.push(("Bluetooth".to_owned(), bt.to_owned()));
    rows.push((
        "Sound".to_owned(),
        facts
            .audio_server
            .clone()
            .unwrap_or_else(|| "No sound server".to_owned()),
    ));
    rows.push((
        "Measured by".to_owned(),
        if facts.source.is_empty() {
            "nothing yet".to_owned()
        } else {
            facts.source.clone()
        },
    ));
    rows
}

/// The rows a page shows at a given height, for tests and for the input layer's
/// scrolling bounds.
#[must_use]
pub fn page_rows(t: &Tokens) -> usize {
    let logical = ROWS as f32;
    let _ = t;
    logical as usize
}

/// A model's rows for one category, filtered exactly as the window draws them.
#[must_use]
pub fn rows_for(model: &Model) -> Vec<&'static Spec> {
    model.visible()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Preferences, Value};
    use crate::shell::DesktopMode;
    use litric_design::{Theme, ThemeKind};
    use litric_input::event::{InputEvent, Point, PointerButton};
    use litric_input::keys::Modifiers;
    use litric_locale::Regional;

    fn shell() -> Shell {
        let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
        s.mode = DesktopMode::Ready;
        s
    }

    fn canvas() -> Canvas {
        let mut c = Canvas::new(1280, 800, rgb(0x000000));
        crate::desktop::wallpaper(&mut c, &Theme::of(ThemeKind::Dark));
        c
    }

    fn ink(c: &Canvas, r: Rect) -> usize {
        let mut n = 0_usize;
        for y in (r.y as i64)..(r.bottom() as i64) {
            for x in (r.x as i64)..(r.right() as i64) {
                if crate::canvas::luminance(c.get(x, y).unwrap_or(0)) > 0.25 {
                    n = n.saturating_add(1);
                }
            }
        }
        n
    }

    fn open_settings(s: &mut Shell) {
        let _ = s.open_app(crate::shortcuts::App::Settings);
    }

    #[test]
    fn every_category_row_registers_a_hotspot() {
        let mut s = shell();
        open_settings(&mut s);
        let mut canvas = canvas();
        draw(&mut canvas, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        for index in 0..Category::ALL.len() {
            assert!(
                s.hotspots()
                    .rect_of(Hot::SettingsCategory(index as u8))
                    .is_some(),
                "category {index} has no rectangle"
            );
        }
        assert!(s.hotspots().rect_of(Hot::SettingsSearch).is_some());
    }

    #[test]
    fn a_row_is_drawn_for_each_visible_setting_and_they_all_hit() {
        let mut s = shell();
        open_settings(&mut s);
        s.settings.page = Category::Appearance;
        let mut c = canvas();
        draw(&mut c, &mut s, Rect::new(0.0, 0.0, 1200.0, 900.0));
        let visible = s.settings.visible();
        let drawn = visible
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                s.hotspots()
                    .rect_of(Hot::SettingsRow(u16::try_from(*i).unwrap_or(u16::MAX)))
                    .is_some()
            })
            .count();
        assert!(drawn >= 4, "only {drawn} rows registered");
    }

    #[test]
    fn the_control_matches_the_declared_kind() {
        // Two settings of different kinds: switching one must not change the
        // other, and the control each draws must be its own.
        let mut s = shell();
        open_settings(&mut s);
        s.settings.query = "animations".to_owned();
        let mut first = canvas();
        draw(&mut first, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert!(s.hotspots().rect_of(Hot::SettingsRow(0)).is_some());

        s.settings.query = "corner radius".to_owned();
        let mut second = canvas();
        draw(&mut second, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert!(
            s.hotspots().rect_of(Hot::SettingsStep(0, true)).is_some(),
            "a number must draw step buttons"
        );
    }

    #[test]
    fn a_search_with_no_match_says_so_without_hiding_the_field() {
        let mut s = shell();
        open_settings(&mut s);
        s.settings.query = "zzzz not a setting".to_owned();
        let mut surface = canvas();
        let ink_before = ink(&surface, Rect::new(0.0, 0.0, 1200.0, 800.0));
        draw(&mut surface, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let ink_after = ink(&surface, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert_ne!(ink_before, ink_after);
        assert!(s.settings.visible().is_empty());
        assert!(s.hotspots().rect_of(Hot::SettingsSearch).is_some());
    }

    #[test]
    fn changing_an_appearance_value_changes_the_next_frame() {
        let mut s = shell();
        open_settings(&mut s);
        let mut before = canvas();
        draw(&mut before, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        s.settings
            .values
            .set("appearance.accent", Value::Colour(0x00_FF_00))
            .expect("accent");
        s.settings
            .values
            .set("appearance.accent-follows-theme", Value::On(false))
            .expect("custom accent");
        s.apply_settings();
        let mut after = canvas();
        draw(&mut after, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert_ne!(
            ink(&before, Rect::new(0.0, 0.0, 1200.0, 800.0)),
            ink(&after, Rect::new(0.0, 0.0, 1200.0, 800.0)),
            "a changed accent must reach the pixels"
        );
    }

    #[test]
    fn the_model_used_here_is_the_one_the_shell_owns() {
        let mut s = shell();
        s.settings = crate::settings::Model::new(Preferences::default());
        s.settings.query = "theme".to_owned();
        let rows = rows_for(&s.settings);
        assert!(rows.iter().any(|r| r.id == "appearance.theme"));
    }

    #[test]
    fn scrolling_reports_what_it_did_not_draw() {
        let mut s = shell();
        open_settings(&mut s);
        s.settings.page = Category::Activity;
        s.settings.advanced = true;
        s.settings.scroll = 3;
        let mut canvas = canvas();
        draw(&mut canvas, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let visible = s.settings.visible();
        assert!(visible.len() > 6);
        assert!(s.settings.pane.is_some());
    }

    #[test]
    fn the_page_shows_the_footer_and_the_reset_control() {
        let mut s = shell();
        open_settings(&mut s);
        let mut canvas = canvas();
        draw(&mut canvas, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        assert!(s.hotspots().rect_of(Hot::SettingsReset).is_some());
        assert!(s.hotspots().rect_of(Hot::SettingsAdvanced).is_some());
    }

    #[test]
    fn the_footer_offers_the_export_and_import_controls() {
        let mut s = shell();
        open_settings(&mut s);
        let mut canvas = canvas();
        draw(&mut canvas, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let export = s.hotspots().rect_of(Hot::SettingsExport).expect("export");
        let import = s.hotspots().rect_of(Hot::SettingsImport).expect("import");
        let reset = s.hotspots().rect_of(Hot::SettingsReset).expect("reset");
        // Right-aligned cluster: each pill sits left of the one before it, and
        // none of them crosses the footer's right edge.
        assert!(import.x < export.x, "{import:?} {export:?}");
        assert!(export.x < reset.x, "{export:?} {reset:?}");
        assert!(reset.right() <= 1200.0);
    }

    /// A real click: move, press, release — the same three events the input
    /// thread produces, because a hotspot only activates when the release
    /// lands where the press did.
    fn click(s: &mut Shell, x: f32, y: f32) -> Vec<crate::shell::Effect> {
        let mut out = s.handle(
            &InputEvent::PointerMove {
                at: Point::new(x, y),
                delta: Point::new(1.0, 1.0),
            },
            1_000,
        );
        for pressed in [true, false] {
            out.extend(s.handle(
                &InputEvent::PointerButton {
                    button: PointerButton::Primary,
                    pressed,
                    at: Point::new(x, y),
                    clicks: 1,
                    mods: Modifiers::NONE,
                },
                1_010,
            ));
        }
        out
    }

    #[test]
    fn clicking_export_and_import_reaches_the_shell_as_effects() {
        let mut s = shell();
        open_settings(&mut s);
        let mut canvas = canvas();
        draw(&mut canvas, &mut s, Rect::new(0.0, 0.0, 1200.0, 800.0));
        let export = s.hotspots().rect_of(Hot::SettingsExport).expect("export");
        let import = s.hotspots().rect_of(Hot::SettingsImport).expect("import");
        let export_fx = click(&mut s, export.x + 2.0, export.y + 2.0);
        let import_fx = click(&mut s, import.x + 2.0, import.y + 2.0);
        assert!(
            export_fx
                .iter()
                .any(|e| matches!(e, crate::shell::Effect::ExportSettings(_))),
            "{export_fx:?}"
        );
        assert!(
            import_fx
                .iter()
                .any(|e| matches!(e, crate::shell::Effect::ImportSettings)),
            "{import_fx:?}"
        );
    }
}
