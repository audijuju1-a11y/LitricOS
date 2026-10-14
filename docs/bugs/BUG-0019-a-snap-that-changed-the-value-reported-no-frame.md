# BUG-0019 — A spring that snapped to its target reported "nothing moved"

```
Bug ID        : BUG-0019
Severity      : S3 (MEDIUM) — a contained rendering-correctness defect: the
                pixels a spring drives stay wrong until some unrelated repaint
Class         : UI / RENDERER (damage tracking)
Subsystem     : src/litric-shell/src/anim.rs (`Anim::step`),
                src/litric-shell/src/shell.rs (`Shell::tick`),
                caught by src/litric-shell/src/main.rs
                (`a_partial_redraw_is_pixel_identical_to_a_full_redraw`)
First Observed: 2026-10-14, the first run of the partial-redraw oracle after the
                frame-pacing work made the partial path reachable at all
Status        : FIXED (regression tests added)
```

## Reproducer

```bash
cd src
cargo test -p litric-shell --bin litric-shell a_partial_redraw_is_pixel_identical
```

## Actual

Before the fix, on the first step of the oracle ("pointer over the empty
desktop"):

```
the partial redraw differs from a full redraw in 18768 pixels,
within (29,59)-(388,358), worst channel difference 42
```

The frame that produced those pixels was a **partial** frame whose entire damage
consisted of two small rects, from `Shell::take_damage`:

```
full=false damage=[Rect { x: 477.0, y: 297.0, w: 20.0, h: 26.0 },
                   Rect { x: 477.0, y: 517.0, w: 20.0, h: 26.0 }]
```

So 18 768 pixels outside the damage region — the whole place where a window
sits, shadow included — ended up different from what a full redraw of the same
shell produced. A full redraw is idempotent (measured: `full-vs-full = 0`), so
the difference was not the renderer; it was that the partial frame had painted
the wrong thing and had not been told to repair the right thing.

## Root cause

`Anim::step` returned `false` on the step where the spring reached its settle
budget and **snapped** to its target:

```rust
if budget_hit || converged {
    self.snap_to(self.target);
    return false;              // <-- "no longer moving"
}
```

`Shell::tick` reads that `false` as the compositor's permission to idle:

```rust
if self.anims.step(dt) {
    self.request_full_redraw();
}
```

A snap is not necessarily small. `Anim::step` clamps the integration delta to
`MAX_STEP_MS` (100 ms) but counts `settle_ms` in **real** milliseconds, so one
step after a stall can reach the budget while the value is still far from its
target — and then the spring covers the whole remaining distance in that single
frame. On the frame this was measured, the focused window's focus-elevation
spring jumped the whole way (0 → 1, the weight of its shadow) and reported
"nothing moved", so no frame was requested and the window's shadow kept its old
value on screen.

The two questions — *did the value change* and *is it still going* — were
conflated in one `bool`. The redraw needs the first; the idle path needs the
second. See the module docs in `anim.rs`.

## Why the existing tests missed it

The oracle was written to catch exactly this class, and it was passing. It passed
**vacuously**: its fixture, `oracle_session`, ticks once and the test's clock then
starts 1 000 ms later, so the first real frame sees a 1 016 ms delta. Before the
frame-pacing work, `Shell::tick` stepped every spring by a constant 16 ms, so a
spring that had just been given a target was always still *moving* on that step —
`step` returned `true`, `Shell::tick` asked for a full redraw, and the oracle
compared a full frame against a full frame. The partial path it believed it was
testing was never entered.

Wall-time stepping is what changed that: the 1 016 ms delta hit the settle budget
immediately, the spring snapped in one step and reported nothing, and for the
first time the oracle compared a genuinely partial frame against a full one. The
defect is therefore older than the change that exposed it; the change removed the
accident that had been hiding it. That is the useful half of this record: a test
that cannot fail is not evidence, and this one could not fail for a reason
nobody had written down (compare BUG-0005, the same class of defect in the boot
smoke test).

## Fix

`Anim::step` now answers *both* questions honestly — it returns whether the value
moved **or** the spring is still moving:

```rust
if budget_hit || converged {
    self.snap_to(self.target);
    return self.value != before;
}
```

A snap that changed the value reports `true`, so `Shell::tick` requests the frame
that paints it. A snap that lands where the value already was reports `false`,
because then the pixels really are unchanged and idling is correct. `FocusAnims`
and `ShellAnims` document the same contract, since they aggregate it.

## Regression tests

* `anim::tests::a_budget_snap_that_moves_the_value_reports_the_move` — the exact
  defect: one 10 000 ms step, `value != before`, and the call must report it.
* `anim::tests::a_step_that_changes_nothing_reports_nothing` — the other half of
  the contract, which the idle path depends on.
* `anim::tests::reduced_motion_arrives_in_one_step_without_travel` — updated. It
  asserted `!a.step(DT)`, which is the same conflation, and now asserts that the
  spring *settled* on that step (the property it was named for) while reporting
  the change it did make.
* `main::tests::a_partial_redraw_is_pixel_identical_to_a_full_redraw` — the
  integration oracle that found it. After the fix every step of every journey
  reports `partial-vs-full = 0`.

## Verification

* `cargo test --offline --workspace` — PASS (see the 2026-10-14 entry in
  [docs/EVIDENCE.md](../EVIDENCE.md)).
* The oracle's own measurement, by instrumenting `assert_matches_full_redraw`:
  before the fix `full-vs-full = 0, partial-vs-full = 18768` on the first step;
  after, `partial-vs-full = 0` on every step of the journey. The idempotence
  check is why the root cause could be assigned to damage rather than to the
  renderer.

## Residual risk

The fix restores correctness, not cost. `Shell::tick` still asks for a **full**
redraw whenever any spring moves, so an animating frame is a full frame —
which is the shared root cause of BUG-0015 and BUG-0018, and is unchanged here.
That defect is about *which* pixels must be repainted; this one was that a
necessary repaint was not requested at all. Localising animation damage to the
animated surfaces is the next piece of work, and the oracle above is the test
that will police it.
