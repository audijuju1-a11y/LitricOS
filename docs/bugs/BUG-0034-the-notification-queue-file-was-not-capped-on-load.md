# BUG-0034 — The queue file could load more notifications than the queue allows

```
Bug ID        : BUG-0034
Severity      : S3 (MEDIUM) — a corrupt or hand-edited queue file bypassed the
                memory bound the daemon promises
Class         : RESOURCE EXHAUSTION / INVARIANT (bound enforced on post, not on
                load)
Subsystem     : src/litric-notifyd/src/queue.rs (`parse`, `Queue::CAPACITY`)
First Observed: 2026-10-28, full-system audit of persisted service input
Status        : FIXED (one regression test)
```

## Why this exists

The module states the bound and why it matters:

> Anything that accepts text from any program on the machine is a way to fill
> memory. [`Queue::CAPACITY`] is the ceiling…

`Queue::post` enforces it, but `parse` did not:

```rust
"end" => {
    if let Some((id, lines)) = current.take() {
        match id.map(|id| crate::proto::note_from_lines(id, &lines)) {
            Some(Some(note)) => {
                queue.next_id = queue.next_id.max(note.id);
                queue.items.push(note);
            }
```

Every readable record was pushed, so a file with ten thousand notes loaded ten
thousand notes. The file is written `0600` under a `0700` directory, so this is
not a remote attack; it is the invariant failing on the one input the daemon
cannot fully trust — a file written by an older or buggy build, restored from a
backup, or edited by an administrator. The queue is also rewritten whole on the
next mutation, so an oversized file stays oversized.

## Fix

Cap at the same place the bound is defined, and count what was dropped so the
daemon logs it instead of losing it silently:

```rust
let loaded = queue.items.len();
queue.items.truncate(Queue::CAPACITY);
skipped = skipped.saturating_add(loaded.saturating_sub(queue.items.len()));
```

`encode` writes newest first and `parse` pushes in file order, so truncating the
tail drops the oldest notifications — exactly the order `post` evicts in, and
`next_id` is still the maximum id seen, so no id is reissued.

## Regression test

`queue::tests::a_file_holding_more_notes_than_capacity_loads_only_the_capacity`
— a file with `CAPACITY + 5` records loads exactly `CAPACITY`, reports `5`
skipped, keeps the newest record first, and leaves `next_id` at the highest id
in the file.

## Verification

```text
$ cargo test -p litric-notifyd --lib
47 passed, 0 failed
$ cargo test --workspace
TESTS passed=1643 failed=0
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ python3 ci/lint-line-endings.py
OK: 352 file(s) checked, all LF
```

## Residual risk

* **The bound is a count, not a byte budget.** 64 notes of bounded length is a
  few tens of kilobytes; the note fields are already length-limited by the
  protocol parser, but `parse` accepts a queue file whose individual fields were
  not validated by that parser. Worth a byte cap when the file format changes.
* **A corrupted file still loses records** (by design: it shows what it can
  read). The skipped count is the only signal, and only the daemon log carries
  it.
* No fuzz run was performed on the queue parser; the test above is a directed
  case, not proof of parser robustness.
