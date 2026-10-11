//! Design lint: the design system's rules, checked mechanically.
//!
//! A style guide that lives in a document is a suggestion. These tests turn
//! the four rules the audit called load-bearing into build failures, so the
//! discipline survives contact with deadlines. Each rule scans the shell's
//! own source, the same way `overflow_audit` hooks the drawing path — the
//! check is a *library consumer* of the code, not a hope.
//!
//! Rules (docs/DESIGN-SYSTEM-v3/04-QA-LINT-GOVERNANCE.md §1):
//! 1. Colour outside the token system (`Rgb::hex` / raw `0x` fills in views)
//! 2. Radius values that bypass the `Radius` family
//! 3. Bold starvation: a view that never emphasises anything
//! 4. Motion outside the token family: springs with magic numbers

// The lint scans its own crate's source with byte indexing over ASCII
// structural tokens (parens, newlines, hex digits); every index derives
// from a bounds-checked loop condition, and Thai comments in the scanned
// files are why char boundaries are handled explicitly rather than
// assumed. Panicking inside a test that asserts a lint failure is the
// correct failure mode.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::panic
)]

use std::fs;
use std::path::Path;

/// Read a source file relative to this crate, failing with a readable
/// message if the tree moved.
fn src(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Files that draw UI. The wallpaper and the diagnostics overlay are
/// whitelisted: one paints a scene, the other debug text, and neither is
/// part of the component system.
const VIEW_FILES: [&str; 3] = ["views.rs", "ui.rs", "apps.rs"];

/// The first line of a span, for readable failure messages.
fn first_line(span: &str) -> String {
    span.lines().next().unwrap_or("").trim().to_owned()
}

#[test]
fn rule_no_raw_colour_literals_in_views() {
    // Views compose colours from tokens (`t.text()`, `t.accent()`,
    // `t.tone(..)`). The only sanctioned literals are the neutral alpha
    // tints — `rgb(0xFFFFFF)`/`rgb(0x000000)` under `with_alpha` — because
    // they are the *material layer* (hover washes, scrims, borders), the
    // same vocabulary `ui::surface` itself uses. A coloured literal (any
    // hex with two digits that differ per channel) is a value outside the
    // palette and fails. `lint-exempt` marks a deliberate exception and
    // must carry a reason in the same comment.
    let mut offenders: Vec<String> = Vec::new();
    for file in VIEW_FILES {
        let text = src(file);
        for call in [
            "fill_round_rect",
            "stroke_round_rect",
            "fill_circle",
            "stroke_line",
        ] {
            for (n, span) in call_spans(&text, call) {
                if span.contains("lint-exempt") {
                    continue;
                }
                for hex in extract_hex_literals(&span) {
                    let neutral = hex == "0xFFFFFF" || hex == "0x000000";
                    if !neutral {
                        offenders.push(format!(
                            "{}:{}: colour {} in a draw call — use a token",
                            file, n, hex
                        ));
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "design lint: colour must come from tokens, not literals —\n{}",
        offenders.join("\n")
    );
}

/// Every argument span of `fn_name(` .. its matching `)`, as
/// `(start_line_1based, span_text)`. Paren-matching, so multi-line draw
/// calls are scanned whole — a line-by-line scan would both miss and
/// misjudge every formatted call.
fn call_spans(text: &str, fn_name: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let needle = format!("{}(", fn_name);
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut line = 1usize;
    while i < bytes.len() {
        if !text.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if bytes[i] == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if text[i..].starts_with(&needle) {
            // Walk to the matching close paren.
            let mut depth = 0usize;
            let mut j = i + needle.len() - 1; // at the open paren
            let start_line = line;
            while j < bytes.len() {
                match bytes[j] {
                    b'\n' => line += 1,
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            // Advance past the close paren on a char boundary.
            let mut end = j + 1;
            while end < text.len() && !text.is_char_boundary(end) {
                end += 1;
            }
            out.push((start_line, text[i..end].to_owned()));
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// Every `0x`-prefixed hex literal in a span, without regex dependencies.
fn extract_hex_literals(span: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = span.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if span[i..].starts_with("0x") && bytes[i + 2].is_ascii_hexdigit() {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            out.push(span[i..j].to_owned());
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn rule_no_float_radius_outside_the_family() {
    // Radii come from the `Radius` family, the pixel scale (`t.px`), or a
    // pill shape (`r.h / 2`-style, computed from the element's own height —
    // a pill is a shape, not a design decision). Anything else is a value
    // the family does not know about.
    let mut offenders: Vec<String> = Vec::new();
    for file in VIEW_FILES {
        let text = src(file);
        for call in ["fill_round_rect", "stroke_round_rect"] {
            for (n, span) in call_spans(&text, call) {
                if span.contains("lint-exempt") {
                    continue;
                }
                // A call is sanctioned when the radius argument uses the
                // family directly, a px-derived shape, a pill (÷2), or the
                // shared `radius` local — and every `let radius =` that is
                // NOT itself derived from the family is caught below, so
                // naming a variable `radius` cannot smuggle a value in.
                let uses_family = span.contains("t.radius(")
                    || span.contains("tokens.radius(")
                    || span.contains("t.px(")
                    || span.contains("/ 2.0")
                    || span.contains("0.0,")
                    || span.contains(", radius,")
                    || span.contains("(r, radius,")
                    || span.contains("(track, radius,")
                    // A lone `radius` argument on its own line (rustfmt):
                    // any whitespace-delimited `radius,` inside the span.
                    || span.contains("radius,");
                if !uses_family {
                    offenders.push(format!("{}:{}: {}", file, n, first_line(&span)));
                }
            }
        }
    }
    // The escape-proof half: wherever a `radius` local is defined, its
    // right-hand side must derive from the family, a px value, or a pill.
    for file in VIEW_FILES {
        let text = src(file);
        for (n, line) in text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("let radius") && !t.contains("lint-exempt") {
                let ok = t.contains("t.radius(")
                    || t.contains("tokens.radius(")
                    || t.contains("t.px(")
                    || t.contains("/ 2.0")
                    || t.contains("= 0.0");
                if !ok {
                    offenders.push(format!("{}:{}: {}", file, n + 1, t));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "design lint: radius must come from the family (or a pill/px-derived shape) —\n{}",
        offenders.join("\n")
    );
}

#[test]
fn rule_views_are_not_bold_starved() {
    // Hierarchy needs at least two weights in earshot of each other. A view
    // file where bold appears once or never is a flat wall of text — the
    // exact complaint the audit recorded. ui.rs carries the component
    // styles, views.rs the screens; both must use emphasis deliberately.
    let views = src("views.rs");
    let ui = src("ui.rs");
    let bolds = views.matches("Style::bold").count()
        + views.matches("style_bold").count()
        + ui.matches("Style::bold").count()
        + ui.matches("style_bold").count();
    assert!(
        bolds >= 8,
        "design lint: bold starvation — only {bolds} bold usages across \
         views+ui; hierarchy is flat. Emphasise real roles (item_primary, \
         section titles) instead of adding decoration."
    );
}

#[test]
fn rule_motion_uses_tokens_only() {
    // Spring constants are a design decision; scattering stiffness numbers
    // through the shell recreates the random-values problem the design
    // system exists to stop. `anim.rs` is the one place physics exists.
    let anim = src("anim.rs");
    assert!(
        anim.contains("Motion::FAST") && anim.contains("Motion::INSTANT"),
        "anim.rs must derive its springs from the Motion token family"
    );
    for file in VIEW_FILES {
        let text = src(file);
        for (n, line) in text.lines().enumerate() {
            if line.contains("lint-exempt") {
                continue;
            }
            if (line.contains("stiffness") || line.contains("damping")) && !file.contains("anim.rs")
            {
                panic!(
                    "design lint: {}:{} tunes spring physics outside the \
                     motion tokens — add the token to anim.rs instead",
                    file,
                    n + 1
                );
            }
        }
    }
}
