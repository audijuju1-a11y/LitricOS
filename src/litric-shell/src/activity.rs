//! The Activity window: the machine meters and the process table.
//!
//! Everything here is drawn *from* [`TaskManager`]: the columns, their order,
//! their widths, their alignment, the sort, the filters, the row height, the row
//! tints and the zebra shading are all preferences, so a preference the model
//! does not read cannot appear on this screen. That is the property a picture
//! cannot check and a test can.
//!
//! Three rules hold here as everywhere else in this shell:
//!
//! 1. **A cell is elided to its own column**, never to the window. A value that
//!    runs past its column is a value drawn over its neighbour, which is the
//!    defect this project has already fixed once.
//! 2. **A reading that was never measured is a dash.** A machine that did not
//!    report its swap is not a machine using no swap.
//! 3. **Nothing is dropped in silence.** Rows the window had no room for, and
//!    columns too narrow to hold their own heading, are counted in the footer:
//!    a monitor that hides half a process table without saying so is worse than
//!    one that shows it badly.
//!
//! The decisions are separate functions returning plain data ([`column_widths`],
//! [`Report`]) so they can be tested without a canvas, and the table's report is
//! returned rather than printed so a test can assert what a frame really drew.

use litric_design::{Radius, Space, TypeStep};

use crate::canvas::{with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::monitor::fit;
use crate::taskmgr::{heading, Align, Column, Row, TaskManager};
use crate::ui::{Tokens, Tone};

/// What the table drew, and what it could not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Report {
    /// Process rows drawn.
    pub drawn: usize,
    /// Process rows the window had no room for.
    pub hidden: usize,
    /// Columns drawn.
    pub columns: usize,
    /// Columns the window had no room for.
    pub columns_hidden: usize,
}

impl Report {
    /// Whether the table showed every row and every column it was asked for.
    #[must_use]
    pub fn complete(&self) -> bool {
        self.hidden == 0 && self.columns_hidden == 0
    }
}

/// `n` and the word for it, so a count is never "1 items".
#[must_use]
pub fn processes(n: usize) -> String {
    if n == 1 {
        "1 process".to_owned()
    } else {
        format!("{n} processes")
    }
}

/// `n` and the word for it.
#[must_use]
pub fn columns_word(n: usize) -> String {
    if n == 1 {
        "1 column".to_owned()
    } else {
        format!("{n} columns")
    }
}

/// The columns to draw and the width each gets, plus how many were dropped.
///
/// A column keeps its own width — the width the preferences gave it — while
/// there is room. When there is not, it is dropped rather than squeezed: a
/// number elided to three digits is not a smaller measurement, it is a wrong
/// one, and a heading cut in half is not a heading.
///
/// The first column is always kept, clipped if it must be: a table with no name
/// column is a list of numbers nobody can read.
#[must_use]
pub fn column_widths(m: &TaskManager, scale: f32, room: f32) -> (Vec<(Column, f32)>, usize) {
    let mut out: Vec<(Column, f32)> = Vec::new();
    let mut left = room;
    let mut dropped: usize = 0;
    for c in &m.prefs.columns {
        let natural = c.width() * m.prefs.width_of(*c).factor() * scale;
        if natural <= left {
            out.push((*c, natural));
            left -= natural;
        } else if out.is_empty() {
            out.push((*c, room));
            left = 0.0;
        } else {
            dropped = dropped.saturating_add(1);
        }
    }
    (out, dropped)
}

/// The height one row of the table is drawn at, in pixels.
///
/// It comes from the density the user chose, measured against the type it holds:
/// a row is at least as tall as its own text, which is the rule that stops the
/// letters of one row meeting the letters of the next.
#[must_use]
pub fn row_height(t: &Tokens, m: &TaskManager) -> f32 {
    let body = Style::new(t.text_px(TypeStep::Footnote), t.text());
    (m.prefs.density.row_h() * t.scale + t.sp(Space::Xs)).max(body.size + t.sp(Space::Xxs))
}

/// The baseline of a run of `size` inside a row starting at `top`.
///
/// One function, because a table where each cell picks its own baseline is a
/// table whose columns do not line up.
fn baseline(top: f32, size: f32) -> f32 {
    top + size * 0.85
}

