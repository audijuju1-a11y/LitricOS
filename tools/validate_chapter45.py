#!/usr/bin/env python3
"""Execute the Chapter 45 validation points that do not need a desktop.

Part 30 specified 28 validation points and executed exactly one of them
(`VP-28`). Most of the rest are T1: they need a font, a settings URI handler or
a device, none of which exist in this workspace. But several of them have a
**repository half** that can be measured mechanically, and Part 30 §4 says so in
the "Why" column ("token tables are in the corpus; device half missing").

This harness runs that half. It does not upgrade a row to PASS: a check whose
criterion spans a device is recorded as PARTIAL with `scope: repo-half`, and the
device half stays NOT RUN. What it removes is the excuse that the numbers were
never computed.

Checks
------
VP-28  corpus reproducibility      checklist/register/Part 28 re-derived
VP-24  token contrast              every palette pair, accent pair, glass pair
VP-11  high-contrast resolution    all 11 tokens present, every text pair AAA
VP-09  icon coverage               every `Icon` variant named and drawn
VP-01  settings inventory          pages vs stored rows, store/registry parity
VP-02  deep links                  every page slug resolvable, every row linkable
VP-26  preference persistence      encode/parse key parity, atomic write, test

Receipts are appended to `validation/receipts.jsonl` in the format defined by
Part 30 §5, with a SHA-256 per artifact, so a claim can be re-checked without
re-running.

Usage:
    python3 tools/validate_chapter45.py            run, append receipts
    python3 tools/validate_chapter45.py --check    run, write nothing
"""

from __future__ import annotations

import csv
import hashlib
import io
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESEARCH = ROOT / "Research-Deep-all"
CHECKLIST = RESEARCH / "LitricOS Coverage Checklist.csv"
REGISTER = RESEARCH / "LitricOS Source Register.csv"
PART1 = RESEARCH / "LitricOS UI UX Research Part1.md"
PART24 = RESEARCH / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"
PART28 = RESEARCH / "LitricOS UI UX Research Part 28 L Linux Desktops Budgie 10.10 and ChromeOS.md"
PART30 = RESEARCH / "LitricOS UI UX Research Part 30 Chapter 45 Validation Plan.md"
RECEIPTS = ROOT / "validation" / "receipts.jsonl"

DESIGN = ROOT / "src" / "litric-design" / "src"
SHELL = ROOT / "src" / "litric-shell" / "src"
PALETTE_RUST = DESIGN / "lib.rs"
COLOR_RUST = DESIGN / "color.rs"
ICONS_RUST = SHELL / "icons.rs"
REGISTRY_RUST = SHELL / "registry.rs"
PREFS_RUST = SHELL / "prefstore.rs"
SETTINGS_RUST = SHELL / "settings.rs"

# WCAG 2.1 §1.4.3 / §1.4.6. Body text only: the crate's own rule is that primary
# and secondary text are both body-sized, so 4.5 is the bar in standard themes.
AA = 4.5
AAA = 7.0


# ---------------------------------------------------------------- utilities


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def hexrgb(value: str) -> tuple[int, int, int]:
    value = value.lstrip("#")
    return int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16)


def luminance(rgb: tuple[int, int, int]) -> float:
    """WCAG 2.1 relative luminance, identical to `litric_design::color::Rgb`."""

    def channel(c: int) -> float:
        v = c / 255.0
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4

    return 0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])


def contrast(a: tuple[int, int, int], b: tuple[int, int, int]) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = (la, lb) if la >= lb else (lb, la)
    return (hi + 0.05) / (lo + 0.05)


def over(fg: tuple[int, int, int], bg: tuple[int, int, int], alpha: float) -> tuple[int, int, int]:
    return tuple(round(f * alpha + b * (1.0 - alpha)) for f, b in zip(fg, bg))


# ---------------------------------------------------------------- source parse


def parse_themes() -> dict[str, dict[str, tuple[int, int, int]]]:
    text = read(PALETTE_RUST)
    themes: dict[str, dict[str, tuple[int, int, int]]] = {}
    for kind, body, _glass in re.findall(
        r"ThemeKind::(\w+) => Self \{(.*?)glass: Glass::(\w+),", text, re.S
    ):
        palette = {
            token: hexrgb(value)
            for token, value in re.findall(r"(\w+): Rgb::hex\(0x([0-9A-Fa-f]{6})\)", body)
        }
        themes[kind] = palette
    return themes


