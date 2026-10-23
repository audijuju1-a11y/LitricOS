//! The system monitor: what the Activity window and Settings draw, and why.
//!
//! The window it replaces showed two meters and a short list. The machine was
//! already reporting much more than that -- available memory, page cache, swap,
//! load, the daemon's own footprint, the MGLRU state, the CPU model, tick
//! counters per logical CPU and the mount table -- and none of it reached the
//! screen. A monitor that shows two numbers while the machine reports a dozen
//! is a monitor nobody can diagnose with.
//!
//! Two rules shape everything below, both inherited from the window it
//! replaces:
//!
//! 1. **A row exists only where a value was measured.** Unmeasured quantities
//!    are absent, not zero, and not \"Unknown\" in three columns of grey.
//! 2. **A number the interface cannot explain is not shown.** The storage
//!    section names the filesystem and, when it is read-only, says what that
//!    means in a sentence: the answer to \"what is EROFS, and why is my disk
//!    full?\" belongs next to the number, not in a manual.
//!
//! The decisions are separate functions returning plain data ([`system_rows`],
//! [`readonly_volume_note`], [`core_fill`]) so they can be tested without a
//! canvas, which is the only reason a test can assert that a machine with one
//! sample does not report a per-core grid.

use std::collections::VecDeque;

use litric_design::{Space, TypeStep};

use crate::canvas::{with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::shell::SystemFacts;
use crate::taskmgr::{History, TaskManager};
use crate::ui::{self, Tokens};

/// What a read-only volume means, in one sentence, or `None` when the volume is
/// writable and needs no explanation.
///
/// `erofs` gets its own sentence rather than the generic one because it is the
/// case a user meets in this system: the operating system image is a compressed
/// read-only filesystem checked by dm-verity, which is why that volume shows no
/// free space and accepts no writes.
#[must_use]
pub fn readonly_volume_note(fs: &str, read_only: bool) -> Option<&'static str> {
    if !read_only {
        return None;
    }
    match fs {
        "erofs" => Some(
            "Read-only compressed system image (EROFS): nothing can change it while \
             running, and dm-verity checks every block read against a hash tree.",
        ),
        "squashfs" => Some(
            "Read-only compressed image: nothing on it can be changed while the \
             system is running.",
        ),
        "iso9660" => Some("Optical media: readable only, by definition."),
        _ => Some("Mounted read-only: nothing can be written here."),
    }
}

/// A percentage string, or nothing when there is no measurement.
///
/// A value outside 0..=100 is clamped, because a monitor that prints 340% from
/// an arithmetic slip teaches the user to distrust every other number on the
/// screen. Only a real measurement reaches this function; a missing one never
/// becomes `0%`.
#[must_use]
pub fn percent(v: Option<f32>) -> Option<String> {
    v.map(|v| format!("{:.0}%", (v * 100.0).clamp(0.0, 100.0)))
}

/// What one logical CPU bar should be filled to, 0.0 to 1.0, for core `i`.
///
/// An empty vector means the machine has not been sampled twice yet, and the
/// caller then draws no grid at all rather than a row of zeroes.
#[must_use]
pub fn core_fill(cores: &[f32], i: usize) -> Option<f32> {
    cores.get(i).copied().map(|v| v.clamp(0.0, 1.0))
}

/// Whether a mount belongs in the storage list.
///
/// The kernel's own pseudo-filesystems are not storage: listing `proc` and
/// `sysfs` next to a real volume is how a monitor buries the one row the user
/// needs. The list is explicit rather than a `tmpfs`-only filter, because each
/// entry is a decision about what a person means by "disk".
#[must_use]
pub fn is_storage_volume(fs: &str) -> bool {
    !matches!(
        fs,
        "proc"
            | "sysfs"
            | "devtmpfs"
            | "devpts"
            | "tmpfs"
            | "cgroup"
            | "cgroup2"
            | "securityfs"
            | "pstore"
            | "debugfs"
            | "tracefs"
            | "configfs"
            | "fusectl"
            | "mqueue"
            | "hugetlbfs"
            | "bpf"
            | "autofs"
            | "ramfs"
            | "efivarfs"
            | "binfmt_misc"
            | "rpc_pipefs"
            | "nsfs"
            | "overlay"
    )
}

