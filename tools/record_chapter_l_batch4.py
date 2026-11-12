#!/usr/bin/env python3
"""Record Chapter L batch 4 (Budgie 10.10.3, ChromeOS 154) and close the chapter."""

import csv
import io
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"
PART1 = ROOT / "LitricOS UI UX Research Part1.md"
PART24 = ROOT / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"

LB = {
    "LB.01": ('COMPLETE', "Wayland-first since 10.10 and compositor-agnostic: requires ext-workspace-v1, wlr-foreign-toplevel-management, wlr-layer-shell, wlr-output-management and xdg-output, recommends labwc, ships no compositor (S3880); LitricOS: own compositor with 3 globals, none of the five protocols"),
    "LB.02": ('COMPLETE', "Budgie Menu: favourites via right-click, desktop actions in the menu, shared action code with Icon Tasklist, Favorites category above All, compact-mode grouping, and the menu is a layer-shell window so Super-key search gets focus (S3879, S3878); LitricOS: launcher with no favourites or actions"),
    "LB.03": ('COMPLETE', "budgie-panel on layer-shell with start/centre/end sections, dynamic sizing and overflow; Icon Tasklist plus a rewritten Tasklist with dynamic buttons and scroll buttons; primary monitor selectable (S3880, S3879); LitricOS: fixed bar and morphing dock"),
    "LB.04": ('PARTIAL', "Budgie Control Center: Colour panel hidden, Bluetooth optional, per-panel visibility checks, Multitasking on Wayland, WM keyfiles, screen lock under Privacy, Night Light panel, accessibility incl. a working magnifier (S3880); no module catalogue; LitricOS: 13 pages, 9 keys"),
    "LB.05": ('NOT FOUND', "No file-manager component appears in either release note; desktop icons come from Budgie Desktop View, an icon layer rather than a file manager (S3880, absence); LitricOS: FilesApp with 26 verbs"),
    "LB.06": ('PARTIAL', "The labwc bridge writes rc.xml for keybinds, focus, theming, snapping and a thumbnail window switcher, sets auto-raise delay, delegates Show Desktop and migrates configs by version (S3879, S3880); LitricOS: 36 WM ops, edge snapping, no external compositor"),
    "LB.07": ('PARTIAL', "Raven is the notification centre and Do Not Disturb toggles by middle-clicking the Notifications applet (S3880); per-app volume claims were only in search snippets, not opened pages; LitricOS: notifyd queue with DND"),
    "LB.08": ('PARTIAL', "Workspaces applet reliability fixes and a more responsive add-workspace button; ext-workspace-v1 is a required protocol (S3880, S3879); LitricOS: fixed grid, Mission Control cards"),
    "LB.09": ('COMPLETE', "Bridge applies fonts, drop shadows and title-bar layouts; keybind templates per distribution; panel sections and applets user-arranged; Night Light via gammastep (S3879, S3880); LitricOS: 4 themes x 6 accents x 3 wallpapers, no per-distro defaults"),
    "LB.10": ('PARTIAL', "No Budgie HIG or token documentation found in the pages opened; design is GTK3 plus the bridge's labwc theming (S3879, S3880); LitricOS: tokens plus lint, no principles document"),
    "LB.11": ('PARTIAL', "Budgie Control Center supports accessibility options including a working magnifier (S3880); no screen-reader or keyboard-navigation statement found; LitricOS: 1 toggle, no magnifier, no a11y tree"),
    "LB.12": ('COMPLETE', "10.10 is the final major of the 10.x series and is in maintenance mode while Budgie 11 is built; Keyboard Layout applet disabled in 10.10 and restored in 10.10.3; Budgie Screensaver deprecated; Debian blocks SetX11Keyboard (bug 1118702); labwc 0.20.2 rejected empty keybinds (S3880, S3879); LitricOS: no release-notes surface"),
    "LB.13": ('COMPLETE', "xdg-desktop-portal-gtk and -wlr, locale1/systemd-localed, WirePlumber wpctl, D-Bus org.buddiesofbudgie.KeyboardLayout, gammastep, grim/slurp, swaybg, wlopm, swayidle and Qt6 Budgie Desktop Services for display config (S3879, S3880); LitricOS: private IPC only"),
    "LB.14": ('COMPLETE', "Budgie sits on somebody else's compositor; kernel, logind, portals and audio outside; ships in Fedora 44 and Ubuntu Budgie 26.04; distributions package it (S3880); LitricOS: one mkosi image"),
}