/// Draw the process table into `body`, and say what it managed to show.
pub fn table(canvas: &mut Canvas, t: &Tokens, m: &TaskManager, body: Rect) -> Report {
    let mut report = Report::default();
    let pad = t.sp(Space::Sm);
    if body.w <= pad * 2.0 || body.h <= 0.0 {
        return report;
    }
    let rect = Rect::new(
        body.x + pad,
        body.y + t.sp(Space::Xs),
        body.w - pad * 2.0,
        (body.h - t.sp(Space::Xs) * 2.0).max(0.0),
    );
    let heading_style = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());
    let cell_style = Style::new(t.text_px(TypeStep::Footnote), t.text());
    let footer_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let rule = with_alpha(t.text_dim(), 0.18);
    let row_h = row_height(t, m);
    let heading_h = heading_style.size + t.sp(Space::Sm);
    let footer_h = footer_style.size + t.sp(Space::Sm);
    if rect.h < heading_h + footer_h + row_h {
        // Too short to hold even one row: the table says why instead of showing
        // a heading with nothing under it.
        font::draw(
            canvas,
            &fit(
                "The window is too short for the process table.",
                &footer_style,
                rect.w,
            ),
            rect.x,
            rect.y + footer_style.size,
            &footer_style,
        );
        report.columns_hidden = m.prefs.columns.len();
        return report;
    }

    let (cols, columns_hidden) = column_widths(m, t.scale, rect.w);
    report.columns = cols.len();
    report.columns_hidden = columns_hidden;

    // ---- headings ------------------------------------------------------
    for (i, (c, w)) in cols.iter().enumerate() {
        let x = cols.iter().take(i).map(|(_, w)| *w).sum::<f32>() + rect.x;
        let align = m.prefs.align_of(*c);
        cell(
            canvas,
            heading(*c, &m.prefs),
            x,
            *w,
            baseline(rect.y, heading_style.size),
            &heading_style,
            align,
            t,
        );
        if i.saturating_add(1) < cols.len() {
            canvas.fill_round_rect(
                Rect::new(x + w - 1.0, rect.y, 1.0, heading_h - t.sp(Space::Xs)),
                0.0,
                rule,
            );
        }
    }
    let head_bottom = rect.y + heading_h;
    crate::ui::separator(
        canvas,
        t,
        Rect::new(rect.x, head_bottom - t.sp(Space::Xs), rect.w, 1.0),
    );

    // ---- rows ----------------------------------------------------------
    let room = (rect.h - heading_h - footer_h).max(0.0);
    let per_page = (room / row_h).floor().max(0.0) as usize;
    let rows = m.rows_to_draw();
    let total = rows
        .iter()
        .filter(|r| matches!(r, Row::Process { .. }))
        .count();
    let visible: Vec<&Row> = rows.iter().skip(m.scroll).take(per_page).collect();
    if m.rows.is_empty() {
        // Nothing reported is not the same as nothing running, and the window
        // says which it is.
        font::draw(
            canvas,
            "No process was reported. The system daemon is not answering.",
            rect.x,
            baseline(head_bottom + t.sp(Space::Xs), cell_style.size),
            &footer_style,
        );
    }
    for (i, row) in visible.iter().enumerate() {
        let top = head_bottom + i as f32 * row_h;
        let band = Rect::new(
            rect.x,
            top,
            rect.w,
            row_h - t.sp(Space::Xxs).min(row_h / 4.0),
        );
        match row {
            Row::Group { name, count, open } => {
                canvas.fill_round_rect(band, t.radius(Radius::Sm), with_alpha(t.text(), 0.05));
                let style = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());
                let text = format!(
                    "{name}  ·  {count}{}",
                    if *open { "" } else { "  (closed)" }
                );
                font::draw(
                    canvas,
                    &fit(&text, &style, rect.w - t.sp(Space::Sm)),
                    rect.x + t.sp(Space::Xs),
                    baseline(top, style.size),
                    &style,
                );
            }
            Row::Process { index } => {
                let Some(p) = m.filtered().get(*index).copied().cloned() else {
                    continue;
                };
                let selected = m.selected == Some(*index);
                let tone = m.row_tone(&p);
                if selected {
                    canvas.fill_round_rect(
                        band,
                        t.radius(Radius::Sm),
                        with_alpha(t.tone(Tone::Accent), 0.30),
                    );
                } else if let Some(tone) = tone {
                    canvas.fill_round_rect(
                        band,
                        t.radius(Radius::Sm),
                        with_alpha(t.tone(tone), 0.16),
                    );
                } else if m.prefs.zebra && i % 2 == 1 {
                    canvas.fill_round_rect(band, t.radius(Radius::Sm), with_alpha(t.text(), 0.035));
                }
                let mut x = rect.x;
                for (c, w) in &cols {
                    let raw = c.cell(&p, &m.prefs, m.total_mb);
                    let text = if *c == Column::Name {
                        crate::copy::process_label(&raw)
                    } else {
                        raw
                    };
                    cell(
                        canvas,
                        &text,
                        x,
                        *w,
                        baseline(top, cell_style.size),
                        &cell_style,
                        m.prefs.align_of(*c),
                        t,
                    );
                    x += w;
                }
                report.drawn = report.drawn.saturating_add(1);
            }
        }
    }
    report.hidden = total.saturating_sub(report.drawn);

    // ---- footer --------------------------------------------------------
    // The notes that matter most come first, and the line is built from the ones
    // that fit rather than elided as a whole: a footer cut off at "43 processes
    // · sorted by..." has lost the one thing it had to say, that rows were left
    // out of the table above it.
    let mut notes: Vec<String> = Vec::new();
    if report.hidden > 0 {
        notes.push(format!("{} more rows", report.hidden));
    }
    if report.columns_hidden > 0 {
        notes.push(format!("{} hidden", columns_word(report.columns_hidden)));
    }
    if m.prefs.paused {
        notes.push("paused".to_owned());
    }
    notes.push(processes(total));
    notes.push(format!(
        "sorted by {} {}",
        m.prefs.sort.0.label(),
        if m.prefs.sort.1 {
            "smallest first"
        } else {
            "largest first"
        }
    ));
    let mut footer = String::new();
    for note in &notes {
        let candidate = if footer.is_empty() {
            note.clone()
        } else {
            format!("{footer}  ·  {note}")
        };
        if font::measure(&candidate, &footer_style) <= rect.w {
            footer = candidate;
        } else if footer.is_empty() {
            // The most important note goes on the screen even when it does not
            // fit: elided, it still says more than nothing does.
            footer = fit(note, &footer_style, rect.w);
        } else {
            // The rest are dropped in order of importance, never reordered.
            break;
        }
    }
    font::draw(
        canvas,
        &footer,
        rect.x,
        rect.bottom() - t.sp(Space::Xs),
        &footer_style,
    );
    report
}