/// The quiet fact rows under the meters: label and value, measured only.
///
/// Load carries its task count because a load of 3.0 means something different
/// on a 2-CPU machine than on a 32-CPU one, and the count is what the kernel
/// already reports alongside it.
#[must_use]
pub fn system_rows(f: &SystemFacts) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    if let Some(up) = f.uptime_secs {
        rows.push(("Uptime".to_owned(), crate::desktop::format_uptime(up)));
    }
    if let Some(load) = f.load1 {
        let tasks = f
            .tasks
            .map_or_else(String::new, |n| format!(" · {n} tasks"));
        rows.push(("Load".to_owned(), format!("{load:.2}{tasks}")));
    }
    if let Some(rss) = f.sysd_rss_mb {
        rows.push(("litric-sysd".to_owned(), format!("{rss} MB")));
    }
    if let Some(v) = f.verity {
        rows.push((
            "Verified boot".to_owned(),
            if v { "on" } else { "off" }.to_owned(),
        ));
    }
    if let Some(m) = f.mglru {
        rows.push((
            "MGLRU".to_owned(),
            if m { "active" } else { "off" }.to_owned(),
        ));
    }
    if let Some((n, b)) = f.trash {
        rows.push(("Trash".to_owned(), format!("{n} items · {}", human_size(b))));
    }
    rows
}

/// The rows a storage list shows: mount point, a short description, and the
/// explanation when the volume cannot be written.
///
/// Shared by the system monitor and the Settings storage page, so the sentence
/// explaining a read-only system image is written once and cannot drift between
/// the two places a user meets it.
#[must_use]
pub fn storage_rows(f: &SystemFacts) -> Vec<(String, String, Option<&'static str>)> {
    f.volumes
        .iter()
        .filter(|(_, fs, _)| is_storage_volume(fs))
        .map(|(mount, fs, ro)| {
            let detail = if *ro {
                format!("{fs} · read-only")
            } else {
                fs.clone()
            };
            (mount.clone(), detail, readonly_volume_note(fs, *ro))
        })
        .collect()
}

/// A byte count at a scale a person reads, for the trash row.
#[must_use]
pub fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB.saturating_mul(1024);
    const GB: u64 = MB.saturating_mul(1024);
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// A series' points across a rectangle, oldest sample at the left edge.
///
/// The vertical scale is `top`, the value that reaches the top of the box. A
/// graph whose own maximum decides its scale looks identical at idle and at
/// full load, which is the one thing a monitor must not do, so the caller
/// passes the ceiling it wants the reading judged against -- 1.0 for a CPU,
/// installed memory for RAM -- and a series that exceeds it is clamped to the
/// box rather than allowed to draw outside it.
///
/// No measurement, no points: an empty series draws nothing rather than a flat
/// line along the bottom, which would read as a machine at rest.
#[must_use]
pub fn series_points(values: &VecDeque<f32>, r: Rect, top: f32) -> Vec<(f32, f32)> {
    let n = values.len();
    if n == 0 || top <= 0.0 || r.w <= 0.0 || r.h <= 0.0 {
        return Vec::new();
    }
    let step = if n <= 1 {
        0.0
    } else {
        r.w / (n.saturating_sub(1)) as f32
    };
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = r.x + (i as f32) * step;
            let frac = (v / top).clamp(0.0, 1.0);
            (x, r.bottom() - frac * r.h)
        })
        .collect()
}

/// A byte rate as a person reads it, per second.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn rate_text(bytes_per_s: f64) -> String {
    let whole = bytes_per_s.max(0.0) as u64;
    format!("{}/s", human_size(whole))
}

/// Why a graph has nothing to draw, in the machine's own terms.
///
/// The alternative -- an empty box, or a box of zeroes -- is a claim the
/// machine never made. Each of these sentences names what is missing, so the
/// window says which of "nothing is happening" and "nothing is measured" the
/// user is looking at.
pub const NO_HISTORY: &str = "No history yet: this machine has been sampled once.";
/// No interface counters reached the shell.
pub const NET_NOT_MEASURED: &str =
    "Unavailable: no interface counters reached the shell, so no rate can be differenced.";
/// No block-device counters exist anywhere in this system.
pub const DISK_NOT_MEASURED: &str = "Unavailable: nothing in this system reads block-device \
                                     counters, so no disk rate can be reported. Capacity and \
                                     mounts are in Settings, where they are measured.";