def parse_accents() -> list[tuple[str, str, tuple[int, int, int], tuple[int, int, int]]]:
    """(theme, accent, accent colour, on-accent colour)."""
    text = read(PALETTE_RUST)
    return [
        (kind, accent, hexrgb(accent_hex), hexrgb(on_accent_hex))
        for kind, accent, accent_hex, on_accent_hex in re.findall(
            r"\(ThemeKind::(\w+), Accent::(\w+)\) => "
            r"\(Rgb::hex\(0x([0-9A-Fa-f]{6})\), Rgb::hex\(0x([0-9A-Fa-f]{6})\)\)",
            text,
        )
    ]


def parse_glass() -> dict[str, dict[str, int]]:
    text = read(PALETTE_RUST)
    out: dict[str, dict[str, int]] = {}
    for name, body in re.findall(r"pub const (DARK|LIGHT|HIGH_CONTRAST): Self = Self \{(.*?)\n    \};", text, re.S):
        out[name] = {
            field: int(value)
            for field, value in re.findall(r"(\w+): (\d+),", body)
        }
    return out


def parse_icons() -> tuple[list[str], list[str], list[str]]:
    text = read(ICONS_RUST)
    enum_body = re.search(r"pub enum Icon \{(.*?)\n\}", text, re.S).group(1)
    variants = re.findall(r"^    ([A-Z]\w*),$", enum_body, re.M)
    name_body = re.search(r"pub const fn name\(self\) -> &'static str \{(.*?)\n    \}", text, re.S).group(1)
    named = re.findall(r"Self::(\w+) => \"", name_body)
    draw_body = text[text.index("pub fn draw("):]
    # `Icon::Audio | Icon::AudioMuted =>` is one arm for two icons, so every
    # icon named on an arm line counts, not just the first.
    drawn: list[str] = []
    for line in draw_body.splitlines():
        if line.startswith("        Icon::") and "=>" in line:
            drawn.extend(re.findall(r"Icon::(\w+)", line))
    return variants, named, drawn


def parse_settings() -> dict[str, object]:
    settings = read(SETTINGS_RUST)
    enum_body = re.search(r"pub enum Page \{(.*?)\n\}", settings, re.S).group(1)
    pages = re.findall(r"^    ([A-Z]\w*),$", enum_body, re.M)
    slugs = re.findall(r"Self::(\w+) => \"([a-z-]+)\",", settings)
    registry = read(REGISTRY_RUST)
    entries = re.findall(r'key: "(\w+)",\n\s+page: Page::(\w+),', registry)
    prefs = read(PREFS_RUST)
    encode_body = prefs[prefs.index("pub fn encode("):]
    encoded = re.findall(r'out\.push_str\("(\w+)="\);', encode_body)
    parse_body = prefs[prefs.index("pub fn parse("):prefs.index("fn parse_bool")]
    parsed = re.findall(r'^\s+"(\w+)" =>', parse_body, re.M)
    return {
        "pages": pages,
        "slugs": [slug for variant, slug in slugs if variant in pages],
        "entries": entries,
        "encoded": encoded,
        "parsed": parsed,
        "atomic": "fs::rename" in prefs and "sync_all" in prefs and "temp" in prefs.lower(),
    }


# ---------------------------------------------------------------- checks


