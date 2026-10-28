# BUG-0035 — A malformed address was searched for instead of reported

```
Bug ID        : BUG-0035
Severity      : S3 (MEDIUM) — a broken address left the machine as a search
                query, and the omnibox sent unvalidated addresses to the engine
Class         : PRIVACY / INPUT VALIDATION (the address bar's decision did not
                match its own contract, and the shell did not use the parser)
Subsystem     : src/litric-browser/src/url.rs (`interpret`, `validate_host`),
                src/litric-browser/src/lib.rs (`Session::go`, `Outcome`),
                src/litric-shell/src/apps.rs (`BrowserApp::commit_omnibox`)
First Observed: 2026-10-28, residual risk recorded by BUG-0033
Status        : FIXED (four new tests: two url, one session-level, one shell-level)
```

## Why this exists

`interpret` documented the rule it meant to follow, and then did the opposite:

```rust
// A malformed address the user clearly meant as one stays a
// navigation attempt, so they get an error about the address
// rather than a search for it.
Err(_) => Intent::Search(s.to_owned()),
```

Nothing consumed the error case, so `Session::go` returned
`Outcome::NoSearchProvider` for `https://example.com:abc/` — the tab was told a
search had been requested, not that the address was broken. With no search
provider configured the text went nowhere, which is why BUG-0033 recorded it as
a hazard rather than an exploit; the day a provider exists, every typo in an
address is a query sent to a third party.

The desktop's own omnibox did not use `litric_browser` at all, so it had the same
class of defect independently: anything containing `://` was passed to the
engine unvalidated, and a host-like word with a bad port (`example.com:abc`) was
turned into `https://example.com:abc` and sent as though it were an address. A
searching round trip for a malformed address is exactly what the comment in
`interpret` said must not happen, so the two paths were fixed together through
one parser.

A truncated IPv6 literal made this worse: `https://[::1` did not fail at all.
The bracket left a colon that the port rule read, producing host `[:` with port
`1`, so the address was accepted.

## Fix

**One explicit outcome, not a search.** `Intent` gained:

```rust
AddressError { input: String, error: UrlError },
```

`interpret` returns it for a scheme this build supports whose text fails
`Url::parse`. A scheme the browser does not support (`javascript:`, `note:`) and
plain text are still `Search`: those are text first and addresses never, which
is why `interpret("javascript:alert(1)")` must not become a navigation attempt.

**A truncated IPv6 literal is a bad host.** `validate_host` now accepts a
bracket only as a complete literal:

```rust
if (host.contains('[') || host.contains(']')) && !(host.starts_with('[') && host.ends_with(']'))
```

**The session reports it and marks the tab.** `Session::go` handles the new
intent, sets the tab's `LoadState::Failed` with `that address is not valid: …`
(which is the tab model the interface renders), does not enter history, and
returns `Outcome::InvalidAddress { input, reason }`.

**The omnibox validates through the same parser.** `litric-shell` now depends on
`litric-browser` (it already depends on `litric-defender`, `litric-files` and
`litric-notifyd`). `BrowserApp::commit_omnibox` builds the address exactly as
before — the returned string is unchanged for every valid input — and parses it
before `navigate_active_to`; on failure it sets `status` to
`that address is not valid: …`, sets the tab's `error` flag, and returns `None`
so the caller sends no engine request. Both the `://` form and the host-like
form (`example.com:abc`) are validated; a search URL is not, because this file
builds and encodes it and it has no host to check.

## Regression tests

* `url::tests::a_malformed_supported_address_is_an_error_not_a_search` — bad
  port (`:abc`, bare `:`), empty host (`https:///path`) and a truncated IPv6
  literal all return `AddressError` with the expected `UrlError`; plain text,
  `javascript:` and whitespace-only still return `Search`.
* `url::tests::a_truncated_ipv6_literal_is_not_a_host` — `https://[::1` is
  `BadHost`.
* `lib::tests::a_malformed_address_is_reported_on_the_tab_instead_of_becoming_a_search`
  — caller-level: `Session::go` returns `InvalidAddress`, the active tab is
  `LoadState::Failed` with the sentence, history stays empty, and the valid and
  search paths still behave as before.
* `apps::tests::a_malformed_address_is_refused_in_the_omnibox_instead_of_searched_or_sent`
  — caller-level: `commit_omnibox` returns `None`, the status line carries the
  sentence, the tab is marked and does not move, `browser_action(BrowserGo)`
  sends nothing for `ftp://…` or `example.com:abc`, and valid addresses and
  searches still resolve exactly as they did.

## Verification

```text
$ cargo fmt --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
Finished dev profile; exit 0
$ cargo test -p litric-browser --lib
92 passed; 0 failed
$ cargo test -p litric-shell --lib -- malformed_address
1 passed; 0 failed (600 filtered out)
$ cargo test --workspace
TESTS passed=1647 failed=0        (was 1643; +4 tests in this batch)
$ python3 ci/check-python-syntax.py
OK: 61 python file(s) parse; exit 0
$ python3 ci/lint-line-endings.py
OK: 352 file(s) checked, all LF; exit 0
$ python3 ci/lint-doc-refs.py
OK: 765 citation(s) across 153 document(s) resolve; exit 0
```

## Residual risk

* **The shell validates, but did not canonicalise** at the time of this fix. A
  valid address was sent as the user spelled it, because changing the string
  changes history and bookmark keys. That residual was taken up and closed in
  BUG-0036: the parser's canonical text is now the stored key on navigation and
  on load.
* **Only `litric-browser`'s `Session` and the shell omnibox are wired.** Any
  future caller of `interpret` must handle `AddressError`; the enum is not
  `#[non_exhaustive]` on `Intent`, so a new `match` will fail to compile rather
  than silently search.
* **The error sentence is English-only**, like the rest of the browser status
  strings; it is not in `litric-locale` yet.
* **No browser UI renders the library tab model.** The caller-level proof for
  `Session::go` is its tab state and outcome, not a pixel test; the shell's own
  caller-level test covers the surface a user actually types into.
* Not a claim of zero vulnerabilities: this closes one documented gap, it does
  not prove the address bar is now complete.