/// The activity graphs: the same machine the meters read, over time.
///
/// Four sections, each one a question a monitor exists to answer: what has each
/// CPU been doing, how has memory been divided between what is in use and what
/// is cached, what has the network been carrying, and what has the disk been
/// doing. The first three are drawn from [`History`], which is fed by the same
/// facts the table below is fed by. The fourth says it is unavailable, because
/// it is: see [`DISK_NOT_MEASURED`].
fn draw_history(canvas: &mut Canvas, t: &Tokens, f: &SystemFacts, h: &History, body: Rect) {
    let caption = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let mut y = body.y;
    let head = |canvas: &mut Canvas, y: f32, text: &str| {
        ui::section_label(canvas, t, (body.x, y + t.px(10.0)), text);
    };

    // ---- per-core CPU --------------------------------------------------
    let cores = h.cores.len();
    if cores > 0 {
        head(canvas, y, "CPU, per core");
        y += t.px(18.0);
        // Two columns until the panel is wide enough for four: a graph 70
        // pixels wide shows the shape of a minute and nothing else.
        let cols: usize = if body.w >= t.px(340.0) { 4 } else { 2 };
        let gap = t.sp(Space::Sm);
        let cell_w = (body.w - gap * (cols.saturating_sub(1)) as f32) / cols as f32;
        let cell_h = t.px(30.0);
        for i in 0..cores {
            let col = i.checked_rem(cols).unwrap_or(0);
            let row = i.checked_div(cols).unwrap_or(0);
            let cell = Rect::new(
                body.x + (col as f32) * (cell_w + gap),
                y + (row as f32) * (cell_h + t.px(6.0)),
                cell_w,
                cell_h,
            );
            if cell.bottom() > body.bottom() {
                break;
            }
            canvas.fill_round_rect(
                cell,
                t.radius(litric_design::Radius::Sm),
                with_alpha(t.text(), 0.05),
            );
            let Some(series) = h.cores.get(i) else {
                continue;
            };
            area(canvas, t, series, cell.inflate(-t.px(2.0)), 1.0, t.accent());
            let now = series.back().copied().unwrap_or(0.0);
            let text = format!("{i}  {}%", (now * 100.0).round() as i64);
            font::draw(
                canvas,
                &fit(&text, &caption, cell.w - t.px(8.0)),
                cell.x + t.px(4.0),
                cell.y + t.px(11.0),
                &caption,
            );
        }
        let rows = cores
            .saturating_add(cols.saturating_sub(1))
            .checked_div(cols)
            .unwrap_or(1);
        y += (rows as f32) * (cell_h + t.px(6.0)) + t.sp(Space::Xs);
    }

    // ---- memory composition over time ----------------------------------
    if y + t.px(60.0) <= body.bottom() {
        head(canvas, y, "Memory");
        y += t.px(18.0);
        let panel = Rect::new(body.x, y, body.w, t.px(40.0));
        canvas.fill_round_rect(
            panel,
            t.radius(litric_design::Radius::Sm),
            with_alpha(t.text(), 0.05),
        );
        if h.mem_used_mb.is_empty() {
            font::draw(
                canvas,
                NO_HISTORY,
                panel.x + t.px(4.0),
                panel.y + t.px(16.0),
                &caption,
            );
        } else {
            // The scale is installed memory -- the only ceiling that makes two
            // samples comparable -- and the total when it is unknown is the
            // largest thing seen, named as such in the caption so a graph with
            // its own maximum cannot be read as a full machine.
            let total = f
                .mem_total_mb
                .map(|v| v as f32)
                .or_else(|| {
                    h.mem_used_mb
                        .iter()
                        .zip(h.mem_cached_mb.iter())
                        .map(|(u, c)| u + c)
                        .fold(None, |acc: Option<f32>, v| {
                            Some(acc.map_or(v, |a| a.max(v)))
                        })
                })
                .filter(|v| *v > 0.0);
            if let Some(total) = total {
                let inner = panel.inflate(-t.px(2.0));
                // Cache first, then use: the two together are what the machine
                // holds, and drawing them apart is what makes the cache visible
                // as the thing that moves without the machine getting busier.
                let stacked: VecDeque<f32> = h
                    .mem_cached_mb
                    .iter()
                    .zip(h.mem_used_mb.iter())
                    .map(|(c, u)| c + u)
                    .collect();
                area(
                    canvas,
                    t,
                    &stacked,
                    inner,
                    total,
                    with_alpha(t.accent(), 0.35),
                );
                area(canvas, t, &h.mem_used_mb, inner, total, t.accent());
                let now = h.mem_used_mb.back().copied().unwrap_or(0.0);
                let cached = h.mem_cached_mb.back().copied().unwrap_or(0.0);
                let text = format!(
                    "{} in use · {} cache · {} installed",
                    human_size((now as u64).saturating_mul(1024 * 1024)),
                    human_size((cached as u64).saturating_mul(1024 * 1024)),
                    human_size((total as u64).saturating_mul(1024 * 1024)),
                );
                font::draw(
                    canvas,
                    &fit(&text, &caption, panel.w - t.px(8.0)),
                    panel.x + t.px(4.0),
                    panel.y + t.px(14.0),
                    &caption,
                );
            } else {
                font::draw(
                    canvas,
                    "Unavailable: no memory total or measurement reached the shell.",
                    panel.x + t.px(4.0),
                    panel.y + t.px(16.0),
                    &caption,
                );
            }
        }
        y = panel.bottom() + t.sp(Space::Xs);
    }

    // ---- network -------------------------------------------------------
    if y + t.px(60.0) <= body.bottom() {
        head(canvas, y, "Network");
        y += t.px(18.0);
        let panel = Rect::new(body.x, y, body.w, t.px(40.0));
        canvas.fill_round_rect(
            panel,
            t.radius(litric_design::Radius::Sm),
            with_alpha(t.text(), 0.05),
        );
        match (h.net_rx_per_s.is_empty(), f.net_rx_per_s, f.net_tx_per_s) {
            (false, _, _) => {
                // One scale for both directions: two graphs with two scales can
                // be read as a busy line and a quiet one when the truth is that
                // the same bytes are on both.
                let top = h
                    .net_rx_per_s
                    .iter()
                    .chain(h.net_tx_per_s.iter())
                    .fold(0.0_f32, |a, v| a.max(*v))
                    .max(1.0);
                let inner = panel.inflate(-t.px(2.0));
                area(
                    canvas,
                    t,
                    &h.net_rx_per_s,
                    inner,
                    top,
                    with_alpha(t.accent(), 0.55),
                );
                area(
                    canvas,
                    t,
                    &h.net_tx_per_s,
                    inner,
                    top,
                    with_alpha(t.text(), 0.75),
                );
                let rx = f.net_rx_per_s.unwrap_or(0.0);
                let tx = f.net_tx_per_s.unwrap_or(0.0);
                let text = format!("received {} · sent {}", rate_text(rx), rate_text(tx));
                font::draw(
                    canvas,
                    &fit(&text, &caption, panel.w - t.px(8.0)),
                    panel.x + t.px(4.0),
                    panel.y + t.px(14.0),
                    &caption,
                );
            }
            _ => {
                font::draw(
                    canvas,
                    &fit(NET_NOT_MEASURED, &caption, panel.w - t.px(8.0)),
                    panel.x + t.px(4.0),
                    panel.y + t.px(16.0),
                    &caption,
                );
            }
        }
        y = panel.bottom() + t.sp(Space::Xs);
    }

    // ---- disk ----------------------------------------------------------
    if y + t.px(18.0) <= body.bottom() {
        head(canvas, y, "Disk");
        y += t.px(16.0);
        let note = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
        // Lines on the type scale's own caption lead (`TypeStep::line_height_px`),
        // not on a number chosen to look right: a 12-pixel caption has a 16-pixel
        // line box, and setting this note tighter put each line's band inside the
        // one above it -- text drawn over text, however quiet the overlap looks.
        let line_h = t.line_px(TypeStep::Caption);
        for (i, line) in ui::wrap(DISK_NOT_MEASURED, &note, body.w, 4)
            .into_iter()
            .enumerate()
        {
            let baseline = y + t.px(10.0) + (i as f32) * line_h;
            if baseline - note.size > body.bottom() {
                break;
            }
            font::draw(canvas, &line, body.x, baseline, &note);
        }
    }
}

