#!/usr/bin/env python3
"""Check that every file-deletion call site lives in a function someone reviewed.

Why this exists. The workspace has no runtime test for the file layer: no booted
image, no compiled crate, no QEMU (see docs/audit/2026-10-09-continuation-recovery-audit.md).
What it does have is a small number of places that destroy data, and a set of
defects found in them (BUG-0010, BUG-0011, BUG-0012). A new
`fs::remove_dir_all` appearing in an unreviewed function is therefore a change
with the largest possible blast radius and the smallest possible test coverage.

What this checks:

  * Every `fs::remove_file(` / `fs::remove_dir_all(` call in `src/**.rs` must be
    inside a function named in ALLOWED, keyed by file and function name.
  * Each entry carries the reason it is allowed, and that reason is printed when
    the check passes, so the list reads as a review record rather than a
    suppression list.

What it deliberately does not do:

  * It does not prove the allowed sites are correct -- that is what the tests in
    `src/litric-files/src/ops.rs` and the trash modules are for, and those have
    not been executed in this mode.
  * It does not do Rust parsing. The enclosing function is found by scanning
    upwards for the nearest `fn` line, so a closure or a `macro_rules!` body
    around a deletion would be attributed to the function that contains it. That
    is the conservative direction: it makes the check stricter, not looser.
  * It does not run anything, write anything, or read anything outside `src/`.

Adding a deletion site is meant to require editing this file with a reason. That
is the review step, and it is cheaper than the alternative.

Usage:
    python3 ci/lint-file-safety.py             check
    python3 ci/lint-file-safety.py --self-test prove it can fail
"""

import os
import re
import sys
import tempfile

# file -> {function name: why deleting there is allowed}
#
# This is a review record, not a suppression list: every production deletion in
# src/ is named here with the reason it is safe to do it there. A new one is a
# FAIL until somebody adds a line with a reason.
ALLOWED = {
    "src/litric-files/src/ops.rs": {
        "remove_host": "the one choke point for deleting a path under the daemon root (ADR-0019 section 3)",
    },
    "src/litric-files/src/trash.rs": {
        "put": "rolls back the .trashinfo reservation when the move failed, so nothing left its place (BUG-0011)",
        "reserve_info": "removes the record it is creating when the write fails (BUG-0011)",
        "restore": "removes the record of an entry it has just put back where it came from",
        "purge": "permanent removal of one trash entry the user asked to purge; the name is validated and lives under files/",
    },
    "src/litric-files/src/server.rs": {
        "serve_unix": "unlinks a stale socket file before bind; never a user path",
    },
    "src/litric-shell/src/fsbridge.rs": {
        "move_item": "deletes the source only after a cross-device copy completed",
        "delete": "permanent removal of a validated logical path the user asked to delete",
        "trash": "rolls back the .trashinfo reservation when the move failed (the BUG-0011 shape, second implementation)",
        "trash_restore": "removes the record of an entry it has just restored",
        "reserve_trash_name": "removes the record it is creating when the write fails",
    },
    "src/litric-browser/src/bin/browserd.rs": {
        "run": "unlinks a stale socket file before bind; never a user path",
    },
    "src/litric-sysd/src/server.rs": {
        "bind": "replaces a stale socket file after a symlink_metadata check; never a user path",
    },
    "src/litric-shell/examples/render_bench.rs": {
        "main": "removes the frame-buffer device file the benchmark created itself",
    },
    "src/litric-bench/src/bin/litricos-test.rs": {
        "cmd_cleanup": "the explicit cleanup command; the path is the one the CLI was given",
    },
    "src/litric-shell/src/e2e.rs": {
        "journey_defender_scans_and_reports_eicar_end_to_end": "removes the temporary root this journey created",
        "journey_trash_window_restores_after_files_trash": "removes the temporary root this journey created",
    },
}