/// The rectangle of every process row the table draws, in draw order.
///
/// The table's geometry is repeated here on purpose rather than published as
/// mutable state: this is the hit-testing half of the same layout, and the
/// journey that clicks a row proves the two agree. Group rows are skipped;
/// a group heading is not a process and cannot be ended.
#[must_use]
pub fn row_bands(t: &Tokens, m: &TaskManager, body: Rect) -> Vec<(Rect, u32)> {
    let pad = t.sp(Space::Sm);
    if body.w <= pad * 2.0 || body.h <= 0.0 {
        return Vec::new();
    }
    let rect = Rect::new(
        body.x + pad,
        body.y + t.sp(Space::Xs),
        body.w - pad * 2.0,
        (body.h - t.sp(Space::Xs) * 2.0).max(0.0),
    );
    let heading_style = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());
    let footer_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let row_h = row_height(t, m);
    let heading_h = heading_style.size + t.sp(Space::Sm);
    let footer_h = footer_style.size + t.sp(Space::Sm);
    if rect.h < heading_h + footer_h + row_h {
        return Vec::new();
    }
    let room = (rect.h - heading_h - footer_h).max(0.0);
    let per_page = (room / row_h).floor().max(0.0) as usize;
    let head_bottom = rect.y + heading_h;
    let filtered = m.filtered();
    let mut out = Vec::new();
    for (i, row) in m
        .rows_to_draw()
        .iter()
        .skip(m.scroll)
        .take(per_page)
        .enumerate()
    {
        if let Row::Process { index } = row {
            if let Some(p) = filtered.get(*index) {
                out.push((
                    Rect::new(
                        rect.x,
                        head_bottom + i as f32 * row_h,
                        rect.w,
                        row_h - t.sp(Space::Xxs).min(row_h / 4.0),
                    ),
                    p.pid,
                ));
            }
        }
    }
    out
}

