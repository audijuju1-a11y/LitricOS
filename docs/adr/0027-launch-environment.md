# ADR-0027 — Launch environment for installed applications

- **Status:** Accepted
- **Date:** 2026-10-31
- **Drives:** S82, the real Linux application install/launch/window proof.

---

## Context

`APP INSTALL` made a validated bundle runnable and wrote a desktop entry whose
`Exec=` names the installed executable. The shell's launcher discovered that
entry and sent `LAUNCH <hex argv>` to `litric-sysd`, which spawned it. The
window bridge in `litric-compositor` and `litric-shell::clients` was already
real: a client that connects to `wayland-0` gets a `litric-wm` window, and its
`wl_shm` frames are painted by `views::draw`.

The missing link was environmental. The daemon is not inside the graphical
session, so its own environment has no `WAYLAND_DISPLAY` or
`XDG_RUNTIME_DIR`. A graphical application launched by the daemon therefore
started without a display and exited before mapping. The Terminal path had
worked because the shell set those variables itself; the installed-application
path did not.

## Decision

**Add one additive protocol request, `LAUNCHENV`, that carries the session's
display environment beside the argv.**

```text
LAUNCHENV <hex argv>... -- <hex NAME=VALUE>...
```

The literal `--` is a separator. It is not hex, so it cannot be confused with
an argument. Existing `LAUNCH` requests are unchanged and still accepted.

The daemon validates every entry before spawning:

- at most `MAX_ENV` (8) entries, each at most `MAX_ENV_VALUE_LEN` (512) bytes;
- no NUL bytes;
- names are exactly `WAYLAND_DISPLAY` or `XDG_RUNTIME_DIR`;
- `WAYLAND_DISPLAY` is one name, not a path; `XDG_RUNTIME_DIR` is absolute;
- duplicate names are refused.

The names are applied with `Command::env`; the rest of the daemon environment
(locale, `PATH`, loader configuration) is inherited as before. The child is
still detached and reaped by the existing thread.

The shell builds the request from `Shell::application_env()`, the same source
the Terminal uses. It sends `LAUNCH` instead when the shell is not hosting a
compositor, so a non-graphical session does not pretend to have a display.

## Why not arbitrary environment variables

`LD_PRELOAD`, `GCONV_PATH` and their relatives change which code runs, not
where a display is. The local shell already holds `ProcessSpawn` and can name
any absolute executable, but widening the environment makes the launch request
a second, less visible code-injection surface. The two display variables are
the complete requirement this path has, and the test proves they are enough
for a real Weston client.

## What this does not change

- No sandbox, namespace, cgroup or portal is added. The application runs as the
  daemon's user, with the daemon's inherited environment plus these two values.
- No `.desktop` parsing rule changes. `Exec=` is still a split, shell-free
  argument vector discovered by the existing code.
- No package manager, signature verification, download path or AppImage
  extractor is added.
- The compositor still supports `wl_shm` only; dmabuf, XWayland, clipboard and
  IME remain absent as ADR-0024 states.

## Evidence

`src/litric-shell/src/e2e_app_install.rs` installs the Ubuntu Weston 13
`weston-simple-shm` through the real Unix socket, launches it through the
shell's launcher, observes its `xdg_toplevel` as a `litric.client` window,
asserts its ring pixels through `views::draw`, closes the window, observes
`Gone`, and uninstalls it. Provenance and hashes for the fixture are in
`src/litric-shell/tests/fixtures/README.md`.

## Revisit if

- An application needs a third session variable (for example
  `XDG_SESSION_TYPE`): add it to the allowlist with a test, rather than
  accepting arbitrary names.
- Peer credentials make the launch caller distinguishable: the environment
  policy can then be tied to that identity instead of the local-session
  constant.
