# App Sandbox: Flatpak updates and system sandbox permissions

**Date:** 2026-10-31 (S90)
**Status:** implemented and fixture-tested; no image build, boot or live Flatpak remote on this host.
**References:** KDE Discover's update list and Flatpak's own permission model were the
interaction references. The implementation keeps ADR-0023's provider boundary: the
shell asks `litric-sysd`, and the daemon speaks to Flatpak's own client.

## The gap this closes

S89 made the App Store real: search, install and remove. It did not answer the two
questions an App Sandbox surface exists to answer — *what updates does Flatpak offer?*
and *what can this application actually reach?* — and it offered no way to change a
system override. The Services page could report that Flatpak was present, but not what
it had confined.

## Design

### Reads are daemon questions, not facts polling

`FACTS` is sampled twice a second and must not spawn a process. The two new reads are
on-demand protocol requests:

```text
STORE UPDATES
STORE SANDBOX <id>
```

`STORE UPDATES` runs
`flatpak remote-ls --updates --system --app --columns=application,name,version,branch,origin`
through the existing `Runner` with a 45-second deadline. Rows are tab-separated,
sorted by application id, branch and origin, de-duplicated and capped at 64. A header,
a partial row or an id this build would not pass to a command is skipped: a partial row
is not an update.

`STORE SANDBOX <id>` makes three bounded reads, in the order Flatpak applies them:

| Read | Plan |
|---|---|
| base | `flatpak info --system --show-permissions <id>` |
| global | `flatpak override --system --show` |
| app | `flatpak override --system --show <id>` |

A missing `flatpak` is `sandbox.state=unavailable`, which is a state, not an error and
not an empty update list. Both reads are allowed to any client that can reach the
socket, exactly like `STORE LIST` and `FACTS`: they change nothing.

### Effective permissions are discrete facts, not a blob

Flatpak prints an INI-like `[Context]`. This build parses four keys — `filesystems`,
`sockets`, `devices`, `shared` — and emits one fact per semicolon-separated entry:

```text
sandbox.perm <source>|<kind>|<value>
```

`source` is `base`, `global`, `app` or `effective`; `kind` is `filesystem`, `socket`,
`device`, `shared`, or `unshare` for a `!entry`. The merge is Flatpak's own: entries
accumulate, and a `!entry` in a later overlay removes the matching inherited entry. The
effective set is therefore a set, sorted, with each entry appearing once — while the
source facts are kept, so the panel can say where a grant came from and where it was
removed. Two identical grants do not produce two rows, and a `!host` is a fact in its
own right rather than disappearing into the merge. The answer is capped at 128 facts.

### Writes are fixed, consented plans

| Action | Plan | Capability and scope | Deadline |
|---|---|---|---|
| update all | `flatpak update --system --noninteractive --assumeyes --app` | `app.update` at `appupdate://all` | 900 s |
| update one | `flatpak update --system --noninteractive --assumeyes --app <id>` | `app.update` at `appupdate://<id>` | 900 s |
| override on | `flatpak override --system --filesystem=host <id>` / `--share=network <id>` | `sandbox.control` at `sandbox://<id>` | 120 s |
| override off | `flatpak override --system --nofilesystem=host <id>` / `--unshare=network <id>` | `sandbox.control` at `sandbox://<id>` | 120 s |
| reset | `flatpak override --system --reset <id>` | `sandbox.control` at `sandbox://<id>` | 120 s |

The id is validated as a reverse-DNS Flatpak id before a plan exists; the key and the
on/off word are a closed enum and a boolean, never free text. `--nofilesystem=` and
`--unshare=` write a removal into *that application's* override, which is why the write
cannot widen the host or another app. `--reset <id>` resets only the named app.

Both capabilities are risk **High**, optional-scoping, and require live consent. The
protocol rejects a missing `CONFIRM`; the broker checks the scope and consent before
the runner is touched. The local session holds one broad policy grant per capability
(`appupdate://`, `sandbox://`) while every request is checked at its narrower scope —
the S88 lesson was that an unscoped capability silently refuses a scoped grant. The
daemon tests prove a scoped request is accepted, and prove refusal before spawn when
the grant is revoked or consent is absent.

### The App Sandbox page

The Settings sidebar gains **App Sandbox** (slug `app-sandbox`), after App Store.
Opening it reads updates and installed applications; the installed list is a chooser,
and the first row's permissions are read through the daemon. The page shows:

* the update state and count, with **Update all**; clicking an update row asks to
  update that app;
* installed system applications, with the selected one's id and version fully shown;
* the selected app's effective facts, labelled by kind and value, plus the entries a
  global or app override removed;
* **Host filesystem** and **Network** controls whose state comes from the effective
  fact set (unknown disables them rather than guessing), and **Reset overrides**;
* one centred confirmation card for every update, override or reset.

Empty, unavailable and error states are distinct. Long lists disclose their remainder
with a count line instead of silently hiding rows; the evidence frame shows every value
whole, with no `ui::elide` call on the page.

## Verification

| Check | Result |
|---|---|
| `cargo test --workspace` | **2,046 passed, 0 failed, 0 ignored** |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| golden frames | 44 fixture frames + 1 real; `45-settings-app-sandbox` new; 10 required screens pass |
| `bash ci/verify.sh` | 39 pass, 0 fail, 3 skipped |

Fixture tests cover update parsing/dedup/ordering, three-call permission reads,
`!entry` removal from global and app overlays, deterministic fact ordering, the fixed
plans and deadlines, malformed ids, protocol parsing/refusal, the two scoped grants,
daemon-authorised update and override calls that reach the runner exactly once, and
refusal before spawn without consent or without the grant. The shell model tests cover
row parsing, protocol lines, toggle direction, confirmation, and unavailable/error
states.

## Limits

* This host has no `flatpak`, so no real remote update, permission read or override was
  exercised. The calls are proven against `RecordingRunner` fixtures and fixed plans.
* Only the system installation (`--system`) is managed. Per-user installations under a
  home directory are not read or changed.
* The image was not built or booted: `mkosi` and `qemu-system-x86_64` are absent. The
  frame is composed by the shipping `views::draw` from fixed facts; it is not a
  screenshot of a booted VM.
* The page draws at most four effective facts before a count line; the daemon returns
  up to 128 and the model keeps the complete answer. The evidence frame has four
  effective facts, so no value is omitted there.
* NetworkManager remains installed but deliberately not enabled. Polkit policy handling
  is unchanged.

## Rollback

Revert the S90 commit. The sandbox provider, protocol reads/writes, `app.update` and
`sandbox.control`, the App Sandbox page, shell model, tests and
`45-settings-app-sandbox` disappear; S89's App Store and all earlier pages remain.
