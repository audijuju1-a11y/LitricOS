# ADR-0019 — Litric Files: a confined, socket-activated file daemon and an Explorer-style window

- **Status:** Accepted — implementation in progress; verification state is recorded at the end
- **Date:** 2026-09-13
- **Implements:** SYSTEM-SCOPE Part E "file manager", using `litric-storage` rules for
  names and `litric-trash` for what deleting means.

---

## Context

The shell draws every window but its unit has `ProtectHome=yes` and
`ProtectSystem=strict`, and must keep them: a display server that can also rewrite
the user's files is a much bigger prize. `litric-trash` already names the daemon that
performs trash operations (`litric-filesd`) and it did not exist.

## Decision

1. **`litric-filesd`** (crate `litric-files`) performs every file operation. It runs
   as the desktop user `litric`, with write access to one directory only
   (`/var/lib/litric-files`, created by `StateDirectory=` and opened through
   `ProtectSystem=strict`; `ProtectHome=yes` hides the user's home from it entirely),
   activated per connection by `litric-files.socket` (`/run/litric/files.sock`,
   `SocketMode=0600`, owner root, `Accept=yes`), exactly like the browser broker in
   ADR-0018. The shell is what opens that socket; it no longer performs a file
   operation of its own.
2. **Logical paths.** The shell never sends an absolute host path. Every path is
   relative to the user root and written `/Documents/report.odt`. Each segment is
   validated: not empty, not `.` or `..`, no `/`, no NUL or other control byte, at most
   255 bytes; the whole path at most 4096 bytes.
3. **No symlink is ever followed.** Every intermediate component is checked with
   `symlink_metadata` and must be a real directory. A symlink as the final component is
   listed, renamed, moved, trashed or deleted *as a link*; copying recreates the link;
   extraction never creates one.
4. **Trash is the freedesktop.org layout** at `<root>/.local/share/Trash/{files,info}`,
   with `Path=` written as a percent-encoded absolute path, which is what the
   specification requires and what other tools read. Names and retention come from
   `litric-trash`.
5. **Archives are implemented here, from the specifications** (RFC 1951 DEFLATE,
   RFC 1952 gzip, PKWARE APPNOTE ZIP, POSIX ustar), with no third-party crate and no
   external `zip`/`tar` binary — the `nano` image removes those binaries, and an
   archive parser is untrusted-input code that belongs in the confined daemon.

### Residual risk accepted

Check-then-act on paths has a TOCTOU window: a process *already running as the same
user* could swap a directory for a symlink between the check and the operation. It
gains nothing it could not already do with its own permissions. Closing the window
needs `openat`/`O_NOFOLLOW`, i.e. `unsafe` or libc, and is deferred and tracked.

## LFP/1 — the shell ↔ daemon wire protocol

Request: one UTF-8 line ending in `\n`, at most 16384 bytes, fields separated by one
space, free text percent-encoded as in ADR-0018. Response: `OK <n>` followed by exactly
`n` lines, or one line `ERR <kind> <pct-msg>`. Long operations may send any number of
`PROGRESS <done-bytes> <total-bytes> <current-pct>` lines before the final `OK`/`ERR`.
Entry lines are `key=value` pairs.

**Paths on the wire.** A path field is normally sent literally — `/Documents/a.txt` —
which is what every example below shows and what a reply always contains. The line
grammar separates fields with single spaces, so a path that holds a space (a folder the
user named *My Documents*), a literal `%`, or a non-ASCII byte cannot be written that
way. Such a path is percent-encoded instead, exactly like a free-text field, and then it
no longer starts with `/` (`/` encodes to `%2F`). The rule has no third case:

* starts with `/` → a literal path, used as it is;
* anything else → a percent-encoded path, which must decode.

Replies always carry **logical** paths (`/Documents/x`), never host paths. On the image
the root is `/` and the two spellings coincide; on a fixture root they do not, and a
host path in a message is a sandbox detail leaking into the interface.

| Command | Reply lines |
|---|---|
| `HELLO 1` | `version=… root=<pct> trash=1` |
| `PLACES` | `place=<home\|desktop\|documents\|downloads\|pictures\|music\|videos\|trash\|recent\|thispc> path=<pct> label=<pct> exists=<0\|1>` |
| `LIST <path> [hidden=0\|1]` | `e name= kind=<dir\|file\|link\|other> size= mtime= mode= hidden= target=` ; a final `truncated=1` if more than 10000 entries |
| `STAT <path>` | one entry line plus `items=` for a directory |
| `DU <path>` | `bytes= files= dirs= partial=<0\|1>` |
| `MKDIR <parent> <name>` · `NEWFILE <parent> <name>` | `path=<pct>` (logical) — never overwrites |
| `WRITETEXT <parent> <name> <content-pct>` | `path=<pct>` — creates with content, exactly three fields, never overwrites. The content is one field, so it is bounded by the line limit (16384 bytes); anything larger is refused with `limit` rather than written short |
| `RENAME <path> <new-name>` | `path=<pct>` — never overwrites |
| `CONFLICTS <dest-dir> <path>…` | `name=<pct>` for every source whose name already exists in `dest-dir` |
| `COPY <dest-dir> <skip\|replace\|keep-both> <path>…` | progress, then `path=<pct>` per created item. Copying into the source's own directory always keeps both, as "name - Copy.ext" |
| `MOVE <dest-dir> <skip\|replace\|keep-both> <path>…` | progress, then `path=` per item. Rename when possible, copy-then-delete across filesystems. A directory can never be moved into itself |
| `TRASH <path>…` | `trashed=<pct-trash-name>` per item |
| `TRASHLIST` | `t name= original=<pct> deleted= size= kind=`. `name` is a file name and is percent-encoded, so `RESTORE`/`PURGE` decode theirs |
| `RESTORE <trash-name>…` | `path=` per item; `ERR restore-would-overwrite` if the original location is occupied |
| `PURGE <trash-name>…` · `EMPTYTRASH` | `removed=<n>` |
| `DELETE <path>…` | `removed=<n>` — permanent, the shell confirms first |
| `COMPRESS <dest-zip> <path>…` | progress, then `path=` of the new archive (never overwrites) |
| `EXTRACT <archive> <dest-dir>` | progress, then `path=` of the directory written; `.zip .tar .tar.gz .tgz .gz`. `dest-dir` must already exist — the daemon writes *into* it and never creates it, so the caller makes the folder first (the shell does, over this protocol). Entries whose own parent folder is absent from the archive are created on the way |
| `SEARCH <path> <query-pct>` | `e path= name= kind= size= mtime=`; at most 500, then `truncated=1` |
| `RECENT` | the 100 most recently modified files under the root, excluding Trash |
| `VOLUMES` | `v target=<pct> fstype= size= used= avail=` |
| `READTEXT <path>` | `binary=<0\|1> truncated=<0\|1> text=<pct>` — first 64 KiB, for preview |
| `QUIT` | closes |

Error kinds: `bad-request`, `invalid-name`, `not-found`, `exists`, `not-a-directory`,
`is-a-directory`, `symlink-refused`, `permission-denied`, `too-large`,
`unsupported-archive`, `corrupt-archive`, `unsafe-archive-entry`, `limit`,
`restore-would-overwrite`, `io`.

### Partial failure

A request that names several items (`COPY`, `MOVE`, `TRASH`, `RESTORE`, `PURGE`,
`DELETE`) stops at the first item that fails. LFP/1 has no reply that reports both a
count and an error, so the reply is one `ERR`; the items already completed **stay**
completed, and the message names the failing item and how many completed before it.
A client that sees `ERR` after any `PROGRESS` line must re-read the destination rather
than assume nothing happened — the daemon cannot un-copy the items it already wrote,
and a window that shows a different state from the disk is worse than an honest error.

### Extraction limits

Entry names are validated with the same segment rules as requests: absolute names,
`..`, drive letters and backslashes are refused (`unsafe-archive-entry`), and so are
symlinks, hard links and device nodes. Limits: 100 000 entries, 4 GiB total output,
and a compression ratio above 1000:1 on any entry larger than 1 MiB. A ZIP whose
central directory points two entries at overlapping data is refused as a bomb.
Output that exceeds its declared size stops the extraction.

## Verification record

Filled in from executed evidence only. Anything not listed here is **NOT VERIFIED**.
