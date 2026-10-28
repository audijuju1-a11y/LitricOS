# BUG-0033 — A default port made a second origin, and a bad port became part of the host

```
Bug ID        : BUG-0033
Severity      : S2 (HIGH) — a site could vary the port spelling to miss a
                stored permission decision, and a malformed port hid a host from
                the tracker list
Class         : SECURITY / URL PARSING (origin not canonical; authority tail
                folded into the host)
Subsystem     : src/litric-browser/src/url.rs (`Url::parse`, `Url::origin`),
                src/litric-browser/src/policy.rs (permission and tracker checks)
First Observed: 2026-10-28, full-system audit of untrusted address-bar input
Status        : FIXED (four url tests, two policy tests)
```

## Why this exists

The permission model is deliberately keyed on the **origin string**
(`Policy::permission` → `url.origin()`), and the module's own test says so:
a permission must follow the origin, not the page. But `Url::parse` kept an
explicit default port:

```text
Url::parse("https://example.com/")      -> origin "https://example.com"
Url::parse("https://example.com:443/")  -> origin "https://example.com:443"
```

Two origins for one site. A site that the user refused (say, the camera) could
load itself with `:443` and the stored `Deny` would not be found, so the decision
fell back to `Ask` — the refusal was not enforced, it was re-asked. The same
split defeated any future per-origin storage, because the key was not canonical.

The authority parser also only recognised a port when the text after the last
colon was all digits, and otherwise kept the colon in the host:

```rust
let (host_part, port) = match authority.rsplit_once(':') {
    Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => { ... }
    _ => (authority, None),
};
```

So `https://tracker.test:abc/pixel` became host `tracker.test:abc`, and
`Policy::is_tracker` (exact host or `.suffix`) did not match the tracker list.
`check_navigation` allowed the navigation. No engine is integrated yet, so
nothing was fetched, but the policy decision itself was wrong and would have
been the one an engine received.

## Fix

**The default port is dropped at parse time.** After the host is validated:

```rust
let port = port.filter(|p| Some(*p) != scheme.default_port());
```

`https://site:443`, `https://site:0443` and `https://site` now have one origin,
and `to_url()` prints the canonical form. A non-default port (`:8443`) is still
an origin of its own and is still displayed, which is correct: it really is a
different service.

**A port that is not a number is an error.** The only authority with colons and
no port is a bare IPv6 literal, so the parser special-cases `[ … ]` and requires
every other post-colon tail to be non-empty digits within `u16`:

```rust
let (host_part, port) = if authority.starts_with('[') && authority.ends_with(']') {
    (authority, None)
} else if let Some((h, p)) = authority.rsplit_once(':') {
    if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
        return Err(UrlError::BadPort);
    }
    ...
```

Port 0 and ports above 65535 were already refused and still are. A bare
`[::1]` keeps its colons; `[::1]:8443` still takes the port.

## Regression tests

* `url::tests::a_default_port_is_the_same_origin_as_no_port` — `:443` and `:80`
  normalise to `None`, `to_url()` is canonical, and `:8443` stays distinct.
* `url::tests::a_bad_port_is_refused` — `:abc`, an empty port, `:0`,
  `:65536` and `:99999` are `BadPort`.
* `url::tests::an_ipv6_literal_keeps_its_colons_and_still_takes_a_port` —
  `[::1]` and `[::1]:8443`.
* `policy::tests::a_default_port_cannot_step_around_a_refusal` — a stored
  Camera `Deny` applies to the `:443` spelling.
* `policy::tests::a_tracker_cannot_hide_behind_a_default_port_or_a_bad_one` —
  `tracker.test:443` is still refused, and `tracker.test:abc` no longer parses
  into a host at all.

## Verification

```text
$ cargo test -p litric-browser -p litric-notifyd --lib
litric-browser: 89 passed, 0 failed
litric-notifyd: 47 passed, 0 failed
$ cargo test --workspace
TESTS passed=1643 failed=0
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo fmt --check
exit 0
```

## Residual risk

* **No engine and no network test.** The fix is in the parser and policy layer;
  no HTTP client exists to observe the host string that would be dialled.
* **Origin normalisation of a persisted store is not migrated**, because no
  store writes `SiteSettings` in this tree yet. When one is added it must key on
  the normalised origin this parser now produces.
* Not a claim of zero vulnerabilities: only the two defects above were found and
  fixed in this pass.
