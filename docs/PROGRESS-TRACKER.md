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
| Remaining "Reused" citation list in Part 23 | **OPEN** | One false attribution was removed (`S0930`); the rest of the list has not been read line by line |
| Installer/image build and QEMU boot | **NOT RUN** | The archive ships the build tooling, but this workspace has no image and the task was scoped to the codebase this turn |

## 5. Next verified step

Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`
on every change; the new workflow does this on push. The next measurement that
would move the tracker is a QEMU boot with `tools/measure-ui.py` and
`tools/measure-idle.py`, because every performance row above is blocked on the
same missing image.
