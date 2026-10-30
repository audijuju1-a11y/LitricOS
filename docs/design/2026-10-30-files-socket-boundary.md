# B2 — the Files socket belongs to the daemon user

Date: 2026-10-30
Batch: B2 (master plan Stage B)
Status: landed, with the residual below.

## The claim that was wrong

`litric-files@.service` runs as `User=litric`, but
`image/mkosi.extra/usr/lib/systemd/system/litric-files.socket` named
`SocketUser=root` and `SocketGroup=root`. The 0600 mode was doing its job — no
uid other than root could connect — but the socket identity did not match the
identity that serves the connection. The beta gate matrix recorded the result
plainly: the daemon runs as `litric`, while both sides were still reached by a
root-owned socket rather than the daemon user's own socket.

That is not the same as a world-connectable socket, and this batch does not
pretend it is. It is a mismatch between the kernel's access check and the
service's identity, and it makes the ordinary client path depend on root's DAC
override instead of on the service user's permission.

## The change

`litric-files.socket` now sets:

```ini
SocketUser=litric
SocketGroup=litric
SocketMode=0600
```

The service already runs as `litric`/`litric`, so the socket and the accepted
connection now belong to the same identity. `root` can still connect through its
DAC override; that is the root-shell trust boundary, not a per-connection check.
Every other local uid gets `EACCES` from `connect` before a request byte exists.

`ci/lint-unit-sockets.py` now rejects a root-owned `litric-files.socket`. Its
`--self-test` plants exactly that unit and proves the checker fails, then plants
the existing world-connectable unit and proves the mode rule still fails too.
This is a static unit check, not a boot claim: no systemd was running here.

## Precedence and what did not change

| Layer | Rule |
|---|---|
| LFP/1 | unchanged; request/reply bytes and error kinds are identical |
| `litric-filesd` | unchanged; it still runs as `litric`, serves one root, and performs no peer-credential read |
| `fsclient` | unchanged; it still connects to `/run/litric/files.sock` |
| systemd | the socket owner/group changed; 0600, `Accept=yes`, `MaxConnections=8` and `RemoveOnStop=yes` are unchanged |
| threat model | extended with an explicit B1 entry and the accepted root-shell residual |

## Failure and degradation rules

* Wrong owner or group → `ci/lint-unit-sockets.py` exits non-zero; it does not
  silently accept the unit.
* `litric` missing on the image → systemd cannot apply `SocketUser`; the socket
  does not start. That is a boot/image failure, not a fallback to root.
* Socket absent → `fsclient` reports the service unreachable; it performs no
  local file operation as a fallback.
* Socket peer is root → allowed by the kernel; no userspace identity check is
  claimed.
* Cross-uid refusal → not executed here. This host has one uid and no root or
  second account, so the evidence is the mode/owner pair plus the kernel rule;
  the gap is recorded rather than converted into a test claim.

## Tests and evidence

* `python3 ci/lint-unit-sockets.py` → 3 units checked.
* `python3 ci/lint-unit-sockets.py --self-test` → rejects a planted 0666 unit and
  a planted root-owned Files socket.
* `src/litric-files/src/server.rs` tests still prove the daemon-created socket is
  0600, belongs to its user, refuses to displace a live socket, and serves a
  same-user connection over a real `UnixStream`.
* `bash ci/verify.sh` `[3i]` runs the checker and its self-test.

## Limits

This is not a per-connection authenticated protocol and not a per-desktop-user
socket. The shell is uid 0; a compromised root shell can use the daemon. The
daemon confines logical paths to `/var/lib/litric-files/root`, but it is not a
sandbox against root. Closing that would require running the shell and daemon
under a shared unprivileged uid, or adding `SO_PEERCRED`, which is unstable in
std on this toolchain and would require a dependency or `unsafe` that the
dependency policy forbids. That remains a real open design decision.
