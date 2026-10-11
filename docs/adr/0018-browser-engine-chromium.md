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
itself on `/dev/fb0`; there is no Wayland server and no client surfaces.

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
| `NEW <url-pct\|->` | Open a tab, optionally navigating (input goes through `url::interpret` and the policy). Replies with a `TAB` event. |
| `CLOSE <tab>` | Close a tab. Event `CLOSED <tab>`. |
| `ACTIVATE <tab>` | Stream frames for this tab only; the previous tab's screencast stops. A fresh frame is sent. |
| `GO <tab> <input-pct>` | Address-bar input: `url::interpret` → `Policy::check_navigation` → navigate. |
| `BACK <tab>` `FORWARD <tab>` `RELOAD <tab>` `STOP <tab>` | History and loading. |
| `MOUSE <tab> <move\|down\|up> <x> <y> <none\|left\|middle\|right> <clicks> <mods>` | Pointer, in content pixels. |
| `WHEEL <tab> <x> <y> <dx> <dy>` | Scroll, in pixels. |
| `KEY <tab> <down\|up> <key-pct> <code-pct> <vk> <mods> <text-pct\|->` | DOM `key`, DOM `code`, Windows virtual-key code, modifiers, produced text. |
| `TEXT <tab> <text-pct>` | Insert text (paste, IME, non-Latin input). |
| `PING` | Reply `PONG`. |
| `QUIT` | Close the connection; the broker stops Chromium and exits. |

`<mods>` is the CDP bitmask: Alt 1, Ctrl 2, Meta 4, Shift 8. `<tab>` is a decimal id,
never reused within a connection. At most 16 tabs.

### Events (server → client)

| Event | Meaning |
|---|---|
| `TAB <tab> state=<empty\|loading\|loaded\|failed\|blocked> url=<pct> title=<pct> back=<0\|1> fwd=<0\|1> secure=<0\|1>` | Full state of one tab after any change. |
| `CLOSED <tab>` | The tab is gone (closed by request, by the page, or by an engine crash). |
| `FRAME <tab> <w> <h> <n>` + `n` bytes | Raw pixels, `n == w*h*4`, each pixel a little-endian `u32` `0xAARRGGBB`. `w,h ≤ 4096`. |
| `NOTICE <tab\|-> <kind> <pct-msg>` | `dialog-dismissed`, `popup-opened`, `download-started`, `download-refused`, `blocked`, `engine-restarted`. |
| `ERR <kind> <pct-msg>` | `bad-request`, `no-such-tab`, `limit`, `too-long`, `engine-unavailable`, `engine-crashed`. |
| `PONG` | — |

Back-pressure: the broker acknowledges a screencast frame to Chromium only after the
frame has been written to the socket, so a slow shell slows the engine instead of
growing a queue.

## Policy enforcement — what each setting actually does

| `litric_browser::Policy` field | Enforced by | Scope |
|---|---|---|
| `https_only` | `check_navigation` refuses typed `http:` and offers the `https:` address. | Address bar only; in-page `http:` links are Chromium's default behaviour. |
| `block_trackers` | `Network.setBlockedURLs` with `*://host/*` and `*://*.host/*` per list entry, on every tab session. | All subresources and navigations. The built-in list is short and is **not** a comprehensive blocker. |
| `javascript` | `Emulation.setScriptExecutionDisabled`. | Per tab. |
| Site permissions (camera, microphone, location, notifications, clipboard read) | `Browser.setPermission … denied` at start-up. There is no prompt UI yet, so `Ask` behaves as `Deny`. | Browser-wide. |
| Downloads | `Browser.setDownloadBehavior`: `allow` into the configured Downloads directory, otherwise `deny` with a `NOTICE`. | Browser-wide. |
| JavaScript dialogs | Dismissed automatically with a `NOTICE`, so a page cannot wedge the tab. | Per tab. |

## Known limitations (true on the day this was written)

- `<select>` pop-ups and context menus are separate widgets in headless Chromium and
  do not appear in screencast frames. Keyboard selection still works.
- No cursor-shape feedback (CDP does not report it).
- Audio is muted (`--mute-audio`): LitricOS has no audio stack yet.
- One frame format (JPEG), so text is very slightly softer than a native renderer.
- File-upload dialogs are not implemented.

## Verification record

Filled in from executed evidence only. Anything not listed here is **NOT VERIFIED**.
