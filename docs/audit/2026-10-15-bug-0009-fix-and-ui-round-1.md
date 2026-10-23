# 2026-10-15 — BUG-0009 + UI round 1: what landed, and the evidence

**Scope:** the confirmed open interoperability bug (BUG-0009, the trash `Path=`
field) fixed across the daemon and the shell in one branch, with regression tests
and legacy-entry recovery; the UI round-1 patch applied and verified; the tree put
under version control for the first time.
**Mode:** real compilation and real test execution on the Windows development
host, plus a Linux type-check pass for the Unix-only code. Nothing was booted, no
image built, no VM run.
**Result vocabulary:** PASS (executed and succeeded) · FAIL (executed and found a
defect) · SKIP (deliberately not run, with a reason) · NOT VERIFIED (nothing that
ran can establish the claim).

---

## สรุปผู้บริหาร (Thai executive summary)

**คำถามแรก "พอจะใช้ได้ไหม" — ได้** ต้นไม้ทั้ง workspace คอมไพล์ผ่าน และเทสต์ผ่าน
ทั้งหมด **1193 passed / 0 failed** (ก่อนแก้ 1181) โดย**ไม่มีการลดทอนเทสต์ใด ๆ**

**บั๊กที่ยืนยันว่ายังเปิดอยู่ แก้แล้วจริง** — `Path=` ใน `.trashinfo` เดิมเขียนเป็น
URI `file://…` ซึ่งตามสเปก Freedesktop Trash 1.0 ถือว่าเป็น *relative path* ทำให้
เครื่องมืออื่น (`gio trash`, KDE, file manager อื่น) คืนไฟล์ผิดตำแหน่ง รอบนี้แก้
**ทั้ง daemon และ shell พร้อมกันในคอมมิตเดียว** ตามที่เอกสารเดิมระบุ พร้อม
**รักษาการกู้คืนรายการเก่าที่เขียนในรูปแบบ URI ไว้** (มีเทสต์ยืนยันทั้งสองทาง)

**ต้นเหตุเชิงโครงสร้างที่ต้องพูด** — รูปแบบไฟล์นี้เคยมี "เจ้าของ" ถึง **3 ที่**
(daemon, shell ที่ก็อปตัวเข้ารหัสไปเอง, และ `litric_trash::TrashedItem::trashinfo`
ที่ปล่อย storage URI ออกมาในชื่อที่อ้างว่าเป็นไฟล์สเปก) นั่นคือเหตุผลที่บั๊กเดียวกัน
ต้องถูกแก้ซ้ำสองรอบ รอบนี้จึงย้ายรูปแบบไปไว้ที่เดียว: `litric-files::trashinfo`
ซึ่ง**ไม่ผูกกับ Unix** ทำให้เทสต์รูปแบบรันได้บนเครื่อง Windows นี้จริง (12 เทสต์ใหม่)

**งาน UI (round 1)** patch ที่ส่งมา apply สะอาด คอมไพล์ผ่าน clippy สะอาด

**ความซื่อสัตย์** — เทสต์กลุ่ม `#[cfg(unix)]` (วงจร put→list→restore เต็มรูปแบบ)
**ยังไม่ได้รัน** เพราะเครื่องนี้ไม่มี Linux runtime (ไม่มี WSL/Docker) แต่
**คอมไพล์และตรวจชนิดครบแล้ว** ด้วย target `x86_64-unknown-linux-gnu`

**git** — เดิม checkout นี้ไม่มี version control เลย รอบนี้ `git init` แล้ว commit
baseline, UI และ fix แยกสาขา พร้อมสาขา integration

---

## 1. Was it usable? — yes, and here is the number

| Check | Command | Result |
|---|---|---|
| Whole workspace compiles | `cargo check --workspace` | **PASS**, exit 0 |
| Whole workspace tests | `cargo test --workspace` (GNU toolchain) | **PASS**, 1193 passed / 0 failed |
| Whole workspace tests, before any change | same, on pristine `master` | **PASS**, 1181 passed / 0 failed |
| Unix-only code type-checks | `cargo check --workspace --all-targets --target x86_64-unknown-linux-gnu` | **PASS**, exit 0 |
| Workspace lints on both targets | `cargo clippy --workspace --all-targets` (+ `--target …-linux-gnu`) | **PASS**, 0 diagnostics |
| Project CI subset | `bash ci/verify.sh` | **22 pass / 2 fail / 6 skip** — identical to pristine `master` |

