# ADR-0018 — Litric Browser renders with Chromium, driven over CDP by a socket-activated broker

- **Status:** Accepted — implementation in progress; verification state is recorded at the end and is the only part of this file that may say "works"
- **Date:** 2026-09-13
- **Extends:** ADR-0017 (framebuffer shell). **Amends** ADR-0009's budget note that the
  desktop profile contains no browser engine: from this ADR on, the `desktop` profile
  ships the Arch `chromium` package, and the `nano`/`core` profiles still do not.

---

## Context

`litric-browser` has a URL parser, a tab/history model and a privacy policy, and its
only engine was `NoEngine`, which refuses every load. `SYSTEM-SCOPE.md` §0 already
decided that the engine is integrated, never written. The shell draws everything
itself on `/dev/fb0`; when this was written there was no Wayland server and no
client surfaces. **Amended by ADR-0024:** the shell now hosts a Wayland server and
a foreign application's surface becomes a LitricOS window. The browser is
unchanged by that: it still renders through its broker, not into a client
surface of its own.

## Decision

1. **Engine:** the distribution's `chromium` binary, run `--headless=new` with its own
   sandbox, one browser process per shell connection.
2. **Control:** the Chrome DevTools Protocol over `--remote-debugging-pipe` (JSON
   messages terminated by NUL on fds 3 and 4). No TCP port is ever opened, so no local
   process can attach to the engine.
3. **Pixels:** `Page.startScreencast` (JPEG) for the active tab only. Frames arrive on
   damage, so an idle page costs nothing. The broker decodes JPEG → 32-bit pixels and
   sends raw frames to the shell.
4. **Broker:** `litric-browserd` (in the `litric-browser` crate). It is the only process
   that parses anything web-derived (CDP JSON, base64, JPEG). The shell receives a
   fixed-format raw frame whose size it checks before allocating.
5. **Activation:** `litric-browser.socket` (`ListenStream=/run/litric/browser.sock`,
   `SocketMode=0600`, owner root, `Accept=yes`, `MaxConnections=1`). systemd performs
   the access check and hands the connected socket to `litric-browser@.service` as
   stdin/stdout. No `unsafe`, no fd passing, and **zero RAM until the browser is opened**.
   The service runs as the unprivileged desktop user `litric`, because Chromium's
   sandbox refuses uid 0.

```text
 litric-shell (root, no caps, AF_UNIX only)
      │  connect /run/litric/browser.sock        LBP/1 (this file)
      ▼
 systemd socket (0600 root) ── Accept=yes ──► litric-browserd (user litric)
                                                 │ fd3 → CDP commands
                                                 │ fd4 ← CDP events/responses
                                                 ▼
                                   chromium --headless=new --remote-debugging-pipe
                                   (own sandbox: user namespaces + seccomp)
```

### Why not the alternatives

| Option | Rejected because |
|---|---|
| Write an engine | Millions of lines and a permanent security commitment (SYSTEM-SCOPE §0). |
| Chromium on Xvfb, read the XWD framebuffer, inject with XTEST | Adds an X server as a second display stack and attack surface; Chromium's own tab strip would duplicate ours; change detection needs polling or XDamage. |
| A Wayland server inside the shell for Chromium's Ozone/Wayland | Needs `SCM_RIGHTS` fd passing and `mmap` of shm buffers — `unsafe` or a C library in the shell — plus xdg-shell, seat and keymap protocols. Correct long-term direction (ADR-0005), far too large for this step. |
| CDP over `--remote-debugging-port` | A TCP listener any local process can attach to and drive. The pipe has no such exposure. |
| Screencast PNG | Chromium's PNG encoder for screencast frames is several times slower than JPEG at desktop sizes. |

## LBP/1 — the shell ↔ broker wire protocol

Transport: one stream socket. **Client → server**: UTF-8 text lines ending in `\n`,
at most 8192 bytes; fields separated by one space. Free-text fields are
percent-encoded (every byte outside `A–Z a–z 0–9 - . _ ~` is `%XX`), and `-` stands for
an empty/absent value. **Server → client**: header lines in the same form; a `FRAME`
header is followed immediately by its binary payload.

### Commands (client → server)

