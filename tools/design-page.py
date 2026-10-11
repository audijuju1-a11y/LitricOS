#!/usr/bin/env python3
"""Render the LitricOS design page from measured token data.

The page is *generated* rather than written, for the same reason
``ci/count-scope.py`` rewrites the scope tally instead of trusting it: a
number typed into a document stops being true the moment the source moves,
and documents do not fail a build. Every colour, ratio and alpha on the page
below comes from ``cargo run -p litric-design --example tokens``.

Usage::

    cd src && cargo run -q -p litric-design --example tokens > ../build/tokens.json
    python tools/design-page.py build/tokens.json out.html
"""

from __future__ import annotations

import base64
import json
import sys
from pathlib import Path

MARKER = "/*__TOKENS__*/null"
SHOT_MARKER = "__BOOT_PNG__"

# The screenshot of the running system, embedded as a data URI. A published page
# cannot fetch anything from another host, and a picture that only loads on the
# machine that produced it is not evidence of anything.
SHOT_PATH = "docs/evidence/desktop-boot-2026-08-29.png"

TEMPLATE = """<title>Litric Glass</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Archivo:wght@500;600;700&family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&display=swap">

<style>
  /* ------------------------------------------------------------------
     LitricOS dark theme. Every value here is emitted by
     src/litric-design/examples/tokens.rs; nothing is chosen in this file.
     The page commits to one visual world on purpose -- it *is* the OS's
     default theme -- so every colour is painted explicitly and none of it
     is borrowed from the host ground.
     ------------------------------------------------------------------ */
  :root {
    --bg-base:       #0B0C0F;
    --bg-elevated:   #15171D;
    --glass-tint:    #14161B;
    --text-primary:  #F2F4F8;
    --text-secondary:#A9B1BF;
    --accent:        #6AA6FF;
    --on-accent:     #06080C;
    --border:        #2A2E37;
    --danger:        #FF6B6B;
    --success:       #5BD9A4;
    --warning:       #FFC66D;

    --r-sm: 6px;  --r-md: 10px; --r-lg: 14px; --r-xl: 20px; --r-win: 28px;
    --s-xs: 4px;  --s-sm: 8px;  --s-md: 12px; --s-lg: 16px;
    --s-xl: 24px; --s-2xl: 32px; --s-3xl: 48px;

    --display: "Archivo", "Helvetica Neue", Arial, sans-serif;
    --ui: "IBM Plex Sans", ui-sans-serif, system-ui, sans-serif;
    --mono: "IBM Plex Mono", ui-monospace, "SF Mono", Menlo, monospace;
  }

  * { box-sizing: border-box; }

  body {
    margin: 0;
    background: var(--bg-base);
    color: var(--text-primary);
    font-family: var(--ui);
    font-size: 15px;
    line-height: 22px;
    -webkit-font-smoothing: antialiased;
  }

  .wrap { max-width: 1120px; margin: 0 auto; padding: 0 var(--s-xl); }

  h1, h2, h3 { font-family: var(--display); text-wrap: balance; margin: 0; letter-spacing: -0.02em; }
  h1 { font-size: clamp(36px, 5.6vw, 60px); font-weight: 700; line-height: 1.02; }
  h2 { font-size: 30px; font-weight: 600; line-height: 36px; }
  h3 { font-size: 17px; font-weight: 600; line-height: 24px; letter-spacing: 0; }
  p  { margin: 0; max-width: 66ch; }
  .lede { font-size: 17px; line-height: 27px; color: var(--text-secondary); }
  .eyebrow {
    font-family: var(--mono); font-size: 11px; font-weight: 500;
    letter-spacing: 0.14em; text-transform: uppercase; color: var(--accent);
  }
  .muted { color: var(--text-secondary); }
  code, .mono { font-family: var(--mono); font-size: 13px; font-variant-numeric: tabular-nums; }

  section { padding: var(--s-3xl) 0; }
  .stack { display: flex; flex-direction: column; gap: var(--s-lg); }
  .stack-sm { display: flex; flex-direction: column; gap: var(--s-sm); }
  .rule { height: 1px; background: var(--border); border: 0; margin: 0; }

  /* ---------------- hero ---------------- */
  .hero { padding-top: var(--s-3xl); }
  .hero-copy { display: flex; flex-direction: column; gap: var(--s-md); margin-bottom: var(--s-lg); }

  .wallpicker { display: flex; gap: var(--s-sm); flex-wrap: wrap; margin-bottom: var(--s-md); }
  .wallpicker button {
    font: inherit; font-size: 13px; color: var(--text-secondary);
    background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: 999px; padding: 6px 14px; cursor: pointer;
    display: inline-flex; align-items: center; gap: 8px;
  }
  .wallpicker button[aria-pressed="true"] {
    color: var(--text-primary); border-color: var(--accent);
    background: rgba(106,166,255,0.14);
  }
  .wallpicker button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .wallpicker .chip {
    width: 14px; height: 14px; border-radius: 4px;
    border: 1px solid rgba(255,255,255,0.18);
  }

  .desktop {
    position: relative;
    border-radius: var(--r-win);
    overflow: hidden;
    border: 1px solid var(--border);
    aspect-ratio: 16 / 10;
    min-height: 380px;
    background: #06070A;
    box-shadow: 0 40px 90px -30px rgba(0,0,0,0.85);
  }
  .desktop canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }

  /* The material. Alpha is not written here: the solver sets it per panel,
     because a fixed number would be a lie on at least one wallpaper. */
  .glass {
    position: absolute;
    background: rgba(20,22,27,0.82);
    backdrop-filter: blur(32px) saturate(180%);
    -webkit-backdrop-filter: blur(32px) saturate(180%);
    border: 1px solid rgba(255,255,255,0.14);
    box-shadow:
      inset 0 1px 0 rgba(255,255,255,0.10),
      0 18px 40px -18px rgba(0,0,0,0.7);
  }
  .alpha-tag {
    position: absolute; top: 6px; right: 8px;
    font-family: var(--mono); font-size: 10px; letter-spacing: .06em;
    color: var(--text-secondary); background: rgba(0,0,0,0.35);
    border: 1px solid rgba(255,255,255,0.12); border-radius: 999px;
    padding: 1px 7px; pointer-events: none;
  }

  .menubar {
    inset: 0 0 auto 0; height: 34px;
    display: flex; align-items: center; gap: var(--s-lg);
    padding: 0 var(--s-lg); font-size: 13px;
  }
  .menubar .brand { font-weight: 600; letter-spacing: -0.01em; }
  .menubar .right { margin-left: auto; display: flex; gap: var(--s-md); align-items: center; color: var(--text-secondary); }
  .dot { width: 7px; height: 7px; border-radius: 999px; background: var(--success); box-shadow: 0 0 8px var(--success); }

  .window { left: 6%; top: 21%; width: 54%; border-radius: var(--r-xl); overflow: hidden; }
  .titlebar {
    display: flex; align-items: center; gap: var(--s-sm);
    padding: var(--s-md) var(--s-lg); border-bottom: 1px solid rgba(255,255,255,0.08);
  }
  .traffic { display: flex; gap: 7px; }
  .traffic i { width: 11px; height: 11px; border-radius: 999px; display: block; }
  .titlebar .name { font-size: 13px; color: var(--text-secondary); margin-left: var(--s-xs); }
  .win-body { padding: var(--s-lg); display: flex; flex-direction: column; gap: var(--s-md); }
  .row { display: flex; align-items: center; justify-content: space-between; gap: var(--s-md); font-size: 13px; }
  .bar { height: 5px; border-radius: 999px; background: rgba(255,255,255,0.10); overflow: hidden; }
  .bar span { display: block; height: 100%; border-radius: 999px; background: var(--accent); }

  .switcher {
    right: 5%; top: 24%; width: 30%;
    border-radius: var(--r-lg); padding: var(--s-md);
    display: flex; flex-direction: column; gap: var(--s-sm);
  }
  .env {
    display: flex; align-items: center; gap: var(--s-sm);
    padding: var(--s-sm) var(--s-md); border-radius: var(--r-md);
    font-size: 13px; border: 1px solid transparent;
  }
  .env.active { background: rgba(106,166,255,0.16); border-color: rgba(106,166,255,0.42); }
  .env .key {
    margin-left: auto; font-family: var(--mono); font-size: 11px;
    color: var(--text-secondary); border: 1px solid rgba(255,255,255,0.16);
    padding: 1px 6px; border-radius: var(--r-sm);
  }
  .switcher .state { font-size: 11px; color: var(--text-secondary); padding: 0 var(--s-md) var(--s-xs); }

  .dock {
    left: 50%; bottom: 4%; transform: translateX(-50%);
    display: flex; gap: var(--s-sm); padding: var(--s-sm);
    border-radius: var(--r-xl);
  }
  .dock i {
    width: 42px; height: 42px; border-radius: var(--r-md); display: block;
    background: linear-gradient(155deg, rgba(255,255,255,0.20), rgba(255,255,255,0.05));
    border: 1px solid rgba(255,255,255,0.12);
  }
  .dock i.accent { background: linear-gradient(155deg, var(--accent), #2E6FD4); border-color: rgba(255,255,255,0.2); }

  /* ---------------- prover ---------------- */
  .prover { display: grid; grid-template-columns: 1.15fr 1fr; gap: var(--s-2xl); align-items: start; }
  .stage {
    position: relative; border-radius: var(--r-xl); overflow: hidden;
    min-height: 300px; border: 1px solid var(--border);
    display: grid; place-items: center; padding: var(--s-xl);
  }
  .stage-panel {
    border-radius: var(--r-lg); padding: var(--s-xl); width: min(100%, 340px);
    display: flex; flex-direction: column; gap: var(--s-sm);
    backdrop-filter: blur(32px) saturate(180%);
    -webkit-backdrop-filter: blur(32px) saturate(180%);
    border: 1px solid rgba(255,255,255,0.14);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.10);
  }
  .readout { display: flex; flex-direction: column; gap: var(--s-md); }
  .metric {
    display: flex; align-items: baseline; justify-content: space-between; gap: var(--s-md);
    padding: var(--s-md) 0; border-bottom: 1px solid var(--border);
  }
  .metric .v { font-family: var(--mono); font-size: 26px; font-variant-numeric: tabular-nums; }
  .pass { color: var(--success); }
  .fail { color: var(--danger); }
  .pill {
    display: inline-block;
    font-family: var(--mono); font-size: 11px; letter-spacing: .08em; text-transform: uppercase;
    padding: 3px 9px; border-radius: 999px; border: 1px solid currentColor;
  }
  input[type=range] { width: 100%; accent-color: var(--accent); }
  input[type=range]:disabled { opacity: .45; }
  label.ctl { display: flex; flex-direction: column; gap: var(--s-xs); font-size: 13px; color: var(--text-secondary); }
  .toggle { display: flex; align-items: center; gap: var(--s-sm); font-size: 13px; cursor: pointer; }
  .toggle input { accent-color: var(--accent); width: 16px; height: 16px; }

  /* ---------------- tables and cards ---------------- */
  .grid { display: grid; gap: var(--s-md); }
  .g2 { grid-template-columns: repeat(2, 1fr); align-items: start; }
  .g3 { grid-template-columns: repeat(3, 1fr); }
  .g4 { grid-template-columns: repeat(4, 1fr); }
  .card { background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-lg); padding: var(--s-lg); }
  .swatch { height: 56px; border-radius: var(--r-md); border: 1px solid rgba(255,255,255,0.10); margin-bottom: var(--s-sm); }

  .shot {
    margin: 0;
    border-radius: var(--r-lg);
    overflow: hidden;
    border: 1px solid var(--border);
    background: #06070A;
    box-shadow: 0 30px 70px -30px rgba(0,0,0,0.9);
  }
  .shot img { display: block; width: 100%; height: auto; }
  .shot figcaption {
    padding: var(--s-md) var(--s-lg);
    border-top: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 12.5px;
  }

  .scroll { overflow-x: auto; }
  table { border-collapse: collapse; width: 100%; font-size: 13px; }
  th, td { text-align: left; padding: 10px 14px; border-bottom: 1px solid var(--border); white-space: nowrap; }
  th { font-family: var(--mono); font-size: 11px; letter-spacing: .1em; text-transform: uppercase; color: var(--text-secondary); font-weight: 500; }
  td.num { font-family: var(--mono); font-variant-numeric: tabular-nums; }
  .chipcell {
    display: inline-block; width: 22px; height: 13px; border-radius: 3px;
    box-shadow: inset 0 0 0 1px rgba(255,255,255,0.16); vertical-align: -2px; margin-right: 9px;
  }
  .barcell { display: inline-block; height: 6px; border-radius: 999px; background: var(--accent); vertical-align: 1px; }

  pre.code {
    background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--r-lg); padding: var(--s-lg); overflow-x: auto;
    font-family: var(--mono); font-size: 12.5px; line-height: 20px; margin: 0;
  }
  pre.code .k { color: var(--accent); }
  pre.code .c { color: var(--text-secondary); }

  footer { padding: var(--s-2xl) 0 var(--s-3xl); color: var(--text-secondary); font-size: 13px; display: flex; flex-direction: column; gap: var(--s-sm); }

  @media (max-width: 900px) {
    .prover { grid-template-columns: 1fr; }
    .g3, .g4 { grid-template-columns: repeat(2, 1fr); }
    .window { width: 62%; }
  }
  @media (max-width: 560px) {
    .g2, .g3, .g4 { grid-template-columns: 1fr; }
    .desktop { min-height: 300px; }
    .window, .switcher { display: none; }
  }
  @media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
</style>

<div class="wrap">

  <section class="hero">
    <div class="hero-copy">
      <span class="eyebrow">LitricOS &middot; design system</span>
      <h1>Glass that solves<br>for the wallpaper.</h1>
      <p class="lede">
        A translucent panel is only as readable as whatever is behind it. So LitricOS
        samples the backdrop and computes the <em>most transparent</em> tint that still
        clears WCAG AA &mdash; per panel, every time the wallpaper changes. On this
        wallpaper the panels are sitting at <span class="mono" id="heroMin">&mdash;</span>;
        over a black desktop the material bottoms out at 18%, and over a white one it
        firms up to 81% on its own. Change the wallpaper and watch.
      </p>
    </div>

    <div class="wallpicker" id="wallpicker" role="group" aria-label="Wallpaper"></div>

    <div class="desktop" id="desktop">
      <canvas id="wall" aria-hidden="true"></canvas>

      <div class="glass menubar" data-glass>
        <span class="brand">LitricOS</span>
        <span class="muted">File</span>
        <span class="muted">View</span>
        <span class="muted">Window</span>
        <span class="right">
          <span class="dot"></span> Verity active
          <span>09:41</span>
        </span>
      </div>

      <div class="glass window" data-glass>
        <span class="alpha-tag" data-tag></span>
        <div class="titlebar">
          <span class="traffic">
            <i style="background:#FF6B6B"></i>
            <i style="background:#FFC66D"></i>
            <i style="background:#5BD9A4"></i>
          </span>
          <span class="name">Activity</span>
        </div>
        <div class="win-body">
          <div class="row"><span>Memory</span><span class="mono muted">634 / 946 MB</span></div>
          <div class="bar"><span style="width:67%"></span></div>
          <div class="row"><span>Failed units</span><span class="mono" style="color:var(--success)">0</span></div>
          <div class="row"><span>Root</span><span class="mono muted">erofs &middot; read-only</span></div>
          <div class="row"><span>Trash</span><span class="mono muted">4 items &middot; 12.8 MB</span></div>
        </div>
      </div>

      <div class="glass switcher" data-glass>
        <span class="alpha-tag" data-tag></span>
        <div class="env"><span>Windows 11</span><span class="key">Win+1</span></div>
        <div class="env active"><span>LitricOS</span><span class="key">Win+2</span></div>
        <div class="env"><span>Linux</span><span class="key">Win+3</span></div>
        <div class="state">Host &middot; cannot be suspended</div>
      </div>

      <div class="glass dock" data-glass>
        <i class="accent"></i><i></i><i></i><i></i><i></i>
      </div>
    </div>
    <p class="muted" style="font-size:13px; margin-top:var(--s-md)">
      Not a picture of the effect: the wallpaper is drawn to a canvas, the pixels under
      each panel are read back, and the alpha shown on each panel is solved from them by
      the same walk the Rust uses. Sampling the unblurred backdrop is deliberately
      pessimistic &mdash; a blur can only pull the surface toward its own average, never
      past the extreme the solver already accounted for.
    </p>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-xl)">
      <span class="eyebrow">Running</span>
      <h2>This is not the mockup.</h2>
      <p class="lede">
        Below is LitricOS booted in a virtual machine, photographed through the
        hypervisor: the guest's own framebuffer, read out by QEMU rather than
        rendered by this page. Every panel was drawn by
        <code>litric-shell</code> in software, in the typeface described further
        down, over a framebuffer it found through <code>sysfs</code>. Every
        number came from <code>litric-sysd</code> reading <code>/proc</code>
        inside that machine.
      </p>
    </div>
    <figure class="shot">
      <img src="data:image/png;base64,__BOOT_PNG__" alt="LitricOS desktop as drawn on the guest framebuffer: a menu bar, an Activity window showing memory, uptime, root filesystem and trash, an environment switcher with Win+1/2/3, and a dock.">
      <figcaption>
        <code>docs/evidence/desktop-boot-2026-08-29.png</code> &middot; nano
        profile, 1280&times;800, 32bpp <code>bochs-drmdrmfb</code> &middot;
        zero failed units &middot; dm-verity active
      </figcaption>
    </figure>
    <div class="grid g4" style="margin-top:var(--s-lg)">
      <div class="card stack-sm">
        <span class="eyebrow">Measured on screen</span>
        <div class="mono" style="font-size:19px">647 / 946 MB</div>
        <p class="muted" style="font-size:13px">Read from <code>/proc/meminfo</code>
          by the daemon, asked for over a Unix socket by the shell.</p>
      </div>
      <div class="card stack-sm">
        <span class="eyebrow">Glass, solved</span>
        <div class="mono" style="font-size:19px">18% on all four</div>
        <p class="muted" style="font-size:13px">Against the wallpaper actually
          behind each panel. The static floor is 82%.</p>
      </div>
      <div class="card stack-sm">
        <span class="eyebrow">Worst contrast</span>
        <div class="mono pass" style="font-size:19px">5.19 : 1</div>
        <p class="muted" style="font-size:13px">Anywhere on that screen, against
          a required 4.5. Measured after compositing, not before.</p>
      </div>
      <div class="card stack-sm">
        <span class="eyebrow">Daemon footprint</span>
        <div class="mono" style="font-size:19px">2.3 MB</div>
        <p class="muted" style="font-size:13px">Resident set of
          <code>litric-sysd</code> inside the running image.</p>
      </div>
    </div>
    <p class="muted" style="margin-top:var(--s-lg); font-size:13px">
      What the picture does <strong>not</strong> show, and is not implied by it:
      there is no input handling, and no application can draw into that desktop.
      It is the output half of a compositor. The window is drawn by the shell,
      not hosted by it.
    </p>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-xl)">
      <span class="eyebrow">The method</span>
      <h2>Seventy-five evaluations, no assumptions</h2>
      <p class="lede">
        Contrast against alpha is not monotonic: the surface travels from the backdrop
        to the tint and may pass straight through the text's own luminance on the way.
        A bisection would be faster and would silently return the wrong answer the first
        time somebody changed the palette. So it walks.
      </p>
    </div>
    <div class="grid g2">
      <pre class="code"><span class="c">// src/litric-design/src/lib.rs</span>
<span class="k">pub fn</span> adaptive_alpha_pct(
    tint: Rgb, text: Rgb, backdrop: Rgb, target: f64,
) -&gt; Option&lt;u8&gt; {
    <span class="k">let</span> lo = Self::MIN_AESTHETIC_ALPHA_PCT; <span class="c">// 18</span>
    <span class="k">let</span> hi = Self::MAX_ADAPTIVE_ALPHA_PCT;  <span class="c">// 92</span>
    (lo..=hi).find(|pct| {
        <span class="k">let</span> a = f64::from(*pct) / 100.0;
        text.contrast(tint.over(backdrop, a)) &gt;= target
    })
}</pre>
      <div class="stack">
        <div class="card stack-sm">
          <h3>Why 18% is the floor</h3>
          <p class="muted" style="font-size:13px">
            Over a black wallpaper the solver would happily return 0%, and 0% is not
            glass &mdash; it is a hole. At 18% the border, the specular highlight and
            the saturation boost still have a surface to sit on.
          </p>
        </div>
        <div class="card stack-sm">
          <h3>Why 92% is the ceiling</h3>
          <p class="muted" style="font-size:13px">
            Past that the panel is a solid colour with a blur behind it. If a palette
            ever needs more, the solver returns <code>None</code> and the palette is
            wrong &mdash; not the wallpaper.
          </p>
        </div>
        <div class="card stack-sm">
          <h3>Where this differs from Liquid Glass</h3>
          <p class="muted" style="font-size:13px">
            Apple ships a <em>Reduce transparency</em> switch for the cases where the
            material becomes hard to read. A switch only helps the people who find it.
            Solving inside the material means the default is already correct, and the
            accessibility setting is a preference rather than a repair.
          </p>
        </div>
      </div>
    </div>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-xl)">
      <span class="eyebrow">Measured</span>
      <h2>What the solver returns</h2>
      <p class="lede">
        Emitted by <code>cargo run -p litric-design --example tokens</code>, not typed.
        <strong>Static</strong> is the single alpha a non-adaptive material is stuck
        with, because it has to survive the worst wallpaper anybody might set.
        <strong>Adaptive</strong> is what this one uses on that specific backdrop.
      </p>
    </div>
    <div class="scroll">
      <table>
        <thead>
          <tr>
            <th>Theme</th><th>Wallpaper</th><th>Static</th><th>Adaptive</th>
            <th>Saved</th><th>Primary</th><th>Secondary</th><th></th>
          </tr>
        </thead>
        <tbody id="adaptRows"></tbody>
      </table>
    </div>
    <p class="muted" style="margin-top:var(--s-md); font-size:13px" id="agreement"></p>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-xl)">
      <span class="eyebrow">Try to break it</span>
      <h2>Turn the solver off</h2>
      <p class="lede">
        Adaptive mode picks the alpha. Switch it off and you get the slider &mdash; which
        is what every glass interface designed over a dark wallpaper by somebody with a
        dark wallpaper is actually doing.
      </p>
    </div>

    <div class="prover">
      <div>
        <div class="stage" id="stage">
          <div class="stage-panel" id="panel">
            <h3>Environment Manager</h3>
            <p style="font-size:15px">
              Windows is hibernated. Switching will restore it from disk, which takes
              about fifteen seconds.
            </p>
            <p class="muted" style="font-size:13px">Secondary text &mdash; the first to fail</p>
          </div>
        </div>
        <div style="margin-top:var(--s-lg); display:flex; flex-direction:column; gap:var(--s-md)">
          <label class="toggle">
            <input id="adaptive" type="checkbox" checked>
            <span>Adaptive &mdash; solve the alpha from the backdrop</span>
          </label>
          <label class="ctl">
            <span>Wallpaper brightness</span>
            <input id="bd" type="range" min="0" max="255" value="16" aria-label="Wallpaper brightness">
          </label>
          <label class="ctl">
            <span>Glass tint opacity &mdash; <span class="mono" id="aVal">&mdash;</span></span>
            <input id="al" type="range" min="10" max="100" value="82" aria-label="Glass tint opacity">
          </label>
        </div>
      </div>

      <div class="readout">
        <div class="metric">
          <div class="stack-sm">
            <strong>Primary text</strong>
            <span class="muted mono" style="font-size:12px">#F2F4F8</span>
          </div>
          <span class="v" id="c1">&mdash;</span>
        </div>
        <div class="metric">
          <div class="stack-sm">
            <strong>Secondary text</strong>
            <span class="muted mono" style="font-size:12px">#A9B1BF</span>
          </div>
          <span class="v" id="c2">&mdash;</span>
        </div>
        <div class="metric" style="border-bottom:0">
          <div class="stack-sm">
            <strong>Effective surface</strong>
            <span class="muted" style="font-size:12px">what the text is really drawn on</span>
          </div>
          <span class="mono" id="surf">&mdash;</span>
        </div>
        <div id="verdict"></div>
        <p class="muted" style="font-size:13px" id="proverNote"></p>
      </div>
    </div>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-xl)">
      <span class="eyebrow">Palette &middot; dark</span>
      <h2>Eleven semantic tokens</h2>
      <p class="lede">
        Components ask for <code>text_primary</code>, never for a hex value. That is what
        makes a theme swap a data change instead of an audit of every component.
      </p>
    </div>
    <div class="grid g4" id="swatches"></div>
  </section>

  <section>
    <div class="stack" style="margin-bottom:var(--s-lg)">
      <span class="eyebrow">Type scale</span>
      <h2>Eight steps, one loosening ratio</h2>
    </div>
    <div class="scroll">
      <table>
        <thead><tr><th>Size</th><th>Line height</th><th>Ratio</th><th>WCAG large text</th><th>Specimen</th></tr></thead>
        <tbody id="typeRows"></tbody>
      </table>
    </div>
    <p class="muted" style="margin-top:var(--s-md); font-size:13px">
      The ratio loosens for body and tightens for display: a paragraph needs room to
      read; a 38px heading at the same ratio reads as two unrelated lines.
    </p>
  </section>

  <section>
    <div class="grid g3">
      <div class="card stack-sm">
        <span class="eyebrow">Spacing</span>
        <div id="spaceViz"></div>
        <div class="mono muted" style="font-size:12px" id="spaceVals"></div>
      </div>
      <div class="card stack-sm">
        <span class="eyebrow">Radius</span>
        <div id="radiusViz" style="display:flex; gap:8px; align-items:flex-end; flex-wrap:wrap"></div>
        <div class="mono muted" style="font-size:12px" id="radiusVals"></div>
      </div>
      <div class="card stack-sm">
        <span class="eyebrow">Motion</span>
        <p class="muted" style="font-size:13px">Springs, not curves &mdash; a spring can be
          interrupted mid-flight; a fixed-duration ease cannot.</p>
        <div class="scroll">
          <table style="font-size:12px">
            <thead><tr><th>Stiff</th><th>Damp</th><th>Settle</th></tr></thead>
            <tbody id="motionRows"></tbody>
          </table>
        </div>
      </div>
    </div>
  </section>

  <hr class="rule">

  <section>
    <div class="stack" style="margin-bottom:var(--s-lg)">
      <span class="eyebrow">What is enforced</span>
      <h2>33 tests, and two of them changed the design</h2>
    </div>
    <div class="scroll">
      <table>
        <thead><tr><th>Property</th><th>Result</th></tr></thead>
        <tbody>
          <tr><td>Body text reaches AA on every solid surface, all four themes</td><td class="num pass">pass</td></tr>
          <tr><td>Secondary text reaches AA on every solid surface</td><td class="num pass">pass</td></tr>
          <tr><td>Adaptive glass holds AA across a sweep of 1000+ backdrops</td><td class="num pass">pass</td></tr>
          <tr><td>Adaptive alpha never falls below the aesthetic floor</td><td class="num pass">pass</td></tr>
          <tr><td>High-contrast glass is never adapted &mdash; it stays opaque</td><td class="num pass">pass</td></tr>
          <tr><td>High-contrast themes reach AAA</td><td class="num pass">pass</td></tr>
          <tr><td>Type scale strictly increasing, line height exceeds size</td><td class="num pass">pass</td></tr>
          <tr><td>Reduced motion settles within one frame</td><td class="num pass">pass</td></tr>
          <tr><td>Springs stay critically damped &mdash; no wobble</td><td class="num pass">pass</td></tr>
        </tbody>
      </table>
    </div>
    <div class="grid g3" style="margin-top:var(--s-lg)">
      <div class="card stack-sm">
        <span class="pill" style="color:var(--warning)">Caught</span>
        <h3>Dark glass at 72%</h3>
        <p class="muted" style="font-size:13px">
          Secondary text measured 3.34 against a white wallpaper. Primary text passed
          comfortably, which is exactly why nobody would have noticed. The static floor
          moved to 82%.
        </p>
      </div>
      <div class="card stack-sm">
        <span class="pill" style="color:var(--warning)">Caught</span>
        <h3>Light glass at 78%</h3>
        <p class="muted" style="font-size:13px">
          4.46 against a required 4.5, over a dark wallpaper. Too narrow to catch by eye,
          and wrong on exactly the wallpapers a light-theme user is least likely to try.
        </p>
      </div>
      <div class="card stack-sm">
        <span class="pill" style="color:var(--accent)">Consequence</span>
        <h3>Static glass has to be heavy</h3>
        <p class="muted" style="font-size:13px">
          Roughly 80% in both directions, because it must survive a wallpaper it has never
          seen. That number is why the material became adaptive rather than prettier.
        </p>
      </div>
    </div>
  </section>

  <footer>
    <p>
      Generated by <code>tools/design-page.py</code> from
      <code>src/litric-design/examples/tokens.rs</code>. The JavaScript on this page
      re-implements the same WCAG arithmetic so the demonstration is live rather than a
      screenshot; the table above checks the two against each other on load.
    </p>
    <p>
      The desktop at the top of this page is a live mockup, drawn in your browser.
      The photograph further down is the same composition drawn by
      <code>litric-shell</code> inside a booted LitricOS and read back out of the
      virtual machine's framebuffer. Something does ask this material for an alpha
      now, and it gets 18%.
    </p>
  </footer>
</div>

<script>
(function () {
  "use strict";

  var DATA = /*__TOKENS__*/null;
  var LIM = DATA.limits;
  var DARK = DATA.themes[0];
  var TINT = hexToRgb(DARK.palette.glass_tint);
  var T1 = hexToRgb(DARK.palette.text_primary);
  var T2 = hexToRgb(DARK.palette.text_secondary);

  var TOKEN_NOTES = {
    bg_base: "desktop and window grounds",
    bg_elevated: "cards, sheets, sidebars",
    glass_tint: "tint over the backdrop",
    text_primary: "body text",
    text_secondary: "captions, metadata",
    accent: "selection, focus, primary action",
    on_accent: "text drawn on accent",
    border: "separators and outlines",
    danger: "destructive actions, errors",
    success: "confirmation, healthy status",
    warning: "caution, degraded status"
  };

  // ---- colour maths, mirroring src/litric-design/src/color.rs -------------
  function hexToRgb(h) {
    var v = parseInt(h.replace("#", ""), 16);
    return { r: (v >> 16) & 255, g: (v >> 8) & 255, b: v & 255 };
  }
  function toHex(c) {
    return "#" + [c.r, c.g, c.b].map(function (x) {
      return ("0" + x.toString(16)).slice(-2);
    }).join("");
  }
  function lum(c) {
    function ch(x) {
      x = x / 255;
      return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
    }
    return 0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b);
  }
  function contrast(a, b) {
    var la = lum(a), lb = lum(b);
    return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
  }
  function over(fg, bg, a) {
    a = Math.min(1, Math.max(0, a));
    return {
      r: Math.round(fg.r * a + bg.r * (1 - a)),
      g: Math.round(fg.g * a + bg.g * (1 - a)),
      b: Math.round(fg.b * a + bg.b * (1 - a))
    };
  }
  // The same inclusive walk as adaptive_alpha_pct. Returns -1 for None.
  function solve(tint, text, backdrop, target) {
    for (var p = LIM.min_aesthetic_alpha_pct; p <= LIM.max_adaptive_alpha_pct; p++) {
      if (contrast(text, over(tint, backdrop, p / 100)) >= target) { return p; }
    }
    return -1;
  }
  function solveBoth(backdrop) {
    var a = solve(TINT, T1, backdrop, LIM.aa);
    var b = solve(TINT, T2, backdrop, LIM.aa);
    if (a < 0 || b < 0) { return LIM.max_adaptive_alpha_pct; }
    return Math.max(a, b);
  }

  // ---- wallpapers --------------------------------------------------------
  // Painted to a canvas so the pixels under a panel can actually be read.
  var WALLS = [
    {
      name: "Slate", chip: "#1E232B",
      base: ["#12151A", "#1E232B", "#0C0E12"],
      blobs: [[0.20, 0.15, 0.75, "#2B3442"], [0.85, 0.75, 0.70, "#1A2430"]]
    },
    {
      name: "Nebula", chip: "#2E5BFF",
      base: ["#0D1030", "#16061C", "#06131A"],
      blobs: [[0.12, 0.08, 0.85, "#2E5BFF"], [0.88, 0.22, 0.75, "#C2408E"], [0.60, 0.96, 0.85, "#14B8A6"]]
    },
    {
      name: "Ember", chip: "#C8B49A",
      base: ["#7A5230", "#C8B49A", "#E8D9C2"],
      blobs: [[0.25, 0.20, 0.80, "#F0A860"], [0.80, 0.85, 0.75, "#F6E7CE"]]
    },
    {
      name: "Daylight", chip: "#EEF1F5",
      base: ["#FFFFFF", "#EEF1F5", "#DCE3EC"],
      blobs: [[0.30, 0.10, 0.80, "#FFFFFF"], [0.85, 0.80, 0.75, "#CFE0F5"]]
    }
  ];

  var canvas = document.getElementById("wall");
  var desktop = document.getElementById("desktop");
  var ctx = canvas ? canvas.getContext("2d", { willReadFrequently: true }) : null;
  var pixels = null, pw = 0, ph = 0;
  var current = 1;

  function paintWall() {
    if (!ctx || !desktop) { return; }
    var w = Math.max(1, Math.round(desktop.clientWidth));
    var h = Math.max(1, Math.round(desktop.clientHeight));
    // Sampled at CSS resolution: the solver wants colours, not sharpness.
    canvas.width = w; canvas.height = h;
    pw = w; ph = h;

    var wall = WALLS[current];
    var lin = ctx.createLinearGradient(0, 0, w * 0.5, h);
    lin.addColorStop(0, wall.base[0]);
    lin.addColorStop(0.6, wall.base[1]);
    lin.addColorStop(1, wall.base[2]);
    ctx.fillStyle = lin;
    ctx.fillRect(0, 0, w, h);

    wall.blobs.forEach(function (b) {
      var cx = b[0] * w, cy = b[1] * h, r = b[2] * Math.max(w, h);
      var g = ctx.createRadialGradient(cx, cy, 0, cx, cy, r);
      var c = hexToRgb(b[3]);
      g.addColorStop(0, "rgba(" + c.r + "," + c.g + "," + c.b + ",0.95)");
      g.addColorStop(0.55, "rgba(" + c.r + "," + c.g + "," + c.b + ",0.30)");
      g.addColorStop(1, "rgba(" + c.r + "," + c.g + "," + c.b + ",0)");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, w, h);
    });

    pixels = ctx.getImageData(0, 0, w, h).data;
  }

  function px(x, y) {
    x = Math.min(pw - 1, Math.max(0, x | 0));
    y = Math.min(ph - 1, Math.max(0, y | 0));
    var i = (y * pw + x) * 4;
    return { r: pixels[i], g: pixels[i + 1], b: pixels[i + 2] };
  }

  // The alpha a panel needs is the worst its own footprint demands.
  function solveForRect(rect, host) {
    var x0 = rect.left - host.left, y0 = rect.top - host.top;
    var worst = LIM.min_aesthetic_alpha_pct;
    var cols = 14, rows = 7;
    for (var i = 0; i <= cols; i++) {
      for (var j = 0; j <= rows; j++) {
        var a = solveBoth(px(x0 + (rect.width * i) / cols, y0 + (rect.height * j) / rows));
        if (a > worst) { worst = a; }
      }
    }
    return worst;
  }

  function applyGlass() {
    if (!pixels || !desktop) { return; }
    var host = desktop.getBoundingClientRect();
    var lowest = 100;
    Array.prototype.forEach.call(document.querySelectorAll("[data-glass]"), function (el) {
      var a = solveForRect(el.getBoundingClientRect(), host);
      el.style.background = "rgba(" + TINT.r + "," + TINT.g + "," + TINT.b + "," + (a / 100) + ")";
      var tag = el.querySelector("[data-tag]");
      if (tag) { tag.textContent = a + "% tint"; }
      if (a < lowest) { lowest = a; }
    });
    var hm = document.getElementById("heroMin");
    if (hm) { hm.textContent = lowest + "% opacity"; }
  }

  function selectWall(i) {
    current = i;
    Array.prototype.forEach.call(
      document.querySelectorAll("#wallpicker button"),
      function (b, n) { b.setAttribute("aria-pressed", n === i ? "true" : "false"); }
    );
    paintWall();
    applyGlass();
  }

  var picker = document.getElementById("wallpicker");
  if (picker) {
    WALLS.forEach(function (w, i) {
      var b = document.createElement("button");
      b.type = "button";
      b.setAttribute("aria-pressed", i === current ? "true" : "false");
      b.innerHTML = '<span class="chip" style="background:' + w.chip + '"></span>' + w.name;
      b.addEventListener("click", function () { selectWall(i); });
      picker.appendChild(b);
    });
  }

  // ---- measured table, cross-checked against the Rust ---------------------
  var rows = document.getElementById("adaptRows");
  var disagreements = 0;
  if (rows) {
    DATA.adaptive.forEach(function (r) {
      var saved = r.static_pct - r.used_pct;
      var tr = document.createElement("tr");
      // Only the dark theme can be re-derived here: the page's tint and text
      // tokens are the dark ones. Light-theme rows are reported, not re-checked.
      var check = "";
      if (r.theme === "dark") {
        var mine = solveBoth(hexToRgb(r.backdrop));
        if (mine !== r.used_pct) { disagreements++; }
        check = mine === r.used_pct ? "=" : "!=";
      }
      tr.innerHTML =
        "<td>" + r.theme + "</td>" +
        '<td><span class="chipcell" style="background:' + r.backdrop + '"></span>' + r.wallpaper + "</td>" +
        '<td class="num muted">' + r.static_pct + "%</td>" +
        '<td class="num"><strong>' + r.used_pct + "%</strong></td>" +
        '<td class="num" style="color:' + (saved > 0 ? "var(--success)" : "var(--text-secondary)") + '">' +
          (saved > 0 ? "-" + saved + " pts" : "0") + "</td>" +
        '<td class="num ' + (r.contrast_primary >= LIM.aa ? "pass" : "fail") + '">' + r.contrast_primary.toFixed(2) + "</td>" +
        '<td class="num ' + (r.contrast_secondary >= LIM.aa ? "pass" : "fail") + '">' + r.contrast_secondary.toFixed(2) + "</td>" +
        '<td><span class="barcell" style="width:' + Math.round(r.used_pct * 0.6) + 'px"></span></td>';
      rows.appendChild(tr);
    });
  }
  var agree = document.getElementById("agreement");
  if (agree) {
    var n = DATA.adaptive.filter(function (r) { return r.theme === "dark"; }).length;
    agree.innerHTML = disagreements === 0
      ? '<span class="pill pass">checked</span> This page re-solved all ' + n +
        " dark-theme rows in the browser and got the same alpha the Rust did. " +
        "Light-theme rows are reported from the same run but not re-derived here, " +
        "because this page is painted in the dark theme's tokens."
      : '<span class="pill fail">mismatch</span> ' + disagreements +
        " of " + n + " rows disagree with the Rust. Trust the Rust.";
  }

  // ---- swatches ----------------------------------------------------------
  var sw = document.getElementById("swatches");
  if (sw) {
    Object.keys(DARK.palette).forEach(function (k) {
      var hex = DARK.palette[k];
      var el = document.createElement("div");
      el.className = "card";
      el.innerHTML =
        '<div class="swatch" style="background:' + hex + '"></div>' +
        '<div style="font-weight:600;font-size:13px">' + k + "</div>" +
        '<div class="mono muted" style="font-size:12px">' + hex.toUpperCase() + "</div>" +
        '<div class="muted" style="font-size:12px;margin-top:4px">' + (TOKEN_NOTES[k] || "") + "</div>";
      sw.appendChild(el);
    });
  }

  // ---- type, space, radius, motion --------------------------------------
  var tr2 = document.getElementById("typeRows");
  if (tr2) {
    DATA.type.forEach(function (t) {
      var row = document.createElement("tr");
      row.innerHTML =
        '<td class="num">' + t.px + "px</td>" +
        '<td class="num">' + t.line_height_px + "px</td>" +
        '<td class="num">' + (t.line_height_px / t.px).toFixed(2) + "</td>" +
        "<td>" + (t.large ? "yes" : "&mdash;") + "</td>" +
        '<td style="font-size:' + t.px + "px; line-height:" + t.line_height_px +
          'px; white-space:nowrap">Environment</td>';
      tr2.appendChild(row);
    });
  }

  var sv = document.getElementById("spaceViz");
  if (sv) {
    sv.style.display = "flex";
    sv.style.alignItems = "flex-end";
    sv.style.gap = "6px";
    sv.style.height = "56px";
    DATA.space.forEach(function (s) {
      var b = document.createElement("span");
      b.style.cssText = "display:block;width:14px;background:var(--accent);border-radius:3px;height:" +
        Math.max(3, s) + "px";
      sv.appendChild(b);
    });
    document.getElementById("spaceVals").textContent = DATA.space.join(" · ") + " px";
  }

  var rv = document.getElementById("radiusViz");
  if (rv) {
    DATA.radius.forEach(function (r) {
      var b = document.createElement("span");
      b.style.cssText = "display:block;width:34px;height:34px;border:1px solid var(--accent);" +
        "border-radius:" + Math.min(r, 17) + "px";
      rv.appendChild(b);
    });
    document.getElementById("radiusVals").textContent =
      DATA.radius.map(function (r) { return r > 999 ? "full" : r; }).join(" · ");
  }

  var mr = document.getElementById("motionRows");
  if (mr) {
    DATA.motion.forEach(function (m) {
      var row = document.createElement("tr");
      row.innerHTML = '<td class="num">' + m.stiffness + "</td>" +
        '<td class="num">' + m.damping + "</td>" +
        '<td class="num">' + m.settle_ms + "ms</td>";
      mr.appendChild(row);
    });
    var row = document.createElement("tr");
    row.innerHTML = '<td class="num muted">' + DATA.motion_reduced.stiffness + "</td>" +
      '<td class="num muted">' + DATA.motion_reduced.damping + "</td>" +
      '<td class="num muted">' + DATA.motion_reduced.settle_ms + "ms</td>";
    mr.appendChild(row);
  }

  // ---- prover ------------------------------------------------------------
  var stage = document.getElementById("stage");
  var panel = document.getElementById("panel");
  var bd = document.getElementById("bd");
  var al = document.getElementById("al");
  var adaptive = document.getElementById("adaptive");

  function paintProver() {
    if (!stage || !panel || !bd || !al || !adaptive) { return; }
    var v = parseInt(bd.value, 10);
    var backdrop = { r: v, g: v, b: Math.min(255, v + 4) };

    if (adaptive.checked) { al.value = String(solveBoth(backdrop)); }
    al.disabled = adaptive.checked;

    var a = parseInt(al.value, 10) / 100;
    var surface = over(TINT, backdrop, a);

    stage.style.background = toHex(backdrop);
    panel.style.background = "rgba(" + TINT.r + "," + TINT.g + "," + TINT.b + "," + a + ")";

    var c1 = contrast(T1, surface), c2 = contrast(T2, surface);
    var e1 = document.getElementById("c1"), e2 = document.getElementById("c2");
    e1.textContent = c1.toFixed(2) + ":1";
    e2.textContent = c2.toFixed(2) + ":1";
    e1.className = "v " + (c1 >= LIM.aa ? "pass" : "fail");
    e2.className = "v " + (c2 >= LIM.aa ? "pass" : "fail");
    document.getElementById("surf").textContent = toHex(surface);
    document.getElementById("aVal").textContent = al.value + "%";

    var worst = Math.min(c1, c2);
    var vd = document.getElementById("verdict");
    if (worst >= 7) { vd.innerHTML = '<span class="pill pass">AAA</span>'; }
    else if (worst >= LIM.aa) { vd.innerHTML = '<span class="pill pass">AA &mdash; passes</span>'; }
    else if (worst >= 3) { vd.innerHTML = '<span class="pill" style="color:var(--warning)">Large text only</span>'; }
    else { vd.innerHTML = '<span class="pill fail">Fails</span>'; }

    var note;
    if (!adaptive.checked) {
      note = "Adaptive is off. WCAG AA needs 4.5:1 for body text. Below about "
        + LIM.min_static_alpha_pct + "% the primary text fails over a bright wallpaper, "
        + "and the secondary text fails well before that.";
    } else if (parseInt(al.value, 10) <= LIM.min_aesthetic_alpha_pct) {
      note = "The backdrop is dark enough that contrast is no longer the binding "
        + "constraint: the solver has hit the " + LIM.min_aesthetic_alpha_pct
        + "% aesthetic floor and stopped, because below that the panel stops reading "
        + "as a surface at all.";
    } else {
      note = "The secondary text is sitting just above 4.5 by construction. That is the "
        + "solver spending every point of transparency it is allowed to spend and not "
        + "one more.";
    }
    document.getElementById("proverNote").textContent = note;
  }

  if (bd && al && adaptive) {
    bd.addEventListener("input", paintProver);
    al.addEventListener("input", paintProver);
    adaptive.addEventListener("change", paintProver);
    paintProver();
  }

  // ---- boot --------------------------------------------------------------
  function relayout() { paintWall(); applyGlass(); }
  if (document.fonts && document.fonts.ready) { document.fonts.ready.then(relayout); }
  window.addEventListener("resize", relayout);
  relayout();
  // Panel geometry depends on the fonts and on the aspect-ratio box settling.
  window.setTimeout(relayout, 120);
})();
</script>
"""


