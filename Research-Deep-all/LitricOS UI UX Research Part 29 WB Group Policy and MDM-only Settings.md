# LitricOS UI/UX Research — Part 29
## WB.12.08: Group Policy and MDM-only settings

**Part:** 29
**Chapter:** B (Windows Settings catalogue), row `WB.12.08`
**Row:** Group Policy / MDM-only settings — the settings that exist in Windows but not on a Settings page
**Date:** 2026-10-28
**Evidence base:** three Microsoft Learn pages opened in this pass (`S3888`–`S3890`).

---

## PHASE A — state check before writing

Part 28's published figures were re-derived from the files and all matched (597 checklist rows / 0 malformed, `LB` 14/14, `LR` 14/14, PENDING 4, register 4,202 rows ending at S3887, inspected 1,297, domains 382, duplicates 48 / surplus 53, 14 + 14 data rows, S3878–S3887 each once, 0 CRLF). One discrepancy was found, in Part 1 rather than in Part 28: **516 Appendix A rows still printed PENDING** (including all 112 Chapter L rows) while the index above them said those chapters were complete, and index row 44 still carried Part 24's **4,170 rows / 372 domains**. Both are bookkeeping drift and are corrected by `tools/record_chapter_l_batch5.py`; the new figures are in Part 30 §7 and Part 24 `J.14`. No research text was rewritten.

---

## 0. Why this row is separate

Every other Windows Settings row in this corpus describes a page a person can
open. `WB.12.08` describes the opposite: policy that is configured somewhere
else — a domain Group Policy Object, or a mobile-device-management server — and
then appears on the device as state the user cannot change from Settings. The
row was left PENDING because it is not a page inventory; it is a boundary
between three control planes:

1. the **Settings app** (a user's own choices);
2. **Group Policy** (a domain administrator's directives, delivered by ADMX
   templates and the Group Policy service);
3. **MDM** (a cloud administrator's directives, delivered through the Policy
   configuration service provider).

The research question is therefore not "what does the page look like" but "where
does a setting live when it is not on a page at all, and what does the user see".

---

## 1. Sources

| ID | Page | Grade | What it establishes |
|---|---|---|---|
| S3888 | Policy CSP (Microsoft Learn) | T1 | The MDM control plane: `./Vendor/MSFT/Policy/Config/{AreaName}/{PolicyName}`, user and device scope, `Result` paths for conflict resolution, `ADMXInstall` for ingesting templates after the OS ships |
| S3889 | Understanding ADMX policies | T1 | ADMX as the metadata of a Group Policy; the same ADMX file drives both GP and MDM; MDM does **not** need the Group Policy service; inbox ADMX processed at build time, ingested ADMX at runtime |
| S3890 | Enroll a Windows device automatically using Group Policy | T1 | Precedence: from Windows 10 1709, Group Policy wins over MDM for the same policy; from 1803 a setting can flip precedence to MDM |

No page was opened that enumerates "settings present only in Group Policy". That
absence is recorded as the row's limitation and is why the row is PARTIAL rather
than COMPLETE.

---

## 2. The row

| ID | Item | What exists | Evidence | LitricOS today | Status |
|---|---|---|---|---|---|
| `WB.12.08` | Group Policy / MDM-only settings | Settings can be owned by three planes: the user (Settings app), a domain administrator (Group Policy via ADMX, applied by the Group Policy service), or an MDM server (Policy CSP, optionally using the same ADMX definitions). MDM needs no Group Policy service; ADMX templates can be ingested after the OS ships. When both GP and MDM set one policy, **GP wins** on 1709+, unless a 1803+ setting changes the precedence to MDM. The user-visible result is a control that is present, shows the enforced value, and cannot be changed | S3888, S3889, S3890 | No policy plane at all: the shell's own settings are user-owned and there is nothing above them. Capabilities are the only authority (`litric-caps`), and they gate actions rather than values | **PARTIAL** |

### 2.1 What the sources actually say

* **The MDM path is a tree of URIs, not a page.** `./User/Vendor/MSFT/Policy/Config/AreaName/PolicyName`
  configures, and `.../Result/...` returns "the resulting value" after conflict
  resolution — i.e. the platform keeps a resolved value separate from what any
  one source asked for (S3888).
* **ADMX is shared metadata.** An ADMX file "specifies the registry keys (and
  their values) that are associated with a Group Policy", and in an MDM-managed
  client "ADMX files are applied to define policies independent of Group
  Policies… a Group Policy infrastructure, including the Group Policy Service
  (gpsvc.exe), isn't required" (S3889).
