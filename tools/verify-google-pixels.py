#!/usr/bin/env python3
"""Prove the browser really renders Google's page — by pixels, not hope.

Boots the VM, opens the browser from the dock, types a query, commits it,
screendumps, and counts pixels in the frame that match Google's four brand
colors plus the share of near-white (a real page on our dark shell). A page
that only *changed* but did not render would fail the color test.
"""

import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import boot_qmp, click, key, move, type_text  # noqa: E402

GOOGLE = [(0x42, 0x85, 0xF4), (0xEA, 0x43, 0x35), (0xFB, 0xBC, 0x05), (0x34, 0xA8, 0x53)]


def read_ppm(path: str):
    with open(path, "rb") as f:
        data = f.read()
    # P6 header: magic, width, height, maxval, then binary RGB.
    parts = data.split(b"\n", 3)
    w, h = map(int, parts[1].split())
    return w, h, parts[3]


def main() -> None:
    out = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "build", "tour",
        time.strftime("%Y%m%d-%H%M%S") + "-gpage",
    )
    os.makedirs(out, exist_ok=True)
    proc, q, _serial, _diag = boot_qmp(out)

    def S(s: float) -> None:
        time.sleep(s)

    try:
        for t in (14, 15, 16):
            S(t)
        # Dock: rest on the bottom edge, wait for the slide, open Browser.
        move(q, 666, 780)
        S(1.0)
        click(q, 666, 776)
        S(2.0)
        # First boot auto-opens Environments + Activity, so the browser is
        # the first dock-opened window = overall window 3 = cascade(0):
        # (64+28*2, 76+28*2) = (120, 132). Omnibox field = body(132+34)
        # + strip(30) + 4 .. +22 -> center 211; x = 120 + 450.
        click(q, 120 + 450, 211)  # omnibox field center
        S(0.6)
        type_text(q, "rust programming")
        S(0.4)
        key(q, "ret")
        S(12.0)  # spawn + navigate + remote page paint

        ppm = os.path.join(out, "after-search.ppm")
        q.cmd("screendump", filename=ppm)
        S(0.6)
    finally:
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()
        q.close()

    serial = os.path.join(out, "serial.log")
    if os.path.exists(serial):
        for line in open(serial, encoding="utf-8", errors="replace"):
            if any(k in line for k in ("browser.status=", "browser.tab=", "browser.frame=", "net.probe=")):
                print("[serial]", line.strip()[:160])

    w, h, rgb = read_ppm(ppm)
    assert (w, h) == (1280, 800), (w, h)
    total = w * h
    hits = [0, 0, 0, 0]
    white = 0
    page_white = 0
    page_total = 0
    # Browser page area: cascade (120, 132), 900x600, chrome = 64px.
    px0, py0 = 120, 132 + 64
    for i in range(total):
        r, g, b = rgb[3 * i], rgb[3 * i + 1], rgb[3 * i + 2]
        for j, (cr, cg, cb) in enumerate(GOOGLE):
            if abs(r - cr) <= 24 and abs(g - cg) <= 24 and abs(b - cb) <= 24:
                hits[j] += 1
                break
        if r > 235 and g > 235 and b > 235:
            white += 1
            if px0 <= i % w < px0 + 900 and py0 <= i // w < py0 + 600 - 64:
                page_white += 1
        if px0 <= i % w < px0 + 900 and py0 <= i // w < py0 + 600 - 64:
            page_total += 1

    print(f"frame {w}x{h}: brand hits B={hits[0]} R={hits[1]} Y={hits[2]} G={hits[3]}")
    print(f"near-white: whole={100 * white / total:.1f}% page={100 * page_white / page_total:.1f}%")
    ok = sum(hits) > 400 and page_white / page_total > 0.35
    print("VERDICT:", "PASS — Google's page really rendered" if ok else "FAIL — no convincing page")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