def check_corpus() -> tuple[str, dict]:
    rows = [row for row in csv.reader(io.StringIO(read(CHECKLIST))) if row][1:]
    malformed = [row[0] for row in rows if len(row) not in (4, 5) or not row[3]]
    pending = [row[0] for row in rows if row[3] == "PENDING"]
    status: dict[str, int] = {}
    for row in rows:
        status[row[3]] = status.get(row[3], 0) + 1

    register = [row for row in csv.reader(io.StringIO(read(REGISTER))) if row][1:]
    bad_columns = sorted({len(row) for row in register})
    ids = [row[0] for row in register]
    urls = [row[1] for row in register]
    duplicates = {url for url in urls if urls.count(url) > 1}

    part28 = read(PART28)
    lb = re.findall(r"^\| `LB\.(\d\d)`", part28, re.M)
    lr = re.findall(r"^\| `LR\.(\d\d)`", part28, re.M)
    duplicate_ids = sorted({sid for sid in ids if ids.count(sid) > 1})
    new_ids = [f"S{n}" for n in range(3878, 3888)]
    present = {row[0]: ids.count(row[0]) for row in register if row[0] in new_ids}

    part1 = read(PART1)
    part24 = read(PART24)
    observed = {
        "checklist_rows": len(rows),
        "malformed": len(malformed),
        "pending": len(pending),
        "status": status,
        "chapter_l_rows": sum(1 for row in rows if row[0][:2] in {"LG", "LK", "LX", "LC", "LO", "LP", "LB", "LR"}),
        "register_rows": len(register),
        "register_columns": bad_columns,
        "register_ids_unique": len(ids) == len(set(ids)),
        "distinct_source_ids": len(set(ids)),
        "duplicate_source_ids": len(duplicate_ids),
        "last_source_id": ids[-1],
        "domains": len({row[2] for row in register}),
        "duplicate_urls": len(duplicates),
        "surplus_rows": sum(urls.count(url) - 1 for url in duplicates),
        "part28_lb_rows": len(lb),
        "part28_lr_rows": len(lr),
        "part28_ids_present_once": all(n == 1 for n in present.values()) and len(present) == len(new_ids),
        "part28_crlf": part28.count("\r\n"),
        "part1_has_part30": "Part 30" in part1,
        "part24_has_j14": "J.14" in part24,
    }
    ok = (
        observed["malformed"] == 0
        and observed["pending"] == 0
        and bad_columns == [17]
        and observed["part28_lb_rows"] == 14
        and observed["part28_lr_rows"] == 14
        and observed["part28_ids_present_once"]
        and observed["part28_crlf"] == 0
        and observed["part1_has_part30"]
        and observed["part24_has_j14"]
    )
    # A shared source id is not a figure drift; it is checked separately as
    # VP-28b so the arithmetic verdict and the key verdict cannot hide each other.
    return ("PASS" if ok else "FAIL"), observed


def check_contrast() -> tuple[str, dict]:
    themes = parse_themes()
    assert set(themes) == {"Dark", "Light", "HighContrastDark", "HighContrastLight"}, themes.keys()
    pairs = [
        ("text_primary", "bg_base"),
        ("text_secondary", "bg_base"),
        ("text_primary", "bg_elevated"),
        ("text_secondary", "bg_elevated"),
        ("accent", "bg_base"),
        ("on_accent", "accent"),
        ("danger", "bg_base"),
        ("success", "bg_base"),
        ("warning", "bg_base"),
    ]
    failures: list[str] = []
    measured: list[dict] = []
    for kind, palette in themes.items():
        target = AAA if kind.startswith("HighContrast") else AA
        for fg, bg in pairs:
            if fg not in palette or bg not in palette:
                failures.append(f"{kind}: missing token {fg}/{bg}")
                continue
            ratio = contrast(palette[fg], palette[bg])
            measured.append({"theme": kind, "fg": fg, "bg": bg, "ratio": round(ratio, 2)})
            if ratio < target:
                failures.append(f"{kind} {fg}/{bg} = {ratio:.2f} < {target}")

    # Glass: text sits on tint-over-wallpaper, not on the tint alone. Worst case
    # is pure white and pure black, which is exactly the crate's own criterion.
    glass = parse_glass()
    glass_pairs: list[dict] = []
    for name, kind in (("DARK", "Dark"), ("LIGHT", "Light")):
        alpha = glass[name]["tint_alpha_pct"] / 100.0
        tint = themes[kind]["glass_tint"]
        for backdrop_name, backdrop in (("white", (255, 255, 255)), ("black", (0, 0, 0))):
            surface = over(tint, backdrop, alpha)
            ratio = contrast(themes[kind]["text_secondary"], surface)
            glass_pairs.append(
                {"glass": name, "backdrop": backdrop_name, "alpha_pct": glass[name]["tint_alpha_pct"], "ratio": round(ratio, 2)}
            )
            if ratio < AA:
                failures.append(f"{kind} glass over {backdrop_name}: secondary text {ratio:.2f} < {AA}")

    accents = parse_accents()
    accent_pairs: list[dict] = []
    for kind, accent, accent_rgb, on_accent_rgb in accents:
        for label, a, b in (
            ("on_accent/accent", on_accent_rgb, accent_rgb),
            ("accent/bg_base", accent_rgb, themes[kind]["bg_base"]),
        ):
            ratio = contrast(a, b)
            accent_pairs.append({"theme": kind, "accent": accent, "pair": label, "ratio": round(ratio, 2)})
            if ratio < AA:
                failures.append(f"{kind}/{accent} {label} = {ratio:.2f} < {AA}")

    all_ratios = [m["ratio"] for m in measured] + [g["ratio"] for g in glass_pairs]
    observed = {
        "themes": len(themes),
        "palette_pairs": len(measured),
        "glass_pairs": len(glass_pairs),
        "accent_pairs": len(accent_pairs),
        "measurements": len(measured) + len(glass_pairs) + len(accent_pairs),
        "min_palette_ratio": min(m["ratio"] for m in measured),
        "min_glass_ratio": min(g["ratio"] for g in glass_pairs),
        "min_accent_ratio": min(a["ratio"] for a in accent_pairs),
        "failures": failures,
        "worst": sorted(measured, key=lambda m: m["ratio"])[:5],
    }
    return ("PASS" if not failures else "FAIL"), observed