def code_only(text):
    """`text` with comments and string/char literal contents blanked out.

    Without this the checker flagged a comment that merely mentioned
    `fs::remove_file(x)` -- the same class of false positive the memory-budget
    checker had to handle for a directive inside a comment (BUG-0008). The
    scanner is deliberately small: line and block comments (nested, as Rust
    allows), `"…"` with escapes, `r"…"`/`r#"…"#` raw strings, and `'x'` char
    literals. Lifetimes (`'a`) are left alone. Newlines and byte offsets are
    preserved so a hit still points at the right line.
    """
    out = []
    i = 0
    n = len(text)
    block_depth = 0
    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if block_depth:
            if ch == "/" and nxt == "*":
                block_depth += 1
                out.append("  ")
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                out.append("  ")
                i += 2
                continue
            out.append("\n" if ch == "\n" else " ")
            i += 1
            continue
        if ch == "/" and nxt == "/":
            while i < n and text[i] != "\n":
                out.append(" ")
                i += 1
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            out.append("  ")
            i += 2
            continue
        # raw string: r"..." or r#"..."#
        if ch == "r" and (nxt == '"' or nxt == "#"):
            hashes = 0
            j = i + 1
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                closer = '"' + "#" * hashes
                end = text.find(closer, j + 1)
                end = n if end == -1 else end + len(closer)
                out.append(" " * (end - i))
                i = end
                continue
        if ch == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(" " * (j - i))
            i = j
            continue
        if ch == "'":
            # a char literal or a lifetime; only the first is a literal
            j = i + 1
            if j < n and text[j] == "\\":
                j += 2
            else:
                j += 1
            if j < n and text[j] == "'":
                out.append(" " * (j + 1 - i))
                i = j + 1
                continue
        out.append(ch)
        i += 1
    return "".join(out)


def scan_text(text, path):
    """Every deletion call site in `text` as (line_number, function_name, line).

    Sites inside a `#[cfg(test)]` module are skipped: they run only under a test
    build, and their target is a fixture in the temporary directory, not user
    data. The risky sites are the ones a booted system executes, and those are
    the ones that must be reviewed. The test-module gate is the first
    `#[cfg(test)]` in the file, which is how this workspace writes them.
    """
    raw_lines = text.splitlines()
    code_lines = code_only(text).splitlines()
    test_start = None
    for i, line in enumerate(raw_lines):
        if line.strip() == "#[cfg(test)]":
            test_start = i
            break
    fn = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)")
    delete = re.compile(r"(?:std::)?fs::remove_(?:file|dir_all)\s*\(")
    found = []
    for i, line in enumerate(code_lines):
        if not delete.search(line):
            continue
        if test_start is not None and i >= test_start:
            continue
        name = "<top level>"
        for j in range(i, -1, -1):
            m = fn.match(code_lines[j])
            if m:
                name = m.group(1)
                break
        found.append((i + 1, name, raw_lines[i].strip(), False, path))
    return found


def collect(root):
    hits = []
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if not name.endswith(".rs"):
                continue
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full).replace(os.sep, "/")
            with open(full, encoding="utf-8", errors="replace") as handle:
                hits.extend(scan_text(handle.read(), rel))
    return hits


def decide(hits, allowed):
    """Split hits into (allowed, violations). Test-only sites are reported too:
    a deletion inside `#[cfg(test)]` still runs in a build with tests."""
    allowed_hits, violations = [], []
    for line_no, fname, line, _in_tests, path in hits:
        why = allowed.get(path, {}).get(fname)
        if why:
            allowed_hits.append((path, line_no, fname, why, line))
        else:
            violations.append((path, line_no, fname, line))
    return allowed_hits, violations


def report(allowed_hits, violations):
    if violations:
        print(f"FAIL: {len(violations)} unreviewed deletion site(s) in src/")
        for path, line_no, fname, line in violations:
            print(f"  {path}:{line_no} in fn {fname}()")
            print(f"      {line}")
        print()
        print("Each deletion must go through a reviewed function. Either route the")
        print("call through `Root::remove_host` (the one choke point in the file")
        print("layer) or add the (file, function) pair to ALLOWED in this file with")
        print("the reason it is safe.")
        return 1
    print(f"OK: {len(allowed_hits)} deletion site(s), all inside reviewed functions")
    for path, line_no, fname, why, _line in allowed_hits:
        print(f"  {path}:{line_no} fn {fname}() -- {why}")
    return 0


