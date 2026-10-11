# BUG-0007 — Five tool scripts had CRLF line endings, and `ci/verify.sh` was red because of it

```
Bug ID        : BUG-0007
Severity      : S3 (MEDIUM) — no data loss and no boot impact, but the
                repository's central verification run reported FAIL, and five
                executable Python tools could not be run via their shebang
Class         : BUILD-HYGIENE (PROTOCOL LAW 8: a check that is red for a
                boring reason is a check people learn to skip)
Subsystem     : tools/
First Observed: 2026-10-09, code-only audit (`ci/lint-line-endings.py` run as
                part of the permitted CI subset)
```

## Reproducer

```bash
python3 ci/lint-line-endings.py; echo "exit=$?"
```

## Expected

`OK: 218 file(s) checked, all LF`, exit 0 — which is what `ci/verify.sh`
section `[1b]` requires.

## Actual

```
5 file(s) have CRLF line endings and will fail on Linux:
  tools/photo-tour-apps.py  (113 lines)
  tools/photo-tour-real.py  (188 lines)
  tools/probe-browser.py  (95 lines)
  tools/verify-google-pixels.py  (105 lines)
  tools/verify-tour.py  (233 lines)
exit=1
```

All five carry a shebang *and* the executable bit
(`-rwxr-xr-x … #!/usr/bin/env python3\r\n`), so on Linux:

* `./tools/verify-tour.py` fails as `env: 'python3\r': No such file or
  directory` — an interpreter that exists, named as though it does not, which
  is the failure this linter's own docstring was written to describe;
* `python3 tools/verify-tour.py` works (CPython tolerates CRLF), so the defect
  is invisible to anyone who invokes it that way, and invisible in Git Bash,
  which reads files in text mode;
* `ci/verify.sh` reports section `[1b]` as **FAIL** on every run, so the
  repository's answer to "what is actually verified?" is red for a reason
  unrelated to any of the things it verifies.

File dates (2026-09-23/24) place the defect before this session; the write
pattern is the one the linter documents — a `pathlib` `write_text()` on a
Windows host translating every `\n` into `\r\n`.

## Impact

Nothing about LitricOS's behaviour changes. The impact is on verification
quality: a permanently red check in the only script that reports what was
executed. A reader who sees FAIL for a boring reason stops reading the FAILs,
and the next real one is then invisible — the same failure mode as a check that
cannot fail.

## Fix

```bash
python3 ci/lint-line-endings.py --fix
```

The linter rewrites the bytes (never `write_text`, which would translate LF back
to CRLF on this host and report a fix that did not happen). No other change: no
content, no behaviour, no file added or removed.

## Regression Test

`tests/test_harness_integrity.py::test_no_crlf_in_harness_files` covers the
harness files themselves. The repository-wide property is already covered by
`ci/verify.sh` section `[1b]`, which is why this bug was caught rather than
found by reading — the check worked; the defect was simply left in place.

## Verification

| Command | Result |
|---|---|
| `python3 ci/lint-line-endings.py` before | **FAIL**, 5 files |
| `python3 ci/lint-line-endings.py --fix` | converted 5 files, exit 0 |
| `python3 ci/lint-line-endings.py` after | **PASS**, `OK: 218 file(s) checked, all LF` |
| `ast.parse` over all 18 `tools/*.py` after the conversion | **PASS** |
| shebang bytes after | `#!/usr/bin/env python3\n` |

## Residual Risk

The converter is byte-level and was checked against the 5 files it named; it
reports the full 218-file candidate set, so nothing outside that set was
touched. Any file not matched by the linter's `PATTERNS` list (for example a new
extension added later) is not covered — that is a gap in the check, not in this
fix.
