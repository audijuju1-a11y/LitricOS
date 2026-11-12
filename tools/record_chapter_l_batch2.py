#!/usr/bin/env python3
"""Record Chapter L batch 2 (Xfce 4.20, Cinnamon 6.6)."""

import csv
import io
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"
PART1 = ROOT / "LitricOS UI UX Research Part1.md"
PART24 = ROOT / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"

LX = {
    "LX.01": ('PARTIAL', "Components ported to Wayland through libxfce4windowing; xfwm4, xfdashboard and the screensaver are not, and Labwc or Wayfire is required (S3861); LitricOS: own compositor with 3 globals, no session manager"),
    "LX.02": ('PARTIAL', "xfce4-appfinder: Ctrl+N/P navigation, Ctrl+Enter executes, per-app actions on right-click, optional daemon (S3861, S3862); LitricOS: launcher overlay with seek ranking, no per-app actions"),
    "LX.03": ('COMPLETE', "xfce4-panel: border width, XDG plugin discovery, multi-delete, LCD/analog clock options, show desktop on hover, tasklist icon sizing (S3861, S3862); LitricOS: fixed top bar plus morphing dock, no plugin model"),
    "LX.04": ('PARTIAL', "Appearance, Display (profiles, both-dimension scaling, mirror without matching refresh), Mouse (acceleration profile, hi-res scroll), MIME multi-select, Keyboard (S3861, S3862); LitricOS: 13 pages, 9 keys, Display read-only except UI scale"),
    "LX.05": ('COMPLETE', "Thunar 4.20: toolbar overflow, view switcher, tabs, CSD, split view, expandable folders, bulk renamer, undo asks before delete, direct-I/O compare, busy-disk queueing (S3861, S3862, S3864); LitricOS: 26 verbs, no cancel, no tabs or split"),
    "LX.06": ('PARTIAL', "xfwm4: click-to-focus or focus-follows-mouse, raise delay, snapping with distance, workspace wrap with resistance, opaque move, double-click action (S3863); LitricOS: 36 ops, edge snapping, no focus-follows-mouse"),
    "LX.07": ('PARTIAL', "libnotify is an optional Thunar dependency for mount notifications (S3864); no xfce4-notifyd page read; LitricOS: notifyd queue, banners, centre, DND"),
    "LX.08": ('PARTIAL', "Workspace wrap at screen edges (S3863); workspace support missing on Wayland because the protocol arrived late (S3861); LitricOS: fixed grid, Mission Control order stable"),
    "LX.09": ('PARTIAL', "Window-decoration themes, panel plugins, shortcut editor with overlap groups, Xfconf, systemd user unit (S3861, S3862); LitricOS: 4 themes x 6 accents x 3 wallpapers, no plugin model"),
    "LX.10": ('PARTIAL', "No Xfce HIG page located; rules are GTK/Adwaita defaults plus per-component options, and icon scaling above 1 was fixed (S3861, S3862); LitricOS: tokens plus lint, no principles document"),
    "LX.11": ('PARTIAL', "No accessibility content in the tour, changelog or xfwm4 page (gap); LitricOS: 1 toggle honoured everywhere, no a11y tree"),
    "LX.12": ('COMPLETE', "Wayland is experimental: no compositor, missing workspaces, tray icons, keyboard/mouse settings, brightness keys and rectangular screenshots, with four components unported (S3861); LitricOS: one output, one scale"),
    "LX.13": ('COMPLETE', "libxfce4windowing, minimal xdg-desktop-portal conf, D-Bus, xfconf systemd user unit, optional polkit, power-profiles-daemon (S3861, S3862); LitricOS: private IPC, no portal, no bus"),
    "LX.14": ('PARTIAL', "Xfce owns panel, session, settings and Thunar; the Wayland compositor is external; kernel, logind, NetworkManager and PipeWire are outside (S3861, S3862); LitricOS: one image"),
}