| Command | Meaning |
|---|---|
| `HELLO 1` | Must be first. Reply `HELLO 1 engine=<pct> version=<pct> sandbox=<on\|off>` or `ERR engine-unavailable <pct-msg>`. |
| `VIEWPORT <w> <h>` | Content size in pixels, 64–4096 each. Applies to every tab. |
| `NEW <tab> <url-pct\|->` | Open a tab under the client's id, optionally navigating (input goes through `url::interpret` and the policy). Replies with a `TAB` event. The client chooses the id, as it does for every other tab command: a broker-chosen id made the two tab lists disagree from the second tab onward. |
| `CLOSE <tab>` | Close a tab. Event `CLOSED <tab>`. |
| `ACTIVATE <tab>` | Stream frames for this tab only; the previous tab's screencast stops. A fresh frame is sent. |
| `GO <tab> <input-pct>` | Address-bar input: `url::interpret` → `Policy::check_navigation` → navigate. |
| `BACK <tab>` `FORWARD <tab>` `RELOAD <tab>` `STOP <tab>` | History and loading. `STOP` answers with a `TAB` line whose state is `stopped` unless the document's `readyState` was already `complete`, in which case nothing was interrupted and it says `loaded` — a stopped page is never reported as a finished one. |
| `MOUSE <tab> <move\|down\|up> <x> <y> <none\|left\|middle\|right> <clicks> <mods>` | Pointer, in content pixels. |
| `WHEEL <tab> <x> <y> <dx> <dy>` | Scroll, in pixels. |
| `KEY <tab> <down\|up> <key-pct> <code-pct> <vk> <mods> <text-pct\|->` | DOM `key`, DOM `code`, Windows virtual-key code, modifiers, produced text. |
| `TEXT <tab> <text-pct>` | Insert text (paste, IME, non-Latin input). |
| `DOWNLOADS` | List finished files in the staging directory: `DOWNLOADS <n>` then `DOWNLOAD <name-pct> <bytes>` per file, or `ERR downloads-are-not-configured`. Names ending `.crdownload` are Chromium's partial writes and are not listed. |
| `DROP <name-pct>` | Remove one staged file after the shell has imported it. The name is one segment; a symlink or a directory is refused. Reply `DROPPED <name-pct>`. |
| `PING` | Reply `PONG`. |
| `QUIT` | Close the connection; the broker stops Chromium and exits. |

`<mods>` is the CDP bitmask: Alt 1, Ctrl 2, Meta 4, Shift 8. `<tab>` is a decimal id,
never reused within a connection. At most 16 tabs.

### Events (server → client)

| Event | Meaning |
|---|---|
| `TAB <tab> state=<empty\|loading\|loaded\|failed\|blocked\|stopped> url=<pct> title=<pct> back=<0\|1> fwd=<0\|1> secure=<0\|1>` | Full state of one tab after any change. |
| `CLOSED <tab>` | The tab is gone (closed by request, by the page, or by an engine crash). |
| `FRAME <tab> <w> <h> <n>` + `n` bytes | Raw pixels, `n == w*h*4`, each pixel a little-endian `u32` `0xAARRGGBB`. `w,h ≤ 4096`. |
| `NOTICE <tab\|-> <kind> <pct-msg>` | `dialog-dismissed`, `popup-opened`, `download-started`, `download-refused`, `blocked`, `engine-restarted`. |
| `ERR <kind> <pct-msg>` | `bad-request`, `no-such-tab`, `limit`, `too-long`, `engine-unavailable`, `engine-crashed`. |
| `PONG` | — |

Frames today are **screenshot-per-command**, not a screencast: `GO`, `RELOAD`,
`BACK`, `FORWARD` and `ACTIVATE` answer with one `TAB` line and one `FRAME`. The
shell asks for a fresh frame (`ACTIVATE`) after input, at most every 50 ms, and
`link::Framing::coalesce_key` keeps at most one undrawn frame per tab in the
queue — a fast page cannot grow the shell's memory. `Page.startScreencast` with
per-frame acknowledgement and back-pressure remains the planned Phase B; it is
**not** what this code does, and nothing in the interface claims otherwise.

## Policy enforcement — what the code actually does

