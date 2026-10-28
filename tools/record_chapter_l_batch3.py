#!/usr/bin/env python3
"""Record Chapter L batch 3 (COSMIC Epoch 1, Pantheon 8.1)."""

import csv
import io
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"
PART1 = ROOT / "LitricOS UI UX Research Part1.md"
PART24 = ROOT / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"

LO = {
    "LO.01": ('COMPLETE', "Wayland-native Rust desktop; shell components and compositor share one toolkit; cosmic-comp on smithay with damage tracking and wl_fixes work; greeter, session and idle are separate projects (S3867, S3870); LitricOS: one shell plus own compositor, no greeter or session manager"),
    "LO.02": ('PARTIAL', "Launcher and app library are separate surfaces; the workspaces overview can open either on typing (S3867, S3870); 1.11 launcher changes could not be read (403/404); LitricOS: one overlay with seek ranking"),
    "LO.03": ('COMPLETE', "Single or dual panels on any edge, applets at start/centre/end, size, opacity, square or rounded corners; applets are Flatpak apps, not extensions (S3868); LitricOS: fixed bar plus morphing dock, no applet API"),
    "LO.04": ('PARTIAL', "Settings pages selectable per distribution by cargo feature flags, e.g. Pop!_OS Upgrades and Restore; pages touched by point releases: appearance, network, language, default and startup apps, shortcuts, workspaces, touchpad (S3868, S3870); LitricOS: 13 fixed pages, 9 keys"),
    "LO.05": ('COMPLETE', "Files: tabs, several directories in separate tabs, tab drag-and-drop, Move to / Copy to, clipboard paste of images, video and text, Recents toggle with clear history, copy path on Shift, ellipsized names (S3870); LitricOS: 26 verbs, no tabs"),
    "LO.06": ('PARTIAL', "Tiling is a first-class flexible element; point releases fix tiled state on maximize/snap, Super+Click on tiled windows and cursor-follows-focus across monitors (S3867, S3870); LitricOS: 36 ops, edge snapping, one output"),
    "LO.07": ('PARTIAL', "cosmic-notifications accepts file:// icons; the status area runs a socket-activated StatusNotifierWatcher so tray icons survive client crashes (S3870); LitricOS: notifyd queue, no tray or status area"),
    "LO.08": ('COMPLETE', "Workspaces dynamic or pinned, and the overview's action on typing is a setting (S3867, S3870); LitricOS: fixed grid, Mission Control cards, no typing action"),
    "LO.09": ('COMPLETE', "Theming built for brand palettes without losing contrast, RON theme files from the CLI, Qt theming via qt5ct/qt6ct, distribution-shipped panel and shortcut defaults (S3867, S3868, S3870); LitricOS: 4 themes x 6 accents x 3 wallpapers, no theme file format"),
    "LO.10": ('PARTIAL', "libcosmic is based on COSMIC's design language with consistent guidelines, a standardized configuration system and cross-desktop theming (S3869); no public HIG with token values opened; LitricOS: tokens plus lint, no principles document"),
    "LO.11": ('PARTIAL', "No accessibility page, screen-reader statement or on-screen keyboard in the eleven pages opened; the only accessibility-adjacent item was a 1.11 touchpad setting that could not be read (403); LitricOS: 1 toggle, no a11y tree"),
    "LO.12": ('PARTIAL', "The point-release log publicly lists fixed failure classes: tiling-placeholder panic, smithay image-copy leak, portal abort without dma-buf, terminal password handling, duplicate wallpaper entries (S3870); LitricOS: no public failure log"),
    "LO.13": ('COMPLETE', "xdg-desktop-portal-cosmic for screencast, clipboard, background and screenshot; StatusNotifierWatcher; AccountsService language handling; Qt theme generation; Flatpak applets; distro feature flags (S3868, S3870); LitricOS: private IPC only"),
    "LO.14": ('COMPLETE', "Runs on Pop!_OS 24.04 LTS and other distributions; distributions choose settings pages at build time and ship tagged or rolling QA releases; kernel, logind and audio stay outside (S3867, S3868, S3870); LitricOS: one mkosi image"),
}

