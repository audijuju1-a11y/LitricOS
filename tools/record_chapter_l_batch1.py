#!/usr/bin/env python3
"""Record Chapter L batch 1 (GNOME 51, Plasma 6.8) in the three registers."""

import csv
import io
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"
PART1 = ROOT / "LitricOS UI UX Research Part1.md"

LG = {
    "LG.01": ('PARTIAL', "GNOME Shell + Mutter; 51 reworked frame scheduling, cheaper capture, legacy NVIDIA removed, SVG cursors (S3856); LitricOS: one shell, 3 compositor globals, cursor drawn in code"),
    "LG.02": ('PARTIAL', "Activities overview: search + app grid; 51 made screen-reader result descriptions and the app grid easier to navigate (S3856); LitricOS: one overlay, seek.rs ranking, no providers or grid"),
    "LG.03": ('PARTIAL', "One top bar; the dash lives in the overview and a persistent dock is an extension (S3856, S3857); LitricOS: one bar plus a dock that morphs into the launcher"),
    "LG.04": ('PARTIAL', "~20 panels; 51 changed Displays (auto-rotate, orientation lock, centre snapping), Mouse, Network (DNS search, WEP removed, SSH sockets), Users and About (S3856); LitricOS: 13 pages, 9 stored keys"),
    "LG.05": ('PARTIAL', "Nautilus 51: drag-count badge, copied files auto-selected, right-click on empty area keeps the selection, emblem precedence (S3856); LitricOS: 26 verbs, 0 cancel, none of the three"),
    "LG.06": ('PARTIAL', "No first-class tiling in the 51 notes; window behaviour is Mutter's (S3856, absence); LitricOS: 36 WM ops, one output, no tiling model"),
    "LG.07": ('PARTIAL', "Shell is the notification server; Files notifications grouped in 51 (S3856); LitricOS: private queue, banner open/close hotspots, 0 grouping, 0 actions"),
    "LG.08": ('PARTIAL', "Dynamic vertical workspaces, unchanged in 51 and therefore absent from its notes (S3856); LitricOS: fixed grid, Mission Control order stable by workspace then open order"),
    "LG.09": ('PARTIAL', "Extensions are the extension point; appearance deliberately narrow, with new wallpaper variants in 51 (S3856, S3857); LitricOS: 4 themes x 6 accents x 3 wallpapers, no registry (I.02)"),
    "LG.10": ('COMPLETE', "HIG is five parts: principles, resources, guidelines, patterns, reference (S3857); 51 extends reduced motion into widgets and the shell (S3856); LitricOS has the reference part as tokens and lacks the first two"),
    "LG.11": ('PARTIAL', "51: motion widened, screenshot region adjustable by keyboard, focus-ring timeout can be turned off, dock announced once (S3856); LitricOS: 1 toggle, no a11y tree, no focus timeout"),
    "LG.12": ('PARTIAL', "Legacy NVIDIA interfaces and WEP removed (S3856); extensions break across major releases; LitricOS: one output, one scale, no capability prompt"),
    "LG.13": ('PARTIAL', "Portals, D-Bus, GSettings/dconf, systemd user units, Flatpak (S3857); LitricOS: LFP/1 + browser socket + notifyd, no bus, no portal, no discovery"),
    "LG.14": ('PARTIAL', "DE owns shell, compositor, settings and session apps; kernel, logind, NetworkManager, BlueZ and PipeWire are outside, and the distribution packages all of it (corpus Parts 19-23); LitricOS: one mkosi image, no package manager"),
}