| Claim | Enforced by | Scope |
|---|---|---|
| HTTPS only | `Broker::handle_go` parses with `litric_browser::url::Url` and refuses anything that is not `https:`. Plain `http:` is refused with the reason shown in the page area. | Every `GO`/`NEW` the shell sends. |
| No filesystem access | `file:` is refused before the engine is asked; the broker has no grant to the user's files (see the unit's `ProtectHome=yes`, `ProtectSystem=strict` and the single `ReadWritePaths`). | Every navigation. |
| No arbitrary schemes | Anything that is not `http`/`https`/`about:` is refused, so `javascript:`, `data:` and external protocols never reach the engine. | Every navigation. |
| TLS is real | Chromium's own verification runs with its own trust store; the broker passes no `--ignore-certificate-errors` and refuses to add one. | All HTTPS. |
| Downloads | `Browser.setDownloadBehavior` points at `--downloads`, which is a **staging** directory on the broker's tmpfs, not the user's Downloads folder. `DOWNLOADS` lists finished files there; the shell hands each one to `litric-filesd` (`IMPORT`, ADR-0019), and only after the daemon confirms the copy does it send `DROP` to the broker. Neither the broker nor the shell writes into `/Downloads`, and a failed import leaves the staged file to be retried rather than losing it. Without `--downloads` downloads are not configured and the shell says so. | Browser-wide. |
| Site permissions | Not surfaced. Headless Chromium has no prompt UI, so a page asking for camera, microphone or location gets nothing; LitricOS does not grant them on the user's behalf. | Browser-wide. |
| Remote debugging | `--remote-debugging-pipe` only. No TCP debugging port is ever opened. | Process-wide. |
| Engine sandbox | Chromium's sandbox is left ON. The unit runs the broker as `litric`, does not set `RestrictNamespaces`, and a Chromium that cannot start its sandbox is reported as an engine failure rather than silently downgraded with `--no-sandbox`. | Process-wide. |

Tracker blocking, a JavaScript switch, dialog dismissal and a permission prompt
UI are **not implemented**. The broker does not pretend they are: no command
claims to change them, and the site-controls panel says so on screen.

## Known limitations (true on the day this was written)

- `<select>` pop-ups and context menus are separate widgets in headless Chromium and
  do not appear in screencast frames. Keyboard selection still works.
- No cursor-shape feedback (CDP does not report it).
- Audio is muted (`--mute-audio`): LitricOS has no audio stack yet.
- One frame format (JPEG), so text is very slightly softer than a native renderer.
- File-upload dialogs are not implemented.

## Verification record

Filled in from executed evidence only. Anything not listed here is **NOT VERIFIED**.

**2026-10-25 — broker ⇄ Chromium ⇄ frames, executed (`tools/browser-e2e.py`).**
Chromium 156.0.8078.47 (Chrome for Testing) on Ubuntu 24.04, broker built with
`cargo build -p litric-browser --features broker`. Twelve stages passed:

```text
PASS start       broker listening on /tmp/litric-browser-e2e.sock
PASS handshake   HELLO 1 engine=chromium version=0.2.0 sandbox=on
PASS keepalive   PONG
PASS policy      file:, http: and unknown commands refused by name
PASS start page  TAB 1 state=empty url=about%3Ablank title=New%20Tab
PASS navigation  https://example.com/ title='Example Domain'
PASS rendering   1280x800 BGRA, 264 sampled colours, page.png
PASS link        clicked through to https://www.iana.org/help/example-domains
PASS scroll      the page moved in response to the wheel
PASS input       click and typed text were accepted
PASS tabs        opened, loaded and closed a second tab
PASS shutdown    the broker exited on QUIT
```

The click-through is the load-bearing line: a pointer event the test dispatched
at the link's own pixels navigated the tab, which means input, CDP dispatch and
the tab's address all agree. `tools/browser-e2e.py` writes `page.png`,
`scrolled.png` and `after-input.png` next to its log so the picture can be
looked at rather than inferred.

**Not verified by that run:** the image itself (no mkosi/QEMU on this host), the
systemd socket and `--stdio` instance path, and Chromium's sandbox inside the
signed image. Those need `tools/build.sh --profile desktop` and a boot on a
Linux host with mkosi and QEMU.