LC = {
    "LC.01": ('PARTIAL', "Cinnamon on Muffin; 6.6 improved Muffin's Wayland compatibility (S3865); LitricOS: one shell, own compositor, 3 globals"),
    "LC.02": ('COMPLETE', "Menu redesigned: sidebar with avatar, places and favourites, smaller categories, configurable search and system-button position, optional no-sidebar layout (S3865); LitricOS: fixed launcher frame"),
    "LC.03": ('PARTIAL', "Panel applets: workspace switcher, XKB+IBus keyboard applet, Night Light (S3865); LitricOS: fixed bar and dock, no applet model"),
    "LC.04": ('PARTIAL', "No Cinnamon Settings page read; 6.6 touched keyboard layouts, fractional-scale direction, Alt-Tab scope and theme grouping (S3865); LitricOS: 13 pages, 9 keys"),
    "LC.05": ('COMPLETE', "Nemo 6.6: template manager, pause/resume of file operations, wildcard MIME and regex search, split-pane location kept, thumbnails (S3865); LitricOS: 0 cancel or pause verbs"),
    "LC.06": ('PARTIAL', "Muffin: Alt-Tab limited to the current monitor, hot corners in fullscreen, fractional scaling up or down (S3865); LitricOS: 36 ops, one output"),
    "LC.07": ('PARTIAL', "A missed notification shows a badge in the window list (S3865); LitricOS: count badge on the bell only, no per-window badge"),
    "LC.08": ('PARTIAL', "Workspace switcher shows only visible windows with their icons (S3865); LitricOS: fixed grid and Mission Control cards, no switcher applet"),
    "LC.09": ('PARTIAL', "Menu element visibility, themes grouped by family and variant, applets, fractional-scale direction (S3865); LitricOS: closed theme and accent set"),
    "LC.10": ('PARTIAL', "XSI replaces Adwaita symbolic icons after GNOME restricted them; all Cinnamon and Mint projects migrated (S3865); no Cinnamon HIG opened; LitricOS: 32 icons from one module"),
    "LC.11": ('PARTIAL', "Native on-screen keyboard with input-method and layout switching (S3865); no accessibility page read; LitricOS: 1 toggle, no OSK, no a11y tree"),
    "LC.12": ('COMPLETE', "Known issues: HWE 6.14 breaks VirtualBox and the 470 NVIDIA driver, 10 s shutdown timeout, PipeWire with a PulseAudio fallback, ZFS removed, ecryptfs unmount regression, Snap disabled (S3866); LitricOS: no release-notes surface"),
    "LC.13": ('PARTIAL', "XApp and XSI, apt:// handling, Update Manager tray, Thunderbolt tool, Night Light (S3865); no portal page read; LitricOS: private IPC only"),
    "LC.14": ('COMPLETE', "Ubuntu 24.04 base with kernel 6.14; PipeWire, systemd, logind and the package manager are distribution-owned (S3865, S3866); LitricOS: the boundary is a build decision"),
}

CHAPTERS = {
    "LC": ('PARTIAL', "Part 26 batch 2: all 14 rows measured from the Mint 22.3 notes; no Cinnamon HIG or settings catalogue read, projects.linuxmint.com did not resolve"),
    "LX": ('PARTIAL', "Part 26 batch 2: all 14 rows measured from the 4.20 tour, changelog and component docs; no Xfce HIG page was located"),
}

