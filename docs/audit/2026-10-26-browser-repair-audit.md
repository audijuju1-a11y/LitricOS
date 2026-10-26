# Litric Browser — repair audit, 2026-10-26

Scope: the fourteen questions in the browser-repair brief, answered against
`HEAD` (branch `freebuff/71c43910455d319c26d9858f`, parent `abf70e2`), plus two
defects found and fixed during the audit.

The brief was written against an older tree. Most of what it lists as broken is
already implemented at this commit, and the most useful thing this document can
do is say *which* parts are real code with tests, which parts were fixed in this
session, and which parts were **not** verified here because this machine cannot
verify them. A claim of "fixed" that nobody can reproduce is the failure mode
this project's gates exist to prevent.

## 1. What the machine could and could not do

| Capability | Available here | Consequence |
|---|---|---|
| Rust toolchain 1.99.1 + clippy + rustfmt | yes (installed during the previous phase) | every crate builds and tests |
| `cargo build -p litric-browser --features broker` | yes | the broker binary is proven to exist |
| Chromium | **no** (`/usr/bin/chromium` absent) | no live CDP session |
| `mkosi`, `pacstrap` | **no** | no image build, no unit-install proof |
| QEMU | **no** | no boot, no framebuffer, no input E2E |

Everything below is therefore `UNIT` / `CONTRACT` / `STATIC` evidence unless it
says otherwise. No row in this document is `VM E2E`.

## 2. The fourteen questions

| # | Question | Answer at this HEAD | Evidence |
|---|---|---|---|
| 1 | Is `litric-browserd` built? | Yes, and only with the broker feature | `src/litric-browser/Cargo.toml` `[[bin]] required-features = ["broker"]`; `cargo build --release --locked -p litric-browser --features broker` exits 0 |
| 2 | Is `broker` enabled correctly? | Yes, by `tools/build.sh` | the script's second `cargo build` names the package and the feature explicitly |
| 3 | Is the binary staged into the image? | Yes, and the build fails if it is not | `tools/build.sh` stages `$CARGO_TARGET_DIR/release/*` and `die`s for `desktop`/`gaming` when `litric-browserd` is missing |
| 4 | Can the build pass with the binary absent? | **It could, through the unit linter** — fixed today | §3.1 |
| 5 | Is Chromium in the desktop profile? | Declared, and the build refuses a desktop build that drops it | `image/mkosi.profiles/desktop.conf`; the `grep -qE '^[[:space:]]*chromium'` guard in `tools/build.sh` |
| 6 | Is the socket unit enabled? | Enabled by `mkosi.postinst.chroot`; **not verified in an image here** | no mkosi on this host |
| 7 | Does the socket path match? | Yes | unit `ListenStream=/run/litric/browser.sock`; shell `BROWSER_SOCKET` in `shell.rs`; test `browser_session_uses_the_socket_the_unit_declares` |
| 8 | Does the LBP/1 protocol match on both sides? | Yes, for every command the shell sends | `browserd.rs` `handle_line`; shell tests asserting the exact bytes sent (`GO 1 https%3A…`, `MOUSE 1 down …`) |
| 9 | Does Chromium start? | **Not verified** — no Chromium on this host | spawn path is `--headless=new --remote-debugging-pipe`, fd 3/4 wired in `cdp_fd`; sandbox left ON, never `--no-sandbox` |
| 10 | Does the CDP pipe work? | **Not verified** — needs a Chromium | code + JSON decoder tests only |
| 11 | Navigation, screenshot, JPEG decode, frame transfer | unit-tested pieces, not end-to-end | `litric-browser` 85 tests + 7 broker tests, incl. `jpeg` and `base64` suites |
| 12 | Does the shell display the frame? | Yes, with a length check and aspect fit | `accept_frame` rejects a length that is not `w*h*4`; `blit_engine_frame` preserves aspect and converts BGRA |
| 13 | Input to Chromium | Yes: `MOUSE`, `WHEEL`, `KEY`, `TEXT` implemented and unit-tested | `Input.dispatchMouseEvent` / `dispatchKeyEvent` / `insertText` |
| 14 | Root cause of "the browser does not work" | **Two distinct causes, both real:** the feature-gated binary was invisible to the static checker, and the chosen search engine was never saved. Neither is a Chromium bug. | §3 |

## 3. Defects found and fixed in this session

### 3.1 A feature-gated binary passed the "unit starts a real binary" gate

