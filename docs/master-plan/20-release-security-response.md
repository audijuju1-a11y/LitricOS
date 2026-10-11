# 20 — Release and security response

## Release artefacts

Every candidate carries signed image hashes, SBOM, provenance, release notes,
known issues, HCL, update metadata, test summary, raw performance evidence,
rollback instructions, recovery image and a list of intentionally unverified
claims. `PASS_WITH_LIMITATIONS` lists its limitations prominently.

## Gate policy

| Gate | Blocking condition |
|---|---|
| Functional | boot/install/update/recovery path fails |
| Security | unresolved S0/S1 or unclear privilege boundary |
| Data | corruption/loss scenario lacks proven recovery |
| Performance | hard budget or foreground p99 regression unexplained |
| Hardware | a supported HCL class fails its mandatory matrix |
| Quality | crash loop, unowned flake, or incomplete release evidence |

## Vulnerability response

- Intake: private reporting channel, acknowledgement, severity and disclosure
  coordinator.
- Triage: reproduce, establish affected versions/hardware, assign CVSS-like
  impact plus Litric S0–S5 severity, contain release channels.
- Remediate: root cause, regression test, signed update, rollback guidance,
  release notes and post-incident architecture/test review.
- Practice key revocation, emergency update, compromised build host, malicious
  package and widespread boot failure drills before stable v1.

## Channel policy

`nightly` is for automation; `preview` for developer/HCL testing; `beta` for
opt-in daily use with rollback; `stable` only after gates. Promotion copies exact
signed artifacts and evidence; it never rebuilds an untracked candidate.

## Governance

Security, performance and architecture exceptions require an ADR containing
owner, deadline, measured impact, compensating control, rollback, and removal
condition. Expired exceptions block promotion.