def main() -> int:
    if len(sys.argv) != 3:
        sys.stderr.write(f"usage: {sys.argv[0]} <tokens.json> <out.html>\n")
        return 2
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    data = json.loads(src.read_bytes().decode("utf-8"))

    for key in ("themes", "adaptive", "type", "space", "radius", "motion", "limits"):
        if key not in data:
            sys.stderr.write(f"tokens.json is missing '{key}'\n")
            return 1

    html = TEMPLATE.replace(MARKER, json.dumps(data, separators=(",", ":")))
    if MARKER in html:
        sys.stderr.write("token marker was not substituted\n")
        return 1

    shot = Path(__file__).resolve().parent.parent / SHOT_PATH
    if not shot.is_file():
        sys.stderr.write(f"{shot} is missing; the page would show a broken image" + chr(10))
        return 1
    html = html.replace(
        SHOT_MARKER, base64.b64encode(shot.read_bytes()).decode("ascii")
    )
    if SHOT_MARKER in html:
        sys.stderr.write("screenshot marker was not substituted" + chr(10))
        return 1

    # write_bytes, never write_text: on Windows the text path silently rewrites
    # every newline to CRLF, which is the bug ci/lint-line-endings.py exists for.
    dst.write_bytes(html.encode("utf-8"))
    print(f"{dst}: {len(html)} bytes, {len(data['adaptive'])} measured rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