LP = {
    "LP.01": ('PARTIAL', "Pantheon shell with Dock, Multitasking View, picture-in-picture and Do Not Disturb; 8.1 improves Secure Session and Dock (S3877); the compositor is not described on any page opened; LitricOS: one shell, own compositor, no secure session"),
    "LP.02": ('COMPLETE', "The HIG specifies launcher metadata as UI: Name without descriptive words, GenericName, a Comment beginning with a verb, Categories and Keywords, because the menu and Dock display them (S3875); LitricOS: launcher ranks names only"),
    "LP.03": ('PARTIAL', "A Dock, improved in 8.1 (S3877); no dock configuration page opened; LitricOS: one bar plus morphing dock, no dock settings"),
    "LP.04": ('PARTIAL', "System Settings is where app permissions can be revoked (S3877); no module list opened; LitricOS: 13 pages, 9 stored keys"),
    "LP.05": ('COMPLETE', "Files: smart pathbar with breadcrumbs, search and path completion, column view, browser-class tabs with tab history (S3877); LitricOS: breadcrumb and table, 26 verbs, no tabs"),
    "LP.06": ('PARTIAL', "Multitasking View and workspaces are advertised (S3877); no window-manager or tiling page opened; LitricOS: 36 WM ops, edge snapping"),
    "LP.07": ('PARTIAL', "Do Not Disturb stops notifications (S3877); the HIG Toast page exists with a heading and no body (S3876); LitricOS: queue with DND, banners, centre, no toast component"),
    "LP.08": ('PARTIAL', "Workspaces organise work by task and are reached from the Multitasking View (S3877); LitricOS: fixed grid, Mission Control cards"),
    "LP.09": ('PARTIAL', "No theme, accent or layout customization page opened; the HIG covers iconography and text as references (S3872); LitricOS: 4 themes, 6 accents, 3 wallpapers"),
    "LP.10": ('COMPLETE', "Full HIG: four design philosophies (What Design Is Not, Concision, Accessible Configuration, Minimal Documentation), user workflow, app launcher, widgets, iconography and text references; unwritten sections explicitly defer to the GNOME HIG (S3871, S3872, S3873); LitricOS: tokens plus lint, no principles"),
    "LP.11": ('PARTIAL', "The HIG names Accessible Configuration as a philosophy and 8.1 enhances accessibility (S3872, S3873, S3877); no specific feature, shortcut or contrast rule was described on a page opened; LitricOS: 1 toggle, no a11y tree"),
    "LP.12": ('NOT FOUND', "No known-issues page and no release announcement could be opened: the OS 8 announcement returned 404 and no other page lists defects (S3871-S3877); LitricOS: no release-notes surface"),
    "LP.13": ('PARTIAL', "AppCenter reviews permission use and System Settings can revoke permissions (S3877); no portal page opened; LitricOS: private IPC, no store or permission UI"),
    "LP.14": ('PARTIAL', "elementary ships its own OS with its own apps and AppCenter (S3877); the package base is not stated on any page opened; LitricOS: one mkosi image"),
}

CHAPTERS = {
    "LO": ('PARTIAL', "Part 27 batch 3: all 14 rows measured; Epoch 1.11 release pages returned 403/404, so no row rests on 1.11-specific text and LO.02, LO.06 and LO.11 say so"),
    "LP": ('PARTIAL', "Part 27 batch 3: all 14 rows measured from the elementary HIG and site; the OS 8 announcement 404'd, LP.12 is NOT FOUND and three rows are structural"),
}