* **Post-ship ingestion is a real path.** `ConfigOperations/ADMXInstall` adds
  policies for Win32 and Desktop Bridge apps; installing the file adds a unique
  id, and deleting it removes the file, its metadata and every policy set from
  it (S3888).
* **Precedence is versioned and documented.** "Starting in Windows 10, version
  1709, when the same policy is configured in Group Policy and MDM, Group Policy
  policy takes precedence over MDM"; from 1803 a setting can change that (S3890).
* **A policy can be invisible in the tool that set it.** The same page notes the
  GPEdit console "doesn't reflect the status of policies set by your organization
  on your device" (S3890) — the editor is for local policy, not a view of what is
  actually enforced.

### 2.2 Findings

**L29.01 — Windows has a third settings plane, and the user interface has no
vocabulary for it.** A Settings row can be user-owned, administrator-owned or
server-owned, and the same control looks the same in all three cases. The corpus
has spent Chapter B cataloguing pages; this row is the reason a page inventory
cannot be the whole model.

**L29.02 — The resolved value is stored separately from the requested value.**
`Policy/Result` exists because two sources can disagree (S3888). That is an
architectural idea LitricOS does not have anywhere: its preference file stores
what the user chose, and nothing stores "what is actually in force after the
system, the image and a policy had their say".

**L29.03 — ADMX proves a design system can ship policy metadata as data.** The
same file drives a Group Policy editor, an MDM console and the device. LitricOS
hard-codes its settings in a Rust enum and a `key=value` file; there is no
machine-readable description of a setting that a second tool could read.

**L29.04 — An installed policy can outlive the decision that created it.**
Deleting an ingested ADMX file deletes every policy set from it (S3888). This is
the failure mode a settings store without provenance produces: no record of who
set a value, so no way to tell an inherited value from a chosen one.

**L29.05 — The tool an administrator uses can disagree with the device.** GPEdit
does not show organization policies (S3890). Any interface that edits state it
does not own needs to say so; LitricOS' Settings edits only its own store, which
makes this a gap it does not yet have — and a rule for when it grows one.

---

## 3. What to change in LitricOS because of this row

| # | Change | Why | Size |
|---|---|---|---|
| 1 | Store the *source* of each preference value (user, image default, policy) next to the value | `L29.01`, `L29.02` | M |
| 2 | Add a resolved-value view for any setting that can be overridden | `L29.02` | S |
| 3 | Describe settings as data (a small schema) so a second tool can read them | `L29.03` | M |
| 4 | Show provenance in Settings when a value is not the user's | `L29.04`, `L29.05` | S |

---

## 4. Limitations

* **No enumeration of GP-only settings was read.** The three pages describe the
  mechanism, not a list; no page was opened that counts how many settings exist
  only as policy. `WB.12.08` is PARTIAL for that reason.
* **The precedence rule is quoted, not observed.** No domain, no Intune tenant
  and no managed device were available; nothing here is a measurement of a
  managed Windows machine.
* **Not run on real hardware.** The Windows column is documentation; the
  LitricOS column is code in this repository.
* **Chrome Enterprise and dconf lockdown were not opened.** The row is scoped to
  Windows, and adding two more ecosystems would have made a mechanism row into a
  chapter.

---

## 5. Sources read in this part

| ID | URL | What it gave |
|---|---|---|
| S3888 | https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-configuration-service-provider | Policy CSP paths, user/device scope, Result paths, ADMXInstall |
| S3889 | https://learn.microsoft.com/en-us/windows/client-management/understanding-admx-backed-policies | ADMX as shared metadata; MDM without gpsvc; build-time vs ingested templates |
| S3890 | https://learn.microsoft.com/en-us/windows/client-management/enroll-a-windows-10-device-automatically-using-group-policy | GP-over-MDM precedence on 1709+, the 1803 switch, GPEdit not showing organization policy |

No page failed to open in this part.
