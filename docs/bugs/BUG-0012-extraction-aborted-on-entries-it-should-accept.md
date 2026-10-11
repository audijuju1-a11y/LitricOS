# BUG-0012 — Extraction aborted on an archive it should have accepted

```
Bug ID        : BUG-0012
Severity      : S3 (MEDIUM) — a valid archive fails, or fails part-way with the
                entries already written left behind
Class         : FUNCTIONAL / PARTIAL-FAILURE SEMANTICS
Subsystem     : src/litric-files/src/archive.rs
First Observed: 2026-10-09, source inspection of `extract_single_gzip` and
                `write_entry`
```

## Reproducer

```bash
sed -n '/fn extract_single_gzip/,/make_extract_root/p' src/litric-files/src/archive.rs
sed -n '/fn write_entry/,/^}/p' src/litric-files/src/archive.rs
sed -n '/pub fn from_name/,/^    }/p' src/litric-files/src/archive.rs
```

No build, no execution; both defects are provable from the classification and the
write path.

## Defect 1 — the suffix match was case-sensitive while the classification is not

`ArchiveKind::from_name` lowercases the name, so `NOTES.TXT.GZ` is a gzip archive.
`extract_single_gzip` then used `base.strip_suffix(".gz")`, which does not match
`.GZ`, so the stem stayed `NOTES.TXT.GZ`. The output logical path is therefore the
archive's own name; `write_entry` opens it with `create_new` (`O_EXCL`) and fails
with `exists`. Extracting `NOTES.TXT.GZ` inside its own directory — the ordinary
case — therefore fails with "extract file: File exists" and writes nothing.

**Expected:** the same stem rule as for the lowercase name, `NOTES.TXT`.

## Defect 2 — a directory entry that is already there aborted the extraction

`write_entry` called `fs::create_dir` for a directory entry and turned any error
into a failure. Archives legitimately carry the same directory more than once —
ustar writers store `docs/`, and tooling that walks a tree also stores `docs` —
and the second entry then failed with `AlreadyExists` in the middle of the write
loop, leaving every entry already written on disk and reporting an error.

**Expected:** a repeat of a directory the extraction itself created is accepted.
It must *not* be accepted when a symlink or a non-directory stands there: that is
the case the `O_EXCL`/no-follow rule exists to catch.

## Fix

* `strip_suffix_ci` (new, private) strips an ASCII-case-insensitive suffix using
  `text.get(cut..)` + `eq_ignore_ascii_case`, so no input can make the cut a
  non-boundary (no panic) and no lowercasing of the whole string is involved.
  `.tgz` is checked before `.gz`; the two are unambiguous.
* `write_entry` accepts `AlreadyExists` for a directory entry **only** after
  `symlink_metadata` shows a real directory (not a symlink, not a file). Anything
  else is reported as `unsafe-archive-entry`.

Rejected alternative: pre-scanning the entry list for duplicate names. It would
have to re-implement the archive's own naming rules and would not cover a
directory created by an earlier entry's implicit parent chain
(`ensure_dir_chain`).

## Regression Test

`src/litric-files/src/archive.rs` has no test module today; the cases to add (in a
build phase) are: an entry list with `docs/` twice extracts once and succeeds; a
symlink standing where the archive wants a directory is still refused; a
`NOTES.TXT.GZ` file extracts to `NOTES.TXT`; and `strip_suffix_ci` is exercised
against `"a.GZ"`, `"a.tgz"`, `"a.gz"`, `"Å.gz"` and a string shorter than the
suffix. **Rust tests are not executed in this session.**

## Verification

| Check | Result |
|---|---|
| `rustfmt --check --edition 2021 src/litric-files/src/archive.rs` | **PASS** (exit 0) |
| `bash ci/verify.sh` (cargo hidden) | permitted subset **PASS**, Rust stage **SKIPPED** |
| Behaviour of either fix | **NOT VERIFIED** — needs a build |

## Residual Risk

* Extraction still leaves the entries it already wrote when a *later* entry fails
  (a real I/O error, a `create_new` collision with a pre-existing file). That is
  deliberate and unchanged: the whole archive is validated and decompressed
  before the first byte is written, so only filesystem failures remain, and a
  partial extraction that reports an error is honest. "Extract into a temporary
  directory and rename it into place" would remove the partial state and is worth
  its own change, with its own tests.
* `write_entry` on a *file* entry still refuses a pre-existing file
  (`create_new`). Overwriting on extract would need an explicit policy; today the
  answer is "no", which is the safe one.