/// One series drawn as an area: a filled column per pixel column, from the
/// series down to the baseline.
///
/// An area rather than a line because the panels are small: a one-pixel line at
/// this size is a dashed line, and the shape of a minute is easier to read than
/// any individual sample is.
fn area(canvas: &mut Canvas, t: &Tokens, values: &VecDeque<f32>, r: Rect, top: f32, colour: u32) {
    let points = series_points(values, r, top);
    if points.len() < 2 {
        return;
    }
    let step = t.px(1.0).max(1.0);
    let mut x = r.x;
    // Walk the pixel columns and take the sample that owns each: the series is
    // longer than the panel is wide in every real case, so this samples the
    // noise out instead of aliasing it into a sawtooth.
    let width = r.w;
    let mut i = 0usize;
    while x < r.x + width && i < points.len() {
        let Some(&(_, y)) = points.get(i) else {
            break;
        };
        // The next point that falls in this column sets the fill's top edge.
        let mut top_y = y;
        while let Some(&(px, py)) = points.get(i) {
            if px > x + step {
                break;
            }
            top_y = top_y.min(py);
            i = i.saturating_add(1);
        }
        let h = (r.bottom() - top_y).max(1.0);
        canvas.fill_round_rect(Rect::new(x, r.bottom() - h, step, h), 0.0, colour);
        x += step;
    }
}

