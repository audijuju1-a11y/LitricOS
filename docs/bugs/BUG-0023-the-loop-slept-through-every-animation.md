# BUG-0023 — The frame loop slept through every animation

```
Bug ID        : BUG-0023
Severity      : S1 (CRITICAL) — the motion system was structurally unable to
                animate; every spring advanced in one-second jumps
Class         : SCHEDULING / MOTION
Subsystem     : src/litric-shell/src/shell.rs (Shell::wake_after_ms)
First Observed: 2026-10-25, reading the loop order against `wake_after_ms`
Status        : FIXED — `cargo test -p litric-shell --lib mid_flight`
```

## Symptom

A panel opened from the dock arrived late and in jumps: the spring was retargeted
by the click, the first frame drew it barely moved, and the animation then
completed roughly a second later in two visible steps. Nothing in `anim.rs` was
wrong — `Anim::step` is a correct semi-implicit Euler integrator with a settle
budget — so no amount of work on the springs would have fixed it.

## Reproducer (test, not a manual impression)

```rust
let mut s = shell();
s.activate(Hot::Logo);          // open the launcher
s.tick(30_000);                 // the frame that starts it
let _ = s.take_damage();        // the frame is drawn
assert!(s.anims().animating());
s.wake_after_ms(30_000)         // returns 1000, not 16
```

`wake_after_ms` answered **1000 ms** — `Shell::IDLE_SLEEP_CAP_MS`, the longest it
may sleep before the top bar's clock goes stale — for a desktop with a spring
mid-flight.

## Cause

The loop order is:

```
wake_after_ms → poll(budget) → handle → tick (damages) → render → take_damage → wake_after_ms → …
```

`tick` produces the damage a moving spring needs, and the same iteration's draw
consumes it. By the time the loop asks how long it may sleep, a moving spring is
therefore **not dirty** — it is *about to be* dirty, on the next tick. The
question was answered from damage alone:

```rust
if self.is_dirty() {
    match self.life() { … _ if self.animating() => Active, _ => 0 }
}
// animating but not dirty falls through to:
until_clock.min(ms_until_idle_change).min(Self::IDLE_SLEEP_CAP_MS)
```

so an animation in flight slept up to a second, then advanced with one step whose
delta `Anim::step` clamps to `MAX_STEP_MS` (100 ms). The settle budget counted
that one second, so a `FAST` (200 ms) spring landed in two jumps.

The existing test `motion_that_outlives_the_idle_timer_still_runs_at_the_active_pace`
covered the *other* half of the question — animating **and** dirty — and passed
throughout the time this was broken.

## Fix

`Shell::wake_after_ms` asks whether a spring is in flight before it falls back to
the clock, and answers with the current life state's frame interval. A locked
session keeps its own pace; anything else runs at the active pace. The idle
answer is unchanged: a settled desktop still sleeps to its next clock change.

```rust
if self.animating() {
    return match self.life() {
        LifeState::Locked => LifeState::Locked.frame_interval_ms(),
        _ => LifeState::Active.frame_interval_ms(),
    };
}
```

## Test

`shell::tests::a_spring_mid_flight_asks_for_the_next_frame_after_the_damage_was_taken`
asserts the frame interval while the spring travels, then steps 60 frames and
asserts the loop goes back to a sleep longer than a frame once everything has
landed — the idle regression that a naive "always return 16" would introduce.

## What this did not change

`wake_after_ms` is a *ceiling on the next sleep*, not a frame clock; the loop
still merges input into damage and draws on its own frame clock. No FPS was
manufactured: the same work now happens at the pace the life state asks for.

## Rollback

Delete the `if self.animating()` block in `wake_after_ms` (shell.rs) and the
`a_spring_mid_flight_…` test. Nothing else depends on it.
