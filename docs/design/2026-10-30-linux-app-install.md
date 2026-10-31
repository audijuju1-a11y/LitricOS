# Linux application install contract

Date: 2026-10-30
Batch: §79
Status: landed for local, unsigned bundles. This is not an app store, a package
manager or a signed-package verifier.

## The gap

Since §52 the daemon can launch a program and the shell can discover
`.desktop` entries. Nothing on the machine could put a new application into
either path: an app had to be copied there by hand. The launcher could show a
desktop entry, but the system had no install or uninstall operation of its own.

This batch adds the smallest real operation that closes that gap. It installs a
local bundle, writes the executable and a generated desktop entry, and removes
exactly those two paths. It does not download anything and does not require
root.

## On-disk contract

`<application-root>` is the user's XDG data root (`$HOME/.local/share` in
production). `Sysd::with_application_root` replaces it only through Rust
configuration for a test; no request field can name it.

```text
<application-root>/
  applications/<id>.desktop          generated, discovered by the shell
  litric/apps/<id>/app               installed executable, mode 0755
  litric/incoming/<bundle>/app        source, supplied locally
  litric/incoming/<bundle>/app.desktop
```

A bundle directory is exactly two regular files, with no links:

* `app` is a regular executable no larger than 64 MiB.
* `app.desktop` is at most 64 KiB and contains an INI `[Desktop Entry]` with
  `Type=Application`, a non-empty `Name`, and exactly `Exec=app`.

The installer rejects `NoDisplay=true`, `Hidden=true` and `Terminal=true` for
the same reason the shell's discovery refuses them: the image has no terminal
to attach and an app that asks not to be shown cannot be offered.

The generated entry is:

```ini
[Desktop Entry]
Type=Application
Name=<validated name>
Exec="<application-root>/litric/apps/<id>/app"
X-Litric-AppId=<id>
```

`Name` is copied only after it is checked for control characters and the
128-character bound. The absolute `Exec` is built by the daemon from its own
configured root, never from request text.

## Protocol

```text
APP INSTALL <reverse-dns-id> <bundle-name>
APP UNINSTALL <reverse-dns-id>
```

Success is two lines with no path:

```text
OK 2
id=com.example.fixture
state=installed
```

or `state=removed`. Every refusal uses the existing `ERR <kind> <detail>`
framing. The stable kinds are:

| Kind | Meaning |
|---|---|
| `bad-id` | not a valid reverse-DNS application id |
| `bad-bundle` | not exactly one safe directory segment |
| `not-found` | the bundle or installed id does not exist |
| `unsafe-bundle` | a link, non-directory or unsupported entry was found |
| `bad-desktop` | the desktop description is absent or unusable |
| `exists` | that id is already installed |
| `not-installed` | uninstall found no installed id |
| `install-io` | an I/O operation failed |

No detail contains a path or the request's bundle text.

## Validation and atomicity

The bundle name is one segment of `[A-Za-z0-9._-]`, may not start with `.`,
and may not be `.` or `..`. The id is validated by `litric-core`'s `AppId`, so
the target paths are built from a value that cannot traverse. The bundle
directory, both source files and both target paths are checked with
`symlink_metadata`; a symlink is refused rather than followed.

Install copies into a pid-tagged staging directory inside `litric/apps`, sets
the executable mode, writes the desktop file there, renames the directory into
place, and then atomically renames the generated desktop entry. A failure
removes only the staging directory, or the target this attempt had just
committed. It does not replace an existing id.

Uninstall removes the validated application directory and its desktop entry.
If either is a link or the wrong type, it refuses and removes nothing. A canary
file outside those paths survives the unit test.

## Authority

Install and uninstall reuse the local shell's `ProcessSpawn` grant scoped to
`app://`, which is the authority that already lets the shell start a program.
The installer narrows what can be written and where; it does not grant a new
machine-wide capability. A separate `AppInstall` capability would not narrow
the shell's existing ability to start any absolute executable.

The socket remains owner-only and the local client identity is a daemon
constant, not a request field.

## Evidence

* Five focused installer tests cover a valid install, traversal and a linked
  executable, duplicate/unknown-id refusals, a canary-preserving uninstall, and
  a bad desktop description.
* One protocol test covers both new requests and their arity/subcommand
  refusals.
* `journey_app_install_launch_uninstall_through_sysd` binds a real
  `litric-sysd` to a temporary Unix socket, installs a real shell-script
  bundle, discovers the generated entry through the shell's own discovery
  code, launches the installed copy through the daemon's real `LAUNCH` path,
  waits for the marker that copy writes, uninstalls, and asserts that the
  executable, desktop entry and discovery result are all gone.
* File-safety review names the new deletion sites and their reasons.

## Limits

| Claim | Why not |
|---|---|
| that a signed or trusted package was installed | there is no signature or trust store; the bundle is local input |
| that an app store or registry exists | no catalog, dependency solver or manifest integration |
| that root-owned system applications can be installed | the root is the daemon user's XDG data root; there is no privilege escalation |
| that the program is an ELF binary | the e2e fixture is a real executable shell script; a compiled fixture was not required to prove the install/launch chain |
| that F1 is closed | F1 also requires the `litric-manifest` registry, which this batch does not add |
| that the shell has an install UI | none was added; existing discovery notices the generated entry on its watch path |

A production image must run this operation as the session user. Running the
daemon as root would leave root-owned files in the user's data root, which is a
deployment error rather than a property of this contract.

## Failure visibility (S84)

The shell now keeps a bounded `PendingLaunch` after an `OK pid` reply. If the
process is gone before any client window appears, the launcher replaces its
optimistic `Started` status with `<name> exited before opening a window` and
posts a high-priority Applications notification. A process still alive at the
two-second deadline is left alone; this is not a supervisor. The real-socket
journey in `src/litric-shell/src/e2e_app_install.rs` also covers an unsafe
install refusal and a launch whose executable disappeared after discovery.

## User-facing install (S85)

The launcher's Applications surface exposes **Install app**. It opens the
shell's existing desktop dialog with two fields:

* **Bundle folder (litric/incoming)** — the one bundle name the daemon already
  accepts. A path is refused locally rather than silently reduced to a name.
* **Application id (optional)** — a reverse-DNS id. When blank, the shell sends
  `com.litric.<bundle-slug>`; the generated id is not retained into a later
  attempt with a different bundle.

The request remains the existing `APP INSTALL <id> <bundle>` message. It runs on
one worker thread so the compositor never blocks on the socket. The dialog shows
a real loading button and disables both fields and Cancel while the request is
in flight. A reply either:

* closes the dialog, posts `Installed <name>` under Applications, and reloads the
  watched application directories so the new tile is present immediately; or
* keeps the dialog open and shows the refusal beside the fields.

Empty and path-like sources are refused before the request. The daemon's stable
kinds (`not-found`, `unsafe-bundle`, `bad-executable`, `bad-desktop`, `exists`,
`bad-id`, `install-io`, `refused`) map to distinct sentences. A worker that never
answers is retired at `INSTALL_TIMEOUT_MS` (15 s), so a loading button cannot
become permanent. This is a UI deadline, not a daemon timeout.

The entry point does not add a download, signature verifier, package manager,
root operation or uninstall surface.