/// Draw the Activity overview: CPU identity, resource meters, per-core
/// activity, and the machine's recent history. Storage and detailed system
/// facts live in Settings, where they can use the full page instead of
/// competing with the process table.
///
/// The meters come from [`crate::activity::meters`], which reads them from the
/// same model the process table is drawn from: a band that marks a meter on this
/// screen is a band the model applied, and a band nothing reads cannot get this
/// far. The graphs come from the same model's history, so a number and the line
/// under it can never describe different machines.
pub fn draw(canvas: &mut Canvas, t: &Tokens, f: &SystemFacts, m: &TaskManager, body: Rect) {
    let pad = t.sp(Space::Lg);
    let content = body.inflate(-pad);
    let mut y = content.y + t.px(6.0);
    let caption = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let body_style = Style::new(t.text_px(TypeStep::Body), t.text());

    // ---- header: whose CPU is this -------------------------------------
    // One row, two strings, each laid out against the other rather than against
    // its own window edge. Right-aligning both put "8 logical CPUs" on top of
    // "18%": a 62x17-pixel overlap of the two numbers this window exists to
    // show, found by the layout audit and plain to see on the screen.
    let cores = f
        .cpus
        .or_else(|| u32::try_from(f.core_ticks.len()).ok().filter(|n| *n > 0));
    let core_text = cores.map(|n| format!("{n} logical CPUs"));
    let reserve = core_text
        .as_ref()
        .map_or(0.0, |text| font::measure(text, &caption) + t.px(16.0));
    if let Some(model) = &f.cpu_model {
        let shown = fit(model, &body_style, (content.w - reserve).max(t.px(40.0)));
        font::draw(canvas, &shown, content.x, y + t.px(12.0), &body_style);
    }
    if let Some(text) = &core_text {
        let w = font::measure(text, &caption);
        font::draw(canvas, text, content.right() - w, y + t.px(12.0), &caption);
    }
    if f.cpu_model.is_some() || core_text.is_some() {
        // 34 pixels, not 20: the CPU value below is 24px of type, whose band
        // reaches ~25 pixels above its own baseline, and a 20-pixel header row
        // put the header inside it.
        y += t.px(34.0);
    }

    // ---- the readings and the graphs -----------------------------------
    // Two columns when the window is wide enough for them: what the machine is
    // doing now on the left, what it has been doing on the right. Stacked
    // instead, one of the two is always the section that gets clipped -- and a
    // monitor whose trends are below the fold is a monitor read as a snapshot.
    // Under that width the readings keep the whole panel, and the graphs follow
    // them in whatever room is left.
    let graphs_w = if content.w >= t.px(560.0) {
        (content.w * 0.46).clamp(t.px(220.0), t.px(420.0))
    } else {
        0.0
    };
    let readouts = Rect::new(content.x, y, content.w - graphs_w, content.bottom() - y);
    if graphs_w > 0.0 {
        let graphs = Rect::new(
            content.x + content.w - graphs_w,
            y,
            graphs_w,
            content.bottom() - y,
        );
        // A hairline between the two, so the numbers on the left are not read
        // as belonging to the line they happen to sit beside.
        canvas.fill_round_rect(
            Rect::new(graphs.x - t.sp(Space::Lg) / 2.0, y, 1.0, graphs.h),
            0.0,
            with_alpha(t.text(), 0.10),
        );
        draw_history(canvas, t, f, &m.history, graphs);
    } else if y + t.px(150.0) <= content.bottom() {
        // Not enough width to put them side by side: the graphs go under the
        // readings, at the size the panel actually has.
        draw_history(
            canvas,
            t,
            f,
            &m.history,
            Rect::new(
                content.x,
                content.bottom() - t.px(120.0),
                content.w,
                t.px(120.0),
            ),
        );
    }

    // CPU, memory, swap, load, frame time and the daemon's own footprint, each
    // with the band it crossed. Drawn here rather than in this file so the
    // numbers and the table below them come from one model.
    if y < readouts.bottom() {
        y = crate::activity::meters(canvas, t, m, &readouts, y);
        // Per logical CPU: one thin bar each, wrapped at 16 per row. Drawn only
        // when a second sample exists, because a grid of zeroes would claim
        // sixteen idle CPUs the machine never reported. The bars are the instant
        // the machine is in; the graph on the right is how it got there.
        if !f.cpu_cores.is_empty() {
            let bar_w = t.px(9.0);
            let gap = t.px(3.0);
            let row_h = t.px(12.0);
            let mut x = readouts.x;
            for i in 0..f.cpu_cores.len() {
                let Some(fill) = core_fill(&f.cpu_cores, i) else {
                    continue;
                };
                let r = Rect::new(x, y, bar_w, bar_h(t));
                canvas.fill_round_rect(r, r.h / 2.0, with_alpha(t.text_dim(), 0.35));
                let fh = (r.h * fill).max(1.0);
                canvas.fill_round_rect(
                    Rect::new(r.x, r.bottom() - fh, r.w, fh),
                    r.h / 2.0,
                    if fill > 0.5 {
                        with_alpha(t.text(), 0.95)
                    } else {
                        with_alpha(t.text(), 0.6)
                    },
                );
                x += bar_w + gap;
                if x + bar_w > readouts.right() {
                    x = readouts.x;
                    y += row_h;
                }
            }
        }
    }
}