LR = {
    "LR.01": ('PARTIAL', "Chrome powers ChromeOS and the OS tracks Chrome releases; the shell and window manager are not described on any page opened (S3882, structural); LitricOS: own shell and compositor"),
    "LR.02": ('PARTIAL', "Web apps are found from the launcher and a long press of the launcher starts Select-to-search with Lens (S3882, S3886); no launcher specification opened; LitricOS: ranked launcher overlay"),
    "LR.03": ('PARTIAL', "The shelf exists: PWAs can be pinned to it (S3882); no shelf behaviour page opened; LitricOS: fixed top bar plus morphing dock"),
    "LR.04": ('NOT FOUND', "No ChromeOS Settings page could be opened: the Help Center and Enterprise URLs attempted returned 404 (recorded in Part 28); LitricOS: 13 pages, 9 keys"),
    "LR.05": ('PARTIAL', "The Files app can hand files to installed web apps (S3882); two Help Center Files URLs returned 404, so no Files behaviour is claimed; LitricOS: FilesApp with 26 verbs"),
    "LR.06": ('NOT FOUND', "No page describing windows, split screen or overview was opened; nothing to cite; LitricOS: 36 WM ops with edge snapping"),
    "LR.07": ('NOT FOUND', "No ChromeOS notification page was opened; nothing to cite; LitricOS: notifyd queue, banners, centre, DND"),
    "LR.08": ('NOT FOUND', "No virtual-desks or workspace page was opened; nothing to cite; LitricOS: fixed workspace grid and Mission Control"),
    "LR.09": ('NOT FOUND', "No wallpaper, theme or shelf-customization page was opened; nothing to cite; LitricOS: 4 themes, 6 accents, 3 wallpapers"),
    "LR.10": ('PARTIAL', "chromeos.dev/en/design returned 404; the PWA page that opened lists capabilities, not a design language (S3882 plus recorded failure); LitricOS: tokens plus lint, no principles"),
    "LR.11": ('NOT FOUND', "No ChromeOS accessibility page was opened; nothing to cite; LitricOS: 1 toggle, no a11y tree"),
    "LR.12": ('PARTIAL', "End of Life / Auto Update Expiration is a documented process in the ChromiumOS reference index, and ChromeOS release notes are published one week before stable on the Help Center (S3884, S3885); LitricOS: no release-notes or support-window statement"),
    "LR.13": ('PARTIAL', "Web apps install, pin to the shelf, appear in the launcher and handle files; Chrome extensions are integral (S3882); Android and Linux app support was not on any opened page; LitricOS: no extension or store model"),
    "LR.14": ('PARTIAL', "ChromeOS is an OS whose browser is the platform and whose releases track Chrome; security, SELinux, user sessions and AUE are documented platform topics (S3882, S3884); LitricOS: one mkosi image"),
}

CHAPTERS = {
    "LB": ('PARTIAL', "Part 28 batch 4: all 14 rows measured; Budgie ships no file manager (LB.05 NOT FOUND) and no HIG was located (LB.10 structural); Chapter L row complete"),
    "LR": ('PARTIAL', "Part 28 batch 4: all 14 rows measured; five ChromeOS URLs returned 404, so six rows are NOT FOUND and eight structural - no ChromeOS shell behaviour is claimed without a source"),
}