NEW_SOURCES = [
    ["S3867", "https://system76.com/cosmic", "system76.com", "system76.com",
     "COSMIC DE", "System76", "", "2026-10-28", "COSMIC Epoch 1",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "COSMIC is a Wayland-native desktop written in Rust; shell components and the compositor use the same toolkit; panels, applets, theming, tiling, launcher, app library, keyboard shortcuts and dynamic or pinned workspaces are presented as flexible; Rust is claimed as a security property against keylogging and input spoofing; free to download and usable on other distributions",
     "high", "Product page: marketing claims, no control metrics, token values or failure modes", "LO.01; LO.06; LO.08; LO.09; LO.14"],
    ["S3868", "https://system76.com/cosmic/ux", "system76.com", "system76.com",
     "COSMIC UX", "System76", "", "2026-10-28", "COSMIC Epoch 1",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Panel layouts: single or dual panels on any edge, applets at beginning/centre/end, size and opacity, square or rounded corners. Settings pages are extensible per distribution through cargo feature flags, with Pop!_OS adding Upgrades and Restore. Applets are apps packaged as Flatpak for the COSMIC Store; no extension API is described",
     "high", "Describes the extensibility model, not individual settings pages or their controls", "LO.03; LO.04; LO.09; LO.13; LO.14"],
    ["S3869", "https://pop-os.github.io/libcosmic-book/introduction.html", "pop-os.github.io", "pop-os.github.io",
     "Introduction - COSMIC Toolkit", "System76", "", "2026-10-28", "COSMIC Epoch 1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "libcosmic is a GUI toolkit for COSMIC-themed applets and applications, based on iced for runtime and rendering; it provides a responsive widget library based on COSMIC's design language with personalizable desktop themes, cross-desktop theming, consistent interface guidelines, a standardized configuration system and platform integrations; cross-platform targets include Linux X11/Wayland, Redox, Windows, macOS and Android",
     "high", "Introduction only: no token names, measurements or component rules", "LO.10"],
    ["S3870", "https://system76.com/blog/post/cosmic-1-0-8-released", "system76.com", "system76.com",
     "COSMIC 1.0.8 released!", "System76", "2026-02-24", "2026-10-28", "COSMIC Epoch 1.0.3-1.0.8",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Point-release log for Epoch 1: Files gains tabs, opening several directories in tabs, tab drag-and-drop, Move to/Copy to, clipboard paste of images/video/text, a Recents toggle with clear history and copy-path on Shift; cosmic-comp fixes tiled-state preservation, Super+Click on tiled windows and cursor-follows-focus across monitors; cosmic-notifications accepts file:// icons; the status area uses a socket-activated StatusNotifierWatcher; xdg-desktop-portal-cosmic fixes dma-buf aborts and screencast flags; Settings adds default/startup app handling, XKB layouts and Qt theme generation; cosmic-session, greeter and idle changes are listed",
     "high", "Change log for point releases only; no design rules and no 1.11 content", "LO.01; LO.02; LO.04; LO.05; LO.06; LO.07; LO.08; LO.09; LO.12; LO.13; LO.14"],
    ["S3871", "https://docs.elementary.io/hig/", "docs.elementary.io", "docs.elementary.io",
     "Introduction | Human Interface Guidelines", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The HIG is a living document, explicitly 'a set of guidelines, not a rule book', aimed at designers and developers; it covers interface elements, how to use them and how to integrate with the desktop; for sections not yet written it defers to the GNOME HIG",
     "high", "Introduction only: states the scope and the deferral, not the rules themselves", "LP.10; LP.12"],
    ["S3872", "https://docs.elementary.io/hig/llms.txt", "docs.elementary.io", "docs.elementary.io",
     "Human Interface Guidelines index", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Complete page index: design philosophy (What Design Is Not, Concision, Accessible Configuration, Minimal Documentation), user workflow (First Launch Experience, Normal Launch, Always Provide an Undo, Always Saved, Closing, Background Tasks), App Launcher, widgets (creating layouts, providing feedback, selection controls, avoid widgets that do nothing, interactive widgets, Welcome Screen, Button, Search Entry, Notebook, Toast, containers, lists) and reference (Iconography, Text) plus a style guide",
     "high", "An index: names the pages and proves what does not exist, but carries no rules", "LP.09; LP.10; LP.11"],
    ["S3873", "https://docs.elementary.io/hig/design-philosophy.md", "docs.elementary.io", "docs.elementary.io",
     "Design Philosophy", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The philosophy section is deliberately flexible and extensible rather than a set of concrete rules; it names four guiding philosophies and links a design crash course. The page itself does not enumerate them beyond the child pages listed in the index",
     "high", "Short page: an introduction to four philosophies, not their content", "LP.10; LP.11"],
    ["S3874", "https://docs.elementary.io/hig/user-workflow/always-provide-an-undo.md", "docs.elementary.io", "docs.elementary.io",
     "Always Provide an Undo", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Rather than warning the user, apps should let them undo for an appropriate amount of time: closing an app saves automatically, deleting makes the item disappear with an easy undo, sending mail allows a short undo or edit, destructive photo edits are reversible with the original kept. The recommended implementation is a buffer between showing the result and performing the action, with the action still performed if the window closes, and Ctrl+Z bound to the undo where valid; a toast or info bar is the conventional affordance",
     "high", "Rule text only: no timings, no per-action duration guidance", "LP.07; LP.10"],
    ["S3875", "https://docs.elementary.io/hig/app-launcher.md", "docs.elementary.io", "docs.elementary.io",
     "App Launcher", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The launcher file is treated as interface: Name must not contain descriptive words, GenericName carries the category, Comment is a sentence beginning with a verb in sentence case without terminal punctuation, Categories sort the app in AppCenter and the menu, and Keywords let users find an app by words it does not contain (e.g. Files: Folders;Browser;Explorer;Finder;Manager;)",
     "high", "Specifies metadata, not the launcher's own layout or search ranking", "LP.02"],
    ["S3876", "https://docs.elementary.io/hig/widgets/ui-toolkit-elements/toast.md", "docs.elementary.io", "docs.elementary.io",
     "Toast", "elementary", "", "2026-10-28", "elementary OS 8.1",
     "Design language / HIG (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The page exists and contains a heading only: no description, no rules, no example. Another HIG page nevertheless recommends a toast as the conventional affordance for an undo, so the pattern is referenced and undocumented at the same time",
     "high", "Empty body; recorded as a documentation gap rather than as content", "LP.07; LP.10"],
    ["S3877", "https://elementary.io/", "elementary.io", "elementary.io",
     "Thoughtful, capable, and ethical computing", "elementary", "", "2026-10-28", "elementary OS 8.1.1",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "OS 8.1.1 Circe: 8.1 improves the Secure Session and Dock, enhances accessibility, improves device support and fixes over 1,100 issue reports. Feature claims: Multitasking View and workspaces, picture-in-picture, Do Not Disturb, a curated app set including Files (smart pathbar with breadcrumbs, search and path completion, column view, browser-class tabs with tab history) and Terminal; AppCenter as a curated, pay-what-you-can store; microphone and energy indicators; permissions must be requested up front and can be revoked in System Settings; automatic cleanup of temporary and trashed files",
     "high", "Marketing site: no control metrics, module list, package base or known-issues page", "LP.01; LP.03; LP.04; LP.05; LP.06; LP.07; LP.08; LP.10; LP.11; LP.13; LP.14"],
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    wanted = {**CHAPTERS, **LO, **LP}
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
    old = ("PARTIAL (Parts 25-26: GNOME 51, Plasma 6.8, Xfce 4.20 and Cinnamon 6.6, 56 rows; "
           "four environments still PENDING)")
    new = ("PARTIAL (Parts 25-27: GNOME 51, Plasma 6.8, Xfce 4.20, Cinnamon 6.6, COSMIC Epoch 1 "
           "and Pantheon 8.1, 84 rows; Budgie and ChromeOS still PENDING)")
    assert text.count(old) == 1, text.count(old)
    PART1.write_text(text.replace(old, new), encoding="utf-8")


def figures() -> dict:
    checklist = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    tally = Counter(row[3] for row in checklist[1:])
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    data = register[1:]
    status = Counter(row[12] for row in data)
    urls = Counter(row[1] for row in data)
    parents = [row[0] for row in checklist[1:] if row[3] == "PENDING" and "." not in row[0]]
    return {
        "checklist": len(checklist) - 1,
        "tally": dict(tally),
        "register": len(data),
        "status": dict(status),
        "domains": len({row[2] for row in data}),
        "last": data[-1][0],
        "dup_urls": sum(1 for n in urls.values() if n > 1),
        "surplus": sum(n - 1 for n in urls.values() if n > 1),
        "pending_parents": parents,
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.12 Addendum" in text:
        return
    f = figures()
    pending = f["tally"].get("PENDING", 0)
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.12 Addendum — after Chapter L batch 3 (2026-10-28)

Third recount, after COSMIC Epoch 1 and Pantheon 8.1. The prose above stays as
written; the numbers move here.

| Measure | J.11 | Now |
|---|---|---|
| Register rows | 4,181 | {f['register']:,} |
| Domains | 375 / 400 | {f['domains']} / 400 ({f['domains'] / 400:.1%}) |
| Inspected | 1,276 | {f['status'].get('INSPECTED', 0):,} |
| Last source id | S3866 | {f['last']} |
| Checklist rows | 597 | {f['checklist']} |
| PENDING rows | 64 | {pending} |
| PARTIAL rows | 489 | {f['tally'].get('PARTIAL', 0)} |
| COMPLETE rows | 40 | {f['tally'].get('COMPLETE', 0)} |
| NOT FOUND rows | 1 | {f['tally'].get('NOT FOUND', 0)} |
| Duplicate URLs / surplus rows | 47 / 52 | {f['dup_urls']} / {f['surplus']} |

What this recount shows:

1. **PENDING is now {pending}** — the 28 rows of the two environments not yet measured
   (Budgie, ChromeOS), {len(f['pending_parents'])} parent rows ({', '.join(f['pending_parents'])}),
   and the one remaining small row. Chapter L is 84 of 112.
2. **One NOT FOUND was recorded**, not a guess: `LP.12` has no reachable
   known-issues page, because the elementary OS 8 announcement returned 404 and
   no other opened page lists defects. A row that cannot be evidenced is
   recorded as such rather than downgraded to a vague PARTIAL.
3. **Duplicates still did not grow.** None of the eleven new URLs repeats an
   existing row, so `J.02`'s 47 / 52 stands for a third batch.
4. **Inspected count rises by eleven, not twelve**, because the twelfth page in
   this batch (`blog.elementary.io/...`) failed with a 404 and was not recorded
   as a source at all. Failed fetches are gaps, not rows.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    patch_part1()
    append_addendum()
    print(figures())