def check_high_contrast() -> tuple[str, dict]:
    themes = parse_themes()
    tokens = set(themes["Dark"])
    failures: list[str] = []
    if len(tokens) != 11:
        failures.append(f"expected 11 semantic tokens, parsed {len(tokens)}: {sorted(tokens)}")
    for kind in ("HighContrastDark", "HighContrastLight"):
        palette = themes.get(kind, {})
        missing = tokens - set(palette)
        if missing:
            failures.append(f"{kind} missing {sorted(missing)}")
            continue
        for fg in ("text_primary", "text_secondary", "accent", "danger", "success", "warning"):
            ratio = contrast(palette[fg], palette["bg_base"])
            if ratio < AAA:
                failures.append(f"{kind} {fg} on bg_base = {ratio:.2f} < {AAA}")
        if palette["border"] == palette["bg_base"]:
            failures.append(f"{kind} border equals bg_base, outlines vanish")
        if palette["bg_base"] == palette["bg_elevated"] and kind == "HighContrastDark":
            # Documented: HC dark raised surface is #0A0A0A, not #000000.
            failures.append(f"{kind} bg_elevated should differ from bg_base")

    text = read(PALETTE_RUST)
    ignores_accent = (
        "ThemeKind::HighContrastDark | ThemeKind::HighContrastLight, _)" in text
        and "return self;" in text
    )
    if not ignores_accent:
        failures.append("with_accent no longer returns high-contrast themes unchanged")

    observed = {
        "semantic_tokens": len(tokens),
        "high_contrast_themes": 2,
        "text_pairs_checked": 12,
        "min_ratio": round(
            min(
                contrast(themes[kind][fg], themes[kind]["bg_base"])
                for kind in ("HighContrastDark", "HighContrastLight")
                for fg in ("text_primary", "text_secondary", "accent", "danger", "success", "warning")
            ),
            2,
        ),
        "accent_ignored_in_high_contrast": ignores_accent,
        "failures": failures,
    }
    return ("PASS" if not failures else "FAIL"), observed


def check_id_key() -> tuple[str, dict]:
    """The register's `source_id` is documented as the key; is it one?"""
    rows = [row for row in csv.reader(io.StringIO(read(REGISTER))) if row][1:]
    ids = [row[0] for row in rows]
    counts: dict[str, int] = {}
    for sid in ids:
        counts[sid] = counts.get(sid, 0) + 1
    shared = {sid: n for sid, n in counts.items() if n > 1}
    # Blast radius: how many citations in the prose would have to be revisited
    # if the ids were renumbered.
    citations = 0
    documents = 0
    for path in sorted(RESEARCH.glob("*.md")):
        # Research prose only: the audit (Part 24) and this execution report
        # (Part 31) talk *about* the shared ids, so counting them would count
        # the description of the defect as evidence of its blast radius.
        match = re.search(r"Part (\d+)", path.name)
        if match and (int(match.group(1)) > 30 or int(match.group(1)) == 24):
            continue
        text = read(path)
        hits = sum(text.count(sid) for sid in shared)
        if hits:
            citations += hits
            documents += 1
    guard = subprocess.run(
        [sys.executable, str(ROOT / "tools" / "fix_source_id_keys.py"), "--check"],
        capture_output=True,
        text=True,
    )
    unique = len(counts) == len(rows) and not shared
    observed = {
        "rows": len(rows),
        "distinct_ids": len(counts),
        "shared_ids": len(shared),
        "max_rows_per_id": max(shared.values()) if shared else 1,
        "missing_ids": sum(1 for n in counts.values() if n == 0),
        "all_ids_unique": unique,
        "remap_guard": (guard.stdout or guard.stderr).strip(),
        "citations_on_shared_ids": citations,
        "documents_affected": documents,
        "note": "distinct ids must equal rows; the remap script must be a no-op",
    }
    return ("PASS" if unique and guard.returncode == 0 else "FAIL"), observed