LK = {
    "LK.01": ('PARTIAL', "plasmashell + KWin on Wayland; the 6.8 announcement yielded no feature list (S3860); LitricOS: one process draws bar, dock, overlays and every app"),
    "LK.02": ('PARTIAL', "Kickoff and KRunner; HIG forbids asking preferences in a first-run wizard (S3859); LitricOS: one overlay, no wizard, but first-run asks three preferences (H.01)"),
    "LK.03": ('PARTIAL', "Configurable panels; no KDE page read in this batch (gap); LitricOS: fixed top bar, dock is an overlay state"),
    "LK.04": ('PARTIAL', "System Settings modules not enumerated by any page read in this batch (gap); LitricOS: 13 pages, 9 stored keys"),
    "LK.05": ('PARTIAL', "Dolphin; HIG requires inline rename, trash instead of delete, and undo for content that is not trivial to re-add (S3859); LitricOS: modal rename, 5 trash verbs, 0 undo"),
    "LK.06": ('PARTIAL', "KWin tiling and quick tile; no page read in this batch (gap); LitricOS: 36 WM ops, no tiling model"),
    "LK.07": ('PARTIAL', "Plasma notification server; HIG separates passive notification from persistent inline message (S3859); LitricOS: TTL banners, no inline message component"),
    "LK.08": ('PARTIAL', "Virtual desktops plus Activities; no page read in this batch (gap); LitricOS: fixed workspace grid, no activities"),
    "LK.09": ('PARTIAL', "Simple by default, powerful when needed (S3859); LitricOS: closed theme and accent set, no third-party surface"),
    "LK.10": ('COMPLETE', "Nine HIG sections; tokens Kirigami.Theme, iconsSizes, small/medium/largeSpacing, veryShort/short/long/veryLongDuration (S3858, S3859); LitricOS Space and Motion are the same axes under different names"),
    "LK.11": ('PARTIAL', "HIG has an Accessibility and inclusiveness section; the page was not read in this batch (gap); LitricOS: 1 toggle, no AT-SPI tree"),
    "LK.12": ('COMPLETE', "HIG states window size and position are remembered on X11 only, i.e. not on Wayland (S3859); LitricOS restores neither on any output"),
    "LK.13": ('PARTIAL', "D-Bus, KIO, KConfig, KWallet, KRecentDocument, portals (S3859); LitricOS: no bus, no key store, no KIO equivalent"),
    "LK.14": ('PARTIAL', "Same boundary as GNOME: DRM, logind, NetworkManager, BlueZ and PipeWire outside the DE, distribution packages the DE (S3858); LitricOS: the boundary is a build decision"),
}

CHAPTERS = {
    "LG": ('PARTIAL', "Part 25 batch 1: all 14 rows measured; page-level evidence from the GNOME 51 release notes and the HIG front page; the individual pattern pages returned 404 and are logged as gaps"),
    "LK": ('PARTIAL', "Part 25 batch 1: all 14 rows measured; page-level evidence from the KDE HIG front page and its Simple-by-default page; the layout and navigation page returned 404 and four rows are structural"),
}