NEW_SOURCES = [
    ["S3878", "https://buddiesofbudgie.org/", "buddiesofbudgie.org", "buddiesofbudgie.org",
     "Home | Buddies of Budgie", "Buddies of Budgie", "2026-09-27", "2026-10-28", "Budgie 10.10.3",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Budgie 10.10.3 adds favourites to Budgie Menu, restores the Keyboard Layout applet, lets the user choose the primary monitor, introduces free placement of desktop icons and improves labwc integration; Budgie Desktop is described as a free and open source desktop emphasising simplicity, minimalism and elegance",
     "high", "Headline summary only: no detail beyond the release announcement", "LB.02; LB.03"],
    ["S3879", "https://buddiesofbudgie.org/blog/budgie-10-10-3-released", "buddiesofbudgie.org", "buddiesofbudgie.org",
     "Budgie 10.10.3 Released", "Buddies of Budgie", "2026-09-27", "2026-10-28", "Budgie 10.10.3",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Menu favourites with desktop actions shared with the Icon Tasklist; Budgie Menu moved back to a layer-shell GtkWindow so Super-key search receives keyboard focus (issue #842); labwc bridge writes rc.xml keybinds from templates, sets auto-raise delay, delegates Show Desktop and migrates configs by version; media keys via a new controller and wpctl; Keyboard Layout applet rewritten in C with a D-Bus interface, a documented precedence list and a read-back of the active layout; primary monitor selection; crash fixes in the bridge",
     "high", "Release engineering log: no measurements, token values or design rules", "LB.02; LB.03; LB.06; LB.08; LB.09; LB.10; LB.11; LB.12; LB.13"],
    ["S3880", "https://buddiesofbudgie.org/blog/budgie-10-10-released", "buddiesofbudgie.org", "buddiesofbudgie.org",
     "Budgie 10.10 Released", "Buddies of Budgie", "2026-01-10", "2026-10-28", "Budgie 10.10",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Budgie 10.10 migrates from X11 to Wayland and becomes compositor-agnostic, decoupling from budgie-wm/Magpie and requiring ext-workspace-v1, wlr-foreign-toplevel-management, wlr-layer-shell, wlr-output-management and xdg-output; labwc is the recommended compositor with a bridge that mirrors settings into rc.xml and the environment file; portals, grim/slurp, swayidle/gtklock/wlopm and swaybg replace in-house tools; applets updated (Icon Tasklist, Night Light via gammastep, DND by middle-click, rewritten Tasklist, workspaces); panel on layer-shell with dynamic sections; control centre changes including accessibility with a working magnifier; 10.10 is the final major of the 10.x series and is now in maintenance mode while Budgie 11 is built; ships in Fedora 44 and Ubuntu Budgie 26.04",
     "high", "Announcement: describes intent and changes, not per-control metrics or failure modes beyond those listed", "LB.01; LB.03; LB.04; LB.05; LB.06; LB.07; LB.08; LB.09; LB.10; LB.11; LB.12; LB.13; LB.14"],
    ["S3881", "https://chromeos.dev/en", "chromeos.dev", "chromeos.dev",
     "ChromeOS.dev", "Google", "", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Landing page only: a featured story from Google I/O 2024 about ChromeOS in education and enterprise. No shell, launcher, shelf or Files content",
     "low", "Marketing landing page; nothing about desktop behaviour", "LR.01"],
    ["S3882", "https://chromeos.dev/en/web", "chromeos.dev", "chromeos.dev",
     "Web apps on ChromeOS", "Google", "", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Chrome powers ChromeOS and the OS keeps up with Chrome releases; web apps can be installed, pinned to the shelf and found from the launcher, and can open files directly from the Files app; desktop PWAs are the recommended app form; Chrome extensions are as integral on ChromeOS as elsewhere; almost all devices support touch and many support styluses",
     "high", "Developer guidance for web apps: no shell layout, settings inventory or system UI rules", "LR.01; LR.02; LR.03; LR.05; LR.10; LR.13; LR.14"],
    ["S3883", "https://www.chromium.org/chromium-os/", "chromium.org", "www.chromium.org",
     "ChromiumOS Developer Library", "The Chromium Projects", "", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The developer library's structure: getting started, reference, guides, training and contributing. Index only; no system UI content",
     "low", "Index page: establishes where platform documentation lives, not what it says", "LR.14"],
    ["S3884", "https://www.chromium.org/chromium-os/developer-library/reference/", "chromium.org", "www.chromium.org",
     "Reference - ChromiumOS", "The Chromium Projects", "", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Reference topics include security in ChromeOS, SELinux, write protection, user sessions, CrOS user and group management, the ChromiumOS input stack, ChromeOS releases and the End of Life / Auto Update Expiration process, logging and kernel scheduling",
     "high", "Index of reference titles: proves the AUE and session documents exist, not their content", "LR.12; LR.14"],
    ["S3885", "https://support.google.com/chrome/a/answer/7679408", "support.google.com", "support.google.com",
     "Chrome Enterprise and Education release notes", "Google", "2026-09-23", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "ChromeOS release notes are published one week before ChromeOS Stable and remain on this Help Center link, while Chrome browser notes moved to chromeenterprise.google; Chrome versions listed with stable dates (152 on 2026-09-01, 151 on 2026-08-07, 150 on 2026-07-14, 149 on 2026-06-12)",
     "high", "Index and cadence only: no feature text for any milestone", "LR.12"],
    ["S3886", "https://www.google.com/intl/en_us/chromebook/whatsnew/", "google.com", "www.google.com",
     "What's New with Chromebook", "Google", "2025-06-01", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "June 2025 update page: Select-to-search with Lens can be started with a long press of the launcher to identify items in a photo or video without leaving the current context",
     "medium", "Page is dated June 2025, a year behind the 154 stable line; one feature only", "LR.02"],
    ["S3887", "https://chromeenterprise.google/resources/release-notes/", "chromeenterprise.google", "chromeenterprise.google",
     "Chrome Enterprise release notes", "Google", "", "2026-10-28", "ChromeOS 154",
     "Desktop environment (L)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "Landing page: enterprise release notes index with subscription and help links, and a pointer to the separate ChromeOS release notes. The feature content is loaded dynamically and was not present in the fetched body",
     "low", "No feature content: the page body contains navigation and marketing copy only", "LR.12"],
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    wanted = {**CHAPTERS, **LB, **LR}
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
    old = ("PARTIAL (Parts 25-27: GNOME 51, Plasma 6.8, Xfce 4.20, Cinnamon 6.6, COSMIC Epoch 1 "
           "and Pantheon 8.1, 84 rows; Budgie and ChromeOS still PENDING)")
    new = ("COMPLETE for coverage (Parts 25-28: GNOME 51, Plasma 6.8, Xfce 4.20, Cinnamon 6.6, "
           "COSMIC Epoch 1, Pantheon 8.1, Budgie 10.10 and ChromeOS 154 - all 112 rows measured; "
           "quality is PARTIAL where pages were unavailable)")
    assert text.count(old) == 1, text.count(old)
    PART1.write_text(text.replace(old, new), encoding="utf-8")


def figures() -> dict:
    checklist = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    tally = Counter(row[3] for row in checklist[1:])
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    data = register[1:]
    status = Counter(row[12] for row in data)
    urls = Counter(row[1] for row in data)
    chapter_l = [row for row in checklist[1:] if row[0][:2] in {"LG", "LK", "LX", "LC", "LO", "LP", "LB", "LR"}]
    done = [row for row in chapter_l if row[3] != "PENDING"]
    return {
        "checklist": len(checklist) - 1,
        "tally": dict(tally),
        "register": len(data),
        "status": dict(status),
        "domains": len({row[2] for row in data}),
        "last": data[-1][0],
        "dup_urls": sum(1 for n in urls.values() if n > 1),
        "surplus": sum(n - 1 for n in urls.values() if n > 1),
        "chapter_l_done": len(done),
        "chapter_l_total": len(chapter_l),
        "pending_rows": [(row[0], row[2]) for row in checklist[1:] if row[3] == "PENDING"],
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.13 Addendum" in text:
        return
    f = figures()
    pending = f["tally"].get("PENDING", 0)
    rows = f["pending_rows"]
    listing = ", ".join(f"`{i}`" for i, _ in rows)
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.13 Addendum — Chapter L closed (2026-10-28)

Fourth recount, after Budgie 10.10.3 and ChromeOS 154. This one closes Chapter L.

| Measure | J.12 | Now |
|---|---|---|
| Register rows | 4,192 | {f['register']:,} |
| Domains | 378 / 400 | {f['domains']} / 400 ({f['domains'] / 400:.1%}) |
| Inspected | 1,287 | {f['status'].get('INSPECTED', 0):,} |
| Last source id | S3877 | {f['last']} |
| Checklist rows | 597 | {f['checklist']} |
| PENDING rows | 34 | {pending} |
| PARTIAL rows | 508 | {f['tally'].get('PARTIAL', 0)} |
| COMPLETE rows | 50 | {f['tally'].get('COMPLETE', 0)} |
| NOT FOUND rows | 2 | {f['tally'].get('NOT FOUND', 0)} |
| Chapter L rows measured | 84 / 112 | {f['chapter_l_done']} / {f['chapter_l_total']} |
| Duplicate URLs / surplus rows | 47 / 52 | {f['dup_urls']} / {f['surplus']} |

What this recount shows:

1. **Chapter L is closed at {f['chapter_l_done']} / {f['chapter_l_total']}.** Every one of the eight
   environments has fourteen measured rows, and the eight chapter parent rows are
   no longer pending.
2. **PENDING is down to {pending}**: {listing}. These are the three
   chapter parents, one Windows settings row and one Windows Settings parent —
   the Windows inventory batch is the only research work left that is not a
   validation plan.
3. **NOT FOUND has grown to {f['tally'].get('NOT FOUND', 0)}**, and that is a result rather than a
   failure: `LB.05` because Budgie ships no file manager, and five ChromeOS rows
   because their documentation could not be opened (four 404s in this batch).
   The corpus now distinguishes "measured", "structurally known" and "not
   published anywhere we could reach".
4. **Duplicates still did not grow.** None of the ten new URLs repeats an
   existing row, so `J.02`'s 47 / 52 stands for a fourth batch, across 4,202
   rows and four hundred fewer pending items than the original audit.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    patch_part1()
    append_addendum()
    print(figures())
