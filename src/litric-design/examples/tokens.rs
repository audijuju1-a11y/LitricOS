//! Emit every design token, and the adaptive-glass solution for a set of
//! wallpapers, as JSON on stdout.
//!
//! This exists so that the published design page is *reported* rather than
//! typed. A number written by hand into a document is a number that stops being
//! true the first time the palette moves, and nobody notices, because documents
//! do not fail. Run:
//!
//! ```text
//! cargo run -p litric-design --example tokens > tokens.json
//! ```

use litric_design::{Glass, Motion, Radius, Rgb, Space, Theme, ThemeKind, TypeStep};

/// Backdrops the material has to survive. Not a random spread: the two extremes
/// where the guarantee is hardest, plus the mid-greys where a naive linear
/// average of channels would give the wrong answer.
const WALLPAPERS: &[(&str, u32)] = &[
    ("pure black", 0x000000),
    ("deep navy", 0x0A1633),
    ("dark slate", 0x1E232B),
    ("forest", 0x14361F),
    ("mid grey", 0x808080),
    ("warm sand", 0xC8B49A),
    ("light grey", 0xD8DCE2),
    ("pure white", 0xFFFFFF),
];

/// WCAG AA for body text.
const AA: f64 = 4.5;

fn rgb_json(name: &str, c: Rgb) -> String {
    format!("\"{}\":\"{}\"", name, c.to_hex())
}

fn main() {
    let mut out = String::from("{\n");

    // ---- themes ----------------------------------------------------------
    out.push_str("\"themes\":[\n");
    for (i, kind) in ThemeKind::all().iter().enumerate() {
        let t = Theme::of(*kind);
        let p = t.palette;
        if i > 0 {
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "{{\"kind\":\"{}\",\"requires_aaa\":{},\"palette\":{{{}}},\
             \"glass\":{{\"blur_px\":{},\"saturation_pct\":{},\"tint_alpha_pct\":{},\
             \"border_alpha_pct\":{},\"highlight_alpha_pct\":{}}}}}",
            kind.token(),
            kind.requires_aaa(),
            [
                rgb_json("bg_base", p.bg_base),
                rgb_json("bg_elevated", p.bg_elevated),
                rgb_json("glass_tint", p.glass_tint),
                rgb_json("text_primary", p.text_primary),
                rgb_json("text_secondary", p.text_secondary),
                rgb_json("accent", p.accent),
                rgb_json("on_accent", p.on_accent),
                rgb_json("border", p.border),
                rgb_json("danger", p.danger),
                rgb_json("success", p.success),
                rgb_json("warning", p.warning),
            ]
            .join(","),
            t.glass.blur_px,
            t.glass.saturation_pct,
            t.glass.tint_alpha_pct,
            t.glass.border_alpha_pct,
            t.glass.highlight_alpha_pct,
        ));
    }
    out.push_str("\n],\n");

    // ---- adaptive glass --------------------------------------------------
    //
    // The panel must satisfy the *worse* of the two text roles, so the alpha it
    // actually uses is the larger of the two solutions. Reporting only the
    // primary-text figure would flatter the material by roughly 20 points.
    out.push_str("\"adaptive\":[\n");
    let mut first = true;
    for kind in [ThemeKind::Dark, ThemeKind::Light] {
        let t = Theme::of(kind);
        let tint = t.palette.glass_tint;
        for (name, hex) in WALLPAPERS {
            let bg = Rgb::hex(*hex);
            let a_pri = Glass::adaptive_alpha_pct(tint, t.palette.text_primary, bg, AA);
            let a_sec = Glass::adaptive_alpha_pct(tint, t.palette.text_secondary, bg, AA);
            let used = match (a_pri, a_sec) {
                (Some(a), Some(b)) => Some(a.max(b)),
                _ => None,
            };
            let adapted = t
                .glass
                .adapted_to(tint, t.palette.text_secondary, bg, AA)
                .adapted_to(tint, t.palette.text_primary, bg, AA);
            let surface = Glass {
                tint_alpha_pct: used.unwrap_or(t.glass.tint_alpha_pct),
                ..adapted
            }
            .effective_surface(tint, bg);
            if !first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(&format!(
                "{{\"theme\":\"{}\",\"wallpaper\":\"{}\",\"backdrop\":\"{}\",\
                 \"static_pct\":{},\"primary_pct\":{},\"secondary_pct\":{},\"used_pct\":{},\
                 \"surface\":\"{}\",\"contrast_primary\":{:.2},\"contrast_secondary\":{:.2}}}",
                kind.token(),
                name,
                bg.to_hex(),
                t.glass.tint_alpha_pct,
                a_pri.map_or(-1i32, i32::from),
                a_sec.map_or(-1i32, i32::from),
                used.map_or(-1i32, i32::from),
                surface.to_hex(),
                t.palette.text_primary.contrast(surface),
                t.palette.text_secondary.contrast(surface),
            ));
        }
    }
    out.push_str("\n],\n");

    // ---- scales ----------------------------------------------------------
    out.push_str("\"type\":[");
    for (i, s) in TypeStep::all().iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"px\":{},\"line_height_px\":{},\"large\":{}}}",
            s.px(),
            s.line_height_px(),
            s.is_large_text()
        ));
    }
    out.push_str("],\n\"space\":[");
    for (i, s) in Space::all().iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{}", s.px()));
    }
    out.push_str("],\n\"radius\":[");
    for (i, r) in Radius::all().iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{}", r.px()));
    }
    out.push_str("],\n\"motion\":[");
    for (i, m) in Motion::all().iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"stiffness\":{},\"damping\":{},\"settle_ms\":{}}}",
            m.stiffness, m.damping, m.settle_ms
        ));
    }
    let reduced = Motion::reduced();
    out.push_str(&format!(
        "],\n\"motion_reduced\":{{\"stiffness\":{},\"damping\":{},\"settle_ms\":{}}},\n",
        reduced.stiffness, reduced.damping, reduced.settle_ms
    ));
    out.push_str(&format!(
        "\"limits\":{{\"min_aesthetic_alpha_pct\":{},\"max_adaptive_alpha_pct\":{},\
         \"min_static_alpha_pct\":{},\"max_settle_ms\":{},\"aa\":{AA}}}\n}}\n",
        Glass::MIN_AESTHETIC_ALPHA_PCT,
        Glass::MAX_ADAPTIVE_ALPHA_PCT,
        Glass::MIN_TINT_ALPHA_PCT,
        Motion::MAX_SETTLE_MS,
    ));

    println!("{out}");
}