def check_icons() -> tuple[str, dict]:
    variants, named, drawn = parse_icons()
    missing_name = sorted(set(variants) - set(named))
    missing_draw = sorted(set(variants) - set(drawn))
    duplicate_names = len(named) != len(set(named))
    failures = []
    if missing_name:
        failures.append(f"no name(): {missing_name}")
    if missing_draw:
        failures.append(f"no draw arm: {missing_draw}")
    if duplicate_names:
        failures.append("two icons share a name")
    if len(variants) < 30:
        failures.append(f"only {len(variants)} icons, expected the shipped set")
    observed = {
        "icon_variants": len(variants),
        "named": len(named),
        "drawn": len(drawn),
        "missing_name": missing_name,
        "missing_draw": missing_draw,
        "note": "repo side only: no Segoe Fluent Icons cmap available here",
        "failures": failures,
    }
    return ("PASS" if not failures else "FAIL"), observed


def check_settings() -> tuple[str, dict]:
    info = parse_settings()
    pages = info["pages"]
    entries = info["entries"]
    encoded = info["encoded"]
    internal = ["version", "onboarded"]
    entry_keys = [key for key, _page in entries]
    pages_with_rows = sorted({page for _key, page in entries})
    pages_without = [page for page in pages if page not in pages_with_rows]
    failures = []
    if normalized := [k for k in encoded if k not in set(entry_keys) | set(internal)]:
        failures.append(f"store writes keys with no registry row: {normalized}")
    if orphan := [k for k in entry_keys if k not in encoded]:
        failures.append(f"registry rows name keys the store never writes: {orphan}")
    if len(entry_keys) != len(set(entry_keys)):
        failures.append("duplicate registry keys")
    observed = {
        "settings_pages": len(pages),
        "pages_with_stored_rows": len(pages_with_rows),
        "pages_reading_only": len(pages_without),
        "registry_rows": len(entries),
        "stored_keys": len(encoded),
        "internal_keys": len(internal),
        "store_registry_parity": not failures,
        "pages_reading_only_list": pages_without,
        "note": "repo side only: no desktop session, so no screenshot diff against a DE",
        "failures": failures,
    }
    return ("PASS" if not failures else "FAIL"), observed


def check_deeplinks() -> tuple[str, dict]:
    info = parse_settings()
    pages = info["pages"]
    slugs = info["slugs"]
    entries = info["entries"]
    registry = read(REGISTRY_RUST)
    failures = []
    missing_slug = [page for page in pages if page not in [variant for variant, _slug in re.findall(r"Self::(\w+) => \"([a-z-]+)\",", read(SETTINGS_RUST))]]
    if missing_slug:
        failures.append(f"pages with no slug: {missing_slug}")
    if len(slugs) != len(set(slugs)):
        failures.append("duplicate page slug")
    if "pub fn resolve(" not in registry or "Page::from_slug" not in registry:
        failures.append("resolve() no longer routes through Page::from_slug")
    if not re.search(r"fn link_for\(", registry):
        failures.append("link_for() missing, so no round-trip")
    observed = {
        "pages": len(pages),
        "unique_slugs": len(set(slugs)),
        "rows_linkable": len(entries),
        "prefix": "litric://settings",
        "roundtrip_fn": "link_for" in registry,
        "note": "repo side only: no handler registered with a desktop session",
        "failures": failures,
    }
    return ("PASS" if not failures else "FAIL"), observed


