# Wayland reference application fixture

`weston-simple-shm` is the Weston project's reference `wl_shm` client. It is a
real, third-party Linux application: C, `libwayland-client`, `xdg-shell`, and
`wl_shm`. It connects to a compositor, creates a toplevel, and draws animated
concentric rings into a shared-memory buffer. It is not a LitricOS test double.

The binary and its one non-glibc shared library are vendored so the S82
install/launch/window e2e is reproducible without a package manager or network
access. They are test fixtures only and are not installed into the image.

## Exact provenance

Downloaded from the Ubuntu 24.04 (noble) archive on 2026-10-31:

| File | Package | SHA-256 |
|---|---|---|
| `weston-simple-shm` | `weston_13.0.0-4build3_amd64.deb` | `bebef1213abf4919fda5b126cf89cc5c0faf39f31ab95ceeb7f872868e7550b1` |
| `wayland-libs/libwayland-client.so.0` | `libwayland-client0_1.22.0-2.1build1_amd64.deb` | `1c22abb422953442aa12822ea51ac76d1c8be92ed143e0159e0f830776cde2b2` |

Package URLs:

- <https://archive.ubuntu.com/ubuntu/pool/universe/w/weston/weston_13.0.0-4build3_amd64.deb>
- <https://archive.ubuntu.com/ubuntu/pool/main/w/wayland/libwayland-client0_1.22.0-2.1build1_amd64.deb>

`weston-simple-shm` was extracted unchanged from the first package. The library
is the real `libwayland-client.so.0.22.0`, renamed to the soname the loader asks
for. The licenses are in `weston-copyright` and
`libwayland-client-copyright`; both projects are MIT-licensed.

## Why the library is separate

The Ubuntu binary is dynamically linked against
`libwayland-client.so.0`; this development container has no system copy. The
test points the child's loader at `wayland-libs/` through `LD_LIBRARY_PATH`.
That is fixture setup, not part of LitricOS: the installed executable and the
desktop entry are produced by the real `APP INSTALL` path.