The two `ci/verify.sh` failures are **not** from this work and are present on the
untouched baseline: the generated development signing keys have never been created
in this checkout (`tools/gen-dev-keys.sh`), so `image/mkosi.conf`'s `SecureBootKey=`
and `pcr.key` references and the key-pair stage both fail. Raw logs, including the
pristine-master run for comparison, are in `docs/evidence/2026-10-15/`.

## 2. Environment findings that shaped the work

None of these are code defects; each one silently changes what "verified" means, so
they are recorded.

1. **The Rust toolchain is installed but not on `PATH`.** `rustc`/`cargo` 1.99.0
   live in the user's `~/.cargo/bin`. A bare `cargo` in this shell is "command not
   found", which is easy to mistake for "no toolchain".
2. **`cargo test` fails to *link*** with the default `x86_64-pc-windows-msvc`
   toolchain: there is no MSVC linker on this machine, and Git-for-Windows'
   `/usr/bin/link.exe` (a coreutils `link`, not Microsoft's) is what resolves
   first. The error is `link: missing operand after '\377\376'`. `cargo check`
   works (it does not link), which is why "it compiles" and "the tests run" were
   different answers here.
3. **`stable-x86_64-pc-windows-gnu` *is* installed**, and tests run correctly
   under it (`cargo +stable-x86_64-pc-windows-gnu test`). This is what produced
   the 1193-pass figure.
4. **No Unix runtime exists here** — no WSL, no Docker, no Podman, no Linux target
   toolchain. So `#[cfg(unix)]` code is *invisible* to a Windows build: it is
   neither compiled nor type-checked. That is precisely how a bug like BUG-0009 can
   survive in a green tree.
5. **Fix: `rustup target add x86_64-unknown-linux-gnu`** (network was available).
   `cargo check --all-targets --target x86_64-unknown-linux-gnu` then compiles the
   Unix-only modules *and their tests*. It cannot run them, but it turns "the
   `cfg(unix)` half is unread" into "the `cfg(unix)` half is type-checked". This
   immediately caught a real mistake made in this session (a field declaration
   dropped while rewriting a doc comment).
6. **`ci/verify.sh` prefers `python3`**, which on this host resolves to the
   WindowsApps *stub*, not Python — so 19 stages failed with "Python was not
   found" on the first run. With a `python3` shim pointing at the real interpreter
   the same script reports 22/2/6. The failures were environmental, and the
   comparison against pristine `master` is what proves it.

## 3. BUG-0009 — the trash `Path=` field

Full record: `docs/bugs/BUG-0009-trash-path-field-is-not-a-location.md`, now marked
FIXED. Summary:

**What was wrong.** Both writers stored `Path=file%3A///Documents/report.odt`.
Decoded that is `file:///Documents/report.odt`, which starts with `f`, so a
conforming reader treats it as a **relative** pathname and restores the item under
the trash directory's parent. The specification allows no relative spelling of a
home-trash item here — the only available one would contain `..`, which the same
clause forbids — so the absolute host pathname is the only correct value. Litric's
own round trip worked because both of its readers stripped the `file://` first.

**How it was fixed.**

| Site | Change |
|---|---|
| `src/litric-files/src/trashinfo.rs` | **NEW.** Owns the format: `record()`, `pct_encode`/`pct_decode`, `strip_root()`, `logical_location()`, `is_not_a_uri()`. Deliberately not `#[cfg(unix)]`. |
| `src/litric-files/src/trash.rs` | `DiskTrash` keeps its `host_root`; `put` writes `record(&host, …)` and reports the logical path; `read_one` decodes and maps back; the private `pct_path`/`unpct_path` are gone. |
| `src/litric-files/src/server.rs` | `restore` parses `item.original_path` directly; the `file://` strip and its `corrupt-archive` branch are gone. |
| `src/litric-shell/src/fsbridge.rs` | Same two changes; its private encoder copies are gone. |
| `src/litric-trash/src/lib.rs` | `TrashedItem::trashinfo` removed — it was unused and emitted `Path=user://…`, a third instance of the same defect. |
| `src/litric-files/src/lib.rs` | Registers `trashinfo` and adds it to the module map. |

**Three design points worth naming.**

* *Legacy recovery is first-class.* `logical_location` understands the current
  absolute form **and** the legacy `file://` form, so an entry an older LitricOS
  wrote still lists with its real location and still restores. That is tested, not
  asserted.
* *A refused restore beats a guessed one.* A `Path=` the root cannot claim is passed
  through verbatim, so `LogicalPath::parse` rejects it as `invalid-name`. The
  alternative — inventing a destination — is the bug being fixed.
* *`strip_root` matches on a path boundary.* `/var/home/litric2/x` is not under
  `/var/home/litric`, though its text begins with the root's. A string-prefix check
  would have handed the caller a logical path of `/2/x` and restored into the wrong
  directory — a fresh bug in the fix for the old one.

**Regression tests.**

| Location | Tests | Executed here? |
|---|---|---|
| `src/litric-files/src/trashinfo.rs` | 12: the body's exact bytes; `Path=` absolute and never a URI; root-is-`/`; trailing slash on the root; sibling text-prefix; the root itself; unknown pass-through; encode/decode inverses | **PASS, ran here** |
| `src/litric-files/src/trash.rs` | on-disk record is the absolute location; legacy URI still lists+restores; outside-root is listed but refused by the parser | type-checked only |
| `src/litric-files/src/server.rs` | `TRASHLIST` reports the logical original and never the word `file`; legacy entry restores | type-checked only |
| `src/litric-shell/src/fsbridge.rs` | shell record is the absolute location and the shared reader maps it back; legacy record still lists with the right `restore_dir` and restores | type-checked only |

`the_record_never_writes_a_uri` is the guard that would have failed before the fix:
the old value `file%3A///Documents/x` contains `file` and is not the host path.

## 4. UI round 1

The supplied patch (`0001-UI-round-1-lock-liquid-glass-and-desktop-B-dock-matc.patch`)
applied to `src/litric-shell/src/lock.rs`, `src/litric-shell/src/views.rs` and
`src/litric-shell/examples/frame_dump.rs` with `git apply --check` clean, and is
committed unmodified on `ui/round-1-lock-dock`. It compiles with `--all-targets`
(so the example builds) and clippy is clean. Its own commit message carries the
change list; nothing about it was altered.

It is **not** a regression risk for the trash work: the two touch disjoint files.

## 5. Verification log

Every row below was executed in this session. Logs are in
`docs/evidence/2026-10-15/`.

| # | Command | Why | Result |
|---|---|---|---|
| 1 | `cargo check --workspace` | does the tree build at all? | **PASS**, exit 0 |
| 2 | `cargo test --workspace` (default MSVC) | run the tests | **FAIL** — link error, no MSVC linker; see §2.2 |
| 3 | `cargo +stable-x86_64-pc-windows-gnu test --workspace` (pristine `master`) | baseline number | **PASS**, 1181 / 0 |
| 4 | `git apply --check <the UI patch>` | does it apply? | **PASS** |
| 5 | `cargo check --workspace --all-targets` (after the patch) | does it compile + the example | **PASS** |
| 6 | `cargo clippy --workspace --all-targets` (after the patch) | workspace lints | **PASS**, 0 diagnostics |
| 7 | `rustup target add x86_64-unknown-linux-gnu` | make the Unix half visible | **PASS** |
| 8 | `cargo check --workspace --all-targets --target x86_64-unknown-linux-gnu` | type-check the `cfg(unix)` writers, readers and their tests | **PASS**, exit 0 — and it caught a dropped struct field mid-session |
| 9 | `cargo clippy …--target x86_64-unknown-linux-gnu` | same, with lints | **PASS**, 0 diagnostics |
| 10 | `cargo test --workspace` (GNU) after the fix | did anything break? | **PASS**, 1193 / 0 (1181 + 12 new) |
| 11 | `bash ci/verify.sh` with a real `python3` on `PATH` | project CI subset, branch | **22 pass / 2 fail / 6 skip** |
| 12 | `bash ci/verify.sh`, same, on a **pristine `master` worktree** | are those 2 failures mine? | **22 pass / 2 fail / 6 skip** — identical, so they are pre-existing (missing dev keys) |
| 13 | `python ci/lint-doc-refs.py` · `lint-file-safety.py` · `check-python-syntax.py` | documentation citations and deletion-site review still hold after the edits | **PASS** (448 citations across 117 documents resolve) |
| 14 | `cargo check` / `cargo clippy` / `cargo test` on `fix/bug-0009-trashinfo-path` alone | the branch the request asked for | **PASS**, 1193 / 0, 0 diagnostics |
| 15 | same three on `integration/round-1` (UI + fix together) | the delivered state | **PASS**, 1193 / 0, 0 diagnostics |

Side effects: `rustup target add` downloaded the Linux `rust-std`; a `python3` shim
was written under `build/` (git-ignored); `src/target` was deleted once to clear a
corrupt incremental-compilation artifact that made a warning appear in a crate
(`litric-storage`) this work never touched.

## 6. What is NOT verified

| Claim | Status | Why |
|---|---|---|
| The `#[cfg(unix)]` end-to-end trash tests pass | **NOT RUN** | no Unix runtime on this host; type-checked only |
| `gio trash --list` shows the original path on a booted image | **NOT RUN** | needs QEMU + an image; the ADR's interoperability acceptance |
| A KDE/`gio` restore of a Litric-trashed file lands in the right place | **NOT RUN** | same |
| The UI renders correctly | **NOT RUN** | needs a booted image or the frame-dump tool on Linux |
| `cargo fmt --check` | **SKIP** | `rustfmt` component not installed (workspace's own CI stage reports SKIP for the same reason) |
| Image build, boot, RAM/CPU/boot-time budgets | **SKIP** | `mkosi`/`qemu` absent; those stages report SKIP with a reason |
| Development signing keys | **FAIL (pre-existing)** | never generated in this checkout; same on pristine `master` |

## 7. Decisions and rejected alternatives

* **The format lives in one module, not three.** Rejected: fix the four I/O sites in
  place and leave the duplicated encoders. That is the arrangement that produced
  this bug, and the previous session's own record says half-fixing it would be worse
  than not fixing it.
* **The format module is host-independent.** Rejected: keep it in `trash.rs` beside
  its only-writer. That keeps the whole format unverifiable on the development host,
  which is the structural reason it was wrong for a year.
* **`litric_trash::TrashedItem::trashinfo` was removed, not corrected.** Rejected:
  parameterise it so it can emit a correct record. A second formatter — even a
  correct one — re-creates the drift. The ledger decides what a delete *means*; the
  layer with a root writes the file.
* **Unknown `Path=` values are refused, not guessed.** Rejected: treat an
  unrecognised value as root-relative (the shell's old `unwrap_or_else("/")`
  default). That silently restores items into `/`.
* **Legacy URIs are read forever, and never written.** Rejected: migrate entries on
  read. A read that rewrites a user's trash is a read with a side effect.

## 8. แนวคิด / ideas for the next round

1. **Run the Unix tests somewhere real, in CI.** The cheapest version: a Linux job
   that runs `cargo test --workspace`. Everything needed is written; only a host is
   missing. Without it, the `cfg(unix)` half of this repository is permanently
   "compiled but never executed".
2. **Make the invisible half visible by default.** Add a CI stage that runs
   `cargo check --all-targets --target x86_64-unknown-linux-gnu` on the Windows job.
   It is seconds, it needs no Linux host, and it is the difference between §2.4 and
   §2.5 — this session's own dropped field was caught only because of it.
3. **One trash collision-name rule.** The daemon names a clash `name.2.ext`; the
   shell names it `name - Copy.ext`. Both are unique, so nothing is broken, but the
   two writers of one directory disagreeing on names is the same drift in a
   different suit. Now that the *format* has one owner, the *naming* is the last
   duplicated decision.
4. **A trash-interop test that does not need Litric.** Write a `.trashinfo` with the
   freedesktop spec open, then read it back with a tiny independent reader in the
   test — the point being that the reader is written from the specification, not
   from the writer. That is what would have caught this bug without a booted image.
5. **`directorysizes` and `Trash/files` on other mounts.** The spec's per-top-directory
   trash (`.Trash` on removable volumes) is not implemented; the home trash is all
   there is. Worth a decision record rather than a silent gap.
6. **Branch hygiene for release.** This session created `master`, `ui/round-1-lock-dock`,
   `fix/bug-0009-trashinfo-path` and `integration/round-1`. Before anything ships,
   decide which of those is the line of record and tag it — four branches with no
   remote is a state that is easy to lose.

## 9. git: what was done, and what push needs

The checkout had **no version control at all** before this session
(`git rev-parse --is-inside-work-tree` was fatal). It now has:

```
f028797  master                        Baseline as received
bf32f98  ui/round-1-lock-dock          UI round 1 (the supplied patch, unmodified)
8c0dfe4  fix/bug-0009-trashinfo-path   the BUG-0009 fix + regression tests
08d1766  integration/round-1           merge of the UI branch
e9f1bce  integration/round-1           merge of the fix branch   <-- checked out
```

`integration/round-1` is the state every verification row above describes.

**Push is not possible from here**: the repository has **no remote configured**, and
none existed to inherit. Adding one and pushing publishes this work, so it is left
to the owner of the repository:

```bash
git remote add origin <the URL>
git push -u origin master integration/round-1 ui/round-1-lock-dock fix/bug-0009-trashinfo-path
```

All commits are local and complete; nothing is waiting on a network step.
