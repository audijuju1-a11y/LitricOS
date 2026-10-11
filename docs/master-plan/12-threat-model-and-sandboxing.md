# 12 — Threat model and sandboxing

## Assets and adversaries

Protect user files, secrets, microphone/camera, clipboard, network identity,
system image, signing keys, recovery keys and integrity metadata from malicious
apps, hostile files, USB devices, network attackers, compromised repositories,
physical attackers and malicious AI prompts/models. `docs/SEC-00-threatmodel.md`
is the baseline STRIDE record and must be updated on every new boundary.

## Defence composition

```text
signed image + verified boot
  → service privilege reduction + cgroup limits
  → Flatpak/bubblewrap namespaces + seccomp
  → Landlock object restrictions + portal mediation
  → Litric capability broker + audit/revocation
```

No individual layer is assumed sufficient. Landlock only adds restrictions and
has ABI/version limitations; portal UX is not enforcement without sandbox and
broker checks; resource limits are security controls as well as performance
controls.

## Required policies

- Deny direct access to other app data/processes/devices by default.
- `no_new_privs`, minimal Linux capabilities, read-only code/runtime mounts,
  controlled writable mounts, PID/network/user namespace policy and seccomp
  profile per app class.
- Device authorization policy for removable USB, cameras and network adapters;
  IOMMU is required before untrusted device passthrough.
- Sensitive events create concise local audit records with app ID, resource,
  decision, scope and timestamp; logs never contain secret contents.
- AI workers have zero ambient file/screen/mic/network capability and acquire
  each input through a user-visible grant.

## Red-team cases

Attempt portal token replay, inherited FD abuse, symlink/rename TOCTOU, D-Bus
identity spoofing, sandbox escape, CPU/memory/I/O exhaustion, malicious USB,
guest shared-folder leakage, downgrade/update takeover, and model prompt
injection. Every confirmed issue creates a severity-tagged bug record, root
cause, regression test, and threat-model update.