SELF_TEST_CASES = (
    (
        "a reviewed function passes",
        {
            "src/a.rs": (
                "fn remove_host(p: &str) {\n"
                "    let _ = std::fs::remove_dir_all(p);\n"
                "}\n"
            )
        },
        {"src/a.rs": {"remove_host": "choke point"}},
        0,
    ),
    (
        "a new function fails",
        {
            "src/a.rs": "fn cleanup(p: &str) {\n    let _ = std::fs::remove_file(p);\n}\n"
        },
        {"src/a.rs": {"remove_host": "choke point"}},
        1,
    ),
    (
        "a comment mentioning the call does not count",
        {"src/a.rs": "// fs::remove_file(x) was removed\nfn keep() {}\n"},
        {},
        0,
    ),
    (
        "a block comment does not count either",
        {
            "src/a.rs": (
                "/* the old code called fs::remove_dir_all(path) here */\n"
                "fn keep() {}\n"
            )
        },
        {},
        0,
    ),
    (
        "a call inside a string literal does not count",
        {"src/a.rs": 'fn keep() {\n    let s = "fs::remove_file(x)";\n}\n'},
        {},
        0,
    ),
    (
        "a deletion inside a test module needs no decision",
        {
            "src/a.rs": (
                "#[cfg(test)]\nmod tests {\n    fn fixture() {\n"
                "        let _ = std::fs::remove_dir_all(\"/tmp/x\");\n    }\n}\n"
            )
        },
        {},
        0,
    ),
    (
        "a raw string cannot hide a call after it",
        {
            "src/a.rs": (
                'fn keep() {\n    let s = r#"raw fs::remove_file(x)"#;\n'
                "    let _ = std::fs::remove_file(\"/tmp/y\");\n}\n"
            )
        },
        # `keep` is deliberately NOT allowed: the call inside the raw string is
        # text, the call after it is code, and only the second is a violation.
        {"src/a.rs": {"some_other_fn": "unrelated"}},
        1,
    ),
)


def self_test():
    """Prove the checker fails on what it is supposed to catch."""
    failures = 0
    for name, files, allowed, want in SELF_TEST_CASES:
        with tempfile.TemporaryDirectory() as tmp:
            for rel, body in files.items():
                full = os.path.join(tmp, rel)
                os.makedirs(os.path.dirname(full), exist_ok=True)
                with open(full, "w", encoding="utf-8") as handle:
                    handle.write(body)
            old_cwd = os.getcwd()
            os.chdir(tmp)
            try:
                hits = collect("src")
            finally:
                os.chdir(old_cwd)
            # Paths inside the fixture are relative to the fixture; match on the
            # allowlist exactly as the fixture wrote it.
            _allowed_hits, violations = decide(hits, allowed)
            got = 1 if violations else 0
            mark = "ok" if got == want else "NOT AS EXPECTED"
            if got != want:
                failures += 1
            print(f"  [{mark}] {name}: expected exit {want}, got {got}")
    if failures:
        print(f"FAIL: {failures} self-test case(s) behaved wrongly")
        return 1
    print(f"OK: {len(SELF_TEST_CASES)} self-test case(s) behaved as designed")
    return 0


def main(argv):
    # A console whose encoding cannot represent a character in a path or a
    # reason must not turn a verdict into a traceback: this checker was itself
    # crashing on a `section 3` sign under a cp874 console before that was
    # fixed, and the crash looked exactly like a passing run with output.
    try:
        sys.stdout.reconfigure(errors="replace")
    except (AttributeError, ValueError):
        pass
    if "--self-test" in argv:
        return self_test()
    root = "src"
    if not os.path.isdir(root):
        print(f"FAIL: no {root}/ directory -- run this from the repository root")
        return 1
    hits = collect(root)
    if not hits:
        print("FAIL: no deletion sites found at all, which cannot be right")
        print("      (a wrong root or an empty allowlist scan would hide every check)")
        return 1
    allowed_hits, violations = decide(hits, ALLOWED)
    return report(allowed_hits, violations)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