NEW_SOURCES = [
    ["S3861", "https://www.xfce.org/about/tour420/", "xfce.org", "www.xfce.org",
     "Xfce 4.20 tour", "Xfce", "", "2026-10-28", "Xfce 4.20",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Xfce 4.20: almost all components ported to Wayland via libxfce4windowing, but xfwm4, xfdashboard, the screensaver and two panel plugins are not, and Wayland needs an external compositor (Labwc or Wayfire); workspace, tray, keyboard/mouse settings, brightness keys and rectangular screenshots are missing on Wayland; Thunar gains toolbar overflow, view switcher, new tab/window buttons, CSD, split-view focus and close confirmation, expandable folders, drag-hover opening, bulk-renamer size memory, undo that asks before deleting, direct-I/O copy comparison instead of MD5, busy-disk queueing, IPv6 remote URLs; appfinder keyboard and action changes; panel and power-manager options; xfce4-session registers a minimal portal conf",
     "high", "Tour describes user-visible change only: no geometry, timing or token values", "LX.01; LX.03; LX.05; LX.07; LX.08; LX.09; LX.12; LX.13; LX.14"],
    ["S3862", "https://www.xfce.org/download/changelogs/4.20/", "xfce.org", "www.xfce.org",
     "Xfce 4.20 Changelog", "Xfce", "", "2026-10-28", "Xfce 4.20",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Component changelog since 4.18: Thunar overflow menu, tabs, CSD, split-view shortcut, direct comparison instead of checksum, queueing on busy disks; panel plugin discovery in XDG base directories and multi-delete in preferences; display profiles, both-dimension scaling and mirror without matching refresh; mouse acceleration profile and hi-res scrolling; MIME multi-select; xfconf systemd user unit; minimal xdg-desktop-portal conf",
     "high", "Summary extraction of per-component NEWS files; not a specification", "LX.02; LX.03; LX.04; LX.05; LX.09; LX.13"],
    ["S3863", "https://docs.xfce.org/xfce/xfwm4/preferences", "docs.xfce.org", "docs.xfce.org",
     "xfwm4 - Preferences", "Xfce", "2025-01-05", "2026-10-28", "Xfce 4.20",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "xfwm4 options: theme and title font/alignment, drag-and-drop button layout with a hidden area; per-action keyboard shortcuts with reset; focus model click-to-focus or focus-follows-mouse, new-window focus, raise-on-focus delay, raise-on-click; window snapping with distance, workspace wrap on pointer and drag with edge resistance, opaque move/resize, double-click action",
     "high", "Describes defaults and dialogs, not tiling (there is none) or Wayland behaviour", "LX.06; LX.08"],
    ["S3864", "https://docs.xfce.org/xfce/thunar/start", "docs.xfce.org", "docs.xfce.org",
     "Thunar File Manager", "Xfce", "2026-10-01", "2026-10-28", "Xfce 4.20 / Thunar 4.21.7",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Thunar scope, plugin model (thunarx API, bulk renamer, custom actions, archive, media tags, shares, volman, VCS), standards compliance (XDG base dirs, shared MIME, XDS, icon theme, thumbnail standard, desktop trash, MIME associations), required and optional packages including libnotify for mount notifications, and the current release line (4.21.7, 2026-10-10)",
     "high", "Landing page: links to the behavioural pages rather than restating them; release line is a development series above the 4.20 stable release", "LX.05; LX.07"],
    ["S3865", "https://linuxmint.com/rel_zena_whatsnew.php", "linuxmint.com", "linuxmint.com",
     "New Features in Linux Mint 22.3 'Zena'", "Linux Mint", "", "2026-10-28", "Linux Mint 22.3 / Cinnamon 6.6",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Cinnamon 6.6: XKB layouts and IBus input methods in one list, in settings and in the keyboard applet, Wayland-compatible; native on-screen keyboard with IM and layout switching; application menu redesigned with avatar/places/favourites sidebar, configurable search and system-button position and a no-sidebar layout; Nemo 6.6 template manager, pause/resume of file operations, wildcard MIME and regex search, split-pane location preservation, thumbnails; notification badges in the window list; workspace switcher redraw; Thunderbolt tool; Night Light applet; Muffin Wayland work, fullscreen hot corners, fractional scale direction, per-monitor Alt-Tab; theme selector grouped by family; XSI symbolic icon project replaces Adwaita icons",
     "high", "Marketing-shaped feature notes: no control metrics, token values or failure modes for the new features", "LC.01; LC.02; LC.03; LC.05; LC.06; LC.07; LC.08; LC.09; LC.10; LC.11; LC.13; LC.14"],
    ["S3866", "https://linuxmint.com/rel_zena.php", "linuxmint.com", "linuxmint.com",
     "Linux Mint 22.3 Release Notes", "Linux Mint", "", "2026-10-28", "Linux Mint 22.3",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Known issues and base system: HWE kernel 6.14 breaks VirtualBox and the 470 NVIDIA driver; shutdown timeout reduced to 10 s with an override path; PipeWire replaces PulseAudio with documented fallback commands; ZFS removed from the installer; ecryptfs is no longer unmounted on logout; guest sessions disabled; Snap Store disabled; libinput default with a synaptics alternative; Ubuntu 24.04 base",
     "high", "Lists problems and workarounds, not the desktop's design rules", "LC.12; LC.14"],
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    wanted = {**CHAPTERS, **LX, **LC}
    seen = set()
    for row in rows[1:]:
        if row and row[0] in wanted:
            status, note = wanted[row[0]]
            row[3] = status
            while len(row) < 5:
                row.append("")
            row[4] = note
            seen.add(row[0])
    missing = set(wanted) - seen
    assert not missing, f"checklist rows not found: {sorted(missing)}"
    buffer = io.StringIO(newline="")
    csv.writer(buffer, lineterminator="\r\n").writerows(rows)
    write_text(CHECKLIST, buffer.getvalue())


def patch_register() -> None:
    rows = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    existing = {row[0] for row in rows[1:]}
    for source in NEW_SOURCES:
        assert source[0] not in existing, f"{source[0]} already recorded"
        assert len(source) == 17, (source[0], len(source))
        rows.append(source)
    buffer = io.StringIO(newline="")
    csv.writer(buffer, lineterminator="\r\n").writerows(rows)
    write_text(REGISTER, buffer.getvalue())


def patch_part1() -> None:
    text = PART1.read_text(encoding="utf-8")
    old = ("PARTIAL (Part 25: GNOME 51 and Plasma 6.8, all 28 rows; six environments still PENDING)")
    new = ("PARTIAL (Parts 25-26: GNOME 51, Plasma 6.8, Xfce 4.20 and Cinnamon 6.6, 56 rows; "
           "four environments still PENDING)")
    assert text.count(old) == 1, text.count(old)
    PART1.write_text(text.replace(old, new), encoding="utf-8")


def figures() -> dict:
    checklist = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    tally = Counter(row[3] for row in checklist[1:])
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    data = register[1:]
    status = Counter(row[12] for row in data)
    urls = Counter(row[1] for row in data)
    return {
        "checklist": len(checklist) - 1,
        "tally": dict(tally),
        "register": len(data),
        "status": dict(status),
        "domains": len({row[2] for row in data}),
        "last": data[-1][0],
        "dup_urls": sum(1 for n in urls.values() if n > 1),
        "surplus": sum(n - 1 for n in urls.values() if n > 1),
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.11 Addendum" in text:
        return
    f = figures()
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.11 Addendum — after Chapter L batch 2 (2026-10-28)

Second recount, taken after Xfce 4.20 and Cinnamon 6.6 were recorded. Same rule
as `J.10`: the audit's prose stays as written, and the numbers move here.

| Measure | J.10 | Now |
|---|---|---|
| Register rows | 4,175 | {f['register']:,} |
| Domains | 374 / 400 | {f['domains']} / 400 ({f['domains'] / 400:.1%}) |
| Inspected | 1,270 | {f['status'].get('INSPECTED', 0):,} |
| Last source id | S3860 | {f['last']} |
| Checklist rows | 597 | {f['checklist']} |
| PENDING rows | 94 | {f['tally'].get('PENDING', 0)} |
| PARTIAL rows | 467 | {f['tally'].get('PARTIAL', 0)} |
| COMPLETE rows | 32 | {f['tally'].get('COMPLETE', 0)} |
| Duplicate URLs / surplus rows | 47 / 52 | {f['dup_urls']} / {f['surplus']} |

Three things this recount shows that the first one could not:

1. **The PENDING set is now almost entirely Chapter L.** {f['tally'].get('PENDING', 0)} rows remain, and 56 of
   the 112 Chapter L rows are done — so the 56 still pending are four
   environments plus the ten parent/small rows that were already open.
2. **Domains barely move.** Six new sources added two new hosts
   (`xfce.org`/`docs.xfce.org` and `linuxmint.com`) and only two domains, because
   most of the new evidence is project documentation on hosts the corpus already
   had. The 400-domain target is a breadth measure, and Chapter L is not what
   moves it.
3. **Duplicates did not grow.** None of the six new URLs repeats an existing row,
   so `J.02`'s finding is unchanged for a second batch.

The structural finding of Part 24 holds a second time: this corpus is broad and
shallow, and the two environments measured here publish almost no interface
rules, which is itself a result recorded in `LX.10` and `LC.10`.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    patch_part1()
    append_addendum()
    print(figures())
