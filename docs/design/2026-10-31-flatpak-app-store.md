# Flatpak App Store: search, install and remove

**Date:** 2026-10-31 (S89)
**Status:** implemented and fixture-tested; no image build or boot on this host.
**References:** KDE Discover's catalogue/search/install flow was the interaction
reference. The implementation keeps ADR-0023's provider boundary: the shell asks
`litric-sysd`, and the daemon speaks to Flatpak's own client.

## The gap this closes

S88 reported that Flatpak was present and offered a catalog refresh, but the
Settings App Store could not answer the questions an app store exists to answer:
what is available, what is installed, and how do I install or remove one. The
only install path was the local `app://` bundle installer, not Flatpak.

## Design

### Reads are daemon questions, not facts polling

`FACTS` is sampled twice a second and must not spawn a process. Catalogue reads
are therefore separate protocol requests:

```text
STORE LIST
STORE SEARCH <query>
```

`litric-sysd` runs `flatpak list --system --app --columns=application,name,version,origin,size`
or `flatpak search --system --columns=application,name,version,remotes <query>`
through the existing `Runner` with a 45-second deadline. Results are parsed from
Flatpak's tab-separated machine columns, sorted by application id and name, and
capped at 24. A missing `flatpak` is `store.state=unavailable`, which is a state,
not an error and not an empty catalogue.

### Writes are fixed, consented plans

`StoreAction` has exactly two variants:

| Action | Plan | Scope | Deadline |
|---|---|---|---|
| `store.install` | `flatpak install --system --noninteractive --assumeyes <remote> <id>` | `appmanage://<id>/<remote>` | 900 s |
| `store.uninstall` | `flatpak uninstall --system --noninteractive --assumeyes <id>` | `appmanage://<id>` | 120 s |

The app id and remote are validated before the plan exists. The id must be a
reverse-DNS Flatpak id; the remote must be a single token that cannot be read as
an option. The protocol rejects a missing `CONFIRM`; the broker checks the new
scoped `app.manage` capability and live consent before the runner is touched.
The local session holds one broad `appmanage://` policy grant, while every
request is checked at its narrower app/remote scope — the S88 lesson was that an
unscoped capability silently refuses a scoped grant.

### The App Store page

The Settings sidebar gains **App Store** (slug `app-store`). The page has a
search field, available results, installed system applications, and one
confirmed install or remove action. Selecting a row and pressing Install/Remove
opens a centred confirmation card; Cancel never writes. Empty, unavailable and
error states are distinct. The evidence frame `44-settings-app-store` shows the
normal page with search results and installed rows; the confirmation card is
covered by shell tests rather than covering the rows the frame exists to review.

## Verification

| Check | Result |
|---|---|
| `cargo test --workspace` | **2,018 passed, 0 failed** |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| golden frames | 43 recorded; `44-settings-app-store` new; 10 required screens pass |
| `bash ci/verify.sh` | 39 pass, 0 fail, 3 skipped |

Fixture tests cover Flatpak tabular output, duplicate removal, malformed rows,
missing-tool state, argument validation, fixed install/uninstall plans, protocol
parsing/refusal, provider reads, the scoped grant and a daemon-authorised
install/uninstall that reaches the runner exactly once.

## Limits

* This host has no `flatpak` and no real remote, so no remote package was
  downloaded, installed or removed end to end. The calls are proven against
  `RecordingRunner` fixtures and the fixed plans.
* Only the system installation (`--system`) is managed. Per-user installations
  under a home directory are not searched or listed.
* The image was not built or booted: `mkosi` and `qemu-system-x86_64` are absent.
  The frame is composed by the shipping `views::draw` from fixed facts; it is
  not a screenshot of a booted VM.
* NetworkManager remains installed but deliberately not enabled. Polkit policy
  handling is unchanged.

## Rollback

Revert the S89 commit. The `store` module, `STORE` reads/writes, `app.manage`,
the App Store page, shell effects, tests and `44-settings-app-store` disappear;
S88's service facts and reloads remain.
