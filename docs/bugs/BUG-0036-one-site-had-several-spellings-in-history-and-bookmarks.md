# BUG-0036 — One site had several spellings in history and bookmarks

```
Bug ID        : BUG-0036
Severity      : S3 (MEDIUM) — one page occupied several history entries and
                bookmarks, and a bookmarked page read as unbookmarked when it
                was reached by another spelling
Class         : CORRECTNESS / IDENTITY (the shell keyed stored state on the
                typed text instead of the parser's canonical address)
Subsystem     : src/litric-browser/src/url.rs (`Url::to_url`),
                src/litric-shell/src/apps.rs (`BrowserApp::commit_omnibox`,
                `BrowserApp::navigate_active_to`, `canonical_address`),
                src/litric-shell/src/main.rs (`load_browser_data`)
First Observed: 2026-10-28, recorded as the residual risk in BUG-0035 and
                docs/PROGRESS-TRACKER.md section 14.4
Status        : FIXED (five new tests plus four stale expectations updated)
```

## Why this exists

`Url::parse` had already been made canonical for the security properties: the
host is lowercased, a scheme's default port is dropped and an empty path becomes
`/`. That was done for permissions, which are keyed on `origin()`, and it is why
BUG-0033 could not be used to step around a stored refusal.

The shell did not use any of it. `BrowserApp::commit_omnibox` returned the text
the user typed (after adding `https://` to a host-like word), and
`navigate_active_to` stored that string in `tabs[].history`, the recent-visits
list, and the bookmark list:

```rust
tab.history.push(url.to_owned());
...
tab.url = url.to_owned();
...
self.bookmarks.iter().any(|saved| saved == &tab.url)
```

`Url::parse` was called only to *validate* the text, and its result was thrown
away. So:

* `https://example.com`, `https://EXAMPLE.com` and `https://example.com:443`
  were three different keys for one page;
* the recent list held three rows for it, and Back walked through all three;
* bookmarking the first spelling and then visiting the second left
  `is_bookmarked()` false, so the star appeared unset on a saved page and
  pressing it added a second bookmark.

The same defect existed one layer down in the saved file: `load_browser_data`
deduplicated the decoded text as written, so a store created before this fix
kept both spellings across a restart.

This is the same class of defect as BUG-0033 — identity taken from a spelling
rather than from the parsed address — one layer up, where it costs the user a
duplicated history and a misleading bookmark state instead of a permission
bypass.

## Fix

**The canonical text is adopted at the storage choke point.**
`BrowserApp::canonical_address` parses through the one authority and returns
`to_url()`; anything the parser does not know (a search URL, a test token) is
returned unchanged, because it is not an address and has no canonical form:

```rust
pub fn canonical_address(input: &str) -> String {
    litric_browser::url::Url::parse(input).map_or_else(|_| input.to_owned(), |u| u.to_url())
}
```

`commit_omnibox` stores and returns `parsed.to_url()`, and `navigate_active_to`
normalises on entry, so every writer of a stored address — omnibox, a bookmark,
a history row — shares one key. `load_browser_data` canonicalises each decoded
entry before the duplicate check, so a file written by an older build loads as
one row per page rather than two that only collapse after both are visited.

**`to_url` writes the `//` an empty-host URL needs.** `file:///etc/hostname`
used to come back as `file:/etc/hostname`: the same path, but not the form a
specification or an address bar writes, and not a string the user would
recognise. The authority is now written when the scheme has a host, when one is
present, or when the path starts at the root. `litric:settings` still has no
authority, because its body is a path and not a host.

## What the user sees

| Typed | Stored, shown and returned |
|---|---|
| `https://example.com` | `https://example.com/` |
| `https://EXAMPLE.com/path` | `https://example.com/path` |
| `https://example.com:443` | `https://example.com/` |
| `file:///etc/hostname` | `file:///etc/hostname` |
| `litric://about` | `litric://about` |
| `litric docs` | the search URL, unchanged |

The path keeps its case and its text: only the host is case-folded, because only
the host is case-insensitive.

## Regression tests

* `url::tests::one_site_has_one_canonical_text_form` — three spellings parse
  equal and print the same text.
* `url::tests::a_file_url_keeps_its_empty_authority_slashes` — the file form
  round-trips as written.
* `url::tests::every_parsed_url_round_trips_through_its_text` — extended with
  `https://EXAMPLE.COM:443/a?b=c#d` and `litric://about`.
* `apps::tests::one_site_has_one_entry_however_it_is_typed` — caller-level:
  three spellings through the omnibox produce one history entry (index 1, not
  3), one row in the recent list, and a bookmark that stays set across a second
  spelling.
* `apps::tests::canonicalisation_leaves_search_urls_and_plain_tokens_alone` —
  `litric://about`, a bare token and a built search URL pass through untouched.
* `tests::saved_addresses_load_as_one_canonical_spelling` — a saved file holding
  two spellings of one page plus a bookmark loads as one canonical row each, and
  no warning is raised.

Four existing expectations in `apps.rs` and `e2e.rs` asserted the old
non-canonical text; they were updated to the new, intended output rather than
loosened.

## Verification

```text
$ cargo fmt --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
Finished dev profile; exit 0
$ cargo test -p litric-browser --lib
94 passed; 0 failed
$ cargo test -p litric-shell --lib
603 passed; 0 failed
$ cargo test --workspace
TOTAL passed=1652 failed=0        (was 1647; +5 tests in this batch)
  test tests::saved_addresses_load_as_one_canonical_spelling ... ok
$ python3 ci/check-python-syntax.py
OK: 61 python file(s) parse (no bytecode written); exit 0
$ python3 ci/lint-line-endings.py
OK: 352 file(s) checked, all LF; exit 0
$ python3 ci/lint-doc-refs.py
OK: 774 citation(s) across 155 document(s) resolve; exit 0
```

## Residual risk

* **Only case, default port, empty path and the authority slash are
  normalised.** Path and query percent-encoding are left as typed, so `%7E` and
  `~` remain different keys. That is a deliberate limit: decoding an encoding
  can change which resource is addressed, which is not a decision to take
  silently.
* **The saved file is rewritten in canonical form on the next save, not on
  load.** Loading leaves the user's file untouched (the existing rule that a
  read must not edit the store), so a file from an older build is deduplicated
  in memory and becomes canonical on disk at the next navigation or bookmark.
* **The canonical text is not a security boundary by itself.** Permissions stay
  keyed on `origin()`; this change makes stored identity agree with that key,
  not replace it.
* **The golden-frame gate remains BLOCKED** on the missing Latin font face
  fixture. No pixel test covers the browser's bookmark star; the proof here is
  model-level and caller-level.
* Not a claim of zero defects: this closes the residual recorded in BUG-0035,
  not every way a stored key can drift.