NEW_SOURCES = [
    ["S3856", "https://release.gnome.org/51/", "release.gnome.org", "release.gnome.org",
     "Introducing GNOME 51", "GNOME", "2026-09-16", "2026-10-28", "GNOME 51",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "GNOME 51: Mutter frame-scheduling rework and cheaper screen capture; monitor brightness remembered across reboots; legacy NVIDIA interfaces removed; Settings adds auto-rotate, touchpad-disable, DNS search, WEP removed, SSH socket servers, fingerprint enrolment, reorganised About; Files adds a drag-count badge, auto-selection of copied files, selection kept on empty-area right-click and emblem precedence, and groups its notifications; accessibility widens reduced motion, makes screenshot regions keyboard-adjustable, allows disabling the focus-ring timeout and cleans up screen-reader output; SVG cursors and oo7 key storage",
     "high", "Release notes, not a shell reference: no geometry, timing or token values", "LG.01; LG.04; LG.05; LG.07; LG.10; LG.11"],
    ["S3857", "https://developer.gnome.org/hig/", "developer.gnome.org", "developer.gnome.org",
     "GNOME Human Interface Guidelines", "GNOME", "", "2026-10-28", "GNOME 51",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The HIG is five parts: design principles; resources; guidelines (text, icons, app identity, input); patterns in four types (containers, navigation, feedback, controls); reference (keyboard shortcuts, UI colours). Intended for use with GTK 4 and libadwaita. The individual pattern pages were not retrievable in this pass (404)",
     "high", "Front page only; the pattern catalogue's own rules are not in this entry", "LG.03; LG.10; LG.13"],
    ["S3858", "https://develop.kde.org/hig/", "develop.kde.org", "develop.kde.org",
     "KDE Human Interface Guidelines", "KDE", "", "2026-10-28", "KDE Plasma 6.8",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Nine sections: what makes a KDE app a KDE app; simple by default; powerful when needed; layout and navigation; displaying content; getting input; communicating status changes; text and labels; icons; accessibility and inclusiveness. Explicitly a living document and not 'an ironclad law code'",
     "high", "Front page only; layout/navigation page returned 404", "LK.01; LK.03; LK.10; LK.14"],
    ["S3859", "https://develop.kde.org/hig/simple_by_default/", "develop.kde.org", "develop.kde.org",
     "Simple by default", "KDE", "", "2026-10-28", "KDE Plasma 6.8",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Rules with concrete consequences: a first-run wizard only for mandatory unskippable setup or as an optional teaching tool, never to express preferences; rename inline rather than in a modal; show progress for tasks over a second, determinate preferred, BusyIndicator when indeterminate; hide destructive actions or confirm them, trash instead of delete, offer undo via a passive notification when survivable and a persistent inline message when not, with deferred deletion; remember window size and position on X11; use named tokens for colours, icon sizes, spacing and durations",
     "high", "Guidance for KDE applications, not a description of plasmashell's own UI", "LK.02; LK.05; LK.07; LK.09; LK.10; LK.12; LK.13"],
    ["S3860", "https://kde.org/announcements/plasma/6/6.8.0/", "kde.org", "kde.org",
     "Plasma 6.8", "KDE", "2026-10-20", "2026-10-28", "KDE Plasma 6.8",
     "Desktop environment (L)", "Release announcement (T2)", "primary", "INSPECTED",
     "Confirms Plasma 6.8 shipped, framed around KDE's 30th anniversary. The page body available to this reader contained the package-download section only, so no feature list was extracted and no Plasma row is based on it",
     "medium", "No feature content read; cited only for existence and framing", "LK.01"],
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    text = read_text(CHECKLIST)
    rows = list(csv.reader(io.StringIO(text, newline="")))
    header = rows[0]
    assert header[:4] == ["id", "parent", "title", "status"], header
    wanted = {**CHAPTERS, **LG, **LK}
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
    writer = csv.writer(buffer, lineterminator="\r\n")
    writer.writerows(rows)
    write_text(CHECKLIST, buffer.getvalue())


def patch_register() -> None:
    text = read_text(REGISTER)
    rows = list(csv.reader(io.StringIO(text, newline="")))
    header = rows[0]
    assert len(header) == 17, len(header)
    existing = {row[0] for row in rows[1:]}
    for source in NEW_SOURCES:
        assert source[0] not in existing, f"{source[0]} already recorded"
        assert len(source) == 17, (source[0], len(source))
        rows.append(source)
    buffer = io.StringIO(newline="")
    writer = csv.writer(buffer, lineterminator="\r\n")
    writer.writerows(rows)
    write_text(REGISTER, buffer.getvalue())


def patch_part1() -> None:
    text = PART1.read_text(encoding="utf-8")
    old = "| 30–37 | GNOME, KDE Plasma, Xfce, Cinnamon, COSMIC, Pantheon, Budgie, ChromeOS (LG–LR) | 1, 2 | PENDING |"
    new = ("| 30–37 | GNOME, KDE Plasma, Xfce, Cinnamon, COSMIC, Pantheon, Budgie, ChromeOS (LG–LR) | 1, 2 | "
           "PARTIAL (Part 25: GNOME 51 and Plasma 6.8, all 28 rows; six environments still PENDING) |")
    assert text.count(old) == 1, text.count(old)
    PART1.write_text(text.replace(old, new), encoding="utf-8")


def report() -> None:
    for path in (CHECKLIST, REGISTER):
        rows = list(csv.reader(io.StringIO(read_text(path), newline="")))
        width = len(rows[0])
        malformed = [i for i, row in enumerate(rows, start=1) if len(row) != width]
        print(f"{path.name}: {len(rows)} rows / {width} columns / malformed={len(malformed)}")
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    tally = {}
    for row in rows[1:]:
        tally[row[3]] = tally.get(row[3], 0) + 1
    print("checklist statuses:", dict(sorted(tally.items())))
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    domains = {row[2] for row in register[1:]}
    print(f"register last id: {register[-1][0]} / domains: {len(domains)}")


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    patch_part1()
    report()
