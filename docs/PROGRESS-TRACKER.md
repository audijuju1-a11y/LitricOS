# LitricOS progress tracker

Living record of verified engineering batches. A line is only here if the
command that produced it ran in this workspace; unrun work is listed under
**Open**, never as done.

- Repository: `audijuju1-a11y/LitricOS`
- Work branch: `freebuff/d7991b8c41a570fd94f0a9bc` (tracks `origin/main`)
- Rust workspace root: `src/` (23 crates, toolchain 1.99.1)
- Date of this entry: 2026-10-28

## 1. Repository identity check

The mission asked for a separate "Deep-research-all" GitHub repository. The
authenticated account (`audijuju1-a11y`) has no such repository: the full
repository list contains LitricOS, hyper, Hypercheck, Basehider_Pro, In-Ear-Pro,
sentinel-shield, audisurround-license-server, AudiSurround-Releases,
audismp-shop and Basehider. The LitricOS research corpus lives inside this
repository at `Research-Deep-all/` (Parts 1–32, coverage checklist and source
register). Public search finds three unrelated repositories with similar names
(`hades905/deep-all-research`, `we11as22/all_included_deep_research`, one
unrelated thesis project); none is this project. Work therefore continued in
`Research-Deep-all/` rather than guessing a target.

## 2. Batch: frame-loop hardening and CI enforcement (verified)

| Change | Evidence |
|---|---|
| `crumb_window` no longer indexes a slice in the frame loop; `skip` is `usize` with `saturating_add` | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 |
| Breadcrumb "last item" arithmetic no longer overflows on a future edit | same clippy run |
| Dead `filled` flag removed from the scale-banner button closure (both branches were `Material::Popover`, so the parameter could only mislead) | same clippy run; visual material unchanged |
| GitHub Actions workflow runs fmt, clippy (`-D warnings`), workspace tests and the research validators | `.github/workflows/ci.yml` (new) |
| Part 23 no longer attributes a Windows-forum article (`S0930`) to registry documentation | Part 32 §3 updated; citation removed rather than repointed |

### Commands and results

```text
$ cargo fmt --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
TESTS passed=1634 failed=0
```

## 3. Benchmark corpus (verified, not a performance claim)

`litric-bench` drives the real `litric-defender` engine; it is a security
benchmark, not a frame-time benchmark.

```text
$ cargo run -q -p litric-bench -- generate --output /tmp/lb --count 32 --seed 7
generated 32 samples (76135 bytes) in /tmp/lb/corpus
exit 0

$ cargo run -q -p litric-bench -- validate --output /tmp/lb
determinism (seed 928374): OK
digest agreement: OK
corpus: 1000 samples, 980 hostile, 20 benign
exit 0
```

The 1,000-sample corpus is generated deterministically by `validate`; the
`--count 32` generation is a separate, smaller artifact. Both are correctness
evidence, not throughput.

## 4. Open, with the reason

| Item | State | Blocker |
|---|---|---|
| Frame time, RAM, startup time, GPU use | **NOT MEASURED** | No display session or booted image in this workspace; timing a headless frame loop would not represent the desktop |
| `VP-01`–`VP-27` device halves | **PARTIAL / NOT RUN** | Need Windows 26H2, macOS, a Linux desktop session or a mkosi image |
| Chapter L rows observed live | **0 / 112** | Same; the corpus remains documentation-derived until `VP-23` runs |
| Remaining "Reused" citation list in Part 23 | **DONE** | Every entry checked in this batch; decisions in §6 |
| Installer/image build and QEMU boot | **NOT RUN** | The archive ships the build tooling, but this workspace has no image and the task was scoped to the codebase this turn |

## 5. Next verified step

Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`
on every change; the new workflow does this on push. The next measurement that
would move the tracker is a QEMU boot with `tools/measure-ui.py` and
`tools/measure-idle.py`, because every performance row above is blocked on the
same missing image.

## 6. Batch: Part 23 citation audit and the trash-name guard (verified)

### Citation decisions, one per entry

| Citation | Decision | Reason |
|---|---|---|
| `S0941` (`defaults`) | **removed** | Windows Central "14 defaults worth changing" is Windows 11 settings, not the macOS `defaults` store. The macOS group is now `S3079` + `S0112b`, and the body's "3 inspected sources" was corrected to 2 |
| `S3079` | kept | macOS 27 `defaults write` report; supports the macOS store claim |
| `S0112b` | kept | macOS defaults/window-appearance issue; supports the same claim |
| `S1740` | kept, label fixed | Syncfusion WinUI compact sizing; "family" implied more rows than exist (one) |
| `S3716` | kept, label fixed | GTK 4 `Gtk.Entry`; one row, not a family |
| `S3042` | kept, label fixed | WWDC26 AppKit session; one row, not a family |
| `S0130` / `S0131` | kept, label fixed | Both are Microsoft Support notification/DND pages, not API references; the label now says settings and DND |
| `S3852` | kept | XDG Desktop Portal documentation; supports the portal claim |
| `S3845`–`S3849` | kept | Chapter H sources: default apps, Windows Backup, recovery, Time Machine, Windows Update |
| `S0930` in the I.03 table | **replaced** | The table still cited the Windows-forum search article as "Registry". It now cites `S3893` (Microsoft Learn, *Registry - Win32 apps*), opened this turn, and the claim was narrowed to what that page actually states: a system-defined database read and written through the registry API |

The register gained one inspected source: `S3893`, 4,208 rows, last id `S3893`,
1,303 inspected, 382 domains.

### Code defect: an empty trash name addressed the whole trash

`DiskTrash::purge` and `restore` validated `/` and a leading `.` but not the
empty string. `files_dir.join("")` is `files/` itself, so `purge("")` reached
`remove_dir_all(files/)` and `restore("")` would have moved the entire trash
directory to the destination. `get("")` happened to return `None`, which hid the
hole from the server path but not from the public API.

| File | Change |
|---|---|
| `src/litric-files/src/trash.rs` | one `valid_name` rule (non-empty, no `/`, no leading `.`) used by `get`, `restore` and `purge`; regression test asserts the entry and `files/` survive three empty-name calls |
| `src/litric-files/src/server.rs` | `RESTORE` and `PURGE` reject an empty decoded field with `bad-request` before reaching the disk layer |

```text
$ cargo fmt --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo test --workspace
TESTS passed=1635 failed=0
```

### Research validators

```text
$ python3 ci/check-python-syntax.py
OK: 60 python file(s) parse (no bytecode written)          exit 0
$ python3 ci/lint-line-endings.py
OK: 351 file(s) checked, all LF                            exit 0
$ python3 ci/lint-doc-refs.py
OK: 748 citation(s) across 151 document(s) resolve        exit 0
$ python3 tools/validate_chapter45.py --check
8 checks: 2 PASS, 6 PARTIAL, 0 FAIL                       exit 0
$ python3 tools/fix_source_id_keys.py --check
no-op: 4208 rows, 4208 distinct ids, 0 duplicates; nothing written
```