`ci/lint-unit-binaries.py` decided a unit was satisfiable when *some* `[[bin]]`
in some member manifest carried the name. `litric-browserd` has
`required-features = ["broker"]`, so a plain `cargo build --release` does not
produce it — exactly the state that once produced an image with an enabled
socket and no broker behind it.

The checker now reads `required-features` from each `[[bin]]` block, parses the
`cargo build` lines in `tools/build.sh` (package + `--features`, including
line continuations and `--all-features`), and fails a unit whose binary needs a
feature no documented build turns on.

Negative control, run on the real tree: deleting the broker line from
`tools/build.sh` makes the checker fail with

```
image/mkosi.extra/usr/lib/systemd/system/litric-browser@.service:8:
/usr/bin/litric-browserd -- src/litric-browser/Cargo.toml [[bin]] requires
feature(s) broker, and no cargo build in tools/build.sh enables them for
litric-browser -- the unit would start a binary that the documented build
never produces
```

and restoring the line makes it pass again. Three new self-test cases cover
"no build line", "the right feature", and "a different feature".

### 3.2 The search-engine choice did not survive a restart

Both routes to the setting — the start page's engine chips and the browser
context menu — assigned `BrowserApp::engine` and returned no effect. Nothing
wrote it, and nothing read it at startup, so DuckDuckGo became Google again at
the next launch.

The engine is a preference, so it now lives in `settings.conf` beside the accent
and the wallpaper, under a stable token (`google`, `duckduckgo`, `bing`) rather
than the translated title:

* `SearchEngine::token()` / `SearchEngine::parse()`;
* `Preferences::search_engine`, parsed and encoded by `prefstore`;
* `Shell::load_preferences` / `save_preferences` keep the field and the live
  model in step, so a failed write rolls the engine back with the rest of the
  record;
* `Shell::set_search_engine` returns `Effect::PersistPreferences`.

The test `choosing_a_search_engine_persists_it_and_a_restart_applies_it` drives
the menu, completes the write, drives the chip, and then loads the saved record
into a fresh shell.

## 4. What the brief lists as missing but is already present at HEAD

Named explicitly, because "fixed" and "already worked" are different claims and
a future reader deserves to know which one they are looking at:

* `tools/build.sh` builds the broker with its feature and refuses a desktop
  image without it;
* `litric-browser.socket` / `litric-browser@.service` exist, are enabled, run as
  `litric`, keep Chromium's sandbox on, and open exactly `/run/litric-browser`;
* the broker finds Chromium through `BROWSERD_CHROMIUM` then the usual paths,
  and reports a missing engine as `ERR engine-unavailable` instead of "loading"
  forever;
* HTTPS-only policy, `file:` refusal and the URL parser are in
  `src/litric-browser/src/policy.rs` and `url.rs`;
* history and bookmarks are written atomically to
  `/var/lib/litric-browser/browser-data.conf`, inside the unit's
  `StateDirectory=litric-browser`;
* the Files app's storage is `/var/lib/litric-files/root`, inside
  `StateDirectory=litric-files`, created by `fsbridge::ensure_storage`;
* the lock screen asks the daemon whether a password can actually be checked,
  refuses to lock when it cannot, and never unlocks on a timeout or a malformed
  reply.

## 5. Not verified, with the command that would verify it

| Claim | Why not here | Command on a builder |
|---|---|---|
| Chromium starts under the unit's sandbox | no Chromium, no systemd user manager | `./tools/build.sh --profile desktop && ./tools/run-qemu.sh` |
| A real page is captured and displayed | needs the booted image | `python3 tools/browser-e2e.py` against that boot |
| The socket activates on first connect | needs systemd in the image | `systemctl status litric-browser.socket` in the guest |
| Downloads land in the broker's directory | needs a running download | `BROWSERD_SOCK=… litric-browserd` + a test page |

Until those run, the honest status of the browser is: **implemented and
unit-tested, not booted.**

## 6. Commands run in this session

```
cargo build --release --locked -p litric-browser --features broker   # ok
cargo test   -p litric-browser --features broker                     # 85 + 7 passed
cargo test   -p litric-shell                                         # 463 + 32 + … passed
cargo clippy -p litric-shell --all-targets                           # clean
cargo fmt --all -- --check                                           # clean
python3 ci/lint-unit-binaries.py --self-test                         # 7 passed
python3 ci/lint-unit-binaries.py                                     # 16 refs, 0 broken
```