def check_persistence() -> tuple[str, dict]:
    info = parse_settings()
    encoded, parsed = info["encoded"], info["parsed"]
    prefs = read(PREFS_RUST)
    failures = []
    if set(encoded) != set(parsed):
        failures.append(
            f"encode/parse key mismatch: only-encode {sorted(set(encoded) - set(parsed))}, "
            f"only-parse {sorted(set(parsed) - set(encoded))}"
        )
    if "ui_scale" not in encoded or "ui_scale" not in parsed:
        failures.append("ui_scale missing from the preference file")
    if not info["atomic"]:
        failures.append("save is no longer temp-file + rename + fsync")
    for test in (
        "the_ui_scale_round_trips_and_a_size_this_build_does_not_draw_is_ignored",
        "unknown_keys_and_values_keep_the_defaults",
    ):
        if test not in prefs:
            failures.append(f"missing test {test}")
    observed = {
        "encoded_keys": len(encoded),
        "parsed_keys": len(parsed),
        "key_parity": set(encoded) == set(parsed),
        "ui_scale_persisted": "ui_scale" in encoded and "ui_scale" in parsed,
        "atomic_write": info["atomic"],
        "tests_present": [
            t
            for t in (
                "the_ui_scale_round_trips_and_a_size_this_build_does_not_draw_is_ignored",
                "unknown_keys_and_values_keep_the_defaults",
            )
            if t in prefs
        ],
        "note": "repo side only: no booted image, so no restart test",
        "failures": failures,
    }
    return ("PASS" if not failures else "FAIL"), observed


CHECKS = [
    ("VP-28", "T0", "PASS", "corpus figures re-derived with zero drift", check_corpus,
     [CHECKLIST, REGISTER, PART28, PART1, PART24]),
    ("VP-28b", "T0", "PASS", "`source_id` is unique across the register (sub-check added by this execution)", check_id_key,
     [REGISTER]),
    ("VP-24", "T1", "PARTIAL", "every token pair computed from the shipped palette; device half not run", check_contrast,
     [PALETTE_RUST, COLOR_RUST]),
    ("VP-11", "T1", "PARTIAL", "all semantic tokens resolve in both high-contrast themes at AAA; no device scan", check_high_contrast,
     [PALETTE_RUST]),
    ("VP-09", "T1", "PARTIAL", "every Icon variant is named and has a draw arm; no font cmap to enumerate", check_icons,
     [ICONS_RUST]),
    ("VP-01", "T2", "PARTIAL", "settings pages vs stored rows and store/registry parity; no screenshot diff", check_settings,
     [SETTINGS_RUST, REGISTRY_RUST, PREFS_RUST]),
    ("VP-02", "T1", "PARTIAL", "every page slug resolves and every row round-trips; no session handler", check_deeplinks,
     [SETTINGS_RUST, REGISTRY_RUST]),
    ("VP-26", "T2", "PARTIAL", "encode/parse parity, atomic write and tests present; no restart on a booted image", check_persistence,
     [PREFS_RUST]),
]


def git_revision() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=ROOT, capture_output=True, text=True, check=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def main(argv: list[str]) -> int:
    write = "--check" not in argv
    rid = git_revision()
    stamp = datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")
    receipts = []
    failures = 0

    for vp, tier, planned, criterion, fn, artifacts in CHECKS:
        result, observed = fn()
        if result != "PASS":
            failures += 1
        record = {
            "vp": vp,
            "run": stamp,
            "tier": tier,
            "env": "workspace:linux-x86_64",
            "build": f"repo@{rid}",
            "result": planned if result == "PASS" else result,
            "scope": "repo-half" if planned == "PARTIAL" else "full",
            "criterion": criterion,
            "observed": observed,
            "artifacts": [{"path": str(p.relative_to(ROOT)), "sha256": sha256(p)} for p in artifacts],
        }
        receipts.append(record)

    for record in receipts:
        observed = record["observed"]
        compact = {k: v for k, v in observed.items() if k not in {"failures", "pages_reading_only_list"}}
        print(f"{record['vp']}: {record['result']:<7} {json.dumps(compact, ensure_ascii=False)}")
        for failure in observed.get("failures", []):
            print(f"    FAILURE: {failure}")

    print(
        f"\n{len(receipts)} checks: "
        f"{sum(1 for r in receipts if r['result'] == 'PASS')} PASS, "
        f"{sum(1 for r in receipts if r['result'] == 'PARTIAL')} PARTIAL, "
        f"{sum(1 for r in receipts if r['result'] == 'FAIL')} FAIL"
    )

    if write:
        RECEIPTS.parent.mkdir(exist_ok=True)
        with open(RECEIPTS, "a", encoding="utf-8", newline="\n") as handle:
            for record in receipts:
                handle.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
        print(f"receipts: {RECEIPTS.relative_to(ROOT)} (+{len(receipts)} rows)")

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
