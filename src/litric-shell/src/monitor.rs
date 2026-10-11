//! The system monitor: what the Activity window draws, and why.
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

use litric_design::{Space, TypeStep};

use crate::canvas::{with_alpha, Canvas, Rect};
use crate::font::{self, Style};
use crate::shell::SystemFacts;
use crate::taskmgr::TaskManager;
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

/// Draw the monitor: the machine's own numbers, then the meters a person set
/// bands on.
///
/// The meters come from [`crate::activity::meters`], which reads them from the
/// same model the process table is drawn from: a band that marks a meter on this
/// screen is a band the model applied, and a band nothing reads cannot get this
/// far.
pub fn draw(canvas: &mut Canvas, t: &Tokens, f: &SystemFacts, m: &TaskManager, body: Rect) {
    let pad = t.sp(Space::Lg);
    let content = body.inflate(-pad);
    let mut y = content.y + t.px(6.0);
    let caption = Style::new(t.text_px(TypeStep::Caption), t.text_dim());
    let body_style = Style::new(t.text_px(TypeStep::Body), t.text());
    let dim_bold = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());

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

    // ---- the meters ----------------------------------------------------
    // CPU, memory, swap, load, frame time and the daemon's own footprint, each
    // with the band it crossed. Drawn here rather than in this file so the
    // numbers and the table below them come from one model.
    if y < content.bottom() {
        y = crate::activity::meters(canvas, t, m, &content, y);
        // Per logical CPU: one thin bar each, wrapped at 16 per row. Drawn only
        // when a second sample exists, because a grid of zeroes would claim
        // sixteen idle CPUs the machine never reported.
        if !f.cpu_cores.is_empty() {
            let bar_w = t.px(9.0);
            let gap = t.px(3.0);
            let row_h = t.px(12.0);
            let mut x = content.x;
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
                if x + bar_w > content.right() {
                    x = content.x;
                    y += row_h;
                }
            }
            y += row_h + t.px(6.0);
        }
    }

    // ---- Memory --------------------------------------------------------
    // The used-of-total reading is a meter above, with its band; what is left
    // here is the breakdown, which no meter shows.
    if y < content.bottom() {
        section(canvas, t, &content, &mut y, "Memory");
        let mut rows: Vec<(String, String)> = Vec::new();
        if let (Some(used), Some(total)) = (f.mem_used_mb, f.mem_total_mb) {
            rows.push(("Used".to_owned(), format!("{used} of {total} MB")));
        }
        if let Some(a) = f.mem_available_mb {
            rows.push(("Available".to_owned(), format!("{a} MB")));
        }
        if let Some(c) = f.mem_cached_mb {
            rows.push(("Cache".to_owned(), format!("{c} MB")));
        }
        if let Some(s) = f.swap_used_mb {
            rows.push(("Swap in use".to_owned(), format!("{s} MB")));
        }
        y = rows_drawn(canvas, t, &content, y, &rows, &caption, &body_style);
    }

    // ---- Storage -------------------------------------------------------
    if y < content.bottom() {
        section(canvas, t, &content, &mut y, "Storage");
        let volumes: Vec<(String, String, bool)> = f
            .volumes
            .iter()
            .filter(|(_, fs, _)| is_storage_volume(fs))
            .cloned()
            .collect();
        if volumes.is_empty() {
            let s = Style::new(t.text_px(TypeStep::Footnote), t.text_dim());
            font::draw(
                canvas,
                "Unavailable - the kernel reported no mounted volumes",
                content.x,
                y + t.px(11.0),
                &s,
            );
            y += t.px(20.0);
        }
        for (mount, fs, ro) in &volumes {
            if y > content.bottom() {
                break;
            }
            let shown = fit(mount, &body_style, content.w * 0.62);
            font::draw(canvas, &shown, content.x, y + t.px(11.0), &body_style);
            let tag = if *ro {
                format!("{fs} · read-only")
            } else {
                fs.clone()
            };
            let w = font::measure(&tag, &caption);
            font::draw(canvas, &tag, content.right() - w, y + t.px(11.0), &caption);
            // 21 pixels of pitch for a 13-pixel row, not 17: the note that may
            // follow is type of its own, and at 16 the two rows overlapped by
            // three pixels -- the exact "letters touching letters" complaint.
            y += t.px(21.0);
            if let Some(note) = readonly_volume_note(fs, *ro) {
                let line = fit(note, &dim_bold, content.w);
                font::draw(canvas, &line, content.x, y + t.px(11.0), &dim_bold);
                y += t.px(20.0);
            }
        }
        y += t.px(4.0);
    }

    // ---- System --------------------------------------------------------
    if y < content.bottom() {
        section(canvas, t, &content, &mut y, "System");
        let rows = system_rows(f);
        rows_drawn(canvas, t, &content, y, &rows, &caption, &body_style);
    }
}

/// The height of a per-core bar.
fn bar_h(t: &crate::ui::Tokens) -> f32 {
    t.px(22.0)
}

/// A section heading with a rule under it.
fn section(canvas: &mut Canvas, t: &crate::ui::Tokens, content: &Rect, y: &mut f32, name: &str) {
    let s = Style::bold(t.text_px(TypeStep::Footnote), t.text_dim());
    font::draw(canvas, name, content.x, *y + t.px(10.0), &s);
    *y += t.px(14.0);
    ui::separator(canvas, t, Rect::new(content.x, *y, content.w, 1.0));
    *y += t.px(8.0);
}

/// Draw label/value rows and return the new cursor.
fn rows_drawn(
    canvas: &mut Canvas,
    t: &crate::ui::Tokens,
    content: &Rect,
    mut y: f32,
    rows: &[(String, String)],
    label_style: &Style,
    value_style: &Style,
) -> f32 {
    for (k, v) in rows {
        if y > content.bottom() {
            break;
        }
        font::draw(canvas, k, content.x, y + t.px(11.0), label_style);
        // The value keeps the space the label is not using: a right-aligned
        // value elided only against the window width still runs into a long
        // label, and two strings in one band is text over text.
        let room = (content.w - font::measure(k, label_style) - t.px(12.0)).max(t.px(24.0));
        let shown = fit(v, value_style, room);
        let w = font::measure(&shown, value_style);
        font::draw(
            canvas,
            &shown,
            content.right() - w,
            y + t.px(11.0),
            value_style,
        );
        y += t.px(21.0);
    }
    y
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