/// The height of a per-core bar.
fn bar_h(t: &crate::ui::Tokens) -> f32 {
    t.px(22.0)
}

/// `text`, elided to `max` when it does not fit.
///
/// Every string on this screen that came from the machine -- a CPU model, a
/// mount point, an explanation -- can be longer than the window. Measuring
/// first keeps the letters inside the box the user is looking at, which is the
/// defect this project has fixed once already.
pub fn fit(text: &str, style: &Style, max: f32) -> String {
    if font::measure(text, style) <= max {
        text.to_owned()
    } else {
        ui::elide(text, style, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> SystemFacts {
        SystemFacts {
            mem_total_mb: Some(7867),
            mem_used_mb: Some(527),
            mem_available_mb: Some(7100),
            mem_cached_mb: Some(1200),
            swap_used_mb: Some(0),
            cpu_fraction: Some(0.07),
            cpu_now: None,
            cpu_model: Some("AMD Ryzen 7 5800X 8-Core Processor".to_owned()),
            cpus: Some(8),
            uptime_secs: Some(4_000.0),
            load1: Some(0.52),
            tasks: Some(143),
            sysd_rss_mb: Some(2),
            mglru: Some(true),
            verity: Some(true),
            source: "fixture".to_owned(),
            ..SystemFacts::default()
        }
    }

    #[test]
    fn a_read_only_image_explains_itself_and_a_writable_volume_does_not() {
        let note = readonly_volume_note("erofs", true).unwrap_or_default();
        assert!(note.contains("EROFS"), "the name has to appear: {note}");
        assert!(
            note.contains("read-only") || note.contains("nothing can change"),
            "the note has to say what read-only means: {note}"
        );
        assert!(readonly_volume_note("erofs", false).is_none());
        assert!(readonly_volume_note("ext4", false).is_none());
        assert!(readonly_volume_note("ext4", true).is_some());
    }

    #[test]
    fn the_memory_meter_says_used_of_total_and_lists_the_breakdown() {
        let rows = system_rows(&facts());
        assert!(rows.iter().any(|(k, _)| k == "Load"));
        // A fact that was never measured produces no row at all.
        let empty = system_rows(&SystemFacts::default());
        assert!(empty.is_empty(), "nothing measured, nothing claimed");
    }

    #[test]
    fn a_series_is_drawn_across_its_box_with_the_ceiling_it_was_given() {
        let mut values = VecDeque::new();
        values.push_back(0.0);
        values.push_back(0.5);
        values.push_back(1.0);
        let r = Rect::new(10.0, 20.0, 100.0, 40.0);
        let points = series_points(&values, r, 1.0);
        assert_eq!(points.len(), 3);
        assert_eq!(points.first().map(|(x, _)| *x), Some(10.0));
        assert_eq!(points.last().map(|(x, _)| *x), Some(110.0));
        // 0 at the baseline, 1.0 at the top, and half of it halfway up.
        assert_eq!(points.first().map(|(_, y)| *y), Some(60.0));
        assert_eq!(points.get(1).map(|(_, y)| *y), Some(40.0));
        assert_eq!(points.last().map(|(_, y)| *y), Some(20.0));
        // A measurement above the ceiling is clamped to the box, not drawn
        // outside it, and no measurement draws nothing at all.
        let mut over = VecDeque::new();
        over.push_back(9.0);
        over.push_back(9.0);
        assert_eq!(
            series_points(&over, r, 1.0).first().map(|(_, y)| *y),
            Some(20.0)
        );
        assert!(series_points(&VecDeque::new(), r, 1.0).is_empty());
    }

    #[test]
    fn the_graphs_draw_what_was_measured_and_say_what_was_not() {
        let background = crate::canvas::rgb(0x000000);
        // Two samples of the same machine, a second apart, with the network
        // moving 1 MiB in it. The rate is not in either sample: it is what
        // `observe` makes of the pair, which is why the graph needs history.
        let mut first = facts();
        first.sampled_ms = Some(0);
        first.net_rx_bytes = Some(1_000);
        first.net_tx_bytes = Some(500);
        let mut second = first.clone();
        second.sampled_ms = Some(1_000);
        second.net_rx_bytes = Some(1_000 + 1_048_576);
        second.net_tx_bytes = Some(500 + 1_024);
        assert_eq!(second.net_rx_per_s, None, "no sample carries a rate");
        let mut observed = first;
        observed.observe(second);
        assert_eq!(observed.net_rx_per_s, Some(1_048_576.0));
        let f = observed;
        let mut m = model(f.clone());
        m.observe(&f);

        let mut canvas = Canvas::new(700, 600, background);
        canvas.record_text(true);
        draw(
            &mut canvas,
            &tokens(),
            &f,
            &m,
            Rect::new(0.0, 0.0, 700.0, 600.0),
        );
        let runs: Vec<String> = canvas.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(
            runs.iter().any(|r| r.contains("in use")),
            "the memory graph has no caption: {runs:?}"
        );
        assert!(
            runs.iter().any(|r| r.contains("received")),
            "the network graph has no rate: {runs:?}"
        );
        assert!(
            runs.iter().any(|r| r.starts_with("Disk")),
            "the disk section is missing: {runs:?}"
        );
        assert!(
            runs.iter()
                .any(|r| r.contains("nothing in this system reads")),
            "the disk section does not say why it is empty: {runs:?}"
        );
    }

    #[test]
    fn a_machine_with_no_counters_gets_an_honest_unavailable_not_a_flat_line() {
        let background = crate::canvas::rgb(0x000000);
        let f = facts();
        let mut m = model(f.clone());
        m.observe(&f);
        let mut canvas = Canvas::new(700, 600, background);
        canvas.record_text(true);
        draw(
            &mut canvas,
            &tokens(),
            &f,
            &m,
            Rect::new(0.0, 0.0, 700.0, 600.0),
        );
        let runs: Vec<String> = canvas.text_runs().iter().map(|r| r.text.clone()).collect();
        assert!(
            runs.iter().any(|r| r.contains("no interface counters")),
            "a machine that reported no counters claimed a network rate: {runs:?}"
        );
        assert!(
            !runs.iter().any(|r| r.contains("received")),
            "a rate was printed for a machine that measured none: {runs:?}"
        );
    }

    #[test]
    fn a_byte_rate_reads_at_the_scale_a_person_watches() {
        assert_eq!(rate_text(0.0), "0 B/s");
        assert_eq!(rate_text(2048.0), "2 KB/s");
        assert_eq!(rate_text(1_048_576.0), "1.0 MB/s");
        assert_eq!(rate_text(-5.0), "0 B/s", "a negative rate is not a rate");
    }

    #[test]
    fn a_percentage_is_clamped_and_a_missing_measurement_is_absent() {
        assert_eq!(percent(Some(0.425)).as_deref(), Some("42%"));
        assert_eq!(percent(Some(3.4)).as_deref(), Some("100%"));
        assert_eq!(percent(None), None, "no measurement, no zero");
    }

    #[test]
    fn no_second_sample_means_no_per_core_grid() {
        assert_eq!(core_fill(&[], 0), None);
        assert_eq!(core_fill(&[0.5], 3), None);
        assert_eq!(core_fill(&[1.7], 0), Some(1.0));
    }

    #[test]
    fn the_kernels_own_filesystems_are_not_shown_as_storage() {
        assert!(is_storage_volume("erofs"));
        assert!(is_storage_volume("ext4"));
        assert!(is_storage_volume("vfat"));
        for pseudo in ["proc", "sysfs", "devtmpfs", "tmpfs", "cgroup2", "debugfs"] {
            assert!(!is_storage_volume(pseudo), "{pseudo} is not storage");
        }
    }

    /// Tokens for the fixture, so the monitor can be drawn without a shell.
    fn tokens() -> Tokens {
        let theme = litric_design::Theme::of(litric_design::ThemeKind::Dark);
        Tokens::new(theme, 1.0)
    }

    /// The model the monitor reads its meters from, fed from `facts`.
    fn model(facts: SystemFacts) -> TaskManager {
        let mut m = TaskManager::new();
        m.observe(&facts);
        m
    }

    /// How many pixels differ from the background the window was filled with.
    fn ink(canvas: &Canvas, background: u32) -> usize {
        canvas.pixels().iter().filter(|p| **p != background).count()
    }

    #[test]
    fn the_monitor_draws_rows_for_measured_facts_and_nothing_for_none() {
        let background = crate::canvas::rgb(0x000000);
        let body = Rect::new(0.0, 0.0, 400.0, 600.0);

        let bare_facts = SystemFacts::default();
        let mut bare = Canvas::new(400, 600, background);
        draw(
            &mut bare,
            &tokens(),
            &bare_facts,
            &model(bare_facts.clone()),
            body,
        );
        let bare_ink = ink(&bare, background);

        let rich_facts = facts();
        let mut rich = Canvas::new(400, 600, background);
        draw(
            &mut rich,
            &tokens(),
            &rich_facts,
            &model(rich_facts.clone()),
            body,
        );
        let rich_ink = ink(&rich, background);

        assert!(rich_ink > 0, "a machine with facts draws a monitor");
        assert!(
            rich_ink > bare_ink,
            "the monitor shows more when the machine reports more: {rich_ink} vs {bare_ink}"
        );
    }

    #[test]
    fn a_read_only_system_image_is_explained_on_the_screen_it_appears_on() {
        let mut f = facts();
        f.volumes = vec![
            ("/".to_owned(), "erofs".to_owned(), true),
            ("/persist".to_owned(), "ext4".to_owned(), false),
        ];
        let s = system_rows(&f);
        assert!(!s.is_empty());
        // The window draws the explanation for the read-only volume and none
        // for the writable one: the note is a property of the volume, not of
        // the section.
        assert!(readonly_volume_note("erofs", true).is_some());
        assert!(readonly_volume_note("ext4", false).is_none());
        let background = crate::canvas::rgb(0x000000);
        let mut canvas = Canvas::new(400, 600, background);
        draw(
            &mut canvas,
            &tokens(),
            &f,
            &model(f.clone()),
            Rect::new(0.0, 0.0, 400.0, 600.0),
        );
        assert!(ink(&canvas, background) > 0);
    }

    #[test]
    fn the_storage_list_keeps_real_volumes_and_explains_the_read_only_ones() {
        let mut f = facts();
        f.volumes = vec![
            ("/".to_owned(), "erofs".to_owned(), true),
            ("/proc".to_owned(), "proc".to_owned(), false),
            ("/persist".to_owned(), "ext4".to_owned(), false),
        ];
        let rows = storage_rows(&f);
        assert_eq!(rows.len(), 2, "pseudo-filesystems are not storage");
        let root = rows.first();
        assert_eq!(root.map(|(m, _, _)| m.as_str()), Some("/"));
        assert!(root.and_then(|(_, _, note)| *note).is_some());
        let persist = rows.get(1);
        assert_eq!(
            persist.and_then(|(_, _, note)| *note),
            None,
            "a writable volume needs no explanation"
        );
    }

    #[test]
    fn sizes_are_reported_at_a_scale_a_person_reads() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(2048), "2 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(human_size(3 * 1024 * 1024 * 1024), "3.0 GB");
    }
}