/// One cell: elided to its own column and aligned inside it.
#[allow(clippy::too_many_arguments)]
fn cell(
    canvas: &mut Canvas,
    text: &str,
    x: f32,
    w: f32,
    y: f32,
    style: &Style,
    align: Align,
    t: &Tokens,
) {
    let inner = (w - t.sp(Space::Sm)).max(t.px(4.0));
    let shown = fit(text, style, inner);
    let width = font::measure(&shown, style);
    let x = match align {
        Align::Left => x + t.sp(Space::Xs),
        Align::Right => x + w - t.sp(Space::Sm) - width,
    };
    font::draw(canvas, &shown, x, y, style);
}

/// A filled meter: a track, then the fraction of it that is real.
fn bar(canvas: &mut Canvas, t: &Tokens, r: Rect, frac: f32, colour: u32) {
    canvas.fill_round_rect(r, r.h / 2.0, with_alpha(t.text_dim(), 0.35));
    let f = frac.clamp(0.0, 1.0);
    canvas.fill_round_rect(
        Rect::new(r.x, r.y, (r.w * f).max(r.h), r.h),
        r.h / 2.0,
        colour,
    );
}

/// Draw the machine meters from the model, and return where the next section
/// starts.
///
/// The band a person set is drawn, not just applied: a reading that has crossed
/// its warn band is tinted with that band's colour, and the marks that are on are
/// printed underneath, because "warn 70 %" is what tells a person how far the
/// number is from being flagged.
pub fn meters(canvas: &mut Canvas, t: &Tokens, m: &TaskManager, content: &Rect, mut y: f32) -> f32 {
    let label_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let marks_style = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    for meter in m.meters() {
        if y + t.px(28.0) > content.bottom() {
            break;
        }
        let colour = if meter.level.is_some() {
            t.tone(meter.tone)
        } else {
            t.text()
        };
        let value_style = Style::bold(t.text_px(TypeStep::Headline), colour);
        font::draw(
            canvas,
            meter.label,
            content.x,
            baseline(y, label_style.size),
            &label_style,
        );
        let room = (content.w - font::measure(meter.label, &label_style) - t.sp(Space::Md))
            .max(t.px(32.0));
        let shown = fit(&meter.text, &value_style, room);
        let w = font::measure(&shown, &value_style);
        font::draw(
            canvas,
            &shown,
            content.right() - w,
            baseline(y, value_style.size),
            &value_style,
        );
        y += t.px(20.0);
        bar(
            canvas,
            t,
            Rect::new(content.x, y, content.w, t.px(4.0)),
            meter.share,
            colour,
        );
        y += t.px(10.0);
        if !meter.marks.is_empty() {
            font::draw(
                canvas,
                &fit(&meter.marks, &marks_style, content.w),
                content.x,
                baseline(y, marks_style.size),
                &marks_style,
            );
            y += t.px(14.0);
        }
        y += t.sp(Space::Xs);
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ProcRow;
    use crate::textaudit;

    fn proc_(pid: u32, name: &str, cpu: Option<f32>, rss: Option<u64>) -> ProcRow {
        ProcRow {
            pid,
            name: name.to_owned(),
            cmdline: format!("/usr/bin/{name} --flag=value"),
            state: "sleeping".to_owned(),
            ppid: 1,
            threads: 3,
            ticks: 400,
            rss_mb: rss,
            uid: Some(1000),
            cpu,
        }
    }

    /// A table with a row that is over a warn band, one that is not, and one
    /// whose rate was never measured -- the three cases a cell has to draw.
    fn model() -> TaskManager {
        let mut m = TaskManager::new();
        m.rows = vec![
            proc_(1, "litric-sysd", Some(0.42), Some(64)),
            proc_(
                2,
                "a-process-with-a-very-long-name-indeed",
                Some(0.01),
                Some(4096),
            ),
            proc_(3, "unmeasured", None, None),
        ];
        m.total_mb = Some(8192);
        m
    }

    fn tokens(scale: f32) -> Tokens {
        Tokens::new(
            litric_design::Theme::of(litric_design::ThemeKind::Dark),
            scale,
        )
    }

    #[test]
    fn a_count_is_pluralised_and_a_single_one_is_not() {
        assert_eq!(processes(0), "0 processes");
        assert_eq!(processes(1), "1 process");
        assert_eq!(processes(2), "2 processes");
        assert_eq!(columns_word(1), "1 column");
        assert_eq!(columns_word(3), "3 columns");
    }

    #[test]
    fn a_row_is_never_shorter_than_its_own_text() {
        let t = tokens(1.0);
        let m = model();
        let text = Style::new(t.text_px(TypeStep::Footnote), t.text()).size;
        for density in [
            crate::taskmgr::Density::Compact,
            crate::taskmgr::Density::Cozy,
            crate::taskmgr::Density::Comfortable,
        ] {
            let mut m = m.clone();
            m.prefs.density = density;
            assert!(
                row_height(&t, &m) > text,
                "{} would draw its rows inside each other",
                density.label()
            );
        }
    }

    #[test]
    fn columns_keep_their_natural_width_and_the_last_ones_are_dropped_when_they_cannot() {
        let m = model();
        let natural: f32 = m.prefs.table_width(1.0);
        let (wide, dropped) = column_widths(&m, 1.0, natural + 40.0);
        assert_eq!(dropped, 0, "a wide window shows every column");
        assert_eq!(wide.len(), m.prefs.columns.len());
        assert!(
            (wide.iter().map(|(_, w)| *w).sum::<f32>() - natural).abs() < 0.01,
            "every column keeps the width the preferences gave it"
        );
        let (narrow, dropped) = column_widths(&m, 1.0, 240.0);
        assert!(dropped > 0, "a 240-pixel window cannot show six columns");
        assert_eq!(narrow.len() + dropped, m.prefs.columns.len());
        assert_eq!(
            narrow.first().map(|(c, _)| *c),
            m.prefs.columns.first().copied(),
            "the first column is kept even when it does not fit: a table of \
             numbers with no names is not a table"
        );
        assert!(
            narrow.iter().map(|(_, w)| *w).sum::<f32>() <= 240.01,
            "what is drawn fits the window it was given"
        );
    }

    /// Draw the table and return the report plus every text run it drew.
    fn drawn(m: &TaskManager, t: &Tokens, w: u32, h: u32) -> (Report, Vec<String>) {
        let mut canvas = Canvas::new(w, h, crate::canvas::rgb(0x000000));
        canvas.record_text(true);
        let report = table(&mut canvas, t, m, Rect::new(0.0, 0.0, w as f32, h as f32));
        let runs: Vec<String> = canvas.text_runs().iter().map(|r| r.text.clone()).collect();
        let collisions = textaudit::collisions(canvas.text_runs());
        assert!(
            collisions.is_empty(),
            "the table drew text over text at {w}x{h}: {collisions:#?}"
        );
        (report, runs)
    }

    #[test]
    fn the_table_shows_every_process_and_says_what_it_could_not_fit() {
        let t = tokens(1.0);
        let mut m = model();
        m.prefs.sort_by(crate::taskmgr::SortKey::Cpu, false);
        let (report, runs) = drawn(&m, &t, 900, 400);
        assert_eq!(report.drawn, 3, "three processes fit in 400 pixels");
        assert_eq!(report.hidden, 0);
        assert!(report.complete(), "nothing was dropped silently");
        assert!(runs.iter().any(|r| r == "42.0"), "the live rate is drawn");
        assert!(
            runs.iter().any(|r| r == "\u{2014}"),
            "a process nobody measured draws a dash, not a zero: {runs:?}"
        );
        assert!(runs.iter().any(|r| r == "Name"), "the heading is drawn");
    }

    #[test]
    fn a_window_with_no_room_says_what_it_hid_rather_than_hiding_it() {
        let t = tokens(1.0);
        let mut m = model();
        // Forty rows in a 200-pixel-high window: most of them cannot be drawn.
        let base = m.rows.first().cloned();
        for i in 0..40 {
            let mut p = base.clone().expect("a fixture row");
            p.pid = 100 + i;
            p.name = format!("worker-{i}");
            m.rows.push(p);
        }
        let (report, runs) = drawn(&m, &t, 260, 200);
        assert!(report.hidden > 0, "a 200-pixel window cannot show 43 rows");
        assert!(!report.complete());
        assert!(
            report.columns_hidden > 0,
            "a 260-pixel window cannot show six columns: {report:?}"
        );
        assert!(
            runs.iter().any(|r| r.contains("more rows")),
            "the footer has to say that rows were left out: {runs:?}"
        );
        assert!(
            report.drawn > 0,
            "the rows that do fit are still drawn: {report:?}"
        );
        // The note about the hidden rows is the one that has to survive a narrow
        // footer; the hidden-column note is added when there is room for it, and
        // it is the note that goes when there is not.
        let (wide_report, wide_runs) = drawn(&m, &t, 420, 220);
        assert!(wide_report.hidden > 0, "{wide_report:?}");
        assert!(wide_report.columns_hidden > 0, "{wide_report:?}");
        assert!(
            wide_runs.iter().any(|r| r.contains("hidden")),
            "a wide footer names the columns it could not show: {wide_runs:?}"
        );
    }

    #[test]
    fn the_same_table_is_still_readable_at_a_larger_scale_and_a_different_width() {
        let mut m = model();
        m.selected = Some(0);
        for scale in [1.0, 1.25, 1.5] {
            for w in [320u32, 560, 1280] {
                let (report, _) = drawn(&m, &tokens(scale), w, 420);
                assert_eq!(report.drawn, 3, "at scale {scale} and width {w}");
            }
        }
    }

    #[test]
    fn an_empty_table_says_the_daemon_did_not_answer_rather_than_nothing_is_running() {
        let t = tokens(1.0);
        let mut m = TaskManager::new();
        m.rows.clear();
        let (report, runs) = drawn(&m, &t, 900, 400);
        assert_eq!(report.drawn, 0);
        assert!(
            runs.iter().any(|r| r.contains("daemon")),
            "an empty table says why it is empty: {runs:?}"
        );
    }

    #[test]
    fn a_meter_draws_the_band_it_crossed_and_the_marks_that_are_on() {
        let t = tokens(1.0);
        let mut m = model();
        m.machine.cpu = Some(0.95);
        m.machine.cpu_live = true;
        let mut canvas = Canvas::new(500, 400, crate::canvas::rgb(0x000000));
        canvas.record_text(true);
        let next = meters(&mut canvas, &t, &m, &Rect::new(0.0, 0.0, 500.0, 400.0), 0.0);
        assert!(next > 0.0, "the meters take room on the screen");
        let text: Vec<String> = canvas.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(text.iter().any(|r| r == "CPU"), "{text:?}");
        assert!(text.iter().any(|r| r == "95%"), "{text:?}");
        assert!(
            text.iter().any(|r| r.contains("critical 90 %")),
            "the marks that are on are printed: {text:?}"
        );
        let collisions = textaudit::collisions(canvas.text_runs());
        assert!(collisions.is_empty(), "{collisions:#?}");
    }

    #[test]
    fn an_unmeasured_meter_is_a_dash_and_claims_nothing() {
        let t = tokens(1.0);
        let m = model();
        let meter = m
            .meters()
            .into_iter()
            .find(|m| m.metric == crate::taskmgr::Metric::Swap)
            .expect("the swap meter is always drawn");
        assert!(!meter.measured);
        assert_eq!(meter.text, "\u{2014}");
        assert_eq!(meter.level, None);
        let mut canvas = Canvas::new(400, 200, crate::canvas::rgb(0x000000));
        canvas.record_text(true);
        meters(&mut canvas, &t, &m, &Rect::new(0.0, 0.0, 400.0, 200.0), 0.0);
        assert!(canvas.text_runs().iter().any(|r| r.text == "\u{2014}"));
    }
}
